//! Init command handler
//!
//! Interactive first-time setup wizard for MonadNode Manager configuration.

use crate::cli::Network as CliNetwork;
use crate::config::{set_private_key, Config, Network};
use crate::staking::{LocalSigner, Signer};
use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

/// Execute init command - interactive first-time setup wizard
pub async fn execute() -> Result<()> {
    println!("{}", "MonadNode Manager - Setup Wizard".cyan().bold());
    println!("{}", "================================".cyan());
    println!();
    println!("This wizard will help you configure MonadNode Manager for your node.");
    println!();

    // Prompt for RPC URL
    let rpc_url = prompt_rpc_url()?;

    // Prompt for Metrics URL
    let metrics_url = prompt_metrics_url()?;

    // Prompt for network selection
    let (network_type, chain_id) = prompt_network_selection()?;

    // Create and save configuration
    let config = create_config(&rpc_url, &metrics_url, network_type, chain_id)?;
    config.save()?;

    print_configuration_summary(&config, &rpc_url, &metrics_url, chain_id, network_type);

    // Staking configuration
    configure_staking_interactive(network_type)?;

    print_next_steps();

    Ok(())
}

/// Prompt user for RPC endpoint URL
fn prompt_rpc_url() -> Result<String> {
    print!("RPC Endpoint URL [http://localhost:8080]: ");
    io::stdout().flush()?;

    let mut rpc_url = String::new();
    io::stdin().read_line(&mut rpc_url)?;
    let rpc_url = rpc_url.trim();

    let rpc_url = if rpc_url.is_empty() {
        "http://localhost:8080".to_string()
    } else {
        rpc_url.to_string()
    };

    Ok(rpc_url)
}

/// Prompt user for OTEL Prometheus metrics URL
fn prompt_metrics_url() -> Result<String> {
    println!();
    println!("OTel Collector metrics endpoint (for StateSync, BlockSync, MPT checks)");
    print!("Metrics URL [http://localhost:8889/metrics]: ");
    io::stdout().flush()?;

    let mut metrics_url = String::new();
    io::stdin().read_line(&mut metrics_url)?;
    let metrics_url = metrics_url.trim();

    let metrics_url = if metrics_url.is_empty() {
        "http://localhost:8889/metrics".to_string()
    } else {
        metrics_url.to_string()
    };

    Ok(metrics_url)
}

/// Prompt user for network selection and return network type with chain ID
fn prompt_network_selection() -> Result<(Network, u64)> {
    println!();
    println!("Select network:");
    println!("  1) mainnet (Chain ID: 143)");
    println!("  2) testnet (Chain ID: 10143)");
    print!("Network [1]: ");
    io::stdout().flush()?;

    let mut network_choice = String::new();
    io::stdin().read_line(&mut network_choice)?;
    let network_choice = network_choice.trim();

    let cli_network = match network_choice {
        "2" | "testnet" => CliNetwork::Testnet,
        _ => CliNetwork::Mainnet,
    };

    let network_type = Network::from(cli_network);
    let chain_id = match network_type {
        Network::Mainnet => 143,
        Network::Testnet => 10143,
    };

    Ok((network_type, chain_id))
}

/// Create configuration from user inputs
fn create_config(
    rpc_url: &str,
    metrics_url: &str,
    network_type: Network,
    chain_id: u64,
) -> Result<Config> {
    let config = Config {
        network: crate::config::NetworkConfig {
            network_type,
            chain_id,
        },
        rpc: crate::config::RpcConfig {
            http_url: rpc_url.to_string(),
            ws_url: rpc_url.replace("http://", "ws://"),
            metrics_url: metrics_url.to_string(),
            timeout: 30,
            max_retries: 3,
        },
        staking: crate::config::StakingConfig::default(),
    };

    Ok(config)
}

/// Print configuration summary after saving
fn print_configuration_summary(
    config: &Config,
    rpc_url: &str,
    metrics_url: &str,
    chain_id: u64,
    _network_type: Network,
) {
    println!();
    println!("{}", "Configuration saved successfully!".green().bold());
    println!();
    println!("Configuration file: {}", config.config_path().display());
    println!("Network: {} (Chain ID: {})", config.network(), chain_id);
    println!("RPC Endpoint: {}", rpc_url);
    println!("Metrics URL: {}", metrics_url);
}

/// Interactive staking configuration
fn configure_staking_interactive(network_type: Network) -> Result<()> {
    println!();
    println!("{}", "Staking Configuration".cyan().bold());
    println!("{}", "--------------------".cyan());

    let network_name = match network_type {
        Network::Mainnet => "mainnet",
        Network::Testnet => "testnet",
    };

    print!("Configure staking for {}? [y/N]: ", network_name);
    io::stdout().flush()?;

    let mut staking_choice = String::new();
    io::stdin().read_line(&mut staking_choice)?;
    let staking_choice = staking_choice.trim().to_lowercase();

    if staking_choice == "y" || staking_choice == "yes" {
        prompt_and_validate_private_key(network_type, network_name)?;
    }

    Ok(())
}

/// Prompt for private key and validate it
fn prompt_and_validate_private_key(network_type: Network, network_name: &str) -> Result<()> {
    loop {
        let private_key = rpassword::prompt_password("Enter private key (input hidden): ")
            .map_err(|e| anyhow::anyhow!("Failed to read private key: {}", e))?;

        let private_key = private_key.trim();

        // Validate private key by attempting to create a signer
        match LocalSigner::from_private_key(private_key) {
            Ok(signer) => {
                // Save to .env file
                set_private_key(network_type, private_key)?;

                println!();
                println!(
                    "{} Private key saved to .env for {}",
                    "OK".green().bold(),
                    network_name
                );
                println!("{} Associated address: {}", ">>".cyan(), signer.address());
                break;
            }
            Err(e) => {
                println!();
                println!("{} Invalid private key: {}", "X".red(), e);
                println!("{}", "Please try again or press Ctrl+C to skip.".yellow());

                if !prompt_retry()? {
                    println!("{}", "Skipping staking configuration.".yellow());
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Ask user if they want to retry entering private key
fn prompt_retry() -> Result<bool> {
    print!("Try again? [Y/n]: ");
    io::stdout().flush()?;

    let mut retry_choice = String::new();
    io::stdin().read_line(&mut retry_choice)?;
    let retry_choice = retry_choice.trim().to_lowercase();

    Ok(retry_choice != "n" && retry_choice != "no")
}

/// Print next steps for the user
fn print_next_steps() {
    println!();
    println!("Next steps:");
    println!(
        "  {} Run 'status' to check your node connection",
        "1.".cyan()
    );
    println!("  {} Run 'doctor' to diagnose any issues", "2.".cyan());
    println!(
        "  {} Run without arguments to launch the TUI dashboard",
        "3.".cyan()
    );
}
