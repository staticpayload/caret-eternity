#!/bin/bash
# Caret Installation Script
#
# Copyright (c) 2025 Caret Contributors
#
# Licensed under the MIT License:
# https://opensource.org/licenses/MIT

set -e

VERSION="${CARET_VERSION:-latest}"
INSTALL_DIR="${CARET_INSTALL_DIR:-/usr/local/bin}"
PLATFORM="$(uname -s)"
ARCH="$(uname -m)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect platform
case "$PLATFORM" in
    Linux)
        PLATFORM="linux"
        ;;
    Darwin)
        PLATFORM="darwin"
        ;;
    *)
        log_error "Unsupported platform: $PLATFORM"
        exit 1
        ;;
esac

# Detect architecture
case "$ARCH" in
    x86_64|amd64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        log_error "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

BINARY_NAME="caret-${PLATFORM}-${ARCH}"
DOWNLOAD_URL="https://github.com/staticpayload/caret-eternity/releases/${VERSION}/download/${BINARY_NAME}.tar.gz"

log_info "Installing Caret ${VERSION} for ${PLATFORM} (${ARCH})..."

# Check if we have write permission to INSTALL_DIR
if [ ! -w "$INSTALL_DIR" ]; then
    log_warn "No write permission to $INSTALL_DIR"
    log_info "Installing to ~/.local/bin instead..."
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

# Download binary
log_info "Downloading from $DOWNLOAD_URL"
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

curl -L "$DOWNLOAD_URL" -o "$TMP_DIR/caret.tar.gz"
tar -xzf "$TMP_DIR/caret.tar.gz" -C "$TMP_DIR"

# Install binary
log_info "Installing to $INSTALL_DIR"
mv "$TMP_DIR/caret" "$INSTALL_DIR/caret"
chmod +x "$INSTALL_DIR/caret"

# Verify installation
if command -v caret &> /dev/null; then
    log_info "Successfully installed Caret!"
    caret --version
else
    log_error "Installation failed. Please add $INSTALL_DIR to your PATH."
    exit 1
fi

log_info "Installation complete!"
