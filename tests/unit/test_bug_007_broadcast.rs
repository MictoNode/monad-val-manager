//! BUG-007 Regression Test - Broadcast vs Dry-run Encoding
//!
//! This test reproduces the BUG-007 regression where broadcast mode fails
//! with "Transaction decoding error" while dry-run works correctly.

use monad_val_manager::staking::signer::{LocalSigner, Signer};
use monad_val_manager::staking::transaction::Eip1559Transaction;

#[test]
fn test_transaction_encoding_consistency() {
    // Create a test transaction
    let tx = Eip1559Transaction::new(10143) // testnet
        .with_nonce(0)
        .with_gas(1_000_000, 500_000_000_000, 1_000_000_000)
        .to("0x0000000000000000000000000000000000001000")
        .expect("Valid address")
        .with_value(1_000_000_000_000_000_000u128) // 1 MON
        .with_data_hex("0x84994fec00000000000000000000000000000000000000000000000000000000000000e0")
        .expect("Valid calldata");

    // Test 1: Encoding for signing (what dry-run uses)
    let signing_encoded = tx.encode_for_signing();

    // Test 2: Create a signer and sign the transaction
    let test_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = LocalSigner::from_private_key(test_key).expect("Valid key");

    let signature = signer.sign_hash(&tx.signing_hash()).expect("Valid signature");
    let signed_encoded = tx
        .encode_signed(signature.v, &signature.r, &signature.s)
        .expect("Valid encoding");

    // Debug output
    println!("Signing encoded length: {}", signing_encoded.len());
    println!("Signing encoded (first 100 bytes): {:?}", &signing_encoded[..100.min(signing_encoded.len())]);
    println!();
    println!("Signed encoded length: {}", signed_encoded.len());
    println!("Signed encoded (first 100 bytes): {:?}", &signed_encoded[..100.min(signed_encoded.len())]);
    println!();

    // Both should start with 0x02 (EIP-1559 type)
    assert_eq!(signing_encoded[0], 0x02, "Signing encoding should start with 0x02");
    assert_eq!(signed_encoded[0], 0x02, "Signed encoding should start with 0x02");

    // The signed version should be longer (includes signature)
    assert!(signed_encoded.len() > signing_encoded.len(), "Signed tx should be longer");

    // Verify the value field is encoded correctly in both
    // Value should be encoded as variable-length (not 32 bytes)
    // 1 MON = 0x0de0b6b3a7640000 (8 bytes)
    let expected_value_hex = "0de0b6b3a7640000";

    // Find the value field in the signing encoding
    let signing_hex = hex::encode(&signing_encoded);
    println!("Signing tx hex: {}", signing_hex);
    assert!(signing_hex.contains(expected_value_hex), "Value should be in signing encoding");

    // Find the value field in the signed encoding
    let signed_hex = hex::encode(&signed_encoded);
    println!("Signed tx hex: {}", signed_hex);
    assert!(signed_hex.contains(expected_value_hex), "Value should be in signed encoding");
}

#[test]
fn test_value_encoding_variable_length() {
    use monad_val_manager::staking::transaction::encode_u128_variable_length;

    // Test various values to ensure they're encoded correctly
    let test_cases = vec![
        (0u128, vec![0u8]),
        (1u128, vec![1u8]),
        (255u128, vec![0xFFu8]),
        (256u128, vec![1u8, 0u8]),
        (1_000_000_000_000_000_000u128, vec![0x0d, 0xe0, 0xb6, 0xb3, 0xa7, 0x64, 0x00, 0x00]),
    ];

    for (value, expected) in test_cases {
        let encoded = encode_u128_variable_length(value);
        assert_eq!(encoded, expected, "Value encoding mismatch for {}", value);
        println!("Value {}: {:02x?}", value, encoded);
    }
}

#[test]
fn test_rlp_stream_value_encoding() {
    // Test that RLP stream correctly handles variable-length encoding
    use rlp::RlpStream;

    let value = 1_000_000_000_000_000_000u128; // 1 MON

    let mut stream = RlpStream::new();
    stream.begin_list(1);

    // This is how we encode in the transaction
    let value_bytes = monad_val_manager::staking::transaction::encode_u128_variable_length(value);
    stream.append(&value_bytes.as_slice());

    let encoded = stream.out();

    println!("RLP encoded value: {:?}", hex::encode(&encoded));

    // Verify the value bytes are in the RLP output
    let value_hex = hex::encode(value_bytes);
    let rlp_hex = hex::encode(&encoded);
    assert!(rlp_hex.contains(&value_hex), "Value should be in RLP encoding");
}
