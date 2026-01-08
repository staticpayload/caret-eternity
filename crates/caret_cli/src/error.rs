// Caret CLI - Error types
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::fmt;

/// CLI error type
#[derive(Debug)]
pub enum Error {
    /// IO error
    Io(std::io::Error),
    /// DSL parsing error
    Dsl(caret_dsl::Error),
    /// Runtime error
    Runtime(caret_core::Error),
    /// Generic error message
    Message(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Dsl(e) => write!(f, "DSL error: {}", e),
            Error::Runtime(e) => write!(f, "Runtime error: {}", e),
            Error::Message(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<caret_dsl::Error> for Error {
    fn from(e: caret_dsl::Error) -> Self {
        Error::Dsl(e)
    }
}

impl From<caret_core::Error> for Error {
    fn from(e: caret_core::Error) -> Self {
        Error::Runtime(e)
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Message(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Message(s.to_string())
    }
}

/// Result type for CLI operations
pub type Result<T> = std::result::Result<T, Error>;
