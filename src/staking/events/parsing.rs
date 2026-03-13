//! Event parsing functions
//!
//! This module provides functions for parsing transaction logs into staking events.

use crate::utils::error::{Error, Result};

use super::helpers::{
    normalize_hex, parse_address_from_topic, parse_uint256_from_data, parse_uint64_from_data,
    parse_uint64_from_topic, parse_uint8_from_data, parse_validator_pubkeys_from_data,
};
use super::signatures::{
    ADD_VALIDATOR_EVENT_SIGNATURE, CHANGE_COMMISSION_EVENT_SIGNATURE,
    CLAIM_REWARDS_EVENT_SIGNATURE, COMPOUND_EVENT_SIGNATURE, DELEGATE_EVENT_SIGNATURE,
    UNDELEGATE_EVENT_SIGNATURE, VALIDATOR_CREATED_EVENT_SIGNATURE,
    VALIDATOR_STATUS_CHANGED_EVENT_SIGNATURE, WITHDRAW_EVENT_SIGNATURE,
};
use super::types::{
    AddValidatorEvent, ChangeCommissionEvent, ClaimRewardsEvent, CompoundEvent, DelegateEvent,
    StakingEvent, TransactionLog, UndelegateEvent, ValidatorCreatedEvent,
    ValidatorStatusChangedEvent, WithdrawEvent,
};

// =============================================================================
// MAIN PARSING FUNCTION
// =============================================================================

/// Parse a transaction log into a staking event
///
/// # Arguments
/// * `log` - Transaction log entry
///
/// # Returns
/// - `Some(StakingEvent)` if the log is a recognized staking event
/// - `None` if the log is not a staking event
///
/// # Errors
/// Returns error if the log data is malformed
pub fn parse_event(log: &TransactionLog) -> Result<Option<StakingEvent>> {
    // Must have at least one topic (event signature)
    if log.topics.is_empty() {
        return Ok(None);
    }

    // Get event signature (topic[0])
    let event_sig = normalize_hex(&log.topics[0]);

    // Match event signature and parse accordingly
    match event_sig.as_str() {
        s if s == normalize_hex(DELEGATE_EVENT_SIGNATURE) => parse_delegate_event(log).map(Some),
        s if s == normalize_hex(UNDELEGATE_EVENT_SIGNATURE) => {
            parse_undelegate_event(log).map(Some)
        }
        s if s == normalize_hex(WITHDRAW_EVENT_SIGNATURE) => parse_withdraw_event(log).map(Some),
        s if s == normalize_hex(CLAIM_REWARDS_EVENT_SIGNATURE) => {
            parse_claim_rewards_event(log).map(Some)
        }
        s if s == normalize_hex(COMPOUND_EVENT_SIGNATURE) => parse_compound_event(log).map(Some),
        s if s == normalize_hex(CHANGE_COMMISSION_EVENT_SIGNATURE) => {
            parse_change_commission_event(log).map(Some)
        }
        s if s == normalize_hex(ADD_VALIDATOR_EVENT_SIGNATURE) => {
            parse_add_validator_event(log).map(Some)
        }
        s if s == normalize_hex(VALIDATOR_CREATED_EVENT_SIGNATURE) => {
            parse_validator_created_event(log).map(Some)
        }
        s if s == normalize_hex(VALIDATOR_STATUS_CHANGED_EVENT_SIGNATURE) => {
            parse_validator_status_changed_event(log).map(Some)
        }
        _ => Ok(None), // Unknown event
    }
}

/// Extract all staking events from a list of logs
///
/// # Arguments
/// * `logs` - List of transaction logs
///
/// # Returns
/// Vector of parsed staking events (ignores non-staking logs)
pub fn extract_staking_events(logs: &[TransactionLog]) -> Result<Vec<StakingEvent>> {
    let mut events = Vec::new();
    for log in logs {
        if let Some(event) = parse_event(log)? {
            events.push(event);
        }
    }
    Ok(events)
}

// =============================================================================
// INDIVIDUAL EVENT PARSERS
// =============================================================================

/// Parse a Delegate event
///
/// Topics: [signature, valId, delegator]
/// Data: amount (uint256), activationEpoch (uint64)
fn parse_delegate_event(log: &TransactionLog) -> Result<StakingEvent> {
    // Need 3 topics: signature + valId + delegator
    if log.topics.len() < 3 {
        return Err(Error::Other(
            "Invalid Delegate event: missing topics".to_string(),
        ));
    }

    // valId is in topics[1], delegator is in topics[2]
    let validator_id = parse_uint64_from_topic(&log.topics[1])?;
    let delegator = parse_address_from_topic(&log.topics[2])?;
    let amount = parse_uint256_from_data(&log.data, 0)?;
    let activation_epoch = parse_uint64_from_data(&log.data, 32)?;

    Ok(StakingEvent::Delegate(DelegateEvent {
        validator_id,
        delegator,
        amount,
        activation_epoch,
    }))
}

