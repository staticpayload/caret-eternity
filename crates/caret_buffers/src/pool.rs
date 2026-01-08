// Caret Buffers - Buffer pool implementation
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::MemoryStats;
use bytes::Bytes;
use caret_core::Error;
use parking_lot::Mutex;
use std::sync::Arc;
use std::vec::Vec;

// Number of size classes (power of 2 buckets)
const NUM_SIZE_CLASSES: usize = 16;

/// Get the size class index for a given size
/// Returns the index of the smallest bucket that can hold this size
fn size_class_index(size: usize) -> Option<usize> {
    if size == 0 {
        return None;
    }
    // Find the next power of 2, then get its log2
    let next_pow2 = size.next_power_of_two();
    // Clamp to max size class (2^15 = 32768, can be adjusted)
    if next_pow2 > (1 << (NUM_SIZE_CLASSES - 1)) {
        return Some(NUM_SIZE_CLASSES - 1);
    }
    Some(next_pow2.trailing_zeros() as usize)
}

/// Configuration for a buffer pool
#[derive(Clone, Debug)]
pub struct BufferPoolConfig {
    /// Minimum buffer size
    pub min_buffer_size: usize,
    /// Maximum buffer size
    pub max_buffer_size: usize,
    /// Maximum number of buffers to pool
    pub max_buffers: usize,
    /// Whether to use slab allocation
    pub use_slab: bool,
}

impl Default for BufferPoolConfig {
    fn default() -> Self {
        Self {
            min_buffer_size: 4096,
            max_buffer_size: 65536,
            max_buffers: 1024,
            use_slab: false,
        }
    }
}

/// A pooled buffer with reference counting
///
/// When dropped, the buffer returns to the pool if there's capacity.
#[derive(Clone)]
pub struct PooledBuffer {
    /// The underlying bytes
    inner: Bytes,
    /// The pool this buffer belongs to
    pool: Option<Arc<BufferPoolInner>>,
}

impl PooledBuffer {
    /// Create a new pooled buffer
    fn new(bytes: Bytes, pool: Arc<BufferPoolInner>) -> Self {
        Self {
            inner: bytes,
            pool: Some(pool),
        }
    }

    /// Get the buffer data
    pub fn as_bytes(&self) -> &Bytes {
        &self.inner
    }

    /// Get the length of the buffer
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Detach from the pool, preventing return
    pub fn detach(mut self) -> Bytes {
        self.pool = None;
        self.inner.clone()
    }
}

impl std::ops::Deref for PooledBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        if let Some(pool) = &self.pool {
            // Only return if we're the last reference and pool has capacity
            if self.inner.is_unique() && pool.can_return() {
                pool.return_buffer(self.inner.clone());
            }
        }
    }
}

struct BufferPoolInner {
    config: BufferPoolConfig,
    /// Size class buckets for O(1) lookup
    /// Each bucket holds buffers of that size class
    size_classes: Mutex<Vec<Vec<Bytes>>>,
    stats: Mutex<MemoryStats>,
}

impl BufferPoolInner {
    /// Get total count of pooled buffers across all size classes
    fn total_pooled(&self) -> usize {
        let classes = self.size_classes.lock();
        classes.iter().map(|v| v.len()).sum()
    }

    fn can_return(&self) -> bool {
        self.total_pooled() < self.config.max_buffers
    }

    fn return_buffer(&self, buffer: Bytes) {
        if self.total_pooled() >= self.config.max_buffers {
            return;
        }

        let size_idx = size_class_index(buffer.len()).unwrap_or(0);
        let mut classes = self.size_classes.lock();
        if size_idx < classes.len() {
            classes[size_idx].push(buffer);
            let mut stats = self.stats.lock();
            stats.deallocation_count += 1;
        }
    }

    /// Try to acquire a buffer from the pool (O(1) with size classes)
    fn try_acquire(&self, size: usize) -> Option<Bytes> {
        let size_idx = size_class_index(size)?;
        let mut classes = self.size_classes.lock();

        // Check the exact size class first
        if size_idx < classes.len() && !classes[size_idx].is_empty() {
            let buffer = classes[size_idx].pop().unwrap();
            let mut stats = self.stats.lock();
            stats.in_use = stats.in_use.saturating_add(buffer.len());
            stats.allocation_count += 1;
            return Some(buffer);
        }

        // Check larger size classes
        for idx in (size_idx + 1)..classes.len() {
            if !classes[idx].is_empty() {
                let buffer = classes[idx].pop().unwrap();
                let mut stats = self.stats.lock();
                stats.in_use = stats.in_use.saturating_add(buffer.len());
                stats.allocation_count += 1;
                return Some(buffer);
            }
        }

        None
    }

    fn allocate_new(&self, size: usize) -> Bytes {
        let actual_size = size.clamp(self.config.min_buffer_size, self.config.max_buffer_size);
        let buffer = Bytes::from(vec![0u8; actual_size]);
        let mut stats = self.stats.lock();
        stats.total_allocated += actual_size;
        stats.in_use += actual_size;
        stats.allocation_count += 1;
        buffer
    }
}

