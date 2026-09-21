//! Opt-in contract checks against a real TON Center v2 server.
//!
//! Requests are paced across test threads. No test creates a signature or sends a
//! valid external message. Set `TONCENTER_V2_URL` and optionally `TONCENTER_API_KEY`.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use expect_test::expect;
use reqwest::blocking::Client;
use serde_json::{Value, json};
use toncenter::v2::{Response, endpoints::*, requests};

const ELECTOR: &str = "-1:3333333333333333333333333333333333333333333333333333333333333333";
const JETTON: &str = "-1:A6FDDC65A0DC39BECD63FB75987D47B6991B168886729B0F7285340C409087FC";
const ZERO_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const EMPTY_CELL: &str = "te6ccgEBAQEAAgAAAA==";

static LAST_REQUEST: Mutex<Option<Instant>> = Mutex::new(None);
static HTTP: OnceLock<Client> = OnceLock::new();
static FIXTURE: OnceLock<Fixture> = OnceLock::new();
static DOCUMENT: OnceLock<Value> = OnceLock::new();

struct Fixture {
    masterchain: Value,
    shard: Value,
    address: String,
    message: Value,
    dns_root: String,
}

#[derive(Debug, Clone, Copy)]
enum Transport {
    Get,
    Post,
    Rpc,
}

fn send(method: &str, params: &Value, transport: Transport) -> Value {
    let base = std::env::var("TONCENTER_V2_URL")
        .unwrap_or_else(|_| "https://toncenter.com/api/v2".to_owned());
    let client = HTTP.get_or_init(|| {
        Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(35))
            .build()
            .expect("HTTP client")
    });
    // All test threads share the public API request budget.
    for attempt in 0..4 {
        let mut last = LAST_REQUEST.lock().expect("live request pacing mutex");
        if let Some(previous) = *last {
            std::thread::sleep(Duration::from_millis(1250).saturating_sub(previous.elapsed()));
        }
        let started = Instant::now();
        let path = if matches!(transport, Transport::Rpc) {
            "jsonRPC"
        } else {
            method
        };
        let url = format!("{}/{path}", base.trim_end_matches('/'));
        let mut request = match transport {
            Transport::Get => {
                let mut pairs = Vec::new();
                for (name, value) in params.as_object().expect("object request") {
                    let values = match value {
                        Value::Array(values) => values.clone(),
                        _ => vec![value.clone()],
                    };
                    for value in values {
                        let value = value
                            .as_str()
                            .map_or_else(|| value.to_string(), str::to_owned);
                        pairs.push((name.clone(), value));
                    }
                }
                client.get(url).query(&pairs)
            }
            Transport::Post => client.post(url).json(params),
            Transport::Rpc => {
                let rpc =
                    json!({"jsonrpc":"2.0", "id":"contract-test", "method":method,"params":params});
                let typed: requests::JsonRpcRequest =
                    serde_json::from_value(rpc).expect("typed RPC request");
                client.post(url).json(&typed)
            }
        };
        if let Ok(key) = std::env::var("TONCENTER_API_KEY") {
            request = request.header("X-API-Key", key);
        }
        let response = request.send().unwrap_or_else(|error| {
            panic!(
                "operation={method} transport={transport:?} request failed: {}",
                error.without_url()
            )
        });
        *last = Some(Instant::now());
        drop(last);
        let status = response.status();
        if (status.as_u16() == 429 || status.as_u16() == 502 || status.as_u16() == 503)
            && attempt < 3
        {
            eprintln!(
                "operation={method} transport={transport:?} duration_ms={} outcome=retry status={status}",
                started.elapsed().as_millis()
            );
            std::thread::sleep(Duration::from_secs(2_u64.pow(attempt + 1)));
            continue;
        }
        let raw: Value = response
            .json()
            .unwrap_or_else(|_| panic!("operation={method} status={status} non-JSON response"));
        eprintln!(
            "operation={method} transport={transport:?} target={} duration_ms={} outcome={} status={status}",
            reqwest::Url::parse(&base)
                .expect("base URL")
                .host_str()
                .unwrap_or("custom"),
            started.elapsed().as_millis(),
            if raw["ok"] == true {
                "success"
            } else {
                "api_error"
            }
        );

        if let Ok(directory) = std::env::var("TONCENTER_RECORD_DIR") {
            let path =
                std::path::Path::new(&directory).join(format!("{method}-{transport:?}.json"));
            std::fs::create_dir_all(&directory).expect("record directory");
            let record = json!({"request":params,"response":raw});
            std::fs::write(
                &path,
                serde_json::to_string_pretty(&record).expect("record JSON") + "\n",
            )
            .expect("write public contract sample");
        }
        return raw;
    }
    unreachable!("final attempt always returns or fails")
}

