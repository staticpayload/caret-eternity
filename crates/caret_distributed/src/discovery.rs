// Caret Distributed - Node discovery
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{NodeInfo, NodeId, Result};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Discovery configuration
#[derive(Clone, Debug)]
pub struct DiscoveryConfig {
    /// Discovery mode
    pub mode: DiscoveryMode,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Node timeout before considering failed
    pub node_timeout: Duration,
    /// Known seed addresses for static discovery
    pub seed_addresses: Vec<SocketAddr>,
    /// Multicast address for mDNS discovery
    pub multicast_addr: Option<String>,
    /// Multicast port
    pub multicast_port: u16,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            mode: DiscoveryMode::Static,
            heartbeat_interval: Duration::from_secs(crate::DEFAULT_HEARTBEAT_INTERVAL_SECS),
            node_timeout: Duration::from_secs(crate::DEFAULT_NODE_TIMEOUT_SECS),
            seed_addresses: Vec::new(),
            multicast_addr: None,
            multicast_port: 9235,
        }
    }
}

impl DiscoveryConfig {
    /// Create a config with the given discovery mode
    pub fn with_mode(mode: DiscoveryMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// Set the heartbeat interval
    pub fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    /// Set the node timeout
    pub fn with_node_timeout(mut self, timeout: Duration) -> Self {
        self.node_timeout = timeout;
        self
    }

    /// Add a seed address
    pub fn with_seed(mut self, addr: SocketAddr) -> Self {
        self.seed_addresses.push(addr);
        self
    }

    /// Set multicast configuration
    pub fn with_multicast(mut self, addr: impl Into<String>, port: u16) -> Self {
        self.mode = DiscoveryMode::Multicast;
        self.multicast_addr = Some(addr.into());
        self.multicast_port = port;
        self
    }
}

/// Discovery mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscoveryMode {
    /// Static configuration only
    Static,
    /// Multicast discovery
    Multicast,
    /// Custom discovery service
    Custom,
}

/// Discovery event
#[derive(Clone, Debug)]
pub enum DiscoveryEvent {
    /// New node discovered
    NodeDiscovered(NodeInfo),
    /// Node updated
    NodeUpdated(NodeInfo),
    /// Node left (graceful shutdown)
    NodeLeft(NodeId),
    /// Node failed (timed out)
    NodeFailed(NodeId),
}

/// Node discovery service
pub struct Discovery {
    config: DiscoveryConfig,
    local_node: NodeId,
    known_nodes: Arc<Mutex<HashMap<NodeId, NodeInfo>>>,
    event_tx: Arc<Mutex<mpsc::Sender<DiscoveryEvent>>>,
    running: Arc<Mutex<bool>>,
}

