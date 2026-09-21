use super::*;
use axum::{Json, Router, routing::post};
use expect_test::expect;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn client_uses_v2_envelopes_and_retries_api_errors() {
    let requests = Arc::new(Mutex::new(Vec::<Value>::new()));
    let captured = Arc::clone(&requests);
    let router = Router::new().route(
        "/api/v2/jsonRPC",
        post(move |Json(request): Json<Value>| {
            let mut requests = captured.lock().unwrap();
            requests.push(request.clone());
            let response = match request["method"].as_str().unwrap() {
                "runGetMethod" => json!({
                    "ok": true,
                    "@extra": "test",
                    "result": {
                        "@type": "smc.runResult",
                        "gas_used": 17,
                        "exit_code": 0,
                        "stack": [["num", "0x2a"]],
                        "block_id": {
                            "@type": "ton.blockIdExt", "workchain": -1,
                            "shard": "-9223372036854775808", "seqno": 1,
                            "root_hash": "root", "file_hash": "file"
                        },
                        "last_transaction_id": {
                            "@type": "internal.transactionId", "lt": "0", "hash": "hash"
                        }
                    }
                }),
                "getAddressBalance" if requests.len() == 2 => json!({
                    "ok": false, "error": "temporary rate limit", "code": 429
                }),
                "getAddressBalance" => json!({
                    "ok": true, "@extra": "test", "result": "1000000000"
                }),
                "sendBoc" => json!({
                    "ok": false, "error": "invalid BoC", "code": 400
                }),
                _ => panic!("unexpected method"),
            };
            async move { Json(response) }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let client = ToncenterClient::new(&ToncenterConfig {
        api_key: None,
        url: format!("http://{address}"),
        timeout_seconds: 5,
        connect_timeout_seconds: 5,
        max_retries: 1,
        retry_base_delay_ms: 1,
    })
    .unwrap();

    let seqno = client.get_wallet_seqno("wallet").await.unwrap();
    let balance = client.get_address_balance("wallet").await.unwrap();
    let error = client.send_boc("invalid").await.unwrap_err();
    server.abort();

    let summary = json!({
        "seqno": seqno,
        "balance": balance,
        "error": error.to_string(),
        "requests": &*requests.lock().unwrap(),
    });
    expect![[r#"
        {
          "balance": 1000000000,
          "error": "TON Center JSON-RPC error for sendBoc: TON Center error 400: invalid BoC",
          "requests": [
            {
              "id": "1",
              "jsonrpc": "2.0",
              "method": "runGetMethod",
              "params": {
                "address": "wallet",
                "method": "seqno",
                "stack": []
              }
            },
            {
              "id": "1",
              "jsonrpc": "2.0",
              "method": "getAddressBalance",
              "params": {
                "address": "wallet"
              }
            },
            {
              "id": "1",
              "jsonrpc": "2.0",
              "method": "getAddressBalance",
              "params": {
                "address": "wallet"
              }
            },
            {
              "id": "1",
              "jsonrpc": "2.0",
              "method": "sendBoc",
              "params": {
                "boc": "invalid"
              }
            }
          ],
          "seqno": 42
        }"#]]
    .assert_eq(&serde_json::to_string_pretty(&summary).unwrap());
}
