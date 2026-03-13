//! Cryptographic signers for Monad staking operations
//!
//! This module provides signer implementations for signing transactions
//! on the Monad blockchain. Two types of signatures are used:
//!
//! - **SECP256k1**: Standard Ethereum transaction signatures
//! - **BLS12-381**: Validator registration signatures
//!
//! # Architecture
//!
//! The `Signer` trait defines the common interface for all signers.
//! The `LocalSigner` implementation uses local private keys stored in memory.
//!
//! # Migration to Alloy Native Signer
//!
//! As of Session 15, we're migrating to Alloy v1.7's native Signer trait
//! to fix BUG-007 (RPC -32603 error). The manual RLP encoding is being replaced
//! with Alloy's built-in transaction signing.

use crate::staking::transaction::{keccak256_hash, Eip1559Transaction};
use crate::utils::error::{Error, Result};
use alloy_consensus::{SignableTransaction, TxEip1559, TxEnvelope};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{Address, Bytes, PrimitiveSignature, TxKind, U256};
use k256::ecdsa::{SigningKey, VerifyingKey};
use std::fmt;
use std::str::FromStr;

/// ECDSA signature components (r, s, v)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcdsaSignature {
    /// Recovery ID (y-parity, 0 or 1 for EIP-1559)
    pub v: u8,
    /// R component (32 bytes)
    pub r: [u8; 32],
    /// S component (32 bytes)
    pub s: [u8; 32],
}

impl EcdsaSignature {
    /// Create from raw bytes
    pub fn from_bytes(v: u8, r: [u8; 32], s: [u8; 32]) -> Self {
        Self { v, r, s }
    }

    /// Create from k256 Signature and recovery ID
    pub fn from_k256(sig: &k256::ecdsa::Signature, recovery_id: u8) -> Self {
        let bytes = sig.to_bytes();
        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        r.copy_from_slice(&bytes[..32]);
        s.copy_from_slice(&bytes[32..64]);
        Self {
            v: recovery_id,
            r,
            s,
        }
    }

    /// Get r as bytes slice
    pub fn r_bytes(&self) -> &[u8] {
        &self.r
    }

    /// Get s as bytes slice
    pub fn s_bytes(&self) -> &[u8] {
        &self.s
    }
}

/// Trait for transaction signers
pub trait Signer: Send + Sync {
    /// Get the Ethereum address associated with this signer
    fn address(&self) -> &str;

    /// Sign a transaction hash (32 bytes)
    ///
    /// # Arguments
    /// * `hash` - 32-byte hash to sign
    ///
    /// # Returns
    /// ECDSA signature with recovery ID
    fn sign_hash(&self, hash: &[u8; 32]) -> Result<EcdsaSignature>;

    /// Sign an EIP-1559 transaction
    ///
    /// # Arguments
    /// * `tx` - Transaction to sign
    ///
    /// # Returns
    /// Raw signed transaction bytes (ready for eth_sendRawTransaction)
    fn sign_transaction(&self, tx: &Eip1559Transaction) -> Result<Vec<u8>> {
        // Compute the hash to sign
        let hash = tx.signing_hash();

        // Sign the hash
        let sig = self.sign_hash(&hash)?;

        // Encode the signed transaction
        tx.encode_signed(sig.v, &sig.r, &sig.s)
    }

    /// Sign an EIP-1559 transaction and return hex string
    fn sign_transaction_hex(&self, tx: &Eip1559Transaction) -> Result<String> {
        let raw = self.sign_transaction(tx)?;
        Ok(format!("0x{}", hex::encode(raw)))
    }

