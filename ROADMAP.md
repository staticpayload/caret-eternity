# Caret Roadmap

## Vision

Caret aims to be the premier runtime for streaming dataflow graphs, combining the performance of native code with the ergonomics of modern tooling.

## Milestones

### Milestone 1: Repo skeleton and governance
- [x] Repository structure
- [ ] Governance documents
- [ ] Initial CI/CD

### Milestone 2: Core data types and error model
- [ ] Packet type definitions
- [ ] Error hierarchy
- [ ] Metadata system

### Milestone 3: Buffer pools and bounded queues
- [ ] Slab allocator
- [ ] Reference counted buffers
- [ ] Bounded queue with overflow policies

### Milestone 4: Graph representation and node trait
- [ ] Node trait definition
- [ ] Graph topology
- [ ] Port connections

### Milestone 5: Scheduler and runtime executor
- [ ] Push mode with backpressure
- [ ] Pull mode support
- [ ] Scheduling policies

### Milestone 6: Minimal IO nodes and toy codecs
- [ ] File source/sink
- [ ] Memory source/sink
- [ ] Basic codec interface

### Milestone 7: DSL v0 with validate and run
- [ ] Grammar definition
- [ ] Parser implementation
- [ ] Semantic analysis
- [ ] Runtime compiler

### Milestone 8: CLI tool suite v0
- [ ] `caret run`
- [ ] `caret validate`
- [ ] `caret graph export`
- [ ] `caret bench`

### Milestone 9: Metrics and tracing v0
- [ ] Structured logging
- [ ] Metrics export
- [ ] Trace spans

### Milestone 10: Inspector service API
- [ ] HTTP API
- [ ] WebSocket streaming
- [ ] Runtime introspection

### Milestone 11: Reactive UI primitives
- [ ] Signal type
- [ ] Computed values
- [ ] Effect system
- [ ] Component model

### Milestone 12: Inspector app v0
- [ ] Graph visualization
- [ ] Node statistics
- [ ] Real-time updates

### Milestone 13: Plugin system v0 in process
- [ ] Plugin manifest
- [ ] Dynamic loading
- [ ] ABI definition

### Milestone 14: Record and replay v0
- [ ] Event capture
- [ ] Replay executor
- [ ] Format specification

### Milestone 15: Bench suite and perf tuning
- [ ] Core benchmarks
- [ ] Profiling integration
- [ ] Performance targets

### Milestone 16: Bindings v0
- [ ] Python bindings
- [ ] Node.js bindings

### Milestone 17: Hardening, fuzzing, docs expansion
- [ ] Fuzz targets
- [ ] Property tests
- [ ] Complete documentation

### Milestone 18: Stable API surface and semver policy
- [ ] API review
- [ ] Versioning policy
- [ ] Stability guarantees

### Milestone 19: Example gallery and tutorials
- [ ] Example pipelines
- [ ] Tutorial content
- [ ] Video demos

### Milestone 20: Release candidate
- [ ] 1.0.0 release
- [ ] Packaging and distribution
- [ ] Launch preparation

## Timeline

This is an open-ended project. Progress continues indefinitely after the initial 20 milestones.
