//! EIP-1559 Transaction Builder for Monad staking operations
//!
//! This module provides structures and functions for building and encoding
//! EIP-1559 typed transactions for the Monad blockchain.
//!
//! # EIP-1559 Transaction Format
//!
//! ```text
//! 0x02 || rlp([
//!     chain_id,
//!     nonce,
//!     max_priority_fee_per_gas,
//!     max_fee_per_gas,
//!     gas_limit,
//!     destination,
//!     value,
//!     data,
//!     access_list,
//!     signature_y_parity,
//!     signature_r,
//!     signature_s
//! ])
//! ```

use crate::utils::error::{Error, Result};
use rlp::RlpStream;

/// EIP-1559 transaction type prefix
const EIP1559_TX_TYPE: u8 = 0x02;

/// Default gas limit for staking operations
pub const DEFAULT_GAS_LIMIT: u64 = 1_000_000;

/// Default max priority fee per gas (1 Gwei)
pub const DEFAULT_MAX_PRIORITY_FEE: u64 = 1_000_000_000;

/// Default max fee per gas (500 Gwei - Monad has higher fees)
pub const DEFAULT_MAX_FEE: u64 = 500_000_000_000;

/// EIP-1559 transaction structure (unsigned)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Eip1559Transaction {
    /// Chain ID (143 for mainnet, 10143 for testnet)
    pub chain_id: u64,
    /// Sender's nonce (from eth_getTransactionCount)
    pub nonce: u64,
    /// Maximum priority fee per gas (tip to miner)
    pub max_priority_fee_per_gas: u64,
    /// Maximum total fee per gas (base fee + priority fee)
    pub max_fee_per_gas: u64,
    /// Gas limit for the transaction
    pub gas_limit: u64,
    /// Recipient address (20 bytes, hex with 0x prefix)
    pub to: String,
    /// Value to transfer in wei
    pub value: u128,
    /// Transaction data (calldata)
    pub data: Vec<u8>,
    /// Access list (EIP-2930) - typically empty for simple transactions
    pub access_list: Vec<AccessListItem>,
}

/// Access list item for EIP-2930
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccessListItem {
    /// Address to access
    pub address: String,
    /// Storage keys to access
    pub storage_keys: Vec<[u8; 32]>,
}

impl Default for Eip1559Transaction {
    fn default() -> Self {
        Self {
            chain_id: 143, // Monad mainnet
            nonce: 0,
            max_priority_fee_per_gas: DEFAULT_MAX_PRIORITY_FEE,
            max_fee_per_gas: DEFAULT_MAX_FEE,
            gas_limit: DEFAULT_GAS_LIMIT,
            to: String::new(),
            value: 0,
            data: Vec::new(),
            access_list: Vec::new(),
        }
    }
}

impl Eip1559Transaction {
    /// Create a new EIP-1559 transaction builder
    pub fn new(chain_id: u64) -> Self {
        Self {
            chain_id,
            ..Default::default()
        }
    }

    /// Set the nonce
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    /// Set the gas parameters
    pub fn with_gas(mut self, gas_limit: u64, max_fee: u64, max_priority_fee: u64) -> Self {
        self.gas_limit = gas_limit;
        self.max_fee_per_gas = max_fee;
        self.max_priority_fee_per_gas = max_priority_fee;
        self
    }

    /// Set the recipient address
    pub fn to(mut self, address: &str) -> Result<Self> {
        let clean = address.strip_prefix("0x").unwrap_or(address);
        if clean.len() != 40 {
            return Err(Error::InvalidInput(format!(
                "Invalid address length: {}",
                clean.len()
            )));
        }
        self.to = address.to_string();
        Ok(self)
    }

    /// Set the value to transfer
    pub fn with_value(mut self, value: u128) -> Self {
        self.value = value;
        self
    }

    /// Set the transaction data (calldata)
    pub fn with_data(mut self, data: &[u8]) -> Self {
        self.data = data.to_vec();
        self
    }

    /// Set the transaction data from hex string
    pub fn with_data_hex(mut self, hex: &str) -> Result<Self> {
        let clean = hex.strip_prefix("0x").unwrap_or(hex);
        self.data = hex::decode(clean)
            .map_err(|e| Error::Serialization(format!("Failed to decode hex data: {}", e)))?;
        Ok(self)
    }

