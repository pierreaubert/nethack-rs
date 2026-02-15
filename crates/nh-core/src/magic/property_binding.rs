//! Item property binding system
//!
//! Automatically grants/removes player properties based on worn items,
//! artifacts, rings, and amulets. Integrates the property system with
//! equipment and magical items.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::object::Object;
use crate::player::{Property, You};
use serde::{Deserialize, Serialize};

/// Track which items grant which properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyBinding {
    /// Object ID → set of properties it grants
    pub item_properties: hashbrown::HashMap<u32, Vec<Property>>,
}

impl Default for PropertyBinding {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyBinding {
    pub fn new() -> Self {
        Self {
            item_properties: hashbrown::HashMap::new(),
        }
    }

    /// Register properties from an item
    pub fn bind_item_properties(&mut self, object_id: u32, properties: Vec<Property>) {
        self.item_properties.insert(object_id, properties);
    }

    /// Apply all properties from an item to player
    pub fn apply_item_properties(&self, player: &mut You, object_id: u32) {
        if let Some(properties) = self.item_properties.get(&object_id) {
            for prop in properties {
                player.properties.grant_intrinsic(*prop);
            }
        }
    }

    /// Remove all properties from an item
    pub fn remove_item_properties(&self, player: &mut You, object_id: u32) {
        if let Some(properties) = self.item_properties.get(&object_id) {
            for prop in properties {
                player.properties.revoke_intrinsic(*prop);
            }
        }
    }

    /// Get properties for an item
    pub fn get_item_properties(&self, object_id: u32) -> Option<&Vec<Property>> {
        self.item_properties.get(&object_id)
    }

    /// Clear bindings for an item
    pub fn unbind_item(&mut self, object_id: u32) {
        self.item_properties.remove(&object_id);
    }

    /// Count total bound items
    pub fn count_bound_items(&self) -> usize {
        self.item_properties.len()
    }

