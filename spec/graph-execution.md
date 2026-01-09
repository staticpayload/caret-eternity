# Graph Execution Specification

This specification defines the execution semantics of Caret graphs.

## Status: Stable

## Version: 1.0

## Overview

A Caret graph is a directed graph where nodes transform data and edges route data between nodes.

### Formal Definition

A graph `G = (N, E)` where:
- `N` is a set of nodes
- `E ⊆ N × P × N × P` is a set of edges connecting ports

Where `P` is the set of port identifiers.

## Graph Structure

### Nodes

A node `n ∈ N` consists of:
- **id**: Unique identifier
- **type**: Node type determining behavior
- **input_ports**: Set of input port identifiers
- **output_ports**: Set of output port identifiers
- **config**: Type-specific configuration

### Edges

An edge `e ∈ E` is a tuple `(n1, p1, n2, p2)` where:
- `n1` is the source node
- `p1` is the source port (must be in `n1.output_ports`)
- `n2` is the destination node
- `p2` is the destination port (must be in `n2.input_ports`)

### Validity Conditions

A graph is valid iff:
1. All node IDs are unique
2. All edge references exist
3. Port connections are type-compatible
4. The graph is acyclic (or contains only valid cycles)

## Execution Model

### Packets

A packet is a unit of data flow:
```
Packet = (Value, Metadata, Timestamp)
```

### Execution Cycle

Each tick consists of:

1. **Scheduling**: Select nodes to execute
2. **Execution**: Invoke selected nodes
3. **Routing**: Move packets to destinations
4. **Backpressure**: Handle buffer limits

### Pseudocode

```
function execute(graph):
    while graph.running:
        ready = scheduler.get_ready_nodes(graph)
        for node in ready:
            execute_node(node)
        route_packets(graph)
        check_backpressure(graph)
```

### Node Execution

A node executes when:
1. At least one input port has data available, OR
2. The node is a source node (generates data)

### Semantics

#### Determinism

For a given input sequence, a deterministic node produces the same output sequence.

#### Side Effects

Nodes SHOULD NOT have side effects except:
- Emitting packets to output ports
- Updating internal state
- I/O through designated sink nodes

#### Stateful Nodes

Stateful nodes maintain internal state across invocations:
```
state[t+1] = f(state[t], inputs[t])
outputs[t] = g(state[t], inputs[t])
```

## Topological Ordering

For acyclic graphs, nodes execute in topological order:
```
order = topological_sort(graph)
for node in order:
    if node.is_ready():
        execute(node)
```

## Cycles

### Valid Cycles

A cycle is valid iff:
1. At least one node in the cycle is stateful, OR
2. The cycle contains a backpressure-inducing node

### Cycle Execution

Cycles execute using fixed-point iteration:
```
repeat
    progress = false
    for node in cycle:
        if node.execute():
            progress = true
until not progress
```

## Backpressure

### Buffer Limits

Each edge has a buffer capacity:
```
buffer[edge].size <= buffer.capacity
```

### Backpressure Propagation

When a buffer is full:
1. Upstream node is blocked from writing
2. Block propagates upstream through the graph

### Semantics

A node MUST NOT block indefinitely:
- If blocked, node yields
- Node becomes ready when buffer has space

## Concurrency

### Parallel Execution

Non-conflicting nodes may execute concurrently:
```
nodes A and B are concurrent iff:
  - No path from A to B
  - No path from B to A
  - No shared mutable state
```

### Scheduler Contract

The scheduler MUST ensure:
1. Ready nodes are eventually executed
2. No starvation of any node
3. Fair execution among ready nodes

## Error Handling

### Error Propagation

Errors propagate as special error packets:
```
ErrorPacket = (Error, Metadata, Timestamp)
```

### Error Semantics

1. Node errors emit error packets on error port
2. Error packets propagate like normal packets
3. Unhandled errors terminate graph execution

## Time

### Timestamps

All packets carry timestamps:
```
packet.timestamp = current_time()
```

### Timing Semantics

- Source nodes set initial timestamp
- Timestamps are preserved through transformations
- Aggregate nodes use input timestamps for windowing

## Metrics

### Required Metrics

Implementations MUST track:
- `packets_processed`: Total packets processed
- `packets_dropped`: Packets dropped due to backpressure
- `execution_time_ns`: Total execution time
- `buffer_utilization`: Current buffer usage

### Metric Collection

Metrics are collected at:
- Per-node granularity
- Per-edge granularity
- Graph-level aggregation

## Compliance

Implementations MUST:
1. Execute graphs according to these semantics
2. Preserve packet ordering within edges
3. Honor backpressure signals
4. Provide required metrics

Implementations MAY:
- Use different scheduling strategies
- Parallelize execution
- Optimize while preserving semantics

## Examples

### Simple Pipeline

```
[A] -> [B] -> [C]

Execution order: A, B, C
(assuming sequential scheduler)
```

### Diamond Graph

```
    /-> [B] --\
[A]           => [D]
    \-> [C] --/

D executes after both B and C complete
```

### Cycle with Feedback

```
[A] -> [B] -> [C]
       ^      |
       \------/

B is stateful, enabling valid cycle
```

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-01 | Initial specification |
