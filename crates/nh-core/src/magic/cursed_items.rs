//! Cursed item consequence system
//!
//! Tracks negative effects from cursed equipment, including fumbling,
//! stat penalties, forced equipping/unequipping, and other curses.

use crate::object::Object;
use crate::player::{Attribute, You};
use crate::rng::GameRng;
use serde::{Deserialize, Serialize};

/// Type of cursed item effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CursedEffect {
    // Weapon curses
    Fumbling,      // Chance to drop weapon
    WeaponPenalty, // Attack penalty

    // Armor curses
    ArmorPenalty, // AC penalty (worse)
    Encumbrance,  // Movement speed reduced

    // Equipment curses
    StatsLower,  // One stat reduced
    ForcedEquip, // Can't unequip item

    // General curses
    Weakness,   // Damage output reduced
    Slowness,   // Actions slower
    Clumsiness, // Dexterity reduced, fumble chance
}

impl CursedEffect {
    /// Get description of cursed effect
    pub fn description(&self) -> &'static str {
        match self {
            CursedEffect::Fumbling => "You feel clumsy",
            CursedEffect::WeaponPenalty => "The weapon feels wrong in your hand",
            CursedEffect::ArmorPenalty => "The armor constricts around you",
            CursedEffect::Encumbrance => "You feel unusually heavy",
            CursedEffect::StatsLower => "You feel weaker",
            CursedEffect::ForcedEquip => "You cannot remove this item",
            CursedEffect::Weakness => "Your grip weakens",
            CursedEffect::Slowness => "You feel sluggish",
            CursedEffect::Clumsiness => "You feel uncoordinated",
        }
    }

    /// Severity rating 1-5
    pub fn severity(&self) -> i32 {
        match self {
            CursedEffect::Fumbling => 3,
            CursedEffect::WeaponPenalty => 2,
            CursedEffect::ArmorPenalty => 3,
            CursedEffect::Encumbrance => 2,
            CursedEffect::StatsLower => 4,
            CursedEffect::ForcedEquip => 5,
            CursedEffect::Weakness => 2,
            CursedEffect::Slowness => 2,
            CursedEffect::Clumsiness => 3,
        }
    }
}

/// Active cursed item consequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursedConsequence {
    pub effect: CursedEffect,
    pub object_id: u32,
    pub affected_attribute: Option<Attribute>,
    pub magnitude: i32, // Penalty amount
}

impl CursedConsequence {
    pub fn new(effect: CursedEffect, object_id: u32) -> Self {
        Self {
            effect,
            object_id,
            affected_attribute: None,
            magnitude: 1,
        }
    }

    pub fn with_stat(mut self, attr: Attribute, reduction: i32) -> Self {
        self.affected_attribute = Some(attr);
        self.magnitude = reduction;
        self
    }

    pub fn with_magnitude(mut self, mag: i32) -> Self {
        self.magnitude = mag;
        self
    }
}

/// Tracker for all active cursed item effects
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CursedItemTracker {
    pub active_curses: Vec<CursedConsequence>,
}

impl CursedItemTracker {
    pub fn new() -> Self {
        Self {
            active_curses: Vec::new(),
        }
    }

    /// Add a cursed consequence
    pub fn add_curse(&mut self, consequence: CursedConsequence) {
        self.active_curses.push(consequence);
    }

    /// Remove all curses from a specific item
    pub fn remove_item_curses(&mut self, object_id: u32) {
        self.active_curses.retain(|c| c.object_id != object_id);
    }

    /// Check if item has a specific curse type
    pub fn has_curse_type(&self, object_id: u32, effect: CursedEffect) -> bool {
        self.active_curses
            .iter()
            .any(|c| c.object_id == object_id && c.effect == effect)
    }

    /// Get all active curse effects
    pub fn get_all_curses(&self, object_id: u32) -> Vec<&CursedConsequence> {
        self.active_curses
            .iter()
            .filter(|c| c.object_id == object_id)
            .collect()
    }

    /// Check if item is stuck (forced equip)
    pub fn is_item_stuck(&self, object_id: u32) -> bool {
        self.has_curse_type(object_id, CursedEffect::ForcedEquip)
    }

    /// Total fumbling chance from all equipped cursed weapons
    pub fn total_fumble_chance(&self) -> i32 {
        self.active_curses
            .iter()
            .filter(|c| c.effect == CursedEffect::Fumbling)
            .map(|c| c.magnitude)
            .sum()
    }

