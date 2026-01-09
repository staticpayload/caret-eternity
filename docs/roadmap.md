# Caret Roadmap

This document outlines the planned development milestones for Caret.

## Vision

Caret aims to be the most performant and flexible stream processing engine for:

- **Real-time analytics**: Low-latency data processing
- **Edge computing**: Efficient resource usage on constrained devices
- **Desktop automation**: User-friendly automation workflows
- **IoT coordination**: Device coordination and data aggregation

## Version Strategy

Caret follows Semantic Versioning:
- **0.x.x**: Pre-release, breaking changes may occur
- **1.0.0**: Stable API, backwards compatibility guaranteed
- **x.y.z**: Standard semver thereafter

## Current Milestone

### Milestone 22: Distributed Graph Execution (IN PROGRESS)

**Goal**: Production-ready distributed execution

- [x] Graph serialization for transmission
- [x] Graph partitioning strategies
- [x] TCP transport implementation
- [x] mDNS node discovery
- [ ] Worker registration and lifecycle
- [ ] Cross-node packet routing
- [ ] Integration tests for distributed scenarios
- [ ] Performance benchmarks

**Expected**: Q1 2025

## Upcoming Milestones

### Milestone 23: Production Readiness

**Goal**: Ready for production use

- [ ] Comprehensive error handling
- [ ] Graceful shutdown
- [ ] Resource limits and quotas
- [ ] Health check endpoints
- [ ] Metrics export (Prometheus format)
- [ ] Structured logging
- [ ] Configuration file support
- [ ] Signal handling (SIGTERM, SIGINT)

**Expected**: Q1 2025

### Milestone 24: Language Bindings

**Goal**: First-class Python and Node.js support

- [ ] Python bindings via PyO3
- [ ] Node.js bindings via napi-rs
- [ ] Language-specific documentation
- [ ] Example programs in each language
- [ ] Package publishing (PyPI, npm)

**Expected**: Q2 2025

### Milestone 25: Advanced Features

**Goal**: Power-user features

- [ ] Windowing operators (tumble, slide, session)
- [ ] Aggregation functions
- [ ] Join operators
- [ ] Backpressure handling strategies
- [ ] Async node support
- [ ] Custom serialization

**Expected**: Q2 2025

### Milestone 26: Tooling

**Goal**: Developer and user tooling

- [ ] Web-based graph inspector
- [ ] CLI graph debugging tools
- [ ] Performance profiler
- [ ] Graph visualizer
- [ ] Configuration validation
- [ ] Migration tools

**Expected**: Q3 2025

### Milestone 27: Security & Hardening

**Goal**: Production security

- [ ] TLS for network communication
- [ ] Plugin sandboxing (WASM)
- [ ] Authentication/authorization
- [ ] Input validation hardening
- [ ] Security audit
- [ ] Fuzzing integration

**Expected**: Q3 2025

### Milestone 28: Ecosystem

**Goal**: Plugin ecosystem

- [ ] Plugin registry
- [ ] Plugin development tools
- [ ] Official plugin library
  - Database connectors
  - Message queue connectors
  - File format readers
  - Protocol handlers
- [ ] Plugin signing and verification

**Expected**: Q4 2025

## Future Considerations

### Potential Features (Post-1.0)

1. **Query Language**: SQL-like or declarative DSL
2. **State Management**: Checkpointing and recovery
3. **Exactly-Once**: Exactly-once processing semantics
4. **Cloud Native**: Kubernetes integration
5. **Stream Storage**: Persistent stream storage
6. **Machine Learning**: ML pipeline integration
7. **Real-time Dashboard**: Built-in visualization

### Platform Support

Currently supported:
- **Linux**: x86_64, ARM64
- **macOS**: x86_64, ARM64 (Apple Silicon)
- **Windows**: x86_64 (in progress)

Future targets:
- **Embedded**: ARM Cortex-M, RISC-V
- **WASM**: Browser and edge execution
- **Android**: Mobile devices

## Contributing

We welcome contributions! See:
- [contributing.md](contributing.md)
- [Good First Issues](https://github.com/staticpayload/caret-eternity/labels/good%20first%20issue)
- [Help Wanted](https://github.com/staticpayload/caret-eternity/labels/help%20wanted)

## Timeline Summary

| Milestone | Focus | Target |
|-----------|-------|--------|
| 22 | Distributed Execution | Q1 2025 |
| 23 | Production Readiness | Q1 2025 |
| 24 | Language Bindings | Q2 2025 |
| 25 | Advanced Features | Q2 2025 |
| 26 | Tooling | Q3 2025 |
| 27 | Security | Q3 2025 |
| 28 | Ecosystem | Q4 2025 |
| 1.0 | Stable Release | Q4 2025 |

**Note**: Dates are estimates and subject to change based on community feedback and priorities.

## Tracking

Current progress tracked in:
- [STATE.md](../STATE.md) - Development status
- [GitHub Projects](https://github.com/staticpayload/caret-eternity/projects) - Issue tracking
- [GitHub Milestones](https://github.com/staticpayload/caret-eternity/milestones) - Version planning
