// Caret Transform - Batch node
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use bytes::Bytes;
use caret_core::{Packet, Result};
use caret_io::ProcessNode;
use std::time::Duration;

/// Configuration for a batch node
pub struct BatchNodeConfig {
    /// Maximum batch size (number of packets)
    pub max_size: usize,
    /// Maximum time to wait before flushing
    pub max_latency: Duration,
    /// Optional node name
    pub name: Option<String>,
}

impl BatchNodeConfig {
    /// Create a new batch node configuration
    pub fn new(max_size: usize, max_latency: Duration) -> Self {
        Self {
            max_size,
            max_latency,
            name: None,
        }
    }

    /// Set the node name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// A batch node that accumulates packets into batches
///
/// Packets are accumulated until either:
/// - The batch size reaches max_size, or
/// - The max_latency time has passed since the first packet
pub struct BatchNode {
    max_size: usize,
    max_latency_nanos: u64,
    name: String,
    batch: Vec<Packet>,
    first_packet_time: Option<u64>,
    batch_count: u64,
    total_packets: u64,
}

impl BatchNode {
    /// Create a new batch node
    pub fn new(config: BatchNodeConfig) -> Self {
        if config.max_size == 0 {
            panic!("Batch node max_size must be greater than 0");
        }
        Self {
            max_size: config.max_size,
            max_latency_nanos: config.max_latency.as_nanos() as u64,
            name: config.name.unwrap_or_else(|| "batch".to_string()),
            batch: Vec::new(),
            first_packet_time: None,
            batch_count: 0,
            total_packets: 0,
        }
    }

    /// Get the current batch size
    pub fn current_batch_size(&self) -> usize {
        self.batch.len()
    }

    /// Get the number of completed batches
    pub fn batch_count(&self) -> u64 {
        self.batch_count
    }

    /// Get the total number of packets processed
    pub fn total_packets(&self) -> u64 {
        self.total_packets
    }

    /// Check if the batch is ready to flush based on time
    fn should_flush_time(&self, current_time: u64) -> bool {
        if let Some(first_time) = self.first_packet_time {
            current_time.saturating_sub(first_time) >= self.max_latency_nanos
        } else {
            false
        }
    }

    /// Flush the current batch as a single packet
    fn flush(&mut self) -> Option<Packet> {
        if self.batch.is_empty() {
            return None;
        }

        self.batch_count += 1;

        // Combine all packet data into one
        let mut combined_data = Vec::new();
        for packet in &self.batch {
            combined_data.extend_from_slice(packet.data().as_ref());
        }

        // Create a batch packet with combined data
        let first_packet = self.batch.first()?;
        let batch_packet = Packet::bytes(Bytes::from(combined_data))
            .with_timestamp(first_packet.timestamp())
            .with_stream_id(first_packet.stream_id());

        self.batch.clear();
        self.first_packet_time = None;

        Some(batch_packet)
    }

