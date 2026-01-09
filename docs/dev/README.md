# Caret Developer Guide

Welcome to the Caret developer documentation. This section covers contributing to and extending Caret.

## Table of Contents

1. [Setup](setup.md) - Development environment setup
2. [Coding Style](coding-style.md) - Code style guidelines
3. [Testing](testing.md) - Testing guidelines
4. [Debugging](debugging.md) - Debugging techniques
5. [Release Process](release-process.md) - How to make a release
6. [Crate Overview](crate-overview.md) - Crate organization

## Quick Start

### Prerequisites

- Rust 1.75+
- Git
- A code editor (VS Code recommended)

### First Time Setup

```bash
# Clone repository
git clone https://github.com/staticpayload/caret-eternity.git
cd caret-eternity

# Install development tools
cargo install cargo-watch
cargo install cargo-edit

# Run tests
cargo test --workspace

# Build
cargo build --workspace
```

### Development Workflow

```bash
# Watch for changes and run tests
cargo watch -x test

# Watch for changes and check
cargo watch -x 'check --workspace'

# Format code
cargo fmt

# Lint
cargo clippy --workspace -- -D warnings
```

## Crate Development

### Adding a New Crate

1. Create crate directory: `crates/caret_newcrate/`
2. Add to `Cargo.toml` workspace members
3. Implement crate functionality
4. Add tests
5. Update documentation

### Crate Dependencies

Add dependencies to crate's `Cargo.toml`:

```toml
[dependencies]
caret_core = { path = "../caret_core" }

[dev-dependencies]
caret_testkit = { path = "../caret_testkit" }
```

## Testing Strategy

### Unit Tests

Located in each module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Test code
    }
}
```

### Integration Tests

Located in `tests/` directory:

```rust
// tests/integration_test.rs
use caret_core::Graph;

#[test]
fn test_graph_execution() {
    // Integration test
}
```

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p caret_core

# Specific test
cargo test test_graph_execution

# With output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored
```

## Code Review Process

1. Create feature branch
2. Make changes with tests
3. Ensure all tests pass
4. Submit pull request
5. Address review feedback
6. Merge when approved

## Performance Guidelines

### Benchmarking

Use criterion for benchmarks:

```rust
// benches/benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_process(c: &mut Criterion) {
    c.bench_function("process", |b| {
        b.iter(|| process(black_box(data)))
    });
}

criterion_group!(benches, bench_process);
criterion_main!(benches);
```

### Profiling

```bash
# CPU profiling
cargo flamegraph --bin caret

# Memory profiling
valgrind --tool=massif target/release/caret

# Heap profiling
cargo install heaptrack
heaptrack target/release/caret
```

## Documentation

### Code Documentation

```rust
/// Brief description.
///
/// Longer description with examples.
///
/// # Examples
///
/// ```
/// use caret_core::Graph;
///
/// let graph = Graph::new();
/// ```
///
/// # Errors
///
/// Returns an error if...
///
/// # Panics
///
/// Panics if...
pub fn function_name() -> Result<()> {
    // Implementation
}
```

### Building Docs

```bash
# Build documentation
cargo doc --workspace --no-deps --open

# Build with private items
cargo doc --workspace --document-private-items
```

## Continuous Integration

CI runs on:
- Pull requests
- Push to main branch
- Tags for releases

CI checks:
- Formatting (`cargo fmt --check`)
- Linting (`cargo clippy`)
- Tests (`cargo test --workspace`)
- Documentation build

## Resources

- [Contributing](../contributing.md)
- [Architecture](../architecture.md)
- [ADR Index](../../adr/)
