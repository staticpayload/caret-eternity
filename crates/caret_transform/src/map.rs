// Caret Transform - Map node
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use bytes::Bytes;
use caret_core::{Packet, Result};
use caret_io::ProcessNode;
use std::sync::Arc;

/// A function that transforms a packet into another packet
///
/// Returns None if the packet should be dropped instead of transformed.
pub type MapFunction = Arc<dyn Fn(Packet) -> Option<Packet> + Send + Sync>;

/// Configuration for a map node
pub struct MapNodeConfig {
    /// The map function
    pub function: MapFunction,
    /// Optional node name
    pub name: Option<String>,
}

impl MapNodeConfig {
    /// Create a new map node configuration
    pub fn new(function: MapFunction) -> Self {
        Self {
            function,
            name: None,
        }
    }

    /// Set the node name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// A map node that transforms packets
///
/// The map node applies a function to each incoming packet,
/// transforming it into a new packet or dropping it.
pub struct MapNode {
    function: MapFunction,
    name: String,
    transform_count: u64,
    drop_count: u64,
}

impl MapNode {
    /// Create a new map node
    pub fn new(config: MapNodeConfig) -> Self {
        Self {
            function: config.function,
            name: config.name.unwrap_or_else(|| "map".to_string()),
            transform_count: 0,
            drop_count: 0,
        }
    }

    /// Get the number of packets that were transformed
    pub fn transform_count(&self) -> u64 {
        self.transform_count
    }

    /// Get the number of packets that were dropped
    pub fn drop_count(&self) -> u64 {
        self.drop_count
    }

