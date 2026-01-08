// Caret Stream - Stream processing primitives
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

#![warn(missing_docs)]
#![warn(clippy::all)]

mod stream;
mod sink;
mod ext;
mod combinators;
mod window;
mod merge;

pub use stream::{Stream, StreamItem, TryStream};
pub use sink::{Sink, SinkError};
pub use ext::StreamExt;
pub use combinators::{
    Map, Filter, FilterMap, Fold, Scan, FlatMap, Then, Inspect,
    Chain, Take, TakeWhile, Skip, SkipWhile, Fuse, Zip,
};
pub use window::{Window, TumblingWindow, SlidingWindow, CountWindow, TimeWindow};
pub use merge::{Merge, Select};

use std::time::Duration;

/// Default buffer size for stream operations
pub const DEFAULT_BUFFER_SIZE: usize = 32;

/// Default window size
pub const DEFAULT_WINDOW_SIZE: usize = 100;

/// Default window duration
pub const DEFAULT_WINDOW_DURATION: Duration = Duration::from_secs(1);

/// Stream processing configuration
#[derive(Clone, Debug)]
pub struct StreamConfig {
    /// Buffer size for intermediate operations
    pub buffer_size: usize,
    /// Whether to enable backpressure
    pub backpressure: bool,
    /// Maximum pending async operations
    pub max_pending: usize,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: DEFAULT_BUFFER_SIZE,
            backpressure: true,
            max_pending: 100,
        }
    }
}

impl StreamConfig {
    /// Create a new stream config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Enable or disable backpressure
    pub fn with_backpressure(mut self, enabled: bool) -> Self {
        self.backpressure = enabled;
        self
    }

    /// Set the maximum pending operations
    pub fn with_max_pending(mut self, max: usize) -> Self {
        self.max_pending = max;
        self
    }
}
