//! Dungeon level identifier

use serde::{Deserialize, Serialize};

/// Dungeon level identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct DLevel {
    /// Dungeon number (which branch)
    pub dungeon_num: i8,
    /// Level number within the dungeon
    pub level_num: i8,
}

impl DLevel {
    /// Create a new dungeon level identifier
    pub const fn new(dungeon_num: i8, level_num: i8) -> Self {
        Self {
            dungeon_num,
            level_num,
        }
    }

    /// Main dungeon entrance
    pub const fn main_dungeon_start() -> Self {
        Self::new(0, 1)
    }

    /// Check if this is the main dungeon
    pub const fn is_main_dungeon(&self) -> bool {
        self.dungeon_num == 0
    }

    /// Get depth (for difficulty calculations)
    pub const fn depth(&self) -> i32 {
        // TODO: Calculate actual depth based on dungeon topology
        self.level_num as i32
    }

    /// Check if deeper than another level
    pub fn is_deeper(&self, other: &DLevel) -> bool {
        self.depth() > other.depth()
    }
}

impl std::fmt::Display for DLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dlvl:{}", self.level_num)
    }
}
