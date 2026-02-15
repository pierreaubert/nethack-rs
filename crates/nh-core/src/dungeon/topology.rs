//! Dungeon topology (dungeon.h)

#[cfg(not(feature = "std"))]
use crate::compat::*;

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
    NoEnd1 = 1, // No connection at end 1
    NoEnd2 = 2, // No connection at end 2
    Portal = 3, // Magic portal
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

// ============================================================================
// Free functions (C-style API equivalents)
// ============================================================================

/// Initialize dungeon system (init_dungeons equivalent)
///
/// Creates and returns a new dungeon system with all default dungeons and branches.
pub fn init_dungeons() -> DungeonSystem {
    DungeonSystem::new()
}

/// Check if a dungeon is the main dungeon (is_main_dungeon equivalent)
///
/// Returns true if the given dungeon number represents the main dungeon (0).
pub const fn is_main_dungeon(dnum: i8) -> bool {
    dnum == 0
}

/// Check if a level is in the main dungeon
pub fn is_in_main_dungeon(dlevel: &DLevel) -> bool {
    dlevel.is_main_dungeon()
}

/// Get branch type as a string (br_string equivalent)
///
/// Returns a human-readable description of the branch type.
pub const fn br_string(branch_type: BranchType) -> &'static str {
    match branch_type {
        BranchType::Stairs => "stairs",
        BranchType::NoEnd1 => "no-connection-1",
        BranchType::NoEnd2 => "no-connection-2",
        BranchType::Portal => "portal",
    }
}

/// Get branch description (br_string2 equivalent)
///
/// Returns a more detailed description including direction.
pub fn br_string2(branch: &Branch) -> String {
    let direction = if branch.end1_up { "up" } else { "down" };
    match branch.branch_type {
        BranchType::Stairs => format!("stairs ({})", direction),
        BranchType::NoEnd1 => "blocked at start".to_string(),
        BranchType::NoEnd2 => "blocked at end".to_string(),
        BranchType::Portal => "magic portal".to_string(),
    }
}

/// Check if a dungeon is considered hellish
pub fn is_hellish(system: &DungeonSystem, dnum: i8) -> bool {
    system
        .dungeons
        .get(dnum as usize)
        .map(|d| d.flags.hellish)
        .unwrap_or(false)
}

/// Check if a level is in a hellish dungeon
pub fn level_in_hellish_dungeon(system: &DungeonSystem, dlevel: &DLevel) -> bool {
    system.is_maze_dungeon(dlevel) && is_hellish(system, dlevel.dungeon_num)
}

/// Check if a dungeon is maze-like (maze_like equivalent)
pub fn is_maze_like(system: &DungeonSystem, dnum: i8) -> bool {
    system
        .dungeons
        .get(dnum as usize)
        .map(|d| d.flags.maze_like)
        .unwrap_or(false)
}

/// Check if a dungeon is a town
pub fn is_town(system: &DungeonSystem, dnum: i8) -> bool {
    system
        .dungeons
        .get(dnum as usize)
        .map(|d| d.flags.town)
        .unwrap_or(false)
}

/// Get the number of levels in a dungeon (numdungeons equivalent)
pub fn numdungeons(system: &DungeonSystem) -> usize {
    system.dungeons.len()
}

/// Get total branches in dungeon system
pub fn num_branches(system: &DungeonSystem) -> usize {
    system.branches.len()
}

/// Get branch by ID
pub fn get_branch_by_id(system: &DungeonSystem, id: i32) -> Option<&Branch> {
    system.branches.iter().find(|b| b.id == id)
}

/// Find if two levels are connected by a branch
pub fn levels_connected(system: &DungeonSystem, from: &DLevel, to: &DLevel) -> bool {
    system
        .branches
        .iter()
        .any(|b| (b.end1 == *from && b.end2 == *to) || (b.end1 == *to && b.end2 == *from))
}

/// Get portal destination from level (mkportal equivalent)
///
/// Finds the destination of a portal from the given level.
pub fn mkportal(system: &DungeonSystem, dlevel: &DLevel) -> Option<DLevel> {
    system
        .branches
        .iter()
        .find(|b| b.end1 == *dlevel && b.branch_type == BranchType::Portal)
        .map(|b| b.end2)
}

