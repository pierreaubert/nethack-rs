//! Extended command system translated from NetHack cmd.c
//!
//! Provides the extended command dispatcher (#-commands), including:
//! - Extended command registry with metadata (flags, descriptions)
//! - Command lookup and dispatch
//! - Menu-based command selection
//! - Random command selection
//! - Key-to-command mapping

use super::{Command, Direction};
use crate::rng::GameRng;

bitflags::bitflags! {
    /// Flags for extended commands
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CommandFlags: u32 {
        /// Command only available in wizard mode
        const WIZARD_MODE = 1 << 0;
        /// Command for if-buried context
        const IF_BURIED = 1 << 1;
        /// Command supports autocomplete
        const AUTOCOMPLETE = 1 << 2;
        /// General/common command
        const GENERAL = 1 << 3;
        /// Command for explore mode
        const EXPLORE_MODE = 1 << 4;
    }
}

/// Metadata for an extended command
#[derive(Debug, Clone)]
pub struct ExtendedCommand {
    /// Command name (e.g., "adjust", "version")
    pub name: &'static str,
    /// Short description
    pub description: &'static str,
    /// Command implementation
    pub command: Command,
    /// Flags and context
    pub flags: CommandFlags,
}

/// Registry of all extended commands
const EXTENDED_COMMANDS_LIST: &[ExtendedCommand] = &[
    // Meta/Information commands
    ExtendedCommand {
        name: "version",
        description: "show version information",
        command: Command::Help,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "help",
        description: "show help information",
        command: Command::Help,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "history",
        description: "show game history",
        command: Command::History,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "whatis",
        description: "examine an object or creature",
        command: Command::Look,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "whatdoes",
        description: "what does a key do?",
        command: Command::Help,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "list",
        description: "list extended commands",
        command: Command::Help,
        flags: CommandFlags::GENERAL,
    },
    // Inventory/Object commands
    ExtendedCommand {
        name: "adjust",
        description: "adjust inventory letters",
        command: Command::Inventory,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "identify",
        description: "identify an object",
        command: Command::Look,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "sortpack",
        description: "sort inventory",
        command: Command::Inventory,
        flags: CommandFlags::GENERAL,
    },
    // Discovery and knowledge commands
    ExtendedCommand {
        name: "discoveries",
        description: "list discovered objects",
        command: Command::Discoveries,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "annotate",
        description: "annotate the current level",
        command: Command::Discoveries,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "monsters",
        description: "show monster list",
        command: Command::Discoveries,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "artifacts",
        description: "list known artifacts",
        command: Command::Discoveries,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "spells",
        description: "show spell list",
        command: Command::Discoveries,
        flags: CommandFlags::GENERAL,
    },
    // Interaction commands
    ExtendedCommand {
        name: "chat",
        description: "chat with a creature",
        command: Command::Chat,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "pay",
        description: "pay a fee or shopkeeper",
        command: Command::Pay,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "pray",
        description: "pray to your deity",
        command: Command::Pray,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "offer",
        description: "offer something to a deity",
        command: Command::Offer,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "sit",
        description: "sit down",
        command: Command::Sit,
        flags: CommandFlags::GENERAL,
    },
    // Special actions
    ExtendedCommand {
        name: "dip",
        description: "dip an object in a pool",
        command: Command::Dip,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "jump",
        description: "jump to a location",
        command: Command::Move(Direction::Up),
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "rush",
        description: "rush attack",
        command: Command::Fight(Direction::North),
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "save",
        description: "save the game",
        command: Command::Save,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "quit",
        description: "quit the game",
        command: Command::Quit,
        flags: CommandFlags::GENERAL,
    },
    // Game mechanics commands
    ExtendedCommand {
        name: "conduct",
        description: "show current conducts",
        command: Command::Discoveries,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "time",
        description: "show current time",
        command: Command::Help,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "travel",
        description: "travel mode",
        command: Command::Travel,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "autoexplore",
        description: "auto-explore level",
        command: Command::Move(Direction::North),
        flags: CommandFlags::EXPLORE_MODE,
    },
    // Accessibility/Meta commands
    ExtendedCommand {
        name: "options",
        description: "show game options",
        command: Command::Options,
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "open",
        description: "open a door or container",
        command: Command::Open(Direction::North),
        flags: CommandFlags::GENERAL,
    },
    ExtendedCommand {
        name: "close",
        description: "close a door or container",
        command: Command::Close(Direction::North),
        flags: CommandFlags::GENERAL,
    },
    // Wizard mode commands
    ExtendedCommand {
        name: "setwiz",
        description: "enter wizard mode",
        command: Command::Options,
        flags: CommandFlags::WIZARD_MODE,
    },
    ExtendedCommand {
        name: "terrain",
        description: "show terrain info",
        command: Command::Help,
        flags: CommandFlags::WIZARD_MODE,
    },
    ExtendedCommand {
        name: "teleport",
        description: "teleport to a location",
        command: Command::Travel,
        flags: CommandFlags::WIZARD_MODE,
    },
    ExtendedCommand {
        name: "polymorph",
        description: "polymorph into a creature",
        command: Command::Help,
        flags: CommandFlags::WIZARD_MODE,
    },
    ExtendedCommand {
        name: "wish",
        description: "wish for an item",
        command: Command::Help,
        flags: CommandFlags::WIZARD_MODE,
    },
    ExtendedCommand {
        name: "explore",
        description: "explore the level",
        command: Command::Help,
        flags: CommandFlags::WIZARD_MODE,
    },
];

