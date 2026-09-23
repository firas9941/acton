#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Serialize;
use tokio::sync::RwLock;
use ton_node_db::{AccountSnapshot, StateStore};
use toncenter::v2::requests::AddressInformationRequest;
use toncenter::v2::{self as v2, responses as wire};
use tracing::{debug, error};
use tycho_types::boc::Boc;
use tycho_types::models::{AccountState, BlockId, StdAddr, StdAddrFormat};

#[derive(Clone)]
struct Api {
    store: Arc<RwLock<StateStore>>,
    zero_state: BlockId,
}

/// Routes read only committed states. A request holds the read lock through
/// account lookup so its block ID, timestamp, and account share one checkpoint.
pub(crate) fn router(store: Arc<RwLock<StateStore>>, zero_state: BlockId) -> Router {
    Router::new()
        .route("/api/v2/getMasterchainInfo", get(masterchain_info))
        .route("/api/v2/getAddressInformation", get(address_information))
        .route("/api/v2/getAddressBalance", get(address_balance))
        .fallback(|| async { ApiError::new(StatusCode::NOT_FOUND, "unknown API method") })
        .with_state(Api { store, zero_state })
}

async fn masterchain_info(State(api): State<Api>) -> Response {
    read(api, "getMasterchainInfo", |store, zero_state| {
        let state = store.masterchain_state()?;

        Ok(wire::MasterchainInfo {
            type_tag: Default::default(),
            last: block_id(state.block_id()),
            state_root_hash: STANDARD.encode(state.root_hash()),
            init: block_id(zero_state),
        })
    })
    .await
}

async fn address_information(
    State(api): State<Api>,
    query: Result<Query<AddressInformationRequest>, QueryRejection>,
) -> Response {
    account_request(api, query, "getAddressInformation", |info| info).await
}

async fn address_balance(
    State(api): State<Api>,
    query: Result<Query<AddressInformationRequest>, QueryRejection>,
) -> Response {
    account_request(api, query, "getAddressBalance", |info| info.balance).await
}

async fn account_request<T: Serialize + Send + 'static>(
    api: Api,
    query: Result<Query<AddressInformationRequest>, QueryRejection>,
    method: &'static str,
    map: impl FnOnce(wire::AddressInformation) -> T + Send + 'static,
) -> Response {
    let Ok(Query(query)) = query else {
        return ApiError::new(StatusCode::BAD_REQUEST, "invalid query parameters").into_response();
    };
    let Ok((address, _)) = StdAddr::from_str_ext(&query.address, StdAddrFormat::any()) else {
        return ApiError::new(StatusCode::BAD_REQUEST, "invalid account address").into_response();
    };

    read(api, method, move |store, _| {
        if let Some(seqno) = query.seqno {
            let seqno = match seqno {
                v2::Int32Input::Number(value) => Some(value),
                v2::Int32Input::String(value) => value.parse::<i32>().ok(),
            }
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "invalid masterchain seqno"))?;

            if seqno != store.head().seqno {
                return Err(ApiError::new(
                    StatusCode::CONFLICT,
                    "only the current applied masterchain seqno is available",
                )
                .into());
            }
        }

        let utime = store.masterchain_state()?.gen_utime()?;
        let snapshot = store.get_account(&address)?;
        Ok(map(account_info(snapshot, utime)?))
    })
    .await
}

/// Maps the raw account without interpreting its contract code. Missing
/// dictionary entries have the same zero-balance representation as `account_none`.
fn account_info(snapshot: AccountSnapshot, utime: u32) -> Result<wire::AddressInformation> {
    let mut info = wire::AddressInformation {
        type_tag: Default::default(),
        balance: "0".into(),
        extra_currencies: Vec::new(),
        last_transaction_id: wire::InternalTransactionId {
            type_tag: Default::default(),
            lt: "0".into(),
            hash: STANDARD.encode([0_u8; 32]),
        },
        block_id: block_id(snapshot.masterchain_block),
        code: String::new(),
        data: String::new(),
        frozen_hash: String::new(),
        sync_utime: i64::from(utime),
        state: wire::AccountStateEnum::Uninitialized,
        suspended: None,
    };
    let Some(shard_account) = snapshot.account else {
        return Ok(info);
    };
    info.last_transaction_id.lt = shard_account.last_trans_lt.to_string();
    info.last_transaction_id.hash = STANDARD.encode(shard_account.last_trans_hash);

    let Some(account) = shard_account.load_account()? else {
        return Ok(info);
    };
    info.balance = account.balance.tokens.to_string();
    for currency in account.balance.other.as_dict().iter() {
        let (id, amount) = currency?;
        info.extra_currencies.push(wire::ExtraCurrencyBalance {
            type_tag: Default::default(),
            id: id as i32,
            amount: amount.to_string(),
        });
    }

    match account.state {
        AccountState::Uninit => {}
        AccountState::Active(state) => {
            info.state = wire::AccountStateEnum::Active;
            info.code = state.code.map(Boc::encode_base64).unwrap_or_default();
            info.data = state.data.map(Boc::encode_base64).unwrap_or_default();
        }
        AccountState::Frozen(hash) => {
            info.state = wire::AccountStateEnum::Frozen;
            info.frozen_hash = STANDARD.encode(hash);
        }
    }

    Ok(info)
}

fn block_id(id: BlockId) -> wire::TonBlockIdExt {
    wire::TonBlockIdExt {
        type_tag: Default::default(),
        workchain: i64::from(id.shard.workchain()),
        shard: (id.shard.prefix() as i64).to_string(),
        seqno: i64::from(id.seqno),
        root_hash: STANDARD.encode(id.root_hash),
        file_hash: STANDARD.encode(id.file_hash),
    }
}

async fn read<T: Serialize + Send + 'static>(
    api: Api,
    method: &'static str,
    query: impl FnOnce(&StateStore, BlockId) -> Result<T> + Send + 'static,
) -> Response {
    let started = Instant::now();
    let result =
        tokio::task::spawn_blocking(move || query(&api.store.blocking_read(), api.zero_state))
            .await
            .map_err(anyhow::Error::from)
            .and_then(std::convert::identity);

    match result {
        Ok(result) => {
            debug!(
                operation = "http_query",
                target = method,
                duration_ms = started.elapsed().as_millis(),
                outcome = "success",
                "read the applied state",
            );
            Json(v2::TonlibResponse {
                ok: true,
                result,
                extra: String::new(),
                jsonrpc: None,
                id: None,
            })
            .into_response()
        }
        Err(error) => {
            if let Some(error) = error.downcast_ref::<ApiError>() {
                return error.clone().into_response();
            }

            error!(
                operation = "http_query",
                target = method,
                duration_ms = started.elapsed().as_millis(),
                outcome = "failed",
                error = %format!("{error:#}"),
                "could not read the applied state",
            );
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "state query failed; see service logs",
            )
            .into_response()
        }
    }
}

#[derive(Debug, Clone)]
struct ApiError {
    status: StatusCode,
    message: &'static str,
}

impl ApiError {
    const fn new(status: StatusCode, message: &'static str) -> Self {
        Self { status, message }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message)
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(v2::TonlibErrorResponse {
                ok: false,
                error: self.message.into(),
                code: i32::from(self.status.as_u16()),
                extra: None,
                jsonrpc: None,
                id: None,
            }),
        )
            .into_response()
    }
}
