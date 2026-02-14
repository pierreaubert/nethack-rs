//! Spell customization - Modify spell parameters and set triggers
//!
//! Players can customize spells by adjusting range, damage, duration, and targets,
//! and can set automatic triggers for conditional casting.

use crate::magic::spell::SpellType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Custom modifications to a spell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSpellModification {
    pub range_multiplier: f32,
    pub damage_multiplier: f32,
    pub duration_multiplier: f32,
    pub area_multiplier: f32,
    pub split_targets: u8,
}

impl Default for CustomSpellModification {
    fn default() -> Self {
        Self {
            range_multiplier: 1.0,
            damage_multiplier: 1.0,
            duration_multiplier: 1.0,
            area_multiplier: 1.0,
            split_targets: 1,
        }
    }
}

/// Cost changes from customization
#[derive(Debug, Clone, Copy)]
pub struct CustomizationCost {
    pub mana_change: i32,
    pub failure_change: i32,
}

pub fn calculate_customization_cost(mods: &CustomSpellModification) -> CustomizationCost {
    let mut mana_change = 0;
    let mut failure_change = 0;

    if mods.range_multiplier > 1.0 {
        mana_change += ((mods.range_multiplier - 1.0) * 50.0) as i32;
        failure_change += 5;
    }
    if mods.damage_multiplier > 1.0 {
        mana_change += ((mods.damage_multiplier - 1.0) * 30.0) as i32;
        failure_change += 5;
    }
    if mods.duration_multiplier > 1.0 {
        mana_change += ((mods.duration_multiplier - 1.0) * 25.0) as i32;
    }
    if mods.area_multiplier > 1.0 {
        mana_change += ((mods.area_multiplier - 1.0) * 40.0) as i32;
        failure_change += 5;
    }
    if mods.split_targets > 1 {
        mana_change += (mods.split_targets as i32 - 1) * 30;
        failure_change += 10;
    }

    CustomizationCost {
        mana_change,
        failure_change,
    }
}

/// Trigger conditions for automatic casting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerCondition {
    HpBelow,
    EnemyInRange,
    ManaAbove,
    OnHit,
}

impl TriggerCondition {
    pub const fn name(&self) -> &'static str {
        match self {
            TriggerCondition::HpBelow => "HP Below",
            TriggerCondition::EnemyInRange => "Enemy In Range",
            TriggerCondition::ManaAbove => "Mana Above",
            TriggerCondition::OnHit => "On Hit",
        }
    }
}

/// Conditional spell trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalTrigger {
    pub spell: SpellType,
    pub condition: TriggerCondition,
    pub threshold: i32,
    pub active: bool,
}

impl ConditionalTrigger {
    pub fn new(spell: SpellType, condition: TriggerCondition, threshold: i32) -> Self {
        Self {
            spell,
            condition,
            threshold,
            active: true,
        }
    }

    pub fn check(&self, hp: i32, hp_max: i32, mana: i32, enemies_nearby: bool) -> bool {
        if !self.active {
            return false;
        }

        match self.condition {
            TriggerCondition::HpBelow => hp < self.threshold,
            TriggerCondition::EnemyInRange => enemies_nearby && self.threshold == 1,
            TriggerCondition::ManaAbove => mana > self.threshold,
            TriggerCondition::OnHit => false, // Checked separately
        }
    }
}

/// Customization tracker for player
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomizationTracker {
    pub modifications: HashMap<SpellType, CustomSpellModification>,
    pub triggers: Vec<ConditionalTrigger>,
}

impl CustomizationTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn customize_spell(&mut self, spell: SpellType, mods: CustomSpellModification) {
        self.modifications.insert(spell, mods);
    }

    pub fn get_modifications(&self, spell: SpellType) -> Option<&CustomSpellModification> {
        self.modifications.get(&spell)
    }

    pub fn add_trigger(&mut self, trigger: ConditionalTrigger) {
        if self.triggers.len() < 10 {
            // Max 10 triggers
            self.triggers.push(trigger);
        }
    }

    pub fn remove_trigger(&mut self, index: usize) {
        if index < self.triggers.len() {
            self.triggers.remove(index);
        }
    }

    pub fn check_all_triggers(
        &self,
        hp: i32,
        hp_max: i32,
        mana: i32,
        enemies: bool,
    ) -> Vec<SpellType> {
        self.triggers
            .iter()
            .filter(|t| t.check(hp, hp_max, mana, enemies))
            .map(|t| t.spell)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customization_cost() {
        let mods = CustomSpellModification {
            range_multiplier: 2.0,
            ..Default::default()
        };
        let cost = calculate_customization_cost(&mods);
        assert!(cost.mana_change > 0);
    }

    #[test]
    fn test_conditional_trigger() {
        let trigger = ConditionalTrigger::new(SpellType::Healing, TriggerCondition::HpBelow, 50);
        assert!(trigger.check(40, 100, 100, false));
        assert!(!trigger.check(60, 100, 100, false));
    }

    #[test]
    fn test_customization_tracker() {
        let mut tracker = CustomizationTracker::new();
        let mods = CustomSpellModification::default();
        tracker.customize_spell(SpellType::ForceBolt, mods);
        assert!(tracker.get_modifications(SpellType::ForceBolt).is_some());
    }
}
