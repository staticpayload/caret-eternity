// Caret Codec - JSON codec
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{Codec, Error, Result};
use caret_core::{Packet, PacketKind, AudioFormat, PixelFormat, TensorDtype, EventKind, ControlKind};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// JSON codec for packet serialization
#[derive(Debug, Clone)]
pub struct JsonCodec {
    pretty: bool,
}

impl JsonCodec {
    /// Create a new JSON codec
    pub fn new() -> Self {
        Self { pretty: false }
    }

    /// Create a pretty-printed JSON codec
    pub fn pretty() -> Self {
        Self { pretty: true }
    }
}

impl Default for JsonCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Codec for JsonCodec {
    fn encode(&self, packet: &Packet) -> Result<Bytes> {
        let wrapper = JsonPacketWrapper::from_packet(packet);

        if self.pretty {
            let json = serde_json::to_string_pretty(&wrapper)?;
            Ok(Bytes::from(json))
        } else {
            let json = serde_json::to_string(&wrapper)?;
            Ok(Bytes::from(json))
        }
    }

    fn decode(&self, data: &[u8]) -> Result<Packet> {
        let wrapper: JsonPacketWrapper = serde_json::from_slice(data)?;
        wrapper.to_packet()
    }

    fn name(&self) -> &str {
        "json"
    }
}

/// Wrapper for JSON serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonPacketWrapper {
    kind: JsonPacketKind,
    timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    duration: Option<u64>,
    stream_id: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    metadata: Vec<(String, String)>,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    data: String, // base64 encoded
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum JsonPacketKind {
    #[serde(rename = "bytes")]
    Bytes,
    #[serde(rename = "audio")]
    Audio { sample_rate: u32, channels: u16, format: String },
    #[serde(rename = "video")]
    Video { width: u32, height: u32, format: String },
    #[serde(rename = "tensor")]
    Tensor { shape: Vec<usize>, dtype: String },
    #[serde(rename = "event")]
    Event { kind: String },
    #[serde(rename = "control")]
    Control { kind: String },
}

impl JsonPacketWrapper {
    fn from_packet(packet: &Packet) -> Self {
        let kind = match packet.kind() {
            PacketKind::Bytes => JsonPacketKind::Bytes,
            PacketKind::Audio { sample_rate, channels, format } => JsonPacketKind::Audio {
                sample_rate: *sample_rate,
                channels: *channels,
                format: format!("{:?}", format),
            },
            PacketKind::Video { width, height, format } => JsonPacketKind::Video {
                width: *width,
                height: *height,
                format: format!("{:?}", format),
            },
            PacketKind::Tensor { shape, dtype } => JsonPacketKind::Tensor {
                shape: shape.clone(),
                dtype: format!("{:?}", dtype),
            },
            PacketKind::Event(kind) => JsonPacketKind::Event {
                kind: format!("{:?}", kind),
            },
            PacketKind::Control(kind) => JsonPacketKind::Control {
                kind: format!("{:?}", kind),
            },
        };

        let data = base64::encode(packet.data().as_ref());

        Self {
            kind,
            timestamp: packet.timestamp().as_nanos(),
            duration: packet.duration().map(|d| d.as_nanos()),
            stream_id: packet.stream_id().as_u64(),
            metadata: packet
                .metadata()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            data,
        }
    }

    fn to_packet(self) -> Result<Packet> {
        let kind = match self.kind {
            JsonPacketKind::Bytes => PacketKind::Bytes,
            JsonPacketKind::Audio { sample_rate, channels, format } => {
                PacketKind::Audio {
                    sample_rate,
                    channels,
                    format: parse_audio_format(&format)?,
                }
            }
            JsonPacketKind::Video { width, height, format } => {
                PacketKind::Video {
                    width,
                    height,
                    format: parse_pixel_format(&format)?,
                }
            }
            JsonPacketKind::Tensor { shape, dtype } => PacketKind::Tensor {
                shape,
                dtype: parse_tensor_dtype(&dtype)?,
            },
            JsonPacketKind::Event { kind } => PacketKind::Event(parse_event_kind(&kind)?),
            JsonPacketKind::Control { kind } => PacketKind::Control(parse_control_kind(&kind)?),
        };

        let data = base64::decode(&self.data)
            .map_err(|e| Error::Decode(format!("Base64 decode error: {}", e)))?;

        // Build metadata
        let mut metadata = caret_core::Metadata::new();
        for (k, v) in self.metadata {
            metadata.insert(k, v);
        }

        // Reconstruct packet using from_parts
        Ok(Packet::from_parts(
            kind,
            Bytes::from(data),
            caret_core::Timestamp::from_nanos(self.timestamp),
            self.duration,
            caret_core::StreamId::new(self.stream_id),
            metadata,
        ))
    }
}

