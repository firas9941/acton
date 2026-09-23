use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio::sync::RwLock;
use ton_indexer_core::BlockSource;
use ton_indexer_p2p::P2pBlockSource;
use ton_node_db::StateStore;
use tracing::{info, warn};
use tycho_types::models::BlockId;

/// Keeps the store at complete masterchain/shard frontiers. Network failures are
/// retried; invalid state updates and storage failures stop the service.
pub(crate) async fn run(store: Arc<RwLock<StateStore>>, mut source: P2pBlockSource) -> Result<()> {
    loop {
        let after = store.read().await.head();
        let started = Instant::now();
        let batch = match source.next_batch(Some(&after.into())).await {
            Ok(Some(batch)) => batch,
            Ok(None) => {
                tokio::time::sleep(Duration::from_millis(100)).await;
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
        let master_id: BlockId = batch.masterchain().id().try_into()?;
        let (_, master_boc) = source
            .client_mut()
            .masterchain_block(master_id.seqno)
            .await?
            .context("downloaded masterchain block is absent from cache")?;
        let shard_ids = batch
            .shards()
            .iter()
            .map(|block| block.id().try_into())
            .collect::<Result<Vec<BlockId>, _>>()?;
        let shard_bocs = source
            .client_mut()
            .download_shards(&shard_ids)
            .await?
            .into_iter()
            .map(|boc| boc.context("downloaded shard block is absent from cache"))
            .collect::<Result<Vec<_>>>()?;
        let shard_blocks = shard_ids.len();
        let writer = Arc::clone(&store);

        // Cell traversal and synchronous RocksDB writes must not occupy an async worker.
        tokio::task::spawn_blocking(move || {
            writer.blocking_write().apply_batch(
                (master_id, &master_boc),
                shard_ids
                    .into_iter()
                    .zip(shard_bocs.iter().map(Vec::as_slice)),
            )
        })
        .await
        .context("state writer panicked")??;

        info!(
            operation = "state_sync",
            target = %master_id,
            shard_blocks,
            duration_ms = started.elapsed().as_millis(),
            outcome = "committed",
            "downloaded and persisted the complete block batch",
        );
    }
}
