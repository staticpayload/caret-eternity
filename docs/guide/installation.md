# Installation Guide

This guide covers installing Caret on various platforms.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation Methods](#installation-methods)
3. [Platform-Specific Notes](#platform-specific-notes)
4. [Verification](#verification)
5. [Uninstallation](#uninstallation)

## Prerequisites

### Required

- **Operating System**: Linux, macOS, or Windows
- **Architecture**: x86_64 or ARM64
- **Disk Space**: ~50 MB for binary, more for examples

### Optional

- **Rust Toolchain**: If building from source (1.75+)
- **Git**: For cloning repository

## Installation Methods

### Method 1: Pre-built Binary (Recommended)

#### Linux

```bash
# Download latest release
curl -L https://github.com/staticpayload/caret-eternity/releases/latest/download/caret-linux-x86_64.tar.gz -o caret.tar.gz

# Extract
tar -xzf caret.tar.gz

# Install
sudo mv caret /usr/local/bin/

# Verify
caret --version
```

#### macOS

```bash
# Download latest release
curl -L https://github.com/staticpayload/caret-eternity/releases/latest/download/caret-darwin-x86_64.tar.gz -o caret.tar.gz

# For Apple Silicon
curl -L https://github.com/staticpayload/caret-eternity/releases/latest/download/caret-darwin-aarch64.tar.gz -o caret.tar.gz

# Extract
tar -xzf caret.tar.gz

# Install
sudo mv caret /usr/local/bin/

# Verify
caret --version
```

#### Windows

```powershell
# Download latest release
# Visit: https://github.com/staticpayload/caret-eternity/releases/latest

# Extract and add to PATH
# Place caret.exe in a directory in your PATH

# Verify
caret.exe --version
```

### Method 2: Package Manager

#### Homebrew (macOS/Linux)

```bash
# Add tap (if needed)
brew tap caret-eternity/tap

# Install
brew install caret

# Verify
caret --version
```

#### Cargo (Rust Package Manager)

```bash
# Install from crates.io (when published)
cargo install caret

# Or install from source
cargo install --git https://github.com/staticpayload/caret-eternity.git

# Verify
caret --version
```

### Method 3: Build from Source

#### Prerequisites

- Rust 1.75 or later
- Cargo (comes with Rust)

#### Build

```bash
# Clone repository
git clone https://github.com/staticpayload/caret-eternity.git
cd caret-eternity

# Build release binary
cargo build --release

# Install
# Linux/macOS
sudo cp target/release/caret /usr/local/bin/

# Windows
copy target\release\caret.exe C:\Program Files\Caret\

# Verify
caret --version
```

## Platform-Specific Notes

### Linux

#### Dependencies

No external dependencies required. The binary is statically linked.

#### Permissions

Caret runs as a normal user. No special permissions needed.

#### Network

For distributed execution, ensure ports are open:

```bash
# Allow default port
sudo ufw allow 9234/tcp

# Or configure firewall as needed
```

### macOS

#### Dependencies

No external dependencies. The binary is universally compatible.

#### Gatekeeper

On first run, macOS may prompt about the binary:

```bash
# If prompted, allow in System Preferences > Security & Privacy
# Or remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/caret
```

### Windows

#### Dependencies

- Microsoft Visual C++ Redistributable (usually pre-installed)

#### Antivirus

Some antivirus software may flag the binary. Add an exception if needed.

#### Path

Add Caret installation directory to system PATH:

1. Search for "Environment Variables"
2. Edit "Path" variable
3. Add installation directory

## Verification

After installation, verify everything works:

```bash
# Check version
caret --version

# Display help
caret --help

# Run diagnostics
caret doctor

# List available nodes
caret node list
```

## Configuration

### Default Configuration File

Caret looks for configuration in:

1. `./caret.toml` (current directory)
2. `$HOME/.caret/config.toml` (user directory)
3. `/etc/caret/config.toml` (system-wide)

### Example Configuration

```toml
# caret.toml

[general]
log_level = "info"
worker_threads = 4

[network]
bind_address = "0.0.0.0:9234"
discovery_mode = "mdns"

[performance]
buffer_size = 1024
profile = false

[plugins]
paths = ["/usr/local/lib/caret/plugins", "~/.caret/plugins"]
auto_load = []
```

## Plugins Installation

### Install from File

```bash
caret plugin install path/to/plugin.so
```

### Install from Repository

```bash
# From official registry
caret plugin install caret-plugin-kafka

# From git repository
caret plugin install --git https://github.com/example/caret-plugin.git
```

### List Installed Plugins

```bash
caret plugin list
```

## Upgrading

### Using Package Manager

```bash
# Homebrew
brew upgrade caret

# Cargo
cargo install caret --force
```

### Using Binary

```bash
# Download new release
curl -L https://github.com/staticpayload/caret-eternity/releases/latest/download/caret-linux-x86_64.tar.gz -o caret.tar.gz

# Replace binary
sudo tar -xzf caret.tar.gz -C /usr/local/bin/ caret
```

## Uninstallation

### Remove Binary

```bash
# Linux/macOS
sudo rm /usr/local/bin/caret

# Windows
# Delete caret.exe and installation directory
```

### Remove Configuration

```bash
# Remove config
rm -rf ~/.caret

# Remove system config (if exists)
sudo rm /etc/caret/config.toml
```

### Remove Plugins

```bash
# List plugins
caret plugin list

# Remove each plugin
caret plugin remove <plugin-name>
```

## Troubleshooting

### "caret: command not found"

**Cause**: Binary not in PATH

**Solution**:
```bash
# Check if binary exists
which caret

# If not, add to PATH
export PATH="$PATH:/usr/local/bin"  # Linux/macOS

# Or reinstall
```

### "Permission denied"

**Cause**: Insufficient permissions

**Solution**:
```bash
# Make executable
chmod +x caret

# Install with sudo
sudo mv caret /usr/local/bin/
```

### "Library not found"

**Cause**: Missing dependencies

**Solution**:
- Linux: Install missing libraries via package manager
- macOS: Should not happen with universal binary
- Windows: Install Visual C++ Redistributable

## Next Steps

After installation:

1. Read [Getting Started](getting-started.md)
2. Try [Your First Graph](first-graph.md)
3. Explore [Examples](../../examples/)

## Getting Help

- **Documentation**: [docs/](../)
- **Issues**: [GitHub Issues](https://github.com/staticpayload/caret-eternity/issues)
- **Releases**: [GitHub Releases](https://github.com/staticpayload/caret-eternity/releases)