/// Get next portal destination (for chained portals)
pub fn next_portal(system: &DungeonSystem, dlevel: &DLevel) -> Option<DLevel> {
    mkportal(system, dlevel)
}

/// Get Knox portal destination (mk_knox_portal equivalent)
///
/// Gets the Fort Ludios (Knox) portal connection.
pub fn mk_knox_portal(system: &DungeonSystem) -> Option<Branch> {
    system
        .branches
        .iter()
        .find(|b| b.id == 4) // Fort Ludios portal is ID 4
        .cloned()
}

/// Get the parent level of a branch entrance
pub fn parent_level(system: &DungeonSystem, dlevel: &DLevel) -> Option<DLevel> {
    system
        .branches
        .iter()
        .find(|b| b.end2 == *dlevel)
        .map(|b| b.end1)
}

/// Check if a level has any branches from it
pub fn has_branch_from_level(system: &DungeonSystem, dlevel: &DLevel) -> bool {
    system.has_branch_entrance(dlevel)
}

/// Get the index of a dungeon by number
pub fn dungeon_index(dnum: i8) -> Option<usize> {
    if dnum >= 0 && dnum < 8 {
        Some(dnum as usize)
    } else {
        None
    }
}

/// Check alignment of a dungeon (-1 chaotic, 0 neutral, 1 lawful)
pub fn dungeon_alignment(system: &DungeonSystem, dnum: i8) -> i8 {
    system
        .dungeons
        .get(dnum as usize)
        .map(|d| d.flags.alignment)
        .unwrap_or(0)
}

// ============================================================================
// Level region types (from C's sp_lev.h)
// ============================================================================

/// Level region types for placing stairs, portals, and teleport destinations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LRegionType {
    /// Upstairs
    Upstair = 0,
    /// Downstairs
    Downstair = 1,
    /// Level teleport destination
    Tele = 2,
    /// Up level teleport
    UpTele = 3,
    /// Down level teleport
    DownTele = 4,
    /// Magic portal
    Portal = 5,
    /// Branch entrance
    Branch = 6,
}

/// Level region area for constraining placement
#[derive(Debug, Clone, Copy, Default)]
pub struct LRegionArea {
    pub x1: u8,
    pub y1: u8,
    pub x2: u8,
    pub y2: u8,
}

impl LRegionArea {
    pub fn new(x1: u8, y1: u8, x2: u8, y2: u8) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Check if a point is within this area
    pub fn contains(&self, x: u8, y: u8) -> bool {
        x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2
    }

    /// Check if this area is empty (all zeros)
    pub fn is_empty(&self) -> bool {
        self.x1 == 0 && self.y1 == 0 && self.x2 == 0 && self.y2 == 0
    }
}

/// Level region for special placements
#[derive(Debug, Clone, Copy)]
pub struct LRegion {
    /// Area to place within
    pub inarea: LRegionArea,
    /// Area to exclude
    pub delarea: LRegionArea,
    /// Region type
    pub rtype: LRegionType,
    /// Padding/flags
    pub padding: u8,
    /// Destination dungeon number
    pub rname_dnum: i8,
    /// Destination level number
    pub rname_dlevel: i8,
}

impl Default for LRegion {
    fn default() -> Self {
        Self {
            inarea: LRegionArea::default(),
            delarea: LRegionArea::default(),
            rtype: LRegionType::Tele,
            padding: 0,
            rname_dnum: 0,
            rname_dlevel: 0,
        }
    }
}

// ============================================================================
// Serialization functions (for save/restore)
// ============================================================================

/// Restore dungeon structures from serialized data (restore_dungeon equivalent)
///
/// Deserializes dungeon system from JSON format.
///
/// # Arguments
/// * `data` - Serialized dungeon data
///
/// # Returns
/// Restored DungeonSystem if successful
pub fn restore_dungeon(data: &str) -> Result<DungeonSystem, String> {
    serde_json::from_str(data).map_err(|e| format!("Failed to restore dungeon: {}", e))
}

/// Save dungeon structures to serialized data
///
/// Serializes dungeon system to JSON format.
///
/// # Arguments
/// * `system` - DungeonSystem to save
///
/// # Returns
/// Serialized string if successful
pub fn save_dungeon(system: &DungeonSystem) -> Result<String, String> {
    serde_json::to_string(system).map_err(|e| format!("Failed to save dungeon: {}", e))
}

