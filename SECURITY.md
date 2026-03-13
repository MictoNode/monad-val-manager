# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability, please **DO NOT** open a public GitHub issue.

Instead, send your report privately to: **info@mictonode.com**

Please include:
- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact assessment
- Suggested fix (if applicable)

### Response Timeline

- **Initial response**: Within 48 hours
- **Status update**: Within 7 days
- **Resolution**: As soon as a fix is available

## Scope

The following security issues are in scope:

- Private key leakage or exposure
- RPC endpoint manipulation or injection
- Signature bypass or forgery
- Validator authentication bypass
- Unauthorized access to staking operations
- Transaction replay attacks

### Out of Scope

- Issues requiring physical access to the user's device
- Issues in third-party dependencies (report to upstream projects)
- Best practice recommendations (non-vulnerability)

## Security Best Practices for Users

### Private Key Protection

> **⚠️ WARNING:** Your `.env` file contains your private key. **Never:**
> - Commit it to git (verify it's in `.gitignore`)
> - Share it with anyone
> - Display it in logs or error messages

### Hardware Wallet Recommendation

For production use, we recommend using a Ledger hardware wallet:
- Private keys never leave the device
- Transactions must be confirmed physically on the device
- Protection against malware and key loggers

### Network Security

- Run your Monad node behind a firewall
- Use a VPN for remote node access
- Keep your node software updated
- Monitor your node for suspicious activity
