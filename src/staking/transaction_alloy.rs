//! EIP-1559 Transaction Builder using Alloy
//!
//! Modern implementation using Alloy's native transaction types.
//! This module provides proper EIP-1559 transaction signing compatible with
//! Ethereum's transaction format.

use crate::utils::error::{Error, Result};
use alloy_consensus::{SignableTransaction, TxEip1559, TxEnvelope};
use alloy_eips::eip2718::Encodable2718;
use alloy_primitives::{address, Address, Bytes, TxKind, B256, U256};
use alloy_rlp::Encodable;
use alloy_signer::{Signature, Signer};
use k256::ecdsa::SigningKey;
use k256::ecdsa::signature::Signer as K256Signer;
use std::str::FromStr;

/// EIP-1559 transaction wrapped with Alloy types
#[derive(Debug, Clone)]
pub struct AlloyTransaction {
    /// Chain ID (143 for mainnet, 10143 for testnet)
    pub chain_id: u64,
    /// Sender's nonce
    pub nonce: u64,
    /// Maximum priority fee per gas (tip to validator)
    pub max_priority_fee_per_gas: u128,
    /// Maximum total fee per gas (base fee + priority fee)
    pub max_fee_per_gas: u128,
    /// Gas limit for the transaction
    pub gas_limit: u64,
    /// Recipient address
    pub to: Address,
    /// Value to transfer in wei
    pub value: U256,
    /// Transaction data (calldata)
    pub data: Bytes,
}

impl AlloyTransaction {
    /// Create a new transaction builder
    pub fn new(chain_id: u64) -> Self {
        Self {
            chain_id,
            nonce: 0,
            max_priority_fee_per_gas: 1_000_000_000, // 1 Gwei
            max_fee_per_gas: 500_000_000_000,      // 500 Gwei
            gas_limit: 1_000_000,
            to: Address::ZERO,
            value: U256::ZERO,
            data: Bytes::default(),
        }
    }

    /// Set the nonce
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    /// Set the gas parameters
    pub fn with_gas(mut self, gas_limit: u64, max_fee: u128, max_priority_fee: u128) -> Self {
        self.gas_limit = gas_limit;
        self.max_fee_per_gas = max_fee;
        self.max_priority_fee_per_gas = max_priority_fee;
        self
    }

    /// Set the recipient address
    pub fn to(mut self, address: &str) -> Result<Self> {
        self.to = Address::from_str(address)
            .map_err(|_| Error::InvalidInput(format!("Invalid address: {}", address)))?;
        Ok(self)
    }

    /// Set the value to transfer
    pub fn with_value(mut self, value: u128) -> Self {
        self.value = U256::from(value);
        self
    }

    /// Set the transaction data (calldata)
    pub fn with_data(mut self, data: &[u8]) -> Self {
        self.data = Bytes::from(data.to_vec());
        self
    }

    /// Set the transaction data from hex string
    pub fn with_data_hex(mut self, hex: &str) -> Result<Self> {
        let clean = hex.strip_prefix("0x").unwrap_or(hex);
        self.data = Bytes::from(
            hex::decode(clean)
                .map_err(|e| Error::Serialization(format!("Failed to decode hex data: {}", e)))?,
        );
        Ok(self)
    }

    /// Sign the transaction with a private key
    ///
    /// # Arguments
    /// * `private_key` - 32-byte private key
    ///
    /// # Returns
    /// Signed transaction encoded for broadcast
    pub fn sign_with_private_key(&self, private_key: &[u8]) -> Result<Bytes> {
        if private_key.len() != 32 {
            return Err(Error::InvalidInput(
                "Private key must be 32 bytes".to_string(),
            ));
        }

        // Convert to k256 SigningKey
        let signing_key = SigningKey::from_slice(private_key)
            .map_err(|e| Error::InvalidInput(format!("Invalid private key: {:?}", e)))?;

        // Build TxEip1559 using alloy consensus types
        let tx = TxEip1559 {
            chain_id: self.chain_id,
            nonce: self.nonce,
            max_priority_fee_per_gas: self.max_priority_fee_per_gas,
            max_fee_per_gas: self.max_fee_per_gas,
            gas_limit: self.gas_limit,
            to: TxKind::Call(self.to),
            value: self.value,
            input: self.data.clone(),
            access_list: Default::default(),
        };

        // Create SignableTransaction
        let signable = SignableTransaction::from(tx);
        let signature = signing_key
            .sign(&signable.signature_hash())
            .map_err(|e| Error::Signing(format!("Failed to sign: {:?}", e)))?;

        // Convert k256 signature to alloy Signature
        let sig_bytes = signature.to_bytes();
        let r = U256::from_be_slice(&sig_bytes[..32]);
        let s = U256::from_be_slice(&sig_bytes[32..]);
        let v = signature.recovery_id().to_byte();

        // Create signature
        let alloy_sig = Signature { r, s, v: v as u64 };

        // Build TxEnvelope
        let envelope = TxEnvelope::Eip1559(alloy_consensus::Signed::new_unchecked(
            tx,
            alloy_sig,
        ));

        // Encode as EIP-2718
        let encoded = envelope.encoded_2718();
        Ok(Bytes::from(encoded.to_vec()))
    }

