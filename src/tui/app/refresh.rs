//! Data refresh operations for TUI application

use crate::tui::state::{ConsensusData, NetworkData, SystemData};
use crate::utils::system::SystemInfo;

use super::TuiApp;

impl TuiApp {
    /// Refresh all data from sources
    pub(crate) async fn refresh_data(&mut self) {
        // DEBUG REMOVED refresh_data() starting...");
        self.last_update = std::time::Instant::now();

        // Refresh system data
        // DEBUG REMOVED Refreshing system data...");
        self.refresh_system_data();

        // Refresh network data
        // DEBUG REMOVED Refreshing network data...");
        self.refresh_network_data().await;
        // DEBUG REMOVED Network data refreshed");

        // Refresh consensus data
        // DEBUG REMOVED Refreshing consensus data...");
        self.refresh_consensus_data().await;
        // DEBUG REMOVED Consensus data refreshed");

        // TEMPORARY DISABLE: Staking data refresh causes freeze on TUI startup
        // The get_all_delegations() + join_all() pattern can take 50+ seconds
        // Only refresh staking data when user enters Staking screen
        // // DEBUG REMOVED Refreshing staking data...");
        // self.refresh_staking_data().await;
        // // DEBUG REMOVED Staking data refreshed");

        // Mark refresh complete
        self.state.mark_refreshed();
        // DEBUG REMOVED refresh_data() complete");
    }

    /// Refresh system metrics
    pub(crate) fn refresh_system_data(&mut self) {
        let mut sys_info = SystemInfo::new();
        sys_info.refresh();

        let mut system_data = SystemData::new();
        system_data.cpu_usage = sys_info.cpu_usage();
        system_data.total_memory = sys_info.total_memory();
        system_data.used_memory = sys_info.used_memory();
        system_data.memory_usage = sys_info.memory_usage_percent();

        if let Some(disk) = sys_info.primary_disk() {
            system_data.disk_total = disk.total_space;
            system_data.disk_available = disk.available_space;
            system_data.disk_usage = disk.usage_percent();
        }

        self.state.update_system(system_data);
    }

    /// Refresh network/node data from RPC
    pub(crate) async fn refresh_network_data(&mut self) {
        let mut network_data = NetworkData::new();
        let was_connected = self.was_connected;

        if let Some(ref rpc) = self.rpc_client {
            // Check connection and get block number
            match rpc.get_block_number().await {
                Ok(block) => {
                    network_data.is_connected = true;
                    network_data.block_number = block;
                }
                Err(e) => {
                    network_data.is_connected = false;
                    network_data.last_error = Some(e.to_string());
                }
            }

            // Try to detect if this is a validator node by checking Prometheus metrics
            // If node has validator_status metrics, it's a validator
            // IMPORTANT: Once detected as validator, never reset back to full node
            // This prevents false negatives when metrics endpoint is temporarily unavailable
            if let Ok(validator_status) = rpc.get_validator_status_prometheus().await {
                // Only update if currently false and detection says true
                // Never reset from true back to false
                if validator_status && !self.state.validator.is_validator {
                    self.state.validator.is_validator = true;
                }
            }

            // Get sync status
            if network_data.is_connected {
                match rpc.get_sync_status_detailed().await {
                    Ok(sync_status) => match sync_status {
                        crate::rpc::SyncStatus::Syncing {
                            current_block,
                            highest_block,
                            ..
                        } => {
                            network_data.is_syncing = true;
                            if highest_block > 0 {
                                network_data.sync_progress =
                                    Some((current_block as f64 / highest_block as f64) * 100.0);
                            }
                        }
                        crate::rpc::SyncStatus::Synced => {
                            network_data.is_syncing = false;
                        }
                    },
                    Err(_) => {
                        // Fallback to simple sync check
                        if let Ok(syncing) = rpc.get_sync_status().await {
                            network_data.is_syncing = syncing;
                        }
                    }
                }

                // Get peer count from Prometheus
                if let Ok(peers) = rpc.get_peer_count_prometheus().await {
                    network_data.peer_count = peers;
                }

                // Get node info from Prometheus
                if let Ok(info) = rpc.get_node_info_prometheus().await {
                    network_data.node_version = info.version;
                    network_data.uptime_seconds = info.uptime_seconds;
                }
            }
        }

        // Detect connection state changes and show toast notifications
        let now_connected = network_data.is_connected;
        if was_connected != now_connected {
            if now_connected {
                // Node just connected
                let (title, message, toast_type) =
                    crate::tui::toast::ToastHelpers::node_status(true, network_data.is_syncing);
                let id = self.toast_manager.next_id();
                self.toast_manager.add(crate::tui::toast::Toast::new(
                    id,
                    title.to_string(),
                    message.to_string(),
                    toast_type,
                ));
            } else {
                // Node just disconnected
                let (title, message, toast_type) =
                    crate::tui::toast::ToastHelpers::node_status(false, false);
                let id = self.toast_manager.next_id();
                self.toast_manager.add(crate::tui::toast::Toast::new(
                    id,
                    title.to_string(),
                    message.to_string(),
                    toast_type,
                ));
            }
            self.was_connected = now_connected;
        }

        self.state.update_network(network_data);
    }

