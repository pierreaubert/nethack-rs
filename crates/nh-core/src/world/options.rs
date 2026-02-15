//! Game options and configuration (options.c)
//!
//! Handles user preferences, keybindings, and configuration file loading.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use serde::{Deserialize, Serialize};
use hashbrown::HashMap;
#[cfg(feature = "std")]
use std::path::Path;

/// User-configurable game options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameOptions {
    // Character options
    pub name: String,
    pub role: Option<String>,
    pub race: Option<String>,
    pub gender: Option<String>,
    pub alignment: Option<String>,

    // Display options
    pub color: bool,
    pub hilite_pet: bool,
    pub hilite_pile: bool,
    pub lit_corridor: bool,
    pub dark_room: bool,
    pub showexp: bool,
    pub showscore: bool,
    pub time: bool,
    pub toptenwin: bool,

    // Interface options
    pub autopickup: bool,
    pub autopickup_types: String,
    pub pickup_burden: PickupBurden,
    pub safe_pet: bool,
    pub safe_peaceful: bool,
    pub confirm: bool,
    pub verbose: bool,
    pub perm_invent: bool,
    pub popup_dialog: bool,

    // Movement options
    pub number_pad: NumberPadMode,
    pub rest_on_space: bool,
    pub travel: bool,
    pub runmode: RunMode,

    // Message options
    pub msg_window: MessageWindow,
    pub msghistory: u32,

    // Sound options
    pub sound: bool,

    // Misc options
    pub checkpoint: bool,
    pub disclose: DisclosureOptions,
    pub fruit: String,
    pub catname: String,
    pub dogname: String,
    pub horsename: String,

    // Custom keybindings
    pub keybindings: HashMap<String, String>,
}

impl Default for GameOptions {
    fn default() -> Self {
        Self {
            name: String::new(),
            role: None,
            race: None,
            gender: None,
            alignment: None,

            color: true,
            hilite_pet: true,
            hilite_pile: true,
            lit_corridor: false,
            dark_room: true,
            showexp: false,
            showscore: false,
            time: false,
            toptenwin: false,

            autopickup: true,
            autopickup_types: "$?!/\"=".to_string(),
            pickup_burden: PickupBurden::Unencumbered,
            safe_pet: true,
            safe_peaceful: true,
            confirm: true,
            verbose: true,
            perm_invent: false,
            popup_dialog: false,

            number_pad: NumberPadMode::Off,
            rest_on_space: false,
            travel: true,
            runmode: RunMode::Walk,

            msg_window: MessageWindow::Single,
            msghistory: 20,

            sound: true,

            checkpoint: true,
            disclose: DisclosureOptions::default(),
            fruit: "slime mold".to_string(),
            catname: String::new(),
            dogname: String::new(),
            horsename: String::new(),

            keybindings: HashMap::new(),
        }
    }
}

impl GameOptions {
    #[cfg(feature = "std")]
    /// Load options from a file
    pub fn load_from_file(path: &Path) -> Result<Self, OptionsError> {
        let contents =
            std::fs::read_to_string(path).map_err(|e| OptionsError::IoError(e.to_string()))?;

        Self::parse_config(&contents)
    }

    /// Parse options from a config string (nethackrc format)
    pub fn parse_config(contents: &str) -> Result<Self, OptionsError> {
        let mut options = Self::default();

        for line in contents.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse OPTIONS= lines
            if let Some(opts) = line.strip_prefix("OPTIONS=") {
                for opt in opts.split(',') {
                    options.parse_option(opt.trim())?;
                }
            }
            // Parse BIND= lines for keybindings
            else if let Some(bind) = line.strip_prefix("BIND=") {
                if let Some((key, cmd)) = bind.split_once(':') {
                    options
                        .keybindings
                        .insert(key.trim().to_string(), cmd.trim().to_string());
                }
            }
        }

