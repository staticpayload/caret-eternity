# Coding Style

This document defines the coding style for the Caret project.

## General Principles

1. **Readability counts more than cleverness**
2. **Explicit is better than implicit**
3. **Keep functions focused and small**
4. **Prefer composition over inheritance**
5. **Document public APIs**

## Formatting

Use standard Rust formatting:

```bash
cargo fmt
```

### Line Length

- Maximum line length: **100 characters**
- Use `#![allow(clippy::too_long_first_span)]` sparingly

### Indentation

- Use **4 spaces** (no tabs)
- Align function parameters where readable

```rust
// Good
fn process_data(
    input: &Input,
    config: &Config,
    output: &mut Output,
) -> Result<()> {
    // ...
}

// Also acceptable for simple cases
fn process_data(input: &Input, config: &Config, output: &mut Output) -> Result<()> {
    // ...
}
```

## Naming Conventions

### Modules

```rust
mod network_transport;   // snake_case
mod tcp_transport;       // snake_case
mod mDNSDiscovery;       // Avoid - use mdns_discovery
```

### Types

```rust
struct TcpTransport { }      // PascalCase
enum MessagePayload { }       // PascalCase
type PortId = String;         // PascalCase for type aliases
```

### Functions and Methods

```rust
fn process_packet() { }           // snake_case
fn get_node_by_id() { }           // snake_case
pub fn create_transport() { }     // snake_case
```

### Constants

```rust
const MAX_BUFFER_SIZE: usize = 1024;      // SCREAMING_SNAKE_CASE
const DEFAULT_PORT: u16 = 9234;           // SCREAMING_SNAKE_CASE
```

### Acronyms

Capitalize each letter in acronyms:

```rust
struct TcpTransport { }     // Good
struct TCPTransport { }     // Bad
struct TcpTransport { }     // Good

fn parse_html() { }         // Good
fn parseHTML() { }          // Bad

struct XmlDocument { }      // Good
struct XMLDocument { }      // Bad
```

## Code Organization

### File Structure

```rust
// 1. License header
// 2. Module documentation
// 3. Imports (grouped and sorted)
// 4. Type definitions
// 5. Implementation
// 6. Tests
```

### Import Order

1. Standard library
2. Third-party crates
3. Local crates
4. Re-exports

```rust
// Standard library
use std::collections::HashMap;
use std::sync::Arc;

// Third-party
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

// Local
use caret_core::Node;
use caret_graph::Graph;

// Re-exports
pub use self::transport::Transport;
```

### Module Re-exports

Prefer re-exports for common types:

```rust
// In crate root
pub use self::{
    node::Node,
    port::Port,
    packet::Packet,
};
```

## Documentation

### Public Items

All public items must have documentation:

```rust
/// Processes a single packet through the node.
///
/// This function takes a packet from the input port, transforms it
/// according to the node's configuration, and outputs the result
/// to the output port.
///
/// # Arguments
///
/// * `packet` - The packet to process
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if processing fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The packet type is incompatible
/// - The transformation fails
///
/// # Examples
///
/// ```
/// use caret_core::{Node, Packet};
///
/// let mut node = MyNode::new();
/// let result = node.process_packet(packet);
/// assert!(result.is_ok());
/// ```
pub fn process_packet(&mut self, packet: Packet) -> Result<()> {
    // ...
}
```

### Module Documentation

```rust
//! TCP transport implementation.
//!
//! This module provides TCP-based network transport for Caret's
//! distributed execution. It supports both client and server modes
//! with automatic reconnection and keepalive.
//!
//! # Example
//!
//! ```
//! use caret_distributed::TcpTransport;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let transport = TcpTransport::bind("0.0.0.0:9234").await?;
//! # Ok(())
//! # }
//! ```
```

## Error Handling

### Use Result for Fallible Operations

```rust
// Good
pub fn process(&mut self) -> Result<()> {
    let value = self.get_value()?;
    self.transform(value)?;
    Ok(())
}

// Bad - panic in library code
pub fn process(&mut self) {
    let value = self.get_value().unwrap();
    self.transform(value).unwrap();
}
```

### Provide Context

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("I/O error for node {node_id}: {0}")]
    Io {
        node_id: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Port {port} not found on node {node}")]
    PortNotFound { port: String, node: String },
}
```

### Never Panic in Library Code

