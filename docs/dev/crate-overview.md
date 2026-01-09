# Crate Overview

Caret is organized as a Cargo workspace with multiple crates. This document describes each crate's purpose and dependencies.

## Workspace Structure

```
caret-eternity/
├── Cargo.toml              # Workspace root
└── crates/
    ├── caret_core/          # Core types and abstractions
    ├── caret_graph/         # Graph management
    ├── caret_sched/         # Scheduling algorithms
    ├── caret_buffers/       # Queues and buffer pools
    ├── caret_record/        # Event recording
    ├── caret_trace/         # Distributed tracing
    ├── caret_metrics/       # Metrics collection
    ├── caret_io/            # I/O abstractions
    ├── caret_transform/     # Data transformations
    ├── caret_codec/         # Serialization
    ├── caret_distributed/   # Distributed execution
    ├── caret_plugins/       # Plugin system
    ├── caret_ffi/           # Foreign function interface
    ├── caret_bench/         # Benchmarks
    ├── caret_testkit/       # Test utilities
    ├── caret_dsl/           # Domain-specific language
    ├── caret_stream/        # Stream processing
    ├── caret_recovery/      # State recovery
    ├── caret_inspector/     # Debug inspector
    └── caret_cli/           # Command-line interface
```

## Dependency Graph

```
                     ┌──────────────┐
                     │  caret_cli   │
                     └──────┬───────┘
                            │
    ┌─────────────────────────┼─────────────────────────┐
    │                         │                         │
┌───▼────────┐         ┌─────▼──────┐          ┌────▼────────┐
│caret_core  │◄────────│caret_graph │◄────────│caret_sched │
└──────┬─────┘         └─────┬──────┘          └────┬────────┘
       │                     │                      │
       │              ┌──────┴──────┐              │
       │              │             │              │
┌──────▼──────┐ ┌───▼──────┐ ┌────▼────────┐ ┌────▼────────┐
│caret_buffers│ │caret_ffi │ │caret_distributed│ │care_recovery│
└─────────────┘ └──────────┘ └─────────────┘ └─────────────┘
```

## Core Crates

### caret_core

**Purpose**: Fundamental data types and abstractions

**Public Types**:
- `Value` - Universal data container
- `Packet` - Data with metadata
- `NodeId` - Unique node identifier
- `PortId` - Unique port identifier
- `Error` - Error types

**Dependencies**: None (foundational)

**Used by**: All other crates

### caret_graph

**Purpose**: Graph representation and manipulation

**Public Types**:
- `Graph` - Container for nodes and edges
- `NodeInstance` - Runtime node wrapper
- `PortSet` - Port management
- `TopologicalSort` - Topology algorithms

**Dependencies**:
- `caret_core`

**Used by**: caret_cli, caret_distributed, caret_sched

### caret_sched

**Purpose**: Execution scheduling algorithms

**Public Types**:
- `Scheduler` - Scheduler trait
- `PriorityScheduler` - Priority-based scheduling
- `FairScheduler` - Fair round-robin
- `DeadlineScheduler` - Deadline-aware scheduling
- `WorkStealingScheduler` - Work-stealing for multi-threaded

**Dependencies**:
- `caret_core`
- `caret_graph`

**Used by**: caret_cli, caret_distributed

## Buffer and Memory Crates

### caret_buffers

**Purpose**: High-performance queues and buffer pools

**Public Types**:
- `SPSCQueue` - Single-producer single-consumer queue
- `MPSCQueue` - Multi-producer single-consumer queue
- `BufferPool` - Sized buffer pooling

**Dependencies**:
- `caret_core`

**Used by**: caret_graph, caret_distributed

## I/O Crates

### caret_io

**Purpose**: I/O abstractions and sources/sinks

**Public Types**:
- `Source` - Data source trait
- `Sink` - Data sink trait
- `FileSource` - File reading
- `FileSink` - File writing

**Dependencies**:
- `caret_core`

**Used by**: caret_cli

### caret_codec

**Purpose**: Serialization and deserialization

**Public Types**:
- `Encoder` - Encode values to bytes
- `Decoder` - Decode bytes to values
- `JsonEncoder` - JSON encoding
- `BincodeEncoder` - Binary encoding

**Dependencies**:
- `caret_core`

**Used by**: caret_distributed, caret_io

## Distributed Crates

### caret_distributed

**Purpose**: Distributed graph execution

**Public Types**:
- `Transport` - Network abstraction
- `TcpTransport` - TCP implementation
- `Coordinator` - Graph coordinator
- `DistributedExecutor` - Distributed runtime
- `Discovery` - Node discovery

**Dependencies**:
- `caret_core`
- `caret_graph`
- `caret_sched`
- `caret_codec`
- `caret_buffers`

