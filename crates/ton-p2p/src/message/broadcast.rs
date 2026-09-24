//! Signed overlay packets for ordinary external messages and FEC broadcasts.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, ensure};
use everscale_network::{adnl, overlay, proto};
use everscale_raptorq::{Encoder, ObjectTransmissionInformation};
use sha2::{Digest, Sha256};
use tl_proto::TlWrite;

/// Builds a bounded batch of signed packets. Callers pace their transmission.
/// FEC uses one source block and adds 50% repair symbols for UDP packet loss.
pub(super) fn encode(
    key: &adnl::Key,
    overlay: &overlay::IdShort,
    data: &[u8],
) -> Result<Vec<Vec<u8>>> {
    let date = u32::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())?;
    let prefix = tl_proto_network::serialize(proto::overlay::Message {
        overlay: overlay.as_slice(),
    });
    let source = *key.id().as_slice();
    let data_hash = Sha256::digest(data).into();
    let sign = |value: ToSign| {
        key.secret_key()
            .sign_raw(&tl_proto::serialize(value), key.full_id().public_key())
    };
    let packet = |broadcast: proto::overlay::Broadcast<'_>| {
        let mut bytes = prefix.clone();
        tl_proto_network::TlWrite::write_to(&broadcast, &mut bytes);
        bytes
    };

    // Public TON overlays require a named signer. AnySender broadcasts are
    // rejected before message validation, even with a valid signature.
    if data.len() <= 768 {
        let signature = sign(ToSign {
            hash: tl_proto::hash(BroadcastId {
                source,
                data_hash,
                flags: 0,
            }),
            date,
        });
        return Ok(vec![packet(proto::overlay::Broadcast::Broadcast(
            proto::overlay::OverlayBroadcast {
                src: key.full_id().as_tl(),
                certificate: proto::overlay::Certificate::EmptyCertificate,
                flags: 0,
                data,
                date,
                signature: &signature,
            },
        ))]);
    }

    let data_size = u32::try_from(data.len())?;
    let symbol_size = 768_u16;
    let config = ObjectTransmissionInformation::new(u64::from(data_size), symbol_size, 1, 1, 1);
    let encoder = Encoder::new(data, config);
    let block = encoder
        .get_block_encoders()
        .first()
        .context("missing FEC source block")?;
    ensure!(
        encoder.get_block_encoders().len() == 1,
        "external message exceeds one FEC source block"
    );
    let fec = proto::rldp::RaptorQFecType {
        total_len: data_size,
        packet_len: u32::from(symbol_size),
        packet_count: data_size.div_ceil(u32::from(symbol_size)),
    };
    let hash = tl_proto::hash(FecBroadcastId {
        source,
        type_hash: tl_proto_network::hash(fec),
        data_hash,
        size: data_size,
        flags: 0,
    });
    let mut symbols = block.source_packets();
    symbols.extend(block.repair_packets(fec.packet_count, fec.packet_count.div_ceil(2)));
    let mut packets = Vec::with_capacity(symbols.len());

    for symbol in symbols {
        let seqno = symbol.payload_id().encoding_symbol_id();
        let signature = sign(ToSign {
            hash: tl_proto::hash(FecPartId {
                broadcast_hash: hash,
                data_hash: Sha256::digest(symbol.data()).into(),
                seqno,
            }),
            date,
        });
        packets.push(packet(proto::overlay::Broadcast::BroadcastFec(
            proto::overlay::OverlayBroadcastFec {
                src: key.full_id().as_tl(),
                certificate: proto::overlay::Certificate::EmptyCertificate,
                data_hash: &data_hash,
                data_size,
                flags: 0,
                data: symbol.data(),
                seqno,
                fec,
                date,
                signature: &signature,
            },
        )));
    }

    Ok(packets)
}

#[derive(TlWrite)]
#[tl(
    boxed,
    id = "overlay.broadcast.id",
    scheme_inline = "overlay.broadcast.id src:int256 data_hash:int256 flags:int = overlay.broadcast.Id;"
)]
struct BroadcastId {
    source: [u8; 32],
    data_hash: [u8; 32],
    flags: u32,
}

#[derive(TlWrite)]
#[tl(
    boxed,
    id = "overlay.broadcastFec.id",
    scheme_inline = "overlay.broadcastFec.id src:int256 type:int256 data_hash:int256 size:int flags:int = overlay.broadcastFec.Id;"
)]
struct FecBroadcastId {
    source: [u8; 32],
    type_hash: [u8; 32],
    data_hash: [u8; 32],
    size: u32,
    flags: u32,
}

#[derive(TlWrite)]
#[tl(
    boxed,
    id = "overlay.broadcastFec.partId",
    scheme_inline = "overlay.broadcastFec.partId broadcast_hash:int256 data_hash:int256 seqno:int = overlay.broadcastFec.PartId;"
)]
struct FecPartId {
    broadcast_hash: [u8; 32],
    data_hash: [u8; 32],
    seqno: u32,
}

#[derive(TlWrite)]
#[tl(
    boxed,
    id = "overlay.broadcast.toSign",
    scheme_inline = "overlay.broadcast.toSign hash:int256 date:int = overlay.broadcast.ToSign;"
)]
struct ToSign {
    hash: [u8; 32],
    date: u32,
}
