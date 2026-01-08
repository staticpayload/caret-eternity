// Caret Transform - Demux (demultiplexer) node
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use caret_core::{Packet, Result};
use caret_io::ProcessNode;
use std::sync::Arc;

/// A predicate function that determines which output port a packet goes to
///
/// Returns the index of the output port, or None to drop the packet.
pub type DemuxPredicate = Arc<dyn Fn(&Packet) -> Option<usize> + Send + Sync>;

/// Configuration for a demux node
pub struct DemuxNodeConfig {
    /// Number of output ports
    pub outputs: usize,
    /// The demux predicate
    pub predicate: DemuxPredicate,
    /// Optional node name
    pub name: Option<String>,
}

impl DemuxNodeConfig {
    /// Create a new demux node configuration
    pub fn new(outputs: usize, predicate: DemuxPredicate) -> Self {
        Self {
            outputs,
            predicate,
            name: None,
        }
    }

    /// Set the node name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// A demux (demultiplexer) node that routes packets to different outputs
///
/// The demux node evaluates each incoming packet against a predicate
/// to determine which output port it should go to.
pub struct DemuxNode {
    outputs: usize,
    predicate: DemuxPredicate,
    name: String,
    /// Current selected output (for the next packet)
    current_output: Option<usize>,
    /// Count of packets routed to each output
    output_counts: Vec<u64>,
    drop_count: u64,
}

impl DemuxNode {
    /// Create a new demux node
    pub fn new(config: DemuxNodeConfig) -> Self {
        if config.outputs == 0 {
            panic!("Demux node requires at least one output");
        }
        let output_counts = vec![0; config.outputs];
        Self {
            outputs: config.outputs,
            predicate: config.predicate,
            name: config.name.unwrap_or_else(|| "demux".to_string()),
            current_output: None,
            output_counts,
            drop_count: 0,
        }
    }

    /// Get the number of output ports
    pub fn output_count(&self) -> usize {
        self.outputs
    }

    /// Get the selected output for the next packet
    pub fn current_output(&self) -> Option<usize> {
        self.current_output
    }

    /// Get the count of packets routed to each output
    pub fn output_counts(&self) -> &[u64] {
        &self.output_counts
    }

    /// Get the number of dropped packets
    pub fn drop_count(&self) -> u64 {
        self.drop_count
    }

    /// Get the total number of packets processed
    pub fn total_count(&self) -> u64 {
        self.output_counts.iter().sum::<u64>() + self.drop_count
    }

