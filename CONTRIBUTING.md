# Contributing

Thank you for your interest in contributing to Monad Validator Manager!

## Build

```bash
# Clone the repository
git clone https://github.com/MictoNode/monad-val-manager.git
cd monad-val-manager

# Build in release mode
cargo build --release
```

## Test

```bash
# Run all tests (unit, integration, doc)
cargo test --all-features

# Run tests without output
cargo test --quiet

# Run tests with output
cargo test -- --nocapture
```

## Code Style

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter (0 warnings allowed)
cargo clippy -- -D warnings
```

## Before Opening a PR

1. **Ensure tests pass**: `cargo test --all-features`
2. **No clippy warnings**: `cargo clippy -- -D warnings`
3. **Format code**: `cargo fmt`
4. **Update documentation**: If applicable, update docs/cli-reference.md

## Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linter
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## Coding Standards

- Follow Rust idioms and best practices
- Add doc comments to public functions
- Keep functions focused and concise
- Use meaningful variable and function names
- Handle errors appropriately with `Result` types

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE-MIT) or [Apache License, Version 2.0](LICENSE-APACHE).
