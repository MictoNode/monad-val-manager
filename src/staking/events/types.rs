//! Event data types for staking contract events
//!
//! This module defines the data structures used to represent parsed staking events.
//! Each event type corresponds to a specific staking operation.

// =============================================================================
// TRANSACTION LOG
// =============================================================================

/// Represents a transaction log entry
///
/// Transaction logs are emitted by smart contracts during execution.
/// Each log contains the contract address, indexed topics, and non-indexed data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionLog {
    /// Contract address that emitted the event
    pub address: String,
    /// Indexed parameters (topic[0] is event signature)
    pub topics: Vec<String>,
    /// Non-indexed parameters (ABI-encoded)
    pub data: String,
}

// =============================================================================
// STAKING EVENT ENUM
// =============================================================================

/// Parsed staking event types
///
/// This enum represents all possible staking contract events.
/// Use pattern matching to handle specific event types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StakingEvent {
    /// Delegate event: delegator delegated to validator
    Delegate(DelegateEvent),
    /// Undelegate event: delegator undelegated from validator
    Undelegate(UndelegateEvent),
    /// Withdraw event: delegator withdrew funds
    Withdraw(WithdrawEvent),
    /// ClaimRewards event: delegator claimed rewards
    ClaimRewards(ClaimRewardsEvent),
    /// Compound event: delegator compounded rewards
    Compound(CompoundEvent),
    /// ChangeCommission event: validator changed commission
    ChangeCommission(ChangeCommissionEvent),
    /// AddValidator event: new validator registered
    AddValidator(AddValidatorEvent),
    /// ValidatorCreated event: emitted when a new validator is created
    ValidatorCreated(ValidatorCreatedEvent),
    /// ValidatorStatusChanged event: emitted when validator status changes
    ValidatorStatusChanged(ValidatorStatusChangedEvent),
}

// =============================================================================
// INDIVIDUAL EVENT STRUCTS
// =============================================================================

/// Delegate event data
///
/// Event signature: `Delegate(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 activationEpoch)`
///
/// Topics layout:
/// - topics[0]: event signature hash
/// - topics[1]: valId (uint64 indexed)
/// - topics[2]: delegator (address indexed)
///
/// Data layout (ABI-encoded, each slot 32 bytes):
/// - slot 0: amount (uint256)
/// - slot 1: activationEpoch (uint64)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegateEvent {
    /// Validator ID (indexed, in topics[1])
    pub validator_id: u64,
    /// Delegator address (indexed, in topics[2])
    pub delegator: String,
    /// Amount delegated (non-indexed, in data slot 0)
    pub amount: u128,
    /// Epoch when delegation becomes active (non-indexed, in data slot 1)
    pub activation_epoch: u64,
}

/// Undelegate event data
///
/// Event signature: `Undelegate(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)`
///
/// Topics layout:
/// - topics[0]: event signature hash
/// - topics[1]: valId (uint64 indexed)
/// - topics[2]: delegator (address indexed)
///
/// Data layout (ABI-encoded, each slot 32 bytes):
/// - slot 0: withdrawal_id (uint8)
/// - slot 1: amount (uint256)
/// - slot 2: activationEpoch (uint64)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndelegateEvent {
    /// Validator ID (indexed, in topics[1])
    pub validator_id: u64,
    /// Delegator address (indexed, in topics[2])
    pub delegator: String,
    /// Withdrawal request ID (non-indexed, in data slot 0)
    pub withdrawal_id: u8,
    /// Amount undelegated (non-indexed, in data slot 1)
    pub amount: u128,
    /// Epoch when withdrawal becomes available (non-indexed, in data slot 2)
    pub activation_epoch: u64,
}

/// Withdraw event data
///
/// Event signature: `Withdraw(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)`
///
/// Topics layout:
/// - topics[0]: event signature hash
/// - topics[1]: valId (uint64 indexed)
/// - topics[2]: delegator (address indexed)
///
/// Data layout (ABI-encoded, each slot 32 bytes):
/// - slot 0: withdrawal_id (uint8)
/// - slot 1: amount (uint256)
/// - slot 2: activationEpoch (uint64)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithdrawEvent {
    /// Validator ID (indexed, in topics[1])
    pub validator_id: u64,
    /// Delegator address (indexed, in topics[2])
    pub delegator: String,
    /// Withdrawal request ID (non-indexed, in data slot 0)
    pub withdrawal_id: u8,
    /// Amount withdrawn (non-indexed, in data slot 1)
    pub amount: u128,
    /// Epoch when withdrawal was processed (non-indexed, in data slot 2)
    pub activation_epoch: u64,
}

/// ClaimRewards event data
///
/// Event signature: `ClaimRewards(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 epoch)`
///
/// Topics layout:
/// - topics[0]: event signature hash
/// - topics[1]: valId (uint64 indexed)
/// - topics[2]: delegator (address indexed)
///
/// Data layout (ABI-encoded, each slot 32 bytes):
/// - slot 0: amount (uint256)
/// - slot 1: epoch (uint64)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimRewardsEvent {
    /// Validator ID (indexed, in topics[1])
    pub validator_id: u64,
    /// Delegator address (indexed, in topics[2])
    pub delegator: String,
    /// Amount claimed (non-indexed, in data slot 0)
    pub amount: u128,
    /// Epoch when rewards were claimed (non-indexed, in data slot 1)
    pub epoch: u64,
}

