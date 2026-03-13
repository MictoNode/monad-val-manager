//! CLI Bridge - Connect TUI to CLI handlers
//!
//! This module provides a bridge between TUI screens and CLI command handlers.
//! The TUI should NOT duplicate logic - instead, it should call the existing
//! CLI handlers and capture their output for display.

use crate::config::Config;
use crate::doctor::{Doctor, DoctorReport};
use crate::rpc::RpcClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// CLI Bridge - provides TUI access to CLI handler functionality
pub struct CliBridge {
    config: Config,
}

impl CliBridge {
    /// Create new CLI bridge
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Get node status data (for Dashboard screen)
    ///
    /// Corresponds to CLI: `monad-val-manager status`
    pub async fn get_status(&self) -> Result<StatusData> {
        let rpc = RpcClient::new(self.config.rpc_endpoint())?;

        // Check connection
        let connected = rpc.check_connection().await.unwrap_or(false);

        // Get block number
        let block_height = rpc.get_block_number().await.unwrap_or(0);

        // Get sync status
        let syncing = rpc.get_sync_status().await.unwrap_or(false);

        // Get peer count
        let peer_count = rpc.get_peer_count_prometheus().await.unwrap_or(0);

        // Get node info (version, uptime)
        let node_info = rpc.get_node_info_prometheus().await.unwrap_or_default();

        // Get consensus info (epoch, round, forkpoint)
        let consensus_info = rpc
            .get_consensus_info_prometheus()
            .await
            .unwrap_or_default();

        Ok(StatusData {
            connected,
            block_height,
            syncing,
            peer_count,
            version: node_info.version,
            uptime_seconds: node_info.uptime_seconds,
            epoch: consensus_info.epoch,
            round: consensus_info.round,
            forkpoint_epoch: consensus_info.forkpoint_epoch,
            forkpoint_round: consensus_info.forkpoint_round,
        })
    }

    /// Get account balance (for Account screen)
    ///
    /// Corresponds to CLI: `monad-val-manager balance [--address]`
    pub async fn get_balance(&self, address: &str) -> Result<BalanceData> {
        let rpc = RpcClient::new(self.config.rpc_endpoint())?;
        let balance_mon = rpc.get_balance(address).await?;
        // Convert MON (f64) back to wei (u128) for storage
        let balance_wei = (balance_mon * 1e18) as u128;
        Ok(BalanceData {
            address: address.to_string(),
            balance: balance_wei,
        })
    }

    /// Run doctor diagnostics (for Doctor screen)
    ///
    /// Corresponds to CLI: `monad-val-manager doctor`
    pub async fn run_doctor(&self) -> Result<DoctorReport> {
        let doctor = Doctor::new(&self.config);
        doctor.run_diagnostics().await
    }

    /// Get consensus info (for Dashboard)
    ///
    /// Returns epoch, round, and forkpoint from Prometheus metrics
    pub async fn get_consensus_info(&self) -> Result<ConsensusData> {
        let rpc = RpcClient::new(self.config.rpc_endpoint())?;
        let info = rpc.get_consensus_info_prometheus().await?;
        Ok(ConsensusData {
            epoch: info.epoch.unwrap_or(0),
            round: info.round.unwrap_or(0),
            forkpoint_epoch: info.forkpoint_epoch.unwrap_or(0),
            forkpoint_round: info.forkpoint_round.unwrap_or(0),
        })
    }

    /// Get node info (version, uptime)
    ///
    /// Returns from Prometheus metrics
    pub async fn get_node_info(&self) -> Result<NodeInfoData> {
        let rpc = RpcClient::new(self.config.rpc_endpoint())?;
        let info = rpc.get_node_info_prometheus().await?;
        Ok(NodeInfoData {
            version: info.version,
            uptime_seconds: info.uptime_seconds,
        })
    }

    /// Get systemd service status
    ///
    /// Returns status of monad-* services
    pub async fn get_services_status(&self) -> Result<ServicesStatusData> {
        let rpc = RpcClient::new(self.config.rpc_endpoint())?;
        let services = rpc.get_services_status().await.unwrap_or_default();
        Ok(ServicesStatusData {
            bft: services.bft.map(|s| s == crate::rpc::ServiceState::Running),
            execution: services
                .execution
                .map(|s| s == crate::rpc::ServiceState::Running),
            rpc: services.rpc.map(|s| s == crate::rpc::ServiceState::Running),
            archiver: services
                .archiver
                .map(|s| s == crate::rpc::ServiceState::Running),
            otelcol: services
                .otelcol
                .map(|s| s == crate::rpc::ServiceState::Running),
        })
    }