fn parse_audio_format(s: &str) -> Result<AudioFormat> {
    Ok(match s {
        "U8" => AudioFormat::U8,
        "S16" => AudioFormat::S16,
        "S32" => AudioFormat::S32,
        "F32" => AudioFormat::F32,
        "F64" => AudioFormat::F64,
        _ => return Err(Error::Decode(format!("Unknown audio format: {}", s))),
    })
}

fn parse_pixel_format(s: &str) -> Result<PixelFormat> {
    Ok(match s {
        "Rgb8" => PixelFormat::Rgb8,
        "Rgba8" => PixelFormat::Rgba8,
        "Gray8" => PixelFormat::Gray8,
        "Yuv420p" => PixelFormat::Yuv420p,
        "Yuv422p" => PixelFormat::Yuv422p,
        _ => return Err(Error::Decode(format!("Unknown pixel format: {}", s))),
    })
}

fn parse_tensor_dtype(s: &str) -> Result<TensorDtype> {
    Ok(match s {
        "U8" => TensorDtype::U8,
        "S8" => TensorDtype::S8,
        "U16" => TensorDtype::U16,
        "S16" => TensorDtype::S16,
        "S32" => TensorDtype::S32,
        "F32" => TensorDtype::F32,
        "F64" => TensorDtype::F64,
        _ => return Err(Error::Decode(format!("Unknown tensor dtype: {}", s))),
    })
}

fn parse_event_kind(s: &str) -> Result<EventKind> {
    Ok(match s {
        "Start" => EventKind::Start,
        "End" => EventKind::End,
        "Flush" => EventKind::Flush,
        "Marker" => EventKind::Marker,
        other if other.starts_with("Custom(") => {
            let num = other
                .strip_prefix("Custom(")
                .and_then(|s| s.strip_suffix(')'))
                .ok_or_else(|| Error::Decode(format!("Invalid event kind: {}", s)))?;
            let val = num.parse::<u32>()
                .map_err(|_| Error::Decode(format!("Invalid event kind: {}", s)))?;
            EventKind::Custom(val)
        }
        _ => return Err(Error::Decode(format!("Unknown event kind: {}", s))),
    })
}

fn parse_control_kind(s: &str) -> Result<ControlKind> {
    Ok(match s {
        "Pause" => ControlKind::Pause,
        "Resume" => ControlKind::Resume,
        "Seek" => ControlKind::Seek,
        "Stop" => ControlKind::Stop,
        other if other.starts_with("Custom(") => {
            let num = other
                .strip_prefix("Custom(")
                .and_then(|s| s.strip_suffix(')'))
                .ok_or_else(|| Error::Decode(format!("Invalid control kind: {}", s)))?;
            let val = num.parse::<u32>()
                .map_err(|_| Error::Decode(format!("Invalid control kind: {}", s)))?;
            ControlKind::Custom(val)
        }
        _ => return Err(Error::Decode(format!("Unknown control kind: {}", s))),
    })
}

/// Simple base64 encoding/decoding
mod base64 {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn encode(data: &[u8]) -> String {
        let mut result = String::new();
        let chunks = data.chunks(3);

