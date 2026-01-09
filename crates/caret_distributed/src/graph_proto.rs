// Caret Distributed - Graph serialization and partitioning
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::NodeId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use caret_graph::{Graph as CaretGraph, NodeType as CaretNodeType};

/// Errors that can occur during graph partitioning
#[derive(Debug, Error)]
pub enum PartitionError {
    /// Graph has cycles (cannot partition acyclic graphs)
    #[error("graph contains cycles")]
    HasCycles,

    /// No workers available for partitioning
    #[error("no workers available")]
    NoWorkers,

    /// Invalid partition configuration
    #[error("invalid partition configuration: {0}")]
    InvalidConfig(String),

    /// Cannot partition - node not found
    #[error("node {0} not found in graph")]
    NodeNotFound(u64),
}

/// A serialized port definition
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializablePort {
    /// Port name
    pub name: String,
    /// Port ID
    pub id: u64,
    /// Direction (input or output)
    pub direction: PortDirection,
}

/// Port direction
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum PortDirection {
    /// Input port
    In,
    /// Output port
    Out,
}

/// A serialized node definition
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializableNode {
    /// Unique node ID
    pub id: u64,
    /// Node name
    pub name: String,
    /// Node type
    pub node_type: NodeType,
    /// Input ports
    pub inputs: Vec<SerializablePort>,
    /// Output ports
    pub outputs: Vec<SerializablePort>,
}

/// Node type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

/// A serialized edge
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SerializableEdge {
    /// Source node ID
    pub from_node: u64,
    /// Source port name
    pub from_port: String,
    /// Target node ID
    pub to_node: u64,
    /// Target port name
    pub to_port: String,
}

/// A complete serialized graph
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SerializableGraph {
    /// Graph ID (for tracking)
    pub id: String,
    /// All nodes in the graph
    pub nodes: Vec<SerializableNode>,
    /// All edges in the graph
    pub edges: Vec<SerializableEdge>,
    /// Topological order of nodes (for execution)
    pub topological_order: Vec<u64>,
}

