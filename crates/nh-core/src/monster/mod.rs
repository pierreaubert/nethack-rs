//! Monster system
//!
//! Contains monster templates (permonst) and instances (monst).

pub mod ai;
mod monst;
mod permonst;
pub mod tactics;

pub use ai::{process_monster_ai, AiAction};
pub use monst::{Monster, MonsterId, MonsterState, SpeedState};
pub use permonst::{MonsterFlags, MonsterResistances, MonsterSize, MonsterSound, PerMonst};
pub use tactics::{TacticalAction, SpecialAbility, Intelligence, determine_tactics};

/// Reference to a monster instance
pub type MonsterRef = MonsterId;
