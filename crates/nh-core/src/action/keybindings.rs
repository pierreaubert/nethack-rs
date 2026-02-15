//! Key binding management system translated from NetHack cmd.c
//!
//! Provides functions for:
//! - Displaying key bindings
//! - Managing custom key bindings
//! - Menu control key display
//! - Direction key display
//! - Special key handling (Shift, Alt, etc.)

#[cfg(not(feature = "std"))]
use crate::compat::*;

use super::extended::ExtendedCommand;
use hashbrown::HashMap;

/// Custom key binding configuration
#[derive(Debug, Clone, Default)]
pub struct KeyBindings {
    /// Map of command name to key
    pub bindings: HashMap<String, char>,
    /// Map of key to command name
    pub reverse_bindings: HashMap<char, String>,
}

impl KeyBindings {
    /// Create a new empty key bindings map
    pub fn new() -> Self {
        KeyBindings::default()
    }

    /// Bind a command to a key
    pub fn bind_key(&mut self, command: &str, key: char) -> Result<(), String> {
        if is_illegal_menu_cmd_key(key) {
            return Err(format!("Cannot bind key '{}' - illegal key", key));
        }

        // Remove old binding if exists
        if let Some(old_cmd) = self.reverse_bindings.get(&key) {
            self.bindings.remove(old_cmd);
        }

        self.bindings.insert(command.to_string(), key);
        self.reverse_bindings.insert(key, command.to_string());

        Ok(())
    }

    /// Get key for a command
    pub fn get_key(&self, command: &str) -> Option<char> {
        self.bindings.get(command).copied()
    }

    /// Get command for a key
    pub fn get_command(&self, key: char) -> Option<&str> {
        self.reverse_bindings.get(&key).map(|s| s.as_str())
    }

    /// Remove a key binding
    pub fn unbind_key(&mut self, key: char) -> Option<String> {
        if let Some(cmd) = self.reverse_bindings.remove(&key) {
            self.bindings.remove(&cmd);
            Some(cmd)
        } else {
            None
        }
    }
}

/// Display all key bindings
pub fn dokeylist() -> String {
    let mut output = String::from("Key bindings:\n");
    output.push_str("Movement: arrow keys or hjkl, y/u for diagonals\n");
    output.push_str("Commands: ?, /, :, ~, &\n");
    output.push_str("Equipment: w (wear), t (take off), P (put on), R (remove)\n");
    output.push_str("Actions: e (eat), d (drop), z (zap), a (apply)\n");
    output.push_str("Special: # (extended), space (rest), > (descend), < (ascend)\n");
    output
}

/// Helper to add commands to key list
pub fn dokeylist_putcmds(cmd_name: &str) -> String {
    format!("{}: see key bindings list\n", cmd_name)
}

/// Display menu controls
pub fn domenucontrols() -> String {
    let mut output = String::from("Menu controls:\n");
    output.push_str("arrow keys or hjkl - move selection\n");
    output.push_str("space or enter - select item\n");
    output.push_str("q or escape - cancel menu\n");
    output.push_str("* - select all\n");
    output.push_str("- - deselect all\n");
    output
}

/// Display menu key bindings
pub fn show_menu_controls() -> String {
    domenucontrols()
}

/// Display directional keys
pub fn show_direction_keys() -> String {
    let mut output = String::from("Directional keys:\n");
    output.push_str("7 8 9 or y k u\n");
    output.push_str("4 . 6 or h . l\n");
    output.push_str("1 2 3 or b j n\n");
    output.push_str("5 or . - stay in place\n");
    output.push_str("< > - up/down stairs\n");
    output
}

/// Bind a command to a key
pub fn bind_key(bindings: &mut KeyBindings, command: &str, key: char) -> Result<(), String> {
    bindings.bind_key(command, key)
}

