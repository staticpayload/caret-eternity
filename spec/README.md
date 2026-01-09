# Caret Specifications

This directory contains formal specifications for Caret's protocols, formats, and APIs.

## Documents

| Document | Description | Status |
|----------|-------------|--------|
| [graph-execution.md](graph-execution.md) | Graph execution model and semantics | Stable |
| [distributed-protocol.md](distributed-protocol.md) | Distributed network protocol | Stable |
| [plugin-api.md](plugin-api.md) | Plugin interface specification | Stable |
| [serialization.md](serialization.md) | Graph and value serialization | Stable |
| [value-types.md](value-types.md) | Value type system | Stable |

## Conformance

Implementations MUST conform to these specifications to ensure:
- Interoperability between components
- Correctness of execution semantics
- Compatibility across versions

## Versioning

Specifications follow semantic versioning:
- **MAJOR**: Breaking changes to required behavior
- **MINOR**: Backwards-compatible additions
- **PATCH**: Clarifications and corrections

## Contributing

When changing specifications:
1. Update status to `Proposed` for breaking changes
2. Get consensus from maintainers
3. Update all implementations
4. Mark as `Stable` once implemented

## References

- [ADR Index](../adr/)
- [Architecture](../docs/architecture.md)
