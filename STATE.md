# Caret State

**Last updated:** 2025-01-08T23:30:00Z

## Current milestone
Milestone 13: Inspector service API

## Current objective
Implementing inspector service for runtime introspection and debugging

## In progress
- Designing inspector API
- Adding query endpoints for graph state

## Done since last update
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
1. Implement inspector service API - Milestone 13
2. Add record and replay v0 - Milestone 14
3. Performance benchmarking framework - Milestone 15

## Risks
- Plugin system uses unsafe code for dynamic loading - needs audit
- caret_trace has a pre-existing test isolation issue with global state

## Quality gates status
- Build: Passing
- Tests: 270+ tests passing across workspace
- Docs: Core APIs documented
- Lint: Passes (some warnings for missing docs on internal items)
- Format: Passing

## Bench notes
None yet - benchmarks will be added in Milestone 15

## Open ADRs
None yet - ADRs will be created as needed for architectural decisions
