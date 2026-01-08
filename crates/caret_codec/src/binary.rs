// Caret Codec - Binary codec
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{Codec, Error, Result};
use caret_core::{Packet, PacketKind, StreamId};
use bytes::{BufMut, Bytes, BytesMut};
use std::io::Cursor;

/// Binary codec for efficient packet serialization
#[derive(Debug, Clone)]
pub struct BinaryCodec {
    version: u8,
}

impl BinaryCodec {
    /// Create a new binary codec
    pub fn new() -> Self {
        Self { version: 1 }
    }

    /// Create a binary codec with specific version
    pub fn with_version(version: u8) -> Self {
        Self { version }
    }
}

impl Default for BinaryCodec {
    fn default() -> Self {
        Self::new()
    }
}

/// Packet type markers
#[repr(u8)]
enum PacketType {
    Bytes = 0,
    Audio = 1,
    Video = 2,
    Tensor = 3,
    Event = 4,
    Control = 5,
}

impl PacketType {
    fn from_u8(v: u8) -> Result<Self> {
        match v {
            0 => Ok(PacketType::Bytes),
            1 => Ok(PacketType::Audio),
            2 => Ok(PacketType::Video),
            3 => Ok(PacketType::Tensor),
            4 => Ok(PacketType::Event),
            5 => Ok(PacketType::Control),
            _ => Err(Error::Decode(format!("Invalid packet type: {}", v))),
        }
    }
}

// Binary format:
// [version: 1 byte]
// [packet_type: 1 byte]
// [timestamp: 8 bytes]
// [duration: 1 byte (0xFF = none, otherwise value followed by 8 bytes)]
// [stream_id: 8 bytes]
// [metadata_count: 2 bytes]
// [metadata entries: key_len: 1 byte, key, value_len: 2 bytes, value]*
// [data_length: 4 bytes]
// [type_specific_headers: variable]
// [data: bytes]

impl Codec for BinaryCodec {
    fn encode(&self, packet: &Packet) -> Result<Bytes> {
        let mut buf = BytesMut::new();

        // Header
        buf.put_u8(self.version);

        // Packet type
        let packet_type = packet_to_type(packet.kind());
        buf.put_u8(packet_type as u8);

        // Timestamp
        buf.put_u64(packet.timestamp().as_nanos());

        // Duration
        if let Some(dur) = packet.duration() {
            buf.put_u8(1);
            buf.put_u64(dur.as_nanos());
        } else {
            buf.put_u8(0xFF);
        }

        // Stream ID
        buf.put_u64(packet.stream_id().as_u64());

        // Metadata
        let metadata: Vec<_> = packet.metadata().iter().collect();
        buf.put_u16(metadata.len() as u16);
        for (key, value) in metadata {
            let key_bytes = key.as_bytes();
            let value_bytes = value.to_string().as_bytes().to_vec();
            buf.put_u8(key_bytes.len().min(255) as u8);
            buf.put_slice(key_bytes);
            buf.put_u16(value_bytes.len() as u16);
            buf.put_slice(&value_bytes);
        }

        // Data length and data
        buf.put_u32(packet.data().len() as u32);
        buf.put_slice(packet.data().as_ref());

        // Type-specific headers
        encode_type_header(&mut buf, packet.kind())?;

        Ok(buf.freeze())
    }

    fn decode(&self, data: &[u8]) -> Result<Packet> {
        if data.len() < 20 {
            return Err(Error::Decode("Data too short for header".to_string()));
        }

        let mut cursor = Cursor::new(data);

        // Version
        let version = read_u8(&mut cursor)?;
        if version != self.version {
            return Err(Error::Decode(format!(
                "Version mismatch: expected {}, got {}",
                self.version, version
            )));
        }

        // Packet type
        let packet_type = PacketType::from_u8(read_u8(&mut cursor)?)?;

        // Timestamp
        let timestamp = read_u64(&mut cursor)?;

        // Duration
        let has_duration = read_u8(&mut cursor)?;
        let duration_nanos = if has_duration != 0xFF {
            Some(read_u64(&mut cursor)?)
        } else {
            None
        };

        // Stream ID
        let stream_id = read_u64(&mut cursor)?;

        // Metadata
        let metadata_count = read_u16(&mut cursor)? as usize;
        let mut metadata = caret_core::Metadata::new();
        for _ in 0..metadata_count {
            let key_len = read_u8(&mut cursor)? as usize;
            let key = read_bytes(&mut cursor, key_len)?;
            let value_len = read_u16(&mut cursor)? as usize;
            let value = read_bytes(&mut cursor, value_len)?;
            let key_str: String = String::from_utf8_lossy(&key).into();
            let value_str: String = String::from_utf8_lossy(&value).into();
            metadata.insert(key_str, value_str);
        }

        // Data
        let data_len = read_u32(&mut cursor)? as usize;
        let packet_data = read_bytes(&mut cursor, data_len)?;

        // Type-specific data to reconstruct kind
        let kind = decode_type_header(packet_type, &mut cursor)?;

        // Reconstruct packet using from_parts
        Ok(Packet::from_parts(
            kind,
            packet_data,
            caret_core::Timestamp::from_nanos(timestamp),
            duration_nanos,
            StreamId::new(stream_id),
            metadata,
        ))
    }

