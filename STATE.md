# Caret State

**Last updated:** 2025-01-08T19:00:00Z

## Current milestone
Milestone 8: CLI tool commands

## Current objective
Implementing CLI tool commands (run, validate, graph, bench)

## In progress
- Designing CLI command structure
- Implementing run command for executing pipelines
- Implementing validate command for checking DSL files
- Implementing graph command for visualizing pipelines
- Implementing bench command for performance testing

## Done since last update
### Milestone 7: DSL v0 grammar and parser - COMPLETE
- Token definitions and lexer for the Caret DSL
- AST nodes for statements, node declarations, connections, and pipelines
- Recursive descent parser for the DSL
- Error types with source location tracking
- 17 unit tests for lexer, parser, and AST
- DSL syntax support for:
  - Node declarations (source/sink/process)
  - Named properties and positional arguments
  - Pipeline blocks with scoped statements
  - Connection/link statements
  - Import/export declarations

### Milestone 6: Minimal IO nodes - COMPLETE
- SourceNode, SinkNode, ProcessNode traits for IO abstractions
- SourceNodeAdapter, SinkNodeAdapter, ProcessNodeAdapter for NodeProcessor integration
- FileSource for reading files in chunks with loop support
- FileSink for buffered file writing with append mode
- MemorySource for in-memory data with configurable chunking
- MemorySink for collecting data with optional max size limit
- 17 unit tests for all IO nodes

### Milestone 5: Scheduler and runtime executor - COMPLETE
- Runtime executor with tick-based execution model
- NodeProcessor trait for custom node implementations
- Port connections with bounded queues
- Scheduling policies (Realtime, Batch)
- Runtime state management (Running, Paused, Stopped, Completed, Error)
- ProcessingContext for execution tracking
- PassthroughNode helper for simple implementations
- CountingNode helper for testing
- Three-node pipeline integration test

### Previously completed (Milestones 1-4)
- Created all governance documentation files
- Set up complete directory structure
- Initialized Rust workspace with 13 crates
- Implemented core error type with typed error hierarchy
- Implemented all data packet types (Bytes, Audio, Video, Tensor, Event, Control)
- Implemented timestamp and duration abstractions
- Implemented metadata system with typed values
- Implemented buffer pool with slab allocation support
- Implemented bounded queue with overflow policies
- Implemented graph representation with nodes and ports
- Implemented topology management with cycle detection
- Implemented topological sorting for DAG validation
- Added CI workflows for formatting, linting, and testing

## Next objectives
1. Complete CLI tool commands (run, validate, graph, bench) - Milestone 8
2. Add metrics and tracing v0 - Milestone 9
3. Add codec system for serialization - Milestone 10
4. Implement plugin loading system - Milestone 11

## Risks
- None identified yet

## Quality gates status
- Build: Passing
- Tests: All 130 tests passing across 7 crates
- Docs: Core APIs documented
- Lint: Passes (some warnings for missing docs on internal items)
- Format: Passing

## Bench notes
None yet - benchmarks will be added in Milestone 15

## Open ADRs
None yet - ADRs will be created as needed for architectural decisions