    /// Encode the unsigned transaction for signing
    ///
    /// Returns the RLP-encoded transaction without signature
    /// This is what gets hashed and signed
    pub fn encode_for_signing(&self) -> Vec<u8> {
        // Build the transaction array for signing
        // [chain_id, nonce, max_priority_fee, max_fee, gas_limit, to, value, data, access_list]
        let mut stream = RlpStream::new();
        stream.begin_list(9);
        stream.append(&self.chain_id);
        stream.append(&self.nonce);
        stream.append(&self.max_priority_fee_per_gas);
        stream.append(&self.max_fee_per_gas);
        stream.append(&self.gas_limit);

        // 'to' field - 20 bytes address
        let to_bytes =
            hex::decode(self.to.strip_prefix("0x").unwrap_or(&self.to)).unwrap_or_default();
        stream.append(&to_bytes.as_slice());

        // Value as variable-length integer (RLP encoding)
        let value_bytes = encode_u128_variable_length(self.value);
        stream.append(&value_bytes.as_slice());

        // Data
        stream.append(&self.data.as_slice());

        // Access list (empty for simple transactions)
        stream.begin_list(0);

        stream.out().to_vec()
    }

    /// Compute the hash to be signed
    ///
    /// Returns keccak256(0x02 || rlp(tx_payload))
    pub fn signing_hash(&self) -> [u8; 32] {
        let encoded = self.encode_for_signing();
        let mut payload = vec![EIP1559_TX_TYPE];
        payload.extend_from_slice(&encoded);
        keccak256(&payload)
    }

    /// Encode the signed transaction for broadcasting
    ///
    /// # Arguments
    /// * `v` - Recovery ID (y-parity, 0 or 1)
    /// * `r` - Signature r component (32 bytes)
    /// * `s` - Signature s component (32 bytes)
    ///
    /// # Returns
    /// Raw signed transaction bytes (ready for eth_sendRawTransaction)
    pub fn encode_signed(&self, v: u8, r: &[u8], s: &[u8]) -> Result<Vec<u8>> {
        if r.len() != 32 || s.len() != 32 {
            return Err(Error::InvalidInput(
                "Signature r and s must be 32 bytes each".to_string(),
            ));
        }

        let mut stream = RlpStream::new();
        stream.begin_list(12);
        stream.append(&self.chain_id);
        stream.append(&self.nonce);
        stream.append(&self.max_priority_fee_per_gas);
        stream.append(&self.max_fee_per_gas);
        stream.append(&self.gas_limit);

        // 'to' field
        let to_bytes = hex::decode(self.to.strip_prefix("0x").unwrap_or(&self.to))
            .map_err(|e| Error::Serialization(format!("Invalid address: {}", e)))?;
        stream.append(&to_bytes.as_slice());

        // Value as variable-length integer (RLP encoding)
        let value_bytes = encode_u128_variable_length(self.value);
        stream.append(&value_bytes.as_slice());

        // Data
        stream.append(&self.data.as_slice());

        // Access list (empty)
        stream.begin_list(0);

        // Signature
        stream.append(&v);
        stream.append(&r);
        stream.append(&s);

        Ok(stream.out().to_vec())
    }

    /// Encode signed transaction as hex string for RPC
    pub fn encode_signed_hex(&self, v: u8, r: &[u8], s: &[u8]) -> Result<String> {
        let raw = self.encode_signed(v, r, s)?;
        let mut result = vec![EIP1559_TX_TYPE];
        result.extend_from_slice(&raw);
        Ok(format!("0x{}", hex::encode(result)))
    }
}

/// Encode u128 as variable-length big-endian bytes for RLP
///
/// RLP encodes integers as compact byte arrays without leading zeros.
/// For zero, returns a single zero byte.
fn encode_u128_variable_length(value: u128) -> Vec<u8> {
    if value == 0 {
        return vec![0u8];
    }

    let be_bytes = value.to_be_bytes();
    // Find first non-zero byte
    let skip = be_bytes.iter().take_while(|&&b| b == 0).count();
    be_bytes[skip..].to_vec()
}

/// Encode u128 as 32-byte big-endian bytes (deprecated, used only for compatibility tests)
#[allow(dead_code)]
fn encode_u128_to_32_bytes(value: u128) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    bytes[16..32].copy_from_slice(&value.to_be_bytes());
    bytes
}

/// Compute keccak256 hash (Ethereum's Keccak-256, not NIST SHA3-256)
fn keccak256(data: &[u8]) -> [u8; 32] {
    use sha3::Digest;
    let mut hasher = sha3::Keccak256::new();
    hasher.update(data);
    let mut result = [0u8; 32];
    result.copy_from_slice(&hasher.finalize());
    result
}

