use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/status",
    operation_id = "service_status",
    responses(
        (status = 200, description = "Current verifier service status", body = ServiceStatusResponse)
    ),
    tag = "system"
)]
pub async fn handler(State(state): State<AppState>) -> Json<ServiceStatusResponse> {
    Json(ServiceStatusResponse {
        read_only: state.read_only(),
    })
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub(super) struct ServiceStatusResponse {
    read_only: bool,
}
