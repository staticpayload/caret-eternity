// Caret Core - Packet types
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{metadata::Metadata, time::Timestamp};
use bytes::Bytes;
use std::sync::Arc;

/// A stream identifier
///
/// Uniquely identifies a stream within a pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StreamId {
    id: u64,
}

impl StreamId {
    /// Create a new stream ID from a numeric value
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    /// Get the numeric value
    pub fn as_u64(&self) -> u64 {
        self.id
    }

    /// Create a new unique stream ID
    pub fn unique() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
        }
    }
}

/// The kind of data contained in a packet
#[derive(Clone, Debug, PartialEq)]
pub enum PacketKind {
    /// Raw bytes
    Bytes,

    /// Audio buffer with sample rate and channel info
    Audio {
        sample_rate: u32,
        channels: u16,
        format: AudioFormat,
    },

    /// Video frame with resolution and pixel format
    Video {
        width: u32,
        height: u32,
        format: PixelFormat,
    },

    /// Tensor-like multi-dimensional array
    Tensor {
        shape: Vec<usize>,
        dtype: TensorDtype,
    },

    /// Control event
    Event(EventKind),

    /// Control signal
    Control(ControlKind),
}

/// Audio sample format
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioFormat {
    /// Unsigned 8-bit PCM
    U8,
    /// Signed 16-bit PCM
    S16,
    /// Signed 32-bit PCM
    S32,
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
}

impl AudioFormat {
    /// Convert to u8 for serialization
    pub fn as_u8(self) -> u8 {
        match self {
            AudioFormat::U8 => 0,
            AudioFormat::S16 => 1,
            AudioFormat::S32 => 2,
            AudioFormat::F32 => 3,
            AudioFormat::F64 => 4,
        }
    }

    /// Convert from u8
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(AudioFormat::U8),
            1 => Some(AudioFormat::S16),
            2 => Some(AudioFormat::S32),
            3 => Some(AudioFormat::F32),
            4 => Some(AudioFormat::F64),
            _ => None,
        }
    }
}

/// Pixel format for video frames
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    /// RGB 8-bit per channel
    Rgb8,
    /// RGBA 8-bit per channel
    Rgba8,
    /// Grayscale 8-bit
    Gray8,
    /// YUV 4:2:0 planar
    Yuv420p,
    /// YUV 4:2:2 planar
    Yuv422p,
}

impl PixelFormat {
    /// Convert to u8 for serialization
    pub fn as_u8(self) -> u8 {
        match self {
            PixelFormat::Rgb8 => 0,
            PixelFormat::Rgba8 => 1,
            PixelFormat::Gray8 => 2,
            PixelFormat::Yuv420p => 3,
            PixelFormat::Yuv422p => 4,
        }
    }

    /// Convert from u8
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(PixelFormat::Rgb8),
            1 => Some(PixelFormat::Rgba8),
            2 => Some(PixelFormat::Gray8),
            3 => Some(PixelFormat::Yuv420p),
            4 => Some(PixelFormat::Yuv422p),
            _ => None,
        }
    }
}

/// Data type for tensor elements
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TensorDtype {
    /// Unsigned 8-bit integer
    U8,
    /// Signed 8-bit integer
    S8,
    /// Unsigned 16-bit integer
    U16,
    /// Signed 16-bit integer
    S16,
    /// Signed 32-bit integer
    S32,
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
}

impl TensorDtype {
    /// Convert to u8 for serialization
    pub fn as_u8(self) -> u8 {
        match self {
            TensorDtype::U8 => 0,
            TensorDtype::S8 => 1,
            TensorDtype::U16 => 2,
            TensorDtype::S16 => 3,
            TensorDtype::S32 => 4,
            TensorDtype::F32 => 5,
            TensorDtype::F64 => 6,
        }
    }

    /// Convert from u8
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(TensorDtype::U8),
            1 => Some(TensorDtype::S8),
            2 => Some(TensorDtype::U16),
            3 => Some(TensorDtype::S16),
            4 => Some(TensorDtype::S32),
            5 => Some(TensorDtype::F32),
            6 => Some(TensorDtype::F64),
            _ => None,
        }
    }
}

