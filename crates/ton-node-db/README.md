# Query and synchronize TON state from a validator snapshot

`ton-node-db` reads an extracted validator-engine database without running a node.
The directory must come from a stopped node or a consistent backup.
The reader opens RocksDB in read-only mode and includes records from existing WAL files.
`StateStore` saves subsequent state updates in a separate writable database.

## Database layout

| Path | Format | Contents |
| --- | --- | --- |
| `state/` | RocksDB, TL values | Validator progress, garbage-collection checkpoint, database version, and other service records |
| `celldb/` | RocksDB, binary cell records and TL descriptors | Shared cell graph and retained state roots |
| `archive/packages/**/*.pack` | TON package | Block BoCs, proofs, and proof links |
| `archive/packages/**/*.index/` | RocksDB | Block metadata, file offsets, and logical-time indexes |
| `files/packages/` | Temporary packages and RocksDB indexes | Recent files that can also occur in the archive |
| `files/globalindex/` | RocksDB | Registry of archive, key-block, and temporary packages |
| `archive/states/` | State BoCs | Zerostates and persistent states retained by the node |
| `static/` | Files named by file hash | Static data, including zerostates |

The compressed dump is a tar archive inside an lzip stream.
It contains a database directory, not one concatenated blockchain file.

### Packages

A package starts with little-endian magic `0xae8fdd01`.
Each entry contains a `u16` magic (`0x1e8b`), a `u16` filename length, and a `u32` payload length.
The UTF-8 filename and payload immediately follow this header.
Offsets in archive indexes exclude the initial four-byte package magic.

Names such as `block_(workchain,shard,seqno):root_hash:file_hash` identify complete blocks.
Proofs and proof links use their own filename prefixes.
The package has no compression layer. Block payloads are BoCs.

### Cells and states

A 32-byte RocksDB key identifies a cell by its representation hash.
Its value starts with a little-endian signed reference count.
The usual representation then stores two cell descriptor bytes, cell data, and references to child cells.
Each reference stores a level mask, significant hashes, and big-endian depths.

A reference count marker of `-1` selects the other representation: a reference count followed by a complete BoC.
RocksDB merge records contain signed reference-count changes.
The reader implements TON's `MergeOperatorAddCellRefcnt` to resolve them.

Metadata keys beginning with `desc` associate block IDs with state root hashes.
Their `prev` and `next` fields form a garbage-collection list, not the blockchain.
The `desczero` entry is the list sentinel.

A state root decodes as `ShardStateUnsplit`.
It contains account dictionaries, balances, message queues, libraries, and time references.
The masterchain state also contains blockchain configuration and references to current shard blocks.
An account contains its balance, storage statistics, and active, frozen, or uninitialized state.
An active account's `StateInit` contains code, data, and libraries.

Different states share unchanged cells.
The node can delete old state roots while retaining their blocks in packages.
Retained blocks therefore do not imply that every historical state can be read directly.

## Inspect and export

Extract the dump into an empty directory:

```sh
mkdir database
lzip -dc localton-db.tar.lz | tar -xf - -C database
```

Run the inspector from the repository root:

```sh
cargo run -p ton-node-db --example inspect -- database --export output
```

The JSON report lists databases, package entries, unique block counts, checkpoints, states, and accounts.
The inspector reconstructs the shard-client checkpoint and the shard states referenced by that masterchain state.
If the database has no shard-client checkpoint, it uses the validator's initialization checkpoint.

The export directory receives block, state, complete account, and active-account `StateInit` BoCs.
These account and `StateInit` files are different objects.
The inspector preserves the original bytes when it exports a block.
It serializes reconstructed state and account cells into new BoCs with the same representation hashes.

Select a specific retained state with `--state 'workchain:shard:seqno:root_hash:file_hash'`.
Extract a specific block with `--block 'workchain:shard:seqno:root_hash:file_hash' --export output`.
The `--max-cells` option bounds database cell records loaded for each state. The default is one million.

## Reader scope

The library exposes `NodeDb`, retained `StateRecord` values, and a streaming `PackageReader`.
It checks reconstructed cell hashes and reference depths.
Block reads check the file hash, root hash, and block header.
The inspector also matches each reconstructed state root against its block's Merkle update.
These checks do not verify validator signatures or execute state transitions.

Package lookup currently scans entry headers.
RocksDB indexes contribute record counts to the report, but do not yet accelerate block lookup.
Full-state exports reconstruct the selected state in memory.
Account queries use lazy cell reads and do not scan the whole database.
Split persistent-state files are listed but are not assembled into states.

## Read an account by address

Available since trunk.

