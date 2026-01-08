// Caret Codec - Serialization codecs for Caret
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

#![warn(missing_docs)]
#![warn(clippy::all)]

mod binary;
mod error;
mod json;

pub use binary::BinaryCodec;
pub use error::{Error, Result};
pub use json::JsonCodec;

use caret_core::Packet;
use bytes::Bytes;

/// Codec for encoding/decoding packets
pub trait Codec: Send + Sync {
    /// Encode a packet to bytes
    fn encode(&self, packet: &Packet) -> Result<Bytes>;

    /// Decode bytes to a packet
    fn decode(&self, data: &[u8]) -> Result<Packet>;

    /// Get the codec name
    fn name(&self) -> &str {
        "codec"
    }
}

/// Codec types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecType {
    /// JSON codec
    Json,
    /// Binary codec
    Binary,
}

impl CodecType {
    /// Create a codec instance
    pub fn create(self) -> Box<dyn Codec> {
        match self {
            CodecType::Json => Box::new(JsonCodec::new()),
            CodecType::Binary => Box::new(BinaryCodec::new()),
        }
    }

    /// Parse codec type from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "json" => Ok(CodecType::Json),
            "binary" | "bin" => Ok(CodecType::Binary),
            _ => Err(Error::UnknownCodec(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codec_type_from_str() {
        assert_eq!(CodecType::from_str("json"), Ok(CodecType::Json));
        assert_eq!(CodecType::from_str("JSON"), Ok(CodecType::Json));
        assert_eq!(CodecType::from_str("binary"), Ok(CodecType::Binary));
        assert_eq!(CodecType::from_str("bin"), Ok(CodecType::Binary));
        assert!(CodecType::from_str("unknown").is_err());
    }

    #[test]
    fn test_codec_type_create() {
        let json_codec = CodecType::Json.create();
        assert_eq!(json_codec.name(), "json");

        let binary_codec = CodecType::Binary.create();
        assert_eq!(binary_codec.name(), "binary");
    }
}
