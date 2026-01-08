// Caret Buffers - Buffer pools and memory management for Caret
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

#![warn(missing_docs)]
#![warn(clippy::all)]

mod pool;
mod queue;

pub use pool::{BufferPool, BufferPoolConfig, PooledBuffer};
pub use queue::{BoundedQueue, OverflowPolicy};

/// Memory statistics for buffer pool
#[derive(Clone, Debug, Default)]
pub struct MemoryStats {
    /// Total allocated bytes
    pub total_allocated: usize,
    /// Total bytes in use
    pub in_use: usize,
    /// Total bytes available
    pub available: usize,
    /// Number of allocations
    pub allocation_count: u64,
    /// Number of deallocations
    pub deallocation_count: u64,
}
