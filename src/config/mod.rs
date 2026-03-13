//! Configuration module - Settings management

mod env;
mod settings;

pub use env::{
    delete_private_key, env_file_exists, get_config_dir, get_env_path, get_private_key, load_env,
    set_private_key, MONAD_MAINNET_PRIVATE_KEY, MONAD_TESTNET_PRIVATE_KEY,
};
pub use settings::{Config, Network, NetworkConfig, RpcConfig, StakingConfig};

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::cli::Network as CliNetwork;

    #[test]
    fn test_config_defaults() {
        // This test verifies that Config can be created with defaults
        let _ = Config::create_default(CliNetwork::Mainnet);
    }
}
