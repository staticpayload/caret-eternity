// Caret Distributed - mDNS-based discovery
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{NodeId, NodeInfo, Result};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// mDNS service type for Caret nodes
pub const CARET_SERVICE_TYPE: &str = "_caret._tcp.local.";

/// TXT record keys for Caret node information
pub mod txt_keys {
    pub const NODE_ID: &str = "node_id";
    pub const VERSION: &str = "version";
    pub const CAPABILITIES: &str = "capabilities";
}

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

        let mut txt_info = HashMap::new();
        let version = option_env!("CARGO_PKG_VERSION").unwrap_or("0.1.0");
        txt_info.insert(txt_keys::VERSION.to_string(), version.to_string());

        Self {
            service_name: hostname,
            service_type: CARET_SERVICE_TYPE.to_string(),
            port: crate::DEFAULT_PORT,
            browse_interval: Duration::from_secs(30),
            announce_interval: Duration::from_secs(60),
            txt_info,
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
    daemon: Arc<Mutex<Option<ServiceDaemon>>>,
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
            daemon: Arc::new(Mutex::new(None)),
        })
    }

    /// Start mDNS discovery (announce and browse)
    pub async fn start(&self) -> Result<()> {
        *self.running.lock() = true;

        // Create the mDNS daemon
        let daemon = ServiceDaemon::new().map_err(|e| {
            crate::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create mDNS daemon: {}", e),
            ))
        })?;

        *self.daemon.lock() = Some(daemon);

        // Announce our service
        self.announce_service()?;

        // Start browsing for other services
        let running = Arc::clone(&self.running);
        let known_nodes = Arc::clone(&self.known_nodes);
        let event_tx = Arc::clone(&self.event_tx);
        let local_node = self.local_node;
        let service_type = self.config.service_type.clone();
        let daemon_ref = Arc::clone(&self.daemon);

        // Spawn a task to browse for services
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));

            while *running.lock() {
                interval.tick().await;

                // Try to get the daemon and browse
                let daemon = {
                    let guard = daemon_ref.lock();
                    guard.clone()
                };

                if let Some(daemon) = daemon {
                    // Browse for services
                    match daemon.browse(&service_type) {
                        Ok(receiver) => {
                            debug!("Started browsing for {}", service_type);

                            // Process browse events
                            while *running.lock() {
                                match receiver.recv_async().await {
                                    Ok(event) => {
                                        Self::handle_service_event(
                                            event,
                                            &known_nodes,
                                            &event_tx,
                                            local_node,
                                        );
                                    }
                                    Err(e) => {
                                        warn!("mDNS browse error: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to browse mDNS services: {}", e);
                        }
                    }
                }
            }
        });

        info!(
            "mDNS discovery started: service {}._{} at {}",
            self.config.service_name,
            self.config.service_type.trim_end_matches(".local."),
            self.local_addr
        );

        Ok(())
    }

    /// Stop the discovery service
    pub fn stop(&self) {
        *self.running.lock() = false;

        // Unregister our service
        if let Some(daemon) = self.daemon.lock().as_ref() {
            // Use shutdown instead of stop_broadcast
            if let Err(e) = daemon.shutdown() {
                warn!("Failed to shutdown mDNS daemon: {}", e);
            }
        }
    }

    /// Announce our service via mDNS
    fn announce_service(&self) -> Result<()> {
        let daemon = self.daemon.lock();
        let daemon = daemon
            .as_ref()
            .ok_or_else(|| crate::Error::Transport("mDNS daemon not initialized".to_string()))?;

        // Create TXT records with node information as HashMap
        let mut txt_props = self.config.txt_info.clone();

        // Add our node ID to TXT records
        txt_props.insert(txt_keys::NODE_ID.to_string(), self.local_node.to_string());

        // Get the IP address from local_addr
        let ip = self.local_addr.ip();

        // Create the service info - mdns-sd requires:
        // ty_domain, my_name, host_name, ip, port, properties
        let service_info = ServiceInfo::new(
            &self.config.service_type,
            &self.config.service_name,
            &self.config.service_name,
            ip,
            self.local_addr.port(),
            txt_props,
        );

        let service_info = match service_info {
            Ok(info) => info,
            Err(e) => {
                return Err(crate::Error::Transport(format!(
                    "Failed to create service info: {}",
                    e
                )))
            }
        };

        // Broadcast the service
        let full_name = service_info.get_fullname().to_string();

        daemon.register(service_info).map_err(|e| {
            crate::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to broadcast mDNS service: {}", e),
            ))
        })?;

        info!(
            "Announced mDNS service: {} (node_id={}, addr={})",
            full_name, self.local_node, self.local_addr
        );

        Ok(())
    }

    /// Handle a service discovery event
    fn handle_service_event(
        event: ServiceEvent,
        known_nodes: &Arc<Mutex<HashMap<NodeId, NodeInfo>>>,
        event_tx: &Arc<Mutex<mpsc::Sender<super::DiscoveryEvent>>>,
        local_node: NodeId,
    ) {
        match event {
            ServiceEvent::SearchStarted(_) => {
                debug!("mDNS search started");
            }
            ServiceEvent::ServiceFound(_, fullname) => {
                debug!("mDNS service found: {}", fullname);
                // ServiceFound gives (ty, fullname), we need to wait for ServiceResolved
            }
            ServiceEvent::ServiceResolved(info) => {
                debug!("mDNS service resolved: {}", info.get_fullname());
                Self::process_service_info(info, known_nodes, event_tx, local_node);
            }
            ServiceEvent::ServiceRemoved(_, fullname) => {
                debug!("mDNS service removed: {}", fullname);
                Self::remove_service(fullname, known_nodes, event_tx);
            }
            _ => {}
        }
    }

    /// Process service information and emit discovery events
    fn process_service_info(
        info: ServiceInfo,
        known_nodes: &Arc<Mutex<HashMap<NodeId, NodeInfo>>>,
        event_tx: &Arc<Mutex<mpsc::Sender<super::DiscoveryEvent>>>,
        local_node: NodeId,
    ) {
        // Try to extract node ID from TXT records
        let node_id = Self::get_node_id_from_info(&info);

        // Skip our own node
        if node_id == local_node {
            return;
        }

        // Get addresses from the service info
        let addrs = info.get_addresses();
        if addrs.is_empty() {
            warn!("Service {} has no addresses", info.get_fullname());
            return;
        }

        // Use the first address
        let ip = *addrs.iter().next().unwrap();
        let port = info.get_port();

        // Use the hostname from service info
        let hostname = info.get_hostname().to_string();

        let addr = SocketAddr::new(ip, port);

        // Create node info
        let node_info = NodeInfo::new(node_id, hostname, addr, 1, 1024);

        // Check if this is a new node or an update
        let is_new = {
            let nodes = known_nodes.lock();
            !nodes.contains_key(&node_id)
        };

        let event = if is_new {
            super::DiscoveryEvent::NodeDiscovered(node_info.clone())
        } else {
            super::DiscoveryEvent::NodeUpdated(node_info.clone())
        };

        known_nodes.lock().insert(node_id, node_info.clone());
        let _ = event_tx.lock().try_send(event);

        info!(
            "mDNS {} node: {} at {}",
            if is_new { "discovered" } else { "updated" },
            node_id,
            addr
        );
    }

    /// Extract node ID from service info
    fn get_node_id_from_info(info: &ServiceInfo) -> NodeId {
        // Try to get node_id from TXT records using get_property_val_str
        if let Some(node_id_str) = info.get_property_val_str(txt_keys::NODE_ID) {
            if let Ok(id) = NodeId::from_str(node_id_str) {
                return id;
            }
        }

        // No node ID in TXT records, generate from service name
        let hash = md5::compute(info.get_fullname().as_bytes());
        NodeId::from_bytes(hash.0)
    }

    /// Remove a service that's no longer available
    fn remove_service(
        fullname: String,
        known_nodes: &Arc<Mutex<HashMap<NodeId, NodeInfo>>>,
        event_tx: &Arc<Mutex<mpsc::Sender<super::DiscoveryEvent>>>,
    ) {
        // Try to find and remove by generating ID from name
        let hash = md5::compute(fullname.as_bytes());
        let node_id = NodeId::from_bytes(hash.0);

        if let Some(info) = known_nodes.lock().remove(&node_id) {
            let _ = event_tx
                .lock()
                .try_send(super::DiscoveryEvent::NodeLeft(info.id));
            info!("mDNS node left: {}", node_id);
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

    /// Manually add a discovered node (for testing)
    pub fn add_discovered_node(&self, info: NodeInfo) {
        if info.id == self.local_node {
            return;
        }

        let is_new = {
            let nodes = self.known_nodes.lock();
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
        assert!(config.txt_info.contains_key(txt_keys::VERSION));
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
        // Has "version" (from default, overwritten) and "region" (added)
        assert_eq!(config.txt_info.len(), 2);
        assert_eq!(config.txt_info.get("version"), Some(&"0.1.0".to_string()));
        assert_eq!(config.txt_info.get("region"), Some(&"us-west".to_string()));
    }

    #[test]
    fn test_service_type() {
        assert_eq!(CARET_SERVICE_TYPE, "_caret._tcp.local.");
        assert!(CARET_SERVICE_TYPE.ends_with(".local."));
    }

    #[test]
    fn test_txt_keys() {
        assert_eq!(txt_keys::NODE_ID, "node_id");
        assert_eq!(txt_keys::VERSION, "version");
        assert_eq!(txt_keys::CAPABILITIES, "capabilities");
    }

    #[tokio::test]
    async fn test_mdns_discovery_lifecycle() {
        let config = MdnsDiscoveryConfig::default();
        let local_id = NodeId::new_v4();
        let local_addr: SocketAddr = "127.0.0.1:9234".parse().unwrap();

        let discovery = MdnsDiscovery::new(config, local_id, local_addr).unwrap();

        assert!(!discovery.knows_node(&local_id));
        assert_eq!(discovery.node_count(), 0);

        // Add a discovered node manually (simulating mDNS discovery)
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

        // Verify we can retrieve the node
        let retrieved = discovery.get_node(&remote_id);
        assert_eq!(retrieved.as_ref().map(|n| n.id), Some(remote_id));
    }

    #[tokio::test]
    async fn test_mdns_discovery_duplicate() {
        let config = MdnsDiscoveryConfig::default();
        let local_id = NodeId::new_v4();
        let local_addr: SocketAddr = "127.0.0.1:9234".parse().unwrap();

        let discovery = MdnsDiscovery::new(config, local_id, local_addr).unwrap();

        // Add the same node twice
        let remote_id = NodeId::new_v4();
        let remote_info = NodeInfo::new(
            remote_id,
            "remote".into(),
            "127.0.0.1:9235".parse().unwrap(),
            4,
            1024,
        );

        discovery.add_discovered_node(remote_info.clone());
        assert_eq!(discovery.node_count(), 1);

        discovery.add_discovered_node(remote_info.clone());
        assert_eq!(discovery.node_count(), 1);
    }
}