/// Event kind for control events
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKind {
    /// Stream started
    Start,
    /// Stream ended
    End,
    /// Stream flush requested
    Flush,
    /// Marker for synchronization
    Marker,
    /// Custom event
    Custom(u32),
}

impl EventKind {
    /// Convert to u8 for serialization
    pub fn as_u8(self) -> u8 {
        match self {
            EventKind::Start => 0,
            EventKind::End => 1,
            EventKind::Flush => 2,
            EventKind::Marker => 3,
            EventKind::Custom(v) => 128 + (v % 128) as u8,
        }
    }

    /// Convert from u8
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(EventKind::Start),
            1 => Some(EventKind::End),
            2 => Some(EventKind::Flush),
            3 => Some(EventKind::Marker),
            v if v >= 128 => Some(EventKind::Custom((v - 128) as u32)),
            _ => None,
        }
    }
}

/// Control signal kind
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlKind {
    /// Pause the stream
    Pause,
    /// Resume the stream
    Resume,
    /// Seek to a position
    Seek,
    /// Stop the stream
    Stop,
    /// Custom control
    Custom(u32),
}

impl ControlKind {
    /// Convert to u8 for serialization
    pub fn as_u8(self) -> u8 {
        match self {
            ControlKind::Pause => 0,
            ControlKind::Resume => 1,
            ControlKind::Seek => 2,
            ControlKind::Stop => 3,
            ControlKind::Custom(v) => 128 + (v % 128) as u8,
        }
    }

    /// Convert from u8
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(ControlKind::Pause),
            1 => Some(ControlKind::Resume),
            2 => Some(ControlKind::Seek),
            3 => Some(ControlKind::Stop),
            v if v >= 128 => Some(ControlKind::Custom((v - 128) as u32)),
            _ => None,
        }
    }
}

/// A packet of data flowing through the pipeline
///
/// Packets are the primary unit of data in Caret. Each packet
/// carries typed data, timing information, and optional metadata.
#[derive(Clone)]
pub struct Packet {
    /// The kind of data in this packet
    kind: PacketKind,

    /// The packet data
    data: Bytes,

    /// Monotonic timestamp
    timestamp: Timestamp,

    /// Duration of the packet (for media)
    duration: Option<u64>,

    /// Stream identifier
    stream_id: StreamId,

    /// Optional metadata
    metadata: Arc<Metadata>,
}

impl Packet {
    /// Create a new bytes packet
    pub fn bytes(data: impl Into<Bytes>) -> Self {
        Self::new(PacketKind::Bytes, data)
    }

    /// Create a new audio packet
    pub fn audio(
        data: impl Into<Bytes>,
        sample_rate: u32,
        channels: u16,
        format: AudioFormat,
    ) -> Self {
        Self::with_kind(
            PacketKind::Audio {
                sample_rate,
                channels,
                format,
            },
            data,
        )
    }

    /// Create a new video packet
    pub fn video(data: impl Into<Bytes>, width: u32, height: u32, format: PixelFormat) -> Self {
        Self::with_kind(
            PacketKind::Video {
                width,
                height,
                format,
            },
            data,
        )
    }

    /// Create a new tensor packet
    pub fn tensor(data: impl Into<Bytes>, shape: Vec<usize>, dtype: TensorDtype) -> Self {
        Self::with_kind(PacketKind::Tensor { shape, dtype }, data)
    }

    /// Create a new event packet
    pub fn event(kind: EventKind) -> Self {
        Self::with_kind(PacketKind::Event(kind), Bytes::new())
    }

    /// Create a new control packet
    pub fn control(kind: ControlKind) -> Self {
        Self::with_kind(PacketKind::Control(kind), Bytes::new())
    }

    /// Create a new packet with a specific kind
    fn new(kind: PacketKind, data: impl Into<Bytes>) -> Self {
        Self {
            kind,
            data: data.into(),
            timestamp: Timestamp::now(),
            duration: None,
            stream_id: StreamId::unique(),
            metadata: Arc::new(Metadata::new()),
        }
    }

    /// Create a packet with all fields
    fn with_kind(kind: PacketKind, data: impl Into<Bytes>) -> Self {
        Self::new(kind, data)
    }

    /// Get the packet kind
    pub fn kind(&self) -> &PacketKind {
        &self.kind
    }

