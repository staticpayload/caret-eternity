# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the Caret project. ADRs document significant architectural decisions, their context, and consequences.

## Format

ADRs follow the standard template format found in [template.md](./template.md).

## Index

| ID | Title | Status | Date |
|----|-------|--------|------|
| 0001 | [Use Rust for Core Implementation](./0001-use-rust.md) | Accepted | 2024-01-01 |
| 0002 | [Use Tokio for Async Runtime](./0002-async-tokio.md) | Accepted | 2024-01-02 |
| 0003 | [Plugin Architecture with Dynamic Loading](./0003-plugin-architecture.md) | Accepted | 2024-01-05 |
| 0004 | [Distributed Graph Execution](./0004-distributed-execution.md) | Accepted | 2024-01-10 |
| 0005 | [mDNS for Node Discovery](./0005-mdns-discovery.md) | Accepted | 2024-01-15 |
| 0006 | [Graph Serialization Format](./0006-graph-serialization.md) | Accepted | 2024-01-16 |

## Contributing

When making a significant architectural decision:

1. Copy [template.md](./template.md) to `NNNN-title.md` (where NNNN is the next number)
2. Fill in all sections
3. Submit as part of your PR
4. Discuss with the team and revise as needed
5. Mark as **Accepted** once consensus is reached

## Status Values

- **Proposed** - Under discussion
- **Accepted** - Decision made, implementation in progress or complete
- **Deprecated** - No longer recommended, but still in use
- **Superseded** - Replaced by another ADR
- **Rejected** - Decision not pursued
