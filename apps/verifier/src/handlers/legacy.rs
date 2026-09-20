use axum::{
    Json,
    http::{HeaderMap, StatusCode, header::USER_AGENT},
    response::IntoResponse,
};

const UNSUPPORTED_BLUEPRINT_ERROR: &str =
    "This verifier is no longer supported. Update @ton/blueprint to version 0.46.0 or newer";
const UNSUPPORTED_ACTON_ERROR: &str =
    "This version of acton verify is no longer supported. Update Acton to version 1.2.0 or newer";

pub async fn source_handler(headers: HeaderMap) -> impl IntoResponse {
    let is_acton = headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("acton/"));

    let message = if is_acton {
        UNSUPPORTED_ACTON_ERROR
    } else {
        UNSUPPORTED_BLUEPRINT_ERROR
    };

    (StatusCode::GONE, Json(message))
}
