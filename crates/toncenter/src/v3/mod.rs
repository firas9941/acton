//! Requests and indexed blockchain data returned by TON Center API v3.
//!
//! Source schema: <https://toncenter.com/api/v3/doc.json>
//! These models cover selected operations from schema version `1.2.6`.
//! Addresses, hashes, and serialized cells retain their JSON representation.

pub mod endpoints;
pub mod requests;
pub mod responses;

mod wire;

#[cfg(feature = "openapi")]
pub mod openapi;

pub use requests::*;
pub use responses::*;
pub use wire::StringOrNumber;

/// Schema version represented by the supported request and response models.
pub const OPENAPI_VERSION: &str = "1.2.6";
/// Upstream schema URL; the published schema may include additional operations.
pub const OPENAPI_URL: &str = "https://toncenter.com/api/v3/doc.json";
