//! System information utilities

use sysinfo::{Disks, System};

/// System information collector
pub struct SystemInfo {
    sys: System,
    disks: Disks,
    // Store previous network stats for throughput calculation (reserved for future use)
    _prev_rx_bytes: u64,
    _prev_tx_bytes: u64,
}

impl SystemInfo {
    /// Create new system info collector
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        Self {
            sys,
            disks,
            _prev_rx_bytes: 0,
            _prev_tx_bytes: 0,
        }
    }

    /// Refresh system information
    pub fn refresh(&mut self) {
        self.sys.refresh_all();
        self.disks.refresh(true);
    }

    /// Get CPU usage percentage
    pub fn cpu_usage(&self) -> f32 {
        self.sys.global_cpu_usage()
    }

    /// Get per-core CPU usage percentages
    pub fn cpu_core_usages(&self) -> Vec<f32> {
        self.sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect()
    }

    /// Get per-core CPU frequency in MHz
    pub fn cpu_core_frequencies(&self) -> Vec<u64> {
        self.sys.cpus().iter().map(|cpu| cpu.frequency()).collect()
    }

    /// Get CPU count
    pub fn cpu_count(&self) -> usize {
        self.sys.cpus().len()
    }

    /// Get total memory in bytes
    pub fn total_memory(&self) -> u64 {
        self.sys.total_memory()
    }

    /// Get used memory in bytes
    pub fn used_memory(&self) -> u64 {
        self.sys.used_memory()
    }

    /// Get available memory in bytes
    pub fn available_memory(&self) -> u64 {
        self.sys.available_memory()
    }

    /// Get memory usage percentage
    pub fn memory_usage_percent(&self) -> f64 {
        let total = self.total_memory() as f64;
        let used = self.used_memory() as f64;
        if total > 0.0 {
            (used / total) * 100.0
        } else {
            0.0
        }
    }

    /// Get total swap memory in bytes
    pub fn total_swap(&self) -> u64 {
        self.sys.total_swap()
    }

    /// Get used swap memory in bytes
    pub fn used_swap(&self) -> u64 {
        self.sys.used_swap()
    }

    /// Get swap usage percentage
    pub fn swap_usage_percent(&self) -> f64 {
        let total = self.total_swap() as f64;
        let used = self.used_swap() as f64;
        if total > 0.0 {
            (used / total) * 100.0
        } else {
            0.0
        }
    }

    /// Get disk information
    pub fn disks(&self) -> Vec<DiskInfo> {
        self.disks
            .iter()
            .map(|disk| DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
            })
            .collect()
    }

    /// Get primary disk info (root or C:)
    pub fn primary_disk(&self) -> Option<DiskInfo> {
        self.disks().into_iter().next()
    }

    /// Get system uptime in seconds
    pub fn uptime(&self) -> u64 {
        System::uptime()
    }

    /// Get network interface names (returns common interface names)
    ///
    /// Note: sysinfo doesn't provide network interface details.
    /// This returns platform-specific common interface names.
    /// For real interface detection, would need to parse /proc/net/dev on Linux
    /// or use Windows API calls.
    pub fn network_interfaces(&self) -> Vec<String> {
        // BUG-015: Return all common interface names for better coverage
        #[cfg(target_os = "windows")]
        {
            vec![
                "Ethernet".to_string(),
                "Wi-Fi".to_string(),
                "vEthernet (Default Switch)".to_string(),
                "Local Area Connection".to_string(),
            ]
        }
        #[cfg(not(target_os = "windows"))]
        {
            vec![
                "eth0".to_string(),
                "eth1".to_string(),
                "wlan0".to_string(),
                "wlan1".to_string(),
                "lo".to_string(),
            ]
        }
    }

    /// Get network throughput stats (placeholder - would need real implementation)
    ///
    /// BUG-015: This is a simplified implementation that returns zeros.
    /// Real throughput calculation requires:
    /// 1. Reading /proc/net/dev on Linux or Windows API
    /// 2. Calculating delta from previous reading
    /// 3. Dividing by time elapsed
    pub fn network_throughput(&self) -> (u64, u64) {
        // Returns (rx_bytes_per_sec, tx_bytes_per_sec)
        // For now, return 0 as we don't track network stats
        (0, 0)
    }

    /// Get system hostname
    pub fn hostname(&self) -> String {
        System::host_name().unwrap_or_else(|| "unknown".to_string())
    }

    /// Get OS name
    pub fn os_name(&self) -> String {
        System::name().unwrap_or_else(|| "unknown".to_string())
    }

    /// Get OS version
    pub fn os_version(&self) -> String {
        System::os_version().unwrap_or_else(|| "unknown".to_string())
    }

    /// Check if system meets minimum requirements for a Monad node
    pub fn check_requirements(&self) -> Vec<RequirementCheck> {
        let mut checks = Vec::new();

        // CPU check (16 cores recommended)
        let cpu_count = self.cpu_count();
        checks.push(RequirementCheck {
            name: "CPU Cores".to_string(),
            current: cpu_count.to_string(),
            recommended: "16+".to_string(),
            passed: cpu_count >= 16,
        });

        // Memory check (32GB recommended)
        let memory_gb = self.total_memory() / (1024 * 1024 * 1024);
        checks.push(RequirementCheck {
            name: "RAM".to_string(),
            current: format!("{} GB", memory_gb),
            recommended: "32+ GB".to_string(),
            passed: memory_gb >= 32,
        });

        // Disk space check (2TB foundation recommended, 500GB minimum threshold)
        if let Some(disk) = self.primary_disk() {
            let disk_gb = disk.available_space / (1024 * 1024 * 1024);
            checks.push(RequirementCheck {
                name: "Available Disk Space".to_string(),
                current: format!("{} GB", disk_gb),
                recommended: "2000+ GB".to_string(),
                passed: disk_gb >= 500, // Fail only if below 500GB
            });
        }

        checks
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Disk information
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
}

impl DiskInfo {
    /// Get disk usage percentage
    pub fn usage_percent(&self) -> f64 {
        let total = self.total_space as f64;
        let used = (self.total_space - self.available_space) as f64;
        if total > 0.0 {
            (used / total) * 100.0
        } else {
            0.0
        }
    }
}

/// Requirement check result
#[derive(Debug, Clone)]
pub struct RequirementCheck {
    pub name: String,
    pub current: String,
    pub recommended: String,
    pub passed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info_creation() {
        let info = SystemInfo::new();
        // Basic sanity checks
        assert!(info.cpu_count() > 0);
        assert!(info.total_memory() > 0);
    }

    #[test]
    fn test_memory_usage() {
        let info = SystemInfo::new();
        let usage = info.memory_usage_percent();
        assert!((0.0..=100.0).contains(&usage));
    }

    #[test]
    fn test_disk_info() {
        let info = SystemInfo::new();
        let disks = info.disks();
        // Should have at least one disk
        assert!(!disks.is_empty());
    }

    #[test]
    fn test_requirement_checks() {
        let info = SystemInfo::new();
        let checks = info.check_requirements();
        assert!(!checks.is_empty());
    }
}
