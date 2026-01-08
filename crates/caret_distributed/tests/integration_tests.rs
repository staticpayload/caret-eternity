// Caret Distributed - Integration Tests
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

//! Integration tests for distributed execution infrastructure.

use caret_distributed::{
    Coordinator, CoordinatorConfig, DistributedExecutor, ExecutorConfig, FrameCodec,
    LocalNode, Message, MessagePayload, NodeId, NodeInfo, TransportConfig,
};
use std::net::SocketAddr;

/// Helper to create a test node ID
fn test_node_id(seed: u8) -> NodeId {
    NodeId::from_bytes([
        seed, seed, seed, seed, seed, seed, seed, seed, seed, seed, seed, seed, seed, seed,
        seed, seed,
    ])
}

// ============================================================================
// Message Codec Tests
// ============================================================================

#[test]
fn test_message_roundtrip() {
    let codec = FrameCodec::default();

    // Test Heartbeat message
    let msg = Message::new(test_node_id(1), None, MessagePayload::Heartbeat);
    let encoded = codec.encode(&msg).expect("Failed to encode");
    let decoded = codec.decode(&encoded).expect("Failed to decode");

    assert_eq!(msg.from, decoded.from);
    assert_eq!(msg.to, decoded.to);
    assert_eq!(msg.message_type(), decoded.message_type());
}

#[test]
fn test_frame_codec_max_size() {
    let codec = FrameCodec::with_max_size(1024);

    // Create a message larger than max size
    let large_data = vec![0u8; 2000];
    let message = Message::new(
        test_node_id(1),
        None,
        MessagePayload::GraphSubmit {
            graph_id: "large-graph".to_string(),
            graph: large_data,
            mode: caret_distributed::ExecutionMode::Pipeline,
        },
    );

    assert!(codec.encode(&message).is_err());
}

#[test]
fn test_message_creation() {
    let from = test_node_id(1);
    let to = test_node_id(2);
    let msg = Message::new(from, Some(to), MessagePayload::Heartbeat);

    assert_eq!(msg.from, from);
    assert_eq!(msg.to, Some(to));
    assert!(msg.is_addressed_to(&to));
    assert!(!msg.is_broadcast());

    let broadcast = Message::new(from, None, MessagePayload::Heartbeat);
    assert!(broadcast.is_broadcast());
}

// ============================================================================
// Coordinator Tests
// ============================================================================

#[tokio::test]
async fn coordinator_worker_registration() {
    let config = CoordinatorConfig::default();
    let local_id = test_node_id(1);

    let coordinator = Coordinator::new(config, local_id);
    coordinator.start();

    // Register a worker
    let worker_id = test_node_id(2);
    coordinator.register_worker(worker_id).unwrap();

    // Verify worker is registered
    let workers = coordinator.workers();
    assert_eq!(workers.len(), 1);
    assert_eq!(workers[0], worker_id);

    coordinator.stop();
}

#[tokio::test]
async fn coordinator_graph_lifecycle() {
    let config = CoordinatorConfig::default();
    let local_id = test_node_id(1);

    let coordinator = Coordinator::new(config, local_id);
    coordinator.start();

    // Submit a graph
    let graph_id = "test-graph";
    assert!(coordinator
        .submit_graph(graph_id.to_string(), caret_distributed::ExecutionMode::Pipeline)
        .is_ok());

    // Verify graph is in pending state
    let graphs = coordinator.graphs.lock();
    assert!(graphs.contains_key(graph_id));
    assert_eq!(graphs[graph_id].status, caret_distributed::ExecutionStatus::Pending);
    drop(graphs);

    // Start graph execution
    assert!(coordinator.start_graph(graph_id).is_ok());

    // Stop graph execution
    assert!(coordinator.stop_graph(graph_id, false).is_ok());

    coordinator.stop();
}

// ============================================================================
// LocalNode Tests
// ============================================================================

