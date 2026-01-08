// Caret Trace - Span data structures
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::collections::HashMap;
use tracing_core::span;

/// Kind of span
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    /// Internal span (e.g., tick, scheduling)
    Internal,
    /// Process node span
    Process,
    /// Source node span
    Source,
    /// Sink node span
    Sink,
    /// Network/IO span
    Network,
}

/// Recorded span data
#[derive(Debug, Clone)]
pub struct SpanData {
    /// Span ID
    pub id: span::Id,
    /// Parent span ID
    pub parent_id: Option<span::Id>,
    /// Span name
    pub name: String,
    /// Span kind
    pub kind: SpanKind,
    /// Start time in nanoseconds
    pub start_nanos: u64,
    /// Duration in nanoseconds
    pub duration_nanos: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl SpanData {
    /// Create new span data
    pub fn new(
        id: span::Id,
        parent_id: Option<span::Id>,
        name: String,
        kind: SpanKind,
    ) -> Self {
        Self {
            id,
            parent_id,
            name,
            kind,
            start_nanos: 0,
            duration_nanos: 0,
            metadata: HashMap::new(),
        }
    }

    /// Set start time
    pub fn with_start_nanos(mut self, nanos: u64) -> Self {
        self.start_nanos = nanos;
        self
    }

    /// Set duration
    pub fn with_duration_nanos(mut self, nanos: u64) -> Self {
        self.duration_nanos = nanos;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get duration as seconds
    pub fn duration_secs(&self) -> f64 {
        self.duration_nanos as f64 / 1_000_000_000.0
    }

    /// Get duration as milliseconds
    pub fn duration_millis(&self) -> f64 {
        self.duration_nanos as f64 / 1_000_000.0
    }

    /// Get duration as microseconds
    pub fn duration_micros(&self) -> f64 {
        self.duration_nanos as f64 / 1_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_data_new() {
        let id = span::Id::from_u64(1);
        let span_data = SpanData::new(
            id,
            None,
            "test_span".to_string(),
            SpanKind::Internal,
        );

        assert_eq!(span_data.id.into_u64(), 1);
        assert_eq!(span_data.name, "test_span");
        assert_eq!(span_data.kind, SpanKind::Internal);
        assert!(span_data.parent_id.is_none());
    }

    #[test]
    fn test_span_data_builders() {
        let id = span::Id::from_u64(1);
        let span_data = SpanData::new(
            id,
            None,
            "test".to_string(),
            SpanKind::Process,
        )
        .with_start_nanos(1000)
        .with_duration_nanos(500)
        .with_metadata("key".to_string(), "value".to_string());

        assert_eq!(span_data.start_nanos, 1000);
        assert_eq!(span_data.duration_nanos, 500);
        assert_eq!(span_data.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_span_data_conversions() {
        let id = span::Id::from_u64(1);
        let span_data = SpanData::new(
            id,
            None,
            "test".to_string(),
            SpanKind::Internal,
        )
        .with_duration_nanos(1_500_000_000); // 1.5 seconds

        assert!((span_data.duration_secs() - 1.5).abs() < 0.001);
        assert!((span_data.duration_millis() - 1500.0).abs() < 0.1);
        assert!((span_data.duration_micros() - 1_500_000.0).abs() < 1.0);
    }
}
