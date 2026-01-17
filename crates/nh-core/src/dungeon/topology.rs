//! Dungeon topology (dungeon.h)

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use super::DLevel;

/// Dungeon flags
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DungeonFlags {
    pub town: bool,
    pub hellish: bool,
    pub maze_like: bool,
    pub rogue_like: bool,
    pub alignment: i8, // -1 chaotic, 0 neutral, 1 lawful
}

/// Dungeon definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dungeon {
    /// Dungeon name (e.g., "The Dungeons of Doom")
    pub name: String,

    /// Prototype file name
    pub prototype: String,

    /// Character for bones files
    pub bones_char: char,

    /// Dungeon flags
    pub flags: DungeonFlags,

    /// Entry level
    pub entry_level: i8,

    /// Number of levels
    pub num_levels: i8,

    /// Deepest level reached by player
    pub deepest_reached: i8,

    /// Ledger start (for level numbering)
    pub ledger_start: i32,

    /// Depth start (for difficulty)
    pub depth_start: i32,
}

impl Dungeon {
    /// Create the main dungeon
    pub fn main_dungeon() -> Self {
        Self {
            name: "The Dungeons of Doom".to_string(),
            prototype: "dungeon".to_string(),
            bones_char: 'D',
            flags: DungeonFlags::default(),
            entry_level: 1,
            num_levels: 29,
            deepest_reached: 0,
            ledger_start: 0,
            depth_start: 1,
        }
    }

    /// Create Gehennom (Hell)
    pub fn gehennom() -> Self {
        Self {
            name: "Gehennom".to_string(),
            prototype: "gehennom".to_string(),
            bones_char: 'G',
            flags: DungeonFlags {
                hellish: true,
                ..Default::default()
            },
            entry_level: 1,
            num_levels: 20,
            deepest_reached: 0,
            ledger_start: 29,
            depth_start: 30,
        }
    }

    /// Create the Gnomish Mines
    pub fn mines() -> Self {
        Self {
            name: "The Gnomish Mines".to_string(),
            prototype: "mines".to_string(),
            bones_char: 'M',
            flags: DungeonFlags::default(),
            entry_level: 2,
            num_levels: 8,
            deepest_reached: 0,
            ledger_start: 50,
            depth_start: 2,
        }
    }

    /// Create Sokoban
    pub fn sokoban() -> Self {
        Self {
            name: "Sokoban".to_string(),
            prototype: "sokoban".to_string(),
            bones_char: 'S',
            flags: DungeonFlags::default(),
            entry_level: 4,
            num_levels: 4,
            deepest_reached: 0,
            ledger_start: 60,
            depth_start: 6,
        }
    }

    /// Check if a level is in this dungeon
    pub fn contains_level(&self, level: i8) -> bool {
        level >= 1 && level <= self.num_levels
    }
}

/// Branch connection types
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum BranchType {
    #[default]
    Stairs = 0,
    NoEnd1 = 1,  // No connection at end 1
    NoEnd2 = 2,  // No connection at end 2
    Portal = 3,  // Magic portal
}

/// Branch between dungeons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// Branch identifier
    pub id: i32,

    /// Branch type
    pub branch_type: BranchType,

    /// First endpoint
    pub end1: DLevel,

    /// Second endpoint
    pub end2: DLevel,

    /// Is end1 going up?
    pub end1_up: bool,
}

impl Branch {
    /// Create a stairs branch
    pub fn stairs(id: i32, from: DLevel, to: DLevel, going_up: bool) -> Self {
        Self {
            id,
            branch_type: BranchType::Stairs,
            end1: from,
            end2: to,
            end1_up: going_up,
        }
    }

    /// Create a portal branch
    pub fn portal(id: i32, from: DLevel, to: DLevel) -> Self {
        Self {
            id,
            branch_type: BranchType::Portal,
            end1: from,
            end2: to,
            end1_up: false,
        }
    }
}
