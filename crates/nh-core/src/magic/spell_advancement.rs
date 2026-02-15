//! Spell advancement integration
//!
//! Handles spell synergies, school specialization, and mastery advancement
//! mechanics during spell casting and each game turn.
//!
//! When the `extensions` feature is disabled, all functions are no-ops
//! returning neutral values (1.0 multiplier, 0 reduction, etc.).

use crate::magic::spell::{SpellMastery, SpellSchool, SpellType};
use crate::player::You;

/// Tick spell synergy tracker (called once per turn to age recent spells)
pub fn tick_spell_synergies(player: &mut You) {
    #[cfg(feature = "extensions")]
    {
        player.spell_synergy_tracker.tick();
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = player;
    }
}

/// Record a spell cast and update all advancement trackers
pub fn record_spell_cast(
    player: &mut You,
    spell_type: SpellType,
    succeeded: bool,
) -> Vec<String> {
    #[cfg(feature = "extensions")]
    {
        use crate::magic::{
            calculate_specialization_mana_cost, calculate_synergy_mana_cost,
            get_mastery_damage_bonus, get_mastery_mana_efficiency,
            get_specialization_damage_bonus, get_specialization_failure_reduction,
        };

        let mut messages = Vec::new();
        let school = spell_type.school();

        player
            .spell_synergy_tracker
            .record_spell(spell_type, school);
        player.specialization_tracker.spell_cast_in_school(school);
        player
            .mastery_tracker
            .record_spell_cast(spell_type, succeeded);

        if let Some((_milestone, msg)) = player.mastery_tracker.check_milestone(spell_type) {
            messages.push(msg);
        }

        messages
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = (player, spell_type, succeeded);
        Vec::new()
    }
}

/// Record a critical spell hit (exceeded expected effect)
pub fn record_critical_spell_hit(player: &mut You, spell_type: SpellType) {
    #[cfg(feature = "extensions")]
    {
        player.mastery_tracker.record_critical_hit(spell_type);
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = (player, spell_type);
    }
}

/// Get spell damage multiplier from all bonuses (synergies, specialization, mastery)
pub fn get_total_spell_damage_bonus(player: &You, spell_type: SpellType) -> f32 {
    #[cfg(feature = "extensions")]
    {
        use crate::magic::{get_mastery_damage_bonus, get_specialization_damage_bonus};

        let school = spell_type.school();

        let synergies = player
            .spell_synergy_tracker
            .check_synergies(spell_type, school);

        let mut synergy_bonus = 1.0;
        for synergy in &synergies {
            synergy_bonus *= synergy.bonus_multiplier();
        }

        let specialization_bonus =
            get_specialization_damage_bonus(&player.specialization_tracker, school);

        let mastery = player.mastery_tracker.get_mastery(spell_type);
        let mastery_bonus = get_mastery_damage_bonus(mastery);

        synergy_bonus * specialization_bonus * mastery_bonus
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = (player, spell_type);
        1.0
    }
}

/// Calculate final mana cost with all reductions applied
pub fn calculate_final_spell_mana_cost(
    player: &You,
    base_mana_cost: i32,
    spell_type: SpellType,
) -> i32 {
    #[cfg(feature = "extensions")]
    {
        use crate::magic::{
            calculate_specialization_mana_cost, calculate_synergy_mana_cost,
            get_mastery_mana_efficiency,
        };

        let school = spell_type.school();
        let cost_after_spec = calculate_specialization_mana_cost(
            &player.specialization_tracker,
            base_mana_cost,
            school,
        );

        let synergies = player
            .spell_synergy_tracker
            .check_synergies(spell_type, school);
        let cost_after_synergies = calculate_synergy_mana_cost(cost_after_spec, &synergies);

        let mastery = player.mastery_tracker.get_mastery(spell_type);
        let mastery_efficiency = get_mastery_mana_efficiency(mastery);
        let final_cost = (cost_after_synergies as f32 / mastery_efficiency) as i32;

        final_cost.max(base_mana_cost / 2)
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = (player, spell_type);
        base_mana_cost
    }
}

/// Get spell failure chance reduction from specialization and mastery
pub fn get_spell_failure_reduction(player: &You, spell_type: SpellType) -> i32 {
    #[cfg(feature = "extensions")]
    {
        use crate::magic::get_specialization_failure_reduction;

        let school = spell_type.school();
        let spec_reduction =
            get_specialization_failure_reduction(&player.specialization_tracker, school);
        let mastery = player.mastery_tracker.get_mastery(spell_type);

        let mastery_reduction = match mastery {
            SpellMastery::Unknown => 0,
            SpellMastery::Novice => 10,
            SpellMastery::Adept => 20,
            SpellMastery::Expert => 30,
            SpellMastery::Master => 40,
        };

        (spec_reduction + mastery_reduction).min(90)
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = (player, spell_type);
        0
    }
}

/// Spell statistics for UI display
#[derive(Debug, Clone)]
pub struct SpellStats {
    pub highest_mastery: SpellMastery,
    #[cfg(feature = "extensions")]
    pub highest_specialization: crate::magic::school_specialization::SpecializationLevel,
    pub total_spells_cast: i32,
    pub overall_success_rate: i32,
}

/// Get overall spell statistics for display
pub fn get_spell_stats(player: &You) -> SpellStats {
    #[cfg(feature = "extensions")]
    {
        SpellStats {
            highest_mastery: player.mastery_tracker.highest_mastery(),
            highest_specialization: player.specialization_tracker.highest_level(),
            total_spells_cast: player.mastery_tracker.total_spells_cast,
            overall_success_rate: player.mastery_tracker.overall_success_rate(),
        }
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = player;
        SpellStats {
            highest_mastery: SpellMastery::Unknown,
            total_spells_cast: 0,
            overall_success_rate: 0,
        }
    }
}

#[cfg(all(test, feature = "extensions"))]
mod tests {
    use super::*;

    #[test]
    fn test_tick_spell_synergies() {
        let mut player = You::default();
        tick_spell_synergies(&mut player);
        assert!(true);
    }

    #[test]
    fn test_record_spell_cast() {
        let mut player = You::default();
        let _messages = record_spell_cast(&mut player, SpellType::ForceBolt, true);
        assert_eq!(player.mastery_tracker.total_spells_cast, 1);
    }

    #[test]
    fn test_record_critical_spell_hit() {
        let mut player = You::default();
        record_spell_cast(&mut player, SpellType::ForceBolt, true);
        record_critical_spell_hit(&mut player, SpellType::ForceBolt);
        let critical_hits = player
            .mastery_tracker
            .spells
            .get(&SpellType::ForceBolt)
            .map(|p| p.critical_hits)
            .unwrap_or(0);
        assert_eq!(critical_hits, 1);
    }

    #[test]
    fn test_get_total_spell_damage_bonus() {
        let player = You::default();
        let bonus = get_total_spell_damage_bonus(&player, SpellType::ForceBolt);
        assert!(bonus > 0.0);
    }

    #[test]
    fn test_calculate_final_spell_mana_cost() {
        let player = You::default();
        let cost = calculate_final_spell_mana_cost(&player, 100, SpellType::ForceBolt);
        assert!(cost >= 50);
        assert!(cost <= 100);
    }

    #[test]
    fn test_get_spell_failure_reduction() {
        let player = You::default();
        let reduction = get_spell_failure_reduction(&player, SpellType::ForceBolt);
        assert_eq!(reduction, 0);
    }

    #[test]
    fn test_get_spell_stats() {
        let player = You::default();
        let stats = get_spell_stats(&player);
        assert_eq!(stats.total_spells_cast, 0);
    }
}
