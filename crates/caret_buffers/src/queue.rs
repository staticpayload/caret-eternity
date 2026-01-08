// Caret Buffers - Bounded queue implementation
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use caret_core::Error;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

/// Policy for handling queue overflow
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverflowPolicy {
    /// Block until space is available
    Block,
    /// Drop the newest item
    DropNewest,
    /// Drop the oldest item
    DropOldest,
    /// Error instead of dropping
    Error,
}

/// A bounded queue with configurable overflow policy
///
/// This queue provides thread-safe bounded capacity with various
/// overflow handling strategies. It is designed for use between
/// graph nodes to implement backpressure.
pub struct BoundedQueue<T> {
    inner: Arc<Mutex<QueueInner<T>>>,
    capacity: usize,
    policy: OverflowPolicy,
}

struct QueueInner<T> {
    items: VecDeque<T>,
    /// Number of items that were dropped due to overflow
    dropped_count: usize,
}

impl<T> BoundedQueue<T> {
    /// Create a new bounded queue with the given capacity
    ///
    /// Default policy is Block.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(QueueInner {
                items: VecDeque::new(),
                dropped_count: 0,
            })),
            capacity,
            policy: OverflowPolicy::Block,
        }
    }

    /// Create a new bounded queue with the given capacity and overflow policy
    pub fn with_policy(capacity: usize, policy: OverflowPolicy) -> Self {
        Self {
            inner: Arc::new(Mutex::new(QueueInner {
                items: VecDeque::new(),
                dropped_count: 0,
            })),
            capacity,
            policy,
        }
    }

    /// Push an item to the back of the queue
    ///
    /// Returns an error if the queue is full and policy is Error.
    pub fn push(&self, item: T) -> Result<(), Error> {
        let mut inner = self.inner.lock();

        if inner.items.len() >= self.capacity {
            match self.policy {
                OverflowPolicy::Block => {
                    // For blocking behavior, we'd need a condition variable
                    // For now, return an error indicating blocking would be needed
                    return Err(Error::resource_exhausted("queue is full, would block"));
                }
                OverflowPolicy::DropNewest => {
                    // Don't add the new item
                    inner.dropped_count += 1;
                    return Ok(());
                }
                OverflowPolicy::DropOldest => {
                    // Remove oldest to make room
                    inner.items.pop_front();
                    inner.dropped_count += 1;
                }
                OverflowPolicy::Error => {
                    return Err(Error::resource_exhausted("queue is full"));
                }
            }
        }

        inner.items.push_back(item);
        Ok(())
    }

    /// Try to push without blocking
    ///
    /// Returns false if the queue is full.
    pub fn try_push(&self, item: T) -> bool {
        self.push(item).is_ok()
    }

    /// Pop an item from the front of the queue
    ///
    /// Returns None if the queue is empty.
    pub fn pop(&self) -> Option<T> {
        let mut inner = self.inner.lock();
        inner.items.pop_front()
    }

    /// Get the current length of the queue
    pub fn len(&self) -> usize {
        self.inner.lock().items.len()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.inner.lock().items.is_empty()
    }

    /// Check if the queue is full
    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity
    }

    /// Get the capacity of the queue
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the number of items dropped due to overflow
    pub fn dropped_count(&self) -> usize {
        self.inner.lock().dropped_count
    }

    /// Reset the dropped count
    pub fn reset_dropped_count(&self) {
        self.inner.lock().dropped_count = 0;
    }

    /// Clear all items from the queue
    pub fn clear(&self) {
        self.inner.lock().items.clear();
    }

    /// Get the overflow policy
    pub fn policy(&self) -> OverflowPolicy {
        self.policy
    }
}

impl<T> Clone for BoundedQueue<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            capacity: self.capacity,
            policy: self.policy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_create() {
        let queue: BoundedQueue<i32> = BoundedQueue::new(10);
        assert_eq!(queue.capacity(), 10);
        assert!(queue.is_empty());
        assert!(!queue.is_full());
    }

    #[test]
    fn test_queue_push_pop() {
        let queue: BoundedQueue<i32> = BoundedQueue::new(10);
        assert!(queue.try_push(1));
        assert!(queue.try_push(2));
        assert!(queue.try_push(3));

        assert_eq!(queue.len(), 3);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_queue_full() {
        let queue: BoundedQueue<i32> = BoundedQueue::new(2);
        assert!(queue.try_push(1));
        assert!(queue.try_push(2));
        assert!(queue.is_full());
        assert!(!queue.try_push(3));
    }

    #[test]
    fn test_queue_drop_oldest() {
        let queue: BoundedQueue<i32> = BoundedQueue::with_policy(3, OverflowPolicy::DropOldest);

        assert!(queue.try_push(1));
        assert!(queue.try_push(2));
        assert!(queue.try_push(3));
        // This should drop the oldest (1)
        assert!(queue.try_push(4));

        assert_eq!(queue.len(), 3);
        assert_eq!(queue.dropped_count(), 1);

        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), Some(4));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_queue_drop_newest() {
        let queue: BoundedQueue<i32> = BoundedQueue::with_policy(3, OverflowPolicy::DropNewest);

        assert!(queue.try_push(1));
        assert!(queue.try_push(2));
        assert!(queue.try_push(3));
        // This should not add, dropping the newest (4)
        assert!(queue.try_push(4));

        assert_eq!(queue.len(), 3);
        assert_eq!(queue.dropped_count(), 1);

        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn test_queue_error_policy() {
        let queue: BoundedQueue<i32> = BoundedQueue::with_policy(2, OverflowPolicy::Error);

        assert!(queue.try_push(1));
        assert!(queue.try_push(2));
        assert!(queue.is_full());

        let result = queue.push(3);
        assert!(result.is_err());
    }

    #[test]
    fn test_queue_clear() {
        let queue: BoundedQueue<i32> = BoundedQueue::new(10);
        assert!(queue.try_push(1));
        assert!(queue.try_push(2));
        assert!(queue.try_push(3));

        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn test_queue_reset_dropped_count() {
        let queue: BoundedQueue<i32> = BoundedQueue::with_policy(2, OverflowPolicy::DropOldest);

        assert!(queue.try_push(1));
        assert!(queue.try_push(2));
        assert!(queue.try_push(3));

        assert_eq!(queue.dropped_count(), 1);
        queue.reset_dropped_count();
        assert_eq!(queue.dropped_count(), 0);
    }

    #[test]
    fn test_queue_clone() {
        let queue: BoundedQueue<i32> = BoundedQueue::new(10);
        assert!(queue.try_push(1));
        assert!(queue.try_push(2));

        let queue2 = queue.clone();
        assert_eq!(queue2.len(), 2);
        assert_eq!(queue2.pop(), Some(1));
    }
}