fn fixture() -> &'static Fixture {
    FIXTURE.get_or_init(|| {
        let masterchain =
            send("getMasterchainInfo", &json!({}), Transport::Get)["result"]["last"].clone();
        let shards = send(
            "getShards",
            &json!({"seqno":masterchain["seqno"]}),
            Transport::Get,
        );
        let shard = shards["result"]["shards"][0].clone();
        let mut block = block_params(&shard);
        block["count"] = json!(40);
        let transactions = send("getBlockTransactionsExt", &block, Transport::Post);
        let txs = transactions["result"]["transactions"]
            .as_array()
            .expect("nonempty recent shard transactions");
        let (address, message) = txs
            .iter()
            .find_map(|transaction| {
                let input = transaction.get("in_msg").into_iter();
                let output = transaction["out_msgs"].as_array().into_iter().flatten();
                input.chain(output).find_map(|message| {
                    let source = message["source"]["account_address"].as_str()?;
                    let destination = message["destination"]["account_address"].as_str()?;
                    if source.is_empty() || destination.is_empty() {
                        return None;
                    }

                    Some((
                        transaction["account"].as_str()?.to_owned(),
                        json!({
                            "source": source,
                            "destination": destination,
                            "created_lt": message["created_lt"],
                        }),
                    ))
                })
            })
            .expect("recent internal message for all three transaction locator endpoints");

        // The default root resolver is the address in mainnet config parameter 4.
        // Custom networks can supply their own resolver account.
        let dns_root = std::env::var("TONCENTER_DNS_ADDRESS").unwrap_or_else(|_| {
            "-1:E56754F83426F69B09267BD876AC97C44821345B7E266BD956A7BFBFB98DF35C".to_owned()
        });
        Fixture {
            masterchain,
            shard,
            address,
            message,
            dns_root,
        }
    })
}

fn block_params(block: &Value) -> Value {
    json!({"workchain":block["workchain"],"shard":block["shard"],"seqno":block["seqno"]})
}