    /// Total AC penalty from cursed armor
    pub fn total_armor_penalty(&self) -> i32 {
        self.active_curses
            .iter()
            .filter(|c| c.effect == CursedEffect::ArmorPenalty)
            .map(|c| c.magnitude)
            .sum()
    }

    /// Total stat reduction from cursed equipment
    pub fn total_stat_reduction(&self, attr: Attribute) -> i32 {
        self.active_curses
            .iter()
            .filter(|c| c.effect == CursedEffect::StatsLower && c.affected_attribute == Some(attr))
            .map(|c| c.magnitude)
            .sum()
    }

    /// Total encumbrance penalty
    pub fn total_encumbrance(&self) -> i32 {
        self.active_curses
            .iter()
            .filter(|c| c.effect == CursedEffect::Encumbrance)
            .count() as i32
    }
}

/// Determine what cursed effects an item should have
pub fn determine_cursed_effects(obj: &Object, rng: &mut GameRng) -> Vec<CursedEffect> {
    if !obj.buc.is_cursed() {
        return Vec::new();
    }

    use crate::object::ObjectClass;

    let mut effects = Vec::new();

    // Weapon curses
    if matches!(obj.class, ObjectClass::Weapon) {
        if rng.percent(40) {
            effects.push(CursedEffect::Fumbling);
        }
        if rng.percent(30) {
            effects.push(CursedEffect::WeaponPenalty);
        }
        if rng.percent(20) {
            effects.push(CursedEffect::Weakness);
        }
    }

    // Armor curses
    if matches!(obj.class, ObjectClass::Armor) {
        if rng.percent(50) {
            effects.push(CursedEffect::ArmorPenalty);
        }
        if rng.percent(30) {
            effects.push(CursedEffect::Encumbrance);
        }
    }

    // All equipment can be stuck
    if rng.percent(25) {
        effects.push(CursedEffect::ForcedEquip);
    }

    // Stat reduction chance (any cursed equipment)
    if rng.percent(35) {
        effects.push(CursedEffect::StatsLower);
    }

    effects
}

/// Determine severity of curse (how bad the effect is)
pub fn calculate_curse_magnitude(obj: &Object, effect: CursedEffect) -> i32 {
    match effect {
        CursedEffect::Fumbling => {
            // Base 25%, increases with more cursed items
            25 + (obj.enchantment.abs() as i32 * 5).min(15)
        }
        CursedEffect::WeaponPenalty => {
            // -1 to -5 attack penalty
            -(obj.enchantment.abs() as i32 + 1).min(5)
        }
        CursedEffect::ArmorPenalty => {
            // AC worsens by 1-3
            1 + (obj.enchantment.abs() as i32 / 2).min(2)
        }
        CursedEffect::Encumbrance => {
            // Slows by 1 per cursed item
            1
        }
        CursedEffect::StatsLower => {
            // Reduce stat by 1-2
            1 + (obj.enchantment.abs() as i32 / 5).min(1)
        }
        CursedEffect::Weakness => {
            // 20% damage reduction
            20
        }
        CursedEffect::Slowness => {
            // Actions take 20% longer
            20
        }
        CursedEffect::Clumsiness => {
            // Dexterity penalty + fumble chance
            2
        }
        CursedEffect::ForcedEquip => {
            // Binary - either stuck or not
            1
        }
    }
}

/// Apply cursed item effect to player stats
pub fn apply_cursed_effect(player: &mut You, consequence: &CursedConsequence) {
    match consequence.effect {
        CursedEffect::Fumbling => {
            // Fumbling tracked in consequence tracker, not applied here
        }
        CursedEffect::WeaponPenalty => {
            // Combat modifier, applied in combat system
        }
        CursedEffect::ArmorPenalty => {
            // AC penalty applied in armor class calculation
            player.armor_class += consequence.magnitude as i8;
        }
        CursedEffect::Encumbrance => {
            // Movement reduced
            player.movement_points =
                (player.movement_points as i32 - consequence.magnitude).max(1) as i16;
        }
        CursedEffect::StatsLower => {
            if let Some(attr) = consequence.affected_attribute {
                let current = player.attr_current.get(attr) as i32;
                let reduced = (current - consequence.magnitude).max(3) as i8;
                player.attr_current.set(attr, reduced);
            }
        }
        CursedEffect::ForcedEquip => {
            // Can't remove - enforced in equipment system
        }
        CursedEffect::Weakness => {
            // Damage modifier tracked, not applied to player directly
        }
        CursedEffect::Slowness => {
            // Action speed modifier, applied in turn system
        }
        CursedEffect::Clumsiness => {
            // Dexterity penalty
            let current = player.attr_current.get(Attribute::Dexterity) as i32;
            player.attr_current.set(
                Attribute::Dexterity,
                (current - consequence.magnitude).max(3) as i8,
            );
        }
    }
}

