// Caret Graph - Topology management
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::port::PortId;
use caret_core::{Error, Result};
use indexmap::IndexSet;
use std::collections::{HashMap, HashSet};

/// An edge in the graph
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Edge {
    /// Source node ID
    pub from_node: u64,
    /// Source port ID
    pub from_port: PortId,
    /// Target node ID
    pub to_node: u64,
    /// Target port ID
    pub to_port: PortId,
}

impl Edge {
    /// Create a new edge
    pub fn new(from_node: u64, from_port: PortId, to_node: u64, to_port: PortId) -> Self {
        Self {
            from_node,
            from_port,
            to_node,
            to_port,
        }
    }

    /// Reverse this edge (for topological sort)
    pub fn reversed(&self) -> Edge {
        Edge {
            from_node: self.to_node,
            from_port: self.to_port,
            to_node: self.from_node,
            to_port: self.from_port,
        }
    }
}

/// Error during connection operations
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionError {
    /// Port already connected
    AlreadyConnected,
    /// Would create a cycle
    WouldCreateCycle,
    /// Node not found
    NodeNotFound,
}

/// Graph topology
///
/// Manages the connections between nodes in the graph.
#[derive(Clone, Default)]
pub struct Topology {
    /// All edges
    edges: HashSet<Edge>,
    /// Edges by source node
    edges_from: HashMap<u64, Vec<Edge>>,
    /// Edges by target node
    edges_to: HashMap<u64, Vec<Edge>>,
    /// Input port connections (node -> port -> edge)
    input_connections: HashMap<(u64, PortId), Edge>,
    /// Output port connections (node -> port -> edge)
    output_connections: HashMap<(u64, PortId), Vec<Edge>>,
}

impl Topology {
    /// Create a new empty topology
    pub fn new() -> Self {
        Self {
            edges: HashSet::new(),
            edges_from: HashMap::new(),
            edges_to: HashMap::new(),
            input_connections: HashMap::new(),
            output_connections: HashMap::new(),
        }
    }

    /// Connect two ports
    pub fn connect(
        &mut self,
        from_node: u64,
        from_port: PortId,
        to_node: u64,
        to_port: PortId,
    ) -> Result<()> {
        // Check if input port is already connected
        let key = (to_node, to_port);
        if self.input_connections.contains_key(&key) {
            return Err(Error::graph("input port is already connected"));
        }

        let edge = Edge::new(from_node, from_port, to_node, to_port);

        // Check for cycles
        let mut test_topology = self.clone();
        test_topology.add_edge_internal(edge.clone());
        if test_topology.has_cycles() {
            return Err(Error::graph("connection would create a cycle"));
        }

        self.add_edge_internal(edge);
        Ok(())
    }

    /// Internal edge addition
    fn add_edge_internal(&mut self, edge: Edge) {
        self.edges.insert(edge.clone());

        self.edges_from
            .entry(edge.from_node)
            .or_insert_with(Vec::new)
            .push(edge.clone());

        self.edges_to
            .entry(edge.to_node)
            .or_insert_with(Vec::new)
            .push(edge.clone());

        self.input_connections
            .insert((edge.to_node, edge.to_port), edge.clone());

        self.output_connections
            .entry((edge.from_node, edge.from_port))
            .or_insert_with(Vec::new)
            .push(edge);
    }

    /// Disconnect two ports
    pub fn disconnect(
        &mut self,
        from_node: u64,
        from_port: PortId,
        to_node: u64,
        to_port: PortId,
    ) -> Result<()> {
        let edge = Edge::new(from_node, from_port, to_node, to_port);

        if !self.edges.contains(&edge) {
            return Err(Error::graph("edge not found"));
        }

        self.remove_edge_internal(&edge);
        Ok(())
    }

