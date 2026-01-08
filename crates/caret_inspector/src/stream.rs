// Caret Inspector - Event streaming via WebSocket
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::snapshot::{GraphSnapshot, MetricSnapshot, RuntimeSnapshot};
use serde::{Deserialize, Serialize};

/// Event type for streaming updates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    /// Event type
    #[serde(rename = "type")]
    pub event_type: EventType,
    /// Event data
    pub data: EventData,
    /// Timestamp (milliseconds since epoch)
    pub timestamp: u64,
}

/// Type of event
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// Runtime state changed
    RuntimeStateChanged,
    /// Graph state changed
    GraphChanged,
    /// Node state changed
    NodeStateChanged,
    /// Metric updated
    MetricUpdated,
    /// Error occurred
    Error,
    /// Log entry
    Log,
    /// Tick executed
    Tick,
}

/// Event payload data
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventData {
    /// Runtime state snapshot
    Runtime(RuntimeSnapshot),
    /// Graph state snapshot
    Graph(GraphSnapshot),
    /// Node update
    NodeUpdate(NodeUpdateEvent),
    /// Metric update
    Metric(MetricSnapshot),
    /// Error message
    Error(String),
    /// Log entry
    Log(LogEntry),
    /// Tick info
    Tick(TickInfo),
}

/// Update for a single node
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeUpdateEvent {
    /// Node ID
    pub node_id: u64,
    /// New node state
    pub state: crate::snapshot::NodeState,
    /// Packets processed
    pub packets_processed: u64,
    /// Packets dropped
    pub packets_dropped: u64,
}

/// Log entry event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    /// Log level
    pub level: LogLevel,
    /// Message
    pub message: String,
    /// Target/module
    pub target: Option<String>,
}

/// Log level
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace level
    Trace,
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warn level
    Warn,
    /// Error level
    Error,
}

/// Information about a tick
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TickInfo {
    /// Tick number
    pub tick: u64,
    /// Nodes that ran
    pub nodes_run: usize,
    /// Nodes completed
    pub nodes_done: usize,
    /// Nodes with errors
    pub nodes_error: usize,
}

/// Event stream for live updates
pub struct EventStream {
    /// Sender for events
    sender: tokio::sync::broadcast::Sender<Event>,
}

impl EventStream {
    /// Create a new event stream with the given capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    /// Publish an event
    pub fn publish(&self, event: Event) -> Result<(), Error> {
        self.sender
            .send(event)
            .map_err(|_| Error::SendFailed)?;
        Ok(())
    }

    /// Create runtime state changed event
    pub fn runtime_state_changed(&self, snapshot: RuntimeSnapshot) -> Result<(), Error> {
        let event = Event {
            event_type: EventType::RuntimeStateChanged,
            data: EventData::Runtime(snapshot),
            timestamp: self.now(),
        };
        self.publish(event)
    }

    /// Create graph changed event
    pub fn graph_changed(&self, snapshot: GraphSnapshot) -> Result<(), Error> {
        let event = Event {
            event_type: EventType::GraphChanged,
            data: EventData::Graph(snapshot),
            timestamp: self.now(),
        };
        self.publish(event)
    }

    /// Create node state changed event
    pub fn node_state_changed(
        &self,
        node_id: u64,
        state: crate::snapshot::NodeState,
        packets_processed: u64,
        packets_dropped: u64,
    ) -> Result<(), Error> {
        let event = Event {
            event_type: EventType::NodeStateChanged,
            data: EventData::NodeUpdate(NodeUpdateEvent {
                node_id,
                state,
                packets_processed,
                packets_dropped,
            }),
            timestamp: self.now(),
        };
        self.publish(event)
    }

    /// Create metric updated event
    pub fn metric_updated(&self, metric: MetricSnapshot) -> Result<(), Error> {
        let event = Event {
            event_type: EventType::MetricUpdated,
            data: EventData::Metric(metric),
            timestamp: self.now(),
        };
        self.publish(event)
    }

    /// Create error event
    pub fn error(&self, message: String) -> Result<(), Error> {
        let event = Event {
            event_type: EventType::Error,
            data: EventData::Error(message),
            timestamp: self.now(),
        };
        self.publish(event)
    }

    /// Create log event
    pub fn log(&self, level: LogLevel, message: String, target: Option<String>) -> Result<(), Error> {
        let event = Event {
            event_type: EventType::Log,
            data: EventData::Log(LogEntry {
                level,
                message,
                target,
            }),
            timestamp: self.now(),
        };
        self.publish(event)
    }

    /// Create tick event
    pub fn tick(&self, tick: u64, nodes_run: usize, nodes_done: usize, nodes_error: usize) -> Result<(), Error> {
        let event = Event {
            event_type: EventType::Tick,
            data: EventData::Tick(TickInfo {
                tick,
                nodes_run,
                nodes_done,
                nodes_error,
            }),
            timestamp: self.now(),
        };
        self.publish(event)
    }

    /// Get current timestamp in milliseconds
    fn now(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

impl Clone for EventStream {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

/// Error type for event stream operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Failed to send event (no receivers)
    SendFailed,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SendFailed => write!(f, "Failed to send event: no receivers"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = Event {
            event_type: EventType::Tick,
            data: EventData::Tick(TickInfo {
                tick: 42,
                nodes_run: 5,
                nodes_done: 2,
                nodes_error: 0,
            }),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"tick\""));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_event_stream() {
        let stream = EventStream::new(10);
        let mut rx = stream.subscribe();

        stream
            .tick(1, 2, 0, 0)
            .unwrap();

        let received = rx.blocking_recv().unwrap();
        assert!(matches!(received.event_type, EventType::Tick));
    }

    #[test]
    fn test_log_entry() {
        let entry = LogEntry {
            level: LogLevel::Info,
            message: "Test message".to_string(),
            target: Some("test_module".to_string()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("Info"));
        assert!(json.contains("Test message"));
    }
}
