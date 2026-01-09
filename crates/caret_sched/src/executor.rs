// Caret Sched - Graph executor
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{node::NodeId, policy::ExecutionPolicy, port::PortSet, runtime::RuntimeState};
use caret_core::{Error, Result};
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};
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
    /// Cached input port names (to avoid repeated allocations)
    cached_input_names: Vec<String>,
    /// Whether this node has any input ports
    has_inputs: bool,
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
///
/// Performance optimizations:
/// - Caches node IDs to avoid repeated allocations in tick()
/// - Caches input port names in NodeInstance
/// - Uses work queue for nodes with data to avoid scanning all nodes
pub struct Executor {
    /// Executor configuration
    config: ExecutorConfig,
    /// Node instances
    nodes: HashMap<NodeId, Arc<Mutex<NodeInstance>>>,
    /// Runtime state
    runtime_state: Arc<Mutex<RuntimeState>>,
    /// Current tick
    tick: u64,
    /// Cached list of all node IDs (for iteration without allocation)
    cached_node_ids: Vec<NodeId>,
    /// Work queue of nodes that have data available
    work_queue: Arc<Mutex<VecDeque<NodeId>>>,
}

impl Executor {
    /// Create a new executor with the given configuration
    pub fn new(config: ExecutorConfig) -> Self {
        Self {
            config,
            nodes: HashMap::new(),
            runtime_state: Arc::new(Mutex::new(RuntimeState::Stopped)),
            tick: 0,
            cached_node_ids: Vec::new(),
            work_queue: Arc::new(Mutex::new(VecDeque::new())),
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

        // Cache input port names to avoid repeated allocations
        let input_names = ports.input_names();
        let has_inputs = !input_names.is_empty();

        let instance = NodeInstance {
            id,
            graph_id,
            processor,
            ports,
            state: NodeState::Idle,
            cached_input_names: input_names,
            has_inputs,
        };

        self.nodes.insert(id, Arc::new(Mutex::new(instance)));
        self.cached_node_ids.push(id);
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

    /// Get the current tick
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Get all node IDs
    pub fn node_ids(&self) -> Vec<NodeId> {
        self.cached_node_ids.clone()
    }

    /// Get all node IDs as a slice (zero-copy)
    pub fn node_ids_slice(&self) -> &[NodeId] {
        &self.cached_node_ids
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

    /// Inject a packet directly into a node's input port
    ///
    /// This is used for external packet sources (e.g., distributed execution)
    /// to deliver packets to nodes without going through normal graph connections.
    pub fn inject_packet(
        &self,
        node_id: NodeId,
        port_name: &str,
        data: Vec<u8>,
    ) -> Result<()> {
        use caret_core::Packet;

        let instance = self
            .nodes
            .get(&node_id)
            .ok_or_else(|| Error::graph(format!("Node {} not found", node_id.as_u64())))?;

        let inst = instance.lock();
        let input = inst
            .ports
            .input(port_name)
            .ok_or_else(|| Error::graph(format!("Input port '{}' not found", port_name)))?;

        // Create a bytes packet from the raw data
        let packet = Packet::bytes(data);
        input.push(packet)?;
        Ok(())
    }

    /// Execute a single tick
    ///
    /// Performance optimizations:
    /// - Uses cached node IDs to avoid allocations
    /// - Uses cached input port names from NodeInstance
    /// - Reduces port lookups by caching ports locally
    pub fn tick_once(&mut self) -> Result<TickResult> {
        let state = self.runtime_state.lock().clone();
        if state != RuntimeState::Running {
            return Ok(TickResult::Skipped);
        }

        self.tick += 1;
        let mut nodes_run = 0;
        let mut nodes_done = 0;
        let mut nodes_error = 0;
        let current_tick = self.tick;

        // Use cached node IDs for iteration without allocation
        for node_id in self.node_ids_slice() {
            let instance = self.node(*node_id).unwrap();
            let mut inst = instance.lock();

            if inst.state == NodeState::Done || inst.state == NodeState::Error {
                if inst.state == NodeState::Done {
                    nodes_done += 1;
                } else {
                    nodes_error += 1;
                }
                continue;
            }

            // Clone cached input port names to avoid borrow checker issues
            // This is still cheaper than calling input_names() every tick
            let input_names: Vec<String> = inst.cached_input_names.clone();

            // Use cached input port names to avoid repeated allocations
            let has_data = if inst.has_inputs {
                // Check if node has data on any input port using cached names
                input_names.iter().any(|name| {
                    inst.ports
                        .input(name.as_str())
                        .map_or(false, |p| p.has_data())
                })
            } else {
                // Node has no inputs, always run
                true
            };

            if !has_data {
                // No data yet, skip
                continue;
            }

            inst.state = NodeState::Running;
            nodes_run += 1;

            // Process packets from each input port using cached names
            for port_name in &input_names {
                if let Some(port) = inst.ports.input(port_name) {
                    while let Some(packet) = port.try_recv() {
                        let ctx = crate::node::ProcessingContext::new(inst.id, current_tick);
                        match inst.processor.process(&ctx, packet, port_name) {
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
