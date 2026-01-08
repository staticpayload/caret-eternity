// Caret Plugins - Plugin manifest
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{is_api_version_compatible, PluginType};
use caret_core::{Error, Result};
use serde::Deserialize;
use std::path::Path;

/// Default manifest file name
pub const MANIFEST_FILE: &str = "Caret.toml";

/// Plugin manifest parsed from TOML
///
/// This represents the contents of the Caret.toml file
/// that must be present in every plugin directory.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginManifest {
    /// Plugin metadata section
    #[serde(default)]
    pub plugin: PluginSection,

    /// Dependencies section
    #[serde(default)]
    pub dependencies: DependenciesSection,

    /// Capabilities section
    #[serde(default)]
    pub capabilities: CapabilitiesSection,
}

/// Main plugin section of the manifest
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PluginSection {
    /// Plugin name (required)
    pub name: String,

    /// Plugin version (required)
    pub version: String,

    /// Plugin description (required)
    pub description: String,

    /// Plugin author (required)
    pub author: String,

    /// Plugin type (required)
    #[serde(rename = "type")]
    pub plugin_type: String,

    /// Caret API version requirement (optional)
    ///
    /// Format: "0.1.0" or "^0.1.0" or "~0.1.0"
    #[serde(default = "default_api_version")]
    pub caret_api: String,

    /// Optional license information
    #[serde(default)]
    pub license: Option<String>,

    /// Optional repository URL
    #[serde(default)]
    pub repository: Option<String>,

    /// Optional homepage URL
    #[serde(default)]
    pub homepage: Option<String>,
}

fn default_api_version() -> String {
    "0.1.0".to_string()
}

/// Dependencies section
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DependenciesSection {
    /// Required Caret version
    #[serde(default)]
    pub caret: Option<String>,

    /// Other plugins this plugin depends on
    #[serde(default)]
    pub plugins: Vec<PluginDependency>,
}

/// A dependency on another plugin
#[derive(Debug, Clone, Deserialize)]
pub struct PluginDependency {
    /// Plugin name
    pub name: String,

    /// Version requirement
    pub version: Option<String>,

    /// Optional: required for this plugin to function
    #[serde(default)]
    pub required: bool,
}

/// Capabilities section
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CapabilitiesSection {
    /// Node types provided
    #[serde(default)]
    pub nodes: Vec<String>,

    /// Codec names provided
    #[serde(default)]
    pub codecs: Vec<String>,

    /// Source types provided
    #[serde(default)]
    pub sources: Vec<String>,

    /// Sink types provided
    #[serde(default)]
    pub sinks: Vec<String>,

    /// Metric export formats provided
    #[serde(default)]
    pub metric_formats: Vec<String>,

    /// UI panels provided
    #[serde(default)]
    pub ui_panels: Vec<String>,
}

/// A parsed capability from the manifest
#[derive(Clone, Debug)]
pub enum PluginCapability {
    /// Node type
    Node(String),
    /// Codec
    Codec(String),
    /// Source
    Source(String),
    /// Sink
    Sink(String),
    /// Metric export format
    MetricFormat(String),
    /// UI panel
    UiPanel(String),
}

impl PluginManifest {
    /// Parse a manifest from a string
    pub fn from_str(s: &str) -> Result<Self> {
        toml::from_str(s).map_err(|e| Error::config(format!("Invalid manifest TOML: {}", e)))
    }

    /// Parse a manifest from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| Error::io(format!("Failed to read manifest: {}", e)))?;
        Self::from_str(&content)
    }

    /// Get the plugin type
    pub fn plugin_type(&self) -> Result<PluginType> {
        PluginType::from_str(&self.plugin.plugin_type)
            .ok_or_else(|| Error::config(format!("Unknown plugin type: {}", self.plugin.plugin_type)))
    }

    /// Check if the manifest is compatible with the current API
    pub fn check_api_compatibility(&self) -> Result<()> {
        let version_str = self.plugin.caret_api.trim();
        let version = parse_version_requirement(version_str)?;

        if !is_api_version_compatible(version) {
            return Err(Error::config(format!(
                "Plugin requires API version {:?} but we support {:?}",
                version,
                (crate::MIN_API_VERSION, crate::MAX_API_VERSION)
            )));
        }

        Ok(())
    }

    /// Get all capabilities as a list
    pub fn capabilities(&self) -> Vec<PluginCapability> {
        let mut caps = Vec::new();

        for node in &self.capabilities.nodes {
            caps.push(PluginCapability::Node(node.clone()));
        }
        for codec in &self.capabilities.codecs {
            caps.push(PluginCapability::Codec(codec.clone()));
        }
        for source in &self.capabilities.sources {
            caps.push(PluginCapability::Source(source.clone()));
        }
        for sink in &self.capabilities.sinks {
            caps.push(PluginCapability::Sink(sink.clone()));
        }
        for format in &self.capabilities.metric_formats {
            caps.push(PluginCapability::MetricFormat(format.clone()));
        }
        for panel in &self.capabilities.ui_panels {
            caps.push(PluginCapability::UiPanel(panel.clone()));
        }

        caps
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<()> {
        // Check required fields
        if self.plugin.name.is_empty() {
            return Err(Error::config("Plugin name cannot be empty".to_string()));
        }
        if self.plugin.version.is_empty() {
            return Err(Error::config("Plugin version cannot be empty".to_string()));
        }
        if self.plugin.description.is_empty() {
            return Err(Error::config("Plugin description cannot be empty".to_string()));
        }
        if self.plugin.author.is_empty() {
            return Err(Error::config("Plugin author cannot be empty".to_string()));
        }

        // Check plugin type
        self.plugin_type()?;

        // Check API compatibility
        self.check_api_compatibility()?;

        Ok(())
    }
}