        for chunk in chunks {
            let mut group = [0u8; 3];
            group[..chunk.len()].copy_from_slice(chunk);

            let triple = (group[0] as u32) << 16 | (group[1] as u32) << 8 | group[2] as u32;

            result.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
            result.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
            if chunk.len() > 1 {
                result.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
            if chunk.len() > 2 {
                result.push(TABLE[(triple & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
        }

        result
    }

    pub fn decode(data: &str) -> std::result::Result<Vec<u8>, String> {
        let mut lookup = [0u8; 256];
        for (i, &c) in TABLE.iter().enumerate() {
            lookup[c as usize] = i as u8;
        }

        let data = data.trim_end_matches('=');
        let mut result = Vec::new();

        // Process in groups of 4 characters (3 bytes when decoded)
        for chunk in data.as_bytes().chunks(4) {
            if chunk.is_empty() {
                continue;
            }

            // Validate all chars are in the lookup table
            for &c in chunk {
                if lookup[c as usize] == 0 && c != b'A' {
                    return Err(format!("Invalid base64 character: {}", c));
                }
            }

            let c0 = lookup[chunk[0] as usize] as u32;
            let c1 = if chunk.len() > 1 { lookup[chunk[1] as usize] as u32 } else { 0 };
            let c2 = if chunk.len() > 2 { lookup[chunk[2] as usize] as u32 } else { 0 };
            let c3 = if chunk.len() > 3 { lookup[chunk[3] as usize] as u32 } else { 0 };

            let triple = (c0 << 18) | (c1 << 12) | (c2 << 6) | c3;

            result.push((triple >> 16) as u8);
            if chunk.len() > 2 {
                result.push((triple >> 8) as u8);
            }
            if chunk.len() > 3 {
                result.push(triple as u8);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_codec_bytes_roundtrip() {
        let codec = JsonCodec::new();
        let original = Packet::bytes(&b"hello world"[..]);

        let encoded = codec.encode(&original).unwrap();
        let decoded = codec.decode(&encoded).unwrap();

        assert_eq!(decoded.timestamp(), original.timestamp());
        assert_eq!(decoded.data(), original.data());
        assert!(matches!(decoded.kind(), PacketKind::Bytes));
    }

    #[test]
    fn test_json_codec_audio_roundtrip() {
        let codec = JsonCodec::new();
        let original = Packet::audio(vec![0u8; 1024], 48000, 2, AudioFormat::S16);

        let encoded = codec.encode(&original).unwrap();
        let decoded = codec.decode(&encoded).unwrap();

        if let PacketKind::Audio { sample_rate, channels, format } = decoded.kind() {
            assert_eq!(*sample_rate, 48000);
            assert_eq!(*channels, 2);
            assert_eq!(*format, AudioFormat::S16);
        } else {
            panic!("Expected audio packet");
        }
    }

    #[test]
    fn test_json_codec_pretty() {
        let codec = JsonCodec::pretty();
        let packet = Packet::bytes(&b"test"[..]);

        let encoded = codec.encode(&packet).unwrap();
        let json_str = std::str::from_utf8(&encoded).unwrap();

        assert!(json_str.contains('\n'));
        assert!(json_str.contains("  "));
    }

    #[test]
    fn test_json_codec_name() {
        let codec = JsonCodec::new();
        assert_eq!(codec.name(), "json");
    }

    #[test]
    fn test_base64_encode_decode() {
        let data = b"hello world";
        let encoded = base64::encode(data);
        let decoded = base64::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_encode_empty() {
        let data = b"";
        let encoded = base64::encode(data);
        assert_eq!(encoded, "");
    }

    #[test]
    fn test_base64_known_values() {
        assert_eq!(base64::encode(b""), "");
        assert_eq!(base64::encode(b"f"), "Zg==");
        assert_eq!(base64::encode(b"fo"), "Zm8=");
        assert_eq!(base64::encode(b"foo"), "Zm9v");
        assert_eq!(base64::encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64::encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64::encode(b"foobar"), "Zm9vYmFy");

        // Test decode
        assert_eq!(base64::decode("Zm9vYmFy").unwrap(), b"foobar");
        assert_eq!(base64::decode("aGVsbG8gd29ybGQ=").unwrap(), b"hello world");
    }
}
