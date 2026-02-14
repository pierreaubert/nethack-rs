//! Advanced spell system orchestration
//!
//! Coordinates metamagic, channeling, ritual casting, research, stances,
//! and spell customization into a unified casting interface.

use crate::dungeon::Level;
use crate::player::You;
use crate::rng::GameRng;
use serde::{Deserialize, Serialize};

use super::casting_stances::StanceTracker;
use super::metamagic::MetamagicKnowledge;
use super::ritual_casting::RitualTracker;
use super::spell_channeling::SpellChannelTracker;
use super::spell_customization::CustomizationTracker;
use super::spell_research::SpellResearchTracker;

/// Complete advanced spell system state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdvancedSpellState {
    pub metamagic_knowledge: MetamagicKnowledge,
    pub channel_tracker: SpellChannelTracker,
    pub ritual_tracker: RitualTracker,
    pub research_tracker: SpellResearchTracker,
    pub stance_tracker: StanceTracker,
    pub customization_tracker: CustomizationTracker,
}

impl AdvancedSpellState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.metamagic_knowledge = MetamagicKnowledge::new();
        self.channel_tracker = SpellChannelTracker::new();
        self.ritual_tracker = RitualTracker::new();
        self.research_tracker = SpellResearchTracker::new();
        self.stance_tracker = StanceTracker::new();
        self.customization_tracker = CustomizationTracker::new();
    }
}

/// Options for enhanced spell casting
#[derive(Debug, Clone, Default)]
pub struct CastingOptions {
    pub metamagics: Vec<super::metamagic::MetamagicType>,
    pub overcharge_mana: i32,
    pub channeling: bool,
    pub ritual: bool,
}

/// Tick all advanced spell systems (called once per game turn)
pub fn tick_advanced_spell_systems(player: &mut You, level: &mut Level) -> Vec<String> {
    let mut messages = Vec::new();

    // Tick channeling
    if player.casting_spell.is_some() {
        // Concentration will be checked in gameloop
        let upkeep = super::spell_channeling::concentration_upkeep_cost(
            super::spell::SpellType::ForceBolt, // Would need to get actual spell
        );
        if player.energy >= upkeep {
            player.energy -= upkeep;
        } else {
            messages.push("You lose concentration on your spell!".to_string());
            player.casting_spell = None;
            player.casting_turns_remaining = 0;
        }
    }

    // Tick stances
    player.advanced_spell_state.stance_tracker.tick();

    // Tick environmental effects on level
    level.terrain_modifications.tick_all();
    level.persistent_effects.tick_all();

    messages
}

/// Apply enhanced casting with all advanced spell features
pub fn enhanced_cast_spell(
    spell: super::spell::SpellType,
    direction: Option<(i8, i8)>,
    player: &mut You,
    level: &mut Level,
    rng: &mut GameRng,
    options: Option<CastingOptions>,
) -> super::spell::SpellResult {
    let options = options.unwrap_or_default();

    // Check casting conditions
    let components = super::spell_conditions::get_spell_components(spell);
    let requires_verbal = components.contains(&super::spell_conditions::SpellComponent::Verbal);
    let requires_somatic = components.contains(&super::spell_conditions::SpellComponent::Somatic);
    let requires_focus = components.contains(&super::spell_conditions::SpellComponent::Focus);

    if let Err(err) = super::spell_conditions::check_casting_conditions(
        player,
        requires_verbal,
        requires_somatic,
        requires_focus,
    ) {
        return super::spell::SpellResult::new().with_message(err.message());
    }

    // Apply stance modifiers
    let stance_mods = player.advanced_spell_state.stance_tracker.get_modifiers();

    // Apply metamagic
    let base_mana = spell.energy_cost();
    let mut final_mana = base_mana;

    if !options.metamagics.is_empty() {
        if let Some(cost) = super::metamagic::calculate_metamagic_cost(
            base_mana,
            &options.metamagics,
            &player.advanced_spell_state.metamagic_knowledge,
        ) {
            final_mana = cost;
        }
    }

    // Apply overcharge
    let mut overcharge_level = super::spell_overcharge::OverchargeLevel::Normal;
    if options.overcharge_mana > 0 {
        overcharge_level = super::spell_overcharge::calculate_overcharge_level(
            final_mana,
            options.overcharge_mana,
        );
        final_mana += options.overcharge_mana;
    }

    // Apply stance mana cost
    final_mana = (final_mana as f32 * stance_mods.mana_multiplier) as i32;

    // Check energy
    if player.energy < final_mana {
        return super::spell::SpellResult::new()
            .with_message("Not enough energy to cast that spell.");
    }

    // Deduct energy
    player.energy -= final_mana;

    // Delegate to original cast_spell
    let mut result = super::spell::cast_spell(spell, direction, player, level, rng);

    // Apply critical
    let critical = super::spell_critical::check_critical_spell(
        player.exp_level,
        super::spell::SpellMastery::Unknown,
        player.luck,
        rng,
    );

    if critical.is_critical {
        result.messages.push(critical.message);
    }

    result.energy_cost = final_mana;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_spell_state_creation() {
        let state = AdvancedSpellState::new();
        assert!(!state.channel_tracker.is_channeling());
    }

    #[test]
    fn test_advanced_spell_state_reset() {
        let mut state = AdvancedSpellState::new();
        state.reset();
        assert!(!state.channel_tracker.is_channeling());
    }

    #[test]
    fn test_casting_options_default() {
        let options = CastingOptions::default();
        assert!(options.metamagics.is_empty());
        assert_eq!(options.overcharge_mana, 0);
    }
}
