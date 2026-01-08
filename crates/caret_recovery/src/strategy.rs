// Caret Recovery - Recovery strategies
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::sync::Arc;
use parking_lot::Mutex;

/// Actions that can be taken when an error occurs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryAction {
    /// Retry the operation after a delay
    Retry { delay_ms: u64 },
    /// Skip this operation and continue
    Skip,
    /// Use a fallback value or method
    Fallback,
    /// Fail the operation
    Fail,
}

/// Action to take on retryable errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryAction {
    /// Retry the operation
    Retry,
    /// Skip this operation
    Skip,
    /// Fail immediately
    Fail,
}

/// Action to take on non-retryable errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonRetryAction {
    /// Fail immediately
    Fail,
    /// Skip this operation
    Skip,
    /// Use fallback
    Fallback,
}

/// Recovery strategy for handling errors
pub trait RecoveryStrategy: Send + Sync {
    /// Determine the action to take for an error
    fn on_error(&self, error: &caret_core::Error, context: &crate::ErrorContext) -> crate::RecoveryAction;

    /// Get the name of this strategy
    fn name(&self) -> &str {
        "default"
    }
}

/// Default recovery strategy
#[derive(Clone)]
pub struct DefaultRecoveryStrategy {
    /// Max retries
    max_retries: usize,
    /// Retry delay in milliseconds
    retry_delay_ms: u64,
}

impl DefaultRecoveryStrategy {
    /// Create a new default recovery strategy
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 100,
        }
    }

    /// Set the max retries
    pub fn with_max_retries(mut self, max: usize) -> Self {
        self.max_retries = max;
        self
    }

    /// Set the retry delay
    pub fn with_retry_delay(mut self, delay_ms: u64) -> Self {
        self.retry_delay_ms = delay_ms;
        self
    }
}

impl Default for DefaultRecoveryStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RecoveryStrategy for DefaultRecoveryStrategy {
    fn on_error(&self, error: &caret_core::Error, _context: &crate::ErrorContext) -> crate::RecoveryAction {
        if error.is_retryable() {
            crate::RecoveryAction::Retry {
                delay_ms: self.retry_delay_ms,
            }
        } else {
            crate::RecoveryAction::Fail
        }
    }

    fn name(&self) -> &str {
        "default"
    }
}

/// Aggressive recovery strategy - retries more and falls back
#[derive(Clone)]
pub struct AggressiveRecoveryStrategy {
    /// Max retries
    max_retries: usize,
    /// Retry delay in milliseconds
    retry_delay_ms: u64,
}

impl AggressiveRecoveryStrategy {
    /// Create a new aggressive recovery strategy
    pub fn new() -> Self {
        Self {
            max_retries: 10,
            retry_delay_ms: 50,
        }
    }

    /// Set the max retries
    pub fn with_max_retries(mut self, max: usize) -> Self {
        self.max_retries = max;
        self
    }

    /// Set the retry delay
    pub fn with_retry_delay(mut self, delay_ms: u64) -> Self {
        self.retry_delay_ms = delay_ms;
        self
    }
}

impl Default for AggressiveRecoveryStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RecoveryStrategy for AggressiveRecoveryStrategy {
    fn on_error(&self, error: &caret_core::Error, _context: &crate::ErrorContext) -> crate::RecoveryAction {
        if error.is_retryable() {
            crate::RecoveryAction::Retry {
                delay_ms: self.retry_delay_ms,
            }
        } else {
            // Try fallback for non-retryable errors
            crate::RecoveryAction::Fallback
        }
    }

    fn name(&self) -> &str {
        "aggressive"
    }
}

/// Conservative recovery strategy - fails fast
#[derive(Clone)]
pub struct ConservativeRecoveryStrategy;

impl Default for ConservativeRecoveryStrategy {
    fn default() -> Self {
        Self
    }
}

impl RecoveryStrategy for ConservativeRecoveryStrategy {
    fn on_error(&self, _error: &caret_core::Error, _context: &crate::ErrorContext) -> crate::RecoveryAction {
        crate::RecoveryAction::Fail
    }

    fn name(&self) -> &str {
        "conservative"
    }
}

