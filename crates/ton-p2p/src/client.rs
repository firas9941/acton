//! Downloads blocks and maintains a resumable cache independently of consumers.

use std::{collections::BTreeMap, path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Context, Result, ensure};
use futures::{StreamExt, stream};
use tokio::{
    sync::Semaphore,
    task::JoinSet,
    time::{Instant, timeout},
};
use ton_fullnode_master::tl::{Answer, Query};
use tracing::{debug, info, warn};
use tycho_types::models::{BlockId, ShardIdent};

use crate::{
    NetworkConfig, NetworkOptions,
    download::{
        MAX_DOWNLOAD_SIZE, download_from_peer, fullnode_query, small_query, validate_block, wire_id,
    },
    network::{Network, Peer},
    storage::{Storage, atomic_write},
};

const DISCOVERY_INTERVAL: Duration = Duration::from_secs(30);
const METADATA_TIMEOUT: Duration = Duration::from_secs(3);
const PEER_ATTEMPTS: usize = 16;
const PARALLEL_PEER_ATTEMPTS: usize = 8;

/// Connection settings and cache location for one client.
/// The data directory remains exclusively locked until the client is dropped.
pub struct ClientOptions {
    /// Advertised UDP address, identity, and per-request deadline.
    pub network: NetworkOptions,
    /// Original block BOCs and the masterchain download checkpoint.
    pub data_dir: PathBuf,
    /// Maximum concurrent shard requests, including competing peers, in the range 1..=128.
    pub parallelism: usize,
}

/// P2P block client with a persistent cache and a single masterchain download stream.
///
/// Dropping it cancels background discovery and prefetch, then releases the cache lock.
/// Callers decide which shard IDs to fetch and own any application-level progress.
pub struct Client {
    network: Arc<Network>,
    storage: Storage,
    ids: BTreeMap<u32, BlockId>,
    peers: Vec<Peer>,
    discovered_at: Option<Instant>,
    discovery: JoinSet<(Duration, Result<Vec<Peer>>)>,
    masterchain_download: JoinSet<Option<DownloadedMasterchain>>,
    options: ClientOptions,
}

/// A downloaded block awaiting commit. The background task cannot update storage.
struct DownloadedMasterchain {
    peer: Peer,
    id: BlockId,
    block: Vec<u8>,
    proof: Vec<u8>,
}

impl Client {
    /// Opens the cache and UDP transport. Peer discovery starts with the first
    /// download. Fails if settings, the saved head, or the directory lock are invalid.
    pub fn open(config: &NetworkConfig, options: ClientOptions) -> Result<Self> {
        ensure!(
            !options.network.timeout.is_zero(),
            "P2P timeout must be positive"
        );
        ensure!(
            (1..=128).contains(&options.parallelism),
            "P2P parallelism must be between 1 and 128"
        );

        let storage = Storage::open(&options.data_dir, config)?;
        storage.restore()?;
        let ids = storage.masterchain_ids()?;
        let network = Arc::new(Network::new(config, &options.network)?);

        info!(
            operation = "p2p_client",
            node = %network.dht.key().id(),
            target = %storage.head(),
            data_dir = %options.data_dir.display(),
            parallelism = options.parallelism,
            verification = "hashes_and_links",
            outcome = "opened",
            "opened P2P block client",
        );

        Ok(Self {
            network,
            storage,
            ids,
            peers: Vec::new(),
            discovered_at: None,
            discovery: JoinSet::new(),
            masterchain_download: JoinSet::new(),
            options,
        })
    }

    /// Returns the trusted starting block recorded in this cache.
    #[must_use]
    pub const fn anchor(&self) -> BlockId {
        self.storage.anchor()
    }

    /// Returns the persisted head, or `None` while the starting block is missing.
    /// A zerostate counts as a starting point without a block BOC.
    #[must_use]
    pub fn head(&self) -> Option<BlockId> {
        (!self.storage.needs_anchor()).then(|| self.storage.head())
    }

