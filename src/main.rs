//! Monad Validator Manager Entry Point
//!
//! Professional CLI tool for Monad blockchain validator management.

use anyhow::Result;
use clap::Parser;
use monad_val_manager::cli::{Cli, Commands};
use monad_val_manager::config::Config;
use monad_val_manager::handlers;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Load configuration
    let config = Config::load(cli.network)?;

    // Execute command
    match cli.command {
        Some(Commands::Init) => {
            handlers::execute_init().await?;
        }
        Some(Commands::Status) => {
            handlers::execute_status(&config).await?;
        }
        Some(Commands::Balance { address }) => {
            handlers::execute_balance(&config, address).await?;
        }
        Some(Commands::Doctor) => {
            handlers::execute_doctor(&config).await?;
        }
        Some(Commands::ConfigShow) => {
            handlers::execute_config_show(&config).await?;
        }
        Some(Commands::Staking { command }) => {
            handlers::execute_staking(&config, command).await?;
        }
        Some(Commands::Transfer {
            address,
            amount,
            dry_run,
            yes,
        }) => {
            // Parse amount to wei
            let amount_wei = handlers::parse_transfer_amount(&amount)?;
            handlers::execute_transfer(&config, address, amount_wei, dry_run, yes).await?;
        }
        None => {
            // Default: Launch TUI dashboard
            handlers::execute_tui(&config).await?;
        }
    }

    Ok(())
}
