//! Performance benchmarks

use anyhow::Result;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::rpc::RpcClient;
use crate::utils::system::SystemInfo;

/// Benchmark runner
pub struct Benchmark {
    config: Config,
}

/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration: Duration,
    pub operations_per_second: f64,
    pub success: bool,
    pub error: Option<String>,
    pub details: Option<String>,
}

impl Benchmark {
    /// Create new benchmark runner
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Run all benchmarks
    pub async fn run_all(&self) -> Result<Vec<BenchmarkResult>> {
        let mut results = Vec::new();

        // RPC latency benchmark
        results.push(self.benchmark_rpc_latency().await);

        // Block fetch benchmark
        results.push(self.benchmark_block_fetch().await);

        // System resource benchmark
        results.push(self.benchmark_system_resources());

        // Network throughput benchmark
        results.push(self.benchmark_network_throughput().await);

        Ok(results)
    }

    /// Benchmark RPC latency
    pub async fn benchmark_rpc_latency(&self) -> BenchmarkResult {
        let name = "RPC Latency".to_string();
        let rpc = match RpcClient::new(self.config.rpc_endpoint()) {
            Ok(r) => r,
            Err(e) => {
                return BenchmarkResult {
                    name,
                    duration: Duration::ZERO,
                    operations_per_second: 0.0,
                    success: false,
                    error: Some(e.to_string()),
                    details: None,
                };
            }
        };

        let iterations = 10;
        let start = Instant::now();

        for _ in 0..iterations {
            if let Err(e) = rpc.get_block_number().await {
                return BenchmarkResult {
                    name,
                    duration: start.elapsed(),
                    operations_per_second: 0.0,
                    success: false,
                    error: Some(e.to_string()),
                    details: None,
                };
            }
        }

        let duration = start.elapsed();
        let ops_per_sec = iterations as f64 / duration.as_secs_f64();

        BenchmarkResult {
            name,
            duration,
            operations_per_second: ops_per_sec,
            success: true,
            error: None,
            details: Some(format!(
                "Average latency: {:.2}ms",
                duration.as_millis() as f64 / iterations as f64
            )),
        }
    }

    /// Benchmark block fetching
    pub async fn benchmark_block_fetch(&self) -> BenchmarkResult {
        let name = "Block Fetch".to_string();
        let rpc = match RpcClient::new(self.config.rpc_endpoint()) {
            Ok(r) => r,
            Err(e) => {
                return BenchmarkResult {
                    name,
                    duration: Duration::ZERO,
                    operations_per_second: 0.0,
                    success: false,
                    error: Some(e.to_string()),
                    details: None,
                };
            }
        };

        let start = Instant::now();

        match rpc.get_block_number().await {
            Ok(block) => {
                let duration = start.elapsed();
                BenchmarkResult {
                    name,
                    duration,
                    operations_per_second: 1.0 / duration.as_secs_f64(),
                    success: true,
                    error: None,
                    details: Some(format!("Current block: {}", block)),
                }
            }
            Err(e) => BenchmarkResult {
                name,
                duration: start.elapsed(),
                operations_per_second: 0.0,
                success: false,
                error: Some(e.to_string()),
                details: None,
            },
        }
    }

    /// Benchmark system resources
    pub fn benchmark_system_resources(&self) -> BenchmarkResult {
        let name = "System Resources".to_string();
        let start = Instant::now();

        let system_info = SystemInfo::new();
        let cpu_usage = system_info.cpu_usage();
        let memory_usage = system_info.memory_usage_percent();

        let duration = start.elapsed();

        BenchmarkResult {
            name,
            duration,
            operations_per_second: 0.0,
            success: true,
            error: None,
            details: Some(format!(
                "CPU: {:.1}% | Memory: {:.1}%",
                cpu_usage, memory_usage
            )),
        }
    }

    /// Benchmark network throughput
    pub async fn benchmark_network_throughput(&self) -> BenchmarkResult {
        let name = "Network Throughput".to_string();
        let start = Instant::now();

        // Simple network test - fetch multiple blocks
        let rpc = match RpcClient::new(self.config.rpc_endpoint()) {
            Ok(r) => r,
            Err(e) => {
                return BenchmarkResult {
                    name,
                    duration: Duration::ZERO,
                    operations_per_second: 0.0,
                    success: false,
                    error: Some(e.to_string()),
                    details: None,
                };
            }
        };

        // Test by making 5 consecutive calls
        let mut success_count = 0;
        for _ in 0..5 {
            if rpc.get_block_number().await.is_ok() {
                success_count += 1;
            }
        }

        let duration = start.elapsed();
        let success = success_count == 5;

        BenchmarkResult {
            name,
            duration,
            operations_per_second: success_count as f64 / duration.as_secs_f64(),
            success,
            error: if success {
                None
            } else {
                Some("Some requests failed".to_string())
            },
            details: Some(format!("{}/5 requests successful", success_count)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Network;

    #[test]
    fn test_benchmark_creation() {
        let config = Config::create_default(Network::Mainnet).unwrap();
        let _benchmark = Benchmark::new(&config);
        // Benchmark created successfully
    }
}
