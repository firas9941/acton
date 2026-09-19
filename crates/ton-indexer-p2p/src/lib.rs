//! Supplies complete indexing batches from a `ton-p2p` download client.
//!
//! `ton-indexer-core` traverses shard predecessors and handles splits and merges.
//! This adapter supplies cached or downloaded blocks to that traversal. Consumers
//! own the checkpoint for processed batches; the transport owns download progress.

mod source;
pub mod start;

pub use source::P2pBlockSource;