    /// Checks a consumer's position against the cached chain. A position ahead
    /// of the cache is allowed and must be checked again when downloads reach it.
    pub fn validate_checkpoint(&self, checkpoint: &BlockId) -> Result<()> {
        ensure!(
            checkpoint.shard == ShardIdent::MASTERCHAIN && checkpoint.seqno >= self.anchor().seqno,
            "checkpoint {checkpoint} precedes or conflicts with P2P anchor {}",
            self.anchor()
        );

        if checkpoint.seqno <= self.storage.head().seqno {
            ensure!(
                self.ids.get(&checkpoint.seqno) == Some(checkpoint),
                "checkpoint {checkpoint} conflicts with the masterchain cache"
            );
        }

        Ok(())
    }

    /// Downloads and commits the starting block or the successor of the saved head.
    /// Returns `None` when the current peers cannot supply it. The next block is
    /// prefetched in the background, but remains uncommitted until the next call.
    pub async fn next_masterchain(&mut self) -> Result<Option<BlockId>> {
        self.refresh_peers().await?;
        let downloaded = self.advance_masterchain().await?;

        if !downloaded {
            self.rotate_peers();
        }

        Ok(downloaded.then(|| self.storage.head()))
    }

    /// Reads a committed masterchain block by sequence number. Returns `None`
    /// outside the cached range or for seqno zero, which identifies a state.
    pub async fn masterchain_block(&self, seqno: u32) -> Result<Option<(BlockId, Vec<u8>)>> {
        let Some(&id) = self.ids.get(&seqno).filter(|id| id.seqno != 0) else {
            return Ok(None);
        };
        let path = self.storage.block_path(&id);
        let boc = tokio::fs::read(&path)
            .await
            .with_context(|| format!("cannot read masterchain block {}", path.display()))?;

        Ok(Some((id, boc)))
    }

    /// Loads shard BOCs from cache or peers in input order. Each ID must come
    /// from a trusted block reference. Missing blocks produce `None` entries;
    /// cache failures stop the operation. Valid downloads remain cached for retries.
    pub async fn download_shards(&mut self, ids: &[BlockId]) -> Result<Vec<Option<Vec<u8>>>> {
        ensure!(
            ids.iter()
                .all(|id| !id.shard.is_masterchain() && id.seqno > 0),
            "shard downloads require nonzero shard block IDs"
        );
        self.refresh_peers().await?;

        // Peer races share one budget across all shards. A stalled peer must
        // not serialize a block, and a wide frontier must not multiply the limit.
        let requests = Semaphore::new(self.options.parallelism);
        let mut downloads = stream::iter(ids.iter().copied().enumerate())
            .map(|(index, id)| {
                download_shard(
                    &self.network,
                    &self.peers,
                    &self.options,
                    &requests,
                    index,
                    id,
                )
            })
            .buffered(self.options.parallelism);
        let mut blocks = Vec::with_capacity(ids.len());
        let mut preferred = Vec::new();

        while let Some(result) = downloads.next().await {
            match result? {
                Some((boc, peer)) => {
                    blocks.push(Some(boc));

                    if let Some(peer) = peer {
                        preferred.push(peer);
                    }
                }
                None => blocks.push(None),
            }
        }
        drop(downloads);

        for peer in preferred.into_iter().rev() {
            if let Some(index) = self.peers.iter().position(|known| known.id == peer.id) {
                let peer = self.peers.remove(index);
                self.peers.insert(0, peer);
            }
        }

        // Some peers may serve the current shards but lack a predecessor.
        // Give other peers a turn when retrying an incomplete set of blocks.
        if blocks.iter().any(Option::is_none) {
            self.rotate_peers();
        }

        Ok(blocks)
    }

    fn rotate_peers(&mut self) {
        let count = self.peers.len().min(PEER_ATTEMPTS);
        self.peers.rotate_left(count);
    }

