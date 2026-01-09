# Plugins Guide

Caret can be extended with plugins that add custom nodes, transforms, and functionality.

## Overview

Plugins allow you to:

- Add custom node types
- Define data transformations
- Extend Caret without modifying core code
- Distribute functionality as compiled libraries

## Plugin Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Caret Core                           │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────┐  │
│  │   Graph    │  │  Scheduler │  │   Plugin         │  │
│  │  Manager   │  │            │  │   Manager        │  │
│  └─────┬──────┘  └────────────┘  └────────┬─────────┘  │
└────────┼─────────────────────────────────┼────────────┘
         │                                 │
         │    ┌────────────────────────────┘
         │    │
    ┌────▼────▼────────┐
    │   Plugin API     │
    └─────┬────────────┘
          │
    ┌─────┴───────────────────────────────────┐
    │                                          │
┌───▼────┐  ┌─────────┐  ┌─────────┐  ┌─────▼───┐
│Plugin A│  │Plugin B │  │Plugin C │  │Plugin D │
│(.so/   │  │(.so/    │  │(.so/    │  │(.so/    │
│ .dylib)│  │ .dylib) │  │ .dylib) │  │ .dylib) │
└────────┘  └─────────┘  └─────────┘  └─────────┘
```

## Built-in Nodes

Before writing a plugin, check if Caret already has what you need:

| Type | Description |
|------|-------------|
| `source` | Generate data |
| `map` | Transform each value |
| `filter` | Filter values |
| `aggregate` | Aggregate values |
| `join` | Join multiple streams |
| `router` | Route to multiple outputs |
| `sink` | Output data |

## Creating a Plugin

### 1. Create Plugin Project

```bash
cargo new --lib my_caret_plugin
cd my_caret_plugin
```

### 2. Add Dependencies

Edit `Cargo.toml`:

```toml
[package]
name = "my_caret_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
caret_core = { git = "https://github.com/staticpayload/caret-eternity.git" }
caret_plugins = { git = "https://github.com/staticpayload/caret-eternity.git" }
```

### 3. Implement Plugin

Edit `src/lib.rs`:

```rust
use caret_core::{Node, PortSet, Value, Result, Error};
use caret_plugins::{CaretPlugin, PluginRegistry, NodeRegistry};

// Define the plugin struct
pub struct MyPlugin;

impl CaretPlugin for MyPlugin {
    fn name(&self) -> &str {
        "my_caret_plugin"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn register_nodes(&self, registry: &mut NodeRegistry) {
        registry.register("uppercase", Box::new(|| Box::new(UppercaseNode::new())));
        registry.register("reverse", Box::new(|| Box::new(ReverseNode::new())));
    }

    fn initialize(&mut self) -> Result<()> {
        println!("MyPlugin initialized!");
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        println!("MyPlugin shut down!");
        Ok(())
    }
}

// Define a custom node
pub struct UppercaseNode;

impl UppercaseNode {
    pub fn new() -> Self {
        UppercaseNode
    }
}

impl Node for UppercaseNode {
    fn name(&self) -> &str {
        "uppercase"
    }

    fn process(&mut self, input: &mut PortSet, output: &mut PortSet) -> Result<()> {
        while let Some(packet) = input.take("in")? {
            if let Value::String(s) = packet.value() {
                let upper = s.to_uppercase();
                output.push("out", packet.with_value(Value::String(upper.into())))?;
            }
        }
        Ok(())
    }

    fn input_ports(&self) -> Vec<(String, crate::core::PortType)> {
        vec![("in".into(), crate::core::PortType::Data)]
    }

    fn output_ports(&self) -> Vec<(String, crate::core::PortType)> {
        vec![("out".into(), crate::core::PortType::Data)]
    }
}

// Export the plugin
caret_plugins::export_plugin!(MyPlugin);
```

### 4. Build Plugin

```bash
cargo build --release
```

The compiled library will be at:
- Linux: `target/release/libmy_caret_plugin.so`
- macOS: `target/release/libmy_caret_plugin.dylib`
- Windows: `target/release/my_caret_plugin.dll`

## Installing Plugins

### Install from File

```bash
caret plugin install target/release/libmy_caret_plugin.so
```

### Install from Git

```bash
caret plugin install --git https://github.com/example/my-caret-plugin.git
```

### Install from Registry

```bash
caret plugin install caret-plugin-example
```

## Using Plugins

### List Installed Plugins

```bash
$ caret plugin list

Installed plugins:
  my_caret_plugin v0.1.0
    - uppercase
    - reverse
```

### Use in Graph

```json
{
  "nodes": [
    {
      "id": "input",
      "type": "source"
    },
    {
      "id": "upper",
      "type": "uppercase"
    },
    {
      "id": "output",
      "type": "sink"
    }
  ],
  "edges": [
    {"from": "input", "to": "upper"},
    {"from": "upper", "to": "output"}
  ]
}
```

## Plugin Configuration

Plugins can accept configuration:

```rust
impl Node for UppercaseNode {
    // ... existing code ...

