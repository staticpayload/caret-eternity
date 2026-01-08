// Caret Sched - Advanced scheduling strategies
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{node::NodeId, policy::NodePriority};
use parking_lot::Mutex;
use std::collections::{HashMap, BinaryHeap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Trait for scheduling strategies
pub trait Scheduler: Send + Sync {
    /// Add a node to the scheduler
    fn add_node(&self, node_id: NodeId, priority: NodePriority);

    /// Remove a node from the scheduler
    fn remove_node(&self, node_id: NodeId);

    /// Update the priority of a node
    fn update_priority(&self, node_id: NodeId, priority: NodePriority);

    /// Get the next node(s) to execute
    fn schedule(&self) -> SchedulingChoice;

    /// Notify that a node has data available
    fn notify_data_available(&self, node_id: NodeId);

    /// Notify that a node has completed execution
    fn notify_completed(&self, node_id: NodeId);
}

/// Choice made by the scheduler
#[derive(Clone, Debug)]
pub enum SchedulingChoice {
    /// Single node to execute
    Single(NodeId),
    /// Multiple nodes to execute (for parallel execution)
    Multiple(Vec<NodeId>),
    /// No nodes ready to execute
    None,
    /// Stop execution
    Stop,
}

/// Metadata about a node for scheduling
#[derive(Clone, Debug)]
struct NodeMetadata {
    /// Node ID
    id: NodeId,
    /// Current priority
    priority: NodePriority,
    /// Whether the node has data available
    has_data: bool,
    /// Last execution time
    last_executed: Option<Instant>,
    /// Deadline for execution (if any)
    deadline: Option<Instant>,
    /// Execution count
    exec_count: u64,
    /// Average execution duration
    avg_duration: Duration,
}

impl NodeMetadata {
    fn new(id: NodeId, priority: NodePriority) -> Self {
        Self {
            id,
            priority,
            has_data: false,
            last_executed: None,
            deadline: None,
            exec_count: 0,
            avg_duration: Duration::ZERO,
        }
    }

    /// Calculate a score for priority scheduling (higher is better)
    fn priority_score(&self) -> i64 {
        let base_score = self.priority as i64;
        // Bonus for nodes that haven't executed recently
        let recency_bonus = if let Some(last) = self.last_executed {
            let elapsed = last.elapsed().as_millis() as i64;
            elapsed / 100 // 1 point per 100ms waiting
        } else {
            1000 // Never executed bonus
        };
        base_score * 1000 + recency_bonus
    }

    /// Calculate urgency score for deadline scheduling (higher is more urgent)
    fn urgency_score(&self) -> i64 {
        if let Some(deadline) = self.deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            // More urgent = higher score (less time remaining)
            // Use i64::MAX - remaining so that 0 remaining = highest score
            i64::MAX.saturating_sub(remaining.as_millis() as i64)
        } else {
            i64::MIN // No deadline = least urgent
        }
    }

    /// Calculate fairness score (higher should run next)
    fn fairness_score(&self) -> i64 {
        // Prefer nodes with lower execution counts
        -(self.exec_count as i64)
    }
}

/// Priority-based scheduler
///
/// Schedules nodes based on their priority, with recency bonuses
/// to prevent starvation of lower-priority nodes.
pub struct PriorityScheduler {
    /// Node metadata
    nodes: Arc<Mutex<HashMap<NodeId, NodeMetadata>>>,
    /// Ready queue (binary heap by priority)
    ready_queue: Arc<Mutex<BinaryHeap<PriorityNode>>>,
}

#[derive(Clone, Debug)]
struct PriorityNode {
    node_id: NodeId,
    priority: NodePriority,
    score: i64,
}

impl PartialEq for PriorityNode {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.node_id == other.node_id
    }
}

impl Eq for PriorityNode {}

impl PartialOrd for PriorityNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher score = higher priority (BinaryHeap is max-heap)
        self.score.cmp(&other.score)
            .then_with(|| self.node_id.cmp(&other.node_id))
    }
}

impl PriorityScheduler {
    /// Create a new priority scheduler
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(HashMap::new())),
            ready_queue: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }

    /// Rebuild the ready queue from current nodes
    fn rebuild_queue(&self) {
        let nodes = self.nodes.lock();
        let mut queue = self.ready_queue.lock();
        queue.clear();

        for (_, meta) in nodes.iter() {
            if meta.has_data {
                let priority_node = PriorityNode {
                    node_id: meta.id,
                    priority: meta.priority,
                    score: meta.priority_score(),
                };
                queue.push(priority_node);
            }
        }
    }
}

