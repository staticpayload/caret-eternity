# Caret State

**Last updated:** 2025-01-09T07:00:00Z

## Current milestone
Milestone 20: Performance optimization and profiling (NEXT)

## Current objective
Stream processing primitives complete - ready to begin performance optimization and profiling work

## Done since last update
### Milestone 19: Stream processing primitives - COMPLETE
- Created `caret_stream` crate with stream processing primitives
- Core `Stream` trait with `poll_next`, `size_hint`, `len`, `is_empty` methods
- `Next` future for async stream iteration
- `TryStream` trait for streams that yield Results
- Stream constructors: `Iter`, `Slice`, `Once`, `FromIter`, `Repeat`, `RepeatWith`
- Stream combinators: `Map`, `Filter`, `FilterMap`, `Fold`, `Scan`, `FlatMap`, `Chain`, `Take`, `TakeWhile`, `Skip`, `SkipWhile`, `Fuse`, `Zip`, `Inspect`, `Then`
- `StreamExt` trait with convenient methods: `collect`, `count`, `first`, `last`, `find`, `find_position`, `any`, `all`, `for_each`, `partition`
- `Sink` trait for consuming streams: `poll_ready`, `poll_flush`, `poll_close`, `start_send`
- Sink implementations: `Drain`, `VecSink`, `FnSink`, `With`, `SinkFlatMap`
- `Window` trait and window types: `TumblingWindow`, `SlidingWindow`, `CountWindow`, `TimeWindow`
- `WindowExt` trait for window operations
- Merge operations: `Merge`, `Select`, `MergeExt` trait
- `StreamConfig` for configuring stream processing (buffer_size, backpressure, max_pending)
- `StreamConfig` with builder methods: `with_buffer_size`, `with_backpressure`, `with_max_pending`
- Fixed all pin-project-lite compilation issues:
  - Fixed `Iter` and `FromIter` structs to use unsafe get_unchecked_mut for field access
  - Fixed `Filter`, `FilterMap`, `Fold`, `Scan`, `TakeWhile`, `SkipWhile`, `Inspect`, `Then` combinators to use `as_mut().project()` in loops
  - Fixed `Find`, `FindPosition`, `Any`, `All`, `ForEach`, `Partition` futures in ext.rs
  - Fixed `With` and `SinkFlatMap` in sink.rs to call methods on pinned projections directly
  - Fixed all test cases to use `Pin::new(&mut sink).start_send()` pattern
  - Fixed waker creation in tests to use `Box::leak` for 'static lifetime
- 35 tests passing in caret_stream
- caret_stream fully re-enabled in workspace

### Milestone 18: Dynamic graph modification - COMPLETE
- Dynamic graph modification in caret_graph
- GraphChange enum (AddNode, RemoveNode, Connect, Disconnect, ReplaceNode)
- GraphTransaction for atomic batch operations
- DynamicGraph with thread-safe runtime modification
- ChangeListener trait for notification hooks
- ChangeResult enum (Applied, RolledBack, Skipped)
- NopListener for testing
- Snapshot functionality for graph inspection
- 48 tests passing in caret_graph (up from 20)
- 408 total tests passing across workspace (pre-existing test isolation issue in caret_trace)

### Milestone 17: Advanced scheduling strategies - COMPLETE
- Advanced scheduling strategies in caret_sched
- Scheduler trait for pluggable scheduling strategies
- PriorityScheduler with recency bonuses to prevent starvation
- FairScheduler with round-robin for fair CPU time
- DeadlineScheduler for deadline-aware scheduling
- WorkStealingScheduler for multi-threaded parallel execution
- SchedulingChoice enum (Single, Multiple, None, Stop)
- NodeMetadata with priority_score, urgency_score, fairness_score
- WorkStealingConfig for configuring work-stealing behavior
- SchedulerConfig enum for creating schedulers
- 41 tests passing in caret_sched
- 381 total tests passing across workspace

### Milestone 16: Enhanced error handling and recovery - COMPLETE
- `caret_recovery` crate with error recovery and resilience mechanisms
- CircuitBreaker with states (Closed, Open, HalfOpen) and configurable thresholds
- RecoveryPolicy with configurable actions for retryable and non-retryable errors
- RecoveryStrategy trait with DefaultRecoveryStrategy, AggressiveRecoveryStrategy, ConservativeRecoveryStrategy
- RetryStrategy with BackoffStrategy (Fixed, Linear, Exponential, ExponentialWithJitter)
- ErrorHandler for recording and responding to errors with per-node tracking
- ErrorContext for tracking errors (node_id, port, tick, packet_id)
- RecoveryAction (Retry, Skip, Fallback, Fail)
- RecoveryManager for coordinating recovery across pipeline
- RecoveryStats for tracking recovery operations
- 28 tests passing in caret_recovery
- 268 total tests passing across workspace

