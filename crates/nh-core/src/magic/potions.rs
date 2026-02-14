//! Advanced potion effect system
//!
//! Manages potion potency levels, effect stacking, interaction detection,
//! and duration/magnitude scaling for quaffed potions.

use crate::player::{Attribute, Property, You};
use crate::rng::GameRng;
use serde::{Deserialize, Serialize};

/// Potion potency levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PotionPotency {
    Diluted,  // 50% effect
    Normal,   // 100% effect
    Potent,   // 150% effect
    Powerful, // 200% effect
}

impl PotionPotency {
    /// Get potency multiplier (0.5 to 2.0)
    pub fn multiplier(&self) -> f32 {
        match self {
            PotionPotency::Diluted => 0.5,
            PotionPotency::Normal => 1.0,
            PotionPotency::Potent => 1.5,
            PotionPotency::Powerful => 2.0,
        }
    }

    /// Get duration modifier from potency
    pub fn duration_bonus(&self) -> i32 {
        match self {
            PotionPotency::Diluted => -50,
            PotionPotency::Normal => 0,
            PotionPotency::Potent => 100,
            PotionPotency::Powerful => 250,
        }
    }
}

/// Potion effect type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PotionEffectType {
    Healing,
    ExtraHealing,
    Regeneration,
    Gain,
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
    Speed,
    Invisibility,
    Levitation,
    Polymorph,
    Confusion,
    Poison,
}

/// Active potion effect on player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePotionEffect {
    pub effect_type: PotionEffectType,
    pub duration: i32,  // turns remaining
    pub magnitude: i32, // effect strength
    pub potency: PotionPotency,
}

impl ActivePotionEffect {
    pub fn new(
        effect_type: PotionEffectType,
        duration: i32,
        magnitude: i32,
        potency: PotionPotency,
    ) -> Self {
        Self {
            effect_type,
            duration,
            magnitude: (magnitude as f32 * potency.multiplier()) as i32,
            potency,
        }
    }
}

/// Track active potion effects
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PotionEffectTracker {
    pub active_effects: Vec<ActivePotionEffect>,
}

impl PotionEffectTracker {
    pub fn new() -> Self {
        Self {
            active_effects: Vec::new(),
        }
    }

    /// Add a new potion effect
    pub fn add_effect(&mut self, effect: ActivePotionEffect) {
        self.active_effects.push(effect);
    }

    /// Check if a specific effect is active
    pub fn has_effect(&self, effect_type: PotionEffectType) -> bool {
        self.active_effects
            .iter()
            .any(|e| e.effect_type == effect_type)
    }

    /// Get magnitude of active effect (if any)
    pub fn get_effect_magnitude(&self, effect_type: PotionEffectType) -> Option<i32> {
        self.active_effects
            .iter()
            .find(|e| e.effect_type == effect_type)
            .map(|e| e.magnitude)
    }

    /// Tick all active effects (reduce duration)
    pub fn tick_effects(&mut self) {
        self.active_effects.iter_mut().for_each(|e| e.duration -= 1);
        self.active_effects.retain(|e| e.duration > 0);
    }

    /// Count effects of a type
    pub fn count_effect(&self, effect_type: PotionEffectType) -> usize {
        self.active_effects
            .iter()
            .filter(|e| e.effect_type == effect_type)
            .count()
    }
}

/// Determine potion potency from item state
pub fn determine_potion_potency(blessed: bool, cursed: bool, age: i64) -> PotionPotency {
    if cursed {
        PotionPotency::Diluted
    } else if blessed {
        // Blessed potions are more potent
        if age < 1000 {
            PotionPotency::Powerful
        } else {
            PotionPotency::Potent
        }
    } else {
        // Normal uncursed potions
        if age > 5000 {
            PotionPotency::Diluted // Age reduces potency
        } else {
            PotionPotency::Normal
        }
    }
}