// ============================================================================
// Dungeon display functions (for debugging)
// ============================================================================

/// Format dungeon information for display (print_dungeon equivalent)
///
/// Creates a formatted string representation of the dungeon topology.
///
/// # Arguments
/// * `system` - DungeonSystem to display
/// * `show_branches` - Whether to include branch information
///
/// # Returns
/// Formatted string with dungeon information
pub fn print_dungeon(system: &DungeonSystem, show_branches: bool) -> String {
    let mut output = String::new();

    output.push_str("=== Dungeon Topology ===\n\n");

    // Print each dungeon
    for (i, dungeon) in system.dungeons.iter().enumerate() {
        let depth_range = if dungeon.num_levels > 1 {
            format!(
                "{} to {}",
                dungeon.depth_start,
                dungeon.depth_start + dungeon.num_levels as i32 - 1
            )
        } else {
            format!("{}", dungeon.depth_start)
        };

        output.push_str(&format!("{}: levels {}", dungeon.name, depth_range));

        if dungeon.entry_level != 1 {
            if dungeon.entry_level == dungeon.num_levels {
                output.push_str(", entrance from below");
            } else {
                output.push_str(&format!(
                    ", entrance on level {}",
                    dungeon.depth_start + dungeon.entry_level as i32 - 1
                ));
            }
        }

        // Dungeon flags
        let mut flags = Vec::new();
        if dungeon.flags.hellish {
            flags.push("hellish");
        }
        if dungeon.flags.maze_like {
            flags.push("maze-like");
        }
        if dungeon.flags.rogue_like {
            flags.push("rogue-like");
        }
        if dungeon.flags.town {
            flags.push("town");
        }

        if !flags.is_empty() {
            output.push_str(&format!(" [{}]", flags.join(", ")));
        }

        output.push('\n');

        // Show branches from this dungeon
        if show_branches {
            for branch in &system.branches {
                if branch.end1.dungeon_num == i as i8 {
                    let dest_dungeon = system
                        .dungeons
                        .get(branch.end2.dungeon_num as usize)
                        .map(|d| d.name.as_str())
                        .unwrap_or("Unknown");

                    output.push_str(&format!(
                        "  {} at level {} -> {} level {}\n",
                        br_string2(branch),
                        branch.end1.level_num,
                        dest_dungeon,
                        branch.end2.level_num
                    ));
                }
            }
        }
    }

    output
}

/// Format a single dungeon's information
pub fn format_dungeon_info(dungeon: &Dungeon) -> String {
    let mut info = format!(
        "{} ({} levels, depth {}+)",
        dungeon.name, dungeon.num_levels, dungeon.depth_start
    );

    if dungeon.flags.hellish {
        info.push_str(" [Hellish]");
    }
    if dungeon.flags.maze_like {
        info.push_str(" [Maze]");
    }

    info
}

/// Print branch information
pub fn print_branch_info(system: &DungeonSystem, branch_id: i32) -> Option<String> {
    let branch = get_branch_by_id(system, branch_id)?;

    let from_name = system
        .dungeons
        .get(branch.end1.dungeon_num as usize)
        .map(|d| d.name.as_str())
        .unwrap_or("Unknown");
    let to_name = system
        .dungeons
        .get(branch.end2.dungeon_num as usize)
        .map(|d| d.name.as_str())
        .unwrap_or("Unknown");

    Some(format!(
        "Branch {}: {} level {} -> {} level {} ({})",
        branch_id,
        from_name,
        branch.end1.level_num,
        to_name,
        branch.end2.level_num,
        br_string2(branch)
    ))
}

// ============================================================================
// Level region placement (place_lregion, put_lregion_here equivalents)
// ============================================================================

use super::Level;
use crate::rng::GameRng;

/// Check if a location is bad for placing a region
/// (inside exclusion area or on certain terrain)
fn bad_location(level: &Level, x: u8, y: u8, exclude: &LRegionArea) -> bool {
    use super::CellType;

    // Check exclusion area
    if !exclude.is_empty() && exclude.contains(x, y) {
        return true;
    }

    let cell = &level.cells[x as usize][y as usize];

    // Can't place on walls, solid terrain, or existing features
    matches!(
        cell.typ,
        CellType::Stone
            | CellType::VWall
            | CellType::HWall
            | CellType::TLCorner
            | CellType::TRCorner
            | CellType::BLCorner
            | CellType::BRCorner
            | CellType::Stairs
            | CellType::Ladder
            | CellType::Lava
            | CellType::Water
            | CellType::Pool
            | CellType::IronBars
            | CellType::Tree
    )
}

