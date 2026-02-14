//! Spell components system
//!
//! Tracks material components needed for spellcasting and manages component
//! consumption, availability, and spell failure due to missing components.

use serde::{Deserialize, Serialize};

/// Spell component types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    PhilotreesLeaf,
    ToadStool,
    PowderedCorpse,
    DiamondDust,
    FlamingCoal,
    FrozenWater,
    SilverDust,
    HerbOfHealing,
    MoonStonePowder,
    SulfurousDust,
}

impl ComponentType {
    /// Get component name
    pub fn name(&self) -> &'static str {
        match self {
            ComponentType::PhilotreesLeaf => "leaf of philotrees",
            ComponentType::ToadStool => "powdered toadstool",
            ComponentType::PowderedCorpse => "powdered corpse",
            ComponentType::DiamondDust => "diamond dust",
            ComponentType::FlamingCoal => "flaming coal",
            ComponentType::FrozenWater => "frozen water",
            ComponentType::SilverDust => "silver dust",
            ComponentType::HerbOfHealing => "herb of healing",
            ComponentType::MoonStonePowder => "moonstone powder",
            ComponentType::SulfurousDust => "sulfurous dust",
        }
    }

    /// Get component cost (components per use)
    pub fn cost(&self) -> i32 {
        match self {
            ComponentType::PhilotreesLeaf => 1,
            ComponentType::ToadStool => 1,
            ComponentType::PowderedCorpse => 1,
            ComponentType::DiamondDust => 2,
            ComponentType::FlamingCoal => 1,
            ComponentType::FrozenWater => 1,
            ComponentType::SilverDust => 1,
            ComponentType::HerbOfHealing => 1,
            ComponentType::MoonStonePowder => 2,
            ComponentType::SulfurousDust => 1,
        }
    }
}

/// Inventory of spell components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInventory {
    /// Available components by type
    pub components: std::collections::HashMap<ComponentType, i32>,
}

impl ComponentInventory {
    pub fn new() -> Self {
        Self {
            components: std::collections::HashMap::new(),
        }
    }

    /// Check if player has required components
    pub fn has_components(&self, required: &[(ComponentType, i32)]) -> bool {
        required.iter().all(|(comp_type, amount)| {
            self.components.get(comp_type).copied().unwrap_or(0) >= *amount
        })
    }

    /// Get component count
    pub fn get_component_count(&self, comp_type: ComponentType) -> i32 {
        self.components.get(&comp_type).copied().unwrap_or(0)
    }

    /// Add components
    pub fn add_component(&mut self, comp_type: ComponentType, amount: i32) {
        *self.components.entry(comp_type).or_insert(0) += amount;
    }

    /// Remove components (returns true if successful)
    pub fn remove_components(&mut self, comp_type: ComponentType, amount: i32) -> bool {
        let current = self.get_component_count(comp_type);
        if current >= amount {
            *self.components.entry(comp_type).or_insert(0) -= amount;
            true
        } else {
            false
        }
    }

    /// List available components
    pub fn list_available(&self) -> Vec<(ComponentType, i32)> {
        self.components.iter().map(|(&k, &v)| (k, v)).collect()
    }
}

impl Default for ComponentInventory {
    fn default() -> Self {
        Self::new()
    }
}

/// Spell requirement
#[derive(Debug, Clone, Copy)]
pub struct SpellRequirement {
    pub component_type: ComponentType,
    pub amount: i32,
    pub consumed: bool, // Whether component is consumed on use
}

/// Get components required for a spell
pub fn get_spell_components(spell_name: &str) -> Vec<SpellRequirement> {
    match spell_name {
        "force_bolt" => vec![SpellRequirement {
            component_type: ComponentType::SulfurousDust,
            amount: 1,
            consumed: true,
        }],
        "fireball" => vec![
            SpellRequirement {
                component_type: ComponentType::FlamingCoal,
                amount: 1,
                consumed: true,
            },
            SpellRequirement {
                component_type: ComponentType::SulfurousDust,
                amount: 1,
                consumed: true,
            },
        ],
        "cone_of_cold" => vec![SpellRequirement {
            component_type: ComponentType::FrozenWater,
            amount: 1,
            consumed: true,
        }],
        "healing" => vec![SpellRequirement {
            component_type: ComponentType::HerbOfHealing,
            amount: 1,
            consumed: true,
        }],
        "invisibility" => vec![SpellRequirement {
            component_type: ComponentType::PhilotreesLeaf,
            amount: 1,
            consumed: true,
        }],
        "levitation" => vec![SpellRequirement {
            component_type: ComponentType::MoonStonePowder,
            amount: 1,
            consumed: true,
        }],
        "teleport_away" => vec![SpellRequirement {
            component_type: ComponentType::DiamondDust,
            amount: 1,
            consumed: true,
        }],
        "turn_undead" => vec![SpellRequirement {
            component_type: ComponentType::SilverDust,
            amount: 1,
            consumed: true,
        }],
        _ => vec![],
    }
}

