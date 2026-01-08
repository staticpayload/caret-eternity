// Caret Plugins - Plugin system and ABI for Caret
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

#![warn(missing_docs)]
#![warn(clippy::all)]

mod loader;
mod manifest;
mod registry;
mod types;

pub use loader::{PluginLibrary, UnsafePlugin};
pub use manifest::{PluginCapability, PluginManifest, MANIFEST_FILE};
pub use registry::{
    CodecPluginEntry, IoPluginEntry, MetricExporterPluginEntry, NodePluginEntry, PluginRegistry,
};
pub use types::{CodecPlugin, IoPlugin, MetricExporterPlugin, NodePlugin, PluginMetadata, PluginType};

use caret_core::Result;

/// Plugin system version
pub const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Minimum supported plugin API version
pub const MIN_API_VERSION: (u64, u64, u64) = (0, 1, 0);

/// Maximum supported plugin API version
pub const MAX_API_VERSION: (u64, u64, u64) = (0, 1, 0);

/// Check if a version is compatible with the current plugin API
pub fn is_api_version_compatible(version: (u64, u64, u64)) -> bool {
    version >= MIN_API_VERSION && version <= MAX_API_VERSION
}

/// Trait that all plugins must implement
///
/// This provides the basic interface that the plugin system expects.
pub trait Plugin: Send + Sync {
    /// Get the plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin
    ///
    /// Called once after the plugin is loaded.
    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Cleanup the plugin
    ///
    /// Called once before the plugin is unloaded.
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Entry point function that plugins must export
///
/// Plugins should export a function named `caret_plugin_entry` that
/// returns a pointer to the plugin instance.
pub type PluginEntryFn = unsafe extern "C" fn() -> *mut dyn Plugin;

/// Function to get the manifest from a plugin
///
/// Plugins should export a function named `caret_plugin_manifest`
/// that returns a pointer to a null-terminated string containing
/// the manifest in TOML format.
pub type PluginManifestFn = unsafe extern "C" fn() -> *const u8;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_compatibility() {
        assert!(is_api_version_compatible((0, 1, 0)));
        assert!(!is_api_version_compatible((0, 0, 9)));
        assert!(!is_api_version_compatible((0, 2, 0)));
    }

    #[test]
    fn test_plugin_version() {
        assert!(!PLUGIN_VERSION.is_empty());
    }

    #[test]
    fn test_version_bounds() {
        assert!(MIN_API_VERSION <= MAX_API_VERSION);
    }
}
