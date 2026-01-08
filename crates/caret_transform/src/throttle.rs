// Caret Transform - Throttle node
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use caret_core::{Packet, Result};
use caret_io::ProcessNode;

/// Throttle mode for rate limiting
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThrottleMode {
    /// Allow at most N packets per time window
    PacketsPerWindow { packets: u64, window_ticks: u64 },
    /// Allow one packet every N ticks
    OnePerTicks(u64),
    /// Allow a percentage of packets through
    Percentage(u8),
}

impl Default for ThrottleMode {
    fn default() -> Self {
        Self::OnePerTicks(1)
    }
}

/// Configuration for a throttle node
pub struct ThrottleNodeConfig {
    /// Throttle mode
    pub mode: ThrottleMode,
    /// Optional node name
    pub name: Option<String>,
}

impl ThrottleNodeConfig {
    /// Create a new throttle node configuration
    pub fn new(mode: ThrottleMode) -> Self {
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

/// A throttle node that rate-limits packets
///
/// The throttle node controls the rate at which packets pass through.
pub struct ThrottleNode {
    mode: ThrottleMode,
    name: String,
    passed_count: u64,
    dropped_count: u64,
    // State for different modes
    packet_counter: u64,
    window_start: u64,
    packets_in_window: u64,
}

impl ThrottleNode {
    /// Create a new throttle node
    pub fn new(config: ThrottleNodeConfig) -> Self {
        Self {
            mode: config.mode,
            name: config.name.unwrap_or_else(|| "throttle".to_string()),
            passed_count: 0,
            dropped_count: 0,
            packet_counter: 0,
            window_start: 0,
            packets_in_window: 0,
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

    /// Get the throttle mode
    pub fn mode(&self) -> ThrottleMode {
        self.mode
    }

    /// Reset the throttle state
    pub fn reset(&mut self) {
        self.packet_counter = 0;
        self.window_start = 0;
        self.packets_in_window = 0;
    }
}

impl ProcessNode for ThrottleNode {
    fn process(&mut self, packet: Packet) -> Result<Option<Packet>> {
        self.packet_counter += 1;
        let current_tick = self.packet_counter;

        let should_pass = match self.mode {
            ThrottleMode::PacketsPerWindow { packets, window_ticks } => {
                // Check if we need to reset the window
                if current_tick >= self.window_start + window_ticks {
                    self.window_start = current_tick;
                    self.packets_in_window = 0;
                }

                if self.packets_in_window < packets {
                    self.packets_in_window += 1;
                    true
                } else {
                    false
                }
            }
            ThrottleMode::OnePerTicks(ticks) => {
                if ticks == 0 {
                    true
                } else if current_tick % ticks == 1 {
                    true
                } else {
                    false
                }
            }
            ThrottleMode::Percentage(percent) => {
                if percent == 0 {
                    false
                } else if percent >= 100 {
                    true
                } else {
                    // Use packet_counter modulo 100 for deterministic behavior
                    let value = (current_tick % 100) as u8;
                    value < percent
                }
            }
        };

        if should_pass {
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
impl ThrottleNode {
    /// Create a throttle that allows N packets per time window
    pub fn packets_per_window(packets: u64, window_ticks: u64) -> Self {
        Self::new(ThrottleNodeConfig::new(ThrottleMode::PacketsPerWindow {
            packets,
            window_ticks,
        }))
    }

    /// Create a throttle that allows one packet every N ticks
    pub fn one_per_ticks(ticks: u64) -> Self {
        Self::new(ThrottleNodeConfig::new(ThrottleMode::OnePerTicks(ticks)))
    }

    /// Create a throttle that allows a percentage of packets through
    ///
    /// # Panics
    ///
    /// Panics if percent > 100
    pub fn percentage(percent: u8) -> Self {
        assert!(percent <= 100, "Percentage must be <= 100");
        Self::new(ThrottleNodeConfig::new(ThrottleMode::Percentage(percent)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_packet() -> Packet {
        Packet::bytes(&b"data"[..])
    }

    #[test]
    fn test_throttle_node_creation() {
        let mut node = ThrottleNode::new(ThrottleNodeConfig::new(ThrottleMode::OnePerTicks(10)));
        assert_eq!(node.name(), "throttle");
        assert_eq!(node.mode(), ThrottleMode::OnePerTicks(10));
    }

    #[test]
    fn test_throttle_one_per_ticks() {
        let mut node = ThrottleNode::one_per_ticks(5);

        // First packet should pass
        let result = node.process(make_packet()).unwrap();
        assert!(result.is_some());
        assert_eq!(node.passed_count(), 1);
        assert_eq!(node.dropped_count(), 0);

        // Packets 2-4 should be dropped
        for _ in 1..5 {
            let result = node.process(make_packet()).unwrap();
            assert!(result.is_none());
        }
        assert_eq!(node.dropped_count(), 4);

        // Packet 6 should pass (every 5th starting from 1)
        let result = node.process(make_packet()).unwrap();
        assert!(result.is_some());
        assert_eq!(node.passed_count(), 2);
    }

    #[test]
    fn test_throttle_packets_per_window() {
        let mut node = ThrottleNode::packets_per_window(3, 10);

        // First 3 packets should pass
        for _ in 0..3 {
            let result = node.process(make_packet()).unwrap();
            assert!(result.is_some());
        }
        assert_eq!(node.passed_count(), 3);

        // Next packet should be dropped
        let result = node.process(make_packet()).unwrap();
        assert!(result.is_none());
        assert_eq!(node.dropped_count(), 1);

        // After the window, packets should pass again
        for _ in 0..10 {
            node.process(make_packet()).unwrap();
        }
        // First one in new window passes
        assert!(node.passed_count() > 3);
    }

    #[test]
    fn test_throttle_percentage_zero() {
        let mut node = ThrottleNode::percentage(0);

        for _ in 0..10 {
            let result = node.process(make_packet()).unwrap();
            assert!(result.is_none());
        }
        assert_eq!(node.passed_count(), 0);
        assert_eq!(node.dropped_count(), 10);
    }

    #[test]
    fn test_throttle_percentage_hundred() {
        let mut node = ThrottleNode::percentage(100);

        for _ in 0..10 {
            let result = node.process(make_packet()).unwrap();
            assert!(result.is_some());
        }
        assert_eq!(node.passed_count(), 10);
        assert_eq!(node.dropped_count(), 0);
    }

    #[test]
    fn test_throttle_percentage_fifty() {
        let mut node = ThrottleNode::percentage(50);

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

        // Should be approximately 50/50 (exactly 50/100 with modulo)
        assert_eq!(passed, 50);
        assert_eq!(dropped, 50);
    }

    #[test]
    #[should_panic(expected = "Percentage must be <= 100")]
    fn test_throttle_percentage_over_100_panics() {
        ThrottleNode::percentage(101);
    }

    #[test]
    fn test_throttle_with_name() {
        let mut node = ThrottleNode::new(
            ThrottleNodeConfig::new(ThrottleMode::OnePerTicks(5))
                .with_name("my_throttle"),
        );
        assert_eq!(node.name(), "my_throttle");
    }

    #[test]
    fn test_throttle_reset() {
        let mut node = ThrottleNode::one_per_ticks(5);

        node.process(make_packet()).unwrap();
        node.process(make_packet()).unwrap();
        assert_eq!(node.passed_count(), 1);
        assert_eq!(node.dropped_count(), 1);

        node.reset();
        node.process(make_packet()).unwrap();
        assert_eq!(node.passed_count(), 2); // Should pass again after reset
    }

    #[test]
    fn test_throttle_statistics() {
        let mut node = ThrottleNode::packets_per_window(2, 10);

        // 2 pass
        for _ in 0..2 {
            node.process(make_packet()).unwrap();
        }
        // 3 dropped
        for _ in 0..3 {
            node.process(make_packet()).unwrap();
        }

        assert_eq!(node.passed_count(), 2);
        assert_eq!(node.dropped_count(), 3);
        assert_eq!(node.total_count(), 5);
    }
}
