// Caret Record - Error types
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

/// Error types for recording and replay
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A recording is already in progress
    #[error("A recording is already in progress")]
    RecordingInProgress,

    /// No recording is in progress
    #[error("No recording is in progress")]
    NoRecordingInProgress,

    /// Event store error
    #[error("Event store error: {0}")]
    StoreError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(u64),

    /// Invalid event data
    #[error("Invalid event data: {0}")]
    InvalidEventData(String),

    /// Replay error
    #[error("Replay error: {0}")]
    ReplayError(String),

    /// Codec error
    #[error("Codec error: {0}")]
    CodecError(String),
}

/// Result type for recording operations
pub type Result<T> = std::result::Result<T, Error>;
