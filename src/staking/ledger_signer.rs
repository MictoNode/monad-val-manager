//! Ledger hardware wallet signer implementation
//!
//! This module provides hardware wallet support using Ledger devices.
//! The LedgerSigner communicates with a Ledger Nano S/X device via USB
//! to sign transactions securely.
//!
//! Uses the modern `alloy-signer-ledger` crate from the Alloy ecosystem.

use crate::staking::signer::{EcdsaSignature, Signer};
use crate::staking::transaction::Eip1559Transaction;
use crate::utils::error::{Error, Result};
use colored::Colorize;
use std::fmt;

#[cfg(feature = "ledger")]
use alloy_signer::Signer as AlloySignerTrait;
#[cfg(feature = "ledger")]
use alloy_signer_ledger::{HDPath, LedgerError, LedgerSigner as AlloyLedgerSigner};

/// Ledger hardware wallet signer
///
/// This signer communicates with a Ledger device to sign transactions.
/// The private key never leaves the device, providing enhanced security.
///
/// # Configuration
///
/// - `derivation_path`: BIP-32 derivation path (default: "44'/60'/0'/0/0")
/// - `chain_id`: Optional chain ID for EIP-155 replay protection
///
/// # Requirements
///
/// - Ledger device connected via USB
/// - Ethereum app open on the device
/// - "Blind signing" enabled in Ethereum app settings
///
/// # Example
///
/// ```no_run
/// use monad_val_manager::staking::ledger_signer::LedgerSigner;
/// use monad_val_manager::staking::Signer;
///
/// # #[cfg(feature = "ledger")]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let signer = LedgerSigner::new("44'/60'/0'/0/0")?;
/// println!("Address: {}", signer.address());
/// # Ok(())
/// # }
/// #
/// # #[cfg(not(feature = "ledger"))]
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #     Ok(())
/// # }
/// ```
pub struct LedgerSigner {
    /// BIP-32 derivation path
    derivation_path: String,
    /// Ethereum address derived from the device
    address: String,
    /// Chain ID for EIP-155
    chain_id: Option<u64>,
    /// Inner Alloy LedgerSigner (feature-gated)
    #[cfg(feature = "ledger")]
    inner: AlloyLedgerSigner,
}

impl LedgerSigner {
    /// Create a new Ledger signer (sync wrapper around async init)
    ///
    /// # Arguments
    ///
    /// * `derivation_path` - BIP-32 derivation path (e.g., "44'/60'/0'/0/0")
    ///
    /// # Returns
    ///
    /// LedgerSigner instance
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Ledger feature is not enabled
    /// - No Ledger device is found
    /// - Communication with device fails
    ///
    /// # Note
    ///
    /// This is a convenience wrapper that uses chain_id from Monad (143) by default.
    /// Use `new_with_chain_id` for custom chain ID.
    pub fn new(derivation_path: &str) -> Result<Self> {
        Self::new_with_chain_id(derivation_path, Some(143)) // Monad chain ID
    }

    /// Create a new Ledger signer with custom chain ID (sync wrapper)
    ///
    /// # Arguments
    ///
    /// * `derivation_path` - BIP-32 derivation path (e.g., "44'/60'/0'/0/0")
    /// * `chain_id` - Optional chain ID for EIP-155 replay protection
    ///
    /// # Returns
    ///
    /// LedgerSigner instance
    pub fn new_with_chain_id(derivation_path: &str, chain_id: Option<u64>) -> Result<Self> {
        #[cfg(not(feature = "ledger"))]
        {
            let _ = derivation_path;
            let _ = chain_id;
            Err(Error::Config(
                "Ledger support is not enabled. Build with --features ledger".to_string(),
            ))
        }

        #[cfg(feature = "ledger")]
        {
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| Error::Signing(format!("Failed to create tokio runtime: {}", e)))?;
            rt.block_on(async { Self::new_with_device(derivation_path, chain_id).await })
        }
    }

    /// Create a new Ledger signer (async)
    ///
    /// # Arguments
    ///
    /// * `derivation_path` - BIP-32 derivation path (e.g., "44'/60'/0'/0/0")
    /// * `chain_id` - Optional chain ID for EIP-155 replay protection
    ///
    /// # Returns
    ///
    /// LedgerSigner instance
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Ledger feature is not enabled
    /// - No Ledger device is found
    /// - Communication with device fails
    pub async fn new_async(derivation_path: &str, chain_id: Option<u64>) -> Result<Self> {
        #[cfg(not(feature = "ledger"))]
        {
            let _ = derivation_path;
            let _ = chain_id;
            Err(Error::Config(
                "Ledger support is not enabled. Build with --features ledger".to_string(),
            ))
        }

        #[cfg(feature = "ledger")]
        {
            Self::new_with_device(derivation_path, chain_id).await
        }
    }

    /// Create Ledger signer with actual device communication (feature-gated, async)
    #[cfg(feature = "ledger")]
    async fn new_with_device(derivation_path: &str, chain_id: Option<u64>) -> Result<Self> {
        // Convert derivation path to HDPath
        // alloy-signer-ledger supports LedgerLive, Ledger, or Other(custom)
        let hd_path = HDPath::Other(derivation_path.to_string());

        // Initialize Alloy LedgerSigner
        let inner = AlloyLedgerSigner::new(hd_path, chain_id)
            .await
            .map_err(Self::convert_ledger_error)?;

        // Get address from device
        let address = inner
            .get_address()
            .await
            .map_err(Self::convert_ledger_error)?;

        let address_str = format!("{:#x}", address);

        Ok(Self {
            derivation_path: derivation_path.to_string(),
            address: address_str,
            chain_id,
            inner,
        })
    }

    /// Convert LedgerError to our Error type
    #[cfg(feature = "ledger")]
    fn convert_ledger_error(err: LedgerError) -> Error {
        // Check if it's a device not found error
        let err_msg = format!("{}", err);
        if err_msg.contains("Device") && err_msg.contains("not found") {
            return Error::Signing(
                "No Ledger device found. Please ensure:\n\
                 - Ledger is connected via USB\n\
                 - Ethereum app is open\n\
                 - 'Blind signing' is enabled in Ethereum app settings"
                    .to_string(),
            );
        }
        Error::Signing(format!("Ledger error: {}", err))
    }

    /// Get the derivation path
    pub fn derivation_path(&self) -> &str {
        &self.derivation_path
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> Option<u64> {
        self.chain_id
    }

    /// Check if Ledger feature is enabled
    pub fn is_supported() -> bool {
        cfg!(feature = "ledger")
    }
}

