//! CLI argument definitions using clap

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

use super::commands::Commands;

/// Monad Validator Manager - Professional CLI tool for Monad blockchain validator management
#[derive(Parser, Debug)]
#[command(
    name = "monad-val-manager",
    author,
    version,
    about,
    long_about = "A comprehensive CLI tool for managing Monad blockchain validators.\n\n\
                  Features:\n\
                  • Real-time TUI dashboard for monitoring\n\
                  • Smart diagnostics (Doctor)\n\
                  • Performance benchmarking and optimization\n\
                  • Staking operations\n\n\
                  Run without arguments to launch the TUI dashboard."
)]
pub struct Cli {
    /// Network to connect to (mainnet or testnet)
    #[arg(short, long, value_enum, global = true, default_value = "mainnet")]
    pub network: Network,

    /// Path to configuration file
    #[arg(short, long, global = true, env = "MONAD_CONFIG")]
    pub config: Option<PathBuf>,

    /// RPC endpoint URL (overrides config)
    #[arg(short, long, global = true, env = "MONAD_RPC_URL")]
    pub rpc: Option<String>,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Network selection
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Network {
    /// Mainnet (production network)
    Mainnet,
    /// Testnet (testing network)
    Testnet,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::try_parse_from(["monad-val-manager", "--network", "testnet"]);
        assert!(cli.is_ok());
        if let Ok(cli) = cli {
            assert!(matches!(cli.network, Network::Testnet));
        }
    }

    #[test]
    fn test_default_network() {
        let cli = Cli::try_parse_from(["monad-val-manager"]);
        assert!(cli.is_ok());
        if let Ok(cli) = cli {
            assert!(matches!(cli.network, Network::Mainnet));
        }
    }
}
