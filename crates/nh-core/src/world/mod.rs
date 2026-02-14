//! World state
//!
//! Contains global game flags, context, and world state.

mod context;
pub mod errors;
mod flags;
pub mod options;
pub mod paths;
pub mod save;
pub mod time;
pub mod timeout;
pub mod topten;
pub mod version;
pub mod wizkit;

pub use context::Context;
pub use errors::{FileError, validate_file_path, validate_prefix_locations};
pub use flags::Flags;
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
