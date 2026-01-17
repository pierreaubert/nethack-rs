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

/// Dungeon branch identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum DungeonId {
    MainDungeon = 0,
    Gehennom = 1,
    Mines = 2,
    Sokoban = 3,
    Quest = 4,
    FortLudios = 5,
    VladsTower = 6,
    EndGame = 7,
}

impl DungeonId {
    /// Get dungeon definition for this ID
    pub fn definition(&self) -> Dungeon {
        match self {
            Self::MainDungeon => Dungeon::main_dungeon(),
            Self::Gehennom => Dungeon::gehennom(),
            Self::Mines => Dungeon::mines(),
            Self::Sokoban => Dungeon::sokoban(),
            Self::Quest => Dungeon::quest(),
            Self::FortLudios => Dungeon::fort_ludios(),
            Self::VladsTower => Dungeon::vlads_tower(),
            Self::EndGame => Dungeon::endgame(),
        }
    }

    /// Get the dungeon number
    pub fn number(&self) -> i8 {
        *self as i8
    }

    /// Get dungeon from number
    pub fn from_number(num: i8) -> Option<Self> {
        match num {
            0 => Some(Self::MainDungeon),
            1 => Some(Self::Gehennom),
            2 => Some(Self::Mines),
            3 => Some(Self::Sokoban),
            4 => Some(Self::Quest),
            5 => Some(Self::FortLudios),
            6 => Some(Self::VladsTower),
            7 => Some(Self::EndGame),
            _ => None,
        }
    }
}

impl Dungeon {
    /// Create Quest dungeon
    pub fn quest() -> Self {
        Self {
            name: "The Quest".to_string(),
            prototype: "quest".to_string(),
            bones_char: 'Q',
            flags: DungeonFlags::default(),
            entry_level: 1,
            num_levels: 5,
            deepest_reached: 0,
            ledger_start: 70,
            depth_start: 16,
        }
    }

    /// Create Fort Ludios
    pub fn fort_ludios() -> Self {
        Self {
            name: "Fort Ludios".to_string(),
            prototype: "knox".to_string(),
            bones_char: 'K',
            flags: DungeonFlags::default(),
            entry_level: 1,
            num_levels: 1,
            deepest_reached: 0,
            ledger_start: 80,
            depth_start: 18,
        }
    }

    /// Create Vlad's Tower
    pub fn vlads_tower() -> Self {
        Self {
            name: "Vlad's Tower".to_string(),
            prototype: "tower".to_string(),
            bones_char: 'T',
            flags: DungeonFlags::default(),
            entry_level: 1,
            num_levels: 3,
            deepest_reached: 0,
            ledger_start: 85,
            depth_start: 37,
        }
    }

    /// Create Endgame (Elemental Planes + Astral)
    pub fn endgame() -> Self {
        Self {
            name: "The Planes".to_string(),
            prototype: "endgame".to_string(),
            bones_char: 'E',
            flags: DungeonFlags {
                maze_like: true,
                ..Default::default()
            },
            entry_level: 1,
            num_levels: 5,
            deepest_reached: 0,
            ledger_start: 90,
            depth_start: 50,
        }
    }
}

/// Complete dungeon system managing all branches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DungeonSystem {
    /// All dungeon definitions
    pub dungeons: Vec<Dungeon>,

    /// Branch connections between dungeons
    pub branches: Vec<Branch>,
}

impl Default for DungeonSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl DungeonSystem {
    /// Create a new dungeon system with all branches
    pub fn new() -> Self {
        let dungeons = vec![
            Dungeon::main_dungeon(),
            Dungeon::gehennom(),
            Dungeon::mines(),
            Dungeon::sokoban(),
            Dungeon::quest(),
            Dungeon::fort_ludios(),
            Dungeon::vlads_tower(),
            Dungeon::endgame(),
        ];

        // Define branch connections
        let branches = vec![
            // Mines entrance: Main dungeon level 2-4 -> Mines level 1
            Branch::stairs(0, DLevel::new(0, 3), DLevel::new(2, 1), false),
            // Sokoban entrance: Main dungeon level 6-9 -> Sokoban level 1
            Branch::stairs(1, DLevel::new(0, 7), DLevel::new(3, 1), true),
            // Gehennom entrance: Main dungeon level 25 (Castle) -> Gehennom level 1
            Branch::stairs(2, DLevel::new(0, 25), DLevel::new(1, 1), false),
            // Quest entrance: Main dungeon level 14 -> Quest level 1
            Branch::portal(3, DLevel::new(0, 14), DLevel::new(4, 1)),
            // Fort Ludios: Main dungeon (random) -> Fort Ludios
            Branch::portal(4, DLevel::new(0, 12), DLevel::new(5, 1)),
            // Vlad's Tower: Gehennom level 10 -> Vlad's Tower level 1
            Branch::stairs(5, DLevel::new(1, 10), DLevel::new(6, 1), true),
            // Endgame: Sanctum -> Earth Plane
            Branch::portal(6, DLevel::new(1, 20), DLevel::new(7, 1)),
        ];

        Self { dungeons, branches }
    }

