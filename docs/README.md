# Caret Documentation

Welcome to the Caret documentation. Caret is a high-performance stream processing engine designed for real-time data processing, IoT edge computing, and desktop automation.

## Documentation Structure

```
docs/
├── README.md           # This file - overview
├── architecture.md     # System architecture
├── roadmap.md          # Project roadmap
├── contributing.md     # Contribution guidelines
├── cli.md              # CLI reference
├── guide/              # User guides
│   ├── getting-started.md
│   ├── installation.md
│   ├── first-graph.md
│   ├── distributed.md
│   ├── plugins.md
│   └── reference.md
└── dev/                # Developer documentation
    ├── setup.md
    ├── coding-style.md
    ├── testing.md
    ├── debugging.md
    ├── release-process.md
    └── crate-overview.md
```

## Quick Links

- **Getting Started**: See [guide/getting-started.md](guide/getting-started.md)
- **Architecture Overview**: See [architecture.md](architecture.md)
- **Contributing**: See [contributing.md](contributing.md)
- **Developer Setup**: See [dev/setup.md](dev/setup.md)

## What is Caret?

Caret is a graph-based stream processing engine that:

- **Processes data flows** as directed graphs of nodes
- **Executes efficiently** with Rust-powered performance
- **Scales horizontally** across multiple machines
- **Extends dynamically** through plugins
- **Binds to multiple** languages (Python, Node.js, more coming)

## Key Concepts

### Graphs

A Caret graph consists of:
- **Nodes**: Processing units that transform data
- **Edges**: Connections that route data between nodes
- **Ports**: Input and output connection points

### Execution

Caret executes graphs by:
1. Scheduling nodes based on data availability
2. Processing data through connected nodes
3. Managing backpressure and buffering
4. Distributing work across available cores

### Distribution

For large graphs, Caret can:
- Partition graphs across worker nodes
- Route data between partitions
- Discover workers automatically on LAN
- Scale horizontally

## Versioning

Caret follows Semantic Versioning 2.0.0:
- **Major**: Breaking changes
- **Minor**: New features (backwards compatible)
- **Patch**: Bug fixes (backwards compatible)

Current version: See `Cargo.toml` or run `caret --version`.

## License

MIT License - See [LICENSE](../LICENSE) in the repository root.

## Support

- **Issues**: [GitHub Issues](https://github.com/staticpayload/caret-eternity/issues)
- **Discussions**: [GitHub Discussions](https://github.com/staticpayload/caret-eternity/discussions)
- **ADR**: Architecture decisions in [adr/](../adr/)

## Table of Contents

1. [User Guides](guide/) - Learn to use Caret
2. [Developer Docs](dev/) - Contribute to Caret
3. [Architecture](architecture.md) - System design
4. [Roadmap](roadmap.md) - What's planned
