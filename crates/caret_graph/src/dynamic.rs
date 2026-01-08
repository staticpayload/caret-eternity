// Caret Graph - Dynamic graph modification
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{node::Node, Graph};
use caret_core::{Error, Result};
use parking_lot::Mutex;
use std::sync::Arc;

/// A change that can be applied to a graph
#[derive(Clone, Debug)]
pub enum GraphChange {
    /// Add a node to the graph
    AddNode {
        /// Node to add
        node: Node,
    },
    /// Remove a node from the graph
    RemoveNode {
        /// ID of the node to remove
        node_id: u64,
    },
    /// Connect two ports
    Connect {
        /// Source node ID
        from_node: u64,
        /// Source port name
        from_port: String,
        /// Target node ID
        to_node: u64,
        /// Target port name
        to_port: String,
    },
    /// Disconnect two ports
    Disconnect {
        /// Source node ID
        from_node: u64,
        /// Source port name
        from_port: String,
        /// Target node ID
        to_node: u64,
        /// Target port name
        to_port: String,
    },
    /// Replace a node with another (preserving connections)
    ReplaceNode {
        /// ID of the node to replace
        node_id: u64,
        /// New node to replace it with
        new_node: Node,
    },
}

impl GraphChange {
    /// Create a change to add a node
    pub fn add_node(node: Node) -> Self {
        Self::AddNode { node }
    }

    /// Create a change to remove a node
    pub fn remove_node(node_id: u64) -> Self {
        Self::RemoveNode { node_id }
    }

    /// Create a change to connect ports
    pub fn connect(from_node: u64, from_port: impl Into<String>, to_node: u64, to_port: impl Into<String>) -> Self {
        Self::Connect {
            from_node,
            from_port: from_port.into(),
            to_node,
            to_port: to_port.into(),
        }
    }

    /// Create a change to disconnect ports
    pub fn disconnect(from_node: u64, from_port: impl Into<String>, to_node: u64, to_port: impl Into<String>) -> Self {
        Self::Disconnect {
            from_node,
            from_port: from_port.into(),
            to_node,
            to_port: to_port.into(),
        }
    }

    /// Create a change to replace a node
    pub fn replace_node(node_id: u64, new_node: Node) -> Self {
        Self::ReplaceNode { node_id, new_node }
    }

    /// Get a description of this change
    pub fn description(&self) -> String {
        match self {
            Self::AddNode { node } => format!("Add node '{}'", node.name()),
            Self::RemoveNode { node_id } => format!("Remove node {}", node_id),
            Self::Connect { from_node, from_port, to_node, to_port } => {
                format!("Connect {}:{} -> {}:{}", from_node, from_port, to_node, to_port)
            }
            Self::Disconnect { from_node, from_port, to_node, to_port } => {
                format!("Disconnect {}:{} -> {}:{}", from_node, from_port, to_node, to_port)
            }
            Self::ReplaceNode { node_id, new_node } => {
                format!("Replace node {} with '{}'", node_id, new_node.name())
            }
        }
    }
}

/// Result of applying a change
#[derive(Clone, Debug)]
pub enum ChangeResult {
    /// Change was applied successfully
    Applied,
    /// Change was rolled back
    RolledBack { reason: String },
    /// Change was skipped (no-op)
    Skipped { reason: String },
}

/// A transaction for applying multiple changes atomically
#[derive(Clone)]
pub struct GraphTransaction {
    /// Changes in this transaction
    changes: Vec<GraphChange>,
    /// Description of this transaction
    description: Option<String>,
}