/// Compound event data
///
/// Event signature: `Compound(uint64 indexed valId, address indexed delegator, uint256 amount)`
///
/// Topics layout:
/// - topics[0]: event signature hash
/// - topics[1]: valId (uint64 indexed)
/// - topics[2]: delegator (address indexed)
///
/// Data layout (ABI-encoded, each slot 32 bytes):
/// - slot 0: amount (uint256)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundEvent {
    /// Validator ID (indexed, in topics[1])
    pub validator_id: u64,
    /// Delegator address (indexed, in topics[2])
    pub delegator: String,
    /// Amount compounded (non-indexed, in data slot 0)
    pub amount: u128,
}

/// ChangeCommission event data
///
/// Event signature: `ChangeCommission(uint64 indexed valId, uint256 old_commission, uint256 new_commission)`
///
/// Topics layout:
/// - topics[0]: event signature hash
/// - topics[1]: valId (uint64 indexed)
///
/// Data layout (ABI-encoded, each slot 32 bytes):
/// - slot 0: old_commission (uint256)
/// - slot 1: new_commission (uint256)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeCommissionEvent {
    /// Validator ID (indexed, in topics[1])
    pub validator_id: u64,
    /// Old commission rate in basis points
    pub old_commission: u64,
    /// New commission rate in basis points
    pub new_commission: u64,
}

/// AddValidator event data
///
/// Event signature: `AddValidator(address indexed auth_delegator, uint64 indexed valId, bytes secp_pubkey, bytes bls_pubkey)`
///
/// Topics layout:
/// - topics[0]: event signature hash
/// - topics[1]: auth_delegator (address indexed)
/// - topics[2]: valId (uint64 indexed)
///
/// Data layout (ABI-encoded, dynamic bytes):
/// - secp_pubkey offset, bls_pubkey offset, secp_pubkey length, secp_pubkey data, bls_pubkey length, bls_pubkey data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddValidatorEvent {
    /// Owner/auth delegator address (indexed, in topics[1])
    pub owner: String,
    /// Validator ID (indexed, in topics[2])
    pub validator_id: u64,
    /// SECP256k1 public key (non-indexed, in data)
    pub secp_pubkey: String,
    /// BLS12-381 public key (non-indexed, in data)
    pub bls_pubkey: String,
}

/// ValidatorCreated event data
///
/// Event signature: `ValidatorCreated(uint64 indexed valId, address indexed auth_delegator, uint256 commission)`
///
/// Topics layout:
/// - topics[0]: event signature hash
/// - topics[1]: valId (uint64 indexed)
/// - topics[2]: auth_delegator (address indexed)
///
/// Data layout (ABI-encoded, each slot 32 bytes):
/// - slot 0: commission (uint256)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorCreatedEvent {
    /// Validator ID (indexed, in topics[1])
    pub validator_id: u64,
    /// Authorized delegator address (indexed, in topics[2])
    pub auth_delegator: String,
    /// Commission rate in basis points (non-indexed, in data slot 0)
    pub commission: u64,
}

/// ValidatorStatusChanged event data
///
/// Event signature: `ValidatorStatusChanged(uint64 indexed valId, uint64 flags)`
///
/// Topics layout:
/// - topics[0]: event signature hash
/// - topics[1]: valId (uint64 indexed)
///
/// Data layout (ABI-encoded, each slot 32 bytes):
/// - slot 0: flags (uint64)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorStatusChangedEvent {
    /// Validator ID (indexed, in topics[1])
    pub validator_id: u64,
    /// Status flags (non-indexed, in data slot 0)
    pub flags: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegate_event_debug_impl() {
        let event = DelegateEvent {
            validator_id: 1,
            delegator: "0xabc".to_string(),
            amount: 100,
            activation_epoch: 50,
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("DelegateEvent"));
        assert!(debug_str.contains("validator_id"));
        assert!(debug_str.contains("activation_epoch"));
    }

    #[test]
    fn test_staking_event_equality() {
        let event1 = StakingEvent::Delegate(DelegateEvent {
            validator_id: 1,
            delegator: "0xabc".to_string(),
            amount: 100,
            activation_epoch: 50,
        });
        let event2 = StakingEvent::Delegate(DelegateEvent {
            validator_id: 1,
            delegator: "0xabc".to_string(),
            amount: 100,
            activation_epoch: 50,
        });
        let event3 = StakingEvent::Delegate(DelegateEvent {
            validator_id: 2,
            delegator: "0xabc".to_string(),
            amount: 100,
            activation_epoch: 50,
        });

        assert_eq!(event1, event2);
        assert_ne!(event1, event3);
    }
}
