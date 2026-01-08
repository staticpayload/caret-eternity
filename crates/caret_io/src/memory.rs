// Caret IO - Memory IO nodes
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::nodes::{SinkNode, SourceNode};
use caret_core::{Packet, Result};
use std::sync::Arc;

/// Configuration for a memory source
#[derive(Clone, Debug)]
pub struct MemorySourceConfig {
    /// Data to emit
    pub data: Vec<u8>,
    /// Chunk size for splitting data
    pub chunk_size: usize,
    /// Whether to loop the data
    pub loop_data: bool,
}

impl Default for MemorySourceConfig {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            chunk_size: 1024,
            loop_data: false,
        }
    }
}

impl MemorySourceConfig {
    /// Create a new memory source config
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }

    /// Set the chunk size
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Set whether to loop the data
    pub fn with_loop(mut self, loop_data: bool) -> Self {
        self.loop_data = loop_data;
        self
    }

    /// Create from a string
    pub fn from_string(s: impl Into<String>) -> Self {
        Self::new(s.into().into_bytes())
    }
}

/// A source node that reads from in-memory data
pub struct MemorySource {
    config: MemorySourceConfig,
    position: usize,
    exhausted: bool,
}

impl MemorySource {
    /// Create a new memory source
    pub fn new(config: MemorySourceConfig) -> Self {
        Self {
            config,
            position: 0,
            exhausted: false,
        }
    }

    /// Create from bytes
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self::new(MemorySourceConfig::new(data))
    }

    /// Create from a string
    pub fn from_string(s: impl Into<String>) -> Self {
        Self::new(MemorySourceConfig::from_string(s))
    }

    /// Reset to the beginning
    pub fn reset(&mut self) {
        self.position = 0;
        self.exhausted = false;
    }

    /// Get the remaining bytes
    pub fn remaining(&self) -> usize {
        self.config.data.len() - self.position
    }
}

impl SourceNode for MemorySource {
    fn generate(&mut self) -> Result<Option<Packet>> {
        if self.exhausted {
            if self.config.loop_data {
                self.reset();
            } else {
                return Ok(None);
            }
        }

        if self.config.data.is_empty() {
            self.exhausted = true;
            return Ok(None);
        }

        let remaining = self.config.data.len() - self.position;
        let chunk_size = self.config.chunk_size.min(remaining);

        if chunk_size == 0 {
            self.exhausted = true;
            return Ok(None);
        }

        let end = self.position + chunk_size;
        let chunk = self.config.data[self.position..end].to_vec();
        self.position = end;

        Ok(Some(Packet::bytes(chunk)))
    }
}

/// Configuration for a memory sink
#[derive(Clone, Debug, Default)]
pub struct MemorySinkConfig {
    /// Maximum buffer size (0 = unlimited)
    pub max_size: usize,
}

impl MemorySinkConfig {
    /// Create a new memory sink config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum buffer size
    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }
}

/// A sink node that collects data in memory
pub struct MemorySink {
    data: Arc<std::sync::Mutex<Vec<u8>>>,
    config: MemorySinkConfig,
    dropped_packets: Arc<std::sync::atomic::AtomicU64>,
}

impl MemorySink {
    /// Create a new memory sink
    pub fn new() -> Self {
        Self::with_config(MemorySinkConfig::default())
    }

