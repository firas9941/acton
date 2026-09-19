//! Direct `LiteAPI` source for canonical, decoded TON batches.
//!
//! [`TonutilsLiteClient`] owns the ADNL/LiteAPI transport. Canonical traversal
//! lives in [`ton_indexer_core::CanonicalBlockSource`] and is shared with P2P.

#[cfg(test)]
mod tests;

use std::{collections::HashMap, path::Path, time::Duration};

use async_trait::async_trait;
use futures::{StreamExt, future::join_all, stream::FuturesUnordered};
use ton_indexer_core::{
    BlockData, BlockGraphClient, BlockId, BlockIdShort, Hash256, RawBlock, SourceError,
};
use tonutils::{
    liteclient::client::LiteClient,
    network_config::{ConfigGlobal, ConfigLiteServer},
    tl::common::{BlockId as LiteBlockId, BlockIdExt as LiteBlockIdExt, Int256},
};

/// Counts `LiteServer` TL requests issued by [`TonutilsLiteClient`].
///
/// The counters are incremented immediately before a request is sent, so failed
/// requests are included. Establishing the ADNL connection itself is a transport
/// operation and is not included.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LiteRequestStats {
    get_masterchain_info: u64,
    lookup_block: u64,
    get_block: u64,
}

impl LiteRequestStats {
    /// Returns the number of `liteServer.getMasterchainInfo` requests.
    #[must_use]
    pub const fn get_masterchain_info(self) -> u64 {
        self.get_masterchain_info
    }

    /// Returns the number of `liteServer.lookupBlock` requests.
    #[must_use]
    pub const fn lookup_block(self) -> u64 {
        self.lookup_block
    }

    /// Returns the number of `liteServer.getBlock` requests.
    #[must_use]
    pub const fn get_block(self) -> u64 {
        self.get_block
    }

    /// Returns the total number of counted `LiteServer` requests.
    #[must_use]
    pub const fn total(self) -> u64 {
        self.get_masterchain_info + self.lookup_block + self.get_block
    }

    /// Returns requests made since an earlier snapshot.
    #[must_use]
    pub const fn since(self, earlier: Self) -> Self {
        Self {
            get_masterchain_info: self
                .get_masterchain_info
                .saturating_sub(earlier.get_masterchain_info),
            lookup_block: self.lookup_block.saturating_sub(earlier.lookup_block),
            get_block: self.get_block.saturating_sub(earlier.get_block),
        }
    }
}

/// Direct ADNL/LiteAPI client backed by `tonutils`.
pub struct TonutilsLiteClient {
    inner: LiteClient,
    exact_clients: Vec<LiteClient>,
    decoded: HashMap<BlockId, BlockData>,
    request_stats: LiteRequestStats,
}

impl TonutilsLiteClient {
    const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
    const WORKER_CONNECT_TIMEOUT: Duration = Duration::from_secs(1);
    const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
    const DEFAULT_PARALLEL_CLIENTS: usize = 4;
    const MAX_PARALLEL_CLIENTS: usize = 16;

    /// Connects to a responsive liteserver pool from a parsed global config.
    ///
    /// # Errors
    ///
    /// Returns an error when the config has no liteservers or none of them
    /// accepts an ADNL connection and answers a `getMasterchainInfo` probe.
    pub async fn connect(config: &ConfigGlobal) -> Result<Self, SourceError> {
        Self::connect_with_parallelism(config, Self::DEFAULT_PARALLEL_CLIENTS).await
    }

    /// Connects with a bounded number of clients for concurrent exact block loads.
    ///
    /// Values above 16 are capped to protect public liteservers. Zero is treated
    /// as one client.
    ///
    /// # Errors
    ///
    /// Returns an error when the config has no liteservers or none of them
    /// accepts an ADNL connection and answers a `getMasterchainInfo` probe.
    pub async fn connect_with_parallelism(
        config: &ConfigGlobal,
        parallelism: usize,
    ) -> Result<Self, SourceError> {
        if config.liteservers.is_empty() {
            return Err(SourceError::GlobalConfig(
                "network config has no liteservers".into(),
            ));
        }

        let mut failures = Vec::with_capacity(config.liteservers.len());
        let mut request_stats = LiteRequestStats::default();
        let mut attempts = FuturesUnordered::new();
        for (index, liteserver) in config.liteservers.iter().enumerate() {
            attempts.push(async move {
                let (probed, result) = connect_liteserver(liteserver, Self::CONNECT_TIMEOUT).await;
                (index, liteserver.clone(), probed, result)
            });
        }

        let mut selected = None;
        while let Some((index, liteserver, probed, result)) = attempts.next().await {
            request_stats.get_masterchain_info += u64::from(probed);
            match result {
                Ok(client) => {
                    selected = Some((liteserver, client));
                    break;
                }
                Err(error) => failures.push(format!("#{index}: {error}")),
            }
        }
        drop(attempts);

        if let Some((liteserver, inner)) = selected {
            let parallelism = parallelism.clamp(1, Self::MAX_PARALLEL_CLIENTS);
            let worker_attempts = (1..parallelism).map(|_| async {
                connect_liteserver(&liteserver, Self::WORKER_CONNECT_TIMEOUT).await
            });
            let mut exact_clients = Vec::with_capacity(parallelism - 1);
            for (probed, result) in join_all(worker_attempts).await {
                request_stats.get_masterchain_info += u64::from(probed);
                if let Ok(client) = result {
                    exact_clients.push(client);
                }
            }
            return Ok(Self {
                inner,
                exact_clients,
                decoded: HashMap::new(),
                request_stats,
            });
        }

        Err(SourceError::Transport(format!(
            "none of {} configured liteservers is responsive ({})",
            config.liteservers.len(),
            failures.join("; ")
        )))
    }

