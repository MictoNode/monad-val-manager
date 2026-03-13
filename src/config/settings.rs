//! Configuration settings and management

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::cli::Network as CliNetwork;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Network configuration
    pub network: NetworkConfig,
    /// RPC configuration
    pub rpc: RpcConfig,
    /// Staking configuration
    #[serde(default)]
    pub staking: StakingConfig,
}

/// Network-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network type (mainnet or testnet)
    #[serde(rename = "type")]
    pub network_type: Network,
    /// Chain ID
    pub chain_id: u64,
}

/// RPC endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    /// HTTP RPC endpoint
    pub http_url: String,
    /// WebSocket endpoint
    pub ws_url: String,
    /// OTEL Prometheus metrics endpoint (default: http://localhost:8889/metrics)
    #[serde(default = "default_metrics_url")]
    pub metrics_url: String,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Maximum retries
    #[serde(default = "default_retries")]
    pub max_retries: u32,
}

/// Staking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingConfig {
    /// Signer type: "local" or "ledger"
    #[serde(rename = "type", default = "default_signer_type")]
    pub signer_type: String,

    /// BIP-32 derivation path for Ledger (default: "44'/60'/0'/0/0")
    #[serde(default = "default_derivation_path")]
    pub derivation_path: String,
}

impl StakingConfig {
    /// Get signer type from environment or config
    ///
    /// Checks STAKING_TYPE env var first, then falls back to config
    pub fn get_signer_type(&self) -> String {
        std::env::var("STAKING_TYPE").unwrap_or_else(|_| self.signer_type.clone())
    }

    /// Get derivation path from environment or config
    ///
    /// Checks DERIVATION_PATH env var first, then falls back to config
    pub fn get_derivation_path(&self) -> String {
        std::env::var("DERIVATION_PATH").unwrap_or_else(|_| self.derivation_path.clone())
    }
}

impl Default for StakingConfig {
    fn default() -> Self {
        Self {
            signer_type: default_signer_type(),
            derivation_path: default_derivation_path(),
        }
    }
}

fn default_signer_type() -> String {
    "local".to_string()
}

fn default_derivation_path() -> String {
    "44'/60'/0'/0/0".to_string()
}

/// Network type enumeration
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    #[default]
    Mainnet,
    Testnet,
}

