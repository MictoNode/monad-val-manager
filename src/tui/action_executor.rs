//! Action Executor - Executes staking actions from TUI
//!
//! This module provides functions to convert dialog input into staking
//! operations and execute them. It bridges the gap between UI state
//! and blockchain operations.

use crate::rpc::RpcClient;
use crate::staking::operations::{
    claim_rewards, compound, delegate, undelegate, withdraw, StakingResult,
};
use crate::staking::signer::Signer;
use crate::tui::staking::{PendingStakingAction, StakingActionResult, StakingActionType};
use crate::tui::widgets::{DialogType, InputDialogState};

/// Convert dialog type to staking action type
pub fn dialog_type_to_action_type(dialog_type: DialogType) -> Option<StakingActionType> {
    match dialog_type {
        DialogType::Delegate => Some(StakingActionType::Delegate),
        DialogType::Undelegate => Some(StakingActionType::Undelegate),
        DialogType::Withdraw => Some(StakingActionType::Withdraw),
        DialogType::Claim => Some(StakingActionType::ClaimRewards),
        DialogType::Compound => Some(StakingActionType::Compound),
        DialogType::Generic => None,
        // Query dialogs don't map to staking actions
        DialogType::QueryValidator => None,
        DialogType::QueryDelegator => None,
        DialogType::QueryWithdrawalRequest => None,
        DialogType::QueryDelegations => None,
        DialogType::QueryEstimateGas => None,
        DialogType::QueryTransaction => None,
    }
}

/// Parse input dialog value to amount in wei (smallest unit)
///
/// Takes a human-readable amount like "1.5" and converts to wei (10^18)
pub fn parse_amount_to_wei(input: &str) -> Result<u128, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Amount is required".to_string());
    }

    // Parse as float
    let amount: f64 = trimmed
        .parse()
        .map_err(|_| "Invalid number format".to_string())?;

    if amount <= 0.0 {
        return Err("Amount must be positive".to_string());
    }

    // Convert to wei (18 decimals)
    // Maximum precision to avoid overflow
    let wei = (amount * 1e18) as u128;
    Ok(wei)
}

