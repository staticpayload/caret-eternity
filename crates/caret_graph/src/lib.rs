// Caret Graph - Graph representation and topology for Caret
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

#![warn(missing_docs)]
#![warn(clippy::all)]

mod dynamic;
mod graph;
mod node;
mod port;
mod topology;

pub use dynamic::{
    ChangeListener, ChangeResult, DynamicGraph, GraphChange, GraphTransaction, NopListener,
};
pub use graph::Graph;
pub use node::{Node, NodeHandle, NodeId, NodeType};
pub use port::{Port, PortDirection, PortId};
pub use topology::{ConnectionError, Edge, Topology};
