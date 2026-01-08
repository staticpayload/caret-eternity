// Caret Sched - Executor performance benchmarks
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

//! Executor performance benchmark
//!
//! Run with: cargo run --release --bin executor_perf

use std::time::Instant;

fn main() {
    println!("Executor Performance Benchmarks");
    println!("=================================\n");

    // Benchmark tick with 10 nodes
    bench_tick_10_nodes();

    // Benchmark tick with 100 nodes
    bench_tick_100_nodes();

    // Benchmark tick overhead with data
    bench_tick_with_data();
}

fn bench_tick_10_nodes() {
    let mut executor = caret_sched::Executor::with_default_config();
    for i in 0..10 {
        let _ = executor.add_node(i, Box::new(caret_sched::PassthroughNode::new("test")));
    }
    executor.start().unwrap();

    // Warmup
    for _ in 0..100 {
        let _ = executor.tick_once();
    }

    let start = Instant::now();
    for _ in 0..10000 {
        let _ = executor.tick_once();
    }
    let elapsed = start.elapsed();

    println!("tick_10_nodes (10k iterations): {:?}", elapsed);
    println!("  per tick: {:?}\n", elapsed / 10000);
}

fn bench_tick_100_nodes() {
    let mut executor = caret_sched::Executor::with_default_config();
    for i in 0..100 {
        let _ = executor.add_node(i, Box::new(caret_sched::PassthroughNode::new("test")));
    }
    executor.start().unwrap();

    // Warmup
    for _ in 0..100 {
        let _ = executor.tick_once();
    }

    let start = Instant::now();
    for _ in 0..10000 {
        let _ = executor.tick_once();
    }
    let elapsed = start.elapsed();

    println!("tick_100_nodes (10k iterations): {:?}", elapsed);
    println!("  per tick: {:?}\n", elapsed / 10000);
}

fn bench_tick_with_data() {
    let mut executor = caret_sched::Executor::with_default_config();

    // Create nodes with ports
    for i in 0..10 {
        let node_id = executor.add_node(i, Box::new(caret_sched::PassthroughNode::new("test"))).unwrap();
        let instance = executor.node(node_id).unwrap();
        let mut instance_locked = instance.lock();
        instance_locked.ports.add_input("in", 1024).unwrap();
        instance_locked.ports.add_output("out").unwrap();
    }

    executor.start().unwrap();

    // Warmup
    for _ in 0..100 {
        let _ = executor.tick_once();
    }

    let start = Instant::now();
    for _ in 0..10000 {
        let _ = executor.tick_once();
    }
    let elapsed = start.elapsed();

    println!("tick_with_ports_10_nodes (10k iterations): {:?}", elapsed);
    println!("  per tick: {:?}\n", elapsed / 10000);
}
