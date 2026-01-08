// Caret Graph - Port types
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::fmt;

/// Unique identifier for a port
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PortId {
    id: u64,
}

impl PortId {
    /// Create a new port ID
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    /// Get the numeric ID
    pub fn as_u64(&self) -> u64 {
        self.id
    }

    /// Generate a unique port ID
    pub fn unique() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
        }
    }
}

/// Direction of a port
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PortDirection {
    /// Input port (receives data)
    In,
    /// Output port (sends data)
    Out,
}

impl fmt::Display for PortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::In => write!(f, "in"),
            Self::Out => write!(f, "out"),
        }
    }
}

/// A port on a node
///
/// Ports are the connection points for edges in the graph.
/// Each port has a direction, name, and optional type information.
#[derive(Clone, Debug)]
pub struct Port {
    /// Unique port identifier
    id: PortId,
    /// Port name
    name: String,
    /// Port direction
    direction: PortDirection,
    /// Optional type descriptor
    type_name: Option<String>,
}

impl Port {
    /// Create a new port
    pub fn new(name: impl Into<String>, direction: PortDirection) -> Self {
        Self {
            id: PortId::unique(),
            name: name.into(),
            direction,
            type_name: None,
        }
    }

    /// Create a new port with a type descriptor
    pub fn with_type(
        name: impl Into<String>,
        direction: PortDirection,
        type_name: impl Into<String>,
    ) -> Self {
        Self {
            id: PortId::unique(),
            name: name.into(),
            direction,
            type_name: Some(type_name.into()),
        }
    }

    /// Get the port ID
    pub fn id(&self) -> PortId {
        self.id
    }

    /// Get the port name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the port direction
    pub fn direction(&self) -> PortDirection {
        self.direction
    }

    /// Get the type name if set
    pub fn type_name(&self) -> Option<&str> {
        self.type_name.as_deref()
    }

    /// Set the type name
    pub fn set_type(&mut self, type_name: impl Into<String>) {
        self.type_name = Some(type_name.into());
    }

    /// Check if this is an input port
    pub fn is_input(&self) -> bool {
        matches!(self.direction, PortDirection::In)
    }

    /// Check if this is an output port
    pub fn is_output(&self) -> bool {
        matches!(self.direction, PortDirection::Out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_id() {
        let id1 = PortId::new(42);
        assert_eq!(id1.as_u64(), 42);

        let id2 = PortId::unique();
        let id3 = PortId::unique();
        assert_ne!(id2.as_u64(), id3.as_u64());
    }

    #[test]
    fn test_port_create() {
        let port = Port::new("input", PortDirection::In);
        assert_eq!(port.name(), "input");
        assert!(port.is_input());
        assert!(!port.is_output());
        assert!(port.type_name().is_none());
    }

    #[test]
    fn test_port_with_type() {
        let port = Port::with_type("output", PortDirection::Out, "Packet");
        assert_eq!(port.name(), "output");
        assert!(port.is_output());
        assert_eq!(port.type_name(), Some("Packet"));
    }

    #[test]
    fn test_port_set_type() {
        let mut port = Port::new("data", PortDirection::In);
        assert!(port.type_name().is_none());
        port.set_type("Bytes");
        assert_eq!(port.type_name(), Some("Bytes"));
    }
}
