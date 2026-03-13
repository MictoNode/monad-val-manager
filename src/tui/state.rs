//! TUI Application State - Central state management for dashboard

use std::time::Instant;

use super::doctor_state::DoctorState;
use super::staking::{
    AddValidatorState, ChangeCommissionState, DelegateState, QueryDelegatorState,
    QueryValidatorState, StakingState, UndelegateState, WithdrawState,
};
use super::transfer_state::TransferDialogState;
use super::widgets::{ConfirmPopupState, InputDialogState};
use throbber_widgets_tui::ThrobberState;

/// System metrics data
#[derive(Debug, Clone, Default)]
pub struct SystemData {
    /// CPU usage percentage (0-100)
    pub cpu_usage: f32,
    /// Total memory in bytes
    pub total_memory: u64,
    /// Used memory in bytes
    pub used_memory: u64,
    /// Memory usage percentage (0-100)
    pub memory_usage: f64,
    /// Primary disk total space in bytes
    pub disk_total: u64,
    /// Primary disk available space in bytes
    pub disk_available: u64,
    /// Disk usage percentage (0-100)
    pub disk_usage: f64,
}

impl SystemData {
    /// Create new system data
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate memory usage percentage
    pub fn calculate_memory_usage(&self) -> f64 {
        if self.total_memory > 0 {
            (self.used_memory as f64 / self.total_memory as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate disk usage percentage
    pub fn calculate_disk_usage(&self) -> f64 {
        if self.disk_total > 0 {
            let used = self.disk_total.saturating_sub(self.disk_available);
            (used as f64 / self.disk_total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Format memory for display
    pub fn format_memory(&self) -> String {
        let used_gb = self.used_memory as f64 / (1024.0 * 1024.0 * 1024.0);
        let total_gb = self.total_memory as f64 / (1024.0 * 1024.0 * 1024.0);
        format!("{:.1} / {:.1} GB", used_gb, total_gb)
    }

    /// Format disk for display
    /// BUG-011 FIX: Use decimal format like memory for consistency
    pub fn format_disk(&self) -> String {
        let available_gb = self.disk_available as f64 / (1024.0 * 1024.0 * 1024.0);
        let total_gb = self.disk_total as f64 / (1024.0 * 1024.0 * 1024.0);
        format!("{:.1} / {:.1} GB", available_gb, total_gb)
    }
}

/// Network/Node metrics data
#[derive(Debug, Clone, Default)]
pub struct NetworkData {
    /// Current block number
    pub block_number: u64,
    /// Is the node syncing
    pub is_syncing: bool,
    /// Sync progress percentage (if syncing)
    pub sync_progress: Option<f64>,
    /// Number of connected peers
    pub peer_count: u64,
    /// Node version string
    pub node_version: Option<String>,
    /// Node uptime in seconds
    pub uptime_seconds: Option<u64>,
    /// Is the node responding to RPC
    pub is_connected: bool,
    /// Last error message (if any)
    pub last_error: Option<String>,
}

impl NetworkData {
    /// Create new network data
    pub fn new() -> Self {
        Self::default()
    }

    /// Format block number for display
    pub fn format_block_number(&self) -> String {
        format!("{}", self.block_number)
    }

    /// Format sync status for display
    pub fn format_sync_status(&self) -> String {
        if self.is_syncing {
            if let Some(progress) = self.sync_progress {
                format!("Syncing ({:.1}%)", progress)
            } else {
                "Syncing...".to_string()
            }
        } else {
            "Synced".to_string()
        }
    }

    /// Format uptime for display
    pub fn format_uptime(&self) -> String {
        match self.uptime_seconds {
            Some(seconds) => {
                let days = seconds / 86400;
                let hours = (seconds % 86400) / 3600;
                let minutes = (seconds % 3600) / 60;

                if days > 0 {
                    format!("{}d {}h {}m", days, hours, minutes)
                } else if hours > 0 {
                    format!("{}h {}m", hours, minutes)
                } else {
                    format!("{}m", minutes)
                }
            }
            None => "N/A".to_string(),
        }
    }
}

/// Consensus information data (NEW from monad-status.sh)
#[derive(Debug, Clone, Default)]
pub struct ConsensusData {
    /// Current epoch
    pub epoch: u64,
    /// Current round
    pub round: u64,
    /// Forkpoint epoch
    pub forkpoint_epoch: u64,
    /// Forkpoint round
    pub forkpoint_round: u64,
    /// Node uptime formatted string
    pub uptime: String,
}

impl ConsensusData {
    /// Create new consensus data
    pub fn new() -> Self {
        Self::default()
    }
}

/// Validator information data
#[derive(Debug, Clone)]
pub struct ValidatorData {
    /// Chain ID
    pub chain_id: u64,
    /// Network name (mainnet/testnet)
    pub network_name: String,
    /// RPC endpoint URL
    pub rpc_endpoint: String,
    /// Is this a validator node
    pub is_validator: bool,
}

impl Default for ValidatorData {
    fn default() -> Self {
        Self {
            chain_id: 143,
            network_name: "mainnet".to_string(),
            rpc_endpoint: "http://localhost:8080".to_string(),
            is_validator: false,
        }
    }
}

impl ValidatorData {
    /// Create new validator data
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from config
    pub fn from_config(chain_id: u64, network_name: &str, rpc_endpoint: &str) -> Self {
        Self {
            chain_id,
            network_name: network_name.to_string(),
            rpc_endpoint: rpc_endpoint.to_string(),
            is_validator: false,
        }
    }
}

/// Central application state for TUI
#[derive(Debug, Clone)]
pub struct AppState {
    /// System metrics
    pub system: SystemData,
    /// Network/Node metrics
    pub network: NetworkData,
    /// Validator information
    pub validator: ValidatorData,
    /// Consensus information (NEW)
    pub consensus: ConsensusData,
    /// Staking data and state
    pub staking: StakingState,
    /// Doctor diagnostics state
    pub doctor: DoctorState,
    /// Input dialog state for staking operations
    pub input_dialog: InputDialogState,
    /// Confirmation popup state for transaction confirmation
    pub confirm_popup: ConfirmPopupState,
    /// Add Validator dialog state
    pub add_validator: AddValidatorState,
    /// Change Commission dialog state
    pub change_commission: ChangeCommissionState,
    /// Query Delegator dialog state
    pub query_delegator: QueryDelegatorState,
    /// Query Validator dialog state
    pub query_validator: QueryValidatorState,
    /// Delegate dialog state
    pub delegate: DelegateState,
    /// Undelegate dialog state
    pub undelegate: UndelegateState,
    /// Withdraw dialog state
    pub withdraw: WithdrawState,
    /// Transfer dialog state
    pub transfer: TransferDialogState,
    /// Last refresh timestamp
    pub last_refresh: Option<Instant>,
    /// Number of refresh cycles
    pub refresh_count: u64,
    /// Is the application in loading state
    pub is_loading: bool,
    /// Tick counter for animations (incremented every frame)
    pub tick: u64,
    /// Throbber state for loading spinners
    pub throbber_state: ThrobberState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            system: SystemData::default(),
            network: NetworkData::default(),
            validator: ValidatorData::default(),
            consensus: ConsensusData::new(),
            staking: StakingState::new(),
            doctor: DoctorState::new(),
            input_dialog: InputDialogState::new(),
            confirm_popup: ConfirmPopupState::new(),
            add_validator: AddValidatorState::new(),
            change_commission: ChangeCommissionState::new(),
            query_delegator: QueryDelegatorState::new(),
            query_validator: QueryValidatorState::new(),
            delegate: DelegateState::new(),
            undelegate: UndelegateState::new(),
            withdraw: WithdrawState::new(),
            transfer: TransferDialogState::new(),
            last_refresh: None,
            refresh_count: 0,
            is_loading: true,
            tick: 0,
            throbber_state: ThrobberState::default(),
        }
    }
}

impl AppState {
    /// Create new application state
    pub fn new() -> Self {
        Self::default()
    }

    /// Create state with config values
    pub fn with_config(chain_id: u64, network_name: &str, rpc_endpoint: &str) -> Self {
        Self {
            validator: ValidatorData::from_config(chain_id, network_name, rpc_endpoint),
            ..Self::default()
        }
    }

    /// Mark refresh as complete
    pub fn mark_refreshed(&mut self) {
        self.last_refresh = Some(Instant::now());
        self.refresh_count += 1;
        self.is_loading = false;
    }

    /// Get time since last refresh
    pub fn time_since_refresh(&self) -> Option<std::time::Duration> {
        self.last_refresh.map(|t| t.elapsed())
    }

    /// Update system data
    pub fn update_system(&mut self, data: SystemData) {
        self.system = data;
    }

    /// Update network data
    pub fn update_network(&mut self, data: NetworkData) {
        self.network = data;
    }

    /// Update validator data
    pub fn update_validator(&mut self, data: ValidatorData) {
        self.validator = data;
    }

    /// Update staking data
    pub fn update_staking(&mut self, data: StakingState) {
        self.staking = data;
    }

    /// Update consensus data
    pub fn update_consensus(&mut self, data: ConsensusData) {
        self.consensus = data;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_data_creation() {
        let data = SystemData::new();
        assert_eq!(data.cpu_usage, 0.0);
        assert_eq!(data.total_memory, 0);
        assert_eq!(data.used_memory, 0);
    }

    #[test]
    fn test_system_data_memory_calculation() {
        let mut data = SystemData::new();
        data.total_memory = 1024 * 1024 * 1024 * 16; // 16 GB
        data.used_memory = 1024 * 1024 * 1024 * 8; // 8 GB

        let usage = data.calculate_memory_usage();
        assert!((usage - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_system_data_disk_calculation() {
        let mut data = SystemData::new();
        data.disk_total = 1024 * 1024 * 1024 * 1000; // 1 TB
        data.disk_available = 1024 * 1024 * 1024 * 250; // 250 GB

        let usage = data.calculate_disk_usage();
        assert!((usage - 75.0).abs() < 0.1);
    }

    #[test]
    fn test_system_data_format_memory() {
        let mut data = SystemData::new();
        data.used_memory = 1024 * 1024 * 1024 * 8; // 8 GB
        data.total_memory = 1024 * 1024 * 1024 * 16; // 16 GB

        let formatted = data.format_memory();
        assert!(formatted.contains("8.0"));
        assert!(formatted.contains("16.0"));
    }

    #[test]
    fn test_network_data_creation() {
        let data = NetworkData::new();
        assert_eq!(data.block_number, 0);
        assert!(!data.is_syncing);
        assert!(!data.is_connected);
    }

    #[test]
    fn test_network_data_format_sync_status_synced() {
        let data = NetworkData::new();
        assert_eq!(data.format_sync_status(), "Synced");
    }

    #[test]
    fn test_network_data_format_sync_status_syncing_with_progress() {
        let mut data = NetworkData::new();
        data.is_syncing = true;
        data.sync_progress = Some(75.5);

        assert_eq!(data.format_sync_status(), "Syncing (75.5%)");
    }

    #[test]
    fn test_network_data_format_uptime() {
        let mut data = NetworkData::new();
        data.uptime_seconds = Some(90061); // 1 day, 1 hour, 1 minute

        assert_eq!(data.format_uptime(), "1d 1h 1m");
    }

    #[test]
    fn test_network_data_format_uptime_hours() {
        let mut data = NetworkData::new();
        data.uptime_seconds = Some(3661); // 1 hour, 1 minute

        assert_eq!(data.format_uptime(), "1h 1m");
    }

    #[test]
    fn test_network_data_format_uptime_minutes_only() {
        let mut data = NetworkData::new();
        data.uptime_seconds = Some(120); // 2 minutes

        assert_eq!(data.format_uptime(), "2m");
    }

    #[test]
    fn test_validator_data_creation() {
        let data = ValidatorData::new();
        assert_eq!(data.chain_id, 143);
        assert_eq!(data.network_name, "mainnet");
    }

    #[test]
    fn test_validator_data_from_config() {
        let data = ValidatorData::from_config(10143, "testnet", "http://localhost:8080");
        assert_eq!(data.chain_id, 10143);
        assert_eq!(data.network_name, "testnet");
        assert_eq!(data.rpc_endpoint, "http://localhost:8080");
    }

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        assert!(state.is_loading);
        assert_eq!(state.refresh_count, 0);
        assert!(state.last_refresh.is_none());
    }

    #[test]
    fn test_app_state_with_config() {
        let state = AppState::with_config(10143, "testnet", "http://localhost:8080");
        assert_eq!(state.validator.chain_id, 10143);
        assert_eq!(state.validator.network_name, "testnet");
    }

    #[test]
    fn test_app_state_mark_refreshed() {
        let mut state = AppState::new();
        assert!(state.is_loading);

        state.mark_refreshed();

        assert!(!state.is_loading);
        assert_eq!(state.refresh_count, 1);
        assert!(state.last_refresh.is_some());
    }

    #[test]
    fn test_app_state_update_system() {
        let mut state = AppState::new();
        let mut system = SystemData::new();
        system.cpu_usage = 50.0;

        state.update_system(system);

        assert!((state.system.cpu_usage - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_app_state_update_network() {
        let mut state = AppState::new();
        let mut network = NetworkData::new();
        network.block_number = 12345;

        state.update_network(network);

        assert_eq!(state.network.block_number, 12345);
    }
}