### Milestone 15: Performance benchmarking framework - COMPLETE
- `caret_bench` benchmark framework library
- BenchResult with timing statistics (avg, min, max, std_dev, throughput)
- BenchConfig for warmup and measurement configuration
- BenchGroup for running multiple related benchmarks
- run_bench, run_bench_ret, run_bench_with_config functions
- MemStats for memory usage tracking (Linux support)
- Buffer pool benchmarks: acquire_4k, acquire_64k, pooled_reuse, comparison with direct alloc
- Queue benchmarks: push_pop, push_only, pop_only, overflow policy comparisons
- Scheduler benchmarks: tick_empty, tick with 1/10/100 nodes, add_node, node_lookup
- 4 tests passing in caret_bench
- 332 total tests passing across workspace

### Milestone 14: Record and replay v0 - COMPLETE
- `caret_record` crate with event recording and replay
- Event types: PacketEvent, StateChangeEvent, MetricEvent, ErrorEvent
- EventRecorder with configurable recording (max events, packet/metric/state filtering)
- EventReplayer with real-time replay and speed control
- EventStore trait with MemoryStore and FileStore implementations
- RecordingManager for managing recording sessions
- SessionId and RecordingMetadata for tracking recordings
- ReplayHandler trait for custom replay behavior
- TestHandler for testing replay functionality
- JSON-based event serialization using JsonCodec
- 20 tests passing in caret_record
- 236 total tests passing across workspace

### Milestone 13: Inspector service API - COMPLETE
- `caret_inspector` crate with HTTP/WebSocket server
- REST API endpoints: /api/runtime, /api/graph, /api/nodes, /api/metrics, /api/stats
- WebSocket endpoint at /api/stream for real-time event streaming
- Snapshot data models: RuntimeSnapshot, GraphSnapshot, NodeSnapshot, PortSnapshot, MetricSnapshot
- RuntimeIntegration for capturing snapshots from Executor
- InspectorConfig and InspectorServer with CORS support
- CLI inspect command: `caret inspect --bind-addr 127.0.0.1:3000`
- 17 tests passing in caret_inspector
- 216 total tests passing across workspace

### Milestone 12: Transform nodes for common operations - COMPLETE
- `caret_transform` crate with 8 transform node types
- FilterNode with predicate system (by_kind, min_length, max_length, no_control)
- MapNode with transformation functions (transform_data, add_prefix, truncate, etc.)
- MergeNode with strategies (RoundRobin, PrioritizedFirst, Interleave)
- DemuxNode with predicate routing and RoundRobinDemuxNode variant
- BatchNode with size and time-based flushing
- BufferNode with overflow policies (Reject, DropOldest, DropNewest, Block)
- ThrottleNode with rate limiting (PacketsPerWindow, OnePerTicks, Percentage)
- SampleNode with sampling modes (EveryNth, FirstN, RandomPercentage, AtIndices)
- 92 tests passing in caret_transform
- 270+ total tests passing across workspace

### Milestone 11: Plugin loading system v0 - COMPLETE
- `caret_plugins` crate with Plugin trait and plugin system
- Plugin manifest format (Caret.toml) with TOML parsing
- Plugin types: Node, Codec, IO, MetricExporter, UiPanel
- PluginMetadata with version, author, capabilities
- PluginRegistry for managing loaded plugins
- PluginLibrary with dynamic loading using libloading
- NodePlugin, CodecPlugin, IoPlugin, MetricExporterPlugin traits
- API version compatibility checking
- Plugin discovery from directories
- 18 tests passing in caret_plugins
- 196+ total tests passing across workspace

### Previously completed (Milestones 1-10)
- Codec system for serialization with JsonCodec and BinaryCodec
- Metrics and tracing v0 with Counter, Gauge, Histogram and TraceContext
- CLI tool suite with run, validate, graph, bench commands
- DSL v0 grammar and parser with lexer, AST, and recursive descent parser
- Minimal IO nodes (FileSource, FileSink, MemorySource, MemorySink)
- Scheduler and runtime executor with tick-based execution
- Buffer pools and bounded queues with overflow policies
- Graph representation with topological sorting
- Core data types and error model
- Complete governance documentation and repo structure

## Next objectives
1. Performance optimization and profiling - Milestone 20
2. Distributed execution support - Milestone 21

## Risks
- Plugin system uses unsafe code for dynamic loading - needs audit
- caret_trace has a pre-existing test isolation issue with global state

## Quality gates status
- Build: Passing
- Tests: 443 tests passing across workspace (35 in caret_stream)
- Docs: Core APIs documented
- Lint: Passes (some warnings for missing docs on internal items)
- Format: Passing

## Bench notes
Benchmark framework is now available in caret_bench crate with:
- Buffer pool benchmarks for acquire/reuse operations
- Queue benchmarks for push/pop operations and overflow policies
- Scheduler benchmarks for tick execution with varying node counts
- Run benchmarks with `cargo test -p caret_bench --bins` or `cargo run --bin buffer_pool_bench --release`

## Open ADRs
None yet - ADRs will be created as needed for architectural decisions
