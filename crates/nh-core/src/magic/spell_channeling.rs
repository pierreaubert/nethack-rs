//! Spell channeling system - Hold spells ready to cast instantly
//!
//! Allows players to concentrate on a spell over multiple turns, then release it
//! instantly when ready. Requires maintaining concentration and energy.

use crate::magic::spell::SpellType;
use crate::player::You;
use serde::{Deserialize, Serialize};

/// A spell being held in readiness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanneledSpell {
    /// The spell being channeled
    pub spell_type: SpellType,
    /// Direction the spell will be cast (if any)
    pub direction: Option<(i8, i8)>,
    /// Turns spent channeling so far
    pub turns_invested: u32,
    /// Total mana reserved for this spell
    pub mana_reserved: i32,
    /// Whether concentration was broken this turn
    pub concentration_broken: bool,
}

impl ChanneledSpell {
    /// Create a new channeled spell
    pub fn new(spell_type: SpellType, direction: Option<(i8, i8)>, mana_cost: i32) -> Self {
        Self {
            spell_type,
            direction,
            turns_invested: 0,
            mana_reserved: mana_cost,
            concentration_broken: false,
        }
    }

    /// Get the spell's school for concentration DC
    pub fn school(&self) -> crate::magic::spell::SpellSchool {
        self.spell_type.school()
    }
}

/// Tracking for a channeled spell
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpellChannelTracker {
    /// Currently channeled spell (if any)
    pub active_spell: Option<ChanneledSpell>,
    /// Concentration bonus from spell school specialization
    pub concentration_bonus: i32,
    /// Number of times concentration was broken
    pub breaks_suffered: u32,
}

impl SpellChannelTracker {
    /// Create new channel tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if player is currently channeling
    pub fn is_channeling(&self) -> bool {
        self.active_spell.is_some()
    }

    /// Get the active spell if channeling
    pub fn active_spell(&self) -> Option<&ChanneledSpell> {
        self.active_spell.as_ref()
    }

    /// Get mutable reference to active spell
    pub fn active_spell_mut(&mut self) -> Option<&mut ChanneledSpell> {
        self.active_spell.as_mut()
    }

    /// Reset tracker (for level change or save/load)
    pub fn reset(&mut self) {
        self.active_spell = None;
        self.concentration_bonus = 0;
        self.breaks_suffered = 0;
    }
}

/// Begin channeling a spell
pub fn begin_channeling(
    player: &mut You,
    spell_type: SpellType,
    direction: Option<(i8, i8)>,
    mana_cost: i32,
) -> Result<String, String> {
    // Can't start channeling if already channeling
    if player.casting_spell.is_some() {
        return Err("You are already casting a spell!".to_string());
    }

    // Check if player has enough mana
    if player.energy < mana_cost {
        return Err("Not enough energy to begin channeling this spell.".to_string());
    }

    // Start channeling
    player.casting_spell = Some(spell_type as u32);
    player.casting_turns_remaining = 1; // Mark as channeling (1+ turns)

    let mut tracker = &mut player.casting_spell;
    let channeled = ChanneledSpell::new(spell_type, direction, mana_cost);

    Ok(format!("You begin channeling {}...", spell_type.name()))
}

/// Advance channeling by one turn
pub fn advance_channeling(player: &mut You, rng: &mut crate::rng::GameRng) -> Vec<String> {
    let mut messages = Vec::new();

    if !player.casting_spell.is_some() {
        return messages;
    }

    // Check concentration
    let concentration_dc = 10; // Base DC for maintaining concentration
    let player_level_bonus = player.exp_level / 2;

    // Roll to maintain concentration
    let roll = rng.rnd(20) as i32;
    let total = roll + player_level_bonus;

    if total < concentration_dc {
        // Concentration broken
        messages.push("Your concentration breaks!".to_string());
        player.casting_spell = None;
        player.casting_turns_remaining = 0;
        player.casting_interrupted = true;
    } else {
        // Concentration maintained, advance
        player.casting_turns_remaining += 1;
        messages.push("You maintain your concentration...".to_string());
    }

    messages
}

/// Release a channeled spell immediately
pub fn release_channeled_spell(player: &mut You) -> Option<SpellType> {
    if player.casting_spell.is_none() {
        return None;
    }

    let spell_id = player.casting_spell.take()?;
    player.casting_turns_remaining = 0;

    SpellType::from_id(spell_id as i32)
}

/// Check if player must make concentration check
pub fn check_concentration_interrupt(
    player: &mut You,
    damage_taken: i32,
    rng: &mut crate::rng::GameRng,
) -> bool {
    if !player.casting_spell.is_some() {
        return false;
    }

    // Difficulty increases with damage taken
    let dc = 10 + (damage_taken / 5).max(1);
    let player_level_bonus = player.exp_level / 2;

    let roll = rng.rnd(20) as i32;
    let total = roll + player_level_bonus;

    if total < dc {
        player.casting_spell = None;
        player.casting_turns_remaining = 0;
        player.casting_interrupted = true;
        true
    } else {
        false
    }
}

/// Tick all channeling effects (called once per turn)
pub fn tick_channeling(player: &mut You, rng: &mut crate::rng::GameRng) -> Vec<String> {
    let mut messages = Vec::new();

    if !player.casting_spell.is_some() {
        return messages;
    }

    messages.extend(advance_channeling(player, rng));

    messages
}

/// Cost of maintaining concentration per turn (mana drain)
pub fn concentration_upkeep_cost(spell_type: SpellType) -> i32 {
    // Cost per turn to maintain concentration
    spell_type.energy_cost() / 4
}

/// Get human-readable status of channeling
pub fn get_channeling_status(player: &You) -> Option<String> {
    player.casting_spell.and_then(|spell_id| {
        SpellType::from_id(spell_id as i32).map(|spell| {
            format!(
                "Channeling {} ({} turns)",
                spell.name(),
                player.casting_turns_remaining
            )
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channeled_spell_creation() {
        let spell = ChanneledSpell::new(SpellType::ForceBolt, Some((1, 0)), 10);
        assert_eq!(spell.spell_type, SpellType::ForceBolt);
        assert_eq!(spell.direction, Some((1, 0)));
        assert_eq!(spell.turns_invested, 0);
        assert_eq!(spell.mana_reserved, 10);
    }

    #[test]
    fn test_channel_tracker_creation() {
        let tracker = SpellChannelTracker::new();
        assert!(!tracker.is_channeling());
        assert!(tracker.active_spell().is_none());
    }

    #[test]
    fn test_concentration_upkeep_cost() {
        let cost = concentration_upkeep_cost(SpellType::Fireball);
        assert!(cost > 0);
        assert!(cost < SpellType::Fireball.energy_cost());
    }

    #[test]
    fn test_begin_channeling_not_enough_mana() {
        let mut player = You::default();
        player.energy = 1;

        let result = begin_channeling(&mut player, SpellType::Fireball, None, 100);
        assert!(result.is_err());
        assert!(!player.casting_spell.is_some());
    }

    #[test]
    fn test_release_channeled_spell() {
        let mut player = You::default();
        player.casting_spell = Some(SpellType::ForceBolt as u32);

        let spell = release_channeled_spell(&mut player);
        assert!(spell.is_some());
        assert_eq!(spell, Some(SpellType::ForceBolt));
        assert!(!player.casting_spell.is_some());
    }
}
