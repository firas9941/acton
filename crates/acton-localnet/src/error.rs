//! Errors retain a machine-readable code without exposing request bodies.

/// A failed request or runtime operation. HTTP adapters choose the status code;
/// runtime callers retain the original operation and filesystem context.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{message}")]
    InvalidRequest { code: &'static str, message: String },
    #[error("{message}")]
    Conflict { code: &'static str, message: String },
    #[error("Network {environment_id} was not found")]
    NotFound { environment_id: String },
    #[error("{message}")]
    Internal { code: &'static str, message: String },
}

impl Error {
    pub(crate) fn storage(path: &std::path::Path, error: impl std::fmt::Display) -> Self {
        Self::Internal {
            code: "storage_failed",
            message: format!("Failed to access {}: {error}", path.display()),
        }
    }

    pub(crate) fn invalid(message: impl Into<String>) -> Self {
        Self::InvalidRequest {
            code: "invalid_request",
            message: message.into(),
        }
    }

    pub(crate) fn busy() -> Self {
        Self::Conflict {
            code: "operation_in_progress",
            message: "Another operation is running for this network".to_owned(),
        }
    }
}
