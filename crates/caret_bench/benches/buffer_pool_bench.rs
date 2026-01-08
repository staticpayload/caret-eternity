// Caret Bench - Buffer pool benchmarks
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use caret_bench::{run_bench, run_bench_ret, BenchConfig, BenchGroup};
use caret_buffers::{BufferPool, BufferPoolConfig};

#[divan::bench]
fn bench_buffer_pool_acquire_4k() {
    let pool = BufferPool::new();
    run_bench("acquire_4k", || {
        let _buffer = pool.acquire(4096).unwrap();
    });
}

fn bench_buffer_pool_acquire_64k() {
    let pool = BufferPool::new();
    run_bench("acquire_64k", || {
        let _buffer = pool.acquire(65536).unwrap();
    });
}

fn bench_buffer_pool_acquire_and_drop() {
    let pool = BufferPool::new();
    run_bench("acquire_and_drop", || {
        let buffer = pool.acquire(4096).unwrap();
        std::hint::black_box(buffer);
    });
}

fn bench_buffer_pool_pooled_reuse() {
    let pool = BufferPool::new();
    // Pre-populate the pool
    pool.preallocate(100, 4096).unwrap();

    run_bench("pooled_reuse", || {
        let buffer = pool.acquire(4096).unwrap();
        std::hint::black_box(buffer.len());
    });
}

fn bench_buffer_pool_vs_direct_alloc() {
    let mut group = BenchGroup::new("buffer_pool_comparison");

    // With pool
    let pool = BufferPool::new();
    pool.preallocate(100, 4096).unwrap();
    group = group.bench("with_pool", || {
        let _buffer = pool.acquire(4096).unwrap();
    });

    // Direct allocation
    group = group.bench("direct_alloc", || {
        let _buffer = vec![0u8; 4096];
    });

    group.print();
}

fn main() {
    println!("=== Buffer Pool Benchmarks ===\n");

    let config = BenchConfig::new()
        .with_warmup(100)
        .with_measurements(1000);

    // Individual benchmarks
    let result_acquire_4k = run_bench_ret_with_config("acquire_4k", || {
        let pool = BufferPool::new();
        pool.acquire(4096).unwrap()
    }, config.clone());
    println!("{}", result_acquire_4k.format_human());
    println!();

    let result_acquire_64k = run_bench_ret_with_config("acquire_64k", || {
        let pool = BufferPool::new();
        pool.acquire(65536).unwrap()
    }, config.clone());
    println!("{}", result_acquire_64k.format_human());
    println!();

    // Pooled reuse benchmark
    let pool_reuse = BufferPool::new();
    pool_reuse.preallocate(100, 4096).unwrap();
    let result_reuse = run_bench_with_config("pooled_reuse", || {
        let _buffer = pool_reuse.acquire(4096).unwrap();
    }, config.clone());
    println!("{}", result_reuse.format_human());
    println!();

    // Comparison group
    bench_buffer_pool_vs_direct_alloc();
}