    /// Internal edge removal
    fn remove_edge_internal(&mut self, edge: &Edge) {
        self.edges.remove(edge);

        if let Some(edges) = self.edges_from.get_mut(&edge.from_node) {
            edges.retain(|e| e != edge);
            if edges.is_empty() {
                self.edges_from.remove(&edge.from_node);
            }
        }

        if let Some(edges) = self.edges_to.get_mut(&edge.to_node) {
            edges.retain(|e| e != edge);
            if edges.is_empty() {
                self.edges_to.remove(&edge.to_node);
            }
        }

        self.input_connections.remove(&(edge.to_node, edge.to_port));

        if let Some(edges) = self
            .output_connections
            .get_mut(&(edge.from_node, edge.from_port))
        {
            edges.retain(|e| e != edge);
            if edges.is_empty() {
                self.output_connections
                    .remove(&(edge.from_node, edge.from_port));
            }
        }
    }

    /// Disconnect all edges for a node
    pub fn disconnect_node(&mut self, node_id: u64) {
        // Collect all edges to remove
        let to_remove: Vec<Edge> = self
            .edges
            .iter()
            .filter(|e| e.from_node == node_id || e.to_node == node_id)
            .cloned()
            .collect();

        for edge in to_remove {
            self.remove_edge_internal(&edge);
        }
    }

    /// Get all edges
    pub fn edges(&self) -> &HashSet<Edge> {
        &self.edges
    }

