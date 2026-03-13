//! Staking data types - Core structures for Monad staking operations
//!
//! These types represent the data structures returned by the staking contract
//! and used in staking operations.

use serde::{Deserialize, Serialize};

// =============================================================================
// EPOCH TYPES
// =============================================================================

/// Epoch information returned by `get_epoch()`
///
/// ABI returns: `(uint64 epoch, bool is_epoch_transition)`
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpochInfo {
    /// Current epoch number
    pub epoch: u64,
    /// Whether we are in an epoch transition period
    pub is_epoch_transition: bool,
}

// =============================================================================
// VALIDATOR TYPES
// =============================================================================

/// Validator information returned by `get_validator(uint64)`
///
/// ABI returns: `(address, uint256, uint256, uint256, uint256, uint256, uint256, uint256, uint256, uint256, bytes, bytes)`
///
/// Fields in order:
/// 1. auth_delegator - Authorized address for validator operations
/// 2. flags - Validator status flags (NOT commission!)
/// 3. execution_stake - Execution view: Total delegated MON
/// 4. accumulated_rewards_per_token - Accumulated rewards per token (scaled by 1e36)
/// 5. execution_commission - Execution view: Commission rate (scaled by 1e18, 100 = 1%)
/// 6. unclaimed_rewards - Unclaimed rewards in execution view
/// 7. consensus_stake - Consensus view: Total delegated MON
/// 8. consensus_commission - Consensus view: Commission rate (scaled by 1e18, 100 = 1%)
/// 9. snapshot_stake - Snapshot view: Total delegated MON
/// 10. snapshot_commission - Snapshot view: Commission rate (scaled by 1e18, 100 = 1%)
/// 11. secp_pub_key - Secp256k1 public key (64 bytes)
/// 12. bls_pub_key - BLS12-381 public key (48 bytes)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    /// Authorized address for validator operations
    pub auth_delegator: String,
    /// Validator status flags (active, slashed, etc.) - uint256 in contract
    pub flags: u128,
    /// Execution view: Total MON delegated to this validator
    pub execution_stake: u128,
    /// Accumulated rewards per token (scaled by 1e36 for precision)
    pub accumulated_rewards_per_token: u128,
    /// Execution view: Commission rate (scaled by 1e18, 100 = 1%)
    pub execution_commission: u128,
    /// Execution view: Unclaimed rewards
    pub unclaimed_rewards: u128,
    /// Consensus view: Total MON delegated to this validator
    pub consensus_stake: u128,
    /// Consensus view: Commission rate (scaled by 1e18, 100 = 1%)
    pub consensus_commission: u128,
    /// Snapshot view: Total MON delegated to this validator
    pub snapshot_stake: u128,
    /// Snapshot view: Commission rate (scaled by 1e18, 100 = 1%)
    pub snapshot_commission: u128,
    /// Secp256k1 public key (hex-encoded, 64 bytes)
    pub secp_pub_key: String,
    /// BLS12-381 public key (hex-encoded, 48 bytes)
    pub bls_pub_key: String,
}

impl Default for Validator {
    fn default() -> Self {
        Self {
            auth_delegator: String::with_capacity(42),
            flags: 0,
            execution_stake: 0,
            accumulated_rewards_per_token: 0,
            execution_commission: 0,
            unclaimed_rewards: 0,
            consensus_stake: 0,
            consensus_commission: 0,
            snapshot_stake: 0,
            snapshot_commission: 0,
            secp_pub_key: String::with_capacity(128),
            bls_pub_key: String::with_capacity(96),
        }
    }
}

impl Validator {
    /// Get commission rate as percentage (from execution view)
    /// Commission is stored as u128 scaled by 1e18 (100 = 1%)
    pub fn commission(&self) -> f64 {
        (self.execution_commission as f64) / 1e16
    }

    /// Get delegated amount (from execution view)
    pub fn delegated_amount(&self) -> u128 {
        self.execution_stake
    }

    /// Get status flags from the flags field
    pub fn status_flags(&self) -> u64 {
        // flags is u128, but status flags are in the lower 64 bits
        self.flags as u64
    }