    /// Create with a specific config
    pub fn with_config(config: MemorySinkConfig) -> Self {
        Self {
            data: Arc::new(std::sync::Mutex::new(Vec::new())),
            config,
            dropped_packets: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Get all collected data
    pub fn data(&self) -> Vec<u8> {
        self.data.lock().unwrap().clone()
    }

    /// Get the data as a string
    pub fn as_string(&self) -> Result<String> {
        let bytes = self.data.lock().unwrap();
        String::from_utf8(bytes.clone())
            .map_err(|e| caret_core::Error::invalid_input(format!("invalid UTF-8: {}", e)))
    }

    /// Get the length of collected data
    pub fn len(&self) -> usize {
        self.data.lock().unwrap().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.lock().unwrap().is_empty()
    }

    /// Clear collected data
    pub fn clear(&self) {
        self.data.lock().unwrap().clear();
    }

    /// Get the number of packets that were dropped due to buffer overflow
    pub fn dropped_packets(&self) -> u64 {
        self.dropped_packets.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Reset the dropped packet counter
    pub fn reset_dropped_counter(&self) {
        self.dropped_packets.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Default for MemorySink {
    fn default() -> Self {
        Self::new()
    }
}

impl SinkNode for MemorySink {
    fn consume(&mut self, packet: Packet) -> Result<()> {
        let mut data = self.data.lock().unwrap();

        // Check max size limit
        if self.config.max_size > 0 && data.len() >= self.config.max_size {
            self.dropped_packets
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Ok(()); // Drop the packet silently
        }

        data.extend_from_slice(packet.data());
        Ok(())
    }
}

impl Clone for MemorySink {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            config: self.config.clone(),
            dropped_packets: self.dropped_packets.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_source_config() {
        let config = MemorySourceConfig::new(vec![1, 2, 3, 4, 5])
            .with_chunk_size(2)
            .with_loop(true);
        assert_eq!(config.data, vec![1, 2, 3, 4, 5]);
        assert_eq!(config.chunk_size, 2);
        assert!(config.loop_data);
    }

    #[test]
    fn test_memory_source_from_string() {
        let source = MemorySource::from_string("hello");
        assert_eq!(source.config.data, b"hello");
    }

    #[test]
    fn test_memory_source_empty() {
        let mut source = MemorySource::new(MemorySourceConfig::new(vec![]));
        assert!(source.generate().unwrap().is_none());
    }

    #[test]
    fn test_memory_source_chunks() {
        let mut source = MemorySource::new(
            MemorySourceConfig::new(vec![1, 2, 3, 4, 5, 6]).with_chunk_size(2),
        );

        let p1 = source.generate().unwrap().unwrap();
        assert_eq!(p1.data().as_ref(), &[1, 2]);

        let p2 = source.generate().unwrap().unwrap();
        assert_eq!(p2.data().as_ref(), &[3, 4]);

        let p3 = source.generate().unwrap().unwrap();
        assert_eq!(p3.data().as_ref(), &[5, 6]);

        assert!(source.generate().unwrap().is_none());
        assert_eq!(source.remaining(), 0);
    }

    #[test]
    fn test_memory_source_loop() {
        let mut source = MemorySource::new(
            MemorySourceConfig::new(vec![1, 2, 3]).with_loop(true),
        );

        let p1 = source.generate().unwrap().unwrap();
        assert_eq!(p1.data().as_ref(), &[1, 2, 3]);

        assert!(source.generate().unwrap().is_none());

        let p2 = source.generate().unwrap().unwrap();
        assert_eq!(p2.data().as_ref(), &[1, 2, 3]);
    }

    #[test]
    fn test_memory_sink() {
        let mut sink = MemorySink::new();

        sink.consume(Packet::bytes(&b"hello "[..])).unwrap();
        sink.consume(Packet::bytes(&b"world"[..])).unwrap();

        assert_eq!(sink.data(), b"hello world");
        assert_eq!(sink.len(), 11);
        assert!(!sink.is_empty());
        assert_eq!(sink.as_string().unwrap(), "hello world");
    }

    #[test]
    fn test_memory_sink_clear() {
        let mut sink = MemorySink::new();

        sink.consume(Packet::bytes(&b"data"[..])).unwrap();
        assert_eq!(sink.len(), 4);

        sink.clear();
        assert!(sink.is_empty());
    }

    #[test]
    fn test_memory_sink_max_size() {
        let mut sink = MemorySink::with_config(
            MemorySinkConfig::new().with_max_size(5),
        );

        sink.consume(Packet::bytes(&b"hello "[..])).unwrap();
        assert_eq!(sink.len(), 6); // "hello " is 6 bytes

        sink.consume(Packet::bytes(&b"world"[..])).unwrap();
        assert_eq!(sink.len(), 6); // Still 6, packet dropped
        assert_eq!(sink.dropped_packets(), 1);
    }

    #[test]
    fn test_memory_sink_clone() {
        let mut sink = MemorySink::new();
        sink.consume(Packet::bytes(&b"data"[..])).unwrap();

        let sink2 = sink.clone();
        assert_eq!(sink2.data(), b"data");
    }
}
