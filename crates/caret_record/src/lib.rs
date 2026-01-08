// Caret Record - Event recording and replay
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

//! Caret Record - Event recording and replay for debugging and testing
//!
//! This crate provides functionality for recording runtime events and
//! replaying them later for debugging, testing, and analysis.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod event;
mod recorder;
mod replayer;
mod store;

pub use error::{Error, Result};
pub use event::{
    Event, EventData, EventType, PacketEvent, StateChangeEvent,
    MetricEvent, ErrorEvent, Timestamp, TickId, NodeId,
};
pub use recorder::{EventRecorder, RecorderConfig};
pub use replayer::{EventReplayer, ReplayerConfig, ReplayResult};
pub use store::{EventStore, MemoryStore, FileStore, StoreBackend};

use std::sync::Arc;
use parking_lot::Mutex;

/// Recording session identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionId(u64);

impl SessionId {
    /// Create a new session ID
    pub fn new() -> Self {
        use std::time::SystemTime;
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self(nanos)
    }

    /// Get the inner value
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Recording metadata
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RecordingMetadata {
    /// Session ID
    pub session_id: u64,
    /// Recording start time (Unix timestamp)
    pub start_time: u64,
    /// Recording end time (Unix timestamp)
    pub end_time: Option<u64>,
    /// Description of what was recorded
    pub description: Option<String>,
    /// Number of events recorded
    pub event_count: usize,
    /// Ticks recorded
    pub ticks: u64,
    /// Nodes involved in the recording
    pub nodes: Vec<String>,
}

impl RecordingMetadata {
    /// Create new recording metadata
    pub fn new(session_id: SessionId) -> Self {
        Self {
            session_id: session_id.as_u64(),
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            end_time: None,
            description: None,
            event_count: 0,
            ticks: 0,
            nodes: Vec::new(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Mark the recording as ended
    pub fn mark_ended(&mut self) {
        self.end_time = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }
}

/// Recording manager for managing multiple recording sessions
pub struct RecordingManager {
    /// Current active recording
    current_recording: Arc<Mutex<Option<ActiveRecording>>>,
    /// Event store
    store: Arc<dyn EventStore>,
}

#[derive(Clone)]
struct ActiveRecording {
    session_id: SessionId,
    metadata: RecordingMetadata,
    start_tick: u64,
}

impl RecordingManager {
    /// Create a new recording manager
    pub fn new(store: Arc<dyn EventStore>) -> Self {
        Self {
            current_recording: Arc::new(Mutex::new(None)),
            store,
        }
    }

    /// Start a new recording session
    pub fn start_recording(&self, description: Option<String>) -> Result<SessionId> {
        let mut current = self.current_recording.lock();
        if current.is_some() {
            return Err(Error::RecordingInProgress);
        }

        let session_id = SessionId::new();
        let mut metadata = RecordingMetadata::new(session_id);
        if let Some(desc) = description {
            metadata = metadata.with_description(desc);
        }

        *current = Some(ActiveRecording {
            session_id,
            metadata,
            start_tick: 0,
        });

        Ok(session_id)
    }

    /// Stop the current recording session
    pub fn stop_recording(&self) -> Result<Option<SessionId>> {
        let mut current = self.current_recording.lock();
        let recording = current.take();

        if let Some(recording) = recording {
            let session_id = recording.session_id;
            let mut metadata = recording.metadata;
            metadata.mark_ended();
            self.store.save_metadata(&metadata)?;
            Ok(Some(session_id))
        } else {
            Ok(None)
        }
    }

    /// Check if recording is active
    pub fn is_recording(&self) -> bool {
        self.current_recording.lock().is_some()
    }

    /// Get the current session ID
    pub fn current_session_id(&self) -> Option<SessionId> {
        self.current_recording.lock().as_ref().map(|r| r.session_id)
    }

    /// Record an event
    pub fn record_event(&self, event: Event) -> Result<()> {
        let current = self.current_recording.lock();
        if current.is_none() {
            return Ok(()); // Silently ignore events when not recording
        }

        drop(current); // Release lock before calling store
        self.store.store_event(event)
    }

    /// Get metadata for a session
    pub fn get_metadata(&self, session_id: SessionId) -> Result<Option<RecordingMetadata>> {
        self.store.get_metadata(session_id)
    }

    /// List all recording sessions
    pub fn list_sessions(&self) -> Result<Vec<SessionId>> {
        self.store.list_sessions()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_unique() {
        let id1 = SessionId::new();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id2 = SessionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_recording_metadata() {
        let id = SessionId::new();
        let mut metadata = RecordingMetadata::new(id);
        assert_eq!(metadata.session_id, id.as_u64());

        metadata = metadata.with_description("Test recording");
        assert_eq!(metadata.description, Some("Test recording".to_string()));

        metadata.mark_ended();
        assert!(metadata.end_time.is_some());
    }
}
