// Caret Core - Error types
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::backtrace::Backtrace;
use std::fmt;
use std::sync::Arc;

/// Core result type for Caret
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Categories of errors in Caret
///
/// Each error kind represents a distinct category of failure,
/// allowing for targeted error handling and reporting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Invalid configuration provided
    Config,
    /// Invalid input data
    InvalidInput,
    /// IO operation failed
    Io,
    /// Graph topology error (cycle, missing port, etc)
    Graph,
    /// Scheduling error
    Schedule,
    /// Buffer operation failed
    Buffer,
    /// Operation timeout
    Timeout,
    /// Resource exhausted
    ResourceExhausted,
    /// Plugin error
    Plugin,
    /// Codec error
    Codec,
    /// Internal error (should not happen)
    Internal,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Config => write!(f, "configuration error"),
            ErrorKind::InvalidInput => write!(f, "invalid input"),
            ErrorKind::Io => write!(f, "I/O error"),
            ErrorKind::Graph => write!(f, "graph error"),
            ErrorKind::Schedule => write!(f, "scheduling error"),
            ErrorKind::Buffer => write!(f, "buffer error"),
            ErrorKind::Timeout => write!(f, "timeout"),
            ErrorKind::ResourceExhausted => write!(f, "resource exhausted"),
            ErrorKind::Plugin => write!(f, "plugin error"),
            ErrorKind::Codec => write!(f, "codec error"),
            ErrorKind::Internal => write!(f, "internal error"),
        }
    }
}

/// Core error type for Caret
///
/// All Caret errors are typed and include context for actionable
/// error messages. Use this type for all fallible operations.
#[derive(Clone)]
pub struct Error {
    kind: ErrorKind,
    message: String,
    source: Option<Arc<Error>>,
    backtrace: Option<Arc<Backtrace>>,
}

impl Error {
    /// Create a new error with the given kind and message
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Error {
            kind,
            message: message.into(),
            source: None,
            backtrace: Some(Arc::new(Backtrace::capture())),
        }
    }

    /// Get the error kind
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Add context to an existing error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.message = format!("{}: {}", context.into(), self.message);
        self
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::Io | ErrorKind::Timeout | ErrorKind::ResourceExhausted
        )
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("message", &self.message)
            .field("source", &self.source)
            .finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)?;
        if let Some(source) = &self.source {
            write!(f, "\n  caused by: {}", source)?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &dyn std::error::Error)
    }
}

// Convenience constructors

impl Error {
    pub fn config(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Config, msg)
    }

    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::InvalidInput, msg)
    }

    pub fn io(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Io, msg)
    }

    pub fn graph(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Graph, msg)
    }

    pub fn schedule(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Schedule, msg)
    }

    pub fn buffer(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Buffer, msg)
    }

    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Timeout, msg)
    }

    pub fn resource_exhausted(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::ResourceExhausted, msg)
    }

    pub fn plugin(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Plugin, msg)
    }

    pub fn codec(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Codec, msg)
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(ErrorKind::Internal, msg)
    }
}

// Conversions from standard error types

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::io(err.to_string())
    }
}

// Thiserror compatibility for external crates

impl From<Error> for std::io::Error {
    fn from(err: Error) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::invalid_input("test error");
        assert_eq!(err.kind(), &ErrorKind::InvalidInput);
        assert_eq!(err.message(), "test error");
    }

    #[test]
    fn test_error_with_context() {
        let err = Error::invalid_input("test error").with_context("in parser");
        assert!(err.message().contains("in parser"));
        assert!(err.message().contains("test error"));
    }

    #[test]
    fn test_error_retryable() {
        assert!(Error::io("connection failed").is_retryable());
        assert!(Error::timeout("deadline exceeded").is_retryable());
        assert!(!Error::config("missing field").is_retryable());
        assert!(!Error::invalid_input("bad data").is_retryable());
    }

    #[test]
    fn test_error_display() {
        let err = Error::invalid_input("bad value");
        let s = format!("{}", err);
        assert!(s.contains("invalid input"));
        assert!(s.contains("bad value"));
    }
}
