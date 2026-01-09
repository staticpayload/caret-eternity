#!/bin/bash
# Caret Development Setup Script
#
# Sets up development environment for Caret contributors.

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Check prerequisites
log_info "Checking prerequisites..."

if ! command -v rustc &> /dev/null; then
    log_warn "Rust not found. Installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
else
    log_info "Rust found: $(rustc --version)"
fi

if ! command -v git &> /dev/null; then
    log_error "Git not found. Please install Git first."
    exit 1
fi

# Install development tools
log_info "Installing development tools..."

cargo install cargo-watch 2>/dev/null || log_warn "cargo-watch already installed"
cargo install cargo-edit 2>/dev/null || log_warn "cargo-edit already installed"
cargo install cargo-expand 2>/dev/null || log_warn "cargo-expand already installed"

# Install Rust components
log_info "Installing Rust components..."
rustup component add rustfmt clippy rust-src

# Add pre-commit hook (if .git/hooks exists)
if [ -d ".git/hooks" ]; then
    log_info "Setting up pre-commit hook..."
    cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
set -e
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace --no-run
EOF
    chmod +x .git/hooks/pre-commit
    log_info "Pre-commit hook installed"
fi

# Build project
log_info "Building Caret..."
cargo build --workspace

# Run tests
log_info "Running tests..."
cargo test --workspace

log_info "Development setup complete!"
log_info "Run 'cargo watch -x test' to watch for changes and run tests automatically."
