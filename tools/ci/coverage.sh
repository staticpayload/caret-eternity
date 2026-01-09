#!/bin/bash
# CI Coverage Script
#
# Generates code coverage reports

set -e

echo "=== Generating Coverage Report ==="

# Install tarpaulin if needed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# Run coverage
cargo tarpaulin --workspace \
    --out Html \
    --output-dir target/coverage \
    --timeout 300 \
    --exclude-files '*/tests/*' \
    --exclude-files '*/examples/*'

echo "=== Coverage Report Generated ==="
echo "Open: target/coverage/index.html"
