// Caret Testkit - Testing utilities for Caret
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

#![warn(missing_docs)]
#![warn(clippy::all)]

use caret_core::Packet;
use caret_graph::{Graph, Node, PortDirection};

/// Integration test helper for two-node pipelines
pub struct TwoNodePipeline {
    graph: Graph,
    source_id: u64,
    sink_id: u64,
}

impl TwoNodePipeline {
    /// Create a new two-node pipeline with a source and sink
    pub fn new() -> caret_core::Result<Self> {
        let mut graph = Graph::new();

        // Create source node
        let mut source = Node::source("test_source");
        source.add_output("output")?;
        let source_id = source.id().as_u64();
        graph.add_node(source)?;

        // Create sink node
        let mut sink = Node::sink("test_sink");
        sink.add_input("input")?;
        let sink_id = sink.id().as_u64();
        graph.add_node(sink)?;

        // Connect them
        graph.connect(source_id, "output", sink_id, "input")?;

        Ok(Self {
            graph,
            source_id,
            sink_id,
        })
    }

    /// Get the graph
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Get the source node ID
    pub fn source_id(&self) -> u64 {
        self.source_id
    }

    /// Get the sink node ID
    pub fn sink_id(&self) -> u64 {
        self.sink_id
    }

    /// Validate the pipeline
    pub fn validate(&self) -> caret_core::Result<()> {
        self.graph.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_node_pipeline_create() {
        let pipeline = TwoNodePipeline::new().unwrap();
        assert_eq!(pipeline.graph().node_count(), 2);
        assert!(pipeline.validate().is_ok());
    }

    #[test]
    fn test_two_node_pipeline_topology() {
        let pipeline = TwoNodePipeline::new().unwrap();
        let topo = pipeline.graph().topology();

        assert_eq!(topo.edge_count(), 1);
        assert_eq!(topo.edges_from(pipeline.source_id()).len(), 1);
        assert_eq!(topo.edges_to(pipeline.sink_id()).len(), 1);
    }

    #[test]
    fn test_two_node_pipeline_topological_order() {
        let pipeline = TwoNodePipeline::new().unwrap();
        let topo = pipeline.graph().topology();

        let order = topo.topological_order();
        assert_eq!(order, Some(vec![pipeline.source_id(), pipeline.sink_id()]));
    }
}
