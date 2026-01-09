# Testing Guidelines

This document covers testing practices for the Caret project.

## Philosophy

1. **Test public behavior, not implementation**
2. **Tests should be fast and reliable**
3. **Unit tests for isolated logic**
4. **Integration tests for component interaction**
5. **Property-based tests for data transformation**

## Test Organization

### Directory Structure

```
crate/
├── src/
│   ├── lib.rs
│   └── module.rs
├── tests/
│   ├── integration_test.rs
│   └── common/
│       └── mod.rs
├── benches/
│   └── benchmark.rs
└── examples/
    └── example.rs
```

### Unit Tests

Located in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = TestNode::new();
        assert_eq!(node.name(), "test");
    }

    #[test]
    fn test_process_single_value() {
        let mut node = TestNode::new();
        let result = node.process(Value::Int(42)).unwrap();
        assert_eq!(result, Value::Int(84));
    }
}
```

### Integration Tests

Located in `tests/` directory:

```rust
// tests/graph_execution.rs
use caret_core::{Graph, Node, Packet};

#[test]
fn test_simple_graph_execution() {
    let mut graph = Graph::new();
    graph.add_node("source", Box::new(SourceNode::new()));
    graph.add_node("transform", Box::new(MapNode::new(|x| x * 2)));

    let result = graph.execute().unwrap();
    assert!(result.is_success());
}
```

### Common Test Utilities

Create reusable test utilities:

```rust
// tests/common/mod.rs
use caret_core::{Graph, Node};

pub fn create_test_graph() -> Graph {
    let mut graph = Graph::new();
    // Setup common test graph
    graph
}

pub struct TestNode {
    counter: AtomicUsize,
}

impl TestNode {
    pub fn new() -> Self {
        Self { counter: AtomicUsize::new(0) }
    }

    pub fn process_count(&self) -> usize {
        self.counter.load(Ordering::SeqCst)
    }
}
```

## Test Naming

### Descriptive Names

```rust
// Good - describes the scenario and expected outcome
#[test]
fn test_process_with_valid_input_returns_success() {
    // ...
}

#[test]
fn test_process_with_invalid_type_returns_type_error() {
    // ...
}

// Bad - vague
#[test]
fn test_process() {
    // ...
}
```

### Convention: `test_<subject>_<condition>_<expected>`

```
test_<unit>_<scenario>_<expected outcome>
```

Examples:
- `test_buffer_push_returns_new_length`
- `test_graph_with_cycle_returns_error`
- `test_node_process_with_multiple_packets`

## Test Structure

### AAA Pattern (Arrange, Act, Assert)

```rust
#[test]
fn test_map_node_doubles_value() {
    // Arrange
    let mut node = MapNode::new(|x: i64| x * 2);
    let input = Packet::new(Value::Int(21));

    // Act
    let result = node.process(input).unwrap();

    // Assert
    assert_eq!(result.value(), &Value::Int(42));
}
```

### Table-Driven Tests

```rust
#[test]
fn test_value_coercion() {
    let cases = vec![
        (Value::Int(42), "42"),
        (Value::Float(3.14), "3.14"),
        (Value::String("hello".into()), "hello"),
    ];

    for (input, expected) in cases {
        let result = input.to_string();
        assert_eq!(result, expected);
    }
}
```

## Assertions

### Prefer Specific Assertions

```rust
// Good - specific
assert_eq!(result, expected);
assert!(result.is_ok());
assert!(result.contains(&item));

// Less clear
assert!(result == expected);
```

### Provide Context in Assertions

```rust
// Good
assert_eq!(
    result.len(), expected.len(),
    "Length mismatch: got {}, expected {}",
    result.len(), expected.len()
);

