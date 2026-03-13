//! Error types and handling

use thiserror::Error;

/// Application error type
#[derive(Debug, Error)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// RPC connection error
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// TUI error
    #[error("TUI error: {0}")]
    Tui(String),

    /// Key management error
    #[error("Key management error: {0}")]
    KeyManagement(String),

    /// Staking operation error
    #[error("Staking error: {0}")]
    Staking(String),

    /// Cryptographic signing error
    #[error("Signing error: {0}")]
    Signing(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Invalid input error
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Withdrawal not ready (epoch delay not met)
    #[error("Withdrawal is not ready to claim yet.\n  Current epoch: {current_epoch}\n  Withdrawal epoch: {withdrawal_epoch}\n  Required epoch: {required_epoch}\n  Withdrawal ID: {withdrawal_index}")]
    WithdrawalNotReady {
        current_epoch: u64,
        withdrawal_epoch: u64,
        required_epoch: u64,
        withdrawal_index: u8,
    },

    /// No delegation found with this validator
    #[error("No delegation found with validator #{validator_id}. You need to delegate to this validator first.")]
    NoDelegation { validator_id: u64 },

    /// No rewards available to claim
    #[error("No rewards available to claim. You may want to wait for rewards to accumulate.")]
    NoRewardsAvailable { validator_id: u64, rewards: u128 },

    /// Validator not found
    #[error("Validator #{validator_id} not found or not registered.")]
    ValidatorNotFound { validator_id: u64 },

    /// Timeout error
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Generic error with message
    #[error("{0}")]
    Other(String),
}

/// Result type alias for this crate
pub type Result<T> = std::result::Result<T, Error>;

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}

impl From<toml::ser::Error> for Error {
    fn from(e: toml::ser::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Network(e.to_string())
    }
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error::Other(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Config("test error".to_string());
        assert_eq!(format!("{}", err), "Configuration error: test error");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_withdrawal_not_ready_error_format() {
        let err = Error::WithdrawalNotReady {
            current_epoch: 10,
            withdrawal_epoch: 8,
            required_epoch: 9,
            withdrawal_index: 0,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Withdrawal is not ready to claim yet"));
        assert!(msg.contains("Current epoch: 10"));
        assert!(msg.contains("Withdrawal epoch: 8"));
        assert!(msg.contains("Required epoch: 9"));
        assert!(msg.contains("Withdrawal ID: 0"));
    }

    #[test]
    fn test_withdrawal_not_ready_error_with_delay() {
        let err = Error::WithdrawalNotReady {
            current_epoch: 5,
            withdrawal_epoch: 10,
            required_epoch: 11, // 10 + WITHDRAWAL_DELAY (1)
            withdrawal_index: 2,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Current epoch: 5"));
        assert!(msg.contains("Withdrawal epoch: 10"));
        assert!(msg.contains("Required epoch: 11"));
        assert!(msg.contains("Withdrawal ID: 2"));
    }
}
