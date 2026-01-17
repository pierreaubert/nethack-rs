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
    ///
    /// Depth is calculated based on dungeon branch:
    /// - Main dungeon (0): depth = level_num
    /// - Gehennom (1): depth = 30 + level_num - 1
    /// - Gnomish Mines (2): depth = 2 + level_num - 1
    /// - Sokoban (3): depth = 6 + level_num - 1
    /// - Quest (4): depth = 16 + level_num - 1
    /// - Fort Ludios (5): depth = 18 + level_num - 1
    /// - Vlad's Tower (6): depth = 37 + level_num - 1
    pub const fn depth(&self) -> i32 {
        let depth_start = match self.dungeon_num {
            0 => 1,   // Main dungeon
            1 => 30,  // Gehennom
            2 => 2,   // Gnomish Mines
            3 => 6,   // Sokoban
            4 => 16,  // Quest
            5 => 18,  // Fort Ludios
            6 => 37,  // Vlad's Tower
            _ => 1,   // Default
        };
        depth_start + (self.level_num as i32) - 1
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_dungeon_depth() {
        // Main dungeon: depth = level_num
        assert_eq!(DLevel::new(0, 1).depth(), 1);
        assert_eq!(DLevel::new(0, 10).depth(), 10);
        assert_eq!(DLevel::new(0, 29).depth(), 29);
    }

    #[test]
    fn test_gehennom_depth() {
        // Gehennom: depth starts at 30
        assert_eq!(DLevel::new(1, 1).depth(), 30);
        assert_eq!(DLevel::new(1, 10).depth(), 39);
    }

    #[test]
    fn test_mines_depth() {
        // Gnomish Mines: depth starts at 2
        assert_eq!(DLevel::new(2, 1).depth(), 2);
        assert_eq!(DLevel::new(2, 8).depth(), 9);
    }

    #[test]
    fn test_sokoban_depth() {
        // Sokoban: depth starts at 6
        assert_eq!(DLevel::new(3, 1).depth(), 6);
        assert_eq!(DLevel::new(3, 4).depth(), 9);
    }

    #[test]
    fn test_is_deeper() {
        let shallow = DLevel::new(0, 5);
        let deep = DLevel::new(0, 15);
        let gehennom = DLevel::new(1, 1);

        assert!(deep.is_deeper(&shallow));
        assert!(!shallow.is_deeper(&deep));
        assert!(gehennom.is_deeper(&deep)); // Gehennom level 1 is depth 30
    }
}
