# Debugging Guide

This guide covers debugging techniques for Caret development.

## Table of Contents

1. [Logging](#logging)
2. [Debugging Builds](#debugging-builds)
3. [Common Issues](#common-issues)
4. [Tools](#tools)
5. [Performance Debugging](#performance-debugging)

## Logging

### Log Levels

Caret uses the `tracing` crate for structured logging:

```rust
use tracing::{info, warn, error, debug};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Caret");

    // Different levels
    debug!("Detailed debug info: {:?}", value);
    info!("General information");
    warn!("Warning: something unexpected");
    error!("Error: {}", err);
}
```

### Log Configuration

```rust
use tracing_subscriber::{fmt, EnvFilter};

fn init_logging() {
    fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("caret_core=debug")
                .add_directive("caret_distributed=trace"),
        )
        .init();
}
```

### Environment Variables

```bash
# Set log level
RUST_LOG=caret_core=debug,caret_distributed=info cargo run

# Verbose all crates
RUST_LOG=debug cargo run

# Trace specific module
RUST_LOG=caret_distributed::transport=trace cargo run
```

## Debugging Builds

### Debug Symbols

```bash
# Build with debug info
cargo build

# Build with maximum debug info
cargo build --profile=dev

# Check release with debug info
cargo build --profile=release-with-debug
```

### Opt-level Profiles

Add to `.cargo/config.toml`:

```toml
[profile.release-with-debug]
inherits = "release"
debug = true
opt-level = 2  # Faster builds
```

## LLDB (macOS/Linux)

### Basic Usage

```bash
# Start debugger
lldb target/debug/caret

# Set breakpoint
(lldb) breakpoint set --name process_packet

# Run
(lldb) run

# Continue
(lldb) continue

# Print variable
(lldb) print variable_name

# Backtrace
(lldb) bt

# Step
(lldb) step
(lldb) next

# Finish current frame
(lldb) finish
```

### Conditional Breakpoints

```bash
(lldb) breakpoint set --name process --condition 'packet.id == 42'
```

### LLDB Commands

| Command | Description |
|---------|-------------|
| `b <name>` | Set breakpoint |
| `r` | Run program |
| `c` | Continue |
| `s` | Step into |
| `n` | Step over |
| `p <expr>` | Print expression |
| `bt` | Backtrace |
| `f <n>` | Select frame |

## GDB (Linux)

### Basic Usage

```bash
# Start debugger
gdb target/debug/caret

# Run
(gdb) run

# Breakpoint
(gdb) break process_packet

# Continue
(gdb) continue

# Print
(gdb) print variable

# Backtrace
(gdb) backtrace
```

### GDB Commands

| Command | Description |
|---------|-------------|
| `b <loc>` | Set breakpoint |
| `r` | Run program |
| `c` | Continue |
| `s` | Step |
| `n` | Next |
| `p <expr>` | Print |
| `bt` | Backtrace |
| `info locals` | Show local variables |

## VS Code Debugging

### Launch Configuration

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
        "args": ["build", "--bin=caret"],
        "filter": {
          "name": "caret",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug"
      }
    }
  ]
}
```

### Breakpoints

- Click left margin to toggle breakpoint
- Right-click for conditional breakpoint
- Use Debug tab to manage breakpoints

## Common Issues

### Segmentation Faults

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo run

# For more detail
RUST_BACKTRACE=full cargo run
```

### Memory Issues

```bash
# Use valgrind
valgrind --leak-check=full target/debug/caret

# Use address sanitizer
RUSTFLAGS="-Z sanitizer=address" cargo run
```

### Deadlocks

```bash
# Use thread sanitizer
RUSTFLAGS="-Z sanitizer=thread" cargo run
```

### Stack Overflow

```bash
# Increase stack size
ulimit -s unlimited  # Linux
cargo run
```

## Tools

### cargo-expand

Expand macros to see generated code:

```bash
cargo install cargo-expand
cargo expand --bin caret
```

### cargo-tree

View dependency tree:

```bash
cargo install cargo-tree
cargo tree --duplicates
```

### cargo-udeps

Find unused dependencies:

```bash
cargo install cargo-udeps
cargo +nightly udeps
```

### strace (Linux)

Trace system calls:

```bash
strace -e trace=network cargo run
```

### dtruss (macOS)

Trace system calls:

```bash
sudo dtruss -t network cargo run
```

## Performance Debugging

### Flamegraph

```bash
cargo install flamegraph
cargo flamegraph --bin caret
```

### perf (Linux)

```bash
# Record
perf record -g target/release/caret

# Report
perf report

# Flamegraph
perf script | FlameGraph/flamegraph.pl > flamegraph.svg
```

### heaptrack (Memory Profiling)

```bash
heaptrack target/release/caret
heaptrack_print caret.heaptrack.gz
```

## Debugging Async Code

### Tokio Console

```bash
# Add to Cargo.toml
console-subscriber = "0.1"

# Use in code
use console_subscriber::ConsoleSubscriber;

#[tokio::main]
async fn main() {
    ConsoleSubscriber::new().init();
    // ...
}
```

### Task Dumping

```rust
use tokio::task::spawn;

let handle = spawn(async {
    // Async work
});

// Get task ID
let id = handle.id();
```

## Debugging Distributed Systems

### Coordinator Logs

```bash
# Run coordinator with trace logging
RUST_LOG=caret_distributed=trace caret distributed --mode coordinator
```

### Worker Logs

```bash
# Run worker with detailed logs
RUST_LOG=caret_distributed::worker=debug caret distributed --mode worker
```

### Network Debugging

```bash
# Capture traffic
tcpdump -i any port 9234 -w capture.pcap

# View with Wireshark
wireshark capture.pcap
```

## Test Debugging

### Single Test

```bash
cargo test test_name -- --nocapture
```

### Show Output

```bash
cargo test -- --nocapture
```

### Run Ignored Tests

```bash
cargo test -- --ignored
```

### Test with Logging

```bash
RUST_LOG=debug cargo test
```

## Release Build Debugging

### Debug Info in Release

Add to `Cargo.toml`:

```toml
[profile.release]
debug = 1  # Include line info
```

### Strip Symbols Manually

```bash
# Build with symbols
cargo build --release

# Strip to separate file
objcopy --only-keep-debug target/release/caret target/release/caret.debug
strip target/release/caret
objcopy --add-gnu-debuglink=target/release/caret.debug target/release/caret
```

## Remote Debugging

### GDB Server

```bash
# On target machine
gdbserver :1234 target/release/caret

# On development machine
gdb target/release/caret
(gdb) target remote <target-ip>:1234
```

### Core Dumps

```bash
# Enable core dumps
ulimit -c unlimited

# Run until crash
./target/release/caret

# Analyze core
gdb target/release/caret core
```

## Checklist

Before asking for help:

- [ ] Ran with `RUST_LOG=debug`
- [ ] Checked `cargo clippy`
- [ ] Reproduced in debug build
- [ ] Included backtrace if crash
- [ ] Checked for memory issues with valgrind
- [ ] Verified dependencies are up to date

## Resources

- [Rust Debugging Guide](https://doc.rust-lang.org/cargo/guide/build-tool.html)
- [LLDB Tutorial](https://lldb.llvm.org/use/tutorial.html)
- [Tokio Tracing](https://tokio.rs/tokio/topics/tracing)
