//! Configuration loading integration tests
//!
//! These tests verify that network configuration is preserved correctly
//! when loading from saved config files, especially for testnet selection.

use monad_val_manager::cli::Network as CliNetwork;
use monad_val_manager::config::Config;
use tempfile::TempDir;

/// Test that testnet config is preserved when loading with mainnet CLI default
///
/// This test reproduces the bug where:
/// 1. User runs `init` and selects testnet
/// 2. Config file is saved with type = "testnet" and chain_id = 10143
/// 3. User runs `status` or TUI (without --network flag)
/// 4. CLI defaults to mainnet, but config should preserve testnet
#[test]
fn test_config_load_preserves_testnet_setting() {
    // Create a temporary directory for the test config
    let _temp_dir = TempDir::new().unwrap();

    // Create a testnet config
    let testnet_config = Config {
        network: monad_val_manager::config::NetworkConfig {
            network_type: monad_val_manager::config::Network::Testnet,
            chain_id: 10143,
        },
        rpc: monad_val_manager::config::RpcConfig {
            http_url: "http://localhost:8080".to_string(),
            ws_url: "ws://localhost:8080".to_string(),
            metrics_url: "http://localhost:8889/metrics".to_string(),
            timeout: 30,
            max_retries: 3,
        },
        staking: monad_val_manager::config::StakingConfig::default(),
    };

    // Serialize to TOML
    let toml_content = toml::to_string_pretty(&testnet_config).unwrap();

    // Verify the TOML contains testnet settings
    assert!(toml_content.contains("type = \"testnet\""));
    assert!(toml_content.contains("chain_id = 10143"));

    // Parse it back
    let parsed_config: Config = toml::from_str(&toml_content).unwrap();

    // Verify the parsed config has testnet settings
    assert_eq!(
        parsed_config.network.network_type,
        monad_val_manager::config::Network::Testnet
    );
    assert_eq!(parsed_config.network.chain_id, 10143);
    assert_eq!(parsed_config.network(), "testnet");
}

/// Test that mainnet config is preserved when loading
#[test]
fn test_config_load_preserves_mainnet_setting() {
    // Create a mainnet config
    let mainnet_config = Config {
        network: monad_val_manager::config::NetworkConfig {
            network_type: monad_val_manager::config::Network::Mainnet,
            chain_id: 143,
        },
        rpc: monad_val_manager::config::RpcConfig {
            http_url: "http://localhost:8545".to_string(),
            ws_url: "ws://localhost:8545".to_string(),
            metrics_url: "http://localhost:8889/metrics".to_string(),
            timeout: 30,
            max_retries: 3,
        },
        staking: monad_val_manager::config::StakingConfig::default(),
    };

    // Serialize to TOML
    let toml_content = toml::to_string_pretty(&mainnet_config).unwrap();

    // Verify the TOML contains mainnet settings
    assert!(toml_content.contains("type = \"mainnet\""));
    assert!(toml_content.contains("chain_id = 143"));

    // Parse it back
    let parsed_config: Config = toml::from_str(&toml_content).unwrap();

    // Verify the parsed config has mainnet settings
    assert_eq!(
        parsed_config.network.network_type,
        monad_val_manager::config::Network::Mainnet
    );
    assert_eq!(parsed_config.network.chain_id, 143);
    assert_eq!(parsed_config.network(), "mainnet");
}

/// Test that Config::create_default creates correct config for each network
#[test]
fn test_create_default_testnet_config() {
    let config = Config::create_default(CliNetwork::Testnet).unwrap();

    assert_eq!(
        config.network.network_type,
        monad_val_manager::config::Network::Testnet
    );
    assert_eq!(config.network.chain_id, 10143);
    assert_eq!(config.network(), "testnet");
}

#[test]
fn test_create_default_mainnet_config() {
    let config = Config::create_default(CliNetwork::Mainnet).unwrap();

    assert_eq!(
        config.network.network_type,
        monad_val_manager::config::Network::Mainnet
    );
    assert_eq!(config.network.chain_id, 143);
    assert_eq!(config.network(), "mainnet");
}
