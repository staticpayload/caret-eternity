#!/bin/bash
# Format all code in the workspace

set -e

echo "=== Formatting Code ==="

# Format Rust code
cargo fmt --all

echo "=== Code Formatted ==="