impl Default for PriorityScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler for PriorityScheduler {
    fn add_node(&self, node_id: NodeId, priority: NodePriority) {
        let mut nodes = self.nodes.lock();
        nodes.insert(node_id, NodeMetadata::new(node_id, priority));
    }

    fn remove_node(&self, node_id: NodeId) {
        {
            let mut nodes = self.nodes.lock();
            nodes.remove(&node_id);
        }
        self.rebuild_queue();
    }

    fn update_priority(&self, node_id: NodeId, priority: NodePriority) {
        {
            let mut nodes = self.nodes.lock();
            if let Some(meta) = nodes.get_mut(&node_id) {
                meta.priority = priority;
            }
        }
        self.rebuild_queue();
    }

    fn schedule(&self) -> SchedulingChoice {
        let mut queue = self.ready_queue.lock();
        if let Some(node) = queue.pop() {
            SchedulingChoice::Single(node.node_id)
        } else {
            SchedulingChoice::None
        }
    }

    fn notify_data_available(&self, node_id: NodeId) {
        let mut nodes = self.nodes.lock();
        if let Some(meta) = nodes.get_mut(&node_id) {
            meta.has_data = true;
            let priority_node = PriorityNode {
                node_id: meta.id,
                priority: meta.priority,
                score: meta.priority_score(),
            };
            drop(nodes);
            let mut queue = self.ready_queue.lock();
            queue.push(priority_node);
        }
    }

    fn notify_completed(&self, node_id: NodeId) {
        let mut nodes = self.nodes.lock();
        if let Some(meta) = nodes.get_mut(&node_id) {
            meta.last_executed = Some(Instant::now());
            meta.exec_count += 1;
            meta.has_data = false;
        }
    }
}

/// Fair scheduler with round-robin
///
/// Ensures all nodes get fair CPU time by cycling through them.
pub struct FairScheduler {
    /// Node metadata
    nodes: Arc<Mutex<HashMap<NodeId, NodeMetadata>>>,
    /// Ready queue for round-robin
    ready_queue: Arc<Mutex<VecDeque<NodeId>>>,
    /// Current position in the queue
    current_position: Arc<Mutex<usize>>,
}

impl FairScheduler {
    /// Create a new fair scheduler
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(HashMap::new())),
            ready_queue: Arc::new(Mutex::new(VecDeque::new())),
            current_position: Arc::new(Mutex::new(0)),
        }
    }

    /// Add node to the ready queue
    fn enqueue(&self, node_id: NodeId) {
        let mut queue = self.ready_queue.lock();
        if !queue.contains(&node_id) {
            queue.push_back(node_id);
        }
    }
}

impl Default for FairScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler for FairScheduler {
    fn add_node(&self, node_id: NodeId, priority: NodePriority) {
        let mut nodes = self.nodes.lock();
        nodes.insert(node_id, NodeMetadata::new(node_id, priority));
    }

    fn remove_node(&self, node_id: NodeId) {
        let mut nodes = self.nodes.lock();
        nodes.remove(&node_id);
        let mut queue = self.ready_queue.lock();
        queue.retain(|id| *id != node_id);
    }

    fn update_priority(&self, _node_id: NodeId, _priority: NodePriority) {
        // Fair scheduler doesn't use priority for scheduling
    }

    fn schedule(&self) -> SchedulingChoice {
        let mut queue = self.ready_queue.lock();
        if let Some(node_id) = queue.pop_front() {
            SchedulingChoice::Single(node_id)
        } else {
            SchedulingChoice::None
        }
    }

    fn notify_data_available(&self, node_id: NodeId) {
        self.enqueue(node_id);
    }

    fn notify_completed(&self, _node_id: NodeId) {
        // Node will be re-enqueued when data is available
    }
}

/// Deadline-aware scheduler
///
/// Prioritizes nodes with approaching deadlines.
pub struct DeadlineScheduler {
    /// Node metadata
    nodes: Arc<Mutex<HashMap<NodeId, NodeMetadata>>>,
    /// Urgency queue (binary heap by deadline urgency)
    urgency_queue: Arc<Mutex<BinaryHeap<UrgentNode>>>,
}