/// Check for potion effect interactions
pub fn check_potion_interaction(
    new_effect: PotionEffectType,
    existing_effects: &[ActivePotionEffect],
) -> Option<String> {
    for effect in existing_effects {
        match (new_effect, effect.effect_type) {
            // Attribute conflicts
            (PotionEffectType::Strength, PotionEffectType::Poison) => {
                return Some("The potions conflict! You feel sick.".to_string());
            }
            (PotionEffectType::Speed, PotionEffectType::Poison) => {
                return Some("The speed potion turns sour!".to_string());
            }
            // Vision/movement conflicts
            (PotionEffectType::Invisibility, PotionEffectType::Levitation) => {
                return Some("The potions mix into a shimmer.".to_string());
            }
            (PotionEffectType::Confusion, PotionEffectType::Invisibility) => {
                return Some("You feel confused about where you are.".to_string());
            }
            _ => {}
        }
    }
    None
}

/// Apply beneficial potion effects
pub fn apply_potion_effect(player: &mut You, effect: &ActivePotionEffect) {
    match effect.effect_type {
        PotionEffectType::Healing => {
            player.hp = (player.hp + effect.magnitude).min(player.hp_max);
        }
        PotionEffectType::ExtraHealing => {
            player.hp = player.hp_max;
        }
        PotionEffectType::Regeneration => {
            player.properties.grant_intrinsic(Property::Regeneration);
        }
        PotionEffectType::Strength => {
            let current = player.attr_current.get(Attribute::Strength);
            player
                .attr_current
                .set(Attribute::Strength, (current + 1).min(25));
        }
        PotionEffectType::Dexterity => {
            let current = player.attr_current.get(Attribute::Dexterity);
            player
                .attr_current
                .set(Attribute::Dexterity, (current + 1).min(25));
        }
        PotionEffectType::Constitution => {
            let current = player.attr_current.get(Attribute::Constitution);
            player
                .attr_current
                .set(Attribute::Constitution, (current + 1).min(25));
        }
        PotionEffectType::Intelligence => {
            let current = player.attr_current.get(Attribute::Intelligence);
            player
                .attr_current
                .set(Attribute::Intelligence, (current + 1).min(25));
        }
        PotionEffectType::Wisdom => {
            let current = player.attr_current.get(Attribute::Wisdom);
            player
                .attr_current
                .set(Attribute::Wisdom, (current + 1).min(25));
        }
        PotionEffectType::Charisma => {
            let current = player.attr_current.get(Attribute::Charisma);
            player
                .attr_current
                .set(Attribute::Charisma, (current + 1).min(25));
        }
        PotionEffectType::Speed => {
            player.movement_points = (player.movement_points + 5).min(50);
        }
        PotionEffectType::Invisibility => {
            player.properties.grant_intrinsic(Property::Invisibility);
        }
        PotionEffectType::Levitation => {
            player.properties.grant_intrinsic(Property::Levitation);
        }
        _ => {} // Harmful effects handled elsewhere
    }
}