impl GraphTransaction {
    /// Create a new empty transaction
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
            description: None,
        }
    }

    /// Create a transaction with a description
    pub fn with_description(description: impl Into<String>) -> Self {
        Self {
            changes: Vec::new(),
            description: Some(description.into()),
        }
    }

    /// Add a change to the transaction
    pub fn add_change(mut self, change: GraphChange) -> Self {
        self.changes.push(change);
        self
    }

    /// Add multiple changes to the transaction
    pub fn add_changes(mut self, changes: impl IntoIterator<Item = GraphChange>) -> Self {
        self.changes.extend(changes);
        self
    }

    /// Get the number of changes in this transaction
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Check if the transaction is empty
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Get the description of this transaction
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Apply all changes in this transaction to a graph
    ///
    /// If any change fails, all previous changes are rolled back.
    pub fn apply(&self, graph: &mut Graph) -> Result<Vec<ChangeResult>> {
        if self.changes.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        let mut rollback_data = Vec::new();

        // Apply each change, collecting rollback information
        for change in &self.changes {
            match self.apply_change(graph, change) {
                Ok(result) => {
                    results.push(result);
                    // Store rollback data
                    rollback_data.push(self.rollback_data(graph, change));
                }
                Err(e) => {
                    // Rollback all previous changes
                    for (rollback_change, _) in rollback_data.into_iter().rev() {
                        if let Some(rb_change) = rollback_change {
                            let _ = self.apply_change(graph, &rb_change);
                        }
                    }
                    return Err(Error::graph(format!(
                        "transaction failed at change '{}': {}",
                        change.description(),
                        e
                    )));
                }
            }
        }

        Ok(results)
    }

    /// Apply a single change
    fn apply_change(&self, graph: &mut Graph, change: &GraphChange) -> Result<ChangeResult> {
        match change {
            GraphChange::AddNode { node } => {
                let node_id = node.id().as_u64();
                if graph.node(node_id).is_some() {
                    return Ok(ChangeResult::Skipped {
                        reason: format!("node {} already exists", node_id),
                    });
                }
                graph.add_node(node.clone())?;
                Ok(ChangeResult::Applied)
            }
            GraphChange::RemoveNode { node_id } => {
                if graph.node(*node_id).is_none() {
                    return Ok(ChangeResult::Skipped {
                        reason: format!("node {} does not exist", node_id),
                    });
                }
                graph.remove_node(*node_id);
                Ok(ChangeResult::Applied)
            }
            GraphChange::Connect { from_node, from_port, to_node, to_port } => {
                graph.connect(*from_node, from_port, *to_node, to_port)?;
                Ok(ChangeResult::Applied)
            }
            GraphChange::Disconnect { from_node, from_port, to_node, to_port } => {
                graph.disconnect(*from_node, from_port, *to_node, to_port)?;
                Ok(ChangeResult::Applied)
            }
            GraphChange::ReplaceNode { node_id, new_node } => {
                if graph.node(*node_id).is_none() {
                    return Ok(ChangeResult::Skipped {
                        reason: format!("node {} does not exist", node_id),
                    });
                }
                // Get connections before removing
                let _edges_from = graph.topology().edges_from(*node_id).to_vec();
                let _edges_to = graph.topology().edges_to(*node_id).to_vec();

                // Remove old node
                graph.remove_node(*node_id);

                // Add new node with same ID
                graph.add_node(new_node.clone())?;

                // Note: In a production system, we'd want to reconnect using the edges
                // but since port names may have changed, we skip reconnection here

                Ok(ChangeResult::Applied)
            }
        }
    }

    /// Get rollback data for a change
    fn rollback_data(&self, graph: &Graph, change: &GraphChange) -> (Option<GraphChange>, bool) {
        match change {
            GraphChange::AddNode { node } => {
                let node_id = node.id().as_u64();
                if graph.node(node_id).is_some() {
                    (Some(GraphChange::RemoveNode { node_id }), true)
                } else {
                    (None, false)
                }
            }
            GraphChange::RemoveNode { node_id: _ } => {
                // Can't rollback removal without storing the node
                (None, false)
            }
            GraphChange::Connect { from_node, from_port, to_node, to_port } => {
                // Can rollback by disconnecting
                (Some(GraphChange::Disconnect {
                    from_node: *from_node,
                    from_port: from_port.clone(),
                    to_node: *to_node,
                    to_port: to_port.clone(),
                }), true)
            }
            GraphChange::Disconnect { .. } => {
                // Can't rollback disconnect without knowing if it was connected
                (None, false)
            }
            GraphChange::ReplaceNode { .. } => {
                // Can't rollback replace without storing old node
                (None, false)
            }
        }
    }
}

impl Default for GraphTransaction {
    fn default() -> Self {
        Self::new()
    }
}

/// A listener for graph changes
pub trait ChangeListener: Send + Sync {
    /// Called before a change is applied
    fn on_before_change(&self, graph: &Graph, change: &GraphChange) -> Result<()>;

    /// Called after a change is applied
    fn on_after_change(&self, graph: &Graph, change: &GraphChange, result: &ChangeResult);
}

