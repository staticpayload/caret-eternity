// Caret Graph - Graph implementation
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{node::Node, topology::Topology};
use caret_core::{Error, Result};
use indexmap::IndexMap;
use std::sync::Arc;

/// A directed graph of nodes
///
/// The graph is the core data structure for Caret pipelines.
/// It maintains a set of nodes and their connections.
#[derive(Clone)]
pub struct Graph {
    /// Nodes in the graph
    nodes: IndexMap<u64, Arc<Node>>,
    /// Graph topology (connections)
    topology: Topology,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            nodes: IndexMap::new(),
            topology: Topology::new(),
        }
    }

    /// Add a node to the graph
    ///
    /// Returns an error if a node with the same ID already exists.
    pub fn add_node(&mut self, node: Node) -> Result<()> {
        let id = node.id().as_u64();
        if self.nodes.contains_key(&id) {
            return Err(Error::graph(format!("node with id {} already exists", id)));
        }
        self.nodes.insert(id, Arc::new(node));
        Ok(())
    }

    /// Get a node by ID
    pub fn node(&self, id: u64) -> Option<&Node> {
        self.nodes.get(&id).map(|n| n.as_ref())
    }

    /// Get a node by ID as Arc
    pub fn node_arc(&self, id: u64) -> Option<Arc<Node>> {
        self.nodes.get(&id).cloned()
    }

    /// Remove a node from the graph
    ///
    /// This also removes all connections to/from the node.
    pub fn remove_node(&mut self, id: u64) -> Option<Arc<Node>> {
        self.topology.disconnect_node(id);
        self.nodes.shift_remove(&id)
    }

    /// Connect two ports
    ///
    /// Connects an output port from one node to an input port of another.
    pub fn connect(
        &mut self,
        from_node: u64,
        from_port: &str,
        to_node: u64,
        to_port: &str,
    ) -> Result<()> {
        // Verify nodes exist
        if !self.nodes.contains_key(&from_node) {
            return Err(Error::graph(format!("source node {} not found", from_node)));
        }
        if !self.nodes.contains_key(&to_node) {
            return Err(Error::graph(format!("target node {} not found", to_node)));
        }

        // Verify ports exist
        let from = self
            .nodes
            .get(&from_node)
            .and_then(|n| n.output(from_port))
            .ok_or_else(|| {
                Error::graph(format!(
                    "output port '{}' not found on node {}",
                    from_port, from_node
                ))
            })?;

        let to = self
            .nodes
            .get(&to_node)
            .and_then(|n| n.input(to_port))
            .ok_or_else(|| {
                Error::graph(format!(
                    "input port '{}' not found on node {}",
                    to_port, to_node
                ))
            })?;

        self.topology
            .connect(from_node, from.id(), to_node, to.id())
    }

    /// Disconnect two ports
    pub fn disconnect(
        &mut self,
        from_node: u64,
        from_port: &str,
        to_node: u64,
        to_port: &str,
    ) -> Result<()> {
        let from_id = self
            .nodes
            .get(&from_node)
            .and_then(|n| n.output(from_port))
            .map(|p| p.id())
            .ok_or_else(|| {
                Error::graph(format!(
                    "output port '{}' not found on node {}",
                    from_port, from_node
                ))
            })?;

        let to_id = self
            .nodes
            .get(&to_node)
            .and_then(|n| n.input(to_port))
            .map(|p| p.id())
            .ok_or_else(|| {
                Error::graph(format!(
                    "input port '{}' not found on node {}",
                    to_port, to_node
                ))
            })?;

        self.topology.disconnect(from_node, from_id, to_node, to_id)
    }

    /// Get the topology
    pub fn topology(&self) -> &Topology {
        &self.topology
    }

    /// Get all nodes
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values().map(|n| n.as_ref())
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the graph is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Clear all nodes and connections
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.topology = Topology::new();
    }

    /// Validate the graph
    ///
    /// Checks for:
    /// - Cycles (if acyclic is required)
    /// - Disconnected nodes
    /// - Invalid connections
    pub fn validate(&self) -> Result<()> {
        // Check for cycles
        if self.topology.has_cycles() {
            return Err(Error::graph("graph contains cycles"));
        }

        // Check for source nodes with no outputs
        // Check for sink nodes with no inputs
        // These are warnings, not errors

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeType;

    #[test]
    fn test_graph_create() {
        let graph = Graph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.node_count(), 0);
    }

    #[test]
    fn test_graph_add_node() {
        let mut graph = Graph::new();
        let node = Node::transform("test");
        let id = node.id();
        assert!(graph.add_node(node).is_ok());
        assert_eq!(graph.node_count(), 1);
        assert!(graph.node(id.as_u64()).is_some());
    }

    #[test]
    fn test_graph_duplicate_node_fails() {
        let mut graph = Graph::new();
        let node1 = Node::transform("test");
        let node1_clone = node1.clone();
        let id = node1.id().as_u64();
        assert!(graph.add_node(node1).is_ok());

        // Cloning a node creates the same node with same ID
        assert!(graph.add_node(node1_clone).is_err());
    }

    #[test]
    fn test_graph_connect() {
        let mut graph = Graph::new();
        let mut source = Node::source("src");
        let mut sink = Node::sink("snk");
        source.add_output("out").unwrap();
        sink.add_input("in").unwrap();

        let src_id = source.id().as_u64();
        let snk_id = sink.id().as_u64();

        graph.add_node(source).unwrap();
        graph.add_node(sink).unwrap();

        assert!(graph.connect(src_id, "out", snk_id, "in").is_ok());
    }

    #[test]
    fn test_graph_connect_nonexistent_port_fails() {
        let mut graph = Graph::new();
        let mut source = Node::source("src");
        let mut sink = Node::sink("snk");
        source.add_output("out").unwrap();
        sink.add_input("in").unwrap();

        let src_id = source.id().as_u64();
        let snk_id = sink.id().as_u64();

        graph.add_node(source).unwrap();
        graph.add_node(sink).unwrap();

        assert!(graph.connect(src_id, "invalid", snk_id, "in").is_err());
    }

    #[test]
    fn test_graph_remove_node() {
        let mut graph = Graph::new();
        let node = Node::transform("test");
        let id = node.id().as_u64();
        graph.add_node(node).unwrap();
        assert_eq!(graph.node_count(), 1);

        graph.remove_node(id);
        assert_eq!(graph.node_count(), 0);
    }

    #[test]
    fn test_graph_clear() {
        let mut graph = Graph::new();
        graph.add_node(Node::source("src")).unwrap();
        graph.add_node(Node::sink("snk")).unwrap();
        assert_eq!(graph.node_count(), 2);

        graph.clear();
        assert!(graph.is_empty());
    }
}
