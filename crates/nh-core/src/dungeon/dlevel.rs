//! Dungeon level identifier (dungeon.c)
//!
//! Functions for managing dungeon levels and dungeon system queries.
//! These are equivalents of NetHack's C functions for level identification and navigation.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use serde::{Deserialize, Serialize};

use super::topology::DungeonSystem;

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
            0 => 1,  // Main dungeon
            1 => 30, // Gehennom
            2 => 2,  // Gnomish Mines
            3 => 6,  // Sokoban
            4 => 16, // Quest
            5 => 18, // Fort Ludios
            6 => 37, // Vlad's Tower
            _ => 1,  // Default
        };
        depth_start + (self.level_num as i32) - 1
    }

    /// Check if deeper than another level
    pub fn is_deeper(&self, other: &DLevel) -> bool {
        self.depth() > other.depth()
    }

    /// Check if this is the same level as another (on_level equivalent)
    pub fn on_level(&self, other: &DLevel) -> bool {
        self.dungeon_num == other.dungeon_num && self.level_num == other.level_num
    }

    /// Check if this level has been assigned (has valid values)
    pub fn is_assigned(&self) -> bool {
        self.dungeon_num != 0 || self.level_num != 0
    }
}

impl core::fmt::Display for DLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Dlvl:{}", self.level_num)
    }
}

/// Calculate level difficulty (level_difficulty equivalent)
///
/// This is used for monster/trap generation difficulty.
/// The difficulty is based on the dungeon depth plus player level divided by 2.
///
/// # Arguments
/// * `dlevel` - The dungeon level
/// * `player_level` - The player's experience level
pub fn level_difficulty(dlevel: &DLevel, player_level: i32) -> i32 {
    let depth = dlevel.depth();
    // C formula: depth + (u.ulevel / 2)
    depth + player_level / 2
}

/// Get number of levels in a dungeon
/// Matches C's dunlevs_in_dungeon()
///
/// # Arguments
/// * `dungeon_system` - The global dungeon system
/// * `dlevel` - The dungeon level
///
/// # Returns
/// The total number of levels in the dungeon containing this level
pub fn dunlevs_in_dungeon(dungeon_system: &DungeonSystem, dlevel: &DLevel) -> i8 {
    dungeon_system
        .dungeons
        .get(dlevel.dungeon_num as usize)
        .map(|d| d.num_levels)
        .unwrap_or(0)
}

/// Get parent dungeon level (entrance to current dungeon)
/// Matches C's parent_dlevel()
///
/// # Arguments
/// * `dungeon_system` - The global dungeon system
/// * `dlevel` - The dungeon level
///
/// # Returns
/// The parent dungeon level if this level is a branch entrance, None otherwise
pub fn parent_dlevel(dungeon_system: &DungeonSystem, dlevel: &DLevel) -> Option<DLevel> {
    // Find branch where this level is the entrance (end2)
    dungeon_system
        .branches
        .iter()
        .find(|b| b.end2 == *dlevel)
        .map(|b| b.end1)
}

/// Get parent dungeon number
/// Matches C's parent_dnum()
///
/// # Arguments
/// * `dungeon_system` - The global dungeon system
/// * `dungeon_num` - The dungeon number
///
/// # Returns
/// The parent dungeon number if this dungeon is a branch, None otherwise
pub fn parent_dnum(dungeon_system: &DungeonSystem, dungeon_num: i8) -> Option<i8> {
    // Find any level in this dungeon and get its parent
    let sample_level = DLevel::new(dungeon_num, 1);
    parent_dlevel(dungeon_system, &sample_level).map(|p| p.dungeon_num)
}

/// Convert dungeon name to dungeon number
/// Matches C's dname_to_dnum()
///
/// # Arguments
/// * `dungeon_system` - The global dungeon system
/// * `name` - The dungeon name
///
/// # Returns
/// The dungeon number (0-7) if found, None otherwise (C version panics)
pub fn dname_to_dnum(dungeon_system: &DungeonSystem, name: &str) -> Option<i8> {
    dungeon_system
        .dungeons
        .iter()
        .position(|d| d.name == name)
        .map(|idx| idx as i8)
}

