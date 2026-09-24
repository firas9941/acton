#[cfg(test)]
mod tests;

mod transaction;

use std::collections::HashSet;
use std::convert::Infallible;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use futures::stream;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::{OwnedSemaphorePermit, Semaphore, mpsc};
use ton_indexer_core::Batch;
use tracing::{debug, warn};
use tycho_types::models::{StdAddr, StdAddrFormat};

const MAX_SUBSCRIBERS: usize = 64;
const MAX_ADDRESSES: usize = 100;
const QUEUE_EVENTS: usize = 32;
const QUEUE_BYTES: usize = 2 * 1024 * 1024;
const MAX_EVENT_BYTES: usize = 1024 * 1024;
const OPEN: u8 = 0;
const LAGGED: u8 = 1;
const FAILED: u8 = 2;
const CLOSED: u8 = 3;

/// Live subscriptions to batches committed by the P2P state synchronizer.
/// No cells or transaction history survive publication. Each connection owns a
/// bounded queue; overflow terminates that subscription with an explicit gap.
#[derive(Clone)]
pub(crate) struct Transactions {
    subscribers: Arc<Mutex<Vec<Subscriber>>>,
    connections: Arc<Semaphore>,
}

struct Subscriber {
    addresses: HashSet<String>,
    sender: mpsc::Sender<QueuedEvent>,
    budget: Arc<Semaphore>,
    terminal: Arc<AtomicU8>,
}

struct QueuedEvent {
    json: Arc<str>,
    _bytes: OwnedSemaphorePermit,
}

struct Connection {
    receiver: mpsc::Receiver<QueuedEvent>,
    terminal: Arc<AtomicU8>,
    _slot: OwnedSemaphorePermit,
    first: bool,
    done: bool,
}