/// Get potion effect message
pub fn get_effect_message(effect_type: PotionEffectType, potency: PotionPotency) -> String {
    let potency_word = match potency {
        PotionPotency::Diluted => "faintly",
        PotionPotency::Normal => "",
        PotionPotency::Potent => "strongly",
        PotionPotency::Powerful => "powerfully",
    };

    match effect_type {
        PotionEffectType::Healing => format!("You feel {} better.", potency_word),
        PotionEffectType::ExtraHealing => "You feel completely healed!".to_string(),
        PotionEffectType::Regeneration => format!("Your wounds {} regenerate.", potency_word),
        PotionEffectType::Strength => format!("You feel {} stronger.", potency_word),
        PotionEffectType::Speed => format!("You feel {} faster.", potency_word),
        PotionEffectType::Invisibility => format!("You become {} invisible.", potency_word),
        PotionEffectType::Levitation => format!("You feel {} light.", potency_word),
        PotionEffectType::Confusion => format!("Everything spins {}!", potency_word),
        PotionEffectType::Poison => format!("You feel {} poisoned.", potency_word),
        _ => "You quaff the potion.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_potion_potency_multiplier() {
        assert_eq!(PotionPotency::Diluted.multiplier(), 0.5);
        assert_eq!(PotionPotency::Normal.multiplier(), 1.0);
        assert_eq!(PotionPotency::Potent.multiplier(), 1.5);
        assert_eq!(PotionPotency::Powerful.multiplier(), 2.0);
    }

    #[test]
    fn test_active_potion_effect_magnitude() {
        let effect =
            ActivePotionEffect::new(PotionEffectType::Healing, 100, 10, PotionPotency::Powerful);
        assert_eq!(effect.magnitude, 20); // 10 * 2.0
    }

    #[test]
    fn test_potion_effect_tracker_add() {
        let mut tracker = PotionEffectTracker::new();
        let effect = ActivePotionEffect::new(PotionEffectType::Speed, 50, 5, PotionPotency::Normal);

        tracker.add_effect(effect);
        assert!(tracker.has_effect(PotionEffectType::Speed));
    }

    #[test]
    fn test_potion_effect_tracker_tick() {
        let mut tracker = PotionEffectTracker::new();
        let effect = ActivePotionEffect::new(PotionEffectType::Speed, 3, 5, PotionPotency::Normal);

        tracker.add_effect(effect);
        tracker.tick_effects();
        assert!(tracker.has_effect(PotionEffectType::Speed));

        tracker.tick_effects();
        tracker.tick_effects();
        assert!(!tracker.has_effect(PotionEffectType::Speed));
    }

    #[test]
    fn test_determine_potion_potency_blessed() {
        let potency = determine_potion_potency(true, false, 0);
        assert_eq!(potency, PotionPotency::Powerful);
    }

    #[test]
    fn test_determine_potion_potency_cursed() {
        let potency = determine_potion_potency(false, true, 0);
        assert_eq!(potency, PotionPotency::Diluted);
    }

    #[test]
    fn test_determine_potion_potency_aged() {
        let potency = determine_potion_potency(false, false, 6000);
        assert_eq!(potency, PotionPotency::Diluted);
    }

    #[test]
    fn test_check_potion_interaction_conflict() {
        let existing = vec![ActivePotionEffect::new(
            PotionEffectType::Poison,
            50,
            5,
            PotionPotency::Normal,
        )];

        let interaction = check_potion_interaction(PotionEffectType::Strength, &existing);
        assert!(interaction.is_some());
    }

    #[test]
    fn test_check_potion_interaction_no_conflict() {
        let existing = vec![ActivePotionEffect::new(
            PotionEffectType::Speed,
            50,
            5,
            PotionPotency::Normal,
        )];

        let interaction = check_potion_interaction(PotionEffectType::Healing, &existing);
        assert!(interaction.is_none());
    }

    #[test]
    fn test_apply_potion_healing() {
        let mut player = You::default();
        player.hp = 50;
        player.hp_max = 100;

        let effect =
            ActivePotionEffect::new(PotionEffectType::Healing, 100, 30, PotionPotency::Normal);
        apply_potion_effect(&mut player, &effect);

        assert_eq!(player.hp, 80);
    }

    #[test]
    fn test_get_effect_message() {
        let msg = get_effect_message(PotionEffectType::Speed, PotionPotency::Powerful);
        assert!(msg.contains("powerfully"));
    }

    #[test]
    fn test_potion_effect_magnitude_scaling() {
        let diluted =
            ActivePotionEffect::new(PotionEffectType::Healing, 100, 10, PotionPotency::Diluted);
        let powerful =
            ActivePotionEffect::new(PotionEffectType::Healing, 100, 10, PotionPotency::Powerful);

        assert_eq!(diluted.magnitude, 5);
        assert_eq!(powerful.magnitude, 20);
    }

    #[test]
    fn test_count_effect() {
        let mut tracker = PotionEffectTracker::new();
        tracker.add_effect(ActivePotionEffect::new(
            PotionEffectType::Speed,
            50,
            5,
            PotionPotency::Normal,
        ));
        tracker.add_effect(ActivePotionEffect::new(
            PotionEffectType::Speed,
            30,
            3,
            PotionPotency::Normal,
        ));

        assert_eq!(tracker.count_effect(PotionEffectType::Speed), 2);
    }
}
