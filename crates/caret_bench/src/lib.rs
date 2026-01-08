// Caret Bench - Benchmark framework and utilities
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

//! Caret Bench - Benchmark framework for Caret
//!
//! This crate provides utilities for running benchmarks and measuring
//! performance of core Caret components.

#![warn(missing_docs)]
#![warn(clippy::all)]

use std::time::{Duration, Instant};

/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchResult {
    /// Name of the benchmark
    pub name: String,
    /// Number of iterations
    pub iterations: usize,
    /// Total duration
    pub duration: Duration,
    /// Average time per iteration
    pub avg_per_iter: Duration,
    /// Minimum time per iteration
    pub min_per_iter: Duration,
    /// Maximum time per iteration
    pub max_per_iter: Duration,
    /// Standard deviation of iteration times (in nanoseconds)
    pub std_dev: f64,
    /// Throughput (iterations per second)
    pub throughput: f64,
}

impl BenchResult {
    /// Create a new benchmark result from raw timing data
    pub fn from_timings(name: impl Into<String>, timings: &[Duration]) -> Self {
        let name = name.into();
        let iterations = timings.len();

        if timings.is_empty() {
            return Self {
                name,
                iterations: 0,
                duration: Duration::ZERO,
                avg_per_iter: Duration::ZERO,
                min_per_iter: Duration::ZERO,
                max_per_iter: Duration::ZERO,
                std_dev: 0.0,
                throughput: 0.0,
            };
        }

        let total: Duration = timings.iter().sum();
        let avg = total / iterations as u32;
        let min = *timings.iter().min().unwrap();
        let max = *timings.iter().max().unwrap();

        // Calculate standard deviation
        let avg_nanos = avg.as_nanos() as f64;
        let variance = timings.iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - avg_nanos;
                diff * diff
            })
            .sum::<f64>() / iterations as f64;
        let std_dev = variance.sqrt();

        // Calculate throughput (iterations per second)
        let throughput = if total.as_secs_f64() > 0.0 {
            iterations as f64 / total.as_secs_f64()
        } else {
            0.0
        };

        Self {
            name,
            iterations,
            duration: total,
            avg_per_iter: avg,
            min_per_iter: min,
            max_per_iter: max,
            std_dev,
            throughput,
        }
    }

    /// Format the result as a human-readable string
    pub fn format_human(&self) -> String {
        format!(
            "{}:\n  Iterations: {}\n  Total: {:?}\n  Avg/iter: {:?}\n  Min/iter: {:?}\n  Max/iter: {:?}\n  Std dev: {:.2} ns\n  Throughput: {:.2} iter/s",
            self.name,
            self.iterations,
            self.duration,
            self.avg_per_iter,
            self.min_per_iter,
            self.max_per_iter,
            self.std_dev,
            self.throughput,
        )
    }

    /// Format the result as JSON
    pub fn format_json(&self) -> String {
        format!(
            r#"{{"name":"{}","iterations":{},"duration_ns":{},"avg_ns":{},"min_ns":{},"max_ns":{},"std_dev":{},"throughput":{}}}"#,
            self.name,
            self.iterations,
            self.duration.as_nanos(),
            self.avg_per_iter.as_nanos(),
            self.min_per_iter.as_nanos(),
            self.max_per_iter.as_nanos(),
            self.std_dev,
            self.throughput,
        )
    }
}

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchConfig {
    /// Number of warmup iterations (not counted in results)
    pub warmup_iters: usize,
    /// Number of measured iterations
    pub measure_iters: usize,
    /// Whether to print detailed results
    pub verbose: bool,
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            warmup_iters: 10,
            measure_iters: 100,
            verbose: false,
        }
    }
}

impl BenchConfig {
    /// Create a new benchmark configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of warmup iterations
    pub fn with_warmup(mut self, iters: usize) -> Self {
        self.warmup_iters = iters;
        self
    }

    /// Set the number of measured iterations
    pub fn with_measurements(mut self, iters: usize) -> Self {
        self.measure_iters = iters;
        self
    }

    /// Enable verbose output
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }
}

/// Run a benchmark function and return the result
pub fn run_bench<F>(name: impl Into<String>, mut f: F) -> BenchResult
where
    F: FnMut() -> (),
{
    run_bench_with_config(name, f, BenchConfig::default())
}

/// Run a benchmark function with custom configuration
pub fn run_bench_with_config<F>(name: impl Into<String>, mut f: F, config: BenchConfig) -> BenchResult
where
    F: FnMut() -> (),
{
    let name = name.into();

    // Warmup iterations
    for _ in 0..config.warmup_iters {
        f();
    }

    // Measured iterations
    let mut timings = Vec::with_capacity(config.measure_iters);
    for _ in 0..config.measure_iters {
        let start = Instant::now();
        f();
        timings.push(start.elapsed());
    }

    BenchResult::from_timings(name, &timings)
}

