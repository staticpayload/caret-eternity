// Caret Bench - Scheduler benchmarks
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use caret_bench::{run_bench, BenchConfig, BenchGroup};
use caret_sched::{Executor, ExecutorConfig, PassthroughNode, NodeId};
use caret_core::Packet;

fn bench_executor_tick_empty() {
    let mut executor = Executor::with_default_config();
    executor.start().unwrap();

    run_bench("tick_empty", || {
        let _ = executor.tick_once();
    });
}

fn bench_executor_tick_single_node() {
    let mut executor = Executor::with_default_config();
    let node = Box::new(PassthroughNode::new());
    executor.add_node(1, node).unwrap();
    executor.start().unwrap();

    run_bench("tick_single_node", || {
        let _ = executor.tick_once();
    });
}

fn bench_executor_tick_multiple_nodes() {
    let mut executor = Executor::with_default_config();

    // Add 10 passthrough nodes
    for i in 0..10 {
        let node = Box::new(PassthroughNode::new());
        executor.add_node(i, node).unwrap();
    }

    executor.start().unwrap();

    run_bench("tick_10_nodes", || {
        let _ = executor.tick_once();
    });
}

fn bench_executor_tick_many_nodes() {
    let mut executor = Executor::with_default_config();

    // Add 100 passthrough nodes
    for i in 0..100 {
        let node = Box::new(PassthroughNode::new());
        executor.add_node(i, node).unwrap();
    }

    executor.start().unwrap();

    run_bench("tick_100_nodes", || {
        let _ = executor.tick_once();
    });
}

fn bench_executor_add_node() {
    run_bench("add_node", || {
        let mut executor = Executor::with_default_config();
        let node = Box::new(PassthroughNode::new());
        let _ = executor.add_node(1, node);
    });
}

fn bench_executor_node_lookup() {
    let mut executor = Executor::with_default_config();
    let mut node_ids = Vec::new();

    // Add 100 nodes
    for i in 0..100 {
        let node = Box::new(PassthroughNode::new());
        let id = executor.add_node(i, node).unwrap();
        node_ids.push(id);
    }

    // Benchmark looking up a node
    run_bench("node_lookup", || {
        let _ = executor.node(node_ids[50]);
    });
}

fn bench_executor_get_all_node_ids() {
    let mut executor = Executor::with_default_config();

    // Add 100 nodes
    for i in 0..100 {
        let node = Box::new(PassthroughNode::new());
        executor.add_node(i, node).unwrap();
    }

    run_bench("get_all_node_ids", || {
        let _ = executor.node_ids();
    });
}

fn bench_executor_comparison() {
    let config = BenchConfig::new()
        .with_warmup(50)
        .with_measurements(500);

    let mut group = BenchGroup::new("executor_node_count");

    for node_count in [1, 10, 50, 100] {
        let mut executor = Executor::with_default_config();

        for i in 0..node_count {
            let node = Box::new(PassthroughNode::new());
            executor.add_node(i, node).unwrap();
        }

        executor.start().unwrap();

        group = group.bench_with_config(
            format!("tick_{}_nodes", node_count),
            || {
                let _ = executor.tick_once();
            },
            config.clone(),
        );
    }

    group.print();
}

fn main() {
    println!("=== Scheduler/Executor Benchmarks ===\n");

    let config = BenchConfig::new()
        .with_warmup(100)
        .with_measurements(1000);

    // Empty executor tick
    let mut executor_empty = Executor::with_default_config();
    executor_empty.start().unwrap();
    let result_empty = run_bench_with_config("tick_empty", || {
        let _ = executor_empty.tick_once();
    }, config.clone());
    println!("{}", result_empty.format_human());
    println!();

    // Single node tick
    let mut executor_single = Executor::with_default_config();
    executor_single.add_node(0, Box::new(PassthroughNode::new())).unwrap();
    executor_single.start().unwrap();
    let result_single = run_bench_with_config("tick_single_node", || {
        let _ = executor_single.tick_once();
    }, config.clone());
    println!("{}", result_single.format_human());
    println!();

    // Multiple nodes comparison
    println!("--- Node Count Scaling ---");
    bench_executor_comparison();
    println!();

    // Add node benchmark
    let result_add = run_bench_with_config("add_node", || {
        let mut executor = Executor::with_default_config();
        let node = Box::new(PassthroughNode::new());
        let _ = executor.add_node(1, node);
    }, config.clone());
    println!("{}", result_add.format_human());
    println!();

    // Node lookup benchmark
    let mut executor_lookup = Executor::with_default_config();
    let mut node_ids = Vec::new();
    for i in 0..100 {
        let node = Box::new(PassthroughNode::new());
        let id = executor_lookup.add_node(i, node).unwrap();
        node_ids.push(id);
    }
    let result_lookup = run_bench_with_config("node_lookup", || {
        let _ = executor_lookup.node(node_ids[50]);
    }, config.clone());
    println!("{}", result_lookup.format_human());
    println!();

    // Get all node IDs benchmark
    let result_get_ids = run_bench_with_config("get_all_node_ids", || {
        let _ = executor_lookup.node_ids();
    }, config.clone());
    println!("{}", result_get_ids.format_human());
}
