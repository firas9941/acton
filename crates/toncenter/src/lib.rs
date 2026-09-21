//! Serialize TON Center API requests and decode their responses.
//!
//! Use [`v2::requests`] with your HTTP client and decode replies as [`v2::Response`].
//! Use [`v3::requests`] and [`v3::responses`] for supported indexed API operations.
//! Enable `openapi` to generate v2 and v3 API documents with request and response schemas.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "openapi")]
mod openapi_support;

pub mod v2;
pub mod v3;
