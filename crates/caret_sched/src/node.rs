// Caret Sched - Node processing
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use caret_core::{Packet, Result};
use std::fmt;

/// Unique identifier for a node instance in the runtime
///
/// This is different from the graph NodeId - this is the runtime
/// instance identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId {
    id: u64,
}

impl NodeId {
    /// Create a new node ID
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    /// Get the numeric ID
    pub fn as_u64(&self) -> u64 {
        self.id
    }

    /// Generate a unique node ID
    pub fn unique() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "runtime-node-{}", self.id)
    }
}

/// Result of processing a packet
///
/// Indicates what the node wants to do next.
#[derive(Clone, Debug)]
pub enum ProcessingResult {
    /// Successfully processed, continue with next packet
    Continue,
    /// Successfully processed, request reschedule
    Reschedule,
    /// Successfully processed, done (no more packets expected)
    Done,
    /// Node encountered an error
    Error(caret_core::Error),
}

impl ProcessingResult {
    /// Check if processing should continue
    pub fn should_continue(&self) -> bool {
        matches!(self, Self::Continue | Self::Reschedule)
    }

    /// Check if processing completed successfully
    pub fn is_success(&self) -> bool {
        !matches!(self, Self::Error(_))
    }
}

impl From<caret_core::Error> for ProcessingResult {
    fn from(err: caret_core::Error) -> Self {
        Self::Error(err)
    }
}

impl From<Result<()>> for ProcessingResult {
    fn from(result: Result<()>) -> Self {
        match result {
            Ok(()) => Self::Continue,
            Err(e) => Self::Error(e),
        }
    }
}

/// Context provided to nodes during processing
pub struct ProcessingContext {
    /// Node ID
    pub node_id: NodeId,
    /// Current tick number
    pub tick: u64,
    /// Output ports for sending packets
    pub outputs: std::collections::HashMap<String, crate::port::OutputPort>,
}

impl ProcessingContext {
    /// Create a new processing context
    pub fn new(
        node_id: NodeId,
        tick: u64,
        outputs: std::collections::HashMap<String, crate::port::OutputPort>,
    ) -> Self {
        Self {
            node_id,
            tick,
            outputs,
        }
    }

    /// Send a packet to an output port
    pub fn send(&self, port_name: &str, packet: Packet) -> Result<()> {
        if let Some(output) = self.outputs.get(port_name) {
            output.send(packet)
        } else {
            Err(caret_core::Error::graph(format!(
                "Output port '{}' not found",
                port_name
            )))
        }
    }
}

/// Trait for nodes that can process packets
///
/// Implement this trait to create custom processing nodes.
pub trait NodeProcessor: Send + Sync {
    /// Process a single packet
    ///
    /// The node receives a packet on an input port and returns
    /// the result of processing.
    fn process(
        &mut self,
        ctx: &ProcessingContext,
        packet: Packet,
        port: &str,
    ) -> Result<ProcessingResult>;

    /// Initialize the node
    ///
    /// Called once before any processing begins.
    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Cleanup the node
    ///
    /// Called once after processing ends.
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }

    /// Get the node name
    fn name(&self) -> &str {
        "unknown"
    }
}

/// A boxed node processor
pub type BoxedNodeProcessor = Box<dyn NodeProcessor>;

/// Helper for implementing simple pass-through nodes
pub struct PassthroughNode {
    name: String,
}

impl PassthroughNode {
    /// Create a new passthrough node
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl NodeProcessor for PassthroughNode {
    fn process(
        &mut self,
        _ctx: &ProcessingContext,
        _packet: Packet,
        _port: &str,
    ) -> Result<ProcessingResult> {
        Ok(ProcessingResult::Continue)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        let id1 = NodeId::new(42);
        assert_eq!(id1.as_u64(), 42);
        assert_eq!(format!("{}", id1), "runtime-node-42");

        let id2 = NodeId::unique();
        let id3 = NodeId::unique();
        assert_ne!(id2.as_u64(), id3.as_u64());
    }

    #[test]
    fn test_processing_result() {
        assert!(ProcessingResult::Continue.should_continue());
        assert!(ProcessingResult::Reschedule.should_continue());
        assert!(!ProcessingResult::Done.should_continue());
        assert!(!ProcessingResult::Error(caret_core::Error::internal("")).should_continue());

        assert!(ProcessingResult::Continue.is_success());
        assert!(ProcessingResult::Done.is_success());
        assert!(!ProcessingResult::Error(caret_core::Error::internal("")).is_success());
    }

    #[test]
    fn test_processing_context() {
        let node_id = NodeId::new(1);
        let ctx = ProcessingContext::new(node_id, 100, std::collections::HashMap::new());
        assert_eq!(ctx.node_id, node_id);
        assert_eq!(ctx.tick, 100);
    }

    #[test]
    fn test_passthrough_node() {
        let mut node = PassthroughNode::new("test");
        assert_eq!(node.name(), "test");

        let ctx = ProcessingContext::new(NodeId::new(1), 1, std::collections::HashMap::new());
        let packet = Packet::bytes(&[][..]);
        let result = node.process(&ctx, packet, "input");
        assert!(result.is_ok());
        assert!(result.unwrap().should_continue());
    }

    #[test]
    fn test_boxed_node_processor() {
        let node: BoxedNodeProcessor = Box::new(PassthroughNode::new("boxed"));
        assert_eq!(node.name(), "boxed");
    }
}
