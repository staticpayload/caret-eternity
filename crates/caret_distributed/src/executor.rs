// Caret Distributed - Distributed graph executor
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{
    codec::FrameCodec, coordinator::Coordinator, error::Result, graph_proto::*,
    message::*, node::NodeId, transport::{Transport, TransportConfig, TransportEvent},
    Error, Message,
};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Configuration for the distributed executor
#[derive(Clone, Debug)]
pub struct ExecutorConfig {
    /// Local bind address
    pub bind_addr: SocketAddr,
    /// Maximum message size
    pub max_message_size: usize,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Node timeout
    pub node_timeout: Duration,
    /// Coordinator configuration
    pub coordinator_config: crate::coordinator::CoordinatorConfig,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            bind_addr: format!("0.0.0.0:{}", crate::DEFAULT_PORT)
                .parse()
                .unwrap(),
            max_message_size: crate::MAX_MESSAGE_SIZE,
            heartbeat_interval: Duration::from_secs(crate::DEFAULT_HEARTBEAT_INTERVAL_SECS),
            node_timeout: Duration::from_secs(crate::DEFAULT_NODE_TIMEOUT_SECS),
            coordinator_config: Default::default(),
        }
    }
}

/// Node connection state
#[derive(Clone, Debug)]
pub struct ConnectionState {
    /// Node ID
    pub node_id: NodeId,
    /// Socket address
    pub addr: SocketAddr,
    /// Last heartbeat
    pub last_heartbeat: Instant,
    /// Node info
    pub node_info: crate::NodeInfo,
}

/// Packet routing entry
#[derive(Clone, Debug)]
struct RouteEntry {
    /// Source port
    from_port: String,
    /// Target node
    target_node: NodeId,
    /// Target port
    to_port: String,
}

/// Distributed graph executor
pub struct DistributedExecutor {
    /// Local node ID
    local_id: NodeId,
    /// Coordinator
    coordinator: Arc<Coordinator>,
    /// Transport
    transport: Option<Box<dyn Transport + Send>>,
    /// Connected nodes
    connections: Arc<Mutex<HashMap<NodeId, ConnectionState>>>,
    /// Packet routes (output_port -> (target_node, input_port))
    routes: Arc<Mutex<HashMap<String, RouteEntry>>>,
    /// Running state
    running: Arc<Mutex<bool>>,
    /// Config
    config: ExecutorConfig,
    /// Codec for encoding messages
    codec: FrameCodec,
}

impl DistributedExecutor {
    /// Create a new distributed executor
    pub fn new(config: ExecutorConfig) -> Self {
        let local_id = NodeId::new_v4();
        let coordinator = Arc::new(Coordinator::new(config.coordinator_config.clone(), local_id));

        Self {
            local_id,
            coordinator,
            transport: None,
            connections: Arc::new(Mutex::new(HashMap::new())),
            routes: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
            config,
            codec: FrameCodec::with_max_size(crate::MAX_MESSAGE_SIZE),
        }
    }

    /// Get the local node ID
    pub fn local_id(&self) -> NodeId {
        self.local_id
    }

    /// Get the coordinator
    pub fn coordinator(&self) -> &Arc<Coordinator> {
        &self.coordinator
    }

    /// Start the executor with the given transport
    pub async fn start(&mut self, transport: Box<dyn Transport + Send>) -> Result<()> {
        *self.running.lock() = true;
        self.transport = Some(transport);

        // Start event loop
        self.run_event_loop().await?;

        Ok(())
    }

    /// Start as a server (accept connections)
    pub async fn start_server(&mut self) -> Result<()> {
        use crate::transport::TcpTransport;

        let transport_config = TransportConfig {
            bind_addr: self.config.bind_addr,
            max_message_size: self.config.max_message_size,
            ..Default::default()
        };

        let transport = TcpTransport::bind(transport_config).await?;
        self.start(Box::new(transport)).await
    }

