// Caret Bench - Bounded queue benchmarks
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use caret_bench::{run_bench, BenchConfig, BenchGroup};
use caret_buffers::{BoundedQueue, OverflowPolicy};

fn bench_queue_push_pop() {
    let queue = BoundedQueue::new(1000);
    run_bench("push_pop", || {
        queue.push(42).unwrap();
        let _ = queue.pop();
    });
}

fn bench_queue_push_only() {
    let queue = BoundedQueue::new(1000);
    run_bench("push_only", || {
        queue.push(42).unwrap();
    });
}

fn bench_queue_pop_only() {
    let queue = BoundedQueue::new(1000);
    // Pre-fill the queue
    for i in 0..1000 {
        queue.push(i).unwrap();
    }

    run_bench("pop_only", || {
        let _ = queue.pop();
    });
}

fn bench_queue_overflow_drop_oldest() {
    let queue = BoundedQueue::with_policy(100, OverflowPolicy::DropOldest);
    // Fill the queue
    for i in 0..100 {
        queue.push(i).unwrap();
    }

    run_bench("overflow_drop_oldest", || {
        queue.push(42).unwrap();
    });
}

fn bench_queue_overflow_drop_newest() {
    let queue = BoundedQueue::with_policy(100, OverflowPolicy::DropNewest);
    // Fill the queue
    for i in 0..100 {
        queue.push(i).unwrap();
    }

    run_bench("overflow_drop_newest", || {
        queue.push(42).unwrap();
    });
}

fn bench_queue_overflow_error() {
    let queue = BoundedQueue::with_policy(100, OverflowPolicy::Error);
    // Fill the queue
    for i in 0..100 {
        queue.push(i).unwrap();
    }

    run_bench("overflow_error", || {
        let _ = queue.push(42);
    });
}

fn bench_queue_parallel_push_pop() {
    use std::thread;

    let queue = BoundedQueue::new(10000);
    let queue_clone = queue.clone();

    let handle = thread::spawn(move || {
        let mut count = 0;
        for i in 0..100000 {
            if queue_clone.push(i).is_ok() {
                count += 1;
            }
        }
        count
    });

    run_bench("parallel_pop", || {
        let _ = queue.pop();
    });

    let _ = handle.join();
}

fn bench_queue_comparison() {
    let mut group = BenchGroup::new("queue_overflow_policies");

    let queue_drop_oldest = BoundedQueue::with_policy(100, OverflowPolicy::DropOldest);
    for i in 0..100 {
        queue_drop_oldest.push(i).unwrap();
    }
    group = group.bench("drop_oldest", || {
        queue_drop_oldest.push(42).unwrap();
    });

    let queue_drop_newest = BoundedQueue::with_policy(100, OverflowPolicy::DropNewest);
    for i in 0..100 {
        queue_drop_newest.push(i).unwrap();
    }
    group = group.bench("drop_newest", || {
        queue_drop_newest.push(42).unwrap();
    });

    let queue_error = BoundedQueue::with_policy(100, OverflowPolicy::Error);
    for i in 0..99 {
        queue_error.push(i).unwrap();
    }
    group = group.bench("error_policy", || {
        let _ = queue_error.push(42);
    });

    group.print();
}

fn main() {
    println!("=== Bounded Queue Benchmarks ===\n");

    let config = BenchConfig::new()
        .with_warmup(100)
        .with_measurements(10000);

    // Push/Pop benchmark
    let queue_push_pop = BoundedQueue::new(1000);
    let result_push_pop = run_bench_with_config("push_pop", || {
        queue_push_pop.push(42).unwrap();
        let _ = queue_push_pop.pop();
    }, config.clone());
    println!("{}", result_push_pop.format_human());
    println!();

    // Push only benchmark
    let queue_push = BoundedQueue::new(10000);
    let result_push = run_bench_with_config("push_only", || {
        queue_push.push(42).unwrap();
    }, config.clone());
    println!("{}", result_push.format_human());
    println!();

    // Pop only benchmark
    let queue_pop = BoundedQueue::new(10000);
    for i in 0..10000 {
        queue_pop.push(i).unwrap();
    }
    let result_pop = run_bench_with_config("pop_only", || {
        let _ = queue_pop.pop();
    }, config.clone());
    println!("{}", result_pop.format_human());
    println!();

    // Overflow policy comparison
    println!("--- Overflow Policy Comparison ---");
    bench_queue_comparison();
}
