//! Environment variable management for private key storage
//!
//! This module provides secure .env file management for storing network-specific
//! private keys in the application's configuration directory.

use anyhow::{Context, Result};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use super::settings::Network;

/// Environment variable name for mainnet private key
pub const MONAD_MAINNET_PRIVATE_KEY: &str = "MONAD_MAINNET_PRIVATE_KEY";

/// Environment variable name for testnet private key
pub const MONAD_TESTNET_PRIVATE_KEY: &str = "MONAD_TESTNET_PRIVATE_KEY";

/// Get the configuration directory path
///
/// Returns the same directory used by the main config:
/// - Windows: `%APPDATA%\monad-val-manager\`
/// - Linux/macOS: `~/.config/monad-val-manager/`
pub fn get_config_dir() -> Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("xyz", "monad", "monad-val-manager")
        .context("Failed to determine config directory")?;
    Ok(project_dirs.config_dir().to_path_buf())
}

/// Get the .env file path in the configuration directory
pub fn get_env_path() -> Result<PathBuf> {
    get_config_dir().map(|dir| dir.join(".env"))
}

/// Load environment variables from the .env file in the config directory
///
/// This function loads the .env file if it exists. If the file doesn't exist,
/// it silently returns Ok(()) without setting any variables.
///
/// # Example
///
/// ```no_run
/// use monad_val_manager::config::load_env;
///
/// load_env().expect("Failed to load .env file");
/// ```
pub fn load_env() -> Result<()> {
    let env_path = get_env_path()?;

    if !env_path.exists() {
        return Ok(());
    }

    dotenvy::from_path(&env_path).context("Failed to load .env file")?;

    Ok(())
}

/// Get the private key for a specific network from environment variables
///
/// Returns the private key if it exists in the environment, or None if not set.
///
/// # Example
///
/// ```no_run
/// use monad_val_manager::config::{load_env, get_private_key, Network};
///
/// load_env().ok();
/// if let Some(key) = get_private_key(Network::Mainnet) {
///     println!("Mainnet private key found");
/// }
/// ```
pub fn get_private_key(network: Network) -> Option<String> {
    let env_var = match network {
        Network::Mainnet => MONAD_MAINNET_PRIVATE_KEY,
        Network::Testnet => MONAD_TESTNET_PRIVATE_KEY,
    };

    std::env::var(env_var).ok()
}

/// Set a private key for a specific network in the .env file
///
/// This function creates or updates the .env file with the specified private key.
/// The configuration directory is created if it doesn't exist.
///
/// # Arguments
///
/// * `network` - The network (Mainnet or Testnet)
/// * `key` - The private key to store
///
/// # Example
///
/// ```no_run
/// use monad_val_manager::config::{set_private_key, Network};
///
/// set_private_key(Network::Testnet, "0x...").expect("Failed to set private key");
/// ```
pub fn set_private_key(network: Network, key: &str) -> Result<()> {
    let env_path = get_env_path()?;

    // Ensure config directory exists
    if let Some(parent) = env_path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    let env_var = match network {
        Network::Mainnet => MONAD_MAINNET_PRIVATE_KEY,
        Network::Testnet => MONAD_TESTNET_PRIVATE_KEY,
    };

    // Read existing content and update or add the key
    let mut lines: Vec<String> = Vec::new();
    let mut key_found = false;

    if env_path.exists() {
        let file = File::open(&env_path).context("Failed to open .env file for reading")?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.context("Failed to read line from .env file")?;
            if line.starts_with(&format!("{}=", env_var)) {
                lines.push(format!("{}={}", env_var, key));
                key_found = true;
            } else {
                lines.push(line);
            }
        }
    }

    // If key wasn't found, add it
    if !key_found {
        lines.push(format!("{}={}", env_var, key));
    }

    // Write back to file
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&env_path)
        .context("Failed to open .env file for writing")?;

    for line in lines {
        writeln!(file, "{}", line).context("Failed to write to .env file")?;
    }

    // Also set in current process environment
    std::env::set_var(env_var, key);

    Ok(())
}

/// Check if the .env file exists in the configuration directory
///
/// # Example
///
/// ```no_run
/// use monad_val_manager::config::env_file_exists;
///
/// if env_file_exists() {
///     println!(".env file found");
/// }
/// ```
pub fn env_file_exists() -> bool {
    get_env_path().map(|path| path.exists()).unwrap_or(false)
}

/// Delete a private key for a specific network from the .env file
///
/// Returns Ok(true) if the key was found and removed, Ok(false) if the key wasn't found.
///
/// # Example
///
/// ```no_run
/// use monad_val_manager::config::{delete_private_key, Network};
///
/// if delete_private_key(Network::Testnet).expect("Failed") {
///     println!("Private key removed");
/// }
/// ```
pub fn delete_private_key(network: Network) -> Result<bool> {
    let env_path = get_env_path()?;

    if !env_path.exists() {
        return Ok(false);
    }

    let env_var = match network {
        Network::Mainnet => MONAD_MAINNET_PRIVATE_KEY,
        Network::Testnet => MONAD_TESTNET_PRIVATE_KEY,
    };

    let file = File::open(&env_path).context("Failed to open .env file for reading")?;
    let reader = BufReader::new(file);

    let mut lines: Vec<String> = Vec::new();
    let mut key_found = false;

    for line in reader.lines() {
        let line = line.context("Failed to read line from .env file")?;
        if !line.starts_with(&format!("{}=", env_var)) {
            lines.push(line);
        } else {
            key_found = true;
        }
    }

    if key_found {
        // Write back to file without the deleted key
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&env_path)
            .context("Failed to open .env file for writing")?;

        for line in lines {
            writeln!(file, "{}", line).context("Failed to write to .env file")?;
        }

        // Also remove from current process environment
        std::env::remove_var(env_var);
    }

    Ok(key_found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_names() {
        assert_eq!(MONAD_MAINNET_PRIVATE_KEY, "MONAD_MAINNET_PRIVATE_KEY");
        assert_eq!(MONAD_TESTNET_PRIVATE_KEY, "MONAD_TESTNET_PRIVATE_KEY");
    }

    #[test]
    fn test_get_config_dir() {
        let result = get_config_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("monad-val-manager"));
    }

    #[test]
    fn test_get_env_path() {
        let result = get_env_path();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("monad-val-manager"));
        assert!(path.to_string_lossy().contains(".env"));
    }
}
