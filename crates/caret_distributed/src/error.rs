// Caret Distributed - Error types
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use caret_core::Error as CoreError;
use std::io;

/// Distributed execution errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Core error
    #[error("Core error: {0}")]
    Core(#[from] CoreError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Transport error
    #[error("Transport error: {0}")]
    Transport(String),

    /// Node not found
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    /// Node timeout
    #[error("Node timeout: {0}")]
    NodeTimeout(String),

    /// Connection refused
    #[error("Connection refused: {0}")]
    ConnectionRefused(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Invalid message
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Version mismatch
    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },

    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Coordinator error
    #[error("Coordinator error: {0}")]
    Coordinator(String),

    /// Execution error
    #[error("Execution error: {0}")]
    Execution(String),
}

/// Result type for distributed operations
pub type Result<T> = std::result::Result<T, Error>;