/// Bind a special key (with modifiers like Shift, Alt, Ctrl)
pub fn bind_specialkey(
    bindings: &mut KeyBindings,
    command: &str,
    base_key: char,
    shift: bool,
    alt: bool,
    ctrl: bool,
) -> Result<(), String> {
    // Create a modified key representation
    let mut key_str = String::new();

    if ctrl {
        key_str.push_str("Ctrl-");
    }
    if alt {
        key_str.push_str("Alt-");
    }
    if shift {
        key_str.push_str("Shift-");
    }

    key_str.push(base_key);

    // For now, use the first character as the actual key
    // In a real system, this would need more sophisticated key encoding
    bindings.bind_key(command, base_key)?;

    Ok(())
}

/// Convert character to special keys
pub fn ch2spkeys(ch: char) -> Vec<(char, bool, bool, bool)> {
    // Returns (key, shift, alt, ctrl)
    vec![(ch, false, false, false)]
}

/// Add a menu command alias
pub fn add_menu_cmd_alias(
    bindings: &mut KeyBindings,
    original: &str,
    alias: &str,
) -> Result<(), String> {
    if let Some(key) = bindings.get_key(original) {
        bindings.bind_key(alias, key)?;
        Ok(())
    } else {
        Err(format!("Original command '{}' not bound", original))
    }
}

/// Get key for a menu command
pub fn get_menu_cmd_key(bindings: &KeyBindings, command: &str) -> Option<char> {
    bindings.get_key(command)
}

/// Map a command to its function
pub fn map_menu_cmd(command_name: &str) -> Option<String> {
    match command_name {
        "help" => Some("Show help".to_string()),
        "inventory" => Some("Show inventory".to_string()),
        "quit" => Some("Quit game".to_string()),
        _ => None,
    }
}

/// Check if a key binding is illegal
pub fn is_illegal_menu_cmd_key(key: char) -> bool {
    match key {
        // Control characters that should be reserved
        '\0'..='\x1f' => true,
        // DEL
        '\x7f' => true,
        // Reserved special keys
        '?' | '/' | ':' => true,
        _ => false,
    }
}

/// Check if a key can be bound to menu commands
pub fn illegal_menu_cmd_key(key: char) -> bool {
    is_illegal_menu_cmd_key(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybindings_new() {
        let kb = KeyBindings::new();
        assert_eq!(kb.bindings.len(), 0);
    }

    #[test]
    fn test_bind_key() {
        let mut kb = KeyBindings::new();
        let result = kb.bind_key("rest", ' ');
        assert!(result.is_ok());
        assert_eq!(kb.get_key("rest"), Some(' '));
    }

    #[test]
    fn test_get_command() {
        let mut kb = KeyBindings::new();
        kb.bind_key("rest", ' ').unwrap();
        assert_eq!(kb.get_command(' '), Some("rest"));
    }

    #[test]
    fn test_bind_illegal_key() {
        let mut kb = KeyBindings::new();
        let result = kb.bind_key("test", '\0');
        assert!(result.is_err());
    }

    #[test]
    fn test_unbind_key() {
        let mut kb = KeyBindings::new();
        kb.bind_key("rest", ' ').unwrap();
        let removed = kb.unbind_key(' ');
        assert_eq!(removed, Some("rest".to_string()));
        assert_eq!(kb.get_key("rest"), None);
    }

    #[test]
    fn test_dokeylist() {
        let list = dokeylist();
        assert!(list.contains("Movement"));
        assert!(list.contains("hjkl"));
    }

    #[test]
    fn test_domenucontrols() {
        let controls = domenucontrols();
        assert!(controls.contains("space"));
        assert!(controls.contains("escape"));
    }

    #[test]
    fn test_show_direction_keys() {
        let dirs = show_direction_keys();
        // Check that vi-keys are listed (h, j, k, l individually)
        assert!(dirs.contains("h . l"));
        assert!(dirs.contains("y k u"));
        assert!(dirs.contains("stairs"));
    }

    #[test]
    fn test_is_illegal_menu_cmd_key() {
        assert!(is_illegal_menu_cmd_key('\0'));
        assert!(is_illegal_menu_cmd_key('\x1f'));
        assert!(is_illegal_menu_cmd_key('?'));
        assert!(!is_illegal_menu_cmd_key('a'));
    }
}