/// Get current level number in dungeon (dunlev equivalent)
///
/// This is a simple accessor for DLevel.level_num, provided for C compatibility.
///
/// # Arguments
/// * `dlevel` - The dungeon level
///
/// # Returns
/// The level number within the dungeon
pub const fn dunlev(dlevel: &DLevel) -> i8 {
    dlevel.level_num
}

// ============================================================================
// Ledger functions (for bookkeeping/save-restore)
// ============================================================================

/// Get ledger number for a level (ledger_no equivalent)
///
/// Returns a unique bookkeeping number for any dungeon level.
/// Used for save files and level comparisons.
///
/// # Arguments
/// * `dungeon_system` - The global dungeon system
/// * `dlevel` - The dungeon level
///
/// # Returns
/// The ledger number
pub fn ledger_no(dungeon_system: &DungeonSystem, dlevel: &DLevel) -> i32 {
    dungeon_system
        .dungeons
        .get(dlevel.dungeon_num as usize)
        .map(|d| d.ledger_start + dlevel.level_num as i32)
        .unwrap_or(0)
}

/// Get the maximum ledger number (maxledgerno equivalent)
///
/// Returns the highest possible ledger number in the dungeon system.
///
/// # Arguments
/// * `dungeon_system` - The global dungeon system
///
/// # Returns
/// The maximum ledger number
pub fn maxledgerno(dungeon_system: &DungeonSystem) -> i32 {
    dungeon_system
        .dungeons
        .last()
        .map(|d| d.ledger_start + d.num_levels as i32)
        .unwrap_or(0)
}

/// Convert ledger number to dungeon number (ledger_to_dnum equivalent)
///
/// # Arguments
/// * `dungeon_system` - The global dungeon system
/// * `ledgerno` - The ledger number
///
/// # Returns
/// The dungeon number containing this ledger entry, or None if invalid
pub fn ledger_to_dnum(dungeon_system: &DungeonSystem, ledgerno: i32) -> Option<i8> {
    for (i, dungeon) in dungeon_system.dungeons.iter().enumerate() {
        if dungeon.ledger_start < ledgerno
            && ledgerno <= dungeon.ledger_start + dungeon.num_levels as i32
        {
            return Some(i as i8);
        }
    }
    None
}

/// Convert ledger number to dungeon level number (ledger_to_dlev equivalent)
///
/// # Arguments
/// * `dungeon_system` - The global dungeon system
/// * `ledgerno` - The ledger number
///
/// # Returns
/// The level number within its dungeon, or None if invalid
pub fn ledger_to_dlev(dungeon_system: &DungeonSystem, ledgerno: i32) -> Option<i8> {
    let dnum = ledger_to_dnum(dungeon_system, ledgerno)?;
    let dungeon = dungeon_system.dungeons.get(dnum as usize)?;
    Some((ledgerno - dungeon.ledger_start) as i8)
}

/// Convert ledger number to DLevel (combines ledger_to_dnum and ledger_to_dlev)
///
/// # Arguments
/// * `dungeon_system` - The global dungeon system
/// * `ledgerno` - The ledger number
///
/// # Returns
/// The DLevel if valid, None otherwise
pub fn ledgerno_to_dlevel(dungeon_system: &DungeonSystem, ledgerno: i32) -> Option<DLevel> {
    let dnum = ledger_to_dnum(dungeon_system, ledgerno)?;
    let dlev = ledger_to_dlev(dungeon_system, ledgerno)?;
    Some(DLevel::new(dnum, dlev))
}

// ============================================================================
// Level query functions (Is_* and In_* macros from C)
// ============================================================================

/// Check if this is the bottom level of a dungeon (Is_botlevel equivalent)
///
/// # Arguments
/// * `dungeon_system` - The global dungeon system
/// * `dlevel` - The dungeon level
///
/// # Returns
/// True if this is the bottom level of its dungeon
pub fn is_botlevel(dungeon_system: &DungeonSystem, dlevel: &DLevel) -> bool {
    dungeon_system
        .dungeons
        .get(dlevel.dungeon_num as usize)
        .map(|d| dlevel.level_num == d.num_levels)
        .unwrap_or(false)
}

