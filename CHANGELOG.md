# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.3] - 2026-11-27

### Security
- Refreshed the dependency lockfile: 280 crates updated within their existing
  semver constraints, clearing 17 RustSec/OSV advisories. Affected paths included
  TLS and HTTP (`rustls` 0.23.46, `rustls-webpki` 0.103.17, `quinn-proto` 0.11.19,
  `h2` 0.4.21, `openssl` 0.10.82), URL parsing (`url` 2.5.9), runtime
  (`tokio` 1.54.1) and error handling (`anyhow` 1.0.105).
- `ratatui-textarea` is pinned to `0.9` instead of `*`; a wildcard let a lockfile
  refresh move it from 0.8 to 0.9 without a manifest change.

### Changed
- Remaining advisories are structural and need an `alloy` 0.6 → 2.x migration:
  `crypto-bigint` 0.5.5 (timing variability, via `elliptic-curve`), `lru` 0.12.5
  (via `alloy-provider` and old `ratatui` releases).

## [1.0.2] - 2026-10-05

### Fixed
- **Ledger transactions were rejected by the node** with "Transaction decoding error":
  `encode_signed` returned the RLP payload without the EIP-2718 type byte, and `LedgerSigner`
  inherited the trait's `sign_transaction_hex`, so hardware-wallet transactions were broadcast
  without the `0x02` prefix. `encode_signed` now returns the full EIP-2718 envelope.

### Added
- `.gitignore` for private key material, build output and logs.
- CI workflow: release build plus the hermetic test suite on every push and pull request.
- Release workflow: publishing a release builds the binary and attaches it together with a
  SHA256 checksum.

### Changed
- `encode_for_signing` documentation clarified: it returns the RLP payload without the type byte,
  and `signing_hash` prepends `0x02` before hashing. Behavior unchanged.
- Test suite: the BUG-007 regression tests are compiled and executed again (they had never run),
  and the assertion-free dry-run test module was replaced with tests that drive the real code.

## [1.0.1] - 2026-03-13

### Changed
- TUI brand name updated
- README documentation updates

## [1.0.0] - 2026-03-12

### Added
- **TUI Dashboard** - 5 screens (Dashboard, Staking, Transfer, Doctor, Help) with Monad brand theme
- **Staking Operations**:
  - delegate - Delegate MON to a validator
  - undelegate - Undelegate MON from a validator
  - withdraw - Withdraw undelegated MON (2-epoch delay)
  - claim-rewards - Claim staking rewards
  - compound-rewards - Re-stake claimed rewards
- **Validator Management**:
  - add-validator - Register new validators (100k MON minimum)
  - change-commission - Change validator commission rates (0-100%)
- **Query Operations**:
  - validator - Query validator details
  - delegator - Query delegator information
  - epoch - Query current epoch info
  - validator-set - Query 100 validators list
  - proposer - Query current proposer
  - delegations - Query all delegations for address
  - delegators - Query all delegators for validator
  - withdrawal-request - Query withdrawal request status
  - list-withdrawals - List all withdrawal requests
  - estimate-gas - Estimate gas for transactions
  - tx - Query transaction details
- **Doctor Diagnostics** - 16 automated checks with actionable recommendations
- **Hardware Wallet Support** - Ledger device integration via `alloy-signer-ledger` (built-in, untested - feedback welcome)
- **Multi-network Support** - Mainnet (chain ID 143) and testnet (chain ID 10143)
- **Transfer** - Native MON token transfers
- **Dry-run Mode** - Preview transactions without sending (all write operations)

[1.0.2]: https://github.com/MictoNode/monad-val-manager/releases/tag/v1.0.2
[1.0.1]: https://github.com/MictoNode/monad-val-manager/releases/tag/v1.0.1
[1.0.0]: https://github.com/MictoNode/monad-val-manager/releases/tag/v1.0.0
