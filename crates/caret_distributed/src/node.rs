// Caret Distributed - Node types
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Instant;

/// Unique node identifier
pub type NodeId = uuid::Uuid;

/// Node state in the cluster
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    /// Node is joining
    Joining,
    /// Node is active and ready
    Active,
    /// Node is draining (shutting down gracefully)
    Draining,
    /// Node is offline
    Offline,
    /// Node has failed
    Failed,
}

impl NodeState {
    /// Check if the node is available for work
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Active | Self::Joining)
    }

    /// Check if the node is alive
    pub fn is_alive(&self) -> bool {
        matches!(self, Self::Joining | Self::Active | Self::Draining)
    }
}

/// Node information
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Unique node ID
    pub id: NodeId,
    /// Node hostname or identifier
    pub hostname: String,
    /// Listen address
    pub addr: SocketAddr,
    /// Current state
    pub state: NodeState,
    /// Node capabilities
    pub capabilities: Vec<String>,
    /// Number of CPU cores
    pub cores: usize,
    /// Available memory in bytes
    pub memory_bytes: u64,
    /// Current load (0.0 to 1.0)
    pub load: f64,
    /// Number of assigned graphs
    pub graph_count: usize,
    /// Protocol version
    pub protocol_version: String,
}

impl NodeInfo {
    /// Create new node info
    pub fn new(
        id: NodeId,
        hostname: String,
        addr: SocketAddr,
        cores: usize,
        memory_bytes: u64,
    ) -> Self {
        Self {
            id,
            hostname,
            addr,
            state: NodeState::Joining,
            capabilities: Vec::new(),
            cores,
            memory_bytes,
            load: 0.0,
            graph_count: 0,
            protocol_version: crate::PROTOCOL_VERSION.to_string(),
        }
    }

    /// Create node info for local system
    pub fn local(addr: SocketAddr) -> Result<Self> {
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "localhost".to_string());

        let cores = num_cpus::get();
        let memory_bytes = system_memory_bytes();

        Ok(Self::new(
            NodeId::new_v4(),
            hostname,
            addr,
            cores,
            memory_bytes,
        ))
    }

    /// Add a capability
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Update load
    pub fn with_load(mut self, load: f64) -> Self {
        self.load = load.clamp(0.0, 1.0);
        self
    }

    /// Check if the node has a specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    /// Check protocol version compatibility
    pub fn is_compatible(&self) -> bool {
        self.protocol_version == crate::PROTOCOL_VERSION
    }
}

/// Get system memory in bytes
fn system_memory_bytes() -> u64 {
    // For now, use a default value
    // TODO: Implement platform-specific memory detection
    // Linux: /proc/meminfo or sysconf
    // macOS: sysctl hw.memsize
    // Windows: GlobalMemoryStatusEx
    8 * 1024 * 1024 * 1024 // 8GB default
}

/// Local node instance
pub struct LocalNode {
    /// Node information
    info: NodeInfo,
    /// Last heartbeat time
    last_heartbeat: Instant,
    /// Peer nodes
    peers: HashMap<NodeId, PeerInfo>,
    /// Started timestamp
    started_at: Instant,
}

/// Information about a peer node
#[derive(Debug, Clone)]
struct PeerInfo {
    /// Node information
    info: NodeInfo,
    /// Last seen
    last_seen: Instant,
}

impl LocalNode {
    /// Create a new local node
    pub fn new(info: NodeInfo) -> Self {
        Self {
            info,
            last_heartbeat: Instant::now(),
            peers: HashMap::new(),
            started_at: Instant::now(),
        }
    }

    /// Create local node with system info
    pub fn bind(addr: SocketAddr) -> Result<Self> {
        Ok(Self::new(NodeInfo::local(addr)?))
    }

    /// Get the node ID
    pub fn id(&self) -> NodeId {
        self.info.id
    }

    /// Get node information
    pub fn info(&self) -> &NodeInfo {
        &self.info
    }

    /// Update node information
    pub fn update_info(&mut self, info: NodeInfo) {
        self.info = info;
    }

    /// Record a heartbeat
    pub fn record_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
    }

    /// Get time since last heartbeat
    pub fn time_since_heartbeat(&self) -> std::time::Duration {
        self.last_heartbeat.elapsed()
    }

    /// Add or update a peer
    pub fn add_peer(&mut self, info: NodeInfo) {
        let peer = PeerInfo {
            info: info.clone(),
            last_seen: Instant::now(),
        };
        self.peers.insert(info.id, peer);
    }

    /// Remove a peer
    pub fn remove_peer(&mut self, id: &NodeId) -> Option<NodeInfo> {
        self.peers.remove(id).map(|p| p.info)
    }

    /// Get a peer by ID
    pub fn peer(&self, id: &NodeId) -> Option<&NodeInfo> {
        self.peers.get(id).map(|p| &p.info)
    }

    /// Get all peers
    pub fn peers(&self) -> impl Iterator<Item = &NodeInfo> {
        self.peers.values().map(|p| &p.info)
    }

    /// Get peer count
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Clean up stale peers
    pub fn cleanup_stale_peers(&mut self, timeout: std::time::Duration) -> usize {
        let now = Instant::now();
        let stale_peers: Vec<NodeId> = self
            .peers
            .iter()
            .filter(|(_, p)| now.duration_since(p.last_seen) > timeout)
            .map(|(id, _)| *id)
            .collect();

        for id in &stale_peers {
            self.peers.remove(id);
        }

        stale_peers.len()
    }

    /// Get uptime
    pub fn uptime(&self) -> std::time::Duration {
        self.started_at.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_state_available() {
        assert!(NodeState::Active.is_available());
        assert!(NodeState::Joining.is_available());
        assert!(!NodeState::Draining.is_available());
        assert!(!NodeState::Offline.is_available());
        assert!(!NodeState::Failed.is_available());
    }

    #[test]
    fn test_node_state_alive() {
        assert!(NodeState::Active.is_alive());
        assert!(NodeState::Joining.is_alive());
        assert!(NodeState::Draining.is_alive());
        assert!(!NodeState::Offline.is_alive());
        assert!(!NodeState::Failed.is_alive());
    }

    #[test]
    fn test_node_info_builder() {
        let id = NodeId::new_v4();
        let addr = "127.0.0.1:9234".parse().unwrap();
        let info = NodeInfo::new(id.clone(), "test".into(), addr, 4, 1024)
            .with_capability("video")
            .with_capability("audio")
            .with_load(0.5);

        assert_eq!(info.id, id);
        assert_eq!(info.cores, 4);
        assert!(info.has_capability("video"));
        assert!(info.has_capability("audio"));
        assert!(!info.has_capability("unknown"));
        assert_eq!(info.load, 0.5);
    }

    #[test]
    fn test_local_node_peers() {
        let addr = "127.0.0.1:9234".parse().unwrap();
        let mut node = LocalNode::new(NodeInfo::new(
            NodeId::new_v4(),
            "local".into(),
            addr,
            4,
            1024,
        ));

        let peer1 = NodeInfo::new(
            NodeId::new_v4(),
            "peer1".into(),
            "127.0.0.1:9235".parse().unwrap(),
            2,
            512,
        );
        let peer1_id = peer1.id;

        node.add_peer(peer1);

        assert_eq!(node.peer_count(), 1);
        assert!(node.peer(&peer1_id).is_some());
    }
}