/// Build a pending action from dialog input and selected validator
///
/// All operations require explicit input (no default to selected validator).
/// Format requirements:
/// - Delegate: "VALIDATOR_ID AMOUNT"
/// - Undelegate: "VALIDATOR_ID AMOUNT WITHDRAWAL_ID"
/// - Withdraw: "VALIDATOR_ID WITHDRAWAL_ID"
/// - Claim: "VALIDATOR_ID"
/// - Compound: "VALIDATOR_ID"
pub fn build_pending_action(
    dialog_state: &InputDialogState,
    _validator_id: u64,
    _withdrawal_index: Option<u8>,
) -> Result<PendingStakingAction, String> {
    let action_type = dialog_type_to_action_type(dialog_state.dialog_type)
        .ok_or_else(|| "Unknown dialog type".to_string())?;

    match action_type {
        StakingActionType::Delegate => {
            // Format: "VALIDATOR_ID AMOUNT" (both required)
            let input = dialog_state.get_input_str().trim();
            let parts: Vec<&str> = input.split_whitespace().collect();

            if parts.len() != 2 {
                return Err(
                    "Invalid format. Use: VALIDATOR_ID AMOUNT (e.g., \"1 100.5\")".to_string(),
                );
            }

            let vid = parts[0]
                .parse::<u64>()
                .map_err(|_| "Invalid validator ID".to_string())?;
            let amount = parse_amount_to_wei(parts[1])?;
            Ok(PendingStakingAction::delegate(vid, amount))
        }
        StakingActionType::Undelegate => {
            // Format: "VALIDATOR_ID AMOUNT WITHDRAWAL_ID" (all required)
            let input = dialog_state.get_input_str().trim();
            let parts: Vec<&str> = input.split_whitespace().collect();

            if parts.len() != 3 {
                return Err(
                    "Invalid format. Use: VALIDATOR_ID AMOUNT WITHDRAWAL_ID (e.g., \"1 50.0 0\")"
                        .to_string(),
                );
            }

            let vid = parts[0]
                .parse::<u64>()
                .map_err(|_| "Invalid validator ID".to_string())?;
            let amount = parse_amount_to_wei(parts[1])?;
            let w_index = parts[2]
                .parse::<u8>()
                .map_err(|_| "Invalid withdrawal ID (must be 0-255)".to_string())?;
            Ok(PendingStakingAction::undelegate(vid, amount, w_index))
        }
        StakingActionType::Withdraw => {
            // Format: "VALIDATOR_ID WITHDRAWAL_ID" (both required)
            let input = dialog_state.get_input_str().trim();
            let parts: Vec<&str> = input.split_whitespace().collect();

            if parts.len() != 2 {
                return Err(
                    "Invalid format. Use: VALIDATOR_ID WITHDRAWAL_ID (e.g., \"1 0\")".to_string(),
                );
            }

            let vid = parts[0]
                .parse::<u64>()
                .map_err(|_| "Invalid validator ID".to_string())?;
            let w_index = parts[1]
                .parse::<u8>()
                .map_err(|_| "Invalid withdrawal ID (must be 0-255)".to_string())?;
            Ok(PendingStakingAction::withdraw(vid, w_index))
        }
        StakingActionType::ClaimRewards => {
            // Format: "VALIDATOR_ID" (required)
            let input = dialog_state.get_input_str().trim();
            if input.is_empty() {
                return Err("Validator ID is required".to_string());
            }
            let vid = input
                .parse::<u64>()
                .map_err(|_| "Invalid validator ID format".to_string())?;
            Ok(PendingStakingAction::claim_rewards(vid))
        }
        StakingActionType::Compound => {
            // Format: "VALIDATOR_ID" (required)
            let input = dialog_state.get_input_str().trim();
            if input.is_empty() {
                return Err("Validator ID is required".to_string());
            }
            let vid = input
                .parse::<u64>()
                .map_err(|_| "Invalid validator ID format".to_string())?;
            Ok(PendingStakingAction::compound(vid))
        }
    }
}

/// Execute a staking action
///
/// This is an async function that calls the appropriate staking operation
/// based on the action type.
pub async fn execute_staking_action(
    client: &RpcClient,
    signer: &dyn Signer,
    action: &PendingStakingAction,
) -> StakingActionResult {
    let result = match action.action_type {
        StakingActionType::Delegate => {
            let amount = action.amount.unwrap_or(0);
            delegate(client, signer, action.validator_id, amount).await
        }
        StakingActionType::Undelegate => {
            let amount = action.amount.unwrap_or(0);
            let w_index = action.withdrawal_index.unwrap_or(0);
            undelegate(client, signer, action.validator_id, amount, w_index).await
        }
        StakingActionType::Withdraw => {
            let w_index = action.withdrawal_index.unwrap_or(0);
            withdraw(client, signer, action.validator_id, w_index).await
        }
        StakingActionType::ClaimRewards => claim_rewards(client, signer, action.validator_id).await,
        StakingActionType::Compound => compound(client, signer, action.validator_id).await,
    };

    match result {
        Ok(StakingResult { tx_hash, .. }) => StakingActionResult::success(tx_hash),
        Err(e) => StakingActionResult::failure(e.to_string()),
    }
}

