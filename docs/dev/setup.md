# Development Setup

This guide covers setting up a Caret development environment.

## Prerequisites

### Required

- **Rust**: 1.75 or later
- **Git**: For version control
- **A code editor**: VS Code recommended

### Optional

- **cargo-watch**: For continuous testing
- **cargo-edit**: For dependency management
- **protobuf-compiler**: For protocol development

## Installing Rust

### Using Rustup (Recommended)

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Configure
source $HOME/.cargo/env

# Verify
rustc --version
cargo --version
```

### Specific Components

```bash
# Add rustfmt
rustup component add rustfmt

# Add clippy
rustup component add clippy

# Add rust-src (for IDE support)
rustup component add rust-src
```

## Cloning the Repository

```bash
# Clone repository
git clone https://github.com/staticpayload/caret-eternity.git
cd caret-eternity

# Verify
git remote -v
```

## Building

### Debug Build

```bash
# Build all workspace members
cargo build --workspace

# Build specific crate
cargo build -p caret_core

# Build with examples
cargo build --workspace --examples
```

### Release Build

```bash
# Optimized build
cargo build --workspace --release

# With specific optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --workspace --release
```

## Development Tools

### cargo-watch

Watch for changes and run commands:

```bash
cargo install cargo-watch

# Usage
cargo watch -x test
cargo watch -x "check --workspace"
cargo watch -x "run --example example_name"
```

### cargo-edit

Manage dependencies from command line:

```bash
cargo install cargo-edit

# Add dependency
cargo add serde

# Add dev dependency
cargo add --dev criterion

# Remove dependency
cargo rm serde
```

### cargo-expand

Expand macros:

```bash
cargo install cargo-expand

# Usage
cargo expand --bin caret
```

## IDE Setup

### VS Code

Install extensions:

- **rust-analyzer**: Rust language server
- **CodeLLDB**: Debugging support
- **Even Better TOML**: TOML support
- **Error Lens**: Inline error display

#### Settings

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.server.extraEnv": {
    "CARGO_TARGET_DIR": "target/rust-analyzer"
  }
}
```

#### Tasks

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "cargo test",
      "type": "shell",
      "command": "cargo test --workspace",
      "group": {
        "kind": "test",
        "isDefault": true
      }
    },
    {
      "label": "cargo clippy",
      "type": "shell",
      "command": "cargo clippy --workspace -- -D warnings"
    },
    {
      "label": "cargo fmt",
      "type": "shell",
      "command": "cargo fmt"
    }
  ]
}
```

### IntelliJ IDEA

1. Install the "Rust" plugin
2. Enable "External Linter" in settings
3. Choose "clippy" as the linter
4. Enable "Run cargo check on the fly"

### Neovim/Vim

Using vim-plug:

```vim
Plug 'rust-lang/rust.vim'
Plug 'simrat39/rust-tools.nvim'
Plug 'neovim/nvim-lspconfig'

lua << EOF
require('rust-tools').setup({})
require('lspconfig').rust_analyzer.setup({
  settings = {
    ['rust-analyzer'] = {
      checkOnSave = {
        command = "clippy"
      }
    }
  }
})
EOF
```

## Running Tests

### All Tests

```bash
cargo test --workspace
```

### Specific Crate

```bash
cargo test -p caret_core
```

### Specific Test

```bash
cargo test test_name
```

### With Output

```bash
cargo test -- --nocapture
```

### Watch Mode

```bash
cargo watch -x test
```

## Formatting and Linting

### Format Code

```bash
# Check formatting
cargo fmt --check

# Format code
cargo fmt

# Format all packages
cargo fmt --all
```

### Lint

```bash
# Check with clippy
cargo clippy --workspace

# Fix automatically
cargo clippy --workspace --fix
```

## Debugging

### Using LLDB

```bash
# Install lldb
brew install lldb  # macOS
apt install lldb   # Linux

# Debug
cargo build
lldb target/debug/caret
(lldb) run
(lldb) bt
```

### Using GDB

```bash
# Install gdb
apt install gdb

# Build with debug symbols
cargo build

# Debug
gdb target/debug/caret
(gdb) run
(gdb) backtrace
```

### VS Code Debugging

Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug caret",
      "cargo": {
        "args": [
          "build",
          "--bin=caret"
        ],
        "filter": {
          "name": "caret",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

## Cross-Compilation

### Targets

```bash
# Add target
rustup target add x86_64-unknown-linux-musl
rustup target add aarch64-unknown-linux-gnu
rustup target add wasm32-wasi
```

### Build for Target

```bash
# Build for specific target
cargo build --target x86_64-unknown-linux-musl

# Cross-compile with linker config
mkdir -p .cargo
cat > .cargo/config.toml << EOF
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"
EOF
```

## Troubleshooting

### Build Failures

**Error**: "Linking with cc failed"

**Solution**: Install C compiler:
```bash
# macOS
xcode-select --install

# Ubuntu/Debian
apt install build-essential

# Fedora
dnf install gcc
```

### Slow Builds

**Solution**: Use sccache:

```bash
cargo install sccache

# Configure
export RUSTC_WRAPPER=sccache
```

### Test Failures

**Error**: "Test flaky"

**Solution**: Run with retries:
```bash
cargo test -- --test-threads=1
```

### IDE Issues

**rust-analyzer not working**

```bash
# Ensure rust-src is installed
rustup component add rust-src

# Restart rust-analyzer
```

## Next Steps

- [Coding Style](coding-style.md)
- [Testing](testing.md)
- [Debugging](debugging.md)
- [Contributing](../contributing.md)
