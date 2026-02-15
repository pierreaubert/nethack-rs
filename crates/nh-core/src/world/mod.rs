//! World state
//!
//! Contains global game flags, context, and world state.

mod context;
#[cfg(feature = "std")]
pub mod errors;
mod flags;
pub mod options;
#[cfg(feature = "std")]
pub mod paths;
#[cfg(feature = "std")]
pub mod save;
#[cfg(feature = "std")]
pub mod time;
pub mod timeout;
#[cfg(feature = "std")]
pub mod topten;
pub mod version;
#[cfg(feature = "std")]
pub mod wizkit;

pub use context::Context;
#[cfg(feature = "std")]
pub use errors::{FileError, validate_file_path, validate_prefix_locations};
pub use flags::Flags;
#[cfg(feature = "std")]
pub use paths::ParsedArgs;
pub use timeout::{TimedEvent, TimedEventType, TimeoutManager};

/// Game turn information
#[derive(Debug, Clone, Default)]
pub struct TurnInfo {
    /// Total turns elapsed
    pub turns: u64,
    /// Monster turn counter (can differ from player turns)
    pub monster_turns: u64,
}
