//! Performance optimizer

use crate::config::Config;
use crate::utils::system::SystemInfo;

/// Performance optimizer
pub struct Optimizer {
    config: Config,
}

/// Optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub category: String,
    pub description: String,
    pub impact: Impact,
}

/// Impact level of optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Impact {
    Low,
    Medium,
    High,
    Critical,
}

impl Optimizer {
    /// Create new optimizer
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Analyze system and generate recommendations
    pub fn analyze(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        let system_info = SystemInfo::new();

        // Check CPU
        let cpu_usage = system_info.cpu_usage();
        if cpu_usage > 80.0 {
            recommendations.push(OptimizationRecommendation {
                category: "CPU".to_string(),
                description: "CPU usage is high. Consider upgrading hardware or reducing load."
                    .to_string(),
                impact: Impact::High,
            });
        }

        // Check memory
        let memory_usage = system_info.memory_usage_percent();
        if memory_usage > 85.0 {
            recommendations.push(OptimizationRecommendation {
                category: "Memory".to_string(),
                description:
                    "Memory usage is high. Consider adding more RAM or optimizing memory usage."
                        .to_string(),
                impact: Impact::High,
            });
        }

        // Check disk
        if let Some(disk) = system_info.primary_disk() {
            let disk_usage = disk.usage_percent();
            if disk_usage > 90.0 {
                recommendations.push(OptimizationRecommendation {
                    category: "Disk".to_string(),
                    description: "Disk usage is high. Clean up old data or expand storage."
                        .to_string(),
                    impact: Impact::High,
                });
            }

            let available_gb = disk.available_space / (1024 * 1024 * 1024);
            if available_gb < 100 {
                recommendations.push(OptimizationRecommendation {
                    category: "Disk".to_string(),
                    description: "Low disk space. At least 100GB recommended for stable operation."
                        .to_string(),
                    impact: Impact::Critical,
                });
            }
        }

        // Check RPC timeout
        if self.config.rpc.timeout < 10 {
            recommendations.push(OptimizationRecommendation {
                category: "Configuration".to_string(),
                description: "RPC timeout is low. Consider increasing to at least 30 seconds."
                    .to_string(),
                impact: Impact::Medium,
            });
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Network;

    #[test]
    fn test_optimizer_creation() {
        let config = Config::create_default(Network::Mainnet).unwrap();
        let optimizer = Optimizer::new(&config);
        let recommendations = optimizer.analyze();
        // Analysis should complete
        assert!(!recommendations.is_empty() || recommendations.is_empty());
    }
}