/// Get all extended commands
pub fn extended_commands() -> &'static [ExtendedCommand] {
    EXTENDED_COMMANDS_LIST
}

/// Get extended commands for a specific category
pub fn extended_commands_by_category(category: &str) -> Vec<&'static ExtendedCommand> {
    EXTENDED_COMMANDS_LIST
        .iter()
        .filter(|cmd| match category {
            "general" => cmd.flags.contains(CommandFlags::GENERAL),
            "wizard" => cmd.flags.contains(CommandFlags::WIZARD_MODE),
            "explore" => cmd.flags.contains(CommandFlags::EXPLORE_MODE),
            "buried" => cmd.flags.contains(CommandFlags::IF_BURIED),
            _ => false,
        })
        .collect()
}

/// Get all general commands (non-wizard mode)
pub fn general_commands() -> Vec<&'static ExtendedCommand> {
    EXTENDED_COMMANDS_LIST
        .iter()
        .filter(|cmd| {
            cmd.flags.contains(CommandFlags::GENERAL)
                && !cmd.flags.contains(CommandFlags::WIZARD_MODE)
        })
        .collect()
}

/// Get all wizard mode commands
pub fn wizard_commands() -> Vec<&'static ExtendedCommand> {
    EXTENDED_COMMANDS_LIST
        .iter()
        .filter(|cmd| cmd.flags.contains(CommandFlags::WIZARD_MODE))
        .collect()
}

/// Search for commands by partial name match
pub fn search_commands(query: &str) -> Vec<&'static ExtendedCommand> {
    let lower_query = query.to_lowercase();
    EXTENDED_COMMANDS_LIST
        .iter()
        .filter(|cmd| cmd.name.contains(&lower_query) || cmd.description.contains(&lower_query))
        .collect()
}

/// Get command by name (exact match)
pub fn get_command(name: &str) -> Option<&'static ExtendedCommand> {
    EXTENDED_COMMANDS_LIST
        .iter()
        .find(|cmd| cmd.name.eq_ignore_ascii_case(name))
}

/// Get a list of command names by category
pub fn command_names_by_category(category: &str) -> Vec<&'static str> {
    extended_commands_by_category(category)
        .iter()
        .map(|cmd| cmd.name)
        .collect()
}

/// Execute an extended command by name
/// Returns Some(Command) if found, None otherwise
pub fn doextcmd(command_name: &str) -> Option<Command> {
    let normalized = command_name.to_lowercase();

    for cmd in EXTENDED_COMMANDS_LIST {
        if cmd.name.eq_ignore_ascii_case(&normalized) {
            return Some(cmd.command.clone());
        }
    }
    None
}

/// Get list of all extended command names
pub fn doextlist() -> Vec<&'static str> {
    EXTENDED_COMMANDS_LIST.iter().map(|c| c.name).collect()
}

/// Get a random extended command
pub fn rnd_extcmd_idx(rng: &mut GameRng) -> Option<usize> {
    if EXTENDED_COMMANDS_LIST.is_empty() {
        None
    } else {
        let idx = rng.rn2(EXTENDED_COMMANDS_LIST.len() as u32) as usize;
        Some(idx)
    }
}

/// Get command description from key/name
pub fn key2extcmddesc(key: &str) -> Option<&'static str> {
    for cmd in EXTENDED_COMMANDS_LIST {
        if cmd.name.eq_ignore_ascii_case(key) {
            return Some(cmd.description);
        }
    }
    None
}