impl SerializableGraph {
    /// Create a new empty serialized graph
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            nodes: Vec::new(),
            edges: Vec::new(),
            topological_order: Vec::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: SerializableNode) {
        self.nodes.push(node);
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: SerializableEdge) {
        self.edges.push(edge);
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Validate the graph
    pub fn validate(&self) -> std::result::Result<(), PartitionError> {
        // Check that all referenced nodes exist
        let node_ids: HashSet<u64> = self.nodes.iter().map(|n| n.id).collect();

        for edge in &self.edges {
            if !node_ids.contains(&edge.from_node) {
                return Err(PartitionError::NodeNotFound(edge.from_node));
            }
            if !node_ids.contains(&edge.to_node) {
                return Err(PartitionError::NodeNotFound(edge.to_node));
            }
        }

        Ok(())
    }

    /// Get node by ID
    pub fn node(&self, id: u64) -> Option<&SerializableNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Get all nodes
    pub fn nodes(&self) -> &[SerializableNode] {
        &self.nodes
    }

    /// Get edges from a node
    pub fn edges_from(&self, node_id: u64) -> Vec<&SerializableEdge> {
        self.edges
            .iter()
            .filter(|e| e.from_node == node_id)
            .collect()
    }

    /// Get edges to a node
    pub fn edges_to(&self, node_id: u64) -> Vec<&SerializableEdge> {
        self.edges
            .iter()
            .filter(|e| e.to_node == node_id)
            .collect()
    }

    /// Check if the graph is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Serialize to JSON for network transmission
    pub fn to_json(&self) -> std::result::Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> std::result::Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> std::result::Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> std::result::Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

impl SerializableGraph {
    /// Convert from caret_graph's Graph
    pub fn from_caret_graph(id: impl Into<String>, graph: &CaretGraph) -> Self {
        let mut serializable = Self::new(id);

        // Add all nodes
        for node in graph.nodes() {
            let serializable_node = SerializableNode {
                id: node.id().as_u64(),
                name: node.name().to_string(),
                node_type: match node.node_type() {
                    CaretNodeType::Source => NodeType::Source,
                    CaretNodeType::Transform => NodeType::Transform,
                    CaretNodeType::Sink => NodeType::Sink,
                    CaretNodeType::Custom(ref s) => NodeType::Custom(s.clone()),
                },
                inputs: node
                    .inputs()
                    .iter()
                    .map(|(name, port)| SerializablePort {
                        name: name.clone(),
                        id: port.id().as_u64(),
                        direction: PortDirection::In,
                    })
                    .collect(),
                outputs: node
                    .outputs()
                    .iter()
                    .map(|(name, port)| SerializablePort {
                        name: name.clone(),
                        id: port.id().as_u64(),
                        direction: PortDirection::Out,
                    })
                    .collect(),
            };
            serializable.add_node(serializable_node);
        }

        // Add all edges from topology
        for edge in graph.topology().edges() {
            // Find port names by looking up the ports
            let from_node = graph.node(edge.from_node);
            let to_node = graph.node(edge.to_node);

            if let (Some(from_node), Some(to_node)) = (from_node, to_node) {
                // Find the output port by ID
                let from_port_name = from_node
                    .outputs()
                    .values()
                    .find(|p| p.id().as_u64() == edge.from_port.as_u64())
                    .map(|p| p.name().to_string())
                    .unwrap_or_else(|| format!("port_{}", edge.from_port.as_u64()));

                // Find the input port by ID
                let to_port_name = to_node
                    .inputs()
                    .values()
                    .find(|p| p.id().as_u64() == edge.to_port.as_u64())
                    .map(|p| p.name().to_string())
                    .unwrap_or_else(|| format!("port_{}", edge.to_port.as_u64()));

                serializable.add_edge(SerializableEdge {
                    from_node: edge.from_node,
                    from_port: from_port_name,
                    to_node: edge.to_node,
                    to_port: to_port_name,
                });
            }
        }

        // Set topological order if available
        if let Some(order) = graph.topology().topological_order() {
            serializable.topological_order = order;
        }

        serializable
    }
}

/// A partition of a graph assigned to a worker
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphPartition {
    /// Worker ID this partition is assigned to
    pub worker_id: NodeId,
    /// Nodes in this partition
    pub nodes: Vec<u64>,
    /// Internal edges (within this partition)
    pub internal_edges: Vec<SerializableEdge>,
    /// Input edges (from other partitions to this one)
    pub input_edges: Vec<CrossPartitionEdge>,
    /// Output edges (from this partition to others)
    pub output_edges: Vec<CrossPartitionEdge>,
}

/// An edge that crosses partition boundaries
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrossPartitionEdge {
    /// Source node ID
    pub from_node: u64,
    /// Source port name
    pub from_port: String,
    /// Source partition (worker)
    pub from_worker: NodeId,
    /// Target node ID
    pub to_node: u64,
    /// Target port name
    pub to_port: String,
    /// Target partition (worker)
    pub to_worker: NodeId,
}

/// Complete partition assignment for a graph
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartitionAssignment {
    /// Graph ID
    pub graph_id: String,
    /// All partitions (indexed by worker ID)
    pub partitions: HashMap<NodeId, GraphPartition>,
    /// Cross-partition routes (for packet routing)
    pub routes: Vec<CrossNodeRoute>,
}

/// A route for packets crossing node boundaries
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CrossNodeRoute {
    /// Source node ID
    pub from_node: u64,
    /// Source port name
    pub from_port: String,
    /// Source worker
    pub from_worker: NodeId,
    /// Target node ID
    pub to_node: u64,
    /// Target port name
    pub to_port: String,
    /// Target worker
    pub to_worker: NodeId,
}

/// Strategy for partitioning a graph across workers
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartitionStrategy {
    /// Round-robin assignment of nodes to workers
    RoundRobin,

    /// Assign contiguous ranges of nodes to workers
    Contiguous,

    /// Minimize cross-partition edges (requires graph analysis)
    MinimizeCrossEdges,

    /// Manual partition assignment
    Manual,
}

impl Default for PartitionStrategy {
    fn default() -> Self {
        Self::RoundRobin
    }
}

/// Graph partitioner
pub struct GraphPartitioner;

impl GraphPartitioner {
    /// Partition a graph using the specified strategy
    ///
    /// # Arguments
    /// * `graph` - The graph to partition
    /// * `workers` - Available worker nodes
    /// * `strategy` - Partitioning strategy to use
    ///
    /// # Returns
    /// A partition assignment mapping nodes to workers
    pub fn partition(
        graph: &SerializableGraph,
        workers: &[NodeId],
        strategy: PartitionStrategy,
    ) -> std::result::Result<PartitionAssignment, PartitionError> {
        if workers.is_empty() {
            return Err(PartitionError::NoWorkers);
        }

        if graph.is_empty() {
            return Ok(PartitionAssignment {
                graph_id: graph.id.clone(),
                partitions: HashMap::new(),
                routes: Vec::new(),
            });
        }

        // Validate the graph
        graph.validate()?;

        // Assign nodes to workers based on strategy
        let node_assignments = match strategy {
            PartitionStrategy::RoundRobin => Self::round_robin_assign(graph, workers),
            PartitionStrategy::Contiguous => Self::contiguous_assign(graph, workers),
            PartitionStrategy::MinimizeCrossEdges => {
                Self::minimize_cross_edges_assign(graph, workers)?
            }
            PartitionStrategy::Manual => {
                return Err(PartitionError::InvalidConfig(
                    "Manual partitioning requires pre-assignment".into(),
                ))
            }
        };

        // Build partitions from assignments
        let partitions = Self::build_partitions(graph, &node_assignments);

        // Build cross-partition routes
        let routes = Self::build_routes(graph, &node_assignments);

        Ok(PartitionAssignment {
            graph_id: graph.id.clone(),
            partitions,
            routes,
        })
    }

    /// Assign nodes to workers using round-robin
    fn round_robin_assign(
        graph: &SerializableGraph,
        workers: &[NodeId],
    ) -> HashMap<u64, NodeId> {
        let mut assignments = HashMap::new();
        for (i, node) in graph.nodes.iter().enumerate() {
            let worker = workers[i % workers.len()];
            assignments.insert(node.id, worker);
        }
        assignments
    }

    /// Assign nodes to workers using contiguous ranges
    fn contiguous_assign(
        graph: &SerializableGraph,
        workers: &[NodeId],
    ) -> HashMap<u64, NodeId> {
        let mut assignments = HashMap::new();
        let nodes_per_worker = (graph.nodes.len() + workers.len() - 1) / workers.len();

        for (i, node) in graph.nodes.iter().enumerate() {
            let worker_idx = i / nodes_per_worker;
            let worker = workers[worker_idx.min(workers.len() - 1)];
            assignments.insert(node.id, worker);
        }
        assignments
    }

    /// Assign nodes to minimize cross-partition edges
    ///
    /// Uses a simple greedy algorithm:
    /// 1. Start with round-robin assignment
    /// 2. Iteratively move nodes to reduce cross-edge count
    fn minimize_cross_edges_assign(
        graph: &SerializableGraph,
        workers: &[NodeId],
    ) -> std::result::Result<HashMap<u64, NodeId>, PartitionError> {
        // Start with round-robin
        let mut assignments = Self::round_robin_assign(graph, workers);

        // Count initial cross-partition edges
        let mut cross_edges = Self::count_cross_edges(graph, &assignments);
        let mut improved = true;

        // Greedy improvement
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 100;

        while improved && iterations < MAX_ITERATIONS {
            improved = false;
            iterations += 1;

            for node in &graph.nodes {
                let current_worker = assignments[&node.id];

                // Try moving this node to each other worker
                for &worker in workers {
                    if worker == current_worker {
                        continue;
                    }

                    // Temporarily move node
                    assignments.insert(node.id, worker);
                    let new_cross_edges = Self::count_cross_edges(graph, &assignments);

                    if new_cross_edges < cross_edges {
                        cross_edges = new_cross_edges;
                        improved = true;
                        break;
                    } else {
                        // Revert
                        assignments.insert(node.id, current_worker);
                    }
                }
            }
        }

        Ok(assignments)
    }

    /// Count edges that cross partition boundaries
    fn count_cross_edges(graph: &SerializableGraph, assignments: &HashMap<u64, NodeId>) -> usize {
        graph
            .edges
            .iter()
            .filter(|e| {
                let from_worker = assignments.get(&e.from_node);
                let to_worker = assignments.get(&e.to_node);
                match (from_worker, to_worker) {
                    (Some(fw), Some(tw)) => fw != tw,
                    _ => false,
                }
            })
            .count()
    }

    /// Build partition structures from node assignments
    fn build_partitions(
        graph: &SerializableGraph,
        assignments: &HashMap<u64, NodeId>,
    ) -> HashMap<NodeId, GraphPartition> {
        let mut partitions: HashMap<NodeId, GraphPartition> = HashMap::new();

        // Initialize partitions for each worker
        let workers: HashSet<NodeId> = assignments.values().copied().collect();
        for worker in &workers {
            partitions.insert(
                *worker,
                GraphPartition {
                    worker_id: *worker,
                    nodes: Vec::new(),
                    internal_edges: Vec::new(),
                    input_edges: Vec::new(),
                    output_edges: Vec::new(),
                },
            );
        }

        // Assign nodes to partitions
        for node in &graph.nodes {
            if let Some(&worker) = assignments.get(&node.id) {
                if let Some(partition) = partitions.get_mut(&worker) {
                    partition.nodes.push(node.id);
                }
            }
        }

        // Categorize edges
        for edge in &graph.edges {
            let from_worker = assignments.get(&edge.from_node);
            let to_worker = assignments.get(&edge.to_node);

            match (from_worker, to_worker) {
                (Some(fw), Some(tw)) if fw == tw => {
                    // Internal edge
                    if let Some(partition) = partitions.get_mut(fw) {
                        partition.internal_edges.push(edge.clone());
                    }
                }
                (Some(fw), Some(tw)) => {
                    // Cross-partition edge
                    let cross_edge = CrossPartitionEdge {
                        from_node: edge.from_node,
                        from_port: edge.from_port.clone(),
                        from_worker: *fw,
                        to_node: edge.to_node,
                        to_port: edge.to_port.clone(),
                        to_worker: *tw,
                    };

                    if let Some(partition) = partitions.get_mut(fw) {
                        partition.output_edges.push(cross_edge.clone());
                    }
                    if let Some(partition) = partitions.get_mut(tw) {
                        partition.input_edges.push(cross_edge);
                    }
                }
                _ => {}
            }
        }

        partitions
    }

    /// Build cross-partition routes
    fn build_routes(
        graph: &SerializableGraph,
        assignments: &HashMap<u64, NodeId>,
    ) -> Vec<CrossNodeRoute> {
        graph
            .edges
            .iter()
            .filter_map(|e| {
                let from_worker = assignments.get(&e.from_node)?;
                let to_worker = assignments.get(&e.to_node)?;

                if from_worker != to_worker {
                    Some(CrossNodeRoute {
                        from_node: e.from_node,
                        from_port: e.from_port.clone(),
                        from_worker: *from_worker,
                        to_node: e.to_node,
                        to_port: e.to_port.clone(),
                        to_worker: *to_worker,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Create a partition assignment from a manual mapping
    pub fn manual_partition(
        graph: &SerializableGraph,
        assignments: HashMap<u64, NodeId>,
    ) -> std::result::Result<PartitionAssignment, PartitionError> {
        graph.validate()?;

        let partitions = Self::build_partitions(graph, &assignments);
        let routes = Self::build_routes(graph, &assignments);

        Ok(PartitionAssignment {
            graph_id: graph.id.clone(),
            partitions,
            routes,
        })
    }
}

impl GraphPartition {
    /// Get a list of local node IDs in this partition
    pub fn node_ids(&self) -> &[u64] {
        &self.nodes
    }

    /// Get the number of nodes in this partition
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of internal edges (within this partition)
    pub fn internal_edge_count(&self) -> usize {
        self.internal_edges.len()
    }

    /// Get the number of input edges (from other partitions)
    pub fn input_edge_count(&self) -> usize {
        self.input_edges.len()
    }

    /// Get the number of output edges (to other partitions)
    pub fn output_edge_count(&self) -> usize {
        self.output_edges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_graph() -> SerializableGraph {
        let mut graph = SerializableGraph::new("test-graph");

        // Add nodes: source -> transform -> sink
        graph.add_node(SerializableNode {
            id: 1,
            name: "source".to_string(),
            node_type: NodeType::Source,
            inputs: vec![],
            outputs: vec![SerializablePort {
                name: "out".to_string(),
                id: 10,
                direction: PortDirection::Out,
            }],
        });

        graph.add_node(SerializableNode {
            id: 2,
            name: "transform".to_string(),
            node_type: NodeType::Transform,
            inputs: vec![SerializablePort {
                name: "in".to_string(),
                id: 20,
                direction: PortDirection::In,
            }],
            outputs: vec![SerializablePort {
                name: "out".to_string(),
                id: 21,
                direction: PortDirection::Out,
            }],
        });

        graph.add_node(SerializableNode {
            id: 3,
            name: "sink".to_string(),
            node_type: NodeType::Sink,
            inputs: vec![SerializablePort {
                name: "in".to_string(),
                id: 30,
                direction: PortDirection::In,
            }],
            outputs: vec![],
        });

        // Add edges
        graph.add_edge(SerializableEdge {
            from_node: 1,
            from_port: "out".to_string(),
            to_node: 2,
            to_port: "in".to_string(),
        });

        graph.add_edge(SerializableEdge {
            from_node: 2,
            from_port: "out".to_string(),
            to_node: 3,
            to_port: "in".to_string(),
        });

        graph
    }

    #[test]
    fn test_serializable_graph_create() {
        let graph = SerializableGraph::new("test");
        assert!(graph.is_empty());
        assert_eq!(graph.node_count(), 0);
    }

    #[test]
    fn test_serializable_graph_validate() {
        let graph = create_test_graph();
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_serializable_graph_validate_missing_node() {
        let mut graph = create_test_graph();
        graph.add_edge(SerializableEdge {
            from_node: 99, // Non-existent node
            from_port: "out".to_string(),
            to_node: 1,
            to_port: "in".to_string(),
        });

        assert!(graph.validate().is_err());
    }

    #[test]
    fn test_partition_round_robin() {
        let graph = create_test_graph();
        let workers = vec![
            uuid::Uuid::from_u128(100),
            uuid::Uuid::from_u128(101),
        ];

        let assignment =
            GraphPartitioner::partition(&graph, &workers, PartitionStrategy::RoundRobin)
                .unwrap();

        assert_eq!(assignment.partitions.len(), 2);
        // Round-robin: node 1->worker 0, node 2->worker 1, node 3->worker 0
        // Edges: 1->2 (cross), 2->3 (cross) = 2 cross-partition edges
        assert_eq!(assignment.routes.len(), 2);
    }

    #[test]
    fn test_partition_contiguous() {
        let graph = create_test_graph();
        let workers = vec![
            uuid::Uuid::from_u128(100),
            uuid::Uuid::from_u128(101),
        ];

        let assignment =
            GraphPartitioner::partition(&graph, &workers, PartitionStrategy::Contiguous)
                .unwrap();

        // Nodes 1-2 on worker 0, node 3 on worker 1
        assert_eq!(assignment.partitions.len(), 2);
        assert_eq!(assignment.routes.len(), 1);
    }

    #[test]
    fn test_partition_no_workers() {
        let graph = create_test_graph();
        let workers = vec![];

        let result =
            GraphPartitioner::partition(&graph, &workers, PartitionStrategy::RoundRobin);

        assert!(matches!(result, Err(PartitionError::NoWorkers)));
    }

    #[test]
    fn test_partition_empty_graph() {
        let graph = SerializableGraph::new("empty");
        let workers = vec![uuid::Uuid::from_u128(100)];

        let assignment =
            GraphPartitioner::partition(&graph, &workers, PartitionStrategy::RoundRobin)
                .unwrap();

        assert!(assignment.partitions.is_empty());
        assert!(assignment.routes.is_empty());
    }

    #[test]
    fn test_manual_partition() {
        let graph = create_test_graph();
        let worker1 = uuid::Uuid::from_u128(100);
        let worker2 = uuid::Uuid::from_u128(101);

        let mut assignments = HashMap::new();
        assignments.insert(1, worker1); // source on worker1
        assignments.insert(2, worker1); // transform on worker1
        assignments.insert(3, worker2); // sink on worker2

        let assignment = GraphPartitioner::manual_partition(&graph, assignments).unwrap();

        assert_eq!(assignment.partitions.len(), 2);
        assert_eq!(assignment.routes.len(), 1);

        // Check routes
        let route = &assignment.routes[0];
        assert_eq!(route.from_node, 2);
        assert_eq!(route.to_node, 3);
        assert_eq!(route.from_worker, worker1);
        assert_eq!(route.to_worker, worker2);
    }

    #[test]
    fn test_cross_node_route() {
        let route = CrossNodeRoute {
            from_node: 1,
            from_port: "output".to_string(),
            from_worker: uuid::Uuid::from_u128(100),
            to_node: 2,
            to_port: "input".to_string(),
            to_worker: uuid::Uuid::from_u128(101),
        };

        assert_eq!(route.from_node, 1);
        assert_eq!(route.to_node, 2);
    }

    #[test]
    fn test_from_caret_graph() {
        use caret_graph::{Graph, Node as CaretNode};

        let mut caret_graph = Graph::new();

        // Add source node
        let mut source = CaretNode::source("camera");
        source.add_output("frame").unwrap();
        let source_id = source.id().as_u64();
        caret_graph.add_node(source).unwrap();

        // Add sink node
        let mut sink = CaretNode::sink("display");
        sink.add_input("in").unwrap();
        let sink_id = sink.id().as_u64();
        caret_graph.add_node(sink).unwrap();

        // Connect them
        caret_graph.connect(source_id, "frame", sink_id, "in").unwrap();

        // Convert to serializable graph
        let serializable = SerializableGraph::from_caret_graph("test-graph", &caret_graph);

        assert_eq!(serializable.id, "test-graph");
        assert_eq!(serializable.node_count(), 2);
        assert_eq!(serializable.edge_count(), 1);

        // Check nodes
        assert_eq!(serializable.nodes[0].name, "camera");
        assert_eq!(serializable.nodes[0].node_type, NodeType::Source);
        assert_eq!(serializable.nodes[0].outputs.len(), 1);
        assert_eq!(serializable.nodes[0].outputs[0].name, "frame");

        assert_eq!(serializable.nodes[1].name, "display");
        assert_eq!(serializable.nodes[1].node_type, NodeType::Sink);
        assert_eq!(serializable.nodes[1].inputs.len(), 1);
        assert_eq!(serializable.nodes[1].inputs[0].name, "in");

        // Check edge
        assert_eq!(serializable.edges[0].from_node, source_id);
        assert_eq!(serializable.edges[0].from_port, "frame");
        assert_eq!(serializable.edges[0].to_node, sink_id);
        assert_eq!(serializable.edges[0].to_port, "in");
    }

    #[test]
    fn test_serializable_graph_json_serialization() {
        let graph = create_test_graph();

        // Serialize to JSON
        let json = graph.to_json().unwrap();
        assert!(json.len() > 0);

        // Deserialize from JSON
        let deserialized = SerializableGraph::from_json(&json).unwrap();
        assert_eq!(deserialized.id, graph.id);
        assert_eq!(deserialized.node_count(), graph.node_count());
        assert_eq!(deserialized.edge_count(), graph.edge_count());
    }
}
