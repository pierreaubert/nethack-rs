//! Casting stances - Different stances modify spell behavior
//!
//! Players can adopt stances that modify their spell casting characteristics.

use serde::{Deserialize, Serialize};

/// Casting stances
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CastingStance {
    #[default]
    Balanced,
    Aggressive,
    Defensive,
    Concentrated,
    Reckless,
    Efficient,
}

impl CastingStance {
    pub const fn name(&self) -> &'static str {
        match self {
            CastingStance::Balanced => "Balanced",
            CastingStance::Aggressive => "Aggressive",
            CastingStance::Defensive => "Defensive",
            CastingStance::Concentrated => "Concentrated",
            CastingStance::Reckless => "Reckless",
            CastingStance::Efficient => "Efficient",
        }
    }

    pub fn all() -> &'static [CastingStance] {
        &[
            CastingStance::Balanced,
            CastingStance::Aggressive,
            CastingStance::Defensive,
            CastingStance::Concentrated,
            CastingStance::Reckless,
            CastingStance::Efficient,
        ]
    }
}

/// Modifiers applied by a stance
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StanceModifiers {
    pub damage_multiplier: f32,
    pub mana_multiplier: f32,
    pub failure_rate_change: i32,
    pub ac_bonus: i8,
    pub movement_penalty: i8,
    pub backlash_risk: i32,
}

impl StanceModifiers {
    pub fn for_stance(stance: CastingStance) -> Self {
        match stance {
            CastingStance::Balanced => Self {
                damage_multiplier: 1.0,
                mana_multiplier: 1.0,
                failure_rate_change: 0,
                ac_bonus: 0,
                movement_penalty: 0,
                backlash_risk: 0,
            },
            CastingStance::Aggressive => Self {
                damage_multiplier: 1.5,
                mana_multiplier: 1.1,
                failure_rate_change: 10,
                ac_bonus: -2,
                movement_penalty: 1,
                backlash_risk: 15,
            },
            CastingStance::Defensive => Self {
                damage_multiplier: 0.7,
                mana_multiplier: 0.9,
                failure_rate_change: -10,
                ac_bonus: 3,
                movement_penalty: 0,
                backlash_risk: -10,
            },
            CastingStance::Concentrated => Self {
                damage_multiplier: 1.2,
                mana_multiplier: 0.8,
                failure_rate_change: -15,
                ac_bonus: -1,
                movement_penalty: 2,
                backlash_risk: 0,
            },
            CastingStance::Reckless => Self {
                damage_multiplier: 2.0,
                mana_multiplier: 1.5,
                failure_rate_change: 25,
                ac_bonus: -4,
                movement_penalty: 2,
                backlash_risk: 50,
            },
            CastingStance::Efficient => Self {
                damage_multiplier: 0.9,
                mana_multiplier: 0.6,
                failure_rate_change: 5,
                ac_bonus: 1,
                movement_penalty: 0,
                backlash_risk: -5,
            },
        }
    }
}

/// Tracker for current stance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StanceTracker {
    pub current_stance: CastingStance,
    pub turns_in_stance: u32,
}

impl StanceTracker {
    pub fn new() -> Self {
        Self {
            current_stance: CastingStance::Balanced,
            turns_in_stance: 0,
        }
    }

    pub fn set_stance(&mut self, stance: CastingStance) {
        self.current_stance = stance;
        self.turns_in_stance = 0;
    }

    pub fn tick(&mut self) {
        self.turns_in_stance += 1;
    }

    pub fn get_modifiers(&self) -> StanceModifiers {
        StanceModifiers::for_stance(self.current_stance)
    }
}

pub fn get_stance_modifiers(stance: CastingStance) -> StanceModifiers {
    StanceModifiers::for_stance(stance)
}

pub fn apply_stance_to_spell(
    base_damage: i32,
    base_mana: i32,
    base_failure: i32,
    modifiers: StanceModifiers,
) -> (i32, i32, i32) {
    let damage = (base_damage as f32 * modifiers.damage_multiplier) as i32;
    let mana = (base_mana as f32 * modifiers.mana_multiplier) as i32;
    let failure = (base_failure + modifiers.failure_rate_change)
        .max(0)
        .min(100);
    (damage, mana, failure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stance_names() {
        assert_eq!(CastingStance::Aggressive.name(), "Aggressive");
        assert_eq!(CastingStance::Defensive.name(), "Defensive");
    }

    #[test]
    fn test_aggressive_modifiers() {
        let mods = StanceModifiers::for_stance(CastingStance::Aggressive);
        assert!(mods.damage_multiplier > 1.0);
        assert!(mods.failure_rate_change > 0);
    }

    #[test]
    fn test_stance_tracker() {
        let mut tracker = StanceTracker::new();
        assert_eq!(tracker.current_stance, CastingStance::Balanced);
        tracker.set_stance(CastingStance::Aggressive);
        assert_eq!(tracker.current_stance, CastingStance::Aggressive);
    }
}
