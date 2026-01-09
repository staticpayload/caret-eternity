#!/bin/bash
# CI Build Script
#
# Builds Caret for all targets

set -e

TARGETS="${1:-x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-apple-darwin aarch64-apple-darwin}"

echo "=== Building Caret ==="
echo "Targets: $TARGETS"
echo ""

for target in $TARGETS; do
    echo "Building for $target..."

    # Install target if needed
    rustup target add "$target" 2>/dev/null || true

    # Build
    cargo build --release --bins --target "$target"

    echo "✓ Built for $target"
done

echo "=== Build Complete ==="