    /// Reads a global config and connects to its first responsive liteserver.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be read or parsed, or when the
    /// ADNL connection cannot be established.
    pub async fn connect_path(path: impl AsRef<Path>) -> Result<Self, SourceError> {
        Self::connect_path_with_parallelism(path, Self::DEFAULT_PARALLEL_CLIENTS).await
    }

    /// Reads a global config and connects with configurable exact-load parallelism.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be read or parsed, or when the
    /// ADNL connection cannot be established.
    pub async fn connect_path_with_parallelism(
        path: impl AsRef<Path>,
        parallelism: usize,
    ) -> Result<Self, SourceError> {
        let path = path.as_ref();
        let source = tokio::fs::read_to_string(path).await.map_err(|error| {
            SourceError::GlobalConfig(format!("failed to read {}: {error}", path.display()))
        })?;
        let config = source.parse::<ConfigGlobal>().map_err(|error| {
            SourceError::GlobalConfig(format!("failed to parse {}: {error}", path.display()))
        })?;
        Self::connect_with_parallelism(&config, parallelism).await
    }

    /// Returns the latest masterchain id without constructing a source.
    ///
    /// # Errors
    ///
    /// Returns an error when the `LiteAPI` request or id conversion fails.
    pub async fn latest(&mut self) -> Result<BlockId, SourceError> {
        self.latest_masterchain_block().await
    }

    /// Resolves a bootstrap anchor while checking the server's network identity.
    /// The returned ID is trusted metadata, not a verified chain of block proofs.
    /// No block bodies or account states are downloaded by this request.
    pub async fn latest_for_network(
        &mut self,
        zero_state: BlockId,
    ) -> Result<BlockId, SourceError> {
        self.request_stats.get_masterchain_info += 1;
        let info = self
            .inner
            .get_masterchain_info()
            .await
            .map_err(|error| SourceError::Transport(error.to_string()))?;

        if !zero_state.is_masterchain()
            || zero_state.shard != BlockId::FULL_SHARD
            || zero_state.seqno != 0
            || info.init.workchain != zero_state.workchain
            || info.init.root_hash.0 != zero_state.root_hash.into_bytes()
            || info.init.file_hash.0 != zero_state.file_hash.into_bytes()
        {
            return Err(SourceError::InvalidBlockId(
                "liteserver zerostate does not match the requested network".into(),
            ));
        }

        let last = from_lite_block_id(&info.last)?;
        if !last.is_masterchain() || last.shard != BlockId::FULL_SHARD {
            return Err(SourceError::InvalidBlockId(
                "liteserver head must belong to the masterchain".into(),
            ));
        }
        Ok(last)
    }

    /// Returns the number of messages waiting in all shard outbound queues.
    ///
    /// The liteserver reports one queue size per shard. Summing them produces
    /// the network-wide backlog expected by monitoring consumers.
    ///
    /// # Errors
    ///
    /// Returns an error when the `LiteAPI` request fails.
    pub async fn out_msg_queue_size(&mut self) -> Result<u64, SourceError> {
        let sizes = self
            .inner
            .get_out_msg_queue_sizes(None)
            .await
            .map_err(|error| SourceError::Transport(error.to_string()))?;

        Ok(sizes.shards.into_iter().fold(0_u64, |total, shard| {
            total.saturating_add(u64::from(shard.size))
        }))
    }

    /// Returns a snapshot of the requests issued by this client.
    #[must_use]
    pub const fn request_stats(&self) -> LiteRequestStats {
        self.request_stats
    }

    /// Returns the maximum number of exact block downloads issued concurrently.
    #[must_use]
    pub const fn exact_block_parallelism(&self) -> usize {
        1 + self.exact_clients.len()
    }

    fn decode_cached(&mut self, raw: &RawBlock) -> Result<&BlockData, SourceError> {
        Ok(match self.decoded.entry(raw.id()) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(BlockData::decode(raw.id(), raw.boc())?)
            }
        })
    }
}