/// Place a level region at specific coordinates (put_lregion_here equivalent)
///
/// Attempts to place a region feature at the given location.
///
/// # Arguments
/// * `level` - Level to modify
/// * `x`, `y` - Coordinates to place at
/// * `exclude` - Area to exclude
/// * `rtype` - Region type to place
/// * `oneshot` - If true, must place here even if bad
/// * `dest` - Destination level for portals/branches
///
/// # Returns
/// true if placed successfully
pub fn put_lregion_here(
    level: &mut Level,
    x: u8,
    y: u8,
    exclude: &LRegionArea,
    rtype: LRegionType,
    oneshot: bool,
    dest: Option<DLevel>,
) -> bool {
    use super::CellType;
    use super::level::{Stairway, TrapType};

    if bad_location(level, x, y, exclude) {
        if !oneshot {
            return false;
        }
        // Try to remove a trap if that's blocking us
        if let Some(idx) = level
            .traps
            .iter()
            .position(|t| t.x == x as i8 && t.y == y as i8)
        {
            let trap = &level.traps[idx];
            // Don't remove magic portals (they're important for level connectivity)
            if trap.trap_type != TrapType::MagicPortal {
                level.traps.remove(idx);
            } else {
                return false;
            }
        }
        // Still bad?
        if bad_location(level, x, y, exclude) {
            return false;
        }
    }

    match rtype {
        LRegionType::Tele | LRegionType::UpTele | LRegionType::DownTele => {
            // For teleport destinations, just clear the spot
            // The player will be placed here by the caller
            level.cells[x as usize][y as usize].typ = CellType::Room;
            true
        }
        LRegionType::Portal => {
            // Create magic portal
            if let Some(destination) = dest {
                level.add_trap(x as i8, y as i8, TrapType::MagicPortal);
                // Store destination in level's portal info
                level.stairs.push(Stairway {
                    x: x as i8,
                    y: y as i8,
                    destination,
                    up: false,
                });
            }
            true
        }
        LRegionType::Downstair => {
            level.cells[x as usize][y as usize].typ = CellType::Stairs;
            if let Some(destination) = dest {
                level.stairs.push(Stairway {
                    x: x as i8,
                    y: y as i8,
                    destination,
                    up: false,
                });
            }
            true
        }
        LRegionType::Upstair => {
            level.cells[x as usize][y as usize].typ = CellType::Stairs;
            if let Some(destination) = dest {
                level.stairs.push(Stairway {
                    x: x as i8,
                    y: y as i8,
                    destination,
                    up: true,
                });
            }
            true
        }
        LRegionType::Branch => {
            // Branch placement is handled by place_branch
            level.cells[x as usize][y as usize].typ = CellType::Stairs;
            level.flags.has_branch = true;
            true
        }
    }
}

