//! Synchronize durable TON states and expose the applied checkpoint over HTTP.

mod api;
mod streaming;
mod submit;
mod sync;

use std::net::{SocketAddr, SocketAddrV4};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;
use tokio::sync::watch;
use ton_indexer_p2p::P2pBlockSource;
use ton_node_db::StateStore;
use ton_p2p::{Client, ClientOptions, NetworkConfig, NetworkOptions, load_identity};
use tracing::info;

#[derive(Parser)]
#[command(about = "Synchronize a TON snapshot and serve its applied state over HTTP")]
struct Args {
    /// Immutable database from a stopped validator or consistent backup
    database: PathBuf,

    /// Global config of the network that produced the snapshot
    global_config: PathBuf,

    /// Writable state updates and downloaded blocks, outside the snapshot
    data_dir: PathBuf,

    /// HTTP listener
    #[arg(long, default_value = "127.0.0.1:8080")]
    http: SocketAddr,

    /// Reachable UDP address advertised to P2P peers
    #[arg(long, default_value = "127.0.0.1:19005")]
    address: SocketAddrV4,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();
    let started = Instant::now();
    let mut config = NetworkConfig::load(&args.global_config)?;
    let store = StateStore::open(&args.database, &args.data_dir.join("states"), 1_000_000)?;
    let head = store.head();
    config.set_initial_block(head)?;
    let downloads = args.data_dir.join("blocks");
    let client = Client::open(
        &config,
        ClientOptions {
            network: NetworkOptions {
                address: args.address,
                secret_key: load_identity(&downloads)?,
                timeout: Duration::from_secs(5),
            },
            data_dir: downloads,
            peers_file: None,
            parallelism: 16,
        },
    )?;
    client.validate_checkpoint(&head)?;
    let sender = client.message_sender();
    let source = P2pBlockSource::new(client)?;
    let (checkpoints, state) = watch::channel(store.snapshot());
    let transactions = streaming::Transactions::default();
    let router = api::router(state.clone(), config.zero_state())
        .merge(transactions.clone().router())
        .merge(submit::router(sender));
    let listener = tokio::net::TcpListener::bind(args.http)
        .await
        .with_context(|| format!("cannot bind HTTP listener {}", args.http))?;

    info!(
        operation = "state_service_start",
        target = %head,
        http = %args.http,
        address = %args.address,
        data_dir = %args.data_dir.display(),
        duration_ms = started.elapsed().as_millis(),
        outcome = "ready",
        "serving the applied state and synchronizing through P2P",
    );

    let shutdown_transactions = transactions.clone();
    let synchronization = sync::run(store, source, checkpoints, transactions.clone());
    drop(transactions);
    let result = tokio::select! {
        result = synchronization => result,
        result = axum::serve(listener, router).with_graceful_shutdown(async move {
            shutdown().await;
            shutdown_transactions.close();
        }) => {
            result.context("HTTP server failed")
        }
    };

    info!(
        operation = "state_service_stop",
        target = %state.borrow().head(),
        duration_ms = started.elapsed().as_millis(),
        outcome = if result.is_ok() { "stopped" } else { "failed" },
        "service stopped; the next start resumes the applied checkpoint",
    );

    result
}

async fn shutdown() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("cannot install SIGTERM handler");

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = terminate.recv() => {},
        }
    }

    #[cfg(not(unix))]
    tokio::signal::ctrl_c()
        .await
        .expect("cannot install Ctrl-C handler");
}
