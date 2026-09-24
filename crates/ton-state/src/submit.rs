//! Admission and P2P submission of signed external messages.

#[cfg(test)]
mod tests;

use std::{sync::Arc, time::Instant};

use axum::extract::rejection::JsonRejection;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use base64::{Engine, engine::general_purpose::STANDARD};
use tokio::sync::Semaphore;
use ton_p2p::{ExternalMessage, MessageSender};
use toncenter::v2::{TonlibResponse, requests::SendBocRequest, responses::ResultOk};
use tracing::warn;

use crate::api::ApiError;

#[derive(Clone)]
struct Submission {
    sender: MessageSender,
    capacity: Arc<Semaphore>,
}

/// Submission shares the synchronization transport. Bounded admission keeps
/// message decoding and discovery from exhausting the HTTP worker pool.
pub(crate) fn router(sender: MessageSender) -> Router {
    Router::new()
        .route("/api/v2/sendBoc", post(send_boc))
        .layer(DefaultBodyLimit::max(96 * 1024))
        .with_state(Submission {
            sender,
            capacity: Arc::new(Semaphore::new(16)),
        })
}

/// Submit an external message
///
/// Broadcast a signed inbound external message through P2P. Accepts a base64 `BoC`
/// up to 65,535 decoded bytes for a standard masterchain or basechain destination
///
/// Only the envelope is checked; the message is not emulated. Success means
/// queued for broadcast, not accepted or included in a block
#[utoipa::path(
    post,
    path = "/api/v2/sendBoc",
    operation_id = "sendBoc",
    request_body = SendBocRequest,
    responses(
        (status = 200, description = "P2P broadcast queued", body = TonlibResponse<ResultOk>),
        (status = 400, description = "Invalid JSON, base64 or message", body = toncenter::v2::TonlibErrorResponse),
        (status = 413, description = "BoC exceeds 65,535 bytes or JSON exceeds 96 KiB", body = toncenter::v2::TonlibErrorResponse),
        (status = 415, description = "Expected application/json", body = toncenter::v2::TonlibErrorResponse),
        (status = 422, description = "Invalid request fields", body = toncenter::v2::TonlibErrorResponse),
        (status = 429, description = "Submission queue is full", body = toncenter::v2::TonlibErrorResponse),
        (status = 500, description = "Message decoding failed", body = toncenter::v2::TonlibErrorResponse),
        (status = 503, description = "P2P submission failed; retry later", body = toncenter::v2::TonlibErrorResponse),
    ),
)]
async fn send_boc(
    State(submission): State<Submission>,
    request: Result<Json<SendBocRequest>, JsonRejection>,
) -> Response {
    let started = Instant::now();
    let request = match request {
        Ok(Json(request)) => request,
        Err(error) => {
            return ApiError::new(error.status(), "expected JSON with a base64 boc field")
                .into_response();
        }
    };
    let Ok(permit) = submission.capacity.try_acquire_owned() else {
        return ApiError::new(StatusCode::TOO_MANY_REQUESTS, "too many pending messages")
            .into_response();
    };

    // Hold the permit inside the worker so cancellation cannot free its slot
    // while decoding is still running.
    let parsed = tokio::task::spawn_blocking(move || {
        let message = parse_message(request);
        (permit, message)
    })
    .await;
    let (_permit, message) = match parsed {
        Ok((permit, Ok(message))) => (permit, message),
        Ok((_, Err(error))) => return error.into_response(),
        Err(_) => {
            return ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "message decoding failed")
                .into_response();
        }
    };

    let hash = message.hash();
    if let Err(error) = submission.sender.send(message).await {
        warn!(
            operation = "http_message_submission",
            target = %hash,
            duration_ms = started.elapsed().as_millis(),
            outcome = "failed",
            error = %format!("{error:#}"),
            "could not submit external message through P2P",
        );

        return ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "P2P submission failed; retry later",
        )
        .into_response();
    }

    Json(TonlibResponse {
        ok: true,
        result: ResultOk {
            type_tag: Default::default(),
        },
        extra: String::new(),
        jsonrpc: None,
        id: None,
    })
    .into_response()
}

fn parse_message(request: SendBocRequest) -> Result<ExternalMessage, ApiError> {
    let boc = STANDARD
        .decode(request.boc)
        .map_err(|_| ApiError::new(StatusCode::BAD_REQUEST, "boc must contain valid base64"))?;
    if boc.len() > ExternalMessage::MAX_BYTES {
        return Err(ApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "boc exceeds 65535 bytes",
        ));
    }

    ExternalMessage::new(boc)
        .map_err(|_| ApiError::new(StatusCode::BAD_REQUEST, "invalid inbound external message"))
}
