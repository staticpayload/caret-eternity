# ADR 0004: Distributed Graph Execution

**Date:** 2024-01-10

**Status:** Accepted

## Context

As graphs grow larger and data volumes increase, single-machine execution becomes insufficient. We need:

1. **Horizontal Scaling**: Distribute graph execution across multiple machines
2. **Fault Tolerance**: Handle node failures gracefully
3. **Transparent Location**: Graphs execute similarly whether local or distributed
4. **Discovery**: Automatic node discovery and coordination

## Decision

Implement distributed graph execution with:

1. **Coordinator-Worker Model**: One coordinator, multiple workers
2. **Graph Partitioning**: Automatic partition assignment across workers
3. **TCP Transport**: Reliable network communication
4. **mDNS Discovery**: Local network node discovery
5. **Cross-Node Routing**: Packet routing between partitions

### Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      Coordinator Node                        │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────────┐  │
│  │   Graph     │  │   Partition  │  │     Transport      │  │
│  │  Manager    │  │    Router    │  │      (TCP)         │  │
│  └─────────────┘  └──────────────┘  └────────────────────┘  │
└───────────────────────────┬──────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
┌───────▼───────┐   ┌───────▼───────┐   ┌───────▼───────┐
│  Worker Node  │   │  Worker Node  │   │  Worker Node  │
│  Partition 0  │   │  Partition 1  │   │  Partition 2  │
│              │   │              │   │              │
│ ┌───────────┐ │   │ ┌───────────┐ │   │ ┌───────────┐ │
│ │ Local     │ │   │ │ Local     │ │   │ │ Local     │ │
│ │ Executor  │ │   │ │ Executor  │ │   │ │ Executor  │ │
│ └───────────┘ │   │ └───────────┘ │   │ └───────────┘ │
└───────────────┘   └───────────────┘   └───────────────┘
```

### Key Components

1. **Transport Layer**: Abstraction for network communication
   - `TcpTransport`: Production TCP transport
   - `MemoryTransport`: In-memory for testing

2. **Protocol**: Binary message format with:
   - Frame-based encoding
   - Checksums for integrity
   - Message types (Hello, Heartbeat, Packet, Control)

3. **Discovery**:
   - mDNS for local network
   - Static configuration for cloud

4. **Partitioning**:
   - RoundRobin: Even distribution
   - Contiguous: Topology-aware
   - MinimizeCrossEdges: Minimize network traffic
   - Manual: User-specified

## Alternatives Considered

### 1. P2P Mesh Network
- **Pros**: No single point of failure, self-organizing
- **Cons**: Complex coordination, harder to reason about, network overhead

### 2. Centralized Message Queue (Kafka, RabbitMQ)
- **Pros**: Proven scalability, durability
- **Cons**: External dependency, operational complexity, overkill for edge cases

### 3. gRPC / Remote Procedure Call
- **Pros**: Standard protocol, code generation
- **Cons**: Less flexible for streaming, coupling to RPC semantics

## Consequences

### Positive

- Horizontal scaling capability
- Fault tolerance through worker redundancy
- Works on local networks and cloud
- Testable via mock transports

### Negative

- Increased complexity in codebase
- Network latency introduced
- Debugging distributed systems is harder

### Mitigation

- Memory transport for unit tests
- Comprehensive tracing/logging
- Status monitoring dashboards (future)

## Implementation

Located in `caret_distributed` crate:
- `transport/` - Network abstraction
- `protocol/` - Message types and encoding
- `coordinator/` - Coordination logic
- `discovery/` - Node discovery

## References

- [CAP Theorem](https://en.wikipedia.org/wiki/CAP_theorem)
- [Distributed Systems Patterns](https://www.amazon.com/Distributed-Systems-Patterns-Reliable-Scalable-Microservices/dp/1617298642)

## Supersedes

None
