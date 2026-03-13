//! Doctor diagnostic check execution for TUI application

use super::TuiApp;

impl TuiApp {
    /// Run doctor diagnostic checks
    pub(crate) async fn run_doctor_checks(&mut self) {
        // Start checks - set all to running
        self.state.doctor.start_checks();

        // Run system checks
        self.run_system_doctor_checks();

        // Run network checks
        self.run_network_doctor_checks().await;

        // Run service checks (Linux only)
        #[cfg(target_os = "linux")]
        self.run_service_doctor_checks();

        // Run config checks
        self.run_config_doctor_checks();

        // Run consensus checks (NEW)
        self.run_consensus_doctor_checks().await;

        // Finish and calculate summary
        self.state.doctor.finish_checks();
    }

    /// Run system-related doctor checks
    pub(crate) fn run_system_doctor_checks(&mut self) {
        use crate::tui::doctor_state::CheckStatus;

        // CPU check - recommend 16 cores
        let cpu_cores = num_cpus::get();
        let (cpu_status, cpu_msg) = if cpu_cores >= 16 {
            (CheckStatus::Pass, format!("{} cores available", cpu_cores))
        } else {
            (CheckStatus::Fail, format!("{} cores (need 16+)", cpu_cores))
        };
        self.state.doctor.update_check(
            "CPU Cores",
            cpu_status,
            &cpu_msg,
            if cpu_cores < 16 {
                Some("Upgrade to 16+ cores")
            } else {
                None
            },
        );

        // Memory check - recommend 32GB
        let total_mem_gb = self.state.system.total_memory / (1024 * 1024 * 1024);
        let (mem_status, mem_msg) = if total_mem_gb >= 32 {
            (CheckStatus::Pass, format!("{} GB RAM", total_mem_gb))
        } else {
            (
                CheckStatus::Fail,
                format!("{} GB (need 32GB)", total_mem_gb),
            )
        };
        self.state.doctor.update_check(
            "Memory (RAM)",
            mem_status,
            &mem_msg,
            if total_mem_gb < 32 {
                Some("Upgrade to 32GB RAM")
            } else {
                None
            },
        );

        // PHASE-5-FIX: OS Disk check using df -h / (matches CLI doctor)
        self.run_os_disk_check();

        // PHASE-5-FIX: MPT Storage check using monad-mpt --storage /dev/triedb (matches CLI doctor)
        #[cfg(target_os = "linux")]
        self.run_mpt_disk_check();
    }

