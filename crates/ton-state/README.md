# Synchronize TON state and query it over HTTP

`ton-state` starts from a validator database snapshot, downloads successor blocks
through P2P, and stores masterchain and shard states. Its HTTP API reads the last
fully applied checkpoint.
Each HTTP request keeps that checkpoint for its entire read. Synchronization can
commit newer blocks without making the request wait for state application.

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
Open [`/docs`](http://127.0.0.1:8080/docs) to browse all supported methods and send
requests, including live SSE subscriptions. The page uses Scalar and loads its
pinned JavaScript bundle from jsDelivr. API requests go directly to this service.
The generated OpenAPI document is at [`/openapi.json`](http://127.0.0.1:8080/openapi.json).

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
`sync_utime` contains the account's shard-state time, as in TONLib.
`suspended` is currently always `false`; account suspension is not evaluated.

These GET routes use the [TON Center API v2](https://toncenter.com/api/v2/)
response format. Success responses contain `ok: true` and `result`.
Errors contain `ok: false`, `error`, and `code`.
An optional `seqno` must equal the current applied checkpoint. Other heights
return HTTP 409. These account routes do not support historical queries, POST,
or JSON-RPC.

The reported checkpoint can lag behind the network head. Network errors cause
download retries. An invalid state update or a storage error stops the service.

## Send a signed message

Submit a signed inbound external message saved as `message.boc`:

```sh
base64 < message.boc | tr -d '\n' | jq -Rs '{boc: .}' | \
  curl -s http://127.0.0.1:8080/api/v2/sendBoc \
    -H 'Content-Type: application/json' --data-binary @-
```

A successful response is `{"ok":true,"result":{"@type":"ok"},"@extra":""}`.
It means the service queued a P2P broadcast, not that a validator accepted the
message or included it in a block. Check the resulting transaction separately,
for example through an SSE subscription opened before submission.

The service checks the message envelope and accepts BoCs up to 65,535 bytes
with a standard masterchain or basechain destination. It does not emulate the
message or validate its signature, expiry, balance, or wallet sequence number.
Receiving peers enforce network admission rules. Larger messages use FEC
broadcasts. Peers can discard repeated messages.

Invalid input returns an HTTP error in the v2 response format. HTTP 429 means
the submission queue is full; HTTP 503 means P2P submission failed and the
request can be retried. This route accepts POST JSON only.

## Subscribe to finalized transactions

Open a live SSE subscription on the same HTTP listener:

```sh
curl -N http://127.0.0.1:8080/api/streaming/v2/sse \
  -H 'Content-Type: application/json' \
  -d '{"types":["transactions"],"addresses":["-1:3333333333333333333333333333333333333333333333333333333333333333"],"min_finality":"finalized"}'
```

The first SSE data message is `{"status":"subscribed"}`. Subsequent messages
contain `type: "transaction"`, `finality: "finalized"`, and one `transaction`
object with TON Center v3 fields. The service sends `: keepalive` comments after
15 seconds without an event. Ignore comment lines in the client.

Subscriptions accept 1–100 raw or user-friendly addresses. An address matches
the transaction's account, not its message destinations. Repeated forms of the
same address produce one event. `types` defaults to `["transactions"]` and
`min_finality` defaults to `"finalized"`. Other event types, finality levels,
and subscription fields return HTTP 400.

Events come from the existing P2P block download. They are published only after
the masterchain block and all associated shard state updates commit. The
transaction includes its shard `block_ref` and the committing `mc_block_seqno`.
During catch-up, events follow the service's applied checkpoint and can be older
than the network head. Keepalive confirms the connection, not sync progress.

The transaction object includes messages, execution phases, fees in nanograms,
and account state hashes. Account balances, code, data, normalized message hashes,
and trace links are not resolved. Their optional fields remain null or absent;
`child_transactions` is empty and does not imply that no child transactions exist.
The envelope is specific to this service: each event contains one transaction,
without TON Center's trace grouping. See the reference
[SSE subscription](https://docs.ton.org/api/streaming/sse) and
[notification schemas](https://docs.ton.org/api/streaming/reference).

Delivery is live only. Events published before subscription, during disconnection,
or across process restarts are not replayed. `Last-Event-ID` returns HTTP 400.
Clients must treat a disconnect as a possible gap and reconcile separately if
they need a complete history.

At most 64 connections are accepted. Each has a queue limited to 32 events and
2 MiB of serialized data. Queue overflow sends
`{"type":"error","error":"slow_consumer"}` and ends that connection.
An encoding failure or an event larger than 1 MiB ends current subscriptions
with `{"type":"error","error":"stream_failed"}`. State synchronization
continues. Reconnect creates a new live subscription and does not recover the gap.

## Verification and storage limits

The service checks block hashes, predecessor links, Merkle updates, and complete
shard frontiers. It trusts the snapshot and does not verify validator signatures
or re-execute transactions. New workchains require zerostates and are not supported.

Stored cells and downloaded blocks accumulate on disk. The service has no
garbage collection or authentication. The default HTTP listener accepts only
local connections.