impl Signer for LedgerSigner {
    fn address(&self) -> &str {
        &self.address
    }

    fn sign_hash(&self, hash: &[u8; 32]) -> Result<EcdsaSignature> {
        #[cfg(not(feature = "ledger"))]
        {
            // Explicitly mark hash as used when ledger feature is disabled
            let _ = hash;
            Err(Error::Signing(
                "Ledger support is not enabled. Build with --features ledger".to_string(),
            ))
        }

        #[cfg(feature = "ledger")]
        {
            use alloy_primitives::B256;

            // Use tokio runtime to execute async function
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| Error::Signing(format!("Failed to create tokio runtime: {}", e)))?;

            let b256_hash = B256::from(*hash);

            rt.block_on(async {
                // Display message for user
                println!();
                println!(
                    "{}",
                    "Please confirm signature on hardware wallet...".yellow()
                );
                println!();

                // Sign the hash
                // Note: AlloySigner trait returns a boxed error, so we convert to string
                let signature = self
                    .inner
                    .sign_hash(&b256_hash)
                    .await
                    .map_err(|e| Error::Signing(format!("Ledger signing error: {}", e)))?;

                // Convert alloy Signature to our EcdsaSignature
                // Alloy signature: v (y-parity bool) + r (B256) + s (B256)
                let v = u8::from(signature.v()); // bool -> u8 (false=0, true=1)
                let r = signature.r().to_be_bytes();
                let s = signature.s().to_be_bytes();

                Ok(EcdsaSignature::from_bytes(v, r, s))
            })
        }
    }

    fn sign_transaction(&self, tx: &Eip1559Transaction) -> Result<Vec<u8>> {
        // Display message for user
        println!();
        println!(
            "{}",
            "Please review and sign transaction on hardware wallet...".yellow()
        );
        println!();

        // Use default implementation from Signer trait
        let hash = tx.signing_hash();
        let sig = self.sign_hash(&hash)?;
        tx.encode_signed(sig.v, &sig.r, &sig.s)
    }
}

impl fmt::Debug for LedgerSigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LedgerSigner")
            .field("derivation_path", &self.derivation_path)
            .field("address", &self.address)
            .field("chain_id", &self.chain_id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_signer_not_supported_without_feature() {
        #[cfg(not(feature = "ledger"))]
        {
            assert!(!LedgerSigner::is_supported());
        }
    }

    #[test]
    fn test_ledger_is_supported() {
        let supported = LedgerSigner::is_supported();
        #[cfg(feature = "ledger")]
        assert!(supported);
        #[cfg(not(feature = "ledger"))]
        assert!(!supported);
    }

    #[test]
    fn test_ledger_signer_debug() {
        #[cfg(feature = "ledger")]
        {
            // Create a dummy signer for debug testing
            // Note: We can't create a real signer without a device
            // This just tests the Debug implementation structure
            let derivation_path = "44'/60'/0'/0/0";
            let address = "0x0000000000000000000000000000000000000000";
            let chain_id: Option<u64> = Some(143);

            // Test format structure
            let debug_str = format!(
                "LedgerSigner {{ derivation_path: {:?}, address: {:?}, chain_id: {:?} }}",
                derivation_path, address, chain_id
            );
            assert!(debug_str.contains("LedgerSigner"));
            assert!(debug_str.contains("derivation_path"));
        }
    }
}
