//! Adapts cached P2P blocks to the canonical batch traversal in ton-indexer-core.

use std::collections::{HashMap, hash_map::Entry};

use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::time::Instant;
use ton_indexer_core::{
    Batch, BlockData, BlockGraphClient, BlockId, BlockIdShort, BlockSource, CanonicalBlockSource,
    RawBlock, SourceError,
};
use ton_p2p::Client;
use tracing::{debug, info};

/// Supplies complete masterchain and shard batches over P2P.
///
/// New consumers start after the configured anchor. On resume, consumers pass
/// their committed checkpoint to `next_batch`. The source verifies hashes and
/// predecessor links, but does not verify validator signatures or execute blocks.
pub struct P2pBlockSource {
    inner: CanonicalBlockSource<P2pAdapter>,
}

impl P2pBlockSource {
    /// Wraps a P2P client. The first batch follows its configured anchor;
    /// callers can resume later by passing their own checkpoint to `next_batch`.
    pub fn new(client: Client) -> Result<Self> {
        let start_seqno = client
            .anchor()
            .seqno
            .checked_add(1)
            .context("masterchain sequence overflow")?;

        Ok(Self {
            inner: CanonicalBlockSource::new(
                P2pAdapter {
                    client,
                    decoded: HashMap::new(),
                },
                start_seqno,
            ),
        })
    }

    /// Borrows the download client to inspect its anchor, head, or cached chain.
    #[must_use]
    pub const fn client(&self) -> &Client {
        &self.inner.client().client
    }

    /// Borrows the download client for masterchain-only downloads or cache access.
    /// Its download checkpoint remains independent of the consumer's batch checkpoint.
    pub const fn client_mut(&mut self) -> &mut Client {
        &mut self.inner.client_mut().client
    }
}

#[async_trait]
impl BlockSource for P2pBlockSource {
    async fn next_batch(
        &mut self,
        after: Option<&BlockId>,
    ) -> ton_indexer_core::Result<Option<Batch>> {
        if let Some(checkpoint) = after {
            let checkpoint = (*checkpoint)
                .try_into()
                .map_err(ton_indexer_core::Error::source)?;
            self.client()
                .validate_checkpoint(&checkpoint)
                .map_err(ton_indexer_core::Error::source)?;
        }

        let started = Instant::now();
        let result = self.inner.next_batch(after).await;

        // CanonicalBlockSource keeps the previous shard frontier. Decoded cells
        // are needed only for this attempt and must not accumulate on retries.
        self.inner.client_mut().decoded.clear();

        match result {
            Ok(Some(batch)) => {
                info!(
                    operation = "p2p_batch",
                    target = %batch.masterchain().id(),
                    shard_blocks = batch.shards().len(),
                    duration_ms = started.elapsed().as_millis(),
                    outcome = "assembled",
                    "assembled complete masterchain and shard batch",
                );
                Ok(Some(batch))
            }
            Ok(None) => Ok(None),
            Err(SourceError::Unavailable(id)) => {
                debug!(
                    operation = "p2p_batch",
                    target = %id,
                    duration_ms = started.elapsed().as_millis(),
                    outcome = "waiting",
                    "waiting for missing block; indexer checkpoint is unchanged",
                );
                Ok(None)
            }
            Err(error) => Err(ton_indexer_core::Error::source(error)),
        }
    }
}

struct P2pAdapter {
    client: Client,
    decoded: HashMap<BlockId, BlockData>,
}

impl P2pAdapter {
    fn decode_cached(&mut self, raw: &RawBlock) -> Result<&BlockData, SourceError> {
        Ok(match self.decoded.entry(raw.id()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(BlockData::decode(raw.id(), raw.boc())?),
        })
    }
}

#[async_trait]
impl BlockGraphClient for P2pAdapter {
    async fn latest_masterchain_block(&mut self) -> Result<BlockId, SourceError> {
        self.client
            .next_masterchain()
            .await
            .map_err(transport_error)?;
        self.client
            .head()
            .map(BlockId::from)
            .ok_or_else(|| SourceError::Unavailable(self.client.anchor().into()))
    }

    async fn load_block(&mut self, id: BlockIdShort) -> Result<RawBlock, SourceError> {
        if id.workchain != -1 || id.shard != BlockId::FULL_SHARD {
            return Err(SourceError::InvalidBlockId(
                "P2P shard requests require full IDs".into(),
            ));
        }

        // Zerostate references contain no block BOC. They mark the baseline for block 1.
        if id.seqno == 0 && self.client.anchor().seqno == 0 {
            return Ok(RawBlock::new(self.client.anchor().into(), Vec::new()));
        }

        let (id, boc) = self
            .client
            .masterchain_block(id.seqno)
            .await
            .map_err(transport_error)?
            .ok_or_else(|| {
                SourceError::Transport(format!("masterchain block {} is not cached", id.seqno))
            })?;
        Ok(RawBlock::new(id.into(), boc))
    }

    async fn load_blocks_exact(&mut self, ids: &[BlockId]) -> Result<Vec<RawBlock>, SourceError> {
        let full_ids = ids
            .iter()
            .copied()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, SourceError>>()?;
        let downloaded = self
            .client
            .download_shards(&full_ids)
            .await
            .map_err(transport_error)?;
        let mut blocks = Vec::with_capacity(ids.len());

        for (&id, boc) in ids.iter().zip(downloaded) {
            let boc = boc.ok_or(SourceError::Unavailable(id))?;
            self.decoded.insert(id, BlockData::decode(id, &boc)?);
            blocks.push(RawBlock::new(id, boc));
        }

        Ok(blocks)
    }

    async fn shard_frontier(&mut self, block: &RawBlock) -> Result<Vec<BlockId>, SourceError> {
        if block.id().seqno == 0 {
            return Ok(Vec::new());
        }
        Ok(self.decode_cached(block)?.shard_frontier()?)
    }

    async fn predecessors(&mut self, block: &RawBlock) -> Result<Vec<BlockId>, SourceError> {
        Ok(self.decode_cached(block)?.predecessors()?)
    }

    async fn decode_block(&mut self, block: RawBlock) -> Result<BlockData, SourceError> {
        match self.decoded.remove(&block.id()) {
            Some(decoded) => Ok(decoded),
            None => Ok(BlockData::decode(block.id(), block.boc())?),
        }
    }
}

fn transport_error(error: anyhow::Error) -> SourceError {
    SourceError::Transport(format!("{error:#}"))
}
