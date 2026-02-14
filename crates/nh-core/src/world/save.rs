//! Save and restore game state (save.c, restore.c)
//!
//! Handles saving and loading game state to/from files.
//!
//! Note: Full implementation requires bincode dependency.
//! This module provides the interface and utility functions.

use serde::{Deserialize, Serialize};

/// Save file format version
pub const SAVE_VERSION: u32 = 1;

/// Save file magic bytes
pub const SAVE_MAGIC: &[u8; 4] = b"NHRS";

/// Save file header (JSON format for portability)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveHeader {
    pub magic: String,
    pub version: u32,
    pub player_name: String,
    pub save_time: u64,
    pub turns: u64,
    pub dlevel: String,
}

/// Save error types
#[derive(Debug, Clone)]
pub enum SaveError {
    IoError(String),
    SerializeError(String),
    InvalidMagic,
    VersionMismatch { expected: u32, found: u32 },
    ChecksumMismatch,
    CorruptedData(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::IoError(e) => write!(f, "IO error: {}", e),
            SaveError::SerializeError(e) => write!(f, "Serialization error: {}", e),
            SaveError::InvalidMagic => write!(f, "Invalid save file (bad magic number)"),
            SaveError::VersionMismatch { expected, found } => {
                write!(f, "Save version mismatch: expected {}, found {}", expected, found)
            }
            SaveError::ChecksumMismatch => write!(f, "Save file corrupted (checksum mismatch)"),
            SaveError::CorruptedData(e) => write!(f, "Corrupted save data: {}", e),
        }
    }
}

impl std::error::Error for SaveError {}

/// Calculate a simple checksum for data integrity
pub fn calculate_checksum(data: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    for (i, &byte) in data.iter().enumerate() {
        sum = sum.wrapping_add((byte as u32).wrapping_mul((i as u32).wrapping_add(1)));
    }
    sum
}

/// Get current timestamp as u64
#[cfg(not(target_arch = "wasm32"))]
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Get current timestamp as u64 (WASM stub — no system clock)
#[cfg(target_arch = "wasm32")]
pub fn current_timestamp() -> u64 {
    0
}

/// Basic save file information
#[derive(Debug, Clone)]
pub struct SaveInfo {
    pub player_name: String,
    pub save_time: u64,
    pub version: u32,
    pub turns: u64,
    pub dlevel: String,
}

/// Get the default save directory (relative to current dir)
pub fn get_save_directory() -> std::path::PathBuf {
    std::path::PathBuf::from("save")
}

/// Get the save file path for a player
pub fn get_save_path(player_name: &str) -> std::path::PathBuf {
    get_save_directory().join(format!("{}.nhrs", player_name))
}

// --- Filesystem-dependent functions (not available on WASM) ---

#[cfg(not(target_arch = "wasm32"))]
mod fs_impl {
    use super::*;
    use std::fs::File;
    use std::io::{BufReader, BufWriter, Read, Write};
    use std::path::Path;

    /// Write save header to a file
    pub fn write_save_header(writer: &mut BufWriter<File>, header: &SaveHeader) -> Result<(), SaveError> {
        // Write magic bytes
        writer.write_all(SAVE_MAGIC)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        // Serialize header as JSON line
        let header_json = serde_json::to_string(header)
            .map_err(|e| SaveError::SerializeError(e.to_string()))?;

        // Write header length and data
        let header_bytes = header_json.as_bytes();
        let len = header_bytes.len() as u32;
        writer.write_all(&len.to_le_bytes())
            .map_err(|e| SaveError::IoError(e.to_string()))?;
        writer.write_all(header_bytes)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Read save header from a file
    pub fn read_save_header(reader: &mut BufReader<File>) -> Result<SaveHeader, SaveError> {
        // Read and verify magic bytes
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        if &magic != SAVE_MAGIC {
            return Err(SaveError::InvalidMagic);
        }

        // Read header length
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes)
            .map_err(|e| SaveError::IoError(e.to_string()))?;
        let len = u32::from_le_bytes(len_bytes) as usize;

        // Read header JSON
        let mut header_bytes = vec![0u8; len];
        reader.read_exact(&mut header_bytes)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        let header: SaveHeader = serde_json::from_slice(&header_bytes)
            .map_err(|e| SaveError::CorruptedData(e.to_string()))?;

        // Verify version
        if header.version != SAVE_VERSION {
            return Err(SaveError::VersionMismatch {
                expected: SAVE_VERSION,
                found: header.version,
            });
        }

        Ok(header)
    }

    /// Delete a save file (after successful load or on death)
    pub fn delete_save(path: &Path) -> Result<(), SaveError> {
        if path.exists() {
            std::fs::remove_file(path)
                .map_err(|e| SaveError::IoError(e.to_string()))?;
        }
        Ok(())
    }

    /// Check if a save file exists
    pub fn save_exists(path: &Path) -> bool {
        path.exists()
    }

    /// Get save file info without loading the full game
    pub fn get_save_info(path: &Path) -> Result<SaveInfo, SaveError> {
        let file = File::open(path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;
        let mut reader = BufReader::new(file);

        let header = read_save_header(&mut reader)?;

        Ok(SaveInfo {
            player_name: header.player_name,
            save_time: header.save_time,
            version: header.version,
            turns: header.turns,
            dlevel: header.dlevel,
        })
    }

    /// Ensure save directory exists
    pub fn ensure_save_directory() -> Result<(), SaveError> {
        let dir = super::get_save_directory();
        std::fs::create_dir_all(&dir)
            .map_err(|e| SaveError::IoError(e.to_string()))
    }

    /// List all save files in the save directory
    pub fn list_saves() -> Result<Vec<SaveInfo>, SaveError> {
        let dir = super::get_save_directory();
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut saves = Vec::new();
        let entries = std::fs::read_dir(&dir)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
            let path = entry.path();
            if path.extension().map(|e| e == "nhrs").unwrap_or(false)
                && let Ok(info) = get_save_info(&path)
            {
                saves.push(info);
            }
        }

        Ok(saves)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use fs_impl::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        let data = b"Hello, NetHack!";
        let checksum1 = calculate_checksum(data);
        let checksum2 = calculate_checksum(data);
        assert_eq!(checksum1, checksum2);

        let different_data = b"Hello, World!";
        let checksum3 = calculate_checksum(different_data);
        assert_ne!(checksum1, checksum3);
    }

    #[test]
    fn test_save_path() {
        let path = get_save_path("TestPlayer");
        assert!(path.to_string_lossy().contains("TestPlayer"));
        assert!(path.to_string_lossy().ends_with(".nhrs"));
    }

    #[test]
    fn test_timestamp() {
        let ts = current_timestamp();
        // On native, should be > 0; on WASM, returns 0
        #[cfg(not(target_arch = "wasm32"))]
        assert!(ts > 0);
        #[cfg(target_arch = "wasm32")]
        assert_eq!(ts, 0);
    }
}
