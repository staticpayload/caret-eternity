// Caret Record - Event recorder
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{Error, Event, Result, SessionId, Timestamp, NodeId, PacketEvent};
use crate::{MetricEvent, StateChangeEvent, ErrorEvent};
use crate::event::{PortId, MetricValue, ErrorSeverity};
use crate::store::EventStore;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;

/// Configuration for the event recorder
#[derive(Debug, Clone)]
pub struct RecorderConfig {
    /// Maximum number of events to record (0 = unlimited)
    pub max_events: usize,
    /// Whether to record packet contents
    pub record_packets: bool,
    /// Whether to record metrics
    pub record_metrics: bool,
    /// Whether to record state changes
    pub record_state_changes: bool,
}

impl Default for RecorderConfig {
    fn default() -> Self {
        Self {
            max_events: 0, // Unlimited
            record_packets: true,
            record_metrics: true,
            record_state_changes: true,
        }
    }
}

impl RecorderConfig {
    /// Create a new recorder config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of events
    pub fn with_max_events(mut self, max: usize) -> Self {
        self.max_events = max;
        self
    }

    /// Disable packet recording
    pub fn without_packets(mut self) -> Self {
        self.record_packets = false;
        self
    }

    /// Disable metrics recording
    pub fn without_metrics(mut self) -> Self {
        self.record_metrics = false;
        self
    }

    /// Disable state change recording
    pub fn without_state_changes(mut self) -> Self {
        self.record_state_changes = false;
        self
    }
}

/// Event recorder for capturing runtime events
pub struct EventRecorder {
    /// Current session ID
    session_id: Arc<Mutex<Option<SessionId>>>,
    /// Event store
    store: Arc<dyn EventStore>,
    /// Recording start time
    start_time: Arc<Mutex<Option<Instant>>>,
    /// Current tick
    tick: Arc<Mutex<u64>>,
    /// Configuration
    config: RecorderConfig,
    /// Number of events recorded in current session
    event_count: Arc<Mutex<usize>>,
}

