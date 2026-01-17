//! Player attacks monster combat (uhitm.c)
//!
//! Handles all combat initiated by the player against monsters.

use super::CombatResult;
use crate::monster::Monster;
use crate::object::Object;
use crate::player::You;
use crate::rng::GameRng;

/// Calculate the player's to-hit bonus
///
/// Based on find_roll_to_hit() in uhitm.c
pub fn calculate_to_hit(player: &You, _target: &Monster, _weapon: Option<&Object>) -> i32 {
    let mut to_hit: i32 = 1; // base

    // Add luck
    to_hit += player.luck as i32;

    // TODO: Add all modifiers from find_roll_to_hit()
    // - Armor bonuses
    // - Target AC
    // - Player level
    // - Encumbrance penalties
    // - Trap penalties
    // - Weapon hit bonuses
    // - Monster state bonuses (stunned, fleeing, sleeping, etc.)

    to_hit
}

/// Roll to hit a monster
///
/// Returns true if the attack hits
pub fn attack_hits(to_hit: i32, target_ac: i8, rng: &mut GameRng) -> bool {
    let roll = rng.rnd(20) as i32;
    roll + to_hit > 10 - target_ac as i32
}

/// Player melee attack against monster
pub fn player_attack_monster(
    player: &mut You,
    target: &mut Monster,
    weapon: Option<&Object>,
    rng: &mut GameRng,
) -> CombatResult {
    let to_hit = calculate_to_hit(player, target, weapon);

    // Get target AC from monster data
    // TODO: Get actual AC from permonst data
    let target_ac = 10i8;

    if !attack_hits(to_hit, target_ac, rng) {
        return CombatResult::MISS;
    }

    // Calculate damage
    // TODO: Implement full damage calculation
    let base_damage = match weapon {
        Some(_w) => {
            // TODO: Get weapon damage dice
            rng.dice(1, 6) as i32
        }
        None => {
            // Bare hands - 1d2 for most, more for monks
            rng.dice(1, 2) as i32
        }
    };

    // TODO: Add strength bonus, enchantment, etc.
    let damage = base_damage;

    // Apply damage to monster
    target.hp -= damage;

    CombatResult {
        hit: true,
        defender_died: target.hp <= 0,
        attacker_died: false,
        damage,
        special_effect: None,
    }
}