/// Convert a command key to text representation
pub fn key2txt(key: u32) -> String {
    match key {
        0..=26 => format!("Ctrl-{}", ((b'A' + key as u8) as char)),
        27..=31 => format!("Ctrl-{}", (b'A' + (key as u8 - 27) as u8) as char),
        127 => "Delete".to_string(),
        32..=126 => (key as u8 as char).to_string(),
        _ => format!("key-{}", key),
    }
}

/// Parse text representation to key code
pub fn txt2key(text: &str) -> Option<u32> {
    if text.len() == 1 {
        Some(text.as_bytes()[0] as u32)
    } else if text.starts_with("Ctrl-") && text.len() == 6 {
        let ch = text.chars().nth(5)?;
        if ch.is_uppercase() {
            Some((ch as u32) - (b'A' as u32))
        } else {
            None
        }
    } else if text == "Delete" {
        Some(127)
    } else {
        None
    }
}

/// Display a menu for selecting extended commands
/// Returns the selected command name
pub fn extcmd_via_menu() -> Option<String> {
    // This would typically show a UI menu
    // For now, returns None to indicate no selection
    None
}

/// Enhanced menu for extended command list
pub fn hmenu_doextlist() -> Option<String> {
    extcmd_via_menu()
}

/// Menu for viewing version via menu
pub fn hmenu_doextversion() -> Option<String> {
    Some("version".to_string())
}

/// Menu for viewing history via menu
pub fn hmenu_dohistory() -> Option<String> {
    Some("history".to_string())
}

/// Menu for "what does this do?" feature
pub fn hmenu_dowhatdoes(key: u32) -> Option<String> {
    let text = key2txt(key);
    // Return description of what this key does
    Some(format!("Key {} description", text))
}

/// Menu for "what is?" feature
pub fn hmenu_dowhatis(subject: &str) -> Option<String> {
    Some(format!("Information about: {}", subject))
}

/// Handle command prefix (numbers for repeat, etc.)
pub fn prefix_cmd(_count: u32, base_cmd: Command) -> Command {
    // This would handle repeated commands
    // For now, just return the base command
    base_cmd
}

/// Adjust prefix value (for numeric prefixes)
pub fn adjust_prefix(current: u32, delta: i32) -> u32 {
    let result = current as i32 + delta;
    if result < 0 { 0 } else { result as u32 }
}

/// Process menu prefix input
pub fn accept_menu_prefix(prefix: &str) -> Option<u32> {
    prefix.parse::<u32>().ok()
}

// ============================================================================
// Wizard Mode Commands
// ============================================================================

/// Wizard mode command types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WizardCommand {
    /// Teleport to a specific level
    Teleport { dungeon: String, level: i32 },
    /// Polymorph into a creature
    Polymorph { creature: String },
    /// Wish for an item
    Wish { item: String },
    /// Show terrain information at a location
    ShowTerrain,
    /// List all monsters on the level
    ListMonsters,
    /// Modify dungeon (create/remove features)
    EditDungeon,
    /// Increment experience/levels
    GainLevel { amount: i32 },
    /// Set player attributes
    SetAttribute { attribute: String, value: i32 },
    /// Reveal full map
    RevealMap,
    /// Remove all traps
    RemoveTraps,
    /// Genocide a monster type
    Genocide { monster: String },
}

/// Check if a command is wizard-only
pub fn is_wizard_command(command_name: &str) -> bool {
    EXTENDED_COMMANDS_LIST
        .iter()
        .find(|cmd| cmd.name.eq_ignore_ascii_case(command_name))
        .map_or(false, |cmd| cmd.flags.contains(CommandFlags::WIZARD_MODE))
}

/// Get all available wizard commands
pub fn wizard_command_list() -> Vec<&'static str> {
    EXTENDED_COMMANDS_LIST
        .iter()
        .filter(|cmd| cmd.flags.contains(CommandFlags::WIZARD_MODE))
        .map(|cmd| cmd.name)
        .collect()
}

