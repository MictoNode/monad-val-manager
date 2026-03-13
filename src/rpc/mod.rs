//! RPC module - Monad node JSON-RPC client

mod client;
mod methods;

pub use client::{ConsensusInfo, NodeInfo, RpcClient, ServiceState, ServicesStatus, SyncStatus};
pub use methods::{BlockInfo, HealthCheck, NodeInfo as RpcNodeInfo, PeerInfo};
