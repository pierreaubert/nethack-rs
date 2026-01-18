//! Bones file system (bones.c)
//!
//! Implements saving and loading of bones files - levels where previous
//! characters died, which can be encountered by future characters.
//!
//! In NetHack, when a character dies, their level may be saved as a "bones"
//! file. Future characters can encounter this level, complete with:
//! - The ghost of the dead character
//! - Their possessions (possibly cursed)
//! - Monsters that were present
//!
//! Bones files add variety and a sense of persistence to the game.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::level::Level;
use super::DLevel;
use crate::monster::{Monster, MonsterId, MonsterState};
use crate::object::{BucStatus, Object};
use crate::rng::GameRng;

/// Bones file format version
pub const BONES_VERSION: u32 = 1;

/// Monster type index for ghost (from nh-data)
const PM_GHOST: i16 = 332; // MonsterType::Ghost equivalent

/// Bones file header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonesHeader {
    /// Version of the bones file format
    pub version: u32,
    /// Character name who died here
    pub player_name: String,
    /// Character role
    pub role: String,
    /// Character race
    pub race: String,
    /// Dungeon level where death occurred
    pub dlevel: DLevel,
    /// Cause of death
    pub death_reason: String,
    /// Turn count at death
    pub turn_count: u32,
    /// Experience level at death
    pub exp_level: u8,
    /// Gold the player had
    pub gold: i64,
    /// Maximum HP at death
    pub max_hp: i32,
}

impl BonesHeader {
    pub fn new(
        player_name: String,
        role: String,
        race: String,
        dlevel: DLevel,
        death_reason: String,
        turn_count: u32,
        exp_level: u8,
        gold: i64,
        max_hp: i32,
    ) -> Self {
        Self {
            version: BONES_VERSION,
            player_name,
            role,
            race,
            dlevel,
            death_reason,
            turn_count,
            exp_level,
            gold,
            max_hp,
        }
    }
}

/// Complete bones file data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonesFile {
    /// Header with metadata
    pub header: BonesHeader,
    /// The level data (sanitized for bones)
    pub level: Level,
}

impl BonesFile {
    /// Create a new bones file from a level and death info
    pub fn new(header: BonesHeader, level: Level) -> Self {
        Self { header, level }
    }

    /// Get the standard bones filename for a level
    /// Format: bonDL.dat where D=dungeon number, L=level number
    pub fn filename(dlevel: &DLevel) -> String {
        format!("bon{}{}.dat", dlevel.dungeon_num, dlevel.level_num)
    }

    /// Get the bones directory path
    pub fn bones_dir() -> PathBuf {
        // Use current directory with bones subfolder
        PathBuf::from("bones")
    }

    /// Get the full path for a bones file
    pub fn path(dlevel: &DLevel) -> PathBuf {
        Self::bones_dir().join(Self::filename(dlevel))
    }

    /// Check if a bones file exists for a level
    pub fn exists(dlevel: &DLevel) -> bool {
        Self::path(dlevel).exists()
    }

    /// Validate bones file version compatibility
    pub fn is_compatible(&self) -> bool {
        self.header.version == BONES_VERSION
    }
}

/// Bones manager for handling bones file operations during gameplay
#[derive(Debug, Default)]
pub struct BonesManager {
    /// Whether bones files are enabled
    pub enabled: bool,
    /// Probability of loading bones (1 in N chance)
    pub load_chance: u32,
}

impl BonesManager {
    pub fn new() -> Self {
        Self {
            enabled: true,
            load_chance: 3, // 1 in 3 chance to load bones
        }
    }

    /// Check if we should try to load bones for this level
    pub fn should_load_bones(&self, rng: &mut GameRng) -> bool {
        self.enabled && rng.one_in(self.load_chance)
    }