/// Compute keccak256 hash - public interface
pub fn keccak256_hash(data: &[u8]) -> [u8; 32] {
    keccak256(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_default() {
        let tx = Eip1559Transaction::default();
        assert_eq!(tx.chain_id, 143);
        assert_eq!(tx.nonce, 0);
        assert_eq!(tx.gas_limit, DEFAULT_GAS_LIMIT);
    }

    #[test]
    fn test_transaction_builder() {
        let tx = Eip1559Transaction::new(10143)
            .with_nonce(5)
            .with_gas(500_000, 100_000_000_000, 2_000_000_000)
            .to("0x0000000000000000000000000000000000001000")
            .expect("Valid address")
            .with_value(1_000_000_000_000_000_000u128) // 1 MON
            .with_data_hex(
                "0x84994fec0000000000000000000000000000000000000000000000000000000000000001",
            )
            .expect("Valid hex");

        assert_eq!(tx.chain_id, 10143);
        assert_eq!(tx.nonce, 5);
        assert_eq!(tx.gas_limit, 500_000);
        assert_eq!(tx.value, 1_000_000_000_000_000_000u128);
        assert!(!tx.data.is_empty());
    }

    #[test]
    fn test_encode_for_signing() {
        let tx = Eip1559Transaction::new(143)
            .with_nonce(0)
            .to("0x0000000000000000000000000000000000001000")
            .expect("Valid address")
            .with_data(&[]);

        let encoded = tx.encode_for_signing();
        // encode_for_signing returns RLP-encoded payload WITHOUT type prefix
        // The first byte should be the RLP list prefix (not EIP1559_TX_TYPE)
        assert!(encoded.len() > 10);
        // Verify it's a valid RLP list (first byte >= 0xc0)
        assert!(encoded[0] >= 0xc0);
    }

    #[test]
    fn test_signing_hash() {
        let tx = Eip1559Transaction::new(143)
            .with_nonce(0)
            .to("0x0000000000000000000000000000000000001000")
            .expect("Valid address");

        let hash = tx.signing_hash();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_encode_signed() {
        let tx = Eip1559Transaction::new(143)
            .with_nonce(0)
            .to("0x0000000000000000000000000000000000001000")
            .expect("Valid address");

        let r = [0u8; 32];
        let s = [1u8; 32];
        let v = 0u8;

        let encoded = tx.encode_signed(v, &r, &s).expect("Valid signature");
        // encode_signed returns RLP-encoded payload WITHOUT type prefix
        // The first byte should be the RLP list prefix
        assert!(encoded.len() > 100);
        assert!(encoded[0] >= 0xc0); // RLP list prefix
    }

    #[test]
    fn test_encode_signed_hex() {
        let tx = Eip1559Transaction::new(143)
            .with_nonce(0)
            .to("0x0000000000000000000000000000000000001000")
            .expect("Valid address");

        let r = [0u8; 32];
        let s = [1u8; 32];
        let v = 0u8;

        let hex = tx.encode_signed_hex(v, &r, &s).expect("Valid signature");
        // encode_signed_hex adds the type prefix
        assert!(hex.starts_with("0x02"));
    }

    #[test]
    fn test_encode_u128_variable_length() {
        // Test zero value
        let result = encode_u128_variable_length(0u128);
        assert_eq!(result, vec![0u8]);

        // Test value 1
        let result = encode_u128_variable_length(1u128);
        assert_eq!(result, vec![1u8]);

        // Test value 256 (needs 2 bytes)
        let result = encode_u128_variable_length(256u128);
        assert_eq!(result, vec![1u8, 0u8]);

        // Test 1 MON (0x0de0b6b3a7640000)
        let result = encode_u128_variable_length(1_000_000_000_000_000_000u128);
        assert_eq!(result.len(), 8); // Should be 8 bytes, not 32
        assert_eq!(result[0], 0x0d);

        // Test maximum value
        let result = encode_u128_variable_length(u128::MAX);
        assert_eq!(result.len(), 16); // u128 MAX needs 16 bytes
    }

    #[test]
    #[allow(deprecated)]
    fn test_encode_u128_to_32_bytes_deprecated() {
        let result = encode_u128_to_32_bytes(1u128);
        // Should be right-padded with zeros in first 16 bytes
        assert_eq!(result[31], 1);
        assert_eq!(result[0], 0);

        let result = encode_u128_to_32_bytes(u128::MAX);
        // Last 16 bytes should be all 0xFF
        for byte in result.iter().skip(16) {
            assert_eq!(*byte, 0xFF);
        }
    }

    #[test]
    fn test_invalid_address() {
        let result = Eip1559Transaction::new(143).to("0x123");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_signature_length() {
        let tx = Eip1559Transaction::new(143)
            .to("0x0000000000000000000000000000000000001000")
            .expect("Valid address");

        let r = [0u8; 16]; // Wrong length
        let s = [1u8; 32];
        let result = tx.encode_signed(0, &r, &s);
        assert!(result.is_err());
    }
}