    fn configure(&mut self, config: &Value) -> Result<()> {
        // Read configuration
        if let Some(locale) = config.get("locale") {
            // Apply locale-specific rules
        }
        Ok(())
    }
}
```

Use in graph:

```json
{
  "id": "upper",
  "type": "uppercase",
  "config": {
    "locale": "tr-TR"
  }
}
```

## Advanced Plugin Features

### Stateful Nodes

Maintain state across invocations:

```rust
pub struct CounterNode {
    count: AtomicU64,
}

impl CounterNode {
    pub fn new() -> Self {
        CounterNode { count: AtomicU64::new(0) }
    }
}

impl Node for CounterNode {
    fn process(&mut self, input: &mut PortSet, output: &mut PortSet) -> Result<()> {
        let current = self.count.fetch_add(1, Ordering::SeqCst);
        // Use current count...
    }
}
```

### Async Nodes

Perform async operations:

```rust
pub struct HttpNode {
    client: reqwest::Client,
}

impl Node for HttpNode {
    fn process_async<'a>(
        &'a mut self,
        input: &'a mut PortSet,
        output: &'a mut PortSet
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            // Async HTTP request...
            Ok(())
        })
    }
}
```

### Custom Transforms

```rust
use caret_core::Transform;

pub struct Base64Transform;

impl Transform for Base64Transform {
    fn transform(&self, value: Value) -> Result<Value> {
        match value {
            Value::Bytes(data) => {
                let encoded = base64::encode(&data);
                Ok(Value::String(encoded.into()))
            }
            _ => Err(Error::InvalidType),
        }
    }
}

// Register in plugin
fn register_transforms(&self, registry: &mut TransformRegistry) {
    registry.register("base64_encode", Box::new(Base64Transform));
}
```

## Plugin Best Practices

### 1. Error Handling

Always return `Result` with descriptive errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
```

### 2. Resource Cleanup

Implement `Drop` for cleanup:

```rust
impl Drop for MyNode {
    fn drop(&mut self) {
        // Close connections, free resources
    }
}
```

### 3. Thread Safety

Ensure nodes are `Send + Sync`:

```rust
pub struct MyNode {
    // Use Arc<Mutex<T>> for shared state
    state: Arc<Mutex<State>>,
}
```

### 4. Documentation

Document your plugin:

```rust
/// Converts strings to uppercase using locale-aware rules.
///
/// # Configuration
///
/// - `locale`: Optional locale string (e.g., "tr-TR")
///
/// # Example
///
/// ```json
/// {
///   "type": "uppercase",
///   "config": {"locale": "en-US"}
/// }
/// ```
pub struct UppercaseNode;
```

## Plugin Distribution

### Package Structure

```
my-caret-plugin/
├── Cargo.toml
├── README.md
├── LICENSE
├── src/
│   └── lib.rs
├── examples/
│   └── usage.json
└── tests/
    └── integration_test.rs
```

### Publishing

1. Tag release in git
2. Build for all platforms
3. Upload to GitHub Releases
4. Register in Caret plugin registry

## Examples

### File Watcher Plugin

```rust
pub struct FileWatcherNode {
    watcher: RecommendedWatcher,
}

impl FileWatcherNode {
    pub fn new(path: PathBuf) -> Result<Self> {
        let (tx, rx) = channel(100);
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;
        watcher.watch(&path, RecursiveMode::Recursive)?;
        Ok(FileWatcherNode { watcher })
    }
}
```

### Database Plugin

```rust
pub struct DatabaseSink {
    pool: PgPool,
}

impl Node for DatabaseSink {
    fn process(&mut self, input: &mut PortSet, _output: &mut PortSet) -> Result<()> {
        while let Some(packet) = input.take("in")? {
            let query = packet_to_query(&packet)?;
            self.pool.execute(query).await?;
        }
        Ok(())
    }
}
```

## Troubleshooting

### Plugin Not Found

```bash
# Check plugin path
caret plugin list

# Verify plugin is installed
caret plugin info my_caret_plugin

# Check file exists
ls ~/.caret/plugins/
```

### Symbol Not Found

Ensure plugin is built with matching Rust version:

```bash
# Check Caret's Rust version
caret --version | grep rust

# Build plugin with same version
rustup default 1.75.0
cargo build --release
```

### Runtime Crash

Check plugin logs:

```bash
caret run --verbose my_graph.json
```

## Resources

- [caret_core API docs](https://docs.rs/caret-core/)
- [caret_plugins API docs](https://docs.rs/caret-plugins/)
- [Plugin Examples](https://github.com/staticpayload/caret-eternity/tree/main/examples/plugins)
- [Contributing](../contributing.md)

## Next Steps

- [Developer Guide](../dev/) - Core development
- [Examples](../../examples/) - Example plugins
- [Architecture](../architecture.md) - Plugin architecture details
