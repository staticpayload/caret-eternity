# Distributed Execution Guide

This guide explains how to run Caret graphs across multiple machines.

## Overview

Caret can distribute graph execution across multiple worker nodes:

```
┌─────────────────────────────────────────────────────────────┐
│                      Coordinator                             │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐  │
│  │   Graph      │  │  Partitioner  │  │    Router          │  │
│  │   Manager    │  │              │  │                    │  │
│  └──────────────┘  └──────────────┘  └────────────────────┘  │
└─────────────────────────┬───────────────────────────────────┘
                          │ Network
         ┌────────────────┼────────────────┐
         │                │                │
┌────────▼────────┐ ┌────▼──────┐ ┌───────▼────────┐
│   Worker 1      │ │  Worker 2  │ │   Worker 3     │
│  Partition A    │ │Partition B │ │  Partition C   │
│                 │ │            │ │                │
│  [Nodes 1-10]   │ │[Nodes 11-20]│ │ [Nodes 21-30]  │
└─────────────────┘ └────────────┘ └────────────────┘
```

## Benefits

- **Horizontal Scaling**: Process more data by adding workers
- **Parallel Execution**: Multiple partitions run simultaneously
- **Fault Tolerance**: Workers can fail without stopping the entire graph
- **Resource Utilization**: Distribute CPU/memory across machines

## Quick Start

### 1. Start a Coordinator

On your first machine:

```bash
caret distributed --mode coordinator --bind 0.0.0.0:9234
```

Output:

```
[INFO] Starting coordinator mode
[INFO] Binding to 0.0.0.0:9234
[INFO] mDNS discovery enabled
[INFO] Waiting for workers...
```

### 2. Start Workers

On each worker machine:

```bash
caret distributed --mode worker --connect <coordinator-ip>:9234
```

Example:

```bash
# Worker 1
caret distributed --mode worker --connect 192.168.1.100:9234

# Worker 2
caret distributed --mode worker --connect 192.168.1.100:9234

# Worker 3
caret distributed --mode worker --connect 192.168.1.100:9234
```

### 3. Submit a Graph

From any machine (or the coordinator):

```bash
caret run --coordinator <coordinator-ip>:9234 my_graph.json
```

## Configuration

### Coordinator Options

```bash
caret distributed --mode coordinator [OPTIONS]
```

| Option | Description | Default |
|--------|-------------|---------|
| `--bind <ADDR>` | Bind address | `0.0.0.0:9234` |
| `--discovery <MODE>` | Discovery mode | `mdns` |
| `--worker-timeout <SECS>` | Worker timeout | `30` |
| `--heartbeat-interval <SECS>` | Heartbeat interval | `5` |

### Worker Options

```bash
caret distributed --mode worker [OPTIONS]
```

| Option | Description | Default |
|--------|-------------|---------|
| `--bind <ADDR>` | Local bind address | `0.0.0.0:0` |
| `--connect <ADDR>` | Coordinator address | required |
| `--node-id <ID>` | Custom node ID | auto-generated |
| `--cpu-cores <N>` | Usable CPU cores | all |

### Discovery Modes

#### mDNS (Default)

Automatic discovery on local network:

```bash
caret distributed --mode coordinator --discovery mdns
```

#### Static

Manual IP configuration:

```bash
# Coordinator
caret distributed --mode coordinator --discovery static

# Workers
caret distributed --mode worker --connect 192.168.1.100:9234
```

#### None

No discovery, manual connection only:

```bash
caret distributed --mode coordinator --discovery none
```

## Graph Partitioning

Caret automatically partitions graphs for distributed execution.

### Partitioning Strategies

#### Round Robin (Default)

Distributes nodes evenly across workers:

```json
{
  "partitioning": {
    "strategy": "round_robin"
  }
}
```

#### Contiguous

Groups connected nodes together:

```json
{
  "partitioning": {
    "strategy": "contiguous"
  }
}
```

#### Minimize Cross Edges

Minimizes network traffic:

```json
{
  "partitioning": {
    "strategy": "minimize_cross_edges"
  }
}
```

#### Manual

Specify partition for each node:

```json
{
  "partitioning": {
    "strategy": "manual",
    "assignments": {
      "node1": "worker1",
      "node2": "worker1",
      "node3": "worker2"
    }
  }
}
```

### Example Graph with Partitioning

