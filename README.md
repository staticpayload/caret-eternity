# Caret

A high performance stream and graph runtime for media, data, and realtime compute, with a reactive UI layer and a plugin ecosystem.

## What Caret Is

Caret is a dataflow runtime that executes directed graphs of nodes with:
- **Backpressure** - Bounded queues prevent unbounded memory growth
- **Timestamps** - Every packet carries temporal context
- **Scheduling policies** - Realtime and batch execution modes
- **Zero copy buffers** - Efficient data sharing where practical

## Three Pillars

### Caret Core
A fast, safe, deterministic dataflow runtime for executing graphs of streaming nodes.

### Caret Tools
A CLI and developer toolchain for building, testing, debugging, profiling, and shipping pipelines.

### Caret UI
A lightweight reactive UI library and inspector app for pipeline dashboards.

## Quick Start

### Install

```bash
cargo install caret
```

### Your First Pipeline

Create a file `pipeline.ct`:

```caret
pipeline "demo" {
  source camera0 type video
  transform scale width 1280 height 720
  sink file path "out.mp4"
}
```

Run it:

```bash
caret run pipeline.ct
```

## Architecture Overview

Caret is organized as a Rust workspace with multiple crates:

- `caret_core` - Core runtime and execution engine
- `caret_graph` - Graph representation and topology
- `caret_buffers` - Buffer pools and memory management
- `caret_sched` - Scheduling policies and executor
- `caret_plugins` - Plugin system and ABI
- `caret_io` - IO nodes and codecs
- `caret_metrics` - Metrics collection
- `caret_trace` - Distributed tracing
- `caret_cli` - Command line interface
- `caret_testkit` - Testing utilities
- `caret_bench` - Benchmark suite
- `caret_ffi` - C ABI for language bindings

## Documentation

- [User Guide](docs/guide/)
- [Developer Guide](docs/dev/)
- [Architecture Decision Records](adr/)
- [API Documentation](https://docs.rs/caret)

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Project Status

Current phase: Early development.

See [ROADMAP.md](ROADMAP.md) for planned milestones.