async fn connect_liteserver(
    liteserver: &ConfigLiteServer,
    connect_timeout: Duration,
) -> (bool, Result<LiteClient, String>) {
    let connection = LiteClient::connect_with_timeout(
        liteserver.socket_addr(),
        liteserver.public_key(),
        connect_timeout,
    )
    .await;
    let mut client = match connection {
        Ok(client) => client.with_request_timeout(TonutilsLiteClient::REQUEST_TIMEOUT),
        Err(error) => return (false, Err(format!("connect failed: {error}"))),
    };

    match client.get_masterchain_info().await {
        Ok(_) => (true, Ok(client)),
        Err(error) => (true, Err(format!("probe failed: {error}"))),
    }
}

#[async_trait]
impl BlockGraphClient for TonutilsLiteClient {
    async fn latest_masterchain_block(&mut self) -> Result<BlockId, SourceError> {
        self.request_stats.get_masterchain_info += 1;
        let info = self
            .inner
            .get_masterchain_info()
            .await
            .map_err(|error| SourceError::Transport(error.to_string()))?;
        from_lite_block_id(&info.last)
    }

    async fn load_block(&mut self, id: BlockIdShort) -> Result<RawBlock, SourceError> {
        let lite_id = to_lite_short_id(id)?;
        self.request_stats.lookup_block += 1;
        let header = self
            .inner
            .lookup_block(
                (),
                lite_id,
                Some(()),
                None,
                None,
                false,
                false,
                false,
                false,
                false,
            )
            .await
            .map_err(|error| SourceError::Transport(error.to_string()))?;
        let full_id = from_lite_block_id(&header.id)?;
        self.request_stats.get_block += 1;
        let boc = self
            .inner
            .get_block(header.id)
            .await
            .map_err(|error| SourceError::Transport(error.to_string()))?;
        Ok(RawBlock::new(full_id, boc))
    }

    async fn load_block_exact(&mut self, id: BlockId) -> Result<RawBlock, SourceError> {
        self.load_blocks_exact(&[id])
            .await?
            .pop()
            .ok_or_else(|| SourceError::InvalidBatch("exact block load returned no block".into()))
    }

    async fn load_blocks_exact(&mut self, ids: &[BlockId]) -> Result<Vec<RawBlock>, SourceError> {
        let request_count = u64::try_from(ids.len()).unwrap_or(u64::MAX);
        self.request_stats.get_block = self.request_stats.get_block.saturating_add(request_count);

        let mut blocks = Vec::with_capacity(ids.len());
        let parallelism = 1 + self.exact_clients.len();
        for ids in ids.chunks(parallelism) {
            let Some((&first, rest)) = ids.split_first() else {
                continue;
            };
            let mut requests = Vec::with_capacity(ids.len());
            requests.push(download_exact_block(&mut self.inner, first));
            requests.extend(
                self.exact_clients
                    .iter_mut()
                    .zip(rest)
                    .map(|(client, &id)| download_exact_block(client, id)),
            );

            for result in join_all(requests).await {
                let (id, boc) = result?;
                // `tonutils::get_block` returns only the response payload. Decode
                // it now to verify both hashes and all block coordinates. Caching
                // also prevents a second decode during traversal.
                self.decoded.insert(id, BlockData::decode(id, &boc)?);
                blocks.push(RawBlock::new(id, boc));
            }
        }
        Ok(blocks)
    }

    async fn shard_frontier(&mut self, mc_block: &RawBlock) -> Result<Vec<BlockId>, SourceError> {
        Ok(self.decode_cached(mc_block)?.shard_frontier()?)
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

fn to_lite_short_id(id: BlockIdShort) -> Result<LiteBlockId, SourceError> {
    Ok(LiteBlockId {
        workchain: id.workchain,
        shard: i64::from_ne_bytes(id.shard.to_ne_bytes()),
        seqno: i32::try_from(id.seqno).map_err(|_| {
            SourceError::InvalidBlockId(format!("seqno {} exceeds signed TL range", id.seqno))
        })?,
    })
}

fn to_lite_block_id_ext(id: BlockId) -> Result<LiteBlockIdExt, SourceError> {
    let short = to_lite_short_id(id.into())?;
    Ok(LiteBlockIdExt {
        workchain: short.workchain,
        shard: short.shard,
        seqno: short.seqno,
        root_hash: Int256(id.root_hash.into_bytes()),
        file_hash: Int256(id.file_hash.into_bytes()),
    })
}

async fn download_exact_block(
    client: &mut LiteClient,
    id: BlockId,
) -> Result<(BlockId, Vec<u8>), SourceError> {
    let boc = client
        .get_block(to_lite_block_id_ext(id)?)
        .await
        .map_err(|error| SourceError::Transport(error.to_string()))?;
    Ok((id, boc))
}

fn from_lite_block_id(id: &LiteBlockIdExt) -> Result<BlockId, SourceError> {
    Ok(BlockId {
        workchain: id.workchain,
        shard: u64::from_ne_bytes(id.shard.to_ne_bytes()),
        seqno: u32::try_from(id.seqno)
            .map_err(|_| SourceError::InvalidBlockId(format!("negative seqno {}", id.seqno)))?,
        root_hash: Hash256::new(id.root_hash.0),
        file_hash: Hash256::new(id.file_hash.0),
    })
}
