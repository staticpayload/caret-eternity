// Caret Recovery - Recovery policies
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::strategy::{RetryAction, NonRetryAction};

/// Recovery policy for handling errors
#[derive(Debug, Clone)]
pub struct RecoveryPolicy {
    /// Action to take on retryable errors
    pub on_retryable_error: RetryAction,
    /// Action to take on non-retryable errors
    pub on_non_retryable_error: NonRetryAction,
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Initial retry delay in milliseconds
    pub initial_retry_delay_ms: u64,
    /// Maximum retry delay in milliseconds
    pub max_retry_delay_ms: u64,
    /// Failure threshold for circuit breaker
    pub failure_threshold: usize,
    /// Success threshold for circuit breaker
    pub success_threshold: usize,
    /// Circuit timeout in milliseconds
    pub circuit_timeout_ms: u64,
}

impl Default for RecoveryPolicy {
    fn default() -> Self {
        Self {
            on_retryable_error: RetryAction::Retry,
            on_non_retryable_error: NonRetryAction::Fail,
            max_retries: 3,
            initial_retry_delay_ms: 100,
            max_retry_delay_ms: 5000,
            failure_threshold: 5,
            success_threshold: 2,
            circuit_timeout_ms: 60000,
        }
    }
}

impl RecoveryPolicy {
    /// Create a new recovery policy
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the action for retryable errors
    pub fn with_retryable_action(mut self, action: RetryAction) -> Self {
        self.on_retryable_error = action;
        self
    }

    /// Set the action for non-retryable errors
    pub fn with_non_retryable_action(mut self, action: NonRetryAction) -> Self {
        self.on_non_retryable_error = action;
        self
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max: usize) -> Self {
        self.max_retries = max;
        self
    }

    /// Set the initial retry delay
    pub fn with_initial_retry_delay(mut self, delay_ms: u64) -> Self {
        self.initial_retry_delay_ms = delay_ms;
        self
    }

    /// Set the maximum retry delay
    pub fn with_max_retry_delay(mut self, delay_ms: u64) -> Self {
        self.max_retry_delay_ms = delay_ms;
        self
    }

    /// Set the failure threshold for circuit breaker
    pub fn with_failure_threshold(mut self, threshold: usize) -> Self {
        self.failure_threshold = threshold;
        self
    }

    /// Set the success threshold for circuit breaker
    pub fn with_success_threshold(mut self, threshold: usize) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Set the circuit timeout
    pub fn with_circuit_timeout(mut self, timeout_ms: u64) -> Self {
        self.circuit_timeout_ms = timeout_ms;
        self
    }
}

/// Builder for recovery policies
pub struct RecoveryPolicyBuilder {
    policy: RecoveryPolicy,
}

impl Default for RecoveryPolicyBuilder {
    fn default() -> Self {
        Self {
            policy: RecoveryPolicy::default(),
        }
    }
}

impl RecoveryPolicyBuilder {
    /// Create a new policy builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the action for retryable errors
    pub fn retryable_action(mut self, action: RetryAction) -> Self {
        self.policy.on_retryable_error = action;
        self
    }

    /// Set the action for non-retryable errors
    pub fn non_retryable_action(mut self, action: NonRetryAction) -> Self {
        self.policy.on_non_retryable_error = action;
        self
    }

    /// Set the maximum number of retries
    pub fn max_retries(mut self, max: usize) -> Self {
        self.policy.max_retries = max;
        self
    }

    /// Set the initial retry delay
    pub fn initial_retry_delay(mut self, delay_ms: u64) -> Self {
        self.policy.initial_retry_delay_ms = delay_ms;
        self
    }

    /// Set the maximum retry delay
    pub fn max_retry_delay(mut self, delay_ms: u64) -> Self {
        self.policy.max_retry_delay_ms = delay_ms;
        self
    }

    /// Set the failure threshold for circuit breaker
    pub fn failure_threshold(mut self, threshold: usize) -> Self {
        self.policy.failure_threshold = threshold;
        self
    }

    /// Set the success threshold for circuit breaker
    pub fn success_threshold(mut self, threshold: usize) -> Self {
        self.policy.success_threshold = threshold;
        self
    }

    /// Set the circuit timeout
    pub fn circuit_timeout(mut self, timeout_ms: u64) -> Self {
        self.policy.circuit_timeout_ms = timeout_ms;
        self
    }

    /// Build the policy
    pub fn build(self) -> RecoveryPolicy {
        self.policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_policy_default() {
        let policy = RecoveryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.initial_retry_delay_ms, 100);
    }

    #[test]
    fn test_recovery_policy_builder() {
        let policy = RecoveryPolicyBuilder::new()
            .max_retries(5)
            .initial_retry_delay(50)
            .failure_threshold(10)
            .build();

        assert_eq!(policy.max_retries, 5);
        assert_eq!(policy.initial_retry_delay_ms, 50);
        assert_eq!(policy.failure_threshold, 10);
    }

    #[test]
    fn test_recovery_policy_chain() {
        let policy = RecoveryPolicy::new()
            .with_max_retries(10)
            .with_circuit_timeout(30000);

        assert_eq!(policy.max_retries, 10);
        assert_eq!(policy.circuit_timeout_ms, 30000);
    }
}