    /// Get MPT disk info from monad-mpt command
    ///
    /// Parses output of `monad-mpt --storage /dev/triedb`
    pub async fn get_mpt_disk_info(&self) -> Result<MptDiskData> {
        use std::process::Command;

        let output = Command::new("monad-mpt")
            .args(["--storage", "/dev/triedb"])
            .output();

        match output {
            Ok(output) => {
                let text = String::from_utf8_lossy(&output.stdout);
                // Parse output like:
                // "Storage: /dev/triedb, Capacity: 200 GB, Used: 180 GB (90%)"
                Self::parse_mpt_output(&text)
            }
            Err(_) => Ok(MptDiskData::default()),
        }
    }

    /// Parse monad-mpt output
    fn parse_mpt_output(text: &str) -> Result<MptDiskData> {
        // Parse capacity and usage from monad-mpt output
        let mut capacity_gb = 0.0;
        let mut used_gb = 0.0;

        for line in text.lines() {
            // Check for Capacity line
            if line.contains("Capacity:") {
                let parts: Vec<&str> = line
                    .split("Capacity:")
                    .nth(1)
                    .unwrap_or("")
                    .split_whitespace()
                    .collect();
                if parts.len() >= 2 {
                    if let Ok(val) = parts[0].parse::<f64>() {
                        capacity_gb = val;
                        // Convert TB to GB if needed
                        if parts[1].starts_with('T') || parts[1].starts_with('t') {
                            capacity_gb *= 1024.0;
                        }
                    }
                }
            }
            // Check for Used line
            if line.contains("Used:") {
                // Remove percentage if present
                let line_without_pct = line.split('(').next().unwrap_or(line);
                let parts: Vec<&str> = line_without_pct
                    .split("Used:")
                    .nth(1)
                    .unwrap_or("")
                    .split_whitespace()
                    .collect();
                if parts.len() >= 2 {
                    if let Ok(val) = parts[0].parse::<f64>() {
                        used_gb = val;
                        // Convert TB to GB if needed
                        if parts[1].starts_with('T') || parts[1].starts_with('t') {
                            used_gb *= 1024.0;
                        }
                    }
                }
            }
        }

        Ok(MptDiskData {
            used_gb: used_gb as u64,
            capacity_gb: capacity_gb as u64,
            usage_pct: if capacity_gb > 0.0 {
                (used_gb / capacity_gb * 100.0) as f32
            } else {
                0.0
            },
        })
    }
}

/// Status data from CLI status command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusData {
    pub connected: bool,
    pub block_height: u64,
    pub syncing: bool,
    pub peer_count: u64,
    pub version: Option<String>,
    pub uptime_seconds: Option<u64>,
    pub epoch: Option<u64>,
    pub round: Option<u64>,
    pub forkpoint_epoch: Option<u64>,
    pub forkpoint_round: Option<u64>,
}

/// Balance data from CLI balance command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceData {
    pub address: String,
    pub balance: u128, // Raw balance in wei
}

/// Consensus data from Prometheus metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusData {
    pub epoch: u64,
    pub round: u64,
    pub forkpoint_epoch: u64,
    pub forkpoint_round: u64,
}

/// Node info data from Prometheus metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfoData {
    pub version: Option<String>,
    pub uptime_seconds: Option<u64>,
}

/// Services status data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesStatusData {
    pub bft: Option<bool>,
    pub execution: Option<bool>,
    pub rpc: Option<bool>,
    pub archiver: Option<bool>,
    pub otelcol: Option<bool>,
}

/// MPT disk data from monad-mpt command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MptDiskData {
    pub used_gb: u64,
    pub capacity_gb: u64,
    pub usage_pct: f32,
}

impl Default for MptDiskData {
    fn default() -> Self {
        Self {
            used_gb: 0,
            capacity_gb: 0,
            usage_pct: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mpt_output() {
        let output = "Storage: /dev/triedb\nCapacity: 200 GB\nUsed: 180 GB (90%)";
        let result = CliBridge::parse_mpt_output(output).unwrap();
        assert_eq!(result.used_gb, 180);
        assert_eq!(result.capacity_gb, 200);
        assert!((result.usage_pct - 90.0).abs() < 0.1);
    }

    #[test]
    fn test_parse_mpt_output_tb() {
        let output = "Capacity: 1.8 T\nUsed: 180 GB";
        let result = CliBridge::parse_mpt_output(output).unwrap();
        assert_eq!(result.used_gb, 180);
        // 1.8 TB = 1.8 * 1024 GB = 1843.2 GB, so approximately 1843
        assert!((result.capacity_gb as f64 - 1843.0).abs() < 1.0);
    }

    #[test]
    fn test_parse_mpt_simple() {
        let output = "Capacity: 200 GB\nUsed: 100 GB";
        let result = CliBridge::parse_mpt_output(output).unwrap();
        assert_eq!(result.used_gb, 100);
        assert_eq!(result.capacity_gb, 200);
        assert!((result.usage_pct - 50.0).abs() < 0.1);
    }
}
