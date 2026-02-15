//! MapSeen - Player's memory of dungeon levels (dungeon overview)
//!
//! This module implements the dungeon overview feature, which tracks what the player
//! remembers about each visited level. This allows display of a dungeon map without
//! requiring the player to take notes.
//!
//! Corresponds to NetHack's mapseen structure in dungeon.h

#[cfg(not(feature = "std"))]
use crate::compat::*;

use serde::{Deserialize, Serialize};

use super::DLevel;
use super::room::RoomType;
use super::topology::{Branch, DungeonSystem};

/// Maximum number of rooms tracked per level
pub const MAXNROFROOMS: usize = 40;

/// Mapseen alignment - special handling for altar alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum MapseenAlignment {
    /// No alignment or multiple non-matching alignments
    #[default]
    None = 0,
    /// Lawful
    Lawful = 1,
    /// Neutral
    Neutral = 2,
    /// Chaotic
    Chaotic = 3,
}

impl MapseenAlignment {
    /// Convert from alignment value (-1, 0, 1) to mapseen alignment
    pub fn from_alignment(align: i8) -> Self {
        match align {
            1 => Self::Lawful,
            0 => Self::Neutral,
            -1 => Self::Chaotic,
            _ => Self::None,
        }
    }
}

/// Feature knowledge from the map
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapseenFeatures {
    /// Number of fountains (0-3 encoded)
    pub fountains: u8,
    /// Number of sinks (0-3 encoded)
    pub sinks: u8,
    /// Number of altars (0-3 encoded)
    pub altars: u8,
    /// Number of thrones (0-3 encoded)
    pub thrones: u8,
    /// Number of graves (0-3 encoded)
    pub graves: u8,
    /// Number of trees (0-3 encoded)
    pub trees: u8,
    /// Has water (0-3: none, some, lots)
    pub water: u8,
    /// Has lava (0-3: none, some, lots)
    pub lava: u8,
    /// Has ice (0-3: none, some, lots)
    pub ice: u8,
    /// Number of shops (0-3 encoded)
    pub shops: u8,
    /// Number of temples (0-3 encoded)
    pub temples: u8,
    /// Altar alignment (if single type)
    pub altar_align: MapseenAlignment,
    /// Shop type if single shop
    pub shop_type: Option<RoomType>,
}

impl MapseenFeatures {
    /// Encode a count as 0-3 (0, 1, 2, 3+)
    fn encode_count(count: usize) -> u8 {
        match count {
            0 => 0,
            1 => 1,
            2 => 2,
            _ => 3,
        }
    }

    /// Update fountain count
    pub fn set_fountains(&mut self, count: usize) {
        self.fountains = Self::encode_count(count);
    }

    /// Update sink count
    pub fn set_sinks(&mut self, count: usize) {
        self.sinks = Self::encode_count(count);
    }

    /// Update altar count
    pub fn set_altars(&mut self, count: usize, align: Option<i8>) {
        self.altars = Self::encode_count(count);
        if count == 1 {
            self.altar_align = align
                .map(MapseenAlignment::from_alignment)
                .unwrap_or(MapseenAlignment::None);
        } else {
            self.altar_align = MapseenAlignment::None;
        }
    }

    /// Update throne count
    pub fn set_thrones(&mut self, count: usize) {
        self.thrones = Self::encode_count(count);
    }

    /// Update grave count
    pub fn set_graves(&mut self, count: usize) {
        self.graves = Self::encode_count(count);
    }

    /// Update tree count
    pub fn set_trees(&mut self, count: usize) {
        self.trees = Self::encode_count(count);
    }

    /// Update shop count
    pub fn set_shops(&mut self, count: usize, shop_type: Option<RoomType>) {
        self.shops = Self::encode_count(count);
        if count == 1 {
            self.shop_type = shop_type;
        } else {
            self.shop_type = None;
        }
    }

    /// Update temple count
    pub fn set_temples(&mut self, count: usize) {
        self.temples = Self::encode_count(count);
    }
}

