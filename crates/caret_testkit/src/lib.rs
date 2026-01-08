// Caret Testkit - Testing utilities for Caret
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

#![warn(missing_docs)]
#![warn(clippy::all)]

use caret_core::{Packet, Result};
use caret_graph::{Graph, Node};
use caret_sched::{NodeProcessor, PassthroughNode, ProcessingContext, ProcessingResult, Runtime};

/// Integration test helper for two-node pipelines
pub struct TwoNodePipeline {
    graph: Graph,
    source_id: u64,
    sink_id: u64,
}

impl TwoNodePipeline {
    /// Create a new two-node pipeline with a source and sink
    pub fn new() -> Result<Self> {
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
    pub fn validate(&self) -> Result<()> {
        self.graph.validate()
    }
}

/// A simple counting node for testing
pub struct CountingNode {
    name: String,
    count: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl CountingNode {
    /// Create a new counting node
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            count: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Get the current count
    pub fn count(&self) -> u64 {
        self.count.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Reset the count
    pub fn reset(&self) {
        self.count.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get a shared reference to the counter
    pub fn counter(&self) -> std::sync::Arc<std::sync::atomic::AtomicU64> {
        self.count.clone()
    }
}

impl NodeProcessor for CountingNode {
    fn process(
        &mut self,
        _ctx: &ProcessingContext,
        _packet: Packet,
        _port: &str,
    ) -> Result<ProcessingResult> {
        self.count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(ProcessingResult::Continue)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Integration test helper for three-node pipelines (source -> transform -> sink)
pub struct ThreeNodePipeline {
    runtime: Runtime,
    source_id: u64,
    transform_id: u64,
    sink_id: u64,
}

impl ThreeNodePipeline {
    /// Create a new three-node pipeline
    pub fn new() -> Result<Self> {
        let mut runtime = Runtime::new();

        // Create source node with output port
        let source = Box::new(PassthroughNode::new("source"));
        let source_id = runtime.executor_mut().add_node(1, source)?;
        let source_instance = runtime.executor().node(source_id).unwrap();
        source_instance.lock().ports.add_output("output")?;

        // Create transform node with input and output ports
        let transform = Box::new(PassthroughNode::new("transform"));
        let transform_id = runtime.executor_mut().add_node(2, transform)?;
        let transform_instance = runtime.executor().node(transform_id).unwrap();
        transform_instance.lock().ports.add_input("input", 1024)?;
        transform_instance.lock().ports.add_output("output")?;

        // Create sink node with input port
        let sink = Box::new(PassthroughNode::new("sink"));
        let sink_id = runtime.executor_mut().add_node(3, sink)?;
        let sink_instance = runtime.executor().node(sink_id).unwrap();
        sink_instance.lock().ports.add_input("input", 1024)?;

        // Connect source -> transform
        runtime
            .executor_mut()
            .connect(source_id, "output", transform_id, "input")?;

        // Connect transform -> sink
        runtime
            .executor_mut()
            .connect(transform_id, "output", sink_id, "input")?;

        Ok(Self {
            runtime,
            source_id: source_id.as_u64(),
            transform_id: transform_id.as_u64(),
            sink_id: sink_id.as_u64(),
        })
    }

    /// Get the runtime
    pub fn runtime(&mut self) -> &mut Runtime {
        &mut self.runtime
    }

    /// Get the source node runtime ID
    pub fn source_id(&self) -> u64 {
        self.source_id
    }

    /// Get the transform node runtime ID
    pub fn transform_id(&self) -> u64 {
        self.transform_id
    }

    /// Get the sink node runtime ID
    pub fn sink_id(&self) -> u64 {
        self.sink_id
    }

    /// Run the pipeline to completion
    pub fn run(&mut self) -> Result<caret_sched::RunResult> {
        self.runtime.run()
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

    #[test]
    fn test_three_node_pipeline_create() {
        let pipeline = ThreeNodePipeline::new().unwrap();
        // Just verify the pipeline was created successfully
        assert!(pipeline.source_id() > 0);
        assert!(pipeline.transform_id() > 0);
        assert!(pipeline.sink_id() > 0);
        // IDs should be unique
        assert!(pipeline.source_id() != pipeline.transform_id());
        assert!(pipeline.transform_id() != pipeline.sink_id());
        assert!(pipeline.source_id() != pipeline.sink_id());
    }

    #[test]
    fn test_counting_node() {
        let node = CountingNode::new("counter");
        assert_eq!(node.count(), 0);

        let ctx = ProcessingContext::new(caret_sched::NodeId::new(1), 1);
        let packet = Packet::bytes(&[][..]);

        let mut node = node;
        let result = node.process(&ctx, packet.clone(), "input").unwrap();
        assert!(matches!(result, ProcessingResult::Continue));
        assert_eq!(node.count(), 1);

        node.process(&ctx, packet, "input").unwrap();
        assert_eq!(node.count(), 2);
    }

    #[test]
    fn test_three_node_pipeline_run() {
        let mut pipeline = ThreeNodePipeline::new().unwrap();
        let result = pipeline.run().unwrap();
        // Empty pipeline completes (runs until safety valve at 10000 ticks)
        assert!(result.ticks > 1);
    }
}
