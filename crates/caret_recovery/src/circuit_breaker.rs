// Caret Recovery - Circuit breaker
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Configuration for a circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before tripping
    pub failure_threshold: usize,
    /// Number of consecutive successes before closing
    pub success_threshold: usize,
    /// Timeout in milliseconds before attempting to close circuit
    pub timeout_ms: u64,
    /// Half-open max calls
    pub half_open_max_calls: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout_ms: 60000, // 1 minute
            half_open_max_calls: 3,
        }
    }
}

impl CircuitBreakerConfig {
    /// Create a new circuit breaker config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the failure threshold
    pub fn with_failure_threshold(mut self, threshold: usize) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set the success threshold
    pub fn with_success_threshold(mut self, threshold: usize) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

/// State of the circuit breaker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests are allowed
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker for preventing cascading failures
pub struct CircuitBreaker {
    /// Name of this circuit breaker
    name: String,
    /// Current state
    state: AtomicUsize, // Uses CircuitState as usize
    /// Failure count
    failures: AtomicU64,
    /// Success count
    successes: AtomicU64,
    /// Consecutive failures
    consecutive_failures: AtomicUsize,
    /// Consecutive successes
    consecutive_successes: AtomicUsize,
    /// Last failure time
    last_failure_time: Arc<parking_lot::Mutex<Option<Instant>>>,
    /// Half-open call count
    half_open_calls: AtomicUsize,
    /// Configuration
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            name,
            state: AtomicUsize::new(CircuitState::Closed as usize),
            failures: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            consecutive_failures: AtomicUsize::new(0),
            consecutive_successes: AtomicUsize::new(0),
            last_failure_time: Arc::new(parking_lot::Mutex::new(None)),
            half_open_calls: AtomicUsize::new(0),
            config,
        }
    }

    /// Get the current state
    pub fn state(&self) -> CircuitState {
        match self.state.load(Ordering::Relaxed) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }

    /// Set the state
    fn set_state(&self, state: CircuitState) {
        self.state.store(state as usize, Ordering::Relaxed);
    }

    /// Check if a request is allowed
    pub fn allow_request(&self) -> bool {
        match self.state() {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has elapsed
                let last_failure = self.last_failure_time.lock();
                if let Some(last) = *last_failure {
                    let elapsed = last.elapsed();
                    if elapsed >= Duration::from_millis(self.config.timeout_ms) {
                        drop(last_failure);
                        self.set_state(CircuitState::HalfOpen);
                        self.half_open_calls.store(0, Ordering::Relaxed);
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => {
                // Limit calls in half-open state
                self.half_open_calls.fetch_add(1, Ordering::Relaxed) <= self.config.half_open_max_calls
            }
        }
    }

    /// Record a successful call
    pub fn record_success(&self) {
        self.successes.fetch_add(1, Ordering::Relaxed);
        self.consecutive_failures.store(0, Ordering::Relaxed);

        let consec = self.consecutive_successes.fetch_add(1, Ordering::Relaxed) + 1;

        match self.state() {
            CircuitState::HalfOpen => {
                if consec >= self.config.success_threshold {
                    self.set_state(CircuitState::Closed);
                    self.consecutive_successes.store(0, Ordering::Relaxed);
                }
            }
            CircuitState::Open | CircuitState::Closed => {
                // In closed state, reset on success
                if consec >= self.config.success_threshold {
                    self.consecutive_successes.store(0, Ordering::Relaxed);
                }
            }
        }
    }

    /// Record a failed call
    pub fn record_failure(&self) {
        self.failures.fetch_add(1, Ordering::Relaxed);
        self.consecutive_successes.store(0, Ordering::Relaxed);

        let consec = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;

        match self.state() {
            CircuitState::Closed | CircuitState::HalfOpen => {
                if consec >= self.config.failure_threshold {
                    self.set_state(CircuitState::Open);
                    *self.last_failure_time.lock() = Some(Instant::now());
                    self.consecutive_failures.store(0, Ordering::Relaxed);
                }
            }
            CircuitState::Open => {
                // Already open, update failure time
                *self.last_failure_time.lock() = Some(Instant::now());
            }
        }
    }

    /// Get the number of failures
    pub fn failure_count(&self) -> u64 {
        self.failures.load(Ordering::Relaxed)
    }

    /// Get the number of successes
    pub fn success_count(&self) -> u64 {
        self.successes.load(Ordering::Relaxed)
    }

    /// Get the name of this circuit breaker
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Reset the circuit breaker to closed state
    pub fn reset(&self) {
        self.set_state(CircuitState::Closed);
        self.failures.store(0, Ordering::Relaxed);
        self.successes.store(0, Ordering::Relaxed);
        self.consecutive_failures.store(0, Ordering::Relaxed);
        self.consecutive_successes.store(0, Ordering::Relaxed);
        *self.last_failure_time.lock() = None;
        self.half_open_calls.store(0, Ordering::Relaxed);
    }

    /// Get the configuration
    pub fn config(&self) -> &CircuitBreakerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closed_by_default() {
        let cb = CircuitBreaker::new(
            "test".to_string(),
            CircuitBreakerConfig::default(),
        );
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let cb = CircuitBreaker::new("test".to_string(), config);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_closes_on_successes() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout_ms: 50,
            ..Default::default()
        };
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Trip the circuit
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for timeout
        std::thread::sleep(std::time::Duration::from_millis(60));

        // First call should succeed (half-open)
        assert!(cb.allow_request());
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Second success should close
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let cb = CircuitBreaker::new(
            "test".to_string(),
            CircuitBreakerConfig::default(),
        );

        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();

        assert!(cb.failure_count() > 0);
        assert_eq!(cb.state(), CircuitState::Open);

        cb.reset();

        assert_eq!(cb.failure_count(), 0);
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_config() {
        let config = CircuitBreakerConfig::new()
            .with_failure_threshold(10)
            .with_success_threshold(5)
            .with_timeout(5000);

        assert_eq!(config.failure_threshold, 10);
        assert_eq!(config.success_threshold, 5);
        assert_eq!(config.timeout_ms, 5000);
    }
}