    /// PHASE-5-FIX: OS Disk check using df -h / (CLI doctor parity)
    #[cfg(target_os = "linux")]
    fn run_os_disk_check(&mut self) {
        use crate::tui::doctor_state::CheckStatus;
        use std::process::Command;

        // Run df -h to get disk usage for root filesystem (OS disk)
        let output = Command::new("df").args(["-h", "/"]).output();

        match output {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);

                // Parse df output: /dev/mapper/ubuntu--vg-ubuntu--lv  1.8T   75G  1.7T   5% /
                for line in output_str.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        let total = parts[1]; // Size (e.g., "1.8T")
                        let used = parts[2]; // Used (e.g., "75G")
                        let avail = parts[3]; // Available (e.g., "1.7T")
                        let percent = parts[4]; // Use% (e.g., "5%")

                        // Parse available space - use CLI doctor's parse_size_to_gb logic
                        let avail_gb = Self::parse_size_to_gb(avail);
                        let passed = avail_gb >= 500.0; // Fail if less than 500GB available

                        let status = if passed {
                            CheckStatus::Pass
                        } else {
                            CheckStatus::Fail
                        };

                        let msg = format!(
                            "{} total, {} used, {} available ({}) - Threshold: 500GB",
                            total, used, avail, percent
                        );

                        self.state.doctor.update_check(
                            "Disk Space (OS)",
                            status,
                            &msg,
                            if !passed {
                                Some("Free up disk space (minimum 500GB required)")
                            } else {
                                None
                            },
                        );
                        return;
                    }
                }

                // Couldn't parse df output
                self.state.doctor.update_check(
                    "Disk Space (OS)",
                    CheckStatus::Error,
                    "Could not parse df output",
                    Some("Check disk manually with: df -h /"),
                );
            }
            Err(_) => {
                self.state.doctor.update_check(
                    "Disk Space (OS)",
                    CheckStatus::Error,
                    "df command failed",
                    Some("Install util-linux or check disk manually"),
                );
            }
        }
    }

    /// PHASE-5-FIX: MPT Storage check using monad-mpt --storage /dev/triedb (CLI doctor parity)
    #[cfg(target_os = "linux")]
    #[allow(dead_code)] // Linux-specific function
    fn run_mpt_disk_check(&mut self) {
        use crate::tui::doctor_state::CheckStatus;
        use std::process::Command;

        // Try monad-mpt command (matches CLI doctor implementation)
        let output = Command::new("monad-mpt")
            .args(["--storage", "/dev/triedb"])
            .output();

        match output {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);

                // Parse monad-mpt output format:
                // "/dev/triedb:\n    1.80TiB 839GiB  1.73%"
                for line in output_str.lines() {
                    // Look for line with percentage
                    if line.contains('%') {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            // Find the part with %
                            for part in &parts {
                                if part.ends_with('%') {
                                    let percent_str = part.trim_end_matches('%');
                                    if let Ok(percentage) = percent_str.parse::<f64>() {
                                        let passed = percentage < 90.0;
                                        let status = if passed {
                                            CheckStatus::Pass
                                        } else {
                                            CheckStatus::Fail
                                        };

                                        self.state.doctor.update_check(
                                            "MPT Storage (Triedb)",
                                            status,
                                            &format!("{:.1}% used", percentage),
                                            if !passed {
                                                Some("MPT storage >90% full - consider cleanup or expansion")
                                            } else {
                                                None
                                            },
                                        );
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }

                // If we have output but couldn't parse
                if !output_str.trim().is_empty() {
                    self.state.doctor.update_check(
                        "MPT Storage (Triedb)",
                        CheckStatus::Pass,
                        "Unable to parse (check monad-mpt manually)",
                        None,
                    );
                } else {
                    self.state.doctor.update_check(
                        "MPT Storage (Triedb)",
                        CheckStatus::Pass,
                        "MPT query unavailable (non-blocking)",
                        None,
                    );
                }
            }
            Err(_) => {
                // monad-mpt not available - non-blocking
                self.state.doctor.update_check(
                    "MPT Storage (Triedb)",
                    CheckStatus::Pass,
                    "MPT query unavailable (non-blocking)",
                    None,
                );
            }
        }
    }

    /// Helper to parse human-readable size to GB (e.g., "1.8T" -> 1843.2 GB, "75G" -> 75.0 GB)
    /// Matches CLI doctor implementation
    #[allow(dead_code)] // Used in Linux-only code paths
    fn parse_size_to_gb(size: &str) -> f64 {
        let size_lower = size.trim().to_lowercase();

        // Find where the numeric part ends
        let mut numeric_end = 0;
        for (i, c) in size_lower.chars().enumerate() {
            if c.is_ascii_digit() || c == '.' {
                numeric_end = i + 1;
            } else {
                break;
            }
        }

        if numeric_end == 0 {
            return 0.0;
        }

        let numeric_str = &size_lower[..numeric_end];
        let suffix = size_lower.chars().nth(numeric_end).unwrap_or(' ');

        if let Ok(value) = numeric_str.parse::<f64>() {
            match suffix {
                't' => value * 1024.0,            // TB to GB
                'g' => value,                     // GB
                'm' => value / 1024.0,            // MB to GB
                'k' => value / (1024.0 * 1024.0), // KB to GB
                _ => value,
            }
        } else {
            0.0
        }
    }

    // Non-Linux: Use basic disk check (not df -h /)
    #[cfg(not(target_os = "linux"))]
    fn run_os_disk_check(&mut self) {
        use crate::tui::doctor_state::CheckStatus;

        // Use SystemInfo for non-Linux systems
        let disk_avail_gb = self.state.system.disk_available / (1024 * 1024 * 1024);
        let disk_total_gb = self.state.system.disk_total / (1024 * 1024 * 1024);
        let (disk_status, disk_msg) = if disk_avail_gb >= 100 && disk_total_gb >= 2000 {
            (
                CheckStatus::Pass,
                format!("{} GB available of {} GB", disk_avail_gb, disk_total_gb),
            )
        } else if disk_avail_gb >= 100 {
            (CheckStatus::Pass, format!("{} GB available", disk_avail_gb))
        } else {
            (CheckStatus::Fail, format!("Only {} GB free", disk_avail_gb))
        };
        self.state.doctor.update_check(
            "Disk Space (OS)",
            disk_status,
            &disk_msg,
            if disk_avail_gb < 100 {
                Some("Free up disk space or upgrade to 2TB NVMe")
            } else {
                None
            },
        );
    }

    // Non-Linux: Skip MPT check
    #[cfg(not(target_os = "linux"))]
    #[allow(dead_code)] // Stub for cross-platform compatibility
    fn run_mpt_disk_check(&mut self) {
        // MPT check not available on non-Linux
        // Don't add a check - it's Linux-specific
    }

    /// Run network-related doctor checks
    pub(crate) async fn run_network_doctor_checks(&mut self) {
        use crate::tui::doctor_state::CheckStatus;

        // RPC Connection check
        let (rpc_status, rpc_msg) = if self.state.network.is_connected {
            (CheckStatus::Pass, "Connected".to_string())
        } else {
            let err = self
                .state
                .network
                .last_error
                .as_deref()
                .unwrap_or("Unknown error");
            (CheckStatus::Fail, format!("Failed: {}", err))
        };
        self.state.doctor.update_check(
            "RPC Connection",
            rpc_status,
            &rpc_msg,
            if !self.state.network.is_connected {
                Some("Check if node is running")
            } else {
                None
            },
        );

        // RPC Port check (localhost:8080)
        let port_open = tokio::net::TcpStream::connect("127.0.0.1:8080")
            .await
            .is_ok();
        let (port_status, port_msg) = if port_open {
            (CheckStatus::Pass, "Port 8080 open".to_string())
        } else {
            (CheckStatus::Fail, "Port 8080 not accessible".to_string())
        };
        self.state.doctor.update_check(
            "RPC Port",
            port_status,
            &port_msg,
            if !port_open {
                Some("Start the Monad node")
            } else {
                None
            },
        );

        // Sync status check
        let (sync_status, sync_msg) = if !self.state.network.is_connected {
            (
                CheckStatus::Error,
                "Cannot check - no connection".to_string(),
            )
        } else if self.state.network.is_syncing {
            let progress = self
                .state
                .network
                .sync_progress
                .map(|p| format!("{}%", p))
                .unwrap_or_else(|| "in progress".to_string());
            (CheckStatus::Fail, format!("Syncing ({})", progress))
        } else {
            (CheckStatus::Pass, "Synced".to_string())
        };
        self.state.doctor.update_check(
            "Sync Status",
            sync_status,
            &sync_msg,
            if self.state.network.is_syncing {
                Some("Wait for sync to complete")
            } else {
                None
            },
        );
    }

    /// Run service-related doctor checks (Linux only)
    #[cfg(target_os = "linux")]
    pub(crate) fn run_service_doctor_checks(&mut self) {
        use crate::tui::doctor_state::CheckStatus;
        use std::process::Command;

        // PHASE-5-FIX: Mark monad-archiver as optional service
        // Optional services won't fail the check if not running
        let optional_services = ["monad-archiver"];

        let services = [
            ("Monad BFT Service", "monad-bft"),
            ("Monad Execution Service", "monad-execution"),
            ("Monad RPC Service", "monad-rpc"),
            ("Monad Archiver Service", "monad-archiver"),
        ];

        for (check_name, service) in services {
            let is_optional = optional_services.contains(&service);

            let output = Command::new("systemctl")
                .args(["is-active", service])
                .output();

            let (status, msg) = match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if stdout == "active" {
                        (CheckStatus::Pass, "Running".to_string())
                    } else {
                        // Optional services don't fail if not active
                        if is_optional {
                            (
                                CheckStatus::Pass,
                                format!("Not running [optional] ({})", stdout),
                            )
                        } else {
                            (CheckStatus::Fail, format!("Status: {}", stdout))
                        }
                    }
                }
                Err(e) => {
                    // Optional services don't fail on error
                    if is_optional {
                        (CheckStatus::Pass, format!("Check failed [optional]: {}", e))
                    } else {
                        (CheckStatus::Error, format!("Check failed: {}", e))
                    }
                }
            };

            // Only suggest fix for non-optional services that failed
            let fix_msg = if !is_optional && status != CheckStatus::Pass {
                Some(format!("Start service: sudo systemctl start {}", service))
            } else {
                None
            };
            self.state
                .doctor
                .update_check(check_name, status, &msg, fix_msg.as_deref());
        }
    }

    /// Run configuration-related doctor checks
    pub(crate) fn run_config_doctor_checks(&mut self) {
        use crate::tui::doctor_state::CheckStatus;

        // Chain ID validation
        let chain_id = self.state.validator.chain_id;
        let (chain_status, chain_msg) = if chain_id == 143 {
            (CheckStatus::Pass, "Mainnet (143)".to_string())
        } else if chain_id == 10143 {
            (CheckStatus::Pass, "Testnet (10143)".to_string())
        } else {
            (CheckStatus::Fail, format!("Unknown chain: {}", chain_id))
        };
        self.state.doctor.update_check(
            "Chain ID Config",
            chain_status,
            &chain_msg,
            if chain_id != 143 && chain_id != 10143 {
                Some("Configure correct chain ID")
            } else {
                None
            },
        );
    }

    /// Run consensus-related doctor checks (NEW from monad-status.sh)
    ///
    /// Uses state.consensus data that was already refreshed during startup/refresh.
    /// This ensures Epoch/Round values match Dashboard display and avoids redundant calls.
    pub(crate) async fn run_consensus_doctor_checks(&mut self) {
        use crate::tui::doctor_state::CheckStatus;

        // Use consensus data from state (already refreshed from Prometheus/forkpoint file)
        let epoch = self.state.consensus.epoch;
        let round = self.state.consensus.round;
        let fork_epoch = self.state.consensus.forkpoint_epoch;
        let fork_round = self.state.consensus.forkpoint_round;

        // Current Epoch check
        if epoch > 0 {
            self.state.doctor.update_check(
                "Current Epoch",
                CheckStatus::Pass,
                &format!("Epoch {}", epoch),
                None,
            );
        } else {
            self.state.doctor.update_check(
                "Current Epoch",
                CheckStatus::Error,
                "Not available",
                Some("Check if node is running"),
            );
        }

        // Current Round check
        if round > 0 {
            self.state.doctor.update_check(
                "Current Round",
                CheckStatus::Pass,
                &format!("Round {}", round),
                None,
            );
        } else {
            self.state.doctor.update_check(
                "Current Round",
                CheckStatus::Error,
                "Not available",
                Some("Check if node is running"),
            );
        }

        // Forkpoint Info check
        if fork_epoch > 0 || fork_round > 0 {
            self.state.doctor.update_check(
                "Forkpoint Info",
                CheckStatus::Pass,
                &format!("Epoch {}, Round {}", fork_epoch, fork_round),
                None,
            );
        } else {
            self.state.doctor.update_check(
                "Forkpoint Info",
                CheckStatus::Pass,
                "Using default forkpoint",
                None,
            );
        }

        // StateSync Status check
        let statesync_info = if let Some(ref rpc) = self.rpc_client {
            rpc.get_statesync_info_prometheus()
                .await
                .unwrap_or_default()
        } else {
            Default::default()
        };

        if let Some(is_syncing) = statesync_info.is_syncing {
            if is_syncing {
                let progress = statesync_info.progress_estimate.unwrap_or(0.0) * 100.0;
                self.state.doctor.update_check(
                    "StateSync Status",
                    CheckStatus::Fail,
                    &format!("In progress ({:.1}%)", progress),
                    Some("Wait for statesync to complete"),
                );
            } else {
                self.state.doctor.update_check(
                    "StateSync Status",
                    CheckStatus::Pass,
                    "Live mode (not syncing)",
                    None,
                );
            }
        } else {
            self.state.doctor.update_check(
                "StateSync Status",
                CheckStatus::Pass,
                "Not configured (using default)",
                None,
            );
        }
    }
}
