# Caret Architecture

This document describes the high-level architecture of the Caret stream processing engine.

## Overview

Caret is designed as a modular, graph-based stream processing engine. The architecture emphasizes:

- **Performance**: Zero-copy data paths where possible, lock-free queues
- **Extensibility**: Plugin system for custom nodes and transforms
- **Scalability**: Distributed execution across multiple machines
- **Safety**: Memory-safe Rust implementation with minimal unsafe code

## Core Components

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Caret Engine                                   │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐       │
│  │   caret_core     │  │   caret_sched    │  │  caret_graph     │       │
│  │  (Data Types)    │  │  (Scheduling)    │  │  (Graph Mgmt)    │       │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘       │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐       │
│  │  caret_buffers   │  │  caret_record    │  │  caret_metrics   │       │
│  │  (Queues/Pools)  │  │  (Recording)     │  │  (Telemetry)     │       │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘       │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐       │
│  │ caret_plugins    │  │ caret_distributed│  │  caret_ffi       │       │
│  │  (Extensions)    │  │  (Distribution)  │  │  (Language FFI)  │       │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘       │
└─────────────────────────────────────────────────────────────────────────┘
```

## Crate Organization

### Core Crates

#### caret_core
**Purpose**: Fundamental data types and abstractions

**Key Types**:
- `Value`: Universal data container (primitive, string, bytes, list, map)
- `Packet`: Data container with metadata and timestamp
- `PortId`: Unique port identifier
- `NodeId`: Unique node identifier

**Dependencies**: None (foundational crate)

#### caret_graph
**Purpose**: Graph representation and management

**Key Types**:
- `Graph`: Container for nodes and edges
- `NodeInstance`: Runtime node with state
- `PortSet`: Input/output port management
- `TopologicalSort`: Dependency resolution

**Key Operations**:
- Add/remove nodes and edges
- Topological ordering
- Cycle detection
- Dynamic modification

#### caret_sched
**Purpose**: Execution scheduling

**Schedulers**:
- `PriorityScheduler`: Priority-based node selection
- `FairScheduler`: Round-robin fair scheduling
- `DeadlineScheduler`: Deadline-aware scheduling
- `WorkStealingScheduler`: Multi-threaded work stealing

### Execution Crates

#### caret_executor
**Purpose**: Main execution engine (typically embedded in caret CLI)

**Responsibilities**:
- Tick loop management
- Node invocation
- Data routing
- Backpressure handling

### Buffer Management

#### caret_buffers
**Purpose**: High-performance queues and buffer pools

**Components**:
- `SPSCQueue`: Single-producer single-consumer queue
- `MPSCQueue`: Multi-producer single-consumer queue
- `BufferPool`: Sized buffer pooling with size classes

**Performance**:
- Lock-free operations for single-producer/consumer
- Atomic length queries
- O(1) buffer acquisition via size classes

### Distribution

#### caret_distributed
**Purpose**: Distributed graph execution

**Components**:
- `Transport`: Network abstraction (TCP, in-memory)
- `Protocol`: Message framing and types
- `Coordinator`: Graph partitioning and assignment
- `Discovery`: Node discovery (mDNS, static)
- `DistributedExecutor`: Distributed runtime

### Plugin System

#### caret_plugins
**Purpose**: Dynamic plugin loading

**Components**:
- `PluginManager`: Lifecycle management
- `PluginRegistry`: Plugin discovery and registration
- `PluginApi`: Stable FFI interface

### Language Bindings

#### caret_ffi
**Purpose**: C ABI for language bindings

**Exports**:
- Graph manipulation functions
- Execution control
- Value/packet handling

**Bindings**:
- **Python** (`bindings/python`): PyO3-based bindings
- **Node.js** (`bindings/node`): napi-rs bindings

## Data Flow

### Local Execution

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Source    │────▶│  Transform  │────▶│    Sink     │
│   Node      │     │    Node     │     │    Node     │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                  ┌────────▼────────┐
                  │   Scheduler     │
                  │  + Executor     │
                  └─────────────────┘
```

### Distributed Execution

