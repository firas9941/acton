//! Transport-independent traversal of masterchain commitments and shard ancestry.

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use thiserror::Error;

use crate::{Batch, BlockData, BlockId, BlockSource, Error as IndexerError};

/// Errors produced by the block transports and canonical traversal.
#[derive(Debug, Error)]
pub enum SourceError {
    /// A global config could not be read or parsed.
    #[error("invalid global config: {0}")]
    GlobalConfig(String),
    /// A block transport or its local storage failed.
    #[error("block transport failed: {0}")]
    Transport(String),
    /// No currently reachable peer has this block; callers may poll again.
    #[error("block is not available from the current peers: {0}")]
    Unavailable(BlockId),
    /// Block coordinates are invalid for the requested operation.
    #[error("invalid block id: {0}")]
    InvalidBlockId(String),
    /// TON model decoding failed while inspecting a block.
    #[error(transparent)]
    Decode(#[from] crate::DecodeError),
    /// A loaded block differs from the full id committed by its successor.
    #[error("requested block {expected}, transport returned {actual}")]
    UnexpectedBlock {
        /// Full block id expected by the traversal.
        expected: Box<BlockId>,
        /// Full block id returned by the transport.
        actual: Box<BlockId>,
    },
    /// Two consecutive masterchain lookups do not form one chain.
    #[error("masterchain block {next} does not directly follow {previous}; references {actual:?}")]
    MasterchainDiscontinuity {
        /// Previous masterchain block loaded by sequence number.
        previous: Box<BlockId>,
        /// Next masterchain block loaded by sequence number.
        next: Box<BlockId>,
        /// Predecessors declared by the next block.
        actual: Vec<BlockId>,
    },
    /// A frontier walk exceeded its configured safety bound.
    #[error("shard delta exceeded the {limit}-block traversal limit")]
    TraversalLimit {
        /// Configured maximum.
        limit: usize,
    },
    /// The current and previous masterchain frontiers do not form one connected delta.
    #[error("shard frontier did not reach previous blocks: {missing:?}")]
    DisconnectedFrontier {
        /// Previous frontier ids not reached from the current frontier.
        missing: Vec<BlockId>,
    },
    /// A canonical batch invariant was violated.
    #[error(transparent)]
    Indexer(#[from] IndexerError),
    /// Raw transport blocks did not form one canonical batch.
    #[error("invalid raw batch: {0}")]
    InvalidBatch(String),
}

/// Block coordinates used when the transport must resolve representation hashes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BlockIdShort {
    /// Workchain identifier.
    pub workchain: i32,
    /// Shard prefix.
    pub shard: u64,
    /// Block sequence number.
    pub seqno: u32,
}

impl From<BlockId> for BlockIdShort {
    fn from(id: BlockId) -> Self {
        Self {
            workchain: id.workchain,
            shard: id.shard,
            seqno: id.seqno,
        }
    }
}

/// Exact block payload returned by a transport or local cache.
#[derive(Clone, Debug)]
pub struct RawBlock {
    id: BlockId,
    boc: Vec<u8>,
}

impl RawBlock {
    /// Creates a raw block with its full transport id.
    pub fn new(id: BlockId, boc: impl Into<Vec<u8>>) -> Self {
        Self {
            id,
            boc: boc.into(),
        }
    }

    /// Returns the full block id.
    #[must_use]
    pub const fn id(&self) -> BlockId {
        self.id
    }

    /// Returns the serialized block `BoC`.
    #[must_use]
    pub fn boc(&self) -> &[u8] {
        &self.boc
    }
}

/// Block access required by the canonical traversal.
///
/// Implement this trait to reuse the canonicalizer with another transport,
/// cache, archive, or test fixture.
#[async_trait]
pub trait BlockGraphClient: Send {
    /// Returns the latest known masterchain block.
    async fn latest_masterchain_block(&mut self) -> Result<BlockId, SourceError>;

    /// Resolves a short id and downloads the exact block `BoC`.
    async fn load_block(&mut self, id: BlockIdShort) -> Result<RawBlock, SourceError>;

