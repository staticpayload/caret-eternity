// Caret Inspector - Runtime introspection and debugging API
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

//! Caret Inspector - Runtime introspection and debugging API
//!
//! This crate provides HTTP/WebSocket APIs for inspecting the state of
//! a running Caret pipeline, including graph topology, node states,
//! metrics, and real-time event streaming.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod api;
mod runtime_integration;
mod server;
mod snapshot;
mod stream;

pub use api::{ApiError, InspectorApi};
pub use runtime_integration::RuntimeIntegration;
pub use server::{InspectorConfig, InspectorServer};
pub use snapshot::{
    BufferPoolSnapshot, EdgeSnapshot, GraphSnapshot, MetricSnapshot, NodeSnapshot,
    PortSnapshot, RuntimeSnapshot,
};
pub use stream::{Event, EventStream, EventType};

use std::sync::Arc;
use parking_lot::Mutex;

/// Inspector for runtime introspection
///
/// The inspector provides real-time access to runtime state including
/// graph topology, node states, metrics, and buffer pool statistics.
#[derive(Clone)]
pub struct Inspector {
    /// Runtime state snapshot
    runtime: Arc<Mutex<RuntimeSnapshot>>,
    /// Graph state snapshot
    graph: Arc<Mutex<GraphSnapshot>>,
    /// Metrics snapshot
    metrics: Arc<Mutex<Vec<MetricSnapshot>>>,
}

impl Inspector {
    /// Create a new inspector
    pub fn new() -> Self {
        Self {
            runtime: Arc::new(Mutex::new(RuntimeSnapshot::default())),
            graph: Arc::new(Mutex::new(GraphSnapshot::default())),
            metrics: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the runtime snapshot
    pub fn runtime_snapshot(&self) -> RuntimeSnapshot {
        self.runtime.lock().clone()
    }

    /// Get the graph snapshot
    pub fn graph_snapshot(&self) -> GraphSnapshot {
        self.graph.lock().clone()
    }

    /// Get the metrics snapshot
    pub fn metrics_snapshot(&self) -> Vec<MetricSnapshot> {
        self.metrics.lock().clone()
    }

    /// Update the runtime snapshot
    pub fn update_runtime(&self, snapshot: RuntimeSnapshot) {
        *self.runtime.lock() = snapshot;
    }

    /// Update the graph snapshot
    pub fn update_graph(&self, snapshot: GraphSnapshot) {
        *self.graph.lock() = snapshot;
    }

    /// Update the metrics snapshot
    pub fn update_metrics(&self, metrics: Vec<MetricSnapshot>) {
        *self.metrics.lock() = metrics;
    }
}

impl Default for Inspector {
    fn default() -> Self {
        Self::new()
    }
}

/// Inspector handle for external access
///
/// This handle provides thread-safe access to the inspector
/// from external contexts like HTTP handlers.
#[derive(Clone)]
pub struct InspectorHandle {
    inspector: Inspector,
}

impl InspectorHandle {
    /// Create a new inspector handle
    pub fn new(inspector: Inspector) -> Self {
        Self { inspector }
    }

    /// Get the inspector
    pub fn inspector(&self) -> &Inspector {
        &self.inspector
    }

    /// Get a snapshot of all inspector data
    pub fn snapshot(&self) -> InspectorSnapshot {
        InspectorSnapshot {
            runtime: self.inspector.runtime_snapshot(),
            graph: self.inspector.graph_snapshot(),
            metrics: self.inspector.metrics_snapshot(),
        }
    }
}

/// Complete snapshot of inspector data
#[derive(Clone, Debug, serde::Serialize)]
pub struct InspectorSnapshot {
    /// Runtime state
    pub runtime: RuntimeSnapshot,
    /// Graph state
    pub graph: GraphSnapshot,
    /// Metrics
    pub metrics: Vec<MetricSnapshot>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_create() {
        let inspector = Inspector::new();
        let runtime = inspector.runtime_snapshot();
        assert_eq!(runtime.state, crate::snapshot::RuntimeState::Stopped);
    }

    #[test]
    fn test_inspector_handle() {
        let inspector = Inspector::new();
        let handle = InspectorHandle::new(inspector);
        let snapshot = handle.snapshot();
        assert_eq!(snapshot.runtime.state, crate::snapshot::RuntimeState::Stopped);
    }
}