    /// Get the total number of packets processed
    pub fn total_count(&self) -> u64 {
        self.transform_count + self.drop_count
    }
}

impl ProcessNode for MapNode {
    fn process(&mut self, packet: Packet) -> Result<Option<Packet>> {
        match (self.function)(packet) {
            Some(p) => {
                self.transform_count += 1;
                Ok(Some(p))
            }
            None => {
                self.drop_count += 1;
                Ok(None)
            }
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Common map functions
impl MapNode {
    /// Create a map function that transforms the packet data
    pub fn transform_data<F>(f: F) -> MapFunction
    where
        F: Fn(Bytes) -> Bytes + Send + Sync + 'static,
    {
        Arc::new(move |packet: Packet| {
            let new_data = f(packet.data().clone());
            // Use with_timestamp/with_metadata to create a new packet with transformed data
            let mut result = Packet::bytes(new_data);
            result = result.with_timestamp(packet.timestamp());
            result = result.with_stream_id(packet.stream_id());
            Some(result)
        })
    }

    /// Create a map function that adds a prefix to the data
    pub fn add_prefix(prefix: Vec<u8>) -> MapFunction {
        Arc::new(move |packet: Packet| {
            let mut data = prefix.clone();
            data.extend_from_slice(packet.data().as_ref());
            let mut result = Packet::bytes(data);
            result = result.with_timestamp(packet.timestamp());
            result = result.with_stream_id(packet.stream_id());
            Some(result)
        })
    }

    /// Create a map function that adds a suffix to the data
    pub fn add_suffix(suffix: Vec<u8>) -> MapFunction {
        Arc::new(move |packet: Packet| {
            let mut data = packet.data().to_vec();
            data.extend_from_slice(&suffix);
            let mut result = Packet::bytes(data);
            result = result.with_timestamp(packet.timestamp());
            result = result.with_stream_id(packet.stream_id());
            Some(result)
        })
    }

    /// Create a map function that truncates data to a maximum length
    pub fn truncate(max_len: usize) -> MapFunction {
        Arc::new(move |packet: Packet| {
            let data = if packet.data().len() > max_len {
                packet.data().slice(..max_len)
            } else {
                packet.data().clone()
            };
            let mut result = Packet::bytes(data);
            result = result.with_timestamp(packet.timestamp());
            result = result.with_stream_id(packet.stream_id());
            Some(result)
        })
    }

    /// Create a map function that transforms bytes as UTF-8 strings
    ///
    /// Returns None if the data is not valid UTF-8.
    pub fn transform_str<F>(f: F) -> MapFunction
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        Arc::new(move |packet: Packet| {
            let original = std::str::from_utf8(packet.data().as_ref()).ok()?;
            let transformed = f(original);
            let mut result = Packet::bytes(transformed);
            result = result.with_timestamp(packet.timestamp());
            result = result.with_stream_id(packet.stream_id());
            Some(result)
        })
    }

    /// Create a map function that uppercases UTF-8 string data
    pub fn uppercase() -> MapFunction {
        Self::transform_str(|s| s.to_uppercase())
    }

    /// Create a map function that lowercases UTF-8 string data
    pub fn lowercase() -> MapFunction {
        Self::transform_str(|s| s.to_lowercase())
    }

    /// Create a map function that reverses UTF-8 string data
    pub fn reverse_str() -> MapFunction {
        Self::transform_str(|s| s.chars().rev().collect())
    }

    /// Create a map function that trims whitespace from UTF-8 string data
    pub fn trim_str() -> MapFunction {
        Self::transform_str(|s| s.trim().to_string())
    }

    /// Create a map function that updates the packet timestamp
    pub fn with_timestamp<F>(f: F) -> MapFunction
    where
        F: Fn(caret_core::Timestamp) -> caret_core::Timestamp + Send + Sync + 'static,
    {
        Arc::new(move |packet: Packet| {
            let new_ts = f(packet.timestamp());
            let mut result = Packet::bytes(packet.data().clone());
            result = result.with_timestamp(new_ts);
            result = result.with_stream_id(packet.stream_id());
            Some(result)
        })
    }

    /// Create a map function that sets a fixed timestamp on all packets
    pub fn set_timestamp(ts: caret_core::Timestamp) -> MapFunction {
        Self::with_timestamp(move |_| ts)
    }

    /// Create a map function that normalizes timestamps to start from zero
    pub fn normalize_timestamps(base_ts: caret_core::Timestamp) -> MapFunction {
        Self::with_timestamp(move |ts| {
            if ts.as_nanos() > base_ts.as_nanos() {
                caret_core::Timestamp::from_nanos(
                    ts.as_nanos().saturating_sub(base_ts.as_nanos())
                )
            } else {
                caret_core::Timestamp::from_nanos(0)
            }
        })
    }

    /// Create a map function that clones the packet (1-to-1 transform)
    pub fn clone_packet() -> MapFunction {
        Arc::new(|packet: Packet| Some(packet.clone()))
    }

    /// Compose two map functions
    pub fn chain(f1: MapFunction, f2: MapFunction) -> MapFunction {
        Arc::new(move |packet: Packet| {
            let p1 = (f1)(packet)?;
            (f2)(p1)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_transform_data() {
        let mut node = MapNode::new(MapNodeConfig::new(MapNode::transform_data(|data| {
            Bytes::from(format!(" transformed: {:?}", data))
        })));

        let input = Packet::bytes(&b"hello"[..]);
        let result = node.process(input).unwrap();
        assert!(result.is_some());
        assert_eq!(node.transform_count(), 1);
        assert_eq!(node.drop_count(), 0);
    }

    #[test]
    fn test_map_add_prefix() {
        let mut node = MapNode::new(MapNodeConfig::new(MapNode::add_prefix(b"prefix: ".to_vec())));

        let input = Packet::bytes(&b"data"[..]);
        let result = node.process(input).unwrap().unwrap();
        assert_eq!(result.data().as_ref(), b"prefix: data");
    }

    #[test]
    fn test_map_add_suffix() {
        let mut node = MapNode::new(MapNodeConfig::new(MapNode::add_suffix(b" :suffix".to_vec())));

        let input = Packet::bytes(&b"data"[..]);
        let result = node.process(input).unwrap().unwrap();
        assert_eq!(result.data().as_ref(), b"data :suffix");
    }

    #[test]
    fn test_map_truncate() {
        let mut node = MapNode::new(MapNodeConfig::new(MapNode::truncate(5)));

        let input = Packet::bytes(&b"hello world"[..]);
        let result = node.process(input).unwrap().unwrap();
        assert_eq!(result.data().as_ref(), b"hello");
    }

    #[test]
    fn test_map_truncate_short() {
        let mut node = MapNode::new(MapNodeConfig::new(MapNode::truncate(100)));

        let input = Packet::bytes(&b"hello"[..]);
        let result = node.process(input).unwrap().unwrap();
        assert_eq!(result.data().as_ref(), b"hello");
    }

    #[test]
    fn test_map_uppercase() {
        let mut node = MapNode::new(MapNodeConfig::new(MapNode::uppercase()));

        let input = Packet::bytes(&b"hello world"[..]);
        let result = node.process(input).unwrap().unwrap();
        assert_eq!(result.data().as_ref(), b"HELLO WORLD");
    }

    #[test]
    fn test_map_lowercase() {
        let mut node = MapNode::new(MapNodeConfig::new(MapNode::lowercase()));

        let input = Packet::bytes(&b"HELLO WORLD"[..]);
        let result = node.process(input).unwrap().unwrap();
        assert_eq!(result.data().as_ref(), b"hello world");
    }

    #[test]
    fn test_map_reverse_str() {
        let mut node = MapNode::new(MapNodeConfig::new(MapNode::reverse_str()));

        let input = Packet::bytes(&b"hello"[..]);
        let result = node.process(input).unwrap().unwrap();
        assert_eq!(result.data().as_ref(), b"olleh");
    }

    #[test]
    fn test_map_trim_str() {
        let mut node = MapNode::new(MapNodeConfig::new(MapNode::trim_str()));

        let input = Packet::bytes(&b"  hello  "[..]);
        let result = node.process(input).unwrap().unwrap();
        assert_eq!(result.data().as_ref(), b"hello");
    }

    #[test]
    fn test_map_invalid_utf8() {
        let mut node = MapNode::new(MapNodeConfig::new(MapNode::uppercase()));

        // Invalid UTF-8 should be dropped
        let input = Packet::bytes(&[0xFF, 0xFE, 0xFD][..]);
        let result = node.process(input).unwrap();
        assert!(result.is_none());
        assert_eq!(node.transform_count(), 0);
        assert_eq!(node.drop_count(), 1);
    }

    #[test]
    fn test_map_set_timestamp() {
        let ts = caret_core::Timestamp::from_secs(100);
        let mut node = MapNode::new(MapNodeConfig::new(MapNode::set_timestamp(ts)));

        let input = Packet::bytes(&b"data"[..]);
        let result = node.process(input).unwrap().unwrap();
        assert_eq!(result.timestamp(), ts);
    }

    #[test]
    fn test_map_chain() {
        let f1 = MapNode::add_prefix(b"1: ".to_vec());
        let f2 = MapNode::add_suffix(b" :2".to_vec());
        let chained = MapNode::chain(f1, f2);

        let mut node = MapNode::new(MapNodeConfig::new(chained));

        let input = Packet::bytes(&b"data"[..]);
        let result = node.process(input).unwrap().unwrap();
        assert_eq!(result.data().as_ref(), b"1: data :2");
    }

    #[test]
    fn test_map_drop() {
        // A function that drops all packets
        let drop_fn: MapFunction = Arc::new(|_packet| None);
        let mut node = MapNode::new(MapNodeConfig::new(drop_fn));

        let input = Packet::bytes(&b"data"[..]);
        let result = node.process(input).unwrap();
        assert!(result.is_none());
        assert_eq!(node.transform_count(), 0);
        assert_eq!(node.drop_count(), 1);
    }

    #[test]
    fn test_map_custom() {
        let custom_fn: MapFunction = Arc::new(|packet| {
            // Double the data
            let mut data = packet.data().to_vec();
            let original = data.clone();
            data.extend_from_slice(&original);
            let mut result = Packet::bytes(data);
            result = result.with_timestamp(packet.timestamp());
            result = result.with_stream_id(packet.stream_id());
            Some(result)
        });
        let mut node = MapNode::new(MapNodeConfig::new(custom_fn));

        let input = Packet::bytes(&b"hi"[..]);
        let result = node.process(input).unwrap().unwrap();
        assert_eq!(result.data().as_ref(), b"hihi");
    }

    #[test]
    fn test_map_with_name() {
        let mut node = MapNode::new(
            MapNodeConfig::new(MapNode::uppercase()).with_name("my_mapper"),
        );
        assert_eq!(node.name(), "my_mapper");
    }

    #[test]
    fn test_map_counts() {
        let mut node = MapNode::new(MapNodeConfig::new(MapNode::uppercase()));

        for _ in 0..3 {
            let packet = Packet::bytes(&b"hello"[..]);
            node.process(packet).unwrap();
        }
        // Invalid UTF-8 gets dropped
        for _ in 0..2 {
            let packet = Packet::bytes(&[0xFF][..]);
            node.process(packet).unwrap();
        }

        assert_eq!(node.transform_count(), 3);
        assert_eq!(node.drop_count(), 2);
        assert_eq!(node.total_count(), 5);
    }
}