// Also good
assert!(
    result.is_ok(),
    "Expected Ok but got Err: {:?}",
    result
);
```

## Async Testing

### Using Tokio

```rust
#[tokio::test]
async fn test_async_node_execution() {
    let node = AsyncNode::new().await;
    let result = node.process().await.unwrap();
    assert!(result.is_success());
}
```

### Test Timeout

```rust
#[tokio::test]
#[timeout(1000)]  // Requires tokio::test timeout feature
async fn test_with_timeout() {
    // Test that times out after 1 second
}
```

## Property-Based Testing

### Using proptest

Add to dev-dependencies:

```toml
[dev-dependencies]
proptest = "1.0"
```

Write property tests:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_roundtrip(val in any::<i64>()) {
        let encoded = encode(val);
        let decoded = decode(&encoded).unwrap();
        prop_assert_eq!(val, decoded);
    }
}
```

## Mocking

### Trait-Based Mocks

```rust
#[cfg(test)]
mockall::automock! {
    trait DataSource {
        fn fetch(&self) -> Result<Data>;
    }
}

#[test]
fn test_with_mock_data_source() {
    let mut mock = MockDataSource::new();
    mock.expect_fetch()
        .returning(Ok(Data::default()));

    let result = process_with_source(&mock);
    assert!(result.is_ok());
}
```

## Performance Testing

### Criterion Benchmarks

```rust
// benches/node_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_node_process(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_process");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut node = TestNode::new();
            let data = create_test_data(size);
            b.iter(|| {
                node.process(black_box(data.clone()))
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_node_process);
criterion_main!(benches);
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench node_bench

# Save baseline
cargo bench -- --save-baseline main

# Compare with baseline
cargo bench -- --baseline main
```

## Testing Best Practices

### 1. Test Isolation

Each test should be independent:

```rust
#[test]
fn test_one() {
    // Don't rely on state from other tests
}

#[test]
fn test_two() {
    // Clean setup, no shared state
}
```

### 2. Use Test Fixtures

```rust
fn setup_test_graph() -> Graph {
    let mut graph = Graph::new();
    // Common setup
    graph
}

fn teardown_test_graph(graph: Graph) {
    // Common cleanup
}

#[test]
fn test_with_fixture() {
    let graph = setup_test_graph();
    // Test
}
```

### 3. Parametrized Tests

```rust
#[test]
fn test_multiple_configs() {
    let configs = vec![
        Config::fast(),
        Config::balanced(),
        Config::thorough(),
    ];

    for config in configs {
        let result = process_with_config(&config);
        assert!(result.is_ok());
    }
}
```

### 4. Error Cases

Test error handling:

```rust
#[test]
fn test_process_with_empty_input_returns_error() {
    let result = process(&[]);
    assert!(matches!(result, Err(Error::EmptyInput)));
}
```

## Continuous Integration

### Test Matrix

```yaml
# .github/workflows/test.yml
test:
  strategy:
    matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta, nightly]
  steps:
    - uses: actions/checkout@v2
    - uses: actions-rust-lang/setup-rust-toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
    - run: cargo test --workspace
    - run: cargo clippy --workspace -- -D warnings
```

## Coverage

### Using tarpaulin

```bash
# Install
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --workspace --out Html

# View report
open tarpaulin-report.html
```

### Target Coverage

Aim for:
- **Core crates**: 80%+ coverage
- **Utility crates**: 70%+ coverage
- **Examples**: No requirement (documentation only)

## Flaky Tests

### Identifying Flaky Tests

```bash
# Run multiple times
for i in {1..10}; do
    cargo test
done
```

### Fixing Flaky Tests

Common causes:
- Time-dependent tests
- Shared state between tests
- Resource cleanup issues
- Async race conditions

Fixes:
- Use deterministic time sources
- Isolate test state
- Proper cleanup in `Drop` implementation
- Use barriers/flags for async coordination

## Test Checklist

Before committing, ensure:

- [ ] All tests pass locally
- [ ] New functionality has tests
- [ ] Edge cases are covered
- [ ] Error paths are tested
- [ ] Tests are properly isolated
- [ ] Test names are descriptive
- [ ] No `unwrap()` or `expect()` in test setup

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)
- [Criterion User Guide](https://bheisler.github.io/criterion.rs/book/index.html)