    /// Take the packet for wiring to an output port
    pub fn take_packet(&mut self, port: usize) -> Option<Packet> {
        if self.current_output == Some(port) {
            self.current_output = None;
            // Note: The actual packet storage would be handled by the runtime
            // This is a marker interface for the ProcessNodeAdapter
            None
        } else {
            None
        }
    }
}

impl ProcessNode for DemuxNode {
    fn process(&mut self, packet: Packet) -> Result<Option<Packet>> {
        match (self.predicate)(&packet) {
            Some(port_idx) if port_idx < self.outputs => {
                self.output_counts[port_idx] += 1;
                self.current_output = Some(port_idx);
                Ok(Some(packet))
            }
            Some(_) => {
                // Port index out of range, drop the packet
                self.drop_count += 1;
                self.current_output = None;
                Ok(None)
            }
            None => {
                // Predicate says drop
                self.drop_count += 1;
                self.current_output = None;
                Ok(None)
            }
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Common demux predicates
impl DemuxNode {
    /// Route packets in round-robin fashion
    pub fn round_robin() -> DemuxPredicate {
        Arc::new(|_packet| {
            // This can't be stateless; users should implement their own
            // or use RoundRobinDemuxNode
            None
        })
    }

    /// Route bytes packets to port 0, all others to port 1
    pub fn by_kind_bytes_vs_others() -> DemuxPredicate {
        Arc::new(move |packet: &Packet| {
            if matches!(packet.kind(), caret_core::PacketKind::Bytes) {
                Some(0)
            } else {
                Some(1)
            }
        })
    }

    /// Route packets based on data length
    ///
    /// Packets below the threshold go to port 0, others to port 1.
    pub fn by_length_threshold(threshold: usize) -> DemuxPredicate {
        Arc::new(move |packet: &Packet| {
            if packet.len() < threshold {
                Some(0)
            } else {
                Some(1)
            }
        })
    }

    /// Route packets based on a modulo of data length
    pub fn by_length_modulo(outputs: usize) -> DemuxPredicate {
        Arc::new(move |packet: &Packet| {
            if outputs == 0 {
                return None;
            }
            Some(packet.len() % outputs)
        })
    }

    /// Route packets based on custom function
    pub fn custom<F>(f: F) -> DemuxPredicate
    where
        F: Fn(&Packet) -> Option<usize> + Send + Sync + 'static,
    {
        Arc::new(f)
    }
}

/// A stateful round-robin demux node
///
/// This variant maintains internal state for round-robin routing.
pub struct RoundRobinDemuxNode {
    outputs: usize,
    current: usize,
    name: String,
    output_counts: Vec<u64>,
    current_output: Option<usize>,
}

impl RoundRobinDemuxNode {
    /// Create a new round-robin demux node
    pub fn new(outputs: usize) -> Self {
        if outputs == 0 {
            panic!("Demux node requires at least one output");
        }
        Self {
            outputs,
            current: 0,
            name: "round_robin_demux".to_string(),
            output_counts: vec![0; outputs],
            current_output: None,
        }
    }

    /// Create with a custom name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Get the number of output ports
    pub fn output_count(&self) -> usize {
        self.outputs
    }

    /// Get the count of packets routed to each output
    pub fn output_counts(&self) -> &[u64] {
        &self.output_counts
    }

    /// Get the current output port
    pub fn current_output(&self) -> Option<usize> {
        self.current_output
    }
}

impl ProcessNode for RoundRobinDemuxNode {
    fn process(&mut self, packet: Packet) -> Result<Option<Packet>> {
        let port_idx = self.current;
        self.current = (self.current + 1) % self.outputs;
        self.output_counts[port_idx] += 1;
        self.current_output = Some(port_idx);
        Ok(Some(packet))
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
    fn test_demux_node_creation() {
        let predicate = Arc::new(|_: &Packet| Some(0));
        let mut node = DemuxNode::new(DemuxNodeConfig::new(3, predicate));
        assert_eq!(node.output_count(), 3);
        assert_eq!(node.name(), "demux");
    }

    #[test]
    fn test_demux_zero_outputs_panics() {
        let predicate = Arc::new(|_: &Packet| Some(0));
        let result = std::panic::catch_unwind(|| {
            DemuxNode::new(DemuxNodeConfig::new(0, predicate));
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_demux_with_name() {
        let predicate = Arc::new(|_: &Packet| Some(0));
        let mut node = DemuxNode::new(DemuxNodeConfig::new(2, predicate).with_name("my_demux"));
        assert_eq!(node.name(), "my_demux");
    }

    #[test]
    fn test_demux_by_length_threshold() {
        let mut node = DemuxNode::new(DemuxNodeConfig::new(
            2,
            DemuxNode::by_length_threshold(10),
        ));
        let ctx = ProcessingContext::new(caret_sched::NodeId::new(1), 1);

        let small = Packet::bytes(&b"hello"[..]);
        let large = Packet::bytes(&b"hello world!!"[..]);

        node.process(small).unwrap();
        assert_eq!(node.current_output(), Some(0));
        assert_eq!(node.output_counts()[0], 1);
        assert_eq!(node.output_counts()[1], 0);

        node.process(large).unwrap();
        assert_eq!(node.current_output(), Some(1));
        assert_eq!(node.output_counts()[0], 1);
        assert_eq!(node.output_counts()[1], 1);
    }

    #[test]
    fn test_demux_by_length_modulo() {
        let mut node = DemuxNode::new(DemuxNodeConfig::new(3, DemuxNode::by_length_modulo(3)));
        let ctx = ProcessingContext::new(caret_sched::NodeId::new(1), 1);

        let p0 = Packet::bytes(&b"ab"[..]); // len 2 -> 2 % 3 = 2
        let p1 = Packet::bytes(&b"abc"[..]); // len 3 -> 3 % 3 = 0
        let p2 = Packet::bytes(&b"abcd"[..]); // len 4 -> 4 % 3 = 1

        node.process(p0).unwrap();
        assert_eq!(node.current_output(), Some(2));

        node.process(p1).unwrap();
        assert_eq!(node.current_output(), Some(0));

        node.process(p2).unwrap();
        assert_eq!(node.current_output(), Some(1));
    }

    #[test]
    fn test_demux_drop_on_out_of_range() {
        let mut node = DemuxNode::new(DemuxNodeConfig::new(2, Arc::new(|_: &Packet| Some(5))));
        let ctx = ProcessingContext::new(caret_sched::NodeId::new(1), 1);

        let packet = Packet::bytes(&b"data"[..]);
        let result = node.process(packet).unwrap();
        assert!(result.is_none());
        assert_eq!(node.drop_count(), 1);
        assert_eq!(node.total_count(), 1);
    }

    #[test]
    fn test_demux_drop_on_none() {
        let mut node = DemuxNode::new(DemuxNodeConfig::new(2, Arc::new(|_: &Packet| None)));
        let ctx = ProcessingContext::new(caret_sched::NodeId::new(1), 1);

        let packet = Packet::bytes(&b"data"[..]);
        let result = node.process(packet).unwrap();
        assert!(result.is_none());
        assert_eq!(node.drop_count(), 1);
    }

    #[test]
    fn test_round_robin_demux() {
        let mut node = RoundRobinDemuxNode::new(3).with_name("rr");
        let ctx = ProcessingContext::new(caret_sched::NodeId::new(1), 1);

        assert_eq!(node.name(), "rr");
        assert_eq!(node.output_count(), 3);

        for _ in 0..6 {
            let packet = Packet::bytes(&b"data"[..]);
            node.process(packet).unwrap();
        }

        assert_eq!(node.output_counts(), &[2, 2, 2]);
    }

    #[test]
    fn test_round_robin_demux_current_output() {
        let mut node = RoundRobinDemuxNode::new(2);
        let ctx = ProcessingContext::new(caret_sched::NodeId::new(1), 1);

        let packet = Packet::bytes(&b"data"[..]);
        node.process(packet).unwrap();
        assert_eq!(node.current_output(), Some(0));

        let packet = Packet::bytes(&b"data"[..]);
        node.process(packet).unwrap();
        assert_eq!(node.current_output(), Some(1));

        let packet = Packet::bytes(&b"data"[..]);
        node.process(packet).unwrap();
        assert_eq!(node.current_output(), Some(0)); // Wrapped around
    }

    #[test]
    fn test_round_robin_zero_outputs_panics() {
        let result = std::panic::catch_unwind(|| {
            RoundRobinDemuxNode::new(0);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_demux_custom_predicate() {
        let predicate = DemuxNode::custom(|packet| {
            // Route based on first byte
            if packet.data().is_empty() {
                None
            } else {
                Some(packet.data()[0] as usize)
            }
        });

        let mut node = DemuxNode::new(DemuxNodeConfig::new(5, predicate));
        let ctx = ProcessingContext::new(caret_sched::NodeId::new(1), 1);

        let p0 = Packet::bytes(&b"\x00data"[..]);
        let p2 = Packet::bytes(&b"\x02data"[..]);

        node.process(p0).unwrap();
        assert_eq!(node.current_output(), Some(0));

        node.process(p2).unwrap();
        assert_eq!(node.current_output(), Some(2));
    }
}