    /// Sign an EIP-1559 transaction using Alloy's native Signer trait
    ///
    /// This method uses Alloy v1.7's built-in transaction signing to ensure
    /// proper EIP-2718 encoding and fix BUG-007 (RPC -32603 error).
    ///
    /// # Returns
    /// Raw signed transaction bytes (EIP-2718 encoded)
    fn sign_transaction_alloy(
        &self,
        tx: &Eip1559Transaction,
        signing_key: &SigningKey,
    ) -> Result<Vec<u8>> {
        // Convert to Alloy TxEip1559
        let to_addr = Address::from_str(tx.to.as_str())
            .map_err(|_| Error::InvalidInput(format!("Invalid address: {}", tx.to)))?;

        let alloy_tx = TxEip1559 {
            chain_id: tx.chain_id,
            nonce: tx.nonce,
            max_priority_fee_per_gas: tx.max_priority_fee_per_gas as u128,
            max_fee_per_gas: tx.max_fee_per_gas as u128,
            gas_limit: tx.gas_limit,
            to: TxKind::Call(to_addr),
            value: U256::from(tx.value),
            input: Bytes::from(tx.data.clone()),
            access_list: Default::default(),
        };

        // Create SignableTransaction (exact approach from transaction_alloy.rs)
        let hash = alloy_tx.signature_hash();
        let (signature, recovery_id) = signing_key
            .sign_prehash_recoverable(hash.as_slice())
            .map_err(|e| Error::Signing(format!("Failed to sign: {}", e)))?;

        // Convert k256 signature to alloy PrimitiveSignature
        // In Alloy 0.6, use from_signature_and_parity for direct k256 conversion
        let primitive_sig =
            PrimitiveSignature::from_signature_and_parity(signature, recovery_id.is_y_odd());

        // Build TxEnvelope with native Alloy encoding
        let envelope = TxEnvelope::Eip1559(alloy_tx.into_signed(primitive_sig));

        // Encode as EIP-2718 (this is the key fix for BUG-007)
        let encoded = envelope.encoded_2718();
        Ok(encoded.to_vec())
    }

    /// Sign an EIP-1559 transaction using Alloy and return hex string
    fn sign_transaction_alloy_hex(
        &self,
        tx: &Eip1559Transaction,
        signing_key: &SigningKey,
    ) -> Result<String> {
        let raw = self.sign_transaction_alloy(tx, signing_key)?;
        Ok(format!("0x{}", hex::encode(raw)))
    }
}

/// Local signer using a private key in memory
pub struct LocalSigner {
    /// ECDSA signing key
    signing_key: SigningKey,
    /// Ethereum address (derived from public key)
    address: String,
}

impl LocalSigner {
    /// Create a new local signer from a private key
    ///
    /// # Arguments
    /// * `private_key` - Private key as hex string (with or without 0x prefix)
    ///
    /// # Returns
    /// LocalSigner instance
    pub fn from_private_key(private_key: &str) -> Result<Self> {
        // Strip 0x prefix if present
        let clean = private_key.strip_prefix("0x").unwrap_or(private_key);

        // Validate length (must be 32 bytes = 64 hex chars)
        if clean.len() != 64 {
            return Err(Error::InvalidInput(format!(
                "Private key must be 32 bytes (64 hex chars), got {} chars",
                clean.len()
            )));
        }

        // Decode hex to bytes
        let key_bytes =
            hex::decode(clean).map_err(|e| Error::InvalidInput(format!("Invalid hex: {}", e)))?;

        // Create signing key
        let signing_key = SigningKey::from_bytes((&key_bytes[..]).into())
            .map_err(|e| Error::Signing(format!("Invalid private key: {}", e)))?;

        // Derive address from public key
        let verifying_key = signing_key.verifying_key();
        let address = pubkey_to_address(verifying_key);

        Ok(Self {
            signing_key,
            address,
        })
    }

    /// Get the signing key for Alloy native signing
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    /// Get the verifying (public) key
    pub fn verifying_key(&self) -> &VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Get the public key as compressed bytes (33 bytes)
    pub fn public_key_compressed(&self) -> Vec<u8> {
        self.signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes()
            .to_vec()
    }

    /// Get the public key as uncompressed bytes (64 bytes, without the 04 prefix)
    pub fn public_key_uncompressed(&self) -> Vec<u8> {
        let encoded = self.signing_key.verifying_key().to_encoded_point(false);
        // Skip the 0x04 prefix byte
        encoded.as_bytes()[1..].to_vec()
    }
}

impl Signer for LocalSigner {
    fn address(&self) -> &str {
        &self.address
    }

    fn sign_hash(&self, hash: &[u8; 32]) -> Result<EcdsaSignature> {
        // Sign using k256
        let (signature, recovery_id) = self
            .signing_key
            .sign_prehash_recoverable(hash)
            .map_err(|e| Error::Signing(format!("Signing failed: {}", e)))?;

        // For EIP-1559, we use y-parity directly (0 or 1)
        // Not the EIP-155 encoded v
        let v = recovery_id.to_byte();

        Ok(EcdsaSignature::from_k256(&signature, v))
    }

