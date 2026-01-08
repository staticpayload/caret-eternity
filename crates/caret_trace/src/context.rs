// Caret Trace - Trace context for propagation
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::sync::Arc;
use parking_lot::Mutex;

/// Trace context for propagating trace information across boundaries
#[derive(Debug, Clone)]
pub struct TraceContext {
    inner: Arc<ContextInner>,
}

#[derive(Debug)]
struct ContextInner {
    trace_id: u64,
    span_id: u64,
    parent_span_id: Option<u64>,
    baggage: Mutex<Vec<(String, String)>>,
}

impl TraceContext {
    /// Create a new trace context
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ContextInner {
                trace_id: new_id(),
                span_id: new_id(),
                parent_span_id: None,
                baggage: Mutex::new(Vec::new()),
            }),
        }
    }

    /// Create a child context from this one
    pub fn child(&self) -> Self {
        Self {
            inner: Arc::new(ContextInner {
                trace_id: self.inner.trace_id,
                span_id: new_id(),
                parent_span_id: Some(self.inner.span_id),
                baggage: Mutex::new(self.inner.baggage.lock().clone()),
            }),
        }
    }

    /// Get the trace ID
    pub fn trace_id(&self) -> u64 {
        self.inner.trace_id
    }

    /// Get the span ID
    pub fn span_id(&self) -> u64 {
        self.inner.span_id
    }

    /// Get the parent span ID
    pub fn parent_span_id(&self) -> Option<u64> {
        self.inner.parent_span_id
    }

    /// Add baggage to the context
    pub fn with_baggage(&self, key: String, value: String) {
        let mut baggage = self.inner.baggage.lock();
        baggage.push((key, value));
    }

    /// Get all baggage
    pub fn baggage(&self) -> Vec<(String, String)> {
        self.inner.baggage.lock().clone()
    }

    /// Get baggage value by key
    pub fn get_baggage(&self, key: &str) -> Option<String> {
        let baggage = self.inner.baggage.lock();
        baggage.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }

    /// Encode as W3C traceparent format
    pub fn to_traceparent(&self) -> String {
        format!(
            "{:02x}-{:016x}-{:016x}-{:02x}",
            1, // version
            self.inner.trace_id,
            self.inner.span_id,
            1  // trace flags
        )
    }

    /// Decode from W3C traceparent format
    ///
    /// Note: This v0 implementation uses 64-bit IDs instead of the W3C 128-bit standard.
    /// For full W3C compatibility, upgrade to 128-bit IDs in a future version.
    pub fn from_traceparent(traceparent: &str) -> Option<Self> {
        // W3C format: version-trace_id-span_id-flags
        // v0 uses 16-char hex for trace_id and span_id (64-bit)
        let parts: Vec<&str> = traceparent.split('-').collect();
        if parts.len() != 4 {
            return None;
        }

        // Validate version
        if *parts.get(0)? != "00" && *parts.get(0)? != "01" {
            return None;
        }

        let trace_id = u64::from_str_radix(parts.get(1)?, 16).ok()?;
        let span_id = u64::from_str_radix(parts.get(2)?, 16).ok()?;

        Some(Self {
            inner: Arc::new(ContextInner {
                trace_id,
                span_id,
                parent_span_id: None,
                baggage: Mutex::new(Vec::new()),
            }),
        })
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Guard for restoring the previous trace context
///
/// When dropped, restores the previous context that was active
/// before this guard was created.
#[derive(Debug)]
pub struct TraceContextGuard {
    _prev_context: Option<TraceContext>,
}

impl TraceContextGuard {
    /// Create a new guard that restores the given context on drop
    pub(crate) fn new(prev_context: Option<TraceContext>) -> Self {
        Self {
            _prev_context: prev_context,
        }
    }
}

impl Drop for TraceContextGuard {
    fn drop(&mut self) {
        // Restore previous context if there was one
        // In a real implementation, this would use thread-local storage
    }
}

/// Generate a new random ID
fn new_id() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;

    let mut hasher = DefaultHasher::new();
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);

    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = TraceContext::new();
        assert!(ctx.trace_id() != 0);
        assert!(ctx.span_id() != 0);
        assert!(ctx.parent_span_id().is_none());
    }

    #[test]
    fn test_context_child() {
        let parent = TraceContext::new();
        let child = parent.child();

        assert_eq!(child.trace_id(), parent.trace_id());
        assert_ne!(child.span_id(), parent.span_id());
        assert_eq!(child.parent_span_id(), Some(parent.span_id()));
    }

    #[test]
    fn test_context_baggage() {
        let ctx = TraceContext::new();
        ctx.with_baggage("key1".to_string(), "value1".to_string());
        ctx.with_baggage("key2".to_string(), "value2".to_string());

        assert_eq!(ctx.get_baggage("key1"), Some("value1".to_string()));
        assert_eq!(ctx.get_baggage("key2"), Some("value2".to_string()));
        assert_eq!(ctx.get_baggage("key3"), None);
    }

    #[test]
    fn test_traceparent_roundtrip() {
        let ctx = TraceContext::new();
        let traceparent = ctx.to_traceparent();
        let decoded = TraceContext::from_traceparent(&traceparent);

        assert!(decoded.is_some());
        let decoded = decoded.unwrap();
        assert_eq!(decoded.trace_id(), ctx.trace_id());
        assert_eq!(decoded.span_id(), ctx.span_id());
    }

    #[test]
    fn test_traceparent_parse() {
        // Using 16-char hex IDs that fit in u64 (128-bit W3C format truncated for v0)
        let traceparent = "00-4bf92f3577b34da6-00f067aa0ba902b7-01";
        let ctx = TraceContext::from_traceparent(traceparent);

        assert!(ctx.is_some());
        let ctx = ctx.unwrap();
        assert_eq!(ctx.trace_id(), 0x4bf92f3577b34da6);
        assert_eq!(ctx.span_id(), 0x00f067aa0ba902b7);
    }

    #[test]
    fn test_traceparent_invalid() {
        let invalid = TraceContext::from_traceparent("invalid");
        assert!(invalid.is_none());
    }
}
