// Caret IO - IO nodes and codecs for Caret
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

#![warn(missing_docs)]
#![warn(clippy::all)]

mod file;
mod memory;
mod nodes;

pub use file::{FileSource, FileSink, FileSourceConfig, FileSinkConfig};
pub use memory::{MemorySource, MemorySink, MemorySourceConfig, MemorySinkConfig};
pub use nodes::{
    SourceNode, SinkNode, ProcessNode,
    SourceNodeAdapter, SinkNodeAdapter, ProcessNodeAdapter,
};
