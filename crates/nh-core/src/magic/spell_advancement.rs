//! Spell advancement integration
//!
//! Handles spell synergies, school specialization, and mastery advancement
//! mechanics during spell casting and each game turn.

use crate::magic::spell::{SpellMastery, SpellSchool, SpellType};
use crate::magic::{
    calculate_specialization_mana_cost, calculate_synergy_mana_cost, get_mastery_damage_bonus,
    get_mastery_mana_efficiency, get_specialization_damage_bonus,
    get_specialization_failure_reduction,
};
use crate::player::You;

/// Tick spell synergy tracker (called once per turn to age recent spells)
pub fn tick_spell_synergies(player: &mut You) {
    player.spell_synergy_tracker.tick();
}

/// Record a spell cast and update all advancement trackers
pub fn record_spell_cast(player: &mut You, spell_type: SpellType, succeeded: bool) -> Vec<String> {
    let mut messages = Vec::new();

    // Get spell school for specialization tracking
    let school = spell_type.school();

    // Record to spell synergy tracker
    player
        .spell_synergy_tracker
        .record_spell(spell_type, school);

    // Record to specialization tracker
    player.specialization_tracker.spell_cast_in_school(school);

    // Record to mastery tracker
    player
        .mastery_tracker
        .record_spell_cast(spell_type, succeeded);

    // Check for mastery milestone achievements
    if let Some((_milestone, msg)) = player.mastery_tracker.check_milestone(spell_type) {
        messages.push(msg);
    }

    messages
}

/// Record a critical spell hit (exceeded expected effect)
pub fn record_critical_spell_hit(player: &mut You, spell_type: SpellType) {
    player.mastery_tracker.record_critical_hit(spell_type);
}

/// Get spell damage multiplier from all bonuses (synergies, specialization, mastery)
pub fn get_total_spell_damage_bonus(player: &You, spell_type: SpellType) -> f32 {
    let school = spell_type.school();

    // Get synergies from recent spell tracker
    let synergies = player
        .spell_synergy_tracker
        .check_synergies(spell_type, school);

    // Calculate synergy bonus from all active synergies
    let mut synergy_bonus = 1.0;
    for synergy in &synergies {
        synergy_bonus *= synergy.bonus_multiplier();
    }

    // Get bonus from specialization (1.0 - 2.0x)
    let specialization_bonus =
        get_specialization_damage_bonus(&player.specialization_tracker, school);

    // Get bonus from mastery (0.5 - 1.6x)
    let mastery = player.mastery_tracker.get_mastery(spell_type);
    let mastery_bonus = get_mastery_damage_bonus(mastery);

    // Combine all bonuses multiplicatively
    synergy_bonus * specialization_bonus * mastery_bonus
}

/// Calculate final mana cost with all reductions applied
pub fn calculate_final_spell_mana_cost(
    player: &You,
    base_mana_cost: i32,
    spell_type: SpellType,
) -> i32 {
    let school = spell_type.school();

    // Start with specialization reduction
    let cost_after_spec =
        calculate_specialization_mana_cost(&player.specialization_tracker, base_mana_cost, school);

    // Get synergies and apply mana reduction
    let synergies = player
        .spell_synergy_tracker
        .check_synergies(spell_type, school);
    let cost_after_synergies = calculate_synergy_mana_cost(cost_after_spec, &synergies);

    // Apply mastery mana efficiency
    let mastery = player.mastery_tracker.get_mastery(spell_type);
    let mastery_efficiency = get_mastery_mana_efficiency(mastery);
    let final_cost = (cost_after_synergies as f32 / mastery_efficiency) as i32;

    // Ensure minimum cost is at least half base
    final_cost.max(base_mana_cost / 2)
}

/// Get spell failure chance reduction from specialization and mastery
pub fn get_spell_failure_reduction(player: &You, spell_type: SpellType) -> i32 {
    let school = spell_type.school();
    let spec_reduction =
        get_specialization_failure_reduction(&player.specialization_tracker, school);
    let mastery = player.mastery_tracker.get_mastery(spell_type);

    // Mastery failure reduction: Unknown=0, Novice=10, Adept=20, Expert=30, Master=40
    let mastery_reduction = match mastery {
        SpellMastery::Unknown => 0,
        SpellMastery::Novice => 10,
        SpellMastery::Adept => 20,
        SpellMastery::Expert => 30,
        SpellMastery::Master => 40,
    };

    // Combine reductions (cap at 90% total reduction)
    (spec_reduction + mastery_reduction).min(90)
}

/// Get overall spell statistics for display
pub fn get_spell_stats(player: &You) -> SpellStats {
    let highest_mastery = player.mastery_tracker.highest_mastery();
    let highest_specialization = player.specialization_tracker.highest_level();
    let total_spells_cast = player.mastery_tracker.total_spells_cast;
    let overall_success_rate = player.mastery_tracker.overall_success_rate();

    SpellStats {
        highest_mastery,
        highest_specialization,
        total_spells_cast,
        overall_success_rate,
    }
}

/// Spell statistics for UI display
#[derive(Debug, Clone)]
pub struct SpellStats {
    pub highest_mastery: SpellMastery,
    pub highest_specialization: crate::magic::school_specialization::SpecializationLevel,
    pub total_spells_cast: i32,
    pub overall_success_rate: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_spell_synergies() {
        let mut player = You::default();
        tick_spell_synergies(&mut player);
        // Should not panic
        assert!(true);
    }

    #[test]
    fn test_record_spell_cast() {
        let mut player = You::default();
        let messages = record_spell_cast(&mut player, SpellType::ForceBolt, true);
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
        // With all defaults, should be around 0.5 * 1.0 * 1.0 = 0.5
        assert!(bonus > 0.0);
    }

    #[test]
    fn test_calculate_final_spell_mana_cost() {
        let player = You::default();
        let cost = calculate_final_spell_mana_cost(&player, 100, SpellType::ForceBolt);
        // Should be at least half the base cost
        assert!(cost >= 50);
        assert!(cost <= 100);
    }

    #[test]
    fn test_get_spell_failure_reduction() {
        let player = You::default();
        let reduction = get_spell_failure_reduction(&player, SpellType::ForceBolt);
        // Should be 0 at start
        assert_eq!(reduction, 0);
    }

    #[test]
    fn test_get_spell_stats() {
        let player = You::default();
        let stats = get_spell_stats(&player);
        assert_eq!(stats.total_spells_cast, 0);
    }
}