/// Place a level region in an area (place_lregion equivalent)
///
/// Picks a location in the given area (but not in the exclusion area)
/// and places a feature based on region type.
///
/// # Arguments
/// * `level` - Level to modify
/// * `area` - Area to place within (0,0,0,0 means whole level)
/// * `exclude` - Area to exclude from placement
/// * `rtype` - Region type to place
/// * `dest` - Destination level for portals/branches
/// * `rng` - Random number generator
///
/// # Returns
/// true if placed successfully
pub fn place_lregion(
    level: &mut Level,
    area: &LRegionArea,
    exclude: &LRegionArea,
    rtype: LRegionType,
    dest: Option<DLevel>,
    rng: &mut GameRng,
) -> bool {
    use crate::{COLNO, ROWNO};

    // Default to whole level if area is empty
    let (lx, ly, hx, hy) = if area.is_empty() {
        (1u8, 0u8, (COLNO - 1) as u8, (ROWNO - 1) as u8)
    } else {
        (area.x1, area.y1, area.x2, area.y2)
    };

    let oneshot = lx == hx && ly == hy;

    // First try probabilistic approach
    for _ in 0..200 {
        let x = (rng.rn2((hx - lx + 1) as u32) + lx as u32) as u8;
        let y = (rng.rn2((hy - ly + 1) as u32) + ly as u32) as u8;

        if put_lregion_here(level, x, y, exclude, rtype, oneshot, dest) {
            return true;
        }
    }

    // Then try deterministic approach
    for x in lx..=hx {
        for y in ly..=hy {
            if put_lregion_here(level, x, y, exclude, rtype, true, dest) {
                return true;
            }
        }
    }

    // Failed to place
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::CellType;

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

        assert_eq!(
            system.dungeon_name(&DLevel::new(0, 1)),
            "The Dungeons of Doom"
        );
        assert_eq!(system.dungeon_name(&DLevel::new(1, 1)), "Gehennom");
        assert_eq!(system.dungeon_name(&DLevel::new(2, 1)), "The Gnomish Mines");
        assert_eq!(system.dungeon_name(&DLevel::new(3, 1)), "Sokoban");
    }

    #[test]
    fn test_init_dungeons() {
        let system = init_dungeons();
        assert_eq!(system.dungeons.len(), 8);
        assert_eq!(system.branches.len(), 7);
    }

    #[test]
    fn test_is_main_dungeon_const() {
        assert!(is_main_dungeon(0));
        assert!(!is_main_dungeon(1));
        assert!(!is_main_dungeon(2));
        assert!(!is_main_dungeon(99));
    }

    #[test]
    fn test_is_in_main_dungeon() {
        assert!(is_in_main_dungeon(&DLevel::new(0, 1)));
        assert!(is_in_main_dungeon(&DLevel::new(0, 29)));
        assert!(!is_in_main_dungeon(&DLevel::new(1, 1)));
        assert!(!is_in_main_dungeon(&DLevel::new(2, 1)));
    }

    #[test]
    fn test_br_string() {
        assert_eq!(br_string(BranchType::Stairs), "stairs");
        assert_eq!(br_string(BranchType::NoEnd1), "no-connection-1");
        assert_eq!(br_string(BranchType::NoEnd2), "no-connection-2");
        assert_eq!(br_string(BranchType::Portal), "portal");
    }

    #[test]
    fn test_br_string2() {
        let system = DungeonSystem::new();

        // Mines branch (going down)
        let mines_branch = system.get_branch_from(&DLevel::new(0, 3)).unwrap();
        let desc = br_string2(mines_branch);
        assert!(desc.contains("stairs"));
        assert!(desc.contains("down"));

        // Sokoban branch (going up)
        let sokoban_branch = system.get_branch_from(&DLevel::new(0, 7)).unwrap();
        let desc = br_string2(sokoban_branch);
        assert!(desc.contains("stairs"));
        assert!(desc.contains("up"));

        // Quest portal
        let quest_branch = system.get_branch_from(&DLevel::new(0, 14)).unwrap();
        let desc = br_string2(quest_branch);
        assert_eq!(desc, "magic portal");
    }

    #[test]
    fn test_is_hellish() {
        let system = DungeonSystem::new();

        // Gehennom is hellish
        assert!(is_hellish(&system, 1));

        // Main dungeon is not hellish
        assert!(!is_hellish(&system, 0));

        // Invalid dungeon
        assert!(!is_hellish(&system, 99));
    }

    #[test]
    fn test_level_in_hellish_dungeon() {
        let system = DungeonSystem::new();

        // Gehennom is hellish (maze_like && hellish)
        assert!(level_in_hellish_dungeon(&system, &DLevel::new(1, 1)));
        assert!(level_in_hellish_dungeon(&system, &DLevel::new(1, 10)));

        // Main dungeon is not hellish
        assert!(!level_in_hellish_dungeon(&system, &DLevel::new(0, 5)));

        // Mines is not hellish
        assert!(!level_in_hellish_dungeon(&system, &DLevel::new(2, 1)));
    }

    #[test]
    fn test_is_maze_like() {
        let system = DungeonSystem::new();

        // Endgame is maze-like
        assert!(is_maze_like(&system, 7));

        // Main dungeon is not maze-like
        assert!(!is_maze_like(&system, 0));

        // Gehennom is hellish but not explicitly maze-like
        assert!(!is_maze_like(&system, 1));

        // Invalid dungeon
        assert!(!is_maze_like(&system, 99));
    }

    #[test]
    fn test_is_town() {
        let system = DungeonSystem::new();

        // None of the standard dungeons are towns
        assert!(!is_town(&system, 0));
        assert!(!is_town(&system, 1));
        assert!(!is_town(&system, 7));

        // Invalid dungeon
        assert!(!is_town(&system, 99));
    }

    #[test]
    fn test_numdungeons() {
        let system = DungeonSystem::new();
        assert_eq!(numdungeons(&system), 8);
    }

    #[test]
    fn test_num_branches() {
        let system = DungeonSystem::new();
        assert_eq!(num_branches(&system), 7);
    }

    #[test]
    fn test_get_branch_by_id() {
        let system = DungeonSystem::new();

        // Fort Ludios portal is ID 4
        let knox_branch = get_branch_by_id(&system, 4);
        assert!(knox_branch.is_some());
        assert_eq!(knox_branch.unwrap().id, 4);

        // Invalid ID
        assert!(get_branch_by_id(&system, 999).is_none());
    }

    #[test]
    fn test_levels_connected() {
        let system = DungeonSystem::new();

        // Mines entrance connects level 3 to Mines level 1
        assert!(levels_connected(
            &system,
            &DLevel::new(0, 3),
            &DLevel::new(2, 1)
        ));

        // Reverse order should also work
        assert!(levels_connected(
            &system,
            &DLevel::new(2, 1),
            &DLevel::new(0, 3)
        ));

        // Unconnected levels
        assert!(!levels_connected(
            &system,
            &DLevel::new(0, 1),
            &DLevel::new(2, 1)
        ));
    }

    #[test]
    fn test_mkportal() {
        let system = DungeonSystem::new();

        // Quest portal
        let quest_dest = mkportal(&system, &DLevel::new(0, 14));
        assert_eq!(quest_dest, Some(DLevel::new(4, 1)));

        // Fort Ludios portal
        let knox_dest = mkportal(&system, &DLevel::new(0, 12));
        assert_eq!(knox_dest, Some(DLevel::new(5, 1)));

        // Endgame portal
        let endgame_dest = mkportal(&system, &DLevel::new(1, 20));
        assert_eq!(endgame_dest, Some(DLevel::new(7, 1)));

        // Non-portal level
        assert_eq!(mkportal(&system, &DLevel::new(0, 1)), None);
    }

    #[test]
    fn test_next_portal() {
        let system = DungeonSystem::new();

        // next_portal should be same as mkportal
        assert_eq!(
            next_portal(&system, &DLevel::new(0, 14)),
            mkportal(&system, &DLevel::new(0, 14))
        );
    }

    #[test]
    fn test_mk_knox_portal() {
        let system = DungeonSystem::new();

        let knox = mk_knox_portal(&system);
        assert!(knox.is_some());
        let knox = knox.unwrap();
        assert_eq!(knox.id, 4);
        assert_eq!(knox.branch_type, BranchType::Portal);
        assert_eq!(knox.end2, DLevel::new(5, 1)); // Fort Ludios
    }

    #[test]
    fn test_parent_level() {
        let system = DungeonSystem::new();

        // Mines level 1's parent is main dungeon level 3
        let parent = parent_level(&system, &DLevel::new(2, 1));
        assert_eq!(parent, Some(DLevel::new(0, 3)));

        // Sokoban level 1's parent is main dungeon level 7
        let parent = parent_level(&system, &DLevel::new(3, 1));
        assert_eq!(parent, Some(DLevel::new(0, 7)));

        // Main dungeon level 3 has no parent (not an entrance)
        let parent = parent_level(&system, &DLevel::new(0, 3));
        assert_eq!(parent, None);
    }

    #[test]
    fn test_has_branch_from_level() {
        let system = DungeonSystem::new();

        // Main dungeon level 3 has Mines entrance
        assert!(has_branch_from_level(&system, &DLevel::new(0, 3)));

        // Main dungeon level 7 has Sokoban entrance
        assert!(has_branch_from_level(&system, &DLevel::new(0, 7)));

        // Main dungeon level 1 has no branch
        assert!(!has_branch_from_level(&system, &DLevel::new(0, 1)));

        // Mines level 1 has no outgoing branch
        assert!(!has_branch_from_level(&system, &DLevel::new(2, 1)));
    }

    #[test]
    fn test_dungeon_index() {
        // Valid indices
        assert_eq!(dungeon_index(0), Some(0));
        assert_eq!(dungeon_index(1), Some(1));
        assert_eq!(dungeon_index(7), Some(7));

        // Invalid indices
        assert_eq!(dungeon_index(-1), None);
        assert_eq!(dungeon_index(8), None);
        assert_eq!(dungeon_index(99), None);
    }

    #[test]
    fn test_dungeon_alignment() {
        let system = DungeonSystem::new();

        // All dungeons have neutral alignment by default
        assert_eq!(dungeon_alignment(&system, 0), 0);
        assert_eq!(dungeon_alignment(&system, 1), 0);
        assert_eq!(dungeon_alignment(&system, 7), 0);

        // Invalid dungeon
        assert_eq!(dungeon_alignment(&system, 99), 0);
    }

    #[test]
    fn test_save_restore_dungeon() {
        let system = DungeonSystem::new();

        // Save
        let saved = save_dungeon(&system).expect("should serialize");
        assert!(!saved.is_empty());

        // Restore
        let restored = restore_dungeon(&saved).expect("should deserialize");
        assert_eq!(restored.dungeons.len(), system.dungeons.len());
        assert_eq!(restored.branches.len(), system.branches.len());
    }

    #[test]
    fn test_print_dungeon() {
        let system = DungeonSystem::new();

        let output = print_dungeon(&system, false);
        assert!(output.contains("Dungeon Topology"));
        assert!(output.contains("The Dungeons of Doom"));
        assert!(output.contains("Gehennom"));

        // With branches
        let output_with_branches = print_dungeon(&system, true);
        assert!(output_with_branches.contains("stairs"));
    }

    #[test]
    fn test_format_dungeon_info() {
        let dungeon = Dungeon::gehennom();
        let info = format_dungeon_info(&dungeon);
        assert!(info.contains("Gehennom"));
        assert!(info.contains("Hellish"));
    }

    #[test]
    fn test_print_branch_info() {
        let system = DungeonSystem::new();

        let info = print_branch_info(&system, 0);
        assert!(info.is_some());
        let info_str = info.unwrap();
        assert!(info_str.contains("Branch 0"));

        // Invalid branch
        assert!(print_branch_info(&system, 999).is_none());
    }

    #[test]
    fn test_lregion_area() {
        let area = LRegionArea::new(5, 5, 15, 10);
        assert!(area.contains(10, 7));
        assert!(!area.contains(3, 3));
        assert!(!area.is_empty());

        let empty = LRegionArea::default();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_place_lregion() {
        let mut rng = crate::rng::GameRng::new(42);
        let mut level = Level::new(DLevel::new(0, 5));

        // Fill with room tiles for testing
        for x in 10..30 {
            for y in 5..15 {
                level.cells[x][y].typ = CellType::Room;
            }
        }

        let area = LRegionArea::new(10, 5, 29, 14);
        let exclude = LRegionArea::default();

        // Place downstairs
        let result = place_lregion(
            &mut level,
            &area,
            &exclude,
            LRegionType::Downstair,
            Some(DLevel::new(0, 6)),
            &mut rng,
        );

        assert!(result);
        assert!(!level.stairs.is_empty());
    }

    #[test]
    fn test_put_lregion_here() {
        let mut level = Level::new(DLevel::new(0, 5));

        // Set up a room cell
        level.cells[20][10].typ = CellType::Room;

        let exclude = LRegionArea::default();

        // Place upstairs
        let result = put_lregion_here(
            &mut level,
            20,
            10,
            &exclude,
            LRegionType::Upstair,
            false,
            Some(DLevel::new(0, 4)),
        );

        assert!(result);
        assert_eq!(level.cells[20][10].typ, CellType::Stairs);
        assert!(!level.stairs.is_empty());
        assert!(level.stairs[0].up);
    }

    #[test]
    fn test_put_lregion_here_bad_location() {
        let mut level = Level::new(DLevel::new(0, 5));

        // Stone is a bad location
        let exclude = LRegionArea::default();

        let result = put_lregion_here(
            &mut level,
            20,
            10,
            &exclude,
            LRegionType::Upstair,
            false, // not oneshot, so should fail
            None,
        );

        assert!(!result);
    }
}
