# Development Tools

This directory contains utility scripts for Caret development.

## Scripts

| Script | Description |
|--------|-------------|
| `fmt.sh` | Format all code |
| `check.sh` | Quick format/lint/test check |
| `watch.sh` | Watch for changes and run tests |
| `flamegraph.sh` | Generate performance flamegraph |

## Usage

### Format Code

```bash
./tools/dev/fmt.sh
```

### Quick Check

```bash
./tools/dev/check.sh
```

### Watch Mode

```bash
./tools/dev/watch.sh
```

### Generate Flamegraph

```bash
./tools/dev/flamegraph.sh caret
```

## Requirements

- `cargo-watch`: Install with `cargo install cargo-watch`
- `cargo-flamegraph`: Install with `cargo install flamegraph`
- `llvm-tools`: Install with `rustup component add llvm-tools-preview`
- `flamegraph`: Install from https://github.com/brendangregg/FlameGraph