    /// Get the packet data
    pub fn data(&self) -> &Bytes {
        &self.data
    }

    /// Get the packet timestamp
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    /// Set the packet timestamp
    pub fn with_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Get the packet duration in nanoseconds
    pub fn duration(&self) -> Option<Timestamp> {
        self.duration.map(Timestamp::from_nanos)
    }

    /// Set the packet duration in nanoseconds
    pub fn with_duration(mut self, duration: u64) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Get the stream ID
    pub fn stream_id(&self) -> StreamId {
        self.stream_id
    }

    /// Set the stream ID
    pub fn with_stream_id(mut self, stream_id: StreamId) -> Self {
        self.stream_id = stream_id;
        self
    }

    /// Get the packet metadata
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Set the packet metadata
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Arc::new(metadata);
        self
    }

    /// Get the data length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the packet has no data
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Check if this is a control packet
    pub fn is_control(&self) -> bool {
        matches!(self.kind, PacketKind::Control(_))
    }

    /// Check if this is an event packet
    pub fn is_event(&self) -> bool {
        matches!(self.kind, PacketKind::Event(_))
    }

    /// Reconstruct a packet from its component parts.
    ///
    /// This is primarily used by codecs when deserializing packets.
    /// The metadata is consumed and wrapped in an Arc.
    pub fn from_parts(
        kind: PacketKind,
        data: impl Into<Bytes>,
        timestamp: Timestamp,
        duration: Option<u64>,
        stream_id: StreamId,
        metadata: Metadata,
    ) -> Self {
        Self {
            kind,
            data: data.into(),
            timestamp,
            duration,
            stream_id,
            metadata: Arc::new(metadata),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_id() {
        let id1 = StreamId::new(42);
        assert_eq!(id1.as_u64(), 42);

        let id2 = StreamId::unique();
        let id3 = StreamId::unique();
        assert_ne!(id2.as_u64(), id3.as_u64());
    }

    #[test]
    fn test_bytes_packet() {
        let packet = Packet::bytes(&b"hello world"[..]);
        assert!(matches!(packet.kind(), PacketKind::Bytes));
        assert_eq!(packet.data().as_ref(), b"hello world");
        assert_eq!(packet.len(), 11);
        assert!(!packet.is_empty());
        assert!(!packet.is_control());
        assert!(!packet.is_event());
    }

    #[test]
    fn test_audio_packet() {
        let data = vec![0u8; 1024];
        let packet = Packet::audio(data.clone(), 48000, 2, AudioFormat::S16);

        match packet.kind() {
            PacketKind::Audio {
                sample_rate,
                channels,
                format,
            } => {
                assert_eq!(*sample_rate, 48000);
                assert_eq!(*channels, 2);
                assert_eq!(*format, AudioFormat::S16);
            }
            _ => panic!("Expected audio packet"),
        }
    }

    #[test]
    fn test_video_packet() {
        let data = vec![0u8; 640 * 480 * 3];
        let packet = Packet::video(data.clone(), 640, 480, PixelFormat::Rgb8);

        match packet.kind() {
            PacketKind::Video {
                width,
                height,
                format,
            } => {
                assert_eq!(*width, 640);
                assert_eq!(*height, 480);
                assert_eq!(*format, PixelFormat::Rgb8);
            }
            _ => panic!("Expected video packet"),
        }
    }

    #[test]
    fn test_event_packet() {
        let packet = Packet::event(EventKind::Start);
        assert!(packet.is_event());
        assert_eq!(packet.len(), 0);
        assert!(packet.is_empty());
    }

    #[test]
    fn test_control_packet() {
        let packet = Packet::control(ControlKind::Pause);
        assert!(packet.is_control());
        assert_eq!(packet.len(), 0);
    }

    #[test]
    fn test_packet_with_timestamp() {
        let ts = Timestamp::from_secs(10);
        let packet = Packet::bytes(&b"data"[..]).with_timestamp(ts);
        assert_eq!(packet.timestamp(), ts);
    }

    #[test]
    fn test_packet_with_duration() {
        let packet = Packet::bytes(&b"data"[..]).with_duration(1_000_000_000);
        assert_eq!(packet.duration(), Some(Timestamp::from_secs(1)));
    }
}