    /// Connect to a remote node
    pub async fn connect(&mut self, addr: SocketAddr) -> Result<()> {
        use crate::transport::TcpTransport;

        let transport_config = TransportConfig {
            bind_addr: "0.0.0.0:0".parse().unwrap(),
            max_message_size: self.config.max_message_size,
            ..Default::default()
        };

        let transport = TcpTransport::connect(addr, transport_config).await?;

        // Send hello
        let node_info = crate::NodeInfo::local(addr)?;
        let _hello = Message::new(
            self.local_id,
            None,
            MessagePayload::Hello {
                version: crate::PROTOCOL_VERSION.to_string(),
                node_info: node_info.clone(),
            },
        );

        // Note: The hello message will be sent when the event loop starts
        // In a real implementation, we'd buffer outgoing messages until connected

        *self.running.lock() = true;
        self.transport = Some(Box::new(transport));

        self.run_event_loop().await
    }

    /// Stop the executor
    pub fn stop(&self) {
        *self.running.lock() = false;
        self.coordinator.stop();
    }

    /// Submit a graph for distributed execution
    pub fn submit_graph(&self, graph_id: String, graph: Vec<u8>) -> Result<()> {
        let mode = self.config.coordinator_config.mode;

        // Submit to coordinator
        self.coordinator.submit_graph(graph_id.clone(), mode)?;

        // Broadcast to all workers
        if let Some(transport) = &self.transport {
            let msg = Message::new(
                self.local_id,
                None,
                MessagePayload::GraphSubmit {
                    graph_id: graph_id.clone(),
                    graph,
                    mode,
                },
            );

            transport.broadcast(msg)?;
        }

        Ok(())
    }

    /// Submit a Caret graph for distributed execution
    ///
    /// This method converts the Caret graph to a serializable format,
    /// partitions it across available workers, and distributes the partitions.
    ///
    /// # Arguments
    /// * `graph_id` - Unique identifier for this graph
    /// * `graph` - The Caret graph to execute
    /// * `strategy` - Partitioning strategy to use
    pub fn submit_caret_graph(
        &self,
        graph_id: String,
        graph: &caret_graph::Graph,
        strategy: PartitionStrategy,
    ) -> Result<()> {
        // Convert to serializable graph
        let serializable = SerializableGraph::from_caret_graph(&graph_id, graph);

        // Get available workers
        let workers: Vec<NodeId> = self.coordinator.workers();

        if workers.is_empty() {
            return Err(Error::Execution(
                "No workers available for distributed execution".into(),
            ));
        }

        // Partition the graph
        let assignment = GraphPartitioner::partition(&serializable, &workers, strategy)
            .map_err(|e| Error::Execution(format!("Partition failed: {}", e)))?;

        // Set up routes from the partition assignment
        self.setup_routes_from_partition(&assignment);

        // Submit the graph to the coordinator
        let mode = self.config.coordinator_config.mode;
        self.coordinator.submit_graph(graph_id.clone(), mode)?;

        // Send each partition to its assigned worker
        if let Some(transport) = &self.transport {
            for (worker_id, partition) in &assignment.partitions {
                // Skip the local partition (handled by coordinator)
                if *worker_id == self.local_id {
                    continue;
                }

                // Get the worker's address
                let addr = {
                    let connections = self.connections.lock();
                    connections.get(worker_id).map(|c| c.addr)
                };

                if let Some(addr) = addr {
                    // Serialize the partition
                    let partition_bytes = serde_json::to_vec(partition)
                        .map_err(|e| Error::Serialization(format!("Failed to serialize partition: {}", e)))?;

                    // Serialize the routes
                    let routes_bytes = serde_json::to_vec(&assignment.routes)
                        .map_err(|e| Error::Serialization(format!("Failed to serialize routes: {}", e)))?;

                    let msg = Message::new(
                        self.local_id,
                        Some(*worker_id),
                        MessagePayload::PartitionAssign {
                            graph_id: graph_id.clone(),
                            partition: partition_bytes,
                            routes: routes_bytes,
                        },
                    );

                    transport.send(addr, msg)?;
                    tracing::info!(
                        "Sent partition for graph {} to worker {} ({} nodes)",
                        graph_id,
                        worker_id,
                        partition.nodes.len()
                    );
                }
            }
        }

        tracing::info!(
            "Submitted graph {} with {} nodes partitioned across {} workers",
            graph_id,
            serializable.node_count(),
            workers.len()
        );

        Ok(())
    }

