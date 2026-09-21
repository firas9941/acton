use crate::common::assertion;
use crate::support::localnet::pretty_json_for_snapshot;
use crate::support::toncenter::{
    extract_canonical_addr_marker, jetton_v1_action_project, run_localnet_action_project,
};
use base64::Engine as _;
use serde_json::json;
use toncenter::v2::{requests, responses};

const ZERO_HASH_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[test]
fn transaction_history_matches_upstream_cursor_and_boundary_semantics() {
    let project = jetton_v1_action_project("localnet-v2-transaction-history");
    let (node, output) = run_localnet_action_project(&project, "scripts/jetton.tolk");
    let account = extract_canonical_addr_marker(&output, "OWNER=");
    let all: toncenter::v2::TonlibResponse<Vec<responses::Transaction>> = node.get_json_as(
        &format!("/api/v2/getTransactions?address={account}&limit=100"),
    );
    assert!(
        all.result.len() >= 4,
        "fixture must produce at least four account transactions"
    );

    let cursor = &all.result[0].transaction_id;
    let cursor_lt = cursor.lt.parse::<u64>().expect("transaction LT must parse");
    let cursor_bytes = base64::engine::general_purpose::STANDARD
        .decode(&cursor.hash)
        .expect("transaction hash must be base64");
    let encodings = [
        ("base64", cursor.hash.clone()),
        ("hex", hex::encode(&cursor_bytes)),
        (
            "base64url",
            base64::engine::general_purpose::URL_SAFE.encode(&cursor_bytes),
        ),
        (
            "base64url_no_pad",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&cursor_bytes),
        ),
    ];
    let encoding_results = encodings
        .into_iter()
        .enumerate()
        .map(|(index, (encoding, hash))| {
            let response: toncenter::v2::TonlibResponse<Vec<responses::Transaction>> = node
                .post_v2_json_rpc(
                    "/api/v2",
                    (index as u64).to_string().into(),
                    "getTransactions",
                    transaction_request(
                        &account,
                        Some(cursor_lt),
                        Some(hash),
                        None,
                        Some(index.is_multiple_of(2)),
                    ),
                );
            json!({
                "encoding": encoding,
                "count": response.result.len(),
                "starts_at_cursor": response
                    .result
                    .first()
                    .is_some_and(|tx| tx.transaction_id.hash == cursor.hash),
            })
        })
        .collect::<Vec<_>>();

    let exact_rest: toncenter::v2::TonlibResponse<Vec<responses::Transaction>> =
        node.get_json_as(&format!(
            "/api/v2/getTransactions?address={account}&limit=2&lt={cursor_lt}&hash={}",
            hex::encode(&cursor_bytes)
        ));
    let boundary_lt = all.result[1]
        .transaction_id
        .lt
        .parse::<u64>()
        .expect("boundary LT must parse");
    let bounded: toncenter::v2::TonlibResponse<Vec<responses::Transaction>> = node.get_json_as(
        &format!("/api/v2/getTransactions?address={account}&limit=100&to_lt={boundary_lt}"),
    );
    let negative_bound: toncenter::v2::TonlibResponse<Vec<responses::Transaction>> = node
        .get_json_as(&format!(
            "/api/v2/getTransactions?address={account}&limit=100&to_lt=-1"
        ));

    let (invalid_rest_status, invalid_rest): (u16, toncenter::v2::TonlibErrorResponse) = node
        .get_json_with_status_as(&format!(
            "/api/v2/getTransactions?address={account}&limit=2&lt={cursor_lt}&hash={ZERO_HASH_HEX}"
        ));
    let (invalid_rpc_status, invalid_rpc): (u16, toncenter::v2::TonlibErrorResponse) = node
        .post_v2_json_rpc_with_status(
            "/api/v2",
            "invalid-cursor".into(),
            "getTransactions",
            transaction_request(
                &account,
                Some(cursor_lt),
                Some(ZERO_HASH_HEX.to_owned()),
                None,
                None,
            ),
        );

    let (decoded_zero_status, decoded_zero): (u16, toncenter::v2::TonlibErrorResponse) = node
        .get_json_with_status_as(&format!(
            "/api/v2/getTransactions?address={account}&limit=2&lt=0&hash={ZERO_HASH_HEX}"
        ));
    let std_zero: toncenter::v2::TonlibResponse<responses::TransactionsStd> = node.get_json_as(
        &format!("/api/v2/getTransactionsStd?address={account}&limit=2&lt=0&hash={ZERO_HASH_HEX}"),
    );
    let std_zero_rpc: toncenter::v2::TonlibResponse<responses::TransactionsStd> = node
        .post_v2_json_rpc(
            "/api/v2",
            "zero-cursor".into(),
            "getTransactionsStd",
            transaction_request(
                &account,
                Some(0),
                Some(ZERO_HASH_HEX.to_owned()),
                None,
                None,
            ),
        );
    let std_bounded: toncenter::v2::TonlibResponse<responses::TransactionsStd> = node.get_json_as(
        &format!("/api/v2/getTransactionsStd?address={account}&limit=3&to_lt={boundary_lt}"),
    );

    let snapshot = json!({
        "fixture_transaction_count": all.result.len(),
        "hash_encodings": encoding_results,
        "exact_rest": {
            "count": exact_rest.result.len(),
            "starts_at_cursor": exact_rest
                .result
                .first()
                .is_some_and(|tx| tx.transaction_id.hash == cursor.hash),
        },
        "exclusive_to_lt": {
            "count": bounded.result.len(),
            "all_above_boundary": all_lts_match(
                &bounded.result,
                |tx| &tx.transaction_id,
                |lt| lt > boundary_lt,
            ),
            "negative_bound_is_unbounded": transaction_hashes(&negative_bound.result)
                == transaction_hashes(&all.result),
        },
        "invalid_cursor": {
            "rest_status": invalid_rest_status,
            "rest_code": invalid_rest.code,
            "rest_error": invalid_rest.error,
            "rpc_status": invalid_rpc_status,
            "rpc_code": invalid_rpc.code,
            "rpc_error": invalid_rpc.error,
        },
        "zero_lt": {
            "decoded_status": decoded_zero_status,
            "decoded_code": decoded_zero.code,
            "std_rest_count": std_zero.result.transactions.len(),
            "std_rest_previous_is_zero": std_zero.result.previous_transaction_id.lt == "0",
            "std_rpc_count": std_zero_rpc.result.transactions.len(),
            "std_rpc_previous_is_zero": std_zero_rpc.result.previous_transaction_id.lt == "0",
        },
        "std_boundary": {
            "count": std_bounded.result.transactions.len(),
            "all_above_boundary": all_lts_match(
                &std_bounded.result.transactions,
                |tx| &tx.transaction_id,
                |lt| lt > boundary_lt,
            ),
            "previous_is_nonzero": std_bounded.result.previous_transaction_id.lt != "0",
        },
    });
    assertion().eq(
        pretty_json_for_snapshot(&snapshot, project.path()),
        snapbox::file!("snapshots/v2_transaction_history.json"),
    );

    node.stop();
}

fn transaction_request(
    address: &str,
    lt: Option<u64>,
    hash: Option<String>,
    to_lt: Option<i64>,
    archival: Option<bool>,
) -> requests::TransactionsRequest {
    requests::TransactionsRequest {
        address: address.to_owned(),
        limit: Some(2.into()),
        lt: lt.map(|lt| lt.to_string().into()),
        hash,
        to_lt: to_lt.map(Into::into),
        archival: archival.map(Into::into),
    }
}

fn all_lts_match<T>(
    transactions: &[T],
    transaction_id: impl Fn(&T) -> &responses::InternalTransactionId,
    predicate: impl Fn(u64) -> bool,
) -> bool {
    transactions
        .iter()
        .all(|tx| transaction_id(tx).lt.parse::<u64>().is_ok_and(&predicate))
}

fn transaction_hashes(transactions: &[responses::Transaction]) -> Vec<&str> {
    transactions
        .iter()
        .map(|tx| tx.transaction_id.hash.as_str())
        .collect()
}