    fn name(&self) -> &str {
        "binary"
    }
}

fn packet_to_type(kind: &PacketKind) -> PacketType {
    match kind {
        PacketKind::Bytes => PacketType::Bytes,
        PacketKind::Audio { .. } => PacketType::Audio,
        PacketKind::Video { .. } => PacketType::Video,
        PacketKind::Tensor { .. } => PacketType::Tensor,
        PacketKind::Event(_) => PacketType::Event,
        PacketKind::Control(_) => PacketType::Control,
    }
}

fn encode_type_header(buf: &mut BytesMut, kind: &PacketKind) -> Result<()> {
    match kind {
        PacketKind::Bytes => {}
        PacketKind::Audio { sample_rate, channels, format } => {
            buf.put_u32(*sample_rate);
            buf.put_u16(*channels);
            buf.put_u8(format.as_u8());
        }
        PacketKind::Video { width, height, format } => {
            buf.put_u32(*width);
            buf.put_u32(*height);
            buf.put_u8(format.as_u8());
        }
        PacketKind::Tensor { shape, dtype } => {
            buf.put_u8(dtype.as_u8());
            buf.put_u8(shape.len().min(255) as u8);
            for &dim in shape.iter().take(255) {
                buf.put_u64(dim as u64);
            }
        }
        PacketKind::Event(kind) => {
            buf.put_u8(kind.as_u8());
        }
        PacketKind::Control(kind) => {
            buf.put_u8(kind.as_u8());
        }
    }
    Ok(())
}

fn decode_type_header(packet_type: PacketType, cursor: &mut Cursor<&[u8]>) -> Result<PacketKind> {
    Ok(match packet_type {
        PacketType::Bytes => PacketKind::Bytes,
        PacketType::Audio => {
            let sample_rate = read_u32(cursor)?;
            let channels = read_u16(cursor)?;
            let format = read_u8(cursor)?;
            PacketKind::Audio {
                sample_rate,
                channels,
                format: caret_core::AudioFormat::from_u8(format)
                    .ok_or_else(|| Error::Decode(format!("Invalid audio format: {}", format)))?,
            }
        }
        PacketType::Video => {
            let width = read_u32(cursor)?;
            let height = read_u32(cursor)?;
            let format = read_u8(cursor)?;
            PacketKind::Video {
                width,
                height,
                format: caret_core::PixelFormat::from_u8(format)
                    .ok_or_else(|| Error::Decode(format!("Invalid pixel format: {}", format)))?,
            }
        }
        PacketType::Tensor => {
            let dtype = caret_core::TensorDtype::from_u8(read_u8(cursor)?)
                .ok_or_else(|| Error::Decode("Invalid tensor dtype".to_string()))?;
            let ndim = read_u8(cursor)? as usize;
            let mut shape = Vec::with_capacity(ndim);
            for _ in 0..ndim {
                shape.push(read_u64(cursor)? as usize);
            }
            PacketKind::Tensor { shape, dtype }
        }
        PacketType::Event => {
            let kind = read_u8(cursor)?;
            PacketKind::Event(
                caret_core::EventKind::from_u8(kind)
                    .ok_or_else(|| Error::Decode(format!("Invalid event kind: {}", kind)))?,
            )
        }
        PacketType::Control => {
            let kind = read_u8(cursor)?;
            PacketKind::Control(
                caret_core::ControlKind::from_u8(kind)
                    .ok_or_else(|| Error::Decode(format!("Invalid control kind: {}", kind)))?,
            )
        }
    })
}