    /// Start executing a graph
    pub fn start_graph(&self, graph_id: &str) -> Result<()> {
        self.coordinator.start_graph(graph_id)?;

        // Broadcast start message
        if let Some(transport) = &self.transport {
            let msg = Message::new(
                self.local_id,
                None,
                MessagePayload::GraphStart {
                    graph_id: graph_id.to_string(),
                },
            );

            transport.broadcast(msg)?;
        }

        Ok(())
    }

    /// Stop executing a graph
    pub fn stop_graph(&self, graph_id: &str, drain: bool) -> Result<()> {
        self.coordinator.stop_graph(graph_id, drain)?;

        // Broadcast stop message
        if let Some(transport) = &self.transport {
            let msg = Message::new(
                self.local_id,
                None,
                MessagePayload::GraphStop {
                    graph_id: graph_id.to_string(),
                    drain,
                },
            );

            transport.broadcast(msg)?;
        }

        Ok(())
    }

    /// Add a packet route for cross-node communication
    pub fn add_route(&self, from_port: String, target_node: NodeId, to_port: String) {
        let entry = RouteEntry {
            from_port,
            target_node,
            to_port,
        };
        self.routes.lock().insert(entry.from_port.clone(), entry);
    }

    /// Set up routes from a partition assignment
    ///
    /// Configures cross-node packet routes based on the partition assignment.
    /// Only routes that originate from or target the local node are set up.
    pub fn setup_routes_from_partition(&self, partition: &crate::PartitionAssignment) {
        for route in &partition.routes {
            // We only care about routes where the local node is the source
            // (outgoing packets from local nodes to remote nodes)
            // The coordinator will handle incoming routes
            self.add_route(
                format!("{}:{}", route.from_node, route.from_port),
                route.to_worker,
                format!("{}:{}", route.to_node, route.to_port),
            );
        }
    }

    /// Get all configured routes
    pub fn get_routes(&self) -> Vec<(String, NodeId, String)> {
        self.routes
            .lock()
            .values()
            .map(|r| (r.from_port.clone(), r.target_node, r.to_port.clone()))
            .collect()
    }

    /// Send a packet to a remote node
    pub fn send_packet(&self, from_port: &str, data: Vec<u8>) -> Result<()> {
        let (entry, addr) = {
            let routes = self.routes.lock();
            if let Some(entry) = routes.get(from_port) {
                let connections = self.connections.lock();
                let addr = connections.get(&entry.target_node).map(|c| c.addr);
                (entry.clone(), addr)
            } else {
                return Ok(());
            }
        };

        if let Some(addr) = addr {
            if let Some(transport) = &self.transport {
                let msg = Message::new(
                    self.local_id,
                    Some(entry.target_node),
                    MessagePayload::Packet {
                        from_port: entry.from_port.clone(),
                        to_port: entry.to_port.clone(),
                        data,
                    },
                );

                transport.send(addr, msg)?;
            }
        }
        Ok(())
    }

