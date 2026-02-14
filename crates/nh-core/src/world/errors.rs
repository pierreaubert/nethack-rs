//! Error handling and reporting functions translated from NetHack hacklib.c
//!
//! Provides functions for error reporting, security violation detection,
//! and file error handling with proper panic/panic conditions.

use std::path::{Path, PathBuf};
use thiserror::Error;

/// File-related errors that can occur during game operations
#[derive(Error, Debug, Clone)]
pub enum FileError {
    #[error("Could not open file '{path}': {reason}")]
    CouldNotOpen { path: String, reason: String },

    #[error("File was suspiciously removed: '{path}'")]
    TrickeryDetected { path: String },

    #[error("Security violation: {message}")]
    SecurityViolation { message: String },

    #[error("Invalid file path: {path}")]
    InvalidPath { path: String },
}

/// Report a file that couldn't be opened and panic
/// Translated from: void couldnt_open_file(int fd, const char *file)
pub fn couldnt_open_file(file_path: &str) -> ! {
    let message = format!("Could not open file: {}", file_path);
    panic!("{}", message);
}

/// Report that a file was suspiciously removed
/// This indicates a security issue or file system problem
/// Translated from: void tricked_fileremoved(const char *file)
pub fn tricked_fileremoved(file_path: &str) -> ! {
    let message = format!(
        "File was suspiciously removed: {}. This could indicate a security problem.",
        file_path
    );
    panic!("{}", message);
}

/// Report a general security violation and panic
/// Translated from: void trickery(const char *str)
pub fn trickery(message: &str) -> ! {
    panic!("Security violation: {}", message);
}

/// Format and panic with a formatted error message
/// Provides similar functionality to error4 in C version
/// This function never returns - it always panics
pub fn error4(format_str: &str, arg1: &str, arg2: &str, arg3: &str) -> ! {
    let message = format_str
        .replace("{}", &arg1)
        .replace("{}", &arg2)
        .replace("{}", &arg3);
    panic!("NetHack error: {}", message);
}

/// Validate a file path for security issues
/// Checks for:
/// - Path traversal attempts (..)
/// - Absolute paths (should be relative to game directory)
/// - Special characters that might indicate injection
///
/// Returns Ok(()) if path is valid, Err(FileError) if suspicious
pub fn validate_file_path(path: &str) -> Result<(), FileError> {
    // Check for path traversal attempts
    if path.contains("..") {
        return Err(FileError::SecurityViolation {
            message: format!("Path traversal attempt detected in: {}", path),
        });
    }

    // Check for absolute paths
    if Path::new(path).is_absolute() {
        return Err(FileError::SecurityViolation {
            message: format!("Absolute paths not allowed: {}", path),
        });
    }

    // Check for null bytes
    if path.contains('\0') {
        return Err(FileError::SecurityViolation {
            message: "Null bytes in path detected".to_string(),
        });
    }

    Ok(())
}

/// Validate multiple prefix locations to ensure they're safe
/// All prefixes should exist and be readable
pub fn validate_prefix_locations(prefixes: &[PathBuf]) -> Result<(), FileError> {
    for prefix in prefixes {
        if !prefix.is_absolute() {
            return Err(FileError::SecurityViolation {
                message: format!("Prefix path should be absolute: {}", prefix.display()),
            });
        }

        if !prefix.exists() {
            return Err(FileError::InvalidPath {
                path: format!("Prefix path does not exist: {}", prefix.display()),
            });
        }

        if !prefix.is_dir() {
            return Err(FileError::InvalidPath {
                path: format!("Prefix path is not a directory: {}", prefix.display()),
            });
        }
    }

    Ok(())
}

/// Check if a path looks suspicious
/// Returns true if the path might be an attack attempt
pub fn is_suspicious_path(path: &str) -> bool {
    // Contains path traversal
    path.contains("..") ||
    // Contains shell metacharacters
    path.contains('$') ||
    path.contains('`') ||
    path.contains('\\') ||
    // Contains null bytes
    path.contains('\0') ||
    // Starts with special characters
    path.starts_with('-')
}

/// Log a security violation message (used for debugging)
/// In production, these are logged and/or reported
pub fn log_security_issue(issue: &str) {
    eprintln!("[SECURITY] {}", issue);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_file_path_valid() {
        assert!(validate_file_path("save/player.sav").is_ok());
        assert!(validate_file_path("level.1").is_ok());
        assert!(validate_file_path("./data/file.txt").is_ok());
    }

    #[test]
    fn test_validate_file_path_traversal() {
        assert!(validate_file_path("../../../etc/passwd").is_err());
        assert!(validate_file_path("save/../../../etc/passwd").is_err());
    }

    #[test]
    fn test_validate_file_path_absolute() {
        assert!(validate_file_path("/etc/passwd").is_err());
        assert!(validate_file_path("/home/user/file").is_err());
    }

    #[test]
    fn test_validate_file_path_null_bytes() {
        assert!(validate_file_path("file\0.txt").is_err());
    }

    #[test]
    fn test_is_suspicious_path() {
        assert!(is_suspicious_path("../etc/passwd"));
        assert!(is_suspicious_path("$(whoami)"));
        assert!(is_suspicious_path("`id`"));
        assert!(is_suspicious_path("file\0.txt"));
        assert!(is_suspicious_path("-e rm -rf /"));

        assert!(!is_suspicious_path("normal_file.txt"));
        assert!(!is_suspicious_path("save/level1"));
    }

    #[test]
    #[should_panic]
    fn test_couldnt_open_file_panics() {
        couldnt_open_file("nonexistent.txt");
    }

    #[test]
    #[should_panic]
    fn test_trickery_panics() {
        trickery("Suspicious activity detected");
    }

    #[test]
    fn test_file_error_display() {
        let err = FileError::CouldNotOpen {
            path: "/path/to/file".to_string(),
            reason: "permission denied".to_string(),
        };
        assert!(err.to_string().contains("Could not open file"));
        assert!(err.to_string().contains("/path/to/file"));
    }
}
