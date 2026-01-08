// Caret Transform - Merge node
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use caret_core::{Packet, Result};
use caret_io::ProcessNode;
use std::collections::VecDeque;

/// Strategy for merging multiple input streams
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Round-robin through inputs (default)
    RoundRobin,
    /// Prioritize first input, fall back to others
    PrioritizedFirst,
    /// Interleave packets based on availability
    Interleave,
}

/// Configuration for a merge node
pub struct MergeNodeConfig {
    /// Number of input ports
    pub inputs: usize,
    /// Merge strategy
    pub strategy: MergeStrategy,
    /// Optional node name
    pub name: Option<String>,
}

impl MergeNodeConfig {
    /// Create a new merge node configuration
    pub fn new(inputs: usize) -> Self {
        Self {
            inputs,
            strategy: MergeStrategy::default(),
            name: None,
        }
    }

    /// Set the merge strategy
    pub fn with_strategy(mut self, strategy: MergeStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set the node name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl Default for MergeStrategy {
    fn default() -> Self {
        Self::RoundRobin
    }
}

/// A merge node that combines multiple input streams into one output
///
/// The merge node has multiple input ports and produces packets on a single output.
/// The merge strategy determines how packets from different inputs are interleaved.
pub struct MergeNode {
    inputs: usize,
    strategy: MergeStrategy,
    name: String,
    current_index: usize,
    output_queue: VecDeque<Packet>,
    first_packet_sent: bool,
}

impl MergeNode {
    /// Create a new merge node
    pub fn new(config: MergeNodeConfig) -> Self {
        if config.inputs == 0 {
            panic!("Merge node requires at least one input");
        }
        Self {
            inputs: config.inputs,
            strategy: config.strategy,
            name: config.name.unwrap_or_else(|| "merge".to_string()),
            current_index: 0,
            output_queue: VecDeque::new(),
            first_packet_sent: false,
        }
    }

    /// Get the number of input ports
    pub fn input_count(&self) -> usize {
        self.inputs
    }

    /// Get the merge strategy
    pub fn strategy(&self) -> MergeStrategy {
        self.strategy
    }

    /// Get the number of packets currently queued for output
    pub fn queued_count(&self) -> usize {
        self.output_queue.len()
    }

    /// Take the next output packet (if any)
    pub fn take_output(&mut self) -> Option<Packet> {
        self.output_queue.pop_front()
    }
}

impl ProcessNode for MergeNode {
    fn process(&mut self, packet: Packet) -> Result<Option<Packet>> {
        // Note: The merge node implementation here assumes packets arrive
        // from a single source. For true multi-port merging, you'd use
        // the NodeProcessor trait directly with port information.
        // This implementation queues all packets for round-robin style
        // processing by the runtime.

        match self.strategy {
            MergeStrategy::RoundRobin => {
                // Just queue the packet; the runtime handles round-robin by
                // calling process on different ports
                self.output_queue.push_back(packet);
                Ok(None)
            }
            MergeStrategy::PrioritizedFirst => {
                // Prioritize by draining the queue first
                if !self.output_queue.is_empty() {
                    // Queue has items: add new packet, flush oldest
                    self.output_queue.push_back(packet);
                    Ok(self.output_queue.pop_front())
                } else if !self.first_packet_sent {
                    // First packet: send directly
                    self.first_packet_sent = true;
                    Ok(Some(packet))
                } else {
                    // Queue is empty but we've sent the first packet: queue this one
                    self.output_queue.push_back(packet);
                    Ok(None)
                }
            }
            MergeStrategy::Interleave => {
                // Simply queue all packets; they'll be drained in order
                self.output_queue.push_back(packet);
                Ok(None)
            }
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use caret_sched::{ProcessingContext, ProcessingResult};

    #[test]
    fn test_merge_node_creation() {
        let node = MergeNode::new(MergeNodeConfig::new(3));
        assert_eq!(node.input_count(), 3);
        assert_eq!(node.strategy(), MergeStrategy::RoundRobin);
        assert_eq!(node.name(), "merge");
    }

    #[test]
    fn test_merge_with_strategy() {
        let node = MergeNode::new(
            MergeNodeConfig::new(2)
                .with_strategy(MergeStrategy::Interleave)
                .with_name("my_merge"),
        );
        assert_eq!(node.input_count(), 2);
        assert_eq!(node.strategy(), MergeStrategy::Interleave);
        assert_eq!(node.name(), "my_merge");
    }

    #[test]
    fn test_merge_round_robin() {
        let mut node = MergeNode::new(MergeNodeConfig::new(2));
        let ctx = ProcessingContext::new(caret_sched::NodeId::new(1), 1);

        let p1 = Packet::bytes(&b"input1"[..]);
        let p2 = Packet::bytes(&b"input2"[..]);

        // Process from port 0
        let result = node.process(p1).unwrap();
        assert!(result.is_none()); // Queued
        assert_eq!(node.queued_count(), 1);

        // Process from port 1
        let result = node.process(p2).unwrap();
        assert!(result.is_none()); // Queued
        assert_eq!(node.queued_count(), 2);

        // Take outputs
        let out1 = node.take_output().unwrap();
        assert_eq!(out1.data().as_ref(), b"input1");
        let out2 = node.take_output().unwrap();
        assert_eq!(out2.data().as_ref(), b"input2");
        assert!(node.take_output().is_none());
    }

    #[test]
    fn test_merge_prioritized_first() {
        let mut node = MergeNode::new(
            MergeNodeConfig::new(2)
                .with_strategy(MergeStrategy::PrioritizedFirst),
        );

        let p1 = Packet::bytes(&b"packet1"[..]);
        let p2 = Packet::bytes(&b"packet2"[..]);
        let p3 = Packet::bytes(&b"packet3"[..]);

        // First packet passes through immediately (queue is empty)
        let result = node.process(p1).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().data().as_ref(), b"packet1");
        assert_eq!(node.queued_count(), 0);

        // Second packet gets queued
        let result = node.process(p2).unwrap();
        assert!(result.is_none());
        assert_eq!(node.queued_count(), 1);

        // Third packet flushes the queue and is added
        let result = node.process(p3).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().data().as_ref(), b"packet2");
        assert_eq!(node.queued_count(), 1);
    }

    #[test]
    fn test_merge_basic() {
        // Test that merge node can process packets
        let mut node = MergeNode::new(MergeNodeConfig::new(2));

        let packet = Packet::bytes(&b"data"[..]);
        let result = node.process(packet);
        assert!(result.is_ok());
    }

    #[test]
    fn test_merge_zero_inputs_panics() {
        // This should panic during construction
        let result = std::panic::catch_unwind(|| {
            MergeNode::new(MergeNodeConfig::new(0));
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_interleave() {
        let mut node = MergeNode::new(
            MergeNodeConfig::new(3)
                .with_strategy(MergeStrategy::Interleave),
        );
        let ctx = ProcessingContext::new(caret_sched::NodeId::new(1), 1);

        for i in 0..3 {
            let packet = Packet::bytes(format!("input{}", i).into_bytes());
            node.process(packet).unwrap();
        }

        assert_eq!(node.queued_count(), 3);

        // Packets come out in order they were received
        let out1 = node.take_output().unwrap();
        assert_eq!(out1.data().as_ref(), b"input0");
        let out2 = node.take_output().unwrap();
        assert_eq!(out2.data().as_ref(), b"input1");
        let out3 = node.take_output().unwrap();
        assert_eq!(out3.data().as_ref(), b"input2");
    }
}
