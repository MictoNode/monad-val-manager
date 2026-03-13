//! Staking command handler
//!
//! Handles all staking operations: delegate, undelegate, withdraw, claim, compound.

use crate::cli::StakingCommands;
use crate::config::{load_env, Config};
use crate::handlers::parse_amount;
use crate::rpc::RpcClient;
use crate::staking::{create_signer, SignerType};
use anyhow::Result;
use colored::Colorize;

use super::dry_run;
use super::staking_query;

/// Execute staking command
pub async fn execute(config: &Config, command: StakingCommands) -> Result<()> {
    println!("{}", "MonadNode Manager - Staking".cyan().bold());
    println!("{}", "============================".cyan());
    println!();

    // Load environment variables from .env file
    load_env().ok();

    // Get the network type from config
    let _network_type = config.network.network_type;

    match command {
        StakingCommands::Delegate {
            validator_id,
            amount,
            dry_run,
        } => {
            // Parse amount (MON to wei)
            let amount_wei = parse_amount(&amount, None)?;

            println!("Validator ID: {}", validator_id);
            println!("Amount: {} MON", amount);
            println!();

            // Display signer info if using Ledger
            display_signer_info(config)?;

            let signer = create_signer(config)?;
            let rpc_client = RpcClient::new(config.rpc_endpoint())?;

            if dry_run {
                dry_run::execute_dry_run_delegate(
                    &rpc_client,
                    signer.as_ref(),
                    validator_id,
                    amount_wei,
                    &amount,
                    None,
                )
                .await?;
            } else {
                let result = crate::staking::delegate(
                    &rpc_client,
                    signer.as_ref(),
                    validator_id,
                    amount_wei,
                )
                .await?;

                println!("{} Transaction submitted", "OK".green().bold());
                println!("Transaction hash: {}", result.tx_hash);
            }
        }
        StakingCommands::Undelegate {
            validator_id,
            amount,
            withdrawal_id,
            dry_run,
        } => {
            let amount_wei = parse_amount(&amount, None)?;

            println!("Validator ID: {}", validator_id);
            println!("Amount: {} MON", amount);
            println!("Withdrawal ID: {}", withdrawal_id);
            println!();

            display_signer_info(config)?;
            let signer = create_signer(config)?;
            let rpc_client = RpcClient::new(config.rpc_endpoint())?;

            if dry_run {
                dry_run::execute_dry_run_undelegate(
                    &rpc_client,
                    signer.as_ref(),
                    validator_id,
                    amount_wei,
                    withdrawal_id,
                    &amount,
                    None,
                )
                .await?;
            } else {
                let result = crate::staking::undelegate(
                    &rpc_client,
                    signer.as_ref(),
                    validator_id,
                    amount_wei,
                    withdrawal_id,
                )
                .await?;

                println!("{} Transaction submitted", "OK".green().bold());
                println!("Transaction hash: {}", result.tx_hash);
            }
        }
        StakingCommands::Withdraw {
            validator_id,
            withdrawal_id,
            dry_run,
        } => {
            println!("Validator ID: {}", validator_id);
            println!("Withdrawal ID: {}", withdrawal_id);
            println!();

            display_signer_info(config)?;
            let signer = create_signer(config)?;
            let rpc_client = RpcClient::new(config.rpc_endpoint())?;

            if dry_run {
                dry_run::execute_dry_run_withdraw(
                    &rpc_client,
                    signer.as_ref(),
                    validator_id,
                    withdrawal_id,
                )
                .await?;
            } else {
                let result = crate::staking::withdraw(
                    &rpc_client,
                    signer.as_ref(),
                    validator_id,
                    withdrawal_id,
                )
                .await?;

                println!("{} Transaction submitted", "OK".green().bold());
                println!("Transaction hash: {}", result.tx_hash);
            }
        }
        StakingCommands::ClaimRewards {
            validator_id,
            dry_run,
        } => {
            println!("Validator ID: {}", validator_id);
            println!();

            display_signer_info(config)?;
            let signer = create_signer(config)?;
            let rpc_client = RpcClient::new(config.rpc_endpoint())?;

            if dry_run {
                dry_run::execute_dry_run_claim_rewards(&rpc_client, signer.as_ref(), validator_id)
                    .await?;
            } else {
                // Preflight checks
                println!("{}", "Running Preflight Checks...".yellow());
                let preflight = crate::staking::operations::claim_rewards_preflight(
                    &rpc_client,
                    validator_id,
                    signer.address(),
                )
                .await?;

                // Display preflight results
                println!("  Delegation: {}", "Found ✅".green().bold());
                println!("  Active stake: {} MON", preflight.active_stake_mon());
                println!("  Pending stake: {} MON", preflight.pending_stake_mon());
                println!("  Available rewards: {} MON", preflight.rewards_mon());
                println!("  Validator: {}", "Active ✅".green().bold());
                if let Some(balance) = preflight.balance_before {
                    println!("  Balance before: {} MON", balance as f64 / 1e18);
                }
                println!();

                // Validate and proceed
                match crate::staking::operations::validate_claim_preflight(&preflight) {
                    Ok(()) => {
                        let result = crate::staking::claim_rewards(
                            &rpc_client,
                            signer.as_ref(),
                            validator_id,
                        )
                        .await?;

                        println!("{}", "Transaction Results".cyan().bold());
                        println!("Status: {}", "✅ Success".green().bold());
                        println!("Transaction hash: {}", result.tx_hash);

                        // Post-transaction validation would require receipt waiting
                        // For now, just show the transaction hash
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return Err(e.into());
                    }
                }
            }
        }
        StakingCommands::CompoundRewards {
            validator_id,
            dry_run,
        } => {
            println!("Validator ID: {}", validator_id);
            println!();

            display_signer_info(config)?;
            let signer = create_signer(config)?;
            let rpc_client = RpcClient::new(config.rpc_endpoint())?;

            if dry_run {
                dry_run::execute_dry_run_compound_rewards(
                    &rpc_client,
                    signer.as_ref(),
                    validator_id,
                )
                .await?;
            } else {
                // Preflight checks
                println!("{}", "Running Preflight Checks...".yellow());
                let preflight = crate::staking::operations::claim_rewards_preflight(
                    &rpc_client,
                    validator_id,
                    signer.address(),
                )
                .await?;

                // Display preflight results
                println!("  Delegation: {}", "Found ✅".green().bold());
                println!("  Active stake: {} MON", preflight.active_stake_mon());
                println!("  Pending stake: {} MON", preflight.pending_stake_mon());
                println!("  Available rewards: {} MON", preflight.rewards_mon());
                println!("  Validator: {}", "Active ✅".green().bold());
                println!();

                // Validate and proceed
                match crate::staking::operations::validate_claim_preflight(&preflight) {
                    Ok(()) => {
                        let result =
                            crate::staking::compound(&rpc_client, signer.as_ref(), validator_id)
                                .await?;

                        println!("{}", "Transaction Results".cyan().bold());
                        println!("Status: {}", "✅ Success".green().bold());
                        println!("Transaction hash: {}", result.tx_hash);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return Err(e.into());
                    }
                }
            }
        }
        StakingCommands::AddValidator {
            secp_privkey,
            bls_privkey,
            auth_address,
            amount,
            dry_run,
        } => {
            // Parse private keys
            let secp_key = secp_privkey.strip_prefix("0x").unwrap_or(&secp_privkey);
            let bls_key = bls_privkey.strip_prefix("0x").unwrap_or(&bls_privkey);

            let secp_bytes = hex::decode(secp_key)
                .map_err(|e| anyhow::anyhow!("Invalid SECP private key: {}", e))?;
            let bls_bytes = hex::decode(bls_key)
                .map_err(|e| anyhow::anyhow!("Invalid BLS private key: {}", e))?;

            if secp_bytes.len() != 32 {
                return Err(anyhow::anyhow!(
                    "SECP private key must be 32 bytes (64 hex chars), got {} bytes",
                    secp_bytes.len()
                ));
            }
            if bls_bytes.len() != 32 {
                return Err(anyhow::anyhow!(
                    "BLS private key must be 32 bytes (64 hex chars), got {} bytes",
                    bls_bytes.len()
                ));
            }

            // Parse amount (in MON, convert to wei)
            let amount_mon: u128 = amount
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid amount: {}", e))?;
            let amount_wei = amount_mon * 1_000_000_000_000_000_000; // MON to wei

            // Validate minimum stake
            if amount_mon < 100_000 {
                return Err(anyhow::anyhow!(
                    "Minimum stake for validator registration is 100,000 MON"
                ));
            }

            // Commission is fixed at 0 for now (encoded in payload)
            let commission_value = 0u64;

            // Get tx signer (funded account)
            display_signer_info(config)?;
            let signer = create_signer(config)?;
            let rpc_client = RpcClient::new(config.rpc_endpoint())?;

            if dry_run {
                dry_run::execute_dry_run_add_validator(
                    &rpc_client,
                    signer.as_ref(),
                    &secp_bytes,
                    &bls_bytes,
                    &auth_address,
                    amount_wei,
                    commission_value,
                    0.0,
                    &amount,
                )
                .await?;
            } else {
                let result = crate::staking::add_validator_from_privkeys(
                    &rpc_client,
                    signer.as_ref(),
                    &secp_bytes,
                    &bls_bytes,
                    &auth_address,
                    amount_wei,
                    commission_value,
                )
                .await?;

                println!("{} Transaction submitted", "OK".green().bold());
                println!("Transaction hash: {}", result.tx_hash);
            }
        }
        StakingCommands::ChangeCommission {
            validator_id,
            commission,
            dry_run,
        } => {
            // Validate commission range (0.0 to 100.0)
            if !(0.0..=100.0).contains(&commission) {
                return Err(anyhow::anyhow!(
                    "Commission must be between 0.0 and 100.0 (got {})",
                    commission
                ));
            }

            println!("Validator ID: {}", validator_id);
            println!();

            // Get private key
            display_signer_info(config)?;
            let signer = create_signer(config)?;
            let rpc_client = RpcClient::new(config.rpc_endpoint())?;

            // Try to get current commission for display
            let current_commission_pct =
                match crate::staking::getters::get_validator(&rpc_client, validator_id).await {
                    Ok(validator) => Some(validator.commission()),
                    Err(_) => None,
                };
            let current_commission_bps = current_commission_pct.map(|p| (p * 100.0) as u64);

            // Display current commission if available
            if let Some(current_pct) = current_commission_pct {
                println!("Current Commission: {:.2}%", current_pct);
                println!("New Commission: {:.2}%", commission);
                println!();
            }

            // Convert percentage to 1e18 scale (1% = 10^16)
            let commission_value = (commission * 10_000_000_000_000_000.0) as u64;

            if dry_run {
                dry_run::execute_dry_run_change_commission(
                    &rpc_client,
                    signer.as_ref(),
                    validator_id,
                    commission,
                    current_commission_bps,
                )
                .await?;
            } else {
                let result = crate::staking::change_commission(
                    &rpc_client,
                    signer.as_ref(),
                    validator_id,
                    commission_value,
                )
                .await?;

                println!("{} Transaction submitted", "OK".green().bold());
                println!("Transaction hash: {}", result.tx_hash);
                println!();
                println!(
                    "Commission change: {} -> {:.2}%",
                    current_commission_pct
                        .map(|c| format!("{:.2}", c))
                        .unwrap_or_else(|| "unknown".to_string()),
                    commission
                );
            }
        }
        StakingCommands::Query { command: query } => {
            staking_query::execute(config, query).await?;
            return Ok(());
        }
    }

    println!();
    println!("Network: {}", config.network());
    println!("RPC Endpoint: {}", config.rpc_endpoint());

    Ok(())
}

/// Display signer information
///
/// Shows a warning when using local signer
fn display_signer_info(config: &Config) -> Result<()> {
    let signer_type = crate::staking::get_signer_type(config);

    match signer_type {
        SignerType::Local => {
            println!();
            println!(
                "{}",
                "Signing with a local private key (non-production)...".yellow()
            );
            println!(
                "{}",
                "For mainnet, use a hardware wallet and verify on-device.".red()
            );
            println!();
        }
        SignerType::Ledger => {
            // LedgerSigner will show its own prompts during signing
        }
    }

    Ok(())
}