/// Parse an Undelegate event
///
/// Topics: [signature, valId, delegator]
/// Data: withdrawal_id (uint8), amount (uint256), activationEpoch (uint64)
fn parse_undelegate_event(log: &TransactionLog) -> Result<StakingEvent> {
    if log.topics.len() < 3 {
        return Err(Error::Other(
            "Invalid Undelegate event: missing topics".to_string(),
        ));
    }

    // valId is in topics[1], delegator is in topics[2]
    let validator_id = parse_uint64_from_topic(&log.topics[1])?;
    let delegator = parse_address_from_topic(&log.topics[2])?;
    let withdrawal_id = parse_uint8_from_data(&log.data, 0)?;
    let amount = parse_uint256_from_data(&log.data, 32)?;
    let activation_epoch = parse_uint64_from_data(&log.data, 64)?;

    Ok(StakingEvent::Undelegate(UndelegateEvent {
        validator_id,
        delegator,
        withdrawal_id,
        amount,
        activation_epoch,
    }))
}

/// Parse a Withdraw event
///
/// Topics: [signature, valId, delegator]
/// Data: withdrawal_id (uint8), amount (uint256), activationEpoch (uint64)
fn parse_withdraw_event(log: &TransactionLog) -> Result<StakingEvent> {
    if log.topics.len() < 3 {
        return Err(Error::Other(
            "Invalid Withdraw event: missing topics".to_string(),
        ));
    }

    // valId is in topics[1], delegator is in topics[2]
    let validator_id = parse_uint64_from_topic(&log.topics[1])?;
    let delegator = parse_address_from_topic(&log.topics[2])?;
    let withdrawal_id = parse_uint8_from_data(&log.data, 0)?;
    let amount = parse_uint256_from_data(&log.data, 32)?;
    let activation_epoch = parse_uint64_from_data(&log.data, 64)?;

    Ok(StakingEvent::Withdraw(WithdrawEvent {
        validator_id,
        delegator,
        withdrawal_id,
        amount,
        activation_epoch,
    }))
}

/// Parse a ClaimRewards event
///
/// Topics: [signature, valId, delegator]
/// Data: amount (uint256), epoch (uint64)
fn parse_claim_rewards_event(log: &TransactionLog) -> Result<StakingEvent> {
    if log.topics.len() < 3 {
        return Err(Error::Other(
            "Invalid ClaimRewards event: missing topics".to_string(),
        ));
    }

    // valId is in topics[1], delegator is in topics[2]
    let validator_id = parse_uint64_from_topic(&log.topics[1])?;
    let delegator = parse_address_from_topic(&log.topics[2])?;
    let amount = parse_uint256_from_data(&log.data, 0)?;
    let epoch = parse_uint64_from_data(&log.data, 32)?;

    Ok(StakingEvent::ClaimRewards(ClaimRewardsEvent {
        validator_id,
        delegator,
        amount,
        epoch,
    }))
}

/// Parse a Compound event
///
/// Topics: [signature, valId, delegator]
/// Data: amount (uint256)
fn parse_compound_event(log: &TransactionLog) -> Result<StakingEvent> {
    if log.topics.len() < 3 {
        return Err(Error::Other(
            "Invalid Compound event: missing topics".to_string(),
        ));
    }

    // valId is in topics[1], delegator is in topics[2]
    let validator_id = parse_uint64_from_topic(&log.topics[1])?;
    let delegator = parse_address_from_topic(&log.topics[2])?;
    let amount = parse_uint256_from_data(&log.data, 0)?;

    Ok(StakingEvent::Compound(CompoundEvent {
        validator_id,
        delegator,
        amount,
    }))
}

/// Parse a ChangeCommission event
///
/// Topics: [signature, valId]
/// Data: old_commission (uint256), new_commission (uint256)
fn parse_change_commission_event(log: &TransactionLog) -> Result<StakingEvent> {
    if log.topics.len() < 2 {
        return Err(Error::Other(
            "Invalid ChangeCommission event: missing topics".to_string(),
        ));
    }

    // valId is in topics[1]
    let validator_id = parse_uint64_from_topic(&log.topics[1])?;
    let old_commission = parse_uint64_from_data(&log.data, 0)?;
    let new_commission = parse_uint64_from_data(&log.data, 32)?;

    Ok(StakingEvent::ChangeCommission(ChangeCommissionEvent {
        validator_id,
        old_commission,
        new_commission,
    }))
}