```sh
cargo run -p ton-node-db --example account -- database -- \
  '-1:3333333333333333333333333333333333333333333333333333333333333333'
```

Use the actual address for your network. Several addresses can follow the database path.
Put options before the final `--`, which allows masterchain addresses that start with `-1:`.
The example returns account fields, code/data BoCs, exact block IDs, read counts, and elapsed time.
Pass `--block` to select a retained masterchain block and `--max-cells` to bound database reads per query.

`NodeDb::get_account(masterchain, address, max_cells)` selects the shard from that masterchain state's frontier.
It reads the dictionary path and the selected account's cells.
The returned `ShardAccount` owns its cells, including code, data, and libraries.
An absent account returns `None`. A missing shard state or cell returns an error.

`NodeDb::state` opens a reusable `StateView` for one shard.
It verifies each fetched cell against its database key and parent reference metadata.
Hashes and depths of unloaded branches come from the stored reference metadata.
The read budget covers the view's lifetime. A cell read failure makes further queries fail.

Cell lookups share a 32 MiB LRU record cache per open snapshot. It fills on demand
and is separate from each view's cell read budget.

## Read the elector after Localton block updates

Available since trunk.

Start the Localton network that produced the snapshot. Keep the extracted snapshot directory unchanged.
The example connects to that network through P2P, starting from the snapshot's masterchain checkpoint:

```sh
RUST_LOG=info cargo run -p ton-node-db --example elector -- database \
  --global-config global.config.json \
  --data-dir .elector-p2p \
  --address 127.0.0.1:19005 \
  --blocks 10
```

The UDP address must be reachable from the Localton node.
Use a separate writable directory for the P2P cache.
The example reads config parameter 1 for the elector address and applies ten successor blocks in memory.
It returns the resulting account, before/after account hashes, and database read counts.
The reported block identifies the queried state; `--blocks` does not select the current network head.

`StateView::apply_masterchain_block` checks block hashes, the predecessor, and the Merkle update.
Unchanged branches stay backed by the snapshot. Changed branches live in memory.
The method does not verify consensus signatures, execute transactions, or persist new states.

## Persist updates and resume

For continuous synchronization and HTTP account queries, run
[`ton-state`](../ton-state/README.md).

Keep the snapshot unchanged and use a separate directory for updates. The local
network that produced the snapshot must be running. This example downloads and
applies ten complete masterchain/shard batches, then reads the elector:

```sh
RUST_LOG=info cargo run -p ton-node-db --example sync -- \
  database global.config.json .ton-state
```

Run the same command again to apply the next ten batches. It opens the last
committed state, without replaying earlier updates. The example advertises
`127.0.0.1:19005` for a Localton node on the same computer.

`.ton-state/states` contains new cells and the applied state checkpoint.
`.ton-state/blocks` contains the separate P2P download cache. A downloaded block
does not advance the applied state checkpoint by itself.

`StateStore::apply_batch` checks block hashes, predecessor links, Merkle updates,
and the complete shard frontier, including splits and merges. New cells and the
checkpoint are committed in one synchronous RocksDB batch. Missing or invalid shard blocks leave the
previous checkpoint intact. Repeating the committed masterchain ID is a no-op.
After a storage write error, reopen the store before retrying.

Pass shard blocks in predecessor-first order. A split uses the same parent
state for both children. A merge combines the left and right states before
applying the block's Merkle update. Their sequence numbers can differ.

`StateStore::get_account` reads an account at the committed masterchain frontier.
Unchanged cells stay in the original snapshot and are loaded lazily. Updated
cells are read from the writable database. Each query or batch has its own cell
read budget, and completed batches release their in-memory state graphs.

Use `StateStore::snapshot()` to read one committed frontier while the writer
applies further blocks. Cloning this `StateSnapshot` shares database handles and
state roots without copying cells. All its queries use the same masterchain and
shard checkpoint. Capture a new snapshot after a successful commit to read newer
data. Old snapshots remain readable after the writer is dropped; drop them before
reopening the update directory, because they keep its database handle open.

The snapshot must remain available after restart. The writable database is an
append-only update store; it is not a replacement validator-engine database.
It has no cell garbage collection, validator signature verification, or TVM
transaction execution. Starting a new workchain requires its zerostate and is
not supported by this update store.

The storage layouts follow TON's
[package implementation](https://github.com/ton-blockchain/ton/blob/master/validator/db/package.cpp),
[cell storage implementation](https://github.com/ton-blockchain/ton/blob/master/crypto/vm/db/CellStorage.cpp),
and [database TL schema](https://github.com/ton-blockchain/ton/blob/master/tl/generate/scheme/ton_api.tl).
