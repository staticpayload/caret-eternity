// Caret Metrics - Metric Registry
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::Metric;
use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::HashMap;

/// Unique identifier for a metric
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetricId {
    /// Registry ID (for multi-registry scenarios)
    pub registry_id: u64,
    /// Metric index within the registry
    pub metric_index: u64,
}

impl MetricId {
    /// Create a new metric ID
    pub const fn new(registry_id: u64, metric_index: u64) -> Self {
        Self {
            registry_id,
            metric_index,
        }
    }
}

/// Registry for metrics
///
/// The registry owns all metrics and provides thread-safe access to them.
#[derive(Debug, Clone)]
pub struct MetricRegistry {
    inner: Arc<RegistryInner>,
}

#[derive(Debug)]
struct RegistryInner {
    id: u64,
    metrics: Mutex<HashMap<MetricId, Arc<Metric>>>,
    next_index: Mutex<u64>,
}

impl MetricRegistry {
    /// Create a new metric registry
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Create a registry builder
    pub fn builder() -> MetricRegistryBuilder {
        MetricRegistryBuilder::default()
    }

    /// Register a counter metric
    pub fn register_counter(
        &self,
        name: impl Into<Arc<str>>,
        description: impl Into<Arc<str>>,
        labels: Vec<(impl Into<Arc<str>>, impl Into<Arc<str>>)>,
        initial: u64,
    ) -> Arc<Metric> {
        self.register(
            name,
            description,
            labels,
            crate::MetricValue::Counter(initial),
        )
    }

    /// Register a gauge metric
    pub fn register_gauge(
        &self,
        name: impl Into<Arc<str>>,
        description: impl Into<Arc<str>>,
        labels: Vec<(impl Into<Arc<str>>, impl Into<Arc<str>>)>,
        initial: i64,
    ) -> Arc<Metric> {
        self.register(
            name,
            description,
            labels,
            crate::MetricValue::Gauge(initial),
        )
    }

    /// Register a histogram metric
    ///
    /// Returns the metric and the bucket boundaries used
    pub fn register_histogram(
        &self,
        name: impl Into<Arc<str>>,
        description: impl Into<Arc<str>>,
        labels: Vec<(impl Into<Arc<str>>, impl Into<Arc<str>>)>,
        buckets: Vec<f64>,
        bucket_count: usize,
    ) -> (Arc<Metric>, Vec<f64>) {
        let metric = self.register(
            name,
            description,
            labels,
            crate::MetricValue::Histogram(vec![0; bucket_count]),
        );
        (metric, buckets)
    }

    fn register(
        &self,
        name: impl Into<Arc<str>>,
        description: impl Into<Arc<str>>,
        labels: Vec<(impl Into<Arc<str>>, impl Into<Arc<str>>)>,
        initial_value: crate::MetricValue,
    ) -> Arc<Metric> {
        let mut next_index = self.inner.next_index.lock();
        let id = MetricId::new(self.inner.id, *next_index);
        *next_index += 1;

        let metric = Arc::new(Metric::new(
            id, name, description, labels, initial_value,
        ));

        let mut metrics = self.inner.metrics.lock();
        metrics.insert(id, metric.clone());

        metric
    }

    /// Get a metric by ID
    pub fn get(&self, id: MetricId) -> Option<Arc<Metric>> {
        let metrics = self.inner.metrics.lock();
        metrics.get(&id).cloned()
    }

    /// Get all metrics
    pub fn get_all(&self) -> Vec<Arc<Metric>> {
        let metrics = self.inner.metrics.lock();
        metrics.values().cloned().collect()
    }

    /// Get the number of registered metrics
    pub fn count(&self) -> usize {
        let metrics = self.inner.metrics.lock();
        metrics.len()
    }

    /// Clear all metrics
    pub fn clear(&self) {
        let mut metrics = self.inner.metrics.lock();
        metrics.clear();
        *self.inner.next_index.lock() = 0;
    }