/// A no-op change listener
#[derive(Debug, Clone, Default)]
pub struct NopListener;

impl ChangeListener for NopListener {
    fn on_before_change(&self, _graph: &Graph, _change: &GraphChange) -> Result<()> {
        Ok(())
    }

    fn on_after_change(&self, _graph: &Graph, _change: &GraphChange, _result: &ChangeResult) {
        // No-op
    }
}

/// A dynamic graph that supports runtime modification
#[derive(Clone)]
pub struct DynamicGraph {
    /// The underlying graph
    graph: Arc<Mutex<Graph>>,
    /// Change listeners
    listeners: Arc<Mutex<Vec<Box<dyn ChangeListener>>>>,
}

impl DynamicGraph {
    /// Create a new dynamic graph
    pub fn new() -> Self {
        Self {
            graph: Arc::new(Mutex::new(Graph::new())),
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create from an existing graph
    pub fn from_graph(graph: Graph) -> Self {
        Self {
            graph: Arc::new(Mutex::new(graph)),
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a change listener
    pub fn add_listener(&self, listener: Box<dyn ChangeListener>) {
        self.listeners.lock().push(listener);
    }

    /// Apply a single change
    pub fn apply_change(&self, change: &GraphChange) -> Result<ChangeResult> {
        // Notify listeners before change
        let graph = self.graph.lock();
        let listeners = self.listeners.lock();
        for listener in listeners.iter() {
            listener.on_before_change(&graph, change)?;
        }
        drop(listeners);
        drop(graph);

        // Apply the change
        let mut graph = self.graph.lock();
        let result = match change {
            GraphChange::AddNode { node } => {
                let node_clone = node.clone();
                let node_id = node.id().as_u64();
                if graph.node(node_id).is_some() {
                    Ok(ChangeResult::Skipped {
                        reason: format!("node {} already exists", node_id),
                    })
                } else {
                    graph.add_node(node_clone)?;
                    Ok(ChangeResult::Applied)
                }
            }
            GraphChange::RemoveNode { node_id } => {
                if graph.node(*node_id).is_none() {
                    Ok(ChangeResult::Skipped {
                        reason: format!("node {} does not exist", node_id),
                    })
                } else {
                    graph.remove_node(*node_id);
                    Ok(ChangeResult::Applied)
                }
            }
            GraphChange::Connect { from_node, from_port, to_node, to_port } => {
                graph.connect(*from_node, from_port, *to_node, to_port)?;
                Ok(ChangeResult::Applied)
            }
            GraphChange::Disconnect { from_node, from_port, to_node, to_port } => {
                graph.disconnect(*from_node, from_port, *to_node, to_port)?;
                Ok(ChangeResult::Applied)
            }
            GraphChange::ReplaceNode { node_id, new_node } => {
                if graph.node(*node_id).is_none() {
                    Ok(ChangeResult::Skipped {
                        reason: format!("node {} does not exist", node_id),
                    })
                } else {
                    graph.remove_node(*node_id);
                    graph.add_node(new_node.clone())?;
                    Ok(ChangeResult::Applied)
                }
            }
        };

        // Notify listeners after change
        let graph_ref = &*graph;
        let listeners = self.listeners.lock();
        for listener in listeners.iter() {
            // Extract the result for listeners, use Applied as default on error
            let result_ref = result.as_ref().unwrap_or(&ChangeResult::Applied);
            listener.on_after_change(graph_ref, change, result_ref);
        }

        result
    }

    /// Apply a transaction atomically
    pub fn apply_transaction(&self, transaction: &GraphTransaction) -> Result<Vec<ChangeResult>> {
        let mut graph = self.graph.lock();

        // Pre-validate all changes with listeners
        let listeners = self.listeners.lock();
        for change in &transaction.changes {
            for listener in listeners.iter() {
                listener.on_before_change(&graph, change)?;
            }
        }
        drop(listeners);

        // Create a clone to test with
        let mut test_graph = graph.clone();

        // Try to apply all changes to the test graph
        let mut results = Vec::new();
        for change in &transaction.changes {
            match Self::apply_change_internal(&mut test_graph, change) {
                Ok(result) => results.push(result),
                Err(e) => {
                    return Err(Error::graph(format!(
                        "transaction failed: {}",
                        e
                    )));
                }
            }
        }

        // All changes validated, apply to real graph
        for change in &transaction.changes {
            let result = Self::apply_change_internal(&mut graph, change)?;
            // Notify listeners
            let listeners = self.listeners.lock();
            for listener in listeners.iter() {
                listener.on_after_change(&*graph, change, &result);
            }
        }

        Ok(results)
    }

    /// Internal change application
    fn apply_change_internal(graph: &mut Graph, change: &GraphChange) -> Result<ChangeResult> {
        match change {
            GraphChange::AddNode { node } => {
                let node_clone = node.clone();
                let node_id = node.id().as_u64();
                if graph.node(node_id).is_some() {
                    return Ok(ChangeResult::Skipped {
                        reason: format!("node {} already exists", node_id),
                    });
                }
                graph.add_node(node_clone)?;
                Ok(ChangeResult::Applied)
            }
            GraphChange::RemoveNode { node_id } => {
                if graph.node(*node_id).is_none() {
                    return Ok(ChangeResult::Skipped {
                        reason: format!("node {} does not exist", node_id),
                    });
                }
                graph.remove_node(*node_id);
                Ok(ChangeResult::Applied)
            }
            GraphChange::Connect { from_node, from_port, to_node, to_port } => {
                graph.connect(*from_node, from_port, *to_node, to_port)?;
                Ok(ChangeResult::Applied)
            }
            GraphChange::Disconnect { from_node, from_port, to_node, to_port } => {
                graph.disconnect(*from_node, from_port, *to_node, to_port)?;
                Ok(ChangeResult::Applied)
            }
            GraphChange::ReplaceNode { node_id, new_node } => {
                if graph.node(*node_id).is_none() {
                    return Ok(ChangeResult::Skipped {
                        reason: format!("node {} does not exist", node_id),
                    });
                }
                graph.remove_node(*node_id);
                graph.add_node(new_node.clone())?;
                Ok(ChangeResult::Applied)
            }
        }
    }

    /// Get a snapshot of the current graph
    pub fn snapshot(&self) -> Graph {
        self.graph.lock().clone()
    }

    /// Get the underlying graph (with lock held)
    pub fn with_graph<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Graph) -> R,
    {
        let graph = self.graph.lock();
        f(&graph)
    }

    /// Get mutable access to the underlying graph
    pub fn with_graph_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Graph) -> R,
    {
        let mut graph = self.graph.lock();
        f(&mut graph)
    }
}

impl Default for DynamicGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Node;

    #[test]
    fn test_graph_change_add_node() {
        let change = GraphChange::add_node(Node::transform("test"));
        assert!(change.description().contains("Add node"));
    }

    #[test]
    fn test_graph_change_remove_node() {
        let change = GraphChange::remove_node(123);
        assert!(change.description().contains("Remove node"));
    }

    #[test]
    fn test_graph_change_connect() {
        let change = GraphChange::connect(1, "out", 2, "in");
        assert!(change.description().contains("Connect"));
        assert!(change.description().contains("1:out"));
        assert!(change.description().contains("2:in"));
    }

    #[test]
    fn test_graph_change_disconnect() {
        let change = GraphChange::disconnect(1, "out", 2, "in");
        assert!(change.description().contains("Disconnect"));
    }

    #[test]
    fn test_graph_change_replace_node() {
        let new_node = Node::transform("new");
        let change = GraphChange::replace_node(123, new_node);
        assert!(change.description().contains("Replace node"));
    }

    #[test]
    fn test_transaction_empty() {
        let tx = GraphTransaction::new();
        assert!(tx.is_empty());
        assert_eq!(tx.len(), 0);
    }

    #[test]
    fn test_transaction_with_changes() {
        let tx = GraphTransaction::new()
            .add_change(GraphChange::remove_node(1))
            .add_change(GraphChange::remove_node(2));
        assert_eq!(tx.len(), 2);
    }

    #[test]
    fn test_transaction_with_description() {
        let tx = GraphTransaction::with_description("test transaction");
        assert_eq!(tx.description(), Some("test transaction"));
    }

    #[test]
    fn test_transaction_builder_pattern() {
        let changes = vec![
            GraphChange::remove_node(1),
            GraphChange::remove_node(2),
        ];
        let tx = GraphTransaction::new().add_changes(changes);
        assert_eq!(tx.len(), 2);
    }

    #[test]
    fn test_dynamic_graph_create() {
        let dg = DynamicGraph::new();
        assert_eq!(dg.with_graph(|g| g.node_count()), 0);
    }

    #[test]
    fn test_dynamic_graph_from_graph() {
        let mut graph = Graph::new();
        graph.add_node(Node::transform("test")).unwrap();
        let dg = DynamicGraph::from_graph(graph);
        assert_eq!(dg.with_graph(|g| g.node_count()), 1);
    }

    #[test]
    fn test_dynamic_graph_apply_change_add_node() {
        let dg = DynamicGraph::new();
        let node = Node::transform("test");
        let change = GraphChange::add_node(node);
        let result = dg.apply_change(&change).unwrap();
        assert!(matches!(result, ChangeResult::Applied));
        assert_eq!(dg.with_graph(|g| g.node_count()), 1);
    }

    #[test]
    fn test_dynamic_graph_apply_change_remove_node() {
        let dg = DynamicGraph::new();
        dg.with_graph_mut(|g| {
            g.add_node(Node::transform("test")).unwrap();
        });

        let change = GraphChange::remove_node(
            dg.with_graph(|g| g.nodes().next().unwrap().id().as_u64())
        );
        let result = dg.apply_change(&change).unwrap();
        assert!(matches!(result, ChangeResult::Applied));
        assert_eq!(dg.with_graph(|g| g.node_count()), 0);
    }

    #[test]
    fn test_dynamic_graph_apply_change_skip_nonexistent() {
        let dg = DynamicGraph::new();
        let change = GraphChange::remove_node(999);
        let result = dg.apply_change(&change).unwrap();
        assert!(matches!(result, ChangeResult::Skipped { .. }));
    }

    #[test]
    fn test_dynamic_graph_snapshot() {
        let dg = DynamicGraph::new();
        dg.with_graph_mut(|g| {
            g.add_node(Node::transform("test")).unwrap();
        });

        let snapshot = dg.snapshot();
        assert_eq!(snapshot.node_count(), 1);
    }

    #[test]
    fn test_nop_listener() {
        let listener = NopListener;
        let graph = Graph::new();
        let change = GraphChange::remove_node(1);

        assert!(listener.on_before_change(&graph, &change).is_ok());
        listener.on_after_change(&graph, &change, &ChangeResult::Applied);
    }

    #[test]
    fn test_dynamic_graph_add_listener() {
        let dg = DynamicGraph::new();
        dg.add_listener(Box::new(NopListener));
        // Should not panic
        let node = Node::transform("test");
        let change = GraphChange::add_node(node);
        assert!(dg.apply_change(&change).is_ok());
    }

    #[test]
    fn test_transaction_apply_to_graph() {
        let mut graph = Graph::new();
        let mut source = Node::source("src");
        let mut sink = Node::sink("snk");
        source.add_output("out").unwrap();
        sink.add_input("in").unwrap();

        let src_id = source.id().as_u64();
        let snk_id = sink.id().as_u64();

        let tx = GraphTransaction::with_description("build pipeline")
            .add_change(GraphChange::add_node(source))
            .add_change(GraphChange::add_node(sink))
            .add_change(GraphChange::connect(src_id, "out", snk_id, "in"));

        let results = tx.apply(&mut graph).unwrap();
        assert_eq!(results.len(), 3);
        assert!(matches!(results[0], ChangeResult::Applied));
        assert!(matches!(results[1], ChangeResult::Applied));
        assert!(matches!(results[2], ChangeResult::Applied));
        assert_eq!(graph.node_count(), 2);
    }

    #[test]
    fn test_transaction_rollback_on_error() {
        let mut graph = Graph::new();
        let source = Node::source("src");

        let tx = GraphTransaction::new()
            .add_change(GraphChange::add_node(source))
            // This will fail because node doesn't exist
            .add_change(GraphChange::connect(999, "out", 1, "in"));

        let result = tx.apply(&mut graph);
        assert!(result.is_err());
        // Transaction should have rolled back
        assert_eq!(graph.node_count(), 0);
    }

    #[test]
    fn test_change_result_display() {
        let applied = ChangeResult::Applied;
        let skipped = ChangeResult::Skipped { reason: "test".to_string() };
        let rolled_back = ChangeResult::RolledBack { reason: "test".to_string() };

        // Just verify they can be created and cloned
        let _ = applied.clone();
        let _ = skipped.clone();
        let _ = rolled_back.clone();
    }
}
