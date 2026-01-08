// Caret Metrics - Gauge metric
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{Metric, MetricValue, MetricRegistry};
use std::sync::Arc;

/// A gauge metric - value that can go up or down
///
/// Gauges are used for things like:
/// - Current temperature
/// - Queue depth
/// - Active connections
/// - Memory usage
#[derive(Debug, Clone)]
pub struct Gauge {
    metric: Arc<Metric>,
}

impl Gauge {
    /// Create a new gauge builder
    pub fn builder() -> GaugeBuilder {
        GaugeBuilder::default()
    }

    /// Set the gauge to a specific value
    pub fn set(&self, value: i64) {
        self.metric.set_value(MetricValue::Gauge(value));
    }

    /// Increment the gauge by 1
    pub fn inc(&self) {
        self.inc_by(1);
    }

    /// Decrement the gauge by 1
    pub fn dec(&self) {
        self.inc_by(-1);
    }

    /// Increment the gauge by a specific amount
    pub fn inc_by(&self, amount: i64) {
        let mut value = self.metric.value.lock();
        if let MetricValue::Gauge(ref mut v) = *value {
            *v = v.saturating_add(amount);
        }
    }

    /// Get the current value
    pub fn get(&self) -> i64 {
        self.metric.value()
            .as_gauge()
            .unwrap_or(0)
    }

    /// Get the underlying metric
    pub fn metric(&self) -> &Arc<Metric> {
        &self.metric
    }
}

/// Builder for creating gauges
#[derive(Debug, Clone, Default)]
pub struct GaugeBuilder {
    name: Option<String>,
    description: Option<String>,
    labels: Vec<(String, String)>,
    initial: i64,
}

impl GaugeBuilder {
    /// Set the gauge name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the gauge description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a label to the gauge
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.push((key.into(), value.into()));
        self
    }

    /// Set the initial value
    pub fn initial(mut self, value: i64) -> Self {
        self.initial = value;
        self
    }

    /// Register the gauge in a registry
    pub fn register(self, registry: &MetricRegistry) -> Gauge {
        let name = self.name.unwrap_or_else(|| "gauge".to_string());
        let description = self.description.unwrap_or_else(|| "A gauge metric".to_string());

        let metric = registry.register_gauge(
            name,
            description,
            self.labels,
            self.initial,
        );

        Gauge { metric }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gauge_set() {
        let registry = MetricRegistry::new();
        let gauge = Gauge::builder()
            .name("test_gauge")
            .register(&registry);

        assert_eq!(gauge.get(), 0);
        gauge.set(42);
        assert_eq!(gauge.get(), 42);
        gauge.set(-10);
        assert_eq!(gauge.get(), -10);
    }

    #[test]
    fn test_gauge_inc_dec() {
        let registry = MetricRegistry::new();
        let gauge = Gauge::builder()
            .name("inc_dec_test")
            .register(&registry);

        assert_eq!(gauge.get(), 0);
        gauge.inc();
        assert_eq!(gauge.get(), 1);
        gauge.inc_by(5);
        assert_eq!(gauge.get(), 6);
        gauge.dec();
        assert_eq!(gauge.get(), 5);
        gauge.inc_by(-10);
        assert_eq!(gauge.get(), -5);
    }

    #[test]
    fn test_gauge_builder() {
        let registry = MetricRegistry::new();
        let gauge = Gauge::builder()
            .name("builder_test")
            .description("Test gauge builder")
            .label("label1", "value1")
            .initial(100)
            .register(&registry);

        assert_eq!(gauge.get(), 100);
        gauge.dec();
        assert_eq!(gauge.get(), 99);
    }

    #[test]
    fn test_gauge_saturating() {
        let registry = MetricRegistry::new();
        let gauge = Gauge::builder()
            .name("saturating_test")
            .initial(i64::MAX - 10)
            .register(&registry);

        gauge.inc_by(100);
        assert_eq!(gauge.get(), i64::MAX);

        gauge.set(i64::MIN + 10);
        gauge.inc_by(-100);
        assert_eq!(gauge.get(), i64::MIN);
    }
}