    /// Check if validator is active
    pub fn is_active(&self) -> bool {
        (self.flags & (ValidatorStatus::ACTIVE as u128)) != 0
    }

    /// Check if validator is slashed
    pub fn is_slashed(&self) -> bool {
        (self.flags & (ValidatorStatus::SLASHED as u128)) != 0
    }

    /// Check if validator is in cooldown
    pub fn is_in_cooldown(&self) -> bool {
        (self.flags & (ValidatorStatus::COOLDOWN as u128)) != 0
    }

    /// Check if validator opted out
    pub fn is_opted_out(&self) -> bool {
        (self.flags & (ValidatorStatus::OPTED_OUT as u128)) != 0
    }
}

/// Validator status flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatorStatus(u64);

impl ValidatorStatus {
    /// Validator is active and can accept delegations
    pub const ACTIVE: u64 = 0x01;
    /// Validator has been slashed
    pub const SLASHED: u64 = 0x02;
    /// Validator is in cooldown period
    pub const COOLDOWN: u64 = 0x04;
    /// Validator has opted out of consensus
    pub const OPTED_OUT: u64 = 0x08;

    pub fn from_flags(flags: u64) -> Self {
        Self(flags)
    }

    pub fn is_active(&self) -> bool {
        (self.0 & Self::ACTIVE) != 0
    }

    pub fn is_slashed(&self) -> bool {
        (self.0 & Self::SLASHED) != 0
    }

    pub fn is_in_cooldown(&self) -> bool {
        (self.0 & Self::COOLDOWN) != 0
    }

    pub fn is_opted_out(&self) -> bool {
        (self.0 & Self::OPTED_OUT) != 0
    }
}

// =============================================================================
// DELEGATOR TYPES
// =============================================================================

/// Delegator information returned by `get_delegator(uint64, address)`
///
/// ABI returns: `(uint256, uint256, uint256, uint256, uint256, uint64, uint64)`
///
/// Fields in order:
/// 1. delegated_amount - Current delegated MON amount (Stake)
/// 2. accumulated_rewards_per_token - Rewards per token (fixed-point, divide by 1e36 for actual value)
/// 3. rewards - Unclaimed rewards (Total Rewards)
/// 4. delta_stake - Amount pending undelegation (Delta Stake)
/// 5. next_delta_stake - Next amount pending undelegation (Next Delta Stake)
/// 6. delta_epoch - Epoch when current undelegation completes (Delta Epoch)
/// 7. next_delta_epoch - Epoch when next undelegation completes (Next Delta Epoch)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delegator {
    /// Current delegated MON amount (Stake)
    pub delegated_amount: u128,
    /// Accumulated rewards per token (fixed-point, divide by 1e36 for actual value)
    pub accumulated_rewards_per_token: u128,
    /// Unclaimed rewards (Total Rewards)
    pub rewards: u128,
    /// Amount pending undelegation (Delta Stake)
    pub delta_stake: u128,
    /// Next amount pending undelegation (Next Delta Stake)
    pub next_delta_stake: u128,
    /// Epoch when current undelegation completes (Delta Epoch)
    pub delta_epoch: u64,
    /// Epoch when next undelegation completes (Next Delta Epoch)
    pub next_delta_epoch: u64,
}

// =============================================================================
// WITHDRAWAL TYPES
// =============================================================================

/// Withdrawal request information returned by `get_withdrawal_request(uint64, address, uint8)`
///
/// ABI returns: `(uint256, uint256, uint64)`
///
/// Fields in order:
/// 1. amount - Amount to withdraw
/// 2. withdrawal_index - Index of this withdrawal (for tracking)
/// 3. activation_epoch - Epoch when withdrawal can be executed
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WithdrawalRequest {
    /// Amount to withdraw
    pub amount: u128,
    /// Index of this withdrawal request (0-255)
    pub withdrawal_index: u8,
    /// Epoch when withdrawal becomes available
    pub activation_epoch: u64,
}

/// Maximum number of concurrent withdrawal requests per delegator per validator
pub const MAX_CONCURRENT_WITHDRAWALS: u8 = 8;

// =============================================================================
// VALIDATOR SET TYPES
// =============================================================================

