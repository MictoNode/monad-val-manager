# Command Reference

## Write Operations

### Add Validator

Register a new validator on the network.

**Requirements:**

- Minimum stake to join register validator: 100,000 MON
- Valid SECP256k1 private key (64 hex chars, with or without 0x prefix)
- Valid BLS private key (64 hex chars, with or without 0x prefix)
- Make sure the `auth-address` is an address you control and intend to perform validator operations with. This can be the same as the funded address. You can provide another address here to decouple staking and operations.

```bash
monad-val-manager stake add-validator \
  --secp-privkey "{{ VALIDATOR PRIVATE SECP KEY }}" \
  --bls-privkey "{{ VALIDATOR PRIVATE BLS KEY }}" \
  --auth-address "{{ AN ADDRESS THAT YOU CONTROL }}" \
  --amount 100000
```

**Expected Output:**

```bash
SECP Pubkey: 02a1b2c3d4e5f6789...
BLS Pubkey: b1a2b3c4d5e6f789...
Validator Created! ID: 1, Delegator: 0xF88....
Tx hash: 0x1234567890abcdef...
```

**Note:** Commission is hardcoded to 0% in the validator payload.

### Delegate

Delegate MON tokens to a validator.

```bash
monad-val-manager stake delegate \
  --validator-id 1 \
  --amount 1000

# Preview without sending (dry-run)
monad-val-manager stake delegate \
  --validator-id 1 \
  --amount 1000 \
  --dry-run
```

**Expected Output:**

```bash
Delegated 1000 MON to validator 1
Tx hash: 0xabcdef1234567890...
```

### Undelegate

Create a withdrawal request to undelegate tokens.

```bash
monad-val-manager stake undelegate \
  --validator-id 1 \
  --amount 500 \
  --withdrawal-id 0
```

**Expected Output:**

```bash
Undelegation requested: 500 MON from validator 1
Withdrawal ID: 0
Activation epoch: 367
Tx hash: 0x1234567890abcdef...
```

### Withdraw

Withdraw tokens from a completed undelegation request.

```bash
monad-val-manager stake withdraw \
  --validator-id 1 \
  --withdrawal-id 0
```

**Note:** Withdrawals can only be processed after the required waiting period (1 epoch after activation).

### Claim Rewards

Claim accumulated staking rewards.

```bash
monad-val-manager stake claim-rewards \
  --validator-id 1
```

**Expected Output:**

```bash
Claimed rewards for validator 1
Amount: 12.34567890 MON
Tx hash: 0xabcdef1234567890...
```

### Compound Rewards

Automatically restake rewards as additional delegation.

```bash
monad-val-manager stake compound-rewards \
  --validator-id 1
```

**Expected Output:**

```bash
Compounded 12.34567890 MON for validator 1
Tx hash: 0xabcdef1234567890...
```

### Change Commission

Update the commission for a Validator. Commission is specified as percentage (0.0 to 100.0).

```bash
monad-val-manager stake change-commission \
  --validator-id 1 \
  --commission 5.0
```

**Expected Output:**

```bash
Validator ID: 1
Current commission: 10.0%
New commission: 5.0%
Commission successfully changed from 10.0% to 5.0% for validator 1
Tx hash: 0xabcdef1234567890...
```

**Note:** Only the Validator's authorized address can change the commission.

---

## Query Commands

### Query Validator Information

```bash
monad-val-manager stake query validator --id 1
```

### Query Delegator Information

```bash
monad-val-manager stake query delegator \
  --validator-id 1 \
  --address 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
```

### Query Withdrawal Request

```bash
monad-val-manager stake query withdrawal-request \
  --validator-id 1 \
  --delegator-address 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --withdrawal-id 0
```

### Query Withdrawal List

Get all withdrawal requests (0-7) for a delegator:

```bash
monad-val-manager stake query list-withdrawals \
  --validator-id 1 \
  --address 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
```

### Query Validator Set

```bash
# Options: consensus, execution, snapshot
monad-val-manager stake query validator-set --set-type consensus
```

### Query Delegations for an Address

```bash
monad-val-manager stake query delegations \
  --address 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
```

### Query Delegators for a Validator

```bash
monad-val-manager stake query delegators \
  --validator-id 1
```

### Query Epoch Information

```bash
monad-val-manager stake query epoch
```

### Query Proposer

```bash
monad-val-manager stake query proposer
```

### Estimate Gas

```bash
monad-val-manager stake query estimate-gas \
  --from 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --to 0x0000000000000000000000000000000000001000 \
  --data 0x \
  --value 0x0
```

---

## Transfer Commands

### Transfer Native MON

```bash
monad-val-manager transfer \
  --address 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --amount 1000
```

---

## Get Help for Any Command

```bash
monad-val-manager --help
monad-val-manager stake --help
monad-val-manager stake delegate --help
# etc.
```
