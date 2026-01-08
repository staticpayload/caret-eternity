// Caret Buffers - Buffer and queue performance benchmarks
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

//! Buffer pool and queue performance benchmarks
//!
//! Run with: cargo run --release --bin buffer_perf

use caret_buffers::{BoundedQueue, BufferPool, OverflowPolicy};
use std::time::Instant;

fn main() {
    println!("Caret Buffers Performance Benchmarks");
    println!("=====================================\\n");

    // Buffer pool benchmarks
    bench_buffer_acquire();
    bench_buffer_acquire_release();

    // Queue benchmarks
    bench_queue_push_pop();
    bench_queue_len();
    bench_queue_contention();
}

fn bench_buffer_acquire() {
    let pool = BufferPool::new();

    // Warmup
    for _ in 0..100 {
        let _ = pool.acquire(4096);
    }

    let start = Instant::now();
    for _ in 0..100_000 {
        let _ = pool.acquire(4096);
    }
    let elapsed = start.elapsed();

    println!("buffer_acquire (100k iterations): {:?}", elapsed);
    println!("  per op: {:?}\\n", elapsed / 100_000);
}

fn bench_buffer_acquire_release() {
    let pool = BufferPool::new();

    // Pre-allocate some buffers
    pool.preallocate(100, 4096).unwrap();

    // Warmup
    for _ in 0..100 {
        let _ = pool.acquire(4096);
    }

    let start = Instant::now();
    for _ in 0..100_000 {
        let buffer = pool.acquire(4096).unwrap();
        drop(buffer);
    }
    let elapsed = start.elapsed();

    println!("buffer_acquire_release (100k iterations): {:?}", elapsed);
    println!("  per op: {:?}\\n", elapsed / 100_000);
}

fn bench_queue_push_pop() {
    let queue: BoundedQueue<i32> = BoundedQueue::new(1000);

    // Warmup
    for i in 0..100 {
        queue.try_push(i);
        let _ = queue.pop();
    }

    let start = Instant::now();
    for i in 0..100_000 {
        queue.try_push(i);
        let _ = queue.pop();
    }
    let elapsed = start.elapsed();

    println!("queue_push_pop (100k iterations): {:?}", elapsed);
    println!("  per op: {:?}\\n", elapsed / 100_000);
}

fn bench_queue_len() {
    let queue: BoundedQueue<i32> = BoundedQueue::new(1000);

    // Pre-fill
    for i in 0..100 {
        queue.try_push(i);
    }

    // Warmup
    for _ in 0..1000 {
        let _ = queue.len();
    }

    let start = Instant::now();
    for _ in 0..1_000_000 {
        let _ = queue.len();
    }
    let elapsed = start.elapsed();

    println!("queue_len (1M iterations): {:?}", elapsed);
    println!("  per op: {:?}\\n", elapsed / 1_000_000);
}

fn bench_queue_contention() {
    let queue: BoundedQueue<i32> = BoundedQueue::with_policy(1000, OverflowPolicy::DropNewest);

    // Single-threaded contention simulation
    let start = Instant::now();
    for i in 0..100_000 {
        if queue.try_push(i) {
            let _ = queue.pop();
        }
    }
    let elapsed = start.elapsed();

    println!("queue_contention (100k iterations): {:?}", elapsed);
    println!("  per op: {:?}\\n", elapsed / 100_000);
}
