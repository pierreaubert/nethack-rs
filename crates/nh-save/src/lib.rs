//! nh-save: Save/restore system for NetHack clone
//!
//! Handles saving and loading game state.

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use nh_core::GameState;

/// Current save file format version
pub const SAVE_VERSION: u32 = 1;

/// Save/restore errors
#[derive(Debug, Error)]
pub enum SaveError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Save file not found")]
    NotFound,

    #[error("Save file corrupted")]
    Corrupted,

    #[error("Incompatible save version: expected {expected}, found {found}")]
    IncompatibleVersion { expected: u32, found: u32 },

    #[error("Invalid save file header")]
    InvalidHeader,
}

/// Save file header for versioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveHeader {
    /// Magic identifier
    pub magic: String,
    /// Save format version
    pub version: u32,
    /// Player name
    pub player_name: String,
    /// Turn count at save time
    pub turns: u64,
    /// Dungeon level at save time
    pub dlevel: String,
    /// Timestamp of save
    pub timestamp: u64,
}

impl SaveHeader {
    const MAGIC: &'static str = "NHRS";

    pub fn new(state: &GameState) -> Self {
        Self {
            magic: Self::MAGIC.to_string(),
            version: SAVE_VERSION,
            player_name: state.player.name.clone(),
            turns: state.turns,
            dlevel: format!("{}", state.current_level.dlevel),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    pub fn validate(&self) -> Result<(), SaveError> {
        if self.magic != Self::MAGIC {
            return Err(SaveError::InvalidHeader);
        }
        if self.version != SAVE_VERSION {
            return Err(SaveError::IncompatibleVersion {
                expected: SAVE_VERSION,
                found: self.version,
            });
        }
        Ok(())
    }
}

/// Complete save file structure
#[derive(Serialize, Deserialize)]
pub struct SaveFile {
    pub header: SaveHeader,
    pub state: GameState,
}

/// Save game state to a file
pub fn save_game(state: &GameState, path: impl AsRef<Path>) -> Result<(), SaveError> {
    let json = serde_json::to_string_pretty(state)?;
    let save_file = SaveFile {
        header: SaveHeader::new(state),
        state: serde_json::from_str(&json)?,
    };

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &save_file)?;
    Ok(())
}

/// Save game state to a compact file (no pretty printing)
pub fn save_game_compact(state: &GameState, path: impl AsRef<Path>) -> Result<(), SaveError> {
    let json = serde_json::to_string(state)?;
    let save_file = SaveFile {
        header: SaveHeader::new(state),
        state: serde_json::from_str(&json)?,
    };

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &save_file)?;
    Ok(())
}

/// Load game state from a file
pub fn load_game(path: impl AsRef<Path>) -> Result<GameState, SaveError> {
    let file = File::open(path).map_err(|_| SaveError::NotFound)?;
    let reader = BufReader::new(file);
    let save_file: SaveFile = serde_json::from_reader(reader)?;

    save_file.header.validate()?;
    Ok(save_file.state)
}

/// Load only the header from a save file (for save game browser)
pub fn load_header(path: impl AsRef<Path>) -> Result<SaveHeader, SaveError> {
    let file = File::open(path).map_err(|_| SaveError::NotFound)?;
    let reader = BufReader::new(file);

    // Read just enough to get the header
    // This is a simplified approach - reads the whole file but only parses header
    let save_file: SaveFile = serde_json::from_reader(reader)?;
    save_file.header.validate()?;
    Ok(save_file.header)
}

/// Check if a save file exists
pub fn save_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

/// Delete a save file
pub fn delete_save(path: impl AsRef<Path>) -> Result<(), SaveError> {
    std::fs::remove_file(path)?;
    Ok(())
}

/// Get the default save path for a player name
pub fn default_save_path(player_name: &str) -> std::path::PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push("nethack-rs");
    path.push("saves");
    std::fs::create_dir_all(&path).ok();
    path.push(format!("{}.json", player_name));
    path
}

/// Get the bones file path for a level
pub fn bones_path(dlevel: &str) -> std::path::PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push("nethack-rs");
    path.push("bones");
    std::fs::create_dir_all(&path).ok();
    path.push(format!("bones.{}.json", dlevel));
    path
}

/// List all save files in the default save directory
pub fn list_saves() -> Result<Vec<(std::path::PathBuf, SaveHeader)>, SaveError> {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push("nethack-rs");
    path.push("saves");

    if !path.exists() {
        return Ok(Vec::new());
    }

    let mut saves = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(header) = load_header(&path) {
                saves.push((path, header));
            }
        }
    }

    // Sort by timestamp, newest first
    saves.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));
    Ok(saves)
}

/// Bones file data - contains dead player's remains
#[derive(Debug, Serialize, Deserialize)]
pub struct BonesFile {
    /// Player name who died
    pub player_name: String,
    /// How they died
    pub death_reason: String,
    /// Level data with corpse and items
    pub level: nh_core::dungeon::Level,
    /// Ghost monster data (if applicable)
    pub ghost: Option<nh_core::monster::Monster>,
}

/// Save bones file for a dead player
pub fn save_bones(
    player_name: &str,
    death_reason: &str,
    level: &nh_core::dungeon::Level,
    ghost: Option<&nh_core::monster::Monster>,
) -> Result<(), SaveError> {
    let bones = BonesFile {
        player_name: player_name.to_string(),
        death_reason: death_reason.to_string(),
        level: level.clone(),
        ghost: ghost.cloned(),
    };

    let path = bones_path(&format!("{}", level.dlevel));
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &bones)?;
    Ok(())
}

/// Load bones file for a level (if it exists)
pub fn load_bones(dlevel: &str) -> Result<Option<BonesFile>, SaveError> {
    let path = bones_path(dlevel);
    if !path.exists() {
        return Ok(None);
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let bones: BonesFile = serde_json::from_reader(reader)?;

    // Delete bones file after loading (one-time use)
    delete_save(bones_path(dlevel)).ok();

    Ok(Some(bones))
}

/// Create an emergency save (panic save)
pub fn emergency_save(state: &GameState) -> Result<std::path::PathBuf, SaveError> {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push("nethack-rs");
    path.push("saves");
    std::fs::create_dir_all(&path).ok();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    path.push(format!("emergency_{}.json", timestamp));
    save_game(state, &path)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nh_core::GameState;

    #[test]
    fn test_save_and_load() {
        let path = std::env::temp_dir().join("nethack_test_save.json");

        let state = GameState::default();
        save_game(&state, &path).unwrap();

        assert!(path.exists());

        let loaded = load_game(&path).unwrap();
        assert_eq!(loaded.turns, state.turns);

        // Cleanup
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_header_validation() {
        let state = GameState::default();
        let header = SaveHeader::new(&state);

        assert!(header.validate().is_ok());

        let mut bad_header = header.clone();
        bad_header.magic = "XXXX".to_string();
        assert!(matches!(
            bad_header.validate(),
            Err(SaveError::InvalidHeader)
        ));

        let mut old_header = header;
        old_header.version = 999;
        assert!(matches!(
            old_header.validate(),
            Err(SaveError::IncompatibleVersion { .. })
        ));
    }

    #[test]
    fn test_load_nonexistent() {
        let result = load_game("/nonexistent/path/save.json");
        assert!(matches!(result, Err(SaveError::NotFound)));
    }
}