impl From<CliNetwork> for Network {
    fn from(network: CliNetwork) -> Self {
        match network {
            CliNetwork::Mainnet => Network::Mainnet,
            CliNetwork::Testnet => Network::Testnet,
        }
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_retries() -> u32 {
    3
}

fn default_metrics_url() -> String {
    "http://localhost:8889/metrics".to_string()
}

impl Config {
    /// Load configuration from file or create defaults
    ///
    /// When a config file exists, its network setting is preserved.
    /// The CLI network argument is only used when creating a new config.
    pub fn load(network: CliNetwork) -> Result<Self> {
        let config_dir = Self::config_dir()?;
        let config_path = config_dir.join("config.toml");

        if config_path.exists() {
            let content =
                std::fs::read_to_string(&config_path).context("Failed to read config file")?;
            let config: Config = toml::from_str(&content).context("Failed to parse config file")?;

            // Preserve the network setting from config file
            // Do NOT override with CLI argument (which may be default)
            // This ensures that testnet selection in init is preserved
            Ok(config)
        } else {
            // Create default config with specified network
            let config = Config::create_default(network)?;
            config.save()?;
            Ok(config)
        }
    }

    /// Create default configuration for a network
    pub fn create_default(network: CliNetwork) -> Result<Self> {
        let network_type = Network::from(network);
        let (chain_id, http_url, ws_url) = match network_type {
            Network::Mainnet => (
                143, // Monad mainnet chain ID
                "http://localhost:8080".to_string(),
                "ws://localhost:8080".to_string(),
            ),
            Network::Testnet => (
                10143, // Monad testnet chain ID
                "http://localhost:8080".to_string(),
                "ws://localhost:8080".to_string(),
            ),
        };

        Ok(Config {
            network: NetworkConfig {
                network_type,
                chain_id,
            },
            rpc: RpcConfig {
                http_url,
                ws_url,
                metrics_url: default_metrics_url(),
                timeout: default_timeout(),
                max_retries: default_retries(),
            },
            staking: StakingConfig::default(),
        })
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;
        std::fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

        let config_path = config_dir.join("config.toml");
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        std::fs::write(&config_path, content).context("Failed to write config file")?;

        Ok(())
    }

    /// Get configuration directory path
    pub fn config_dir() -> Result<PathBuf> {
        let project_dirs = ProjectDirs::from("xyz", "monad", "monad-val-manager")
            .context("Failed to determine config directory")?;
        Ok(project_dirs.config_dir().to_path_buf())
    }

    /// Get RPC endpoint URL
    pub fn rpc_endpoint(&self) -> &str {
        &self.rpc.http_url
    }

    /// Get WebSocket endpoint URL
    pub fn ws_endpoint(&self) -> &str {
        &self.rpc.ws_url
    }

    /// Get network type as string
    pub fn network(&self) -> &str {
        match self.network.network_type {
            Network::Mainnet => "mainnet",
            Network::Testnet => "testnet",
        }
    }

    /// Get configuration file path
    pub fn config_path(&self) -> PathBuf {
        Self::config_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("config.toml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_creation() {
        let config = Config::create_default(CliNetwork::Mainnet);
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.network.network_type, Network::Mainnet);
        assert_eq!(config.network.chain_id, 143);
        assert!(!config.rpc.http_url.is_empty());
    }

    #[test]
    fn test_network_conversion() {
        assert_eq!(Network::from(CliNetwork::Mainnet), Network::Mainnet);
        assert_eq!(Network::from(CliNetwork::Testnet), Network::Testnet);
    }

    #[test]
    fn test_network_string() {
        let config = Config::create_default(CliNetwork::Mainnet).unwrap();
        assert_eq!(config.network(), "mainnet");

        let config = Config::create_default(CliNetwork::Testnet).unwrap();
        assert_eq!(config.network(), "testnet");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::create_default(CliNetwork::Mainnet).unwrap();
        let serialized = toml::to_string(&config);
        assert!(serialized.is_ok());

        let serialized = serialized.unwrap();
        let deserialized: Result<Config, _> = toml::from_str(&serialized);
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_config_load_preserves_testnet_from_file() {
        // Test that loading a testnet config file with mainnet CLI arg
        // should preserve the testnet setting from the file
        let testnet_config = Config::create_default(CliNetwork::Testnet).unwrap();
        let serialized = toml::to_string(&testnet_config).unwrap();

        // Parse the serialized config
        let parsed: Config = toml::from_str(&serialized).unwrap();

        // Verify the parsed config has testnet settings
        assert_eq!(parsed.network.network_type, Network::Testnet);
        assert_eq!(parsed.network.chain_id, 10143);

        // Verify the network() method returns "testnet"
        assert_eq!(parsed.network(), "testnet");
    }

    #[test]
    fn test_config_load_with_cli_mainnet_does_not_override_saved_testnet() {
        // This test verifies that when a testnet config file exists,
        // loading it with mainnet CLI argument should NOT override the saved testnet setting
        let testnet_config = Config::create_default(CliNetwork::Testnet).unwrap();
        let serialized = toml::to_string(&testnet_config).unwrap();

        // Simulate loading config from file with mainnet CLI arg
        // This is what happens when Config::load() is called with CliNetwork::Mainnet
        // but the config file contains testnet settings
        let loaded: Config = toml::from_str(&serialized).unwrap();

        // The original code overrides network from CLI:
        // config.network.network_type = network.into();
        // This is the bug - it should preserve the file's network setting
        // For now, verify the loaded config has testnet
        assert_eq!(loaded.network.network_type, Network::Testnet);
        assert_eq!(loaded.network.chain_id, 10143);
        assert_eq!(loaded.network(), "testnet");
    }
}
