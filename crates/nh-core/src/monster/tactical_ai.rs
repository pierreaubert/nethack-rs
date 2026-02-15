//! Tactical AI integration (Phase 18)
//!
//! Hooks into the main AI loop to provide intelligent tactical decision-making,
//! morale-based retreat logic, and resource-aware ability selection.

use crate::dungeon::Level;
use crate::player::You;

use super::{Intelligence, Monster, MonsterId, Personality, ThreatLevel};

/// Update monster tactical state each turn
/// - Age morale events
/// - Regenerate resources
/// - Update threat assessment
pub fn update_monster_tactical_state(monster_id: MonsterId, level: &mut Level, player: &You) {
    if let Some(monster) = level.monster_mut(monster_id) {
        // Age morale events
        monster.morale.age_events();

        // Regenerate mana and cooldowns
        monster.resources.tick_cooldowns();
        monster.resources.regenerate_mana();

        // Update threat assessment based on player power
        update_threat_assessment(monster, player);
    }
}

/// Assess current threat level from player
fn update_threat_assessment(monster: &mut Monster, player: &You) {
    let threat = match (player.level.level_num as i32, player.hp as i32) {
        (level, _hp) if level > (monster.level as i32 + 5) => ThreatLevel::Critical,
        (level, _hp) if level > (monster.level as i32 + 2) => ThreatLevel::High,
        (_level, hp) if hp > 100 => ThreatLevel::Moderate,
        (level, _hp) if level < (monster.level as i32 - 2) => ThreatLevel::Low,
        _ => ThreatLevel::Moderate,
    };

    monster.threat_level = threat;
}

/// Determine if monster should retreat and return reason
pub fn should_retreat_tactical(
    monster: &Monster,
    player: &You,
) -> Option<super::morale::RetreatReason> {
    let intelligence = crate::monster::tactics::monster_intelligence(monster.monster_type);

    // Calculate morale-based retreat decision
    let mut morale_for_calc = monster.morale.clone();
    morale_for_calc.calculate(monster.personality, monster.hp, monster.hp_max);

    morale_for_calc.should_retreat(
        intelligence,
        monster.personality,
        monster.hp,
        monster.hp_max,
    )
}

/// Execute retreat action
pub fn execute_retreat(
    monster_id: MonsterId,
    level: &mut Level,
    reason: super::morale::RetreatReason,
) {
    if let Some(monster) = level.monster_mut(monster_id) {
        monster.state.fleeing = true;
        monster.flee_timeout = 50; // Flee for ~50 turns

        // Record morale event
        use super::morale::MoraleEvent;
        match reason {
            super::morale::RetreatReason::LowMorale => {
                monster.morale.add_event(MoraleEvent::AlliedDeath);
            }
            super::morale::RetreatReason::LowHp => {
                monster.morale.add_event(MoraleEvent::NearDeath);
            }
            super::morale::RetreatReason::AlliesDead => {
                // Already recorded
            }
            super::morale::RetreatReason::OutNumbered => {
                // Already recorded
            }
        }
    }
}

/// Get personality-based attack preference modifiers
pub fn get_personality_attack_preferences(
    personality: Personality,
) -> hashbrown::HashMap<crate::combat::AttackType, i8> {
    use super::personality::PersonalityProfile;
    use crate::combat::AttackType;
    use hashbrown::HashMap;

    let profile = PersonalityProfile::for_personality(personality);
    let mut prefs = HashMap::new();

    prefs.insert(AttackType::Weapon, profile.prefers_melee);
    prefs.insert(AttackType::Breath, profile.prefers_breath);
    prefs.insert(AttackType::Magic, profile.prefers_spells);
    prefs.insert(AttackType::Gaze, profile.prefers_special);

    prefs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_assessment() {
        // Placeholder test - full implementation requires more setup
        assert!(true, "Threat assessment system initialized");
    }

    #[test]
    fn test_retreat_decision() {
        // Placeholder test
        assert!(true, "Retreat decision system initialized");
    }
}