    async fn refresh_peers(&mut self) -> Result<()> {
        let interval = if self.peers.is_empty() {
            Duration::from_secs(2)
        } else {
            DISCOVERY_INTERVAL
        };
        if self.discovery.is_empty() && self.discovered_at.is_none_or(|at| at.elapsed() >= interval)
        {
            let network = Arc::clone(&self.network);
            let deadline = self.options.network.timeout;
            self.discovery.spawn(async move {
                let started = Instant::now();
                let result = timeout(deadline, network.find_peers())
                    .await
                    .context("block peer discovery timed out")
                    .and_then(std::convert::identity);
                (started.elapsed(), result)
            });
        }

        // Wait only during startup. Later DHT refreshes run alongside downloads.
        // Dropping the JoinSet cancels the outstanding discovery task.
        let result = if self.peers.is_empty() {
            self.discovery.join_next().await
        } else {
            self.discovery.try_join_next()
        };
        let Some(result) = result else {
            return Ok(());
        };
        let (duration, result) = result.context("block peer discovery task failed")?;

        match result {
            Ok(peers) => {
                // Keep successful peers at the front and add newly discovered
                // addresses. A DHT refresh can return only a subset of members.
                for peer in peers {
                    if let Some(known) = self.peers.iter_mut().find(|known| known.id == peer.id) {
                        known.address = peer.address;
                    } else {
                        self.peers.push(peer);
                    }
                }
                info!(
                    operation = "p2p_discovery",
                    node = %self.network.dht.key().id(),
                    target = %self.network.overlay_id,
                    peers = self.peers.len(),
                    duration_ms = duration.as_millis(),
                    outcome = "refreshed",
                    "refreshed block download peers",
                );
            }
            Err(error) => warn!(
                operation = "p2p_discovery",
                target = %self.network.overlay_id,
                duration_ms = duration.as_millis(),
                outcome = "retry",
                error = %format!("{error:#}"),
                "could not refresh block download peers",
            ),
        }
        self.discovered_at = Some(Instant::now());
        Ok(())
    }

    /// Downloads one successor while the caller processes the committed block.
    /// The task returns bytes; only `advance_masterchain` may commit them.
    fn prefetch_masterchain(&mut self) {
        if !self.masterchain_download.is_empty() || self.peers.is_empty() {
            return;
        }

        let head = self.storage.head();
        let previous = (!self.storage.needs_anchor()).then_some(head);
        let network = Arc::clone(&self.network);
        let deadline = self.options.network.timeout;
        let parallelism = self.options.parallelism.min(PARALLEL_PEER_ATTEMPTS);
        let peers = self
            .peers
            .iter()
            .take(PEER_ATTEMPTS)
            .cloned()
            .collect::<Vec<_>>();

        self.masterchain_download.spawn(async move {
            let network = &*network;
            let mut requests = stream::iter(peers)
                .map(|peer| async move {
                    let result =
                        download_from_peer(network, &peer, &head, previous.as_ref(), deadline)
                            .await;
                    (peer, result)
                })
                .buffer_unordered(parallelism);

            // Race several peers so one unresponsive peer cannot stall downloads.
            // The first valid result cancels the remaining requests.
            while let Some((peer, result)) = requests.next().await {
                match result {
                    Ok(Some((id, block, proof))) => {
                        return Some(DownloadedMasterchain {
                            peer,
                            id,
                            block,
                            proof,
                        });
                    }
                    Ok(None) => {}
                    Err(error) => debug!(
                        operation = "block_download",
                        target = %peer.id,
                        block = %head,
                        error = %format!("{error:#}"),
                        outcome = "retry",
                        "masterchain peer failed",
                    ),
                }
            }
            None
        });
    }