#[derive(Clone, Debug)]
struct UrgentNode {
    node_id: NodeId,
    urgency: i64,
}

impl PartialEq for UrgentNode {
    fn eq(&self, other: &Self) -> bool {
        self.urgency == other.urgency && self.node_id == other.node_id
    }
}

impl Eq for UrgentNode {}

impl PartialOrd for UrgentNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UrgentNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher urgency = higher priority (BinaryHeap is max-heap)
        self.urgency.cmp(&other.urgency)
            .then_with(|| self.node_id.cmp(&other.node_id))
    }
}

impl DeadlineScheduler {
    /// Create a new deadline scheduler
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(Mutex::new(HashMap::new())),
            urgency_queue: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }

    /// Set the deadline for a node
    pub fn set_deadline(&self, node_id: NodeId, deadline: Instant) {
        let mut nodes = self.nodes.lock();
        if let Some(meta) = nodes.get_mut(&node_id) {
            meta.deadline = Some(deadline);
        }
    }

    /// Clear the deadline for a node
    pub fn clear_deadline(&self, node_id: NodeId) {
        let mut nodes = self.nodes.lock();
        if let Some(meta) = nodes.get_mut(&node_id) {
            meta.deadline = None;
        }
    }

    /// Rebuild the urgency queue
    fn rebuild_queue(&self) {
        let nodes = self.nodes.lock();
        let mut queue = self.urgency_queue.lock();
        queue.clear();

        for (_, meta) in nodes.iter() {
            if meta.has_data {
                let urgent_node = UrgentNode {
                    node_id: meta.id,
                    urgency: meta.urgency_score(),
                };
                queue.push(urgent_node);
            }
        }
    }
}

impl Default for DeadlineScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler for DeadlineScheduler {
    fn add_node(&self, node_id: NodeId, priority: NodePriority) {
        let mut nodes = self.nodes.lock();
        nodes.insert(node_id, NodeMetadata::new(node_id, priority));
    }

    fn remove_node(&self, node_id: NodeId) {
        {
            let mut nodes = self.nodes.lock();
            nodes.remove(&node_id);
        }
        self.rebuild_queue();
    }

    fn update_priority(&self, node_id: NodeId, priority: NodePriority) {
        {
            let mut nodes = self.nodes.lock();
            if let Some(meta) = nodes.get_mut(&node_id) {
                meta.priority = priority;
            }
        }
        self.rebuild_queue();
    }

    fn schedule(&self) -> SchedulingChoice {
        let mut queue = self.urgency_queue.lock();
        if let Some(node) = queue.pop() {
            SchedulingChoice::Single(node.node_id)
        } else {
            SchedulingChoice::None
        }
    }

    fn notify_data_available(&self, node_id: NodeId) {
        let mut nodes = self.nodes.lock();
        if let Some(meta) = nodes.get_mut(&node_id) {
            meta.has_data = true;
            let urgent_node = UrgentNode {
                node_id: meta.id,
                urgency: meta.urgency_score(),
            };
            drop(nodes);
            let mut queue = self.urgency_queue.lock();
            queue.push(urgent_node);
        }
    }

    fn notify_completed(&self, node_id: NodeId) {
        let mut nodes = self.nodes.lock();
        if let Some(meta) = nodes.get_mut(&node_id) {
            meta.last_executed = Some(Instant::now());
            meta.exec_count += 1;
            meta.has_data = false;
        }
    }
}

/// Configuration for work-stealing scheduler
#[derive(Clone, Debug)]
pub struct WorkStealingConfig {
    /// Number of worker threads
    pub num_workers: usize,
    /// Maximum queue size per worker before stealing is enabled
    pub steal_threshold: usize,
    /// How often to check for work to steal (in ticks)
    pub steal_interval: u64,
}

impl Default for WorkStealingConfig {
    fn default() -> Self {
        Self {
            num_workers: num_cpus::get(),
            steal_threshold: 2,
            steal_interval: 10,
        }
    }
}

