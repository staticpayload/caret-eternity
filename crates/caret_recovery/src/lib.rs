// Caret Recovery - Error recovery and resilience
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

//! Caret Recovery - Error recovery and resilience mechanisms
//!
//! This crate provides error recovery strategies, circuit breakers,
//! and resilience patterns for Caret pipelines.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod circuit_breaker;
mod error;
mod policy;
mod retry;
mod strategy;

pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
pub use error::{RecoveryError, RecoveryResult};
pub use policy::{RecoveryPolicy, RecoveryPolicyBuilder};
pub use retry::{RetryConfig, RetryStrategy, BackoffStrategy};
pub use strategy::{RecoveryStrategy, RetryAction, NonRetryAction, ErrorHandler};
pub use strategy::RecoveryAction;

use std::sync::Arc;
use parking_lot::Mutex;

/// Recovery manager for coordinating error recovery across a pipeline
pub struct RecoveryManager {
    /// Recovery policy
    policy: RecoveryPolicy,
    /// Circuit breakers by node ID
    circuit_breakers: Arc<Mutex<Vec<(String, Arc<CircuitBreaker>)>>>,
    /// Error statistics
    stats: Arc<Mutex<RecoveryStats>>,
}

/// Statistics for recovery operations
#[derive(Debug, Clone, Default)]
pub struct RecoveryStats {
    /// Total errors encountered
    pub total_errors: u64,
    /// Errors recovered successfully
    pub recovered: u64,
    /// Errors that failed recovery
    pub failed: u64,
    /// Retries attempted
    pub retries: u64,
    /// Fallback activations
    pub fallbacks: u64,
    /// Circuit breaker trips
    pub circuit_trips: u64,
}

impl RecoveryManager {
    /// Create a new recovery manager with the given policy
    pub fn new(policy: RecoveryPolicy) -> Self {
        Self {
            policy,
            circuit_breakers: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(RecoveryStats::default())),
        }
    }

    /// Create with default policy
    pub fn with_default_policy() -> Self {
        Self::new(RecoveryPolicy::default())
    }

    /// Get or create a circuit breaker for a node
    pub fn circuit_breaker(&self, node_id: &str) -> Arc<CircuitBreaker> {
        let breakers = self.circuit_breakers.lock();

        // Check if circuit breaker exists
        if let Some((_, cb)) = breakers.iter().find(|(id, _)| id == node_id) {
            return cb.clone();
        }

        // Create new circuit breaker with policy defaults
        let config = CircuitBreakerConfig {
            failure_threshold: self.policy.failure_threshold,
            success_threshold: self.policy.success_threshold,
            timeout_ms: self.policy.circuit_timeout_ms,
            ..Default::default()
        };

        let cb = Arc::new(CircuitBreaker::new(node_id.to_string(), config));
        drop(breakers);
        let mut breakers = self.circuit_breakers.lock();
        breakers.push((node_id.to_string(), cb.clone()));
        cb
    }

    /// Handle an error and determine the recovery action
    pub fn handle_error(&self, error: &caret_core::Error, context: &ErrorContext) -> RecoveryAction {
        {
            let mut stats = self.stats.lock();
            stats.total_errors += 1;
        }

        // Check if circuit breaker allows execution
        let cb = self.circuit_breaker(&context.node_id);
        if !cb.allow_request() {
            let mut stats = self.stats.lock();
            stats.circuit_trips += 1;
            return RecoveryAction::Skip;
        }

        // Determine recovery strategy based on error kind
        let action = if error.is_retryable() {
            match self.policy.on_retryable_error {
                strategy::RetryAction::Retry => {
                    let mut stats = self.stats.lock();
                    stats.retries += 1;
                    RecoveryAction::Retry { delay_ms: self.policy.initial_retry_delay_ms }
                }
                strategy::RetryAction::Skip => RecoveryAction::Skip,
                strategy::RetryAction::Fail => {
                    let mut stats = self.stats.lock();
                    stats.failed += 1;
                    RecoveryAction::Fail
                }
            }
        } else {
            match self.policy.on_non_retryable_error {
                strategy::NonRetryAction::Fail => {
                    let mut stats = self.stats.lock();
                    stats.failed += 1;
                    RecoveryAction::Fail
                }
                strategy::NonRetryAction::Skip => RecoveryAction::Skip,
                strategy::NonRetryAction::Fallback => {
                    let mut stats = self.stats.lock();
                    stats.fallbacks += 1;
                    RecoveryAction::Fallback
                }
            }
        };

        action
    }

    /// Record a successful recovery
    pub fn record_success(&self, node_id: &str) {
        let cb = self.circuit_breaker(node_id);
        cb.record_success();

        let mut stats = self.stats.lock();
        stats.recovered += 1;
    }

    /// Record a failed recovery
    pub fn record_failure(&self, node_id: &str) {
        let cb = self.circuit_breaker(node_id);
        cb.record_failure();

        let mut stats = self.stats.lock();
        stats.failed += 1;
    }

    /// Get the current statistics
    pub fn stats(&self) -> RecoveryStats {
        self.stats.lock().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.lock() = RecoveryStats::default();
    }

    /// Get the recovery policy
    pub fn policy(&self) -> &RecoveryPolicy {
        &self.policy
    }

    /// Update the recovery policy
    pub fn set_policy(&mut self, policy: RecoveryPolicy) {
        self.policy = policy;
    }
}

/// Context for an error
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Node where the error occurred
    pub node_id: String,
    /// Port where the error occurred (if applicable)
    pub port: Option<String>,
    /// Tick when the error occurred
    pub tick: u64,
    /// Packet being processed (if applicable)
    pub packet_id: Option<String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            port: None,
            tick: 0,
            packet_id: None,
        }
    }

    /// Set the port
    pub fn with_port(mut self, port: impl Into<String>) -> Self {
        self.port = Some(port.into());
        self
    }

    /// Set the tick
    pub fn with_tick(mut self, tick: u64) -> Self {
        self.tick = tick;
        self
    }

    /// Set the packet ID
    pub fn with_packet_id(mut self, packet_id: impl Into<String>) -> Self {
        self.packet_id = Some(packet_id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context() {
        let ctx = ErrorContext::new("node1")
            .with_port("output")
            .with_tick(42)
            .with_packet_id("pkt-123");

        assert_eq!(ctx.node_id, "node1");
        assert_eq!(ctx.port, Some("output".to_string()));
        assert_eq!(ctx.tick, 42);
        assert_eq!(ctx.packet_id, Some("pkt-123".to_string()));
    }

    #[test]
    fn test_recovery_manager() {
        let manager = RecoveryManager::with_default_policy();
        let stats = manager.stats();
        assert_eq!(stats.total_errors, 0);
    }

    #[test]
    fn test_recovery_stats() {
        let stats = RecoveryStats::default();
        assert_eq!(stats.total_errors, 0);
        assert_eq!(stats.recovered, 0);
        assert_eq!(stats.failed, 0);
    }

    #[test]
    fn test_recovery_action_equality() {
        assert_eq!(
            RecoveryAction::Retry { delay_ms: 100 },
            RecoveryAction::Retry { delay_ms: 100 }
        );
        assert_ne!(
            RecoveryAction::Retry { delay_ms: 100 },
            RecoveryAction::Retry { delay_ms: 200 }
        );
    }

    #[test]
    fn test_circuit_breaker_creation() {
        let manager = RecoveryManager::with_default_policy();
        let cb = manager.circuit_breaker("test_node");

        // Should get the same circuit breaker on second call
        let cb2 = manager.circuit_breaker("test_node");
        assert!(Arc::ptr_eq(&cb, &cb2));
    }
}
