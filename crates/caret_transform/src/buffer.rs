// Caret Transform - Buffer node
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use caret_core::{Packet, Result};
use caret_io::ProcessNode;
use std::collections::VecDeque;

/// Overflow policy for buffered packets
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferPolicy {
    /// Reject new packets when buffer is full
    Reject,
    /// Drop oldest packets when buffer is full
    DropOldest,
    /// Drop newest packet when buffer is full
    DropNewest,
    /// Block (return error) when buffer is full
    Block,
}

/// Configuration for a buffer node
pub struct BufferNodeConfig {
    /// Maximum buffer size (number of packets)
    pub capacity: usize,
    /// Overflow policy
    pub policy: BufferPolicy,
    /// Optional node name
    pub name: Option<String>,
}

impl BufferNodeConfig {
    /// Create a new buffer node configuration
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            policy: BufferPolicy::default(),
            name: None,
        }
    }

    /// Set the overflow policy
    pub fn with_policy(mut self, policy: BufferPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Set the node name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl Default for BufferPolicy {
    fn default() -> Self {
        Self::Reject
    }
}

/// A buffer node that temporarily stores packets
///
/// The buffer node accumulates packets and releases them based on
/// the configured policy.
pub struct BufferNode {
    capacity: usize,
    policy: BufferPolicy,
    name: String,
    buffer: VecDeque<Packet>,
    rejected_count: u64,
    dropped_oldest_count: u64,
    dropped_newest_count: u64,
}

impl BufferNode {
    /// Create a new buffer node
    pub fn new(config: BufferNodeConfig) -> Self {
        Self {
            capacity: config.capacity,
            policy: config.policy,
            name: config.name.unwrap_or_else(|| "buffer".to_string()),
            buffer: VecDeque::with_capacity(config.capacity),
            rejected_count: 0,
            dropped_oldest_count: 0,
            dropped_newest_count: 0,
        }
    }

    /// Get the current buffer size
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Check if the buffer is full
    pub fn is_full(&self) -> bool {
        self.buffer.len() >= self.capacity
    }

    /// Get the buffer capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the number of rejected packets
    pub fn rejected_count(&self) -> u64 {
        self.rejected_count
    }

    /// Get the number of dropped oldest packets
    pub fn dropped_oldest_count(&self) -> u64 {
        self.dropped_oldest_count
    }

    /// Get the number of dropped newest packets
    pub fn dropped_newest_count(&self) -> u64 {
        self.dropped_newest_count
    }

    /// Get the total number of packets that didn't make it into the buffer
    pub fn total_lost_count(&self) -> u64 {
        self.rejected_count + self.dropped_oldest_count + self.dropped_newest_count
    }

    /// Pull a packet from the buffer (FIFO)
    pub fn pull(&mut self) -> Option<Packet> {
        self.buffer.pop_front()
    }

    /// Pull multiple packets from the buffer
    pub fn pull_many(&mut self, count: usize) -> Vec<Packet> {
        let mut result = Vec::with_capacity(count.min(self.buffer.len()));
        for _ in 0..count.min(self.buffer.len()) {
            if let Some(packet) = self.pull() {
                result.push(packet);
            }
        }
        result
    }

    /// Pull all packets from the buffer
    pub fn pull_all(&mut self) -> Vec<Packet> {
        self.pull_many(usize::MAX)
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl ProcessNode for BufferNode {
    fn process(&mut self, packet: Packet) -> Result<Option<Packet>> {
        // If buffer is not full, add the packet
        if self.buffer.len() < self.capacity {
            self.buffer.push_back(packet);
            return Ok(None);
        }

        // Buffer is full - apply policy
        match self.policy {
            BufferPolicy::Reject => {
                self.rejected_count += 1;
                Ok(None)
            }
            BufferPolicy::DropOldest => {
                self.buffer.pop_front();
                self.dropped_oldest_count += 1;
                self.buffer.push_back(packet);
                Ok(None)
            }
            BufferPolicy::DropNewest => {
                self.dropped_newest_count += 1;
                Ok(None)
            }
            BufferPolicy::Block => {
                Err(caret_core::Error::internal("Buffer is full"))
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
    use caret_sched::{ProcessingContext, NodeId};

    fn make_ctx() -> ProcessingContext {
        ProcessingContext::new(NodeId::new(1), 1)
    }

    fn make_packet(data: Vec<u8>) -> Packet {
        Packet::bytes(data)
    }

    #[test]
    fn test_buffer_node_creation() {
        let mut node = BufferNode::new(BufferNodeConfig::new(10));
        assert_eq!(node.capacity(), 10);
        assert!(node.is_empty());
        assert!(!node.is_full());
        assert_eq!(node.name(), "buffer");
    }

    #[test]
    fn test_buffer_accumulates() {
        let mut node = BufferNode::new(BufferNodeConfig::new(3));

        for i in 0..3 {
            let packet = make_packet(vec![i]);
            node.process(packet).unwrap();
        }

        assert_eq!(node.len(), 3);
        assert!(node.is_full());
    }

    #[test]
    fn test_buffer_pull() {
        let mut node = BufferNode::new(BufferNodeConfig::new(5));

        let p1 = make_packet(vec![1]);
        let p2 = make_packet(vec![2]);
        let p3 = make_packet(vec![3]);

        node.process(p1).unwrap();
        node.process(p2).unwrap();
        node.process(p3).unwrap();

        assert_eq!(node.len(), 3);

        let out = node.pull().unwrap();
        assert_eq!(out.data()[0], 1);
        assert_eq!(node.len(), 2);

        let out = node.pull().unwrap();
        assert_eq!(out.data()[0], 2);
        assert_eq!(node.len(), 1);
    }

    #[test]
    fn test_buffer_reject_policy() {
        let mut node = BufferNode::new(
            BufferNodeConfig::new(2)
                .with_policy(BufferPolicy::Reject)
        );

        let ctx = make_ctx();

        node.process(make_packet(vec![1])).unwrap();
        node.process(make_packet(vec![2])).unwrap();
        node.process(make_packet(vec![3])).unwrap();

        assert_eq!(node.len(), 2);
        assert_eq!(node.rejected_count(), 1);
    }

    #[test]
    fn test_buffer_drop_oldest_policy() {
        let mut node = BufferNode::new(
            BufferNodeConfig::new(2)
                .with_policy(BufferPolicy::DropOldest)
        );

        let ctx = make_ctx();

        node.process(make_packet(vec![1])).unwrap();
        node.process(make_packet(vec![2])).unwrap();
        node.process(make_packet(vec![3])).unwrap();

        assert_eq!(node.len(), 2);
        assert_eq!(node.dropped_oldest_count(), 1);

        // First packet was dropped
        let out = node.pull().unwrap();
        assert_eq!(out.data()[0], 2);
    }

    #[test]
    fn test_buffer_drop_newest_policy() {
        let mut node = BufferNode::new(
            BufferNodeConfig::new(2)
                .with_policy(BufferPolicy::DropNewest)
        );

        let ctx = make_ctx();

        node.process(make_packet(vec![1])).unwrap();
        node.process(make_packet(vec![2])).unwrap();
        node.process(make_packet(vec![3])).unwrap();

        assert_eq!(node.len(), 2);
        assert_eq!(node.dropped_newest_count(), 1);

        // Newest packet was dropped
        let out = node.pull().unwrap();
        assert_eq!(out.data()[0], 1);
    }

    #[test]
    fn test_buffer_block_policy() {
        let mut node = BufferNode::new(
            BufferNodeConfig::new(2)
                .with_policy(BufferPolicy::Block)
        );

        let ctx = make_ctx();

        node.process(make_packet(vec![1])).unwrap();
        node.process(make_packet(vec![2])).unwrap();

        // This should return an error
        let result = node.process(make_packet(vec![3]));
        assert!(result.is_err());
    }

    #[test]
    fn test_buffer_pull_many() {
        let mut node = BufferNode::new(BufferNodeConfig::new(5));

        for i in 0..5 {
            node.process(make_packet(vec![i])).unwrap();
        }

        let packets = node.pull_many(3);
        assert_eq!(packets.len(), 3);
        assert_eq!(packets[0].data()[0], 0);
        assert_eq!(packets[1].data()[0], 1);
        assert_eq!(packets[2].data()[0], 2);
        assert_eq!(node.len(), 2);
    }

    #[test]
    fn test_buffer_pull_all() {
        let mut node = BufferNode::new(BufferNodeConfig::new(5));

        for i in 0..3 {
            node.process(make_packet(vec![i])).unwrap();
        }

        let packets = node.pull_all();
        assert_eq!(packets.len(), 3);
        assert!(node.is_empty());
    }

    #[test]
    fn test_buffer_clear() {
        let mut node = BufferNode::new(BufferNodeConfig::new(5));

        for i in 0..3 {
            node.process(make_packet(vec![i])).unwrap();
        }

        assert_eq!(node.len(), 3);
        node.clear();
        assert!(node.is_empty());
    }

    #[test]
    fn test_buffer_with_name() {
        let mut node = BufferNode::new(
            BufferNodeConfig::new(5).with_name("my_buffer")
        );
        assert_eq!(node.name(), "my_buffer");
    }

    #[test]
    fn test_buffer_total_lost_count() {
        let mut node = BufferNode::new(
            BufferNodeConfig::new(2)
                .with_policy(BufferPolicy::Reject)
        );

        let ctx = make_ctx();

        node.process(make_packet(vec![1])).unwrap();
        node.process(make_packet(vec![2])).unwrap();
        node.process(make_packet(vec![3])).unwrap();
        node.process(make_packet(vec![4])).unwrap();

        assert_eq!(node.rejected_count(), 2);
        assert_eq!(node.total_lost_count(), 2);
    }

    #[test]
    fn test_buffer_fifo_order() {
        let mut node = BufferNode::new(BufferNodeConfig::new(5));

        for i in 1..=5 {
            node.process(make_packet(vec![i])).unwrap();
        }

        for i in 1..=5 {
            let packet = node.pull().unwrap();
            assert_eq!(packet.data()[0], i);
        }
    }
}
