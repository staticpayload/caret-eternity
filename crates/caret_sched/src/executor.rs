// Caret Sched - Graph executor
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{node::NodeId, policy::ExecutionPolicy, port::PortSet, runtime::RuntimeState};
use caret_core::{Error, Result};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration for the executor
#[derive(Clone, Debug)]
pub struct ExecutorConfig {
    /// Default queue capacity for ports
    pub queue_capacity: usize,
    /// Execution policy
    pub policy: ExecutionPolicy,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            queue_capacity: 1024,
            policy: ExecutionPolicy::default(),
        }
    }
}

/// A node instance in the executor
pub struct NodeInstance {
    /// Runtime node ID
    pub id: NodeId,
    /// Graph node ID (for mapping back to the graph)
    pub graph_id: u64,
    /// Node processor
    pub processor: Box<dyn crate::node::NodeProcessor>,
    /// Port set
    pub ports: PortSet,
    /// Current state
    pub state: NodeState,
}

/// State of a node in the executor
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// Graph executor
///
/// The executor manages the runtime execution of a graph,
/// handling scheduling, data flow, and state management.
pub struct Executor {
    /// Executor configuration
    config: ExecutorConfig,
    /// Node instances
    nodes: HashMap<NodeId, Arc<Mutex<NodeInstance>>>,
    /// Runtime state
    runtime_state: Arc<Mutex<RuntimeState>>,
    /// Current tick
    tick: u64,
}

impl Executor {
    /// Create a new executor with the given configuration
    pub fn new(config: ExecutorConfig) -> Self {
        Self {
            config,
            nodes: HashMap::new(),
            runtime_state: Arc::new(Mutex::new(RuntimeState::Stopped)),
            tick: 0,
        }
    }

    /// Create an executor with default configuration
    pub fn with_default_config() -> Self {
        Self::new(ExecutorConfig::default())
    }

    /// Add a node to the executor
    pub fn add_node(
        &mut self,
        graph_id: u64,
        mut processor: Box<dyn crate::node::NodeProcessor>,
    ) -> Result<NodeId> {
        let id = NodeId::unique();
        let ports = PortSet::new();

        // Initialize the processor
        processor.initialize()?;

        let instance = NodeInstance {
            id,
            graph_id,
            processor,
            ports,
            state: NodeState::Idle,
        };

        self.nodes.insert(id, Arc::new(Mutex::new(instance)));
        Ok(id)
    }

    /// Connect two nodes by port name
    pub fn connect(
        &mut self,
        from_node: NodeId,
        from_port: &str,
        to_node: NodeId,
        to_port: &str,
    ) -> Result<()> {
        let from_instance = self
            .nodes
            .get(&from_node)
            .ok_or_else(|| Error::graph(format!("source node {} not found", from_node.as_u64())))?;
        let to_instance = self
            .nodes
            .get(&to_node)
            .ok_or_else(|| Error::graph(format!("target node {} not found", to_node.as_u64())))?;

        // Get ports
        let output = {
            let from = from_instance.lock();
            from.ports.output(from_port).ok_or_else(|| {
                Error::graph(format!(
                    "output port '{}' not found on node {}",
                    from_port,
                    from_node.as_u64()
                ))
            })?
        };

        let input = {
            let to = to_instance.lock();
            to.ports.input(to_port).ok_or_else(|| {
                Error::graph(format!(
                    "input port '{}' not found on node {}",
                    to_port,
                    to_node.as_u64()
                ))
            })?
        };

        // Connect
        output.connect(input)?;

        Ok(())
    }

    /// Get the runtime state
    pub fn runtime_state(&self) -> RuntimeState {
        self.runtime_state.lock().clone()
    }

    /// Get a node instance by ID
    pub fn node(&self, id: NodeId) -> Option<Arc<Mutex<NodeInstance>>> {
        self.nodes.get(&id).cloned()
    }

    /// Get all node IDs
    pub fn node_ids(&self) -> Vec<NodeId> {
        self.nodes.keys().copied().collect()
    }