/// Run a benchmark function that returns a value
pub fn run_bench_ret<F, T>(name: impl Into<String>, mut f: F) -> BenchResult
where
    F: FnMut() -> T,
{
    run_bench_ret_with_config(name, f, BenchConfig::default())
}

/// Run a benchmark function that returns a value with custom configuration
pub fn run_bench_ret_with_config<F, T>(name: impl Into<String>, mut f: F, config: BenchConfig) -> BenchResult
where
    F: FnMut() -> T,
{
    let name = name.into();

    // Warmup iterations
    for _ in 0..config.warmup_iters {
        let _ = f();
    }

    // Measured iterations
    let mut timings = Vec::with_capacity(config.measure_iters);
    for _ in 0..config.measure_iters {
        let start = Instant::now();
        let _ = f();
        timings.push(start.elapsed());
    }

    BenchResult::from_timings(name, &timings)
}

/// Benchmark group for running multiple related benchmarks
pub struct BenchGroup {
    name: String,
    results: Vec<BenchResult>,
}

impl BenchGroup {
    /// Create a new benchmark group
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            results: Vec::new(),
        }
    }

    /// Run a benchmark and add it to the group
    pub fn bench<F>(mut self, name: impl Into<String>, f: F) -> Self
    where
        F: FnMut() -> (),
    {
        let result = run_bench(name, f);
        if result.iterations > 0 {
            self.results.push(result);
        }
        self
    }

    /// Run a benchmark with config and add it to the group
    pub fn bench_with_config<F>(mut self, name: impl Into<String>, f: F, config: BenchConfig) -> Self
    where
        F: FnMut() -> (),
    {
        let result = run_bench_with_config(name, f, config);
        if result.iterations > 0 {
            self.results.push(result);
        }
        self
    }

    /// Print all results in the group
    pub fn print(&self) {
        println!("=== {} ===", self.name);
        for result in &self.results {
            println!("{}", result.format_human());
            println!();
        }
    }

    /// Get all results
    pub fn results(&self) -> &[BenchResult] {
        &self.results
    }

    /// Find a result by name
    pub fn get_result(&self, name: &str) -> Option<&BenchResult> {
        self.results.iter().find(|r| r.name == name)
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemStats {
    /// Current memory usage in bytes
    pub current: usize,
    /// Peak memory usage in bytes
    pub peak: usize,
}

impl MemStats {
    /// Get current memory statistics (platform-specific)
    #[cfg(target_os = "linux")]
    pub fn current() -> Self {
        use std::fs;

        let current = fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|content| {
                content.lines()
                    .find(|line| line.starts_with("VmRSS:"))
                    .and_then(|line| {
                        line.split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse::<usize>().ok())
                    })
            })
            .unwrap_or(0);

        // Peak is harder to get accurately, use current as approximation
        Self {
            current: current * 1024, // Convert KB to bytes
            peak: current * 1024,
        }
    }

    /// Get current memory statistics (fallback implementation)
    #[cfg(not(target_os = "linux"))]
    pub fn current() -> Self {
        Self {
            current: 0,
            peak: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_result_from_timings() {
        let timings = vec![
            Duration::from_nanos(100),
            Duration::from_nanos(200),
            Duration::from_nanos(300),
        ];
        let result = BenchResult::from_timings("test", &timings);

        assert_eq!(result.name, "test");
        assert_eq!(result.iterations, 3);
        assert_eq!(result.avg_per_iter.as_nanos(), 200);
        assert_eq!(result.min_per_iter.as_nanos(), 100);
        assert_eq!(result.max_per_iter.as_nanos(), 300);
    }

    #[test]
    fn test_bench_result_empty() {
        let timings: Vec<Duration> = vec![];
        let result = BenchResult::from_timings("test", &timings);

        assert_eq!(result.iterations, 0);
    }

    #[test]
    fn test_bench_group() {
        let group = BenchGroup::new("test_group")
            .bench("bench1", || drop(std::hint::black_box(42)))
            .bench("bench2", || drop(std::hint::black_box(42 + 42)));

        assert_eq!(group.results().len(), 2);
        assert!(group.get_result("bench1").is_some());
        assert!(group.get_result("bench2").is_some());
        assert!(group.get_result("nonexistent").is_none());
    }

    #[test]
    fn test_bench_config() {
        let config = BenchConfig::new()
            .with_warmup(5)
            .with_measurements(50)
            .verbose();

        assert_eq!(config.warmup_iters, 5);
        assert_eq!(config.measure_iters, 50);
        assert!(config.verbose);
    }
}