**Used by**: caret_cli

## Extension Crates

### caret_plugins

**Purpose**: Dynamic plugin loading

**Public Types**:
- `Plugin` - Plugin trait
- `PluginManager` - Plugin lifecycle
- `PluginRegistry` - Plugin discovery

**Dependencies**:
- `caret_core`
- `caret_graph`

**Used by**: caret_cli

### caret_ffi

**Purpose**: C ABI for language bindings

**Public Types**:
- `FFIGraph` - C-compatible graph handle
- `FFINode` - C-compatible node handle
- `FFIValue` - C-compatible value

**Dependencies**:
- `caret_core`
- `caret_graph`

**Used by**: bindings/python, bindings/node

## Testing and Benchmarking Crates

### caret_testkit

**Purpose**: Testing utilities and fixtures

**Public Types**:
- `TestNode` - Mock node for testing
- `TestGraph` - Graph test fixtures
- `assert_packets_eq!` - Packet assertion macro

**Dependencies**:
- `caret_core`
- `caret_graph`

**Used by**: All crates (dev-dependency)

### caret_bench

**Purpose**: Performance benchmarks

**Benchmarks**:
- Scheduler performance
- Buffer operations
- Queue operations
- Serialization

**Dependencies**:
- `caret_core`
- `caret_graph`
- `caret_sched`
- `caret_buffers`
- `criterion` (dev)

**Used by**: None (benchmark binary)

## Utility Crates

### caret_transform

**Purpose**: Data transformation utilities

**Public Types**:
- `Transform` - Transform trait
- `MapTransform` - Map transformation
- `FilterTransform` - Filter transformation

**Dependencies**:
- `caret_core`

**Used by**: caret_graph

### caret_metrics

**Purpose**: Metrics collection

**Public Types**:
- `Metric` - Metric trait
- `Counter` - Counter metric
- `Gauge` - Gauge metric
- `Histogram` - Histogram metric

**Dependencies**:
- `caret_core`

**Used by**: caret_cli, caret_distributed

### caret_record

**Purpose**: Event recording and replay

**Public Types**:
- `Recorder` - Event recorder
- `Replay` - Event replayer

**Dependencies**:
- `caret_core`
- `caret_graph`

**Used by**: caret_cli, caret_inspector

### caret_trace

**Purpose**: Distributed tracing

**Public Types**:
- `Tracer` - Distributed tracer
- `Span` - Trace span
- `SpanContext` - Span context

**Dependencies**:
- `caret_core`

**Used by**: caret_cli, caret_distributed

### caret_recovery

**Purpose**: State recovery and checkpointing

**Public Types**:
- `Checkpoint` - Checkpoint data
- `Recovery` - Recovery manager

**Dependencies**:
- `caret_core`
- `caret_graph`
- `caret_record`

**Used by**: caret_cli, caret_distributed

### caret_inspector

**Purpose**: Debug inspection tools

**Public Types**:
- `Inspector` - Graph inspector
- `Snapshot` - Execution snapshot

**Dependencies**:
- `caret_core`
- `caret_graph`
- `caret_record`

**Used by**: caret_cli (inspector subcommand)

### caret_dsl

**Purpose**: Domain-specific language for graphs

**Public Types**:
- `DslParser` - DSL parser
- `DslCompiler` - DSL compiler

**Dependencies**:
- `caret_core`
- `caret_graph`

**Used by**: caret_cli (dsl subcommand)

### caret_stream

**Purpose**: Stream processing primitives

**Public Types**:
- `Stream` - Stream trait
- `StreamExt` - Stream extensions

**Dependencies**:
- `caret_core`

**Used by**: caret_graph, caret_distributed

## Application Crates

### caret_cli

**Purpose**: Command-line interface

**Dependencies**: Most other crates

**Binary**: `caret`

## Adding a New Crate

1. Create directory: `crates/caret_newcrate/`
2. Create `Cargo.toml`:

```toml
[package]
name = "caret_newcrate"
version.workspace = true
edition.workspace = true

[dependencies]
caret_core = { path = "../caret_core" }
```

3. Add to workspace `Cargo.toml`:

```toml
[workspace.members]
    # ...
    "crates/caret_newcrate",
```

4. Implement crate functionality
5. Add tests and documentation

## Crate Guidelines

### Dependencies

- Minimize external dependencies
- Prefer workspace crates
- Use feature flags for optional functionality

### Public API

- Document all public items
- Follow semantic versioning
- Prefer traits over concrete types

### Testing

- Minimum 70% code coverage
- Integration tests for functionality
- Benchmarks for performance-critical code

## Resources

- [Crate Documentation](https://docs.rs/caret-eternity/)
- [Workspace Guide](https://doc.rust-lang.org/cargo/reference/workspaces.html)