```json
{
  "partitioning": {
    "strategy": "round_robin",
    "workers": ["auto", "auto", "auto"]
  },
  "nodes": [
    {"id": "source", "type": "generator"},
    {"id": "process1", "type": "map", "config": {"expression": "x * 2"}},
    {"id": "process2", "type": "map", "config": {"expression": "x + 10"}},
    {"id": "process3", "type": "filter", "config": {"predicate": "x > 20"}},
    {"id": "sink", "type": "console"}
  ],
  "edges": [
    {"from": "source", "to": "process1"},
    {"from": "process1", "to": "process2"},
    {"from": "process2", "to": "process3"},
    {"from": "process3", "to": "sink"}
  ]
}
```

## Cross-Partition Routing

When edges cross partition boundaries, Caret automatically routes packets:

```
Worker 1              Worker 2
────────              ────────
[Node A] ────packet───▶ [Node C]
[Node B]              [Node D]
```

Packets from `Node A` (Worker 1) to `Node C` (Worker 2) are automatically sent over the network.

## Monitoring

### Cluster Status

```bash
# List all nodes
caret cluster list

# Show worker status
caret cluster status

# Show graph execution
caret cluster graphs
```

### Logs

```bash
# Follow logs from all nodes
caret cluster logs --follow

# Filter by node
caret cluster logs --node-id <node-id>

# Filter by level
caret cluster logs --level error
```

## Best Practices

### 1. Minimize Cross-Partition Traffic

Group connected nodes on the same worker:

```json
{
  "partitioning": {
    "strategy": "contiguous"
  }
}
```

### 2. Balance Worker Load

Ensure even distribution of work:

```json
{
  "partitioning": {
    "strategy": "minimize_cross_edges"
  }
}
```

### 3. Handle Worker Failures

Set appropriate timeouts:

```bash
caret distributed --mode coordinator --worker-timeout 60
```

### 4. Network Configuration

- Use low-latency networks (<10ms preferred)
- Ensure firewall allows traffic on port 9234
- Use dedicated network for large clusters

## Troubleshooting

### Workers Not Discovered

```bash
# Check coordinator is running
caret cluster status

# Verify network connectivity
ping <coordinator-ip>

# Check firewall
telnet <coordinator-ip> 9234

# Use static discovery if mDNS fails
caret distributed --mode worker --connect <ip>:9234
```

### Graph Not Distributed

```bash
# Verify partitioning in graph config
caret info --verbose my_graph.json

# Check worker count
caret cluster list

# Ensure workers are connected
caret cluster status
```

### Poor Performance

```bash
# Check cross-partition traffic
caret cluster stats --graph <graph-id>

# Use appropriate partitioning strategy
# Modify graph to minimize cross-edges

# Profile workers
caret distributed --mode worker --profile
```

## Advanced Configuration

### Custom Partitioning

```json
{
  "partitioning": {
    "strategy": "custom",
    "function": "path/to/partition_lib.so:partition_function"
  }
}
```

### Network Configuration

```json
{
  "network": {
    "tcp": {
      "keepalive": true,
      "keepalive_interval_secs": 10,
      "tcp_nodelay": true
    },
    "buffer_size": 65536,
    "compression": "snappy"
  }
}
```

### Security

```json
{
  "security": {
    "tls": {
      "enabled": true,
      "cert_file": "/path/to/cert.pem",
      "key_file": "/path/to/key.pem",
      "ca_file": "/path/to/ca.pem"
    },
    "authentication": {
      "type": "token",
      "token_file": "/path/to/token.txt"
    }
  }
}
```

## Example: Complete Setup

### Coordinator (machine1)

```bash
#!/bin/bash
# start_coordinator.sh

caret distributed \
  --mode coordinator \
  --bind 0.0.0.0:9234 \
  --discovery mdns \
  --worker-timeout 60 \
  --heartbeat-interval 5 \
  --log-level info
```

### Workers (machines 2-4)

```bash
#!/bin/bash
# start_worker.sh

COORDINATOR_IP="192.168.1.100"

caret distributed \
  --mode worker \
  --connect $COORDINATOR_IP:9234 \
  --bind 0.0.0.0:0 \
  --cpu-cores 4 \
  --log-level info
```

### Submit Graph

```bash
#!/bin/bash
# submit_graph.sh

COORDINATOR_IP="192.168.1.100"

caret run \
  --coordinator $COORDINATOR_IP:9234 \
  --workers 3 \
  --partition-strategy minimize_cross_edges \
  my_distributed_graph.json
```

## Next Steps

- [CLI Reference](../cli.md) - Full command documentation
- [Architecture](../architecture.md) - Distributed architecture details
- [Examples](../../examples/) - Distributed graph examples

## Getting Help

- **Documentation**: [docs/](../)
- **Issues**: [GitHub Issues](https://github.com/staticpayload/caret-eternity/issues)
- **ADRs**: [adr/](../../adr/) - Architecture decisions