/// Validate dialog input for the given action type
///
/// For Delegate, accepts either:
/// - "AMOUNT" (uses selected validator from delegation list)
/// - "VALIDATOR_ID AMOUNT" (specifies validator directly)
pub fn validate_dialog_input(dialog_state: &InputDialogState) -> Result<(), String> {
    let action_type = dialog_type_to_action_type(dialog_state.dialog_type);

    match action_type {
        Some(StakingActionType::Delegate) => {
            // Format: "VALIDATOR_ID AMOUNT" (both required)
            let input = dialog_state.get_input_str().trim();
            if input.is_empty() {
                return Err("Validator ID and amount are required".to_string());
            }

            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() != 2 {
                return Err(
                    "Invalid format. Use: VALIDATOR_ID AMOUNT (e.g., \"1 100.5\")".to_string(),
                );
            }

            // Validate validator ID
            parts[0]
                .parse::<u64>()
                .map_err(|_| "Invalid validator ID".to_string())?;
            // Validate amount
            parse_amount_to_wei(parts[1])?;
            Ok(())
        }
        Some(StakingActionType::Undelegate) => {
            // Format: "VALIDATOR_ID AMOUNT WITHDRAWAL_ID" (all required)
            let input = dialog_state.get_input_str().trim();
            if input.is_empty() {
                return Err("Validator ID, amount, and withdrawal ID are required".to_string());
            }

            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() != 3 {
                return Err(
                    "Invalid format. Use: VALIDATOR_ID AMOUNT WITHDRAWAL_ID (e.g., \"1 50.0 0\")"
                        .to_string(),
                );
            }

            // Validate validator ID
            parts[0]
                .parse::<u64>()
                .map_err(|_| "Invalid validator ID".to_string())?;
            // Validate amount
            parse_amount_to_wei(parts[1])?;
            // Validate withdrawal ID
            parts[2]
                .parse::<u8>()
                .map_err(|_| "Invalid withdrawal ID (must be 0-255)".to_string())?;
            Ok(())
        }
        Some(StakingActionType::Withdraw) => {
            // Format: "VALIDATOR_ID WITHDRAWAL_ID" (both required)
            let input = dialog_state.get_input_str().trim();
            if input.is_empty() {
                return Err("Validator ID and withdrawal ID are required".to_string());
            }

            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() != 2 {
                return Err(
                    "Invalid format. Use: VALIDATOR_ID WITHDRAWAL_ID (e.g., \"1 0\")".to_string(),
                );
            }

            // Validate validator ID
            parts[0]
                .parse::<u64>()
                .map_err(|_| "Invalid validator ID".to_string())?;
            // Validate withdrawal ID
            parts[1]
                .parse::<u8>()
                .map_err(|_| "Invalid withdrawal ID (must be 0-255)".to_string())?;
            Ok(())
        }
        Some(StakingActionType::ClaimRewards) => {
            // Format: "VALIDATOR_ID" (required)
            let input = dialog_state.get_input_str().trim();
            if input.is_empty() {
                return Err("Validator ID is required".to_string());
            }
            input
                .parse::<u64>()
                .map_err(|_| "Invalid validator ID format".to_string())?;
            Ok(())
        }
        Some(StakingActionType::Compound) => {
            // Format: "VALIDATOR_ID" (required)
            let input = dialog_state.get_input_str().trim();
            if input.is_empty() {
                return Err("Validator ID is required".to_string());
            }
            input
                .parse::<u64>()
                .map_err(|_| "Invalid validator ID format".to_string())?;
            Ok(())
        }
        None => Err("Unknown action type".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_textarea::TextArea;

    #[test]
    fn test_dialog_type_to_action_type() {
        assert_eq!(
            dialog_type_to_action_type(DialogType::Delegate),
            Some(StakingActionType::Delegate)
        );
        assert_eq!(
            dialog_type_to_action_type(DialogType::Undelegate),
            Some(StakingActionType::Undelegate)
        );
        assert_eq!(
            dialog_type_to_action_type(DialogType::Withdraw),
            Some(StakingActionType::Withdraw)
        );
        assert_eq!(
            dialog_type_to_action_type(DialogType::Claim),
            Some(StakingActionType::ClaimRewards)
        );
        assert_eq!(
            dialog_type_to_action_type(DialogType::Compound),
            Some(StakingActionType::Compound)
        );
        assert_eq!(dialog_type_to_action_type(DialogType::Generic), None);
    }

    #[test]
    fn test_parse_amount_to_wei_whole() {
        let result = parse_amount_to_wei("1").unwrap();
        assert_eq!(result, 1_000_000_000_000_000_000);
    }

    #[test]
    fn test_parse_amount_to_wei_decimal() {
        let result = parse_amount_to_wei("1.5").unwrap();
        assert_eq!(result, 1_500_000_000_000_000_000);
    }

    #[test]
    fn test_parse_amount_to_wei_small() {
        let result = parse_amount_to_wei("0.001").unwrap();
        assert_eq!(result, 1_000_000_000_000_000);
    }

    #[test]
    fn test_parse_amount_to_wei_empty() {
        let result = parse_amount_to_wei("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount is required");
    }

    #[test]
    fn test_parse_amount_to_wei_invalid() {
        let result = parse_amount_to_wei("abc");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid number format");
    }

    #[test]
    fn test_parse_amount_to_wei_negative() {
        let result = parse_amount_to_wei("-1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount must be positive");
    }

    #[test]
    fn test_parse_amount_to_wei_zero() {
        let result = parse_amount_to_wei("0");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount must be positive");
    }

    #[test]
    fn test_parse_amount_to_wei_whitespace() {
        let result = parse_amount_to_wei("  1.5  ").unwrap();
        assert_eq!(result, 1_500_000_000_000_000_000);
    }

    #[test]
    fn test_build_pending_action_delegate() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Delegate);
        dialog.input = TextArea::from(["1 100".to_string()]);

        let action = build_pending_action(&dialog, 42, None).unwrap();
        assert_eq!(action.action_type, StakingActionType::Delegate);
        assert_eq!(action.validator_id, 1);
        assert_eq!(action.amount, Some(100_000_000_000_000_000_000));
    }

    #[test]
    fn test_build_pending_action_undelegate() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Undelegate);
        dialog.input = TextArea::from(["1 50.0 0".to_string()]);

        let action = build_pending_action(&dialog, 1, Some(3)).unwrap();
        assert_eq!(action.action_type, StakingActionType::Undelegate);
        assert_eq!(action.validator_id, 1);
        assert_eq!(action.amount, Some(50_000_000_000_000_000_000));
        assert_eq!(action.withdrawal_index, Some(0));
    }

    #[test]
    fn test_build_pending_action_withdraw() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Withdraw);
        dialog.input = TextArea::from(["1 0".to_string()]);

        let action = build_pending_action(&dialog, 5, Some(2)).unwrap();
        assert_eq!(action.action_type, StakingActionType::Withdraw);
        assert_eq!(action.validator_id, 1);
        assert_eq!(action.withdrawal_index, Some(0));
    }

    #[test]
    fn test_build_pending_action_claim() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Claim);
        dialog.input = TextArea::from(["10".to_string()]);

        let action = build_pending_action(&dialog, 1, None).unwrap();
        assert_eq!(action.action_type, StakingActionType::ClaimRewards);
        assert_eq!(action.validator_id, 10);
    }

    #[test]
    fn test_build_pending_action_compound() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Compound);
        dialog.input = TextArea::from(["7".to_string()]);

        let action = build_pending_action(&dialog, 1, None).unwrap();
        assert_eq!(action.action_type, StakingActionType::Compound);
        assert_eq!(action.validator_id, 7);
    }

    #[test]
    fn test_validate_dialog_input_delegate_valid() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Delegate);
        dialog.input = TextArea::from(["1 100".to_string()]);

        assert!(validate_dialog_input(&dialog).is_ok());
    }

    #[test]
    fn test_validate_dialog_input_delegate_invalid() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Delegate);
        dialog.input = TextArea::from(["abc".to_string()]);

        assert!(validate_dialog_input(&dialog).is_err());
    }

    #[test]
    fn test_validate_dialog_input_claim_empty_input() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Claim);
        dialog.input = TextArea::from(["".to_string()]);

        assert!(validate_dialog_input(&dialog).is_err()); // Now requires explicit validator ID
    }

    #[test]
    fn test_validate_dialog_input_claim_with_validator_id() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Claim);
        dialog.input = TextArea::from(["42".to_string()]);

        assert!(validate_dialog_input(&dialog).is_ok());
    }

    #[test]
    fn test_validate_dialog_input_claim_invalid_validator_id() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Claim);
        dialog.input = TextArea::from(["abc".to_string()]);

        assert!(validate_dialog_input(&dialog).is_err());
    }

    #[test]
    fn test_validate_dialog_input_compound_empty_input() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Compound);
        dialog.input = TextArea::from(["".to_string()]);

        assert!(validate_dialog_input(&dialog).is_err()); // Now requires explicit validator ID
    }

    #[test]
    fn test_validate_dialog_input_compound_with_validator_id() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Compound);
        dialog.input = TextArea::from(["123".to_string()]);

        assert!(validate_dialog_input(&dialog).is_ok());
    }

    #[test]
    fn test_validate_dialog_input_delegate_with_validator_id() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Delegate);
        dialog.input = TextArea::from(["1 100.5".to_string()]);

        assert!(validate_dialog_input(&dialog).is_ok());
    }

    #[test]
    fn test_validate_dialog_input_delegate_invalid_validator_id() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Delegate);
        dialog.input = TextArea::from(["abc 100".to_string()]);

        assert!(validate_dialog_input(&dialog).is_err());
    }

    #[test]
    fn test_validate_dialog_input_delegate_invalid_format() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Delegate);
        dialog.input = TextArea::from(["1 100 50".to_string()]); // Too many parts

        assert!(validate_dialog_input(&dialog).is_err());
    }

    #[test]
    fn test_build_pending_action_delegate_with_validator_id() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Delegate);
        dialog.input = TextArea::from(["5 200".to_string()]);

        let action = build_pending_action(&dialog, 1, None).unwrap();
        assert_eq!(action.action_type, StakingActionType::Delegate);
        assert_eq!(action.validator_id, 5); // Uses parsed validator_id
        assert_eq!(action.amount, Some(200_000_000_000_000_000_000));
    }

    #[test]
    fn test_validate_dialog_input_undelegate_with_withdrawal_id() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Undelegate);

        // "VALIDATOR_ID AMOUNT WITHDRAWAL_ID" format should be valid
        dialog.input = TextArea::from(["1 100 0".to_string()]);
        assert!(validate_dialog_input(&dialog).is_ok());
    }

    #[test]
    fn test_validate_dialog_input_undelegate_invalid_withdrawal_id() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Undelegate);

        // Invalid withdrawal ID should fail
        dialog.input = TextArea::from(["1 100 abc".to_string()]);
        assert!(validate_dialog_input(&dialog).is_err());
    }

    #[test]
    fn test_validate_dialog_input_undelegate_invalid_format() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Undelegate);

        // Too many parts should fail
        dialog.input = TextArea::from(["1 100 5 3".to_string()]);
        assert!(validate_dialog_input(&dialog).is_err());
    }

    #[test]
    fn test_build_pending_action_undelegate_with_withdrawal_id() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Undelegate);
        dialog.input = TextArea::from(["1 50 7".to_string()]);

        let action = build_pending_action(&dialog, 1, None).unwrap();
        assert_eq!(action.action_type, StakingActionType::Undelegate);
        assert_eq!(action.validator_id, 1);
        assert_eq!(action.amount, Some(50_000_000_000_000_000_000));
        assert_eq!(action.withdrawal_index, Some(7));
    }

    #[test]
    fn test_build_pending_action_undelegate_default_withdrawal_id() {
        let mut dialog = InputDialogState::new();
        dialog.open(DialogType::Undelegate);
        dialog.input = TextArea::from(["1 50 0".to_string()]); // Now requires all 3 parts

        let action = build_pending_action(&dialog, 1, None).unwrap();
        assert_eq!(action.action_type, StakingActionType::Undelegate);
        assert_eq!(action.validator_id, 1);
        assert_eq!(action.amount, Some(50_000_000_000_000_000_000));
        assert_eq!(action.withdrawal_index, Some(0));
    }
}
