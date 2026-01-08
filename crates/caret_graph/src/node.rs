// Caret Graph - Node types
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::port::{Port, PortDirection};
use caret_core::Result;
use std::collections::HashMap;
use std::fmt;

/// Unique identifier for a node
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId {
    id: u64,
}

impl NodeId {
    /// Create a new node ID
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    /// Get the numeric ID
    pub fn as_u64(&self) -> u64 {
        self.id
    }

    /// Generate a unique node ID
    pub fn unique() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "node-{}", self.id)
    }
}

/// Type of node
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeType {
    /// Source node (produces data)
    Source,
    /// Transform node (processes data)
    Transform,
    /// Sink node (consumes data)
    Sink,
    /// Custom node type
    Custom(String),
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source => write!(f, "source"),
            Self::Transform => write!(f, "transform"),
            Self::Sink => write!(f, "sink"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Handle to a node in the graph
///
/// Node handles are used to reference nodes when connecting ports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeHandle {
    /// Node ID
    pub node_id: NodeId,
}

impl NodeHandle {
    /// Create a new node handle
    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }

    /// Get the node ID
    pub fn id(&self) -> NodeId {
        self.node_id
    }
}

/// A node in the graph
///
/// Nodes are the processing units in a Caret pipeline. Each node
/// has input ports, output ports, and a type.
#[derive(Clone)]
pub struct Node {
    /// Unique node identifier
    id: NodeId,
    /// Node name
    name: String,
    /// Node type
    node_type: NodeType,
    /// Input ports
    inputs: HashMap<String, Port>,
    /// Output ports
    outputs: HashMap<String, Port>,
}

impl Node {
    /// Create a new node
    pub fn new(name: impl Into<String>, node_type: NodeType) -> Self {
        let name = name.into();
        Self {
            id: NodeId::unique(),
            name,
            node_type,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        }
    }

    /// Create a new source node
    pub fn source(name: impl Into<String>) -> Self {
        Self::new(name, NodeType::Source)
    }

    /// Create a new transform node
    pub fn transform(name: impl Into<String>) -> Self {
        Self::new(name, NodeType::Transform)
    }

    /// Create a new sink node
    pub fn sink(name: impl Into<String>) -> Self {
        Self::new(name, NodeType::Sink)
    }

    /// Get the node ID
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Get the node name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the node type
    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }

    /// Add an input port
    pub fn add_input(&mut self, name: impl Into<String>) -> Result<Port> {
        let name = name.into();
        if self.inputs.contains_key(&name) {
            return Err(caret_core::Error::invalid_input(format!(
                "input port '{}' already exists",
                name
            )));
        }
        let port = Port::new(&name, PortDirection::In);
        self.inputs.insert(name, port.clone());
        Ok(port)
    }

    /// Add an output port
    pub fn add_output(&mut self, name: impl Into<String>) -> Result<Port> {
        let name = name.into();
        if self.outputs.contains_key(&name) {
            return Err(caret_core::Error::invalid_input(format!(
                "output port '{}' already exists",
                name
            )));
        }
        let port = Port::new(&name, PortDirection::Out);
        self.outputs.insert(name, port.clone());
        Ok(port)
    }

    /// Get an input port by name
    pub fn input(&self, name: &str) -> Option<&Port> {
        self.inputs.get(name)
    }

    /// Get an output port by name
    pub fn output(&self, name: &str) -> Option<&Port> {
        self.outputs.get(name)
    }

    /// Get all input ports
    pub fn inputs(&self) -> &HashMap<String, Port> {
        &self.inputs
    }

    /// Get all output ports
    pub fn outputs(&self) -> &HashMap<String, Port> {
        &self.outputs
    }

    /// Get the number of input ports
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// Get the number of output ports
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }

    /// Create a handle for this node
    pub fn handle(&self) -> NodeHandle {
        NodeHandle::new(self.id)
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("node_type", &self.node_type)
            .field("input_count", &self.inputs.len())
            .field("output_count", &self.outputs.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        let id1 = NodeId::new(42);
        assert_eq!(id1.as_u64(), 42);
        assert_eq!(format!("{}", id1), "node-42");

        let id2 = NodeId::unique();
        let id3 = NodeId::unique();
        assert_ne!(id2.as_u64(), id3.as_u64());
    }

    #[test]
    fn test_node_create() {
        let node = Node::transform("passthrough");
        assert_eq!(node.name(), "passthrough");
        assert_eq!(node.node_type(), &NodeType::Transform);
        assert_eq!(node.input_count(), 0);
        assert_eq!(node.output_count(), 0);
    }

    #[test]
    fn test_node_add_ports() {
        let mut node = Node::transform("process");
        assert!(node.add_input("in").is_ok());
        assert!(node.add_output("out").is_ok());

        assert_eq!(node.input_count(), 1);
        assert_eq!(node.output_count(), 1);
        assert!(node.input("in").is_some());
        assert!(node.output("out").is_some());
    }

    #[test]
    fn test_node_duplicate_port_fails() {
        let mut node = Node::transform("process");
        assert!(node.add_input("in").is_ok());
        assert!(node.add_input("in").is_err());
    }

    #[test]
    fn test_node_source_sink() {
        let source = Node::source("camera");
        assert_eq!(source.node_type(), &NodeType::Source);

        let sink = Node::sink("display");
        assert_eq!(sink.node_type(), &NodeType::Sink);
    }

    #[test]
    fn test_node_handle() {
        let node = Node::transform("process");
        let handle = node.handle();
        assert_eq!(handle.id(), node.id());
    }

    #[test]
    fn test_node_type_display() {
        assert_eq!(format!("{}", NodeType::Source), "source");
        assert_eq!(format!("{}", NodeType::Transform), "transform");
        assert_eq!(format!("{}", NodeType::Sink), "sink");
        assert_eq!(format!("{}", NodeType::Custom("filter".into())), "filter");
    }
}