        Ok(options)
    }

    /// Parse a single option
    fn parse_option(&mut self, opt: &str) -> Result<(), OptionsError> {
        // Handle negation
        let (negated, opt_name) = if let Some(name) = opt.strip_prefix('!') {
            (true, name)
        } else if let Some(name) = opt.strip_prefix("no") {
            (true, name)
        } else {
            (false, opt)
        };

        // Handle key=value options
        if let Some((key, value)) = opt_name.split_once(':') {
            return self.set_option(key.trim(), Some(value.trim()));
        }
        if let Some((key, value)) = opt_name.split_once('=') {
            return self.set_option(key.trim(), Some(value.trim()));
        }

        // Boolean option
        self.set_bool_option(opt_name, !negated)
    }

    /// Set a boolean option
    fn set_bool_option(&mut self, name: &str, value: bool) -> Result<(), OptionsError> {
        match name {
            "color" => self.color = value,
            "hilite_pet" => self.hilite_pet = value,
            "hilite_pile" => self.hilite_pile = value,
            "lit_corridor" => self.lit_corridor = value,
            "dark_room" => self.dark_room = value,
            "showexp" => self.showexp = value,
            "showscore" => self.showscore = value,
            "time" => self.time = value,
            "toptenwin" => self.toptenwin = value,
            "autopickup" => self.autopickup = value,
            "safe_pet" => self.safe_pet = value,
            "safe_peaceful" => self.safe_peaceful = value,
            "confirm" => self.confirm = value,
            "verbose" => self.verbose = value,
            "perm_invent" => self.perm_invent = value,
            "popup_dialog" => self.popup_dialog = value,
            "rest_on_space" => self.rest_on_space = value,
            "travel" => self.travel = value,
            "sound" => self.sound = value,
            "checkpoint" => self.checkpoint = value,
            _ => return Err(OptionsError::UnknownOption(name.to_string())),
        }
        Ok(())
    }

    /// Set an option with a value
    fn set_option(&mut self, name: &str, value: Option<&str>) -> Result<(), OptionsError> {
        let value = value.ok_or_else(|| OptionsError::MissingValue(name.to_string()))?;

        match name {
            "name" => self.name = value.to_string(),
            "role" => self.role = Some(value.to_string()),
            "race" => self.race = Some(value.to_string()),
            "gender" => self.gender = Some(value.to_string()),
            "align" | "alignment" => self.alignment = Some(value.to_string()),
            "autopickup_types" | "pickup_types" => self.autopickup_types = value.to_string(),
            "pickup_burden" => {
                self.pickup_burden = match value.to_lowercase().as_str() {
                    "unencumbered" | "u" => PickupBurden::Unencumbered,
                    "burdened" | "b" => PickupBurden::Burdened,
                    "stressed" | "s" => PickupBurden::Stressed,
                    "strained" | "n" => PickupBurden::Strained,
                    "overtaxed" | "o" => PickupBurden::Overtaxed,
                    "overloaded" | "l" => PickupBurden::Overloaded,
                    _ => {
                        return Err(OptionsError::InvalidValue(
                            name.to_string(),
                            value.to_string(),
                        ));
                    }
                };
            }
            "number_pad" | "numpad" => {
                self.number_pad = match value {
                    "0" | "off" => NumberPadMode::Off,
                    "1" | "on" => NumberPadMode::On,
                    "2" => NumberPadMode::Phone,
                    _ => {
                        return Err(OptionsError::InvalidValue(
                            name.to_string(),
                            value.to_string(),
                        ));
                    }
                };
            }
            "runmode" => {
                self.runmode = match value.to_lowercase().as_str() {
                    "teleport" => RunMode::Teleport,
                    "run" => RunMode::Run,
                    "walk" => RunMode::Walk,
                    "crawl" => RunMode::Crawl,
                    _ => {
                        return Err(OptionsError::InvalidValue(
                            name.to_string(),
                            value.to_string(),
                        ));
                    }
                };
            }
            "msg_window" | "msgwindow" => {
                self.msg_window = match value.to_lowercase().as_str() {
                    "single" | "s" => MessageWindow::Single,
                    "full" | "f" => MessageWindow::Full,
                    "combination" | "c" => MessageWindow::Combination,
                    "reverse" | "r" => MessageWindow::Reverse,
                    _ => {
                        return Err(OptionsError::InvalidValue(
                            name.to_string(),
                            value.to_string(),
                        ));
                    }
                };
            }
            "msghistory" => {
                self.msghistory = value
                    .parse()
                    .map_err(|_| OptionsError::InvalidValue(name.to_string(), value.to_string()))?;
            }
            "fruit" => self.fruit = value.to_string(),
            "catname" => self.catname = value.to_string(),
            "dogname" => self.dogname = value.to_string(),
            "horsename" => self.horsename = value.to_string(),
            _ => return Err(OptionsError::UnknownOption(name.to_string())),
        }
        Ok(())
    }

    #[cfg(feature = "std")]
    /// Save options to a file
    pub fn save_to_file(&self, path: &Path) -> Result<(), OptionsError> {
        let contents = self.to_config_string();
        std::fs::write(path, contents).map_err(|e| OptionsError::IoError(e.to_string()))
    }

    /// Convert options to config file format
    pub fn to_config_string(&self) -> String {
        let mut lines = Vec::new();
        lines.push("# NetHack-rs configuration file".to_string());
        lines.push(String::new());

        // Character options
        if !self.name.is_empty() {
            lines.push(format!("OPTIONS=name:{}", self.name));
        }
        if let Some(ref role) = self.role {
            lines.push(format!("OPTIONS=role:{}", role));
        }
        if let Some(ref race) = self.race {
            lines.push(format!("OPTIONS=race:{}", race));
        }

        // Boolean options
        lines.push(String::new());
        lines.push("# Display options".to_string());
        lines.push(format!(
            "OPTIONS={}",
            if self.color { "color" } else { "!color" }
        ));
        lines.push(format!(
            "OPTIONS={}",
            if self.hilite_pet {
                "hilite_pet"
            } else {
                "!hilite_pet"
            }
        ));
        lines.push(format!(
            "OPTIONS={}",
            if self.showexp { "showexp" } else { "!showexp" }
        ));
        lines.push(format!(
            "OPTIONS={}",
            if self.time { "time" } else { "!time" }
        ));

        lines.push(String::new());
        lines.push("# Gameplay options".to_string());
        lines.push(format!(
            "OPTIONS={}",
            if self.autopickup {
                "autopickup"
            } else {
                "!autopickup"
            }
        ));
        lines.push(format!("OPTIONS=pickup_types:{}", self.autopickup_types));
        lines.push(format!(
            "OPTIONS={}",
            if self.safe_pet {
                "safe_pet"
            } else {
                "!safe_pet"
            }
        ));
        lines.push(format!(
            "OPTIONS={}",
            if self.confirm { "confirm" } else { "!confirm" }
        ));

        // Keybindings
        if !self.keybindings.is_empty() {
            lines.push(String::new());
            lines.push("# Keybindings".to_string());
            for (key, cmd) in &self.keybindings {
                lines.push(format!("BIND={}:{}", key, cmd));
            }
        }

        lines.join("\n")
    }
}

