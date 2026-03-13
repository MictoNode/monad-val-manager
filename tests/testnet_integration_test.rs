//! Testnet Integration Tests
//!
//! These tests run against the real Monad testnet to verify
//! that our RPC calls and contract interactions work correctly.
//!
//! **Note**: These tests require internet connectivity and access to
//! the Monad testnet RPC endpoint.
//!
//! Run with: cargo test --test testnet_integration_test

use monad_val_manager::rpc::RpcClient;
use monad_val_manager::staking::getters;

/// Monad testnet RPC endpoint
const TESTNET_RPC: &str = "https://testnet-rpc.monad.xyz";

#[tokio::test]
async fn testnet_test_rpc_connection() {
    // Test basic RPC connectivity
    let client = RpcClient::new(TESTNET_RPC).expect("Failed to create RPC client");

    // Should be able to get block number
    let block_number = client
        .get_block_number()
        .await
        .expect("Failed to get block number");

    // Testnet should have blocks
    assert!(block_number > 0, "Testnet should have blocks");
    println!("✅ Connected to testnet, current block: {}", block_number);
}

#[tokio::test]
async fn testnet_test_get_epoch() {
    let client = RpcClient::new(TESTNET_RPC).expect("Failed to create RPC client");

    // Get current epoch from staking contract
    let epoch_info = getters::get_epoch(&client)
        .await
        .expect("Failed to get epoch");

    // Epoch should be a reasonable number
    assert!(epoch_info.epoch > 0, "Epoch should be > 0");
    println!(
        "✅ Current epoch: {}, is_transition: {}",
        epoch_info.epoch, epoch_info.is_epoch_transition
    );
}

#[tokio::test]
async fn testnet_test_get_validator_exists() {
    let client = RpcClient::new(TESTNET_RPC).expect("Failed to create RPC client");

    // Try to get validator ID 1 (likely exists on testnet)
    let result = getters::get_validator(&client, 1).await;

    // Validator 1 might or might not exist on testnet
    match result {
        Ok(validator) => {
            println!(
                "✅ Validator 1 found: {} (commission: {}%)",
                validator.auth_delegator,
                validator.commission()
            );
        }
        Err(e) => {
            println!(
                "⚠️  Validator 1 not found (expected on fresh testnet): {}",
                e
            );
            // This is ok - validator might not exist yet
        }
    }
}

#[tokio::test]
async fn testnet_test_get_proposer() {
    let client = RpcClient::new(TESTNET_RPC).expect("Failed to create RPC client");

    // Get current proposer
    let proposer_id = getters::get_proposer_val_id(&client)
        .await
        .expect("Failed to get proposer");

    // Proposer should be a valid validator ID (0 or higher)
    println!("✅ Current proposer validator ID: {}", proposer_id);
    assert!(proposer_id < 1000000, "Proposer ID should be reasonable");
}

#[tokio::test]
async fn testnet_test_get_consensus_valset() {
    let client = RpcClient::new(TESTNET_RPC).expect("Failed to create RPC client");

    // Get first page of consensus validator set
    let valset = getters::get_consensus_valset(&client, 0)
        .await
        .expect("Failed to get validator set");

    println!(
        "✅ Consensus validator set: {} validators, has_more: {}",
        valset.validator_ids.len(),
        valset.has_more
    );

    // Should have at least some data structure
    // Might be empty on fresh testnet
    if !valset.validator_ids.is_empty() {
        println!(
            "   First validators: {:?}",
            &valset.validator_ids[..valset.validator_ids.len().min(5)]
        );
    }
}

#[tokio::test]
async fn testnet_test_chain_id() {
    let client = RpcClient::new(TESTNET_RPC).expect("Failed to create RPC client");

    // Get chain ID
    let chain_id = client.get_chain_id().await.expect("Failed to get chain ID");

    // Monad testnet chain ID is 10143
    println!("✅ Chain ID: {}", chain_id);
    assert_eq!(chain_id, 10143, "Expected Monad testnet chain ID (10143)");
}