/// Mapseen flags - level characteristics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapseenFlags {
    /// Level is unreachable (no way back)
    pub unreachable: bool,
    /// Player has forgotten about this level (amnesia)
    pub forgot: bool,
    /// Player knows about bones on this level
    pub knownbones: bool,
    /// Oracle is on this level
    pub oracle: bool,
    /// Sokoban was solved
    pub sokosolved: bool,
    /// Big room level
    pub bigroom: bool,
    /// Castle (stronghold) level
    pub castle: bool,
    /// Castle tune hint should be shown
    pub castletune: bool,
    /// Valley of the Dead
    pub valley: bool,
    /// Moloch's Sanctum
    pub sanctum: bool,
    /// Fort Ludios
    pub ludios: bool,
    /// Rogue level
    pub roguelevel: bool,
    /// Quest summons heard (for entry level)
    pub quest_summons: bool,
    /// Quest is unlocked (for home level)
    pub questing: bool,
}

/// Room information for mapseen
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapseenRoom {
    /// Room has been seen
    pub seen: bool,
    /// Shop without shopkeeper (looted/dead)
    pub untended: bool,
}

/// What the player knows about a single dungeon level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapSeen {
    /// The dungeon level this refers to
    pub lev: DLevel,
    /// Branch taken from this level (if any)
    pub branch_id: Option<i32>,
    /// Features seen on this level
    pub feat: MapseenFeatures,
    /// Level flags
    pub flags: MapseenFlags,
    /// Custom annotation by player
    pub custom: Option<String>,
    /// Room information
    pub rooms: Vec<MapseenRoom>,
}

impl MapSeen {
    /// Create a new MapSeen for a level
    pub fn new(lev: DLevel) -> Self {
        let mut rooms = Vec::with_capacity(MAXNROFROOMS * 2);
        rooms.resize_with(MAXNROFROOMS * 2, MapseenRoom::default);

        Self {
            lev,
            branch_id: None,
            feat: MapseenFeatures::default(),
            flags: MapseenFlags::default(),
            custom: None,
            rooms,
        }
    }

    /// Set custom annotation
    pub fn set_custom(&mut self, annotation: Option<String>) {
        self.custom = annotation;
    }

    /// Get custom annotation
    pub fn custom(&self) -> Option<&str> {
        self.custom.as_deref()
    }

    /// Record that a branch was taken from this level
    pub fn set_branch(&mut self, branch_id: i32) {
        self.branch_id = Some(branch_id);
    }

    /// Mark a room as seen
    pub fn mark_room_seen(&mut self, room_index: usize) {
        if room_index < self.rooms.len() {
            self.rooms[room_index].seen = true;
        }
    }

    /// Mark a room as untended (shop without keeper)
    pub fn mark_room_untended(&mut self, room_index: usize) {
        if room_index < self.rooms.len() {
            self.rooms[room_index].untended = true;
        }
    }

    /// Check if level has any interesting features
    pub fn is_interesting(&self) -> bool {
        // Custom annotation
        if self.custom.is_some() {
            return true;
        }

        // Special level flags
        if self.flags.oracle
            || self.flags.bigroom
            || self.flags.castle
            || self.flags.valley
            || self.flags.sanctum
            || self.flags.ludios
            || self.flags.roguelevel
        {
            return true;
        }

        // Branch
        if self.branch_id.is_some() {
            return true;
        }

        // Features
        if self.feat.altars > 0 || self.feat.shops > 0 || self.feat.temples > 0 {
            return true;
        }

        false
    }
}

/// Chain of all mapseen records
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MapSeenChain {
    /// All mapseen records
    pub entries: Vec<MapSeen>,
}

impl MapSeenChain {
    /// Create a new empty chain
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Initialize mapseen for a level (init_mapseen equivalent)
    pub fn init_mapseen(&mut self, lev: DLevel) {
        // Check if already exists
        if self.find(&lev).is_some() {
            return;
        }
        self.entries.push(MapSeen::new(lev));
    }

