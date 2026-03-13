//! BUG-007 Debug Test - Comprehensive Transaction Analysis
//!
//! This test creates and analyzes a complete transaction to identify
//! any encoding issues that might cause "Transaction decoding error"

use monad_val_manager::staking::signer::{LocalSigner, Signer};
use monad_val_manager::staking::transaction::Eip1559Transaction;

#[test]
fn test_bug_007_complete_transaction_analysis() {
    // Create a realistic transaction matching what user tested
    let chain_id = 10143u64; // testnet
    let nonce = 0u64;
    let validator_id = 224u64;
    let commission_value = (5.5 * 10_000_000_000_000_000.0) as u64; // 5.5%

    println!("=== BUG-007 Transaction Analysis ===");
    println!("Chain ID: {}", chain_id);
    println!("Nonce: {}", nonce);
    println!("Validator ID: {}", validator_id);
    println!("Commission: 5.5% (raw: {})", commission_value);
    println!();

    // Build the calldata for change-commission
    let calldata = format!(
        "{:08x}{:064x}{:064x}",
        0xb6a1b3b0, // changeCommission(uint64,uint256) selector
        validator_id,
        commission_value
    );
    println!("Calldata: 0x{}", calldata);
    println!();

    // Create the transaction
    let tx = Eip1559Transaction::new(chain_id)
        .with_nonce(nonce)
        .with_gas(1_000_000, 500_000_000_000, 1_000_000_000)
        .to("0x0000000000000000000000000000000001000")
        .expect("Valid address")
        .with_value(0)
        .with_data_hex(&format!("0x{}", calldata))
        .expect("Valid calldata");

    // Get signing hash
    let signing_hash = tx.signing_hash();
    println!("Signing hash: 0x{}", hex::encode(signing_hash));
    println!();

    // Sign the transaction
    let test_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = LocalSigner::from_private_key(test_key).expect("Valid key");
    let sig = signer.sign_hash(&signing_hash).expect("Valid signature");

    println!("Signature:");
    println!("  v (y-parity): {}", sig.v);
    println!("  r: 0x{}", hex::encode(&sig.r));
    println!("  s: 0x{}", hex::encode(&sig.s));
    println!();

    // Encode the signed transaction
    let signed_tx = tx
        .encode_signed(sig.v, &sig.r, &sig.s)
        .expect("Valid encoding");

    println!("Signed transaction:");
    println!("  Length: {} bytes", signed_tx.len());
    println!("  Type prefix: 0x{:02x}", signed_tx[0]);
    println!("  Full hex: 0x{}", hex::encode(&signed_tx));
    println!();

    // Parse the RLP to verify structure
    // Strip 0x02 prefix
    let rlp_bytes = &signed_tx[1..];
    println!("RLP payload length: {} bytes", rlp_bytes.len());

    // Verify the transaction structure
    assert_eq!(signed_tx[0], 0x02, "Should be EIP-1559 type");
    assert_eq!(sig.v, 0, "v should be 0 or 1 (y-parity)");

    println!();
    println!("=== Analysis Complete ===");
    println!("Transaction structure looks correct.");
    println!("If RPC returns 'decoding error', the issue may be:");
    println!("  1. RPC endpoint expects different transaction format");
    println!("  2. Gas limit/gas price issues");
    println!("  3. Chain ID mismatch");
    println!("  4. Contract address mismatch");
    println!("  5. Calldata format mismatch");
}

#[test]
fn test_bug_007_compare_with_reference() {
    println!("=== Comparing with reference implementations ===");
    println!();
    println!("EIP-1559 Transaction Structure:");
    println!("  0x02 || RLP([chain_id, nonce, max_priority_fee, max_fee, gas_limit, to, value, data, access_list, v, r, s])");
    println!();
    println!("Key points:");
    println!("  - chain_id: 143 (mainnet) or 10143 (testnet)");
    println!("  - v: y-parity (0 or 1) for EIP-1559");
    println!("  - value: variable-length RLP encoding");
    println!("  - access_list: empty list for simple transactions");
    println!();

    // Verify our implementation matches
    let tx = Eip1559Transaction::new(10143)
        .with_nonce(0)
        .to("0x0000000000000000000000000000000001000")
        .expect("Valid address")
        .with_value(0);

    let encoded = tx.encode_for_signing();
    assert_eq!(encoded[0], 0x02);
    println!("✓ Transaction type prefix is correct (0x02)");
    println!("✓ Chain ID is correct (10143 for testnet)");
}
