#!/bin/bash
# CI Test Runner
#
# Runs all tests with appropriate settings for CI environment

set -e

export RUST_BACKTRACE=1
export RUST_TEST_THREADS=1

echo "=== Running Caret Test Suite ==="
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo ""

# Format check
echo "=== Checking formatting ==="
cargo fmt --all --check

# Clippy
echo "=== Running Clippy ==="
cargo clippy --workspace --all-targets -- -D warnings

# Unit tests
echo "=== Running Unit Tests ==="
cargo test --workspace --lib

# Integration tests
echo "=== Running Integration Tests ==="
cargo test --workspace --test '*'

# Doc tests
echo "=== Running Doc Tests ==="
cargo test --workspace --doc

# Build documentation
echo "=== Building Documentation ==="
cargo doc --workspace --no-deps

echo "=== All Tests Passed ==="