    /// Find mapseen for a level (find_mapseen equivalent)
    pub fn find(&self, lev: &DLevel) -> Option<&MapSeen> {
        self.entries.iter().find(|m| m.lev == *lev)
    }

    /// Find mapseen for a level (mutable)
    pub fn find_mut(&mut self, lev: &DLevel) -> Option<&mut MapSeen> {
        self.entries.iter_mut().find(|m| m.lev == *lev)
    }

    /// Find mapseen by custom annotation (find_mapseen_by_str equivalent)
    pub fn find_by_custom(&self, annotation: &str) -> Option<&MapSeen> {
        self.entries.iter().find(|m| {
            m.custom
                .as_ref()
                .map(|c| c.eq_ignore_ascii_case(annotation))
                .unwrap_or(false)
        })
    }

    /// Remove mapseen for a level (rm_mapseen equivalent)
    pub fn remove(&mut self, lev: &DLevel) {
        self.entries.retain(|m| m.lev != *lev);
    }

    /// Remove mapseen by ledger number (remdun_mapseen equivalent)
    pub fn remove_by_ledger(&mut self, dungeon_system: &DungeonSystem, ledger_num: i32) {
        self.entries.retain(|m| {
            let m_ledger = dungeon_system
                .dungeons
                .get(m.lev.dungeon_num as usize)
                .map(|d| d.ledger_start + m.lev.level_num as i32)
                .unwrap_or(0);
            m_ledger != ledger_num
        });
    }

    /// Forget a level (for amnesia)
    pub fn forget_mapseen(&mut self, lev: &DLevel) {
        if let Some(mapseen) = self.find_mut(lev) {
            mapseen.flags.forgot = true;
            mapseen.branch_id = None;
            mapseen.custom = None;
            for room in &mut mapseen.rooms {
                room.seen = false;
                room.untended = false;
            }
        }
    }

    /// Record branch taken from a level (recbranch_mapseen equivalent)
    pub fn record_branch(
        &mut self,
        source: &DLevel,
        dest: &DLevel,
        dungeon_system: &DungeonSystem,
    ) {
        // Not a branch if same dungeon
        if source.dungeon_num == dest.dungeon_num {
            return;
        }

        // Find the branch
        for branch in &dungeon_system.branches {
            if branch.end1 == *source && branch.end2 == *dest {
                if let Some(mapseen) = self.find_mut(source) {
                    mapseen.set_branch(branch.id);
                }
                return;
            }
            // Reverse branch - don't record
            if branch.end2 == *source && branch.end1 == *dest {
                return;
            }
        }
    }

    /// Get annotation for a level (get_annotation equivalent)
    pub fn get_annotation(&self, lev: &DLevel) -> Option<&str> {
        self.find(lev).and_then(|m| m.custom())
    }

    /// Set annotation for a level
    pub fn set_annotation(&mut self, lev: &DLevel, annotation: Option<String>) {
        if let Some(mapseen) = self.find_mut(lev) {
            mapseen.set_custom(annotation);
        }
    }

    /// Traverse all mapseen entries
    pub fn traverse<F>(&self, mut f: F)
    where
        F: FnMut(&MapSeen),
    {
        for mapseen in &self.entries {
            f(mapseen);
        }
    }

