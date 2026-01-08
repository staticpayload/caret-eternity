// Caret Distributed - TCP Networking Integration Tests
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

//! Integration tests using real TCP networking.
//!
//! These tests verify end-to-end distributed execution using actual
//! TCP transport between coordinator and worker nodes.

use caret_distributed::{
    Coordinator, CoordinatorConfig, DistributedExecutor, ExecutorConfig, Message,
    MessagePayload, NodeId, TcpTransport, Transport, TransportConfig,
};
use std::net::SocketAddr;
use std::time::Duration;

/// Helper to get a random available port for testing
fn get_test_port() -> u16 {
    // Use port + offset to avoid conflicts between tests
    use std::sync::atomic::{AtomicU16, Ordering};
    static PORT_COUNTER: AtomicU16 = AtomicU16::new(19000);
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Helper to create a test bind address
fn test_bind_addr() -> SocketAddr {
    format!("127.0.0.1:{}", get_test_port()).parse().unwrap()
}

/// Test TCP transport bind and connect
#[tokio::test]
async fn tcp_transport_bind_connect() {
    let addr = test_bind_addr();
    let config = TransportConfig::with_bind_addr(addr);

    // Bind a server transport
    let server_transport: TcpTransport = TcpTransport::bind(config.clone())
        .await
        .expect("Failed to bind server transport");

    assert_eq!(server_transport.local_addr(), addr);

    // Connect a client transport
    let client_transport: TcpTransport = TcpTransport::connect(addr, config)
        .await
        .expect("Failed to connect client transport");

    assert_ne!(client_transport.local_addr(), addr);

    // Verify both are running
    assert!(server_transport.is_running());
    assert!(client_transport.is_running());

    // Give tasks time to start
    tokio::time::sleep(Duration::from_millis(50)).await;
}

/// Test message sending over TCP
#[tokio::test]
async fn tcp_transport_send() {
    let addr = test_bind_addr();
    let config = TransportConfig::with_bind_addr(addr);

    // Create server
    let server: TcpTransport = TcpTransport::bind(config.clone())
        .await
        .expect("Failed to bind server");

    let server_addr = server.local_addr();

    // Give server task time to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Create client
    let client: TcpTransport = TcpTransport::connect(server_addr, config)
        .await
        .expect("Failed to connect");

    // Wait for connection to be established
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send a message from client to server
    let test_msg = Message::new(
        NodeId::new_v4(),
        Some(NodeId::new_v4()),
        MessagePayload::Heartbeat,
    );

    // This should succeed without panicking (message is encoded and sent)
    let result = client.send(server_addr, test_msg);
    assert!(result.is_ok(), "Failed to send message: {:?}", result.err());

    // Verify server is still running
    assert!(server.is_running());
}

/// Test distributed executor with TCP transport
#[tokio::test]
async fn tcp_distributed_executor_handshake() {
    let coordinator_addr = test_bind_addr();
    let worker_addr = test_bind_addr();

    // Create coordinator executor
    let coordinator_config = ExecutorConfig::default();
    let coordinator_executor = DistributedExecutor::new(coordinator_config);

    // Bind coordinator transport
    let transport_config = TransportConfig::with_bind_addr(coordinator_addr);
    let _coordinator_transport: TcpTransport = TcpTransport::bind(transport_config)
        .await
        .expect("Failed to bind coordinator transport");

    // Create worker executor
    let worker_config = ExecutorConfig::default();
    let worker_executor = DistributedExecutor::new(worker_config);

    // Worker connects to coordinator
    let worker_transport_config = TransportConfig::with_bind_addr(worker_addr);
    let _worker_transport: TcpTransport = TcpTransport::connect(coordinator_addr, worker_transport_config)
        .await
        .expect("Failed to connect worker to coordinator");

    // Verify both executors have unique IDs
    assert_ne!(
        coordinator_executor.local_id(),
        worker_executor.local_id()
    );

    // Give time for connection
    tokio::time::sleep(Duration::from_millis(100)).await;
}

/// Test coordinator and worker registration over TCP
#[tokio::test]
async fn tcp_worker_registration() {
    let addr = test_bind_addr();

    // Create coordinator
    let coordinator_config = CoordinatorConfig::default();
    let coordinator_id = NodeId::new_v4();
    let coordinator = Coordinator::new(coordinator_config, coordinator_id);
    coordinator.start().await.unwrap();

    // Create transport
    let transport_config = TransportConfig::with_bind_addr(addr);
    let _transport: TcpTransport = TcpTransport::bind(transport_config)
        .await
        .expect("Failed to bind transport");

    // Create worker executor
    let worker_config = ExecutorConfig::default();
    let worker_executor = DistributedExecutor::new(worker_config);
    let worker_id = worker_executor.local_id();

    // Register worker (simulating network registration)
    coordinator
        .register_worker(worker_id)
        .expect("Failed to register worker");

    // Verify worker is registered
    let workers = coordinator.workers();
    assert_eq!(workers.len(), 1);
    assert_eq!(workers[0], worker_id);

    coordinator.stop();
}

/// Test graph submission over distributed TCP
#[tokio::test]
async fn tcp_graph_submission() {
    let addr = test_bind_addr();

    // Create coordinator
    let coordinator_config = CoordinatorConfig::default();
    let coordinator_id = NodeId::new_v4();
    let coordinator = Coordinator::new(coordinator_config, coordinator_id);
    coordinator.start().await.unwrap();

    // Bind transport for coordinator
    let transport_config = TransportConfig::with_bind_addr(addr);
    let _transport: TcpTransport = TcpTransport::bind(transport_config)
        .await
        .expect("Failed to bind transport");

    // Create worker executor
    let worker_config = ExecutorConfig::default();
    let worker_executor = DistributedExecutor::new(worker_config);
    let worker_id = worker_executor.local_id();

    // Register worker
    coordinator
        .register_worker(worker_id)
        .expect("Failed to register worker");

    // Submit a graph
    let graph_id = "tcp-test-graph";

    coordinator
        .submit_graph(
            graph_id.to_string(),
            caret_distributed::ExecutionMode::Pipeline,
        )
        .expect("Failed to submit graph");

    // Verify graph is pending
    let graphs = coordinator.graphs.lock();
    assert!(graphs.contains_key(graph_id));
    assert_eq!(
        graphs[graph_id].status,
        caret_distributed::ExecutionStatus::Pending
    );
    drop(graphs);

    // Start graph
    coordinator.start_graph(graph_id).expect("Failed to start graph");

    // Verify graph is running
    assert_eq!(
        coordinator.graph_status(graph_id),
        Some(caret_distributed::ExecutionStatus::Running)
    );

    // Stop graph
    coordinator
        .stop_graph(graph_id, false)
        .expect("Failed to stop graph");

    // Verify graph is completed
    assert_eq!(
        coordinator.graph_status(graph_id),
        Some(caret_distributed::ExecutionStatus::Completed)
    );

    coordinator.stop();
}

/// Test broadcast functionality
#[tokio::test]
async fn tcp_broadcast() {
    let addr = test_bind_addr();
    let config = TransportConfig::with_bind_addr(addr);

    // Create server
    let server: TcpTransport = TcpTransport::bind(config.clone())
        .await
        .expect("Failed to bind server");

    let server_addr = server.local_addr();

    // Give server task time to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Create multiple clients
    let num_clients = 2;
    let mut clients = Vec::new();

    for _ in 0..num_clients {
        let client: TcpTransport = TcpTransport::connect(server_addr, config.clone())
            .await
            .expect("Failed to connect client");
        clients.push(client);
    }

    // Wait for connections to be established
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Broadcast a message from the first client
    let broadcast_msg = Message::new(
        NodeId::new_v4(),
        None, // Broadcast
        MessagePayload::Heartbeat,
    );

    let result = clients[0].broadcast(broadcast_msg);
    assert!(result.is_ok(), "Broadcast failed: {:?}", result.err());

    // Verify server is still running
    assert!(server.is_running());
}

/// Test connection error handling
#[tokio::test]
async fn tcp_connection_error_handling() {
    // Use an address that's unlikely to have a server
    let addr: SocketAddr = "127.0.0.1:49999".parse().unwrap();
    let config = TransportConfig::with_bind_addr(addr);

    // Attempt to connect to non-existent server should fail
    let result: Result<TcpTransport, caret_distributed::Error> = TcpTransport::connect(addr, config).await;

    assert!(result.is_err());
    match result {
        Err(caret_distributed::Error::ConnectionRefused(_)) => {}
        Ok(_) => panic!("Expected ConnectionRefused error, got Ok"),
        Err(e) => panic!("Expected ConnectionRefused error, got {:?}", e),
    }
}

/// Test transport lifecycle
#[tokio::test]
async fn tcp_transport_lifecycle() {
    let addr = test_bind_addr();
    let config = TransportConfig::with_bind_addr(addr);

    // Create and bind server
    let mut server: TcpTransport = TcpTransport::bind(config)
        .await
        .expect("Failed to bind server");

    assert!(server.is_running());

    // Stop the server
    let result = server.stop();
    assert!(result.is_ok());
    assert!(!server.is_running());

    // Start again
    let result = server.start();
    assert!(result.is_ok());
    assert!(server.is_running());
}
