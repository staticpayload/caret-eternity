# Caret State

**Last updated:** 2025-01-09T02:00:00Z

## Current milestone
Milestone 22: Distributed graph execution with real Caret graphs - IN PROGRESS

## Current objective
Implementing distributed graph execution with real Caret graphs

## Done since last update
### Milestone 22: Distributed graph execution with real Caret graphs (IN PROGRESS)
- Graph serialization for distributed transmission:
  - `SerializableGraph`: Network-transmittable graph representation
  - `SerializableNode`: Node definition with ports and type
  - `SerializableEdge`: Connection between node ports
  - JSON and binary serialization via serde
- Graph partitioning for multi-worker execution:
  - `GraphPartitioner`: Partitions graphs across worker nodes
  - `PartitionStrategy`: RoundRobin, Contiguous, MinimizeCrossEdges, Manual
  - `PartitionAssignment`: Complete partition with cross-node routes
  - `GraphPartition`: Per-worker node and edge assignment
  - `CrossPartitionEdge`: Edge crossing partition boundaries
  - `CrossNodeRoute`: Route for cross-node packet delivery
- Integration with caret_graph:
  - `SerializableGraph::from_caret_graph()`: Convert from caret_graph Graph
  - Preserves nodes, edges, ports, and topological order
  - Port name resolution for proper routing
- DistributedExecutor enhancements:
  - `submit_caret_graph()`: Submit Caret graph for distributed execution
  - `setup_routes_from_partition()`: Configure routing from partition assignment
  - `get_routes()`: Query configured routes
  - Enhanced cross-node packet routing with node:port format
- Protocol message types:
  - `PartitionAssign`: Message type for assigning partitions to workers
  - Worker-side partition deserialization and route setup
- 43 tests passing in caret_distributed (10 new graph_proto/integration tests)

### Milestone 21: Distributed execution support - COMPLETE
- Created `caret_distributed` crate with:
  - Transport layer abstraction (`Transport`, `MemoryTransport` for testing, `TcpTransport` for real networking)
  - Message framing codec (`FrameCodec`, `FrameDecoder`) with CARET magic bytes
  - Protocol messages (`Message`, `MessagePayload`, `MessageType`)
  - Node types (`NodeId`, `NodeInfo`, `NodeState`, `LocalNode`)
  - Discovery service (`Discovery`, `DiscoveryConfig`, `DiscoveryEvent`)
  - Coordinator for distributed execution (`Coordinator`, `ExecutionMode`)
  - DistributedExecutor for runtime execution
  - Error types specific to distributed operations
- TCP transport implementation:
  - Server mode with `TcpTransport::bind()` for accepting connections
  - Client mode with `TcpTransport::connect()` for outbound connections
  - Per-connection read/write tasks using tokio async primitives
  - Proper frame-based message encoding/decoding with checksums
- Distributed executor implementation:
  - `DistributedExecutor`: Main runtime connecting coordinator and transport
  - Graph lifecycle: submit, start, stop operations
  - Worker management with Hello handshake and heartbeat monitoring
  - Packet routing for cross-node data transfer
  - Event loop for transport events and message handling
- Transport configuration with customizable buffers and message sizes
- Node discovery with support for static and multicast modes
- mDNS-based service discovery (`MdnsDiscovery`, `MdnsDiscoveryConfig`)
  - Service type `_caret._tcp.local.` for Caret node discovery
  - Node lifecycle tracking with discovery events
  - Builder pattern for configuration
- Worker registration and assignment logic
- Graph execution state management
- Integration tests covering all major components (16 tests passing)
- 43 total tests passing in caret_distributed

### Milestone 20: Performance optimization and profiling - COMPLETE
- Executor tick performance optimizations:
  - Added cached node IDs to avoid Vec allocation in every tick
  - Added `node_ids_slice()` for zero-copy access to node IDs
  - Added cached input port names in NodeInstance
  - Added `has_inputs` flag to avoid empty checks
  - Optimized tick_once to use cached data and reduce allocations
- Port set optimizations:
  - Added `input_count()` and `output_count()` for cheap size checks
- Queue optimizations (caret_buffers):
  - Added atomic length for lock-free `len()`, `is_empty()`, `is_full()` queries
  - Reduced queue_len from 2ns to ~0ns (lock-free)
  - Optimized push/pop with atomic operations for concurrent access
- Buffer pool optimizations (caret_buffers):
  - Implemented size classes (power-of-2 buckets) for O(1) buffer lookup
  - Replaced linear search (O(n)) with direct bucket access
  - buffer_acquire: 35ns → 29ns (17% faster)
  - buffer_acquire_release: 80ns → 27ns (66% faster)
- Performance benchmark binaries:
  - executor_perf: Executor tick benchmarks
  - buffer_perf: Buffer pool and queue benchmarks

### Milestone 19: Stream processing primitives - COMPLETE
- Created `caret_stream` crate with stream processing primitives
- 35 tests passing in caret_stream

### Milestone 18: Dynamic graph modification - COMPLETE
- Dynamic graph modification in caret_graph
- 48 tests passing in caret_graph

### Previously completed (Milestones 1-17)
- Scheduler with PriorityScheduler, FairScheduler, DeadlineScheduler, WorkStealingScheduler
- Graph representation with topological sorting
- Core data types and error model
- Complete governance documentation and repo structure

## Next objectives
1. Milestone 22: Implement distributed graph execution with real Caret graphs
2. Implement actual mDNS functionality (currently using simplified implementation)
3. Add TLS support for secure transport
4. Add distributed system benchmarks

## Risks
- Plugin system uses unsafe code for dynamic loading - needs audit
- caret_trace has a pre-existing test isolation issue with global state (tests pass when run individually)

## Quality gates status
- Build: Passing
- Tests: 43 tests passing in caret_distributed (10 new graph_proto/integration tests)
- Docs: Core APIs documented
- Lint: Passes (some warnings for missing docs on internal items)
- Format: Passing

## Bench notes
### Recent benchmark results

**executor_perf:**
- tick_10_nodes (10k iterations): ~4ms total, 407ns per tick
- tick_100_nodes (10k iterations): ~24ms total, 2.37µs per tick
- tick_with_ports_10_nodes (10k iterations): ~1.9ms total, 186ns per tick

**buffer_perf (after optimizations):**
- buffer_acquire (100k iterations): ~2.9ms total, 29ns per op
- buffer_acquire_release (100k iterations): ~2.7ms total, 27ns per op
- queue_push_pop (100k iterations): ~800µs total, 8ns per op
- queue_len (1M iterations): ~349µs total, ~0ns per op (lock-free)
- queue_contention (100k iterations): ~822µs total, 8ns per op

Run benchmarks with:
- `cargo run --release --bin executor_perf`
- `cargo run --release --bin buffer_perf`

## Open ADRs
None yet - ADRs will be created as needed for architectural decisions
