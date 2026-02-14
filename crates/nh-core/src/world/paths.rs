//! File path and file locking utilities translated from NetHack files.c, save.c
//!
//! Provides functions for constructing file paths, file locking,
//! compression, and file operations with proper security validation.

use crate::world::errors::{FileError, validate_file_path};
use std::fs::{File, OpenOptions, create_dir_all, remove_file};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

/// Construct a lock filename from a base name
/// Typically adds a .lock suffix
pub fn make_lockname(base_name: &str) -> String {
    format!("{}.lock", base_name)
}

/// Construct a compressed filename from a base name
/// Adds a .gz suffix for gzip compression
pub fn make_compressed_name(base_name: &str) -> String {
    format!("{}.gz", base_name)
}

/// Lock a file exclusively using filesystem-based locking
/// Creates a lock file that acts as a mutex
pub fn lock_file(path: &Path) -> Result<File, FileError> {
    validate_file_path(&path.to_string_lossy())?;

    let lock_path = make_lockname(&path.to_string_lossy());

    match OpenOptions::new().write(true).create(true).open(&lock_path) {
        Ok(file) => Ok(file),
        Err(e) => Err(FileError::CouldNotOpen {
            path: lock_path,
            reason: e.to_string(),
        }),
    }
}

/// Unlock a file by removing the lock file
pub fn unlock_file(path: &Path) -> Result<(), FileError> {
    let lock_path = make_lockname(&path.to_string_lossy());

    remove_file(&lock_path).map_err(|e| FileError::CouldNotOpen {
        path: lock_path,
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Clear all level file locks in a directory
pub fn clearlocks(lock_dir: &Path) -> Result<u32, FileError> {
    let mut count = 0;

    if !lock_dir.exists() {
        return Ok(0);
    }

    match std::fs::read_dir(lock_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("lock") {
                            if remove_file(&path).is_ok() {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(FileError::CouldNotOpen {
                path: lock_dir.to_string_lossy().to_string(),
                reason: e.to_string(),
            });
        }
    }

    Ok(count)
}

/// Ensure a system config file exists
/// Creates the file with default content if it doesn't exist
pub fn assure_syscf_file(file_path: &Path, default_content: &str) -> Result<(), FileError> {
    validate_file_path(&file_path.to_string_lossy())?;

    if !file_path.exists() {
        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            create_dir_all(parent).map_err(|e| FileError::CouldNotOpen {
                path: parent.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path)
            .map_err(|e| FileError::CouldNotOpen {
                path: file_path.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?;

        file.write_all(default_content.as_bytes())
            .map_err(|e| FileError::CouldNotOpen {
                path: file_path.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?;
    }

    Ok(())
}

/// Copy file contents from source to destination
pub fn copyfile(source: &Path, dest: &Path) -> Result<(), FileError> {
    validate_file_path(&source.to_string_lossy())?;
    validate_file_path(&dest.to_string_lossy())?;

    let mut src_file =
        OpenOptions::new()
            .read(true)
            .open(source)
            .map_err(|e| FileError::CouldNotOpen {
                path: source.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?;

    let mut dest_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest)
        .map_err(|e| FileError::CouldNotOpen {
            path: dest.to_string_lossy().to_string(),
            reason: e.to_string(),
        })?;

    let mut buffer = vec![0; 8192];
    loop {
        match src_file.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                dest_file
                    .write_all(&buffer[..n])
                    .map_err(|e| FileError::CouldNotOpen {
                        path: dest.to_string_lossy().to_string(),
                        reason: e.to_string(),
                    })?;
            }
            Err(e) => {
                return Err(FileError::CouldNotOpen {
                    path: source.to_string_lossy().to_string(),
                    reason: e.to_string(),
                });
            }
        }
    }

    Ok(())
}

/// Compress a file in place using gzip
pub fn docompress_file(path: &Path) -> Result<(), FileError> {
    validate_file_path(&path.to_string_lossy())?;

    let compressed_path = make_compressed_name(&path.to_string_lossy());

    let src_file =
        OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|e| FileError::CouldNotOpen {
                path: path.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?;

    let dest_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&compressed_path)
        .map_err(|e| FileError::CouldNotOpen {
            path: compressed_path.clone(),
            reason: e.to_string(),
        })?;

    let mut reader = BufReader::new(src_file);
    let mut encoder = flate2::write::GzEncoder::new(dest_file, flate2::Compression::default());

    match std::io::copy(&mut reader, &mut encoder) {
        Ok(_) => {
            encoder.finish().map_err(|e| FileError::CouldNotOpen {
                path: compressed_path.clone(),
                reason: e.to_string(),
            })?;
            // Remove original file after successful compression
            std::fs::remove_file(path).ok();
            Ok(())
        }
        Err(e) => {
            // Clean up failed compressed file
            std::fs::remove_file(&compressed_path).ok();
            Err(FileError::CouldNotOpen {
                path: compressed_path,
                reason: e.to_string(),
            })
        }
    }
}

/// Compress file contents using gzip
pub fn nh_compress(data: &[u8]) -> Result<Vec<u8>, FileError> {
    use flate2::Compression;
    use flate2::write::GzEncoder;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| FileError::CouldNotOpen {
            path: "<memory>".to_string(),
            reason: e.to_string(),
        })?;

    encoder.finish().map_err(|e| FileError::CouldNotOpen {
        path: "<memory>".to_string(),
        reason: e.to_string(),
    })
}

/// Decompress file contents using gzip
pub fn nh_uncompress(compressed: &[u8]) -> Result<Vec<u8>, FileError> {
    use flate2::read::GzDecoder;

    let mut decoder = GzDecoder::new(compressed);
    let mut decompressed = Vec::new();

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| FileError::CouldNotOpen {
            path: "<memory>".to_string(),
            reason: e.to_string(),
        })?;

    Ok(decompressed)
}

/// Open a data file with a prefix path
/// Searches multiple common prefix locations
pub fn fopen_datafile(filename: &str, prefixes: &[PathBuf]) -> Result<File, FileError> {
    validate_file_path(filename)?;

    for prefix in prefixes {
        let full_path = prefix.join(filename);

        if let Ok(file) = OpenOptions::new().read(true).open(&full_path) {
            return Ok(file);
        }
    }

    Err(FileError::CouldNotOpen {
        path: filename.to_string(),
        reason: "not found in any prefix location".to_string(),
    })
}

/// Open a symbol file (for graphics)
pub fn fopen_sym_file(filename: &str, prefixes: &[PathBuf]) -> Result<File, FileError> {
    fopen_datafile(&format!("symbols/{}", filename), prefixes)
}

/// Open a wizard kit configuration file
pub fn fopen_wizkit_file(filename: &str, prefixes: &[PathBuf]) -> Result<File, FileError> {
    fopen_datafile(&format!("wizkit/{}", filename), prefixes)
}

/// Find a file in multiple locations
/// Returns the first matching file path
pub fn find_file(filename: &str, search_paths: &[PathBuf]) -> Option<PathBuf> {
    if validate_file_path(filename).is_err() {
        return None;
    }

    for search_path in search_paths {
        let full_path = search_path.join(filename);
        if full_path.exists() && full_path.is_file() {
            return Some(full_path);
        }
    }

    None
}

/// Construct a fully-qualified filename with prefix
/// Combines a prefix with a relative filename
pub fn fqname(prefix: &Path, filename: &str) -> Result<PathBuf, FileError> {
    validate_file_path(filename)?;

    let path = prefix.join(filename);
    Ok(path)
}

/// Get an environment variable with platform handling
pub fn nh_getenv(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Parse and validate command-line arguments
/// Returns Ok with parsed arguments or Err with error message
pub fn argcheck(args: &[String]) -> Result<ParsedArgs, String> {
    let mut parsed = ParsedArgs::default();

    let mut i = 1; // Skip program name
    while i < args.len() {
        let arg = &args[i];

        if arg.starts_with('-') {
            match arg.as_str() {
                "-D" => parsed.recovery_mode = true,
                "-s" => parsed.show_scores = true,
                "-v" | "--version" => parsed.show_version = true,
                "-h" | "--help" => parsed.show_help = true,
                _ => return Err(format!("Unknown option: {}", arg)),
            }
        } else {
            parsed.player_name = Some(arg.clone());
        }

        i += 1;
    }

    Ok(parsed)
}

/// Parsed command-line arguments
#[derive(Debug, Default, Clone)]
pub struct ParsedArgs {
    pub player_name: Option<String>,
    pub recovery_mode: bool,
    pub show_scores: bool,
    pub show_version: bool,
    pub show_help: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_lockname() {
        assert_eq!(make_lockname("level.1"), "level.1.lock");
        assert_eq!(make_lockname("save.sav"), "save.sav.lock");
    }

    #[test]
    fn test_make_compressed_name() {
        assert_eq!(make_compressed_name("level.1"), "level.1.gz");
        assert_eq!(make_compressed_name("save.sav"), "save.sav.gz");
    }

    #[test]
    fn test_nh_compress_decompress() {
        let original = b"Hello, World! This is test data.";
        let compressed = nh_compress(original).expect("Compression failed");
        assert!(compressed.len() > 0);

        let decompressed = nh_uncompress(&compressed).expect("Decompression failed");
        assert_eq!(&decompressed, original);
    }

    #[test]
    fn test_fqname() {
        let prefix = PathBuf::from("/tmp");
        let result = fqname(&prefix, "test.txt");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("test.txt"));
    }

    #[test]
    fn test_argcheck_basic() {
        let args = vec!["nethack".to_string(), "player1".to_string()];
        let parsed = argcheck(&args).expect("Parse failed");
        assert_eq!(parsed.player_name, Some("player1".to_string()));
    }

    #[test]
    fn test_argcheck_flags() {
        let args = vec!["nethack".to_string(), "-D".to_string(), "-v".to_string()];
        let parsed = argcheck(&args).expect("Parse failed");
        assert!(parsed.recovery_mode);
        assert!(parsed.show_version);
    }

    #[test]
    fn test_argcheck_invalid() {
        let args = vec!["nethack".to_string(), "-x".to_string()];
        assert!(argcheck(&args).is_err());
    }
}
