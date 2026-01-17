//! Monster system
//!
//! Contains monster templates (permonst) and instances (monst).

pub mod ai;
mod monst;
mod permonst;

pub use ai::{process_monster_ai, AiAction};
pub use monst::{Monster, MonsterId, MonsterState};
pub use permonst::{MonsterFlags, MonsterResistances, MonsterSize, MonsterSound, PerMonst};

/// Reference to a monster instance
pub type MonsterRef = MonsterId;