/// Work-stealing scheduler for multi-threaded execution
///
/// Uses multiple worker queues where idle workers can steal tasks
/// from busy workers.
pub struct WorkStealingScheduler {
    /// Node metadata
    nodes: Arc<Mutex<HashMap<NodeId, NodeMetadata>>>,
    /// Per-worker queues
    worker_queues: Vec<Arc<Mutex<VecDeque<NodeId>>>>,
    /// Current worker index for round-robin assignment
    current_worker: Arc<Mutex<usize>>,
    /// Configuration
    config: WorkStealingConfig,
    /// Current tick
    tick: Arc<Mutex<u64>>,
}

impl WorkStealingScheduler {
    /// Create a new work-stealing scheduler
    pub fn new(config: WorkStealingConfig) -> Self {
        let num_workers = config.num_workers.max(1);
        let mut worker_queues = Vec::with_capacity(num_workers);
        for _ in 0..num_workers {
            worker_queues.push(Arc::new(Mutex::new(VecDeque::new())));
        }

        Self {
            nodes: Arc::new(Mutex::new(HashMap::new())),
            worker_queues,
            current_worker: Arc::new(Mutex::new(0)),
            config,
            tick: Arc::new(Mutex::new(0)),
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(WorkStealingConfig::default())
    }

    /// Get the number of workers
    pub fn num_workers(&self) -> usize {
        self.config.num_workers
    }

    /// Assign a node to a worker queue
    fn assign_to_worker(&self, node_id: NodeId) {
        let mut current = self.current_worker.lock();
        let worker_idx = *current;
        *current = (*current + 1) % self.worker_queues.len();
        drop(current);

        let queue = self.worker_queues[worker_idx].clone();
        let mut q = queue.lock();
        if !q.contains(&node_id) {
            q.push_back(node_id);
        }
    }

    /// Try to steal work from another queue
    fn try_steal(&self, from_worker: usize) -> Option<NodeId> {
        let num_workers = self.worker_queues.len();
        for offset in 1..num_workers {
            let target_worker = (from_worker + offset) % num_workers;
            let target_queue = self.worker_queues[target_worker].clone();
            let mut q = target_queue.lock();

            // Only steal if the target has more than the threshold
            if q.len() > self.config.steal_threshold {
                if let Some(node_id) = q.pop_back() {
                    return Some(node_id);
                }
            }
        }
        None
    }

    /// Get all nodes ready for parallel execution
    pub fn schedule_parallel(&self) -> Vec<NodeId> {
        let mut nodes = Vec::new();
        let tick = *self.tick.lock();

        // Check if we should try stealing
        let try_stealing = tick % self.config.steal_interval == 0;

        for (worker_idx, queue) in self.worker_queues.iter().enumerate() {
            let mut q = queue.lock();
            if let Some(node_id) = q.pop_front() {
                nodes.push(node_id);
            } else if try_stealing {
                drop(q);
                if let Some(stolen) = self.try_steal(worker_idx) {
                    nodes.push(stolen);
                }
            }
        }

        nodes
    }

    /// Advance the tick counter
    pub fn advance_tick(&self) {
        let mut tick = self.tick.lock();
        *tick += 1;
    }
}

impl Scheduler for WorkStealingScheduler {
    fn add_node(&self, node_id: NodeId, priority: NodePriority) {
        let mut nodes = self.nodes.lock();
        nodes.insert(node_id, NodeMetadata::new(node_id, priority));
    }

    fn remove_node(&self, node_id: NodeId) {
        let mut nodes = self.nodes.lock();
        nodes.remove(&node_id);
        for queue in self.worker_queues.iter() {
            let mut q = queue.lock();
            q.retain(|id| *id != node_id);
        }
    }

    fn update_priority(&self, node_id: NodeId, priority: NodePriority) {
        let mut nodes = self.nodes.lock();
        if let Some(meta) = nodes.get_mut(&node_id) {
            meta.priority = priority;
        }
    }

    fn schedule(&self) -> SchedulingChoice {
        // Find the first non-empty queue
        for queue in self.worker_queues.iter() {
            let q = queue.lock();
            if let Some(&node_id) = q.front() {
                return SchedulingChoice::Single(node_id);
            }
        }
        SchedulingChoice::None
    }

    fn notify_data_available(&self, node_id: NodeId) {
        self.assign_to_worker(node_id);
    }

