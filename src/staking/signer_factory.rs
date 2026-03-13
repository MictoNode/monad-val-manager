//! Signer factory for creating signers based on configuration
//!
//! This module provides a unified interface for creating signers
//! based on the staking configuration (local or Ledger hardware wallet).

use crate::config::{Config, Network};
use crate::staking::ledger_signer::LedgerSigner;
use crate::staking::signer::{LocalSigner, Signer};
use crate::utils::error::{Error, Result};
use std::sync::Arc;

/// Signer type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignerType {
    /// Local signer using a private key in memory
    Local,
    /// Ledger hardware wallet signer
    Ledger,
}

impl SignerType {
    /// Parse signer type from string input
    pub fn from_str_input(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "local" => Ok(SignerType::Local),
            "ledger" => Ok(SignerType::Ledger),
            _ => Err(Error::Config(format!(
                "Invalid signer type: '{}'. Must be 'local' or 'ledger'",
                s
            ))),
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            SignerType::Local => "local",
            SignerType::Ledger => "ledger",
        }
    }
}

/// Create a signer based on the configuration
///
/// This function reads the staking configuration and creates the appropriate signer:
/// - `local`: Creates a LocalSigner using the private key from environment
/// - `ledger`: Creates a LedgerSigner that communicates with a hardware wallet
///
/// Environment variables are checked first:
/// - `STAKING_TYPE`: Overrides config signer type
/// - `DERIVATION_PATH`: Overrides config derivation path (for ledger)
///
/// # Arguments
///
/// * `config` - Application configuration
///
/// # Returns
///
/// A boxed Signer trait object
///
/// # Errors
///
/// Returns an error if:
/// - Private key not found for local signer
/// - Ledger device not found for ledger signer
/// - Invalid signer type in config
///
/// # Example
///
/// ```no_run
/// use monad_val_manager::staking::signer_factory::create_signer;
/// use monad_val_manager::config::Config;
/// # use monad_val_manager::cli::Network;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let config = Config::create_default(Network::Mainnet)?;
/// let signer = create_signer(&config)?;
/// println!("Signer address: {}", signer.address());
/// # Ok(())
/// # }
/// ```
pub fn create_signer(config: &Config) -> Result<Arc<dyn Signer>> {
    // Check environment variable first, then fall back to config
    let signer_type_str = config.staking.get_signer_type();
    let signer_type = SignerType::from_str_input(&signer_type_str)?;

    match signer_type {
        SignerType::Local => {
            let network = config.network.network_type;
            let private_key = crate::config::get_private_key(network).ok_or_else(|| {
                Error::Config(format!(
                    "Private key not found for {}. Run 'init' to configure staking first.",
                    match network {
                        Network::Mainnet => "mainnet",
                        Network::Testnet => "testnet",
                    }
                ))
            })?;

            let signer = LocalSigner::from_private_key(&private_key)?;
            Ok(Arc::new(signer))
        }
        SignerType::Ledger => {
            let derivation_path = config.staking.get_derivation_path();
            let signer = LedgerSigner::new(&derivation_path)?;
            Ok(Arc::new(signer))
        }
    }
}

/// Create a signer with explicit signer type
///
/// This is useful when you want to override the config setting
/// or create a signer without loading the full config.
///
/// # Arguments
///
/// * `signer_type` - Type of signer to create
/// * `private_key` - Private key for local signer (required if signer_type is Local)
/// * `derivation_path` - Derivation path for Ledger signer (required if signer_type is Ledger)
///
/// # Returns
///
/// A boxed Signer trait object
pub fn create_signer_with_type(
    signer_type: SignerType,
    private_key: Option<&str>,
    derivation_path: Option<&str>,
) -> Result<Arc<dyn Signer>> {
    match signer_type {
        SignerType::Local => {
            let key = private_key.ok_or_else(|| {
                Error::Config("Private key required for local signer".to_string())
            })?;
            let signer = LocalSigner::from_private_key(key)?;
            Ok(Arc::new(signer))
        }
        SignerType::Ledger => {
            let path = derivation_path
                .ok_or_else(|| {
                    Error::Config("Derivation path required for ledger signer".to_string())
                })?
                .to_string();
            let signer = LedgerSigner::new(&path)?;
            Ok(Arc::new(signer))
        }
    }
}

/// Check if Ledger support is available
///
/// Returns true if the ledger feature is enabled
pub fn is_ledger_supported() -> bool {
    LedgerSigner::is_supported()
}

/// Get the signer type from config
///
/// Checks STAKING_TYPE env var first, then falls back to config
///
/// # Arguments
///
/// * `config` - Application configuration
///
/// # Returns
///
/// The signer type from config or environment
pub fn get_signer_type(config: &Config) -> SignerType {
    let signer_type_str = config.staking.get_signer_type();
    SignerType::from_str_input(&signer_type_str).unwrap_or(SignerType::Local)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signer_type_from_str() {
        assert_eq!(
            SignerType::from_str_input("local").unwrap(),
            SignerType::Local
        );
        assert_eq!(
            SignerType::from_str_input("LOCAL").unwrap(),
            SignerType::Local
        );
        assert_eq!(
            SignerType::from_str_input("ledger").unwrap(),
            SignerType::Ledger
        );
        assert_eq!(
            SignerType::from_str_input("LEDGER").unwrap(),
            SignerType::Ledger
        );
        assert!(SignerType::from_str_input("invalid").is_err());
    }

    #[test]
    fn test_signer_type_as_str() {
        assert_eq!(SignerType::Local.as_str(), "local");
        assert_eq!(SignerType::Ledger.as_str(), "ledger");
    }

    #[test]
    fn test_is_ledger_supported() {
        let supported = is_ledger_supported();
        #[cfg(feature = "ledger")]
        assert!(supported);
        #[cfg(not(feature = "ledger"))]
        assert!(!supported);
    }

    #[test]
    fn test_create_signer_with_type_local() {
        let result = create_signer_with_type(
            SignerType::Local,
            Some("0000000000000000000000000000000000000000000000000000000000000001"),
            None,
        );
        assert!(result.is_ok());
        let signer = result.unwrap();
        assert!(signer.address().starts_with("0x"));
    }

    #[test]
    fn test_create_signer_with_type_local_no_key() {
        let result = create_signer_with_type(SignerType::Local, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_signer_with_type_ledger_no_path() {
        let result = create_signer_with_type(SignerType::Ledger, None, None);
        assert!(result.is_err());
    }

    #[cfg(feature = "ledger")]
    #[test]
    fn test_create_signer_with_type_ledger() {
        // This test requires actual hardware
        if is_ledger_supported() {
            let result = create_signer_with_type(SignerType::Ledger, None, Some("44'/60'/0'/0/0"));
            // May fail if no device connected, but should not panic
            let _ = result;
        }
    }
}
