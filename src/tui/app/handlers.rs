//! User input event handlers for TUI application

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::crossterm::event::{
    KeyCode as RatKeyCode, KeyEvent as RatKeyEvent, KeyEventKind as RatKeyEventKind,
    KeyEventState as RatKeyEventState, KeyModifiers as RatKeyModifiers,
};

use super::TuiApp;

/// Character validation helpers for TextArea input filtering
mod char_validation {
    /// Check if a character is valid for a numeric field (validator_id, withdrawal_id)
    pub fn is_valid_numeric_char(c: char) -> bool {
        c.is_ascii_digit()
    }

    /// Check if a character is valid for a decimal amount field
    pub fn is_valid_amount_char(c: char) -> bool {
        c.is_ascii_digit() || c == '.'
    }

    /// Check if a character is valid for a hex field (private keys, addresses)
    /// Allows 0-9, a-f, A-F, and 'x'/'X' for 0x prefix
    pub fn is_valid_hex_char(c: char) -> bool {
        c.is_ascii_hexdigit() || c == 'x' || c == 'X'
    }

    /// Check if a character is valid for the current field in Delegate dialog
    pub fn is_valid_delegate_char(c: char, focused_field: u8) -> bool {
        match focused_field {
            0 => is_valid_numeric_char(c), // Validator ID
            1 => is_valid_amount_char(c),  // Amount
            _ => true,
        }
    }

    /// Check if a character is valid for the current field in Undelegate dialog
    pub fn is_valid_undelegate_char(c: char, focused_field: u8) -> bool {
        match focused_field {
            0 => is_valid_numeric_char(c), // Validator ID
            1 => is_valid_amount_char(c),  // Amount
            _ => true,
        }
    }

    /// Check if a character is valid for the current field in Withdraw dialog
    pub fn is_valid_withdraw_char(c: char, focused_field: u8) -> bool {
        match focused_field {
            0 => is_valid_numeric_char(c), // Validator ID
            1 => is_valid_numeric_char(c), // Withdrawal ID
            _ => true,
        }
    }

    /// Check if a character is valid for the current field in Add Validator dialog
    pub fn is_valid_add_validator_char(c: char, focused_field: u8) -> bool {
        match focused_field {
            0 => is_valid_hex_char(c),    // SECP Privkey
            1 => is_valid_hex_char(c),    // BLS Privkey
            2 => is_valid_hex_char(c),    // Auth Address
            3 => is_valid_amount_char(c), // Amount
            _ => true,
        }
    }

    /// Check if a character is valid for the current field in Change Commission dialog
    pub fn is_valid_change_commission_char(c: char, focused_field: u8) -> bool {
        match focused_field {
            0 => is_valid_numeric_char(c), // Validator ID
            1 => is_valid_amount_char(c),  // Commission (allows decimal)
            _ => true,
        }
    }

    /// Check if a character is valid for the current field in Query Delegator dialog
    pub fn is_valid_query_delegator_char(c: char, focused_field: u8) -> bool {
        match focused_field {
            0 => is_valid_numeric_char(c), // Validator ID
            1 => is_valid_hex_char(c),     // Delegator Address
            _ => true,
        }
    }

    /// Check if a character is valid for Transfer address field
    pub fn is_valid_transfer_address_char(c: char) -> bool {
        is_valid_hex_char(c)
    }

    /// Check if a character is valid for Transfer amount field
    pub fn is_valid_transfer_amount_char(c: char) -> bool {
        is_valid_amount_char(c)
    }
}

impl TuiApp {
    /// Handle keyboard input based on current state
    pub(crate) async fn handle_key_event(&mut self, mut key: KeyEvent) {
        // NORMALIZE BACKSPACE: Handle different terminal backspace codes
        // - xterm with stty erase=^H sends \x08 (Ctrl+H) as Char('\x08')
        // - Some terminals send \x7f (DEL) as Char('\x7f')
        // - Normal backspace sends KeyCode::Backspace
        let is_backspace_variant = matches!(
            key.code,
            KeyCode::Char('\x08') | KeyCode::Char('\x7f') | KeyCode::Backspace
        );
        if is_backspace_variant {
            key.code = KeyCode::Backspace;
            key.modifiers = KeyModifiers::empty();
        }

        // CRITICAL: On Windows, crossterm sends Press AND Release events for every key.
        // We only process Press events to avoid double-firing actions (e.g., backspace deleting twice).
        if key.kind != KeyEventKind::Press {
            return;
        }
        use crate::tui::handler::{
            handle_confirm_popup_key_event, handle_doctor_key_event, handle_staking_key_event,
            handle_transfer_key_event,
        };

        // CRITICAL: Check if ANY dialog is active FIRST, before any other key processing
        // When a dialog is open, ALL keys go ONLY to that dialog - NO screen shortcuts work
        let any_dialog_active = self.state.input_dialog.is_active()
            || self.state.add_validator.is_active()
            || self.state.change_commission.is_active()
            || self.state.query_delegator.is_active()
            || self.state.query_validator.is_active()
            || self.state.delegate.is_active()
            || self.state.undelegate.is_active()
            || self.state.withdraw.is_active()
            || self.state.transfer.is_active
            || self.state.confirm_popup.is_active();

        // When ANY dialog is active, route ONLY to that specific dialog handler
        // This prevents ALL screen-specific shortcuts from working (q, r, h, 1-5, o/d/u/w/c/m/a/x, etc.)
        if any_dialog_active {
            // Route to the specific dialog that is active
            // Only ONE dialog should be active at a time
            if self.state.confirm_popup.is_active() {
                if let Some(confirm_action) = handle_confirm_popup_key_event(key) {
                    self.handle_confirm_action(confirm_action).await;
                }
                return; // STOP: Do not process any other handlers
            }
            if self.state.add_validator.is_active() {
                self.handle_add_validator_key_event(key).await;
                return; // STOP: Do not process any other handlers
            }
            if self.state.query_delegator.is_active() {
                self.handle_query_delegator_key_event(key).await;
                return; // STOP: Do not process any other handlers
            }
            if self.state.query_validator.is_active() {
                self.handle_query_validator_key_event(key).await;
                return; // STOP: Do not process any other handlers
            }
            if self.state.change_commission.is_active() {
                self.handle_change_commission_key_event(key).await;
                return; // STOP: Do not process any other handlers
            }
            if self.state.delegate.is_active() {
                self.handle_delegate_key_event(key).await;
                return; // STOP: Do not process any other handlers
            }
            if self.state.undelegate.is_active() {
                self.handle_undelegate_key_event(key).await;
                return; // STOP: Do not process any other handlers
            }
            if self.state.withdraw.is_active() {
                self.handle_withdraw_key_event(key).await;
                return; // STOP: Do not process any other handlers
            }
            if self.state.input_dialog.is_active() {
                use crossterm::event::KeyCode;
                // Special handling for Enter/Esc
                match key.code {
                    KeyCode::Enter => {
                        self.handle_dialog_action(crate::tui::handler::DialogAction::Confirm)
                            .await;
                    }
                    KeyCode::Esc => {
                        self.handle_dialog_action(crate::tui::handler::DialogAction::Cancel)
                            .await;
                    }
                    // Filter invalid characters, then pass valid input to textarea
                    _ => {
                        // Claim and Compound dialogs use input_dialog for validator_id
                        // Only allow numeric characters (0-9) for validator_id
                        if let KeyCode::Char(c) = key.code {
                            if !char_validation::is_valid_numeric_char(c) {
                                return; // Reject non-numeric character
                            }
                        }
                        // Create ratatui crossterm event directly for textarea
                        let rat_key = RatKeyEvent {
                            code: match key.code {
                                KeyCode::Backspace => RatKeyCode::Backspace,
                                KeyCode::Char(c) => RatKeyCode::Char(c),
                                KeyCode::Left => RatKeyCode::Left,
                                KeyCode::Right => RatKeyCode::Right,
                                KeyCode::Home => RatKeyCode::Home,
                                KeyCode::End => RatKeyCode::End,
                                KeyCode::Delete => RatKeyCode::Delete,
                                _ => return,
                            },
                            modifiers: RatKeyModifiers::from_bits_truncate(key.modifiers.bits()),
                            kind: RatKeyEventKind::Press,
                            state: RatKeyEventState::empty(),
                        };
                        self.state.input_dialog.input.input(rat_key);
                    }
                }
                return; // STOP: Do not process any other handlers
            }
            if self.state.transfer.is_active {
                self.handle_transfer_dialog_key_event(key).await;
                return; // STOP: Do not process any other handlers
            }
            // If we reach here, something is wrong - a dialog is marked active but none of the
            // is_active() checks returned true. This should never happen, but if it does,
            // return early to prevent keys from leaking to screen handlers.
            return;
        }

        // NO DIALOG IS ACTIVE - Process screen-specific and general keys below

        // On staking screen, check for staking-specific actions
        // Only reached when NO dialog is active
        if self.current_screen == crate::tui::screens::Screen::Staking {
            if let Some(staking_action) = handle_staking_key_event(key) {
                self.handle_staking_action(staking_action).await;
                return;
            }
        }

        // On doctor screen, check for doctor-specific actions
        if self.current_screen == crate::tui::screens::Screen::Doctor {
            if let Some(doctor_action) = handle_doctor_key_event(key) {
                self.handle_doctor_action(doctor_action).await;
                return;
            }
        }

        // On transfer screen, check for transfer-specific actions
        if self.current_screen == crate::tui::screens::Screen::Transfer {
            if let Some(transfer_action) = handle_transfer_key_event(key) {
                self.handle_transfer_action(transfer_action).await;
                return;
            }
        }

        // Fall back to general actions (q, r, h, 1-5, navigation, etc.)
        if let Some(action) = crate::tui::handler::handle_key_event(key) {
            self.handle_action(action).await;
        }
    }