    /// Check if a room type was discovered on any level (room_discovered equivalent)
    pub fn room_discovered(&self, room_type: RoomType) -> bool {
        for mapseen in &self.entries {
            if mapseen.flags.forgot {
                continue;
            }
            // Check for specific room types
            match room_type {
                // Any shop type
                RoomType::GeneralShop
                | RoomType::ArmorShop
                | RoomType::ScrollShop
                | RoomType::PotionShop
                | RoomType::WeaponShop
                | RoomType::FoodShop
                | RoomType::RingShop
                | RoomType::WandShop
                | RoomType::ToolShop
                | RoomType::BookShop
                | RoomType::HealthFoodShop
                | RoomType::CandleShop => {
                    if mapseen.feat.shops > 0 {
                        return true;
                    }
                }
                RoomType::Temple => {
                    if mapseen.feat.temples > 0 {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Check if any shop was discovered on any level
    pub fn shop_discovered(&self) -> bool {
        for mapseen in &self.entries {
            if !mapseen.flags.forgot && mapseen.feat.shops > 0 {
                return true;
            }
        }
        false
    }

    /// Check if any temple was discovered on any level
    pub fn temple_discovered(&self) -> bool {
        for mapseen in &self.entries {
            if !mapseen.flags.forgot && mapseen.feat.temples > 0 {
                return true;
            }
        }
        false
    }
}

/// Format level description for mapseen display (print_mapseen equivalent)
pub fn format_mapseen(mapseen: &MapSeen, dungeon_system: &DungeonSystem, show_all: bool) -> String {
    let mut output = String::new();

    // Get dungeon name
    let dungeon_name = dungeon_system
        .dungeons
        .get(mapseen.lev.dungeon_num as usize)
        .map(|d| d.name.as_str())
        .unwrap_or("Unknown");

    // Level identifier
    output.push_str(&format!("{} level {}", dungeon_name, mapseen.lev.level_num));

    // Special level flags
    if mapseen.flags.oracle {
        output.push_str(", oracle");
    }
    if mapseen.flags.bigroom {
        output.push_str(", big room");
    }
    if mapseen.flags.castle {
        output.push_str(", castle");
        if mapseen.flags.castletune {
            output.push_str(" (tune known)");
        }
    }
    if mapseen.flags.valley {
        output.push_str(", valley");
    }
    if mapseen.flags.sanctum {
        output.push_str(", sanctum");
    }
    if mapseen.flags.ludios {
        output.push_str(", ludios");
    }
    if mapseen.flags.roguelevel {
        output.push_str(", rogue level");
    }
    if mapseen.flags.sokosolved {
        output.push_str(", sokoban solved");
    }

    // Features
    if show_all || mapseen.feat.altars > 0 {
        if mapseen.feat.altars == 1 {
            let align_str = match mapseen.feat.altar_align {
                MapseenAlignment::Lawful => "lawful",
                MapseenAlignment::Neutral => "neutral",
                MapseenAlignment::Chaotic => "chaotic",
                MapseenAlignment::None => "",
            };
            if !align_str.is_empty() {
                output.push_str(&format!(", {} altar", align_str));
            } else {
                output.push_str(", altar");
            }
        } else if mapseen.feat.altars > 1 {
            output.push_str(", altars");
        }
    }

    if show_all || mapseen.feat.shops > 0 {
        if mapseen.feat.shops == 1 {
            output.push_str(", shop");
        } else if mapseen.feat.shops > 1 {
            output.push_str(", shops");
        }
    }

    if show_all || mapseen.feat.temples > 0 {
        if mapseen.feat.temples == 1 {
            output.push_str(", temple");
        } else if mapseen.feat.temples > 1 {
            output.push_str(", temples");
        }
    }

    if show_all || mapseen.feat.fountains > 0 {
        if mapseen.feat.fountains == 1 {
            output.push_str(", fountain");
        } else if mapseen.feat.fountains > 1 {
            output.push_str(", fountains");
        }
    }

    // Branch
    if let Some(branch_id) = mapseen.branch_id {
        if let Some(branch) = dungeon_system.branches.iter().find(|b| b.id == branch_id) {
            let dest_name = dungeon_system
                .dungeons
                .get(branch.end2.dungeon_num as usize)
                .map(|d| d.name.as_str())
                .unwrap_or("Unknown");
            output.push_str(&format!(", branch to {}", dest_name));
        }
    }

    // Custom annotation
    if let Some(custom) = &mapseen.custom {
        output.push_str(&format!(" \"{}\"", custom));
    }

    // Status
    if mapseen.flags.forgot {
        output.push_str(" (forgotten)");
    }
    if mapseen.flags.unreachable {
        output.push_str(" (unreachable)");
    }

    output
}

/// Check if mapseen is interesting enough to display (interest_mapseen equivalent)
pub fn interest_mapseen(mapseen: &MapSeen) -> bool {
    mapseen.is_interesting()
}

// ============================================================================
// Serialization functions
// ============================================================================

/// Save mapseen chain to JSON
#[cfg(feature = "std")]
pub fn save_mapseen(chain: &MapSeenChain) -> Result<String, String> {
    serde_json::to_string(chain).map_err(|e| format!("Failed to save mapseen: {}", e))
}

/// Load mapseen chain from JSON
#[cfg(feature = "std")]
pub fn load_mapseen(data: &str) -> Result<MapSeenChain, String> {
    serde_json::from_str(data).map_err(|e| format!("Failed to load mapseen: {}", e))
}

// ============================================================================
// Temple tracking (mapseen_temple equivalent)
// ============================================================================

/// Record temple discovery on a level
pub fn mapseen_temple(chain: &mut MapSeenChain, lev: &DLevel, align: i8) {
    if let Some(mapseen) = chain.find_mut(lev) {
        let current = mapseen.feat.temples as usize;
        mapseen.feat.set_temples(current + 1);
        // Update altar alignment if this is the first altar
        if mapseen.feat.altars == 0 {
            mapseen.feat.set_altars(1, Some(align));
        }
    }
}

/// Recalculate mapseen from level data (recalc_mapseen equivalent)
///
/// This would scan the level and update the mapseen record.
/// Simplified implementation - caller should update specific features.
pub fn recalc_mapseen(
    chain: &mut MapSeenChain,
    lev: &DLevel,
    features: MapseenFeatures,
    flags: MapseenFlags,
) {
    if let Some(mapseen) = chain.find_mut(lev) {
        mapseen.feat = features;
        // Preserve some flags, update others
        mapseen.flags.oracle = flags.oracle;
        mapseen.flags.bigroom = flags.bigroom;
        mapseen.flags.castle = flags.castle;
        mapseen.flags.castletune = flags.castletune;
        mapseen.flags.valley = flags.valley;
        mapseen.flags.sanctum = flags.sanctum;
        mapseen.flags.ludios = flags.ludios;
        mapseen.flags.roguelevel = flags.roguelevel;
        mapseen.flags.sokosolved = flags.sokosolved;
    }
}

/// Print dungeon overview (print_mapseen loop equivalent)
pub fn print_mapseen(chain: &MapSeenChain, dungeon_system: &DungeonSystem) -> String {
    let mut output = String::new();
    output.push_str("=== Dungeon Overview ===\n\n");

    let mut current_dungeon: i8 = -1;

    for mapseen in &chain.entries {
        // Skip forgotten levels
        if mapseen.flags.forgot {
            continue;
        }

        // Skip uninteresting levels unless changing dungeons
        if !mapseen.is_interesting() && mapseen.lev.dungeon_num == current_dungeon {
            continue;
        }

        // Print dungeon header if changed
        if mapseen.lev.dungeon_num != current_dungeon {
            current_dungeon = mapseen.lev.dungeon_num;
            let dungeon_name = dungeon_system
                .dungeons
                .get(current_dungeon as usize)
                .map(|d| d.name.as_str())
                .unwrap_or("Unknown");
            output.push_str(&format!("\n{}:\n", dungeon_name));
        }

        // Print level info
        let level_info = format_mapseen(mapseen, dungeon_system, false);
        output.push_str(&format!("  {}\n", level_info));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapseen_creation() {
        let lev = DLevel::new(0, 5);
        let mapseen = MapSeen::new(lev);
        assert_eq!(mapseen.lev, lev);
        assert!(mapseen.custom.is_none());
        assert!(mapseen.branch_id.is_none());
    }

    #[test]
    fn test_mapseenchain_init() {
        let mut chain = MapSeenChain::new();
        let lev = DLevel::new(0, 5);

        chain.init_mapseen(lev);
        assert!(chain.find(&lev).is_some());

        // Re-init should not duplicate
        chain.init_mapseen(lev);
        assert_eq!(chain.entries.len(), 1);
    }

    #[test]
    fn test_mapseenchain_find() {
        let mut chain = MapSeenChain::new();
        let lev1 = DLevel::new(0, 5);
        let lev2 = DLevel::new(0, 10);

        chain.init_mapseen(lev1);
        chain.init_mapseen(lev2);

        assert!(chain.find(&lev1).is_some());
        assert!(chain.find(&lev2).is_some());
        assert!(chain.find(&DLevel::new(0, 15)).is_none());
    }

    #[test]
    fn test_custom_annotation() {
        let mut chain = MapSeenChain::new();
        let lev = DLevel::new(0, 5);

        chain.init_mapseen(lev);
        chain.set_annotation(&lev, Some("test annotation".to_string()));

        assert_eq!(chain.get_annotation(&lev), Some("test annotation"));
    }

    #[test]
    fn test_find_by_custom() {
        let mut chain = MapSeenChain::new();
        let lev = DLevel::new(0, 5);

        chain.init_mapseen(lev);
        chain.set_annotation(&lev, Some("test".to_string()));

        let found = chain.find_by_custom("TEST"); // case insensitive
        assert!(found.is_some());
        assert_eq!(found.unwrap().lev, lev);

        assert!(chain.find_by_custom("nonexistent").is_none());
    }

    #[test]
    fn test_forget_mapseen() {
        let mut chain = MapSeenChain::new();
        let lev = DLevel::new(0, 5);

        chain.init_mapseen(lev);
        chain.set_annotation(&lev, Some("test".to_string()));

        chain.forget_mapseen(&lev);

        let mapseen = chain.find(&lev).unwrap();
        assert!(mapseen.flags.forgot);
        assert!(mapseen.custom.is_none());
    }

    #[test]
    fn test_remove_mapseen() {
        let mut chain = MapSeenChain::new();
        let lev = DLevel::new(0, 5);

        chain.init_mapseen(lev);
        assert!(chain.find(&lev).is_some());

        chain.remove(&lev);
        assert!(chain.find(&lev).is_none());
    }

    #[test]
    fn test_features() {
        let mut feat = MapseenFeatures::default();

        feat.set_fountains(2);
        assert_eq!(feat.fountains, 2);

        feat.set_fountains(5);
        assert_eq!(feat.fountains, 3); // Capped at 3

        feat.set_altars(1, Some(1));
        assert_eq!(feat.altars, 1);
        assert_eq!(feat.altar_align, MapseenAlignment::Lawful);
    }

    #[test]
    fn test_is_interesting() {
        let mut mapseen = MapSeen::new(DLevel::new(0, 5));

        // Plain level is not interesting
        assert!(!mapseen.is_interesting());

        // Custom annotation makes it interesting
        mapseen.custom = Some("test".to_string());
        assert!(mapseen.is_interesting());

        // Reset and check oracle flag
        mapseen.custom = None;
        mapseen.flags.oracle = true;
        assert!(mapseen.is_interesting());

        // Reset and check shops
        mapseen.flags.oracle = false;
        mapseen.feat.shops = 1;
        assert!(mapseen.is_interesting());
    }

    #[test]
    fn test_save_load_mapseen() {
        let mut chain = MapSeenChain::new();
        chain.init_mapseen(DLevel::new(0, 5));
        chain.init_mapseen(DLevel::new(0, 10));
        chain.set_annotation(&DLevel::new(0, 5), Some("test".to_string()));

        let saved = save_mapseen(&chain).expect("should save");
        let loaded = load_mapseen(&saved).expect("should load");

        assert_eq!(loaded.entries.len(), 2);
        assert_eq!(loaded.get_annotation(&DLevel::new(0, 5)), Some("test"));
    }
}
