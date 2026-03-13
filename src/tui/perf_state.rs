//! Perf State - State management for btop-style performance monitoring
//!
//! This module provides data structures for real-time system metrics:
//! - Per-core CPU usage and frequency with historical tracking
//! - Disk usage and I/O statistics
//! - Network throughput per interface
//! - Historical data for sparkline graphs

use std::time::Instant;

/// Maximum history length for sparkline graphs
const MAX_HISTORY: usize = 30;

/// Per-core CPU data
#[derive(Debug, Clone)]
pub struct CpuCoreData {
    /// Core identifier (0-indexed)
    pub core_id: usize,
    /// CPU usage percentage (0.0 - 100.0)
    pub usage_pct: f32,
    /// CPU frequency in MHz
    pub frequency_mhz: u64,
}

impl CpuCoreData {
    /// Create new CPU core data
    pub fn new(core_id: usize, usage_pct: f32, frequency_mhz: u64) -> Self {
        Self {
            core_id,
            usage_pct,
            frequency_mhz,
        }
    }

    /// Get usage color based on percentage
    pub fn usage_color(&self) -> &'static str {
        if self.usage_pct < 50.0 {
            "green"
        } else if self.usage_pct < 75.0 {
            "yellow"
        } else {
            "red"
        }
    }

    /// Format usage for display
    pub fn format_usage(&self) -> String {
        format!("{:5.1}%", self.usage_pct)
    }

    /// Format frequency for display
    pub fn format_frequency(&self) -> String {
        if self.frequency_mhz >= 1000 {
            format!("{:.1} GHz", self.frequency_mhz as f64 / 1000.0)
        } else {
            format!("{} MHz", self.frequency_mhz)
        }
    }
}

impl Default for CpuCoreData {
    fn default() -> Self {
        Self {
            core_id: 0,
            usage_pct: 0.0,
            frequency_mhz: 0,
        }
    }
}

/// Disk data with usage and I/O statistics
#[derive(Debug, Clone)]
pub struct DiskData {
    /// Disk name (e.g., "C:", "/dev/sda1")
    pub name: String,
    /// Mount point
    pub mount_point: String,
    /// Total space in GB
    pub total_gb: u64,
    /// Used space in GB
    pub used_gb: u64,
    /// Usage percentage (0.0 - 100.0)
    pub usage_pct: f64,
    /// Read speed in bytes per second
    pub read_bps: u64,
    /// Write speed in bytes per second
    pub write_bps: u64,
}

impl DiskData {
    /// Create new disk data
    pub fn new(
        name: String,
        mount_point: String,
        total_gb: u64,
        used_gb: u64,
        read_bps: u64,
        write_bps: u64,
    ) -> Self {
        let usage_pct = if total_gb > 0 {
            (used_gb as f64 / total_gb as f64) * 100.0
        } else {
            0.0
        };
        Self {
            name,
            mount_point,
            total_gb,
            used_gb,
            usage_pct,
            read_bps,
            write_bps,
        }
    }

    /// Get usage color based on percentage
    pub fn usage_color(&self) -> &'static str {
        if self.usage_pct < 70.0 {
            "green"
        } else if self.usage_pct < 85.0 {
            "yellow"
        } else {
            "red"
        }
    }

    /// Format usage for display
    pub fn format_usage(&self) -> String {
        format!("{:.1}%", self.usage_pct)
    }

    /// Format space for display
    pub fn format_space(&self) -> String {
        format!("{} / {} GB", self.used_gb, self.total_gb)
    }

    /// Format read speed for display
    pub fn format_read_speed(&self) -> String {
        format_bytes_per_sec(self.read_bps)
    }

    /// Format write speed for display
    pub fn format_write_speed(&self) -> String {
        format_bytes_per_sec(self.write_bps)
    }
}

impl Default for DiskData {
    fn default() -> Self {
        Self {
            name: String::new(),
            mount_point: String::new(),
            total_gb: 0,
            used_gb: 0,
            usage_pct: 0.0,
            read_bps: 0,
            write_bps: 0,
        }
    }
}

/// Network interface throughput data
#[derive(Debug, Clone, Default)]
pub struct NetworkThroughput {
    /// Interface name (e.g., "eth0", "wlan0")
    pub interface: String,
    /// Receive speed in bytes per second
    pub rx_bps: u64,
    /// Transmit speed in bytes per second
    pub tx_bps: u64,
}