```
┌──────────────────────────────────────────────────────────────┐
│                      Coordinator                              │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────┐ │
│  │ Graph Manager│──│ Partitioner  │──│    Router           │ │
│  └──────────────┘  └──────────────┘  └─────────────────────┘ │
└─────────────────────────┬────────────────────────────────────┘
                          │ TCP
         ┌────────────────┼────────────────┐
         │                │                │
┌────────▼────────┐ ┌────▼─────┐ ┌───────▼────────┐
│  Worker 1       │ │Worker 2   │ │  Worker 3      │
│  Partition A    │ │Partition B│ │  Partition C   │
│  ┌──────────┐   │ │           │ │                │
│  │ Local    │   │ │           │ │                │
│  │Executor  │   │ │           │ │                │
│  └──────────┘   │ │           │ │                │
└─────────────────┘ └───────────┘ └────────────────┘
```

## Memory Architecture

### Value Representation

```
Value (enum)
├── Null
├── Bool(bool)
├── Int(i64)
├── Float(f64)
├── String(Arc<str>)
├── Bytes(Arc<Vec<u8>>)
├── List(Arc<Vec<Value>>)
├── Map(Arc<HashMap<String, Value>>)
└── Timestamp(DateTime<Utc>)
```

### Packet Flow

1. **Source Node**: Creates new packets
2. **Output Port**: Packet queued to edge buffer
3. **Edge Transport**: Lock-free queue or network
4. **Input Port**: Packet dequeued by destination
5. **Node Processing**: Transform/consume packet
6. **Output**: New packets created

## Concurrency Model

### Scheduler-Executor Pattern

```
┌─────────────────────────────────────────────────────────────┐
│                     Main Thread                              │
│  ┌──────────────┐                                           │
│  │  Scheduler   │──▶ Determines which nodes to run          │
│  └──────────────┘                                           │
│         │                                                   │
│         ▼                                                   │
│  ┌──────────────┐                                           │
│  │  Task Queue  │──▶ Ready nodes with available data        │
│  └──────────────┘                                           │
└────────────────────────┬────────────────────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
┌───────▼────────┐ ┌────▼──────┐ ┌───────▼────────┐
│  Worker Thread │ │Worker Thr  │ │ Worker Thread  │
│  Executes Node │ │Executes Nd │ │ Executes Node  │
└────────────────┘ └───────────┘ └────────────────┘
```

## Extension Points

### Custom Nodes

Implement the `Node` trait:

```rust
pub trait Node: Send + Sync {
    fn name(&self) -> &str;

    fn process(&mut self,
        input: &mut PortSet,
        output: &mut PortSet
    ) -> Result<()>;

    fn input_ports(&self) -> Vec<PortDef>;
    fn output_ports(&self) -> Vec<PortDef>;
}
```

### Custom Transforms

Implement transform functions for data pipelines:

```rust
pub trait Transform: Send + Sync {
    fn transform(&self, value: Value) -> Result<Value>;
}
```

### Custom Schedulers

Implement the `Scheduler` trait:

```rust
pub trait Scheduler: Send + Sync {
    fn schedule(&mut self, graph: &Graph) -> Vec<NodeId>;
}
```

## Performance Characteristics

### Scheduling

- Single tick: ~200-400ns for 10-node graphs
- Scaling: Linear with graph complexity
- Optimization: Cached node IDs, atomic length queries

### Buffering

- Queue push/pop: ~8ns per operation
- Buffer pool: ~27ns for acquire+release
- Zero-copy: Arc-based value sharing

### Distribution

- TCP framing: ~5µs per message
- mDNS discovery: Sub-second local discovery
- Partition assignment: O(nodes) complexity

## Security Considerations

### Plugin Isolation

- Plugins run in same process (future: sandbox/WASM)
- FFI boundary validates all inputs
- Plugin failures isolated to plugin scope

### Network Security

- Current: Unencrypted TCP (future: TLS support)
- mDNS: Local network only
- Authentication: Roadmap item

### Resource Limits

- Configurable buffer sizes
- Memory limits per graph
- CPU limits via scheduler priority

## Future Architecture

### Planned Enhancements

1. **WASM Plugins**: Sandboxed plugin execution
2. **TLS**: Encrypted network communication
3. **Streaming**: Continuous query processing
4. **Persistence**: Graph state checkpointing
5. **Hot Reload**: Runtime graph modification

### Roadmap

See [roadmap.md](roadmap.md) for detailed plans.

## References

- [ADR Index](../adr/)
- [Developer Guide](dev/)
- [User Guide](guide/)