    /// Check if bones should be saved for this level
    /// Based on NetHack's can_make_bones()
    pub fn should_save_bones(&self, dlevel: &DLevel) -> bool {
        if !self.enabled {
            return false;
        }
        // Don't save bones for:
        // - Endgame levels (dungeon 7+)
        // - Quest levels (dungeon 3)
        // - Fort Ludios (dungeon 4)
        // - Vlad's Tower (dungeon 5)
        // - Sokoban (dungeon 6)
        (0..=2).contains(&dlevel.dungeon_num) // Main dungeon, Mines, Gehennom
    }

    /// Create bones data for saving (actual I/O handled by caller)
    /// This prepares the level for bones by:
    /// 1. Creating a ghost of the dead player
    /// 2. Sanitizing objects (curse some, remove artifacts)
    /// 3. Removing tame monsters
    /// 4. Resetting monster hostility
    pub fn create_bones(
        &self,
        mut level: Level,
        player_name: &str,
        role: &str,
        race: &str,
        death_reason: &str,
        turn_count: u32,
        exp_level: u8,
        gold: i64,
        max_hp: i32,
        player_x: i8,
        player_y: i8,
        inventory: &[Object],
        rng: &mut GameRng,
    ) -> Option<BonesFile> {
        if !self.should_save_bones(&level.dlevel) {
            return None;
        }

        // Sanitize the level for bones
        sanitize_level_for_bones(&mut level, rng);

        // Create the player's ghost
        let ghost = create_ghost(player_name, exp_level, max_hp, player_x, player_y);
        level.add_monster(ghost);

        // Drop player inventory on the level (sanitized)
        drop_bones_inventory(&mut level, inventory, player_x, player_y, rng);

        let header = BonesHeader::new(
            player_name.to_string(),
            role.to_string(),
            race.to_string(),
            level.dlevel,
            death_reason.to_string(),
            turn_count,
            exp_level,
            gold,
            max_hp,
        );

        Some(BonesFile::new(header, level))
    }

    /// Process a loaded bones level
    /// Adjusts the level for the new game
    pub fn process_loaded_bones(&self, level: &mut Level, rng: &mut GameRng) {
        // Wake up sleeping monsters
        for monster in &mut level.monsters {
            if monster.state.sleeping {
                // 50% chance to wake up
                if rng.one_in(2) {
                    monster.state.sleeping = false;
                }
            }
        }

        // Mark level as having bones
        level.flags.wizard_bones = true;
    }
}

/// Create a ghost monster for the dead player
fn create_ghost(name: &str, exp_level: u8, max_hp: i32, x: i8, y: i8) -> Monster {
    let mut ghost = Monster::new(MonsterId::NONE, PM_GHOST, x, y);
    ghost.name = format!("ghost of {}", name);
    ghost.level = exp_level;
    // Ghost HP scales with player's max HP
    ghost.hp = max_hp / 2;
    ghost.hp_max = max_hp / 2;
    ghost.state = MonsterState::active();
    ghost.state.invisible = true; // Ghosts are invisible
    ghost
}

/// Sanitize a level for bones storage
/// - Remove tame monsters
/// - Reset peaceful monsters to hostile
/// - Remove unique monsters
fn sanitize_level_for_bones(level: &mut Level, rng: &mut GameRng) {
    // Remove tame monsters and reset others
    level.monsters.retain(|m| !m.state.tame);

    for monster in &mut level.monsters {
        // Reset peaceful to hostile (they don't remember the old player)
        if monster.state.peaceful && !monster.is_shopkeeper && !monster.is_priest {
            monster.state.peaceful = false;
        }

        // Put some monsters to sleep
        if rng.one_in(3) {
            monster.state.sleeping = true;
        }
    }

    // Sanitize objects on the level
    for obj in &mut level.objects {
        sanitize_object_for_bones(obj, rng);
    }
}

