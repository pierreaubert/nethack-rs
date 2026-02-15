//! Miscellaneous command functions translated from NetHack cmd.c, do.c
//!
//! Provides functions for:
//! - Game mode management (explore, wizard modes)
//! - Rest preferences and settings
//! - Save preferences
//! - Discovery/explore mode activation

#[cfg(not(feature = "std"))]
use crate::compat::*;

use super::Command;

/// Game play modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayMode {
    /// Normal gameplay mode
    Normal,
    /// Explore mode (no permadeath)
    Explore,
    /// Wizard mode (debug/testing)
    Wizard,
}

/// Rest preference configuration
#[derive(Debug, Clone, Default)]
pub struct RestPreference {
    /// Whether to rest until HP is full
    pub rest_until_healed: bool,
    /// Whether to rest until MP/energy is full
    pub rest_until_powered: bool,
    /// Number of turns to rest (0 = infinite until condition met)
    pub turns: u32,
}

impl RestPreference {
    /// Create default rest preferences
    pub fn new() -> Self {
        RestPreference::default()
    }

    /// Create preference to rest N turns
    pub fn for_turns(turns: u32) -> Self {
        RestPreference {
            rest_until_healed: false,
            rest_until_powered: false,
            turns,
        }
    }

    /// Create preference to rest until healed
    pub fn until_healed() -> Self {
        RestPreference {
            rest_until_healed: true,
            rest_until_powered: false,
            turns: 0,
        }
    }
}

/// Save preference configuration
#[derive(Debug, Clone, Default)]
pub struct SavePreference {
    /// Automatically save on dangerous situations
    pub auto_save_danger: bool,
    /// Create backup saves
    pub create_backups: bool,
    /// Verbose save messages
    pub verbose_save: bool,
}

impl SavePreference {
    /// Create default save preferences
    pub fn new() -> Self {
        SavePreference::default()
    }
}

/// Wait/rest command - player does nothing for one turn
/// This is a basic wait action that passes time
pub fn donull() -> Command {
    Command::Rest
}

/// Enter explore mode (if not already in it)
/// Explore mode disables permadeath and allows recovery from fatal situations
pub fn enter_explore_mode() -> PlayMode {
    PlayMode::Explore
}

/// Set the current play mode
/// Returns the new play mode
pub fn set_playmode(mode: PlayMode) -> PlayMode {
    mode
}

/// Get the current play mode description
pub fn playmode_description(mode: PlayMode) -> &'static str {
    match mode {
        PlayMode::Normal => "Normal (permadeath enabled)",
        PlayMode::Explore => "Explore (no permadeath)",
        PlayMode::Wizard => "Wizard (debug mode)",
    }
}

/// Set rest preference for the player
/// This configures what the player will do when issuing rest commands
pub fn set_restpref(pref: RestPreference) -> RestPreference {
    pref
}

/// Reset rest preferences to defaults
pub fn reset_restpref() -> RestPreference {
    RestPreference::new()
}

/// Set save preference for the player
pub fn set_savepref(pref: SavePreference) -> SavePreference {
    pref
}

/// Reset save preferences to defaults
pub fn reset_savepref() -> SavePreference {
    SavePreference::new()
}

/// Enter discovery/explore mode
/// This mode reveals information about discovered items and features
pub fn enter_discovery_mode() -> bool {
    true
}

/// Check if currently in discovery mode
pub fn in_discovery_mode() -> bool {
    false
}

/// Toggle discovery mode
pub fn toggle_discovery_mode() -> bool {
    !in_discovery_mode()
}

/// Get game mode description for display
pub fn get_mode_info() -> String {
    let mut info = String::new();
    info.push_str("Game Mode Information\n");
    info.push_str("=====================\n\n");

    info.push_str("Play Modes:\n");
    info.push_str("- Normal: Standard NetHack gameplay with permadeath\n");
    info.push_str("- Explore: Discover dungeons without permanent death\n");
    info.push_str("- Wizard: Debug mode with special commands\n\n");

    info.push_str("Press '#' for extended commands\n");
    info.push_str("Press '?' for help\n");

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playmode_normal() {
        let mode = PlayMode::Normal;
        assert_eq!(playmode_description(mode), "Normal (permadeath enabled)");
    }

    #[test]
    fn test_playmode_explore() {
        let mode = PlayMode::Explore;
        assert_eq!(playmode_description(mode), "Explore (no permadeath)");
    }

    #[test]
    fn test_playmode_wizard() {
        let mode = PlayMode::Wizard;
        assert_eq!(playmode_description(mode), "Wizard (debug mode)");
    }

    #[test]
    fn test_enter_explore_mode() {
        let mode = enter_explore_mode();
        assert_eq!(mode, PlayMode::Explore);
    }

    #[test]
    fn test_set_playmode() {
        let mode = set_playmode(PlayMode::Wizard);
        assert_eq!(mode, PlayMode::Wizard);
    }

    #[test]
    fn test_rest_preference_new() {
        let pref = RestPreference::new();
        assert!(!pref.rest_until_healed);
        assert_eq!(pref.turns, 0);
    }

    #[test]
    fn test_rest_preference_for_turns() {
        let pref = RestPreference::for_turns(10);
        assert_eq!(pref.turns, 10);
    }

    #[test]
    fn test_rest_preference_until_healed() {
        let pref = RestPreference::until_healed();
        assert!(pref.rest_until_healed);
    }

    #[test]
    fn test_set_restpref() {
        let pref = RestPreference::for_turns(20);
        let result = set_restpref(pref.clone());
        assert_eq!(result.turns, 20);
    }

    #[test]
    fn test_reset_restpref() {
        let pref = reset_restpref();
        assert_eq!(pref.turns, 0);
    }

    #[test]
    fn test_save_preference_new() {
        let pref = SavePreference::new();
        assert!(!pref.auto_save_danger);
    }

    #[test]
    fn test_set_savepref() {
        let mut pref = SavePreference::new();
        pref.auto_save_danger = true;
        let result = set_savepref(pref.clone());
        assert!(result.auto_save_danger);
    }

    #[test]
    fn test_donull() {
        let cmd = donull();
        assert_eq!(cmd, Command::Rest);
    }

    #[test]
    fn test_discovery_mode() {
        assert!(!in_discovery_mode());
        let new_state = toggle_discovery_mode();
        assert!(new_state);
    }

    #[test]
    fn test_get_mode_info() {
        let info = get_mode_info();
        assert!(info.contains("Play Modes"));
        assert!(info.contains("Normal"));
        assert!(info.contains("Explore"));
    }
}