/// Validator set information returned by `get_consensus_valset`, `get_snapshot_valset`, `get_execution_valset`
///
/// ABI returns: `(bool, uint64, uint64[])`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorSet {
    /// Whether more validators exist beyond this page
    pub has_more: bool,
    /// Total number of validators in set
    pub total_count: u64,
    /// List of validator IDs in this page
    pub validator_ids: Vec<u64>,
}

// =============================================================================
// DELEGATION TYPES
// =============================================================================

/// Delegation list returned by `get_delegations(address, uint64)`
///
/// ABI returns: `(bool, uint64, uint64[])`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationList {
    /// Whether more delegations exist beyond this page
    pub has_more: bool,
    /// Total delegation count
    pub total_count: u64,
    /// List of validator IDs the address has delegated to
    pub validator_ids: Vec<u64>,
}

/// Delegator list returned by `get_delegators(uint64, address)`
///
/// ABI returns: `(bool, address, address[])`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegatorList {
    /// Whether more delegators exist beyond this page
    pub has_more: bool,
    /// Last address in pagination
    pub last_address: String,
    /// List of delegator addresses
    pub addresses: Vec<String>,
}

// =============================================================================
// EVENT TYPES (for future event parsing)
// =============================================================================

/// Event signature for ValidatorCreated
pub const EVENT_VALIDATOR_CREATED: &str =
    "ValidatorCreated(uint64 indexed valId, address indexed auth_delegator, uint256 commission)";

/// Event signature for ValidatorStatusChanged
pub const EVENT_VALIDATOR_STATUS_CHANGED: &str =
    "ValidatorStatusChanged(uint64 indexed valId, uint64 flags)";

/// Event signature for Delegate
pub const EVENT_DELEGATE: &str = "Delegate(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 activationEpoch)";

/// Event signature for Undelegate
pub const EVENT_UNDELEGATE: &str = "Undelegate(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)";

/// Event signature for Withdraw
pub const EVENT_WITHDRAW: &str = "Withdraw(uint64 indexed valId, address indexed delegator, uint8 withdrawal_id, uint256 amount, uint64 activationEpoch)";

/// Event signature for ClaimRewards
pub const EVENT_CLAIM_REWARDS: &str =
    "ClaimRewards(uint64 indexed valId, address indexed delegator, uint256 amount, uint64 epoch)";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_info_default() {
        let epoch = EpochInfo::default();
        assert_eq!(epoch.epoch, 0);
        assert!(!epoch.is_epoch_transition);
    }

    #[test]
    fn test_validator_default() {
        let validator = Validator::default();
        assert!(validator.auth_delegator.is_empty());
        assert_eq!(validator.flags, 0);
        assert_eq!(validator.execution_stake, 0);
    }

    #[test]
    fn test_validator_status_flags() {
        let status =
            ValidatorStatus::from_flags(ValidatorStatus::ACTIVE | ValidatorStatus::COOLDOWN);
        assert!(status.is_active());
        assert!(status.is_in_cooldown());
        assert!(!status.is_slashed());
        assert!(!status.is_opted_out());
    }

    #[test]
    fn test_delegator_default() {
        let delegator = Delegator::default();
        assert_eq!(delegator.delegated_amount, 0);
        assert_eq!(delegator.rewards, 0);
    }

    #[test]
    fn test_withdrawal_request_default() {
        let withdrawal = WithdrawalRequest::default();
        assert_eq!(withdrawal.amount, 0);
        assert_eq!(withdrawal.withdrawal_index, 0);
    }

    #[test]
    fn test_max_concurrent_withdrawals() {
        assert_eq!(MAX_CONCURRENT_WITHDRAWALS, 8);
    }

    #[test]
    fn test_event_signatures_not_empty() {
        assert!(!EVENT_VALIDATOR_CREATED.is_empty());
        assert!(!EVENT_DELEGATE.is_empty());
        assert!(!EVENT_UNDELEGATE.is_empty());
        assert!(!EVENT_WITHDRAW.is_empty());
        assert!(!EVENT_CLAIM_REWARDS.is_empty());
    }
}
