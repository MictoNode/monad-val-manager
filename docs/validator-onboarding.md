# Validator Onboarding Workflow

## Summary

1. **Install and initialize** monad-val-manager according to the [installation instructions](../README.md#installation)
2. **Get a Funded Address** and populate the config with its private key via `monad-val-manager init`
   - **Please use hardware wallet for production environment**
   - **Don't commit your `.env` file accidentally** (it's already in .gitignore)
3. **Make sure the wallet is funded** with enough balance
   - **Minimum stake: 100,000 MON** to register
   - **Sufficient gas** to execute the transactions
4. **Choose between CLI or TUI mode** and execute the `add-validator` workflow
5. **Follow verification steps** to confirm successful registration

---

## Step 1: Initialize Configuration

First run the init command to set up your configuration:

```bash
monad-val-manager init
```

You'll be prompted for:
- **Network**: mainnet or testnet
- **Private Key**: Enter your funded address private key (hidden input)
- **RPC Endpoint**: Default is `http://localhost:8080`

This creates:
- Config file: `~/.config/monad-val-manager/config.toml`
- `.env` file: Contains your private key (never commit this!)

---

## Step 2: Extract Validator Keys

⚠️ **DO NOT USE ANY KEYS IN BACKUP FILES**: Due to changes in keystore versions over upgrades make sure you follow the method below to get your private keys. Use `monad-keystore` to extract private keys and **verify** the derived keys with their respective public keys!

Extract your SECP and BLS keys from keystores using the `monad-keystore` binary:

```bash
source /home/monad/.env
monad-keystore recover --password "$KEYSTORE_PASSWORD" --keystore-path /home/monad/monad-bft/config/id-secp  --key-type secp
monad-keystore recover --password "$KEYSTORE_PASSWORD" --keystore-path /home/monad/monad-bft/config/id-bls  --key-type bls
```

Store the extracted private keys securely. You'll need them for validator registration.

---

## Step 3: CLI Workflow

### Add Validator Command

Use the command below and **fill in the values carefully** before executing. Replace the variables with actual values:

```bash
monad-val-manager stake add-validator \
  --secp-privkey "${SECP_PRIVATE_KEY}" \
  --bls-privkey "${BLS_PRIVATE_KEY}" \
  --auth-address "${AUTH_ADDRESS}" \
  --amount 100000
```

**Parameters:**
- `--secp-privkey`: SECP256k1 private key (64 hex chars, with or without 0x prefix)
- `--bls-privkey`: BLS private key (64 hex chars, with or without 0x prefix)
- `--auth-address`: Address that will control validator operations (can be same as funded address)
- `--amount`: Stake amount in MON (minimum: 100,000)

### Verify Before Confirming

⚠️ **Verify the public keys are matching** before entering `yes` to continue:

```bash
SECP Pubkey: 03bbf692002bda53050f22289d4da8fe0bec8b81a6b0d4f641760....
BLS Pubkey: 985d3f7052ac5ad586592ba1a240b0260b5351a9c3973a471fff79....

Do the derived public keys match? (make sure that the private keys were recovered using monad-keystore) [y/n]: y
```

If the public keys don't match, **STOP** and re-extract your keys using monad-keystore.

### Expected Output

```bash
Validator Created! ID: 1, Delegator: 0xF88....
Tx hash: 0xe11114c8e6dd1dc5e0cde400ce5014dab257....
```

---

## Step 4: TUI Workflow

### Launch TUI

```bash
monad-val-manager
```

### Navigate to Staking Screen

1. Press `2` or navigate to **Staking** screen
2. Press `a` to open **Add Validator** dialog

### Fill in the Required Fields

1. **SECP Private Key**: Enter your SECP256k1 private key (64 hex chars, 0x prefix optional)
2. **BLS Private Key**: Enter your BLS private key (64 hex chars, 0x prefix optional)
3. **Auth Address**: Enter the address that will control validator operations
4. **Amount**: Enter stake amount (minimum: 100,000 MON)

### Verify and Confirm

The TUI will display:
- Derived SECP public key
- Derived BLS public key
- Transaction details

**Verify the public keys match** your validator keys before pressing Enter to confirm.

---

## Step 5: Verify Registration

### 1. Check Transaction Status

Make sure the transaction succeeded:
- `Tx status: 1` = Success
- `Tx status: 0` = Failed (see troubleshooting below)

### 2. Note Your Validator ID

After successful registration, you'll get:
```bash
Validator Created! ID: 1, Delegator: 0xF88....
```

In this example, **1** is your validator ID. Save it for future operations.

### 3. Verify You're in the Validator Set

Check if you're part of the execution validator set:

```bash
monad-val-manager stake query validator-set --set-type execution
```

Look for your SECP pubkey in the output. Your validator ID should be listed.

### 4. Query Validator Information

```bash
monad-val-manager stake query validator --id 1
```

**Expected Output:**

```bash
Validator ID: 1
SECP Pubkey: 03bbf692002bda53050f22289d4da8fe0bec8b81a6b0d4f641760....
Auth Delegator: 0xF88....
Commission: 0.00%
Execution Stake: 100000.000000000000000000 MON
...
```

### 5. Check Node Status

```bash
monad-val-manager status
```

Verify your node is running and synced.

---

## Troubleshooting

### Transaction Failed (Status: 0)

If the transaction failed, get the trace to debug:

#### Get Transaction Data

```bash
curl --location 'http://localhost:8080' \
  --header 'Content-Type: application/json' \
  --data '{
    "jsonrpc":"2.0",
    "method":"eth_getTransactionByHash",
    "params":["0xe57ada...."],
    "id":1
  }'
```

#### Debug with eth_call

Use the transaction data to simulate the call:

```bash
curl --location 'http://localhost:8080' \
  --header 'Content-Type: application/json' \
  --data '{
    "jsonrpc":"2.0",
    "method":"eth_call",
    "params":[{
      "from": "0xf88c...",
      "to": "0x0000000000000000000000000000000000001000",
      "gas": "0x7a120",
      "gasPrice": "0x0",
      "value": "0x0",
      "data": "0xf145204c0000000000...."
    }, "latest"],
    "id":1
  }'
```

Common error responses:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32603,
    "message": "insufficient balance"
  },
  "id": 1
}
```

### Common Issues

| Issue | Solution |
|-------|----------|
| **insufficient balance** | Fund your address with more MON (100k MON + gas) |
| **Invalid SECP key** | Verify key is 64 hex chars, extract again with monad-keystore |
| **Invalid BLS key** | Verify key is 64 hex chars, extract again with monad-keystore |
| **Unauthorized** | Check auth-address is correct and you control it |
| **Already registered** | Check if you already have a validator ID |

### Get Help

If you can't debug from the trace:
- Open a [GitHub Issue](https://github.com/MictoNode/monad-val-manager/issues)

---

## Security Best Practices

1. **Use Hardware Wallet**: For production, always use a Ledger device
2. **Verify Keys**: Always check derived public keys match your validator keys
3. **Testnet First**: Test your workflow on testnet before mainnet
4. **Backup Keys**: Store your validator keys securely (keystore files)
5. **Monitor Node**: Use `monad-val-manager doctor` to check node health