    /// Export metrics in a text format (Prometheus-style)
    pub fn export_text(&self) -> String {
        let metrics = self.get_all();
        let mut output = String::new();

        // Sort by name for consistent output
        let mut sorted_metrics = metrics.clone();
        sorted_metrics.sort_by(|a, b| a.name.as_ref().cmp(b.name.as_ref()));

        for metric in sorted_metrics {
            let value = metric.value();
            let labels_str = if metric.labels.is_empty() {
                String::new()
            } else {
                let labels: Vec<String> = metric
                    .labels
                    .iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect();
                format!("{{{}}}", labels.join(","))
            };

            // Help comment
            output.push_str(&format!("# HELP {} {}\n", metric.name, metric.description));

            // Type comment
            let type_str = match value {
                crate::MetricValue::Counter(_) => "counter",
                crate::MetricValue::Gauge(_) => "gauge",
                crate::MetricValue::Histogram(_) => "histogram",
            };
            output.push_str(&format!("# TYPE {} {}\n", metric.name, type_str));

            // Value(s)
            match value {
                crate::MetricValue::Counter(v) => {
                    output.push_str(&format!("{}{} {}\n", metric.name, labels_str, v));
                }
                crate::MetricValue::Gauge(v) => {
                    output.push_str(&format!("{}{} {}\n", metric.name, labels_str, v));
                }
                crate::MetricValue::Histogram(ref counts) => {
                    for (i, count) in counts.iter().enumerate() {
                        if i == counts.len() - 1 {
                            output.push_str(&format!(
                                "{}_bucket{{le=\"+Inf\"{}}} {}\n",
                                metric.name,
                                if !labels_str.is_empty() {
                                    format!(",{}", &labels_str[1..labels_str.len() - 1])
                                } else {
                                    String::new()
                                },
                                count
                            ));
                        } else {
                            output.push_str(&format!(
                                "{}_bucket{{le=\"{}\"{}}} {}\n",
                                metric.name,
                                i as f64, // Simplified - real implementation would track actual bounds
                                if !labels_str.is_empty() {
                                    format!(",{}", &labels_str[1..labels_str.len() - 1])
                                } else {
                                    String::new()
                                },
                                count
                            ));
                        }
                    }
                }
            }

            output.push('\n');
        }

        output
    }
}

impl Default for MetricRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating metric registries
#[derive(Debug, Clone, Default)]
pub struct MetricRegistryBuilder {
    _registry_id: Option<u64>,
}

impl MetricRegistryBuilder {
    /// Set the registry ID
    pub fn registry_id(mut self, id: u64) -> Self {
        self._registry_id = Some(id);
        self
    }

    /// Build the registry
    pub fn build(self) -> MetricRegistry {
        let id = self._registry_id.unwrap_or_else(|| {
            use std::sync::atomic::{AtomicU64, Ordering};
            static NEXT_ID: AtomicU64 = AtomicU64::new(1);
            NEXT_ID.fetch_add(1, Ordering::SeqCst)
        });

        MetricRegistry {
            inner: Arc::new(RegistryInner {
                id,
                metrics: Mutex::new(HashMap::new()),
                next_index: Mutex::new(0),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_create() {
        let registry = MetricRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_register_counter() {
        let registry = MetricRegistry::new();
        let counter = crate::Counter::builder()
            .name("test_counter")
            .description("A test counter")
            .register(&registry);

        assert_eq!(registry.count(), 1);
        assert_eq!(counter.get(), 0);
        counter.inc();
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_registry_register_gauge() {
        let registry = MetricRegistry::new();
        let gauge = crate::Gauge::builder()
            .name("test_gauge")
            .description("A test gauge")
            .initial(10)
            .register(&registry);

        assert_eq!(registry.count(), 1);
        assert_eq!(gauge.get(), 10);
    }

    #[test]
    fn test_registry_get() {
        let registry = MetricRegistry::new();
        let counter = crate::Counter::builder()
            .name("get_test")
            .register(&registry);

        let metric = counter.metric();
        let retrieved = registry.get(metric.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, metric.id);
    }

    #[test]
    fn test_registry_clear() {
        let registry = MetricRegistry::new();
        let _counter = crate::Counter::builder()
            .name("clear_test")
            .register(&registry);

        assert_eq!(registry.count(), 1);
        registry.clear();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_registry_export_text() {
        let registry = MetricRegistry::new();
        let counter = crate::Counter::builder()
            .name("export_test")
            .description("A counter for export testing")
            .register(&registry);

        counter.inc_by(42);

        let exported = registry.export_text();
        assert!(exported.contains("export_test"));
        assert!(exported.contains("42"));
        assert!(exported.contains("# HELP"));
        assert!(exported.contains("# TYPE"));
    }

    #[test]
    fn test_metric_id() {
        let id1 = MetricId::new(0, 0);
        let id2 = MetricId::new(0, 0);
        let id3 = MetricId::new(0, 1);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
}
