use axum::http::StatusCode;

use crate::LONG_VERSION;

pub(super) async fn ok() -> StatusCode {
    StatusCode::OK
}

pub(super) async fn version() -> &'static str {
    LONG_VERSION
}
