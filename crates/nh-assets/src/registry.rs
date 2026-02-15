use std::path::Path;
use serde_json;
use nh_core::object::{Object, o_material};
use crate::mapping::{AssetMapping, AssetMappingEntry, ItemIconDefinition, ItemIdentifier};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Asset mapping not found for object: {0:?}")]
    NotFound(String),
}

/// A registry that maps game objects to their visual icons.
pub struct AssetRegistry {
    mapping: AssetMapping,
}

impl AssetRegistry {
    /// Create a new registry from an existing mapping.
    pub fn new(mapping: AssetMapping) -> Self {
        Self { mapping }
    }

    /// Load the registry from a JSON file and validate coverage.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, RegistryError> {
        let content = std::fs::read_to_string(path)?;
        let mapping: AssetMapping = serde_json::from_str(&content)?;
        let registry = Self::new(mapping);
        registry.validate_coverage()?;
        Ok(registry)
    }

    /// Validate that all basic object classes have at least one mapping.
    fn validate_coverage(&self) -> Result<(), RegistryError> {
        use nh_core::object::{ObjectClass, ObjectId, Object};
        use strum::IntoEnumIterator;

        for class in ObjectClass::iter() {
            if matches!(class, ObjectClass::Random | ObjectClass::IllObj) {
                continue;
            }
            
            let obj = Object::new(ObjectId(0), 0, class);
            if self.get_icon(&obj).is_err() {
                return Err(RegistryError::NotFound(format!("Missing mapping for class: {:?}", class)));
            }
        }
        Ok(())
    }

    /// Find the best matching icon for a given object.
    ///
    /// It iterates through all mappings and selects the one with the highest specificity
    /// that matches the object's properties.
    pub fn get_icon(&self, obj: &Object) -> Result<&ItemIconDefinition, RegistryError> {
        let mut best_match: Option<(&AssetMappingEntry, u32)> = None;

        for entry in &self.mapping.mappings {
            if self.matches(obj, &entry.identifier) {
                let specificity = entry.identifier.specificity();
                if best_match.is_none() || specificity > best_match.as_ref().unwrap().1 {
                    best_match = Some((entry, specificity));
                }
            }
        }

        best_match
            .map(|(entry, _)| &entry.icon)
            .ok_or_else(|| RegistryError::NotFound(format!("{:?}", obj)))
    }

    fn matches(&self, obj: &Object, id: &ItemIdentifier) -> bool {
        if let Some(class) = id.class {
            if obj.class != class { return false; }
        }
        if let Some(obj_type) = id.object_type {
            if obj.object_type != obj_type { return false; }
        }
        if let Some(mat) = id.material {
            if o_material(obj) != mat { return false; }
        }
        if let Some(is_id) = id.is_identified {
            // NetHack items are considered identified if known flag is set.
            // Some frontends might care about desc_known too.
            if obj.known != is_id { return false; }
        }
        if let Some(artifact) = id.artifact {
            if obj.artifact != artifact { return false; }
        }
        if let Some(corpse) = id.corpse_type {
            if obj.corpse_type != corpse { return false; }
        }
        true
    }

    /// Helper to convert a color string to a ratatui color.
    pub fn parse_color(color_name: &str) -> Option<ratatui::style::Color> {
        use ratatui::style::Color;
        match color_name.to_lowercase().as_str() {
            "black" => Some(Color::Black),
            "red" => Some(Color::Red),
            "green" => Some(Color::Green),
            "yellow" => Some(Color::Yellow),
            "blue" => Some(Color::Blue),
            "magenta" => Some(Color::Magenta),
            "cyan" => Some(Color::Cyan),
            "gray" => Some(Color::Gray),
            "darkgray" | "dark_gray" => Some(Color::DarkGray),
            "lightred" | "light_red" => Some(Color::LightRed),
            "lightgreen" | "light_green" => Some(Color::LightGreen),
            "lightyellow" | "light_yellow" => Some(Color::LightYellow),
            "lightblue" | "light_blue" => Some(Color::LightBlue),
            "lightmagenta" | "light_magenta" => Some(Color::LightMagenta),
            "lightcyan" | "light_cyan" => Some(Color::LightCyan),
            "white" => Some(Color::White),
            _ => {
                // Potential hex color parsing could go here
                None
            }
        }
    }
}
