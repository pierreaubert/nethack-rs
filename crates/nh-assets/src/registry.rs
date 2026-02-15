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

    /// Load the registry from a JSON file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, RegistryError> {
        let content = std::fs::read_to_string(path)?;
        let mapping: AssetMapping = serde_json::from_str(&content)?;
        Ok(Self::new(mapping))
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
}
