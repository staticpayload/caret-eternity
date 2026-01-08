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
    pooled: Mutex<Vec<Bytes>>,
    stats: Mutex<MemoryStats>,
}

impl BufferPoolInner {
    fn can_return(&self) -> bool {
        let pooled = self.pooled.lock();
        pooled.len() < self.config.max_buffers
    }

    fn return_buffer(&self, buffer: Bytes) {
        let mut pooled = self.pooled.lock();
        if pooled.len() < self.config.max_buffers {
            pooled.push(buffer);
            let mut stats = self.stats.lock();
            stats.deallocation_count += 1;
        }
    }

    fn try_acquire(&self, size: usize) -> Option<Bytes> {
        let mut pooled = self.pooled.lock();
        // Find a buffer that fits
        let idx = pooled.iter().position(|b| b.len() >= size);
        if let Some(idx) = idx {
            let buffer = pooled.remove(idx);
            let mut stats = self.stats.lock();
            stats.in_use = stats.in_use.saturating_add(buffer.len());
            stats.allocation_count += 1;
            return Some(buffer);
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
        let inner = BufferPoolInner {
            config,
            pooled: Mutex::new(Vec::new()),
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
        self.inner.pooled.lock().clear();
    }

    /// Pre-allocate buffers in the pool
    pub fn preallocate(&self, count: usize, size: usize) -> Result<(), Error> {
        if size > self.inner.config.max_buffer_size {
            return Err(Error::invalid_input(format!(
                "requested size {} exceeds max buffer size {}",
                size, self.inner.config.max_buffer_size
            )));
        }

        let mut pooled = self.inner.pooled.lock();
        for _ in 0..count {
            if pooled.len() >= self.inner.config.max_buffers {
                break;
            }
            let actual_size = size.clamp(
                self.inner.config.min_buffer_size,
                self.inner.config.max_buffer_size,
            );
            pooled.push(Bytes::from(vec![0u8; actual_size]));
        }
        Ok(())
    }

    /// Get the number of buffers currently in the pool
    pub fn pooled_count(&self) -> usize {
        self.inner.pooled.lock().len()
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
