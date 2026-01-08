// Caret Plugins - Plugin type definitions
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::Plugin;
use caret_core::{Packet, Result};
use caret_sched::{NodeProcessor, ProcessingContext, ProcessingResult};

/// The type of a plugin
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginType {
    /// Node plugin - provides custom processing nodes
    Node,
    /// Codec plugin - provides packet serialization/deserialization
    Codec,
    /// IO plugin - provides source/sink functionality
    Io,
    /// Metric exporter plugin - exports metrics to external systems
    MetricExporter,
    /// UI panel plugin - adds panels to the inspector UI
    UiPanel,
}

impl PluginType {
    /// Get the string name of the plugin type
    pub fn as_str(&self) -> &str {
        match self {
            PluginType::Node => "node",
            PluginType::Codec => "codec",
            PluginType::Io => "io",
            PluginType::MetricExporter => "metric_exporter",
            PluginType::UiPanel => "ui_panel",
        }
    }

    /// Parse a plugin type from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "node" => Some(PluginType::Node),
            "codec" => Some(PluginType::Codec),
            "io" => Some(PluginType::Io),
            "metric_exporter" => Some(PluginType::MetricExporter),
            "ui_panel" => Some(PluginType::UiPanel),
            _ => None,
        }
    }
}

/// Metadata about a plugin
///
/// This is returned by the Plugin::metadata() method.
#[derive(Clone, Debug)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,

    /// Plugin version (semver)
    pub version: String,

    /// Plugin description
    pub description: String,

    /// Plugin author
    pub author: String,

    /// Plugin type
    pub plugin_type: PluginType,

    /// Caret API version this plugin targets
    pub api_version: (u64, u64, u64),

    /// List of capabilities provided by this plugin
    pub capabilities: Vec<String>,
}

impl PluginMetadata {
    /// Create new plugin metadata
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
        plugin_type: PluginType,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: description.into(),
            author: author.into(),
            plugin_type,
            api_version: (0, 1, 0),
            capabilities: Vec::new(),
        }
    }

    /// Set the API version
    pub fn with_api_version(mut self, version: (u64, u64, u64)) -> Self {
        self.api_version = version;
        self
    }

    /// Add a capability
    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Check if the plugin has a specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }
}

/// A node plugin
///
/// Node plugins provide custom processing nodes that can be
/// used in pipelines.
pub trait NodePlugin: Plugin {
    /// Create a new node processor instance
    ///
    /// The `config` parameter contains any configuration for the node
    /// as key-value pairs.
    fn create_node(&self, config: &[(String, String)]) -> Result<Box<dyn NodeProcessor>>;

    /// Get the list of node types this plugin provides
    fn node_types(&self) -> &[String];
}

/// A codec plugin
///
/// Codec plugins provide serialization/deserialization for packets.
pub trait CodecPlugin: Plugin {
    /// Get the list of codec names this plugin provides
    fn codec_names(&self) -> &[String];

    /// Encode a packet
    fn encode(&self, packet: &Packet, codec_name: &str) -> Result<Vec<u8>>;

    /// Decode a packet
    fn decode(&self, data: &[u8], codec_name: &str) -> Result<Packet>;
}

/// An IO plugin
///
/// IO plugins provide sources and sinks for external data.
pub trait IoPlugin: Plugin {
    /// Get the list of source types this plugin provides
    fn source_types(&self) -> &[String];

    /// Get the list of sink types this plugin provides
    fn sink_types(&self) -> &[String];

    /// Create a source node
    fn create_source(&self, source_type: &str, config: &[(String, String)]) -> Result<Box<dyn NodeProcessor>>;

    /// Create a sink node
    fn create_sink(&self, sink_type: &str, config: &[(String, String)]) -> Result<Box<dyn NodeProcessor>>;
}

/// A metric exporter plugin
///
/// Metric exporter plugins export metrics to external systems.
pub trait MetricExporterPlugin: Plugin {
    /// Export metrics
    ///
    /// The `metrics` parameter contains serialized metric data.
    fn export(&self, metrics: &str) -> Result<()>;

    /// Get the export format this plugin supports
    fn export_format(&self) -> &str;
}

/// Helper adapter to convert a NodePlugin into a NodeProcessor factory
pub struct NodePluginAdapter {
    plugin: Box<dyn NodePlugin>,
    node_type: String,
    config: Vec<(String, String)>,
}

impl NodePluginAdapter {
    /// Create a new adapter
    pub fn new(plugin: Box<dyn NodePlugin>, node_type: String, config: Vec<(String, String)>) -> Self {
        Self {
            plugin,
            node_type,
            config,
        }
    }
}

impl NodeProcessor for NodePluginAdapter {
    fn process(
        &mut self,
        ctx: &ProcessingContext,
        packet: Packet,
        port: &str,
    ) -> Result<ProcessingResult> {
        // Create a fresh node instance for each processing step
        // This is less efficient but allows for stateless nodes
        let mut node = self.plugin.create_node(&self.config)?;
        node.process(ctx, packet, port)
    }

    fn name(&self) -> &str {
        &self.node_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_type_conversions() {
        assert_eq!(PluginType::Node.as_str(), "node");
        assert_eq!(PluginType::Codec.as_str(), "codec");
        assert_eq!(PluginType::Io.as_str(), "io");
        assert_eq!(PluginType::MetricExporter.as_str(), "metric_exporter");
        assert_eq!(PluginType::UiPanel.as_str(), "ui_panel");

        assert_eq!(PluginType::from_str("node"), Some(PluginType::Node));
        assert_eq!(PluginType::from_str("codec"), Some(PluginType::Codec));
        assert_eq!(PluginType::from_str("unknown"), None);
    }

    #[test]
    fn test_plugin_metadata_builder() {
        let metadata = PluginMetadata::new(
            "test_plugin",
            "1.0.0",
            "A test plugin",
            "Test Author",
            PluginType::Node,
        )
        .with_api_version((0, 1, 0))
        .with_capability("process")
        .with_capability("transform");

        assert_eq!(metadata.name, "test_plugin");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.plugin_type, PluginType::Node);
        assert_eq!(metadata.api_version, (0, 1, 0));
        assert!(metadata.has_capability("process"));
        assert!(metadata.has_capability("transform"));
        assert!(!metadata.has_capability("unknown"));
    }

    #[test]
    fn test_plugin_metadata_default_api_version() {
        let metadata = PluginMetadata::new("test", "1.0.0", "desc", "author", PluginType::Node);
        assert_eq!(metadata.api_version, (0, 1, 0));
    }
}
