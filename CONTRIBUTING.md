# Contributing to Caret

Thank you for your interest in contributing to Caret!

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Node.js 18+ (for UI development)
- Python 3.11+ (for Python bindings)

### Building

```bash
# Build all crates
cargo build --release

# Run tests
cargo test --workspace

# Run with clippy
cargo clippy --workspace --all-targets
```

## Development Workflow

1. Fork and clone the repository
2. Create a branch for your work
3. Make your changes with tests
4. Ensure all checks pass:
   - `cargo fmt --check`
   - `cargo clippy --workspace`
   - `cargo test --workspace`
5. Submit a pull request

## Code Standards

- Follow Rust API guidelines
- Add doc comments for all public APIs
- Write tests for new functionality
- Update relevant documentation

## Pull Request Process

1. Describe your changes clearly
2. Link related issues
3. Ensure CI passes
4. Request review from maintainers

## Coding Style

- Prefer small, focused functions
- Use explicit types over inference for public APIs
- Document invariants near code that depends on them
- Avoid unsafe Rust unless absolutely necessary

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