    /// Get the current tick
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Start the executor
    pub fn start(&mut self) -> Result<()> {
        *self.runtime_state.lock() = RuntimeState::Running;
        Ok(())
    }

    /// Pause the executor
    pub fn pause(&mut self) -> Result<()> {
        *self.runtime_state.lock() = RuntimeState::Paused;
        Ok(())
    }

    /// Stop the executor
    pub fn stop(&mut self) -> Result<()> {
        *self.runtime_state.lock() = RuntimeState::Stopped;
        Ok(())
    }

    /// Execute a single tick
    pub fn tick_once(&mut self) -> Result<TickResult> {
        let state = self.runtime_state.lock().clone();
        if state != RuntimeState::Running {
            return Ok(TickResult::Skipped);
        }

        self.tick += 1;
        let mut nodes_run = 0;
        let mut nodes_done = 0;
        let mut nodes_error = 0;

        for node_id in self.node_ids() {
            let instance = self.node(node_id).unwrap();
            let mut inst = instance.lock();

            if inst.state == NodeState::Done || inst.state == NodeState::Error {
                if inst.state == NodeState::Done {
                    nodes_done += 1;
                } else {
                    nodes_error += 1;
                }
                continue;
            }

            // Check if node has data on any input port
            let has_data = inst.ports.input_names().iter().any(|name| {
                inst.ports
                    .input(name.as_str())
                    .map_or(false, |p| p.has_data())
            });

            if !has_data && !inst.ports.input_names().is_empty() {
                // No data yet, skip
                continue;
            }

            inst.state = NodeState::Running;
            nodes_run += 1;

            // Process packets from each input port
            for port_name in inst.ports.input_names() {
                if let Some(port) = inst.ports.input(&port_name) {
                    while let Some(packet) = port.try_recv() {
                        let ctx = crate::node::ProcessingContext::new(inst.id, self.tick);
                        match inst.processor.process(&ctx, packet, &port_name) {
                            Ok(result) => match result {
                                crate::node::ProcessingResult::Done => {
                                    inst.state = NodeState::Done;
                                }
                                crate::node::ProcessingResult::Error(_e) => {
                                    inst.state = NodeState::Error;
                                }
                                _ => {
                                    inst.state = NodeState::Idle;
                                }
                            },
                            Err(_e) => {
                                inst.state = NodeState::Error;
                            }
                        }
                    }
                }
            }

            inst.state = NodeState::Idle;
        }

        Ok(TickResult::Executed {
            nodes_run,
            nodes_done,
            nodes_error,
        })
    }
}

/// Result of a tick execution
#[derive(Clone, Debug)]
pub enum TickResult {
    /// Tick was skipped (executor not running)
    Skipped,
    /// Tick was executed
    Executed {
        /// Number of nodes that ran
        nodes_run: usize,
        /// Number of nodes that completed
        nodes_done: usize,
        /// Number of nodes that errored
        nodes_error: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::PassthroughNode;

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert_eq!(config.queue_capacity, 1024);
        assert_eq!(config.policy, ExecutionPolicy::Realtime);
    }

    #[test]
    fn test_executor_create() {
        let executor = Executor::with_default_config();
        assert_eq!(executor.tick(), 0);
        assert_eq!(executor.runtime_state(), RuntimeState::Stopped);
    }

    #[test]
    fn test_executor_add_node() {
        let mut executor = Executor::with_default_config();
        let processor = Box::new(PassthroughNode::new("test"));
        let node_id = executor.add_node(1, processor).unwrap();

        assert!(executor.node(node_id).is_some());
        assert_eq!(executor.node_ids().len(), 1);
    }

    #[test]
    fn test_executor_start_stop() {
        let mut executor = Executor::with_default_config();
        assert_eq!(executor.runtime_state(), RuntimeState::Stopped);

        executor.start().unwrap();
        assert_eq!(executor.runtime_state(), RuntimeState::Running);

        executor.pause().unwrap();
        assert_eq!(executor.runtime_state(), RuntimeState::Paused);

        executor.stop().unwrap();
        assert_eq!(executor.runtime_state(), RuntimeState::Stopped);
    }
}