/// A pool of reusable buffers
///
/// The buffer pool manages a set of pre-allocated buffers that can be
/// reused to reduce allocation overhead. Buffers are reference counted
/// and automatically return to the pool when no longer in use.
#[derive(Clone)]
pub struct BufferPool {
    inner: Arc<BufferPoolInner>,
}

impl BufferPool {
    /// Create a new buffer pool with default configuration
    pub fn new() -> Self {
        Self::with_config(BufferPoolConfig::default())
    }

    /// Create a new buffer pool with the given configuration
    pub fn with_config(config: BufferPoolConfig) -> Self {
        // Initialize size class buckets
        let size_classes = (0..NUM_SIZE_CLASSES).map(|_| Vec::new()).collect();

        let inner = BufferPoolInner {
            config,
            size_classes: Mutex::new(size_classes),
            stats: Mutex::new(MemoryStats::default()),
        };
        Self {
            inner: Arc::new(inner),
        }
    }

    /// Acquire a buffer of at least the given size
    ///
    /// Returns a pooled buffer or an error if the size is too large.
    pub fn acquire(&self, size: usize) -> Result<PooledBuffer, Error> {
        if size > self.inner.config.max_buffer_size {
            return Err(Error::invalid_input(format!(
                "requested size {} exceeds max buffer size {}",
                size, self.inner.config.max_buffer_size
            )));
        }

        // Try to get from pool
        let buffer = self
            .inner
            .try_acquire(size)
            .unwrap_or_else(|| self.inner.allocate_new(size));

        Ok(PooledBuffer::new(buffer, self.inner.clone()))
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        self.inner.stats.lock().clone()
    }

    /// Clear all pooled buffers
    pub fn clear(&self) {
        let mut classes = self.inner.size_classes.lock();
        for class in classes.iter_mut() {
            class.clear();
        }
    }

    /// Pre-allocate buffers in the pool
    pub fn preallocate(&self, count: usize, size: usize) -> Result<(), Error> {
        if size > self.inner.config.max_buffer_size {
            return Err(Error::invalid_input(format!(
                "requested size {} exceeds max buffer size {}",
                size, self.inner.config.max_buffer_size
            )));
        }

        let actual_size = size.clamp(
            self.inner.config.min_buffer_size,
            self.inner.config.max_buffer_size,
        );
        let size_idx = size_class_index(actual_size).unwrap_or(0);

        if size_idx >= NUM_SIZE_CLASSES {
            return Ok(());
        }

        let mut classes = self.inner.size_classes.lock();

        for _ in 0..count {
            // Count total while holding lock
            let current_total: usize = classes.iter().map(|v| v.len()).sum();
            if current_total >= self.inner.config.max_buffers {
                break;
            }
            classes[size_idx].push(Bytes::from(vec![0u8; actual_size]));
        }
        Ok(())
    }

    /// Get the number of buffers currently in the pool
    pub fn pooled_count(&self) -> usize {
        self.inner.total_pooled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_create() {
        let pool = BufferPool::new();
        assert_eq!(pool.pooled_count(), 0);
    }

    #[test]
    fn test_buffer_pool_acquire() {
        let pool = BufferPool::new();
        let buffer = pool.acquire(4096).unwrap();
        assert_eq!(buffer.len(), 4096);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_buffer_pool_acquire_too_large() {
        let pool = BufferPool::with_config(BufferPoolConfig {
            max_buffer_size: 4096,
            ..Default::default()
        });
        assert!(pool.acquire(8192).is_err());
    }

    #[test]
    fn test_buffer_pool_reuse() {
        let pool = BufferPool::new();
        let buffer1 = pool.acquire(4096).unwrap();
        // Drop returns to pool
        drop(buffer1);
        // Next acquire might reuse (implementation dependent)
        let _buffer2 = pool.acquire(4096).unwrap();
    }

    #[test]
    fn test_buffer_pool_preallocate() {
        let pool = BufferPool::new();
        pool.preallocate(10, 4096).unwrap();
        assert_eq!(pool.pooled_count(), 10);
    }

    #[test]
    fn test_buffer_pool_clear() {
        let pool = BufferPool::new();
        pool.preallocate(10, 4096).unwrap();
        assert_eq!(pool.pooled_count(), 10);
        pool.clear();
        assert_eq!(pool.pooled_count(), 0);
    }

    #[test]
    fn test_buffer_pool_stats() {
        let pool = BufferPool::new();
        let _buffer = pool.acquire(4096).unwrap();
        let stats = pool.stats();
        assert!(stats.total_allocated >= 4096);
        assert!(stats.allocation_count > 0);
    }

    #[test]
    fn test_pooled_buffer_deref() {
        let pool = BufferPool::new();
        let mut buffer = pool.acquire(4096).unwrap();
        // Can modify through Bytes
        let bytes = std::sync::Arc::from(&buffer.inner);
        assert_eq!(bytes.len(), 4096);
    }
}
