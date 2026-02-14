//! Combat hooks for Phase 18 integration (combat_hooks.rs)
//!
//! Provides integration points for Phase 18 combat AI systems to hook into
//! the combat resolution pipeline. These functions are called AFTER combat
//! is resolved to update morale, combat memory, and other AI state.

use crate::combat::{AttackType, DamageType};
use crate::dungeon::Level;
use crate::monster::{Monster, MonsterId};
use crate::player::You;

use super::attack_selection::AttackResult;
use super::morale::MoraleEvent;

/// Called after a monster successfully hits the player
/// Updates combat memory and morale based on the hit
pub fn on_monster_hit_player(
    monster_id: MonsterId,
    level: &mut Level,
    damage_dealt: i32,
    attack_type: AttackType,
    damage_type: DamageType,
) {
    if let Some(monster) = level.monster_mut(monster_id) {
        // Record successful attack in combat memory
        monster.combat_memory.record_attack(AttackResult {
            attack_type,
            hit: true,
            damage_dealt,
            age_turns: 0,
        });

        // Add morale event for successful attack
        monster.morale.add_event(MoraleEvent::SuccessfulAttack);
    }
}

/// Called after a monster misses the player
/// Updates combat memory to reflect missed attacks
pub fn on_monster_miss_player(monster_id: MonsterId, level: &mut Level, attack_type: AttackType) {
    if let Some(monster) = level.monster_mut(monster_id) {
        // Record missed attack
        monster.combat_memory.record_attack(AttackResult {
            attack_type,
            hit: false,
            damage_dealt: 0,
            age_turns: 0,
        });
    }
}

/// Called when the player hits a monster
/// Updates monster threat assessment and morale
pub fn on_player_hit_monster(
    monster_id: MonsterId,
    level: &mut Level,
    damage_dealt: i32,
    player: &You,
) {
    if let Some(monster) = level.monster_mut(monster_id) {
        let hp_before = monster.hp;
        let hp_percent = (monster.hp as f32 / monster.hp_max as f32) * 100.0;

        // Record heavy damage as a morale event
        if damage_dealt > monster.hp_max / 4 {
            monster.morale.add_event(MoraleEvent::TookHeavyDamage);
        }

        // Record near-death
        if monster.hp < monster.hp_max / 4 {
            monster.morale.add_event(MoraleEvent::NearDeath);
        }

        // Record player power observation if player dealt significant damage
        if damage_dealt > 20 || (damage_dealt > 10 && hp_percent < 50.0) {
            monster.morale.add_event(MoraleEvent::WitnessedPlayerPower);
        }

        // Update threat level based on player demonstrated power
        if player.level.level_num as i32 > (monster.level as i32 + 2) {
            monster.threat_level = super::ThreatLevel::High;
        }
    }
}

/// Called when a nearby monster is killed
/// Triggers morale events for witnesses
pub fn on_nearby_monster_death(
    monster_id: MonsterId,
    level: &mut Level,
    dead_monster_id: MonsterId,
    proximity_sq: i32, // Distance squared
) {
    // Get dead monster's type first to avoid borrow issues
    let dead_monster_type = level.monster(dead_monster_id).map(|m| m.monster_type);

    if let Some(witness) = level.monster_mut(monster_id) {
        // Only record if relatively nearby (within 100 squares = radius 10)
        if proximity_sq <= 100 {
            // Same type allies feel more impact
            if dead_monster_type == Some(witness.monster_type) {
                witness.morale.add_event(MoraleEvent::AlliedDeath);
                witness.morale.ally_deaths_witnessed += 1;
            } else if proximity_sq <= 25 {
                // Nearby death of any creature still affects morale
                witness.morale.add_event(MoraleEvent::AlliedDeath);
                witness.morale.ally_deaths_witnessed =
                    witness.morale.ally_deaths_witnessed.saturating_add(1);
            }
        }
    }
}

/// Called after an ability is used to update cooldowns and resources
pub fn on_ability_used(
    monster_id: MonsterId,
    level: &mut Level,
    ability_name: &str,
    mana_cost: i32,
    cooldown_turns: u16,
) {
    if let Some(monster) = level.monster_mut(monster_id) {
        monster
            .resources
            .use_ability(ability_name, mana_cost, cooldown_turns, false);
    }
}

/// Called when breath weapon is used
pub fn on_breath_weapon_used(monster_id: MonsterId, level: &mut Level) {
    if let Some(monster) = level.monster_mut(monster_id) {
        monster.resources.use_breath();
    }
}

/// Called when spell is cast
pub fn on_spell_cast(monster_id: MonsterId, level: &mut Level, mana_cost: i32) {
    if let Some(monster) = level.monster_mut(monster_id) {
        monster.resources.use_spell();
        monster.resources.mana_current = (monster.resources.mana_current - mana_cost).max(0);
    }
}

/// Record observed resistance from combat
pub fn record_resistance_observation(
    monster_id: MonsterId,
    level: &mut Level,
    damage_type: DamageType,
    target_resists: bool,
) {
    if let Some(monster) = level.monster_mut(monster_id) {
        monster
            .combat_memory
            .record_resistance(damage_type, target_resists);
    }
}

/// Initialize combat resources for a monster based on level
pub fn initialize_monster_combat_resources(monster_id: MonsterId, level: &mut Level) {
    if let Some(monster) = level.monster_mut(monster_id) {
        let monster_level = monster.level;
        monster.resources.initialize(monster_level);
    }
}

/// Called each turn to update all AI state for combat readiness
pub fn update_monster_combat_readiness(monster_id: MonsterId, level: &mut Level, player: &You) {
    if let Some(monster) = level.monster_mut(monster_id) {
        // Age morale events
        monster.morale.age_events();

        // Tick down cooldowns and regenerate resources
        monster.resources.tick_cooldowns();
        monster.resources.regenerate_mana();

        // Update threat assessment
        let threat = match (player.level.level_num as i32, player.hp as i32) {
            (level, _hp) if level > (monster.level as i32 + 5) => super::ThreatLevel::Critical,
            (level, _hp) if level > (monster.level as i32 + 2) => super::ThreatLevel::High,
            (_level, hp) if hp > 100 => super::ThreatLevel::Moderate,
            (level, _hp) if level < (monster.level as i32 - 2) => super::ThreatLevel::Low,
            _ => super::ThreatLevel::Moderate,
        };
        monster.threat_level = threat;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::DLevel;
    use crate::monster::MonsterId;

    #[test]
    fn test_on_monster_hit_player() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 5, 5, 5);
        level.add_monster(monster);

        on_monster_hit_player(
            MonsterId(1),
            &mut level,
            10,
            AttackType::Bite,
            DamageType::Physical,
        );

        let monster = level.monster(MonsterId(1)).unwrap();
        assert_eq!(monster.combat_memory.recent_attacks.len(), 1);
    }

    #[test]
    fn test_on_nearby_monster_death() {
        let mut level = Level::new(DLevel::main_dungeon_start());

        let m1 = Monster::new(MonsterId(1), 5, 5, 5);
        level.add_monster(m1);

        let m2 = Monster::new(MonsterId(2), 5, 7, 7);
        level.add_monster(m2);

        on_nearby_monster_death(MonsterId(1), &mut level, MonsterId(2), 8);

        let witness = level.monster(MonsterId(1)).unwrap();
        assert_eq!(witness.morale.ally_deaths_witnessed, 1);
    }
}