impl Discovery {
    /// Create a new discovery service
    pub fn new(config: DiscoveryConfig, local_node: NodeId) -> Self {
        let (event_tx, _rx) = mpsc::channel(256);

        Self {
            config,
            local_node,
            known_nodes: Arc::new(Mutex::new(HashMap::new())),
            event_tx: Arc::new(Mutex::new(event_tx)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the discovery service
    pub async fn start(&self) -> Result<()> {
        *self.running.lock() = true;

        // Start based on mode
        match self.config.mode {
            DiscoveryMode::Static => {
                // Seed nodes are added via add_seed()
            }
            DiscoveryMode::Multicast => {
                // TODO: Start multicast listener
            }
            DiscoveryMode::Custom => {
                // TODO: Connect to custom discovery service
            }
        }

        Ok(())
    }

    /// Stop the discovery service
    pub fn stop(&self) {
        *self.running.lock() = false;
    }

    /// Add a seed node
    pub fn add_seed(&self, info: NodeInfo) {
        if info.id == self.local_node {
            return;
        }

        let is_new = {
            let mut nodes = self.known_nodes.lock();
            let is_new = !nodes.contains_key(&info.id);
            nodes.insert(info.id, info.clone());
            is_new
        };

        let event = if is_new {
            DiscoveryEvent::NodeDiscovered(info)
        } else {
            DiscoveryEvent::NodeUpdated(info)
        };

        let _ = self.event_tx.lock().try_send(event);
    }

    /// Update a node (heartbeat)
    pub fn update_node(&self, info: NodeInfo) {
        if info.id == self.local_node {
            return;
        }

        let existed = {
            let mut nodes = self.known_nodes.lock();
            nodes.insert(info.id, info.clone()).is_some()
        };

        let event = if existed {
            DiscoveryEvent::NodeUpdated(info)
        } else {
            DiscoveryEvent::NodeDiscovered(info)
        };

        let _ = self.event_tx.lock().try_send(event);
    }

    /// Remove a node
    pub fn remove_node(&self, id: NodeId) {
        let existed = {
            let mut nodes = self.known_nodes.lock();
            nodes.remove(&id).is_some()
        };

        if existed {
            let _ = self
                .event_tx
                .lock()
                .try_send(DiscoveryEvent::NodeLeft(id));
        }
    }

    /// Mark a node as failed
    pub fn mark_failed(&self, id: NodeId) {
        let existed = {
            let mut nodes = self.known_nodes.lock();
            nodes.remove(&id).is_some()
        };

        if existed {
            let _ = self
                .event_tx
                .lock()
                .try_send(DiscoveryEvent::NodeFailed(id));
        }
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &NodeId) -> Option<NodeInfo> {
        self.known_nodes.lock().get(id).cloned()
    }

    /// Get all known nodes
    pub fn all_nodes(&self) -> Vec<NodeInfo> {
        self.known_nodes.lock().values().cloned().collect()
    }

    /// Get active nodes (not failed or offline)
    pub fn active_nodes(&self) -> Vec<NodeInfo> {
        self.known_nodes
            .lock()
            .values()
            .filter(|n| n.state.is_alive())
            .cloned()
            .collect()
    }

    /// Get available nodes for work
    pub fn available_nodes(&self) -> Vec<NodeInfo> {
        self.known_nodes
            .lock()
            .values()
            .filter(|n| n.state.is_available())
            .cloned()
            .collect()
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.known_nodes.lock().len()
    }

    /// Subscribe to discovery events
    pub fn subscribe(&self) -> mpsc::Receiver<DiscoveryEvent> {
        let (tx, rx) = mpsc::channel(256);
        *self.event_tx.lock() = tx;
        rx
    }

    /// Check if a node is known
    pub fn knows_node(&self, id: &NodeId) -> bool {
        self.known_nodes.lock().contains_key(id)
    }

    /// Cleanup stale nodes
    pub fn cleanup_stale(&self) -> usize {
        let mut to_remove = Vec::new();
        let nodes = self.known_nodes.lock();

        for (id, info) in nodes.iter() {
            if !info.state.is_alive() {
                to_remove.push(*id);
            }
        }
        drop(nodes);

        for id in &to_remove {
            self.mark_failed(*id);
        }

        to_remove.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert_eq!(config.mode, DiscoveryMode::Static);
        assert_eq!(
            config.heartbeat_interval,
            Duration::from_secs(crate::DEFAULT_HEARTBEAT_INTERVAL_SECS)
        );
    }

    #[test]
    fn test_discovery_config_builder() {
        let addr: SocketAddr = "127.0.0.1:9234".parse().unwrap();
        let config = DiscoveryConfig::with_mode(DiscoveryMode::Static)
            .with_seed(addr)
            .with_heartbeat_interval(Duration::from_secs(10));

        assert_eq!(config.mode, DiscoveryMode::Static);
        assert_eq!(config.seed_addresses.len(), 1);
        assert_eq!(config.heartbeat_interval, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_discovery_add_seed() {
        let config = DiscoveryConfig::default();
        let local_id = NodeId::new_v4();
        let discovery = Discovery::new(config, local_id);

        let seed_id = NodeId::new_v4();
        let seed = NodeInfo::new(
            seed_id,
            "seed".into(),
            "127.0.0.1:9234".parse().unwrap(),
            4,
            1024,
        );

        discovery.add_seed(seed.clone());

        assert!(discovery.knows_node(&seed_id));
        assert_eq!(discovery.node_count(), 1);
        assert_eq!(discovery.get_node(&seed_id).unwrap().id, seed_id);
    }
}