fn parameters(method: &str) -> Value {
    match method {
        "unpackAddress" => json!({"address":"Ef8zMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzM0vF"}),
        "detectHash" => json!({"hash":ZERO_HASH}),
        "getMasterchainInfo" | "getConsensusBlock" | "getOutMsgQueueSize" | "getConfigAll" => {
            json!({})
        }
        "getLibraries" => {
            json!({"libraries":[ZERO_HASH, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="]})
        }
        "getConfigParam" => json!({"param":0}),
        "getTokenData" => {
            json!({"address":std::env::var("TONCENTER_TOKEN_ADDRESS").unwrap_or_else(|_| JETTON.to_owned())})
        }
        "detectAddress"
        | "packAddress"
        | "getAddressInformation"
        | "getExtendedAddressInformation"
        | "getShardAccountCell"
        | "getWalletInformation"
        | "getAddressBalance"
        | "getAddressState" => json!({"address":ELECTOR}),
        "getTransactions" | "getTransactionsStd" => {
            json!({"address":fixture().address,"limit":2,"archival":true})
        }
        "getShards" | "shards" | "getMasterchainBlockSignatures" => {
            json!({"seqno":fixture().masterchain["seqno"]})
        }
        "getShardBlockProof" => block_params(&fixture().shard),
        "getBlock" | "getBlockHeader" | "lookupBlock" => block_params(&fixture().masterchain),
        "getBlockTransactions" | "getBlockTransactionsExt" => {
            let mut params = block_params(&fixture().shard);
            params["count"] = json!(2);
            params
        }
        "tryLocateTx" | "tryLocateResultTx" | "tryLocateSourceTx" => fixture().message.clone(),
        "dnsResolve" => json!({"address":fixture().dns_root,"name":"foundation.ton","ttl":1}),
        "runGetMethod" | "runGetMethodStd" => {
            json!({"address":ELECTOR,"method":"participant_list_extended","stack":[]})
        }
        "sendBoc" | "sendBocReturnHash" | "sendBocReturnHashNoError" => {
            json!({"boc":"not-a-valid-boc"})
        }
        "estimateFee" => json!({"address":ELECTOR,"body":EMPTY_CELL,"ignore_chksig":true}),
        _ => panic!("missing request fixture for {method}"),
    }
}

fn differences(raw: &Value, typed: &Value, path: &str, errors: &mut Vec<String>) {
    if raw == typed {
        return;
    }
    match (raw, typed) {
        (Value::Object(left), Value::Object(right)) => {
            for key in left.keys().chain(right.keys()) {
                let field_path = format!("{path}/{key}");
                match (left.get(key), right.get(key)) {
                    (Some(left), Some(right)) => differences(left, right, &field_path, errors),
                    _ => errors.push(field_path),
                }
            }
        }
        (Value::Array(left), Value::Array(right)) if left.len() == right.len() => {
            for (i, (left, right)) in left.iter().zip(right).enumerate() {
                differences(left, right, &format!("{path}/{i}"), errors);
            }
        }
        _ => errors.push(path.to_owned()),
    }
}

fn verify<E: Endpoint>(params: Value, transport: Transport, success: bool)
where
    E::Response: std::fmt::Debug,
{
    let request: E::Request = serde_json::from_value(params).expect(E::METHOD);
    let request = serde_json::to_value(request).expect("request JSON");
    let raw = send(E::METHOD, &request, transport);
    let response: Response<E::Response> = serde_json::from_value(raw.clone())
        .unwrap_or_else(|error| panic!("{} response contract mismatch: {error}", E::METHOD));
    let typed = serde_json::to_value(&response).expect("response JSON");
    let mut changes = Vec::new();
    differences(&raw, &typed, "", &mut changes);
    expect![["[]\n"]].assert_debug_eq(&changes);

    let document = DOCUMENT
        .get_or_init(|| serde_json::to_value(toncenter::v2::openapi::document()).expect("OpenAPI"));
    let status = if success { "200" } else { "default" };
    let mut schema = document["paths"][E::PATH]["post"]["responses"][status]["content"]["application/json"]["schema"].clone();
    schema["components"] = document["components"].clone();
    let validator = jsonschema::validator_for(&schema).expect("response schema");
    let errors: Vec<String> = validator
        .iter_errors(&raw)
        .map(|error| error.instance_path().to_string())
        .collect();
    expect![["[]\n"]].assert_debug_eq(&errors);

    match (success, response) {
        (true, Response::Success(_)) | (false, Response::Error(_)) => {}
        (true, Response::Error(error)) => panic!(
            "{} expected success, received code {}: {}",
            E::METHOD,
            error.code,
            error.error
        ),
        (false, Response::Success(_)) => {
            panic!("{} accepted a deliberately invalid BoC", E::METHOD)
        }
    }
}

fn contract<E: Endpoint>()
where
    E::Response: std::fmt::Debug,
{
    let params = parameters(E::METHOD);
    let success = !E::METHOD.starts_with("sendBoc");
    if E::SUPPORTS_GET {
        verify::<E>(params.clone(), Transport::Get, success);
    }
    verify::<E>(params.clone(), Transport::Post, success);
    verify::<E>(params, Transport::Rpc, success);
}

macro_rules! live_tests {
    ($($name:ident => $endpoint:ty),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                contract::<$endpoint>();
            }
        )+

        #[test]
        fn every_registered_method_has_a_live_test() {
            let expected = METHODS
                .iter()
                .map(|name| ((*name).to_owned(), true))
                .collect::<BTreeMap<_, _>>();
            let actual = [$((<$endpoint>::METHOD.to_owned(), true)),+]
                .into_iter()
                .collect::<BTreeMap<_, _>>();
            let mut changes = Vec::new();
            differences(&json!(expected), &json!(actual), "", &mut changes);
            expect![["[]\n"]].assert_debug_eq(&changes);
        }
    };
}

live_tests! {
    detect_address => DetectAddress,
    detect_hash => DetectHash,
    pack_address => PackAddress,
    unpack_address => UnpackAddress,
    get_address_information => GetAddressInformation,
    get_extended_address_information => GetExtendedAddressInformation,
    get_shard_account_cell => GetShardAccountCell,
    get_wallet_information => GetWalletInformation,
    get_address_balance => GetAddressBalance,
    get_address_state => GetAddressState,
    get_token_data => GetTokenData,
    dns_resolve => DnsResolve,
    get_masterchain_info => GetMasterchainInfo,
    get_masterchain_block_signatures => GetMasterchainBlockSignatures,
    get_shard_block_proof => GetShardBlockProof,
    get_consensus_block => GetConsensusBlock,
    lookup_block => LookupBlock,
    get_shards => GetShards,
    get_block => GetBlock,
    get_block_header => GetBlockHeader,
    get_out_msg_queue_size => GetOutMsgQueueSize,
    get_block_transactions => GetBlockTransactions,
    get_block_transactions_ext => GetBlockTransactionsExt,
    get_transactions => GetTransactions,
    get_transactions_std => GetTransactionsStd,
    try_locate_tx => TryLocateTx,
    try_locate_result_tx => TryLocateResultTx,
    try_locate_source_tx => TryLocateSourceTx,
    get_config_param => GetConfigParam,
    get_config_all => GetConfigAll,
    get_libraries => GetLibraries,
    run_get_method => RunGetMethod,
    run_get_method_std => RunGetMethodStd,
    send_boc => SendBoc,
    send_boc_return_hash => SendBocReturnHash,
    estimate_fee => EstimateFee,
    shards => ShardsAlias,
    send_boc_return_hash_no_error => SendBocReturnHashNoError,
}