// Helper functions for reading from cursor
fn read_u8<R: std::io::Read>(cursor: &mut R) -> Result<u8> {
    let mut buf = [0u8; 1];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| Error::Decode(format!("Failed to read u8: {}", e)))?;
    Ok(buf[0])
}

fn read_u16<R: std::io::Read>(cursor: &mut R) -> Result<u16> {
    let mut buf = [0u8; 2];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| Error::Decode(format!("Failed to read u16: {}", e)))?;
    Ok(u16::from_be_bytes(buf))
}

fn read_u32<R: std::io::Read>(cursor: &mut R) -> Result<u32> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| Error::Decode(format!("Failed to read u32: {}", e)))?;
    Ok(u32::from_be_bytes(buf))
}

fn read_u64<R: std::io::Read>(cursor: &mut R) -> Result<u64> {
    let mut buf = [0u8; 8];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| Error::Decode(format!("Failed to read u64: {}", e)))?;
    Ok(u64::from_be_bytes(buf))
}

fn read_bytes<R: std::io::Read>(cursor: &mut R, len: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| Error::Decode(format!("Failed to read bytes: {}", e)))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_codec_bytes_roundtrip() {
        let codec = BinaryCodec::new();
        let original = Packet::bytes(&b"hello world"[..]);

        let encoded = codec.encode(&original).unwrap();
        let decoded = codec.decode(&encoded).unwrap();

        assert_eq!(decoded.timestamp(), original.timestamp());
        assert_eq!(decoded.data(), original.data());
        assert!(matches!(decoded.kind(), PacketKind::Bytes));
    }

    #[test]
    fn test_binary_codec_audio_roundtrip() {
        let codec = BinaryCodec::new();
        let original = Packet::audio(vec![0u8; 1024], 48000, 2, caret_core::AudioFormat::S16);

        let encoded = codec.encode(&original).unwrap();
        let decoded = codec.decode(&encoded).unwrap();

        if let PacketKind::Audio { sample_rate, channels, format } = decoded.kind() {
            assert_eq!(*sample_rate, 48000);
            assert_eq!(*channels, 2);
            assert_eq!(*format, caret_core::AudioFormat::S16);
        } else {
            panic!("Expected audio packet");
        }
    }

    #[test]
    fn test_binary_codec_event_roundtrip() {
        let codec = BinaryCodec::new();
        let original = Packet::event(caret_core::EventKind::Start);

        let encoded = codec.encode(&original).unwrap();
        let decoded = codec.decode(&encoded).unwrap();

        assert!(matches!(decoded.kind(), PacketKind::Event(caret_core::EventKind::Start)));
    }

    #[test]
    fn test_binary_codec_control_roundtrip() {
        let codec = BinaryCodec::new();
        let original = Packet::control(caret_core::ControlKind::Pause);

        let encoded = codec.encode(&original).unwrap();
        let decoded = codec.decode(&encoded).unwrap();

        assert!(matches!(decoded.kind(), PacketKind::Control(caret_core::ControlKind::Pause)));
    }

    #[test]
    fn test_binary_codec_name() {
        let codec = BinaryCodec::new();
        assert_eq!(codec.name(), "binary");
    }

    #[test]
    fn test_binary_codec_version_mismatch() {
        let codec = BinaryCodec::with_version(2);
        let packet = Packet::bytes(&b"data"[..]);

        let encoded_v1 = BinaryCodec::new().encode(&packet).unwrap();
        assert!(codec.decode(&encoded_v1).is_err());
    }

    #[test]
    fn test_binary_codec_with_metadata() {
        let codec = BinaryCodec::new();
        let mut metadata = caret_core::Metadata::new();
        metadata.insert("key1", "value1");
        metadata.insert("key2", "value2");
        let packet = Packet::bytes(&b"data"[..]).with_metadata(metadata);

        let encoded = codec.encode(&packet).unwrap();
        let decoded = codec.decode(&encoded).unwrap();

        assert_eq!(decoded.metadata().len(), 2);
        assert_eq!(decoded.metadata().get("key1").map(|v| v.as_str()), Some(Some("value1")));
        assert_eq!(decoded.metadata().get("key2").map(|v| v.as_str()), Some(Some("value2")));
    }
}