/// Parse an AddValidator event
///
/// Topics: [signature, auth_delegator, valId]
/// Data: secp_pubkey (bytes), bls_pubkey (bytes)
fn parse_add_validator_event(log: &TransactionLog) -> Result<StakingEvent> {
    if log.topics.len() < 3 {
        return Err(Error::Other(
            "Invalid AddValidator event: missing topics".to_string(),
        ));
    }

    // auth_delegator is in topics[1], valId is in topics[2]
    let owner = parse_address_from_topic(&log.topics[1])?;
    let validator_id = parse_uint64_from_topic(&log.topics[2])?;

    // Parse dynamic bytes arrays from data
    let (secp_pubkey, bls_pubkey) = parse_validator_pubkeys_from_data(&log.data)?;

    Ok(StakingEvent::AddValidator(AddValidatorEvent {
        owner,
        validator_id,
        secp_pubkey,
        bls_pubkey,
    }))
}

/// Parse a ValidatorCreated event
///
/// Topics: [signature, valId, auth_delegator]
/// Data: commission (uint256)
fn parse_validator_created_event(log: &TransactionLog) -> Result<StakingEvent> {
    if log.topics.len() < 3 {
        return Err(Error::Other(
            "Invalid ValidatorCreated event: missing topics".to_string(),
        ));
    }

    // valId is in topics[1], auth_delegator is in topics[2]
    let validator_id = parse_uint64_from_topic(&log.topics[1])?;
    let auth_delegator = parse_address_from_topic(&log.topics[2])?;
    let commission = parse_uint256_from_data(&log.data, 0)? as u64;

    Ok(StakingEvent::ValidatorCreated(ValidatorCreatedEvent {
        validator_id,
        auth_delegator,
        commission,
    }))
}

