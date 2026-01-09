#!/bin/bash
# Quick development check

set -e

echo "=== Quick Check ==="

# Check formatting
echo "Checking formatting..."
cargo fmt --all --check

# Run clippy
echo "Running clippy..."
cargo clippy --workspace --all-targets -- -D warnings

# Run tests (no run)
echo "Running tests (compile only)..."
cargo test --workspace --no-run

echo "=== Check Complete ==="