    fn notify_completed(&self, _node_id: NodeId) {
        // Node will be re-enqueued when data is available
    }
}

/// Scheduler configuration
#[derive(Clone, Debug)]
pub enum SchedulerConfig {
    /// Priority-based scheduling
    Priority,
    /// Fair round-robin scheduling
    Fair,
    /// Deadline-aware scheduling
    Deadline,
    /// Work-stealing scheduling
    WorkStealing(WorkStealingConfig),
}

impl SchedulerConfig {
    /// Create a scheduler instance from this config
    pub fn create(&self) -> Box<dyn Scheduler> {
        match self {
            Self::Priority => Box::new(PriorityScheduler::new()),
            Self::Fair => Box::new(FairScheduler::new()),
            Self::Deadline => Box::new(DeadlineScheduler::new()),
            Self::WorkStealing(config) => Box::new(WorkStealingScheduler::new(config.clone())),
        }
    }
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self::Priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_metadata_priority_score() {
        let id = NodeId::new(1);
        let meta = NodeMetadata::new(id, NodePriority::High);
        let score = meta.priority_score();
        assert!(score > 0);

        // Higher priority should have higher score
        let low_meta = NodeMetadata::new(id, NodePriority::Low);
        assert!(meta.priority_score() > low_meta.priority_score());
    }

    #[test]
    fn test_node_metadata_urgency_score() {
        let id = NodeId::new(1);
        let mut meta = NodeMetadata::new(id, NodePriority::Normal);

        // No deadline = least urgent
        assert_eq!(meta.urgency_score(), i64::MIN);

        // With deadline - should be high (close to i64::MAX)
        meta.deadline = Some(Instant::now() + Duration::from_millis(100));
        let score = meta.urgency_score();
        assert!(score > i64::MIN);
        // With ~100ms remaining, score should be close to MAX
        assert!(score > i64::MAX - 1000);
    }

    #[test]
    fn test_node_metadata_fairness_score() {
        let id = NodeId::new(1);
        let mut meta = NodeMetadata::new(id, NodePriority::Normal);

        let score1 = meta.fairness_score();
        meta.exec_count = 5;
        let score2 = meta.fairness_score();

        // Higher exec count should have lower (more negative) score
        assert!(score2 < score1);
    }

    #[test]
    fn test_priority_scheduler_add_remove() {
        let scheduler = PriorityScheduler::new();
        let id = NodeId::new(1);

        scheduler.add_node(id, NodePriority::High);
        scheduler.remove_node(id);
        // Should not panic
    }

    #[test]
    fn test_priority_scheduler_notify() {
        let scheduler = PriorityScheduler::new();
        let id = NodeId::new(1);

        scheduler.add_node(id, NodePriority::High);
        scheduler.notify_data_available(id);

        let choice = scheduler.schedule();
        match choice {
            SchedulingChoice::Single(node_id) => assert_eq!(node_id, id),
            _ => panic!("Expected Single scheduling choice"),
        }
    }

    #[test]
    fn test_fair_scheduler_add_remove() {
        let scheduler = FairScheduler::new();
        let id = NodeId::new(1);

        scheduler.add_node(id, NodePriority::Normal);
        scheduler.remove_node(id);
        // Should not panic
    }

    #[test]
    fn test_fair_scheduler_notify() {
        let scheduler = FairScheduler::new();
        let id = NodeId::new(1);

        scheduler.add_node(id, NodePriority::Normal);
        scheduler.notify_data_available(id);

        let choice = scheduler.schedule();
        match choice {
            SchedulingChoice::Single(node_id) => assert_eq!(node_id, id),
            _ => panic!("Expected Single scheduling choice"),
        }
    }

    #[test]
    fn test_deadline_scheduler_add_remove() {
        let scheduler = DeadlineScheduler::new();
        let id = NodeId::new(1);

        scheduler.add_node(id, NodePriority::Normal);
        scheduler.remove_node(id);
        // Should not panic
    }

    #[test]
    fn test_deadline_scheduler_deadline() {
        let scheduler = DeadlineScheduler::new();
        let id = NodeId::new(1);

        scheduler.add_node(id, NodePriority::Normal);
        let deadline = Instant::now() + Duration::from_secs(1);
        scheduler.set_deadline(id, deadline);

        scheduler.notify_data_available(id);

        let choice = scheduler.schedule();
        match choice {
            SchedulingChoice::Single(node_id) => assert_eq!(node_id, id),
            _ => panic!("Expected Single scheduling choice"),
        }

        scheduler.clear_deadline(id);
    }