/// Error handler for recording and responding to errors
pub struct ErrorHandler {
    /// Recovery strategy
    strategy: Arc<dyn RecoveryStrategy>,
    /// Error counts by node
    error_counts: Arc<Mutex<std::collections::HashMap<String, usize>>>,
}

impl ErrorHandler {
    /// Create a new error handler
    pub fn new(strategy: Arc<dyn RecoveryStrategy>) -> Self {
        Self {
            strategy,
            error_counts: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Create with default strategy
    pub fn with_default() -> Self {
        Self::new(Arc::new(DefaultRecoveryStrategy::default()))
    }

    /// Create with aggressive strategy
    pub fn with_aggressive() -> Self {
        Self::new(Arc::new(AggressiveRecoveryStrategy::default()))
    }

    /// Create with conservative strategy
    pub fn with_conservative() -> Self {
        Self::new(Arc::new(ConservativeRecoveryStrategy))
    }

    /// Handle an error and determine the recovery action
    pub fn handle(&self, error: &caret_core::Error, context: &crate::ErrorContext) -> crate::RecoveryAction {
        // Record the error
        let mut counts = self.error_counts.lock();
        *counts.entry(context.node_id.clone()).or_insert(0) += 1;
        drop(counts);

        // Get action from strategy
        self.strategy.on_error(error, context)
    }

    /// Get the error count for a node
    pub fn error_count(&self, node_id: &str) -> usize {
        self.error_counts.lock().get(node_id).copied().unwrap_or(0)
    }

    /// Get all error counts
    pub fn all_error_counts(&self) -> std::collections::HashMap<String, usize> {
        self.error_counts.lock().clone()
    }

    /// Reset error counts
    pub fn reset_counts(&self) {
        self.error_counts.lock().clear();
    }

    /// Get the strategy name
    pub fn strategy_name(&self) -> &str {
        self.strategy.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_recovery_strategy() {
        let strategy = DefaultRecoveryStrategy::new();
        let context = crate::ErrorContext::new("test_node");

        let io_error = caret_core::Error::io("connection failed");
        let action = strategy.on_error(&io_error, &context);
        assert!(matches!(action, crate::RecoveryAction::Retry { .. }));

        let config_error = caret_core::Error::config("missing field");
        let action = strategy.on_error(&config_error, &context);
        assert_eq!(action, crate::RecoveryAction::Fail);
    }

    #[test]
    fn test_aggressive_recovery_strategy() {
        let strategy = AggressiveRecoveryStrategy::new();
        let context = crate::ErrorContext::new("test_node");

        let io_error = caret_core::Error::io("connection failed");
        let action = strategy.on_error(&io_error, &context);
        assert!(matches!(action, crate::RecoveryAction::Retry { .. }));

        let config_error = caret_core::Error::config("missing field");
        let action = strategy.on_error(&config_error, &context);
        assert_eq!(action, crate::RecoveryAction::Fallback);
    }

    #[test]
    fn test_conservative_recovery_strategy() {
        let strategy = ConservativeRecoveryStrategy;
        let context = crate::ErrorContext::new("test_node");

        let io_error = caret_core::Error::io("connection failed");
        let action = strategy.on_error(&io_error, &context);
        assert_eq!(action, crate::RecoveryAction::Fail);
    }

    #[test]
    fn test_error_handler() {
        let handler = ErrorHandler::with_default();
        let context = crate::ErrorContext::new("test_node");

        let error = caret_core::Error::io("test error");
        handler.handle(&error, &context);

        assert_eq!(handler.error_count("test_node"), 1);
    }

    #[test]
    fn test_error_handler_multiple_nodes() {
        let handler = ErrorHandler::with_aggressive();
        let ctx1 = crate::ErrorContext::new("node1");
        let ctx2 = crate::ErrorContext::new("node2");

        let error = caret_core::Error::io("test error");
        handler.handle(&error, &ctx1);
        handler.handle(&error, &ctx1);
        handler.handle(&error, &ctx2);

        assert_eq!(handler.error_count("node1"), 2);
        assert_eq!(handler.error_count("node2"), 1);
    }
}
