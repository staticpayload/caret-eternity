// Caret Plugins - Dynamic library loading
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{Plugin, PluginEntryFn, PluginManifestFn};
use caret_core::{Error, Result};
use libloading::{Library, Symbol};
use std::path::Path;

/// A loaded plugin library
///
/// This represents a dynamically loaded plugin library.
/// When dropped, it will close the library.
pub struct PluginLibrary {
    _lib: Library,
    plugin: *mut dyn Plugin,
}

impl PluginLibrary {
    /// Load a plugin from a dynamic library file
    ///
    /// # Safety
    ///
    /// This function is unsafe because it loads foreign code.
    /// The caller must ensure the library is a valid Caret plugin.
    pub unsafe fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // Load the library
        let lib = Library::new(path)
            .map_err(|e| Error::plugin(format!("Failed to load plugin library: {}", e)))?;

        // Get the manifest function
        let _manifest_fn: Symbol<PluginManifestFn> = lib
            .get(b"caret_plugin_manifest\0")
            .map_err(|_| Error::plugin("Plugin missing caret_plugin_manifest function".to_string()))?;

        // Get the entry function
        let entry_fn: Symbol<PluginEntryFn> = lib
            .get(b"caret_plugin_entry\0")
            .map_err(|_| Error::plugin("Plugin missing caret_plugin_entry function".to_string()))?;

        // Call the entry function to get the plugin instance
        let plugin = entry_fn();

        if plugin.is_null() {
            return Err(Error::plugin("Plugin entry function returned null".to_string()));
        }

        Ok(Self {
            _lib: lib,
            plugin,
        })
    }

    /// Get a reference to the plugin
    pub fn plugin(&self) -> &dyn Plugin {
        unsafe { &*self.plugin }
    }

    /// Get a mutable reference to the plugin
    pub fn plugin_mut(&mut self) -> &mut dyn Plugin {
        unsafe { &mut *self.plugin }
    }
}

impl Drop for PluginLibrary {
    fn drop(&mut self) {
        // Note: We don't free the plugin pointer here as it's owned by the plugin library.
        // The library should clean up its own resources when unloaded.
        // In the future, we might want to add a caret_plugin_free function.
    }
}

/// Wrapper for unsafe plugin operations
///
/// This provides a safer interface for working with potentially
/// unsafe plugin code.
pub struct UnsafePlugin {
    inner: *mut dyn Plugin,
}

impl UnsafePlugin {
    /// Create a new unsafe plugin wrapper
    ///
    /// # Safety
    ///
    /// The pointer must be valid and point to a plugin that
    /// outlives this wrapper.
    pub unsafe fn from_ptr(ptr: *mut dyn Plugin) -> Self {
        Self { inner: ptr }
    }

    /// Initialize the plugin
    pub fn initialize(&mut self) -> Result<()> {
        unsafe { (*self.inner).initialize() }
    }

    /// Cleanup the plugin
    pub fn cleanup(&mut self) -> Result<()> {
        unsafe { (*self.inner).cleanup() }
    }

    /// Get the raw pointer
    pub fn as_ptr(&self) -> *mut dyn Plugin {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_library_errors() {
        // Test loading a non-existent library
        let result = unsafe { PluginLibrary::from_path("/nonexistent/plugin.so") };
        assert!(result.is_err());
    }
}