impl EventRecorder {
    /// Create a new event recorder
    pub fn new(store: Arc<dyn EventStore>, config: RecorderConfig) -> Self {
        Self {
            session_id: Arc::new(Mutex::new(None)),
            store,
            start_time: Arc::new(Mutex::new(None)),
            tick: Arc::new(Mutex::new(0)),
            config,
            event_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Create with default config
    pub fn with_store(store: Arc<dyn EventStore>) -> Self {
        Self::new(store, RecorderConfig::default())
    }

    /// Start a new recording session
    pub fn start(&self, description: Option<String>) -> Result<SessionId> {
        let mut session_id = self.session_id.lock();
        if session_id.is_some() {
            return Err(Error::RecordingInProgress);
        }

        let id = SessionId::new();
        *session_id = Some(id);

        *self.start_time.lock() = Some(Instant::now());
        *self.tick.lock() = 0;
        *self.event_count.lock() = 0;

        // Create metadata
        let mut metadata = crate::RecordingMetadata::new(id);
        if let Some(desc) = description {
            metadata = metadata.with_description(desc);
        }
        self.store.save_metadata(&metadata)?;

        Ok(id)
    }

    /// Stop the current recording session
    pub fn stop(&self) -> Result<Option<SessionId>> {
        let mut session_id = self.session_id.lock();
        let id = session_id.take();

        if let Some(sid) = id {
            // Update metadata with end time
            if let Ok(Some(mut metadata)) = self.store.get_metadata(sid) {
                metadata.mark_ended();
                let _ = self.store.save_metadata(&metadata);
            }
        }

        *self.start_time.lock() = None;
        Ok(id)
    }

    /// Check if recording is active
    pub fn is_recording(&self) -> bool {
        self.session_id.lock().is_some()
    }

    /// Get the current session ID
    pub fn current_session(&self) -> Option<SessionId> {
        *self.session_id.lock()
    }

    /// Set the current tick
    pub fn set_tick(&self, tick: u64) {
        *self.tick.lock() = tick;
    }

    /// Get the current tick
    pub fn tick(&self) -> u64 {
        *self.tick.lock()
    }

    /// Get the current timestamp relative to recording start
    fn current_timestamp(&self) -> Timestamp {
        if let Some(start) = *self.start_time.lock() {
            start.elapsed().as_nanos() as u64
        } else {
            0
        }
    }

    /// Record an event
    fn record(&self, mut event: Event) -> Result<()> {
        let session_id = *self.session_id.lock();
        if session_id.is_none() {
            return Ok(()); // Not recording, silently ignore
        }

        event.timestamp = self.current_timestamp();
        event.tick = *self.tick.lock();

        // Check event count limit
        if self.config.max_events > 0 {
            let mut count = self.event_count.lock();
            if *count >= self.config.max_events {
                return Ok(()); // Limit reached
            }
            *count += 1;
        }

        self.store.store_event(event)
    }

    /// Record a packet flow event
    pub fn record_packet(&self, source: Option<PortId>, destination: PortId, packet: &caret_core::Packet) -> Result<()> {
        if !self.config.record_packets {
            return Ok(());
        }

        let session_id = *self.session_id.lock();
        let session_id = session_id.ok_or(Error::NoRecordingInProgress)?;

        let packet_event = PacketEvent::new(source, destination, packet)?;
        let event = Event::packet(session_id, *self.tick.lock(), packet_event);
        self.record(event)
    }

    /// Record a state change event
    pub fn record_state_change(&self, node: NodeId, from: impl Into<String>, to: impl Into<String>) -> Result<()> {
        if !self.config.record_state_changes {
            return Ok(());
        }

        let session_id = *self.session_id.lock();
        let session_id = session_id.ok_or(Error::NoRecordingInProgress)?;

        let state_event = StateChangeEvent::new(node, from, to);
        let event = Event::state_change(session_id, *self.tick.lock(), state_event);
        self.record(event)
    }

    /// Record a metric event
    pub fn record_metric(&self, node: NodeId, name: String, value: MetricValue) -> Result<()> {
        if !self.config.record_metrics {
            return Ok(());
        }

        let session_id = *self.session_id.lock();
        let session_id = session_id.ok_or(Error::NoRecordingInProgress)?;

        let metric_event = MetricEvent::new(node, name, value);
        let event = Event::metric(session_id, *self.tick.lock(), metric_event);
        self.record(event)
    }

    /// Record an error event
    pub fn record_error(&self, node: NodeId, message: String, severity: ErrorSeverity) -> Result<()> {
        let session_id = *self.session_id.lock();
        let session_id = session_id.ok_or(Error::NoRecordingInProgress)?;

        let error_event = ErrorEvent::new(node, message, severity);
        let event = Event::error(session_id, *self.tick.lock(), error_event);
        self.record(event)
    }

    /// Get the number of events recorded in the current session
    pub fn event_count(&self) -> usize {
        *self.event_count.lock()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryStore;

    #[test]
    fn test_recorder_start_stop() {
        let store = Arc::new(MemoryStore::new());
        let recorder = EventRecorder::with_store(store.clone());

        assert!(!recorder.is_recording());

        let session = recorder.start(Some("Test recording".to_string())).unwrap();
        assert!(recorder.is_recording());
        assert_eq!(recorder.current_session(), Some(session));

        let stopped = recorder.stop().unwrap();
        assert_eq!(stopped, Some(session));
        assert!(!recorder.is_recording());
    }

    #[test]
    fn test_recorder_tick() {
        let store = Arc::new(MemoryStore::new());
        let recorder = EventRecorder::with_store(store);

        recorder.set_tick(42);
        assert_eq!(recorder.tick(), 42);
    }

    #[test]
    fn test_recorder_state_change() {
        let store = Arc::new(MemoryStore::new());
        let recorder = EventRecorder::with_store(store.clone());

        let session = recorder.start(None).unwrap();
        recorder.set_tick(5);

        let node = NodeId::new("node1".to_string());
        recorder.record_state_change(node.clone(), "Idle", "Running").unwrap();

        recorder.stop().unwrap();

        let events = store.load_events(session).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].tick, 5);
    }

    #[test]
    fn test_recorder_config() {
        let config = RecorderConfig::new()
            .with_max_events(100)
            .without_metrics();

        assert_eq!(config.max_events, 100);
        assert!(!config.record_metrics);
    }

    #[test]
    fn test_recorder_event_count_limit() {
        let store = Arc::new(MemoryStore::new());
        let config = RecorderConfig::new()
            .with_max_events(2)
            .without_packets()
            .without_metrics();
        let recorder = EventRecorder::new(store.clone(), config);

        recorder.start(None).unwrap();

        let node = NodeId::new("node1".to_string());

        // Record 3 events, but limit is 2
        recorder.record_state_change(node.clone(), "Idle1", "Running1").unwrap();
        recorder.record_state_change(node.clone(), "Idle2", "Running2").unwrap();
        recorder.record_state_change(node.clone(), "Idle3", "Running3").unwrap();

        let session = recorder.stop().unwrap();
        let events = store.load_events(session.unwrap()).unwrap();

        // Should only have 2 events due to limit
        assert_eq!(events.len(), 2);
    }
}