    /// Downloads a block whose full id is already known.
    ///
    /// Clients may override this to avoid resolving the short id. The fallback
    /// preserves compatibility with transports that only support short lookups.
    async fn load_block_exact(&mut self, id: BlockId) -> Result<RawBlock, SourceError> {
        self.load_block(id.into()).await
    }

    /// Downloads multiple blocks whose full ids are already known.
    ///
    /// The default implementation is sequential. Network clients may override
    /// it to issue independent requests concurrently while preserving input
    /// order in the returned blocks.
    async fn load_blocks_exact(&mut self, ids: &[BlockId]) -> Result<Vec<RawBlock>, SourceError> {
        let mut blocks = Vec::with_capacity(ids.len());
        for &id in ids {
            blocks.push(self.load_block_exact(id).await?);
        }
        Ok(blocks)
    }

    /// Reads the shard frontier committed by a masterchain block.
    async fn shard_frontier(&mut self, mc_block: &RawBlock) -> Result<Vec<BlockId>, SourceError>;

    /// Reads the direct predecessor ids committed inside a block.
    async fn predecessors(&mut self, block: &RawBlock) -> Result<Vec<BlockId>, SourceError>;

    /// Decodes an exact raw block after the traversal has made it canonical.
    async fn decode_block(&mut self, block: RawBlock) -> Result<BlockData, SourceError> {
        Ok(BlockData::decode(block.id, &block.boc)?)
    }
}

/// Produces one canonical batch for each masterchain block.
pub struct CanonicalBlockSource<C> {
    client: C,
    start_seqno: u32,
    max_shard_blocks: usize,
    known_tip_seqno: Option<u32>,
    last_emitted_masterchain: Option<CachedMasterchainState>,
}

impl<C> CanonicalBlockSource<C> {
    /// Default maximum number of shard blocks visited for one masterchain step.
    pub const DEFAULT_MAX_SHARD_BLOCKS: usize = 10_000;

    /// Starts at the provided masterchain sequence number when no checkpoint exists.
    pub const fn new(client: C, start_seqno: u32) -> Self {
        Self {
            client,
            start_seqno,
            max_shard_blocks: Self::DEFAULT_MAX_SHARD_BLOCKS,
            known_tip_seqno: None,
            last_emitted_masterchain: None,
        }
    }

    /// Changes the per-batch shard traversal safety bound.
    #[must_use]
    pub const fn with_max_shard_blocks(mut self, limit: usize) -> Self {
        self.max_shard_blocks = limit;
        self
    }

    /// Returns a shared reference to the underlying client.
    pub const fn client(&self) -> &C {
        &self.client
    }

    /// Returns a mutable reference to the underlying client.
    pub const fn client_mut(&mut self) -> &mut C {
        &mut self.client
    }

