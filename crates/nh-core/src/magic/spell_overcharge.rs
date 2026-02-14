//! Spell overcharge system - Spend extra mana for increased power
//!
//! Allows players to spend additional mana beyond a spell's normal cost to increase
//! its effectiveness, at the risk of backlash or critical effects.

use serde::{Deserialize, Serialize};

/// Levels of spell overcharge intensity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OverchargeLevel {
    /// No overcharge
    Normal = 0,
    /// 0-49% extra mana spent
    Minor = 1,
    /// 50-99% extra mana spent
    Moderate = 2,
    /// 100-199% extra mana spent
    Significant = 3,
    /// 200%+ extra mana spent (risky)
    Extreme = 4,
}

impl OverchargeLevel {
    /// Get damage multiplier for this overcharge level
    pub const fn damage_multiplier(&self) -> f32 {
        match self {
            OverchargeLevel::Normal => 1.0,
            OverchargeLevel::Minor => 1.15,
            OverchargeLevel::Moderate => 1.35,
            OverchargeLevel::Significant => 1.75,
            OverchargeLevel::Extreme => 2.5,
        }
    }

    /// Get critical chance bonus for this overcharge level
    pub const fn critical_bonus(&self) -> i32 {
        match self {
            OverchargeLevel::Normal => 0,
            OverchargeLevel::Minor => 5,
            OverchargeLevel::Moderate => 15,
            OverchargeLevel::Significant => 30,
            OverchargeLevel::Extreme => 50,
        }
    }

    /// Get backlash damage if the spell fails or overload happens
    pub const fn backlash_damage(&self) -> i32 {
        match self {
            OverchargeLevel::Normal => 0,
            OverchargeLevel::Minor => 0,
            OverchargeLevel::Moderate => 5,
            OverchargeLevel::Significant => 15,
            OverchargeLevel::Extreme => 40,
        }
    }

    /// Get failure rate increase (%) for this overcharge level
    pub const fn failure_increase(&self) -> i32 {
        match self {
            OverchargeLevel::Normal => 0,
            OverchargeLevel::Minor => 2,
            OverchargeLevel::Moderate => 8,
            OverchargeLevel::Significant => 20,
            OverchargeLevel::Extreme => 40,
        }
    }

    /// Chance (out of 100) that overcharge causes a magical surge
    pub const fn surge_chance(&self) -> i32 {
        match self {
            OverchargeLevel::Normal => 0,
            OverchargeLevel::Minor => 0,
            OverchargeLevel::Moderate => 5,
            OverchargeLevel::Significant => 15,
            OverchargeLevel::Extreme => 40,
        }
    }
}

/// Result of applying spell overcharge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellOverchargeResult {
    /// Level of overcharge applied
    pub level: OverchargeLevel,
    /// Extra mana spent
    pub extra_mana: i32,
    /// Damage multiplier from overcharge
    pub damage_multiplier: f32,
    /// Additional critical chance
    pub critical_bonus: i32,
    /// Potential backlash damage if spell fails
    pub backlash_damage: i32,
    /// Whether a magical surge occurred
    pub surge: bool,
    /// Message describing the effect
    pub message: String,
}

/// Calculate overcharge level based on extra mana spent
pub fn calculate_overcharge_level(base_mana: i32, extra_mana: i32) -> OverchargeLevel {
    if extra_mana <= 0 {
        return OverchargeLevel::Normal;
    }

    let percentage = (extra_mana as f32 / base_mana as f32 * 100.0) as i32;

    match percentage {
        0..=49 => OverchargeLevel::Minor,
        50..=99 => OverchargeLevel::Moderate,
        100..=199 => OverchargeLevel::Significant,
        _ => OverchargeLevel::Extreme,
    }
}