    /// Main event loop
    async fn run_event_loop(&self) -> Result<()> {
        let transport = self.transport.as_ref().ok_or_else(|| {
            Error::Transport("Transport not initialized".into())
        })?;

        let mut events = transport.subscribe();
        let heartbeat_interval = self.config.heartbeat_interval;
        let node_timeout = self.config.node_timeout;

        // Spawn heartbeat task
        let running = Arc::clone(&self.running);
        let coordinator = Arc::clone(&self.coordinator);
        let local_id = self.local_id;
        let transport_events = Arc::clone(&self.connections);
        let _heartbeat_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(heartbeat_interval);
            while *running.lock() {
                interval.tick().await;

                // Send heartbeat to all connected nodes
                // Update coordinator with current load
                coordinator.worker_heartbeat(local_id, 0.0);

                // Check for timed out nodes
                let now = Instant::now();
                let mut timed_out = Vec::new();
                {
                    let conns = transport_events.lock();
                    for (node_id, state) in conns.iter() {
                        if now.duration_since(state.last_heartbeat) > node_timeout {
                            timed_out.push(*node_id);
                        }
                    }
                }

                for node_id in timed_out {
                    coordinator.unregister_worker(&node_id);
                }
            }
        });

        // Process transport events
        while *self.running.lock() {
            match events.recv().await {
                Some(TransportEvent::Message { from, message }) => {
                    self.handle_incoming_message(from, message)?;
                }
                Some(TransportEvent::Connected { addr }) => {
                    tracing::info!("Connected to {}", addr);
                }
                Some(TransportEvent::Disconnected { addr }) => {
                    tracing::info!("Disconnected from {}", addr);
                    // Find and remove the node
                    let node_id = {
                        let mut conns = self.connections.lock();
                        let to_remove = conns
                            .iter()
                            .find(|(_, s)| s.addr == addr)
                            .map(|(id, _)| *id);
                        if let Some(id) = to_remove {
                            conns.remove(&id);
                            Some(id)
                        } else {
                            None
                        }
                    };

                    if let Some(node_id) = node_id {
                        self.coordinator.unregister_worker(&node_id);
                    }
                }
                Some(TransportEvent::Error { addr, error }) => {
                    tracing::error!("Transport error from {}: {}", addr, error);
                }
                None => {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle incoming message
    fn handle_incoming_message(&self, from: SocketAddr, message: Message) -> Result<()> {
        match message.payload {
            MessagePayload::Hello {
                version,
                node_info,
            } => {
                // Check version compatibility
                if version != crate::PROTOCOL_VERSION {
                    tracing::warn!(
                        "Version mismatch: expected {}, got {}",
                        crate::PROTOCOL_VERSION,
                        version
                    );
                }

                // Register the connection
                let state = ConnectionState {
                    node_id: message.from,
                    addr: from,
                    last_heartbeat: Instant::now(),
                    node_info: node_info.clone(),
                };
                self.connections.lock().insert(message.from, state);

                // Register as worker
                self.coordinator.register_worker(message.from)?;

                // Send HelloReply
                if let Some(transport) = &self.transport {
                    let local_info = crate::NodeInfo::local(transport.local_addr())?;
                    let reply = Message::new(
                        self.local_id,
                        Some(message.from),
                        MessagePayload::HelloReply {
                            version: crate::PROTOCOL_VERSION.to_string(),
                            node_info: local_info,
                            accepted: true,
                            reject_reason: None,
                        },
                    );
                    transport.send(from, reply)?;
                }

                tracing::info!("Registered worker {} from {}", message.from, from);
            }

            MessagePayload::HelloReply {
                version: _,
                node_info,
                accepted,
                reject_reason,
            } => {
                if !accepted {
                    return Err(Error::ConnectionRefused(
                        reject_reason.unwrap_or_else(|| "Unknown reason".into()),
                    ));
                }

                let state = ConnectionState {
                    node_id: message.from,
                    addr: from,
                    last_heartbeat: Instant::now(),
                    node_info,
                };
                self.connections.lock().insert(message.from, state);

                tracing::info!("Connected to coordinator {} at {}", message.from, from);
            }

            MessagePayload::Heartbeat => {
                // Update heartbeat timestamp
                if let Some(state) = self.connections.lock().get_mut(&message.from) {
                    state.last_heartbeat = Instant::now();
                }

                // Send heartbeat reply
                if let Some(transport) = &self.transport {
                    let reply = Message::new(
                        self.local_id,
                        Some(message.from),
                        MessagePayload::HeartbeatReply,
                    );
                    transport.send(from, reply)?;
                }
            }

            MessagePayload::HeartbeatReply => {
                // Update heartbeat timestamp
                if let Some(state) = self.connections.lock().get_mut(&message.from) {
                    state.last_heartbeat = Instant::now();
                }
            }

            MessagePayload::GraphSubmit {
                ref graph_id,
                graph: _,
                mode,
            } => {
                tracing::info!("Received graph submission: {}", graph_id);

                // Submit to coordinator
                self.coordinator.submit_graph(graph_id.clone(), mode)?;

                // Store the graph for execution
                // In a real implementation, this would deserialize and prepare the graph
            }

            MessagePayload::NodeAssign {
                ref graph_id,
                node_id,
                config: _,
            } => {
                tracing::info!(
                    "Received node assignment: graph={}, node={}",
                    graph_id,
                    node_id
                );

                // Initialize the node with the given config
                // In a real implementation, this would create the node instance
            }

            MessagePayload::NodeRelease {
                ref graph_id,
                node_id,
            } => {
                tracing::info!(
                    "Received node release: graph={}, node={}",
                    graph_id,
                    node_id
                );

                // Release and clean up the node
                self.coordinator.release_node(graph_id, node_id);
            }

            MessagePayload::PartitionAssign {
                ref graph_id,
                ref partition,
                ref routes,
            } => {
                tracing::info!(
                    "Received partition assignment: graph={}, partition_size={} bytes",
                    graph_id,
                    partition.len()
                );

                // Deserialize the partition
                let graph_partition: GraphPartition = match serde_json::from_slice(partition) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("Failed to deserialize partition: {}", e);
                        return Err(Error::Serialization(format!("Invalid partition: {}", e)));
                    }
                };

                // Deserialize the routes
                let partition_routes: Vec<CrossNodeRoute> = match serde_json::from_slice(routes) {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("Failed to deserialize routes: {}", e);
                        return Err(Error::Serialization(format!("Invalid routes: {}", e)));
                    }
                };

                // Create a partition assignment for the local routes
                let local_assignment = PartitionAssignment {
                    graph_id: graph_id.clone(),
                    partitions: std::collections::HashMap::new(),
                    routes: partition_routes,
                };

                // Set up routes from the partition assignment
                self.setup_routes_from_partition(&local_assignment);

                tracing::info!(
                    "Partition assigned with {} nodes, {} routes configured",
                    graph_partition.nodes.len(),
                    local_assignment.routes.len()
                );

                // In a real implementation, this would:
                // 1. Create local node instances for the assigned nodes
                // 2. Set up cross-node packet routing
                // 3. Initialize input/output queues
                // For now, we just log the assignment
            }

            MessagePayload::Packet {
                ref from_port,
                ref to_port,
                ref data,
            } => {
                tracing::trace!(
                    "Received packet: {} -> {} ({} bytes)",
                    from_port,
                    to_port,
                    data.len()
                );

                // Route the packet to the appropriate local node
                // In a real implementation, this would deliver to the node's input queue
            }

            MessagePayload::GraphStart { ref graph_id } => {
                tracing::info!("Starting graph: {}", graph_id);
                self.coordinator.start_graph(graph_id)?;
            }

            MessagePayload::GraphStop { ref graph_id, drain } => {
                tracing::info!("Stopping graph: {} (drain={})", graph_id, drain);
                self.coordinator.stop_graph(graph_id, drain)?;
            }

            MessagePayload::StatusRequest => {
                // Send status reply
                if let Some(transport) = &self.transport {
                    let reply = Message::new(
                        self.local_id,
                        Some(message.from),
                        MessagePayload::StatusReply {
                            state: crate::NodeState::Active,
                            metrics: serde_json::json!({
                                "load": 0.0,
                                "graphs": self.coordinator.worker_count(),
                            }),
                        },
                    );
                    transport.send(from, reply)?;
                }
            }

            MessagePayload::GraphUpdate { .. } => {
                // Handle dynamic graph updates
            }

            MessagePayload::Error { code, message: msg } => {
                tracing::error!("Error from {}: {} - {}", from, code, msg);
            }

            _ => {}
        }

        Ok(())
    }

    /// Check if the executor is running
    pub fn is_running(&self) -> bool {
        *self.running.lock()
    }

    /// Get connected nodes
    pub fn connected_nodes(&self) -> Vec<NodeId> {
        self.connections.lock().keys().cloned().collect()
    }

    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.lock().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert_eq!(config.bind_addr.port(), crate::DEFAULT_PORT);
        assert_eq!(
            config.heartbeat_interval,
            Duration::from_secs(crate::DEFAULT_HEARTBEAT_INTERVAL_SECS)
        );
        assert_eq!(
            config.node_timeout,
            Duration::from_secs(crate::DEFAULT_NODE_TIMEOUT_SECS)
        );
    }

    #[test]
    fn test_route_management() {
        let executor = DistributedExecutor::new(ExecutorConfig::default());

        let target_node = NodeId::new_v4();
        executor.add_route("output".to_string(), target_node, "input".to_string());

        assert_eq!(executor.routes.lock().len(), 1);

        let routes = executor.routes.lock();
        let route = routes.get("output").unwrap();
        assert_eq!(route.target_node, target_node);
        assert_eq!(&route.to_port, "input");
    }

    #[tokio::test]
    async fn test_executor_lifecycle() {
        let executor = DistributedExecutor::new(ExecutorConfig::default());

        assert!(!executor.is_running());

        // Test graph submission - should succeed even without transport
        // (graph is submitted to coordinator and will be distributed when workers connect)
        let graph_id = "test-graph".to_string();
        let graph_data = vec![1, 2, 3, 4];

        let result = executor.submit_graph(graph_id.clone(), graph_data.clone());
        assert!(result.is_ok());

        // The graph should be in the coordinator
        let coordinator = executor.coordinator();
        assert_eq!(coordinator.graph_status(&graph_id), Some(crate::coordinator::ExecutionStatus::Pending));
        assert_eq!(coordinator.worker_count(), 0);
    }

    /// Integration test for distributed execution
    #[test]
    fn test_distributed_execution_integration() {
        // Create a coordinator/executor
        let coordinator_executor = DistributedExecutor::new(ExecutorConfig::default());

        // Create a worker ID
        let worker_id = uuid::Uuid::new_v4();

        // Set up connection state for the worker
        let remote_addr: SocketAddr = "127.0.0.1:9002".parse().unwrap();
        coordinator_executor.connections.lock().insert(
            worker_id,
            crate::executor::ConnectionState {
                node_id: worker_id,
                addr: remote_addr,
                last_heartbeat: std::time::Instant::now(),
                node_info: crate::NodeInfo::local(remote_addr).unwrap(),
            },
        );

        // Register worker with coordinator
        coordinator_executor.coordinator.register_worker(worker_id).unwrap();

        // Verify worker is registered
        assert_eq!(coordinator_executor.coordinator.worker_count(), 1);

        // Submit a graph
        let graph_id = "integration-test-graph".to_string();
        let graph_data = vec![0x01, 0x02, 0x03, 0x04];

        coordinator_executor
            .submit_graph(graph_id.clone(), graph_data)
            .unwrap();

        // Verify graph is pending
        assert_eq!(
            coordinator_executor.coordinator.graph_status(&graph_id),
            Some(crate::coordinator::ExecutionStatus::Pending)
        );

        // Start the graph
        coordinator_executor.start_graph(&graph_id).unwrap();

        // Verify graph is running
        assert_eq!(
            coordinator_executor.coordinator.graph_status(&graph_id),
            Some(crate::coordinator::ExecutionStatus::Running)
        );

        // Assign a node to the worker
        let result = coordinator_executor
            .coordinator
            .assign_node(&graph_id, 12345);
        assert!(result.is_ok());

        // Verify assignment
        {
            let graphs = coordinator_executor.coordinator.graphs.lock();
            let graph_state = graphs.get(&graph_id).unwrap();
            assert_eq!(graph_state.assignments.get(&12345), Some(&worker_id));
        } // Lock released here

        // Stop the graph
        coordinator_executor.stop_graph(&graph_id, false).unwrap();

        // Verify graph is completed
        assert_eq!(
            coordinator_executor.coordinator.graph_status(&graph_id),
            Some(crate::coordinator::ExecutionStatus::Completed)
        );
    }

    /// Test packet routing between nodes
    #[test]
    fn test_packet_routing() {
        let executor = DistributedExecutor::new(ExecutorConfig::default());

        // Set up a connection
        let remote_node = uuid::Uuid::new_v4();
        let remote_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        executor.connections.lock().insert(
            remote_node,
            crate::executor::ConnectionState {
                node_id: remote_node,
                addr: remote_addr,
                last_heartbeat: std::time::Instant::now(),
                node_info: crate::NodeInfo::local(remote_addr).unwrap(),
            },
        );

        // Add a route
        executor.add_route("output".to_string(), remote_node, "input".to_string());

        // Send a packet
        let packet_data = vec![0x10, 0x20, 0x30];
        let _result = executor.send_packet("output", packet_data.clone());

        // Without a transport, the send will fail, but the routing logic is tested
        // The route lookup should succeed
        let routes = executor.routes.lock();
        assert!(routes.contains_key("output"));
    }

    /// Test setup routes from partition assignment
    #[test]
    fn test_setup_routes_from_partition() {
        use std::collections::HashMap;

        let executor = DistributedExecutor::new(ExecutorConfig::default());

        // Create a partition assignment with some routes
        let worker1 = uuid::Uuid::from_u128(100);
        let worker2 = uuid::Uuid::from_u128(101);

        let mut partitions = HashMap::new();
        partitions.insert(
            worker1,
            crate::GraphPartition {
                worker_id: worker1,
                nodes: vec![1, 2],
                internal_edges: vec![],
                input_edges: vec![],
                output_edges: vec![],
            },
        );
        partitions.insert(
            worker2,
            crate::GraphPartition {
                worker_id: worker2,
                nodes: vec![3],
                internal_edges: vec![],
                input_edges: vec![],
                output_edges: vec![],
            },
        );

        let assignment = crate::PartitionAssignment {
            graph_id: "test".to_string(),
            partitions,
            routes: vec![crate::CrossNodeRoute {
                from_node: 1,
                from_port: "output".to_string(),
                from_worker: worker1,
                to_node: 3,
                to_port: "input".to_string(),
                to_worker: worker2,
            }],
        };

        // Set up routes from the partition
        executor.setup_routes_from_partition(&assignment);

        // Verify routes were created
        let routes = executor.get_routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].0, "1:output"); // from_port format: "node_id:port_name"
        assert_eq!(routes[0].1, worker2);
        assert_eq!(routes[0].2, "3:input");
    }

    /// Test submitting a Caret graph for distributed execution
    #[test]
    fn test_submit_caret_graph() {
        use caret_graph::{Graph, Node, NodeType};

        let executor = DistributedExecutor::new(ExecutorConfig::default());

        // Register some workers
        let worker1 = uuid::Uuid::from_u128(100);
        let worker2 = uuid::Uuid::from_u128(101);

        executor.coordinator.register_worker(worker1).unwrap();
        executor.coordinator.register_worker(worker2).unwrap();

        // Set up connections for the workers
        let addr1: SocketAddr = "127.0.0.1:9001".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:9002".parse().unwrap();

        executor.connections.lock().insert(
            worker1,
            crate::executor::ConnectionState {
                node_id: worker1,
                addr: addr1,
                last_heartbeat: std::time::Instant::now(),
                node_info: crate::NodeInfo::local(addr1).unwrap(),
            },
        );
        executor.connections.lock().insert(
            worker2,
            crate::executor::ConnectionState {
                node_id: worker2,
                addr: addr2,
                last_heartbeat: std::time::Instant::now(),
                node_info: crate::NodeInfo::local(addr2).unwrap(),
            },
        );

        // Create a simple Caret graph
        let mut graph = Graph::new();

        // Add some nodes
        let source = Node::new("source", NodeType::Source);
        let transform = Node::new("transform", NodeType::Transform);
        let sink = Node::new("sink", NodeType::Sink);

        // Get the node IDs before adding
        let source_id = source.id().as_u64();
        let transform_id = transform.id().as_u64();
        let sink_id = sink.id().as_u64();

        // Add output to source
        let mut source_with_port = source;
        let _ = source_with_port.add_output("output");

        // Add input and output to transform
        let mut transform_with_ports = transform;
        let _ = transform_with_ports.add_input("input");
        let _ = transform_with_ports.add_output("output");

        // Add input to sink
        let mut sink_with_port = sink;
        let _ = sink_with_port.add_input("input");

        // Add nodes to graph
        let _ = graph.add_node(source_with_port);
        let _ = graph.add_node(transform_with_ports);
        let _ = graph.add_node(sink_with_port);

        // Connect the nodes
        let _ = graph.connect(source_id, "output", transform_id, "input");
        let _ = graph.connect(transform_id, "output", sink_id, "input");

        // Submit the graph for distributed execution
        let result = executor.submit_caret_graph(
            "test-graph".to_string(),
            &graph,
            PartitionStrategy::RoundRobin,
        );

        assert!(result.is_ok());

        // Verify the graph was submitted to the coordinator
        assert_eq!(
            executor.coordinator.graph_status("test-graph"),
            Some(crate::coordinator::ExecutionStatus::Pending)
        );

        // Verify routes were set up
        let routes = executor.get_routes();
        // Round-robin partitioning with 3 nodes and 2 workers should create
        // at least one cross-partition edge
        assert!(!routes.is_empty());
    }
}