    /// Sign the transaction with a private key and return hex string
    pub fn sign_with_private_key_hex(&self, private_key: &[u8]) -> Result<String> {
        let signed = self.sign_with_private_key(private_key)?;
        Ok(format!("0x{}", hex::encode(signed.as_ref())))
    }

    /// Get the signing hash for this transaction
    pub fn signing_hash(&self, private_key: &[u8]) -> Result<[u8; 32]> {
        if private_key.len() != 32 {
            return Err(Error::InvalidInput(
                "Private key must be 32 bytes".to_string(),
            ));
        }

        let tx = TxEip1559 {
            chain_id: self.chain_id,
            nonce: self.nonce,
            max_priority_fee_per_gas: self.max_priority_fee_per_gas,
            max_fee_per_gas: self.max_fee_per_gas,
            gas_limit: self.gas_limit,
            to: TxKind::Call(self.to),
            value: self.value,
            input: self.data.clone(),
            access_list: Default::default(),
        };

        let signable = SignableTransaction::from(tx);
        Ok(signable.signature_hash().0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_default() {
        let tx = AlloyTransaction::new(143);
        assert_eq!(tx.chain_id, 143);
        assert_eq!(tx.nonce, 0);
        assert_eq!(tx.gas_limit, 1_000_000);
    }

    #[test]
    fn test_transaction_builder() {
        let tx = AlloyTransaction::new(10143)
            .with_nonce(5)
            .with_gas(500_000, 100_000_000_000, 2_000_000_000)
            .to("0x0000000000000000000000000000000000001000")
            .expect("Valid address")
            .with_value(1_000_000_000_000_000_000u128)
            .with_data_hex("0x84994fec0000000000000000000000000000000000000000000000000000000000000001")
            .expect("Valid hex");

        assert_eq!(tx.chain_id, 10143);
        assert_eq!(tx.nonce, 5);
        assert_eq!(tx.gas_limit, 500_000);
    }

    #[test]
    fn test_invalid_address() {
        let result = AlloyTransaction::new(143).to("0xinvalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_with_private_key() {
        // Create a test private key
        let private_key = hex::decode("0x0000000000000000000000000000000000000000000000000000000000000001")
            .expect("Valid hex");

        let tx = AlloyTransaction::new(143)
            .with_nonce(0)
            .to("0x0000000000000000000000000000000000001000")
            .expect("Valid address")
            .with_data(&[]);

        let result = tx.sign_with_private_key(&private_key);
        assert!(result.is_ok());

        let signed = result.unwrap();
        assert!(!signed.is_empty());
        assert_eq!(signed[0], 0x02); // EIP-1559 transaction type
    }

    #[test]
    fn test_signing_hash() {
        let private_key = hex::decode("0x0000000000000000000000000000000000000000000000000000000000000001")
            .expect("Valid hex");

        let tx = AlloyTransaction::new(143)
            .with_nonce(0)
            .to("0x0000000000000000000000000000000000001000")
            .expect("Valid address");

        let hash = tx.signing_hash(&private_key).expect("Valid hash");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_sign_with_private_key_hex() {
        let private_key = hex::decode("0x0000000000000000000000000000000000000000000000000000000000000001")
            .expect("Valid hex");

        let tx = AlloyTransaction::new(143)
            .with_nonce(0)
            .to("0x0000000000000000000000000000000000001000")
            .expect("Valid address")
            .with_data(&[]);

        let hex_result = tx.sign_with_private_key_hex(&private_key);
        assert!(hex_result.is_ok());

        let hex_string = hex_result.unwrap();
        assert!(hex_string.starts_with("0x02"));
    }

    #[test]
    fn test_invalid_private_key() {
        let invalid_key = vec![0u8; 16]; // Wrong length

        let tx = AlloyTransaction::new(143)
            .with_nonce(0)
            .to("0x0000000000000000000000000000000000001000")
            .expect("Valid address");

        let result = tx.sign_with_private_key(&invalid_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_transaction() {
        let private_key = hex::decode("0x0000000000000000000000000000000000000000000000000000000000000001")
            .expect("Valid hex");

        // Test with realistic staking transaction parameters
        let tx = AlloyTransaction::new(10143)
            .with_nonce(0)
            .with_gas(1_000_000, 500_000_000_000, 1_000_000_000)
            .to("0x0000000000000000000000000000000000001000")
            .expect("Valid address")
            .with_value(1_000_000_000_000_000_000u128) // 1 MON
            .with_data_hex("0x84994fec00000000000000000000000000000000000000000000000000000000000000e0")
            .expect("Valid hex");

        let signed = tx.sign_with_private_key(&private_key).expect("Valid signature");
        assert!(!signed.is_empty());
        assert_eq!(signed[0], 0x02); // EIP-1559 transaction type

        // Verify it's properly encoded
        let hex_string = hex::encode(signed.as_ref());
        assert!(hex_string.len() > 2);
    }

}
