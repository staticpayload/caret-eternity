// Caret Codec - Error types
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::fmt;

/// Codec errors
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Unknown codec type
    UnknownCodec(String),
    /// Encoding error
    Encode(String),
    /// Decoding error
    Decode(String),
    /// Invalid data
    InvalidData(String),
    /// IO error
    Io(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownCodec(name) => write!(f, "Unknown codec: {}", name),
            Error::Encode(msg) => write!(f, "Encoding error: {}", msg),
            Error::Decode(msg) => write!(f, "Decoding error: {}", msg),
            Error::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            Error::Io(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Codec result type
pub type Result<T> = std::result::Result<T, Error>;

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Encode(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::UnknownCodec("test".to_string());
        assert_eq!(err.to_string(), "Unknown codec: test");

        let err = Error::Encode("failed".to_string());
        assert_eq!(err.to_string(), "Encoding error: failed");

        let err = Error::Decode("invalid".to_string());
        assert_eq!(err.to_string(), "Decoding error: invalid");
    }
}
