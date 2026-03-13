# CLI Reference - Complete Command Bank

**Version:** 1.0.0
**Last Updated:** 2026-03-12
**Project:** Monad Validator Manager
**Binary:** `monad-val-manager`

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Global Options](#global-options)
3. [Main Commands](#main-commands)
4. [Staking Write Operations](#staking-write-operations)
5. [Staking Query Operations](#staking-query-operations)
6. [Transfer Commands](#transfer-commands)
7. [Diagnostic Commands](#diagnostic-commands)
8. [Configuration Management](#configuration-management)
9. [Usage Patterns & Best Practices](#usage-patterns--best-practices)
10. [Network Configuration](#network-configuration)
11. [Error Handling](#error-handling)

---

## Quick Start

### Installation

```bash
# Build from source
cargo build --release

# Binary will be at: target/release/monad-val-manager
```

### First-Time Setup

```bash
# Initialize configuration (REQUIRED first step)
monad-val-manager init

# Check node status
monad-val-manager status

# Check balance
monad-val-manager balance

# Run TUI
monad-val-manager
```

### Init Wizard Prompts

When you run `monad-val-manager init`, you'll be prompted for:

| Prompt | Description | Default |
|--------|-------------|---------|
| Network | mainnet or testnet | mainnet |
| Private Key | Your funded address private key (hidden input) | - |
| RPC Endpoint | Monad node RPC URL | http://localhost:8080 |

This creates:
- Config: `~/.config/monad-val-manager/config.toml`
- Secrets: `~/.config/monad-val-manager/.env`

---

## Global Options

These options can be used with ANY command:

| Option | Short | Type | Description | Default |
|--------|-------|------|-------------|---------|
| `--network` | `-n` | string | Network to use (mainnet/testnet) | mainnet |
| `--config` | `-c` | path | Custom config file path | ~/.config/monad-val-manager/config.toml |
| `--rpc` | `-r` | url | Override RPC endpoint | From config |
| `--verbose` | `-v` | flag | Increase verbosity (can be used multiple times) | Info level |
| `--help` | `-h` | flag | Show help message | - |

### Examples

```bash
# Use testnet
monad-val-manager status --network testnet

# Use custom RPC
monad-val-manager status --rpc https://monad-testnet.rpc.io

# Verbose output
monad-val-manager -vv status

# Combine options
monad-val-manager -n testnet -r http://localhost:8545 stake delegate --validator-id 1 --amount 100
```

---

## Main Commands

| Command | Description | Status |
|---------|-------------|--------|
| `init` | First-time setup wizard | ✅ Stable |
| `status` | Node status, sync, peers, block info | ✅ Stable |
| `balance` [--address] | Account balance query | ✅ Stable |
| `doctor` | Run 16 diagnostic checks | ✅ Stable |
| `config-show` | Display current configuration | ✅ Stable |
| `stake <subcommand>` | Staking operations | ✅ Stable |
| `transfer` | Native MON transfers | ✅ Stable |

---

## Staking Write Operations

⚠️ **WARNING:** These operations send transactions and consume gas!

### Delegate

Delegate MON tokens to a validator.

```bash
monad-val-manager stake delegate \
  --validator-id <VALIDATOR_ID> \
  --amount <AMOUNT> \
  [--dry-run]
```

#### Parameters

| Parameter | Short | Type | Required | Description |
|-----------|-------|------|----------|-------------|
| `--validator-id` | `-V` | u64 | Yes | Validator ID to delegate to |
| `--amount` | `-a` | decimal | Yes | Amount in MON (e.g., 100, 1.5, 0.01) |
| `--dry-run` | | flag | No | Preview without sending |

#### Examples

```bash
# Preview delegation
monad-val-manager stake delegate --validator-id 1 --amount 100 --dry-run

# Execute delegation
monad-val-manager stake delegate --validator-id 1 --amount 100

# With global options
monad-val-manager -n testnet stake delegate -V 1 -a 50
```

#### Expected Output

```
✓ Delegation Preview:
  Validator ID: 1
  Amount: 100.000000000000000000 MON
  From: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb

Proceed? [y/N]: y

Delegated 100 MON to validator 1
Tx hash: 0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890
```

---

### Undelegate

Create a withdrawal request to undelegate tokens from a validator.

```bash
monad-val-manager stake undelegate \
  --validator-id <VALIDATOR_ID> \
  --amount <AMOUNT> \
  --withdrawal-id <WITHDRAWAL_ID> \
  [--dry-run]
```

#### Parameters

| Parameter | Short | Type | Required | Range | Description |
|-----------|-------|------|----------|-------|-------------|
| `--validator-id` | `-V` | u64 | Yes | - | Validator ID to undelegate from |
| `--amount` | `-a` | decimal | Yes | - | Amount to undelegate (MON) |
| `--withdrawal-id` | `-w` | u8 | Yes | 0-255 | Unique withdrawal slot ID |
| `--dry-run` | | flag | No | - | Preview without sending |

#### Withdrawal ID Explained

Each address can have up to 256 withdrawal slots per validator (0-255). This is a contract constraint (uint8). Choose an unused slot for each new undelegation.

#### Examples

```bash
# Create withdrawal request
monad-val-manager stake undelegate --validator-id 1 --amount 50 --withdrawal-id 0

# Preview first
monad-val-manager stake undelegate -V 1 -a 50 -w 0 --dry-run

# Use different slot for second undelegation
monad-val-manager stake undelegate -V 1 -a 25 -w 1
```

#### Expected Output

```
✓ Undelegation Preview:
  Validator ID: 1
  Amount: 50.000000000000000000 MON
  Withdrawal ID: 0

Proceed? [y/N]: y

Undelegation requested: 50 MON from validator 1
Withdrawal ID: 0
Activation epoch: 367
Tx hash: 0x1234567890abcdef...
```

---

### Withdraw

Withdraw tokens from a completed undelegation request.

```bash
monad-val-manager stake withdraw \
  --validator-id <VALIDATOR_ID> \
  --withdrawal-id <WITHDRAWAL_ID> \
  [--dry-run]
```

#### Parameters

| Parameter | Short | Type | Required | Range | Description |
|-----------|-------|------|----------|-------|-------------|
| `--validator-id` | `-V` | u64 | Yes | - | Validator ID |
| `--withdrawal-id` | `-w` | u8 | Yes | 0-255 | Withdrawal slot to withdraw from |
| `--dry-run` | | flag | No | - | Preview without sending |

#### Withdrawal Timing

Withdrawals become available **2 epochs after** the undelegation request epoch. Use `list-withdrawals` to check status.

#### Examples

```bash
# Check withdrawal status first
monad-val-manager stake query list-withdrawals --validator-id 1 --address 0x742d...

# Withdraw if ready
monad-val-manager stake withdraw --validator-id 1 --withdrawal-id 0
```

---

### Claim Rewards

Claim accumulated staking rewards from a validator.

```bash
monad-val-manager stake claim-rewards \
  --validator-id <VALIDATOR_ID> \
  [--dry-run]
```

#### Parameters

| Parameter | Short | Type | Required | Description |
|-----------|-------|------|----------|-------------|
| `--validator-id` | `-V` | u64 | Yes | Validator ID to claim from |
| `--dry-run` | | flag | No | Preview without sending |

#### Examples

```bash
# Preview rewards
monad-val-manager stake claim-rewards --validator-id 1 --dry-run

# Claim rewards
monad-val-manager stake claim-rewards -V 1
```

#### Expected Output

```
Claimed rewards for validator 1
Amount: 12.345678901234567890 MON
Tx hash: 0xabcdef1234567890...
```

---

### Compound Rewards

Automatically restake claimed rewards as additional delegation.

```bash
monad-val-manager stake compound-rewards \
  --validator-id <VALIDATOR_ID> \
  [--dry-run]
```

#### What is Compounding?

Compound = Claim Rewards + Delegate in one transaction. This is more gas-efficient than doing them separately.

#### Requirements

- Must have existing delegation to the validator
- Validator must have accrued rewards
- Validator must be active

#### Examples

```bash
# Compound rewards
monad-val-manager stake compound-rewards --validator-id 1

# Preview first
monad-val-manager stake compound-rewards -V 1 --dry-run
```

#### Expected Output

```
Compounded 12.34567890 MON for validator 1
New delegation: 1012.34567890 MON
Tx hash: 0xabcdef1234567890...
```

---

### Add Validator

Register a new validator on the network.

⚠️ **Minimum 100,000 MON required**

```bash
monad-val-manager stake add-validator \
  --secp-privkey <SECP_PRIVKEY> \
  --bls-privkey <BLS_PRIVKEY> \
  --auth-address <AUTH_ADDRESS> \
  --amount <AMOUNT> \
  [--dry-run]
```

#### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `--secp-privkey` | 64 hex chars | Yes | SECP256k1 private key (with/without 0x) |
| `--bls-privkey` | 64 hex chars | Yes | BLS12-381 private key (with/without 0x) |
| `--auth-address` | 0x... | Yes | Address authorized for validator operations |
| `--amount` | MON (decimal) | Yes | Minimum 100,000 MON |
| `--dry-run` | flag | No | Preview without sending |

#### Key Extraction

Use `monad-keystore` to extract keys from your node:

```bash
# Extract SECP key
monad-keystore recover --password "$KEYSTORE_PASSWORD" \
  --keystore-path /home/monad/monad-bft/config/id-secp --key-type secp

# Extract BLS key
monad-keystore recover --password "$KEYSTORE_PASSWORD" \
  --keystore-path /home/monad/monad-bft/config/id-bls --key-type bls
```

#### Examples

```bash
# Register validator (production - use Ledger!)
monad-val-manager stake add-validator \
  --secp-privkey "a1b2c3d4...64chars" \
  --bls-privkey "e5f6g7h8...64chars" \
  --auth-address 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --amount 100000
```

#### Expected Output

```
SECP Pubkey: 03bbf692002bda53050f22289d4da8fe0bec8b81a6b0d4f641760....
BLS Pubkey: 985d3f7052ac5ad586592ba1a240b0260b5351a9c3973a471fff79....

Do the derived public keys match? [y/N]: y

Validator Created! ID: 1, Delegator: 0xF88....
Tx hash: 0xe11114c8e6dd1dc5e0cde400ce5014dab257....
```

---

### Change Commission

Update the commission rate for a validator you control.

```bash
monad-val-manager stake change-commission \
  --validator-id <VALIDATOR_ID> \
  --commission <COMMISSION> \
  [--dry-run]
```

#### Parameters

| Parameter | Short | Type | Required | Range | Description |
|-----------|-------|------|----------|-------|-------------|
| `--validator-id` | `-V` | u64 | Yes | - | Your validator ID |
| `--commission` | | f64 % | Yes | 0.0-100.0 | New commission rate |
| `--dry-run` | | flag | No | - | Preview without sending |

#### Authorization

Only the validator's authorized address can change commission.

#### Examples

```bash
# Change to 5%
monad-val-manager stake change-commission --validator-id 1 --commission 5.0

# Set to 0% (community validator)
monad-val-manager stake change-commission -V 1 --commission 0.0

# Preview
monad-val-manager stake change-commission -V 1 --commission 10.0 --dry-run
```

#### Expected Output

```
Validator ID: 1
Current commission: 10.0%
New commission: 5.0%

Commission successfully changed from 10.0% to 5.0% for validator 1
Tx hash: 0xabcdef1234567890...
```

---

## Staking Query Operations

🔍 **Safe to run** - These are read-only queries that don't send transactions.

### Query Epoch

Get current epoch information.

```bash
monad-val-manager stake query epoch
```

**Output:**
```
Current epoch: 367
```

---

### Query Validator

Get detailed information about a specific validator.

```bash
monad-val-manager stake query validator --id <VALIDATOR_ID>
```

#### Parameters

| Parameter | Short | Type | Required |
|-----------|-------|------|----------|
| `--id` | `-V` | u64 | Yes |

#### Output

```
Validator ID: 1
SECP Pubkey: 03bbf692002bda53050f22289d4da8fe0bec8b81a6b0d4f641760...
Auth Delegator: 0xF88...
Commission: 0.00%
Execution Stake: 100000.000000000000000000 MON
Consensus Stake: 100000.000000000000000000 MON
```

---

### Query Delegator

Get delegation and reward information for an address.

```bash
monad-val-manager stake query delegator \
  --validator-id <VALIDATOR_ID> \
  --address <ADDRESS>
```

#### Parameters

| Parameter | Short | Type | Required |
|-----------|-------|------|----------|
| `--validator-id` | `-V` | u64 | Yes |
| `--address` | `-a` | 0x... | Yes |

#### Output

```
Delegator: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
Stake: 1000.000000000000000000 MON
Rewards: 12.345678901234567890 MON
ARPT (Annual Reward Rate): 5.2%
```

---

### Query Withdrawal Request

Check status of a specific withdrawal request.

```bash
monad-val-manager stake query withdrawal-request \
  --validator-id <VALIDATOR_ID> \
  --delegator-address <ADDRESS> \
  --withdrawal-id <WITHDRAWAL_ID>
```

#### Parameters

| Parameter | Short | Type | Required | Range |
|-----------|-------|------|----------|-------|
| `--validator-id` | `-V` | u64 | Yes | - |
| `--delegator-address` | `-a` | 0x... | Yes | - |
| `--withdrawal-id` | `-w` | u8 | Yes | 0-255 |

---

### Query List Withdrawals

List all withdrawal requests (0-255) for a delegator.

```bash
monad-val-manager stake query list-withdrawals \
  --validator-id <VALIDATOR_ID> \
  --address <ADDRESS>
```

#### Parameters

| Parameter | Short | Type | Required |
|-----------|-------|------|----------|
| `--validator-id` | `-V` | u64 | Yes |
| `--address` | `-a` | 0x... | Yes |

#### Output

```
Withdrawal ID 0:
  Amount: 50.0 MON
  Creation epoch: 365
  Activation epoch: 367
  Status: Ready to withdraw ✓

Withdrawal ID 1:
  Amount: 25.0 MON
  Creation epoch: 366
  Activation epoch: 368
  Status: Not ready (current epoch: 367)
```

---

### Query Delegations

Get all delegations for an address across all validators.

```bash
monad-val-manager stake query delegations --address <ADDRESS>
```

#### Parameters

| Parameter | Short | Type | Required |
|-----------|-------|------|----------|
| `--address` | `-a` | 0x... | Yes |

---

### Query Validator Set

Get the list of validators for a specific set type.

```bash
monad-val-manager stake query validator-set --set-type <SET_TYPE>
```

#### Parameters

| Parameter | Short | Type | Required | Values |
|-----------|-------|------|----------|--------|
| `--set-type` | `-t` | string | No | consensus, execution, snapshot |

**Default:** `consensus`

**Values:**
- `consensus` - Consensus validator set (100 validators)
- `execution` - Execution validator set
- `snapshot` - Snapshot validator set

#### Examples

```bash
# Get consensus validators
monad-val-manager stake query validator-set --set-type consensus

# Get execution validators
monad-val-manager stake query validator-set -t execution

# Snapshot set
monad-val-manager stake query validator-set -t snapshot
```

---

### Query Delegators

Get all delegators for a specific validator.

```bash
monad-val-manager stake query delegators --validator-id <VALIDATOR_ID>
```

---

### Query Proposer

Get the current block proposer information.

```bash
monad-val-manager stake query proposer
```

#### Output

```
Current Proposer:
  Validator ID: 42
  SECP Pubkey: 0xabc123...
```

---

### Query Estimate Gas

Estimate gas for a transaction.

```bash
monad-val-manager stake query estimate-gas \
  --from <FROM_ADDRESS> \
  --to <TO_ADDRESS> \
  --data <DATA> \
  [--value <VALUE>]
```

#### Parameters

| Parameter | Short | Type | Required | Default |
|-----------|-------|------|----------|---------|
| `--from` | `-f` | 0x... | Yes | - |
| `--to` | `-t` | 0x... | Yes | - |
| `--data` | `-d` | 0x... | Yes | - |
| `--value` | `-v` | 0x... | No | 0x0 |

---

### Query Transaction

Get transaction details and receipt.

```bash
monad-val-manager stake query tx --hash <TX_HASH>
```

#### Parameters

| Parameter | Type | Required |
|-----------|------|----------|
| `--hash` | 0x... | Yes |

---

## Transfer Commands

### Transfer Native MON

Transfer native MON tokens to another address.

```bash
monad-val-manager transfer \
  --address <ADDRESS> \
  --amount <AMOUNT> \
  [--dry-run] \
  [--yes]
```

#### Parameters

| Parameter | Short | Type | Required | Description |
|-----------|-------|------|----------|-------------|
| `--address` | | 0x... | Yes | Recipient address (42 chars) |
| `--amount` | `-a` | decimal | Yes | Amount to transfer (MON) |
| `--dry-run` | | flag | No | Preview without sending |
| `--yes` | `-y` | flag | No | Skip confirmation |

#### Examples

```bash
# Preview transfer
monad-val-manager transfer --address 0x742d... --amount 100 --dry-run

# Execute transfer
monad-val-manager transfer --address 0x742d... --amount 100

# Auto-confirm
monad-val-manager transfer --address 0x742d... --amount 50 -y

# Small test amount
monad-val-manager transfer --address 0x742d... --amount 0.001
```

---

## Diagnostic Commands

### Doctor

Run 16 diagnostic checks on your node.

```bash
monad-val-manager doctor
```

#### Checks Performed

| Check | Description |
|-------|-------------|
| RPC Connectivity | Can reach the RPC endpoint |
| Chain ID | Chain ID matches network |
| Block Progress | Node is producing blocks |
| Sync Status | Node is synced |
| Peer Count | Has connected peers |
| Metrics | Metrics endpoint is accessible |
| Config | Configuration is valid |
| Balance | Account has balance |
| And 8 more... | - |

#### Output

```
Running diagnostics...

✓ RPC Connectivity
✓ Chain ID (143)
✓ Block Progress (1,234,567)
✓ Sync Status (synced)
✓ Peer Count (25)
✓ Metrics Endpoint
✓ Configuration Valid
✓ Account Balance (1,000 MON)
✓ Staking Contract Access
✓ Validator Set Access
✓ Gas Estimation
✓ Transaction Signing
✓ Nonce Retrieval
✓ Epoch Data
✓ Consensus Info
✓ Proposer Data

All checks passed! ✓
```

---

## Configuration Management

### Config Show

Display current configuration.

```bash
monad-val-manager config-show
```

#### Output

```
# Monad Validator Manager Configuration

[network]
type = "mainnet"
chain_id = 143
rpc_url = "http://localhost:8080"

[staking]
contract_address = "0x0000000000000000000000000000000000001000"
gas_limit = 1000000
max_fee_per_gas = 500000000000
priority_fee_per_gas = 1000000000

[signer]
type = "ledger"  # or "local"

[advanced]
request_timeout = 30000
poll_interval = 1000
```

### Init (Reset Configuration)

Recreate configuration from scratch.

```bash
monad-val-manager init
```

This will:
1. Backup existing config to `config.toml.bak`
2. Prompt for new configuration
3. Create new config and .env files

---

## Usage Patterns & Best Practices

### 1. Always Dry-Run First

```bash
# Preview
monad-val-manager stake delegate --validator-id 1 --amount 100 --dry-run

# Then execute
monad-val-manager stake delegate --validator-id 1 --amount 100
```

### 2. Test with Small Amounts

```bash
# Test with < 1 MON on testnet
monad-val-manager -n testnet stake delegate --validator-id 1 --amount 0.01
```

### 3. Verify Validator ID

```bash
# See available validators
monad-val-manager stake query validator-set --set-type consensus

# Check validator info
monad-val-manager stake query validator --id 1
```

### 4. Check Balance First

```bash
# Account balance
monad-val-manager balance

# Delegation status
monad-val-manager stake query delegator --validator-id 1 --address 0x742d...
```

### 5. Use Ledger for Production

Always use hardware wallet for mainnet operations involving significant amounts.

---

## Network Configuration

### Networks

| Network | Chain ID | RPC Default |
|---------|----------|-------------|
| mainnet | 143 | http://localhost:8080 |
| testnet | 10143 | http://localhost:8080 |

### Switching Networks

```bash
# Use testnet for a single command
monad-val-manager --network testnet status

# Or set globally in config.toml
[network]
type = "testnet"
chain_id = 10143
```

---

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `insufficient balance` | Not enough MON for transaction + gas | Fund your address |
| `insufficient funds for gas` | Balance can't cover gas fee | Add more MON for gas |
| `invalid validator id` | Validator doesn't exist | Check validator ID with `validator-set` |
| `nonce too low` | Another transaction pending | Wait for pending transaction |
| `connection refused` | RPC endpoint unavailable | Check node is running |
| `timeout` | Request took too long | Check network, increase timeout |

### Debug Mode

Use verbose flags for debugging:

```bash
# Verbose output
monad-val-manager -vv status

# Very verbose (trace)
monad-val-manager -vvv stake delegate --validator-id 1 --amount 100
```

---

## Gas Parameters

### Default Gas Settings

| Operation | Gas Limit |
|-----------|-----------|
| Standard staking | 1,000,000 |
| Add validator | 2,000,000 |

### EIP-1559 Fees

| Parameter | Value |
|-----------|-------|
| Max fee per gas | 500 gwei |
| Priority fee | 1 gwei |

---

## Get Help

### Built-in Help

```bash
# General help
monad-val-manager --help

# Command help
monad-val-manager stake --help

# Subcommand help
monad-val-manager stake delegate --help

# Query subcommand help
monad-val-manager stake query --help
```

### Additional Resources

- [README.md](../README.md) - General project information
- [validator-onboarding.md](./validator-onboarding.md) - Validator registration guide
- [command-reference.md](./command-reference.md) - Quick command reference
- [GitHub Issues](https://github.com/MictoNode/monad-val-manager/issues) - Report bugs

---

*Last Updated: 2026-03-12 (Session 92)*
