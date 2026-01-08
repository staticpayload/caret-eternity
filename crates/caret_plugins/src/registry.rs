// Caret Plugins - Plugin registry
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{CodecPlugin, IoPlugin, MetricExporterPlugin, NodePlugin, Plugin, PluginLibrary, PluginType};
use caret_core::{Error, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Global plugin registry
///
/// The registry manages all loaded plugins and provides
/// access to their capabilities.
#[derive(Clone)]
pub struct PluginRegistry {
    inner: Arc<RwLock<RegistryInner>>,
}

struct RegistryInner {
    libraries: Vec<PluginLibrary>,
    plugins: HashMap<String, Arc<dyn Plugin + 'static>>,
    node_factories: HashMap<String, Arc<dyn NodePlugin + 'static>>,
    codec_factories: HashMap<String, Arc<dyn CodecPlugin + 'static>>,
    io_factories: HashMap<String, Arc<dyn IoPlugin + 'static>>,
    metric_exporters: Vec<Arc<dyn MetricExporterPlugin + 'static>>,
}

impl Default for RegistryInner {
    fn default() -> Self {
        Self {
            libraries: Vec::new(),
            plugins: HashMap::new(),
            node_factories: HashMap::new(),
            codec_factories: HashMap::new(),
            io_factories: HashMap::new(),
            metric_exporters: Vec::new(),
        }
    }
}

impl PluginRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(RegistryInner::default())),
        }
    }

    /// Load a plugin from a dynamic library file
    pub fn load_plugin<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let path = path.as_ref();

        // Load the library
        let library = unsafe { PluginLibrary::from_path(path)? };
        let plugin = library.plugin();
        let metadata = plugin.metadata();

        let plugin_name = metadata.name.clone();

        // Check for duplicate plugin names
        let inner = self.inner.read().map_err(|e| Error::internal(e.to_string()))?;
        if inner.plugins.contains_key(&plugin_name) {
            return Err(Error::plugin(format!("Plugin '{}' already loaded", plugin_name)));
        }
        drop(inner);

        // Register based on plugin type
        let mut inner = self.inner.write().map_err(|e| Error::internal(e.to_string()))?;

        // Store the library to keep it alive
        let _library_index = inner.libraries.len();
        inner.libraries.push(library);

        // We can't directly store the plugin from the library due to lifetime issues.
        // For v0, plugins register their factories explicitly.
        // In a future version, we might use a different approach.

        Ok(plugin_name)
    }

    /// Register a node plugin
    pub fn register_node_plugin(&self, name: String, plugin: Arc<dyn NodePlugin + 'static>) -> Result<()> {
        let mut inner = self.inner.write().map_err(|e| Error::internal(e.to_string()))?;

        // Register the plugin itself
        inner.plugins.insert(name.clone(), plugin.clone());

        // Register node types
        for node_type in plugin.node_types() {
            inner.node_factories.insert(node_type.clone(), plugin.clone());
        }

        Ok(())
    }

    /// Register a codec plugin
    pub fn register_codec_plugin(&self, name: String, plugin: Arc<dyn CodecPlugin + 'static>) -> Result<()> {
        let mut inner = self.inner.write().map_err(|e| Error::internal(e.to_string()))?;

        inner.plugins.insert(name.clone(), plugin.clone());

        for codec_name in plugin.codec_names() {
            inner.codec_factories.insert(codec_name.clone(), plugin.clone());
        }

        Ok(())
    }

    /// Register an IO plugin
    pub fn register_io_plugin(&self, name: String, plugin: Arc<dyn IoPlugin + 'static>) -> Result<()> {
        let mut inner = self.inner.write().map_err(|e| Error::internal(e.to_string()))?;

        inner.plugins.insert(name.clone(), plugin.clone());

        for source_type in plugin.source_types() {
            inner.io_factories.insert(format!("source:{}", source_type), plugin.clone());
        }

        for sink_type in plugin.sink_types() {
            inner.io_factories.insert(format!("sink:{}", sink_type), plugin.clone());
        }

        Ok(())
    }

    /// Register a metric exporter plugin
    pub fn register_metric_exporter_plugin(&self, plugin: Arc<dyn MetricExporterPlugin + 'static>) -> Result<()> {
        let mut inner = self.inner.write().map_err(|e| Error::internal(e.to_string()))?;
        inner.metric_exporters.push(plugin);
        Ok(())
    }

    /// Get a node plugin entry by name
    pub fn get_node_plugin(&self, name: &str) -> Option<NodePluginEntry> {
        let inner = self.inner.read().ok()?;
        inner.node_factories.get(name).map(|p| NodePluginEntry { plugin: p.clone() })
    }

    /// Get a codec plugin entry by name
    pub fn get_codec_plugin(&self, name: &str) -> Option<CodecPluginEntry> {
        let inner = self.inner.read().ok()?;
        inner.codec_factories.get(name).map(|p| CodecPluginEntry { plugin: p.clone() })
    }

    /// Get an IO plugin entry by type
    pub fn get_io_plugin(&self, io_type: &str) -> Option<IoPluginEntry> {
        let inner = self.inner.read().ok()?;
        inner.io_factories.get(io_type).map(|p| IoPluginEntry { plugin: p.clone() })
    }

    /// Get all metric exporter plugins
    pub fn get_metric_exporters(&self) -> Vec<MetricExporterPluginEntry> {
        let inner = self.inner.read().ok();
        inner
            .map(|i| i.metric_exporters.iter().map(|p| MetricExporterPluginEntry { plugin: p.clone() }).collect())
            .unwrap_or_default()
    }

    /// List all registered plugin names
    pub fn list_plugins(&self) -> Vec<String> {
        let inner = self.inner.read().ok();
        inner
            .map(|i| i.plugins.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get plugin names by type
    pub fn list_plugins_by_type(&self, plugin_type: PluginType) -> Vec<String> {
        let inner = self.inner.read().ok();
        inner
            .map(|i| {
                i.plugins
                    .values()
                    .filter(|p| p.metadata().plugin_type == plugin_type)
                    .map(|p| p.metadata().name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Discover plugins in a directory
    ///
    /// Searches for plugin manifest files (Caret.toml) and
    /// attempts to load the associated dynamic libraries.
    pub fn discover_plugins(&self, dir: &Path) -> Result<Vec<String>> {
        let mut loaded = Vec::new();

        let entries = std::fs::read_dir(dir)
            .map_err(|e| Error::io(format!("Failed to read plugin directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| Error::io(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();

            // Look for dynamic library files
            let lib_name = path.file_name().and_then(|n| n.to_str());
            if let Some(name) = lib_name {
                if name.ends_with(".so") || name.ends_with(".dylib") || name.ends_with(".dll") {
                    match self.load_plugin(&path) {
                        Ok(name) => loaded.push(name),
                        Err(e) => {
                            eprintln!("Warning: Failed to load plugin {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        Ok(loaded)
    }

    /// Initialize all loaded plugins
    pub fn initialize_all(&self) -> Result<()> {
        let _inner = self.inner.read().map_err(|e| Error::internal(e.to_string()))?;

        // Note: We can't call initialize through a trait object directly
        // because initialize takes &mut self. This is a limitation we'll
        // address in a future version.
        // For now, plugins should be initialized before registration.

        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A node plugin entry from the registry
pub struct NodePluginEntry {
    plugin: Arc<dyn NodePlugin + 'static>,
}

impl NodePluginEntry {
    /// Get the plugin
    pub fn plugin(&self) -> &Arc<dyn NodePlugin + 'static> {
        &self.plugin
    }
}

/// A codec plugin entry from the registry
pub struct CodecPluginEntry {
    plugin: Arc<dyn CodecPlugin + 'static>,
}

impl CodecPluginEntry {
    /// Get the plugin
    pub fn plugin(&self) -> &Arc<dyn CodecPlugin + 'static> {
        &self.plugin
    }
}

/// An IO plugin entry from the registry
pub struct IoPluginEntry {
    plugin: Arc<dyn IoPlugin + 'static>,
}

impl IoPluginEntry {
    /// Get the plugin
    pub fn plugin(&self) -> &Arc<dyn IoPlugin + 'static> {
        &self.plugin
    }
}

/// A metric exporter plugin entry from the registry
pub struct MetricExporterPluginEntry {
    plugin: Arc<dyn MetricExporterPlugin + 'static>,
}

impl MetricExporterPluginEntry {
    /// Get the plugin
    pub fn plugin(&self) -> &Arc<dyn MetricExporterPlugin + 'static> {
        &self.plugin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = PluginRegistry::new();
        assert!(registry.list_plugins().is_empty());
    }

    #[test]
    fn test_registry_default() {
        let registry = PluginRegistry::default();
        assert!(registry.list_plugins().is_empty());
    }

    #[test]
    fn test_list_plugins_by_type() {
        let registry = PluginRegistry::new();
        assert!(registry.list_plugins_by_type(PluginType::Node).is_empty());
    }

    #[test]
    fn test_discover_nonexistent_dir() {
        let registry = PluginRegistry::new();
        let result = registry.discover_plugins(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
