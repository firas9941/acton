use super::*;

#[test]
fn wallet_dtos_accept_signed_int64_identifiers() {
    let information: V2WalletInformation = serde_json::from_value(serde_json::json!({
        "balance": "1",
        "seqno": -1,
        "wallet_id": 4_294_967_295_i64,
        "last_transaction_lt": "0",
        "last_transaction_hash": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "status": "active"
    }))
    .expect("v3 wallet information uses upstream int64 fields");

    assert_eq!(information.seqno, Some(-1));
    assert_eq!(information.wallet_id, Some(4_294_967_295));

    let state: WalletState = serde_json::from_value(serde_json::json!({
        "address": "0:0000000000000000000000000000000000000000000000000000000000000000",
        "is_wallet": true,
        "seqno": -1,
        "wallet_id": 4_294_967_295_i64
    }))
    .expect("v3 wallet state uses upstream int64 fields");

    assert_eq!(state.seqno, Some(-1));
    assert_eq!(state.wallet_id, Some(4_294_967_295));
}

#[test]
fn trace_accepts_additional_openapi_fields() {
    let trace: Trace = serde_json::from_value(serde_json::json!({
        "trace_id": "trace",
        "transactions_order": [],
        "transactions": {},
        "is_incomplete": false,
        "mc_seqno_start": "1",
        "mc_seqno_end": "1",
        "start_lt": "1",
        "start_utime": 1,
        "trace_info": {
            "transactions": 0,
            "messages": 0,
            "pending_messages": 0,
            "trace_state": "complete",
            "classification_state": "unclassified"
        },
        "warning": "upstream field not consumed by this response projection"
    }))
    .expect("v3 trace response must accept the full upstream envelope");

    assert_eq!(trace.trace_id, "trace");
}

#[test]
fn traces_response_accepts_null_from_pending_traces() {
    let response: TracesResponse = serde_json::from_value(serde_json::json!({
        "traces": null,
        "address_book": {},
        "metadata": {}
    }))
    .expect("pendingTraces returns null instead of an empty OpenAPI array");

    assert!(response.traces.is_empty());
}

#[test]
fn account_state_accepts_null_contract_methods() {
    let state: AccountStateFull = serde_json::from_value(serde_json::json!({
        "address": "0:B68DE1A8AC80058762D7C2C0E31DF44AF864A8F54CDC85D204321DB803B4FB00",
        "account_state_hash": "X5b9nGGVrdlr+FYN0HoaNZfLHEBNOBlT91T3Zmmc0b0=",
        "balance": "6049945",
        "extra_currencies": {},
        "status": "active",
        "contract_methods": null,
        "interfaces": ["jetton_wallet_v2"]
    }))
    .expect("accountStates returns null for unknown contract methods");

    assert!(state.contract_methods.is_empty());
}
