//! TON Center v2 requests, response envelopes, and `TONLib` objects.
//!
//! Supports TON Center HTTP API 2.1.15. Balances and TVM integers use strings
//! to preserve values that exceed native integer ranges. Addresses, hashes, and
//! `BoCs` retain their wire encoding; this crate does not validate their contents.

pub mod endpoints;
pub mod requests;
pub mod responses;
pub mod stack;
pub mod tags;

mod wire;

#[cfg(feature = "openapi")]
pub mod openapi;

pub use wire::{BoolInput, Int32Input, Int64Input, Response, TonlibErrorResponse, TonlibResponse};
