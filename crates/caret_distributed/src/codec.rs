// Caret Distributed - Message framing codec
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{Error, Message, Result};
use bytes::BytesMut;
use std::io::{self, Read, Write};

/// Magic bytes for message framing (CARET in hex)
pub const FRAME_MAGIC: &[u8] = &[0x43, 0x41, 0x52, 0x45, 0x54];

/// Frame header size (magic + version + length + checksum)
pub const FRAME_HEADER_SIZE: usize = 5 + 1 + 4 + 4;

/// Maximum frame size (matches MAX_MESSAGE_SIZE)
pub const MAX_FRAME_SIZE: usize = crate::MAX_MESSAGE_SIZE;

/// Protocol version
pub const FRAME_VERSION: u8 = 1;

/// Frame codec for encoding/decoding messages
#[derive(Debug, Clone)]
pub struct FrameCodec {
    /// Maximum frame size
    max_frame_size: usize,
}

impl Default for FrameCodec {
    fn default() -> Self {
        Self {
            max_frame_size: MAX_FRAME_SIZE,
        }
    }
}

impl FrameCodec {
    /// Create a new codec with the given maximum frame size
    pub fn with_max_size(max_frame_size: usize) -> Self {
        Self { max_frame_size }
    }

    /// Encode a message into a frame
    pub fn encode(&self, msg: &Message) -> Result<Vec<u8>> {
        let payload = serde_json::to_vec(msg)
            .map_err(|e| Error::Serialization(format!("failed to serialize message: {}", e)))?;

        if payload.len() > self.max_frame_size {
            return Err(Error::InvalidMessage(format!(
                "message too large: {} bytes (max {})",
                payload.len(),
                self.max_frame_size
            )));
        }

        let len = payload.len() as u32;
        let checksum = calculate_checksum(&payload);

        let mut frame = Vec::with_capacity(FRAME_HEADER_SIZE + payload.len());

        // Write magic
        frame.extend_from_slice(FRAME_MAGIC);
        // Write version
        frame.push(FRAME_VERSION);
        // Write length (big-endian)
        frame.extend_from_slice(&len.to_be_bytes());
        // Write checksum
        frame.extend_from_slice(&checksum.to_be_bytes());
        // Write payload
        frame.extend_from_slice(&payload);

        Ok(frame)
    }

    /// Decode a frame from bytes
    pub fn decode(&self, data: &[u8]) -> Result<Message> {
        if data.len() < FRAME_HEADER_SIZE {
            return Err(Error::InvalidMessage(format!(
                "frame too short: {} bytes (need at least {})",
                data.len(),
                FRAME_HEADER_SIZE
            )));
        }

        // Check magic (5 bytes)
        if &data[0..5] != FRAME_MAGIC {
            return Err(Error::InvalidMessage("invalid magic bytes".into()));
        }

        // Check version (1 byte at index 5)
        let version = data[5];
        if version != FRAME_VERSION {
            return Err(Error::InvalidMessage(format!(
                "unsupported frame version: {} (expected {})",
                version, FRAME_VERSION
            )));
        }

        // Read length (4 bytes at index 6-9)
        let len = u32::from_be_bytes([data[6], data[7], data[8], data[9]]) as usize;

        if len > self.max_frame_size {
            return Err(Error::InvalidMessage(format!(
                "frame too large: {} bytes (max {})",
                len,
                self.max_frame_size
            )));
        }

        // Read checksum (4 bytes at index 10-13)
        let expected_checksum = u32::from_be_bytes([data[10], data[11], data[12], data[13]]);

        // Check we have the full payload
        if data.len() < FRAME_HEADER_SIZE + len {
            return Err(Error::InvalidMessage("incomplete frame".into()));
        }

        let payload = &data[FRAME_HEADER_SIZE..FRAME_HEADER_SIZE + len];

        // Verify checksum
        let actual_checksum = calculate_checksum(payload.as_ref());
        if actual_checksum != expected_checksum {
            return Err(Error::InvalidMessage(format!(
                "checksum mismatch: expected {}, got {}",
                expected_checksum, actual_checksum
            )));
        }

        // Deserialize message
        serde_json::from_slice(payload)
            .map_err(|e| Error::Serialization(format!("failed to deserialize message: {}", e)))
    }

    /// Decode a frame from a reader
    pub fn decode_from_reader<R: Read>(&self, reader: &mut R) -> Result<Message> {
        let mut header = [0u8; FRAME_HEADER_SIZE];

        reader
            .read_exact(&mut header)
            .map_err(|e| Error::Io(e))?;

        // Check magic (5 bytes)
        if &header[0..5] != FRAME_MAGIC {
            return Err(Error::InvalidMessage("invalid magic bytes".into()));
        }

        // Check version (1 byte at index 5)
        let version = header[5];
        if version != FRAME_VERSION {
            return Err(Error::InvalidMessage(format!(
                "unsupported frame version: {} (expected {})",
                version, FRAME_VERSION
            )));
        }

        // Read length (4 bytes at index 6-9)
        let len = u32::from_be_bytes([header[6], header[7], header[8], header[9]]) as usize;

        if len > self.max_frame_size {
            return Err(Error::InvalidMessage(format!(
                "frame too large: {} bytes (max {})",
                len,
                self.max_frame_size
            )));
        }

        // Read checksum (4 bytes at index 10-13)
        let expected_checksum = u32::from_be_bytes([header[10], header[11], header[12], header[13]]);

        // Read payload
        let mut payload = vec![0u8; len];
        reader
            .read_exact(&mut payload)
            .map_err(|e| Error::Io(e))?;

        // Verify checksum
        let actual_checksum = calculate_checksum(&payload);
        if actual_checksum != expected_checksum {
            return Err(Error::InvalidMessage(format!(
                "checksum mismatch: expected {}, got {}",
                expected_checksum, actual_checksum
            )));
        }

        // Deserialize message
        serde_json::from_slice(&payload)
            .map_err(|e| Error::Serialization(format!("failed to deserialize message: {}", e)))
    }

