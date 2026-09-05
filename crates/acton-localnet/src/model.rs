//! Persisted network definitions and the shared CLI/HTTP response contract.

use serde::{Deserialize, Serialize};

/// Immutable genesis settings and host ports.
///
/// Account `BoCs` are explicit copies of `ShardAccount` state; importing them creates
/// a new chain rather than a live fork.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateNetwork {
    pub name: String,
    pub port_base: Option<u16>,
    pub block_time_ms: Option<u32>,
    pub election_time_seconds: Option<u32>,
    #[serde(default)]
    pub imported_account_bocs: Vec<String>,
}

/// Selected ports and genesis inputs belong to this deployment for its lifetime.
/// Docker image/context identity is pinned separately in the runtime descriptor.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkConfig {
    pub port_base: u16,
    pub block_time_ms: Option<u32>,
    pub election_time_seconds: Option<u32>,
    pub imported_account_bocs: Vec<String>,
}

/// Host endpoints remain assigned while an environment is stopped. Availability
/// follows `Network::status`, not the presence of a URL in this structure.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
    pub api_v2: String,
    pub api_v3: String,
    pub admin: String,
    pub config: String,
    pub observability: String,
}

impl NetworkConfig {
    pub(crate) fn endpoints(&self) -> Endpoints {
        let port = self.port_base;
        Endpoints {
            config: format!("http://127.0.0.1:{port}"),
            admin: format!("http://127.0.0.1:{}", port + 1),
            api_v2: format!("http://127.0.0.1:{}/api/v2", port + 2),
            api_v3: format!("http://127.0.0.1:{}/api/v3", port + 3),
            observability: format!("http://127.0.0.1:{}", port + 4),
        }
    }
}

/// An additional host-managed node. The genesis owner is never removable through
/// this collection. Validator entry and exit follow actual TON election rounds.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub name: String,
    pub validator: bool,
    pub port_base: u16,
}

/// Observed lifecycle state. `Unknown` means Docker state has not been confirmed.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
    Unknown,
    Deleted,
}

/// A network's durable definition plus the latest observable runtime state.
/// The service is its sole writer; clients do not edit this record on disk.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Network {
    pub id: String,
    pub name: String,
    pub config: NetworkConfig,
    pub endpoints: Endpoints,
    pub nodes: Vec<Node>,
    pub state: Option<NetworkState>,
    pub status: Status,
    pub operation: Option<Operation>,
    pub error: Option<String>,
}

/// Blockchain files live in a Docker volume, separate from service metadata.
/// The directory is inside the container; it is not a path on the CLI host.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkState {
    pub directory: String,
    pub volume: String,
}

/// Operation progress survives client disconnects. A service restart marks active
/// operations interrupted and reconciles Docker instead of replaying mutations.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub id: String,
    pub kind: String,
    pub phase: String,
    pub status: OperationStatus,
    pub started_at: u64,
    pub duration_ms: u64,
    pub progress: Option<OperationProgress>,
    #[serde(default)]
    pub completed_steps: Vec<OperationStep>,
    pub error: Option<String>,
    pub log_path: String,
    pub result: Option<serde_json::Value>,
}

/// Completed phases remain visible even when they finish between client polls.
/// Durations are measured by the owning service, independently of client latency.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationStep {
    pub phase: String,
    pub duration_ms: u64,
}

/// Measured work within the current phase, shared by terminal and HTTP clients.
///
/// `total` is absent when Docker has not announced the complete workload. Clients
/// must then display the observed count without inventing a completion percentage.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationProgress {
    pub completed: u64,
    pub total: Option<u64>,
    pub unit: String,
    pub detail: String,
}

/// Terminal states are durable; closing a polling client never cancels `Running`.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OperationStatus {
    Running,
    Completed,
    Failed,
}

/// Archive metadata returned by Localton's snapshot implementation. Restoring an
/// archive requires a stopped network and rebuilding the derived indexer data.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub format_version: u32,
    pub id: String,
    pub name: Option<String>,
    pub created_at: u64,
    pub archive_size_bytes: u64,
    pub state_size_bytes: u64,
    pub state_schema_version: u32,
    pub ton_release: String,
    pub masterchain_seqno: Option<u32>,
}
