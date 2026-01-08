// Caret Metrics - Histogram metric
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{Metric, MetricValue, MetricRegistry};
use std::sync::Arc;

/// A histogram bucket boundary
pub type Bucket = f64;

/// A histogram metric - distribution of values
///
/// Histograms are used for things like:
/// - Request latencies
/// - Response sizes
/// - Queue wait times
#[derive(Debug, Clone)]
pub struct Histogram {
    metric: Arc<Metric>,
    buckets: Vec<Bucket>,
}

impl Histogram {
    /// Create a new histogram builder
    pub fn builder() -> HistogramBuilder {
        HistogramBuilder::default()
    }

    /// Record a value in the histogram
    pub fn record(&self, value: f64) {
        let mut histogram = self.metric.value.lock();
        if let MetricValue::Histogram(ref mut counts) = *histogram {
            // Find the right bucket
            let bucket_index = self
                .buckets
                .iter()
                .position(|&b| value <= b)
                .unwrap_or(self.buckets.len());

            // Increment the bucket count (store in +Inf bucket)
            if bucket_index < counts.len() {
                counts[bucket_index] += 1;
            } else if !counts.is_empty() {
                *counts.last_mut().unwrap() += 1;
            }
        }
    }

    /// Get the counts for all buckets
    pub fn get_counts(&self) -> Vec<u64> {
        self.metric
            .value()
            .as_histogram()
            .map(|v| v.to_vec())
            .unwrap_or_default()
    }

    /// Get the underlying metric
    pub fn metric(&self) -> &Arc<Metric> {
        &self.metric
    }
}

/// Builder for creating histograms
#[derive(Debug, Clone, Default)]
pub struct HistogramBuilder {
    name: Option<String>,
    description: Option<String>,
    labels: Vec<(String, String)>,
    buckets: Option<Vec<Bucket>>,
}

impl HistogramBuilder {
    /// Set the histogram name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the histogram description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a label to the histogram
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.push((key.into(), value.into()));
        self
    }

    /// Set custom bucket boundaries
    pub fn buckets(mut self, buckets: impl IntoIterator<Item = f64>) -> Self {
        self.buckets = Some(buckets.into_iter().collect());
        self
    }

    /// Use exponential bucket boundaries
    pub fn exponential_buckets(mut self, start: f64, factor: f64, count: usize) -> Self {
        let mut buckets = Vec::with_capacity(count);
        let mut current = start;
        for _ in 0..count {
            buckets.push(current);
            current *= factor;
        }
        self.buckets = Some(buckets);
        self
    }

    /// Use linear bucket boundaries
    pub fn linear_buckets(mut self, start: f64, width: f64, count: usize) -> Self {
        let mut buckets = Vec::with_capacity(count);
        for i in 0..count {
            buckets.push(start + (i as f64) * width);
        }
        self.buckets = Some(buckets);
        self
    }

    /// Register the histogram in a registry
    pub fn register(self, registry: &MetricRegistry) -> Histogram {
        let name = self.name.unwrap_or_else(|| "histogram".to_string());
        let description = self
            .description
            .unwrap_or_else(|| "A histogram metric".to_string());

        let buckets = self.buckets.unwrap_or_else(Self::default_buckets);
        let bucket_count = buckets.len() + 1; // +1 for +Inf

        let (metric, buckets_out) =
            registry.register_histogram(name, description, self.labels, buckets, bucket_count);

        Histogram {
            metric,
            buckets: buckets_out,
        }
    }

    fn default_buckets() -> Vec<f64> {
        // Default Prometheus-style buckets
        vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_record() {
        let registry = MetricRegistry::new();
        let histogram = Histogram::builder()
            .name("test_histogram")
            .buckets(vec![1.0, 5.0, 10.0])
            .register(&registry);

        histogram.record(0.5); // <= 1.0
        histogram.record(1.0); // <= 1.0
        histogram.record(3.0); // <= 5.0
        histogram.record(7.0); // <= 10.0
        histogram.record(15.0); // +Inf

        let counts = histogram.get_counts();
        assert_eq!(counts, vec![2, 1, 1, 1]); // 1.0, 5.0, 10.0, +Inf
    }

    #[test]
    fn test_histogram_exponential_buckets() {
        let registry = MetricRegistry::new();
        let histogram = Histogram::builder()
            .name("exponential_test")
            .exponential_buckets(1.0, 2.0, 4)
            .register(&registry);

        histogram.record(0.5);
        histogram.record(1.0);
        histogram.record(2.5);
        histogram.record(10.0);

        let counts = histogram.get_counts();
        assert_eq!(counts.len(), 5); // 4 buckets + +Inf
        assert_eq!(counts[0], 2); // <= 1.0
    }

    #[test]
    fn test_histogram_builder() {
        let registry = MetricRegistry::new();
        let histogram = Histogram::builder()
            .name("builder_test")
            .description("Test histogram builder")
            .label("label1", "value1")
            .linear_buckets(0.0, 10.0, 5)
            .register(&registry);

        histogram.record(5.0);
        histogram.record(15.0);
        histogram.record(35.0);

        let counts = histogram.get_counts();
        assert_eq!(counts.len(), 6); // 5 buckets + +Inf
    }

    #[test]
    fn test_histogram_default_buckets() {
        let registry = MetricRegistry::new();
        let histogram = Histogram::builder()
            .name("default_buckets_test")
            .register(&registry);

        // Default buckets should be Prometheus-style
        histogram.record(0.001);
        histogram.record(0.01);
        histogram.record(1.0);
        histogram.record(100.0);

        let counts = histogram.get_counts();
        assert!(counts.len() > 1);
    }
}
