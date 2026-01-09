#!/bin/bash
# Generate documentation

set -e

OUTPUT_DIR="${1:-target/doc}"

echo "=== Generating Documentation ==="

# Build documentation
cargo doc --workspace --no-deps --document-private-items

# Copy to output directory if specified
if [ "$OUTPUT_DIR" != "target/doc" ]; then
    mkdir -p "$OUTPUT_DIR"
    cp -r target/doc/* "$OUTPUT_DIR/"
fi

# Open in browser if on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    open target/doc/index.html 2>/dev/null || true
fi

echo "=== Documentation Generated ==="
echo "Location: target/doc/"