/// Check if a dungeon builds upward (builds_up equivalent)
///
/// Some dungeons like Sokoban and Vlad's Tower are entered from
/// below and build upwards.
///
/// # Arguments
/// * `dungeon_system` - The global dungeon system
/// * `dlevel` - The dungeon level
///
/// # Returns
/// True if this dungeon builds upward
pub fn builds_up(dungeon_system: &DungeonSystem, dlevel: &DLevel) -> bool {
    dungeon_system
        .dungeons
        .get(dlevel.dungeon_num as usize)
        .map(|d| d.num_levels > 1 && d.entry_level == d.num_levels)
        .unwrap_or(false)
}

/// Check if level is in the Quest dungeon (In_quest equivalent)
pub fn in_quest(dlevel: &DLevel) -> bool {
    dlevel.dungeon_num == 4 // Quest is dungeon 4
}

/// Check if level is in the Gnomish Mines (In_mines equivalent)
pub fn in_mines(dlevel: &DLevel) -> bool {
    dlevel.dungeon_num == 2 // Mines is dungeon 2
}

/// Check if level is in Gehennom/Hell (In_hell equivalent)
pub fn in_hell(dlevel: &DLevel) -> bool {
    dlevel.dungeon_num == 1 // Gehennom is dungeon 1
}

/// Check if level is in Vlad's Tower (In_V_tower equivalent)
pub fn in_v_tower(dlevel: &DLevel) -> bool {
    dlevel.dungeon_num == 6 // Vlad's Tower is dungeon 6
}

/// Check if level is in the endgame planes (In_endgame equivalent)
pub fn in_endgame(dlevel: &DLevel) -> bool {
    dlevel.dungeon_num == 7 // Endgame is dungeon 7
}

/// Check if level is in Sokoban (In_sokoban equivalent)
pub fn in_sokoban(dlevel: &DLevel) -> bool {
    dlevel.dungeon_num == 3 // Sokoban is dungeon 3
}

/// Check if level is in Fort Ludios (In_knox equivalent)
pub fn in_knox(dlevel: &DLevel) -> bool {
    dlevel.dungeon_num == 5 // Fort Ludios is dungeon 5
}

/// Get the observable depth (for display purposes)
///
/// This is slightly different from depth() - it shows what the player sees.
///
/// # Arguments
/// * `dlevel` - The dungeon level
///
/// # Returns
/// The depth as displayed to the player
pub fn observable_depth(dlevel: &DLevel) -> i32 {
    dlevel.depth()
}

// ============================================================================
// Special level checks
// ============================================================================

/// Well-known special level identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialLevel {
    /// Oracle level (main dungeon around 6-9)
    Oracle,
    /// Big room level (optional, main dungeon around 10-12)
    BigRoom,
    /// Rogue level (main dungeon around 15-18)
    RogueLike,
    /// Medusa level (main dungeon around 24)
    Medusa,
    /// Castle/Stronghold level (main dungeon 25)
    Castle,
    /// Valley of the Dead (Gehennom entrance)
    Valley,
    /// Asmodeus lair (Gehennom)
    Asmodeus,
    /// Baalzebub lair (Gehennom)
    Baalzebub,
    /// Juiblex lair (Gehennom)
    Juiblex,
    /// Orcus lair (Gehennom)
    Orcus,
    /// Wizard tower level 1
    Wizard1,
    /// Wizard tower level 2
    Wizard2,
    /// Wizard tower level 3
    Wizard3,
    /// Sanctum (bottom of Gehennom)
    Sanctum,
    /// Mine's end
    MinesEnd,
    /// Sokoban end
    SokobanEnd,
    /// Quest home level
    QuestStart,
    /// Quest locate level
    QuestLocate,
    /// Quest goal level (nemesis)
    QuestGoal,
    /// Fort Ludios
    Knox,
    /// Earth plane
    Earth,
    /// Air plane
    Air,
    /// Fire plane
    Fire,
    /// Water plane
    Water,
    /// Astral plane
    Astral,
}

