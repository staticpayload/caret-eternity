// Caret Trace - Tracing and observability for Caret
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

#![warn(missing_docs)]
#![warn(clippy::all)]

//! Tracing and observability integration for Caret runtime.
//!
//! This crate provides:
//! - Trace context propagation
//! - Custom tracing layer for span collection
//! - Integration with the metrics system

mod context;
mod layer;
mod span;

pub use context::{TraceContext, TraceContextGuard};
pub use layer::{CaretLayer, CaretLayerBuilder};
pub use span::{SpanData, SpanKind};

use std::sync::Arc;
use parking_lot::Mutex;

/// Caret tracing configuration
#[derive(Debug, Clone)]
pub struct TraceConfig {
    /// Service name for traces
    pub service_name: String,
    /// Whether to export traces
    pub export_enabled: bool,
    /// Sample rate (0.0 to 1.0)
    pub sample_rate: f32,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            service_name: "caret".to_string(),
            export_enabled: true,
            sample_rate: 1.0,
        }
    }
}

/// Global trace registry for collecting span data
#[derive(Debug, Clone)]
pub struct TraceRegistry {
    inner: Arc<RegistryInner>,
}

#[derive(Debug)]
struct RegistryInner {
    config: TraceConfig,
    spans: Mutex<Vec<SpanData>>,
}

impl TraceRegistry {
    /// Create a new trace registry
    pub fn new(config: TraceConfig) -> Self {
        Self {
            inner: Arc::new(RegistryInner {
                config,
                spans: Mutex::new(Vec::new()),
            }),
        }
    }

    /// Record a span
    pub(crate) fn record_span(&self, span: SpanData) {
        let mut spans = self.inner.spans.lock();
        spans.push(span);
    }

    /// Get all recorded spans
    pub fn get_spans(&self) -> Vec<SpanData> {
        let spans = self.inner.spans.lock();
        spans.clone()
    }

    /// Clear all recorded spans
    pub fn clear(&self) {
        let mut spans = self.inner.spans.lock();
        spans.clear();
    }

    /// Get the number of recorded spans
    pub fn span_count(&self) -> usize {
        let spans = self.inner.spans.lock();
        spans.len()
    }

    /// Export spans as JSON
    pub fn export_json(&self) -> String {
        let spans = self.get_spans();
        serde_json::json!({
            "service": self.inner.config.service_name,
            "spans": spans.iter().map(|s| {
                serde_json::json!({
                    "id": s.id.into_u64(),
                    "parent_id": s.parent_id.as_ref().map(|id| id.into_u64()),
                    "name": s.name,
                    "kind": format!("{:?}", s.kind),
                    "start_nanos": s.start_nanos,
                    "duration_nanos": s.duration_nanos,
                    "metadata": s.metadata,
                })
            }).collect::<Vec<_>>()
        }).to_string()
    }

    /// Get the registry config
    pub fn config(&self) -> &TraceConfig {
        &self.inner.config
    }
}

/// Initialize tracing with Caret layer
///
/// This sets up the global tracing subscriber with the Caret layer
/// and a standard stdout formatter.
pub fn init_tracing(config: TraceConfig) -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::prelude::*;

    let registry = TraceRegistry::new(config.clone());
    let layer = CaretLayer::builder()
        .registry(registry.clone())
        .build();

    let subscriber = tracing_subscriber::registry()
        .with(layer)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr));

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_config_default() {
        let config = TraceConfig::default();
        assert_eq!(config.service_name, "caret");
        assert!(config.export_enabled);
        assert_eq!(config.sample_rate, 1.0);
    }

    #[test]
    fn test_registry_create() {
        let registry = TraceRegistry::new(TraceConfig::default());
        assert_eq!(registry.span_count(), 0);
    }

    #[test]
    fn test_registry_record_span() {
        let registry = TraceRegistry::new(TraceConfig::default());

        let span_data = SpanData {
            id: tracing_core::span::Id::from_u64(1),
            parent_id: None,
            name: "test_span".to_string(),
            kind: SpanKind::Internal,
            start_nanos: 1000,
            duration_nanos: 500,
            metadata: std::collections::HashMap::new(),
        };

        registry.record_span(span_data);
        assert_eq!(registry.span_count(), 1);
    }

    #[test]
    fn test_registry_clear() {
        let registry = TraceRegistry::new(TraceConfig::default());

        let span_data = SpanData {
            id: tracing_core::span::Id::from_u64(1),
            parent_id: None,
            name: "test_span".to_string(),
            kind: SpanKind::Internal,
            start_nanos: 1000,
            duration_nanos: 500,
            metadata: std::collections::HashMap::new(),
        };

        registry.record_span(span_data);
        assert_eq!(registry.span_count(), 1);
        registry.clear();
        assert_eq!(registry.span_count(), 0);
    }

    #[test]
    fn test_registry_export_json() {
        let registry = TraceRegistry::new(TraceConfig::default());

        let span_data = SpanData {
            id: tracing_core::span::Id::from_u64(1),
            parent_id: None,
            name: "test_span".to_string(),
            kind: SpanKind::Internal,
            start_nanos: 1000,
            duration_nanos: 500,
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("key".to_string(), "value".to_string());
                map
            },
        };

        registry.record_span(span_data);
        let json = registry.export_json();

        assert!(json.contains("test_span"));
        assert!(json.contains("caret"));
    }
}
