# Synchronize TON state and query it over HTTP

`ton-state` starts from a validator database snapshot, downloads successor blocks
through P2P, and stores masterchain and shard states. Its HTTP API reads the last
fully applied checkpoint.

## Run with Localton

Use an extracted database from a stopped validator or a consistent backup.
The global config and snapshot must belong to the same network.
Keep the snapshot unchanged and store updates in a separate directory.

Start the Localton network that produced your snapshot:

```sh
cargo run --release --manifest-path apps/localton/Cargo.toml -- \
  bootstrap --state-dir /path/to/localton-network
```

In another terminal, run the service from the repository root:

```sh
cargo run --release -p ton-state -- \
  /path/to/snapshot /path/to/localton-network/global.config.json .ton-state
```

The default HTTP endpoint is `http://127.0.0.1:8080`.
The service advertises `127.0.0.1:19005` for P2P UDP traffic.
Use `--http` or `--address` to change these endpoints.
The advertised UDP address must be reachable from peers.

Stop the service with `Ctrl-C`. Run the same command to resume from its saved
checkpoint. `.ton-state/states` stores applied updates; `.ton-state/blocks` stores
downloaded blocks. The original snapshot remains necessary after restart.

## Query the applied state

Read the masterchain checkpoint:

```sh
curl -s http://127.0.0.1:8080/api/v2/getMasterchainInfo | jq
```

Read an account. This example uses the standard elector address:

```sh
curl -sG http://127.0.0.1:8080/api/v2/getAddressInformation \
  --data-urlencode 'address=-1:3333333333333333333333333333333333333333333333333333333333333333' \
  | jq
```

Read its balance in nanograms. One GRAM equals 1,000,000,000 nanograms:

```sh
curl -sG http://127.0.0.1:8080/api/v2/getAddressBalance \
  --data-urlencode 'address=-1:3333333333333333333333333333333333333333333333333333333333333333' \
  | jq
```

Account methods accept raw and user-friendly addresses. An absent account has
zero balance and `uninitialized` state. Account information includes code and
data as base64 BoCs, the last transaction, and the masterchain checkpoint.
`sync_utime` contains the checkpoint's chain time.

These GET routes use the [TON Center API v2](https://toncenter.com/api/v2/)
response format. Success responses contain `ok: true` and `result`.
Errors contain `ok: false`, `error`, and `code`.
An optional `seqno` must equal the current applied checkpoint. Other heights
return HTTP 409. Historical queries, POST requests, and JSON-RPC are not supported.

The reported checkpoint can lag behind the network head. Network errors cause
download retries. An invalid state update or a storage error stops the service.

## Verification and storage limits

The service checks block hashes, predecessor links, Merkle updates, and complete
shard frontiers. It trusts the snapshot and does not verify validator signatures
or re-execute transactions. New workchains require zerostates and are not supported.

Stored cells and downloaded blocks accumulate on disk. The service has no
garbage collection or authentication. The default HTTP listener accepts only
local connections.
