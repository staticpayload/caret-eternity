// Caret Distributed - mDNS-based discovery
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{NodeInfo, NodeId, Result};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// mDNS service type for Caret nodes
pub const CARET_SERVICE_TYPE: &str = "_caret._tcp.local.";

/// mDNS discovery configuration
#[derive(Clone, Debug)]
pub struct MdnsDiscoveryConfig {
    /// Service instance name (defaults to hostname)
    pub service_name: String,
    /// Service type
    pub service_type: String,
    /// Service port
    pub port: u16,
    /// Discovery interval (how often to browse)
    pub browse_interval: Duration,
    /// Announce interval (how often to re-announce)
    pub announce_interval: Duration,
    /// TXT record data
    pub txt_info: HashMap<String, String>,
}

impl Default for MdnsDiscoveryConfig {
    fn default() -> Self {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "caret-node".to_string());

        Self {
            service_name: hostname,
            service_type: CARET_SERVICE_TYPE.to_string(),
            port: crate::DEFAULT_PORT,
            browse_interval: Duration::from_secs(30),
            announce_interval: Duration::from_secs(60),
            txt_info: HashMap::new(),
        }
    }
}

impl MdnsDiscoveryConfig {
    /// Create with a custom service name
    pub fn with_service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = name.into();
        self
    }

    /// Set the service port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Add a TXT record entry
    pub fn with_txt(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.txt_info.insert(key.into(), value.into());
        self
    }
}

/// mDNS-based node discovery
pub struct MdnsDiscovery {
    config: MdnsDiscoveryConfig,
    local_node: NodeId,
    local_addr: SocketAddr,
    known_nodes: Arc<Mutex<HashMap<NodeId, NodeInfo>>>,
    event_tx: Arc<Mutex<mpsc::Sender<super::DiscoveryEvent>>>,
    running: Arc<Mutex<bool>>,
}

impl MdnsDiscovery {
    /// Create a new mDNS discovery service
    pub fn new(
        config: MdnsDiscoveryConfig,
        local_node: NodeId,
        local_addr: SocketAddr,
    ) -> Result<Self> {
        let (event_tx, _rx) = mpsc::channel(256);

        Ok(Self {
            config,
            local_node,
            local_addr,
            known_nodes: Arc::new(Mutex::new(HashMap::new())),
            event_tx: Arc::new(Mutex::new(event_tx)),
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// Start mDNS discovery (announce and browse)
    pub async fn start(&self) -> Result<()> {
        *self.running.lock() = true;

        // Announce our service
        self.announce_service()?;

        // Start browsing for other services
        let running = Arc::clone(&self.running);
        let known_nodes = Arc::clone(&self.known_nodes);
        let event_tx = Arc::clone(&self.event_tx);
        let local_node = self.local_node;
        let service_type = self.config.service_type.clone();
        let browse_interval = self.config.browse_interval;

        // Spawn a task to browse for services
        // Note: This is a simplified implementation that doesn't use actual mDNS
        // A full implementation would use the mdns-sd crate's browse API
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(browse_interval);

            while *running.lock() {
                interval.tick().await;

                // In a real implementation, we'd:
                // 1. Call mdns_sd::browse() to discover services
                // 2. Process ServiceDiscovery events
                // 3. Parse node info from TXT records
                // 4. Emit DiscoveryEvents

                debug!("mDNS browse tick for {}", service_type);
            }
        });

        info!("mDNS discovery started for service {}", self.config.service_name);

        Ok(())
    }

    /// Stop the discovery service
    pub fn stop(&self) {
        *self.running.lock() = false;
    }

    /// Announce our service via mDNS
    fn announce_service(&self) -> Result<()> {
        // In a real implementation, we'd:
        // 1. Create a ServiceDaemon
        // 2. Register our service with TXT records
        // 3. The daemon would broadcast mDNS announcements

        info!(
            "Would announce mDNS service: {}.{} at {} (node_id={})",
            self.config.service_name,
            self.config.service_type,
            self.local_addr,
            self.local_node
        );

        Ok(())
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &NodeId) -> Option<NodeInfo> {
        self.known_nodes.lock().get(id).cloned()
    }

    /// Get all known nodes
    pub fn all_nodes(&self) -> Vec<NodeInfo> {
        self.known_nodes.lock().values().cloned().collect()
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.known_nodes.lock().len()
    }

    /// Check if a node is known
    pub fn knows_node(&self, id: &NodeId) -> bool {
        self.known_nodes.lock().contains_key(id)
    }

    /// Subscribe to discovery events
    pub fn subscribe(&self) -> mpsc::Receiver<super::DiscoveryEvent> {
        let (tx, rx) = mpsc::channel(256);
        *self.event_tx.lock() = tx;
        rx
    }

    /// Manually add a discovered node
    pub fn add_discovered_node(&self, info: NodeInfo) {
        if info.id == self.local_node {
            return;
        }

        let is_new = {
            let mut nodes = self.known_nodes.lock();
            !nodes.contains_key(&info.id)
        };

        let event = if is_new {
            super::DiscoveryEvent::NodeDiscovered(info.clone())
        } else {
            super::DiscoveryEvent::NodeUpdated(info.clone())
        };

        self.known_nodes.lock().insert(info.id, info);
        let _ = self.event_tx.lock().try_send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mdns_config_default() {
        let config = MdnsDiscoveryConfig::default();
        assert_eq!(config.service_type, CARET_SERVICE_TYPE);
        assert_eq!(config.port, crate::DEFAULT_PORT);
    }

    #[test]
    fn test_mdns_config_builder() {
        let config = MdnsDiscoveryConfig::default()
            .with_service_name("test-node")
            .with_port(8080)
            .with_txt("version", "0.1.0")
            .with_txt("region", "us-west");

        assert_eq!(config.service_name, "test-node");
        assert_eq!(config.port, 8080);
        assert_eq!(config.txt_info.len(), 2);
        assert_eq!(config.txt_info.get("version"), Some(&"0.1.0".to_string()));
    }

    #[test]
    fn test_service_type() {
        assert_eq!(CARET_SERVICE_TYPE, "_caret._tcp.local.");
        assert!(CARET_SERVICE_TYPE.ends_with(".local."));
    }

    #[tokio::test]
    async fn test_mdns_discovery_lifecycle() {
        let config = MdnsDiscoveryConfig::default();
        let local_id = NodeId::new_v4();
        let local_addr: SocketAddr = "127.0.0.1:9234".parse().unwrap();

        let discovery = MdnsDiscovery::new(config, local_id, local_addr).unwrap();

        assert!(!discovery.knows_node(&local_id));
        assert_eq!(discovery.node_count(), 0);

        // Start discovery
        discovery.start().await.unwrap();

        // Add a discovered node
        let remote_id = NodeId::new_v4();
        let remote_info = NodeInfo::new(
            remote_id,
            "remote".into(),
            "127.0.0.1:9235".parse().unwrap(),
            4,
            1024,
        );

        discovery.add_discovered_node(remote_info.clone());

        assert!(discovery.knows_node(&remote_id));
        assert_eq!(discovery.node_count(), 1);

        // Stop discovery
        discovery.stop();
    }
}
