// Caret Distributed - Node factory for distributed execution
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::graph_proto::{NodeType, SerializableNode};
use caret_sched::{NodeProcessor, ProcessingContext, ProcessingResult};
use caret_core::{Packet, Result};

/// Factory for creating node processors from serialized node definitions
pub struct NodeFactory;

impl NodeFactory {
    /// Create a node processor from a serialized node definition
    pub fn create_processor(node: &SerializableNode) -> Result<Box<dyn NodeProcessor>> {
        match &node.node_type {
            NodeType::Source => Ok(Box::new(SourceNode::new(&node.name))),
            NodeType::Transform => Ok(Box::new(TransformNode::new(&node.name))),
            NodeType::Sink => Ok(Box::new(SinkNode::new(&node.name))),
            NodeType::Custom(type_name) => {
                // For custom nodes, we'd look up registered node types
                // For now, use a generic passthrough
                Ok(Box::new(GenericNode::new(&node.name, type_name)))
            }
        }
    }

    /// Get the input ports that should be created for a node
    pub fn get_input_ports(node: &SerializableNode) -> Vec<(String, usize)> {
        node.inputs
            .iter()
            .map(|p| (p.name.clone(), 1024)) // Default capacity
            .collect()
    }

    /// Get the output ports that should be created for a node
    pub fn get_output_ports(node: &SerializableNode) -> Vec<String> {
        node.outputs.iter().map(|p| p.name.clone()).collect()
    }
}

/// A source node processor
///
/// Source nodes generate data and have only output ports.
pub struct SourceNode {
    name: String,
}

impl SourceNode {
    /// Create a new source node
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl NodeProcessor for SourceNode {
    fn process(
        &mut self,
        _ctx: &ProcessingContext,
        _packet: Packet,
        _port: &str,
    ) -> Result<ProcessingResult> {
        // Source nodes typically generate data, not process it
        // For now, just continue
        Ok(ProcessingResult::Continue)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn initialize(&mut self) -> Result<()> {
        tracing::debug!("Initializing source node: {}", self.name);
        Ok(())
    }
}

/// A transform node processor
///
/// Transform nodes process incoming data and produce output.
pub struct TransformNode {
    name: String,
}

impl TransformNode {
    /// Create a new transform node
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl NodeProcessor for TransformNode {
    fn process(
        &mut self,
        _ctx: &ProcessingContext,
        packet: Packet,
        _port: &str,
    ) -> Result<ProcessingResult> {
        // For now, just process the packet and continue
        // In a real implementation, this would transform the data
        tracing::trace!(
            "Transform node {} processing packet ({} bytes)",
            self.name,
            packet.len()
        );

        // The executor handles sending to output ports automatically
        Ok(ProcessingResult::Continue)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn initialize(&mut self) -> Result<()> {
        tracing::debug!("Initializing transform node: {}", self.name);
        Ok(())
    }
}

/// A sink node processor
///
/// Sink nodes consume data and have no output.
pub struct SinkNode {
    name: String,
}

impl SinkNode {
    /// Create a new sink node
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl NodeProcessor for SinkNode {
    fn process(
        &mut self,
        _ctx: &ProcessingContext,
        packet: Packet,
        _port: &str,
    ) -> Result<ProcessingResult> {
        // Consume the packet
        tracing::trace!(
            "Sink node {} consuming packet ({} bytes)",
            self.name,
            packet.len()
        );

        // Data is consumed and dropped
        drop(packet);

        Ok(ProcessingResult::Continue)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn initialize(&mut self) -> Result<()> {
        tracing::debug!("Initializing sink node: {}", self.name);
        Ok(())
    }
}

/// A generic node processor for custom node types
///
/// This provides a basic passthrough implementation for custom nodes.
pub struct GenericNode {
    name: String,
    type_name: String,
}

impl GenericNode {
    /// Create a new generic node
    pub fn new(name: &str, type_name: &str) -> Self {
        Self {
            name: name.to_string(),
            type_name: type_name.to_string(),
        }
    }
}

impl NodeProcessor for GenericNode {
    fn process(
        &mut self,
        _ctx: &ProcessingContext,
        packet: Packet,
        _port: &str,
    ) -> Result<ProcessingResult> {
        // Passthrough behavior
        tracing::trace!(
            "Generic node {} ({}) passing through packet ({} bytes)",
            self.name,
            self.type_name,
            packet.len()
        );

        Ok(ProcessingResult::Continue)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn initialize(&mut self) -> Result<()> {
        tracing::debug!(
            "Initializing generic node {} of type {}",
            self.name,
            self.type_name
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_proto::{SerializableNode, SerializablePort, PortDirection, NodeType};

    #[test]
    fn test_node_factory_source() {
        let node = SerializableNode {
            id: 1,
            name: "test_source".to_string(),
            node_type: NodeType::Source,
            inputs: vec![],
            outputs: vec![
                SerializablePort {
                    name: "data".to_string(),
                    id: 1,
                    direction: PortDirection::Out,
                }
            ],
        };

        let processor = NodeFactory::create_processor(&node).unwrap();
        assert_eq!(processor.name(), "test_source");

        let input_ports = NodeFactory::get_input_ports(&node);
        assert!(input_ports.is_empty());

        let output_ports = NodeFactory::get_output_ports(&node);
        assert_eq!(output_ports.len(), 1);
        assert_eq!(output_ports[0], "data");
    }

    #[test]
    fn test_node_factory_transform() {
        let node = SerializableNode {
            id: 2,
            name: "test_transform".to_string(),
            node_type: NodeType::Transform,
            inputs: vec![
                SerializablePort {
                    name: "input".to_string(),
                    id: 1,
                    direction: PortDirection::In,
                }
            ],
            outputs: vec![
                SerializablePort {
                    name: "output".to_string(),
                    id: 2,
                    direction: PortDirection::Out,
                }
            ],
        };

        let processor = NodeFactory::create_processor(&node).unwrap();
        assert_eq!(processor.name(), "test_transform");

        let input_ports = NodeFactory::get_input_ports(&node);
        assert_eq!(input_ports.len(), 1);
        assert_eq!(input_ports[0].0, "input");

        let output_ports = NodeFactory::get_output_ports(&node);
        assert_eq!(output_ports.len(), 1);
        assert_eq!(output_ports[0], "output");
    }

    #[test]
    fn test_node_factory_sink() {
        let node = SerializableNode {
            id: 3,
            name: "test_sink".to_string(),
            node_type: NodeType::Sink,
            inputs: vec![
                SerializablePort {
                    name: "input".to_string(),
                    id: 1,
                    direction: PortDirection::In,
                }
            ],
            outputs: vec![],
        };

        let processor = NodeFactory::create_processor(&node).unwrap();
        assert_eq!(processor.name(), "test_sink");

        let input_ports = NodeFactory::get_input_ports(&node);
        assert_eq!(input_ports.len(), 1);
        assert_eq!(input_ports[0].0, "input");

        let output_ports = NodeFactory::get_output_ports(&node);
        assert!(output_ports.is_empty());
    }
}