#[test]
fn test_local_node_creation() {
    let addr: SocketAddr = "127.0.0.1:9234".parse().unwrap();
    let info = NodeInfo::new(test_node_id(1), "test".into(), addr, 4, 1024);
    let local_node = LocalNode::new(info);

    assert_eq!(local_node.info().cores, 4);
    assert_eq!(local_node.info().memory_bytes, 1024);
}

// ============================================================================
// Distributed Executor Tests
// ============================================================================

#[test]
fn test_distributed_executor_creation() {
    let executor_config = ExecutorConfig::default();
    let executor = DistributedExecutor::new(executor_config);
    assert!(!executor.local_id().is_nil());
}

// ============================================================================
// MdnsDiscovery Tests
// ============================================================================

#[test]
fn test_mdns_discovery_config_default() {
    let config = caret_distributed::MdnsDiscoveryConfig::default();

    assert_eq!(config.service_type, caret_distributed::CARET_SERVICE_TYPE);
    assert_eq!(config.port, caret_distributed::DEFAULT_PORT);
    assert!(config.service_name.len() > 0);
}

#[test]
fn test_mdns_discovery_config_builder() {
    let config = caret_distributed::MdnsDiscoveryConfig::default()
        .with_service_name("test-node")
        .with_port(8080)
        .with_txt("version", "0.1.0");

    assert_eq!(config.service_name, "test-node");
    assert_eq!(config.port, 8080);
    assert_eq!(config.txt_info.get("version"), Some(&"0.1.0".to_string()));
}

// ============================================================================
// NodeId Tests
// ============================================================================

#[test]
fn test_node_id_from_bytes() {
    let bytes = [1u8; 16];
    let id = NodeId::from_bytes(bytes);

    assert!(!id.is_nil());
    assert_eq!(id.as_bytes(), &bytes);
}

#[test]
fn test_node_id_generation() {
    let id1 = NodeId::new_v4();
    let id2 = NodeId::new_v4();

    assert_ne!(id1, id2);
    assert!(!id1.is_nil());
    assert!(!id2.is_nil());
}

#[test]
fn test_node_id_nil() {
    let nil_id = NodeId::nil();
    assert!(nil_id.is_nil());
    assert_eq!(nil_id.as_bytes(), &[0u8; 16]);
}

// ============================================================================
// NodeInfo Tests
// ============================================================================

#[test]
fn test_node_info_creation() {
    let id = test_node_id(1);
    let addr: SocketAddr = "127.0.0.1:9001".parse().unwrap();

    let info = NodeInfo::new(id.clone(), "test-node".into(), addr, 4, 1024);

    assert_eq!(info.id, id);
    assert_eq!(info.hostname, "test-node");
    assert_eq!(info.addr, addr);
    assert_eq!(info.cores, 4);
    assert_eq!(info.memory_bytes, 1024);
}

// ============================================================================
// Transport Config Tests
// ============================================================================

#[test]
fn test_transport_config_default() {
    let config = TransportConfig::default();

    assert_eq!(config.max_message_size, caret_distributed::MAX_MESSAGE_SIZE);
    assert_eq!(config.send_buffer_size, 1024);
    assert_eq!(config.recv_buffer_size, 1024);
}

#[test]
fn test_transport_config_builder() {
    let config = TransportConfig::default()
        .with_max_message_size(1024)
        .with_send_buffer_size(2048)
        .with_recv_buffer_size(4096);

    assert_eq!(config.max_message_size, 1024);
    assert_eq!(config.send_buffer_size, 2048);
    assert_eq!(config.recv_buffer_size, 4096);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_error_display() {
    use caret_distributed::Error;

    let errors = vec![
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "test",
        )),
        Error::Serialization("invalid data".to_string()),
        Error::NodeTimeout("timeout".to_string()),
        Error::VersionMismatch {
            expected: "1.0".to_string(),
            actual: "2.0".to_string(),
        },
    ];

    for error in errors {
        let display = format!("{}", error);
        assert!(!display.is_empty());
    }
}
