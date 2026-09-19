# Actonscan backend

The backend indexes canonical TON blocks from LiteServer or native P2P.
It stores TPS samples, opcode statistics, and the indexer checkpoint in SQLite.

## Run locally

```sh
ACTONSCAN_CONFIG=apps/actonscan-backend/config.toml \
  cargo run --manifest-path apps/actonscan-backend/Cargo.toml --locked
```

The server listens on `127.0.0.1:3008`. It provides these endpoints:

- `GET /healthz`
- `GET /openapi.json`
- `GET /api/v1/stats/tps`
- `GET /api/v1/stats/opcodes`

## Opcode statistics

`GET /api/v1/stats/opcodes` returns all-time opcode statistics. The default
values are `min_messages=2` and `limit=100`. The maximum limit is 1000.

The backend extracts opcodes from internal messages and outgoing external
messages at their source transaction. It does not count internal messages
again at their destination transaction.

Incoming external messages remain in `total_messages`, but the backend does
not extract opcodes from them. Bodies shorter than 32 bits also remain in
`total_messages`, but not in `messages_with_opcode`.

For a bounced message, the backend skips the 32-bit bounce prefix. SQLite
stores one row per opcode and up to two example transaction hashes. Each hash
is a 32-byte BLOB. A singleton opcode has no stored hash.

After a restart, the indexer continues from the stored checkpoint.

## Configuration

The backend reads `config.toml`. Set `ACTONSCAN_CONFIG` to use a different
file. The container uses `docker/config.toml`.

Set `[storage].database_path` to the SQLite database path. If the parent
directory does not exist, the backend creates it.

## Index blocks through P2P

Set these values in the backend config:

```toml
[indexer]
source = "p2p"
global_config_path = "global.config.json"
poll_interval_ms = 1000

[indexer.p2p]
address = "164.132.76.12:19002"
data_dir = ".actonscan-p2p"
parallelism = 16
timeout_seconds = 30
from_latest = true
```

Replace `address` with your reachable IPv4 address and UDP port.
P2P uses ADNL UDP, DHT, and RLDP2 for all block downloads.
`parallelism` limits concurrent shard downloads to 1–128. Its default is 16.

Available since trunk: `from_latest = true` uses LiteServer to select a recent starting block ID for a new index and download directory.
It checks the server's network zerostate, then closes the LiteServer connections before downloading blocks through P2P.
Earlier history is skipped. The default is `false`, which uses the global config's initial block without LiteServer.

The default directory is `.actonscan-p2p`.
Stop `ton-sync sync` before the backend uses the same directory or UDP port.
The backend reuses masterchain and shard BOCs from that directory.

A new indexer starts after the configured anchor. After a restart, it resumes from its SQLite checkpoint.
An existing P2P checkpoint keeps its original anchor, even with `from_latest = true`.
If only the SQLite checkpoint exists, a new download directory starts from that exact ID and resumes its successor.
Neither checkpoint is replaced by a newer LiteServer tip.
`tps_backfill_batches` applies only to the LiteServer source, which remains the default.
The SQLite checkpoint advances only after a complete batch and its statistics are committed.
The CLI `batch-checkpoint.json` does not advance the backend checkpoint.

P2P has no global latest-tip query.
TPS reports `ready` when all windows are complete and the latest block time is within 60 seconds.
An unavailable shard leaves the indexer at its previous checkpoint until a peer supplies the block.

Downloads check block hashes and ancestry. The starting ID is trusted; the P2P source does not yet check validator signatures or execute state transitions.
No account-state dump is required for TPS or opcode statistics.

## Docker data

The Docker image declares `/var/lib/actonscan` as a volume. Without an explicit
mount, Docker creates an anonymous volume.

For deployments, mount a named volume at `/var/lib/actonscan`. When you replace
the container, use the same volume.
