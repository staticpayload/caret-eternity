# ADR 0002: Use Tokio for Async Runtime

**Date:** 2024-01-02

**Status:** Accepted

## Context

Caret requires asynchronous I/O operations for:

1. **Network Communication**: Distributed node coordination, data streaming
2. **Plugin I/O**: Loading and communicating with external plugins
3. **File Operations**: Configuration loading, log writing
4. **Timer Management**: Scheduling, timeouts, heartbeats

Rust's async ecosystem has several runtimes available. We need to choose one for consistent async operations across the codebase.

## Decision

Use [Tokio](https://tokio.rs/) as the async runtime for Caret.

### Rationale

1. **Market Leader**: Tokio is the most widely used async runtime in the Rust ecosystem
2. **Feature Complete**: Provides timers, IO, net, sync primitives, and task scheduling
3. **Ecosystem Compatibility**: Most Rust async libraries are built on Tokio
4. **Performance**: Highly optimized with work-stealing scheduler
5. **Interoperability**: Excellent support for async/await, tracing, and backpressure

### Key Features Used

- **Multi-threaded scheduler**: Work-stealing scheduler for efficient CPU utilization
- **Timers**: Precision timing for scheduling and timeouts
- **Networking**: TCP/UDP sockets for distributed execution
- **Channels**: mpsc and oneshot channels for inter-task communication
- **Mutex/RwLock**: Async-friendly synchronization primitives

## Alternatives Considered

### 1. async-std
- **Pros**: Simpler API, more "batteries included" standard library approach
- **Cons**: Smaller ecosystem, less adoption, fewer compatible crates

### 2. smol
- **Pros**: Minimal, simple, easy to understand
- **Cons**: Less feature-rich, smaller community, not as battle-tested

### 3. raw-futures/executor
- **Pros**: Maximum control, no dependencies
- **Cons**: Would need to build all infrastructure ourselves, maintenance burden

## Consequences

### Positive

- Wide ecosystem compatibility
- Excellent performance characteristics
- Strong community support and documentation
- Built-in tracing and instrumentation support

### Negative

- Tokio version 0.1 vs 1.0 migration was painful (now resolved)
- Some overhead for very simple use cases (negligible for our workload)

### Neutral

- Runtime footprint is acceptable (~1MB additional binary size)
- Compile times increased slightly due to macro usage

## Implementation Notes

- All async functions use `tokio::spawn` for task creation
- Use `tracing` instrument for async span tracking
- Graceful shutdown uses `CancellationToken` pattern

## References

- [Tokio Documentation](https://tokio.rs/)
- [Async Rust Book](https://rust-lang.github.io/async-book/)

## Supersedes

None