/// Parse wizard command syntax
pub fn parse_wizard_command(input: &str) -> Option<WizardCommand> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    match parts[0] {
        "teleport" => {
            if parts.len() >= 3 {
                let dungeon = parts[1].to_string();
                let level = parts[2].parse::<i32>().ok()?;
                Some(WizardCommand::Teleport { dungeon, level })
            } else {
                None
            }
        }
        "polymorph" => {
            if parts.len() >= 2 {
                let creature = parts[1..].join(" ");
                Some(WizardCommand::Polymorph { creature })
            } else {
                None
            }
        }
        "wish" => {
            if parts.len() >= 2 {
                let item = parts[1..].join(" ");
                Some(WizardCommand::Wish { item })
            } else {
                None
            }
        }
        "terrain" => Some(WizardCommand::ShowTerrain),
        "monsters" => Some(WizardCommand::ListMonsters),
        "edit" => Some(WizardCommand::EditDungeon),
        "level" => {
            if parts.len() >= 2 {
                let amount = parts[1].parse::<i32>().ok()?;
                Some(WizardCommand::GainLevel { amount })
            } else {
                None
            }
        }
        "set" => {
            if parts.len() >= 3 {
                let attribute = parts[1].to_string();
                let value = parts[2].parse::<i32>().ok()?;
                Some(WizardCommand::SetAttribute { attribute, value })
            } else {
                None
            }
        }
        "reveal" => Some(WizardCommand::RevealMap),
        "cleartraps" => Some(WizardCommand::RemoveTraps),
        "genocide" => {
            if parts.len() >= 2 {
                let monster = parts[1..].join(" ");
                Some(WizardCommand::Genocide { monster })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Get wizard command help text
pub fn wizard_command_help(cmd_name: &str) -> Option<&'static str> {
    match cmd_name {
        "teleport" => Some("Usage: teleport <dungeon> <level> - Jump to a specific dungeon level"),
        "polymorph" => Some("Usage: polymorph <creature> - Transform into any creature"),
        "wish" => Some("Usage: wish <item> - Create any desired item"),
        "terrain" => Some("Usage: terrain - Show terrain information at current location"),
        "monsters" => Some("Usage: monsters - List all monsters currently on the level"),
        "edit" => Some("Usage: edit - Enter dungeon editing mode"),
        "level" => Some("Usage: level <amount> - Gain specified number of experience levels"),
        "set" => Some("Usage: set <attribute> <value> - Set a player attribute (str, dex, etc.)"),
        "reveal" => Some("Usage: reveal - Reveal the entire map"),
        "cleartraps" => Some("Usage: cleartraps - Remove all traps from current level"),
        "genocide" => Some("Usage: genocide <monster> - Eliminate all of a monster type"),
        _ => None,
    }
}

// ============================================================================
// Mode-Based Interactions
// ============================================================================

/// Game modes that affect player interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    /// Normal exploration mode
    Normal,
    /// Interactive command selection mode
    Interactive,
    /// Exploration auto-mode
    Explore,
    /// Targeting mode for ranged actions
    Targeting,
    /// Menu-based selection mode
    Menu,
    /// Travel mode between destinations
    Travel,
}

impl GameMode {
    /// Get a description of the current mode
    pub fn description(&self) -> &'static str {
        match self {
            GameMode::Normal => "Normal mode - direct command input",
            GameMode::Interactive => "Interactive mode - menu-driven actions",
            GameMode::Explore => "Explore mode - automatic level exploration",
            GameMode::Targeting => "Targeting mode - select target location",
            GameMode::Menu => "Menu mode - browse available options",
            GameMode::Travel => "Travel mode - point-to-point navigation",
        }
    }

    /// Check if mode requires special input handling
    pub fn requires_special_input(&self) -> bool {
        matches!(
            self,
            GameMode::Targeting | GameMode::Menu | GameMode::Travel
        )
    }
}

/// Interactive mode state
#[derive(Debug, Clone)]
pub struct InteractiveMode {
    /// Current mode
    pub mode: GameMode,
    /// Previously active mode (for mode stacking)
    pub previous_mode: Option<Box<InteractiveMode>>,
    /// Current selection/focus
    pub selection: Option<String>,
    /// Available options in current menu
    pub options: Vec<String>,
}

impl InteractiveMode {
    /// Create new interactive mode with given game mode
    pub fn new(mode: GameMode) -> Self {
        Self {
            mode,
            previous_mode: None,
            selection: None,
            options: Vec::new(),
        }
    }

    /// Push a new mode on the stack
    pub fn push_mode(&mut self, new_mode: GameMode) {
        let current = std::mem::replace(self, Self::new(new_mode));
        self.previous_mode = Some(Box::new(current));
    }

