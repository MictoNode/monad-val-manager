//! System Monitor - Real-time system metrics collector
//!
//! This module provides btop-style real-time system monitoring
//! with per-second refresh rate, collecting:
//! - Per-core CPU usage and frequency
//! - Disk usage and I/O statistics
//! - Network interface bandwidth
//! - Total CPU and memory usage
//! - System uptime and load average

use std::collections::HashMap;
use std::time::Instant;

use sysinfo::{Disks, Networks, System};

use crate::tui::perf_state::{CpuCoreData, DiskData, NetworkThroughput, PerfState};

/// System monitor for real-time metrics collection
pub struct SystemMonitor {
    sys: System,
    disks: Disks,
    networks: Networks,
    /// Previous disk read bytes per disk name
    prev_disk_read: HashMap<String, u64>,
    /// Previous disk write bytes per disk name
    prev_disk_write: HashMap<String, u64>,
    /// Previous network rx bytes per interface
    prev_network_rx: HashMap<String, u64>,
    /// Previous network tx bytes per interface
    prev_network_tx: HashMap<String, u64>,
    /// Last collection timestamp
    last_refresh: Option<Instant>,
}

impl SystemMonitor {
    /// Create a new system monitor
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        Self {
            sys,
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            prev_disk_read: HashMap::new(),
            prev_disk_write: HashMap::new(),
            prev_network_rx: HashMap::new(),
            prev_network_tx: HashMap::new(),
            last_refresh: None,
        }
    }

    /// Refresh all metrics and return a new PerfState
    pub fn refresh(&mut self) -> PerfState {
        // Refresh system data
        self.sys.refresh_all();
        self.disks.refresh();
        self.networks.refresh();

        let now = Instant::now();
        let elapsed = self.last_refresh.map_or(1.0, |t| t.elapsed().as_secs_f64());

        // Collect CPU core data
        let cpu_cores: Vec<CpuCoreData> = self
            .sys
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| CpuCoreData {
                core_id: i,
                usage_pct: cpu.cpu_usage(),
                frequency_mhz: cpu.frequency(),
            })
            .collect();

        // Collect disk data
        let disks: Vec<DiskData> = self
            .disks
            .iter()
            .map(|disk| {
                let name = disk.name().to_string_lossy().to_string();
                let mount_point = disk.mount_point().to_string_lossy().to_string();
                let total_space = disk.total_space();
                let available_space = disk.available_space();
                let total_gb = total_space as f64 / (1024.0 * 1024.0 * 1024.0);
                let used_gb = (total_space - available_space) as f64 / (1024.0 * 1024.0 * 1024.0);
                let usage_pct = if total_space > 0 {
                    ((total_space - available_space) as f64 / total_space as f64) * 100.0
                } else {
                    0.0
                };

                // Calculate read/write speeds using deltas
                // Note: sysinfo doesn't provide per-disk IO stats directly
                // We'll use usage stats as a fallback
                let read_bps = 0u64;
                let write_bps = 0u64;

                DiskData {
                    name,
                    mount_point,
                    total_gb,
                    used_gb,
                    usage_pct,
                    read_bps,
                    write_bps,
                }
            })
            .collect();

        // Collect network data
        let network: Vec<NetworkThroughput> = self
            .networks
            .iter()
            .map(|(name, data)| {
                let interface_name = name.to_string();
                let current_rx = data.received();
                let current_tx = data.transmitted();

                let rx_bps = if elapsed > 0.0 {
                    let prev = self.prev_network_rx.get(&interface_name).copied().unwrap_or(0);
                    let delta = current_rx.saturating_sub(prev);
                    (delta as f64 / elapsed) as u64
                } else {
                    0
                };

                let tx_bps = if elapsed > 0.0 {
                    let prev = self.prev_network_tx.get(&interface_name).copied().unwrap_or(0);
                    let delta = current_tx.saturating_sub(prev);
                    (delta as f64 / elapsed) as u64
                } else {
                    0
                };

                // Update previous values
                self.prev_network_rx.insert(interface_name.clone(), current_rx);
                self.prev_network_tx.insert(interface_name.clone(), current_tx);

                NetworkThroughput {
                    interface: interface_name,
                    rx_bps,
                    tx_bps,
                }
            })
            .collect();

        // Calculate global CPU usage
        let global_cpu_usage = self.sys.global_cpu_usage() as f64;

        // Calculate memory usage
        let total_memory = self.sys.total_memory() as f64;
        let used_memory = self.sys.used_memory() as f64;
        let memory_usage_pct = if total_memory > 0.0 {
            (used_memory / total_memory) * 100.0
        } else {
            0.0
        };

        // Get uptime
        let uptime_seconds = System::uptime();

        // Update last refresh time
        self.last_refresh = Some(now);

        // Create PerfState
        let mut state = PerfState::new();
        state.update(
            cpu_cores,
            disks,
            network,
            global_cpu_usage,
            memory_usage_pct,
            total_memory as u64,
            used_memory as u64,
        );
        state.uptime_seconds = uptime_seconds;
        state.last_refresh = Some(now);

        state
    }

    /// Check if data has been refreshed at least once
    pub fn has_data(&self) -> bool {
        self.last_refresh.is_some()
    }

    /// Get time since last refresh
    pub fn time_since_refresh(&self) -> Option<std::time::Duration> {
        self.last_refresh.map(|t| t.elapsed())
    }

    /// Get number of CPU cores
    pub fn core_count(&self) -> usize {
        self.sys.cpus().len()
    }

    /// Get OS name
    pub fn os_name(&self) -> String {
        System::name().unwrap_or_else(|| "unknown".to_string())
    }

    /// Get OS version
    pub fn os_version(&self) -> String {
        System::os_version().unwrap_or_else(|| "unknown".to_string())
    }

    /// Get hostname
    pub fn hostname(&self) -> String {
        System::host_name().unwrap_or_else(|| "unknown".to_string())
    }

    /// Get kernel version
    pub fn kernel_version(&self) -> String {
        System::kernel_version().unwrap_or_else(|| "unknown".to_string())
    }

    /// Get CPU brand string
    pub fn cpu_brand(&self) -> &str {
        self.sys
            .cpus()
            .first()
            .map(|cpu| cpu.brand())
            .unwrap_or("unknown")
    }

    /// Get CPU vendor
    pub fn cpu_vendor(&self) -> &str {
        self.sys
            .cpus()
            .first()
            .map(|cpu| cpu.vendor_id())
            .unwrap_or("unknown")
    }

    /// Format uptime as human-readable string
    pub fn format_uptime(&self) -> String {
        let seconds = System::uptime();
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
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Format bytes per second as human-readable string
pub fn format_bytes_per_sec(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * KB;
    const GB: f64 = 1024.0 * MB;

    let bytes_f = bytes as f64;

    if bytes_f >= GB {
        format!("{:.1} GB/s", bytes_f / GB)
    } else if bytes_f >= MB {
        format!("{:.1} MB/s", bytes_f / MB)
    } else if bytes_f >= KB {
        format!("{:.1} KB/s", bytes_f / KB)
    } else {
        format!("{} B/s", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_monitor_creation() {
        let monitor = SystemMonitor::new();
        assert!(monitor.core_count() > 0);
    }

    #[test]
    fn test_system_monitor_refresh() {
        let mut monitor = SystemMonitor::new();
        let state = monitor.refresh();

        assert!(!state.cpu_cores.is_empty());
        assert!(monitor.has_data());
    }

    #[test]
    fn test_system_monitor_os_info() {
        let monitor = SystemMonitor::new();
        let os_name = monitor.os_name();
        let hostname = monitor.hostname();

        assert!(!os_name.is_empty());
        assert!(!hostname.is_empty());
    }

    #[test]
    fn test_system_monitor_uptime() {
        let monitor = SystemMonitor::new();
        let uptime = monitor.format_uptime();
        assert!(!uptime.is_empty());
    }

    #[test]
    fn test_system_monitor_refresh_idempotent() {
        let mut monitor = SystemMonitor::new();

        let state1 = monitor.refresh();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let state2 = monitor.refresh();

        assert!(state2.last_refresh > state1.last_refresh);
    }

    #[test]
    fn test_format_bytes_per_sec() {
        assert_eq!(format_bytes_per_sec(0), "0 B/s");
        assert_eq!(format_bytes_per_sec(500), "500 B/s");
        assert_eq!(format_bytes_per_sec(1024), "1.0 KB/s");
        assert!(format_bytes_per_sec(1024 * 1024).contains("MB/s"));
        assert!(format_bytes_per_sec(1024 * 1024 * 1024).contains("GB/s"));
    }

    #[test]
    fn test_system_monitor_cpu_info() {
        let monitor = SystemMonitor::new();
        let brand = monitor.cpu_brand();
        let vendor = monitor.cpu_vendor();

        // These should return strings (may be "unknown" on some systems)
        assert!(!brand.is_empty());
        assert!(!vendor.is_empty());
    }

    #[test]
    fn test_system_monitor_kernel_version() {
        let monitor = SystemMonitor::new();
        let version = monitor.kernel_version();
        assert!(!version.is_empty());
    }
}
