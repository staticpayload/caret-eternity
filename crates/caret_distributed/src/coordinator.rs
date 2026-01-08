// Caret Distributed - Distributed execution coordinator
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{error::Result, message::Message, node::NodeId, Error};
use serde::{Deserialize, Serialize};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Execution mode for distributed graphs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Pipeline: nodes distributed across workers
    Pipeline,
    /// Data parallel: same graph on multiple workers
    DataParallel,
    /// Hybrid: combination of pipeline and data parallel
    Hybrid,
}

/// Coordinator configuration
#[derive(Clone, Debug)]
pub struct CoordinatorConfig {
    /// Execution mode
    pub mode: ExecutionMode,
    /// Maximum workers to use
    pub max_workers: usize,
    /// Minimum workers required
    pub min_workers: usize,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Request timeout
    pub request_timeout: Duration,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::Pipeline,
            max_workers: usize::MAX,
            min_workers: 1,
            heartbeat_interval: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
        }
    }
}

impl CoordinatorConfig {
    /// Create with the given execution mode
    pub fn with_mode(mode: ExecutionMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// Set max workers
    pub fn with_max_workers(mut self, max: usize) -> Self {
        self.max_workers = max;
        self
    }

    /// Set min workers
    pub fn with_min_workers(mut self, min: usize) -> Self {
        self.min_workers = min;
        self
    }
}

/// Worker state
#[derive(Clone, Debug)]
struct WorkerState {
    /// Worker node ID
    node_id: NodeId,
    /// Assigned graph nodes
    assigned: Vec<u64>,
    /// Last heartbeat
    last_heartbeat: std::time::Instant,
    /// Current load (0.0 to 1.0)
    load: f64,
}

/// Graph execution state
#[derive(Clone, Debug)]
struct GraphState {
    /// Graph ID
    id: String,
    /// Execution mode
    mode: ExecutionMode,
    /// Worker assignments (node_id -> worker_node_id)
    assignments: HashMap<u64, NodeId>,
    /// Current state
    status: ExecutionStatus,
}

/// Execution status
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Graph is pending
    Pending,
    /// Graph is running
    Running,
    /// Graph is paused
    Paused,
    /// Graph completed
    Completed,
    /// Graph failed
    Failed,
}

/// Distributed execution coordinator
pub struct Coordinator {
    config: CoordinatorConfig,
    local_node: NodeId,
    workers: Arc<Mutex<HashMap<NodeId, WorkerState>>>,
    graphs: Arc<Mutex<HashMap<String, GraphState>>>,
    event_tx: Arc<Mutex<mpsc::Sender<CoordinatorEvent>>>,
    running: Arc<Mutex<bool>>,
}

