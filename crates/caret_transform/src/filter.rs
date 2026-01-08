// Caret Transform - Filter node
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use caret_core::{Packet, PacketKind, Result};
use caret_io::ProcessNode;
use std::sync::Arc;

/// A predicate function for filtering packets
///
/// Returns true if the packet should pass through, false if it should be dropped.
pub type FilterPredicate = Arc<dyn Fn(&Packet) -> bool + Send + Sync>;

/// Configuration for a filter node
pub struct FilterNodeConfig {
    /// The filter predicate
    pub predicate: FilterPredicate,
    /// Optional node name
    pub name: Option<String>,
}

impl FilterNodeConfig {
    /// Create a new filter node configuration
    pub fn new(predicate: FilterPredicate) -> Self {
        Self {
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

/// A filter node that drops packets based on a predicate
///
/// The filter node evaluates each incoming packet against a predicate.
/// If the predicate returns true, the packet passes through.
/// If false, the packet is dropped (returns None).
pub struct FilterNode {
    predicate: FilterPredicate,
    name: String,
    pass_count: u64,
    drop_count: u64,
}

impl FilterNode {
    /// Create a new filter node
    pub fn new(config: FilterNodeConfig) -> Self {
        Self {
            predicate: config.predicate,
            name: config.name.unwrap_or_else(|| "filter".to_string()),
            pass_count: 0,
            drop_count: 0,
        }
    }

    /// Get the number of packets that passed through
    pub fn pass_count(&self) -> u64 {
        self.pass_count
    }

    /// Get the number of packets that were dropped
    pub fn drop_count(&self) -> u64 {
        self.drop_count
    }

    /// Get the total number of packets processed
    pub fn total_count(&self) -> u64 {
        self.pass_count + self.drop_count
    }
}

impl ProcessNode for FilterNode {
    fn process(&mut self, packet: Packet) -> Result<Option<Packet>> {
        if (self.predicate)(&packet) {
            self.pass_count += 1;
            Ok(Some(packet))
        } else {
            self.drop_count += 1;
            Ok(None)
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Common filter predicates
impl FilterNode {
    /// Create a predicate that filters by packet kind
    pub fn by_kind(kind: PacketKind) -> FilterPredicate {
        Arc::new(move |packet: &Packet| packet.kind() == &kind)
    }

    /// Create a predicate that filters by minimum data length
    pub fn min_length(min_len: usize) -> FilterPredicate {
        Arc::new(move |packet: &Packet| packet.len() >= min_len)
    }

    /// Create a predicate that filters by maximum data length
    pub fn max_length(max_len: usize) -> FilterPredicate {
        Arc::new(move |packet: &Packet| packet.len() <= max_len)
    }

    /// Create a predicate that filters by data length range
    pub fn length_range(min: usize, max: usize) -> FilterPredicate {
        Arc::new(move |packet: &Packet| {
            let len = packet.len();
            len >= min && len <= max
        })
    }

    /// Create a predicate that filters out control packets
    pub fn no_control() -> FilterPredicate {
        Arc::new(|packet: &Packet| !packet.is_control())
    }

    /// Create a predicate that filters out event packets
    pub fn no_events() -> FilterPredicate {
        Arc::new(|packet: &Packet| !packet.is_event())
    }

    /// Create a predicate that only passes control packets
    pub fn only_control() -> FilterPredicate {
        Arc::new(|packet: &Packet| packet.is_control())
    }

    /// Create a predicate that only passes event packets
    pub fn only_events() -> FilterPredicate {
        Arc::new(|packet: &Packet| packet.is_event())
    }

    /// Create a predicate that filters by a custom function
    pub fn custom<F>(f: F) -> FilterPredicate
    where
        F: Fn(&Packet) -> bool + Send + Sync + 'static,
    {
        Arc::new(f)
    }

    /// Compose two predicates with logical AND
    pub fn and(p1: FilterPredicate, p2: FilterPredicate) -> FilterPredicate {
        Arc::new(move |packet: &Packet| (p1)(packet) && (p2)(packet))
    }

    /// Compose two predicates with logical OR
    pub fn or(p1: FilterPredicate, p2: FilterPredicate) -> FilterPredicate {
        Arc::new(move |packet: &Packet| (p1)(packet) || (p2)(packet))
    }

    /// Negate a predicate
    pub fn not(p: FilterPredicate) -> FilterPredicate {
        Arc::new(move |packet: &Packet| !(p)(packet))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_kind() {
        let mut node = FilterNode::new(FilterNodeConfig::new(FilterNode::by_kind(PacketKind::Bytes)));

        assert_eq!(node.name(), "filter");
        assert_eq!(node.pass_count(), 0);
        assert_eq!(node.drop_count(), 0);

        let bytes_packet = Packet::bytes(&b"data"[..]);
        let event_packet = Packet::event(caret_core::EventKind::Start);

        let result = node.process(bytes_packet).unwrap();
        assert!(result.is_some());
        assert_eq!(node.pass_count(), 1);
        assert_eq!(node.drop_count(), 0);

        let result = node.process(event_packet).unwrap();
        assert!(result.is_none());
        assert_eq!(node.pass_count(), 1);
        assert_eq!(node.drop_count(), 1);
    }

    #[test]
    fn test_filter_min_length() {
        let mut node = FilterNode::new(FilterNodeConfig::new(FilterNode::min_length(10)));

        let small_packet = Packet::bytes(&b"hello"[..]);
        let large_packet = Packet::bytes(&b"hello world!"[..]);

        assert!(node.process(small_packet).unwrap().is_none());
        assert!(node.process(large_packet).unwrap().is_some());
    }

    #[test]
    fn test_filter_no_control() {
        let mut node = FilterNode::new(FilterNodeConfig::new(FilterNode::no_control()));

        let data_packet = Packet::bytes(&b"data"[..]);
        let control_packet = Packet::control(caret_core::ControlKind::Pause);

        assert!(node.process(data_packet).unwrap().is_some());
        assert!(node.process(control_packet).unwrap().is_none());
    }

    #[test]
    fn test_filter_and() {
        let p1 = FilterNode::min_length(5);
        let p2 = FilterNode::max_length(10);
        let combined = FilterNode::and(p1, p2);

        let mut node = FilterNode::new(FilterNodeConfig::new(combined));

        let too_small = Packet::bytes(&b"hi"[..]);
        let just_right = Packet::bytes(&b"hello"[..]);
        let too_big = Packet::bytes(&b"hello world!!"[..]);

        assert!(node.process(too_small).unwrap().is_none());
        assert!(node.process(just_right).unwrap().is_some());
        assert!(node.process(too_big).unwrap().is_none());
    }

    #[test]
    fn test_filter_or() {
        let p1 = FilterNode::only_control();
        let p2 = FilterNode::only_events();
        let combined = FilterNode::or(p1, p2);

        let mut node = FilterNode::new(FilterNodeConfig::new(combined));

        let data = Packet::bytes(&b"data"[..]);
        let control = Packet::control(caret_core::ControlKind::Pause);
        let event = Packet::event(caret_core::EventKind::Start);

        assert!(node.process(data).unwrap().is_none());
        assert!(node.process(control).unwrap().is_some());
        assert!(node.process(event).unwrap().is_some());
    }

    #[test]
    fn test_filter_not() {
        let p = FilterNode::only_control();
        let negated = FilterNode::not(p);

        let mut node = FilterNode::new(FilterNodeConfig::new(negated));

        let data = Packet::bytes(&b"data"[..]);
        let control = Packet::control(caret_core::ControlKind::Pause);

        assert!(node.process(data).unwrap().is_some());
        assert!(node.process(control).unwrap().is_none());
    }

    #[test]
    fn test_filter_custom() {
        // Only pass packets with even-length data
        let mut node = FilterNode::new(FilterNodeConfig::new(FilterNode::custom(|packet| {
            packet.len() % 2 == 0
        })));

        let odd_len = Packet::bytes(&b"hello"[..]); // 5 bytes
        let even_len = Packet::bytes(&b"hello!"[..]); // 6 bytes

        assert!(node.process(odd_len).unwrap().is_none());
        assert!(node.process(even_len).unwrap().is_some());
    }

    #[test]
    fn test_filter_with_name() {
        let mut node = FilterNode::new(
            FilterNodeConfig::new(FilterNode::no_control()).with_name("my_filter"),
        );
        assert_eq!(node.name(), "my_filter");
    }

    #[test]
    fn test_filter_counts() {
        let mut node = FilterNode::new(FilterNodeConfig::new(FilterNode::min_length(5)));

        for _ in 0..3 {
            let packet = Packet::bytes(&b"hello"[..]);
            node.process(packet).unwrap();
        }
        for _ in 0..2 {
            let packet = Packet::bytes(&b"hi"[..]);
            node.process(packet).unwrap();
        }

        assert_eq!(node.pass_count(), 3);
        assert_eq!(node.drop_count(), 2);
        assert_eq!(node.total_count(), 5);
    }
}
