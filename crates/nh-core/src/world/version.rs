//! Version information and display functions translated from NetHack version.c
//!
//! Provides functions for retrieving and formatting version information,
//! including build details and git commit info when available.

/// Get the major version number
const VERSION_MAJOR: u32 = 3;

/// Get the minor version number
const VERSION_MINOR: u32 = 6;

/// Get the patch version number
const VERSION_PATCH: u32 = 7;

/// Get the build type (dev, release, etc.)
const BUILD_TYPE: &str = "rust";

/// Get short version string (e.g., "3.6.7")
pub fn version_string() -> String {
    format!("{}.{}.{}", VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH)
}

/// Get long version string with full details
pub fn getversionstring() -> String {
    let base_version = version_string();

    #[cfg(feature = "git-version")]
    {
        // If compiled with git info, include it
        if let Some(git_sha) = option_env!("GIT_SHA") {
            let short_sha = &git_sha[0..7.min(git_sha.len())];
            return format!("NetHack {} (rust, build: {})", base_version, short_sha);
        }
    }

    format!("NetHack {} (rust)", base_version)
}

/// Display version information to the screen
/// Returns the version string that should be displayed
pub fn doversion() -> String {
    format!(
        "NetHack version {}\nRust implementation\n{}",
        version_string(),
        crate::world::time::yyyymmddhhmmss()
    )
}

/// Display extended version information with build details
pub fn doextversion() -> String {
    let mut info = format!("NetHack {} extended version\n", version_string());
    info.push_str(&format!("Build type: {}\n", BUILD_TYPE));
    info.push_str(&format!(
        "Build date: {}\n",
        crate::world::time::yyyymmddhhmmss()
    ));
    info.push_str(&format!("Platform: {}\n", std::env::consts::OS));
    info.push_str(&format!("Architecture: {}\n", std::env::consts::ARCH));

    #[cfg(feature = "git-version")]
    {
        if let Some(git_sha) = option_env!("GIT_SHA") {
            info.push_str(&format!("Git SHA: {}\n", git_sha));
        }
        if let Some(git_branch) = option_env!("GIT_BRANCH") {
            info.push_str(&format!("Git branch: {}\n", git_branch));
        }
    }

    info
}

/// Check if running at console (for version display purposes)
/// Returns true if output can be displayed interactively
pub fn atconsole() -> bool {
    // Check if stdout is a TTY
    atty::is(atty::Stream::Stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_string() {
        let version = version_string();
        assert_eq!(version, "3.6.7");
    }

    #[test]
    fn test_getversionstring() {
        let version = getversionstring();
        assert!(version.contains("NetHack"));
        assert!(version.contains("3.6.7"));
        assert!(version.contains("rust"));
    }

    #[test]
    fn test_doversion() {
        let info = doversion();
        assert!(info.contains("3.6.7"));
        assert!(info.contains("NetHack"));
    }

    #[test]
    fn test_doextversion() {
        let info = doextversion();
        assert!(info.contains("3.6.7"));
        assert!(info.contains("extended version"));
        assert!(info.contains("Platform"));
    }

    #[test]
    fn test_atconsole() {
        // Just verify it returns a bool
        let _result = atconsole();
    }
}