    /// Handle dialog-specific actions
    pub(crate) async fn handle_dialog_action(&mut self, action: crate::tui::handler::DialogAction) {
        use crate::tui::handler::DialogAction;

        match action {
            DialogAction::Confirm => {
                // For staking dialogs, validate input and create pending action
                if let Err(e) =
                    crate::tui::action_executor::validate_dialog_input(&self.state.input_dialog)
                {
                    self.state.input_dialog.set_error(e);
                    return;
                }

                // All operations now require explicit input (CLI parity)
                // validator_id parameter is ignored - parsing happens in build_pending_action
                let validator_id = 1u64; // Placeholder, actual value comes from input

                // Get withdrawal index if applicable (for legacy Undelegate with just amount)
                let withdrawal_index = self.get_next_withdrawal_index(validator_id);

                // Build the pending action
                let pending_action = match crate::tui::action_executor::build_pending_action(
                    &self.state.input_dialog,
                    validator_id,
                    withdrawal_index,
                ) {
                    Ok(action) => action,
                    Err(e) => {
                        self.state.input_dialog.set_error(e);
                        return;
                    }
                };

                // Close input dialog and open confirmation popup
                self.state.input_dialog.close();
                self.state.confirm_popup.open(pending_action);
            }
            DialogAction::Cancel => {
                self.state.input_dialog.close();
            }
            DialogAction::NextField | DialogAction::PrevField => {
                // Not used by single-field dialogs (claim, compound)
            }
            // Cursor and editing actions are now handled directly by textarea.input()
            // in the main key handler, so we don't need to handle them here
            DialogAction::CursorLeft
            | DialogAction::CursorRight
            | DialogAction::CursorStart
            | DialogAction::CursorEnd
            | DialogAction::Backspace
            | DialogAction::Delete
            | DialogAction::SelectAll
            | DialogAction::DeleteWord
            | DialogAction::InputChar(_) => {
                // These are now handled by textarea.input() directly
            }
        }
    }

    /// Handle confirmation popup actions
    pub(crate) async fn handle_confirm_action(
        &mut self,
        action: crate::tui::handler::ConfirmAction,
    ) {
        use crate::tui::handler::ConfirmAction;

        match action {
            ConfirmAction::Confirm => {
                // Take the pending action from the popup and execute it
                if let Some(pending_action) = self.state.confirm_popup.take_action() {
                    // Execute the action NOW
                    self.execute_pending_staking_action(pending_action).await;
                }
            }
            ConfirmAction::Cancel => {
                // Close popup without executing
                self.state.confirm_popup.close();
            }
        }
    }

    /// Execute a pending staking action
    ///
    /// This function performs the actual blockchain transaction by:
    /// 1. Creating a signer from the config
    /// 2. Getting the RPC client
    /// 3. Calling the appropriate staking operation
    /// 4. Updating the UI state with the result
    async fn execute_pending_staking_action(
        &mut self,
        action: crate::tui::staking::PendingStakingAction,
    ) {
        use crate::staking::create_signer;
        use crate::tui::staking::StakingActionResult;

        // Mark as executing
        self.state.staking.start_execution(action.clone());

        // Get signer and RPC client
        let signer_result = create_signer(&self.config);
        let rpc_client = match &self.rpc_client {
            Some(client) => client,
            None => {
                let error_msg = "No RPC connection. Check your node is running.";
                self.state
                    .staking
                    .complete_execution(StakingActionResult::failure(error_msg.to_string()));
                // Show toast notification
                let id = self.toast_manager.next_id();
                self.toast_manager.add(crate::tui::toast::Toast::new(
                    id,
                    "Transaction Failed".to_string(),
                    error_msg.to_string(),
                    crate::tui::toast::ToastType::Error,
                ));
                return;
            }
        };

        let signer = match signer_result {
            Ok(s) => s,
            Err(e) => {
                let error_msg = format!("Failed to create signer: {}", e);
                self.state
                    .staking
                    .complete_execution(StakingActionResult::failure(error_msg.clone()));
                // Show toast notification
                let id = self.toast_manager.next_id();
                self.toast_manager.add(crate::tui::toast::Toast::new(
                    id,
                    "Signer Error".to_string(),
                    error_msg,
                    crate::tui::toast::ToastType::Error,
                ));
                return;
            }
        };

        // Execute the action using the action_executor
        let result = crate::tui::execute_staking_action(rpc_client, signer.as_ref(), &action).await;

        // Show toast based on result
        if result.success {
            if let Some(tx_hash) = &result.tx_hash {
                let id = self.toast_manager.next_id();
                self.toast_manager.add(crate::tui::toast::Toast::new(
                    id,
                    "Transaction Submitted".to_string(),
                    format!("TX: {}", tx_hash),
                    crate::tui::toast::ToastType::Success,
                ));
            }
        } else if let Some(error) = &result.error {
            let id = self.toast_manager.next_id();
            self.toast_manager.add(crate::tui::toast::Toast::new(
                id,
                "Transaction Failed".to_string(),
                error.clone(),
                crate::tui::toast::ToastType::Error,
            ));
        }

        // Update state with result
        self.state.staking.complete_execution(result);
    }

