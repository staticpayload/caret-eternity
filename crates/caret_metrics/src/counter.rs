// Caret Metrics - Counter metric
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{Metric, MetricValue, MetricRegistry};
use std::sync::Arc;

/// A counter metric - monotonically increasing value
///
/// Counters are used for things like:
/// - Number of packets processed
/// - Number of errors
/// - Number of operations performed
#[derive(Debug, Clone)]
pub struct Counter {
    metric: Arc<Metric>,
}

impl Counter {
    /// Create a new counter builder
    pub fn builder() -> CounterBuilder {
        CounterBuilder::default()
    }

    /// Increment the counter by 1
    pub fn inc(&self) {
        self.inc_by(1);
    }

    /// Increment the counter by a specific amount
    pub fn inc_by(&self, amount: u64) {
        let mut value = self.metric.value.lock();
        if let MetricValue::Counter(ref mut v) = *value {
            *v = v.saturating_add(amount);
        }
    }

    /// Get the current count
    pub fn get(&self) -> u64 {
        self.metric.value()
            .as_counter()
            .unwrap_or(0)
    }

    /// Get the underlying metric
    pub fn metric(&self) -> &Arc<Metric> {
        &self.metric
    }
}

/// Builder for creating counters
#[derive(Debug, Clone, Default)]
pub struct CounterBuilder {
    name: Option<String>,
    description: Option<String>,
    labels: Vec<(String, String)>,
    initial: u64,
}

impl CounterBuilder {
    /// Set the counter name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the counter description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a label to the counter
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.push((key.into(), value.into()));
        self
    }

    /// Set the initial value
    pub fn initial(mut self, value: u64) -> Self {
        self.initial = value;
        self
    }

    /// Register the counter in a registry
    pub fn register(self, registry: &MetricRegistry) -> Counter {
        let name = self.name.unwrap_or_else(|| "counter".to_string());
        let description = self.description.unwrap_or_else(|| "A counter metric".to_string());

        let metric = registry.register_counter(
            name,
            description,
            self.labels,
            self.initial,
        );

        Counter { metric }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_inc() {
        let registry = MetricRegistry::new();
        let counter = Counter::builder()
            .name("test_counter")
            .register(&registry);

        assert_eq!(counter.get(), 0);
        counter.inc();
        assert_eq!(counter.get(), 1);
        counter.inc_by(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_counter_builder() {
        let registry = MetricRegistry::new();
        let counter = Counter::builder()
            .name("builder_test")
            .description("Test counter builder")
            .label("label1", "value1")
            .initial(10)
            .register(&registry);

        assert_eq!(counter.get(), 10);
        counter.inc();
        assert_eq!(counter.get(), 11);
    }
}
