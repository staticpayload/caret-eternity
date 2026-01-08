// Caret Sched - Execution policy
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::node::NodeId;
use std::time::Duration;

/// How the runtime should execute the graph
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionPolicy {
    /// Real-time execution
    ///
    /// Nodes are scheduled as soon as data is available.
    /// Best for streaming and interactive applications.
    Realtime,

    /// Batch execution
    ///
    /// Nodes execute in discrete ticks, processing all available data.
    /// Best for offline processing and deterministic execution.
    Batch {
        /// Maximum number of packets to process per tick
        max_packets: usize,
    },
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self::Realtime
    }
}

/// Decision made by the scheduler
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchedulingDecision {
    /// Run the node immediately
    RunNow,
    /// Run the node after a delay
    RunAfter(Duration),
    /// Skip this tick
    Skip,
    /// Stop scheduling this node
    Stop,
}

/// Priority for node scheduling
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NodePriority {
    /// Low priority - background processing
    Low = 0,
    /// Normal priority - default
    Normal = 1,
    /// High priority - latency-sensitive
    High = 2,
    /// Critical priority - must run immediately
    Critical = 3,
}

impl Default for NodePriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Scheduling request from a node
pub struct SchedulingRequest {
    /// Node making the request
    pub node_id: NodeId,
    /// Desired priority
    pub priority: NodePriority,
    /// Minimum delay before next execution
    pub min_delay: Option<Duration>,
}

impl SchedulingRequest {
    /// Create a new scheduling request
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            priority: NodePriority::default(),
            min_delay: None,
        }
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: NodePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the minimum delay
    pub fn with_min_delay(mut self, delay: Duration) -> Self {
        self.min_delay = Some(delay);
        self
    }

    /// Create a request for immediate execution
    pub fn immediate(node_id: NodeId) -> Self {
        Self {
            node_id,
            priority: NodePriority::Critical,
            min_delay: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_policy_default() {
        let policy = ExecutionPolicy::default();
        assert_eq!(policy, ExecutionPolicy::Realtime);
    }

    #[test]
    fn test_node_priority_ordering() {
        assert!(NodePriority::Critical > NodePriority::High);
        assert!(NodePriority::High > NodePriority::Normal);
        assert!(NodePriority::Normal > NodePriority::Low);
    }

    #[test]
    fn test_scheduling_request_builder() {
        let node_id = NodeId::new(1);
        let request = SchedulingRequest::new(node_id)
            .with_priority(NodePriority::High)
            .with_min_delay(Duration::from_millis(10));

        assert_eq!(request.node_id, node_id);
        assert_eq!(request.priority, NodePriority::High);
        assert_eq!(request.min_delay, Some(Duration::from_millis(10)));
    }

    #[test]
    fn test_scheduling_request_immediate() {
        let node_id = NodeId::new(1);
        let request = SchedulingRequest::immediate(node_id);

        assert_eq!(request.node_id, node_id);
        assert_eq!(request.priority, NodePriority::Critical);
        assert!(request.min_delay.is_none());
    }
}
