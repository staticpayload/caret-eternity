# ADR 0005: mDNS for Node Discovery

**Date:** 2024-01-15

**Status:** Accepted

## Context

For distributed graph execution on local networks, we need automatic node discovery:

1. **Zero Configuration**: Users shouldn't manually specify IP addresses
2. **Dynamic Membership**: Nodes can join/leave the network
3. **Local Network Focus**: Most distributed use cases are on LANs
4. **Standard Protocol**: Use industry standards for compatibility

## Decision

Use mDNS (Multicast DNS) for local network node discovery.

### Implementation

Using the `mdns-sd` crate to:

1. **Announce Services**: Each node broadcasts its presence via mDNS
2. **Browse Services**: Nodes discover other Caret instances
3. **Service Metadata**: TXT records carry node information:
   - `node_id`: Unique node identifier
   - `version`: Caret version
   - `capabilities`: Supported features

### Service Details

```
Service Type: _caret._tcp.local.
Port: 9234 (default)
TXT Records:
  - node_id=<uuid>
  - version=<semver>
  - capabilities=<comma-separated>
```

### Discovery Events

```
NodeDiscovered  -> New node found on network
NodeUpdated     -> Node information changed
NodeLeft        -> Node no longer available
```

## Alternatives Considered

### 1. UDP Broadcast
- **Pros**: Simple, no dependencies
- **Cons**: Not standardized, no metadata support, router issues

### 2. Static Configuration
- **Pros**: Simple, predictable
- **Cons**: Manual, error-prone, no dynamic updates

### 3. Central Registry
- **Pros**: Consistent view
- **Cons**: Single point of failure, external dependency

### 4. Consul / etcd
- **Pros**: Production-grade, feature-rich
- **Cons**: Heavy dependency, overkill for simple LAN discovery

## Consequences

### Positive

- Zero-configuration LAN setup
- Standard protocol (works with Avahi, Bonjour)
- Extensible metadata via TXT records
- Automatic membership tracking

### Negative

- mDNS doesn't cross routers by design
- Requires multicast-enabled network
- Some corporate networks block mDNS

### Mitigation

- Static configuration option for non-LAN scenarios
- Future: Consul/etcd adapter for cloud deployments

## Implementation

Located in `caret_distributed/src/mdns.rs`:
- `MdnsDiscovery`: Main discovery service
- `MdnsDiscoveryConfig`: Builder pattern configuration
- TXT record keys defined in `txt_keys` module

## Future Extensions

- Support for DNS-SD (Service Discovery)
- Integration with Consul for cloud scenarios
- Custom service types for different node roles

## References

- [RFC 6762 - mDNS](https://datatracker.ietf.org/doc/html/rfc6762)
- [RFC 6763 - DNS-SD](https://datatracker.ietf.org/doc/html/rfc6763)
- [mdns-sd crate](https://docs.rs/mdns-sd/)

## Supersedes

None
