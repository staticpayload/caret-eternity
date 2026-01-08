// Caret Transform - Sample node
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use caret_core::{Packet, Result};
use caret_io::ProcessNode;

/// Sample mode for selecting packets
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SampleMode {
    /// Take every Nth packet
    EveryNth(usize),
    /// Take the first N packets, then drop the rest
    FirstN(usize),
    /// Take a random sample (percentage 0-100)
    RandomPercentage(u8),
    /// Take packets at specific indices (0-based)
    AtIndices(Vec<usize>),
}

impl Default for SampleMode {
    fn default() -> Self {
        Self::EveryNth(1)
    }
}

/// Configuration for a sample node
pub struct SampleNodeConfig {
    /// Sample mode
    pub mode: SampleMode,
    /// Optional node name
    pub name: Option<String>,
}

impl SampleNodeConfig {
    /// Create a new sample node configuration
    pub fn new(mode: SampleMode) -> Self {
        Self {
            mode,
            name: None,
        }
    }

    /// Set the node name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// A sample node that selects a subset of packets
///
/// The sample node passes through only a subset of incoming packets
/// based on the sampling mode.
pub struct SampleNode {
    mode: SampleMode,
    name: String,
    passed_count: u64,
    dropped_count: u64,
    packet_index: usize,
    next_index: usize,
}

impl SampleNode {
    /// Create a new sample node
    pub fn new(config: SampleNodeConfig) -> Self {
        Self {
            mode: config.mode,
            name: config.name.unwrap_or_else(|| "sample".to_string()),
            passed_count: 0,
            dropped_count: 0,
            packet_index: 0,
            next_index: 0,
        }
    }

    /// Get the number of packets that passed through
    pub fn passed_count(&self) -> u64 {
        self.passed_count
    }

    /// Get the number of packets that were dropped
    pub fn dropped_count(&self) -> u64 {
        self.dropped_count
    }

    /// Get the total number of packets processed
    pub fn total_count(&self) -> u64 {
        self.passed_count + self.dropped_count
    }

    /// Get the sample mode
    pub fn mode(&self) -> SampleMode {
        self.mode.clone()
    }

