//! Spell critical hit system - Spell casts can land critical effects
//!
//! Similar to weapon critical hits, spells can sometimes have enhanced effects,
//! ignore resistances, affect more targets, or produce other beneficial outcomes.

use serde::{Deserialize, Serialize};

/// Types of critical spell effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CriticalSpellEffect {
    /// Spell damage/healing is doubled
    DoubledDamage,
    /// Spell duration is doubled
    DoubledDuration,
    /// Spell completely ignores target's resistances
    IgnoreResistance,
    /// Spell affects one additional nearby target (if applicable)
    ExtraTarget,
    /// Spell bypasses saving throws
    BypassSave,
    /// Spell casts with maximum effect (no randomness)
    Maximized,
    /// Spell affects area that's twice the normal radius
    ExpandedArea,
    /// Spell doesn't consume mana
    FreeCast,
}

impl CriticalSpellEffect {
    /// Get description of this critical effect
    pub const fn description(&self) -> &'static str {
        match self {
            CriticalSpellEffect::DoubledDamage => "Damage doubled!",
            CriticalSpellEffect::DoubledDuration => "Duration doubled!",
            CriticalSpellEffect::IgnoreResistance => "Resistance ignored!",
            CriticalSpellEffect::ExtraTarget => "Extra target affected!",
            CriticalSpellEffect::BypassSave => "Bypasses saving throw!",
            CriticalSpellEffect::Maximized => "Effect maximized!",
            CriticalSpellEffect::ExpandedArea => "Area expanded!",
            CriticalSpellEffect::FreeCast => "Spell is free!",
        }
    }
}

/// Result of critical spell check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalSpellResult {
    /// Whether the spell was a critical
    pub is_critical: bool,
    /// The critical effect (if any)
    pub effect: Option<CriticalSpellEffect>,
    /// Message describing the critical
    pub message: String,
}

/// Check for critical spell hit
pub fn check_critical_spell(
    player_level: i32,
    school_mastery: crate::magic::spell::SpellMastery,
    luck: i8,
    rng: &mut crate::rng::GameRng,
) -> CriticalSpellResult {
    let base_chance = calculate_critical_chance(player_level, school_mastery, luck);

    if !rng.percent(base_chance as u32) {
        return CriticalSpellResult {
            is_critical: false,
            effect: None,
            message: String::new(),
        };
    }

    // Determine which critical effect occurred
    let effect = pick_critical_effect(rng);
    let message = format!("Critical spell hit! {}", effect.description());

    CriticalSpellResult {
        is_critical: true,
        effect: Some(effect),
        message,
    }
}

/// Calculate base critical chance (0-100)
pub fn calculate_critical_chance(
    player_level: i32,
    school_mastery: crate::magic::spell::SpellMastery,
    luck: i8,
) -> u8 {
    // Base critical chance scales with player level
    let level_chance = (player_level as u8).min(20);

    // Mastery bonuses
    let mastery_bonus = match school_mastery {
        crate::magic::spell::SpellMastery::Unknown => 0,
        crate::magic::spell::SpellMastery::Novice => 5,
        crate::magic::spell::SpellMastery::Adept => 10,
        crate::magic::spell::SpellMastery::Expert => 15,
        crate::magic::spell::SpellMastery::Master => 20,
    };

    // Luck bonus
    let luck_bonus = (luck as i32).max(-10) as u8;

    // Combine and cap at 50% to prevent spam
    (level_chance + mastery_bonus + luck_bonus).min(50)
}

/// Pick which critical effect occurs
fn pick_critical_effect(rng: &mut crate::rng::GameRng) -> CriticalSpellEffect {
    let roll = rng.rn2(100) as i32;

    match roll {
        0..=20 => CriticalSpellEffect::DoubledDamage,
        21..=35 => CriticalSpellEffect::DoubledDuration,
        36..=50 => CriticalSpellEffect::IgnoreResistance,
        51..=60 => CriticalSpellEffect::ExtraTarget,
        61..=70 => CriticalSpellEffect::BypassSave,
        71..=80 => CriticalSpellEffect::Maximized,
        81..=90 => CriticalSpellEffect::ExpandedArea,
        _ => CriticalSpellEffect::FreeCast,
    }
}

/// Apply critical effect to spell damage
pub fn apply_critical_damage(damage: i32, effect: Option<CriticalSpellEffect>) -> i32 {
    match effect {
        Some(CriticalSpellEffect::DoubledDamage) => damage * 2,
        Some(CriticalSpellEffect::Maximized) => (damage as f32 * 1.5) as i32,
        _ => damage,
    }
}