/// Pickup burden threshold
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PickupBurden {
    Unencumbered,
    Burdened,
    Stressed,
    Strained,
    Overtaxed,
    Overloaded,
}

impl Default for PickupBurden {
    fn default() -> Self {
        Self::Unencumbered
    }
}

/// Number pad mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumberPadMode {
    Off,   // Use hjklyubn
    On,    // Use number pad
    Phone, // Phone-style (2=up, 8=down)
}

impl Default for NumberPadMode {
    fn default() -> Self {
        Self::Off
    }
}

/// Run mode (how fast to display running)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunMode {
    Teleport, // Instant
    Run,      // Fast
    Walk,     // Normal
    Crawl,    // Slow
}

impl Default for RunMode {
    fn default() -> Self {
        Self::Walk
    }
}

/// Message window style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageWindow {
    Single,      // Single line
    Full,        // Full window
    Combination, // Combination
    Reverse,     // Reverse order
}

impl Default for MessageWindow {
    fn default() -> Self {
        Self::Single
    }
}

/// End-game disclosure options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisclosureOptions {
    pub inventory: DisclosureChoice,
    pub attributes: DisclosureChoice,
    pub vanquished: DisclosureChoice,
    pub genocided: DisclosureChoice,
    pub conduct: DisclosureChoice,
}

/// Disclosure choice
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisclosureChoice {
    Yes,
    No,
    Prompt,
}

impl Default for DisclosureChoice {
    fn default() -> Self {
        Self::Prompt
    }
}

/// Options parsing error
#[derive(Debug, Clone)]
pub enum OptionsError {
    IoError(String),
    ParseError(String),
    UnknownOption(String),
    InvalidValue(String, String),
    MissingValue(String),
}

impl core::fmt::Display for OptionsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            OptionsError::IoError(e) => write!(f, "IO error: {}", e),
            OptionsError::ParseError(e) => write!(f, "Parse error: {}", e),
            OptionsError::UnknownOption(opt) => write!(f, "Unknown option: {}", opt),
            OptionsError::InvalidValue(opt, val) => {
                write!(f, "Invalid value '{}' for option '{}'", val, opt)
            }
            OptionsError::MissingValue(opt) => write!(f, "Missing value for option '{}'", opt),
        }
    }
}

impl core::error::Error for OptionsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = GameOptions::default();
        assert!(opts.color);
        assert!(opts.autopickup);
        assert!(opts.safe_pet);
    }

    #[test]
    fn test_parse_bool_option() {
        let config = "OPTIONS=color,!autopickup,showexp";
        let opts = GameOptions::parse_config(config).unwrap();
        assert!(opts.color);
        assert!(!opts.autopickup);
        assert!(opts.showexp);
    }

    #[test]
    fn test_parse_value_option() {
        let config = "OPTIONS=name:TestPlayer,fruit:banana";
        let opts = GameOptions::parse_config(config).unwrap();
        assert_eq!(opts.name, "TestPlayer");
        assert_eq!(opts.fruit, "banana");
    }

    #[test]
    fn test_parse_keybinding() {
        let config = "BIND=g:pickup";
        let opts = GameOptions::parse_config(config).unwrap();
        assert_eq!(opts.keybindings.get("g"), Some(&"pickup".to_string()));
    }

    #[test]
    fn test_roundtrip() {
        let mut opts = GameOptions::default();
        opts.name = "TestPlayer".to_string();
        opts.color = false;

        let config_str = opts.to_config_string();
        let parsed = GameOptions::parse_config(&config_str).unwrap();

        assert_eq!(parsed.name, "TestPlayer");
        assert!(!parsed.color);
    }
}
