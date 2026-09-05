//! Standalone management of real TON development networks.
//!
//! The HTTP service owns state and serializes mutations. CLI clients may disconnect
//! without cancelling accepted operations. Studio intentionally retains its own runtime.

pub mod catalog;
pub mod client;
mod docker;
mod error;
pub mod http;
pub mod inspection;
mod model;
mod runtime;
mod storage;

pub use error::Error;
pub use model::{
    CreateNetwork, Endpoints, Network, NetworkConfig, NetworkState, Node, Operation,
    OperationProgress, OperationStatus, OperationStep, Snapshot, Status,
};
pub use runtime::Runtime;
pub use storage::{ServiceDescriptor, service_descriptor_path};