    /// Get the next available withdrawal index for a validator
    pub(crate) fn get_next_withdrawal_index(&self, validator_id: u64) -> Option<u8> {
        // Find the highest withdrawal index for this validator
        let max_index = self
            .state
            .staking
            .pending_withdrawals
            .iter()
            .filter(|w| w.validator_id == validator_id)
            .map(|w| w.withdrawal_index)
            .max()
            .unwrap_or(u8::MAX); // Start at MAX, will wrap to 0 on first call

        // Next index (wrapping)
        Some(max_index.wrapping_add(1))
    }

    /// Fetch validator commission and update ChangeCommissionState
    async fn fetch_validator_commission(&mut self, validator_id: u64) {
        use crate::staking::getters;

        let rpc_client = match &self.rpc_client {
            Some(client) => client,
            None => return,
        };

        match getters::get_validator(rpc_client, validator_id).await {
            Ok(validator) => {
                // Validator.commission() returns f64 percentage (e.g., 5.00 for 5%)
                self.state
                    .change_commission
                    .set_current_commission(validator.commission());
            }
            Err(_) => {
                // Silently fail - current commission will remain None
            }
        }
    }

    /// Handle staking screen-specific actions
    pub(crate) async fn handle_staking_action(
        &mut self,
        action: crate::tui::handler::StakingAction,
    ) {
        use crate::tui::handler::StakingAction;
        use crate::tui::widgets::DialogType;

        match action {
            StakingAction::OpenDelegate => {
                self.state.delegate.open();
                // Set available balance hint
                if let Ok(balance) = self.state.staking.format_balance().parse::<f64>() {
                    self.state.delegate.set_available_balance(balance);
                }
            }
            StakingAction::OpenUndelegate => {
                self.state.undelegate.open();
                // Set delegated amount hint if there's a selected delegation
                if let Some(delegation) = self.state.staking.selected_delegation() {
                    // Convert u128 (wei) to f64 (MON)
                    let delegated_mon = delegation.delegated_amount as f64 / 1e18;
                    self.state.undelegate.set_delegated_amount(delegated_mon);
                }
            }
            StakingAction::OpenWithdraw => {
                self.state.withdraw.open();
                // Calculate ready withdrawal IDs for context hint
                let ready_ids: Vec<u8> = self
                    .state
                    .staking
                    .pending_withdrawals
                    .iter()
                    .filter(|w| w.is_ready(self.state.staking.current_epoch))
                    .map(|w| w.withdrawal_index)
                    .collect();

                let hint = if ready_ids.is_empty() {
                    "No withdrawals ready".to_string()
                } else {
                    format!(
                        "Ready IDs: {}",
                        ready_ids
                            .iter()
                            .map(|id| id.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                self.state.withdraw.set_context_hint(hint);
            }
            StakingAction::OpenClaim => {
                self.state.input_dialog.open(DialogType::Claim);
                self.update_dialog_context_hint();
            }
            StakingAction::OpenCompound => {
                self.state.input_dialog.open(DialogType::Compound);
                self.update_dialog_context_hint();
            }
            StakingAction::OpenQueryDelegator => {
                self.state.query_delegator.open();
            }
            StakingAction::OpenQueryValidator => {
                self.state.query_validator.open();
            }
            StakingAction::OpenAddValidator => {
                self.state.add_validator.open();
            }
            StakingAction::OpenChangeCommission => {
                // Pre-fill with selected validator if available
                if let Some(validator_id) = self.state.staking.selected_validator_id() {
                    self.state
                        .change_commission
                        .open_with_validator(validator_id);
                    // Try to get commission from QueryValidatorState if available
                    if let (Some(query_vid), Some(query_comm)) = (
                        self.state.query_validator.last_validator_id,
                        self.state.query_validator.last_commission,
                    ) {
                        if query_vid == validator_id {
                            self.state
                                .change_commission
                                .set_current_commission(query_comm);
                        } else {
                            // Different validator, fetch fresh data
                            self.fetch_validator_commission(validator_id).await;
                        }
                    } else {
                        // No cached data, fetch from API
                        self.fetch_validator_commission(validator_id).await;
                    }
                } else {
                    self.state.change_commission.open();
                }
            }
            StakingAction::OpenTransfer => {
                // Get available balance from account
                let balance = self.state.staking.format_balance();
                self.state.transfer.open(Some(balance));
            }
            StakingAction::SelectPrev => {
                self.state.staking.select_prev();
            }
            StakingAction::SelectNext => {
                self.state.staking.select_next();
            }
            StakingAction::Refresh => {
                // On Staking screen, [r] only refreshes staking data (not all data)
                // This avoids slow network/consensus queries when user just wants to update staking info
                self.refresh_staking_data().await;
            }
            StakingAction::ConfirmAction => {
                // This is handled by dialog confirm
            }
            StakingAction::CancelAction => {
                self.state.input_dialog.close();
            }
        }
    }

    /// Handle doctor screen-specific actions
    pub(crate) async fn handle_doctor_action(&mut self, action: crate::tui::handler::DoctorAction) {
        use crate::tui::handler::DoctorAction;

        match action {
            DoctorAction::RunChecks => {
                self.run_doctor_checks().await;
            }
            DoctorAction::SelectPrev => {
                self.state.doctor.select_prev();
            }
            DoctorAction::SelectNext => {
                self.state.doctor.select_next();
            }
        }
    }

    /// Handle transfer screen-specific actions
    pub(crate) async fn handle_transfer_action(
        &mut self,
        action: crate::tui::handler::TransferAction,
    ) {
        use crate::tui::handler::TransferAction;

        match action {
            TransferAction::OpenDialog => {
                // Get available balance from account
                let balance = self.state.staking.format_balance();
                self.state.transfer.open(Some(balance));
            }
        }
    }

    /// BUG-013 FIX: Handle keyboard input when transfer dialog is active
    pub(crate) async fn handle_transfer_dialog_key_event(&mut self, key: KeyEvent) {
        use crate::tui::transfer_state::TransferStep;
        use crossterm::event::KeyCode;

        match (key.modifiers, key.code) {
            // Cancel/Exit
            (_, KeyCode::Esc) => {
                self.state.transfer.close();
            }
            // Confirm/Next
            (_, KeyCode::Enter) => {
                match self.state.transfer.step {
                    TransferStep::Address => {
                        // Validate and move to amount step
                        if let Err(e) = self.state.transfer.validate_current() {
                            self.state.transfer.error = Some(e);
                        } else {
                            self.state.transfer.next_step();
                        }
                    }
                    TransferStep::Amount => {
                        // Validate and move to confirm step
                        if let Err(e) = self.state.transfer.validate_current() {
                            self.state.transfer.error = Some(e);
                        } else {
                            self.state.transfer.next_step();
                        }
                    }
                    TransferStep::Confirm => {
                        // Start transfer (move to processing)
                        self.state.transfer.next_step();
                        // Execute actual transfer transaction
                        self.execute_transfer().await;
                    }
                    TransferStep::Complete => {
                        // Close dialog on Enter
                        self.state.transfer.close();
                    }
                    TransferStep::Processing => {
                        // Can't do anything while processing
                    }
                }
            }
            // All other input is handled by textarea (for input steps)
            _ => {
                // Only handle textarea input for Address and Amount steps
                if matches!(
                    self.state.transfer.step,
                    TransferStep::Address | TransferStep::Amount
                ) {
                    // Filter characters based on current step
                    match key.code {
                        KeyCode::Char(c) => {
                            let is_valid = match self.state.transfer.step {
                                TransferStep::Address => {
                                    char_validation::is_valid_transfer_address_char(c)
                                }
                                TransferStep::Amount => {
                                    char_validation::is_valid_transfer_amount_char(c)
                                }
                                _ => true,
                            };
                            if is_valid {
                                // Create ratatui crossterm event directly for textarea
                                let rat_key = RatKeyEvent {
                                    code: RatKeyCode::Char(c),
                                    modifiers: RatKeyModifiers::from_bits_truncate(
                                        key.modifiers.bits(),
                                    ),
                                    kind: RatKeyEventKind::Press,
                                    state: RatKeyEventState::empty(),
                                };
                                self.state.transfer.current_textarea_mut().input(rat_key);
                            }
                            // If invalid, drop the character (don't pass to textarea)
                        }
                        // All other keys (Backspace, Delete, cursor keys, etc.) pass through
                        _ => {
                            let rat_key = RatKeyEvent {
                                code: match key.code {
                                    KeyCode::Backspace => RatKeyCode::Backspace,
                                    KeyCode::Char(c) => RatKeyCode::Char(c),
                                    KeyCode::Left => RatKeyCode::Left,
                                    KeyCode::Right => RatKeyCode::Right,
                                    KeyCode::Home => RatKeyCode::Home,
                                    KeyCode::End => RatKeyCode::End,
                                    KeyCode::Delete => RatKeyCode::Delete,
                                    _ => return,
                                },
                                modifiers: RatKeyModifiers::from_bits_truncate(
                                    key.modifiers.bits(),
                                ),
                                kind: RatKeyEventKind::Press,
                                state: RatKeyEventState::empty(),
                            };
                            self.state.transfer.current_textarea_mut().input(rat_key);
                        }
                    }
                }
            }
        }
    }

    /// Execute the transfer transaction
    async fn execute_transfer(&mut self) {
        use crate::handlers::{parse_transfer_amount, TRANSFER_GAS_LIMIT};
        use crate::staking::create_signer;

        // Get transfer parameters from state (extract from TextArea)
        let to_address = self.state.transfer.formatted_address();
        let amount_str = self
            .state
            .transfer
            .amount
            .lines()
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");

        // Parse amount to wei
        let amount_wei = match parse_transfer_amount(amount_str) {
            Ok(wei) => wei,
            Err(e) => {
                self.state.transfer.error = Some(format!("Invalid amount: {}", e));
                self.state.transfer.step = crate::tui::transfer_state::TransferStep::Amount;
                return;
            }
        };

        // Get RPC client
        let rpc_client = match &self.rpc_client {
            Some(client) => client,
            None => {
                self.state.transfer.error = Some("No RPC connection".to_string());
                self.state.transfer.step = crate::tui::transfer_state::TransferStep::Confirm;
                return;
            }
        };

        // Create signer
        let signer = match create_signer(&self.config) {
            Ok(s) => s,
            Err(e) => {
                self.state.transfer.error = Some(format!("Failed to create signer: {}", e));
                self.state.transfer.step = crate::tui::transfer_state::TransferStep::Confirm;
                return;
            }
        };

        // Get sender address and nonce
        let from_address = signer.address();
        let nonce = match rpc_client.get_transaction_count(from_address).await {
            Ok(n) => n,
            Err(e) => {
                self.state.transfer.error = Some(format!("Failed to get nonce: {}", e));
                self.state.transfer.step = crate::tui::transfer_state::TransferStep::Confirm;
                return;
            }
        };

        // Get chain ID
        let chain_id = match rpc_client.get_chain_id().await {
            Ok(id) => id,
            Err(_) => self.state.validator.chain_id,
        };

        // Build transaction
        let tx = crate::staking::transaction::Eip1559Transaction::new(chain_id)
            .with_nonce(nonce)
            .with_gas(
                TRANSFER_GAS_LIMIT,
                crate::staking::transaction::DEFAULT_MAX_FEE,
                crate::staking::transaction::DEFAULT_MAX_PRIORITY_FEE,
            )
            .to(&to_address);

        let tx = match tx {
            Ok(t) => t,
            Err(e) => {
                self.state.transfer.error = Some(format!("Invalid recipient address: {}", e));
                self.state.transfer.step = crate::tui::transfer_state::TransferStep::Confirm;
                return;
            }
        };

        let tx = match tx.with_value(amount_wei).with_data_hex("0x") {
            Ok(t) => t,
            Err(e) => {
                self.state.transfer.error = Some(format!("Failed to build transaction: {}", e));
                self.state.transfer.step = crate::tui::transfer_state::TransferStep::Confirm;
                return;
            }
        };

        // Sign transaction
        let signed_hex = match signer.sign_transaction_hex(&tx) {
            Ok(s) => s,
            Err(e) => {
                self.state.transfer.error = Some(format!("Failed to sign transaction: {}", e));
                self.state.transfer.step = crate::tui::transfer_state::TransferStep::Confirm;
                return;
            }
        };

        // Broadcast transaction
        let tx_hash = match rpc_client.send_raw_transaction(&signed_hex).await {
            Ok(hash) => hash,
            Err(e) => {
                self.state.transfer.error = Some(format!("Failed to broadcast: {}", e));
                self.state.transfer.step = crate::tui::transfer_state::TransferStep::Confirm;
                return;
            }
        };

        // Success - set TX hash and move to Complete
        self.state.transfer.set_tx_hash(tx_hash);
        self.state.transfer.next_step(); // Processing -> Complete
    }

    /// Handle an action from user input
    pub(crate) async fn handle_action(&mut self, action: crate::tui::handler::Action) {
        use crate::tui::handler::Action;
        use crate::tui::screens::Screen;

        match action {
            Action::Quit => {
                self.should_quit = true;
            }
            Action::Refresh => {
                self.refresh_data().await;
            }
            Action::NextTab => {
                self.current_screen = self.current_screen.next();
            }
            Action::PrevTab => {
                self.current_screen = self.current_screen.prev();
            }
            Action::Help => {
                self.current_screen = Screen::Help;
            }
            Action::Back => {
                // If dialog is open, close it; otherwise go to dashboard
                if self.state.input_dialog.is_active() {
                    self.state.input_dialog.close();
                } else {
                    self.current_screen = Screen::Dashboard;
                }
            }
            Action::GotoDashboard => {
                self.current_screen = Screen::Dashboard;
            }
            Action::GotoStaking => {
                // Auto-refresh balance (fast) when entering Staking screen
                // Delegations are NOT loaded - user must manually add validators with [a] key
                self.refresh_staking_data().await;
                self.current_screen = Screen::Staking;
            }
            Action::GotoTransfer => {
                self.current_screen = Screen::Transfer;
            }
            Action::GotoDoctor => {
                // Refresh consensus data when entering Doctor screen
                self.refresh_consensus_data().await;
                self.current_screen = Screen::Doctor;
            }
            Action::GotoHelp => {
                self.current_screen = Screen::Help;
            }
            _ => {}
        }
    }

    /// Handle keyboard input for Add Validator dialog
    pub(crate) async fn handle_add_validator_key_event(&mut self, key: KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key.modifiers, key.code) {
            // Tab navigation between fields
            (_, KeyCode::Tab) => {
                self.state.add_validator.next_field();
            }
            // Shift+Tab for previous field
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                self.state.add_validator.prev_field();
            }
            // Confirm on Enter
            (_, KeyCode::Enter) => {
                self.handle_add_validator_confirm().await;
            }
            // Cancel on Escape
            (_, KeyCode::Esc) => {
                self.state.add_validator.close();
            }
            // Filter invalid characters, then pass valid input to textarea
            _ => {
                // Filter character input based on current field
                if let KeyCode::Char(c) = key.code {
                    let focused_field = self.state.add_validator.focused_field as u8;
                    if !char_validation::is_valid_add_validator_char(c, focused_field) {
                        return; // Reject invalid character
                    }
                }
                // Create ratatui crossterm event directly for textarea
                let rat_key = RatKeyEvent {
                    code: match key.code {
                        KeyCode::Backspace => RatKeyCode::Backspace,
                        KeyCode::Char(c) => RatKeyCode::Char(c),
                        KeyCode::Left => RatKeyCode::Left,
                        KeyCode::Right => RatKeyCode::Right,
                        KeyCode::Home => RatKeyCode::Home,
                        KeyCode::End => RatKeyCode::End,
                        KeyCode::Delete => RatKeyCode::Delete,
                        _ => return,
                    },
                    modifiers: RatKeyModifiers::from_bits_truncate(key.modifiers.bits()),
                    kind: RatKeyEventKind::Press,
                    state: RatKeyEventState::empty(),
                };
                self.state
                    .add_validator
                    .current_textarea_mut()
                    .input(rat_key);
            }
        }
    }

    /// Handle Add Validator dialog confirmation
    async fn handle_add_validator_confirm(&mut self) {
        use crate::staking::create_signer;
        use crate::staking::operations::add_validator_from_privkeys;

        // First Enter: validate and show confirmation
        if !self.state.add_validator.is_confirmed {
            match self.state.add_validator.validate() {
                Ok(params) => {
                    // Store validated params and mark as confirmed
                    self.state.add_validator.validated_params = Some(params.clone());
                    self.state.add_validator.is_confirmed = true;

                    // Show confirmation message
                    let description = params.description();
                    self.state.add_validator.set_status(
                        format!("CONFIRM: Press Enter again to execute\n{}", description),
                        None,
                    );
                }
                Err(e) => {
                    self.state.add_validator.set_error(e, None);
                }
            }
            return;
        }

        // Second Enter: execute the action
        if let Some(params) = self.state.add_validator.validated_params.take() {
            // Get signer and RPC client
            let signer_result = create_signer(&self.config);
            let rpc_client = match &self.rpc_client {
                Some(client) => client,
                None => {
                    let error_msg = "No RPC connection. Check your node is running.";
                    self.state.add_validator.set_error(error_msg, None);
                    self.state
                        .staking
                        .set_status_message(format!("Add Validator FAILED: {}", error_msg));
                    return;
                }
            };

            let signer = match signer_result {
                Ok(s) => s,
                Err(e) => {
                    let error_msg = format!("Failed to create signer: {}", e);
                    self.state.add_validator.set_error(&error_msg, None);
                    self.state
                        .staking
                        .set_status_message(format!("Add Validator FAILED: {}", error_msg));
                    return;
                }
            };

            // Parse SECP private key (hex string without 0x prefix)
            let secp_privkey_hex = &params.secp_privkey;
            let secp_privkey_bytes = match hex::decode(secp_privkey_hex) {
                Ok(bytes) => bytes,
                Err(e) => {
                    let error_msg = format!("Invalid SECP private key: {}", e);
                    self.state.add_validator.set_error(&error_msg, None);
                    self.state
                        .staking
                        .set_status_message(format!("Add Validator FAILED: {}", error_msg));
                    return;
                }
            };

            // Parse BLS private key (hex string with 0x prefix)
            let bls_privkey_hex = params
                .bls_privkey
                .strip_prefix("0x")
                .unwrap_or(&params.bls_privkey);
            let bls_privkey_bytes = match hex::decode(bls_privkey_hex) {
                Ok(bytes) => bytes,
                Err(e) => {
                    let error_msg = format!("Invalid BLS private key: {}", e);
                    self.state.add_validator.set_error(&error_msg, None);
                    self.state
                        .staking
                        .set_status_message(format!("Add Validator FAILED: {}", error_msg));
                    return;
                }
            };

            // Execute add_validator operation (commission fixed at 0, matching CLI behavior)
            let result = add_validator_from_privkeys(
                rpc_client,
                signer.as_ref(),
                &secp_privkey_bytes,
                &bls_privkey_bytes,
                &params.auth_address,
                (params.amount as u128) * 1_000_000_000_000_000_000, // Convert MON to wei
                0u64, // Commission fixed at 0 (matching CLI)
            )
            .await;

            // Show toast based on result
            match result {
                Ok(staking_result) => {
                    let tx_hash = staking_result.tx_hash;
                    let id = self.toast_manager.next_id();
                    self.toast_manager.add(crate::tui::toast::Toast::new(
                        id,
                        "Transaction Submitted".to_string(),
                        format!("Add Validator TX: {}", tx_hash),
                        crate::tui::toast::ToastType::Success,
                    ));
                    self.state
                        .staking
                        .set_status_message(format!("Add Validator: TX submitted {}", tx_hash));
                }
                Err(e) => {
                    let error_msg = format!("Add Validator failed: {}", e);
                    let id = self.toast_manager.next_id();
                    self.toast_manager.add(crate::tui::toast::Toast::new(
                        id,
                        "Transaction Failed".to_string(),
                        error_msg.clone(),
                        crate::tui::toast::ToastType::Error,
                    ));
                    self.state
                        .staking
                        .set_status_message(format!("Add Validator FAILED: {}", error_msg));
                }
            }

            self.state.add_validator.close();
        }
    }

    /// Handle keyboard input for Change Commission dialog
    pub(crate) async fn handle_change_commission_key_event(&mut self, key: KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key.modifiers, key.code) {
            // Tab navigation between fields
            (_, KeyCode::Tab) => {
                self.state.change_commission.next_field();
            }
            // Shift+Tab for previous field
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                self.state.change_commission.prev_field();
            }
            // Confirm on Enter
            (_, KeyCode::Enter) => {
                self.handle_change_commission_confirm().await;
            }
            // Cancel on Escape
            (_, KeyCode::Esc) => {
                self.state.change_commission.close();
            }
            // Filter invalid characters, then pass valid input to textarea
            _ => {
                // Filter character input based on current field
                if let KeyCode::Char(c) = key.code {
                    let focused_field = self.state.change_commission.focused_field as u8;
                    if !char_validation::is_valid_change_commission_char(c, focused_field) {
                        return; // Reject invalid character
                    }
                }
                // Create ratatui crossterm event directly for textarea
                let rat_key = RatKeyEvent {
                    code: match key.code {
                        KeyCode::Backspace => RatKeyCode::Backspace,
                        KeyCode::Char(c) => RatKeyCode::Char(c),
                        KeyCode::Left => RatKeyCode::Left,
                        KeyCode::Right => RatKeyCode::Right,
                        KeyCode::Home => RatKeyCode::Home,
                        KeyCode::End => RatKeyCode::End,
                        KeyCode::Delete => RatKeyCode::Delete,
                        _ => return,
                    },
                    modifiers: RatKeyModifiers::from_bits_truncate(key.modifiers.bits()),
                    kind: RatKeyEventKind::Press,
                    state: RatKeyEventState::empty(),
                };
                self.state
                    .change_commission
                    .current_textarea_mut()
                    .input(rat_key);
            }
        }
    }

    /// Handle Change Commission dialog confirmation
    async fn handle_change_commission_confirm(&mut self) {
        use crate::staking::create_signer;
        use crate::staking::operations::change_commission;

        // First Enter: validate and show confirmation
        if !self.state.change_commission.is_confirmed {
            match self.state.change_commission.validate() {
                Ok(params) => {
                    // Store validated params and mark as confirmed
                    self.state.change_commission.validated_params = Some(params.clone());
                    self.state.change_commission.is_confirmed = true;

                    // Show confirmation message
                    let description = params.description();
                    self.state.change_commission.set_status(
                        format!("CONFIRM: Press Enter again to execute\n{}", description),
                        None,
                    );
                }
                Err(e) => {
                    self.state.change_commission.set_error(e, None);
                }
            }
            return;
        }

        // Second Enter: execute the action
        if let Some(params) = self.state.change_commission.validated_params.take() {
            // Get signer and RPC client
            let signer_result = create_signer(&self.config);
            let rpc_client = match &self.rpc_client {
                Some(client) => client,
                None => {
                    let error_msg = "No RPC connection. Check your node is running.";
                    self.state.change_commission.set_error(error_msg, None);
                    self.state
                        .staking
                        .set_status_message(format!("Change Commission FAILED: {}", error_msg));
                    return;
                }
            };

            let signer = match signer_result {
                Ok(s) => s,
                Err(e) => {
                    let error_msg = format!("Failed to create signer: {}", e);
                    self.state.change_commission.set_error(&error_msg, None);
                    self.state
                        .staking
                        .set_status_message(format!("Change Commission FAILED: {}", error_msg));
                    return;
                }
            };

            // Execute change_commission operation
            // Convert commission from percentage (0-100) to 1e18 scale
            let commission_value = (params.commission * 10_000_000_000_000_000.0) as u64;
            let result = change_commission(
                rpc_client,
                signer.as_ref(),
                params.validator_id,
                commission_value,
            )
            .await;

            // Show toast based on result
            match result {
                Ok(staking_result) => {
                    let tx_hash = staking_result.tx_hash;
                    let id = self.toast_manager.next_id();
                    self.toast_manager.add(crate::tui::toast::Toast::new(
                        id,
                        "Transaction Submitted".to_string(),
                        format!("Change Commission TX: {}", tx_hash),
                        crate::tui::toast::ToastType::Success,
                    ));
                    self.state
                        .staking
                        .set_status_message(format!("Change Commission: TX submitted {}", tx_hash));
                }
                Err(e) => {
                    let error_msg = format!("Change Commission failed: {}", e);
                    let id = self.toast_manager.next_id();
                    self.toast_manager.add(crate::tui::toast::Toast::new(
                        id,
                        "Transaction Failed".to_string(),
                        error_msg.clone(),
                        crate::tui::toast::ToastType::Error,
                    ));
                    self.state
                        .staking
                        .set_status_message(format!("Change Commission FAILED: {}", error_msg));
                }
            }

            self.state.change_commission.close();
        }
    }

    /// Handle keyboard input for Query Delegator dialog
    pub(crate) async fn handle_query_delegator_key_event(&mut self, key: KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};

        // If querying, ignore all input except Esc to cancel
        if self.state.query_delegator.is_querying {
            if let (_, KeyCode::Esc) = (key.modifiers, key.code) {
                self.state.query_delegator.close();
            }
            return;
        }

        match (key.modifiers, key.code) {
            // Tab navigation between fields
            (_, KeyCode::Tab) => {
                self.state.query_delegator.next_field();
            }
            // Shift+Tab for previous field
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                self.state.query_delegator.prev_field();
            }
            // Confirm on Enter - query delegator and update staking state
            (_, KeyCode::Enter) => {
                self.handle_query_delegator_confirm().await;
            }
            // Cancel on Escape
            (_, KeyCode::Esc) => {
                self.state.query_delegator.close();
            }
            // Filter invalid characters, then pass valid input to textarea
            _ => {
                // Filter character input based on current field
                if let KeyCode::Char(c) = key.code {
                    let focused_field = self.state.query_delegator.focused_field as u8;
                    if !char_validation::is_valid_query_delegator_char(c, focused_field) {
                        return; // Reject invalid character
                    }
                }
                // Create ratatui crossterm event directly to avoid conversion issues
                let rat_key = RatKeyEvent {
                    code: match key.code {
                        KeyCode::Backspace => RatKeyCode::Backspace,
                        KeyCode::Char(c) => RatKeyCode::Char(c),
                        KeyCode::Left => RatKeyCode::Left,
                        KeyCode::Right => RatKeyCode::Right,
                        KeyCode::Home => RatKeyCode::Home,
                        KeyCode::End => RatKeyCode::End,
                        KeyCode::Delete => RatKeyCode::Delete,
                        _ => return,
                    },
                    modifiers: RatKeyModifiers::from_bits_truncate(key.modifiers.bits()),
                    kind: RatKeyEventKind::Press,
                    state: RatKeyEventState::empty(),
                };
                self.state
                    .query_delegator
                    .current_textarea_mut()
                    .input(rat_key);
            }
        }
    }

    /// Handle Query Delegator dialog confirmation
    async fn handle_query_delegator_confirm(&mut self) {
        use crate::staking::getters;

        // Validate inputs
        let params = match self.state.query_delegator.validate() {
            Ok(p) => p,
            Err(e) => {
                self.state.query_delegator.set_error(e, None);
                return;
            }
        };

        // Set querying state
        self.state.query_delegator.set_querying(true);

        // Get RPC client
        let rpc_client = match &self.rpc_client {
            Some(client) => client,
            None => {
                let error_msg = "No RPC connection. Check your node is running.";
                self.state.query_delegator.set_error(error_msg, None);
                self.state.query_delegator.set_querying(false);
                return;
            }
        };

        // Query delegator from RPC
        match getters::get_delegator(rpc_client, params.validator_id, &params.delegator_address)
            .await
        {
            Ok(delegator) => {
                // Update staking state with the queried delegation
                use crate::tui::staking::DelegationInfo;

                // Set delegator address
                self.state.staking.set_address(&params.delegator_address);

                // Create delegation info from query result
                let delegation = DelegationInfo::from_query_result(params.validator_id, &delegator);

                // Clear existing delegations and add this one
                self.state.staking.set_delegations(vec![delegation]);

                // Clone RPC client reference for async call (avoid borrow checker issue)
                let rpc_client_clone = rpc_client.clone();

                // Fetch all withdrawal slots for this delegation (0-7)
                // This updates pending_withdrawals so Withdraw dialog shows correct ready IDs
                self.fetch_withdrawals_for_delegation(
                    &rpc_client_clone,
                    params.validator_id,
                    &params.delegator_address,
                )
                .await;

                // Update balance from delegation amount
                let balance_mon = delegator.delegated_amount as f64 / 1e18;
                self.state.staking.set_balance(balance_mon);

                // Show success toast
                let id = self.toast_manager.next_id();
                self.toast_manager.add(crate::tui::toast::Toast::new(
                    id,
                    "Delegation Loaded".to_string(),
                    format!("Validator {}: {:.2} MON", params.validator_id, balance_mon),
                    crate::tui::toast::ToastType::Success,
                ));

                self.state.query_delegator.close();

                // Refresh staking data after query
                self.refresh_staking_data().await;
            }
            Err(e) => {
                let error_msg = format!("Query failed: {}", e);
                self.state.query_delegator.set_error(&error_msg, None);
                self.state.query_delegator.set_querying(false);

                // Show error toast
                let id = self.toast_manager.next_id();
                self.toast_manager.add(crate::tui::toast::Toast::new(
                    id,
                    "Query Failed".to_string(),
                    error_msg,
                    crate::tui::toast::ToastType::Error,
                ));
            }
        }
    }

    /// Fetch all withdrawal slots (0-7) for a delegation and update pending_withdrawals
    ///
    /// This is called after Q.Delegator succeeds to ensure the Withdraw dialog
    /// shows correct "Ready IDs" in its context hint.
    async fn fetch_withdrawals_for_delegation(
        &mut self,
        rpc_client: &crate::rpc::RpcClient,
        validator_id: u64,
        delegator_address: &str,
    ) {
        use crate::staking::getters;
        use crate::tui::staking::PendingWithdrawal;

        let mut withdrawals = Vec::new();

        // Fetch all 8 withdrawal slots (MAX_CONCURRENT_WITHDRAWALS = 8)
        for withdrawal_id in 0u8..8 {
            match getters::get_withdrawal_request(
                rpc_client,
                validator_id,
                delegator_address,
                withdrawal_id,
            )
            .await
            {
                Ok(withdrawal) => {
                    // Only add non-empty withdrawals (amount > 0)
                    if withdrawal.amount > 0 {
                        withdrawals.push(PendingWithdrawal::new(
                            validator_id,
                            withdrawal.amount,
                            withdrawal.activation_epoch,
                            withdrawal_id,
                        ));
                    }
                }
                Err(_) => {
                    // Skip failed slot queries - treat as empty/available
                }
            }
        }

        // Update pending_withdrawals in staking state
        self.state.staking.set_withdrawals(withdrawals);
    }

    /// Handle keyboard input for Query Validator dialog
    pub(crate) async fn handle_query_validator_key_event(&mut self, key: KeyEvent) {
        use crossterm::event::KeyCode;

        // If querying, ignore all input except Esc to cancel
        if self.state.query_validator.is_querying {
            if let KeyCode::Esc = key.code {
                self.state.query_validator.close();
            }
            return;
        }

        match (key.modifiers, key.code) {
            // Confirm on Enter - query validator
            (_, KeyCode::Enter) => {
                self.handle_query_validator_confirm().await;
            }
            // Cancel on Escape
            (_, KeyCode::Esc) => {
                self.state.query_validator.close();
            }
            // Filter invalid characters, then pass valid input to textarea
            _ => {
                // Filter character input - only numeric for validator_id
                if let KeyCode::Char(c) = key.code {
                    if !c.is_ascii_digit() {
                        return; // Reject non-numeric character
                    }
                }
                // Create ratatui crossterm event directly
                let rat_key = RatKeyEvent {
                    code: match key.code {
                        KeyCode::Backspace => RatKeyCode::Backspace,
                        KeyCode::Char(c) => RatKeyCode::Char(c),
                        KeyCode::Left => RatKeyCode::Left,
                        KeyCode::Right => RatKeyCode::Right,
                        KeyCode::Home => RatKeyCode::Home,
                        KeyCode::End => RatKeyCode::End,
                        KeyCode::Delete => RatKeyCode::Delete,
                        _ => return,
                    },
                    modifiers: RatKeyModifiers::from_bits_truncate(key.modifiers.bits()),
                    kind: RatKeyEventKind::Press,
                    state: RatKeyEventState::empty(),
                };
                self.state.query_validator.validator_id.input(rat_key);
            }
        }
    }

    /// Handle Query Validator dialog confirmation
    async fn handle_query_validator_confirm(&mut self) {
        use crate::staking::getters;

        // Validate input
        let validator_id = match self.state.query_validator.validate() {
            Ok(id) => id,
            Err(e) => {
                self.state.query_validator.set_error(e);
                return;
            }
        };

        // Set querying state
        self.state.query_validator.start_querying();

        // Get RPC client
        let rpc_client = match &self.rpc_client {
            Some(client) => client,
            None => {
                let error_msg = "No RPC connection. Check your node is running.";
                self.state.query_validator.set_error(error_msg);
                return;
            }
        };

        // Query validator
        match getters::get_validator(rpc_client, validator_id).await {
            Ok(validator) => {
                use crate::tui::staking::QueryValidatorResult;

                self.state
                    .query_validator
                    .set_result(QueryValidatorResult::Success(validator));

                // Refresh staking data after query
                self.refresh_staking_data().await;
            }
            Err(e) => {
                let error_msg = format!("Query failed: {}", e);
                self.state.query_validator.set_error(&error_msg);
            }
        }
    }

    /// Update the dialog context hint based on current state
    pub(crate) fn update_dialog_context_hint(&mut self) {
        use crate::tui::widgets::DialogType;

        match self.state.input_dialog.dialog_type {
            DialogType::Delegate => {
                let balance = self.state.staking.format_balance();
                self.state
                    .input_dialog
                    .set_context_hint(format!("Available: {} MON", balance));
            }
            DialogType::Undelegate => {
                if let Some(delegation) = self.state.staking.selected_delegation() {
                    self.state.input_dialog.set_context_hint(format!(
                        "Delegated: {} MON",
                        delegation.format_delegated_amount()
                    ));
                }
            }
            DialogType::Withdraw => {
                // Get ready withdrawal IDs for better UX (match Python SDK list-withdrawals)
                let ready_ids: Vec<u8> = self
                    .state
                    .staking
                    .pending_withdrawals
                    .iter()
                    .filter(|w| w.is_ready(self.state.staking.current_epoch))
                    .map(|w| w.withdrawal_index)
                    .collect();

                let hint = if ready_ids.is_empty() {
                    "No withdrawals ready".to_string()
                } else {
                    format!(
                        "Ready IDs: {}",
                        ready_ids
                            .iter()
                            .map(|id| id.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                self.state.input_dialog.set_context_hint(hint);
            }
            DialogType::Claim => {
                if let Some(delegation) = self.state.staking.selected_delegation() {
                    // Show rewards or "No rewards" warning (match Python SDK claim.py)
                    let hint = if delegation.rewards == 0 {
                        "No rewards available".to_string()
                    } else {
                        format!("Rewards: {} MON", delegation.format_rewards())
                    };
                    self.state.input_dialog.set_context_hint(hint);
                }
            }
            DialogType::Compound => {
                if let Some(delegation) = self.state.staking.selected_delegation() {
                    // Show rewards or "No rewards" warning (match Python SDK claim.py)
                    let hint = if delegation.rewards == 0 {
                        "No rewards to compound".to_string()
                    } else {
                        format!("Rewards: {} MON", delegation.format_rewards())
                    };
                    self.state.input_dialog.set_context_hint(hint);
                }
            }
            DialogType::Generic => {}
            // Query dialogs don't need context hints from staking state
            DialogType::QueryValidator => {}
            DialogType::QueryDelegator => {}
            DialogType::QueryWithdrawalRequest => {}
            DialogType::QueryDelegations => {}
            DialogType::QueryEstimateGas => {}
            DialogType::QueryTransaction => {}
        }
    }

    /// Handle keyboard input for Delegate dialog
    pub(crate) async fn handle_delegate_key_event(&mut self, key: KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key.modifiers, key.code) {
            // Tab navigation between fields
            (_, KeyCode::Tab) => {
                self.state.delegate.next_field();
            }
            // Shift+Tab for previous field
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                self.state.delegate.prev_field();
            }
            // Confirm on Enter - validate and proceed to confirmation
            (_, KeyCode::Enter) => {
                self.handle_delegate_confirm().await;
            }
            // Cancel on Escape
            (_, KeyCode::Esc) => {
                self.state.delegate.close();
            }
            // Filter invalid characters, then pass valid input to textarea
            _ => {
                // Filter character input based on current field
                if let KeyCode::Char(c) = key.code {
                    let focused_field = self.state.delegate.focused_field as u8;
                    if !char_validation::is_valid_delegate_char(c, focused_field) {
                        return; // Reject invalid character
                    }
                }
                // Create ratatui crossterm event directly for textarea
                let rat_key = RatKeyEvent {
                    code: match key.code {
                        KeyCode::Backspace => RatKeyCode::Backspace,
                        KeyCode::Char(c) => RatKeyCode::Char(c),
                        KeyCode::Left => RatKeyCode::Left,
                        KeyCode::Right => RatKeyCode::Right,
                        KeyCode::Home => RatKeyCode::Home,
                        KeyCode::End => RatKeyCode::End,
                        KeyCode::Delete => RatKeyCode::Delete,
                        _ => return,
                    },
                    modifiers: RatKeyModifiers::from_bits_truncate(key.modifiers.bits()),
                    kind: RatKeyEventKind::Press,
                    state: RatKeyEventState::empty(),
                };
                self.state.delegate.current_textarea_mut().input(rat_key);
            }
        }
    }

    /// Handle Delegate dialog confirmation
    async fn handle_delegate_confirm(&mut self) {
        use crate::tui::staking::{PendingStakingAction, StakingActionType};

        // Validate inputs
        let params = match self.state.delegate.validate() {
            Ok(p) => p,
            Err(e) => {
                self.state.delegate.set_error(e, None);
                return;
            }
        };

        // Convert MON to wei (u128)
        let amount_wei = (params.amount * 1e18) as u128;

        // Create pending action
        let pending_action = PendingStakingAction {
            action_type: StakingActionType::Delegate,
            validator_id: params.validator_id,
            amount: Some(amount_wei),
            withdrawal_index: None,
            auth_address: None,
        };

        // Close dialog and open confirmation popup
        self.state.delegate.close();
        self.state.confirm_popup.open(pending_action);
    }

    /// Handle keyboard input for Undelegate dialog
    pub(crate) async fn handle_undelegate_key_event(&mut self, key: KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key.modifiers, key.code) {
            // Tab navigation between fields
            (_, KeyCode::Tab) => {
                self.state.undelegate.next_field();
            }
            // Shift+Tab for previous field
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                self.state.undelegate.prev_field();
            }
            // Confirm on Enter - validate and proceed to confirmation
            (_, KeyCode::Enter) => {
                self.handle_undelegate_confirm().await;
            }
            // Cancel on Escape
            (_, KeyCode::Esc) => {
                self.state.undelegate.close();
            }
            // Filter invalid characters, then pass valid input to textarea
            _ => {
                // Filter character input based on current field
                if let KeyCode::Char(c) = key.code {
                    let focused_field = self.state.undelegate.focused_field as u8;
                    if !char_validation::is_valid_undelegate_char(c, focused_field) {
                        return; // Reject invalid character
                    }
                }
                // Create ratatui crossterm event directly to avoid conversion issues
                let rat_key = RatKeyEvent {
                    code: match key.code {
                        KeyCode::Backspace => RatKeyCode::Backspace,
                        KeyCode::Char(c) => RatKeyCode::Char(c),
                        KeyCode::Left => RatKeyCode::Left,
                        KeyCode::Right => RatKeyCode::Right,
                        KeyCode::Home => RatKeyCode::Home,
                        KeyCode::End => RatKeyCode::End,
                        KeyCode::Delete => RatKeyCode::Delete,
                        _ => return,
                    },
                    modifiers: RatKeyModifiers::from_bits_truncate(key.modifiers.bits()),
                    kind: RatKeyEventKind::Press,
                    state: RatKeyEventState::empty(),
                };
                self.state.undelegate.current_textarea_mut().input(rat_key);
            }
        }
    }

    /// Handle Undelegate dialog confirmation
    async fn handle_undelegate_confirm(&mut self) {
        use crate::tui::staking::{PendingStakingAction, StakingActionType};

        // Validate inputs
        let params = match self.state.undelegate.validate() {
            Ok(p) => p,
            Err(e) => {
                self.state.undelegate.set_error(e, None);
                return;
            }
        };

        // Get next withdrawal index
        let withdrawal_index = self.get_next_withdrawal_index(params.validator_id);

        // Convert MON to wei (u128)
        let amount_wei = (params.amount * 1e18) as u128;

        // Create pending action
        let pending_action = PendingStakingAction {
            action_type: StakingActionType::Undelegate,
            validator_id: params.validator_id,
            amount: Some(amount_wei),
            withdrawal_index,
            auth_address: None,
        };

        // Close dialog and open confirmation popup
        self.state.undelegate.close();
        self.state.confirm_popup.open(pending_action);
    }

    /// Handle keyboard input for Withdraw dialog
    pub(crate) async fn handle_withdraw_key_event(&mut self, key: KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key.modifiers, key.code) {
            // Tab navigation between fields
            (_, KeyCode::Tab) => {
                self.state.withdraw.next_field();
            }
            // Shift+Tab for previous field
            (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                self.state.withdraw.prev_field();
            }
            // Confirm on Enter - validate and proceed to confirmation
            (_, KeyCode::Enter) => {
                self.handle_withdraw_confirm().await;
            }
            // Cancel on Escape
            (_, KeyCode::Esc) => {
                self.state.withdraw.close();
            }
            // Filter invalid characters, then pass valid input to textarea
            _ => {
                // Filter character input based on current field
                if let KeyCode::Char(c) = key.code {
                    let focused_field = self.state.withdraw.focused_field as u8;
                    if !char_validation::is_valid_withdraw_char(c, focused_field) {
                        return; // Reject invalid character
                    }
                }
                // Create ratatui crossterm event directly to avoid conversion issues
                let rat_key = RatKeyEvent {
                    code: match key.code {
                        KeyCode::Backspace => RatKeyCode::Backspace,
                        KeyCode::Char(c) => RatKeyCode::Char(c),
                        KeyCode::Left => RatKeyCode::Left,
                        KeyCode::Right => RatKeyCode::Right,
                        KeyCode::Home => RatKeyCode::Home,
                        KeyCode::End => RatKeyCode::End,
                        KeyCode::Delete => RatKeyCode::Delete,
                        _ => return,
                    },
                    modifiers: RatKeyModifiers::from_bits_truncate(key.modifiers.bits()),
                    kind: RatKeyEventKind::Press,
                    state: RatKeyEventState::empty(),
                };
                self.state.withdraw.current_textarea_mut().input(rat_key);
            }
        }
    }

    /// Handle Withdraw dialog confirmation
    async fn handle_withdraw_confirm(&mut self) {
        use crate::tui::staking::{PendingStakingAction, StakingActionType};

        // Validate inputs
        let params = match self.state.withdraw.validate() {
            Ok(p) => p,
            Err(e) => {
                self.state.withdraw.set_error(e, None);
                return;
            }
        };

        // Create pending action
        let pending_action = PendingStakingAction {
            action_type: StakingActionType::Withdraw,
            validator_id: params.validator_id,
            amount: None,
            withdrawal_index: Some(params.withdrawal_id),
            auth_address: None,
        };

        // Close dialog and open confirmation popup
        self.state.withdraw.close();
        self.state.confirm_popup.open(pending_action);
    }
}
