//! Config show command handler
//!
//! Display current configuration.

use crate::config::Config;
use anyhow::Result;
use colored::Colorize;

/// Execute config show command
pub async fn execute(config: &Config) -> Result<()> {
    println!("{}", "MonadNode Manager - Configuration".cyan().bold());
    println!("{}", "==================================".cyan());
    println!();
    println!("Configuration file: {}", config.config_path().display());
    println!("Network: {}", config.network());
    println!("RPC Endpoint: {}", config.rpc_endpoint());
    println!("Metrics URL: {}", config.rpc.metrics_url);

    Ok(())
}