impl NetworkThroughput {
    /// Create new network throughput data
    pub fn new(interface: String, rx_bps: u64, tx_bps: u64) -> Self {
        Self {
            interface,
            rx_bps,
            tx_bps,
        }
    }

    /// Get total throughput (rx + tx)
    pub fn total_bps(&self) -> u64 {
        self.rx_bps.saturating_add(self.tx_bps)
    }

    /// Format receive speed for display
    pub fn format_rx_speed(&self) -> String {
        format_bytes_per_sec(self.rx_bps)
    }

    /// Format transmit speed for display
    pub fn format_tx_speed(&self) -> String {
        format_bytes_per_sec(self.tx_bps)
    }
}

/// Central performance state for btop-style monitoring
#[derive(Debug, Clone)]
pub struct PerfState {
    /// Per-core CPU data
    pub cpu_cores: Vec<CpuCoreData>,
    /// Disk data
    pub disks: Vec<DiskData>,
    /// Network interfaces
    pub network: Vec<NetworkThroughput>,
    /// Total CPU usage (aggregate)
    pub total_cpu_usage: f32,
    /// Total memory usage percentage
    pub memory_usage_pct: f64,
    /// Used memory in bytes
    pub memory_used: u64,
    /// Total memory in bytes
    pub memory_total: u64,
    /// Historical CPU usage for sparkline (last MAX_HISTORY readings)
    pub cpu_history: Vec<f32>,
    /// Historical memory usage for sparkline (last MAX_HISTORY readings)
    pub memory_history: Vec<f64>,
    /// Last update timestamp
    pub last_update: Option<Instant>,
}

impl Default for PerfState {
    fn default() -> Self {
        Self::new()
    }
}

impl PerfState {
    /// Create new performance state
    pub fn new() -> Self {
        Self {
            cpu_cores: Vec::new(),
            disks: Vec::new(),
            network: Vec::new(),
            total_cpu_usage: 0.0,
            memory_usage_pct: 0.0,
            memory_used: 0,
            memory_total: 0,
            cpu_history: Vec::with_capacity(MAX_HISTORY),
            memory_history: Vec::with_capacity(MAX_HISTORY),
            last_update: None,
        }
    }

    /// Update with new metrics
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        cpu_cores: Vec<CpuCoreData>,
        disks: Vec<DiskData>,
        network: Vec<NetworkThroughput>,
        total_cpu_usage: f32,
        memory_usage_pct: f64,
        memory_used: u64,
        memory_total: u64,
    ) {
        self.cpu_cores = cpu_cores;
        self.disks = disks;
        self.network = network;
        self.total_cpu_usage = total_cpu_usage;
        self.memory_usage_pct = memory_usage_pct;
        self.memory_used = memory_used;
        self.memory_total = memory_total;

        // Update historical data for sparklines
        self.cpu_history.push(total_cpu_usage);
        if self.cpu_history.len() > MAX_HISTORY {
            self.cpu_history.remove(0);
        }

        self.memory_history.push(memory_usage_pct);
        if self.memory_history.len() > MAX_HISTORY {
            self.memory_history.remove(0);
        }

        self.last_update = Some(Instant::now());
    }

    /// Get formatted memory string
    pub fn format_memory(&self) -> String {
        let used_gb = self.memory_used as f64 / (1024.0 * 1024.0 * 1024.0);
        let total_gb = self.memory_total as f64 / (1024.0 * 1024.0 * 1024.0);
        format!("{:.1} / {:.1} GB", used_gb, total_gb)
    }

    /// Check if data has been refreshed at least once
    pub fn has_data(&self) -> bool {
        self.last_update.is_some()
    }

    /// Get average CPU usage across all cores
    pub fn avg_cpu_usage(&self) -> f32 {
        if self.cpu_cores.is_empty() {
            return 0.0;
        }
        self.cpu_cores.iter().map(|c| c.usage_pct).sum::<f32>() / self.cpu_cores.len() as f32
    }

    /// Get total network throughput (rx + tx)
    pub fn total_network_throughput(&self) -> u64 {
        self.network.iter().map(|n| n.total_bps()).sum()
    }

    /// Get total disk read speed
    pub fn total_disk_read(&self) -> u64 {
        self.disks.iter().map(|d| d.read_bps).sum()
    }

    /// Get total disk write speed
    pub fn total_disk_write(&self) -> u64 {
        self.disks.iter().map(|d| d.write_bps).sum()
    }

    /// Generate sparkline string for CPU history
    /// Returns a string like "▂▃▅▇█" representing usage over time
    pub fn cpu_sparkline(&self) -> String {
        self.generate_sparkline_f32(&self.cpu_history, 100.0)
    }

    /// Generate sparkline string for memory history
    pub fn memory_sparkline(&self) -> String {
        self.generate_sparkline(&self.memory_history, 100.0)
    }

    /// Generate sparkline from float values (f32 version)
    fn generate_sparkline_f32(&self, values: &[f32], max_value: f32) -> String {
        if values.is_empty() {
            return String::new();
        }

        const SPARKLINE_CHARS: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        values
            .iter()
            .map(|&v| {
                let normalized = (v / max_value).clamp(0.0, 1.0);
                let idx = (normalized * (SPARKLINE_CHARS.len() - 1) as f32) as usize;
                SPARKLINE_CHARS[idx]
            })
            .collect()
    }

    /// Generate sparkline from float values (f64 version)
    fn generate_sparkline(&self, values: &[f64], max_value: f64) -> String {
        if values.is_empty() {
            return String::new();
        }

        const SPARKLINE_CHARS: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        values
            .iter()
            .map(|&v| {
                let normalized = (v / max_value).clamp(0.0, 1.0);
                let idx = (normalized * (SPARKLINE_CHARS.len() - 1) as f64) as usize;
                SPARKLINE_CHARS[idx]
            })
            .collect()
    }

    /// Get CPU trend indicator (up/down arrow based on recent history)
    pub fn cpu_trend(&self) -> &'static str {
        if self.cpu_history.len() < 2 {
            return "→";
        }

        let recent_avg: f32 = *self.cpu_history.iter().last().unwrap_or(&0.0);
        let older_avg: f32 = self.cpu_history.iter().take(5).sum::<f32>() / 5.0;

        if recent_avg > older_avg + 5.0 {
            "↑"
        } else if recent_avg < older_avg - 5.0 {
            "↓"
        } else {
            "→"
        }
    }
}

