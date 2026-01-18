//! World state
//!
//! Contains global game flags, context, and world state.

mod context;
mod flags;
mod timeout;
pub mod options;
pub mod save;
pub mod topten;

pub use context::Context;
pub use flags::Flags;
pub use timeout::{TimedEvent, TimedEventType, TimeoutManager};

/// Game turn information
#[derive(Debug, Clone, Default)]
pub struct TurnInfo {
    /// Total turns elapsed
    pub turns: u64,
    /// Monster turn counter (can differ from player turns)
    pub monster_turns: u64,
}