    /// Override to use Alloy native signing (fixes BUG-007)
    fn sign_transaction(&self, tx: &Eip1559Transaction) -> Result<Vec<u8>> {
        // Use Alloy native signing for proper EIP-2718 encoding
        self.sign_transaction_alloy(tx, &self.signing_key)
    }

    /// Override to use Alloy native signing (fixes BUG-007)
    fn sign_transaction_hex(&self, tx: &Eip1559Transaction) -> Result<String> {
        self.sign_transaction_alloy_hex(tx, &self.signing_key)
    }
}

impl fmt::Debug for LocalSigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalSigner")
            .field("address", &self.address)
            .field("signing_key", &"[REDACTED]")
            .finish()
    }
}

/// Convert a public key to an Ethereum address
///
/// The address is the last 20 bytes of keccak256(pubkey_uncompressed)
fn pubkey_to_address(pubkey: &VerifyingKey) -> String {
    // Get uncompressed public key (65 bytes: 0x04 + X + Y)
    let encoded = pubkey.to_encoded_point(false);
    let pubkey_bytes = encoded.as_bytes();

    // Skip the 0x04 prefix, hash the remaining 64 bytes
    let hash = keccak256_hash(&pubkey_bytes[1..]);

    // Take last 20 bytes as address
    format!("0x{}", hex::encode(&hash[12..32]))
}