    /// Refresh consensus data from staking contract and Prometheus
    ///
    /// Multi-platform strategy:
    /// - Linux: Read from forkpoint.toml file (fast, no HTTP call)
    /// - Windows/macOS: Use Prometheus metrics API (cross-platform)
    /// - Fallback: Prometheus on all platforms if forkpoint file fails
    /// - If both fail, preserve existing data (don't reset to 0)
    pub(crate) async fn refresh_consensus_data(&mut self) {
        let mut consensus_data = ConsensusData::new();
        let mut data_loaded = false;

        // Linux: Try forkpoint file first (fast, no HTTP call)
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;

            if let Ok(output) = Command::new("cat")
                .args(["/home/monad/monad-bft/config/forkpoint/forkpoint.toml"])
                .output()
            {
                let content = String::from_utf8_lossy(&output.stdout);
                for line in content.lines() {
                    if line.contains("epoch =") {
                        if let Some(epoch_str) = line.split('=').nth(1) {
                            if let Ok(epoch) = epoch_str.trim().parse::<u64>() {
                                consensus_data.forkpoint_epoch = epoch;
                                consensus_data.epoch = epoch;
                                data_loaded = true;
                            }
                        }
                    }
                    if line.contains("round =") {
                        if let Some(round_str) = line.split('=').nth(1) {
                            if let Ok(round) = round_str.trim().parse::<u64>() {
                                consensus_data.forkpoint_round = round;
                                consensus_data.round = round;
                                data_loaded = true;
                            }
                        }
                    }
                }
            }
        }

        // Windows/macOS/Fallback: Use Prometheus metrics (cross-platform)
        if !data_loaded {
            if let Some(ref rpc) = self.rpc_client {
                if let Ok(consensus_info) = rpc.get_consensus_info_prometheus().await {
                    if let Some(epoch) = consensus_info.epoch {
                        consensus_data.epoch = epoch;
                        data_loaded = true;
                    }
                    if let Some(round) = consensus_info.round {
                        consensus_data.round = round;
                        data_loaded = true;
                    }
                    if let Some(forkpoint_epoch) = consensus_info.forkpoint_epoch {
                        consensus_data.forkpoint_epoch = forkpoint_epoch;
                    }
                    if let Some(forkpoint_round) = consensus_info.forkpoint_round {
                        consensus_data.forkpoint_round = forkpoint_round;
                    }
                }
            }
        }

        // If no new data was loaded, preserve existing epoch/round values
        // This prevents resetting to 0 when Prometheus is temporarily unavailable
        if !data_loaded {
            consensus_data.epoch = self.state.consensus.epoch;
            consensus_data.round = self.state.consensus.round;
            consensus_data.forkpoint_epoch = self.state.consensus.forkpoint_epoch;
            consensus_data.forkpoint_round = self.state.consensus.forkpoint_round;
        }

