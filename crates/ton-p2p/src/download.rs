//! Full-node RPCs and integrity checks for downloaded blocks and proofs.
//! Callers select peers and persist validated results.

use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};
use tokio::time::Instant;
use ton_fullnode_master::tl::{Answer, BlockIdExt, Query};
use tracing::debug;
use tycho_types::{
    boc::Boc,
    cell::{HashBytes, Load, LoadCell},
    merkle::MerkleProof,
    models::{Block, BlockId, BlockInfo, PrevBlockRef, ShardIdent},
};

use crate::network::{Network, Peer};

pub(crate) const MAX_DOWNLOAD_SIZE: usize = 8 * 1024 * 1024;

/// Uses uncompressed block/proof downloads, avoiding codecs that require a local
/// shard state. All synchronization RPCs use RLDP2, including metadata queries.
pub(crate) async fn download_from_peer(
    network: &Network,
    peer: &Peer,
    head: &BlockId,
    previous: Option<&BlockId>,
    deadline: Duration,
) -> Result<Option<(BlockId, Vec<u8>, Vec<u8>)>> {
    let id = if previous.is_some() {
        match small_query(
            network,
            peer,
            Query::GetNextBlockDescription {
                prev_block: wire_id(head),
            },
            deadline,
        )
        .await?
        {
            Answer::BlockDescription { id } => {
                ensure!(
                    id.workchain == -1 && id.shard == 1 << 63,
                    "peer returned a non-masterchain block"
                );
                ensure!(
                    head.seqno.checked_add(1) == Some(id.seqno),
                    "peer returned a nonconsecutive block"
                );
                BlockId {
                    shard: ShardIdent::MASTERCHAIN,
                    seqno: id.seqno,
                    root_hash: HashBytes(id.root_hash),
                    file_hash: HashBytes(id.file_hash),
                }
            }
            Answer::BlockDescriptionEmpty => return Ok(None),
            _ => bail!("unexpected next-block description"),
        }
    } else {
        *head
    };

    // getNextBlockDescription already reports availability of both the block
    // and its proof. Only the initial anchor needs separate availability probes.
    // Preparation is a status query, not a prerequisite for either download.
    if previous.is_none() {
        let (block, proof) = tokio::try_join!(
            small_query(
                network,
                peer,
                Query::PrepareBlock {
                    block: wire_id(&id),
                },
                deadline,
            ),
            small_query(
                network,
                peer,
                Query::PrepareBlockProof {
                    block: wire_id(&id),
                    allow_partial: false,
                },
                deadline,
            ),
        )?;

        match block {
            Answer::Prepared => {}
            Answer::NotFound => return Ok(None),
            _ => bail!("unexpected prepare-block response"),
        }
        match proof {
            Answer::PreparedProof => {}
            Answer::PreparedProofEmpty | Answer::PreparedProofLink => return Ok(None),
            _ => bail!("unexpected prepare-proof response"),
        }
    }

    // These two RPCs return raw BOC bytes, despite the tonNode.Data return type
    // in the schema. They do not carry a tonNode.data TL constructor.
    let block = fullnode_query(
        network,
        peer,
        Query::DownloadBlock {
            block: wire_id(&id),
        },
        deadline,
        MAX_DOWNLOAD_SIZE,
    );
    let proof = fullnode_query(
        network,
        peer,
        Query::DownloadBlockProof {
            block: wire_id(&id),
        },
        deadline,
        MAX_DOWNLOAD_SIZE,
    );
    let (block, proof) = tokio::try_join!(
        async {
            block
                .await
                .with_context(|| format!("cannot download block {id}"))
        },
        async {
            proof
                .await
                .with_context(|| format!("cannot download proof for {id}"))
        },
    )?;

    validate_download(&id, previous, &block, &proof)?;
    Ok(Some((id, block, proof)))
}

pub(crate) async fn small_query(
    network: &Network,
    peer: &Peer,
    query: Query,
    deadline: Duration,
) -> Result<Answer> {
    let response = fullnode_query(network, peer, query, deadline, 16 * 1024).await?;

    tl_proto::deserialize(&response).context("invalid full-node metadata response")
}