impl Default for Transactions {
    fn default() -> Self {
        Self {
            subscribers: Arc::default(),
            connections: Arc::new(Semaphore::new(MAX_SUBSCRIBERS)),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Subscription {
    addresses: Vec<String>,
    #[serde(default)]
    types: Option<Vec<String>>,
    #[serde(default)]
    min_finality: Option<String>,
}

impl Transactions {
    /// Serves the minimal transaction-only contract. The event envelope is local;
    /// it does not claim TON Center's trace-grouped streaming semantics.
    pub(crate) fn router(self) -> Router {
        Router::new()
            .route("/api/streaming/v2/sse", post(subscribe))
            .with_state(self)
    }

    /// Publishes an already committed batch. The caller must never call this
    /// before applying all masterchain and shard state updates successfully.
    /// Encoding errors require [`Self::fail`] so clients cannot miss events silently.
    pub(crate) fn publish(&self, batch: &Batch) -> Result<()> {
        let started = Instant::now();
        let mut sent = 0;
        {
            let mut subscribers = self.subscribers.lock().expect("subscription lock poisoned");
            subscribers.retain(|subscriber| !subscriber.sender.is_closed());
            if subscribers.is_empty() {
                return Ok(());
            }
        }

        for block in batch.blocks() {
            for lazy in block.transactions() {
                let tx = lazy.load().with_context(|| {
                    format!(
                        "cannot decode transaction {} in block {}",
                        lazy.inner().repr_hash(),
                        block.id()
                    )
                })?;
                let account = format!("{}:{}", block.id().workchain, tx.account);
                if !self
                    .subscribers
                    .lock()
                    .expect("subscription lock poisoned")
                    .iter()
                    .any(|subscriber| subscriber.addresses.contains(&account))
                {
                    continue;
                }

                let tx = transaction::convert(block.id(), batch.checkpoint().seqno, lazy, &tx)
                    .with_context(|| {
                        format!(
                            "cannot map transaction {} in block {}",
                            lazy.inner().repr_hash(),
                            block.id()
                        )
                    })?;
                let encoded = serde_json::to_string(&json!({
                    "type": "transaction",
                    "finality": "finalized",
                    "transaction": tx,
                }))?;
                ensure!(
                    encoded.len() <= MAX_EVENT_BYTES,
                    "transaction event exceeds 1 MiB"
                );
                let encoded: Arc<str> = encoded.into();
                let mut subscribers = self.subscribers.lock().expect("subscription lock poisoned");
                subscribers.retain(|subscriber| {
                    if subscriber.sender.is_closed() {
                        return false;
                    }
                    if !subscriber.addresses.contains(&account) {
                        return true;
                    }

                    let Ok(bytes) = subscriber
                        .budget
                        .clone()
                        .try_acquire_many_owned(encoded.len() as u32)
                    else {
                        subscriber.terminal.store(LAGGED, Ordering::Release);
                        return false;
                    };
                    let event = QueuedEvent {
                        json: Arc::clone(&encoded),
                        _bytes: bytes,
                    };
                    if subscriber.sender.try_send(event).is_err() {
                        subscriber.terminal.store(LAGGED, Ordering::Release);
                        return false;
                    }
                    true
                });
                drop(subscribers);
                sent += 1;
            }
        }

        debug!(
            operation = "transaction_stream",
            target = %batch.checkpoint(),
            transactions = sent,
            duration_ms = started.elapsed().as_millis(),
            outcome = "published",
            "published committed transactions",
        );
        Ok(())
    }

    /// Ends current subscriptions after an encoding failure. Reconnection starts
    /// a new live subscription; it cannot recover the failed batch.
    pub(crate) fn fail(&self) {
        self.finish(FAILED);
    }

    /// Wakes idle streams and ends them during HTTP shutdown.
    pub(crate) fn close(&self) {
        self.connections.close();
        self.finish(CLOSED);
    }

    fn finish(&self, terminal: u8) {
        let mut subscribers = self.subscribers.lock().expect("subscription lock poisoned");
        for subscriber in subscribers.drain(..) {
            subscriber.terminal.store(terminal, Ordering::Release);
        }
    }
}

async fn subscribe(
    State(transactions): State<Transactions>,
    headers: HeaderMap,
    body: Result<Json<Subscription>, JsonRejection>,
) -> Response {
    let Ok(Json(body)) = body else {
        return reject(StatusCode::BAD_REQUEST, "invalid_subscription");
    };
    if headers.contains_key("last-event-id") {
        return reject(StatusCode::BAD_REQUEST, "replay_not_supported");
    }
    if body
        .types
        .as_ref()
        .is_some_and(|types| types != &["transactions"])
        || body
            .min_finality
            .as_deref()
            .is_some_and(|value| value != "finalized")
    {
        return reject(
            StatusCode::BAD_REQUEST,
            "only_finalized_transactions_supported",
        );
    }
    if body.addresses.is_empty() || body.addresses.len() > MAX_ADDRESSES {
        return reject(StatusCode::BAD_REQUEST, "expected_1_to_100_addresses");
    }
    let addresses = body
        .addresses
        .iter()
        .map(|address| {
            StdAddr::from_str_ext(address, StdAddrFormat::any())
                .map(|(address, _)| address.to_string())
        })
        .collect::<Result<HashSet<_>, _>>();
    let Ok(addresses) = addresses else {
        return reject(StatusCode::BAD_REQUEST, "invalid_address");
    };

    let (sender, receiver) = mpsc::channel(QUEUE_EVENTS);
    let terminal = Arc::new(AtomicU8::new(OPEN));
    let Ok(slot) = transactions.connections.clone().try_acquire_owned() else {
        return reject(StatusCode::SERVICE_UNAVAILABLE, "too_many_subscribers");
    };
    {
        let mut subscribers = transactions
            .subscribers
            .lock()
            .expect("subscription lock poisoned");
        subscribers.retain(|subscriber| !subscriber.sender.is_closed());
        if transactions.connections.is_closed() {
            return reject(StatusCode::SERVICE_UNAVAILABLE, "shutting_down");
        }
        subscribers.push(Subscriber {
            addresses,
            sender,
            budget: Arc::new(Semaphore::new(QUEUE_BYTES)),
            terminal: Arc::clone(&terminal),
        });
    }

    let events = stream::unfold(
        Connection {
            receiver,
            terminal,
            _slot: slot,
            first: true,
            done: false,
        },
        |mut connection| async move {
            if connection.done {
                return None;
            }
            let event = if connection.first {
                connection.first = false;
                Event::default().data(r#"{"status":"subscribed"}"#)
            } else {
                let next = connection.receiver.recv().await;
                let reason = connection.terminal.load(Ordering::Acquire);
                if reason == CLOSED {
                    return None;
                }
                if reason == LAGGED || reason == FAILED {
                    let error = if reason == LAGGED {
                        "slow_consumer"
                    } else {
                        "stream_failed"
                    };
                    warn!(
                        operation = "transaction_subscription",
                        target = "sse",
                        outcome = error,
                        "ending subscription with a gap"
                    );
                    let event =
                        Event::default().data(json!({"type": "error", "error": error}).to_string());
                    connection.done = true;
                    return Some((Ok::<_, Infallible>(event), connection));
                }
                Event::default().data(&*next?.json)
            };
            Some((Ok::<_, Infallible>(event), connection))
        },
    );

    (
        [("cache-control", "no-cache"), ("x-accel-buffering", "no")],
        Sse::new(events).keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keepalive"),
        ),
    )
        .into_response()
}

fn reject(status: StatusCode, error: &'static str) -> Response {
    (status, Json(json!({"error": error}))).into_response()
}