/// Apply critical effect to spell duration
pub fn apply_critical_duration(duration: i32, effect: Option<CriticalSpellEffect>) -> i32 {
    match effect {
        Some(CriticalSpellEffect::DoubledDuration) => duration * 2,
        Some(CriticalSpellEffect::Maximized) => (duration as f32 * 1.5) as i32,
        _ => duration,
    }
}

/// Apply critical effect to spell area
pub fn apply_critical_area(area: i32, effect: Option<CriticalSpellEffect>) -> i32 {
    match effect {
        Some(CriticalSpellEffect::ExpandedArea) => (area as f32 * 2.0) as i32,
        Some(CriticalSpellEffect::Maximized) => (area as f32 * 1.5) as i32,
        _ => area,
    }
}

/// Check if critical bypasses saving throw
pub fn critical_bypasses_save(effect: Option<CriticalSpellEffect>) -> bool {
    matches!(
        effect,
        Some(CriticalSpellEffect::BypassSave) | Some(CriticalSpellEffect::Maximized)
    )
}

/// Check if critical ignores resistance
pub fn critical_ignores_resistance(effect: Option<CriticalSpellEffect>) -> bool {
    matches!(
        effect,
        Some(CriticalSpellEffect::IgnoreResistance) | Some(CriticalSpellEffect::Maximized)
    )
}

/// Get number of extra targets from critical
pub fn get_extra_targets(effect: Option<CriticalSpellEffect>) -> i32 {
    match effect {
        Some(CriticalSpellEffect::ExtraTarget) => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_critical_spell_effect_descriptions() {
        assert_eq!(
            CriticalSpellEffect::DoubledDamage.description(),
            "Damage doubled!"
        );
        assert_eq!(
            CriticalSpellEffect::IgnoreResistance.description(),
            "Resistance ignored!"
        );
    }

    #[test]
    fn test_calculate_critical_chance_base() {
        let chance = calculate_critical_chance(1, crate::magic::spell::SpellMastery::Unknown, 0);
        assert!(chance > 0);
        assert!(chance <= 50);
    }

    #[test]
    fn test_calculate_critical_chance_master() {
        let chance_novice =
            calculate_critical_chance(1, crate::magic::spell::SpellMastery::Novice, 0);
        let chance_master =
            calculate_critical_chance(1, crate::magic::spell::SpellMastery::Master, 0);
        assert!(chance_master > chance_novice);
    }

    #[test]
    fn test_calculate_critical_chance_with_luck() {
        let chance_no_luck =
            calculate_critical_chance(1, crate::magic::spell::SpellMastery::Unknown, 0);
        let chance_lucky =
            calculate_critical_chance(1, crate::magic::spell::SpellMastery::Unknown, 5);
        assert!(chance_lucky > chance_no_luck);
    }

    #[test]
    fn test_critical_chance_capped() {
        let chance = calculate_critical_chance(50, crate::magic::spell::SpellMastery::Master, 10);
        assert!(chance <= 50);
    }

    #[test]
    fn test_apply_critical_damage() {
        let damage = apply_critical_damage(20, Some(CriticalSpellEffect::DoubledDamage));
        assert_eq!(damage, 40);

        let damage = apply_critical_damage(20, None);
        assert_eq!(damage, 20);
    }

    #[test]
    fn test_apply_critical_duration() {
        let duration = apply_critical_duration(10, Some(CriticalSpellEffect::DoubledDuration));
        assert_eq!(duration, 20);
    }

    #[test]
    fn test_apply_critical_area() {
        let area = apply_critical_area(5, Some(CriticalSpellEffect::ExpandedArea));
        assert_eq!(area, 10);
    }

    #[test]
    fn test_critical_bypasses_save() {
        assert!(critical_bypasses_save(Some(
            CriticalSpellEffect::BypassSave
        )));
        assert!(!critical_bypasses_save(Some(
            CriticalSpellEffect::DoubledDamage
        )));
    }

    #[test]
    fn test_critical_ignores_resistance() {
        assert!(critical_ignores_resistance(Some(
            CriticalSpellEffect::IgnoreResistance
        )));
        assert!(!critical_ignores_resistance(Some(
            CriticalSpellEffect::DoubledDamage
        )));
    }

    #[test]
    fn test_get_extra_targets() {
        assert_eq!(get_extra_targets(Some(CriticalSpellEffect::ExtraTarget)), 1);
        assert_eq!(get_extra_targets(None), 0);
    }
}
