# Default recipe
default:
    @just --list

# Build the project
build:
    cargo build --release

# Run with development settings
dev *ARGS:
    cargo run --bin scarff -- --memory {{ARGS}}

# Run tests
test:
    cargo test --workspace

test_core:
    cargo test -p scarff-core
test_cli:
    cargo test -p scarff-cli
test_adapters:
    cargo test -p scarff-adapters

# Run tests with coverage (requires cargo-tarpaulin)
coverage:
    cargo tarpaulin --workspace --out Html --out Stdout

# Lint and format
lint:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo deny check

# Format code
fmt:
    cargo fmt --all

# Generate shell completions
completions:
    mkdir -p completions
    cargo run --bin scarff -- completions bash > completions/scarff.bash
    cargo run --bin scarff -- completions zsh > completions/_scarff
    cargo run --bin scarff -- completions fish > completions/scarff.fish

# Security audit
audit:
    cargo audit

# Update dependencies
update:
    cargo update
    cargo outdated

# Clean build artifacts
clean:
    cargo clean
    rm -rf completions/

# Install locally for testing
install:
    cargo install --path scarff-cli --force

# CI pipeline simulation
ci: lint test build
    cargo deny check
    cargo audit
