//! CLI Command Handlers
//!
//! This module contains all command handler functions extracted from main.rs
//! for better maintainability and single responsibility principle.

// Utility functions
mod utils;

// Command handlers - Phase 7.2 (Simple)
mod config_cmd;
mod status;

// Command handlers - Phase 7.3 (Medium)
mod doctor;
mod init;
mod tui;

// Command handlers - Phase 7.4 (Complex)
mod balance;
mod dry_run;
mod staking;
mod staking_query;
mod transfer;

// Re-export utilities for use by other handlers
pub use utils::*;

// Re-export handler functions
pub use balance::execute as execute_balance;
pub use config_cmd::execute as execute_config_show;
pub use doctor::execute as execute_doctor;
pub use dry_run::{
    build_unsigned_transaction, execute_dry_run_claim_rewards, execute_dry_run_compound_rewards,
    execute_dry_run_delegate, execute_dry_run_undelegate, execute_dry_run_withdraw,
};
pub use init::execute as execute_init;
pub use staking::execute as execute_staking;
pub use staking_query::execute as execute_staking_query;
pub use status::execute as execute_status;
pub use transfer::{execute as execute_transfer, parse_transfer_amount, TRANSFER_GAS_LIMIT};
pub use tui::execute as execute_tui;