    #[test]
    fn test_work_stealing_scheduler_config() {
        let config = WorkStealingConfig::default();
        assert!(config.num_workers > 0);
        assert_eq!(config.steal_threshold, 2);
        assert_eq!(config.steal_interval, 10);
    }

    #[test]
    fn test_work_stealing_scheduler_add_remove() {
        let scheduler = WorkStealingScheduler::with_default_config();
        let id = NodeId::new(1);

        scheduler.add_node(id, NodePriority::Normal);
        assert_eq!(scheduler.num_workers(), num_cpus::get());
        scheduler.remove_node(id);
        // Should not panic
    }

    #[test]
    fn test_work_stealing_scheduler_notify() {
        let scheduler = WorkStealingScheduler::with_default_config();
        let id = NodeId::new(1);

        scheduler.add_node(id, NodePriority::Normal);
        scheduler.notify_data_available(id);

        let choice = scheduler.schedule();
        match choice {
            SchedulingChoice::Single(node_id) => assert_eq!(node_id, id),
            _ => panic!("Expected Single scheduling choice"),
        }
    }

    #[test]
    fn test_work_stealing_scheduler_parallel() {
        let scheduler = WorkStealingScheduler::with_default_config();
        let id1 = NodeId::new(1);
        let id2 = NodeId::new(2);

        scheduler.add_node(id1, NodePriority::Normal);
        scheduler.add_node(id2, NodePriority::Normal);

        scheduler.notify_data_available(id1);
        scheduler.notify_data_available(id2);

        let nodes = scheduler.schedule_parallel();
        assert!(!nodes.is_empty());
        assert!(nodes.len() <= 2);
    }

    #[test]
    fn test_scheduler_config_default() {
        let config = SchedulerConfig::default();
        match config {
            SchedulerConfig::Priority => {}
            _ => panic!("Expected Priority scheduler config"),
        }
    }

    #[test]
    fn test_scheduler_config_create() {
        let configs = vec![
            SchedulerConfig::Priority,
            SchedulerConfig::Fair,
            SchedulerConfig::Deadline,
            SchedulerConfig::WorkStealing(WorkStealingConfig::default()),
        ];

        for config in configs {
            let scheduler = config.create();
            // Should not panic
            drop(scheduler);
        }
    }

    #[test]
    fn test_priority_node_ordering() {
        let node1 = PriorityNode {
            node_id: NodeId::new(1),
            priority: NodePriority::High,
            score: 100,
        };
        let node2 = PriorityNode {
            node_id: NodeId::new(2),
            priority: NodePriority::Low,
            score: 50,
        };

        // Higher score should be "greater" for max-heap
        assert!(node1 > node2);
        // Verify that BinaryHeap pops the higher score first
        use std::collections::BinaryHeap;
        let mut heap = BinaryHeap::new();
        heap.push(node1.clone());
        heap.push(node2.clone());
        assert_eq!(heap.pop().unwrap().node_id, node1.node_id);
    }

    #[test]
    fn test_urgent_node_ordering() {
        let node1 = UrgentNode {
            node_id: NodeId::new(1),
            urgency: i64::MAX - 10, // More urgent (higher value)
        };
        let node2 = UrgentNode {
            node_id: NodeId::new(2),
            urgency: i64::MAX - 100, // Less urgent (lower value)
        };

        // Higher urgency value should be "greater" for max-heap
        assert!(node1 > node2);
        // Verify that BinaryHeap pops the higher urgency first
        use std::collections::BinaryHeap;
        let mut heap = BinaryHeap::new();
        heap.push(node1.clone());
        heap.push(node2.clone());
        assert_eq!(heap.pop().unwrap().node_id, node1.node_id);
    }

    #[test]
    fn test_scheduling_choice_none_when_empty() {
        let scheduler = PriorityScheduler::new();
        let choice = scheduler.schedule();
        assert!(matches!(choice, SchedulingChoice::None));
    }

    #[test]
    fn test_work_stealing_advance_tick() {
        let scheduler = WorkStealingScheduler::with_default_config();
        scheduler.advance_tick();
        scheduler.advance_tick();
        // Should not panic
    }
}