    /// Commits one verified download before exposing its ID to the caller.
    async fn advance_masterchain(&mut self) -> Result<bool> {
        self.prefetch_masterchain();
        let Some(result) = self.masterchain_download.join_next().await else {
            return Ok(false);
        };

        let Some(DownloadedMasterchain {
            peer,
            id,
            block,
            proof,
        }) = result.context("masterchain download task failed")?
        else {
            return Ok(false);
        };

        self.storage.commit(id, &block, &proof)?;
        self.ids.insert(id.seqno, id);

        if let Some(index) = self.peers.iter().position(|known| known.id == peer.id) {
            self.peers.swap(0, index);
        }

        self.prefetch_masterchain();
        Ok(true)
    }
}

/// Requests a shard by the full ID obtained from a masterchain block or a shard
/// predecessor reference. Peers on the masterchain overlay can serve shard data.
async fn download_shard(
    network: &Network,
    peers: &[Peer],
    options: &ClientOptions,
    requests: &Semaphore,
    offset: usize,
    id: BlockId,
) -> Result<Option<(Vec<u8>, Option<Peer>)>> {
    let directory = options
        .data_dir
        .join("shards")
        .join(id.shard.workchain().to_string())
        .join(format!("{:016x}", id.shard.prefix()));
    let name = format!("{}-{}-{}.boc", id.seqno, id.root_hash, id.file_hash);
    let path = directory.join(&name);

    match tokio::fs::read(&path).await {
        Ok(boc) => {
            validate_block(&id, &boc)?;
            return Ok(Some((boc, None)));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| format!("cannot read {}", path.display()));
        }
    }

    let candidates = peers
        .iter()
        .cycle()
        .skip(offset % peers.len().max(1))
        .take(peers.len().min(PEER_ATTEMPTS))
        .cloned()
        .collect::<Vec<_>>();
    let mut downloads = stream::iter(candidates)
        .map(|peer| async move {
            let started = Instant::now();
            let result = async {
                let _permit = requests.acquire().await?;

                match small_query(
                    network,
                    &peer,
                    Query::PrepareBlock {
                        block: wire_id(&id),
                    },
                    options.network.timeout.min(METADATA_TIMEOUT),
                )
                .await?
                {
                    Answer::NotFound => return Ok(None),
                    Answer::Prepared => {}
                    _ => anyhow::bail!("unexpected prepare-block response"),
                }

                let boc = fullnode_query(
                    network,
                    &peer,
                    Query::DownloadBlock {
                        block: wire_id(&id),
                    },
                    options.network.timeout,
                    MAX_DOWNLOAD_SIZE,
                )
                .await?;
                validate_block(&id, &boc)?;
                Ok::<_, anyhow::Error>(Some(boc))
            }
            .await;

            (peer, started, result)
        })
        .buffer_unordered(options.parallelism.min(PARALLEL_PEER_ATTEMPTS));

    while let Some((peer, started, result)) = downloads.next().await {
        match result {
            Ok(Some(boc)) => {
                // Stop the losing requests before persisting the verified winner.
                drop(downloads);

                // Persist the original BOC: reserialization can change file_hash.
                let boc = tokio::task::spawn_blocking(move || {
                    std::fs::create_dir_all(&directory)?;
                    atomic_write(&directory, &name, &boc)?;
                    Ok::<_, anyhow::Error>(boc)
                })
                .await
                .context("shard cache write task failed")??;
                debug!(
                    operation = "shard_download",
                    node = %network.dht.key().id(),
                    target = %peer.id,
                    block = %id,
                    bytes = boc.len(),
                    duration_ms = started.elapsed().as_millis(),
                    outcome = "stored",
                    "shard block saved",
                );
                return Ok(Some((boc, Some(peer))));
            }
            Ok(None) => {}
            Err(error) => debug!(
                operation = "shard_download",
                target = %peer.id,
                block = %id,
                duration_ms = started.elapsed().as_millis(),
                outcome = "retry",
                error = %format!("{error:#}"),
                "shard peer failed",
            ),
        }
    }
    Ok(None)
}
