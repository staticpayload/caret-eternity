// Caret Record - Event replayer
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{Event, EventData, SessionId, Result};
use crate::store::EventStore;
use std::sync::Arc;
use std::time::Duration;

/// Configuration for the event replayer
#[derive(Debug, Clone)]
pub struct ReplayerConfig {
    /// Whether to replay in real-time (respecting original timing)
    pub real_time: bool,
    /// Speed multiplier for real-time replay (1.0 = original speed)
    pub speed_multiplier: f64,
    /// Maximum number of events to replay (0 = unlimited)
    pub max_events: usize,
    /// Stop on error
    pub stop_on_error: bool,
}

impl Default for ReplayerConfig {
    fn default() -> Self {
        Self {
            real_time: false,
            speed_multiplier: 1.0,
            max_events: 0,
            stop_on_error: false,
        }
    }
}

impl ReplayerConfig {
    /// Create a new replayer config
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable real-time replay
    pub fn real_time(mut self) -> Self {
        self.real_time = true;
        self
    }

    /// Set the speed multiplier
    pub fn with_speed(mut self, speed: f64) -> Self {
        self.speed_multiplier = speed;
        self
    }

    /// Set the maximum number of events
    pub fn with_max_events(mut self, max: usize) -> Self {
        self.max_events = max;
        self
    }

    /// Stop on error
    pub fn stop_on_error(mut self) -> Self {
        self.stop_on_error = true;
        self
    }
}

/// Result of a replay operation
#[derive(Debug, Clone)]
pub struct ReplayResult {
    /// Number of events replayed
    pub events_replayed: usize,
    /// Number of errors encountered
    pub errors: usize,
    /// Duration of replay
    pub duration: Duration,
    /// Whether the replay completed successfully
    pub completed: bool,
}

impl ReplayResult {
    /// Create a new replay result
    pub fn new() -> Self {
        Self {
            events_replayed: 0,
            errors: 0,
            duration: Duration::default(),
            completed: false,
        }
    }

    /// Check if the replay was successful
    pub fn is_success(&self) -> bool {
        self.completed && self.errors == 0
    }
}

impl Default for ReplayResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler for replay events
pub trait ReplayHandler: Send + Sync {
    /// Handle a packet event
    fn handle_packet(&mut self, event: crate::PacketEvent) -> Result<()> {
        let _ = event;
        Ok(())
    }

    /// Handle a state change event
    fn handle_state_change(&mut self, event: crate::StateChangeEvent) -> Result<()> {
        let _ = event;
        Ok(())
    }

    /// Handle a metric event
    fn handle_metric(&mut self, event: crate::MetricEvent) -> Result<()> {
        let _ = event;
        Ok(())
    }

    /// Handle an error event
    fn handle_error(&mut self, event: crate::ErrorEvent) -> Result<()> {
        let _ = event;
        Ok(())
    }

    /// Handle a marker event
    fn handle_marker(&mut self, marker: String) -> Result<()> {
        let _ = marker;
        Ok(())
    }
}

/// Event replayer for replaying recorded events
pub struct EventReplayer {
    /// Event store
    store: Arc<dyn EventStore>,
    /// Configuration
    config: ReplayerConfig,
}

impl EventReplayer {
    /// Create a new event replayer
    pub fn new(store: Arc<dyn EventStore>, config: ReplayerConfig) -> Self {
        Self { store, config }
    }

    /// Create with default config
    pub fn with_store(store: Arc<dyn EventStore>) -> Self {
        Self::new(store, ReplayerConfig::default())
    }