    /// Check if a packet should pass based on the current mode
    fn should_pass(&mut self) -> bool {
        match self.mode {
            SampleMode::EveryNth(n) => {
                if n == 0 {
                    return false;
                }
                let result = self.packet_index % n == 0;
                self.packet_index += 1;
                result
            }
            SampleMode::FirstN(n) => {
                if self.packet_index < n {
                    self.packet_index += 1;
                    true
                } else {
                    self.packet_index += 1;
                    false
                }
            }
            SampleMode::RandomPercentage(percent) => {
                self.packet_index += 1;
                if percent == 0 {
                    return false;
                }
                if percent >= 100 {
                    return true;
                }
                use std::sync::atomic::{AtomicU64, Ordering};
                static COUNTER: AtomicU64 = AtomicU64::new(1);
                let value = (COUNTER.fetch_add(1, Ordering::Relaxed) % 100) as u8;
                value < percent
            }
            SampleMode::AtIndices(ref indices) => {
                if indices.is_empty() {
                    self.packet_index += 1;
                    return false;
                }

                // Advance next_index until we find a match or pass the current index
                while self.next_index < indices.len() && indices[self.next_index] < self.packet_index {
                    self.next_index += 1;
                }

                self.packet_index += 1;

                if self.next_index < indices.len() && indices[self.next_index] == self.packet_index - 1 {
                    self.next_index += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Reset the sample state
    pub fn reset(&mut self) {
        self.packet_index = 0;
        self.next_index = 0;
    }
}

impl ProcessNode for SampleNode {
    fn process(&mut self, packet: Packet) -> Result<Option<Packet>> {
        if self.should_pass() {
            self.passed_count += 1;
            Ok(Some(packet))
        } else {
            self.dropped_count += 1;
            Ok(None)
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Convenience constructors
impl SampleNode {
    /// Create a sample that takes every Nth packet
    pub fn every_nth(n: usize) -> Self {
        Self::new(SampleNodeConfig::new(SampleMode::EveryNth(n)))
    }

    /// Create a sample that takes the first N packets only
    pub fn first_n(n: usize) -> Self {
        Self::new(SampleNodeConfig::new(SampleMode::FirstN(n)))
    }

    /// Create a sample that takes a random percentage of packets
    ///
    /// # Panics
    ///
    /// Panics if percent > 100
    pub fn random_percentage(percent: u8) -> Self {
        assert!(percent <= 100, "Percentage must be <= 100");
        Self::new(SampleNodeConfig::new(SampleMode::RandomPercentage(percent)))
    }

    /// Create a sample that takes packets at specific indices
    pub fn at_indices(indices: Vec<usize>) -> Self {
        Self::new(SampleNodeConfig::new(SampleMode::AtIndices(indices)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use caret_sched::{ProcessingContext, NodeId};

    fn make_ctx() -> ProcessingContext {
        ProcessingContext::new(NodeId::new(1), 1)
    }

    fn make_packet() -> Packet {
        Packet::bytes(&b"data"[..])
    }

    #[test]
    fn test_sample_node_creation() {
        let mut node = SampleNode::new(SampleNodeConfig::new(SampleMode::EveryNth(5)));
        assert_eq!(node.name(), "sample");
        assert_eq!(node.mode(), SampleMode::EveryNth(5));
    }

    #[test]
    fn test_sample_every_nth() {
        let mut node = SampleNode::every_nth(3);

        // Packets at indices 0, 3, 6, 9 should pass
        for i in 0..10 {
            let result = node.process(make_packet()).unwrap();
            let should_pass = i % 3 == 0;
            assert_eq!(result.is_some(), should_pass, "Index {}", i);
        }

        assert_eq!(node.passed_count(), 4);
        assert_eq!(node.dropped_count(), 6);
    }

    #[test]
    fn test_sample_every_nth_zero() {
        let mut node = SampleNode::every_nth(0);

        for _ in 0..5 {
            let result = node.process(make_packet()).unwrap();
            assert!(result.is_none());
        }

        assert_eq!(node.passed_count(), 0);
        assert_eq!(node.dropped_count(), 5);
    }

    #[test]
    fn test_sample_every_nth_one() {
        let mut node = SampleNode::every_nth(1);

        for _ in 0..5 {
            let result = node.process(make_packet()).unwrap();
            assert!(result.is_some());
        }

        assert_eq!(node.passed_count(), 5);
        assert_eq!(node.dropped_count(), 0);
    }

    #[test]
    fn test_sample_first_n() {
        let mut node = SampleNode::first_n(3);

        // First 3 packets should pass
        for _ in 0..3 {
            let result = node.process(make_packet()).unwrap();
            assert!(result.is_some());
        }

        // Rest should be dropped
        for _ in 0..5 {
            let result = node.process(make_packet()).unwrap();
            assert!(result.is_none());
        }

        assert_eq!(node.passed_count(), 3);
        assert_eq!(node.dropped_count(), 5);
    }

    #[test]
    fn test_sample_first_n_zero() {
        let mut node = SampleNode::first_n(0);

        for _ in 0..5 {
            let result = node.process(make_packet()).unwrap();
            assert!(result.is_none());
        }

        assert_eq!(node.passed_count(), 0);
    }

    #[test]
    fn test_sample_random_percentage_zero() {
        let mut node = SampleNode::random_percentage(0);

        for _ in 0..10 {
            let result = node.process(make_packet()).unwrap();
            assert!(result.is_none());
        }

        assert_eq!(node.passed_count(), 0);
    }

    #[test]
    fn test_sample_random_percentage_hundred() {
        let mut node = SampleNode::random_percentage(100);

        for _ in 0..10 {
            let result = node.process(make_packet()).unwrap();
            assert!(result.is_some());
        }

        assert_eq!(node.passed_count(), 10);
    }

    #[test]
    fn test_sample_random_percentage_fifty() {
        let mut node = SampleNode::random_percentage(50);

        let mut passed = 0;
        let mut dropped = 0;
        for _ in 0..100 {
            let result = node.process(make_packet()).unwrap();
            if result.is_some() {
                passed += 1;
            } else {
                dropped += 1;
            }
        }

        // Should be approximately 50/50
        assert!((passed as i32 - dropped as i32).abs() < 30);
    }

    #[test]
    #[should_panic(expected = "Percentage must be <= 100")]
    fn test_sample_random_percentage_over_100_panics() {
        SampleNode::random_percentage(101);
    }

    #[test]
    fn test_sample_at_indices() {
        let mut node = SampleNode::at_indices(vec![0, 2, 5, 10]);

        for i in 0..12 {
            let result = node.process(make_packet()).unwrap();
            let should_pass = matches!(i, 0 | 2 | 5 | 10);
            assert_eq!(result.is_some(), should_pass, "Index {}", i);
        }

        assert_eq!(node.passed_count(), 4);
        assert_eq!(node.dropped_count(), 8);
    }

    #[test]
    fn test_sample_at_indices_empty() {
        let mut node = SampleNode::at_indices(vec![]);

        for _ in 0..5 {
            let result = node.process(make_packet()).unwrap();
            assert!(result.is_none());
        }

        assert_eq!(node.passed_count(), 0);
    }

    #[test]
    fn test_sample_with_name() {
        let mut node = SampleNode::new(
            SampleNodeConfig::new(SampleMode::EveryNth(5))
                .with_name("my_sample"),
        );
        assert_eq!(node.name(), "my_sample");
    }

    #[test]
    fn test_sample_reset() {
        let mut node = SampleNode::every_nth(3);

        // Process some packets
        for _ in 0..6 {
            node.process(make_packet()).unwrap();
        }

        assert_eq!(node.passed_count(), 2); // indices 0 and 3

        node.reset();

        // After reset, should start fresh
        let result = node.process(make_packet()).unwrap();
        assert!(result.is_some()); // Index 0 should pass again
    }

    #[test]
    fn test_sample_statistics() {
        let mut node = SampleNode::every_nth(2);

        for _ in 0..10 {
            node.process(make_packet()).unwrap();
        }

        assert_eq!(node.passed_count(), 5);
        assert_eq!(node.dropped_count(), 5);
        assert_eq!(node.total_count(), 10);
    }
}