```rust
// Good
pub fn get_port(&self, name: &str) -> Option<&Port> {
    self.ports.get(name)
}

// Bad
pub fn get_port(&self, name: &str) -> &Port {
    self.ports.get(name).expect("port not found")
}
```

## Concurrency

### Use Arc for Shared Immutable Data

```rust
use std::sync::Arc;

pub struct SharedConfig {
    inner: Arc<Config>,
}
```

### Use Mutex/RwLock for Shared Mutable State

```rust
use parking_lot::Mutex;  // Prefer parking_lot over std

pub struct State {
    nodes: Mutex<HashMap<NodeId, Node>>,
}
```

### Use Channels for Communication

```rust
use tokio::sync::mpsc;

pub struct NodeRunner {
    tx: mpsc::Sender<Packet>,
}

pub struct NodeHandle {
    rx: mpsc::Receiver<Packet>,
}
```

## Async/Await

### Use Tokio for Async

```rust
use tokio::net::TcpListener;

pub async fn serve(addr: &str) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    // ...
}
```

### Prefer Structured Concurrency

```rust
// Good - structured
pub async fn run(&self) -> Result<()> {
    tokio::select! {
        _ = self.serve() => {},
        _ = self.shutdown.recv() => {},
    }
}

// Avoid - unstructured
pub async fn run(&self) -> Result<()> {
    tokio::spawn(self.serve());
    tokio::spawn(self.monitor());
    // Tasks not awaited
}
```

## Performance

### Prefer References Over Cloning

```rust
// Good
fn process(value: &Value) -> Result<()> {
    // ...
}

// Acceptable when needed
fn process(value: Value) -> Result<()> {
    // ...
}
```

### Use Cow for Conditional Ownership

```rust
use std::borrow::Cow;

fn transform(s: Cow<str>) -> Cow<str> {
    if s.contains("bad") {
        Cow::Owned(s.replace("bad", "good"))
    } else {
        s
    }
}
```

### Avoid allocations in hot paths

```rust
// Good - reuses buffer
struct Processor {
    buffer: Vec<u8>,
}

impl Processor {
    fn process(&mut self, data: &[u8]) -> &[u8] {
        self.buffer.clear();
        self.buffer.extend_from_slice(data);
        &self.buffer
    }
}

// Bad - allocates each time
fn process(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}
```

## Macros

### Prefer Functions Over Macros

Only use macros when:
- Code repetition is unavoidable
- Token-level manipulation is needed
- Compile-time evaluation is required

### Macro Naming

```rust
// Declaration macros (snake_case)
macro_rules! generate_node {
    // ...
}

// Derive macros (PascalCase)
#[derive(CustomNode)]
pub struct MyNode;
```

## Testing

### Test Naming

```rust
#[test]
fn test_process_single_packet_returns_success() {
    // Describe: what is being tested
}

#[test]
fn test_process_with_invalid_type_returns_error() {
    // Describe: expected behavior
}
```

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    mod unit_tests {
        use super::*;

        #[test]
        fn test_something() { }
    }

    mod integration_tests {
        use super::*;

        #[tokio::test]
        async fn test_async_operation() { }
    }
}
```

## Clippy Lints

### Enable Strict Lints

```rust
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
```

### Allow Specific Lints (when needed)

```rust
#![allow(clippy::module_name_repetitions)]  // For clarity
#![allow(clippy::too_many_arguments)]       // When necessary
```

## Common Patterns

### Builder Pattern

```rust
pub struct Transport {
    // Private fields
}

impl Transport {
    pub fn builder() -> Builder {
        Builder::default()
    }
}

#[derive(Default)]
pub struct Builder {
    addr: Option<String>,
    timeout: Option<Duration>,
}

impl Builder {
    pub fn addr(mut self, addr: impl Into<String>) -> Self {
        self.addr = Some(addr.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Result<Transport> {
        // Build and validate
    }
}
```

### Newtype Pattern

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u64);

impl NodeId {
    pub fn new() -> Self {
        Self(rand::random())
    }
}
```

## Review Checklist

Before submitting code, check:

- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] All tests pass (`cargo test`)
- [ ] Public APIs documented
- [ ] Error handling is proper
- [ ] No unwraps/panics in library code
- [ ] Async code is properly awaited
- [ ] Resources are cleaned up

## Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Effective Rust](https://doc.rust-lang.org/book/ch13-00-performance.html)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
