#!/bin/bash
# Run all Caret examples

set -e

EXAMPLES_DIR="examples"
CARET_BIN="${CARET_BIN:-./target/debug/caret}"

# Build caret first if needed
if [ ! -f "$CARET_BIN" ]; then
    echo "Building caret..."
    cargo build --bin caret
fi

# Find all example graph files
find "$EXAMPLES_DIR" -name "*.json" -type f | while read -r example; do
    echo "Running example: $example"
    timeout 5 "$CARET_BIN" run "$example" || true
    echo "---"
done

echo "All examples completed!"
