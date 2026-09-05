//! Standalone management of real TON development networks.
//!
//! The HTTP service owns state and serializes mutations. CLI clients may disconnect
//! without cancelling accepted operations. Studio uses the same process and HTTP client as the CLI.

pub mod catalog;
pub mod client;
mod docker;
mod error;
pub mod http;
pub mod inspection;
mod model;
pub mod process;
mod runtime;
mod storage;

pub use error::Error;
pub use model::{
    CreateNetwork, Endpoints, Network, NetworkConfig, NetworkPorts, NetworkState, Node, Operation,
    OperationProgress, OperationStatus, OperationStep, PortOptions, Snapshot, StartupTimings,
    Status,
};
pub use runtime::Runtime;
pub use storage::{ServiceDescriptor, service_descriptor_path};
