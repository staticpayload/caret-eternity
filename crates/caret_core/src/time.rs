// Caret Core - Time types
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::time::{Duration as StdDuration, SystemTime, UNIX_EPOCH};

/// A timestamp in a Caret pipeline
///
/// Timestamps are represented as nanoseconds from a time base.
/// This allows for deterministic replay and precise timing control.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp {
    nanos: u64,
}

impl Timestamp {
    /// Create a new timestamp from nanoseconds
    pub fn from_nanos(nanos: u64) -> Self {
        Self { nanos }
    }

    /// Create a new timestamp from microseconds
    pub fn from_micros(micros: u64) -> Self {
        Self {
            nanos: micros * 1000,
        }
    }

    /// Create a new timestamp from milliseconds
    pub fn from_millis(millis: u64) -> Self {
        Self {
            nanos: millis * 1_000_000,
        }
    }

    /// Create a new timestamp from seconds
    pub fn from_secs(secs: u64) -> Self {
        Self {
            nanos: secs * 1_000_000_000,
        }
    }

    /// Get the timestamp in nanoseconds
    pub fn as_nanos(&self) -> u64 {
        self.nanos
    }

    /// Get the timestamp in microseconds
    pub fn as_micros(&self) -> u64 {
        self.nanos / 1000
    }

    /// Get the timestamp in milliseconds
    pub fn as_millis(&self) -> u64 {
        self.nanos / 1_000_000
    }

    /// Get the timestamp in seconds
    pub fn as_secs(&self) -> u64 {
        self.nanos / 1_000_000_000
    }

    /// Create a timestamp from the current system time
    pub fn now() -> Self {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| Self::from_nanos(d.as_nanos() as u64))
            .unwrap_or_else(|_| Self::from_nanos(0))
    }

    /// Create a timestamp representing the start of the pipeline
    pub fn zero() -> Self {
        Self::from_nanos(0)
    }

    /// Add a duration to this timestamp
    pub fn saturating_add(&self, duration: Duration) -> Self {
        Self {
            nanos: self.nanos.saturating_add(duration.as_nanos()),
        }
    }

    /// Subtract a duration from this timestamp
    pub fn saturating_sub(&self, duration: Duration) -> Self {
        Self {
            nanos: self.nanos.saturating_sub(duration.as_nanos()),
        }
    }

    /// Get the duration since another timestamp
    pub fn duration_since(&self, earlier: Timestamp) -> Option<Duration> {
        self.nanos
            .checked_sub(earlier.nanos)
            .map(Duration::from_nanos)
    }
}

/// A time base for timestamps in a pipeline
///
/// The time base determines how timestamps are interpreted
/// and allows for reproducible execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeBase {
    /// Real-time clock (wall clock time)
    Realtime,
    /// Monotonic clock (since some start point)
    Monotonic,
    /// Media time (frames or samples)
    Media,
    /// Custom time base with an offset
    Custom { offset: u64 },
}

/// A duration of time
///
/// Used for packet durations and timing calculations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration {
    nanos: u64,
}

impl Duration {
    /// Create a duration from nanoseconds
    pub fn from_nanos(nanos: u64) -> Self {
        Self { nanos }
    }

    /// Create a duration from microseconds
    pub fn from_micros(micros: u64) -> Self {
        Self {
            nanos: micros * 1000,
        }
    }

    /// Create a duration from milliseconds
    pub fn from_millis(millis: u64) -> Self {
        Self {
            nanos: millis * 1_000_000,
        }
    }

    /// Create a duration from seconds
    pub fn from_secs(secs: u64) -> Self {
        Self {
            nanos: secs * 1_000_000_000,
        }
    }

    /// Get the duration in nanoseconds
    pub fn as_nanos(&self) -> u64 {
        self.nanos
    }

    /// Get the duration in microseconds
    pub fn as_micros(&self) -> u64 {
        self.nanos / 1000
    }

    /// Get the duration in milliseconds
    pub fn as_millis(&self) -> u64 {
        self.nanos / 1_000_000
    }

    /// Get the duration in seconds
    pub fn as_secs(&self) -> u64 {
        self.nanos / 1_000_000_000
    }

    /// Check if the duration is zero
    pub fn is_zero(&self) -> bool {
        self.nanos == 0
    }
}

impl From<StdDuration> for Duration {
    fn from(d: StdDuration) -> Self {
        Self {
            nanos: d.as_nanos() as u64,
        }
    }
}

impl From<Duration> for StdDuration {
    fn from(d: Duration) -> Self {
        StdDuration::from_nanos(d.nanos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_conversions() {
        let ts = Timestamp::from_secs(1);
        assert_eq!(ts.as_secs(), 1);
        assert_eq!(ts.as_millis(), 1000);
        assert_eq!(ts.as_micros(), 1_000_000);
        assert_eq!(ts.as_nanos(), 1_000_000_000);
    }

    #[test]
    fn test_timestamp_arithmetic() {
        let ts = Timestamp::from_secs(1);
        let dur = Duration::from_secs(1);
        let later = ts.saturating_add(dur);
        assert_eq!(later.as_secs(), 2);

        let earlier = later.saturating_sub(dur);
        assert_eq!(earlier.as_secs(), 1);
    }

    #[test]
    fn test_timestamp_duration_since() {
        let earlier = Timestamp::from_secs(1);
        let later = Timestamp::from_secs(3);
        let dur = later.duration_since(earlier).unwrap();
        assert_eq!(dur.as_secs(), 2);
    }

    #[test]
    fn test_duration_conversions() {
        let dur = Duration::from_secs(1);
        assert_eq!(dur.as_secs(), 1);
        assert_eq!(dur.as_millis(), 1000);
        assert_eq!(dur.as_nanos(), 1_000_000_000);
    }

    #[test]
    fn test_duration_zero() {
        assert!(Duration::from_nanos(0).is_zero());
        assert!(!Duration::from_nanos(1).is_zero());
    }
}
