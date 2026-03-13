//! Staking contract constants - Selectors and addresses
//!
//! These selectors are the first 4 bytes of the keccak256 hash of the function signature.

// =============================================================================
// WRITE SELECTORS - State-modifying operations
// =============================================================================

/// Selector for `add_validator(bytes,bytes,bytes)` - Register a new validator
pub const ADD_VALIDATOR_SELECTOR: &str = "f145204c";

/// Selector for `delegate(uint64)` - Delegate MON to a validator
pub const DELEGATE_SELECTOR: &str = "84994fec";

/// Selector for `undelegate(uint64,uint256,uint8)` - Undelegate MON from a validator
pub const UNDELEGATE_SELECTOR: &str = "5cf41514";

/// Selector for `withdraw(uint64,uint8)` - Withdraw undelegated MON
pub const WITHDRAW_SELECTOR: &str = "aed2ee73";

// =============================================================================
// WITHDRAWAL CONSTANTS
// =============================================================================

/// Number of epochs to wait after undelegation before withdrawal can be claimed
/// Matches Python SDK: WITHDRAWAL_DELAY = 1
pub const WITHDRAWAL_DELAY: u64 = 1;

/// Selector for `claim_rewards(uint64)` - Claim staking rewards
pub const CLAIM_REWARDS_SELECTOR: &str = "a76e2ca5";

/// Selector for `compound(uint64)` - Compound staking rewards
pub const COMPOUND_SELECTOR: &str = "b34fea67";

/// Selector for `change_commission(uint64,uint256)` - Change validator commission
pub const CHANGE_COMMISSION_SELECTOR: &str = "9bdcc3c8";

// =============================================================================
// READ SELECTORS - View operations
// =============================================================================

/// Selector for `get_epoch()` - Get current epoch info
pub const GET_EPOCH_SELECTOR: &str = "757991a8";

/// Selector for `get_validator(uint64)` - Get validator info by ID
pub const GET_VALIDATOR_SELECTOR: &str = "2b6d639a";

/// Selector for `get_delegator(uint64,address)` - Get delegator info
pub const GET_DELEGATOR_SELECTOR: &str = "573c1ce0";

/// Selector for `get_withdrawal_request(uint64,address,uint8)` - Get withdrawal request
pub const GET_WITHDRAWAL_REQUEST_SELECTOR: &str = "56fa2045";

/// Selector for `get_proposer_val_id()` - Get current proposer validator ID
pub const GET_PROPOSER_VAL_ID_SELECTOR: &str = "fbacb0be";

/// Selector for `get_consensus_valset(uint64)` - Get consensus validator set
pub const GET_CONSENSUS_VALSET_SELECTOR: &str = "fb29b729";

/// Selector for `get_snapshot_valset(uint64)` - Get snapshot validator set
pub const GET_SNAPSHOT_VALSET_SELECTOR: &str = "de66a368";

/// Selector for `get_execution_valset(uint64)` - Get execution validator set
pub const GET_EXECUTION_VALSET_SELECTOR: &str = "7cb074df";

/// Selector for `get_delegations(address,uint64)` - Get delegations by address
pub const GET_DELEGATIONS_SELECTOR: &str = "4fd66050";

/// Selector for `get_delegators(uint64,address)` - Get delegators for validator
pub const GET_DELEGATORS_SELECTOR: &str = "a0843a26";

// =============================================================================
// CONTRACT ADDRESS
// =============================================================================

/// Monad staking contract address (predeployed)
pub const STAKING_CONTRACT_ADDRESS: &str = "0x0000000000000000000000000000000000001000";

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Prepend "0x" prefix to a hex string if not already present
pub fn with_0x_prefix(hex: &str) -> String {
    if hex.starts_with("0x") {
        hex.to_string()
    } else {
        format!("0x{}", hex)
    }
}

/// Strip "0x" prefix from a hex string if present
pub fn strip_0x_prefix(hex: &str) -> &str {
    hex.strip_prefix("0x").unwrap_or(hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selectors_are_valid_hex() {
        // All selectors should be 8 hex characters (4 bytes)
        let selectors = [
            ADD_VALIDATOR_SELECTOR,
            DELEGATE_SELECTOR,
            UNDELEGATE_SELECTOR,
            WITHDRAW_SELECTOR,
            CLAIM_REWARDS_SELECTOR,
            COMPOUND_SELECTOR,
            CHANGE_COMMISSION_SELECTOR,
            GET_EPOCH_SELECTOR,
            GET_VALIDATOR_SELECTOR,
            GET_DELEGATOR_SELECTOR,
            GET_WITHDRAWAL_REQUEST_SELECTOR,
            GET_PROPOSER_VAL_ID_SELECTOR,
            GET_CONSENSUS_VALSET_SELECTOR,
            GET_SNAPSHOT_VALSET_SELECTOR,
            GET_EXECUTION_VALSET_SELECTOR,
            GET_DELEGATIONS_SELECTOR,
            GET_DELEGATORS_SELECTOR,
        ];

        for selector in selectors {
            assert_eq!(selector.len(), 8, "Selector {} should be 8 chars", selector);
            assert!(
                selector.chars().all(|c| c.is_ascii_hexdigit()),
                "Selector {} should be valid hex",
                selector
            );
        }
    }

    #[test]
    fn test_staking_contract_address() {
        assert_eq!(STAKING_CONTRACT_ADDRESS.len(), 42);
        assert!(STAKING_CONTRACT_ADDRESS.starts_with("0x"));
    }

    #[test]
    fn test_with_0x_prefix() {
        assert_eq!(with_0x_prefix("abc"), "0xabc");
        assert_eq!(with_0x_prefix("0xabc"), "0xabc");
    }

    #[test]
    fn test_strip_0x_prefix() {
        assert_eq!(strip_0x_prefix("0xabc"), "abc");
        assert_eq!(strip_0x_prefix("abc"), "abc");
    }
}