/// Check if action is fumbled with cursed weapon
pub fn check_fumble(fumble_chance: i32, rng: &mut GameRng) -> bool {
    rng.percent(fumble_chance as u32)
}

/// Get message for cursed effect activation
pub fn cursed_effect_message(consequence: &CursedConsequence) -> String {
    match consequence.effect {
        CursedEffect::Fumbling => {
            format!(
                "{}! You fumble your grip.",
                consequence.effect.description()
            )
        }
        CursedEffect::WeaponPenalty => {
            format!(
                "{}. Your attacks are less effective.",
                consequence.effect.description()
            )
        }
        CursedEffect::ArmorPenalty => {
            format!(
                "{}! Your defense is compromised.",
                consequence.effect.description()
            )
        }
        CursedEffect::Encumbrance => {
            format!(
                "{}. You move more slowly.",
                consequence.effect.description()
            )
        }
        CursedEffect::StatsLower => {
            if let Some(attr) = consequence.affected_attribute {
                format!(
                    "{}! Your {} is reduced.",
                    consequence.effect.description(),
                    attr.name().to_lowercase()
                )
            } else {
                consequence.effect.description().to_string()
            }
        }
        CursedEffect::ForcedEquip => {
            format!(
                "{}! You cannot remove it!",
                consequence.effect.description()
            )
        }
        CursedEffect::Weakness => {
            format!("{}. Your strength fails.", consequence.effect.description())
        }
        CursedEffect::Slowness => {
            format!(
                "{}. Your movements become sluggish.",
                consequence.effect.description()
            )
        }
        CursedEffect::Clumsiness => {
            format!(
                "{}. You feel uncoordinated.",
                consequence.effect.description()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursed_effect_severity() {
        assert_eq!(CursedEffect::ForcedEquip.severity(), 5);
        assert_eq!(CursedEffect::StatsLower.severity(), 4);
        assert_eq!(CursedEffect::Fumbling.severity(), 3);
        assert_eq!(CursedEffect::WeaponPenalty.severity(), 2);
    }

    #[test]
    fn test_cursed_consequence_creation() {
        let consequence = CursedConsequence::new(CursedEffect::Fumbling, 42);
        assert_eq!(consequence.object_id, 42);
        assert_eq!(consequence.effect, CursedEffect::Fumbling);
        assert_eq!(consequence.magnitude, 1);
    }

    #[test]
    fn test_cursed_consequence_with_stat() {
        let consequence =
            CursedConsequence::new(CursedEffect::StatsLower, 42).with_stat(Attribute::Strength, 2);
        assert_eq!(consequence.affected_attribute, Some(Attribute::Strength));
        assert_eq!(consequence.magnitude, 2);
    }

    #[test]
    fn test_cursed_item_tracker_add() {
        let mut tracker = CursedItemTracker::new();
        let consequence = CursedConsequence::new(CursedEffect::Fumbling, 1);
        tracker.add_curse(consequence);

        assert_eq!(tracker.active_curses.len(), 1);
    }

    #[test]
    fn test_cursed_item_tracker_remove() {
        let mut tracker = CursedItemTracker::new();
        tracker.add_curse(CursedConsequence::new(CursedEffect::Fumbling, 1));
        tracker.add_curse(CursedConsequence::new(CursedEffect::ArmorPenalty, 2));

        tracker.remove_item_curses(1);
        assert_eq!(tracker.active_curses.len(), 1);
        assert!(tracker.has_curse_type(2, CursedEffect::ArmorPenalty));
    }

    #[test]
    fn test_cursed_item_tracker_has_curse_type() {
        let mut tracker = CursedItemTracker::new();
        tracker.add_curse(CursedConsequence::new(CursedEffect::Fumbling, 1));

        assert!(tracker.has_curse_type(1, CursedEffect::Fumbling));
        assert!(!tracker.has_curse_type(1, CursedEffect::WeaponPenalty));
        assert!(!tracker.has_curse_type(2, CursedEffect::Fumbling));
    }

    #[test]
    fn test_cursed_item_tracker_is_stuck() {
        let mut tracker = CursedItemTracker::new();
        tracker.add_curse(CursedConsequence::new(CursedEffect::ForcedEquip, 1));

        assert!(tracker.is_item_stuck(1));
        assert!(!tracker.is_item_stuck(2));
    }

    #[test]
    fn test_total_fumble_chance() {
        let mut tracker = CursedItemTracker::new();
        tracker.add_curse(CursedConsequence::new(CursedEffect::Fumbling, 1).with_magnitude(20));
        tracker.add_curse(CursedConsequence::new(CursedEffect::Fumbling, 2).with_magnitude(15));
        tracker.add_curse(CursedConsequence::new(CursedEffect::ArmorPenalty, 3).with_magnitude(2));

        assert_eq!(tracker.total_fumble_chance(), 35);
    }

    #[test]
    fn test_total_armor_penalty() {
        let mut tracker = CursedItemTracker::new();
        tracker.add_curse(CursedConsequence::new(CursedEffect::ArmorPenalty, 1).with_magnitude(1));
        tracker.add_curse(CursedConsequence::new(CursedEffect::ArmorPenalty, 2).with_magnitude(2));

        assert_eq!(tracker.total_armor_penalty(), 3);
    }

    #[test]
    fn test_total_stat_reduction() {
        let mut tracker = CursedItemTracker::new();
        tracker.add_curse(
            CursedConsequence::new(CursedEffect::StatsLower, 1)
                .with_stat(Attribute::Strength, 1)
                .with_magnitude(1),
        );
        tracker.add_curse(
            CursedConsequence::new(CursedEffect::StatsLower, 2)
                .with_stat(Attribute::Strength, 2)
                .with_magnitude(2),
        );
        tracker.add_curse(
            CursedConsequence::new(CursedEffect::StatsLower, 3)
                .with_stat(Attribute::Dexterity, 1)
                .with_magnitude(1),
        );

        assert_eq!(tracker.total_stat_reduction(Attribute::Strength), 3);
        assert_eq!(tracker.total_stat_reduction(Attribute::Dexterity), 1);
    }

    #[test]
    fn test_total_encumbrance() {
        let mut tracker = CursedItemTracker::new();
        tracker.add_curse(CursedConsequence::new(CursedEffect::Encumbrance, 1));
        tracker.add_curse(CursedConsequence::new(CursedEffect::Encumbrance, 2));

        assert_eq!(tracker.total_encumbrance(), 2);
    }

    #[test]
    fn test_calculate_curse_magnitude_fumbling() {
        let mut obj = Object::default();
        obj.buc = crate::object::BucStatus::Cursed;
        obj.enchantment = 0;
        let mag = calculate_curse_magnitude(&obj, CursedEffect::Fumbling);
        assert!(mag >= 25);
    }

    #[test]
    fn test_calculate_curse_magnitude_armor_penalty() {
        let mut obj = Object::default();
        obj.buc = crate::object::BucStatus::Cursed;
        obj.enchantment = 3;
        let mag = calculate_curse_magnitude(&obj, CursedEffect::ArmorPenalty);
        assert!(mag > 0);
    }

    #[test]
    fn test_cursed_effect_message() {
        let consequence = CursedConsequence::new(CursedEffect::Fumbling, 1);
        let msg = cursed_effect_message(&consequence);
        assert!(msg.contains("fumble"));
    }

    #[test]
    fn test_cursed_effect_message_stat() {
        let consequence =
            CursedConsequence::new(CursedEffect::StatsLower, 1).with_stat(Attribute::Strength, 1);
        let msg = cursed_effect_message(&consequence);
        assert!(msg.contains("strength"));
    }

    #[test]
    fn test_check_fumble() {
        let mut rng = crate::rng::GameRng::new(42);
        let fumbled = check_fumble(100, &mut rng);
        assert!(fumbled); // 100% chance should always be true
    }

    #[test]
    fn test_check_fumble_never() {
        let mut rng = crate::rng::GameRng::new(42);
        let fumbled = check_fumble(0, &mut rng);
        assert!(!fumbled); // 0% chance should always be false
    }

    #[test]
    fn test_get_all_curses() {
        let mut tracker = CursedItemTracker::new();
        tracker.add_curse(CursedConsequence::new(CursedEffect::Fumbling, 1));
        tracker.add_curse(CursedConsequence::new(CursedEffect::ArmorPenalty, 1));
        tracker.add_curse(CursedConsequence::new(CursedEffect::Fumbling, 2));

        let item1_curses = tracker.get_all_curses(1);
        assert_eq!(item1_curses.len(), 2);
    }

    #[test]
    fn test_cursed_consequence_with_magnitude() {
        let consequence = CursedConsequence::new(CursedEffect::WeaponPenalty, 42).with_magnitude(5);
        assert_eq!(consequence.magnitude, 5);
    }
}
