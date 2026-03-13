//! Status command handler
//!
//! Display node status including connection, block height, sync status, and peer count.

use crate::config::Config;
use crate::handlers::format_uptime;
use crate::rpc::RpcClient;
use anyhow::Result;
use colored::Colorize;

/// Execute status command - show node status
pub async fn execute(config: &Config) -> Result<()> {
    println!("{}", "MonadNode Manager - Node Status".cyan().bold());
    println!("{}", "================================".cyan());
    println!();

    let rpc = RpcClient::new(config.rpc_endpoint())?;

    // Check connection
    match rpc.check_connection().await {
        Ok(true) => println!("{} Node is responding", "✓".green()),
        Ok(false) => println!("{} Node not responding", "✗".red()),
        Err(e) => println!("{} Connection error: {}", "✗".red(), e),
    }

    // Get block number
    match rpc.get_block_number().await {
        Ok(block) => println!("{} Block Height: {}", "■".blue(), block),
        Err(e) => println!("{} Could not fetch block height: {}", "!".yellow(), e),
    }

    // Get sync status
    match rpc.get_sync_status().await {
        Ok(syncing) => {
            if syncing {
                println!("{} Syncing...", "⟳".yellow());
            } else {
                println!("{} Synced", "✓".green());
            }
        }
        Err(e) => println!("{} Could not check sync status: {}", "!".yellow(), e),
    }

    // Get peer count from Prometheus metrics
    match rpc.get_peer_count_prometheus().await {
        Ok(peers) => println!("{} Peers: {}", "●".blue(), peers),
        Err(_) => {
            // Skip silently if metrics not available
        }
    }

    // Get node info from Prometheus metrics (version, uptime)
    match rpc.get_node_info_prometheus().await {
        Ok(node_info) => {
            if let Some(version) = &node_info.version {
                println!("{} Version: {}", "◆".magenta(), version);
            }
            if let Some(uptime_secs) = node_info.uptime_seconds {
                let uptime_human = format_uptime(uptime_secs);
                println!("{} Uptime: {}", "◆".magenta(), uptime_human);
            }
        }
        Err(_) => {
            // Skip silently if metrics not available
        }
    }

    // Show network info
    println!();
    println!("Network: {}", config.network());
    println!("RPC Endpoint: {}", config.rpc_endpoint());

    Ok(())
}