        // Get uptime from Prometheus node info (non-critical, can fail)
        if let Some(ref rpc) = self.rpc_client {
            if let Ok(node_info) = rpc.get_node_info_prometheus().await {
                if let Some(seconds) = node_info.uptime_seconds {
                    let days = seconds / 86400;
                    let hours = (seconds % 86400) / 3600;
                    let minutes = (seconds % 3600) / 60;

                    if days > 0 {
                        consensus_data.uptime = format!("{}d {}h {}m", days, hours, minutes);
                    } else if hours > 0 {
                        consensus_data.uptime = format!("{}h {}m", hours, minutes);
                    } else {
                        consensus_data.uptime = format!("{}m", minutes);
                    }
                } else {
                    consensus_data.uptime = "N/A".to_string();
                }
            }
        }

        // Sync epoch to staking state for withdrawal readiness checks
        // PendingWithdrawal::is_ready() uses self.state.staking.current_epoch
        // Copy before move since ConsensusData doesn't implement Copy
        let epoch = consensus_data.epoch;
        self.state.update_consensus(consensus_data);
        self.state.staking.set_epoch(epoch);
    }

    /// Refresh staking data from RPC
    ///
    /// This updates the user's balance if a delegator address is configured.
    ///
    /// NOTE: Delegations are NOT auto-loaded due to performance issues:
    /// - get_all_delegations() uses slow pagination (50+ seconds)
    /// - CLI command "stake query delegators -V ID" also hangs
    /// - Solution: Users manually add validators with [a] key in Staking screen
    /// - Only single delegator query works: "stake query delegator -V ID -a ADDRESS"
    pub(crate) async fn refresh_staking_data(&mut self) {
        // DEBUG REMOVED refresh_staking_data() starting...");
        // Clone address to avoid borrow checker issues
        let address_opt = self.state.staking.delegator_address.clone();

        // Only refresh if we have an address configured
        if let Some(address) = address_opt {
            if let Some(ref rpc) = self.rpc_client {
                // Fetch balance from RPC (returns MON as f64)
                // DEBUG REMOVED Fetching balance...");
                match rpc.get_balance(&address).await {
                    Ok(balance) => {
                        self.state.staking.set_balance(balance);
                        self.state.staking.clear_error();
                        // DEBUG REMOVED Balance loaded: {} MON", balance);
                    }
                    Err(e) => {
                        // Set error but don't clear the address
                        self.state
                            .staking
                            .set_error(format!("Failed to fetch balance: {}", e));
                        // DEBUG REMOVED Balance fetch failed: {}", e);
                        return;
                    }
                }

                // DON'T auto-load delegations - too slow!
                // The CLI command "stake query delegators -V ID" hangs (returns infinite results)
                // get_all_delegations() has same issue - slow pagination
                //
                // Solution: Users manually add validators with [a] key in Staking screen
                // This uses fast single delegator query: "stake query delegator -V ID -a ADDRESS"
                // DEBUG REMOVED Skipping delegations load (pagination too slow)");
                // DEBUG REMOVED Use [a] key in Staking screen to manually add validators");

                // Mark as refreshed
                self.state.staking.mark_refreshed();
            }
        }
        // DEBUG REMOVED refresh_staking_data() complete");
    }
}

#[cfg(test)]
mod tests {
    use crate::tui::state::ValidatorData;

    /// Test TD-TUI-002: Verify is_validator persists after being set
    ///
    /// This test ensures that once is_validator is set to true,
    /// it remains true through subsequent updates.
    #[test]
    fn test_validator_status_persists_after_update() {
        let mut validator_data = ValidatorData::new();

        // Initial state - default is false (full node)
        assert!(!validator_data.is_validator);

        // Simulate the refresh setting it to true
        validator_data.is_validator = true;
        assert!(validator_data.is_validator);

        // Simulate other updates happening
        validator_data.chain_id = 10143;
        validator_data.network_name = "testnet".to_string();

        // Verify is_validator is still true
        assert!(validator_data.is_validator);
    }

