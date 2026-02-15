use serde::{Deserialize, Serialize};
use nh_core::object::{Material, ObjectClass};

/// Defines the visual representation of an item for different frontends.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemIconDefinition {
    /// Character used in TUI map and menus.
    pub tui_char: char,
    /// Color name or hex code for TUI (e.g., "yellow", "red", "#FF0000").
    pub tui_color: String,
    /// Path to the sprite or texture asset for Bevy/Graphical UI.
    pub bevy_sprite: String,
}

/// A flexible identifier used to match NetHack items to their icon definitions.
///
/// Mappings can be broad (e.g., just `class`) or specific (e.g., `object_type` + `material`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ItemIdentifier {
    /// The base object class (e.g., Weapon, Potion).
    pub class: Option<ObjectClass>,
    /// The specific object type index (from nh-core).
    pub object_type: Option<i16>,
    /// The material of the item.
    pub material: Option<Material>,
    /// Whether the item is identified. Some icons may change when identified.
    pub is_identified: Option<bool>,
    /// Artifact ID (0 if not an artifact).
    pub artifact: Option<u8>,
    /// Monster type index (for corpses, figurines, statues, eggs).
    pub corpse_type: Option<i16>,
}

/// Links an item identifier to its icon definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetMappingEntry {
    pub identifier: ItemIdentifier,
    pub icon: ItemIconDefinition,
}

/// The root structure for the asset mapping configuration file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AssetMapping {
    pub mappings: Vec<AssetMappingEntry>,
}

impl ItemIdentifier {
    /// Check how many fields are set in this identifier.
    /// Used for prioritizing more specific matches.
    pub fn specificity(&self) -> u32 {
        let mut count = 0;
        if self.class.is_some() { count += 1; }
        if self.object_type.is_some() { count += 10; } // object_type is much more specific
        if self.material.is_some() { count += 2; }
        if self.is_identified.is_some() { count += 1; }
        if self.artifact.is_some() { count += 20; } // artifact is highly specific
        if self.corpse_type.is_some() { count += 5; }
        count
    }
}