/// Check if level is the invocation level (bottom of Gehennom, for sanctum)
pub fn invocation_lev(dungeon_system: &DungeonSystem, dlevel: &DLevel) -> bool {
    // Invocation level is one above the sanctum (bottom - 1) in Gehennom
    if dlevel.dungeon_num != 1 {
        return false;
    }
    if let Some(gehennom) = dungeon_system.dungeons.get(1) {
        // Invocation is level num_levels - 1 (sanctum is num_levels)
        dlevel.level_num == gehennom.num_levels - 1
    } else {
        false
    }
}

/// Check if level contains the Wizard's tower (On_W_tower_level equivalent)
///
/// The Wizard's tower spans 3 levels in Gehennom.
pub fn on_w_tower_level(dlevel: &DLevel) -> bool {
    // Wizard tower is in Gehennom, typically around levels 12-14
    // We check for the specific wizard tower dungeon locations
    if dlevel.dungeon_num != 1 {
        return false;
    }
    // Wizard tower levels are around 12-14 in Gehennom
    // (exact levels depend on dungeon generation)
    matches!(dlevel.level_num, 12..=14)
}

/// Check if a position is inside the Wizard's tower (In_W_tower equivalent)
///
/// The tower occupies a specific region on its level.
/// This is a simplified check - the real implementation uses
/// exclusion regions from the level data.
pub fn in_w_tower(_x: i8, _y: i8, dlevel: &DLevel) -> bool {
    // Simplified: if on tower level, assume inside tower
    // Real implementation would check specific coordinates
    on_w_tower_level(dlevel)
}

/// Check if level is unreachable (unreachable_level equivalent)
///
/// A level is unreachable if:
/// - It's unplaced (floating branch not connected)
/// - Player is in endgame but level is not
/// - It's the dummy level
pub fn unreachable_level(
    _dungeon_system: &DungeonSystem,
    dlevel: &DLevel,
    player_dlevel: &DLevel,
    unplaced: bool,
) -> bool {
    if unplaced {
        return true;
    }
    // If player is in endgame, non-endgame levels are unreachable
    if in_endgame(player_dlevel) && !in_endgame(dlevel) {
        return true;
    }
    false
}

/// Check if level should never generate bones (no_bones_level equivalent)
///
/// Some levels should never save/load bones files.
pub fn no_bones_level(dlevel: &DLevel) -> bool {
    // No bones in endgame planes
    if in_endgame(dlevel) {
        return true;
    }
    // No bones in certain special levels (simplified check)
    // Real implementation checks specific levels
    false
}

/// Get the final level of a dungeon (final_level equivalent)
pub fn final_level(dungeon_system: &DungeonSystem, dnum: i8) -> Option<DLevel> {
    dungeon_system
        .dungeons
        .get(dnum as usize)
        .map(|d| DLevel::new(dnum, d.num_levels))
}

/// Get the deepest level reached by the player (deepest_lev_reached equivalent)
///
/// # Arguments
/// * `dungeon_system` - The global dungeon system
/// * `no_quest` - If true, exclude the Quest dungeon from consideration
///
/// # Returns
/// The deepest depth reached
pub fn deepest_lev_reached(dungeon_system: &DungeonSystem, no_quest: bool) -> i32 {
    let mut max_depth = 0;
    for (i, dungeon) in dungeon_system.dungeons.iter().enumerate() {
        // Skip quest if requested
        if no_quest && i == 4 {
            continue;
        }
        // Skip if player hasn't reached this dungeon
        if dungeon.deepest_reached == 0 {
            continue;
        }
        let level = DLevel::new(i as i8, dungeon.deepest_reached);
        let depth = level.depth();
        if depth > max_depth {
            max_depth = depth;
        }
    }
    max_depth
}

// ============================================================================
// Level description functions
// ============================================================================

/// Get endgame level name (endgamelevelname equivalent)
pub fn endgamelevelname(dlevel: &DLevel) -> Option<&'static str> {
    if dlevel.dungeon_num != 7 {
        return None;
    }
    match dlevel.level_num {
        1 => Some("Earth Plane"),
        2 => Some("Air Plane"),
        3 => Some("Fire Plane"),
        4 => Some("Water Plane"),
        5 => Some("Astral Plane"),
        _ => None,
    }
}

