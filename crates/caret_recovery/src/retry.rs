// Caret Recovery - Retry strategies
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
    /// Backoff strategy
    pub backoff: BackoffStrategy,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff: BackoffStrategy::Exponential,
        }
    }
}

impl RetryConfig {
    /// Create a new retry config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum attempts
    pub fn with_max_attempts(mut self, max: usize) -> Self {
        self.max_attempts = max;
        self
    }

    /// Set the initial delay
    pub fn with_initial_delay(mut self, delay_ms: u64) -> Self {
        self.initial_delay_ms = delay_ms;
        self
    }

    /// Set the maximum delay
    pub fn with_max_delay(mut self, delay_ms: u64) -> Self {
        self.max_delay_ms = delay_ms;
        self
    }

    /// Set the backoff strategy
    pub fn with_backoff(mut self, backoff: BackoffStrategy) -> Self {
        self.backoff = backoff;
        self
    }
}

/// Backoff strategies for retry delays
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed,
    /// Linear increase: delay = initial + (attempt * step)
    Linear { step_ms: u64 },
    /// Exponential increase: delay = initial * 2^attempt (capped at max)
    Exponential,
    /// Exponential with jitter: random variance in delay
    ExponentialWithJitter,
}

impl BackoffStrategy {
    /// Calculate the delay for a given attempt
    pub fn delay(&self, attempt: usize, initial: u64, max: u64) -> u64 {
        match self {
            BackoffStrategy::Fixed => initial.min(max),
            BackoffStrategy::Linear { step_ms } => {
                let delay = initial + (attempt as u64 * step_ms);
                delay.min(max)
            }
            BackoffStrategy::Exponential => {
                let delay = initial.saturating_mul(2u64.saturating_pow(attempt as u32));
                delay.min(max)
            }
            BackoffStrategy::ExponentialWithJitter => {
                let base_delay = initial.saturating_mul(2u64.saturating_pow(attempt as u32));
                let base_delay = base_delay.min(max);
                // Add jitter: +/- 25% of the delay
                let jitter = base_delay / 4;
                let variance = (fastrand::u64(0..) % (2 * jitter + 1)).saturating_sub(jitter);
                base_delay.saturating_add(variance).min(max)
            }
        }
    }
}

/// Retry strategy for executing operations with retries
pub struct RetryStrategy {
    config: RetryConfig,
}

impl RetryStrategy {
    /// Create a new retry strategy
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Create with default config
    pub fn with_defaults() -> Self {
        Self::new(RetryConfig::default())
    }

    /// Execute an operation with retry logic
    pub async fn execute<F, T, E>(&self, mut op: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Display + Send + Sync + 'static,
    {
        let mut last_error = None;

        for attempt in 0..self.config.max_attempts {
            match op() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);

                    // Don't delay after the last attempt
                    if attempt < self.config.max_attempts - 1 {
                        let delay = self.config.backoff.delay(
                            attempt,
                            self.config.initial_delay_ms,
                            self.config.max_delay_ms,
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Calculate the delay for a given attempt
    pub fn delay_for_attempt(&self, attempt: usize) -> u64 {
        self.config.backoff.delay(
            attempt,
            self.config.initial_delay_ms,
            self.config.max_delay_ms,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay_ms, 100);
    }

    #[test]
    fn test_backoff_fixed() {
        let backoff = BackoffStrategy::Fixed;
        assert_eq!(backoff.delay(0, 100, 5000), 100);
        assert_eq!(backoff.delay(5, 100, 5000), 100);
    }

    #[test]
    fn test_backoff_linear() {
        let backoff = BackoffStrategy::Linear { step_ms: 50 };
        assert_eq!(backoff.delay(0, 100, 5000), 100);
        assert_eq!(backoff.delay(1, 100, 5000), 150);
        assert_eq!(backoff.delay(5, 100, 5000), 350);
    }

    #[test]
    fn test_backoff_exponential() {
        let backoff = BackoffStrategy::Exponential;
        assert_eq!(backoff.delay(0, 100, 5000), 100);
        assert_eq!(backoff.delay(1, 100, 5000), 200);
        assert_eq!(backoff.delay(2, 100, 5000), 400);
        assert_eq!(backoff.delay(3, 100, 5000), 800);
    }

    #[test]
    fn test_backoff_capped() {
        let backoff = BackoffStrategy::Exponential;
        assert_eq!(backoff.delay(10, 100, 500), 500);
    }

    #[test]
    fn test_retry_strategy_delay() {
        let strategy = RetryStrategy::with_defaults();
        assert_eq!(strategy.delay_for_attempt(0), 100);
        assert_eq!(strategy.delay_for_attempt(1), 200);
    }

    #[tokio::test]
    async fn test_retry_execute_success() {
        let strategy = RetryStrategy::with_defaults();
        let mut attempts = 0;

        let result: Result<(), String> = strategy.execute(|| {
            attempts += 1;
            if attempts < 2 {
                Err("fail".to_string())
            } else {
                Ok(())
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(attempts, 2);
    }

    #[tokio::test]
    async fn test_retry_execute_failure() {
        let config = RetryConfig {
            max_attempts: 2,
            ..Default::default()
        };
        let strategy = RetryStrategy::new(config);

        let result: Result<(), String> = strategy.execute(|| {
            Err("fail".to_string())
        }).await;

        assert!(result.is_err());
    }
}
