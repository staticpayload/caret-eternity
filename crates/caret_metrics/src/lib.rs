// Caret Metrics - Metrics collection for Caret
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

#![warn(missing_docs)]
#![warn(clippy::all)]

mod counter;
mod gauge;
mod histogram;
mod registry;

pub use counter::{Counter, CounterBuilder};
pub use gauge::{Gauge, GaugeBuilder};
pub use histogram::{Histogram, HistogramBuilder, Bucket};
pub use registry::{MetricRegistry, MetricRegistryBuilder, MetricId};

use std::sync::Arc;
use parking_lot::Mutex;

/// A metric value
#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    /// Counter value (monotonically increasing)
    Counter(u64),
    /// Gauge value (can go up or down)
    Gauge(i64),
    /// Histogram value (distribution)
    Histogram(Vec<u64>),
}

impl MetricValue {
    /// Get as counter
    pub fn as_counter(&self) -> Option<u64> {
        match self {
            MetricValue::Counter(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as gauge
    pub fn as_gauge(&self) -> Option<i64> {
        match self {
            MetricValue::Gauge(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as histogram
    pub fn as_histogram(&self) -> Option<&[u64]> {
        match self {
            MetricValue::Histogram(v) => Some(v),
            _ => None,
        }
    }
}

/// A metric with metadata
#[derive(Debug, Clone)]
pub struct Metric {
    /// Metric ID
    pub id: MetricId,
    /// Metric name
    pub name: Arc<str>,
    /// Metric description
    pub description: Arc<str>,
    /// Labels/tags
    pub labels: Vec<(Arc<str>, Arc<str>)>,
    /// Current value
    pub value: Arc<Mutex<MetricValue>>,
}

impl Metric {
    /// Create a new metric
    fn new(
        id: MetricId,
        name: impl Into<Arc<str>>,
        description: impl Into<Arc<str>>,
        labels: Vec<(impl Into<Arc<str>>, impl Into<Arc<str>>)>,
        initial_value: MetricValue,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.into(),
            labels: labels
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
            value: Arc::new(Mutex::new(initial_value)),
        }
    }

    /// Get the current value
    pub fn value(&self) -> MetricValue {
        self.value.lock().clone()
    }

    /// Update the value (for gauges)
    pub fn set_value(&self, value: MetricValue) {
        *self.value.lock() = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_value_as_counter() {
        let value = MetricValue::Counter(42);
        assert_eq!(value.as_counter(), Some(42));
        assert_eq!(value.as_gauge(), None);
    }

    #[test]
    fn test_metric_value_as_gauge() {
        let value = MetricValue::Gauge(-5);
        assert_eq!(value.as_gauge(), Some(-5));
        assert_eq!(value.as_counter(), None);
    }

    #[test]
    fn test_metric_value_as_histogram() {
        let value = MetricValue::Histogram(vec![1, 2, 3]);
        assert_eq!(value.as_histogram(), Some(&[1, 2, 3][..]));
        assert_eq!(value.as_counter(), None);
    }

    #[test]
    fn test_metric_creation() {
        let metric = Metric::new(
            MetricId::new(0, 0),
            "test_metric",
            "A test metric",
            vec![("label1", "value1")],
            MetricValue::Counter(0),
        );
        assert_eq!(metric.name.as_ref(), "test_metric");
        assert_eq!(metric.value(), MetricValue::Counter(0));
    }
}