/// Get generic level description (generic_lvl_desc equivalent)
pub fn generic_lvl_desc(dungeon_system: &DungeonSystem, dlevel: &DLevel) -> String {
    if let Some(name) = endgamelevelname(dlevel) {
        return name.to_string();
    }

    let dungeon_name = dungeon_system
        .dungeons
        .get(dlevel.dungeon_num as usize)
        .map(|d| d.name.as_str())
        .unwrap_or("Unknown");

    format!("{} level {}", dungeon_name, dlevel.level_num)
}

/// Describe a level for display (describe_level equivalent)
pub fn describe_level(dungeon_system: &DungeonSystem, dlevel: &DLevel) -> String {
    // Check for special endgame names
    if let Some(name) = endgamelevelname(dlevel) {
        return name.to_string();
    }

    let dungeon_name = dungeon_system
        .dungeons
        .get(dlevel.dungeon_num as usize)
        .map(|d| d.name.as_str())
        .unwrap_or("Unknown");

    let depth = dlevel.depth();

    // Show dungeon name and depth
    if dlevel.is_main_dungeon() {
        format!("Dlvl:{}", depth)
    } else {
        format!("{}:{}", dungeon_name, dlevel.level_num)
    }
}

// ============================================================================
// Level navigation helpers
// ============================================================================

/// Assign one level to another (assign_level equivalent)
pub fn assign_level(dest: &mut DLevel, src: &DLevel) {
    dest.dungeon_num = src.dungeon_num;
    dest.level_num = src.level_num;
}

/// Assign a random level in a dungeon (assign_rnd_level equivalent)
pub fn assign_rnd_level(
    dungeon_system: &DungeonSystem,
    dnum: i8,
    rng: &mut crate::rng::GameRng,
) -> Option<DLevel> {
    let dungeon = dungeon_system.dungeons.get(dnum as usize)?;
    let level = rng.rn2(dungeon.num_levels as u32) as i8 + 1;
    Some(DLevel::new(dnum, level))
}

