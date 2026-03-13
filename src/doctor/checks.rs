//! Doctor diagnostic checks

use anyhow::Result;
use std::net::TcpStream;
use std::process::Command;
use std::time::Duration;

use crate::config::Config;
use crate::rpc::RpcClient;
use crate::utils::system::SystemInfo;

/// Parse human-readable size to GB (e.g., "1.8T" -> 1843.2 GB, "75G" -> 75.0 GB)
pub fn parse_size_to_gb(size: &str) -> f64 {
    let size_lower = size.trim().to_lowercase();

    // Find where the numeric part ends (first non-digit, non-dot character)
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

/// Helper to fetch Prometheus metrics value
fn fetch_prometheus_metric(url: &str, metric_name: &str) -> Option<String> {
    match std::process::Command::new("curl")
        .args(["-s", "--max-time", "2", url])
        .output()
    {
        Ok(output) => {
            let metrics = String::from_utf8_lossy(&output.stdout);
            // Parse Prometheus format: metric_name{labels} value
            for line in metrics.lines() {
                if line.starts_with(metric_name) {
                    // Extract value (after the last space)
                    if let Some(idx) = line.rfind(' ') {
                        return Some(line[idx + 1..].trim().to_string());
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
}

/// Monad services to check
const MONAD_SERVICES: &[&str] = &[
    "monad-bft",
    "monad-execution",
    "monad-rpc",
    "monad-archiver",
];

/// Optional services (won't fail the check if not running)
const OPTIONAL_SERVICES: &[&str] = &["monad-archiver"];

/// Doctor - Smart diagnostics engine
pub struct Doctor {
    config: Config,
}

/// Doctor report containing all check results
#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub checks: Vec<Check>,
    pub issues: Vec<String>,
    pub fixable_issues: Vec<String>,
    pub passed: usize,
    pub failed: usize,
}

/// Individual check result
#[derive(Debug, Clone)]
pub struct Check {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

impl Doctor {
    /// Create new Doctor instance
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Run all diagnostics
    pub async fn run_diagnostics(&self) -> Result<DoctorReport> {
        let mut checks = Vec::new();
        let mut issues = Vec::new();
        let mut fixable_issues = Vec::new();

        // System checks
        let system_info = SystemInfo::new();
        for req_check in system_info.check_requirements() {
            // Special message formatting for disk space to show both recommendation and minimum
            let message = if req_check.name == "Available Disk Space" {
                if req_check.passed {
                    format!(
                        "{} (recommended: {}, min: 500GB)",
                        req_check.current, req_check.recommended
                    )
                } else {
                    format!(
                        "{} - below 500GB minimum (recommended: {})",
                        req_check.current, req_check.recommended
                    )
                }
            } else if req_check.passed {
                format!(
                    "{} (recommended: {})",
                    req_check.current, req_check.recommended
                )
            } else {
                format!(
                    "{} - below recommended {}",
                    req_check.current, req_check.recommended
                )
            };

            let check = Check {
                name: req_check.name.clone(),
                passed: req_check.passed,
                message,
            };
            if !check.passed {
                // Only add issue if below 500GB minimum, not just below 2TB recommendation
                if req_check.name != "Available Disk Space" || !req_check.passed {
                    issues.push(format!("{} below 500GB minimum", req_check.name));
                }
            }
            checks.push(check);
        }

        // Network connectivity check
        let rpc_check = self.check_rpc_connection().await;
        if !rpc_check.passed {
            issues.push("Cannot connect to RPC endpoint".to_string());
            fixable_issues.push("rpc_connection".to_string());
        }
        checks.push(rpc_check);

        // Port check
        let port_check = self.check_rpc_port();
        if !port_check.passed {
            issues.push("RPC port not open".to_string());
        }
        checks.push(port_check);

        // Disk space check
        let disk_check = self.check_disk_space();
        if !disk_check.passed {
            issues.push("Low disk space".to_string());
        }
        checks.push(disk_check);

        // Memory check
        let memory_check = self.check_memory();
        if !memory_check.passed {
            issues.push("High memory usage".to_string());
        }
        checks.push(memory_check);

        // Monad services check (Linux only)
        if cfg!(target_os = "linux") {
            for service_check in self.check_monad_services() {
                if !service_check.passed {
                    issues.push(format!("{} service not running", service_check.name));
                }
                checks.push(service_check);
            }

            // OTEL collector check
            let otel_check = self.check_otel_collector();
            if !otel_check.passed {
                issues.push("OTEL collector not running".to_string());
            }
            checks.push(otel_check);
        }

        // Sync status check via Prometheus
        let sync_check = self.check_sync_status().await;
        if !sync_check.passed {
            issues.push("Node is not synced".to_string());
        }
        checks.push(sync_check);

        // New checks from script analysis

        // Log errors check (Linux only - requires journalctl)
        if cfg!(target_os = "linux") {
            let log_errors_check = self.check_log_errors();
            if !log_errors_check.passed {
                issues.push("Critical errors found in logs".to_string());
            }
            checks.push(log_errors_check);
        }

        // StateSync stuck check
        let statesync_stuck_check = self.check_statesync_stuck().await;
        if !statesync_stuck_check.passed {
            issues.push("StateSync appears stuck".to_string());
        }
        checks.push(statesync_stuck_check);

        // BlockSync health check (Linux only)
        if cfg!(target_os = "linux") {
            let blocksync_check = self.check_blocksync_health();
            if !blocksync_check.passed {
                issues.push("BlockSync experiencing timeouts".to_string());
            }
            checks.push(blocksync_check);
        }

        // MPT capacity check (Linux only)
        if cfg!(target_os = "linux") {
            let mpt_check = self.check_mpt_capacity();
            if !mpt_check.passed {
                issues.push("MPT storage capacity warning".to_string());
            }
            checks.push(mpt_check);
        }

        // Count results
        let passed = checks.iter().filter(|c| c.passed).count();
        let failed = checks.len() - passed;

        Ok(DoctorReport {
            checks,
            issues,
            fixable_issues,
            passed,
            failed,
        })
    }

    /// Check RPC connection
    async fn check_rpc_connection(&self) -> Check {
        let endpoint = self.config.rpc_endpoint();

        // Parse URL to extract host and port
        let url = url::Url::parse(endpoint);
        match url {
            Ok(parsed) => {
                let host = parsed.host_str().unwrap_or("localhost");
                let port = parsed.port().unwrap_or(8080);

                match tokio::net::TcpStream::connect((host, port)).await {
                    Ok(_) => Check {
                        name: "RPC Connection".to_string(),
                        passed: true,
                        message: format!("Connected to {}:{}", host, port),
                    },
                    Err(e) => Check {
                        name: "RPC Connection".to_string(),
                        passed: false,
                        message: format!("Failed to connect: {}", e),
                    },
                }
            }
            Err(_) => Check {
                name: "RPC Connection".to_string(),
                passed: false,
                message: "Invalid RPC URL".to_string(),
            },
        }
    }

    /// Check if RPC port is open
    fn check_rpc_port(&self) -> Check {
        let addr = "127.0.0.1:8080";
        match TcpStream::connect_timeout(
            &addr.parse().expect("Valid address"),
            Duration::from_secs(5),
        ) {
            Ok(_) => Check {
                name: "RPC Port".to_string(),
                passed: true,
                message: "Port 8080 is open".to_string(),
            },
            Err(_) => Check {
                name: "RPC Port".to_string(),
                passed: false,
                message: "Port 8080 is not accessible".to_string(),
            },
        }
    }

    /// Check available disk space via df -h (OS disk, not triedb)
    fn check_disk_space(&self) -> Check {
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

                        // Parse available space to GB for threshold check
                        let avail_gb = parse_size_to_gb(avail);
                        let passed = avail_gb >= 500.0; // Fail if less than 500GB available

                        return Check {
                            name: "Disk Space (OS)".to_string(),
                            passed,
                            message: format!(
                                "{} total, {} used, {} available ({}) - Threshold: 500GB",
                                total, used, avail, percent
                            ),
                        };
                    }
                }

                Check {
                    name: "Disk Space (OS)".to_string(),
                    passed: false,
                    message: "Could not parse df output".to_string(),
                }
            }
            Err(_) => Check {
                name: "Disk Space (OS)".to_string(),
                passed: false,
                message: "df command failed".to_string(),
            },
        }
    }

    /// Check memory usage
    fn check_memory(&self) -> Check {
        let system_info = SystemInfo::new();
        let usage_percent = system_info.memory_usage_percent();
        let passed = usage_percent < 90.0; // Less than 90% used

        Check {
            name: "Memory Usage".to_string(),
            passed,
            message: format!("{:.1}% used", usage_percent),
        }
    }

    /// Check Monad systemd services
    fn check_monad_services(&self) -> Vec<Check> {
        MONAD_SERVICES
            .iter()
            .map(|service| {
                let is_optional = OPTIONAL_SERVICES.contains(service);
                let output = Command::new("systemctl")
                    .args(["is-active", service])
                    .output();

                match output {
                    Ok(output) => {
                        let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        let active = status == "active";

                        // Optional services pass even if not running
                        let passed = active || is_optional;

                        Check {
                            name: format!("{} Service", service),
                            passed,
                            message: if active {
                                "running".to_string()
                            } else if is_optional {
                                format!("not running ({}) [optional]", status)
                            } else {
                                format!("not running ({})", status)
                            },
                        }
                    }
                    Err(e) => Check {
                        name: format!("{} Service", service),
                        passed: is_optional, // Optional services pass even on error
                        message: if is_optional {
                            format!("check failed: {} [optional]", e)
                        } else {
                            format!("check failed: {}", e)
                        },
                    },
                }
            })
            .collect()
    }

    /// Check OTEL collector service
    fn check_otel_collector(&self) -> Check {
        let output = Command::new("systemctl")
            .args(["is-active", "otelcol"])
            .output();

        match output {
            Ok(output) => {
                let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let active = status == "active";
                Check {
                    name: "OTEL Collector".to_string(),
                    passed: active,
                    message: if active {
                        "running".to_string()
                    } else {
                        format!("not running ({})", status)
                    },
                }
            }
            Err(e) => Check {
                name: "OTEL Collector".to_string(),
                passed: false,
                message: format!("check failed: {}", e),
            },
        }
    }

    /// Check sync status via RPC
    async fn check_sync_status(&self) -> Check {
        let client = match RpcClient::new(self.config.rpc_endpoint()) {
            Ok(c) => c,
            Err(e) => {
                return Check {
                    name: "Sync Status".to_string(),
                    passed: false,
                    message: format!("client error: {}", e),
                };
            }
        };

        match client.get_sync_status().await {
            Ok(is_syncing) => {
                if is_syncing {
                    Check {
                        name: "Sync Status".to_string(),
                        passed: false,
                        message: "syncing".to_string(),
                    }
                } else {
                    Check {
                        name: "Sync Status".to_string(),
                        passed: true,
                        message: "synced".to_string(),
                    }
                }
            }
            Err(e) => Check {
                name: "Sync Status".to_string(),
                passed: false,
                message: format!("check failed: {}", e),
            },
        }
    }

    /// Check journalctl for critical log errors (Linux only)
    fn check_log_errors(&self) -> Check {
        let output = Command::new("journalctl")
            .args(["-u", "monad-bft", "--since", "1min ago"])
            .output();

        match output {
            Ok(output) => {
                let logs = String::from_utf8_lossy(&output.stdout);

                // Check for critical errors
                let has_high_qc = logs.contains("high QC too far ahead");
                let has_assertion_failure = logs.contains("left == right");
                let has_exit = logs.contains("exited");

                if has_high_qc || has_assertion_failure || has_exit {
                    let mut errors = Vec::new();
                    if has_high_qc {
                        errors.push("BFT ahead of Execution");
                    }
                    if has_assertion_failure {
                        errors.push("Assertion failure");
                    }
                    if has_exit {
                        errors.push("Process exited");
                    }

                    Check {
                        name: "Log Errors".to_string(),
                        passed: false,
                        message: format!("Found: {}", errors.join(", ")),
                    }
                } else {
                    Check {
                        name: "Log Errors".to_string(),
                        passed: true,
                        message: "No critical errors in last 1min".to_string(),
                    }
                }
            }
            Err(e) => Check {
                name: "Log Errors".to_string(),
                passed: false,
                message: format!("Failed to check logs: {}", e),
            },
        }
    }

    /// Check StateSync stuck status via Prometheus metrics
    async fn check_statesync_stuck(&self) -> Check {
        let metrics_url = &self.config.rpc.metrics_url;

        // Check if StateSync is stuck (syncing = 1 for extended period)
        // For now, check if StateSync is syncing and has progress
        if let Some(syncing) = fetch_prometheus_metric(metrics_url, "monad_statesync_syncing") {
            if syncing == "1" {
                // StateSync is active, check progress
                if let Some(progress) =
                    fetch_prometheus_metric(metrics_url, "monad_statesync_progress_estimate")
                {
                    if let Some(target) =
                        fetch_prometheus_metric(metrics_url, "monad_statesync_last_target")
                    {
                        if target == "0" {
                            return Check {
                                name: "StateSync Stuck".to_string(),
                                passed: true,
                                message: "StateSync not running (node synced)".to_string(),
                            };
                        }

                        // If progress exists, StateSync is working
                        return Check {
                            name: "StateSync Stuck".to_string(),
                            passed: true,
                            message: format!("StateSync active, progress: {}%", progress),
                        };
                    }
                }
            } else {
                // Not syncing = already synced
                return Check {
                    name: "StateSync Stuck".to_string(),
                    passed: true,
                    message: "StateSync complete (node synced)".to_string(),
                };
            }
        }

        // Fallback: unable to query metrics
        Check {
            name: "StateSync Stuck".to_string(),
            passed: true, // Don't fail if metrics unavailable
            message: "Unable to verify (Prometheus metrics not available)".to_string(),
        }
    }

    /// Check BlockSync health via journalctl (Linux only)
    fn check_blocksync_health(&self) -> Check {
        let output = Command::new("journalctl")
            .args(["-u", "monad-bft", "--since", "1min ago"])
            .output();

        match output {
            Ok(output) => {
                let logs = String::from_utf8_lossy(&output.stdout);

                // Count timeout occurrences
                let timeout_count = logs.matches("header request timed out").count();
                let unavailable_count = logs.matches("headers response not available").count();

                let total_issues = timeout_count + unavailable_count;

                if total_issues > 10 {
                    Check {
                        name: "BlockSync Health".to_string(),
                        passed: false,
                        message: format!("{} timeouts/unavailable in last 1min", total_issues),
                    }
                } else {
                    Check {
                        name: "BlockSync Health".to_string(),
                        passed: true,
                        message: format!("{} sync issues in last 1min (acceptable)", total_issues),
                    }
                }
            }
            Err(e) => Check {
                name: "BlockSync Health".to_string(),
                passed: false,
                message: format!("Failed to check blocksync: {}", e),
            },
        }
    }

    /// Check MPT storage capacity via Prometheus metrics (with fallback)
    fn check_mpt_capacity(&self) -> Check {
        let metrics_url = &self.config.rpc.metrics_url;

        // Try Prometheus metrics first
        if let Some(storage_bytes) =
            fetch_prometheus_metric(metrics_url, "monad_triedb_storage_bytes")
        {
            if let Some(total_bytes) =
                fetch_prometheus_metric(metrics_url, "monad_triedb_capacity_bytes")
            {
                if let (Ok(used), Ok(total)) =
                    (storage_bytes.parse::<f64>(), total_bytes.parse::<f64>())
                {
                    if total > 0.0 {
                        let percentage = (used / total) * 100.0;
                        let used_gb = used / (1024.0 * 1024.0 * 1024.0);
                        let total_gb = total / (1024.0 * 1024.0 * 1024.0);

                        return Check {
                            name: "MPT Storage (Triedb)".to_string(),
                            passed: percentage < 90.0,
                            message: format!(
                                "{:.1}% used ({:.1} GB / {:.1} GB)",
                                percentage, used_gb, total_gb
                            ),
                        };
                    }
                }
            }
        }

        // Fallback: try monad-mpt command (improved parsing based on monad-status.sh)
        let output = Command::new("monad-mpt")
            .args(["--storage", "/dev/triedb"])
            .output();

        match output {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);

                // Parse monad-mpt output format:
                // "/dev/triedb:\n    1.80TiB 839GiB  1.73%"
                // or with sections:
                // "Fast: 0 chunks...\n    Slow: 63941 chunks, capacity 1.80TiB, used 839GiB\n    Free: ..."
                for line in output_str.lines() {
                    // Look for line with percentage (format: "capacity used %")
                    if line.contains('%') {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        // Expected format: capacity used percentage
                        // e.g., "1.80TiB" "839GiB" "1.73%"
                        if parts.len() >= 3 {
                            // Find the part with %
                            for part in &parts {
                                if part.ends_with('%') {
                                    let percent_str = part.trim_end_matches('%');
                                    if let Ok(percentage) = percent_str.parse::<f64>() {
                                        return Check {
                                            name: "MPT Storage (Triedb)".to_string(),
                                            passed: percentage < 90.0,
                                            message: format!("{:.1}% used", percentage),
                                        };
                                    }
                                }
                            }
                        }
                    }

                    // Alternative: Look for "used" keyword in line (e.g., "used 839GiB")
                    if line.contains("used") && line.contains('%') {
                        // Extract percentage from line like "used 839GiB 1.73%"
                        if let Some(percent_idx) = line.find('%') {
                            // Get text before % and extract number
                            let before_percent = &line[..percent_idx];
                            if let Some(last_space) = before_percent.rfind(' ') {
                                let percent_str = &before_percent[last_space + 1..];
                                if let Ok(percentage) = percent_str.parse::<f64>() {
                                    return Check {
                                        name: "MPT Storage (Triedb)".to_string(),
                                        passed: percentage < 90.0,
                                        message: format!("{:.1}% used", percentage),
                                    };
                                }
                            }
                        }
                    }
                }

                // If we have output but couldn't parse
                if !output_str.trim().is_empty() {
                    Check {
                        name: "MPT Storage (Triedb)".to_string(),
                        passed: true,
                        message: "Unable to parse (check monad-mpt manually)".to_string(),
                    }
                } else {
                    Check {
                        name: "MPT Storage (Triedb)".to_string(),
                        passed: true,
                        message: "MPT query unavailable (non-blocking)".to_string(),
                    }
                }
            }
            Err(_) => Check {
                name: "MPT Storage (Triedb)".to_string(),
                passed: true,
                message: "MPT query unavailable (non-blocking)".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Network;

    #[test]
    fn test_doctor_creation() {
        let config = Config::create_default(Network::Mainnet).unwrap();
        let _doctor = Doctor::new(&config);
        // Doctor created successfully
    }

    #[test]
    fn test_check_creation() {
        let check = Check {
            name: "Test Check".to_string(),
            passed: true,
            message: "Test message".to_string(),
        };
        assert_eq!(check.name, "Test Check");
        assert!(check.passed);
        assert_eq!(check.message, "Test message");
    }

    #[test]
    fn test_doctor_report_creation() {
        let report = DoctorReport {
            checks: vec![],
            issues: vec![],
            fixable_issues: vec![],
            passed: 0,
            failed: 0,
        };
        assert_eq!(report.passed, 0);
        assert_eq!(report.failed, 0);
    }

    #[test]
    fn test_log_errors_check_name() {
        let config = Config::create_default(Network::Mainnet).unwrap();
        let doctor = Doctor::new(&config);

        // This will fail on non-Linux systems, which is expected
        let check = doctor.check_log_errors();
        assert_eq!(check.name, "Log Errors");
    }

    #[test]
    fn test_statesync_stuck_check_name() {
        let config = Config::create_default(Network::Mainnet).unwrap();
        let doctor = Doctor::new(&config);

        // Create a runtime for the async test
        let rt = tokio::runtime::Runtime::new().unwrap();
        let check = rt.block_on(async { doctor.check_statesync_stuck().await });

        assert_eq!(check.name, "StateSync Stuck");
    }

    #[test]
    fn test_blocksync_health_check_name() {
        let config = Config::create_default(Network::Mainnet).unwrap();
        let doctor = Doctor::new(&config);

        // This will fail on non-Linux systems, which is expected
        let check = doctor.check_blocksync_health();
        assert_eq!(check.name, "BlockSync Health");
    }

    #[test]
    fn test_mpt_capacity_check_name() {
        let config = Config::create_default(Network::Mainnet).unwrap();
        let doctor = Doctor::new(&config);

        // This will fail on non-Linux systems, which is expected
        let check = doctor.check_mpt_capacity();
        assert_eq!(check.name, "MPT Storage (Triedb)");
    }
}