    /// Encode a message and write to a writer
    pub fn encode_to_writer<W: Write>(&self, msg: &Message, writer: &mut W) -> Result<()> {
        let frame = self.encode(msg)?;
        writer
            .write_all(&frame)
            .map_err(|e| Error::Io(e))?;
        writer.flush().map_err(|e| Error::Io(e))?;
        Ok(())
    }
}

/// Calculate a simple checksum for the payload
fn calculate_checksum(data: &[u8]) -> u32 {
    // Use a simple Adler-32 variant
    let mut sum1: u32 = 1;
    let mut sum2: u32 = 0;

    for &byte in data.iter() {
        sum1 = (sum1 + byte as u32) % 65521;
        sum2 = (sum2 + sum1) % 65521;
    }

    (sum2 << 16) | sum1
}

/// Frame decoder for Tokio IO
#[derive(Debug)]
pub struct FrameDecoder {
    codec: FrameCodec,
    buffer: BytesMut,
}

impl Default for FrameDecoder {
    fn default() -> Self {
        Self {
            codec: FrameCodec::default(),
            buffer: BytesMut::with_capacity(8192),
        }
    }
}

impl FrameDecoder {
    /// Create a new decoder
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a decoder with custom max size
    pub fn with_max_size(max_frame_size: usize) -> Self {
        Self {
            codec: FrameCodec::with_max_size(max_frame_size),
            buffer: BytesMut::with_capacity(8192),
        }
    }

    /// Feed data to the decoder
    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    /// Try to decode a message from the buffer
    pub fn try_decode(&mut self) -> Result<Option<Message>> {
        // Check if we have at least the header
        if self.buffer.len() < FRAME_HEADER_SIZE {
            return Ok(None);
        }

        // Check magic
        if &self.buffer[0..5] != FRAME_MAGIC {
            return Err(Error::InvalidMessage("invalid magic bytes".into()));
        }

        // Read length (magic[5] + version + length[4])
        let len = u32::from_be_bytes([
            self.buffer[6],
            self.buffer[7],
            self.buffer[8],
            self.buffer[9],
        ]) as usize;

        // Check if we have the full frame
        let frame_size = FRAME_HEADER_SIZE + len;
        if self.buffer.len() < frame_size {
            return Ok(None);
        }

        // Decode the frame
        let frame = self.buffer.split_to(frame_size);
        self.codec.decode(&frame).map(Some)
    }

    /// Get the number of bytes buffered
    pub fn buffered(&self) -> usize {
        self.buffer.len()
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeId;

    #[test]
    fn test_codec_round_trip() {
        let codec = FrameCodec::default();
        let from = NodeId::new_v4();
        let msg = Message::new(from, None, crate::message::MessagePayload::Heartbeat);

        let encoded = codec.encode(&msg).unwrap();
        let decoded = codec.decode(&encoded).unwrap();

        assert_eq!(decoded.id, msg.id);
        assert_eq!(decoded.from, from);
    }

    #[test]
    fn test_codec_magic_validation() {
        let codec = FrameCodec::default();
        let mut data = vec![0u8; FRAME_HEADER_SIZE + 10];

        // Wrong magic
        data[0] = 0xFF;

        let result = codec.decode(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_codec_too_large() {
        let codec = FrameCodec::with_max_size(100);
        let from = NodeId::new_v4();

        // Create a message that would be too large
        let payload = vec![0u8; 200];
        let msg = Message::new(
            from,
            None,
            crate::message::MessagePayload::GraphSubmit {
                graph_id: "test".to_string(),
                graph: payload,
                mode: crate::ExecutionMode::Pipeline,
            },
        );

        let result = codec.encode(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_decoder_incremental() {
        let codec = FrameCodec::default();
        let from = NodeId::new_v4();
        let msg = Message::new(from, None, crate::message::MessagePayload::Heartbeat);

        let encoded = codec.encode(&msg).unwrap();

        let mut decoder = FrameDecoder::new();

        // Feed half the data
        decoder.feed(&encoded[..encoded.len() / 2]);
        assert!(matches!(decoder.try_decode().unwrap(), None));

        // Feed the rest
        decoder.feed(&encoded[encoded.len() / 2..]);
        let decoded = decoder.try_decode().unwrap().unwrap();

        assert_eq!(decoded.id, msg.id);
    }

    #[test]
    fn test_checksum() {
        let data = b"hello world";
        let checksum1 = calculate_checksum(data);
        let checksum2 = calculate_checksum(data);

        assert_eq!(checksum1, checksum2);

        // Different data should have different checksum
        let checksum3 = calculate_checksum(b"hello world!");
        assert_ne!(checksum1, checksum3);
    }

    #[test]
    fn test_frame_size() {
        assert_eq!(FRAME_MAGIC.len(), 5);
        assert_eq!(FRAME_HEADER_SIZE, 5 + 1 + 4 + 4);
        assert_eq!(FRAME_HEADER_SIZE, 14);
    }
}