/// Parse a ValidatorStatusChanged event
///
/// Topics: [signature, valId]
/// Data: flags (uint64)
fn parse_validator_status_changed_event(log: &TransactionLog) -> Result<StakingEvent> {
    if log.topics.len() < 2 {
        return Err(Error::Other(
            "Invalid ValidatorStatusChanged event: missing topics".to_string(),
        ));
    }

    // valId is in topics[1]
    let validator_id = parse_uint64_from_topic(&log.topics[1])?;
    let flags = parse_uint64_from_data(&log.data, 0)?;

    Ok(StakingEvent::ValidatorStatusChanged(
        ValidatorStatusChangedEvent {
            validator_id,
            flags,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a mock log
    fn mock_log(_signature: &str, topics: Vec<&str>, data: &str) -> TransactionLog {
        TransactionLog {
            address: "0x0000000000000000000000000000000000001000".to_string(),
            topics: topics.iter().map(|s| s.to_string()).collect(),
            data: data.to_string(),
        }
    }

    #[test]
    fn test_parse_delegate_event() {
        let log = mock_log(
            DELEGATE_EVENT_SIGNATURE,
            vec![
                DELEGATE_EVENT_SIGNATURE,
                "0x0000000000000000000000000000000000000000000000000000000000000001",
                "0x000000000000000000000000abcdefabcdefabcdefabcdefabcdefabcdefabcd",
            ],
            "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000\
              0000000000000000000000000000000000000000000000000000000000000064",
        );

        let result = parse_event(&log).unwrap();
        assert!(result.is_some());

        if let Some(StakingEvent::Delegate(event)) = result {
            assert_eq!(event.validator_id, 1);
            assert_eq!(
                event.delegator,
                "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"
            );
            assert_eq!(event.amount, 1_000_000_000_000_000_000);
            assert_eq!(event.activation_epoch, 100);
        } else {
            panic!("Expected Delegate event");
        }
    }

    #[test]
    fn test_parse_undelegate_event() {
        let log = mock_log(
            UNDELEGATE_EVENT_SIGNATURE,
            vec![
                UNDELEGATE_EVENT_SIGNATURE,
                "0x0000000000000000000000000000000000000000000000000000000000000002",
                "0x000000000000000000000000abcdefabcdefabcdefabcdefabcdefabcdefabcd",
            ],
            "0x0000000000000000000000000000000000000000000000000000000000000003\
              0000000000000000000000000000000000000000000000000de0b6b3a7640000\
              00000000000000000000000000000000000000000000000000000000000000c8",
        );

        let result = parse_event(&log).unwrap();
        assert!(result.is_some());

        if let Some(StakingEvent::Undelegate(event)) = result {
            assert_eq!(event.validator_id, 2);
            assert_eq!(
                event.delegator,
                "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"
            );
            assert_eq!(event.withdrawal_id, 3);
            assert_eq!(event.amount, 1_000_000_000_000_000_000);
            assert_eq!(event.activation_epoch, 200);
        } else {
            panic!("Expected Undelegate event");
        }
    }

    #[test]
    fn test_parse_withdraw_event() {
        let log = mock_log(
            WITHDRAW_EVENT_SIGNATURE,
            vec![
                WITHDRAW_EVENT_SIGNATURE,
                "0x0000000000000000000000000000000000000000000000000000000000000005",
                "0x0000000000000000000000001111111111111111111111111111111111111111",
            ],
            "0x0000000000000000000000000000000000000000000000000000000000000000\
              000000000000000000000000000000000000000000000002b5e3af16b1880000\
              000000000000000000000000000000000000000000000000000000000000012c",
        );

        let result = parse_event(&log).unwrap();
        assert!(result.is_some());

        if let Some(StakingEvent::Withdraw(event)) = result {
            assert_eq!(event.validator_id, 5);
            assert_eq!(
                event.delegator,
                "0x1111111111111111111111111111111111111111"
            );
            assert_eq!(event.withdrawal_id, 0);
            assert_eq!(event.amount, 50_000_000_000_000_000_000u128);
            assert_eq!(event.activation_epoch, 300);
        } else {
            panic!("Expected Withdraw event");
        }
    }

    #[test]
    fn test_parse_claim_rewards_event() {
        let log = mock_log(
            CLAIM_REWARDS_EVENT_SIGNATURE,
            vec![
                CLAIM_REWARDS_EVENT_SIGNATURE,
                "0x000000000000000000000000000000000000000000000000000000000000000a",
                "0x0000000000000000000000002222222222222222222222222222222222222222",
            ],
            "0x0000000000000000000000000000000000000000000000008ac7230489e80000\
              00000000000000000000000000000000000000000000000000000000000001f4",
        );

        let result = parse_event(&log).unwrap();
        assert!(result.is_some());

        if let Some(StakingEvent::ClaimRewards(event)) = result {
            assert_eq!(event.validator_id, 10);
            assert_eq!(
                event.delegator,
                "0x2222222222222222222222222222222222222222"
            );
            assert_eq!(event.amount, 10_000_000_000_000_000_000u128);
            assert_eq!(event.epoch, 500);
        } else {
            panic!("Expected ClaimRewards event");
        }
    }

    #[test]
    fn test_parse_compound_event() {
        let log = mock_log(
            COMPOUND_EVENT_SIGNATURE,
            vec![
                COMPOUND_EVENT_SIGNATURE,
                "0x0000000000000000000000000000000000000000000000000000000000000014",
                "0x0000000000000000000000003333333333333333333333333333333333333333",
            ],
            "0x0000000000000000000000000000000000000000000000004563918244f40000",
        );

        let result = parse_event(&log).unwrap();
        assert!(result.is_some());

        if let Some(StakingEvent::Compound(event)) = result {
            assert_eq!(event.validator_id, 20);
            assert_eq!(
                event.delegator,
                "0x3333333333333333333333333333333333333333"
            );
            assert_eq!(event.amount, 5_000_000_000_000_000_000u128);
        } else {
            panic!("Expected Compound event");
        }
    }

    #[test]
    fn test_parse_change_commission_event() {
        let log = mock_log(
            CHANGE_COMMISSION_EVENT_SIGNATURE,
            vec![
                CHANGE_COMMISSION_EVENT_SIGNATURE,
                "0x0000000000000000000000000000000000000000000000000000000000000007",
            ],
            "0x00000000000000000000000000000000000000000000000000000000000003e8\
              00000000000000000000000000000000000000000000000000000000000007d0",
        );

        let result = parse_event(&log).unwrap();
        assert!(result.is_some());

        if let Some(StakingEvent::ChangeCommission(event)) = result {
            assert_eq!(event.validator_id, 7);
            assert_eq!(event.old_commission, 1000);
            assert_eq!(event.new_commission, 2000);
        } else {
            panic!("Expected ChangeCommission event");
        }
    }

    #[test]
    fn test_parse_add_validator_event() {
        let mut data = String::from("0x");
        data.push_str("0000000000000000000000000000000000000000000000000000000000000040");
        data.push_str("00000000000000000000000000000000000000000000000000000000000000a0");
        data.push_str("0000000000000000000000000000000000000000000000000000000000000040");
        for _ in 0..64 {
            data.push_str("ab");
        }
        data.push_str("0000000000000000000000000000000000000000000000000000000000000030");
        for _ in 0..48 {
            data.push_str("cd");
        }

        let log = mock_log(
            ADD_VALIDATOR_EVENT_SIGNATURE,
            vec![
                ADD_VALIDATOR_EVENT_SIGNATURE,
                "0x0000000000000000000000004444444444444444444444444444444444444444",
                "0x0000000000000000000000000000000000000000000000000000000000000015",
            ],
            &data,
        );

        let result = parse_event(&log).unwrap();
        assert!(result.is_some());

        if let Some(StakingEvent::AddValidator(event)) = result {
            assert_eq!(event.owner, "0x4444444444444444444444444444444444444444");
            assert_eq!(event.validator_id, 21);
            assert!(event.secp_pubkey.starts_with("0x"));
            assert!(event.secp_pubkey.contains("ab"));
            assert!(event.bls_pubkey.starts_with("0x"));
            assert!(event.bls_pubkey.contains("cd"));
        } else {
            panic!("Expected AddValidator event");
        }
    }

    #[test]
    fn test_parse_validator_created_event() {
        let log = mock_log(
            VALIDATOR_CREATED_EVENT_SIGNATURE,
            vec![
                VALIDATOR_CREATED_EVENT_SIGNATURE,
                "0x0000000000000000000000000000000000000000000000000000000000000015",
                "0x0000000000000000000000005555555555555555555555555555555555555555",
            ],
            "0x00000000000000000000000000000000000000000000000000000000000003e8",
        );

        let result = parse_event(&log).unwrap();
        assert!(result.is_some());

        if let Some(StakingEvent::ValidatorCreated(event)) = result {
            assert_eq!(event.validator_id, 21);
            assert_eq!(
                event.auth_delegator,
                "0x5555555555555555555555555555555555555555"
            );
            assert_eq!(event.commission, 1000); // 10% in basis points
        } else {
            panic!("Expected ValidatorCreated event");
        }
    }

    #[test]
    fn test_parse_validator_status_changed_event() {
        let log = mock_log(
            VALIDATOR_STATUS_CHANGED_EVENT_SIGNATURE,
            vec![
                VALIDATOR_STATUS_CHANGED_EVENT_SIGNATURE,
                "0x0000000000000000000000000000000000000000000000000000000000000016",
            ],
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        );

        let result = parse_event(&log).unwrap();
        assert!(result.is_some());

        if let Some(StakingEvent::ValidatorStatusChanged(event)) = result {
            assert_eq!(event.validator_id, 22);
            assert_eq!(event.flags, 1);
        } else {
            panic!("Expected ValidatorStatusChanged event");
        }
    }

    #[test]
    fn test_parse_unknown_event_returns_none() {
        let log = mock_log(
            "0xunknownsignature00000000000000000000000000000000000000000000000",
            vec!["0xunknownsignature00000000000000000000000000000000000000000000000"],
            "0x",
        );

        let result = parse_event(&log).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_event_empty_topics_returns_none() {
        let log = TransactionLog {
            address: "0x0000000000000000000000000000000000001000".to_string(),
            topics: vec![],
            data: "0x".to_string(),
        };

        let result = parse_event(&log).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_staking_events() {
        let logs = vec![
            mock_log(
                DELEGATE_EVENT_SIGNATURE,
                vec![
                    DELEGATE_EVENT_SIGNATURE,
                    "0x0000000000000000000000000000000000000000000000000000000000000001",
                    "0x000000000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                ],
                "0x000000000000000000000000000000000de0b6b3a76400000000000000000000000\
                  000000000000000000000000000000000000000000000000000000000000000064",
            ),
            mock_log(
                "0xunknownsignature00000000000000000000000000000000000000000000000",
                vec!["0xunknownsignature00000000000000000000000000000000000000000000000"],
                "0x",
            ),
            mock_log(
                COMPOUND_EVENT_SIGNATURE,
                vec![
                    COMPOUND_EVENT_SIGNATURE,
                    "0x0000000000000000000000000000000000000000000000000000000000000002",
                    "0x000000000000000000000000bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                ],
                "0x0000000000000000000000000000000000000000000000008ac7230489e80000",
            ),
        ];

        let events = extract_staking_events(&logs).unwrap();
        assert_eq!(events.len(), 2);

        match &events[0] {
            StakingEvent::Delegate(e) => {
                assert_eq!(e.validator_id, 1);
            }
            _ => panic!("Expected Delegate event"),
        }

        match &events[1] {
            StakingEvent::Compound(e) => {
                assert_eq!(e.validator_id, 2);
            }
            _ => panic!("Expected Compound event"),
        }
    }
}
