// Caret Trace - Tracing layer
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{SpanData, SpanKind, TraceRegistry};
use std::sync::Arc;
use parking_lot::Mutex;
use tracing_core::span::Id;
use tracing_core::{Subscriber, Event, Metadata};
use tracing_subscriber::{layer::Context, Layer};

/// Caret tracing layer for collecting span data
#[derive(Debug, Clone)]
pub struct CaretLayer {
    registry: TraceRegistry,
    active_spans: Arc<Mutex<Vec<ActiveSpan>>>,
}

#[derive(Debug, Clone)]
struct ActiveSpan {
    id: Id,
    parent_id: Option<Id>,
    name: String,
    kind: SpanKind,
    start_nanos: u64,
    metadata: Vec<(String, String)>,
}

impl CaretLayer {
    /// Create a layer builder
    pub fn builder() -> CaretLayerBuilder {
        CaretLayerBuilder::default()
    }

    /// Create a new layer with the given registry
    pub fn new(registry: TraceRegistry) -> Self {
        Self {
            registry,
            active_spans: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<S: Subscriber> Layer<S> for CaretLayer {
    fn on_new_span(
        &self,
        attrs: &tracing_core::span::Attributes<'_>,
        id: &Id,
        ctx: Context<'_, S>,
    ) {
        let metadata = attrs.metadata();
        let name = metadata.name().to_string();

        // Determine span kind - default to Internal
        let kind = SpanKind::Internal;

        let parent_id = ctx.current_span().id().cloned();

        // Extract metadata fields
        let mut extracted_metadata = Vec::new();
        attrs.record(&mut SpanVisitor(&mut extracted_metadata));

        let active_span = ActiveSpan {
            id: id.clone(),
            parent_id,
            name,
            kind,
            start_nanos: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            metadata: extracted_metadata,
        };

        let mut spans = self.active_spans.lock();
        spans.push(active_span);
    }

    fn on_close(&self, id: Id, _ctx: Context<'_, S>) {
        let mut spans = self.active_spans.lock();
        if let Some(pos) = spans.iter().position(|s| s.id == id) {
            let active_span = spans.remove(pos);

            let end_nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;

            let duration_nanos = end_nanos.saturating_sub(active_span.start_nanos);

            let mut metadata = std::collections::HashMap::new();
            for (k, v) in active_span.metadata {
                metadata.insert(k, v);
            }

            let span_data = SpanData {
                id: active_span.id,
                parent_id: active_span.parent_id,
                name: active_span.name,
                kind: active_span.kind,
                start_nanos: active_span.start_nanos,
                duration_nanos,
                metadata,
            };

            self.registry.record_span(span_data);
        }
    }

    fn on_event(
        &self,
        _event: &Event<'_>,
        _ctx: Context<'_, S>,
    ) {
        // Events are logged but not stored separately in this implementation
    }
}

/// Visitor for extracting span fields
struct SpanVisitor<'a>(&'a mut Vec<(String, String)>);

impl<'a> tracing::field::Visit for SpanVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0
            .push((field.name().to_string(), format!("{:?}", value)));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0.push((field.name().to_string(), value.to_string()));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0.push((field.name().to_string(), value.to_string()));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0.push((field.name().to_string(), value.to_string()));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.0.push((field.name().to_string(), value.to_string()));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0.push((field.name().to_string(), value.to_string()));
    }
}

/// Builder for CaretLayer
#[derive(Debug, Clone, Default)]
pub struct CaretLayerBuilder {
    registry: Option<TraceRegistry>,
}

impl CaretLayerBuilder {
    /// Set the trace registry
    pub fn registry(mut self, registry: TraceRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Build the layer
    pub fn build(self) -> CaretLayer {
        let registry = self.registry.unwrap_or_else(|| {
            crate::TraceRegistry::new(crate::TraceConfig::default())
        });
        CaretLayer::new(registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_builder() {
        let registry = TraceRegistry::new(crate::TraceConfig::default());
        let layer = CaretLayer::builder()
            .registry(registry.clone())
            .build();

        assert_eq!(layer.registry.config().service_name, "caret");
    }

    #[test]
    fn test_active_span() {
        let id = Id::from_u64(1);
        let active = ActiveSpan {
            id: id.clone(),
            parent_id: None,
            name: "test".to_string(),
            kind: SpanKind::Process,
            start_nanos: 1000,
            metadata: Vec::new(),
        };

        assert_eq!(active.id.into_u64(), 1);
        assert_eq!(active.name, "test");
        assert_eq!(active.kind, SpanKind::Process);
    }
}
