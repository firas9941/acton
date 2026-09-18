use axum::{Json, http::StatusCode, response::IntoResponse};

const UNSUPPORTED_ERROR: &str =
    "This verifier is no longer supported. Update @ton/blueprint to version 0.46.0 or newer";

pub async fn source_handler() -> impl IntoResponse {
    (StatusCode::GONE, Json(UNSUPPORTED_ERROR))
}