/// Format bytes per second to human-readable string
fn format_bytes_per_sec(bps: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bps >= GB {
        format!("{:.1} GB/s", bps as f64 / GB as f64)
    } else if bps >= MB {
        format!("{:.1} MB/s", bps as f64 / MB as f64)
    } else if bps >= KB {
        format!("{:.1} KB/s", bps as f64 / KB as f64)
    } else {
        format!("{} B/s", bps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_core_data_creation() {
        let core = CpuCoreData::new(0, 45.5, 3200);
        assert_eq!(core.core_id, 0);
        assert!((core.usage_pct - 45.5).abs() < 0.01);
        assert_eq!(core.frequency_mhz, 3200);
    }

    #[test]
    fn test_cpu_core_data_default() {
        let core = CpuCoreData::default();
        assert_eq!(core.core_id, 0);
        assert_eq!(core.usage_pct, 0.0);
        assert_eq!(core.frequency_mhz, 0);
    }

    #[test]
    fn test_cpu_core_usage_color() {
        assert_eq!(CpuCoreData::new(0, 25.0, 3000).usage_color(), "green");
        assert_eq!(CpuCoreData::new(0, 60.0, 3000).usage_color(), "yellow");
        assert_eq!(CpuCoreData::new(0, 80.0, 3000).usage_color(), "red");
    }

    #[test]
    fn test_cpu_core_format_usage() {
        let core = CpuCoreData::new(0, 75.5, 3000);
        assert!(core.format_usage().contains("75.5%"));
    }

    #[test]
    fn test_cpu_core_format_frequency() {
        assert!(CpuCoreData::new(0, 50.0, 3500)
            .format_frequency()
            .contains("3.5 GHz"));
        assert!(CpuCoreData::new(0, 50.0, 800)
            .format_frequency()
            .contains("800 MHz"));
    }

    #[test]
    fn test_disk_data_creation() {
        let disk = DiskData::new("C:".to_string(), "/".to_string(), 500, 250, 1024, 2048);
        assert_eq!(disk.name, "C:");
        assert_eq!(disk.total_gb, 500);
        assert_eq!(disk.used_gb, 250);
        assert!((disk.usage_pct - 50.0).abs() < 0.01);
        assert_eq!(disk.read_bps, 1024);
        assert_eq!(disk.write_bps, 2048);
    }

    #[test]
    fn test_disk_data_usage_calculation() {
        let disk = DiskData::new("sda1".to_string(), "/".to_string(), 1000, 750, 0, 0);
        assert!((disk.usage_pct - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_disk_data_usage_zero_total() {
        let disk = DiskData::new("empty".to_string(), "/".to_string(), 0, 0, 0, 0);
        assert!((disk.usage_pct - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_disk_data_default() {
        let disk = DiskData::default();
        assert!(disk.name.is_empty());
        assert_eq!(disk.total_gb, 0);
        assert_eq!(disk.usage_pct, 0.0);
    }

    #[test]
    fn test_disk_data_usage_color() {
        assert_eq!(
            DiskData::new("test".to_string(), "/".to_string(), 100, 50, 0, 0).usage_color(),
            "green"
        );
        assert_eq!(
            DiskData::new("test".to_string(), "/".to_string(), 100, 75, 0, 0).usage_color(),
            "yellow"
        );
        assert_eq!(
            DiskData::new("test".to_string(), "/".to_string(), 100, 90, 0, 0).usage_color(),
            "red"
        );
    }

    #[test]
    fn test_disk_data_format_space() {
        let disk = DiskData::new("test".to_string(), "/".to_string(), 500, 250, 0, 0);
        assert_eq!(disk.format_space(), "250 / 500 GB");
    }

    #[test]
    fn test_network_throughput_creation() {
        let net = NetworkThroughput::new("eth0".to_string(), 1024 * 1024, 512 * 1024);
        assert_eq!(net.interface, "eth0");
        assert_eq!(net.rx_bps, 1024 * 1024);
        assert_eq!(net.tx_bps, 512 * 1024);
    }

    #[test]
    fn test_network_throughput_default() {
        let net = NetworkThroughput::default();
        assert!(net.interface.is_empty());
        assert_eq!(net.rx_bps, 0);
        assert_eq!(net.tx_bps, 0);
    }

    #[test]
    fn test_network_throughput_total_bps() {
        let net = NetworkThroughput::new("eth0".to_string(), 100, 200);
        assert_eq!(net.total_bps(), 300);
    }

    #[test]
    fn test_network_throughput_total_bps_overflow() {
        let net = NetworkThroughput::new("eth0".to_string(), u64::MAX, u64::MAX);
        assert!(net.total_bps() > 0);
    }

    #[test]
    fn test_perf_state_creation() {
        let state = PerfState::new();
        assert!(state.cpu_cores.is_empty());
        assert!(state.disks.is_empty());
        assert!(state.network.is_empty());
        assert_eq!(state.total_cpu_usage, 0.0);
        assert!(state.cpu_history.is_empty());
        assert!(state.memory_history.is_empty());
        assert!(state.last_update.is_none());
    }

    #[test]
    fn test_perf_state_default() {
        let state = PerfState::default();
        assert!(state.cpu_cores.is_empty());
    }

    #[test]
    fn test_perf_state_update() {
        let mut state = PerfState::new();
        let cores = vec![CpuCoreData::new(0, 50.0, 3000)];
        let disks = vec![DiskData::new(
            "C:".to_string(),
            "/".to_string(),
            500,
            250,
            0,
            0,
        )];
        let network = vec![NetworkThroughput::new("eth0".to_string(), 1024, 512)];

        state.update(
            cores,
            disks,
            network,
            50.0,
            60.0,
            8 * 1024 * 1024 * 1024,
            16 * 1024 * 1024 * 1024,
        );

        assert_eq!(state.cpu_cores.len(), 1);
        assert_eq!(state.disks.len(), 1);
        assert_eq!(state.network.len(), 1);
        assert!((state.total_cpu_usage - 50.0).abs() < 0.01);
        assert!(state.last_update.is_some());
        assert_eq!(state.cpu_history.len(), 1);
        assert_eq!(state.memory_history.len(), 1);
    }

    #[test]
    fn test_perf_state_has_data() {
        let state = PerfState::new();
        assert!(!state.has_data());

        let mut state = PerfState::new();
        state.update(vec![], vec![], vec![], 0.0, 0.0, 0, 0);
        assert!(state.has_data());
    }

    #[test]
    fn test_perf_state_avg_cpu_usage() {
        let mut state = PerfState::new();
        assert!((state.avg_cpu_usage() - 0.0).abs() < 0.01);

        state.cpu_cores = vec![
            CpuCoreData::new(0, 40.0, 3000),
            CpuCoreData::new(1, 60.0, 3000),
        ];
        assert!((state.avg_cpu_usage() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_perf_state_total_network_throughput() {
        let mut state = PerfState::new();
        state.network = vec![
            NetworkThroughput::new("eth0".to_string(), 100, 50),
            NetworkThroughput::new("eth1".to_string(), 200, 100),
        ];
        assert_eq!(state.total_network_throughput(), 450);
    }

    #[test]
    fn test_perf_state_total_disk_speeds() {
        let mut state = PerfState::new();
        state.disks = vec![
            DiskData::new("sda".to_string(), "/".to_string(), 100, 50, 1000, 500),
            DiskData::new("sdb".to_string(), "/data".to_string(), 200, 100, 2000, 1000),
        ];
        assert_eq!(state.total_disk_read(), 3000);
        assert_eq!(state.total_disk_write(), 1500);
    }

    #[test]
    fn test_perf_state_format_memory() {
        let mut state = PerfState::new();
        state.memory_used = 8 * 1024 * 1024 * 1024; // 8 GB
        state.memory_total = 16 * 1024 * 1024 * 1024; // 16 GB

        let formatted = state.format_memory();
        assert!(formatted.contains("8.0"));
        assert!(formatted.contains("16.0"));
    }

    #[test]
    fn test_perf_state_sparkline() {
        let mut state = PerfState::new();
        assert_eq!(state.cpu_sparkline(), "");
        assert_eq!(state.memory_sparkline(), "");

        // Add some history
        state.update(vec![], vec![], vec![], 20.0, 30.0, 0, 0);
        state.update(vec![], vec![], vec![], 40.0, 50.0, 0, 0);
        state.update(vec![], vec![], vec![], 60.0, 70.0, 0, 0);

        let cpu_spark = state.cpu_sparkline();
        let mem_spark = state.memory_sparkline();
        assert!(!cpu_spark.is_empty());
        assert!(!mem_spark.is_empty());
        assert!(cpu_spark.contains('▂') || cpu_spark.contains('▁') || cpu_spark.contains('▃'));
    }

    #[test]
    fn test_perf_state_history_limit() {
        let mut state = PerfState::new();

        // Add more than MAX_HISTORY entries
        for i in 0..=MAX_HISTORY {
            state.update(vec![], vec![], vec![], i as f32, i as f64, 0, 0);
        }

        // History should be capped at MAX_HISTORY
        assert_eq!(state.cpu_history.len(), MAX_HISTORY);
        assert_eq!(state.memory_history.len(), MAX_HISTORY);
    }

    #[test]
    fn test_perf_state_cpu_trend() {
        let mut state = PerfState::new();
        assert_eq!(state.cpu_trend(), "→"); // No history

        // Add increasing values (need enough history to see trend)
        for _ in 0..10 {
            state.update(vec![], vec![], vec![], 60.0, 50.0, 0, 0);
        }
        assert_eq!(state.cpu_trend(), "→"); // Stable at 60%

        // Add decreasing values (need enough to shift trend)
        for _ in 0..10 {
            state.update(vec![], vec![], vec![], 20.0, 50.0, 0, 0);
        }
        assert_eq!(state.cpu_trend(), "↓"); // Trending down (recent 20 vs older 60)

        // Add increasing values again
        for _ in 0..10 {
            state.update(vec![], vec![], vec![], 80.0, 50.0, 0, 0);
        }
        assert_eq!(state.cpu_trend(), "↑"); // Trending up (recent 80 vs older 20)
    }

    #[test]
    fn test_format_bytes_per_sec() {
        assert_eq!(format_bytes_per_sec(500), "500 B/s");
        assert!(format_bytes_per_sec(1024).contains("KB/s"));
        assert!(format_bytes_per_sec(1024 * 1024).contains("MB/s"));
        assert!(format_bytes_per_sec(1024 * 1024 * 1024).contains("GB/s"));
        assert_eq!(format_bytes_per_sec(0), "0 B/s");
    }

    #[test]
    fn test_clone() {
        let core = CpuCoreData::new(0, 50.0, 3000);
        let cloned = core.clone();
        assert_eq!(core.core_id, cloned.core_id);

        let disk = DiskData::new("C:".to_string(), "/".to_string(), 500, 250, 100, 200);
        let cloned = disk.clone();
        assert_eq!(disk.name, cloned.name);

        let net = NetworkThroughput::new("eth0".to_string(), 100, 200);
        let cloned = net.clone();
        assert_eq!(net.interface, cloned.interface);

        let mut state = PerfState::new();
        state.cpu_cores = vec![CpuCoreData::new(0, 50.0, 3000)];
        let cloned = state.clone();
        assert_eq!(state.cpu_cores.len(), cloned.cpu_cores.len());
    }
}
