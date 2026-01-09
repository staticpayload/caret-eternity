// Caret Distributed - Protocol messages
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::NodeId;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

/// Unique message identifier
pub type MessageId = uuid::Uuid;

/// Distributed protocol message
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID
    pub id: MessageId,
    /// Source node ID
    pub from: NodeId,
    /// Destination node ID (empty for broadcast)
    pub to: Option<NodeId>,
    /// Message type
    #[serde(flatten)]
    pub payload: MessagePayload,
    /// Timestamp (Unix millis)
    pub timestamp: u64,
}

impl Message {
    /// Create a new message
    pub fn new(from: NodeId, to: Option<NodeId>, payload: MessagePayload) -> Self {
        Self {
            id: MessageId::new_v4(),
            from,
            to,
            payload,
            timestamp: current_timestamp_millis(),
        }
    }

    /// Get the message type
    pub fn message_type(&self) -> MessageType {
        match &self.payload {
            MessagePayload::Hello { .. } => MessageType::Hello,
            MessagePayload::HelloReply { .. } => MessageType::HelloReply,
            MessagePayload::Heartbeat => MessageType::Heartbeat,
            MessagePayload::HeartbeatReply => MessageType::HeartbeatReply,
            MessagePayload::GraphSubmit { .. } => MessageType::GraphSubmit,
            MessagePayload::GraphUpdate { .. } => MessageType::GraphUpdate,
            MessagePayload::GraphStart { .. } => MessageType::GraphStart,
            MessagePayload::GraphStop { .. } => MessageType::GraphStop,
            MessagePayload::NodeAssign { .. } => MessageType::NodeAssign,
            MessagePayload::NodeRelease { .. } => MessageType::NodeRelease,
            MessagePayload::PartitionAssign { .. } => MessageType::PartitionAssign,
            MessagePayload::Packet { .. } => MessageType::Packet,
            MessagePayload::StatusRequest => MessageType::StatusRequest,
            MessagePayload::StatusReply { .. } => MessageType::StatusReply,
            MessagePayload::Error { .. } => MessageType::Error,
        }
    }

    /// Check if this message is a broadcast
    pub fn is_broadcast(&self) -> bool {
        self.to.is_none()
    }

    /// Check if this message is addressed to the given node
    pub fn is_addressed_to(&self, node_id: &NodeId) -> bool {
        self.to.as_ref().map_or(false, |id| id == node_id)
    }

    /// Check if message is from the given node
    pub fn is_from(&self, node_id: &NodeId) -> bool {
        self.from == *node_id
    }
}

/// Message type discriminator
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    /// Hello handshake
    Hello,
    /// Hello reply
    HelloReply,
    /// Heartbeat
    Heartbeat,
    /// Heartbeat reply
    HeartbeatReply,
    /// Graph submission
    GraphSubmit,
    /// Graph update
    GraphUpdate,
    /// Start execution
    GraphStart,
    /// Stop execution
    GraphStop,
    /// Assign node to worker
    NodeAssign,
    /// Release node from worker
    NodeRelease,
    /// Assign partition to worker
    PartitionAssign,
    /// Data packet
    Packet,
    /// Status request
    StatusRequest,
    /// Status reply
    StatusReply,
    /// Error
    Error,
}

/// Message payload variants
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessagePayload {
    /// Initial handshake
    Hello {
        /// Protocol version
        version: String,
        /// Node information
        node_info: crate::NodeInfo,
    },

    /// Reply to hello
    HelloReply {
        /// Protocol version
        version: String,
        /// Node information
        node_info: crate::NodeInfo,
        /// Whether to accept the connection
        accepted: bool,
        /// Reason for rejection (if any)
        reject_reason: Option<String>,
    },

    /// Heartbeat to keep connection alive
    Heartbeat,

    /// Reply to heartbeat
    HeartbeatReply,

    /// Submit a graph for execution
    GraphSubmit {
        /// Graph ID
        graph_id: String,
        /// Serialized graph definition
        graph: Vec<u8>,
        /// Execution mode
        mode: crate::ExecutionMode,
    },

    /// Update to an existing graph
    GraphUpdate {
        /// Graph ID
        graph_id: String,
        /// Serialized graph changes
        changes: Vec<u8>,
    },

    /// Start graph execution
    GraphStart {
        /// Graph ID
        graph_id: String,
    },

    /// Stop graph execution
    GraphStop {
        /// Graph ID
        graph_id: String,
        /// Whether to drain remaining data
        drain: bool,
    },

    /// Assign a node to a worker
    NodeAssign {
        /// Graph ID
        graph_id: String,
        /// Node ID to assign
        node_id: u64,
        /// Node configuration
        config: Vec<u8>,
    },

    /// Release a node from a worker
    NodeRelease {
        /// Graph ID
        graph_id: String,
        /// Node ID to release
        node_id: u64,
    },

    /// Assign a graph partition to a worker
    PartitionAssign {
        /// Graph ID
        graph_id: String,
        /// Graph definition (serialized)
        graph: Vec<u8>,
        /// Partition assignment (serialized)
        partition: Vec<u8>,
        /// Partition routes (serialized)
        routes: Vec<u8>,
    },

    /// Data packet between nodes
    Packet {
        /// Source port
        from_port: String,
        /// Target port
        to_port: String,
        /// Packet data
        data: Vec<u8>,
    },

    /// Request status from a node
    StatusRequest,

    /// Status reply
    StatusReply {
        /// Current state
        state: crate::NodeState,
        /// Metrics
        metrics: serde_json::Value,
    },

    /// Error message
    Error {
        /// Error code
        code: String,
        /// Error message
        message: String,
    },
}

/// Get current timestamp in milliseconds
fn current_timestamp_millis() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let from = NodeId::new_v4();
        let to = NodeId::new_v4();
        let msg = Message::new(from, Some(to), MessagePayload::Heartbeat);

        assert_eq!(msg.from, from);
        assert_eq!(msg.to, Some(to));
        assert_eq!(msg.message_type(), MessageType::Heartbeat);
    }

    #[test]
    fn test_broadcast_detection() {
        let from = NodeId::new_v4();
        let broadcast = Message::new(from, None, MessagePayload::Heartbeat);
        let direct = Message::new(from, Some(NodeId::new_v4()), MessagePayload::Heartbeat);

        assert!(broadcast.is_broadcast());
        assert!(!direct.is_broadcast());
    }

    #[test]
    fn test_message_serialization() {
        let from = NodeId::new_v4();
        let msg = Message::new(from, None, MessagePayload::Heartbeat);

        let json = serde_json::to_string(&msg).unwrap();
        let decoded: Message = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.id, msg.id);
        assert_eq!(decoded.from, from);
    }
}
