# TON Center API types for Rust

Serialize requests and decode TON Center API v2 and v3 responses with typed Rust models.
Use the request and response types with your HTTP client, or generate an OpenAPI
document for API tooling.

## Query an account

This example uses `reqwest` to fetch an account's balance and state:

```toml
[dependencies]
toncenter = "2.1.15"
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
```

```rust
use toncenter::v2::Response;
use toncenter::v2::requests::AddressInformationRequest;
use toncenter::v2::responses::AddressInformation;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let request = AddressInformationRequest {
        address: "EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c".to_owned(),
        seqno: None,
    };

    let response: Response<AddressInformation> = client
        .post("https://toncenter.com/api/v2/getAddressInformation")
        .json(&request)
        .send()?
        .json()?;
    let account = response.into_result()?;

    println!("Balance: {} nanograms", account.balance);
    println!("State: {:?}", account.state);
    Ok(())
}
```

For authenticated requests, add `.header("X-API-Key", api_key)` to the request.
Use `https://testnet.toncenter.com/api/v2` for testnet.

`Response<T>::into_result()` returns the method result or a `TonlibErrorResponse`
with the server's error code and message. Decode the response body even when its
HTTP status indicates an error to retain these details.

## Work with v2 wire values

- Balances and TVM integers remain strings, preserving values larger than 64 bits.
- Native coin balances and fees use nanograms: 1 GRAM = 1,000,000,000 nanograms.
- Addresses, hashes, and BoCs retain the encoding returned by the server.
- `Int32Input`, `Int64Input`, and `BoolInput` accept the scalar forms used by v2 requests.
- Optional fields are omitted during serialization. `@type` tags and legacy stack
  pair lengths are checked during deserialization.
- Use `stack::TvmStackEntry` for `runGetMethodStd` and `stack::LegacyStackEntry` for
  `runGetMethod`. Both preserve nested tuples and lists.

The `v2::endpoints::Endpoint` trait associates each method with its request and
successful result. `Response<T>` accepts both successful and failed TONLib envelopes;
`into_result()` exposes the result or a structured error. A successful broadcast
response does not prove on-chain inclusion. The C++ `/jsonRPC` proxy usually omits
`jsonrpc` and `id` from its responses; `JsonRpcCall` pairs each method with its params.

All 36 methods in the deployed OpenAPI are available, together with `/jsonRPC`,
`shards`, and `sendBocReturnHashNoError`. The supported C++ server accepts
`dnsResolve.category` but ignores it; `ttl` controls resolution depth, not cache expiry.

## Query indexed data with v3

Use `v3::requests` to select indexed accounts, transactions, blocks, traces, jettons,
NFTs, DNS records, multisig orders, and vesting contracts. The module also supports
fee estimation, get methods, and message submission: 35 operations in total.
It covers a subset of TON Center v3 schema version `1.2.6`.

```rust
use toncenter::v3::{endpoints::{Endpoint, GetMasterchainInfo}, requests::MasterchainInfoQuery};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let response: <GetMasterchainInfo as Endpoint>::Response = client
        .get(format!("https://toncenter.com{}", GetMasterchainInfo::PATH))
        .query(&MasterchainInfoQuery::default())
        .send()?
        .error_for_status()?
        .json()?;

    println!("Latest indexed masterchain block: {}", response.last.seqno);
    Ok(())
}
```

V3 returns the result directly. API errors decode as `v3::responses::RequestError`;
HTTP gateways can also return unstructured errors. For GET requests, omit null
parameters and encode arrays as repeated query parameters, such as
`address=first&address=second`. JSON serialization preserves each model's null and
empty-collection behavior, so a transport must encode URL queries separately.

`v3::StringOrNumber` preserves string and integer representations. It accepts
signed and unsigned 64-bit JSON integers and keeps larger numbers as strings.
Unknown response fields are ignored; the supported v3 models do not preserve every
field provided by newer servers.

## Generate OpenAPI

Enable the `openapi` feature:

```toml
toncenter = { version = "2.1.15", features = ["openapi"] }
```

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let document = toncenter::v2::openapi::document();
    std::fs::write("openapi.json", document.to_pretty_json()?)?;
    Ok(())
}
```

From a checkout of this repository, export or check the bundled document with:

```sh
cargo run -p toncenter --example openapi --features openapi -- crates/toncenter/openapi.json
cargo run -p toncenter --example openapi --features openapi -- --check crates/toncenter/openapi.json
cargo run -p toncenter --example openapi --features openapi -- --v3 crates/toncenter/openapi-v3.json
cargo run -p toncenter --example openapi --features openapi -- --v3 --check crates/toncenter/openapi-v3.json
```

The bundled [v2 document](openapi.json) and [v3 document](openapi-v3.json) include field descriptions, query
parameters, authentication options, and method-specific success and error schemas.
Use `toncenter::v3::openapi::document()` to generate the v3 document in Rust.

## Test the contract

Offline tests round-trip examples for every request and result, including optional
fields, all discriminated unions, nested stacks, and malformed inputs:

```sh
cargo test -p toncenter
cargo test -p toncenter --features openapi
```

Real network tests are compiled only with the `live-tests` feature:

```sh
cargo test -p toncenter --features live-tests --test live -- --test-threads=1 --nocapture
cargo test -p toncenter --features live-tests --test live_v3 -- --test-threads=1 --nocapture
```

The v2 suite tests each method over POST and the JSON-RPC proxy, and GET where supported.
It validates actual replies against both the Rust models and generated OpenAPI and
checks that serialization loses no fields or values. Requests are paced, time-limited,
and retried for rate limiting and temporary gateway failures. Network or API failures
fail the tests; missing credentials do not silently skip checks.

The v3 suite checks typed queries and responses for the supported operations,
including repeated parameters, nullable collections, nested stacks, and API errors.
It decodes the supported response fields; additional server fields are accepted.

Configuration:

| Environment variable | Meaning |
| --- | --- |
| `TONCENTER_V2_URL` | Base URL including `/api/v2`; defaults to mainnet TON Center |
| `TONCENTER_V3_URL` | Base URL including `/api/v3`; defaults to mainnet TON Center |
| `TONCENTER_API_KEY` | Optional key sent in `X-API-Key` |
| `TONCENTER_TOKEN_ADDRESS` | Jetton or NFT contract for `getTokenData` |
| `TONCENTER_DNS_ADDRESS` | Root resolver for `dnsResolve` |
| `TONCENTER_RECORD_DIR` | Optional directory for public request/response samples; keys are excluded |

The default fixtures target mainnet. Configure token and resolver addresses for
other networks. Block and transaction fixtures are discovered from recent blocks.
Broadcast tests submit only an invalid BoC and verify error envelopes. Successful
broadcast results and rare historical contract variants are covered by offline
fixtures; the suite does not sign or broadcast real messages.

## Reference and licensing

Server reference:
[ton-http-api-cpp at ed6e2ee](https://github.com/toncenter/ton-http-api-cpp/tree/ed6e2eefd9784cb292af8c58243ef2f9f48b1026),
deployed API `v2.1.15-8bacaa3`.

V3 schema reference: [TON Center v3](https://toncenter.com/api/v3/doc.json),
version `1.2.6`.

Type and field documentation includes material adapted from TON Center's MIT-licensed
schema; its notice is in [LICENSE-TON-HTTP-API](LICENSE-TON-HTTP-API). This crate is
licensed under [MIT](LICENSE-MIT).
