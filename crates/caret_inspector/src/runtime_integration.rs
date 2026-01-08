// Caret Inspector - Runtime integration
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::snapshot::*;
use caret_sched::{Executor, NodeId};
use caret_sched::NodeState as SchedNodeState;
use caret_sched::RuntimeState as SchedRuntimeState;
use std::sync::Arc;
use parking_lot::Mutex;

/// Runtime integration for the inspector
///
/// Collects snapshots from the runtime executor for the inspector.
pub struct RuntimeIntegration {
    /// Reference to the executor
    executor: Arc<Mutex<Executor>>,
}

impl RuntimeIntegration {
    /// Create a new runtime integration
    pub fn new(executor: Arc<Mutex<Executor>>) -> Self {
        Self { executor }
    }

    /// Capture a snapshot of the current runtime state
    pub fn capture_runtime_snapshot(&self) -> RuntimeSnapshot {
        let exec = self.executor.lock();
        let runtime_state = exec.runtime_state();

        let node_ids = exec.node_ids();
        let total_nodes = node_ids.len();

        let (running, completed, errored) = node_ids.iter().filter_map(|id| {
            exec.node(*id).map(|n| {
                let inst = n.lock();
                inst.state
            })
        }).fold((0, 0, 0), |(r, c, e), state| {
            match state {
                SchedNodeState::Running => (r + 1, c, e),
                SchedNodeState::Done => (r, c + 1, e),
                SchedNodeState::Error => (r, c, e + 1),
                _ => (r, c, e),
            }
        });

        RuntimeSnapshot {
            state: match runtime_state {
                SchedRuntimeState::Stopped => RuntimeState::Stopped,
                SchedRuntimeState::Running => RuntimeState::Running,
                SchedRuntimeState::Paused => RuntimeState::Paused,
                SchedRuntimeState::Error => RuntimeState::Error,
                SchedRuntimeState::Completed => RuntimeState::Completed,
            },
            tick: exec.tick(),
            total_nodes,
            running_nodes: running,
            completed_nodes: completed,
            errored_nodes: errored,
        }
    }

    /// Capture a snapshot of the graph state
    pub fn capture_graph_snapshot(&self) -> GraphSnapshot {
        let exec = self.executor.lock();
        let node_ids = exec.node_ids();

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for node_id in node_ids {
            if let Some(node_arc) = exec.node(node_id) {
                let inst = node_arc.lock();
                let ports = inst.ports.clone();

                // Create node snapshot
                let node_snapshot = NodeSnapshot {
                    id: inst.id.as_u64(),
                    name: inst.processor.name().to_string(),
                    node_type: "Node".to_string(), // TODO: get actual type name
                    state: match inst.state {
                        SchedNodeState::Idle => NodeState::Idle,
                        SchedNodeState::Ready => NodeState::Ready,
                        SchedNodeState::Running => NodeState::Running,
                        SchedNodeState::Done => NodeState::Done,
                        SchedNodeState::Error => NodeState::Error,
                    },
                    inputs: ports.input_names().into_iter().map(|name| {
                        let port = ports.input(&name).unwrap();
                        PortSnapshot {
                            name: name.clone(),
                            direction: PortDirection::Input,
                            depth: port.depth(),
                            capacity: port.capacity(),
                            connections: 0,
                        }
                    }).collect(),
                    outputs: ports.output_names().into_iter().map(|name| {
                        let port = ports.output(&name).unwrap();
                        PortSnapshot {
                            name: name.clone(),
                            direction: PortDirection::Output,
                            depth: 0,
                            capacity: 0,
                            connections: port.connection_count(),
                        }
                    }).collect(),
                    packets_processed: 0, // TODO: get from processor
                    packets_dropped: 0,
                    last_error: None,
                };
                nodes.push(node_snapshot);
            }
        }

        GraphSnapshot { nodes, edges }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use caret_sched::ExecutorConfig;

    #[test]
    fn test_runtime_integration_create() {
        let executor = Executor::new(ExecutorConfig::default());
        let integration = RuntimeIntegration::new(Arc::new(Mutex::new(executor)));
        let snapshot = integration.capture_runtime_snapshot();
        assert_eq!(snapshot.state, crate::snapshot::RuntimeState::Stopped);
    }
}
