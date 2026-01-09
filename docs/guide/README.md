# Caret User Guide

Welcome to the Caret User Guide. This documentation helps you learn how to use Caret effectively.

## Table of Contents

1. [Getting Started](getting-started.md) - Quick start guide
2. [Installation](installation.md) - Installation instructions
3. [Your First Graph](first-graph.md) - Create your first graph
4. [Distributed Execution](distributed.md) - Running distributed graphs
5. [Plugins](plugins.md) - Using and creating plugins
6. [Reference](reference.md) - Complete API reference

## Quick Links

### New Users

Start here: [Getting Started](getting-started.md)

### Graph Examples

See the [examples/](../../examples/) directory for complete examples:
- `simple-graph/` - Basic data flow
- `distributed/` - Multi-node execution
- `custom-node/` - Custom node implementation
- `streaming/` - Continuous data processing

### CLI Reference

See [CLI Reference](../cli.md) for complete command documentation.

## Concepts Overview

### What is a Graph?

A Caret graph represents a data processing pipeline:

```
Input ──▶ [Node A] ──▶ [Node B] ──▶ [Node C] ──▶ Output
```

Each node transforms data as it flows through the graph.

### Key Terms

- **Node**: A processing unit that transforms data
- **Edge**: A connection between nodes
- **Port**: An input or output point on a node
- **Graph**: A collection of nodes and edges
- **Packet**: A unit of data flowing through the graph
- **Scheduler**: Determines which nodes to execute
- **Executor**: Runs the graph

### Graph Representation

Graphs are defined as JSON:

```json
{
  "nodes": [
    {
      "id": "source",
      "type": "source",
      "config": { "interval_ms": 100 }
    },
    {
      "id": "transform",
      "type": "map",
      "config": { "expression": "x * 2" }
    },
    {
      "id": "sink",
      "type": "sink",
      "config": { "path": "output.txt" }
    }
  ],
  "edges": [
    { "from": "source", "to": "transform" },
    { "from": "transform", "to": "sink" }
  ]
}
```

## Common Patterns

### Filter Pattern

```json
{
  "type": "filter",
  "config": { "predicate": "x > 10" }
}
```

### Map Pattern

```json
{
  "type": "map",
  "config": { "expression": "x * 2" }
}
```

### Aggregation Pattern

```json
{
  "type": "aggregate",
  "config": { "window": "tumbling(1s)", "function": "sum" }
}
```

### Branching Pattern

```json
{
  "type": "router",
  "config": {
    "routes": [
      { "predicate": "x > 10", "output": "high" },
      { "predicate": "true", "output": "low" }
    ]
  }
}
```

## Performance Tips

1. **Minimize Edges**: Fewer edges = less overhead
2. **Batch When Possible**: Process multiple values together
3. **Use Appropriate Buffer Sizes**: Larger buffers = more latency, less contention
4. **Profile First**: Use `--profile` to identify bottlenecks
5. **Consider Distribution**: Large graphs benefit from distributed execution

## Getting Help

- **Documentation**: This guide and [API Reference](reference.md)
- **Examples**: [examples/](../../examples/)
- **Issues**: [GitHub Issues](https://github.com/staticpayload/caret-eternity/issues)
- **Discussions**: [GitHub Discussions](https://github.com/staticpayload/caret-eternity/discussions)
