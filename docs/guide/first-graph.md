# Your First Graph

This guide walks you through creating and running your first Caret graph.

## What We'll Build

We'll create a data processing pipeline that:

1. Generates random numbers
2. Filters for values greater than 50
3. Multiplies the values by 2
4. Prints the results

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│ Generator│───▶│ Filter   │───▶│ Multiply │───▶│  Print   │
│  Random  │    │  > 50    │    │   x 2    │    │ Console  │
└──────────┘    └──────────┘    └──────────┘    └──────────┘
```

## Graph Definition

Create a file called `my_first_graph.json`:

```json
{
  "nodes": [
    {
      "id": "generator",
      "type": "generator",
      "config": {
        "interval_ms": 500,
        "min": 1,
        "max": 100
      }
    },
    {
      "id": "filter",
      "type": "filter",
      "config": {
        "predicate": "value > 50"
      }
    },
    {
      "id": "multiply",
      "type": "map",
      "config": {
        "expression": "value * 2"
      }
    },
    {
      "id": "print",
      "type": "console",
      "config": {
        "prefix": "Result: "
      }
    }
  ],
  "edges": [
    {
      "from": "generator",
      "to": "filter"
    },
    {
      "from": "filter",
      "to": "multiply"
    },
    {
      "from": "multiply",
      "to": "print"
    }
  ]
}
```

## Understanding the Graph

### Nodes

Each node has three parts:

1. **id**: Unique identifier for the node
2. **type**: The node type (determines behavior)
3. **config**: Configuration options for the node

### Generator Node

```json
{
  "id": "generator",
  "type": "generator",
  "config": {
    "interval_ms": 500,
    "min": 1,
    "max": 100
  }
}
```

This node generates random numbers between 1 and 100 every 500ms.

### Filter Node

```json
{
  "id": "filter",
  "type": "filter",
  "config": {
    "predicate": "value > 50"
  }
}
```

This node only passes through values greater than 50.

### Map Node

```json
{
  "id": "multiply",
  "type": "map",
  "config": {
    "expression": "value * 2"
  }
}
```

This node multiplies each value by 2.

### Console Node

```json
{
  "id": "print",
  "type": "console",
  "config": {
    "prefix": "Result: "
  }
}
```

This node prints values to the console with a prefix.

### Edges

Edges connect nodes together:

```json
{
  "from": "generator",
  "to": "filter"
}
```

This creates a connection from the generator's output to the filter's input.

## Running the Graph

### Basic Execution

```bash
caret run my_first_graph.json
```

You should see output like:

```
Result: 102
Result: 142
Result: 118
Result: 186
```

Press Ctrl+C to stop.

### Run with Duration

Run for a specific duration:

```bash
caret run --duration 10 my_first_graph.json
```

### Run with Verbose Output

See what's happening internally:

```bash
caret run --verbose my_first_graph.json
```

### Run with Profiling

Profile performance:

```bash
caret run --profile my_first_graph.json
```

## Common Node Types

Here are some commonly used node types:

### Source Nodes

| Type | Description |
|------|-------------|
| `generator` | Generate values (random, sequence) |
| `file-source` | Read from file |
| `http-source` | HTTP polling |
| `kafka-source` | Kafka consumer (plugin) |

### Transform Nodes

| Type | Description |
|------|-------------|
| `map` | Transform each value |
| `filter` | Filter values |
| `flatMap` | One-to-many transformation |
| `aggregate` | Aggregate values |

### Sink Nodes

| Type | Description |
|------|-------------|
| `console` | Print to console |
| `file-sink` | Write to file |
| `http-sink` | HTTP POST |

## More Examples

### Simple Counter

```json
{
  "nodes": [
    {
      "id": "counter",
      "type": "sequence",
      "config": { "start": 1, "step": 1 }
    },
    {
      "id": "print",
      "type": "console",
      "config": {}
    }
  ],
  "edges": [
    { "from": "counter", "to": "print" }
  ]
}
```

### Branching

```json
{
  "nodes": [
    {
      "id": "source",
      "type": "generator",
      "config": { "min": 1, "max": 100 }
    },
    {
      "id": "router",
      "type": "router",
      "config": {
        "routes": [
          { "predicate": "value < 33", "output": "low" },
          { "predicate": "value < 66", "output": "medium" },
          { "predicate": "true", "output": "high" }
        ]
      }
    },
    {
      "id": "print_low",
      "type": "console",
      "config": { "prefix": "Low: " }
    },
    {
      "id": "print_medium",
      "type": "console",
      "config": { "prefix": "Medium: " }
    },
    {
      "id": "print_high",
      "type": "console",
      "config": { "prefix": "High: " }
    }
  ],
  "edges": [
    { "from": "source", "to": "router" },
    { "from": "router", "from_port": "low", "to": "print_low" },
    { "from": "router", "from_port": "medium", "to": "print_medium" },
    { "from": "router", "from_port": "high", "to": "print_high" }
  ]
}
```

### Aggregation

```json
{
  "nodes": [
    {
      "id": "source",
      "type": "generator",
      "config": { "min": 1, "max": 100, "interval_ms": 100 }
    },
    {
      "id": "aggregate",
      "type": "aggregate",
      "config": {
        "function": "average",
        "window": "tumbling(5s)"
      }
    },
    {
      "id": "print",
      "type": "console",
      "config": { "prefix": "Average: " }
    }
  ],
  "edges": [
    { "from": "source", "to": "aggregate" },
    { "from": "aggregate", "to": "print" }
  ]
}
```

## Validating Graphs

Before running, validate your graph:

```bash
caret validate my_first_graph.json
```

This checks for:
- Valid JSON syntax
- Unknown node types
- Missing required config
- Unconnected nodes
- Cycles in the graph

## Graph Information

Get detailed information about a graph:

```bash
caret info my_first_graph.json
```

Output includes:
- Node count and types
- Edge count
- Topological order
- Estimated memory usage

## Next Steps

1. **Distributed Execution**: Learn how to run graphs across multiple machines in [Distributed Execution](distributed.md)
2. **Plugins**: Extend Caret with custom nodes in [Plugins](plugins.md)
3. **Examples**: Check out [examples/](../../examples/) for more examples

## Troubleshooting

### Graph won't load

```bash
# Validate and see errors
caret validate my_graph.json
```

### No output

```bash
# Run with verbose output
caret run --verbose my_graph.json
```

### Slow performance

```bash
# Profile the graph
caret run --profile my_graph.json
```

## Getting Help

- **Documentation**: [docs/](../)
- **Issues**: [GitHub Issues](https://github.com/staticpayload/caret-eternity/issues)
- **Examples**: [examples/](../../examples/)