    /// Replay events from a session
    pub async fn replay(&self, session_id: SessionId, handler: &mut dyn ReplayHandler) -> Result<ReplayResult> {
        let mut result = ReplayResult::new();
        let start = std::time::Instant::now();

        let events = self.store.load_events(session_id)?;
        if events.is_empty() {
            return Ok(result);
        }

        // Sort events by timestamp
        let mut events = events;
        events.sort_by_key(|e| e.timestamp);

        let mut last_timestamp = None;
        let event_limit = self.config.max_events;

        for (i, event) in events.iter().enumerate() {
            if event_limit > 0 && i >= event_limit {
                break;
            }

            // Handle timing for real-time replay
            if self.config.real_time {
                if let Some(last) = last_timestamp {
                    let elapsed = event.timestamp.saturating_sub(last);
                    if elapsed > 0 {
                        let delay = Duration::from_nanos(
                            (elapsed as f64 / self.config.speed_multiplier) as u64
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
                last_timestamp = Some(event.timestamp);
            }

            // Handle the event
            match self.handle_event(event, handler) {
                Ok(()) => {
                    result.events_replayed += 1;
                }
                Err(e) => {
                    result.errors += 1;
                    if self.config.stop_on_error {
                        result.duration = start.elapsed();
                        return Ok(result);
                    }
                    // Log the error but continue
                    tracing::warn!("Replay error: {}", e);
                }
            }
        }

        result.completed = true;
        result.duration = start.elapsed();
        Ok(result)
    }

    /// Handle a single event
    fn handle_event(&self, event: &Event, handler: &mut dyn ReplayHandler) -> Result<()> {
        match &event.data {
            EventData::Packet(packet) => {
                handler.handle_packet(packet.clone())?;
            }
            EventData::StateChange(state) => {
                handler.handle_state_change(state.clone())?;
            }
            EventData::Metric(metric) => {
                handler.handle_metric(metric.clone())?;
            }
            EventData::Error(error) => {
                handler.handle_error(error.clone())?;
            }
            EventData::Marker(marker) => {
                handler.handle_marker(marker.clone())?;
            }
        }
        Ok(())
    }

    /// Get the metadata for a session
    pub fn get_metadata(&self, session_id: SessionId) -> Result<Option<crate::RecordingMetadata>> {
        self.store.get_metadata(session_id)
    }

    /// List all available sessions
    pub fn list_sessions(&self) -> Result<Vec<SessionId>> {
        self.store.list_sessions()
    }
}

/// Simple test handler that collects events
#[derive(Debug, Default)]
pub struct TestHandler {
    /// Collected packet events
    pub packets: Vec<crate::PacketEvent>,
    /// Collected state change events
    pub state_changes: Vec<crate::StateChangeEvent>,
    /// Collected metric events
    pub metrics: Vec<crate::MetricEvent>,
    /// Collected error events
    pub errors: Vec<crate::ErrorEvent>,
    /// Collected markers
    pub markers: Vec<String>,
}

impl ReplayHandler for TestHandler {
    fn handle_packet(&mut self, event: crate::PacketEvent) -> Result<()> {
        self.packets.push(event);
        Ok(())
    }

    fn handle_state_change(&mut self, event: crate::StateChangeEvent) -> Result<()> {
        self.state_changes.push(event);
        Ok(())
    }

    fn handle_metric(&mut self, event: crate::MetricEvent) -> Result<()> {
        self.metrics.push(event);
        Ok(())
    }

    fn handle_error(&mut self, event: crate::ErrorEvent) -> Result<()> {
        self.errors.push(event);
        Ok(())
    }

    fn handle_marker(&mut self, marker: String) -> Result<()> {
        self.markers.push(marker);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryStore;
    use crate::{Event, EventType, EventData, StateChangeEvent, MetricEvent, NodeId};
    use crate::event::MetricValue;

    #[test]
    fn test_replayer_config() {
        let config = ReplayerConfig::new()
            .real_time()
            .with_speed(2.0)
            .with_max_events(100);

        assert!(config.real_time);
        assert_eq!(config.speed_multiplier, 2.0);
        assert_eq!(config.max_events, 100);
    }

    #[tokio::test]
    async fn test_replay_events() {
        let store = Arc::new(MemoryStore::new());
        let session_id = SessionId::new();

        // Create some test events
        let node = NodeId::new("node1".to_string());
        let state_event = Event::new(
            session_id,
            EventType::StateChange,
            0,
            EventData::StateChange(StateChangeEvent::new(node.clone(), "Idle", "Running")),
        );

        let metric_event = Event::new(
            session_id,
            EventType::Metric,
            1,
            EventData::Metric(MetricEvent::new(node.clone(), "count".to_string(), MetricValue::Counter(42))),
        );

        store.store_event(state_event).unwrap();
        store.store_event(metric_event).unwrap();

        // Replay
        let replayer = EventReplayer::with_store(store);
        let mut handler = TestHandler::default();
        let result = replayer.replay(session_id, &mut handler).await.unwrap();

        assert!(result.is_success());
        assert_eq!(result.events_replayed, 2);
        assert_eq!(handler.state_changes.len(), 1);
        assert_eq!(handler.metrics.len(), 1);
    }

    #[test]
    fn test_replay_result() {
        let result = ReplayResult::new();
        assert!(!result.is_success());

        let result = ReplayResult {
            events_replayed: 10,
            errors: 0,
            duration: Duration::from_millis(100),
            completed: true,
        };
        assert!(result.is_success());
    }

    #[test]
    fn test_replay_result_with_errors() {
        let result = ReplayResult {
            events_replayed: 10,
            errors: 1,
            duration: Duration::from_millis(100),
            completed: true,
        };
        assert!(!result.is_success());
    }
}