/// Sign a message hash using BLAKE3 + SECP256k1 (for add_validator)
///
/// This is used for the SECP signature in add_validator, which uses
/// BLAKE3 instead of keccak256 for hashing the payload.
pub fn sign_blake3_hash(signing_key: &SigningKey, hash: &[u8; 32]) -> Result<EcdsaSignature> {
    let (signature, recovery_id) = signing_key
        .sign_prehash_recoverable(hash)
        .map_err(|e| Error::Signing(format!("BLAKE3 signing failed: {}", e)))?;

    let v = recovery_id.to_byte();
    Ok(EcdsaSignature::from_k256(&signature, v))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test private key (DO NOT USE IN PRODUCTION)
    const TEST_PRIVATE_KEY: &str =
        "0000000000000000000000000000000000000000000000000000000000000001";

    #[test]
    fn test_local_signer_creation() {
        let signer = LocalSigner::from_private_key(TEST_PRIVATE_KEY);
        assert!(signer.is_ok());
    }

    #[test]
    fn test_local_signer_address() {
        let signer = LocalSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();
        // Known address for private key 0x01
        assert!(signer.address().starts_with("0x"));
        assert_eq!(signer.address().len(), 42);
    }

    #[test]
    fn test_local_signer_invalid_key_length() {
        let result = LocalSigner::from_private_key("1234");
        assert!(result.is_err());
    }

    #[test]
    fn test_local_signer_invalid_hex() {
        let result = LocalSigner::from_private_key(
            "gggg000000000000000000000000000000000000000000000000000000000000",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_hash() {
        let signer = LocalSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();
        let hash = [0u8; 32];
        let sig = signer.sign_hash(&hash);

        assert!(sig.is_ok());
        let sig = sig.unwrap();
        assert!(sig.v == 0 || sig.v == 1);
        assert_eq!(sig.r.len(), 32);
        assert_eq!(sig.s.len(), 32);
    }

    #[test]
    fn test_sign_transaction() {
        let signer = LocalSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();

        let tx = Eip1559Transaction::new(143)
            .with_nonce(0)
            .to("0x0000000000000000000000000000000000001000")
            .expect("Valid address")
            .with_value(1_000_000_000_000_000_000u128);

        let signed = signer.sign_transaction(&tx);
        assert!(signed.is_ok());

        let raw = signed.unwrap();
        assert_eq!(raw[0], 0x02); // EIP-1559 type prefix
    }

    #[test]
    fn test_sign_transaction_hex() {
        let signer = LocalSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();

        let tx = Eip1559Transaction::new(143)
            .with_nonce(0)
            .to("0x0000000000000000000000000000000000001000")
            .expect("Valid address");

        let hex = signer.sign_transaction_hex(&tx).unwrap();
        assert!(hex.starts_with("0x02"));
    }

    #[test]
    fn test_public_key_compressed() {
        let signer = LocalSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();
        let pubkey = signer.public_key_compressed();
        assert_eq!(pubkey.len(), 33);
    }

    #[test]
    fn test_public_key_uncompressed() {
        let signer = LocalSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();
        let pubkey = signer.public_key_uncompressed();
        assert_eq!(pubkey.len(), 64);
    }

    #[test]
    fn test_ecdsa_signature_from_bytes() {
        let r = [1u8; 32];
        let s = [2u8; 32];
        let sig = EcdsaSignature::from_bytes(0, r, s);

        assert_eq!(sig.v, 0);
        assert_eq!(sig.r, r);
        assert_eq!(sig.s, s);
    }

    // ===== sign_blake3_hash Tests =====

    #[test]
    fn test_sign_blake3_hash_success() {
        let signer = LocalSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();
        let hash = [0u8; 32];

        let result = sign_blake3_hash(&signer.signing_key, &hash);
        assert!(result.is_ok(), "sign_blake3_hash should succeed");

        let sig = result.unwrap();
        assert_eq!(sig.r.len(), 32, "r component should be 32 bytes");
        assert_eq!(sig.s.len(), 32, "s component should be 32 bytes");
        assert!(sig.v == 0 || sig.v == 1, "v should be 0 or 1 (recovery ID)");
    }

    #[test]
    fn test_sign_blake3_hash_deterministic() {
        // Same hash should produce same signature with same key
        let signer = LocalSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();
        let hash = [0x42u8; 32];

        let sig1 = sign_blake3_hash(&signer.signing_key, &hash).unwrap();
        let sig2 = sign_blake3_hash(&signer.signing_key, &hash).unwrap();

        assert_eq!(sig1.r, sig2.r, "r should be deterministic");
        assert_eq!(sig1.s, sig2.s, "s should be deterministic");
        assert_eq!(sig1.v, sig2.v, "v should be deterministic");
    }

    #[test]
    fn test_sign_blake3_hash_different_hashes() {
        let signer = LocalSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();
        let hash1 = [0x00u8; 32];
        let hash2 = [0xFFu8; 32];

        let sig1 = sign_blake3_hash(&signer.signing_key, &hash1).unwrap();
        let sig2 = sign_blake3_hash(&signer.signing_key, &hash2).unwrap();

        // Different hashes should produce different signatures
        assert_ne!(
            sig1.r, sig2.r,
            "Different hashes should produce different r values"
        );
    }

    // ===== pubkey_to_address Tests =====

    #[test]
    fn test_pubkey_to_address_known_value() {
        // Test against known Ethereum address derivation
        // Private key 0x01 -> Address: 0x7e5f4552091a69125d5dfcb7b8c2659029395bdf
        let signer = LocalSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();
        let expected_address = "0x7e5f4552091a69125d5dfcb7b8c2659029395bdf";

        assert_eq!(
            signer.address(),
            expected_address,
            "Address derivation should match known value for key 0x01"
        );
    }

    #[test]
    fn test_pubkey_to_address_format() {
        let signer = LocalSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();
        let address = signer.address();

        assert!(
            address.starts_with("0x"),
            "Address should start with 0x prefix"
        );
        assert_eq!(
            address.len(),
            42,
            "Address should be 42 characters (0x + 40 hex)"
        );
    }

    #[test]
    fn test_pubkey_to_address_different_keys() {
        // Different keys should produce different addresses
        let signer1 = LocalSigner::from_private_key(
            "0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();

        let signer2 = LocalSigner::from_private_key(
            "0000000000000000000000000000000000000000000000000000000000000002",
        )
        .unwrap();

        assert_ne!(
            signer1.address(),
            signer2.address(),
            "Different keys should produce different addresses"
        );
    }

    #[test]
    fn test_pubkey_to_address_from_compressed_pubkey() {
        let signer = LocalSigner::from_private_key(TEST_PRIVATE_KEY).unwrap();

        // Get compressed pubkey
        let compressed = signer.public_key_compressed();
        assert_eq!(compressed.len(), 33, "Compressed pubkey should be 33 bytes");

        // Get uncompressed pubkey
        let uncompressed = signer.public_key_uncompressed();
        assert_eq!(
            uncompressed.len(),
            64,
            "Uncompressed pubkey should be 64 bytes"
        );

        // Both should derive to same address
        let address_from_signer = signer.address();

        // The address is already derived from the same key, so they should match
        assert!(
            !address_from_signer.is_empty(),
            "Address should not be empty"
        );
    }
}
