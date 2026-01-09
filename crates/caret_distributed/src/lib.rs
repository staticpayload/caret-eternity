// Caret Distributed - Distributed execution support for Caret
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

//! Distributed execution support for Caret pipelines.
//!
//! This crate provides:
//! - Network transport layer with TCP and TLS support
//! - Node discovery and cluster management
//! - Distributed graph execution coordination
//! - Protocol messages for cluster communication

#![warn(missing_docs)]
#![warn(clippy::all)]

mod codec;
mod coordinator;
mod discovery;
mod error;
mod executor;
mod graph_proto;
mod mdns;
mod message;
mod node;
mod node_factory;
mod transport;

pub use codec::{FrameCodec, FrameDecoder, FRAME_HEADER_SIZE, FRAME_MAGIC, FRAME_VERSION};
pub use coordinator::{Coordinator, CoordinatorConfig, ExecutionMode, ExecutionStatus, GraphState};
pub use discovery::{Discovery, DiscoveryConfig, DiscoveryEvent};
pub use error::{Error, Result};
pub use executor::{DistributedExecutor, ExecutorConfig};
pub use graph_proto::{
    CrossNodeRoute, CrossPartitionEdge, GraphPartition, GraphPartitioner,
    NodeType as GraphNodeType, NodeType, PartitionAssignment, PartitionError, PartitionStrategy,
    PortDirection, SerializableEdge, SerializableGraph, SerializableNode, SerializablePort,
};
pub use mdns::{MdnsDiscovery, MdnsDiscoveryConfig, CARET_SERVICE_TYPE};
pub use message::{Message, MessagePayload, MessageType};
pub use node::{LocalNode, NodeId, NodeInfo, NodeState};
pub use transport::{
    MemoryTransport, TcpTransport, TlsTransport, Transport, TransportConfig, TransportEvent,
};

/// Version of the distributed protocol
pub const PROTOCOL_VERSION: &str = "0.1.0";

/// Default port for distributed communication
pub const DEFAULT_PORT: u16 = 9234;

/// Maximum message size (64MB)
pub const MAX_MESSAGE_SIZE: usize = 64 * 1024 * 1024;

/// Default heartbeat interval in seconds
pub const DEFAULT_HEARTBEAT_INTERVAL_SECS: u64 = 5;

/// Default node timeout in seconds
pub const DEFAULT_NODE_TIMEOUT_SECS: u64 = 30;
