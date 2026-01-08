// Caret Core - Core types and error model for Caret runtime
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod metadata;
mod packet;
mod time;

pub use error::{Error, ErrorKind, Result};
pub use metadata::{Metadata, MetadataValue};
pub use packet::{
    Packet, PacketKind,
    StreamId,
    AudioFormat, PixelFormat, TensorDtype,
    EventKind, ControlKind,
};
pub use time::{Duration, TimeBase, Timestamp};

/// Core version of Caret
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Core semver of Caret for API compatibility checks
pub const SEMVER: (u64, u64, u64) = (0, 1, 0);
