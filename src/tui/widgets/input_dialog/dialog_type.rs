//! Dialog type enum for different staking operations
//!
//! This module defines the [`DialogType`] enum which specifies the type
//! of staking operation being performed in the input dialog.

/// Dialog type for different staking operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    /// Delegate MON to a validator
    Delegate,
    /// Undelegate MON from a validator
    Undelegate,
    /// Withdraw pending amount
    Withdraw,
    /// Claim rewards
    Claim,
    /// Compound rewards
    Compound,
    /// Generic text input
    Generic,
    /// Query: Validator by ID
    QueryValidator,
    /// Query: Delegator (validator_id, address)
    QueryDelegator,
    /// Query: Withdrawal Request (validator_id, address, withdrawal_id)
    QueryWithdrawalRequest,
    /// Query: Delegations for address
    QueryDelegations,
    /// Query: Estimate Gas
    QueryEstimateGas,
    /// Query: Transaction by hash
    QueryTransaction,
}

impl DialogType {
    /// Get the title for this dialog type
    pub fn title(&self) -> &'static str {
        match self {
            DialogType::Delegate => " Delegate ",
            DialogType::Undelegate => " Undelegate ",
            DialogType::Withdraw => " Withdraw ",
            DialogType::Claim => " Claim Rewards ",
            DialogType::Compound => " Compound ",
            DialogType::Generic => " Input ",
            DialogType::QueryValidator => " Query Validator ",
            DialogType::QueryDelegator => " Query Delegator ",
            DialogType::QueryWithdrawalRequest => " Query Withdrawal ",
            DialogType::QueryDelegations => " Query Delegations ",
            DialogType::QueryEstimateGas => " Estimate Gas ",
            DialogType::QueryTransaction => " Query Transaction ",
        }
    }

    /// Get the placeholder text for this dialog type
    pub fn placeholder(&self) -> &'static str {
        match self {
            DialogType::Delegate => "Enter: VALIDATOR_ID AMOUNT (e.g., \"1 100.5\")",
            DialogType::Undelegate => {
                "Enter: VALIDATOR_ID AMOUNT WITHDRAWAL_ID (e.g., \"1 50.0 0\")"
            }
            DialogType::Withdraw => "Enter: VALIDATOR_ID WITHDRAWAL_ID (e.g., \"1 0\")",
            DialogType::Claim => "Enter validator ID (e.g., 1)",
            DialogType::Compound => "Enter validator ID (e.g., 1)",
            DialogType::Generic => "Enter value",
            DialogType::QueryValidator => "Enter validator ID (e.g., 1)",
            DialogType::QueryDelegator => "validator_id address (e.g., 1 0x1234...)",
            DialogType::QueryWithdrawalRequest => "validator_id address withdrawal_id",
            DialogType::QueryDelegations => "Enter address (e.g., 0x1234...)",
            DialogType::QueryEstimateGas => "from to data [value] (e.g., 0x1... 0x2... 0x...)",
            DialogType::QueryTransaction => "Enter transaction hash (e.g., 0x1234...)",
        }
    }

    /// Get the hint text for this dialog type
    pub fn hint(&self) -> &'static str {
        match self {
            DialogType::Delegate => "Validator ID and amount (both required)",
            DialogType::Undelegate => "Validator ID, amount, and withdrawal ID (all required)",
            DialogType::Withdraw => "Validator ID and withdrawal ID (both required)",
            DialogType::Claim => "Claim rewards for validator (validator ID required)",
            DialogType::Compound => "Compound rewards for validator (validator ID required)",
            DialogType::Generic => "",
            DialogType::QueryValidator => "Validator ID to query",
            DialogType::QueryDelegator => "Format: validator_id address",
            DialogType::QueryWithdrawalRequest => "Format: validator_id address withdrawal_id",
            DialogType::QueryDelegations => "0x-prefixed address",
            DialogType::QueryEstimateGas => "Format: from_address to_address calldata [value]",
            DialogType::QueryTransaction => "Transaction hash (0x-prefixed, 66 chars)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_type_titles() {
        assert_eq!(DialogType::Delegate.title(), " Delegate ");
        assert_eq!(DialogType::Undelegate.title(), " Undelegate ");
        assert_eq!(DialogType::Withdraw.title(), " Withdraw ");
        assert_eq!(DialogType::Claim.title(), " Claim Rewards ");
        assert_eq!(DialogType::Compound.title(), " Compound ");
        assert_eq!(DialogType::Generic.title(), " Input ");
        assert_eq!(DialogType::QueryValidator.title(), " Query Validator ");
        assert_eq!(DialogType::QueryDelegator.title(), " Query Delegator ");
        assert_eq!(
            DialogType::QueryWithdrawalRequest.title(),
            " Query Withdrawal "
        );
        assert_eq!(DialogType::QueryDelegations.title(), " Query Delegations ");
        assert_eq!(DialogType::QueryEstimateGas.title(), " Estimate Gas ");
        assert_eq!(DialogType::QueryTransaction.title(), " Query Transaction ");
    }

    #[test]
    fn test_dialog_type_placeholders() {
        assert_eq!(
            DialogType::Delegate.placeholder(),
            "Enter: VALIDATOR_ID AMOUNT (e.g., \"1 100.5\")"
        );
        assert_eq!(
            DialogType::Undelegate.placeholder(),
            "Enter: VALIDATOR_ID AMOUNT WITHDRAWAL_ID (e.g., \"1 50.0 0\")"
        );
        assert_eq!(
            DialogType::QueryValidator.placeholder(),
            "Enter validator ID (e.g., 1)"
        );
        assert_eq!(
            DialogType::QueryEstimateGas.placeholder(),
            "from to data [value] (e.g., 0x1... 0x2... 0x...)"
        );
        assert_eq!(
            DialogType::QueryTransaction.placeholder(),
            "Enter transaction hash (e.g., 0x1234...)"
        );
    }

    #[test]
    fn test_dialog_type_hints() {
        assert_eq!(
            DialogType::Delegate.hint(),
            "Validator ID and amount (both required)"
        );
        assert_eq!(
            DialogType::Undelegate.hint(),
            "Validator ID, amount, and withdrawal ID (all required)"
        );
        assert_eq!(
            DialogType::Withdraw.hint(),
            "Validator ID and withdrawal ID (both required)"
        );
        assert_eq!(
            DialogType::Claim.hint(),
            "Claim rewards for validator (validator ID required)"
        );
        assert_eq!(
            DialogType::Compound.hint(),
            "Compound rewards for validator (validator ID required)"
        );
        assert_eq!(DialogType::Generic.hint(), "");
        assert_eq!(DialogType::QueryValidator.hint(), "Validator ID to query");
        assert_eq!(
            DialogType::QueryDelegator.hint(),
            "Format: validator_id address"
        );
        assert_eq!(
            DialogType::QueryEstimateGas.hint(),
            "Format: from_address to_address calldata [value]"
        );
        assert_eq!(
            DialogType::QueryTransaction.hint(),
            "Transaction hash (0x-prefixed, 66 chars)"
        );
    }

    #[test]
    fn test_dialog_type_equality() {
        assert_eq!(DialogType::Delegate, DialogType::Delegate);
        assert_ne!(DialogType::Delegate, DialogType::Undelegate);
        assert_eq!(DialogType::QueryValidator, DialogType::QueryValidator);
    }

    #[test]
    fn test_dialog_type_copy() {
        let dialog_type = DialogType::Withdraw;
        let copied = dialog_type;
        assert_eq!(dialog_type, copied);
    }
}
