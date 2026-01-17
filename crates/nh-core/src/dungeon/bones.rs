//! Bones file system (bones.c)
//!
//! Implements saving and loading of bones files - levels where previous
//! characters died, which can be encountered by future characters.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::level::Level;
use super::DLevel;

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
    ) -> Self {
        Self {
            version: 1,
            player_name,
            role,
            race,
            dlevel,
            death_reason,
            turn_count,
            exp_level,
        }
    }
}

/// Complete bones file data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonesFile {
    /// Header with metadata
    pub header: BonesHeader,
    /// The level data
    pub level: Level,
}

impl BonesFile {
    /// Create a new bones file from a level and death info
    pub fn new(header: BonesHeader, level: Level) -> Self {
        Self { header, level }
    }

    /// Get the standard bones filename for a level
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
    pub fn should_load_bones(&self, rng: &mut crate::rng::GameRng) -> bool {
        self.enabled && rng.one_in(self.load_chance)
    }

    /// Check if bones should be saved for this level
    pub fn should_save_bones(&self, dlevel: &DLevel) -> bool {
        if !self.enabled {
            return false;
        }
        // Don't save bones for special levels or endgame
        dlevel.dungeon_num < 7
    }

    /// Create bones data for saving (actual I/O handled by caller)
    pub fn create_bones(
        &self,
        level: Level,
        player_name: &str,
        role: &str,
        race: &str,
        death_reason: &str,
        turn_count: u32,
        exp_level: u8,
    ) -> Option<BonesFile> {
        if !self.should_save_bones(&level.dlevel) {
            return None;
        }

        let header = BonesHeader::new(
            player_name.to_string(),
            role.to_string(),
            race.to_string(),
            level.dlevel,
            death_reason.to_string(),
            turn_count,
            exp_level,
        );

        Some(BonesFile::new(header, level))
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
        );

        assert_eq!(header.version, 1);
        assert_eq!(header.player_name, "TestPlayer");
        assert_eq!(header.dlevel.level_num, 5);
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
        
        // Gehennom - should save
        assert!(manager.should_save_bones(&DLevel::new(1, 10)));
        
        // Endgame - should not save
        assert!(!manager.should_save_bones(&DLevel::new(7, 1)));
    }
}