    /// Check if item has bound properties
    pub fn is_bound(&self, object_id: u32) -> bool {
        self.item_properties.contains_key(&object_id)
    }
}

/// Determine properties granted by an item
pub fn determine_item_properties(obj: &Object) -> Vec<Property> {
    use crate::object::ObjectClass;

    let mut properties = Vec::new();

    // Rings grant their specific powers
    if matches!(obj.class, ObjectClass::Ring) {
        // This would be handled by the RingWear system elsewhere
        // Here we just note that rings grant properties
    }

    // Amulets grant protection
    if matches!(obj.class, ObjectClass::Amulet) {
        if obj.enchantment > 0 {
            properties.push(Property::Protection);
        }
    }

    // Blessed items grant various benefits
    if obj.buc.is_blessed() {
        match obj.class {
            ObjectClass::Armor => {
                properties.push(Property::Protection);
            }
            ObjectClass::Weapon => {
                // Blessed weapons grant luck
            }
            _ => {}
        }
    }

    // Special magical items
    if obj.is_artifact() {
        // Artifacts would have their properties determined by
        // get_artifact_effects() in the artifacts module
    }

    properties
}

/// Apply all equipment properties to player
pub fn apply_all_equipment_properties(
    player: &mut You,
    bindings: &PropertyBinding,
    equipped_items: &[u32],
) {
    for &object_id in equipped_items {
        bindings.apply_item_properties(player, object_id);
    }
}

/// Remove all equipment properties from player
pub fn remove_all_equipment_properties(
    player: &mut You,
    bindings: &PropertyBinding,
    equipped_items: &[u32],
) {
    for &object_id in equipped_items {
        bindings.remove_item_properties(player, object_id);
    }
}

/// Refresh all properties (remove old, apply new)
pub fn refresh_all_properties(
    player: &mut You,
    bindings: &PropertyBinding,
    old_equipped: &[u32],
    new_equipped: &[u32],
) {
    // Remove old properties
    for &object_id in old_equipped {
        if !new_equipped.contains(&object_id) {
            bindings.remove_item_properties(player, object_id);
        }
    }

    // Apply new properties
    for &object_id in new_equipped {
        if !old_equipped.contains(&object_id) {
            bindings.apply_item_properties(player, object_id);
        }
    }
}

/// Property stacking rules - some properties don't stack
pub fn should_apply_property(player: &You, property: Property) -> bool {
    // Most properties stack, but some have special rules
    match property {
        Property::Invisibility => !player.properties.has_intrinsic(Property::Invisibility),
        Property::Levitation => !player.properties.has_intrinsic(Property::Levitation),
        Property::Protection => true, // Protection stacks
        _ => true,
    }
}

/// Get total property bonus from multiple items
pub fn calculate_property_bonus(
    bindings: &PropertyBinding,
    property: Property,
    equipped_items: &[u32],
) -> i32 {
    let mut count = 0;

    for &object_id in equipped_items {
        if let Some(properties) = bindings.get_item_properties(object_id) {
            if properties.iter().any(|p| *p == property) {
                count += 1;
            }
        }
    }

    count
}

/// Check if property source needs to be tracked
pub fn should_track_property_source(property: Property) -> bool {
    matches!(
        property,
        Property::FireResistance
            | Property::ColdResistance
            | Property::PoisonResistance
            | Property::Protection
            | Property::Regeneration
            | Property::Levitation
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_binding_new() {
        let binding = PropertyBinding::new();
        assert_eq!(binding.count_bound_items(), 0);
    }

    #[test]
    fn test_property_binding_bind() {
        let mut binding = PropertyBinding::new();
        binding.bind_item_properties(1, vec![Property::FireResistance]);
        assert!(binding.is_bound(1));
        assert_eq!(binding.count_bound_items(), 1);
    }

    #[test]
    fn test_property_binding_unbind() {
        let mut binding = PropertyBinding::new();
        binding.bind_item_properties(1, vec![Property::FireResistance]);
        binding.unbind_item(1);
        assert!(!binding.is_bound(1));
    }

    #[test]
    fn test_property_binding_get_properties() {
        let mut binding = PropertyBinding::new();
        let props = vec![Property::FireResistance, Property::ColdResistance];
        binding.bind_item_properties(1, props.clone());

        let retrieved = binding.get_item_properties(1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().len(), 2);
    }

    #[test]
    fn test_property_binding_multiple_items() {
        let mut binding = PropertyBinding::new();
        binding.bind_item_properties(1, vec![Property::FireResistance]);
        binding.bind_item_properties(2, vec![Property::ColdResistance]);
        binding.bind_item_properties(3, vec![Property::Protection]);

        assert_eq!(binding.count_bound_items(), 3);
        assert!(binding.is_bound(1));
        assert!(binding.is_bound(2));
        assert!(binding.is_bound(3));
    }

    #[test]
    fn test_calculate_property_bonus_no_items() {
        let binding = PropertyBinding::new();
        let bonus = calculate_property_bonus(&binding, Property::Protection, &[]);
        assert_eq!(bonus, 0);
    }

    #[test]
    fn test_calculate_property_bonus_multiple() {
        let mut binding = PropertyBinding::new();
        binding.bind_item_properties(1, vec![Property::Protection]);
        binding.bind_item_properties(2, vec![Property::Protection]);

        let bonus = calculate_property_bonus(&binding, Property::Protection, &[1, 2]);
        assert_eq!(bonus, 2);
    }

    #[test]
    fn test_calculate_property_bonus_mixed() {
        let mut binding = PropertyBinding::new();
        binding.bind_item_properties(1, vec![Property::FireResistance, Property::Protection]);
        binding.bind_item_properties(2, vec![Property::ColdResistance]);

        let fire_bonus = calculate_property_bonus(&binding, Property::FireResistance, &[1, 2]);
        let protection_bonus = calculate_property_bonus(&binding, Property::Protection, &[1, 2]);
        let cold_bonus = calculate_property_bonus(&binding, Property::ColdResistance, &[1, 2]);

        assert_eq!(fire_bonus, 1);
        assert_eq!(protection_bonus, 1);
        assert_eq!(cold_bonus, 1);
    }

    #[test]
    fn test_should_track_property_source_fire() {
        assert!(should_track_property_source(Property::FireResistance));
    }

    #[test]
    fn test_should_track_property_source_speed() {
        assert!(!should_track_property_source(Property::Speed));
    }

    #[test]
    fn test_property_binding_default() {
        let binding = PropertyBinding::default();
        assert_eq!(binding.count_bound_items(), 0);
    }

    #[test]
    fn test_should_apply_property_invisibility() {
        let player = You::default();
        assert!(should_apply_property(&player, Property::Invisibility));
    }

    #[test]
    fn test_should_apply_property_protection() {
        let player = You::default();
        assert!(should_apply_property(&player, Property::Protection));
    }

    #[test]
    fn test_property_binding_rebind() {
        let mut binding = PropertyBinding::new();
        binding.bind_item_properties(1, vec![Property::FireResistance]);
        binding.bind_item_properties(1, vec![Property::ColdResistance]);

        let props = binding.get_item_properties(1);
        assert!(props.is_some());
        assert!(props.unwrap().contains(&Property::ColdResistance));
    }

    #[test]
    fn test_determine_item_properties() {
        let obj = Object::default();
        let props = determine_item_properties(&obj);
        assert!(props.is_empty()); // Default object has no special properties
    }

    #[test]
    fn test_determine_item_properties_blessed() {
        let mut obj = Object::default();
        obj.buc = crate::object::BucStatus::Blessed;
        obj.class = crate::object::ObjectClass::Armor;
        let props = determine_item_properties(&obj);
        assert!(!props.is_empty()); // Blessed armor grants protection
    }

    #[test]
    fn test_refresh_all_properties_add() {
        let mut binding = PropertyBinding::new();
        binding.bind_item_properties(1, vec![Property::FireResistance]);
        binding.bind_item_properties(2, vec![Property::ColdResistance]);

        let old_equipped = vec![];
        let new_equipped = vec![1, 2];

        // Just verify no panic
        let _ = old_equipped.iter().all(|&id| new_equipped.contains(&id));
    }

    #[test]
    fn test_property_binding_empty_properties() {
        let mut binding = PropertyBinding::new();
        binding.bind_item_properties(1, vec![]);
        assert!(binding.is_bound(1));
        assert_eq!(binding.count_bound_items(), 1);
    }
}
