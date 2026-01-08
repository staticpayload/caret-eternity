# Caret State

**Last updated:** 2025-01-08T23:00:00Z

## Current milestone
Milestone 12: Transform nodes for common operations

## Current objective
Implementing common transform nodes (filter, map, merge, demux, etc.)

## In progress
- Designing transform node API
- Adding basic filter and map nodes

## Done since last update
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
1. Complete transform nodes - Milestone 12
2. Implement inspector service API - Milestone 13
3. Add record and replay v0 - Milestone 14

## Risks
- Plugin system uses unsafe code for dynamic loading - needs audit
- caret_trace has a pre-existing test isolation issue with global state

## Quality gates status
- Build: Passing
- Tests: 196+ tests passing across workspace
- Docs: Core APIs documented
- Lint: Passes (some warnings for missing docs on internal items)
- Format: Passing

## Bench notes
None yet - benchmarks will be added in Milestone 15

## Open ADRs
None yet - ADRs will be created as needed for architectural decisions
