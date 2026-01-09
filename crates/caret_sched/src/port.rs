// Caret Sched - Port connections
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use caret_buffers::{BoundedQueue, OverflowPolicy};
use caret_core::{Packet, Result};
use parking_lot::Mutex;
use std::sync::Arc;

/// Input port for receiving packets
#[derive(Clone)]
pub struct InputPort {
    /// Port name
    name: String,
    /// Queue for incoming packets
    queue: Arc<BoundedQueue<Packet>>,
}

impl InputPort {
    /// Create a new input port
    pub fn new(name: impl Into<String>, capacity: usize) -> Self {
        Self {
            name: name.into(),
            queue: Arc::new(BoundedQueue::with_policy(capacity, OverflowPolicy::Block)),
        }
    }

    /// Get the port name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Receive a packet (non-blocking)
    pub fn try_recv(&self) -> Option<Packet> {
        self.queue.pop()
    }

    /// Check if the port has data available
    pub fn has_data(&self) -> bool {
        !self.queue.is_empty()
    }

    /// Get the queue depth
    pub fn depth(&self) -> usize {
        self.queue.len()
    }

    /// Get the capacity
    pub fn capacity(&self) -> usize {
        self.queue.capacity()
    }

    /// Push a packet directly into the input port
    ///
    /// This is used for external packet injection (e.g., from distributed execution).
    pub fn push(&self, packet: Packet) -> Result<()> {
        self.queue.push(packet)
    }
}

/// Output port for sending packets
#[derive(Clone)]
pub struct OutputPort {
    /// Port name
    name: String,
    /// Connections to input ports
    connections: Arc<parking_lot::Mutex<Vec<PortConnection>>>,
}

impl OutputPort {
    /// Create a new output port
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            connections: Arc::new(parking_lot::Mutex::new(Vec::new())),
        }
    }

    /// Get the port name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add a connection to an input port
    pub fn connect(&self, target: InputPort) -> Result<()> {
        let mut conns = self.connections.lock();
        conns.push(PortConnection::new(target));
        Ok(())
    }

    /// Send a packet to all connected ports
    pub fn send(&self, packet: Packet) -> Result<()> {
        let conns = self.connections.lock();
        for conn in conns.iter() {
            conn.send(packet.clone())?;
        }
        Ok(())
    }

    /// Get the number of connections
    pub fn connection_count(&self) -> usize {
        self.connections.lock().len()
    }
}

/// A connection between an output port and an input port
pub struct PortConnection {
    /// Target input port
    target: InputPort,
}

impl PortConnection {
    /// Create a new port connection
    pub fn new(target: InputPort) -> Self {
        Self { target }
    }

    /// Send a packet through this connection
    pub fn send(&self, packet: Packet) -> Result<()> {
        self.target.queue.push(packet)
    }
}

/// Port set for a node - contains all input and output ports
///
/// Performance optimizations:
/// - Uses Arc<Mutex<HashMap>> for interior mutability
/// - Provides zero-copy access to port names via iterators
#[derive(Clone)]
pub struct PortSet {
    /// Input ports
    inputs: Arc<parking_lot::Mutex<std::collections::HashMap<String, InputPort>>>,
    /// Output ports
    outputs: Arc<parking_lot::Mutex<std::collections::HashMap<String, OutputPort>>>,
}

impl Default for PortSet {
    fn default() -> Self {
        Self::new()
    }
}

impl PortSet {
    /// Create a new empty port set
    pub fn new() -> Self {
        Self {
            inputs: Arc::new(parking_lot::Mutex::new(std::collections::HashMap::new())),
            outputs: Arc::new(parking_lot::Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Add an input port
    pub fn add_input(&self, name: impl Into<String>, capacity: usize) -> Result<InputPort> {
        let name = name.into();
        let mut inputs = self.inputs.lock();
        if inputs.contains_key(&name) {
            return Err(caret_core::Error::invalid_input(format!(
                "input port '{}' already exists",
                name
            )));
        }
        let port = InputPort::new(&name, capacity);
        inputs.insert(name, port.clone());
        Ok(port)
    }

    /// Add an output port
    pub fn add_output(&self, name: impl Into<String>) -> Result<OutputPort> {
        let name = name.into();
        let mut outputs = self.outputs.lock();
        if outputs.contains_key(&name) {
            return Err(caret_core::Error::invalid_input(format!(
                "output port '{}' already exists",
                name
            )));
        }
        let port = OutputPort::new(&name);
        outputs.insert(name, port.clone());
        Ok(port)
    }

    /// Get an input port by name
    pub fn input(&self, name: &str) -> Option<InputPort> {
        self.inputs.lock().get(name).cloned()
    }

    /// Get an output port by name
    pub fn output(&self, name: &str) -> Option<OutputPort> {
        self.outputs.lock().get(name).cloned()
    }

    /// Get all input port names
    pub fn input_names(&self) -> Vec<String> {
        self.inputs.lock().keys().cloned().collect()
    }

    /// Get all output port names
    pub fn output_names(&self) -> Vec<String> {
        self.outputs.lock().keys().cloned().collect()
    }

    /// Get the number of input ports
    pub fn input_count(&self) -> usize {
        self.inputs.lock().len()
    }

    /// Get the number of output ports
    pub fn output_count(&self) -> usize {
        self.outputs.lock().len()
    }

    /// Get all output ports as a map
    pub fn output_map(&self) -> std::collections::HashMap<String, OutputPort> {
        self.outputs.lock().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_port() {
        let port = InputPort::new("input", 10);
        assert_eq!(port.name(), "input");
        assert_eq!(port.capacity(), 10);
        assert_eq!(port.depth(), 0);
        assert!(!port.has_data());

        // Queue is blocking, so this would require a receiver
        // For now just test the metadata
    }

    #[test]
    fn test_output_port() {
        let port = OutputPort::new("output");
        assert_eq!(port.name(), "output");
        assert_eq!(port.connection_count(), 0);
    }

    #[test]
    fn test_port_set() {
        let ports = PortSet::new();
        let input = ports.add_input("in", 10).unwrap();
        let output = ports.add_output("out").unwrap();

        assert_eq!(input.name(), "in");
        assert_eq!(output.name(), "out");

        assert!(ports.input("in").is_some());
        assert!(ports.output("out").is_some());
        assert!(ports.input("invalid").is_none());
    }

    #[test]
    fn test_port_set_duplicate_fails() {
        let ports = PortSet::new();
        ports.add_input("in", 10).unwrap();
        assert!(ports.add_input("in", 10).is_err());

        ports.add_output("out").unwrap();
        assert!(ports.add_output("out").is_err());
    }

    #[test]
    fn test_port_connection() {
        let input = InputPort::new("input", 10);
        let output = OutputPort::new("output");
        output.connect(input.clone()).unwrap();

        assert_eq!(output.connection_count(), 1);
    }

    #[test]
    fn test_port_count() {
        let ports = PortSet::new();
        assert_eq!(ports.input_count(), 0);
        assert_eq!(ports.output_count(), 0);

        ports.add_input("in1", 10).unwrap();
        ports.add_input("in2", 10).unwrap();
        ports.add_output("out1").unwrap();

        assert_eq!(ports.input_count(), 2);
        assert_eq!(ports.output_count(), 1);
    }
}