/// Get level for a given depth (get_level equivalent)
///
/// Given a depth value, find the appropriate level.
/// If the depth is in the current dungeon, return that level.
/// If above current dungeon, trace back through parent branches.
pub fn get_level(dungeon_system: &DungeonSystem, current: &DLevel, depth: i32) -> DLevel {
    let mut dgn = current.dungeon_num;

    // Get current dungeon info
    let Some(dungeon) = dungeon_system.dungeons.get(dgn as usize) else {
        return *current;
    };

    // If depth is within current dungeon, calculate level
    let dungeon_end_depth = dungeon.depth_start + dungeon.num_levels as i32 - 1;

    if depth >= dungeon.depth_start && depth <= dungeon_end_depth {
        let level = (depth - dungeon.depth_start + 1) as i8;
        return DLevel::new(dgn, level);
    }

    // If depth is above current dungeon, trace up through branches
    if depth < dungeon.depth_start {
        // Find parent dungeon
        for branch in &dungeon_system.branches {
            if branch.end2.dungeon_num == dgn {
                dgn = branch.end1.dungeon_num;
                // Recursively find level in parent
                return get_level(dungeon_system, &branch.end1, depth);
            }
        }
    }

    // Default: deepest level of current dungeon
    DLevel::new(dgn, dungeon.num_levels)
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

    #[test]
    fn test_dunlevs_in_dungeon() {
        let system = super::super::topology::DungeonSystem::new();

        // Main dungeon has 29 levels
        assert_eq!(dunlevs_in_dungeon(&system, &DLevel::new(0, 1)), 29);
        assert_eq!(dunlevs_in_dungeon(&system, &DLevel::new(0, 15)), 29);

        // Gehennom has 20 levels
        assert_eq!(dunlevs_in_dungeon(&system, &DLevel::new(1, 1)), 20);
        assert_eq!(dunlevs_in_dungeon(&system, &DLevel::new(1, 20)), 20);

        // Gnomish Mines has 8 levels
        assert_eq!(dunlevs_in_dungeon(&system, &DLevel::new(2, 1)), 8);

        // Sokoban has 4 levels
        assert_eq!(dunlevs_in_dungeon(&system, &DLevel::new(3, 1)), 4);

        // Invalid dungeon returns 0
        assert_eq!(dunlevs_in_dungeon(&system, &DLevel::new(99, 1)), 0);
    }

    #[test]
    fn test_parent_dlevel() {
        let system = super::super::topology::DungeonSystem::new();

        // Mines level 1 has parent at main dungeon level 3
        let parent = parent_dlevel(&system, &DLevel::new(2, 1));
        assert_eq!(parent, Some(DLevel::new(0, 3)));

        // Sokoban level 1 has parent at main dungeon level 7
        let parent = parent_dlevel(&system, &DLevel::new(3, 1));
        assert_eq!(parent, Some(DLevel::new(0, 7)));

        // Gehennom level 1 has parent at main dungeon level 25
        let parent = parent_dlevel(&system, &DLevel::new(1, 1));
        assert_eq!(parent, Some(DLevel::new(0, 25)));

        // Non-entrance level has no parent
        let parent = parent_dlevel(&system, &DLevel::new(0, 5));
        assert_eq!(parent, None);
    }

    #[test]
    fn test_parent_dnum() {
        let system = super::super::topology::DungeonSystem::new();

        // Mines (2) has parent dungeon 0 (Main)
        assert_eq!(parent_dnum(&system, 2), Some(0));

        // Sokoban (3) has parent dungeon 0 (Main)
        assert_eq!(parent_dnum(&system, 3), Some(0));

        // Gehennom (1) has parent dungeon 0 (Main)
        assert_eq!(parent_dnum(&system, 1), Some(0));

        // Main dungeon has no parent
        assert_eq!(parent_dnum(&system, 0), None);
    }

    #[test]
    fn test_dname_to_dnum() {
        let system = super::super::topology::DungeonSystem::new();

        // Standard dungeon names
        assert_eq!(dname_to_dnum(&system, "The Dungeons of Doom"), Some(0));
        assert_eq!(dname_to_dnum(&system, "Gehennom"), Some(1));
        assert_eq!(dname_to_dnum(&system, "The Gnomish Mines"), Some(2));
        assert_eq!(dname_to_dnum(&system, "Sokoban"), Some(3));
        assert_eq!(dname_to_dnum(&system, "The Quest"), Some(4));
        assert_eq!(dname_to_dnum(&system, "Fort Ludios"), Some(5));
        assert_eq!(dname_to_dnum(&system, "Vlad's Tower"), Some(6));
        assert_eq!(dname_to_dnum(&system, "The Planes"), Some(7));

        // Unknown dungeon returns None
        assert_eq!(dname_to_dnum(&system, "Unknown Dungeon"), None);
    }

    #[test]
    fn test_dunlev() {
        assert_eq!(dunlev(&DLevel::new(0, 1)), 1);
        assert_eq!(dunlev(&DLevel::new(0, 15)), 15);
        assert_eq!(dunlev(&DLevel::new(1, 5)), 5);
    }

    #[test]
    fn test_level_difficulty() {
        // Depth 1, player level 5: 1 + 5/2 = 1 + 2 = 3
        assert_eq!(level_difficulty(&DLevel::new(0, 1), 5), 3);

        // Depth 10, player level 10: 10 + 10/2 = 10 + 5 = 15
        assert_eq!(level_difficulty(&DLevel::new(0, 10), 10), 15);

        // Gehennom depth 30, player level 20: 30 + 20/2 = 30 + 10 = 40
        assert_eq!(level_difficulty(&DLevel::new(1, 1), 20), 40);
    }

    #[test]
    fn test_ledger_no() {
        let system = DungeonSystem::new();

        // Main dungeon: ledger_start = 0
        assert_eq!(ledger_no(&system, &DLevel::new(0, 1)), 1);
        assert_eq!(ledger_no(&system, &DLevel::new(0, 10)), 10);
        assert_eq!(ledger_no(&system, &DLevel::new(0, 29)), 29);

        // Gehennom: ledger_start = 29
        assert_eq!(ledger_no(&system, &DLevel::new(1, 1)), 30);
        assert_eq!(ledger_no(&system, &DLevel::new(1, 20)), 49);
    }

    #[test]
    fn test_maxledgerno() {
        let system = DungeonSystem::new();
        // Should be sum of all dungeon levels
        assert!(maxledgerno(&system) > 50);
    }

    #[test]
    fn test_ledger_to_dnum() {
        let system = DungeonSystem::new();

        // Main dungeon levels
        assert_eq!(ledger_to_dnum(&system, 1), Some(0));
        assert_eq!(ledger_to_dnum(&system, 29), Some(0));

        // Gehennom levels (ledger 30-49)
        assert_eq!(ledger_to_dnum(&system, 30), Some(1));
    }

    #[test]
    fn test_ledger_to_dlev() {
        let system = DungeonSystem::new();

        // Main dungeon
        assert_eq!(ledger_to_dlev(&system, 1), Some(1));
        assert_eq!(ledger_to_dlev(&system, 15), Some(15));

        // Gehennom (ledger_start = 29)
        assert_eq!(ledger_to_dlev(&system, 30), Some(1));
    }

    #[test]
    fn test_is_botlevel() {
        let system = DungeonSystem::new();

        // Main dungeon has 29 levels
        assert!(is_botlevel(&system, &DLevel::new(0, 29)));
        assert!(!is_botlevel(&system, &DLevel::new(0, 15)));

        // Gehennom has 20 levels
        assert!(is_botlevel(&system, &DLevel::new(1, 20)));
        assert!(!is_botlevel(&system, &DLevel::new(1, 10)));
    }

    #[test]
    fn test_in_quest() {
        assert!(in_quest(&DLevel::new(4, 1)));
        assert!(in_quest(&DLevel::new(4, 5)));
        assert!(!in_quest(&DLevel::new(0, 1)));
    }

    #[test]
    fn test_in_mines() {
        assert!(in_mines(&DLevel::new(2, 1)));
        assert!(in_mines(&DLevel::new(2, 8)));
        assert!(!in_mines(&DLevel::new(0, 1)));
    }

    #[test]
    fn test_in_hell() {
        assert!(in_hell(&DLevel::new(1, 1)));
        assert!(in_hell(&DLevel::new(1, 20)));
        assert!(!in_hell(&DLevel::new(0, 1)));
    }

    #[test]
    fn test_in_v_tower() {
        assert!(in_v_tower(&DLevel::new(6, 1)));
        assert!(in_v_tower(&DLevel::new(6, 3)));
        assert!(!in_v_tower(&DLevel::new(0, 1)));
    }

    #[test]
    fn test_in_endgame() {
        assert!(in_endgame(&DLevel::new(7, 1)));
        assert!(in_endgame(&DLevel::new(7, 5)));
        assert!(!in_endgame(&DLevel::new(0, 1)));
    }

    #[test]
    fn test_endgamelevelname() {
        assert_eq!(endgamelevelname(&DLevel::new(7, 1)), Some("Earth Plane"));
        assert_eq!(endgamelevelname(&DLevel::new(7, 2)), Some("Air Plane"));
        assert_eq!(endgamelevelname(&DLevel::new(7, 3)), Some("Fire Plane"));
        assert_eq!(endgamelevelname(&DLevel::new(7, 4)), Some("Water Plane"));
        assert_eq!(endgamelevelname(&DLevel::new(7, 5)), Some("Astral Plane"));
        assert_eq!(endgamelevelname(&DLevel::new(0, 1)), None);
    }

    #[test]
    fn test_describe_level() {
        let system = DungeonSystem::new();

        // Main dungeon
        assert!(describe_level(&system, &DLevel::new(0, 5)).contains("Dlvl"));

        // Endgame
        assert_eq!(describe_level(&system, &DLevel::new(7, 5)), "Astral Plane");
    }

    #[test]
    fn test_no_bones_level() {
        // Endgame has no bones
        assert!(no_bones_level(&DLevel::new(7, 1)));
        assert!(no_bones_level(&DLevel::new(7, 5)));

        // Normal levels can have bones
        assert!(!no_bones_level(&DLevel::new(0, 10)));
    }

    #[test]
    fn test_final_level() {
        let system = DungeonSystem::new();

        // Main dungeon final level
        let final_lev = final_level(&system, 0);
        assert!(final_lev.is_some());
        assert_eq!(final_lev.unwrap().level_num, 29);

        // Gehennom final level
        let final_lev = final_level(&system, 1);
        assert!(final_lev.is_some());
        assert_eq!(final_lev.unwrap().level_num, 20);
    }
}
