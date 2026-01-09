# ADR 0003: Plugin Architecture with Dynamic Loading

**Date:** 2024-01-05

**Status:** Accepted

## Context

Caret needs to support user-defined nodes and transformations while maintaining:

1. **Extensibility**: Users can add custom functionality without recompiling core
2. **Isolation**: Plugin failures shouldn't crash the core engine
3. **Performance**: Plugin invocation should have minimal overhead
4. **Distribution**: Plugins should be distributable without source code

## Decision

Implement a plugin architecture using Rust's `libloading` crate with dynamic library loading.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Caret Core Engine                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Graph     │  │  Scheduler  │  │  Executor   │         │
│  │  Manager    │  │             │  │             │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                 │
│         └────────────────┴────────────────┘                 │
│                          │                                  │
│                   ┌──────▼──────┐                           │
│                   │  Plugin     │                           │
│                   │   Manager   │                           │
│                   └──────┬──────┘                           │
│                          │                                  │
└──────────────────────────┼──────────────────────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
    ┌────▼────┐      ┌────▼────┐      ┌────▼────┐
    │ Plugin  │      │ Plugin  │      │ Plugin  │
    │   A     │      │   B     │      │   C     │
    │  .so/   │      │  .so/   │      │  .so/   │
    │  .dylib │      │ .dylib  │      │ .dylib  │
    └─────────┘      └─────────┘      └─────────┘
```

### Plugin API

Plugins implement the `CaretPlugin` trait:

```rust
pub trait CaretPlugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    fn register_nodes(&self, registry: &mut NodeRegistry);
    fn register_transformers(&self, registry: &mut TransformerRegistry);

    fn initialize(&mut self) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;
}
```

## Alternatives Considered

### 1. WASM Plugins
- **Pros**: Complete isolation, portable, sandboxable
- **Cons**: Performance overhead, WASM I/O complexity, smaller ecosystem

### 2. Shared Memory / IPC
- **Pros**: Complete process isolation
- **Cons**: Serialization overhead, complex deployment, OS-specific

### 3. Scripting (Lua, Python, JavaScript)
- **Pros**: Easy to write, no compilation
- **Cons**: Poor performance, dependency on runtime, GIL issues

### 4. Static Compilation
- **Pros**: Maximum performance, type safety
- **Cons**: No extensibility without recompilation, longer builds

## Consequences

### Positive

- Users can distribute binary plugins
- Near-native performance (FFI call overhead only)
- Rust API for plugin development
- Plugin hot-reloading in development mode

### Negative

- Plugins must be compiled for each target platform
- ABI compatibility concerns between versions
- Unsafe code required for FFI boundary (isolated to caret_plugins)

### Mitigation

- Versioned plugin API
- Semantic versioning for plugin compatibility
- Safe wrapper macros around unsafe FFI

## Implementation

Located in `caret_plugins` crate:
- `PluginManager` - Handles loading, lifecycle
- `PluginRegistry` - Tracks loaded plugins
- `PluginApi` - Stable ABI interface

## References

- [libloading crate](https://docs.rs/libloading/)
- [Rust FFI Omniscence](https://michael-f-bryan.github.io/rust-ffi-guide/)

## Supersedes

None