impl Coordinator {
    /// Create a new coordinator
    pub fn new(config: CoordinatorConfig, local_node: NodeId) -> Self {
        let (event_tx, _rx) = mpsc::channel(256);

        Self {
            config,
            local_node,
            workers: Arc::new(Mutex::new(HashMap::new())),
            graphs: Arc::new(Mutex::new(HashMap::new())),
            event_tx: Arc::new(Mutex::new(event_tx)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the coordinator
    pub async fn start(&self) -> Result<()> {
        *self.running.lock() = true;
        Ok(())
    }

    /// Stop the coordinator
    pub fn stop(&self) {
        *self.running.lock() = false;
    }

    /// Register a worker
    pub fn register_worker(&self, node_id: NodeId) -> Result<()> {
        let worker = WorkerState {
            node_id,
            assigned: Vec::new(),
            last_heartbeat: std::time::Instant::now(),
            load: 0.0,
        };

        self.workers.lock().insert(node_id, worker);

        let _ = self
            .event_tx
            .lock()
            .try_send(CoordinatorEvent::WorkerJoined(node_id));

        Ok(())
    }

    /// Unregister a worker
    pub fn unregister_worker(&self, node_id: &NodeId) {
        self.workers.lock().remove(node_id);

        // Reassign any work from this worker
        self.reassign_worker_work(node_id);

        let _ = self
            .event_tx
            .lock()
            .try_send(CoordinatorEvent::WorkerLeft(*node_id));
    }

    /// Update worker heartbeat
    pub fn worker_heartbeat(&self, node_id: NodeId, load: f64) {
        let mut workers = self.workers.lock();
        if let Some(worker) = workers.get_mut(&node_id) {
            worker.last_heartbeat = std::time::Instant::now();
            worker.load = load.clamp(0.0, 1.0);
        }
    }

    /// Assign a graph node to a worker
    pub fn assign_node(&self, graph_id: &str, node_id: u64) -> Result<NodeId> {
        // Find the least loaded worker
        let worker_id = {
            let workers = self.workers.lock();
            let mut best_worker = None;
            let mut best_load = f32::MAX;

            for (id, worker) in workers.iter() {
                let effective_load = worker.load as f32 + worker.assigned.len() as f32 * 0.1;
                if effective_load < best_load {
                    best_load = effective_load;
                    best_worker = Some(*id);
                }
            }

            best_worker.ok_or_else(|| Error::Coordinator("No workers available".into()))?
        };

        // Record the assignment
        {
            let mut graphs = self.graphs.lock();
            if let Some(graph) = graphs.get_mut(graph_id) {
                graph.assignments.insert(node_id, worker_id);
            }
        }

        // Update worker state
        {
            let mut workers = self.workers.lock();
            if let Some(worker) = workers.get_mut(&worker_id) {
                worker.assigned.push(node_id);
            }
        }

        Ok(worker_id)
    }

    /// Release a graph node from a worker
    pub fn release_node(&self, graph_id: &str, node_id: u64) {
        let worker_id = {
            let mut graphs = self.graphs.lock();
            graphs
                .get_mut(graph_id)
                .and_then(|g| g.assignments.remove(&node_id))
        };

        if let Some(worker_id) = worker_id {
            let mut workers = self.workers.lock();
            if let Some(worker) = workers.get_mut(&worker_id) {
                worker.assigned.retain(|n| *n != node_id);
            }
        }
    }

    /// Submit a graph for execution
    pub fn submit_graph(&self, id: String, mode: ExecutionMode) -> Result<()> {
        let state = GraphState {
            id: id.clone(),
            mode,
            assignments: HashMap::new(),
            status: ExecutionStatus::Pending,
        };

        self.graphs.lock().insert(id.clone(), state);

        let _ = self
            .event_tx
            .lock()
            .try_send(CoordinatorEvent::GraphSubmitted(id));

        Ok(())
    }

    /// Start graph execution
    pub fn start_graph(&self, id: &str) -> Result<()> {
        let mut graphs = self.graphs.lock();
        if let Some(graph) = graphs.get_mut(id) {
            graph.status = ExecutionStatus::Running;

            let _ = self
                .event_tx
                .lock()
                .try_send(CoordinatorEvent::GraphStarted(id.to_string()));

            Ok(())
        } else {
            Err(Error::Execution(format!("Graph {} not found", id)))
        }
    }

    /// Stop graph execution
    pub fn stop_graph(&self, id: &str, drain: bool) -> Result<()> {
        let mut graphs = self.graphs.lock();
        if let Some(graph) = graphs.get_mut(id) {
            graph.status = if drain {
                ExecutionStatus::Paused
            } else {
                ExecutionStatus::Completed
            };

            let _ = self.event_tx.lock().try_send(CoordinatorEvent::GraphStopped(
                id.to_string(),
            ));

            Ok(())
        } else {
            Err(Error::Execution(format!("Graph {} not found", id)))
        }
    }

    /// Get graph status
    pub fn graph_status(&self, id: &str) -> Option<ExecutionStatus> {
        self.graphs.lock().get(id).map(|g| g.status)
    }

    /// Get all workers
    pub fn workers(&self) -> Vec<NodeId> {
        self.workers.lock().keys().cloned().collect()
    }

    /// Get worker count
    pub fn worker_count(&self) -> usize {
        self.workers.lock().len()
    }

    /// Subscribe to coordinator events
    pub fn subscribe(&self) -> mpsc::Receiver<CoordinatorEvent> {
        let (tx, rx) = mpsc::channel(256);
        *self.event_tx.lock() = tx;
        rx
    }

    /// Reassign work from a failed worker
    fn reassign_worker_work(&self, node_id: &NodeId) {
        let mut assignments_to_reassign = Vec::new();

        // Collect assignments
        {
            let mut graphs = self.graphs.lock();
            for graph in graphs.values_mut() {
                let mut to_remove = Vec::new();
                for (graph_node, worker) in graph.assignments.iter() {
                    if worker == node_id {
                        to_remove.push(*graph_node);
                    }
                }
                for graph_node in to_remove {
                    if let Some(worker) = graph.assignments.remove(&graph_node) {
                        assignments_to_reassign.push((graph.id.clone(), graph_node, worker));
                    }
                }
            }
        }

        // Reassign to other workers
        for (graph_id, graph_node, _old_worker) in assignments_to_reassign {
            if let Ok(new_worker) = self.assign_node(&graph_id, graph_node) {
                tracing::info!(
                    "Reassigned node {} from {} to {}",
                    graph_node,
                    node_id,
                    new_worker
                );
            }
        }
    }

    /// Handle incoming message
    pub fn handle_message(&self, msg: Message) -> Result<()> {
        match msg.payload {
            crate::message::MessagePayload::Heartbeat => {
                // Reply with heartbeat
            }
            crate::message::MessagePayload::StatusRequest => {
                // Send status reply
            }
            _ => {}
        }
        Ok(())
    }
}

/// Coordinator event
#[derive(Clone, Debug)]
pub enum CoordinatorEvent {
    /// Worker joined
    WorkerJoined(NodeId),
    /// Worker left
    WorkerLeft(NodeId),
    /// Worker failed
    WorkerFailed(NodeId),
    /// Graph submitted
    GraphSubmitted(String),
    /// Graph started
    GraphStarted(String),
    /// Graph stopped
    GraphStopped(String),
    /// Graph completed
    GraphCompleted(String),
    /// Graph failed
    GraphFailed(String, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_config() {
        let config = CoordinatorConfig::with_mode(ExecutionMode::DataParallel)
            .with_max_workers(10)
            .with_min_workers(2);

        assert_eq!(config.mode, ExecutionMode::DataParallel);
        assert_eq!(config.max_workers, 10);
        assert_eq!(config.min_workers, 2);
    }

    #[tokio::test]
    async fn test_coordinator_workers() {
        let local_id = NodeId::new_v4();
        let coordinator = Coordinator::new(CoordinatorConfig::default(), local_id);

        coordinator.start().await.unwrap();

        let worker1 = NodeId::new_v4();
        let worker2 = NodeId::new_v4();

        coordinator.register_worker(worker1).unwrap();
        coordinator.register_worker(worker2).unwrap();

        assert_eq!(coordinator.worker_count(), 2);

        coordinator.unregister_worker(&worker1);
        assert_eq!(coordinator.worker_count(), 1);
    }

    #[tokio::test]
    async fn test_coordinator_graph() {
        let local_id = NodeId::new_v4();
        let coordinator = Coordinator::new(CoordinatorConfig::default(), local_id);

        coordinator.start().await.unwrap();

        let graph_id = "test-graph".to_string();
        coordinator
            .submit_graph(graph_id.clone(), ExecutionMode::Pipeline)
            .unwrap();

        assert_eq!(
            coordinator.graph_status(&graph_id),
            Some(ExecutionStatus::Pending)
        );

        coordinator.start_graph(&graph_id).unwrap();

        assert_eq!(
            coordinator.graph_status(&graph_id),
            Some(ExecutionStatus::Running)
        );
    }
}