    /// Get dungeon by ID
    pub fn get_dungeon(&self, id: DungeonId) -> Option<&Dungeon> {
        self.dungeons.get(id.number() as usize)
    }

    /// Get branch entrance from a level
    pub fn get_branch_from(&self, dlevel: &DLevel) -> Option<&Branch> {
        self.branches.iter().find(|b| b.end1 == *dlevel)
    }

    /// Get branch entrance to a level
    pub fn get_branch_to(&self, dlevel: &DLevel) -> Option<&Branch> {
        self.branches.iter().find(|b| b.end2 == *dlevel)
    }

    /// Check if a level has a branch entrance
    pub fn has_branch_entrance(&self, dlevel: &DLevel) -> bool {
        self.get_branch_from(dlevel).is_some()
    }

    /// Get the destination of a branch from this level
    pub fn branch_destination(&self, dlevel: &DLevel) -> Option<DLevel> {
        self.get_branch_from(dlevel).map(|b| b.end2)
    }

    /// Check if level is in a maze-like dungeon
    pub fn is_maze_dungeon(&self, dlevel: &DLevel) -> bool {
        if let Some(dungeon) = self.dungeons.get(dlevel.dungeon_num as usize) {
            dungeon.flags.maze_like || dungeon.flags.hellish
        } else {
            false
        }
    }

    /// Get dungeon name for a level
    pub fn dungeon_name(&self, dlevel: &DLevel) -> &str {
        self.dungeons
            .get(dlevel.dungeon_num as usize)
            .map(|d| d.name.as_str())
            .unwrap_or("Unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dungeon_system_creation() {
        let system = DungeonSystem::new();

        assert_eq!(system.dungeons.len(), 8);
        assert_eq!(system.branches.len(), 7);
    }

    #[test]
    fn test_branch_lookup() {
        let system = DungeonSystem::new();

        // Mines entrance at main dungeon level 3
        let branch = system.get_branch_from(&DLevel::new(0, 3));
        assert!(branch.is_some());
        assert_eq!(branch.unwrap().end2, DLevel::new(2, 1));

        // No branch at main dungeon level 1
        assert!(system.get_branch_from(&DLevel::new(0, 1)).is_none());
    }

    #[test]
    fn test_dungeon_id() {
        assert_eq!(DungeonId::MainDungeon.number(), 0);
        assert_eq!(DungeonId::Gehennom.number(), 1);
        assert_eq!(DungeonId::Mines.number(), 2);

        assert_eq!(DungeonId::from_number(0), Some(DungeonId::MainDungeon));
        assert_eq!(DungeonId::from_number(99), None);
    }

    #[test]
    fn test_maze_dungeon_check() {
        let system = DungeonSystem::new();

        // Gehennom is hellish (maze-like)
        assert!(system.is_maze_dungeon(&DLevel::new(1, 5)));

        // Main dungeon is not maze-like
        assert!(!system.is_maze_dungeon(&DLevel::new(0, 5)));

        // Endgame is maze-like
        assert!(system.is_maze_dungeon(&DLevel::new(7, 1)));
    }

    #[test]
    fn test_dungeon_names() {
        let system = DungeonSystem::new();

        assert_eq!(system.dungeon_name(&DLevel::new(0, 1)), "The Dungeons of Doom");
        assert_eq!(system.dungeon_name(&DLevel::new(1, 1)), "Gehennom");
        assert_eq!(system.dungeon_name(&DLevel::new(2, 1)), "The Gnomish Mines");
        assert_eq!(system.dungeon_name(&DLevel::new(3, 1)), "Sokoban");
    }
}
