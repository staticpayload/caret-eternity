// Caret IO - Base node traits
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use caret_core::{Packet, Result};
use caret_sched::{NodeProcessor, ProcessingContext, ProcessingResult};

/// A source node that generates packets
pub trait SourceNode: Send + Sync {
    /// Generate the next packet
    ///
    /// Returns None when the source is exhausted.
    fn generate(&mut self) -> Result<Option<Packet>>;
}

/// Adapter that wraps a SourceNode into a NodeProcessor
pub struct SourceNodeAdapter<T> {
    inner: T,
    name: String,
    output: Option<Packet>,
}

impl<T: SourceNode> SourceNodeAdapter<T> {
    /// Create a new adapter
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            name: "source".to_string(),
            output: None,
        }
    }

    /// Set the node name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Get the generated packet (for wiring to output ports)
    pub fn take_output(&mut self) -> Option<Packet> {
        self.output.take()
    }
}

impl<T: SourceNode> NodeProcessor for SourceNodeAdapter<T> {
    fn process(
        &mut self,
        _ctx: &ProcessingContext,
        _packet: Packet,
        _port: &str,
    ) -> Result<ProcessingResult> {
        self.output = self.inner.generate()?;
        if self.output.is_some() {
            Ok(ProcessingResult::Continue)
        } else {
            Ok(ProcessingResult::Done)
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// A sink node that consumes packets
pub trait SinkNode: Send + Sync {
    /// Consume a packet
    fn consume(&mut self, packet: Packet) -> Result<()>;
}

/// Adapter that wraps a SinkNode into a NodeProcessor
pub struct SinkNodeAdapter<T> {
    inner: T,
    name: String,
}

impl<T: SinkNode> SinkNodeAdapter<T> {
    /// Create a new adapter
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            name: "sink".to_string(),
        }
    }

    /// Set the node name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

impl<T: SinkNode> NodeProcessor for SinkNodeAdapter<T> {
    fn process(
        &mut self,
        _ctx: &ProcessingContext,
        packet: Packet,
        _port: &str,
    ) -> Result<ProcessingResult> {
        self.inner.consume(packet)?;
        Ok(ProcessingResult::Continue)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// A processing node that transforms packets
pub trait ProcessNode: Send + Sync {
    /// Process a packet, returning a new packet
    ///
    /// Returns None if the packet should be dropped.
    fn process(&mut self, packet: Packet) -> Result<Option<Packet>>;

    /// Get the node name
    fn name(&self) -> &str {
        "process"
    }
}

/// Adapter that wraps a ProcessNode into a NodeProcessor
pub struct ProcessNodeAdapter<T> {
    inner: T,
    name: String,
    output: Option<Packet>,
}

impl<T: ProcessNode> ProcessNodeAdapter<T> {
    /// Create a new adapter
    pub fn new(inner: T) -> Self {
        let name = inner.name().to_string();
        Self {
            inner,
            name,
            output: None,
        }
    }

    /// Set the node name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Get the output packet (for wiring to output ports)
    pub fn take_output(&mut self) -> Option<Packet> {
        self.output.take()
    }
}

impl<T: ProcessNode> NodeProcessor for ProcessNodeAdapter<T> {
    fn process(
        &mut self,
        _ctx: &ProcessingContext,
        packet: Packet,
        _port: &str,
    ) -> Result<ProcessingResult> {
        self.output = self.inner.process(packet)?;
        Ok(ProcessingResult::Continue)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestSource {
        count: usize,
        max: usize,
    }

    impl SourceNode for TestSource {
        fn generate(&mut self) -> Result<Option<Packet>> {
            if self.count < self.max {
                self.count += 1;
                Ok(Some(Packet::bytes(&[][..])))
            } else {
                Ok(None)
            }
        }
    }

    #[test]
    fn test_source_node_adapter() {
        let source = TestSource { count: 0, max: 5 };
        let adapter = SourceNodeAdapter::new(source).with_name("test_source");
        assert_eq!(adapter.name(), "test_source");
    }

    #[test]
    fn test_sink_node_adapter() {
        struct TestSink;
        impl SinkNode for TestSink {
            fn consume(&mut self, _packet: Packet) -> Result<()> {
                Ok(())
            }
        }
        let adapter = SinkNodeAdapter::new(TestSink).with_name("test_sink");
        assert_eq!(adapter.name(), "test_sink");
    }
}