/// Parse a simple version string like "0.1.0" into a tuple
fn parse_version_requirement(s: &str) -> Result<(u64, u64, u64)> {
    let s = s.trim();
    let s = if let Some(rest) = s.strip_prefix('^') {
        rest
    } else if let Some(rest) = s.strip_prefix('~') {
        rest
    } else if let Some(rest) = s.strip_prefix('=') {
        rest
    } else {
        s
    };

    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return Err(Error::config(format!("Invalid version format: {}", s)));
    }

    let major = parts[0]
        .parse::<u64>()
        .map_err(|_| Error::config(format!("Invalid major version: {}", parts[0])))?;
    let minor = parts[1]
        .parse::<u64>()
        .map_err(|_| Error::config(format!("Invalid minor version: {}", parts[1])))?;
    let patch = parts[2]
        .parse::<u64>()
        .map_err(|_| Error::config(format!("Invalid patch version: {}", parts[2])))?;

    Ok((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_MANIFEST: &str = r#"
[plugin]
name = "test_plugin"
version = "1.0.0"
description = "A test plugin"
author = "Test Author"
type = "node"
caret_api = "0.1.0"

[dependencies]
caret = "0.1.0"

[[dependencies.plugins]]
name = "other_plugin"
version = "1.0.0"
required = true

[capabilities]
nodes = ["process", "transform"]
codecs = ["custom"]
"#;

    #[test]
    fn test_parse_valid_manifest() {
        let manifest = PluginManifest::from_str(VALID_MANIFEST).unwrap();
        assert_eq!(manifest.plugin.name, "test_plugin");
        assert_eq!(manifest.plugin.version, "1.0.0");
        assert_eq!(manifest.plugin.plugin_type, "node");
        assert_eq!(manifest.plugin.caret_api, "0.1.0");
    }

    #[test]
    fn test_manifest_capabilities() {
        let manifest = PluginManifest::from_str(VALID_MANIFEST).unwrap();
        let caps = manifest.capabilities();
        assert_eq!(caps.len(), 3);
        assert!(matches!(caps[0], PluginCapability::Node(_)));
        assert!(matches!(caps[1], PluginCapability::Node(_)));
        assert!(matches!(caps[2], PluginCapability::Codec(_)));
    }

    #[test]
    fn test_manifest_plugin_type() {
        let manifest = PluginManifest::from_str(VALID_MANIFEST).unwrap();
        assert_eq!(manifest.plugin_type().unwrap(), PluginType::Node);
    }

    #[test]
    fn test_manifest_validate() {
        let manifest = PluginManifest::from_str(VALID_MANIFEST).unwrap();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_version_parsing() {
        assert_eq!(parse_version_requirement("0.1.0").unwrap(), (0, 1, 0));
        assert_eq!(parse_version_requirement("^0.1.0").unwrap(), (0, 1, 0));
        assert_eq!(parse_version_requirement("~0.1.0").unwrap(), (0, 1, 0));
        assert_eq!(parse_version_requirement("=0.1.0").unwrap(), (0, 1, 0));
    }

    #[test]
    fn test_minimal_manifest() {
        const MINIMAL: &str = r#"
[plugin]
name = "minimal"
version = "1.0.0"
description = "Minimal"
author = "Author"
type = "codec"
"#;

        let manifest = PluginManifest::from_str(MINIMAL).unwrap();
        assert_eq!(manifest.plugin.name, "minimal");
        assert_eq!(manifest.plugin.caret_api, "0.1.0"); // default
    }

    #[test]
    fn test_invalid_plugin_type() {
        const INVALID_TYPE: &str = r#"
[plugin]
name = "test"
version = "1.0.0"
description = "Test"
author = "Author"
type = "unknown"
"#;

        let manifest = PluginManifest::from_str(INVALID_TYPE).unwrap();
        assert!(manifest.plugin_type().is_err());
    }
}
