//! Serialize TON Center API requests and decode their responses.
//!
//! Use [`v2::requests`] with your HTTP client and decode replies as [`v2::Response`].
//! Enable `openapi` to generate an API document with request and response schemas.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod v2;