    /// Consumes the source and returns its client.
    pub fn into_client(self) -> C {
        self.client
    }
}

#[async_trait]
impl<C> BlockSource for CanonicalBlockSource<C>
where
    C: BlockGraphClient,
{
    async fn next_batch(&mut self, after: Option<&BlockId>) -> crate::Result<Option<Batch>> {
        CanonicalBlockSource::next_batch(self, after)
            .await
            .map_err(IndexerError::source)
    }
}

impl<C> CanonicalBlockSource<C>
where
    C: BlockGraphClient,
{
    /// Loads, canonicalizes, and decodes the next full batch.
    ///
    /// # Errors
    ///
    /// Returns an error for transport, decoding, checkpoint-continuity, or
    /// shard-frontier traversal failures.
    pub async fn next_batch(
        &mut self,
        after: Option<&BlockId>,
    ) -> Result<Option<Batch>, SourceError> {
        let Some(raw) = self.next_raw_batch(after).await? else {
            return Ok(None);
        };

        let masterchain = self.client.decode_block(raw.masterchain).await?;
        let mut shards = Vec::with_capacity(raw.shards.len());
        for block in raw.shards {
            shards.push(self.client.decode_block(block).await?);
        }
        Ok(Some(Batch::try_new(masterchain, shards)?))
    }

    async fn next_raw_batch(
        &mut self,
        after: Option<&BlockId>,
    ) -> Result<Option<RawBatch>, SourceError> {
        let next_seqno = match after {
            Some(checkpoint) => checkpoint.seqno.saturating_add(1),
            None => self.start_seqno,
        };
        if !self.tip_covers(next_seqno).await? {
            return Ok(None);
        }

        let mc_block = self.load_masterchain(next_seqno).await?;
        let previous = if next_seqno == 0 {
            None
        } else if let Some(cached) = self.cached_previous(after, next_seqno) {
            Some(cached)
        } else {
            let block = self.load_masterchain(next_seqno - 1).await?;
            let frontier = self.client.shard_frontier(&block).await?;
            Some(CachedMasterchainState {
                id: block.id,
                frontier,
            })
        };

        if let (Some(checkpoint), Some(previous)) = (after, previous.as_ref()) {
            Self::verify_block_id(checkpoint, previous.id)?;
        }
        if let Some(previous) = previous.as_ref() {
            self.verify_masterchain_link(previous.id, &mc_block).await?;
        }

        let previous_frontier = previous.map_or_else(Vec::new, |previous| previous.frontier);
        let current_frontier = self.client.shard_frontier(&mc_block).await?;
        let shard_blocks = self
            .collect_shard_delta(previous_frontier, current_frontier.clone())
            .await?;
        let batch = RawBatch::try_new(mc_block, shard_blocks)?;

        self.last_emitted_masterchain = Some(CachedMasterchainState {
            id: batch.masterchain.id,
            frontier: current_frontier,
        });
        Ok(Some(batch))
    }

    async fn tip_covers(&mut self, seqno: u32) -> Result<bool, SourceError> {
        if self
            .known_tip_seqno
            .is_some_and(|known_tip| seqno <= known_tip)
        {
            return Ok(true);
        }

        let tip = self.client.latest_masterchain_block().await?;
        self.known_tip_seqno = Some(tip.seqno);
        Ok(seqno <= tip.seqno)
    }

    fn cached_previous(
        &self,
        after: Option<&BlockId>,
        next_seqno: u32,
    ) -> Option<CachedMasterchainState> {
        let checkpoint = after?;
        let cached = self.last_emitted_masterchain.as_ref()?;
        if cached.id.seqno.checked_add(1) != Some(next_seqno) || cached.id != *checkpoint {
            return None;
        }
        Some(cached.clone())
    }

    async fn load_masterchain(&mut self, seqno: u32) -> Result<RawBlock, SourceError> {
        let block = self
            .client
            .load_block(BlockIdShort {
                workchain: BlockId::MASTERCHAIN_WORKCHAIN,
                shard: BlockId::FULL_SHARD,
                seqno,
            })
            .await?;
        if !block.id.is_masterchain()
            || block.id.shard != BlockId::FULL_SHARD
            || block.id.seqno != seqno
        {
            return Err(SourceError::InvalidBlockId(format!(
                "expected masterchain seqno {seqno}, got {}",
                block.id
            )));
        }
        Ok(block)
    }

    fn verify_block_id(expected: &BlockId, actual: BlockId) -> Result<(), SourceError> {
        if actual != *expected {
            return Err(SourceError::UnexpectedBlock {
                expected: Box::new(*expected),
                actual: Box::new(actual),
            });
        }
        Ok(())
    }

    async fn verify_masterchain_link(
        &mut self,
        previous: BlockId,
        next: &RawBlock,
    ) -> Result<(), SourceError> {
        let predecessors = self.client.predecessors(next).await?;
        if predecessors.as_slice() != [previous] {
            return Err(SourceError::MasterchainDiscontinuity {
                previous: Box::new(previous),
                next: Box::new(next.id),
                actual: predecessors,
            });
        }
        Ok(())
    }

    async fn collect_shard_delta(
        &mut self,
        previous_frontier: Vec<BlockId>,
        mut current_frontier: Vec<BlockId>,
    ) -> Result<Vec<RawBlock>, SourceError> {
        current_frontier.sort_unstable();
        let stop = previous_frontier.into_iter().collect::<HashSet<_>>();

        let roots = current_frontier.clone();
        let mut pending = current_frontier;
        let mut discovered = HashSet::new();
        let mut reached = HashSet::new();
        let mut loaded = HashMap::<BlockId, (RawBlock, Vec<BlockId>)>::new();

        while !pending.is_empty() {
            pending.sort_unstable();
            pending.dedup();
            let mut wave = Vec::with_capacity(pending.len());
            for id in std::mem::take(&mut pending) {
                if stop.contains(&id) {
                    reached.insert(id);
                } else if id.seqno == 0 {
                    // Zerostates are ancestry boundaries, not transaction blocks.
                    continue;
                } else if discovered.insert(id) {
                    wave.push(id);
                }
            }
            if discovered.len() > self.max_shard_blocks {
                return Err(SourceError::TraversalLimit {
                    limit: self.max_shard_blocks,
                });
            }
            if wave.is_empty() {
                break;
            }

            let blocks = self.client.load_blocks_exact(&wave).await?;
            if blocks.len() != wave.len() {
                return Err(SourceError::InvalidBatch(format!(
                    "requested {} exact shard blocks, transport returned {}",
                    wave.len(),
                    blocks.len()
                )));
            }
            for (&expected, block) in wave.iter().zip(blocks) {
                Self::verify_block_id(&expected, block.id)?;
                let mut predecessors = self.client.predecessors(&block).await?;
                predecessors.sort_unstable();
                pending.extend(predecessors.iter().copied());
                loaded.insert(expected, (block, predecessors));
            }
        }

        if reached != stop {
            let mut missing = stop.difference(&reached).copied().collect::<Vec<_>>();
            missing.sort_unstable();
            return Err(SourceError::DisconnectedFrontier { missing });
        }

        // Network discovery runs breadth-first to expose parallel requests. Build
        // the result afterwards in the same deterministic predecessor-first order
        // as the former depth-first traversal.
        let mut frames = roots
            .into_iter()
            .rev()
            .map(TraversalFrame::Enter)
            .collect::<Vec<_>>();
        let mut ordered = HashSet::new();
        let mut blocks = Vec::with_capacity(loaded.len());
        while let Some(frame) = frames.pop() {
            match frame {
                TraversalFrame::Enter(id) if stop.contains(&id) || id.seqno == 0 => {}
                TraversalFrame::Enter(id) if !ordered.insert(id) => {}
                TraversalFrame::Enter(id) => {
                    let (_, predecessors) = loaded.get(&id).ok_or_else(|| {
                        SourceError::InvalidBatch(format!(
                            "shard traversal did not load discovered block {id}"
                        ))
                    })?;
                    frames.push(TraversalFrame::Exit(id));
                    frames.extend(
                        predecessors
                            .iter()
                            .rev()
                            .copied()
                            .map(TraversalFrame::Enter),
                    );
                }
                TraversalFrame::Exit(id) => {
                    let (block, _) = loaded.remove(&id).ok_or_else(|| {
                        SourceError::InvalidBatch(format!(
                            "shard traversal emitted missing block {id}"
                        ))
                    })?;
                    blocks.push(block);
                }
            }
        }
        Ok(blocks)
    }
}

#[derive(Debug)]
struct RawBatch {
    masterchain: RawBlock,
    shards: Vec<RawBlock>,
}

impl RawBatch {
    fn try_new(masterchain: RawBlock, shards: Vec<RawBlock>) -> Result<Self, SourceError> {
        if !masterchain.id.is_masterchain() || masterchain.id.shard != BlockId::FULL_SHARD {
            return Err(SourceError::InvalidBatch(format!(
                "{} is not a masterchain block",
                masterchain.id
            )));
        }

        let mut ids = HashSet::with_capacity(shards.len());
        for block in &shards {
            if block.id.is_masterchain() {
                return Err(SourceError::InvalidBatch(format!(
                    "masterchain block {} appeared in the shard delta",
                    block.id
                )));
            }
            if !ids.insert(block.id) {
                return Err(SourceError::InvalidBatch(format!(
                    "duplicate shard block {}",
                    block.id
                )));
            }
        }
        Ok(Self {
            masterchain,
            shards,
        })
    }
}

enum TraversalFrame {
    Enter(BlockId),
    Exit(BlockId),
}

#[derive(Clone)]
struct CachedMasterchainState {
    id: BlockId,
    frontier: Vec<BlockId>,
}
