# Caret Reference

Complete reference for Caret's API, configuration, and behavior.

## Table of Contents

1. [Node Types](#node-types)
2. [Value Types](#value-types)
3. [Graph Schema](#graph-schema)
4. [Configuration](#configuration)
5. [Error Codes](#error-codes)

## Node Types

### Source Nodes

#### generator

Generates values at a specified interval.

```json
{
  "type": "generator",
  "config": {
    "interval_ms": 1000,
    "values": [1, 2, 3]
  }
}
```

| Config | Type | Default | Description |
|--------|------|---------|-------------|
| `interval_ms` | number | 1000 | Milliseconds between values |
| `values` | array | - | Values to emit (cyclic) |
| `min` | number | 0 | Minimum for random generation |
| `max` | number | 100 | Maximum for random generation |
| `seed` | number | - | Random seed |

#### file-source

Reads from a file.

```json
{
  "type": "file-source",
  "config": {
    "path": "/path/to/file",
    "format": "json",
    "emit": "lines"
  }
}
```

| Config | Type | Default | Description |
|--------|------|---------|-------------|
| `path` | string | required | File path |
| `format` | string | "json" | File format (json, csv, text) |
| `emit` | string | "lines" | How to emit (lines, whole) |

### Transform Nodes

#### map

Transforms each value.

```json
{
  "type": "map",
  "config": {
    "expression": "x * 2"
  }
}
```

| Config | Type | Default | Description |
|--------|------|---------|-------------|
| `expression` | string | required | Expression to evaluate |
| `language` | string | "expr" | Expression language |

#### filter

Filters values.

```json
{
  "type": "filter",
  "config": {
    "predicate": "x > 10"
  }
}
```

| Config | Type | Default | Description |
|--------|------|---------|-------------|
| `predicate` | string | required | Filter predicate |

#### flat-map

One-to-many transformation.

```json
{
  "type": "flat-map",
  "config": {
    "expression": "split(x, ',')"
  }
}
```

### Aggregate Nodes

#### aggregate

Aggregates values over a window.

```json
{
  "type": "aggregate",
  "config": {
    "function": "sum",
    "window": "tumbling(5s)"
  }
}
```

| Config | Type | Default | Description |
|--------|------|---------|-------------|
| `function` | string | required | sum, avg, min, max, count |
| `window` | string | required | Window specification |
| `emit` | string | "each" | When to emit (each, end) |

**Window Types:**
- `tumbling(duration)` - Fixed size, non-overlapping
- `sliding(duration, advance)` - Fixed size, overlapping
- `session(timeout)` - Activity-based

#### reduce

Reduces values to a single value.

```json
{
  "type": "reduce",
  "config": {
    "function": "acc + x",
    "initial": 0
  }
}
```

### Routing Nodes

#### router

Routes to different outputs.

```json
{
  "type": "router",
  "config": {
    "routes": [
      {"predicate": "x > 10", "output": "high"},
      {"predicate": "true", "output": "default"}
    ]
  }
}
```

#### merge

Merges multiple inputs.

```json
{
  "type": "merge",
  "config": {
    "strategy": "round-robin"
  }
}
```

**Strategies:**
- `round-robin` - Distribute evenly
- `priority` - Prioritize by input order
- `merge` - Combine all

### Sink Nodes

#### console

Prints to console.

```json
{
  "type": "console",
  "config": {
    "prefix": "Output: ",
    "format": "json"
  }
}
```

| Config | Type | Default | Description |
|--------|------|---------|-------------|
| `prefix` | string | "" | Output prefix |
| `format` | string | "json" | Output format |
| `stream` | string | "stdout" | stdout or stderr |

#### file-sink

Writes to file.

```json
{
  "type": "file-sink",
  "config": {
    "path": "/path/to/output",
    "format": "json",
    "mode": "append"
  }
}
```

| Config | Type | Default | Description |
|--------|------|---------|-------------|
| `path` | string | required | Output file path |
| `format` | string | "json" | Output format |
| `mode` | string | "append" | Write mode |

## Value Types

### Type Hierarchy

```
Value
├── Null
├── Bool(bool)
├── Int(i64)
├── Float(f64)
├── String(Arc<str>)
├── Bytes(Arc<Vec<u8>>)
├── List(Arc<Vec<Value>>)
└── Map(Arc<HashMap<String, Value>>)
```

### Type Coercion

| From | To | Method |
|------|-----|--------|
| Int | Float | Direct conversion |
| String | Int | Parse (may fail) |
| List | String | Join with comma |
| Null | Any | Default value |

## Graph Schema

### Root Structure

```json
{
  "version": "1.0",
  "name": "My Graph",
  "description": "A sample graph",
  "nodes": [...],
  "edges": [...],
  "partitioning": {...}
}
```

### Node Schema

```json
{
  "id": "unique-id",
  "type": "node-type",
  "config": {},
  "metadata": {
    "tags": ["tag1", "tag2"],
    "description": "Node description"
  }
}
```

### Edge Schema

```json
{
  "id": "optional-edge-id",
  "from": "source-node",
  "from_port": "output-port",
  "to": "dest-node",
  "to_port": "input-port",
  "metadata": {}
}
```

### Port Naming

Default ports:
- Single input: `in`
- Single output: `out`
- Multiple: `in0`, `in1`, ... or `out0`, `out1`, ...

## Configuration

### File Locations

Searched in order:
1. `./caret.toml`
2. `$HOME/.caret/config.toml`
3. `/etc/caret/config.toml`

### Config Structure

```toml
[general]
log_level = "info"
worker_threads = 4

[network]
bind_address = "0.0.0.0:9234"
port = 9234
keepalive = true

[performance]
buffer_size = 1024
queue_size = 1000
tick_interval_ms = 10

[logging]
format = "pretty"
targets = ["stdout", "file"]

[plugins]
paths = ["/usr/local/lib/caret"]
auto_load = ["plugin1", "plugin2"]

[distributed]
mode = "coordinator"
coordinator_address = "localhost:9234"
discovery = "mdns"
worker_timeout = 30
heartbeat_interval = 5
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `CARET_CONFIG` | Path to config file |
| `CARET_LOG_LEVEL` | Log level override |
| `CARET_PLUGIN_PATH` | Plugin search path |
| `CARET_BIND_ADDR` | Bind address |
| `CARET_WORKERS` | Worker thread count |
| `NO_COLOR` | Disable colors |

## Expression Language

### Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `x + 1` |
| `-` | Subtraction | `x - 1` |
| `*` | Multiplication | `x * 2` |
| `/` | Division | `x / 2` |
| `%` | Modulo | `x % 10` |
| `==` | Equal | `x == 0` |
| `!=` | Not equal | `x != 0` |
| `<` | Less than | `x < 10` |
| `<=` | Less or equal | `x <= 10` |
| `>` | Greater than | `x > 10` |
| `>=` | Greater or equal | `x >= 10` |
| `&&` | Logical and | `x > 0 && y > 0` |
| `||` | Logical or | `x > 0 || y > 0` |
| `!` | Logical not | `!x` |

### Functions

| Function | Description |
|----------|-------------|
| `abs(x)` | Absolute value |
| `min(a, b)` | Minimum |
| `max(a, b)` | Maximum |
| `sqrt(x)` | Square root |
| `pow(x, n)` | Power |
| `floor(x)` | Floor |
| `ceil(x)` | Ceiling |
| `round(x)` | Round |
| `len(s)` | String/array length |
| `substring(s, start, end)` | Extract substring |
| `split(s, sep)` | Split string |
| `join(arr, sep)` | Join array |
| `to_string(x)` | Convert to string |
| `to_int(x)` | Convert to integer |
| `to_float(x)` | Convert to float |

### Field Access

Access nested fields:

```json
{
  "expression": "user.name",
  "input": {"user": {"name": "Alice"}}
}
```

Array access:

```json
{
  "expression": "items[0]",
  "input": {"items": [1, 2, 3]}
}
```

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 1 | `GenericError` | General error |
| 2 | `InvalidGraph` | Graph validation failed |
| 3 | `NodeNotFound` | Node type not found |
| 4 | `PortNotFound` | Port not found |
| 5 | `TypeMismatch` | Type mismatch |
| 6 | `ConfigurationError` | Invalid configuration |
| 7 | `RuntimeError` | Runtime execution error |
| 8 | `Timeout` | Operation timeout |
| 9 | `NetworkError` | Network-related error |
| 10 | `PluginError` | Plugin-related error |

## Performance Tuning

### Buffer Sizes

```json
{
  "buffer_size": 1024,
  "queue_size": 1000
}
```

- Small buffers (< 256): Lower latency, more contention
- Large buffers (> 4096): Higher latency, less contention

### Worker Threads

```bash
caret run --workers 8 graph.json
```

- Set to number of CPU cores for CPU-bound work
- Set higher (2-4x cores) for I/O-bound work

### Batch Size

```json
{
  "type": "aggregate",
  "config": {
    "batch_size": 100
  }
}
```

Larger batches = better throughput, higher latency

## Monitoring

### Metrics

Exposed metrics:
- `caret_nodes_active` - Active nodes
- `caret_packets_total` - Packets processed
- `caret_packets_dropped` - Packets dropped
- `caret_buffer_utilization` - Buffer usage
- `caret_tick_duration_ms` - Tick duration

### Health Check

```bash
# Check health endpoint
curl http://localhost:9234/health

# Get metrics
curl http://localhost:9234/metrics
```

## See Also

- [CLI Reference](../cli.md)
- [Architecture](../architecture.md)
- [Distributed Guide](distributed.md)
