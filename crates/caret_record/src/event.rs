// Caret Record - Event types
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::SessionId;
use caret_core::Packet;
use serde::{Deserialize, Serialize};

/// Timestamp in nanoseconds since recording started
pub type Timestamp = u64;

/// Tick identifier
pub type TickId = u64;

/// Node identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    /// Create a new node ID
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Get the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Port identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortId {
    /// Node this port belongs to
    pub node: NodeId,
    /// Port name
    pub port: String,
}

impl PortId {
    /// Create a new port ID
    pub fn new(node: NodeId, port: String) -> Self {
        Self { node, port }
    }
}

/// Event type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// Packet sent through a port
    Packet,
    /// Node state changed
    StateChange,
    /// Metric was recorded
    Metric,
    /// An error occurred
    Error,
    /// Recording marker (for synchronization)
    Marker,
}

/// A recorded event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Session this event belongs to
    pub session_id: u64,
    /// Type of event
    #[serde(rename = "type")]
    pub event_type: EventType,
    /// Timestamp within the recording (nanoseconds)
    pub timestamp: Timestamp,
    /// Tick when this event occurred
    pub tick: TickId,
    /// Event-specific data
    pub data: EventData,
}

impl Event {
    /// Create a new event
    pub fn new(session_id: SessionId, event_type: EventType, tick: TickId, data: EventData) -> Self {
        Self {
            session_id: session_id.as_u64(),
            event_type,
            timestamp: 0, // Will be set by recorder
            tick,
            data,
        }
    }

    /// Create a packet event
    pub fn packet(session_id: SessionId, tick: TickId, packet: PacketEvent) -> Self {
        Self::new(session_id, EventType::Packet, tick, EventData::Packet(packet))
    }

    /// Create a state change event
    pub fn state_change(session_id: SessionId, tick: TickId, state: StateChangeEvent) -> Self {
        Self::new(session_id, EventType::StateChange, tick, EventData::StateChange(state))
    }

    /// Create a metric event
    pub fn metric(session_id: SessionId, tick: TickId, metric: MetricEvent) -> Self {
        Self::new(session_id, EventType::Metric, tick, EventData::Metric(metric))
    }

    /// Create an error event
    pub fn error(session_id: SessionId, tick: TickId, error: ErrorEvent) -> Self {
        Self::new(session_id, EventType::Error, tick, EventData::Error(error))
    }
}

/// Event-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventData {
    /// Packet flow event
    Packet(PacketEvent),
    /// State change event
    StateChange(StateChangeEvent),
    /// Metric event
    Metric(MetricEvent),
    /// Error event
    Error(ErrorEvent),
    /// Marker event
    Marker(String),
}

/// Packet flow event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketEvent {
    /// Source port (where packet came from)
    pub source: Option<PortId>,
    /// Destination port (where packet went)
    pub destination: PortId,
    /// The packet data (serialized)
    pub packet: Vec<u8>,
    /// Packet kind
    pub kind: String,
}

impl PacketEvent {
    /// Create a new packet event
    pub fn new(source: Option<PortId>, destination: PortId, packet: &Packet) -> Result<Self, crate::Error> {
        use caret_codec::Codec;

        // Serialize the packet
        let codec = caret_codec::JsonCodec::new();
        let packet_bytes = codec.encode(packet)
            .map_err(|e| crate::Error::CodecError(e.to_string()))?
            .to_vec();

        Ok(Self {
            source,
            destination,
            packet: packet_bytes,
            kind: format!("{:?}", packet.kind()),
        })
    }

    /// Deserialize the packet
    pub fn deserialize_packet(&self) -> Result<Packet, crate::Error> {
        use caret_codec::Codec;

        let codec = caret_codec::JsonCodec::new();
        codec.decode(&self.packet)
            .map_err(|e| crate::Error::CodecError(e.to_string()))
    }
}

/// Node state change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangeEvent {
    /// Node that changed state
    pub node: NodeId,
    /// Previous state
    pub from: String,
    /// New state
    pub to: String,
}

impl StateChangeEvent {
    /// Create a new state change event
    pub fn new(node: NodeId, from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            node,
            from: from.into(),
            to: to.into(),
        }
    }
}

/// Metric event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricEvent {
    /// Node that recorded the metric
    pub node: NodeId,
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: MetricValue,
    /// Metric unit (optional)
    pub unit: Option<String>,
}

impl MetricEvent {
    /// Create a new metric event
    pub fn new(node: NodeId, name: String, value: MetricValue) -> Self {
        Self {
            node,
            name,
            value,
            unit: None,
        }
    }

    /// Set the unit
    pub fn with_unit(mut self, unit: String) -> Self {
        self.unit = Some(unit);
        self
    }
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Counter value
    Counter(u64),
    /// Gauge value
    Gauge(f64),
    /// Histogram data (simplified)
    Histogram {
        /// Count of values
        count: u64,
        /// Sum of values
        sum: f64,
        /// Minimum value
        min: f64,
        /// Maximum value
        max: f64,
    },
}

/// Error event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// Node where error occurred
    pub node: NodeId,
    /// Error message
    pub message: String,
    /// Error severity
    pub severity: ErrorSeverity,
}

impl ErrorEvent {
    /// Create a new error event
    pub fn new(node: NodeId, message: String, severity: ErrorSeverity) -> Self {
        Self {
            node,
            message,
            severity,
        }
    }
}

/// Error severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Warning - non-fatal
    Warning,
    /// Error - may affect operation
    Error,
    /// Fatal - operation cannot continue
    Fatal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        let id = NodeId::new("node1".to_string());
        assert_eq!(id.as_str(), "node1");
    }

    #[test]
    fn test_port_id() {
        let node = NodeId::new("node1".to_string());
        let port = PortId::new(node.clone(), "output".to_string());
        assert_eq!(port.node.as_str(), "node1");
        assert_eq!(port.port, "output");
    }

    #[test]
    fn test_state_change_event() {
        let node = NodeId::new("node1".to_string());
        let event = StateChangeEvent::new(node.clone(), "Idle", "Running");
        assert_eq!(event.node.as_str(), "node1");
        assert_eq!(event.from, "Idle");
        assert_eq!(event.to, "Running");
    }

    #[test]
    fn test_metric_event() {
        let node = NodeId::new("node1".to_string());
        let event = MetricEvent::new(node.clone(), "packets_processed".to_string(), MetricValue::Counter(42));
        assert_eq!(event.node.as_str(), "node1");
        assert_eq!(event.name, "packets_processed");
    }

    #[test]
    fn test_event_creation() {
        let session_id = SessionId::new();
        let node = NodeId::new("node1".to_string());
        let state_event = StateChangeEvent::new(node, "Idle", "Running");

        let event = Event::state_change(session_id, 10, state_event);
        assert_eq!(event.tick, 10);
        assert_eq!(event.event_type, EventType::StateChange);
    }
}
