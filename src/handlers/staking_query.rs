//! Staking query command handler
//!
//! Handles read-only staking queries: epoch, validator, delegator, withdrawal-request, delegations.

use crate::cli::QueryCommands;
use crate::config::Config;
use crate::handlers::format_mon;
use crate::rpc::RpcClient;
use crate::staking::{constants::WITHDRAWAL_DELAY, getters};
use anyhow::Result;
use colored::Colorize;

/// Execute staking query subcommand
pub async fn execute(config: &Config, query: QueryCommands) -> Result<()> {
    let rpc_client = RpcClient::new(config.rpc_endpoint())?;

    match query {
        QueryCommands::Epoch => {
            println!("{}", "Querying current epoch information...".cyan());
            println!();

            match getters::get_epoch(&rpc_client).await {
                Ok(epoch_info) => {
                    println!("{} Epoch Information", "OK".green().bold());
                    println!("{}", "=================".blue());
                    println!("  Current Epoch: {}", epoch_info.epoch);
                    println!(
                        "  Epoch Transition: {}",
                        if epoch_info.is_epoch_transition {
                            "Yes"
                        } else {
                            "No"
                        }
                    );
                }
                Err(e) => {
                    println!("{} Failed to query epoch: {}", "X".red(), e);
                }
            }
        }
        QueryCommands::Validator { id } => {
            println!(
                "{}",
                format!("Querying validator {} information...", id).cyan()
            );
            println!();

            match getters::get_validator(&rpc_client, id).await {
                Ok(validator) => {
                    println!("{} Validator #{}", "OK".green().bold(), id);
                    println!("{}", "=================".blue());
                    println!("  Auth Address: {}", validator.auth_delegator);
                    println!("  Flags: {}", validator.flags);
                    println!();
                    println!("  Execution View:");
                    println!("    Stake: {} MON", format_mon(validator.execution_stake));
                    println!("    Commission: {:.2}%", validator.commission());
                    println!(
                        "    Unclaimed Rewards: {} MON",
                        format_mon(validator.unclaimed_rewards)
                    );
                    println!();
                    println!("  Consensus View:");
                    println!("    Stake: {} MON", format_mon(validator.consensus_stake));
                    println!(
                        "    Commission: {:.2}%",
                        (validator.consensus_commission as f64) / 1e16
                    );
                    println!();
                    println!("  Snapshot View:");
                    println!("    Stake: {} MON", format_mon(validator.snapshot_stake));
                    println!(
                        "    Commission: {:.2}%",
                        (validator.snapshot_commission as f64) / 1e16
                    );
                    println!();
                    println!(
                        "  Accumulated Rewards per Token: {:.18}",
                        validator.accumulated_rewards_per_token as f64 / 1e36
                    );
                    if !validator.secp_pub_key.is_empty() {
                        println!(
                            "  SECP PubKey: {}...",
                            &validator.secp_pub_key[..32.min(validator.secp_pub_key.len())]
                        );
                    }
                }
                Err(e) => {
                    println!("{} Failed to query validator: {}", "X".red(), e);
                }
            }
        }
        QueryCommands::Delegator {
            validator_id,
            address,
        } => {
            println!(
                "{}",
                format!(
                    "Querying delegator {} for validator {}...",
                    address, validator_id
                )
                .cyan()
            );
            println!();

            match getters::get_delegator(&rpc_client, validator_id, &address).await {
                Ok(delegator) => {
                    println!("{} Delegator Information", "OK".green().bold());
                    println!("{}", "=======================".blue());
                    println!("  Address: {}", address);
                    println!("  Validator ID: {}", validator_id);
                    println!(
                        "  Delegated Amount: {} MON",
                        format_mon(delegator.delegated_amount)
                    );
                    // Accumulated rewards per token is a fixed-point number (divide by 1e36)
                    let arpt = delegator.accumulated_rewards_per_token as f64 / 1e36;
                    println!("  Accumulated Rewards per Token: {:.18}", arpt);
                    println!("  Rewards: {} MON", format_mon(delegator.rewards));
                    println!("  Delta Stake: {} MON", format_mon(delegator.delta_stake));
                    println!(
                        "  Next Delta Stake: {} MON",
                        format_mon(delegator.next_delta_stake)
                    );
                    println!("  Delta Epoch: {}", delegator.delta_epoch);
                    println!("  Next Delta Epoch: {}", delegator.next_delta_epoch);
                }
                Err(e) => {
                    println!("{} Failed to query delegator: {}", "X".red(), e);
                }
            }
        }
        QueryCommands::WithdrawalRequest {
            validator_id,
            delegator_address,
            withdrawal_id,
        } => {
            println!(
                "{}",
                format!(
                    "Querying withdrawal request #{} for delegator {} on validator {}...",
                    withdrawal_id, delegator_address, validator_id
                )
                .cyan()
            );
            println!();

            match getters::get_withdrawal_request(
                &rpc_client,
                validator_id,
                &delegator_address,
                withdrawal_id,
            )
            .await
            {
                Ok(withdrawal) => {
                    println!("{} Withdrawal Request Information", "OK".green().bold());
                    println!("{}", "=================================".blue());
                    println!("  Delegator: {}", delegator_address);
                    println!("  Validator ID: {}", validator_id);
                    println!("  Withdrawal ID: {}", withdrawal_id);
                    println!("  Amount: {} MON", format_mon(withdrawal.amount));
                    println!("  Activation Epoch: {}", withdrawal.activation_epoch);

                    // Check if this is an empty/pending slot
                    if withdrawal.amount == 0 {
                        println!();
                        println!(
                            "{}",
                            "Note: This withdrawal slot is empty (no pending request).".yellow()
                        );
                    }
                }
                Err(e) => {
                    println!("{} Failed to query withdrawal request: {}", "X".red(), e);
                }
            }
        }
        QueryCommands::ListWithdrawals {
            validator_id,
            address,
        } => {
            println!(
                "{}",
                format!(
                    "Querying withdrawals for delegator {} on validator {}...",
                    address, validator_id
                )
                .cyan()
            );
            println!();

            // Get current epoch for status calculation
            let current_epoch = match getters::get_epoch(&rpc_client).await {
                Ok(epoch) => epoch.epoch,
                Err(e) => {
                    println!("{} Failed to get current epoch: {}", "X".red(), e);
                    return Ok(());
                }
            };

            // Fetch all 8 withdrawal slots (MAX_CONCURRENT_WITHDRAWALS = 8)
            let mut withdrawals = Vec::new();
            let mut pending_count = 0;
            let mut ready_count = 0;
            let mut available_count = 0;

            for withdrawal_id in 0u8..8 {
                match getters::get_withdrawal_request(
                    &rpc_client,
                    validator_id,
                    &address,
                    withdrawal_id,
                )
                .await
                {
                    Ok(withdrawal) => {
                        let status = if withdrawal.amount == 0 {
                            available_count += 1;
                            "Available ✨".to_string()
                        } else {
                            let required_epoch =
                                withdrawal.activation_epoch.saturating_add(WITHDRAWAL_DELAY);
                            if current_epoch >= required_epoch {
                                ready_count += 1;
                                format!(
                                    "Ready to claim ✅ (epoch {})",
                                    current_epoch.saturating_sub(required_epoch)
                                )
                            } else {
                                pending_count += 1;
                                format!(
                                    "Wait {} epoch(s) ⏳ (current: {}, required: {})",
                                    required_epoch.saturating_sub(current_epoch),
                                    current_epoch,
                                    required_epoch
                                )
                            }
                        };
                        withdrawals.push((withdrawal_id, withdrawal, status));
                    }
                    Err(e) => {
                        println!(
                            "{} Failed to query withdrawal #{}: {}",
                            "WARNING".yellow(),
                            withdrawal_id,
                            e
                        );
                    }
                }
            }

            // Display results in a table format
            println!("{} Withdrawal Requests", "OK".green().bold());
            println!("{}", "=======================".blue());
            println!("  Delegator: {}", address);
            println!("  Validator ID: {}", validator_id);
            println!("  Current Epoch: {}", current_epoch);
            println!();

            if withdrawals.is_empty() {
                println!("  Unable to fetch any withdrawal requests.");
            } else {
                println!(
                    "  {:<3} {:<20} {:<20} {:<30}",
                    "ID", "Amount", "Activation", "Status"
                );
                println!("  {}", "-".repeat(90));

                for (id, withdrawal, status) in &withdrawals {
                    let amount = if withdrawal.amount == 0 {
                        "(empty)".to_string()
                    } else {
                        format!("{} MON", format_mon(withdrawal.amount))
                    };

                    let activation = if withdrawal.amount == 0 {
                        "-".to_string()
                    } else {
                        format!("epoch {}", withdrawal.activation_epoch)
                    };

                    println!("  {:<3} {:<20} {:<20} {}", id, amount, activation, status);
                }

                println!();
                println!("  Summary:");
                println!(
                    "    Pending: {} | Ready to claim: {} | Available: {}",
                    pending_count.to_string().yellow().bold(),
                    ready_count.to_string().green().bold(),
                    available_count.to_string().dimmed()
                );
            }
        }
        QueryCommands::Delegations { address } => {
            println!(
                "{}",
                format!("Querying all delegations for {}...", address).cyan()
            );
            println!();

            match getters::get_all_delegations(&rpc_client, &address).await {
                Ok(validator_ids) => {
                    println!("{} Delegations for {}", "OK".green().bold(), address);
                    println!("{}", "=================================".blue());
                    if validator_ids.is_empty() {
                        println!("  No delegations found.");
                    } else {
                        println!("  Total Delegations: {}", validator_ids.len());
                        println!("  Validator IDs: {:?}", validator_ids);
                    }
                }
                Err(e) => {
                    println!("{} Failed to query delegations: {}", "X".red(), e);
                }
            }
        }
        QueryCommands::ValidatorSet { set_type } => {
            // Validate set_type
            let valid_types = ["consensus", "execution", "snapshot"];
            if !valid_types.contains(&set_type.as_str()) {
                println!(
                    "{} Invalid set type: '{}'. Valid options: consensus, execution, snapshot",
                    "X".red(),
                    set_type
                );
                println!();
                println!("Network: {}", config.network());
                return Ok(());
            }

            println!(
                "{}",
                format!("Querying {} validator set...", set_type).cyan()
            );
            println!();

            // BUG-001: Collect all validator IDs with pagination
            // Maximum attempts for large validator sets
            let mut all_validator_ids = Vec::new();
            let mut index = 0u64;
            let mut has_more = true;
            let max_tries = 1000;
            let mut tries = 0;

            while has_more && tries < max_tries {
                let valset_result = match set_type.as_str() {
                    "consensus" => getters::get_consensus_valset(&rpc_client, index).await,
                    "execution" => getters::get_execution_valset(&rpc_client, index).await,
                    "snapshot" => getters::get_snapshot_valset(&rpc_client, index).await,
                    _ => unreachable!(), // Already validated above
                };

                match valset_result {
                    Ok(valset) => {
                        // Add validators from this page
                        if !valset.validator_ids.is_empty() {
                            all_validator_ids.extend(valset.validator_ids.clone());
                        }

                        // Check if we should continue pagination
                        if !valset.has_more {
                            break;
                        }

                        // Move to next page (using total_count as next index)
                        has_more = valset.has_more;
                        index = valset.total_count;
                        tries += 1;

                        // Small delay between requests
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        println!("{} Failed to query validator set: {}", "X".red(), e);
                        println!();
                        println!("Network: {}", config.network());
                        return Ok(());
                    }
                }
            }

            // Fetch SECP pubkeys for each validator (matching reference Python SDK)
            println!(
                "{} {} Validator Set",
                "OK".green().bold(),
                set_type.to_uppercase()
            );
            println!("{}", "=================================".blue());

            if all_validator_ids.is_empty() {
                println!("  No validators found.");
            } else {
                // Fetch pubkeys in parallel with delay between batches
                for validator_id in all_validator_ids.iter() {
                    match getters::get_validator(&rpc_client, *validator_id).await {
                        Ok(validator) => {
                            // Display in reference format: "id: secp_pubkey"
                            if !validator.secp_pub_key.is_empty() {
                                println!("  {}: {}", validator_id, validator.secp_pub_key);
                            } else {
                                // Fallback if no pubkey
                                println!("  {}: (no pubkey)", validator_id);
                            }
                        }
                        Err(_) => {
                            // Skip failed validators
                            println!("  {}: (error)", validator_id);
                        }
                    }
                    // Small delay to avoid overwhelming RPC
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
            }
        }
        QueryCommands::Delegators { validator_id } => {
            println!(
                "{}",
                format!("Querying delegators for validator {}...", validator_id).cyan()
            );
            println!();

            // Pagination for large delegator lists (matching reference Python SDK)
            let mut all_delegators = Vec::new();
            let mut start_address = String::from("0x0000000000000000000000000000000000000000");
            let mut has_more = true;
            let max_tries = 1000; // Prevent infinite loops
            let mut tries = 0;

            while has_more && tries < max_tries {
                match getters::get_delegators(&rpc_client, validator_id, &start_address).await {
                    Ok(delegator_list) => {
                        // Track count to detect progress
                        let prev_count = all_delegators.len();

                        // Add addresses from this page (only new ones - no duplicates)
                        if !delegator_list.addresses.is_empty() {
                            for addr in &delegator_list.addresses {
                                if !all_delegators.contains(addr) {
                                    all_delegators.push(addr.clone());
                                }
                            }
                        }

                        let new_count = all_delegators.len();

                        // Check for progress - if no new addresses after reasonable tries, stop
                        if new_count == prev_count {
                            // No progress, stop to avoid infinite loop
                            break;
                        }

                        // Update pagination state
                        // CRITICAL FIX: If last_address is zero address, use the last address from the array
                        // This matches Python SDK behavior when contract returns 0x0000...000
                        if delegator_list.last_address
                            == "0x0000000000000000000000000000000000000000"
                            && !delegator_list.addresses.is_empty()
                        {
                            // Use the last address from the response as next start_address
                            start_address = delegator_list.addresses.last().unwrap().clone();
                        } else {
                            start_address = delegator_list.last_address.clone();
                        }
                        has_more = delegator_list.has_more;
                        tries += 1;

                        // Safety check: if we've collected too many, something is wrong
                        if new_count > 10000 {
                            println!(
                                "  {} Reached safety limit (10000 delegators), stopping pagination",
                                "WARNING".yellow()
                            );
                            break;
                        }

                        // Small delay between requests
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        println!("{} Failed to query delegators: {}", "X".red(), e);
                        println!();
                        println!("Network: {}", config.network());
                        return Ok(());
                    }
                }
            }

            println!(
                "{} Delegators for Validator #{}",
                "OK".green().bold(),
                validator_id
            );
            println!("{}", "==============================".blue());
            if all_delegators.is_empty() {
                println!("  No delegators found.");
            } else {
                println!("  {} Delegators", all_delegators.len());
                println!();
                for address in all_delegators.iter() {
                    println!("  {}", address);
                }
            }
        }
        QueryCommands::EstimateGas {
            from,
            to,
            data,
            value,
        } => {
            println!("{}", "Estimating gas for transaction...".cyan());
            println!();

            // BUG-004: Decode function selector for better UX
            let function_name = if data.len() >= 10 {
                let selector = &data[..10];
                // Known function selectors (first 4 bytes = 8 hex chars + 0x prefix)
                match selector {
                    "0x84994fec" => Some("delegate"),
                    "0xf478bacb" => Some("undelegate"),
                    "0x2f984830" => Some("claim_rewards"),
                    "0x697af404" => Some("compound_rewards"),
                    "0x8f1287bc" => Some("withdraw"),
                    "0x9b3f1e25" => Some("change_commission"),
                    "0xc430ccf0" => Some("add_validator"),
                    _ => None,
                }
            } else {
                None
            };

            // Format value as MON
            let value_wei = if let Some(v) = value.strip_prefix("0x") {
                u128::from_str_radix(v, 16).unwrap_or(0)
            } else {
                0
            };
            let value_mon = format_mon(value_wei);

            println!("  From: {}", from);
            println!("  To: {}", to);
            if let Some(fn_name) = function_name {
                println!("  Operation: {}", fn_name);
            }
            if value_wei > 0 {
                println!("  Value: {} MON", value_mon);
            }
            println!("  Data: {}...", &data[..data.len().min(66)]); // Show selector + first param
            println!();

            match rpc_client.estimate_gas(&from, &to, &data, &value).await {
                Ok(gas_estimate) => {
                    println!("{} Gas Estimate", "OK".green().bold());
                    println!("{}", "=============".blue());
                    println!("  Estimated Gas: {}", gas_estimate);
                    println!("  Gas (hex): 0x{:x}", gas_estimate);
                    println!();
                    println!("Note: Add a 20% buffer when setting the actual gas limit.");
                    println!("      Recommended: {}", (gas_estimate as f64 * 1.2) as u64);
                }
                Err(e) => {
                    println!("{} Failed to estimate gas: {}", "X".red(), e);
                    println!();
                    println!("Common causes:");
                    println!("  - Contract execution would revert");
                    println!("  - Insufficient balance for value transfer");
                    println!("  - Invalid calldata");
                    println!("  - From address doesn't exist (send 0 value first)");
                }
            }
        }
        QueryCommands::Proposer => {
            println!("{}", "Querying current proposer validator...".cyan());
            println!();

            match getters::get_proposer_val_id(&rpc_client).await {
                Ok(proposer_id) => {
                    println!("{} Current Proposer", "OK".green().bold());
                    println!("{}", "=================".blue());
                    println!("  Validator ID: {}", proposer_id);
                    println!();

                    // Try to get validator details for more context
                    match getters::get_validator(&rpc_client, proposer_id).await {
                        Ok(validator) => {
                            println!("  Auth Address: {}", validator.auth_delegator);
                            println!("  Commission: {:.2}%", validator.commission());
                            println!(
                                "  Delegated Amount: {} MON",
                                format_mon(validator.execution_stake)
                            );
                        }
                        Err(_) => {
                            // Validator details not available, just show ID
                        }
                    }
                }
                Err(e) => {
                    println!("{} Failed to query proposer: {}", "X".red(), e);
                }
            }
        }
        QueryCommands::Tx { hash } => {
            println!("{}", format!("Querying transaction {}...", hash).cyan());
            println!();

            match rpc_client.get_transaction_by_hash(&hash).await {
                Ok(tx) => {
                    if tx.is_null() {
                        println!("{} Transaction not found", "X".red());
                        println!();
                        println!("The transaction with hash {} was not found.", hash);
                        println!("This could mean:");
                        println!("  - The transaction doesn't exist");
                        println!("  - The transaction is pending");
                        println!("  - The transaction was on a different chain");
                    } else {
                        println!("{} Transaction Details", "OK".green().bold());
                        println!("{}", "=====================".blue());

                        // Extract common fields
                        let from = tx.get("from").and_then(|v| v.as_str()).unwrap_or("N/A");
                        let to = tx.get("to").and_then(|v| v.as_str()).unwrap_or("N/A");
                        let value = tx.get("value").and_then(|v| v.as_str()).unwrap_or("0x0");
                        let gas = tx.get("gas").and_then(|v| v.as_str()).unwrap_or("N/A");
                        let gas_price =
                            tx.get("gasPrice").and_then(|v| v.as_str()).unwrap_or("N/A");
                        let block_hash = tx.get("blockHash").and_then(|v| v.as_str());
                        let block_number = tx.get("blockNumber").and_then(|v| v.as_str());
                        let tx_index = tx.get("transactionIndex").and_then(|v| v.as_str());

                        println!("  Hash: {}", hash);
                        println!("  From: {}", from);
                        println!("  To: {}", to);

                        // BUG-003: Format wei as MON instead of showing raw wei
                        let value_wei = if let Some(v) = value.strip_prefix("0x") {
                            u128::from_str_radix(v, 16).unwrap_or(0)
                        } else {
                            0
                        };
                        let value_mon = format_mon(value_wei);
                        println!("  Value: {} MON", value_mon);

                        if let Some(bh) = block_hash {
                            println!("  Block Hash: {}", bh);
                        }
                        if let Some(bn) = block_number {
                            println!("  Block Number: {}", bn);
                        }
                        if let Some(ti) = tx_index {
                            println!("  Transaction Index: {}", ti);
                        }

                        println!("  Gas: {}", gas);
                        println!("  Gas Price: {}", gas_price);

                        // Show if it's a contract creation or call
                        if to == "0x" || to.is_empty() {
                            println!();
                            println!("  Type: Contract Creation");
                        }

                        // Show input data if present
                        if let Some(input) = tx.get("input").and_then(|v| v.as_str()) {
                            if input.len() > 10 {
                                println!();
                                println!(
                                    "  Input Data: {}... ({} chars total)",
                                    &input[..10],
                                    input.len()
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("{} Failed to query transaction: {}", "X".red(), e);
                    println!();
                    println!("Common causes:");
                    println!("  - Invalid transaction hash format");
                    println!("  - Transaction doesn't exist");
                    println!("  - RPC endpoint not reachable");
                }
            }
        }
    }

    println!();
    println!("Network: {}", config.network());

    Ok(())
}
