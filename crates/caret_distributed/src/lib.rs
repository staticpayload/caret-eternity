// Caret Distributed - Distributed execution support for Caret
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

//! Distributed execution support for Caret pipelines.
//!
//! This crate provides:
//! - Network transport layer with TCP support
//! - Node discovery and cluster management
//! - Distributed graph execution coordination
//! - Protocol messages for cluster communication

#![warn(missing_docs)]
#![warn(clippy::all)]

mod codec;
mod transport;
mod node;
mod discovery;
mod coordinator;
mod message;
mod error;

pub use codec::{FrameCodec, FrameDecoder, FRAME_HEADER_SIZE, FRAME_MAGIC, FRAME_VERSION};
pub use error::{Error, Result};
pub use transport::{Transport, TransportConfig, TransportEvent};
pub use node::{NodeId, NodeInfo, NodeState, LocalNode};
pub use discovery::{Discovery, DiscoveryConfig, DiscoveryEvent};
pub use coordinator::{Coordinator, CoordinatorConfig, ExecutionMode};
pub use message::{Message, MessagePayload, MessageType};

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
