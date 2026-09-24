use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio::sync::watch;
use ton_indexer_core::{BlockData, BlockSource};
use ton_indexer_p2p::P2pBlockSource;
use ton_node_db::{StateSnapshot, StateStore};
use tracing::{info, warn};
use tycho_types::models::BlockId;

use crate::streaming::Transactions;

/// Keeps the store at complete masterchain/shard frontiers. Network failures are
/// retried; invalid state updates and storage failures stop the service.
pub(crate) async fn run(
    mut store: StateStore,
    mut source: P2pBlockSource,
    checkpoints: watch::Sender<StateSnapshot>,
    transactions: Transactions,
) -> Result<()> {
    loop {
        let after = store.head();
        let started = Instant::now();
        let batch = match source.next_batch(Some(&after.into())).await {
            Ok(Some(batch)) => batch,
            Ok(None) => {
                // A missing head is transient. The pool paces each peer; avoid
                // adding another long polling delay after an unavailable reply.
                tokio::time::sleep(Duration::from_millis(10)).await;
                continue;
            }
            Err(error) => {
                warn!(
                    operation = "state_download",
                    target = %after,
                    duration_ms = started.elapsed().as_millis(),
                    outcome = "retry",
                    error = %format!("{error:#}"),
                    "could not download the next complete batch",
                );
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };
        let downloaded = Instant::now();
        let master_id: BlockId = batch.masterchain().id().try_into()?;
        let shard_ids = batch
            .shards()
            .iter()
            .map(|block| block.id().try_into())
            .collect::<Result<Vec<BlockId>, _>>()?;
        let shard_blocks = shard_ids.len();
        let checkpoints = checkpoints.clone();
        let publisher = transactions.clone();

        // Cell traversal and synchronous RocksDB writes must not occupy an async worker.
        let (updated, apply_time, publish_time) = tokio::task::spawn_blocking(move || {
            let applying = Instant::now();
            // BlockSource has already verified file hashes and decoded these roots.
            store.apply_roots(
                (master_id, batch.masterchain().root()),
                shard_ids
                    .into_iter()
                    .zip(batch.shards().iter().map(BlockData::root)),
            )?;
            checkpoints.send_replace(store.snapshot());
            let applied = Instant::now();

            // Finalized events become visible only after the entire batch commits.
            // A streaming failure closes subscriptions without stopping state sync.
            if let Err(error) = publisher.publish(&batch) {
                publisher.fail();
                warn!(
                    operation = "transaction_stream",
                    target = %master_id,
                    duration_ms = started.elapsed().as_millis(),
                    outcome = "failed",
                    error = %format!("{error:#}"),
                    "closed subscriptions after a transaction encoding failure",
                );
            }
            anyhow::Ok((store, applied.duration_since(applying), applied.elapsed()))
        })
        .await
        .context("state writer panicked")??;
        store = updated;

        info!(
            operation = "state_sync",
            target = %master_id,
            shard_blocks,
            download_ms = downloaded.duration_since(started).as_millis(),
            apply_us = apply_time.as_micros(),
            publish_us = publish_time.as_micros(),
            duration_ms = started.elapsed().as_millis(),
            outcome = "committed",
            "downloaded and persisted the complete block batch",
        );
    }
}
