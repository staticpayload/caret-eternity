# Distributed Protocol Specification

This specification defines the network protocol for distributed Caret execution.

## Status: Stable

## Version: 1.0

## Overview

The Caret distributed protocol enables graph execution across multiple worker nodes.

### Participants

- **Coordinator**: Manages graph execution and worker assignment
- **Worker**: Executes assigned graph partitions
- **Discovery Service**: Locates workers on the network

## Transport

### Connection Model

- **Protocol**: TCP
- **Default Port**: 9234
- **Connection Direction**: Worker initiates connection to coordinator

### Handshake

```
Worker                  Coordinator
   |                          |
   |------ Hello --------->   |
   |                          |
   |<----- Welcome --------- |
   |                          |
   |------ Ready ----------> |
   |                          |
```

### Hello Message

```
struct Hello {
    version: u32,           // Protocol version
    node_id: String,        // Unique worker identifier
    capabilities: Capabilities,
}
```

### Welcome Message

```
struct Welcome {
    coordinator_id: String,
    protocol_version: u32,
}
```

## Message Types

### Message Framing

All messages use length-prefixed framing:

```
[Length: u32 BE][Type: u16 BE][Body: Bytes]
```

### Message Types

| Type ID | Name | Direction |
|---------|------|-----------|
| 0x01 | Hello | Worker → Coordinator |
| 0x02 | Welcome | Coordinator → Worker |
| 0x03 | Ready | Worker → Coordinator |
| 0x04 | GraphAssignment | Coordinator → Worker |
| 0x05 | Packet | Bidirectional |
| 0x06 | Heartbeat | Bidirectional |
| 0x07 | Ack | Bidirectional |
| 0x08 | Error | Bidirectional |
| 0x09 | Shutdown | Coordinator → Worker |
| 0x0A | Goodbye | Worker → Coordinator |

## Graph Assignment

### Assignment Message

```
struct GraphAssignment {
    graph_id: String,
    partition: GraphPartition,
    routing_table: RoutingTable,
}
```

### GraphPartition

```
struct GraphPartition {
    nodes: Vec<NodeDefinition>,
    edges: Vec<EdgeDefinition>,
    inputs: Vec<ExternalInput>,
    outputs: Vec<ExternalOutput>,
}
```

### RoutingTable

```
struct RoutingTable {
    entries: Vec<RouteEntry>,
}

struct RouteEntry {
    from_port: PortId,
    to_worker: WorkerId,
    to_port: PortId,
}
```

## Packet Transmission

### Packet Message

```
struct PacketMessage {
    packet_id: u64,
    source_port: PortId,
    destination: Destination,
    payload: PacketPayload,
}
```

### Destination

```
enum Destination {
    Local(PortId),
    Remote(WorkerId, PortId),
}
```

### PacketPayload

```
struct PacketPayload {
    value: SerializedValue,
    metadata: Metadata,
    timestamp: u64,
}
```

### Reliability

- **Ack Required**: All packets require acknowledgment
- **Timeout**: 5 seconds for ack
- **Retry**: Up to 3 retries
- **Drop**: After max retries, packet dropped

## Heartbeat

### Heartbeat Message

```
struct Heartbeat {
    sequence: u64,
    timestamp: u64,
    stats: WorkerStats,
}
```

### WorkerStats

```
struct WorkerStats {
    cpu_usage: f32,
    memory_usage: u64,
    packets_processed: u64,
    buffer_utilization: f32,
}
```

### Heartbeat Interval

- **Default**: 5 seconds
- **Timeout**: 30 seconds without heartbeat

## Error Handling

### Error Message

```
struct Error {
    code: u32,
    message: String,
    context: Bytes,
}
```

### Error Codes

| Code | Name | Description |
|------|------|-------------|
| 1000 | UnknownMessage | Unknown message type |
| 1001 | InvalidFormat | Message format invalid |
| 1002 | VersionMismatch | Protocol version mismatch |
| 2000 | GraphNotFound | Graph ID not found |
| 2001 | PartitionFailed | Partition execution failed |
| 3000 | WorkerTimeout | Worker timed out |
| 3001 | WorkerDisconnected | Worker disconnected |

## Shutdown

### Graceful Shutdown

```
Coordinator                  Worker
   |                          |
   |---- Shutdown -------->  |
   |                          |
   |<---- Goodbye ---------  |
   |                          |
   |------ Ack ---------->  |
   |                          |
```

### Shutdown Timeout

- **Grace Period**: 30 seconds
- **Force Timeout**: 60 seconds

## Discovery

### mDNS Advertisement

```
Service: _caret._tcp.local.
Port: 9234
TXT Records:
  - node_id=<uuid>
  - version=<semver>
  - capabilities=<json>
```

### Discovery Events

```
enum DiscoveryEvent {
    WorkerDiscovered(WorkerInfo),
    WorkerUpdated(WorkerInfo),
    WorkerLeft(WorkerId),
}
```

## Security

### Authentication (Future)

```
struct AuthChallenge {
    nonce: [u8; 32],
}

struct AuthResponse {
    signature: [u8; 64],
}
```

### Encryption (Future)

- **Protocol**: TLS 1.3
- **Cipher Suite**: TLS_AES_256_GCM_SHA384
- **Mutual Auth**: Client certificates required

## Performance

### Requirements

- **Latency**: < 1ms for local area network
- **Throughput**: > 1M packets/second per worker
- **Scalability**: Support 100+ workers

### Optimization

- **Zero Copy**: Where possible
- **Batching**: Multiple packets per message
- **Compression**: Optional for large payloads

## Compliance

Implementations MUST:
1. Support all message types
2. Implement framing correctly
3. Honor timeouts
4. Handle all error codes

Implementations SHOULD:
1. Support mDNS discovery
2. Implement heartbeat
3. Provide metrics
4. Support graceful shutdown

## Examples

### Worker Connection

```
1. Worker connects to coordinator
2. Worker sends Hello(version=1, node_id="w1")
3. Coordinator sends Welcome(protocol_version=1)
4. Worker sends Ready()
5. Coordinator assigns graph partition
```

### Packet Flow

```
1. Worker A generates packet
2. Worker A sends Packet(dest=WorkerB, port="in")
3. Worker B receives packet
4. Worker B sends Ack(packet_id=123)
5. Worker A processes next packet
```

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-10 | Initial specification |
