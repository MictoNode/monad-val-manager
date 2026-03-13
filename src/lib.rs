//! Monad Validator Manager - Professional CLI tool for Monad blockchain validator management
//!
//! This library provides modules for:
//! - CLI command parsing and execution
//! - TUI dashboard for real-time monitoring
//! - RPC client for Monad node communication
//! - Doctor diagnostics and troubleshooting
//! - Staking operations (delegate, undelegate, withdraw, claim, compound)
//! - Performance optimization tools
//! - Configuration management

pub mod cli;
pub mod config;
pub mod doctor;
pub mod handlers;
pub mod perf;
pub mod rpc;
pub mod staking;
pub mod tui;
pub mod utils;

pub use config::Config;
pub use utils::error::{Error, Result};
