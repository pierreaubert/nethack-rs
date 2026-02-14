//! Wizard kit file parsing functions translated from NetHack files.c
//!
//! Provides functions for reading and processing wizard kit configuration files
//! that define starting inventory and equipment for wizard mode.

use crate::world::errors::FileError;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Represents a single item in a wizard kit
#[derive(Debug, Clone)]
pub struct WizardKitItem {
    pub item_name: String,
    pub quantity: u32,
    pub enchantment: Option<i32>,
    pub note: Option<String>,
}

/// Represents a complete wizard kit configuration
#[derive(Debug, Clone, Default)]
pub struct WizardKit {
    pub items: Vec<WizardKitItem>,
}

impl WizardKit {
    /// Create a new empty wizard kit
    pub fn new() -> Self {
        WizardKit::default()
    }

    /// Add an item to the kit
    pub fn add_item(&mut self, item: WizardKitItem) {
        self.items.push(item);
    }

    /// Get total number of items
    pub fn item_count(&self) -> usize {
        self.items.len()
    }
}

/// Process a single line from a wizkit file
/// Format: "item_name quantity [enchantment] [note]"
/// Examples:
///   "knife 1"
///   "potion of healing 2 +1"
///   "amulet of life saving 1 ~ magical device"
pub fn proc_wizkit_line(line: &str, kit: &mut WizardKit) -> Result<(), String> {
    // Skip empty lines and comments
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(());
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }

    // Parse item name (can be multiple words until we hit a number or flag)
    let mut name_parts = Vec::new();
    let mut idx = 0;

    for (i, part) in parts.iter().enumerate() {
        // Stop at numeric quantity
        if part.chars().all(|c| c.is_numeric()) {
            idx = i;
            break;
        }
        name_parts.push(*part);
    }

    if name_parts.is_empty() {
        return Err("No item name found".to_string());
    }

    let item_name = name_parts.join(" ");

    // Parse quantity (default 1)
    let mut quantity = 1u32;
    let mut enchantment = None;
    let mut note = None;

    if idx < parts.len() {
        if let Ok(q) = parts[idx].parse::<u32>() {
            quantity = q;
            idx += 1;
        }
    }

    // Parse enchantment (starts with + or -)
    if idx < parts.len() {
        let part = parts[idx];
        if part.starts_with('+') || part.starts_with('-') {
            if let Ok(e) = part.parse::<i32>() {
                enchantment = Some(e);
                idx += 1;
            }
        }
    }

    // Parse note (everything after ~ marker)
    if idx < parts.len() {
        if parts[idx] == "~" {
            idx += 1;
            if idx < parts.len() {
                note = Some(parts[idx..].join(" "));
            }
        }
    }

    let item = WizardKitItem {
        item_name,
        quantity,
        enchantment,
        note,
    };

    kit.add_item(item);
    Ok(())
}

/// Read and parse a wizard kit configuration file
/// Returns a WizardKit struct containing all items defined in the file
pub fn read_wizkit(file_path: &Path) -> Result<WizardKit, FileError> {
    let file = std::fs::File::open(file_path).map_err(|e| FileError::CouldNotOpen {
        path: file_path.to_string_lossy().to_string(),
        reason: e.to_string(),
    })?;

    let reader = BufReader::new(file);
    let mut kit = WizardKit::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(|e| FileError::CouldNotOpen {
            path: file_path.to_string_lossy().to_string(),
            reason: format!("Error reading line {}: {}", line_num + 1, e),
        })?;

        if let Err(parse_err) = proc_wizkit_line(&line, &mut kit) {
            eprintln!(
                "Warning: Error parsing wizkit line {}: {}",
                line_num + 1,
                parse_err
            );
        }
    }

    Ok(kit)
}

