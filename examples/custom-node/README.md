# Custom Node Example

Demonstrates how to create and use a custom node in Caret.

## Overview

This example shows how to:
1. Define a custom node type
2. Implement the Node trait
3. Register the node with Caret
4. Use the node in a graph

## Custom Node: MovingAverage

The moving average node calculates the average of the last N values.

## Code

See `moving_average.rs` for the implementation.

## Running

```bash
# Build the custom node plugin
cargo build --release --example moving_average

# Run the graph
cargo run --bin caret -- run graph.json
```

## Graph Definition

```json
{
  "nodes": [
    {
      "id": "source",
      "type": "generator"
    },
    {
      "id": "ma",
      "type": "moving_average",
      "config": {
        "window_size": 5
      }
    },
    {
      "id": "output",
      "type": "console"
    }
  ],
  "edges": [
    {"from": "source", "to": "ma"},
    {"from": "ma", "to": "output"}
  ]
}
```

## Concepts Demonstrated

- **Custom Nodes**: Creating new node types
- **Stateful Nodes**: Maintaining internal state
- **Configuration**: Accepting node configuration
- **Plugin System**: Registering custom nodes
