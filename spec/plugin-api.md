# Plugin API Specification

This specification defines the Caret plugin interface.

## Status: Stable

## Version: 1.0

## Overview

Caret plugins extend core functionality by providing custom nodes, transforms, and sinks.

### Plugin Model

- **Loading**: Dynamic library loading (.so, .dylib, .dll)
- **Lifecycle**: Initialize → Register → Execute → Shutdown
- **Isolation**: Plugins run in same process (future: WASM sandbox)

## Plugin Structure

### Exported Symbol

Each plugin MUST export a `caret_plugin_create` function:

```rust
extern "C" fn caret_plugin_create() -> *mut dyn CaretPlugin;
```

### Plugin Trait

```rust
pub trait CaretPlugin {
    /// Returns the plugin name
    fn name(&self) -> &str;

    /// Returns the plugin version
    fn version(&self) -> &str;

    /// Registers custom nodes
    fn register_nodes(&self, registry: &mut NodeRegistry);

    /// Registers custom transforms
    fn register_transforms(&self, registry: &mut TransformRegistry);

    /// Initialize the plugin
    fn initialize(&mut self) -> Result<()>;

    /// Shutdown the plugin
    fn shutdown(&mut self) -> Result<()>;
}
```

## Node API

### Node Trait

```rust
pub trait Node: Send + Sync {
    /// Returns the node name
    fn name(&self) -> &str;

    /// Process data
    fn process(&mut self,
        input: &mut PortSet,
        output: &mut PortSet
    ) -> Result<()>;

    /// Configure the node
    fn configure(&mut self, config: &Value) -> Result<()>;

    /// Input ports
    fn input_ports(&self) -> Vec<PortDef>;

    /// Output ports
    fn output_ports(&self) -> Vec<PortDef>;
}
```

### NodeFactory

```rust
pub type NodeFactory = fn() -> Box<dyn Node>;
```

### Registration

```rust
registry.register("node_name", Box::new(factory_fn));
```

## Port Definition

```rust
pub struct PortDef {
    pub name: String,
    pub port_type: PortType,
    pub schema: ValueSchema,
}

pub enum PortType {
    Data,
    Control,
    State,
}
```

## Value Schema

```rust
pub enum ValueSchema {
    Null,
    Bool,
    Int,
    Float,
    String,
    Bytes,
    List(Box<ValueSchema>),
    Map(Box<ValueSchema>),
    Any,
}
```

## Transform API

### Transform Trait

```rust
pub trait Transform: Send + Sync {
    /// Transform a value
    fn transform(&self, value: Value) -> Result<Value>;

    /// Transform name
    fn name(&self) -> &str;
}
```

### TransformFactory

```rust
pub type TransformFactory = fn() -> Box<dyn Transform>;
```

## Error Handling

### Plugin Error

```rust
pub enum PluginError {
    LoadFailed(String),
    InitializeFailed(String),
    RegistrationFailed(String),
    ExecutionFailed(String),
    ShutdownFailed(String),
}
```

### Error Conversion

All plugin errors MUST convert to `caret_core::Error`.

## Threading

### Thread Safety

- `Node: Send + Sync`: Required for all nodes
- `Transform: Send + Sync`: Required for all transforms

### Concurrent Execution

Multiple nodes may execute concurrently. Plugins MUST:
1. Use appropriate synchronization
2. Avoid global mutable state
3. Use thread-safe data structures

## Memory Management

### Ownership

- Core owns plugin instance
- Plugin owns created nodes
- Nodes own their internal state

### Lifetimes

```rust
// Plugin lifecycle
'load ──> initialize ──> register ──> execute ──> shutdown ──> unload
```

### Resource Cleanup

Plugins MUST implement `Drop` for cleanup:

```rust
impl Drop for MyNode {
    fn drop(&mut self) {
        // Clean up resources
    }
}
```

## Configuration

### Node Configuration

```rust
fn configure(&mut self, config: &Value) -> Result<()> {
    match config {
        Value::Map(settings) => {
            if let Some(Value::String(path)) = settings.get("file") {
                self.file_path = Some(path.clone());
            }
            Ok(())
        }
        _ => Err(Error::InvalidConfig),
    }
}
```

### Configuration Schema

Plugins SHOULD document configuration schema:

```json
{
  "type": "object",
  "properties": {
    "file": {"type": "string"},
    "mode": {"type": "string", "enum": ["read", "write"]}
  },
  "required": ["file"]
}
```

## Versioning

### Plugin Version

- Follow semantic versioning
- Breaking changes: increment MAJOR
- New features: increment MINOR
- Bug fixes: increment PATCH

### API Version

- Core version: Caret version
- Plugin declares compatible core versions
- Mismatch warning on load

## Loading

### Search Paths

Plugins are searched in:
1. `./plugins/`
2. `~/.caret/plugins/`
3. `/usr/local/lib/caret/plugins/`
4. `CARET_PLUGIN_PATH` environment variable

### Load Order

Plugins load in alphabetical order by filename.

### Dependency Resolution

No automatic dependency resolution. Plugins MUST:
- Declare dependencies in metadata
- Handle missing dependencies gracefully

## Security

### Capabilities (Future)

Plugins declare required capabilities:
```rust
fn capabilities(&self) -> Vec<Capability> {
    vec![
        Capability::NetworkAccess,
        Capability::FileRead("/path".into()),
    ]
}
```

### Sandboxing (Future)

WASM plugins:
- No direct system access
- Capability-based security
- Resource limits

## Examples

### Minimal Plugin

```rust
use caret_core::{Node, PortSet, Result};
use caret_plugins::CaretPlugin;

pub struct EchoPlugin;

impl CaretPlugin for EchoPlugin {
    fn name(&self) -> &str { "echo" }
    fn version(&self) -> &str { "1.0.0" }

    fn register_nodes(&self, registry: &mut NodeRegistry) {
        registry.register("echo", Box::new(|| Box::new(EchoNode)));
    }

    fn initialize(&mut self) -> Result<()> { Ok(()) }
    fn shutdown(&mut self) -> Result<()> { Ok(()) }
}

pub struct EchoNode;

impl Node for EchoNode {
    fn name(&self) -> &str { "echo" }

    fn process(&mut self, input: &mut PortSet, output: &mut PortSet) -> Result<()> {
        while let Some(packet) = input.take("in")? {
            output.push("out", packet)?;
        }
        Ok(())
    }

    fn input_ports(&self) -> Vec<PortDef> {
        vec![PortDef::data("in")]
    }

    fn output_ports(&self) -> Vec<PortDef> {
        vec![PortDef::data("out")]
    }
}

caret_plugins::export_plugin!(EchoPlugin);
```

## Compliance

Plugin implementations MUST:
1. Export `caret_plugin_create`
2. Implement `CaretPlugin` trait
3. Handle all errors appropriately
4. Clean up resources on shutdown

Plugin implementations SHOULD:
1. Document configuration
2. Provide examples
3. Handle version mismatches
4. Be thread-safe

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-05 | Initial specification |
