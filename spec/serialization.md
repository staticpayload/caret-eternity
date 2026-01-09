# Serialization Specification

This specification defines the serialization formats for Caret graphs and values.

## Status: Stable

## Version: 1.0

## Overview

Caret supports multiple serialization formats:
- **JSON**: Human-readable, default format
- **Bincode**: Compact binary format
- **MessagePack**: Efficient binary format (future)

## Value Serialization

### JSON Format

| Value Type | JSON Representation |
|------------|-------------------|
| Null | `null` |
| Bool | `true` or `false` |
| Int | `{"int": 42}` |
| Float | `{"float": 3.14}` |
| String | `{"string": "hello"}` |
| Bytes | `{"bytes": "base64..."}` |
| List | `{"list": [...]}` |
| Map | `{"map": {...}}` |

### Binary Format (Bincode)

```
enum Value {
    Null = 0,
    Bool(u8) = 1,
    Int(i64) = 2,
    Float(f64) = 3,
    String(Vec<u8>) = 4,
    Bytes(Vec<u8>) = 5,
    List(Vec<Value>) = 6,
    Map(Vec<(String, Value)>) = 7,
}
```

## Graph Serialization

### JSON Schema

```json
{
  "type": "object",
  "required": ["nodes", "edges"],
  "properties": {
    "version": {"type": "string"},
    "name": {"type": "string"},
    "description": {"type": "string"},
    "nodes": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "type"],
        "properties": {
          "id": {"type": "string"},
          "type": {"type": "string"},
          "config": {"type": "object"},
          "metadata": {"type": "object"}
        }
      }
    },
    "edges": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["from", "to"],
        "properties": {
          "id": {"type": "string"},
          "from": {"type": "string"},
          "from_port": {"type": "string"},
          "to": {"type": "string"},
          "to_port": {"type": "string"}
        }
      }
    },
    "partitioning": {
      "type": "object",
      "properties": {
        "strategy": {"type": "string"},
        "assignments": {"type": "object"}
      }
    }
  }
}
```

### Example

```json
{
  "version": "1.0",
  "name": "Example Graph",
  "nodes": [
    {
      "id": "source",
      "type": "generator",
      "config": {"interval_ms": 100}
    },
    {
      "id": "transform",
      "type": "map",
      "config": {"expression": "x * 2"}
    },
    {
      "id": "sink",
      "type": "console",
      "config": {}
    }
  ],
  "edges": [
    {"from": "source", "to": "transform"},
    {"from": "transform", "to": "sink"}
  ]
}
```

## Packet Serialization

### JSON Format

```json
{
  "id": "uuid",
  "value": {...},
  "metadata": {
    "source": "node_id",
    "trace_id": "uuid"
  },
  "timestamp": 1234567890
}
```

### Binary Format

```
struct Packet {
    id: u128,
    value: Value,
    metadata_len: u32,
    metadata: [u8],
    timestamp: u64,
}
```

## Network Serialization

### Framing

```
[length: u32 BE][checksum: u32 BE][payload: bytes]
```

### Checksum

CRC-32 of payload.

### Compression

Optional compression using snappy:
```
[compressed: bool][compressed_length: u32][data: bytes]
```

## Type Coercion

### Coercion Rules

| From | To | Method |
|------|-----|--------|
| Int | Float | Direct |
| Float | Int | Truncate |
| String | Int | Parse (may fail) |
| Int | String | ToString |
| List(String) | String | Join |

### Coercion Errors

Invalid coercion results in error packet.

## Versioning

### Format Version

Each serialized document includes version:

```json
{"version": "1.0", ...}
```

### Backward Compatibility

- Version 1.0 reader MUST read version 1.x formats
- Version 1.0 reader MAY read future formats with warning

### Forward Compatibility

Unknown fields MUST be ignored:
```json
{
  "known": 123,
  "future_field": "ignored in v1.0"
}
```

## Compliance

Implementations MUST:
1. Support JSON format
2. Handle all value types
3. Validate input format
4. Provide clear error messages

Implementations SHOULD:
1. Support binary formats
2. Implement compression
3. Handle large graphs efficiently

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-15 | Initial specification |