    /// Get edges from a node
    pub fn edges_from(&self, node_id: u64) -> &[Edge] {
        self.edges_from
            .get(&node_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get edges to a node
    pub fn edges_to(&self, node_id: u64) -> &[Edge] {
        self.edges_to
            .get(&node_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Check if topology has cycles
    pub fn has_cycles(&self) -> bool {
        // Build adjacency list
        let mut adj: HashMap<u64, Vec<u64>> = HashMap::new();
        let mut all_nodes: HashSet<u64> = HashSet::new();

        for edge in &self.edges {
            adj.entry(edge.from_node)
                .or_insert_with(Vec::new)
                .push(edge.to_node);
            all_nodes.insert(edge.from_node);
            all_nodes.insert(edge.to_node);
        }

        // Use DFS to detect cycles
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for &node in &all_nodes {
            if !visited.contains(&node) {
                if self.has_cycle_dfs(node, &adj, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }

        false
    }

    fn has_cycle_dfs(
        &self,
        node: u64,
        adj: &HashMap<u64, Vec<u64>>,
        visited: &mut HashSet<u64>,
        rec_stack: &mut HashSet<u64>,
    ) -> bool {
        visited.insert(node);
        rec_stack.insert(node);

        if let Some(neighbors) = adj.get(&node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.has_cycle_dfs(*neighbor, adj, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(&node);
        false
    }

    /// Get topological order of nodes
    ///
    /// Returns None if the graph has cycles.
    pub fn topological_order(&self) -> Option<Vec<u64>> {
        if self.has_cycles() {
            return None;
        }

        // Kahn's algorithm
        let mut in_degree: HashMap<u64, usize> = HashMap::new();
        let mut all_nodes: IndexSet<u64> = IndexSet::new();

        // Collect all nodes and calculate in-degrees
        for edge in &self.edges {
            all_nodes.insert(edge.from_node);
            all_nodes.insert(edge.to_node);
            *in_degree.entry(edge.to_node).or_insert(0) += 1;
            in_degree.entry(edge.from_node).or_insert(0);
        }

        // Start with nodes that have no incoming edges
        let mut queue: Vec<u64> = all_nodes
            .iter()
            .filter(|n| in_degree.get(n).map_or(0, |&d| d) == 0)
            .copied()
            .collect();

        let mut result = Vec::new();

        while let Some(node) = queue.pop() {
            result.push(node);

            for edge in self.edges_from(node) {
                if let Some(degree) = in_degree.get_mut(&edge.to_node) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(edge.to_node);
                    }
                }
            }
        }

        if result.len() != all_nodes.len() {
            // Cycle detected
            None
        } else {
            Some(result)
        }
    }

    /// Get the number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Check if topology is empty
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Clear all edges
    pub fn clear(&mut self) {
        self.edges.clear();
        self.edges_from.clear();
        self.edges_to.clear();
        self.input_connections.clear();
        self.output_connections.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_create() {
        let edge = Edge::new(1, PortId::new(10), 2, PortId::new(20));
        assert_eq!(edge.from_node, 1);
        assert_eq!(edge.to_node, 2);
    }

    #[test]
    fn test_topology_empty() {
        let topo = Topology::new();
        assert!(topo.is_empty());
        assert_eq!(topo.edge_count(), 0);
    }

    #[test]
    fn test_topology_connect() {
        let mut topo = Topology::new();
        let result = topo.connect(1, PortId::new(10), 2, PortId::new(20));
        assert!(result.is_ok());
        assert_eq!(topo.edge_count(), 1);
    }

    #[test]
    fn test_topology_connect_input_twice_fails() {
        let mut topo = Topology::new();
        let p1 = PortId::new(10);
        let p2 = PortId::new(20);
        let p_in = PortId::new(100);

        assert!(topo.connect(1, p1, 2, p_in).is_ok());
        // Same input port connected again
        assert!(topo.connect(3, p2, 2, p_in).is_err());
    }

    #[test]
    fn test_topology_disconnect() {
        let mut topo = Topology::new();
        let p1 = PortId::new(10);
        let p2 = PortId::new(20);
        topo.connect(1, p1, 2, p2).unwrap();
        assert_eq!(topo.edge_count(), 1);

        topo.disconnect(1, p1, 2, p2).unwrap();
        assert!(topo.is_empty());
    }

    #[test]
    fn test_topology_has_cycle() {
        let mut topo = Topology::new();
        let p1 = PortId::new(10);
        let p2 = PortId::new(20);
        let p3 = PortId::new(30);
        let p4 = PortId::new(40);

        // Try to create a cycle: 1 -> 2 -> 3 -> 1
        topo.connect(1, p1, 2, p2).unwrap();
        topo.connect(2, p3, 3, p4).unwrap();
        // This connection would create a cycle and should fail
        assert!(topo.connect(3, p3, 1, p4).is_err());
        // Verify the graph is still acyclic since the cycle was rejected
        assert!(!topo.has_cycles());
    }

    #[test]
    fn test_topology_detects_existing_cycle() {
        // Test that has_cycles() works when a cycle actually exists
        let mut topo = Topology::new();
        // Manually construct a cycle by adding edges through internal state
        let edge1 = Edge::new(1, PortId::new(10), 2, PortId::new(20));
        let edge2 = Edge::new(2, PortId::new(30), 3, PortId::new(40));
        let edge3 = Edge::new(3, PortId::new(50), 1, PortId::new(60));

        // Use internal modification to create a cycle
        topo.edges.insert(edge1.clone());
        topo.edges.insert(edge2.clone());
        topo.edges.insert(edge3.clone());
        topo.edges_from
            .entry(1)
            .or_insert_with(Vec::new)
            .push(edge1.clone());
        topo.edges_from
            .entry(2)
            .or_insert_with(Vec::new)
            .push(edge2.clone());
        topo.edges_from
            .entry(3)
            .or_insert_with(Vec::new)
            .push(edge3.clone());

        assert!(topo.has_cycles());
    }

    #[test]
    fn test_topology_acyclic() {
        let mut topo = Topology::new();
        let p1 = PortId::new(10);
        let p2 = PortId::new(20);
        let p3 = PortId::new(30);

        // Linear chain: 1 -> 2 -> 3
        topo.connect(1, p1, 2, p2).unwrap();
        topo.connect(2, p2, 3, p3).unwrap();
        assert!(!topo.has_cycles());
    }

    #[test]
    fn test_topology_topological_order() {
        let mut topo = Topology::new();
        let p1 = PortId::new(10);
        let p2 = PortId::new(20);
        let p3 = PortId::new(30);

        // Chain: 1 -> 2 -> 3
        topo.connect(1, p1, 2, p2).unwrap();
        topo.connect(2, p2, 3, p3).unwrap();

        let order = topo.topological_order();
        assert_eq!(order, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_topology_disconnect_node() {
        let mut topo = Topology::new();
        let p1 = PortId::new(10);
        let p2 = PortId::new(20);
        let p3 = PortId::new(30);
        let p4 = PortId::new(40);

        // 1 -> 2 -> 3
        topo.connect(1, p1, 2, p2).unwrap();
        topo.connect(2, p3, 3, p4).unwrap();

        topo.disconnect_node(2);
        assert!(topo.is_empty());
    }
}
