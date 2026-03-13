//! CLI command definitions

use clap::Subcommand;

/// Available CLI commands
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Interactive first-time setup wizard
    Init,

    /// Show node status (block height, sync status, peers)
    Status,

    /// Query account balance
    Balance {
        /// Address to query (optional, uses .env if not provided)
        #[arg(short, long)]
        address: Option<String>,
    },

    /// Run diagnostics and health checks
    Doctor,

    /// Show configuration
    ConfigShow,

    /// Staking operations (delegate, undelegate, claim rewards, etc.)
    #[command(name = "stake")]
    Staking {
        #[command(subcommand)]
        command: StakingCommands,
    },

    /// Transfer MON to another address
    Transfer {
        /// Recipient address
        #[arg(long)]
        address: String,

        /// Amount in MON (human-readable, e.g., 1.5)
        #[arg(short = 'a', long)]
        amount: String,

        /// Preview transaction without broadcasting
        #[arg(long)]
        dry_run: bool,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

/// Staking operations for Monad blockchain
#[derive(Debug, Subcommand)]
pub enum StakingCommands {
    /// Delegate MON to a validator
    Delegate {
        /// Validator ID to delegate to
        #[arg(short = 'V', long)]
        validator_id: u64,
        /// Amount in MON (e.g., 10.5)
        #[arg(short = 'a', long)]
        amount: String,
        /// Preview transaction without broadcasting
        #[arg(long)]
        dry_run: bool,
    },

    /// Undelegate MON from a validator
    Undelegate {
        /// Validator ID to undelegate from
        #[arg(short = 'V', long)]
        validator_id: u64,
        /// Amount in MON (e.g., 10.5)
        #[arg(short = 'a', long)]
        amount: String,
        /// Withdrawal queue index (0-255)
        #[arg(short = 'w', long)]
        withdrawal_id: u8,
        /// Preview transaction without broadcasting
        #[arg(long)]
        dry_run: bool,
    },

    /// Withdraw undelegated MON
    Withdraw {
        /// Validator ID to withdraw from
        #[arg(short = 'V', long)]
        validator_id: u64,
        /// Withdrawal queue index (0-255)
        #[arg(short = 'w', long)]
        withdrawal_id: u8,
        /// Preview transaction without broadcasting
        #[arg(long)]
        dry_run: bool,
    },

    /// Claim staking rewards
    ClaimRewards {
        /// Validator ID to claim rewards from
        #[arg(short = 'V', long)]
        validator_id: u64,
        /// Preview transaction without broadcasting
        #[arg(long)]
        dry_run: bool,
    },

    /// Compound staking rewards (re-stake claimed rewards)
    #[command(name = "compound-rewards")]
    CompoundRewards {
        /// Validator ID to compound rewards for
        #[arg(short = 'V', long)]
        validator_id: u64,
        /// Preview transaction without broadcasting
        #[arg(long)]
        dry_run: bool,
    },

    /// Register a new validator on the network
    ///
    /// Requires SECP256k1 and BLS12-381 private keys for the validator.
    /// Minimum stake: 100,000 MON
    AddValidator {
        /// SECP256k1 private key (64 hex chars, without 0x prefix preferred) - REQUIRED
        #[arg(long)]
        secp_privkey: String,
        /// BLS12-381 private key (64 hex chars, with or without 0x prefix) - REQUIRED
        #[arg(long)]
        bls_privkey: String,
        /// Authorized address for validator operations (required)
        #[arg(long, name = "AUTH_ADDRESS")]
        auth_address: String,
        /// Amount to stake in MON (e.g., 100000 for 100,000 MON)
        #[arg(short = 'a', long)]
        amount: String,
        /// Preview transaction without broadcasting
        #[arg(long)]
        dry_run: bool,
    },

    /// Change validator commission rate
    ///
    /// Commission is specified as percentage (0.0 to 100.0).
    /// Only the validator's authorized address can change the commission.
    ChangeCommission {
        /// Validator ID to change commission for
        #[arg(short = 'V', long)]
        validator_id: u64,
        /// New commission rate as percentage (0.0 to 100.0)
        #[arg(long)]
        commission: f64,
        /// Preview transaction without broadcasting
        #[arg(long)]
        dry_run: bool,
    },

    /// Query staking information
    Query {
        #[command(subcommand)]
        command: QueryCommands,
    },
}

/// Query subcommands for staking
#[derive(Debug, Subcommand)]
pub enum QueryCommands {
    /// Get current epoch information
    Epoch,

    /// Get validator information
    Validator {
        /// Validator ID
        #[arg(short = 'V', long)]
        id: u64,
    },

    /// Get delegator information
    Delegator {
        /// Validator ID
        #[arg(short = 'V', long)]
        validator_id: u64,
        /// Delegator address
        #[arg(short, long)]
        address: String,
    },

    /// Get withdrawal request information
    WithdrawalRequest {
        /// Validator ID
        #[arg(short = 'V', long)]
        validator_id: u64,
        /// Delegator address
        #[arg(short, long)]
        delegator_address: String,
        /// Withdrawal queue index (0-255, contract uint8)
        #[arg(short = 'w', long)]
        withdrawal_id: u8,
    },

    /// List all withdrawal requests for a delegator on a validator
    ListWithdrawals {
        /// Validator ID
        #[arg(short = 'V', long)]
        validator_id: u64,
        /// Delegator address (required)
        #[arg(short, long)]
        address: String,
    },

    /// Get all delegations for an address
    Delegations {
        /// Delegator address
        #[arg(short, long)]
        address: String,
    },

    /// Get validator set (consensus, execution, or snapshot)
    ValidatorSet {
        /// Type of validator set: consensus, execution, or snapshot
        #[arg(short = 't', long, default_value = "consensus")]
        set_type: String,
    },

    /// Get all delegators for a validator
    Delegators {
        /// Validator ID
        #[arg(short = 'V', long)]
        validator_id: u64,
    },

    /// Estimate gas for a transaction
    EstimateGas {
        /// From address (sender)
        #[arg(short, long)]
        from: String,
        /// To address (recipient/contract)
        #[arg(short, long)]
        to: String,
        /// Transaction data (calldata as hex string with 0x prefix)
        #[arg(short, long)]
        data: String,
        /// Value to send in wei (hex string with 0x prefix, default: 0x0)
        #[arg(short, long, default_value = "0x0")]
        value: String,
    },

    /// Get current proposer validator ID
    Proposer,

    /// Get transaction by hash
    Tx {
        /// Transaction hash (hex string with 0x prefix)
        #[arg(long)]
        hash: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_doctor_command_parsing() {
        #[derive(Parser)]
        struct TestCli {
            #[command(subcommand)]
            command: Option<Commands>,
        }

        let cli = TestCli::try_parse_from(["test", "doctor"]);
        assert!(cli.is_ok());
        if let Ok(cli) = cli {
            assert!(matches!(cli.command, Some(Commands::Doctor)));
        }
    }
}