/// Check if spell can be cast (has components)
pub fn can_cast_with_components(
    inventory: &ComponentInventory,
    spell_name: &str,
) -> (bool, Vec<(ComponentType, i32)>) {
    let requirements = get_spell_components(spell_name);
    let needed: Vec<_> = requirements
        .iter()
        .map(|r| (r.component_type, r.amount))
        .collect();

    let can_cast = inventory.has_components(&needed);
    (can_cast, needed)
}

/// Consume spell components
pub fn consume_spell_components(inventory: &mut ComponentInventory, spell_name: &str) -> bool {
    let requirements = get_spell_components(spell_name);

    for req in &requirements {
        if req.consumed {
            if !inventory.remove_components(req.component_type, req.amount) {
                return false;
            }
        }
    }

    true
}

/// Get component failure message
pub fn missing_component_message(needed: &[(ComponentType, i32)]) -> String {
    if needed.is_empty() {
        "You lack the necessary components for this spell.".to_string()
    } else {
        let components = needed
            .iter()
            .map(|(ct, amount)| format!("{} {}", amount, ct.name()))
            .collect::<Vec<_>>()
            .join(", ");
        format!("You need: {}", components)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_inventory_add() {
        let mut inv = ComponentInventory::new();
        inv.add_component(ComponentType::HerbOfHealing, 5);

        assert_eq!(inv.get_component_count(ComponentType::HerbOfHealing), 5);
    }

    #[test]
    fn test_component_inventory_remove() {
        let mut inv = ComponentInventory::new();
        inv.add_component(ComponentType::HerbOfHealing, 5);

        let success = inv.remove_components(ComponentType::HerbOfHealing, 3);
        assert!(success);
        assert_eq!(inv.get_component_count(ComponentType::HerbOfHealing), 2);
    }

    #[test]
    fn test_component_inventory_remove_fails_insufficient() {
        let mut inv = ComponentInventory::new();
        inv.add_component(ComponentType::HerbOfHealing, 2);

        let success = inv.remove_components(ComponentType::HerbOfHealing, 5);
        assert!(!success);
        assert_eq!(inv.get_component_count(ComponentType::HerbOfHealing), 2);
    }

    #[test]
    fn test_has_components() {
        let mut inv = ComponentInventory::new();
        inv.add_component(ComponentType::HerbOfHealing, 3);
        inv.add_component(ComponentType::FlamingCoal, 2);

        let required = vec![
            (ComponentType::HerbOfHealing, 2),
            (ComponentType::FlamingCoal, 2),
        ];
        assert!(inv.has_components(&required));

        let required_too_much = vec![
            (ComponentType::HerbOfHealing, 4),
            (ComponentType::FlamingCoal, 1),
        ];
        assert!(!inv.has_components(&required_too_much));
    }

    #[test]
    fn test_can_cast_with_components() {
        let mut inv = ComponentInventory::new();
        inv.add_component(ComponentType::FlamingCoal, 1);
        inv.add_component(ComponentType::SulfurousDust, 1);

        let (can_cast, needed) = can_cast_with_components(&inv, "fireball");
        assert!(can_cast);
    }

    #[test]
    fn test_can_cast_missing_components() {
        let inv = ComponentInventory::new();

        let (can_cast, needed) = can_cast_with_components(&inv, "fireball");
        assert!(!can_cast);
        assert!(!needed.is_empty());
    }

    #[test]
    fn test_consume_spell_components() {
        let mut inv = ComponentInventory::new();
        inv.add_component(ComponentType::FlamingCoal, 1);
        inv.add_component(ComponentType::SulfurousDust, 1);

        let success = consume_spell_components(&mut inv, "fireball");
        assert!(success);
        assert_eq!(inv.get_component_count(ComponentType::FlamingCoal), 0);
        assert_eq!(inv.get_component_count(ComponentType::SulfurousDust), 0);
    }

    #[test]
    fn test_missing_component_message() {
        let needed = vec![
            (ComponentType::HerbOfHealing, 2),
            (ComponentType::FlamingCoal, 1),
        ];
        let msg = missing_component_message(&needed);

        assert!(msg.contains("herb of healing"));
        assert!(msg.contains("flaming coal"));
    }

    #[test]
    fn test_spell_requirements() {
        let fireball_reqs = get_spell_components("fireball");
        assert_eq!(fireball_reqs.len(), 2);

        let healing_reqs = get_spell_components("healing");
        assert_eq!(healing_reqs.len(), 1);
        assert_eq!(healing_reqs[0].component_type, ComponentType::HerbOfHealing);
    }

    #[test]
    fn test_component_type_name() {
        assert_eq!(ComponentType::HerbOfHealing.name(), "herb of healing");
        assert_eq!(ComponentType::SulfurousDust.name(), "sulfurous dust");
    }

    #[test]
    fn test_list_available() {
        let mut inv = ComponentInventory::new();
        inv.add_component(ComponentType::HerbOfHealing, 3);
        inv.add_component(ComponentType::FlamingCoal, 1);

        let available = inv.list_available();
        assert_eq!(available.len(), 2);
    }
}
