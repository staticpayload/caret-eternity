# Contributing to Caret

Thank you for your interest in contributing to Caret! This document provides guidelines for contributing.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Workflow](#development-workflow)
4. [Coding Standards](#coding-standards)
5. [Testing Guidelines](#testing-guidelines)
6. [Documentation](#documentation)
7. [Submitting Changes](#submitting-changes)

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please:

- Be respectful and constructive
- Welcome newcomers and help them learn
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Git
- A code editor (VS Code recommended)

### First Time Setup

```bash
# Clone the repository
git clone https://github.com/staticpayload/caret-eternity.git
cd caret-eternity

# Install development tools
cargo install cargo-watch
cargo install cargo-edit

# Run tests to verify setup
cargo test --workspace

# Run the example
cargo run --bin caret -- --help
```

See [dev/setup.md](dev/setup.md) for detailed setup instructions.

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

Branch naming:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test additions or changes
- `perf/` - Performance improvements

### 2. Make Your Changes

- Write clear, concise code
- Follow the coding standards
- Add tests for new functionality
- Update documentation

### 3. Test Your Changes

```bash
# Run all tests
cargo test --workspace

# Run tests with coverage
cargo tarpaulin --workspace

# Run clippy
cargo clippy --workspace -- -D warnings

# Check formatting
cargo fmt -- --check
```

### 4. Commit Your Changes

Use clear commit messages:

```
<type>: <subject>

<body>

<footer>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Example:
```
feat: add support for window operators

This adds tumbling, sliding, and session window operators
for stream aggregation.

Closes #123
```

### 5. Create a Pull Request

- Push your branch to GitHub
- Create a pull request with a clear description
- Link related issues
- Request review from maintainers

## Coding Standards

### Rust Style

Follow standard Rust conventions:
- Use `cargo fmt` for formatting
- Pass `cargo clippy` with no warnings
- Prefer idiomatic Rust patterns

### Naming Conventions

- **Modules**: `snake_case`
- **Types**: `PascalCase`
- **Functions**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Acronyms**: capitalize each letter (e.g., `TcpTransport` not `TCPTransport`)

### Documentation

All public items must have documentation:

```rust
/// Processes a single packet through the node.
///
/// # Arguments
///
/// * `packet` - The packet to process
///
/// # Returns
///
/// * `Ok(Packet)` - Successfully processed packet
/// * `Err(Error)` - Processing failed
///
/// # Examples
///
/// ```
/// use caret_core::Packet;
///
/// let result = node.process(packet)?;
/// ```
pub fn process(&mut self, packet: Packet) -> Result<Packet> {
    // ...
}
```

### Error Handling

- Use `Result<T, Error>` for fallible operations
- Use `thiserror` for error definitions
- Provide context for errors
- Avoid panicking in library code

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid port configuration: {0}")]
    InvalidPort(String),
}
```

## Testing Guidelines

### Unit Tests

- Write tests alongside code in the same module
- Test both success and failure cases
- Use descriptive test names

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_single_packet_returns_success() {
        // Arrange
        let mut node = TestNode::new();
        let packet = Packet::new(Value::Int(42));

        // Act
        let result = node.process(packet);

        // Assert
        assert!(result.is_ok());
    }
}
```

### Integration Tests

- Place in `tests/` directory of each crate
- Test component interactions
- Use realistic data

### Performance Tests

- Use criterion for benchmarks
- Include in `benches/` directory
- Document expected performance

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_process(c: &mut Criterion) {
    let mut node = TestNode::new();

    c.bench_function("process_packet", |b| {
        b.iter(|| {
            node.process(black_box(Packet::default()))
        })
    });
}

criterion_group!(benches, benchmark_process);
criterion_main!(benches);
```

## Documentation

### Code Documentation

- Document all public APIs
- Include examples for complex APIs
- Keep documentation in sync with code

### User Documentation

- Update user guides for user-facing changes
- Add examples for new features
- Update CHANGELOG for releases

### Architecture Documentation

- Create ADRs for significant decisions
- Update architecture docs for structural changes
- Document protocols and formats

## Submitting Changes

### Before Submitting

- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code is formatted
- [ ] Documentation updated
- [ ] Commit messages are clear
- [ ] PR description is complete

### Pull Request Checklist

- [ ] Title follows conventional commit format
- [ ] Description includes:
  - What changed
  - Why it changed
  - How to test
  - Related issues
- [ ] Tests added/updated
- [ ] Docs updated
- [ ] Only one logical change per PR

### Review Process

1. **Automated Checks**: CI must pass
2. **Code Review**: At least one maintainer approval
3. **Testing**: Additional testing if requested
4. **Merge**: Squash and merge to main

## Getting Help

- **Documentation**: Start with [docs/](./)
- **Issues**: Search [GitHub Issues](https://github.com/staticpayload/caret-eternity/issues)
- **Discussions**: Use [GitHub Discussions](https://github.com/staticpayload/caret-eternity/discussions)
- **Chat**: (Coming soon) Discord/Slack

## Recognizing Contributors

All contributors are recognized in:
- Release notes
- CONTRIBUTORS file
- GitHub contribution graph

Thank you for contributing to Caret!