/// Shares request bounds and diagnostics between metadata and raw BOC downloads.
pub(crate) async fn fullnode_query(
    network: &Network,
    peer: &Peer,
    query: Query,
    deadline: Duration,
    max_answer_size: usize,
) -> Result<Vec<u8>> {
    let method = match &query {
        Query::GetNextBlockDescription { .. } => "tonNode.getNextBlockDescription",
        Query::PrepareBlock { .. } => "tonNode.prepareBlock",
        Query::PrepareBlockProof { .. } => "tonNode.prepareBlockProof",
        Query::DownloadBlock { .. } => "tonNode.downloadBlock",
        Query::DownloadBlockProof { .. } => "tonNode.downloadBlockProof",
        _ => bail!("unsupported synchronization query"),
    };
    let started = Instant::now();

    debug!(
        operation = "fullnode_query",
        node = %network.dht.key().id(),
        target = %peer.id,
        address = %peer.address,
        method,
        transport = "rldp2",
        "sending full-node query",
    );

    let response = network
        .rldp
        .query(
            &network.adnl,
            network.dht.key().id(),
            &peer.id,
            network.query_bytes(query),
            deadline,
            max_answer_size,
        )
        .await
        .with_context(|| format!("{method} to {} at {} failed", peer.id, peer.address))?;

    debug!(
        operation = "fullnode_query",
        node = %network.dht.key().id(),
        target = %peer.id,
        method,
        duration_ms = started.elapsed().as_millis(),
        outcome = "answered",
        bytes = response.len(),
        "full-node query answered",
    );

    Ok(response)
}

pub(crate) const fn wire_id(id: &BlockId) -> BlockIdExt {
    BlockIdExt {
        workchain: id.shard.workchain(),
        shard: id.shard.prefix(),
        seqno: id.seqno,
        root_hash: id.root_hash.0,
        file_hash: id.file_hash.0,
    }
}

/// Checks an original BOC against its requested ID before it enters the cache.
pub(crate) fn validate_block(id: &BlockId, block: &[u8]) -> Result<BlockInfo> {
    ensure!(
        block.len() <= MAX_DOWNLOAD_SIZE,
        "BOC exceeds download limit"
    );
    ensure!(
        Boc::file_hash(block) == id.file_hash,
        "block file hash mismatch"
    );

    let root = Boc::decode(block).context("invalid block BOC")?;
    ensure!(
        root.repr_hash() == &id.root_hash,
        "block root hash mismatch"
    );
    let block = root.parse::<Block>()?;
    let info = block.load_info()?;
    ensure!(
        info.shard == id.shard && info.seqno == id.seqno,
        "block header does not match requested ID"
    );

    Ok(info)
}

/// Verifies block hashes, the predecessor link, and the proof's Merkle root.
/// The proof must contain a signature set, but those signatures are not checked.
pub(crate) fn validate_download(
    id: &BlockId,
    previous: Option<&BlockId>,
    block: &[u8],
    proof: &[u8],
) -> Result<()> {
    ensure!(
        id.shard == ShardIdent::MASTERCHAIN && id.seqno > 0,
        "expected a masterchain block"
    );
    ensure!(
        proof.len() <= MAX_DOWNLOAD_SIZE,
        "proof exceeds download limit"
    );
    let info = validate_block(id, block)?;
    ensure!(
        !info.after_merge && !info.after_split && !info.before_split,
        "invalid masterchain split/merge flags"
    );

    if let Some(previous) = previous {
        ensure!(
            previous.seqno.checked_add(1) == Some(id.seqno),
            "nonconsecutive masterchain blocks"
        );
        let PrevBlockRef::Single(reference) = info.load_prev_ref()? else {
            bail!("masterchain block has multiple predecessors");
        };
        ensure!(
            reference.as_block_id(ShardIdent::MASTERCHAIN) == *previous,
            "block predecessor mismatch"
        );
    }

    let proof_root = Boc::decode(proof).context("invalid proof BOC")?;
    let mut slice = proof_root.as_slice()?;
    ensure!(slice.load_u8()? == 0xc3, "invalid block proof tag");
    ensure!(
        BlockId::load_from(&mut slice)? == *id,
        "proof belongs to a different block"
    );
    let merkle_cell = slice.load_reference_cloned()?;
    let merkle = MerkleProof::load_from_cell(merkle_cell.as_ref())?;
    ensure!(merkle.hash == id.root_hash, "proof Merkle root mismatch");
    ensure!(slice.load_bit()?, "masterchain proof has no signature set");
    let _signatures = slice.load_reference()?;
    ensure!(slice.is_empty(), "trailing data in block proof");

    Ok(())
}