/// Sanitize an object for bones
/// - Curse some items
/// - Remove artifact status
/// - Reset identification
fn sanitize_object_for_bones(obj: &mut Object, rng: &mut GameRng) {
    // 1 in 4 chance to curse the item
    if rng.one_in(4) {
        obj.buc = BucStatus::Cursed;
    }

    // Remove artifact status (artifacts are unique)
    obj.artifact = 0;

    // Reset identification
    obj.known = false;
    obj.buc_known = false;

    // Recursively sanitize container contents
    for contained in &mut obj.contents {
        sanitize_object_for_bones(contained, rng);
    }
}

/// Drop player inventory onto the bones level
fn drop_bones_inventory(level: &mut Level, inventory: &[Object], x: i8, y: i8, rng: &mut GameRng) {
    for obj in inventory {
        let mut bones_obj = obj.clone();
        sanitize_object_for_bones(&mut bones_obj, rng);
        level.add_object(bones_obj, x, y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bones_filename() {
        assert_eq!(BonesFile::filename(&DLevel::new(0, 5)), "bon05.dat");
        assert_eq!(BonesFile::filename(&DLevel::new(1, 10)), "bon110.dat");
        assert_eq!(BonesFile::filename(&DLevel::new(2, 3)), "bon23.dat");
    }

    #[test]
    fn test_bones_header() {
        let header = BonesHeader::new(
            "TestPlayer".to_string(),
            "Valkyrie".to_string(),
            "Human".to_string(),
            DLevel::new(0, 5),
            "killed by a gnome".to_string(),
            1234,
            8,
            500,  // gold
            50,   // max_hp
        );

        assert_eq!(header.version, BONES_VERSION);
        assert_eq!(header.player_name, "TestPlayer");
        assert_eq!(header.dlevel.level_num, 5);
        assert_eq!(header.gold, 500);
        assert_eq!(header.max_hp, 50);
    }

    #[test]
    fn test_bones_manager() {
        let manager = BonesManager::new();
        assert!(manager.enabled);
        assert_eq!(manager.load_chance, 3);
    }

    #[test]
    fn test_bones_path() {
        let path = BonesFile::path(&DLevel::new(0, 5));
        assert!(path.to_string_lossy().contains("bon05.dat"));
    }

    #[test]
    fn test_should_save_bones() {
        let manager = BonesManager::new();

        // Main dungeon - should save
        assert!(manager.should_save_bones(&DLevel::new(0, 5)));

        // Mines - should save
        assert!(manager.should_save_bones(&DLevel::new(1, 3)));

        // Gehennom - should save
        assert!(manager.should_save_bones(&DLevel::new(2, 10)));

        // Quest - should not save
        assert!(!manager.should_save_bones(&DLevel::new(3, 1)));

        // Endgame - should not save
        assert!(!manager.should_save_bones(&DLevel::new(7, 1)));
    }

    #[test]
    fn test_bones_file_compatibility() {
        let header = BonesHeader::new(
            "Test".to_string(),
            "Wizard".to_string(),
            "Elf".to_string(),
            DLevel::new(0, 3),
            "killed by a newt".to_string(),
            100,
            5,
            100,
            30,
        );
        let level = Level::new(DLevel::new(0, 3));
        let bones = BonesFile::new(header, level);

        assert!(bones.is_compatible());
    }

    #[test]
    fn test_create_ghost() {
        let ghost = create_ghost("Hero", 10, 100, 5, 5);

        assert_eq!(ghost.name, "ghost of Hero");
        assert_eq!(ghost.level, 10);
        assert_eq!(ghost.hp, 50); // Half of max_hp
        assert_eq!(ghost.hp_max, 50);
        assert!(ghost.state.invisible);
        assert_eq!(ghost.x, 5);
        assert_eq!(ghost.y, 5);
    }

    #[test]
    fn test_sanitize_object() {
        let mut rng = GameRng::new(42);
        let mut obj = Object::default();
        obj.artifact = 5;
        obj.known = true;
        obj.buc_known = true;

        sanitize_object_for_bones(&mut obj, &mut rng);

        assert_eq!(obj.artifact, 0);
        assert!(!obj.known);
        assert!(!obj.buc_known);
    }
}
