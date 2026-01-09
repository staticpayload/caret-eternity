#!/bin/bash
# Release build script for Caret

set -e

VERSION="${1:-$(grep '^version' Cargo.toml | head -1 | awk '{print $3}' | tr -d '"')}"
BUILD_DIR="target/release"

echo "Building Caret v$VERSION for release..."

# Build for current platform
cargo build --release --bins

# Run tests
echo "Running tests..."
cargo test --workspace

# Run clippy
echo "Running clippy..."
cargo clippy --workspace -- -D warnings

# Check formatting
echo "Checking formatting..."
cargo fmt --check

# Create release directory
RELEASE_DIR="release-$VERSION"
mkdir -p "$RELEASE_DIR"

# Copy binaries
if [ -f "$BUILD_DIR/caret" ]; then
    cp "$BUILD_DIR/caret" "$RELEASE_DIR/"
    echo "Built: $RELEASE_DIR/caret"
fi

# Generate checksums
cd "$RELEASE_DIR"
shasum -a 256 * > SHA256SUMS
cd ..

echo "Release artifacts prepared in $RELEASE_DIR"
echo "Version: $VERSION"