    /// Pop back to previous mode
    pub fn pop_mode(&mut self) -> bool {
        if let Some(prev) = self.previous_mode.take() {
            *self = *prev;
            true
        } else {
            false
        }
    }

    /// Set available options for menu mode
    pub fn set_options(&mut self, options: Vec<String>) {
        self.options = options;
        self.selection = self.options.first().cloned();
    }

    /// Move selection to next option
    pub fn select_next(&mut self) {
        if let Some(ref current) = self.selection {
            if let Some(pos) = self.options.iter().position(|o| o == current) {
                if pos + 1 < self.options.len() {
                    self.selection = Some(self.options[pos + 1].clone());
                }
            }
        }
    }

    /// Move selection to previous option
    pub fn select_previous(&mut self) {
        if let Some(ref current) = self.selection {
            if let Some(pos) = self.options.iter().position(|o| o == current) {
                if pos > 0 {
                    self.selection = Some(self.options[pos - 1].clone());
                }
            }
        }
    }
}

/// Get list of commands available in interactive mode
pub fn interactive_mode_commands() -> Vec<&'static ExtendedCommand> {
    EXTENDED_COMMANDS_LIST
        .iter()
        .filter(|cmd| cmd.flags.contains(CommandFlags::GENERAL))
        .collect()
}

/// Get list of commands available in explore mode
pub fn explore_mode_commands() -> Vec<&'static ExtendedCommand> {
    EXTENDED_COMMANDS_LIST
        .iter()
        .filter(|cmd| {
            cmd.flags.contains(CommandFlags::EXPLORE_MODE)
                || cmd.flags.contains(CommandFlags::GENERAL)
        })
        .collect()
}

/// Organize commands by category for menu presentation
pub fn commands_by_category() -> Vec<(&'static str, Vec<&'static ExtendedCommand>)> {
    let categories = vec![
        (
            "Movement & Navigation",
            vec!["travel", "autoexplore", "jump"],
        ),
        ("Combat", vec!["rush"]),
        ("Object Interaction", vec!["dip", "identify"]),
        (
            "Information",
            vec!["whatis", "discoveries", "monsters", "spells", "conduct"],
        ),
        ("Social", vec!["chat", "pay", "pray", "offer"]),
        ("Meta", vec!["help", "version", "save", "quit"]),
    ];

    categories
        .into_iter()
        .map(|(category, names)| {
            let cmds = EXTENDED_COMMANDS_LIST
                .iter()
                .filter(|cmd| names.contains(&cmd.name))
                .collect();
            (category, cmds)
        })
        .collect()
}

/// Format commands for menu display
pub fn format_command_menu() -> String {
    let mut result = String::from("=== Extended Commands ===\n\n");

    for (category, commands) in commands_by_category() {
        result.push_str(&format!("{}:\n", category));
        for cmd in commands {
            result.push_str(&format!("  {} - {}\n", cmd.name, cmd.description));
        }
        result.push('\n');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doextcmd() {
        let cmd = doextcmd("version");
        assert!(cmd.is_some());
    }

    #[test]
    fn test_doextcmd_case_insensitive() {
        let cmd1 = doextcmd("version");
        let cmd2 = doextcmd("VERSION");
        assert_eq!(cmd1.is_some(), cmd2.is_some());
    }

    #[test]
    fn test_doextcmd_unknown() {
        let cmd = doextcmd("unknowncommand");
        assert!(cmd.is_none());
    }

    #[test]
    fn test_doextlist() {
        let list = doextlist();
        assert!(!list.is_empty());
        assert!(list.contains(&"version"));
    }

    #[test]
    fn test_key2txt() {
        assert_eq!(key2txt(65), "A");
        assert_eq!(key2txt(127), "Delete");
    }

    #[test]
    fn test_txt2key() {
        assert_eq!(txt2key("A"), Some(65));
        assert_eq!(txt2key("Delete"), Some(127));
    }

    #[test]
    fn test_txt2key_ctrl() {
        assert_eq!(txt2key("Ctrl-A"), Some(0));
    }

    #[test]
    fn test_adjust_prefix() {
        assert_eq!(adjust_prefix(5, 3), 8);
        assert_eq!(adjust_prefix(2, -5), 0); // Clamps to 0
    }

    #[test]
    fn test_accept_menu_prefix() {
        assert_eq!(accept_menu_prefix("10"), Some(10));
        assert_eq!(accept_menu_prefix("abc"), None);
    }
}