/// Apply overcharge to a spell result
pub fn apply_overcharge(
    base_damage: i32,
    base_mana: i32,
    extra_mana: i32,
    rng: &mut crate::rng::GameRng,
) -> SpellOverchargeResult {
    let level = calculate_overcharge_level(base_mana, extra_mana);

    let damage_multiplier = level.damage_multiplier();
    let critical_bonus = level.critical_bonus();
    let backlash_damage = level.backlash_damage();

    // Check for magical surge
    let surge_chance = level.surge_chance() as u32;
    let surge = rng.percent(surge_chance);

    let message = match level {
        OverchargeLevel::Normal => "The spell manifests normally.".to_string(),
        OverchargeLevel::Minor => "You feel a slight tingle of extra power.".to_string(),
        OverchargeLevel::Moderate => "The spell surges with heightened energy!".to_string(),
        OverchargeLevel::Significant => {
            "The spell crackles with intense magical power!".to_string()
        }
        OverchargeLevel::Extreme => {
            if surge {
                "The spell wildly overloads with magical energy!".to_string()
            } else {
                "The spell reaches critical overcharge levels!".to_string()
            }
        }
    };

    SpellOverchargeResult {
        level,
        extra_mana,
        damage_multiplier,
        critical_bonus,
        backlash_damage,
        surge,
        message,
    }
}

/// Check if overcharge causes backlash damage
pub fn check_overcharge_backlash(
    level: OverchargeLevel,
    spell_failed: bool,
    rng: &mut crate::rng::GameRng,
) -> Option<i32> {
    if spell_failed || level.backlash_damage() == 0 {
        // Only backlash if spell failed
        if !spell_failed {
            return None;
        }
    }

    let backlash = level.backlash_damage();
    if backlash > 0 && (spell_failed || rng.percent(level.surge_chance() as u32)) {
        Some(rng.rnd(backlash as u32 / 2) as i32 + backlash / 2)
    } else {
        None
    }
}

/// Check if overcharge surge happens (increases effect)
pub fn check_overcharge_surge(level: OverchargeLevel, rng: &mut crate::rng::GameRng) -> bool {
    level.surge_chance() > 0 && rng.percent(level.surge_chance() as u32)
}

/// Calculate maximum safe overcharge for player's current mana
pub fn calculate_max_safe_overcharge(current_mana: i32, base_spell_cost: i32) -> i32 {
    // Can overcharge up to 50% of base cost safely (Minor level)
    let safe_overcharge = (base_spell_cost as f32 * 0.5) as i32;
    safe_overcharge.min(current_mana - base_spell_cost)
}

/// Calculate maximum possible overcharge (dangerous)
pub fn calculate_max_possible_overcharge(current_mana: i32, base_spell_cost: i32) -> i32 {
    // Can spend up to all remaining mana
    (current_mana - base_spell_cost).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overcharge_level_multipliers() {
        assert_eq!(OverchargeLevel::Normal.damage_multiplier(), 1.0);
        assert_eq!(OverchargeLevel::Minor.damage_multiplier(), 1.15);
        assert_eq!(OverchargeLevel::Extreme.damage_multiplier(), 2.5);
    }

    #[test]
    fn test_overcharge_level_backlash() {
        assert_eq!(OverchargeLevel::Normal.backlash_damage(), 0);
        assert_eq!(OverchargeLevel::Moderate.backlash_damage(), 5);
        assert!(OverchargeLevel::Extreme.backlash_damage() > 0);
    }

    #[test]
    fn test_calculate_overcharge_level() {
        assert_eq!(calculate_overcharge_level(100, 20), OverchargeLevel::Minor);
        assert_eq!(
            calculate_overcharge_level(100, 75),
            OverchargeLevel::Moderate
        );
        assert_eq!(
            calculate_overcharge_level(100, 150),
            OverchargeLevel::Significant
        );
        assert_eq!(
            calculate_overcharge_level(100, 250),
            OverchargeLevel::Extreme
        );
    }

    #[test]
    fn test_calculate_max_safe_overcharge() {
        let max_safe = calculate_max_safe_overcharge(200, 100);
        assert_eq!(max_safe, 50);
    }

    #[test]
    fn test_calculate_max_possible_overcharge() {
        let max_possible = calculate_max_possible_overcharge(250, 100);
        assert_eq!(max_possible, 150);
    }
}