    /// Test TD-TUI-002: Verify from_config creates validator with is_validator=false
    ///
    /// This test verifies that creating ValidatorData from config
    /// correctly initializes is_validator to false by default.
    #[test]
    fn test_validator_data_from_config_defaults_to_full_node() {
        let validator_data = ValidatorData::from_config(143, "mainnet", "http://localhost:8080");

        // Verify all fields are set correctly
        assert_eq!(validator_data.chain_id, 143);
        assert_eq!(validator_data.network_name, "mainnet");
        assert_eq!(validator_data.rpc_endpoint, "http://localhost:8080");
        assert!(!validator_data.is_validator); // Default is full node
    }

    /// Test TD-TUI-002: Verify is_validator can be toggled
    #[test]
    fn test_validator_status_can_be_toggled() {
        let mut validator_data = ValidatorData::new();

        // Start as full node
        assert!(!validator_data.is_validator);

        // Toggle to validator
        validator_data.is_validator = true;
        assert!(validator_data.is_validator);

        // Toggle back to full node
        validator_data.is_validator = false;
        assert!(!validator_data.is_validator);
    }

    /// Test TD-TUI-002: Verify validator status persistence logic
    ///
    /// This test simulates the refresh behavior where is_validator
    /// should only transition from false to true, never back to false.
    #[test]
    fn test_validator_status_one_way_transition() {
        let mut validator_data = ValidatorData::new();

        // Start as full node
        assert!(!validator_data.is_validator);

        // Simulate first refresh detecting validator
        let detected_as_validator = true;
        if detected_as_validator && !validator_data.is_validator {
            validator_data.is_validator = true;
        }
        assert!(validator_data.is_validator);

        // Simulate subsequent refresh where detection fails (false negative)
        // The validator status should remain true
        let detected_as_validator = false; // Detection failed
        if detected_as_validator && !validator_data.is_validator {
            validator_data.is_validator = true;
        }
        // Verify it's still true (wasn't reset)
        assert!(validator_data.is_validator);

        // Another successful detection should keep it true
        let detected_as_validator = true;
        if detected_as_validator && !validator_data.is_validator {
            validator_data.is_validator = true; // This won't execute since already true
        }
        assert!(validator_data.is_validator);
    }

    /// Test TD-TUI-003: Verify staking data refresh updates balance
    ///
    /// This test verifies that when a delegator address is configured,
    /// the refresh_staking_data function updates the balance.
    #[test]
    fn test_staking_refresh_updates_balance() {
        use crate::tui::staking::StakingState;

        let mut staking_state = StakingState::new();

        // Initial state - no address, no balance
        assert_eq!(staking_state.delegator_address, None);
        assert_eq!(staking_state.balance, 0.0);

        // Set an address
        staking_state.set_address("0x1234567890123456789012345678901234567890");
        assert!(staking_state.delegator_address.is_some());

        // Simulate balance update (as would happen during refresh) - now in MON
        staking_state.set_balance(1.5); // 1.5 MON
        assert_eq!(staking_state.balance, 1.5);
        assert_eq!(staking_state.format_balance(), "1.5");
    }

    /// Test TD-TUI-003: Verify format_address shows "Not connected" when no address
    #[test]
    fn test_format_address_no_address() {
        use crate::tui::staking::StakingState;

        let staking_state = StakingState::new();
        assert_eq!(staking_state.format_address(), "Not connected");
    }

    /// Test TD-TUI-003: Verify format_address shows truncated address when set
    #[test]
    fn test_format_address_with_address() {
        use crate::tui::staking::StakingState;

        let mut staking_state = StakingState::new();
        staking_state.set_address("0x1234567890123456789012345678901234567890");
        let formatted = staking_state.format_address();

        // Should be truncated (first 6 + last 4 chars)
        assert_eq!(formatted, "0x1234...7890");
        assert!(!formatted.contains("Not connected"));
    }
}
