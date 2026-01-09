# Distributed Execution Example

Demonstrates running a Caret graph across multiple worker nodes.

## What It Does

This example shows a graph that benefits from distributed execution:
1. Data source generates values
2. Processing is split across multiple workers
3. Results are aggregated and printed

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Coordinator                            │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐  │
│  │   Graph      │  │  Partitioner  │  │    Aggregator      │  │
│  │   Manager    │  │              │  │                     │  │
│  └──────────────┘  └──────────────┘  └────────────────────┘  │
└─────────────────────────┬───────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
┌───────▼────────┐ ┌────▼──────┐ ┌───────▼────────┐
│  Worker 1      │ │  Worker 2  │ │  Worker 3      │
│  Partition A   │ │Partition B │ │  Partition C   │
│  Process 1-33  │ │Process 34-66│ │ Process 67-100 │
└─────────────────┘ └────────────┘ └─────────────────┘
```

## Running

### Start Coordinator

```bash
caret distributed --mode coordinator --bind 0.0.0.0:9234
```

### Start Workers (in separate terminals)

```bash
# Worker 1
caret distributed --mode worker --connect localhost:9234

# Worker 2
caret distributed --mode worker --connect localhost:9234

# Worker 3
caret distributed --mode worker --connect localhost:9234
```

### Submit Graph

```bash
caret run --coordinator localhost:9234 graph.json
```

## Graph Definition

The graph is automatically partitioned across workers:
- Round-robin distribution of nodes
- Cross-worker packet routing
- Aggregation at coordinator

## Concepts Demonstrated

- **Coordinator-Worker Model**: Central coordinator with distributed workers
- **Graph Partitioning**: Automatic partition assignment
- **Cross-Node Routing**: Packets routed between workers
- **mDNS Discovery**: Automatic worker discovery
