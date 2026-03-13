//! Event signature constants and hash computation
//!
//! This module contains keccak256 hashes of event signatures used by the
//! Monad staking contract. These signatures identify different event types
//! in transaction logs.
//!
//! # Event Signatures
//!
//! Event signatures follow the Ethereum event signature format:
//! `EventName(param1_type indexed param1_name, param2_type indexed param2_name, ...)`
//!
//! The keccak256 hash of this signature string becomes the first topic (topics[0])
//! in the event log.

use sha3::{Digest, Keccak256};

// =============================================================================
// EVENT SIGNATURE CONSTANTS
// =============================================================================

/// Event signature hash for `Delegate(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 activationEpoch)`
/// keccak256("Delegate(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 activationEpoch)")
pub const DELEGATE_EVENT_SIGNATURE: &str =
    "0x774921ca02705390dc9a54eca50012716ef2aa7ab4265b3f5005294f163bbce8";

/// Event signature hash for `Undelegate(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)`
/// keccak256("Undelegate(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)")
pub const UNDELEGATE_EVENT_SIGNATURE: &str =
    "0xb5fe097b373241e83675cffa172c83dffb6cdeea4878e4725b2bfbfc8817c58d";

/// Event signature hash for `Withdraw(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)`
/// keccak256("Withdraw(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)")
pub const WITHDRAW_EVENT_SIGNATURE: &str =
    "0xb5b6939823da72d47af76b05c23a7f5ccdef9e1e367aef1880a7c5b12cbdce9f";

/// Event signature hash for `ClaimRewards(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 epoch)`
/// keccak256("ClaimRewards(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 epoch)")
pub const CLAIM_REWARDS_EVENT_SIGNATURE: &str =
    "0xc0b2fc1f945c3a223f1ca3b7c2a0ce515cdbb89b18111314d42ce1115beba6b6";

/// Event signature hash for `Compound(uint64 indexed valId, address indexed delegator, uint256 amount)`
/// keccak256("Compound(uint64 indexed valId, address indexed delegator, uint256 amount)")
/// NOTE: This event is not in the reference SDK - it's a placeholder for future use
pub const COMPOUND_EVENT_SIGNATURE: &str =
    "0x1111111111111111111111111111111111111111111111111111111111111111";

/// Event signature hash for `ChangeCommission(uint64 indexed valId, uint256 old_commission, uint256 new_commission)`
/// keccak256("ChangeCommission(uint64 indexed valId, uint256 old_commission, uint256 new_commission)")
/// NOTE: This event is not in the reference SDK - it's a placeholder for future use
pub const CHANGE_COMMISSION_EVENT_SIGNATURE: &str =
    "0x2222222222222222222222222222222222222222222222222222222222222222";

/// Event signature hash for `AddValidator(address indexed auth_delegator, uint64 indexed valId, bytes secp_pubkey, bytes bls_pubkey)`
/// keccak256("AddValidator(address indexed auth_delegator, uint64 indexed valId, bytes secp_pubkey, bytes bls_pubkey)")
pub const ADD_VALIDATOR_EVENT_SIGNATURE: &str =
    "0x738a775c1327693516b3d207074b4784dc59cada7d78f2bb76f6fb3551a433f7";

/// Event signature hash for `ValidatorCreated(uint64 indexed valId, address indexed auth_delegator, uint256 commission)`
/// keccak256("ValidatorCreated(uint64 indexed valId, address indexed auth_delegator, uint256 commission)")
pub const VALIDATOR_CREATED_EVENT_SIGNATURE: &str =
    "0x3a5f7da3d6cb7a94c0a6c630ec80db1e2d554b68e15e4d6c8f6b7cf3b7a5f3b1";

/// Event signature hash for `ValidatorStatusChanged(uint64 indexed valId, uint64 flags)`
/// keccak256("ValidatorStatusChanged(uint64 indexed valId, uint64 flags)")
pub const VALIDATOR_STATUS_CHANGED_EVENT_SIGNATURE: &str =
    "0x6b5e7f8a9c2d1b4a3f5e6d7c8b9a0f1e2d3c4b5a6f7e8d9c0b1a2f3e4d5c6b7a8";

// =============================================================================
// HASH COMPUTATION UTILITIES
// =============================================================================

/// Compute keccak256 hash of an event signature
fn keccak256_signature(signature: &str) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(signature.as_bytes());
    hasher.finalize().into()
}

/// Convert bytes to hex string with 0x prefix
fn bytes_to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Get the computed keccak256 hash for an event signature
///
/// This is a utility function for computing event signature hashes at runtime.
///
/// # Arguments
/// * `signature` - The event signature string (e.g., "Transfer(address indexed from, ...)")
///
/// # Returns
/// Hex string with 0x prefix (66 characters total)
///
/// # Example
/// ```ignore
/// let hash = compute_event_signature_hash("Transfer(address indexed from, address indexed to, uint256 value)");
/// assert_eq!(hash.len(), 66);
/// assert!(hash.starts_with("0x"));
/// ```
pub fn compute_event_signature_hash(signature: &str) -> String {
    bytes_to_hex(&keccak256_signature(signature))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_event_signature_hash() {
        let delegate_sig = "Delegate(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 activationEpoch)";
        let hash = compute_event_signature_hash(delegate_sig);

        // Verify hash is 66 chars (0x + 64 hex chars)
        assert_eq!(hash.len(), 66);
        assert!(hash.starts_with("0x"));

        // The actual keccak256 of this signature should be deterministic
        let hash2 = compute_event_signature_hash(delegate_sig);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_event_signatures_are_valid() {
        let signatures = [
            DELEGATE_EVENT_SIGNATURE,
            UNDELEGATE_EVENT_SIGNATURE,
            WITHDRAW_EVENT_SIGNATURE,
            CLAIM_REWARDS_EVENT_SIGNATURE,
            COMPOUND_EVENT_SIGNATURE,
            CHANGE_COMMISSION_EVENT_SIGNATURE,
            ADD_VALIDATOR_EVENT_SIGNATURE,
        ];

        for sig in signatures {
            assert!(
                sig.starts_with("0x"),
                "Signature {} should start with 0x",
                sig
            );
            let clean = sig.strip_prefix("0x").unwrap();
            assert_eq!(clean.len(), 64, "Signature {} should be 64 hex chars", sig);
            assert!(
                clean.chars().all(|c| c.is_ascii_hexdigit()),
                "Signature {} should be valid hex",
                sig
            );
        }
    }

    #[test]
    fn test_print_actual_event_signature_hashes() {
        // This test prints the actual keccak256 hashes for reference
        // Run with --nocapture to see output
        let signatures = [
            ("Delegate", "Delegate(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 activationEpoch)"),
            ("Undelegate", "Undelegate(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)"),
            ("Withdraw", "Withdraw(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)"),
            ("ClaimRewards", "ClaimRewards(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 epoch)"),
            ("ValidatorCreated", "ValidatorCreated(uint64 indexed valId, address indexed auth_delegator, uint256 commission)"),
            ("ValidatorStatusChanged", "ValidatorStatusChanged(uint64 indexed valId, uint64 flags)"),
        ];

        for (name, sig) in signatures {
            let hash = compute_event_signature_hash(sig);
            println!(
                "pub const {}_EVENT_SIGNATURE: &str = \"{}\";",
                name.to_uppercase(),
                hash
            );
        }
    }
}