/// Add an item from wizkit to player inventory
/// This is a bridge function that would be called by the game loop
/// to actually add items to the player's inventory based on kit definition
///
/// In a full implementation, this would:
/// 1. Look up the item by name in the object system
/// 2. Create an instance with proper stats/enchantments
/// 3. Add to player inventory
pub fn wizkit_addinv(
    item: &WizardKitItem,
    inventory: &mut Vec<String>, // Simplified - would be game object type
) -> Result<(), String> {
    // Validate item exists in game data
    // This is simplified - in real implementation would validate against object DB

    if item.item_name.is_empty() {
        return Err("Item name is empty".to_string());
    }

    if item.quantity == 0 {
        return Err("Quantity must be > 0".to_string());
    }

    // Add item to inventory
    for _ in 0..item.quantity {
        let entry = if let Some(ench) = item.enchantment {
            format!("{} ({})", item.item_name, ench)
        } else {
            item.item_name.clone()
        };
        inventory.push(entry);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizardkit_new() {
        let kit = WizardKit::new();
        assert_eq!(kit.item_count(), 0);
    }

    #[test]
    fn test_wizardkit_add_item() {
        let mut kit = WizardKit::new();
        let item = WizardKitItem {
            item_name: "knife".to_string(),
            quantity: 1,
            enchantment: None,
            note: None,
        };

        kit.add_item(item);
        assert_eq!(kit.item_count(), 1);
    }

    #[test]
    fn test_proc_wizkit_line_simple() {
        let mut kit = WizardKit::new();
        let result = proc_wizkit_line("knife 1", &mut kit);
        assert!(result.is_ok());
        assert_eq!(kit.item_count(), 1);
        assert_eq!(kit.items[0].item_name, "knife");
        assert_eq!(kit.items[0].quantity, 1);
    }

    #[test]
    fn test_proc_wizkit_line_multiword() {
        let mut kit = WizardKit::new();
        let result = proc_wizkit_line("potion of healing 2", &mut kit);
        assert!(result.is_ok());
        assert_eq!(kit.item_count(), 1);
        assert_eq!(kit.items[0].item_name, "potion of healing");
        assert_eq!(kit.items[0].quantity, 2);
    }

    #[test]
    fn test_proc_wizkit_line_with_enchantment() {
        let mut kit = WizardKit::new();
        let result = proc_wizkit_line("sword 1 +3", &mut kit);
        assert!(result.is_ok());
        assert_eq!(kit.item_count(), 1);
        assert_eq!(kit.items[0].enchantment, Some(3));
    }

    #[test]
    fn test_proc_wizkit_line_with_note() {
        let mut kit = WizardKit::new();
        let result = proc_wizkit_line("wand of death 1 ~ magical device", &mut kit);
        assert!(result.is_ok());
        assert_eq!(kit.item_count(), 1);
        assert_eq!(kit.items[0].note, Some("magical device".to_string()));
    }

    #[test]
    fn test_proc_wizkit_line_comment() {
        let mut kit = WizardKit::new();
        let result = proc_wizkit_line("# This is a comment", &mut kit);
        assert!(result.is_ok());
        assert_eq!(kit.item_count(), 0);
    }

    #[test]
    fn test_proc_wizkit_line_empty() {
        let mut kit = WizardKit::new();
        let result = proc_wizkit_line("", &mut kit);
        assert!(result.is_ok());
        assert_eq!(kit.item_count(), 0);
    }

    #[test]
    fn test_wizkit_addinv_simple() {
        let item = WizardKitItem {
            item_name: "knife".to_string(),
            quantity: 1,
            enchantment: None,
            note: None,
        };

        let mut inventory = Vec::new();
        let result = wizkit_addinv(&item, &mut inventory);
        assert!(result.is_ok());
        assert_eq!(inventory.len(), 1);
    }

    #[test]
    fn test_wizkit_addinv_multiple() {
        let item = WizardKitItem {
            item_name: "potion of healing".to_string(),
            quantity: 3,
            enchantment: None,
            note: None,
        };

        let mut inventory = Vec::new();
        let result = wizkit_addinv(&item, &mut inventory);
        assert!(result.is_ok());
        assert_eq!(inventory.len(), 3);
    }

    #[test]
    fn test_wizkit_addinv_with_enchantment() {
        let item = WizardKitItem {
            item_name: "sword".to_string(),
            quantity: 1,
            enchantment: Some(3),
            note: None,
        };

        let mut inventory = Vec::new();
        let result = wizkit_addinv(&item, &mut inventory);
        assert!(result.is_ok());
        assert_eq!(inventory.len(), 1);
        assert_eq!(inventory[0], "sword (3)");
    }
}
