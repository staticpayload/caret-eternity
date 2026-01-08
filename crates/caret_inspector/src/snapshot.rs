// Caret Inspector - Snapshots of runtime state
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use serde::{Deserialize, Serialize};

/// Snapshot of runtime state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSnapshot {
    /// Current runtime state
    pub state: RuntimeState,
    /// Current tick
    pub tick: u64,
    /// Total nodes
    pub total_nodes: usize,
    /// Running nodes
    pub running_nodes: usize,
    /// Completed nodes
    pub completed_nodes: usize,
    /// Errored nodes
    pub errored_nodes: usize,
}

/// Runtime state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeState {
    /// Runtime is stopped
    #[default]
    Stopped,
    /// Runtime is running
    Running,
    /// Runtime is paused
    Paused,
    /// Runtime encountered an error
    Error,
    /// Runtime completed successfully
    Completed,
}

/// Snapshot of graph state
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GraphSnapshot {
    /// Nodes in the graph
    pub nodes: Vec<NodeSnapshot>,
    /// Edges in the graph
    pub edges: Vec<EdgeSnapshot>,
}

/// Snapshot of a node
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeSnapshot {
    /// Node ID
    pub id: u64,
    /// Node name
    pub name: String,
    /// Node type
    pub node_type: String,
    /// Node state
    pub state: NodeState,
    /// Input ports
    pub inputs: Vec<PortSnapshot>,
    /// Output ports
    pub outputs: Vec<PortSnapshot>,
    /// Packets processed
    pub packets_processed: u64,
    /// Packets dropped
    pub packets_dropped: u64,
    /// Last error (if any)
    pub last_error: Option<String>,
}

/// State of a node
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    /// Node is idle, waiting for data
    Idle,
    /// Node is ready to run
    Ready,
    /// Node is currently running
    Running,
    /// Node has completed
    Done,
    /// Node encountered an error
    Error,
}

/// Snapshot of a port
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortSnapshot {
    /// Port name
    pub name: String,
    /// Port direction
    pub direction: PortDirection,
    /// Current queue depth
    pub depth: usize,
    /// Port capacity
    pub capacity: usize,
    /// Number of connections
    pub connections: usize,
}

/// Direction of a port
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    /// Input port
    Input,
    /// Output port
    Output,
}

/// Snapshot of an edge connection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeSnapshot {
    /// Source node ID
    pub from_node: u64,
    /// Source port name
    pub from_port: String,
    /// Target node ID
    pub to_node: u64,
    /// Target port name
    pub to_port: String,
}

/// Snapshot of buffer pool stats
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BufferPoolSnapshot {
    /// Pool name
    pub name: String,
    /// Total buffers allocated
    pub total_allocated: usize,
    /// Total buffers in use
    pub in_use: usize,
    /// Total buffers available
    pub available: usize,
    /// Total bytes allocated
    pub total_bytes: u64,
}

/// Snapshot of a metric
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricSnapshot {
    /// Metric ID
    pub id: String,
    /// Metric name
    pub name: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Metric value
    pub value: MetricValue,
    /// Labels
    pub labels: Vec<(String, String)>,
}

/// Type of metric
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter metric
    Counter,
    /// Gauge metric
    Gauge,
    /// Histogram metric
    Histogram,
}

/// Value of a metric
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricValue {
    /// Counter value (monotonically increasing)
    Counter(u64),
    /// Gauge value (can go up or down)
    Gauge(i64),
    /// Histogram value (distribution)
    Histogram(Vec<u64>),
}

impl Default for MetricValue {
    fn default() -> Self {
        Self::Counter(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_snapshot_serialize() {
        let snapshot = RuntimeSnapshot::default();
        let json = serde_json::to_string(&snapshot).unwrap();
        assert!(json.contains("\"state\""));
    }

    #[test]
    fn test_graph_snapshot_default() {
        let snapshot = GraphSnapshot::default();
        assert!(snapshot.nodes.is_empty());
        assert!(snapshot.edges.is_empty());
    }

    #[test]
    fn test_node_snapshot() {
        let node = NodeSnapshot {
            id: 1,
            name: "test".to_string(),
            node_type: "PassthroughNode".to_string(),
            state: NodeState::Idle,
            inputs: vec![],
            outputs: vec![],
            packets_processed: 0,
            packets_dropped: 0,
            last_error: None,
        };

        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("\"test\""));
    }

    #[test]
    fn test_port_snapshot() {
        let port = PortSnapshot {
            name: "input".to_string(),
            direction: PortDirection::Input,
            depth: 5,
            capacity: 100,
            connections: 1,
        };

        assert_eq!(port.name, "input");
        assert_eq!(port.depth, 5);
    }

    #[test]
    fn test_metric_snapshot() {
        let metric = MetricSnapshot {
            id: "test_counter".to_string(),
            name: "packets_processed".to_string(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(42),
            labels: vec![("node".to_string(), "test".to_string())],
        };

        assert_eq!(metric.value, MetricValue::Counter(42));
    }
}