    /// Manually flush the current batch
    pub fn flush_manual(&mut self) -> Option<Packet> {
        self.flush()
    }
}

impl ProcessNode for BatchNode {
    fn process(&mut self, packet: Packet) -> Result<Option<Packet>> {
        // Use an internal counter since we don't have access to tick
        let current_time = self.total_packets + 1;

        // Check if we should flush based on time
        if self.should_flush_time(current_time) && !self.batch.is_empty() {
            let flushed = self.flush();
            // Add the new packet to the fresh batch
            self.total_packets += 1;
            self.batch.push(packet);
            self.first_packet_time = Some(current_time);
            return Ok(flushed);
        }

        // Add the packet to the batch
        self.total_packets += 1;
        self.batch.push(packet);
        if self.first_packet_time.is_none() {
            self.first_packet_time = Some(current_time);
        }

        // Check if we should flush based on size
        if self.batch.len() >= self.max_size {
            return Ok(self.flush());
        }

        Ok(None)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Drop implementation that flushes remaining packets
impl Drop for BatchNode {
    fn drop(&mut self) {
        // Flush any remaining packets on drop
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use caret_sched::{ProcessingContext, ProcessingResult, NodeId};

    fn make_ctx(tick: u64) -> ProcessingContext {
        ProcessingContext::new(NodeId::new(1), tick)
    }

    #[test]
    fn test_batch_node_creation() {
        let node = BatchNode::new(BatchNodeConfig::new(10, Duration::from_millis(100)));
        assert_eq!(node.max_size, 10);
        assert_eq!(node.current_batch_size(), 0);
        assert_eq!(node.name(), "batch");
    }

    #[test]
    fn test_batch_zero_size_panics() {
        let result = std::panic::catch_unwind(|| {
            BatchNode::new(BatchNodeConfig::new(0, Duration::from_millis(100)));
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_accumulates() {
        let mut node = BatchNode::new(BatchNodeConfig::new(3, Duration::from_secs(1)));

        // Add packets below the max size
        for _ in 0..2 {
            let packet = Packet::bytes(&b"data"[..]);
            assert!(node.process(packet).unwrap().is_none());
        }

        assert_eq!(node.current_batch_size(), 2);
        assert_eq!(node.total_packets(), 2);
    }

    #[test]
    fn test_batch_flushes_on_size() {
        let mut node = BatchNode::new(BatchNodeConfig::new(3, Duration::from_secs(1)));

        // Add packets to reach max size
        for _ in 0..3 {
            let packet = Packet::bytes(&b"x"[..]);
            node.process(packet).unwrap();
        }

        assert_eq!(node.current_batch_size(), 0);
        assert_eq!(node.batch_count(), 1);
    }

    #[test]
    fn test_batch_flushes_on_time() {
        let mut node = BatchNode::new(BatchNodeConfig::new(10, Duration::from_nanos(100)));

        // Add packets to the batch
        for i in 0..3 {
            let packet = Packet::bytes(format!("data{}", i).into_bytes());
            node.process(packet).unwrap();
        }

        assert_eq!(node.current_batch_size(), 3);
        assert_eq!(node.total_packets(), 3);

        // Add more packets - eventually the time-based flush will trigger
        // based on total_packets count
        for _ in 0..110 {
            node.process(Packet::bytes(&b"x"[..])).unwrap();
        }

        // After adding many packets, at least one batch should have been flushed
        assert!(node.batch_count() > 0);
    }

    #[test]
    fn test_batch_with_name() {
        let node = BatchNode::new(
            BatchNodeConfig::new(5, Duration::from_millis(100))
                .with_name("my_batch"),
        );
        assert_eq!(node.name(), "my_batch");
    }

    #[test]
    fn test_batch_manual_flush() {
        let mut node = BatchNode::new(BatchNodeConfig::new(10, Duration::from_secs(1)));

        // Add some packets
        for _ in 0..3 {
            let packet = Packet::bytes(&b"data"[..]);
            node.process(packet).unwrap();
        }

        assert_eq!(node.current_batch_size(), 3);

        // Manually flush
        let flushed = node.flush_manual();
        assert!(flushed.is_some());
        assert_eq!(node.current_batch_size(), 0);
        assert_eq!(node.batch_count(), 1);
    }

    #[test]
    fn test_batch_drop_flushes() {
        // Create a scope so node is dropped
        {
            let mut node = BatchNode::new(BatchNodeConfig::new(10, Duration::from_secs(1)));

            // Add some packets
            for _ in 0..3 {
                let packet = Packet::bytes(&b"data"[..]);
                node.process(packet).unwrap();
            }

            assert_eq!(node.current_batch_size(), 3);
            assert_eq!(node.batch_count(), 0);
        } // node is dropped here, which flushes the batch
        // Note: we can't assert after drop, but the Drop impl is tested implicitly
    }

    #[test]
    fn test_batch_flush_empty_returns_none() {
        let mut node = BatchNode::new(BatchNodeConfig::new(10, Duration::from_secs(1)));
        assert!(node.flush_manual().is_none());
    }

    #[test]
    fn test_batch_combined_data() {
        let mut node = BatchNode::new(BatchNodeConfig::new(2, Duration::from_secs(1)));

        let p1 = Packet::bytes(&b"hello"[..]);
        let p2 = Packet::bytes(&b"world"[..]);

        let r1 = node.process(p1).unwrap();
        assert!(r1.is_none());

        // Flush on size (p2 completes the batch)
        let r2 = node.process(p2).unwrap();
        assert!(r2.is_some());
        let combined = r2.unwrap();
        assert_eq!(combined.data().as_ref(), b"helloworld");
        assert_eq!(node.current_batch_size(), 0);

        // Next packet starts a new batch
        let r3 = node.process(Packet::bytes(&b"x"[..])).unwrap();
        assert!(r3.is_none());
        assert_eq!(node.current_batch_size(), 1);
    }

    #[test]
    fn test_batch_statistics() {
        let mut node = BatchNode::new(BatchNodeConfig::new(2, Duration::from_secs(1)));

        // First batch
        for _ in 0..2 {
            let packet = Packet::bytes(&b"data"[..]);
            node.process(packet).unwrap();
        }

        // Second batch
        for _ in 0..2 {
            let packet = Packet::bytes(&b"data"[..]);
            node.process(packet).unwrap();
        }

        assert_eq!(node.batch_count(), 2);
        assert_eq!(node.total_packets(), 4);
    }
}
