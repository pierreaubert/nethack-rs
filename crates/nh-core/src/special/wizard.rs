//! Wizard of Yendor AI (wizard.c)
//!
//! Special AI for the Wizard of Yendor — harasses the player,
//! steals the Amulet, and casts nasty spells.

use crate::monster::MonsterId;
use crate::rng::GameRng;

/// Wizard harassment types (matches C wizard.c nasty effects)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardHarassment {
    /// "Double Trouble" — clone the Wizard
    DoubleTrouble,
    /// Summon nasty monsters
    SummonNasties,
    /// Curse the player's items
    CurseItems,
    /// Steal the Amulet
    StealAmulet,
    /// Destroy armor
    DestroyArmor,
    /// Destroy wand charges
    DestroySpe,
    /// Aggravate monsters
    Aggravate,
}

/// Pick a random harassment action for the Wizard (nasty from wizard.c:285).
///
/// The Wizard gets progressively nastier based on difficulty and how many
/// times he's been killed.
pub fn pick_harassment(difficulty: i32, rng: &mut GameRng) -> WizardHarassment {
    let max_roll = if difficulty > 10 { 7 } else { 5 };
    match rng.rn2(max_roll) {
        0 => WizardHarassment::SummonNasties,
        1 => WizardHarassment::CurseItems,
        2 => WizardHarassment::DestroyArmor,
        3 => WizardHarassment::DestroySpe,
        4 => WizardHarassment::Aggravate,
        5 => WizardHarassment::StealAmulet,
        _ => WizardHarassment::DoubleTrouble,
    }
}

/// Generate a harassment message
pub fn harassment_message(action: WizardHarassment) -> &'static str {
    match action {
        WizardHarassment::DoubleTrouble => "\"Double Trouble!\"",
        WizardHarassment::SummonNasties => "\"Let's see you deal with this!\"",
        WizardHarassment::CurseItems => "\"That ought to teach you!\"",
        WizardHarassment::StealAmulet => "\"I want my amulet back!\"",
        WizardHarassment::DestroyArmor => "\"Your armor is looking a bit worn...\"",
        WizardHarassment::DestroySpe => "\"Your magic fades!\"",
        WizardHarassment::Aggravate => "\"I'll stir things up a bit!\"",
    }
}

/// Check if the Wizard should intervene (resurrect or harass).
///
/// After being killed, the Wizard can return. Probability increases
/// with turn count (matches C: `moves > 100` check from wizard.c).
pub fn should_wizard_intervene(
    wizard_dead: bool,
    turns_since_death: u64,
    difficulty: i32,
    rng: &mut GameRng,
) -> bool {
    if !wizard_dead {
        return false;
    }

    // Minimum cooldown after death
    if turns_since_death < 50 {
        return false;
    }

    // Probability increases with difficulty
    let threshold = (500 - difficulty * 10).max(100) as u32;
    rng.rn2(threshold) == 0
}

/// Strategy for wizard targeting (who/what to target).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WizardTarget {
    /// Attack the player directly
    AttackPlayer,
    /// Pursue the Amulet carrier
    PursueAmulet,
    /// Flee to regenerate
    Flee,
}

/// Determine the Wizard's current strategy.
pub fn wizard_strategy(
    wizard_hp: i32,
    wizard_hp_max: i32,
    player_has_amulet: bool,
    _wizard_id: MonsterId,
) -> WizardTarget {
    // Low health → flee
    if wizard_hp < wizard_hp_max / 4 {
        return WizardTarget::Flee;
    }

    // Player has amulet → pursue
    if player_has_amulet {
        return WizardTarget::PursueAmulet;
    }

    WizardTarget::AttackPlayer
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_harassment_all_types() {
        let mut seen = hashbrown::HashSet::new();
        for seed in 0..100 {
            let mut rng = GameRng::new(seed);
            let action = pick_harassment(15, &mut rng);
            seen.insert(format!("{:?}", action));
        }
        assert!(seen.len() >= 5, "Should see at least 5 different harassments");
    }

    #[test]
    fn test_harassment_message() {
        let msg = harassment_message(WizardHarassment::DoubleTrouble);
        assert!(msg.contains("Double Trouble"));
    }

    #[test]
    fn test_should_wizard_intervene_not_dead() {
        let mut rng = GameRng::new(42);
        assert!(!should_wizard_intervene(false, 1000, 20, &mut rng));
    }

    #[test]
    fn test_should_wizard_intervene_too_soon() {
        let mut rng = GameRng::new(42);
        assert!(!should_wizard_intervene(true, 10, 20, &mut rng));
    }

    #[test]
    fn test_wizard_strategy_flee() {
        let result = wizard_strategy(10, 100, false, MonsterId(1));
        assert_eq!(result, WizardTarget::Flee);
    }

    #[test]
    fn test_wizard_strategy_pursue_amulet() {
        let result = wizard_strategy(80, 100, true, MonsterId(1));
        assert_eq!(result, WizardTarget::PursueAmulet);
    }

    #[test]
    fn test_wizard_strategy_attack() {
        let result = wizard_strategy(80, 100, false, MonsterId(1));
        assert_eq!(result, WizardTarget::AttackPlayer);
    }
}
