# ADR 0006: Graph Serialization Format

**Date:** 2024-01-16

**Status:** Accepted

## Context

For distributed graph execution, we need to serialize graphs for:

1. **Network Transmission**: Send graph definitions to worker nodes
2. **Persistence**: Save and load graph definitions
3. **Interoperability**: Cross-language graph representation
4. **Debugging**: Human-readable graph inspection

## Decision

Use JSON for primary serialization, binary format for high-performance scenarios.

### Format

#### JSON Format (Primary)

```json
{
  "nodes": [
    {
      "id": "node-1",
      "type": "source",
      "ports": {
        "outputs": ["out"]
      },
      "config": {}
    }
  ],
  "edges": [
    {
      "from": "node-1",
      "from_port": "out",
      "to": "node-2",
      "to_port": "in"
    }
  ],
  "topological_order": ["node-1", "node-2"]
}
```

#### Binary Format (Optional)

Using bincode for compact binary serialization when:
- Network bandwidth is constrained
- Storing large graph collections
- Performance-critical transmission

### Components

1. **SerializableGraph**: Network-transmittable representation
2. **SerializableNode**: Node with ports and type
3. **SerializableEdge**: Connection definition
4. **Conversion**: `from_caret_graph()` and `to_caret_graph()`

## Alternatives Considered

### 1. Protocol Buffers
- **Pros**: Efficient, schema evolution
- **Cons**: External dependency, proto files to maintain

### 2. MessagePack
- **Pros**: Compact binary format
- **Cons**: Less human-readable, smaller ecosystem

### 3. CBOR
- **Pros**: Standardized, compact
- **Cons**: Less familiar to developers

### 4. XML
- **Pros**: Standard, verbose (sometimes good)
- **Cons**: Verbose (usually bad), complex parsing

### 5. YAML
- **Pros**: Human-readable
- **Cons**: Slower parsing, ambiguity issues

## Consequences

### Positive

- Easy debugging with JSON
- Standard serde ecosystem integration
- Optional binary for performance
- Language interoperability

### Negative

- JSON is verbose for large graphs
- Two formats to maintain (mitigated by shared data structures)

### Neutral

- Performance acceptable for typical graph sizes (<10K nodes)

## Implementation

Located in `caret_distributed/src/serialization.rs`:
- `SerializableGraph`, `SerializableNode`, `SerializableEdge`
- serde derive macros for both formats
- Conversion methods to/from caret_core types

## Future Extensions

- Schema versioning for format evolution
- Compression for large graphs
- Streaming format for incremental transmission

## References

- [serde](https://serde.rs/)
- [bincode](https://docs.rs/bincode/)

## Supersedes

None
