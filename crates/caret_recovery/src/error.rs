// Caret Recovery - Error types
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

/// Error type for recovery operations
#[derive(Debug, thiserror::Error)]
pub enum RecoveryError {
    /// Circuit breaker is open
    #[error("circuit breaker '{0}' is open")]
    CircuitOpen(String),

    /// Max retry attempts exceeded
    #[error("max retry attempts exceeded: {0}")]
    MaxRetriesExceeded(usize),

    /// Recovery strategy not available
    #[error("recovery strategy '{0}' not available")]
    StrategyNotAvailable(String),

    /// Fallback not configured
    #[error("fallback not configured for '{0}'")]
    FallbackNotConfigured(String),

    /// Invalid configuration
    #[error("invalid recovery configuration: {0}")]
    InvalidConfig(String),

    /// Timeout during recovery
    #[error("recovery timeout after {0}ms")]
    Timeout(u64),
}

/// Result type for recovery operations
pub type RecoveryResult<T> = std::result::Result<T, RecoveryError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_error_display() {
        let err = RecoveryError::CircuitOpen("test_node".to_string());
        assert_eq!(err.to_string(), "circuit breaker 'test_node' is open");
    }

    #[test]
    fn test_max_retries_exceeded() {
        let err = RecoveryError::MaxRetriesExceeded(5);
        assert_eq!(err.to_string(), "max retry attempts exceeded: 5");
    }
}
