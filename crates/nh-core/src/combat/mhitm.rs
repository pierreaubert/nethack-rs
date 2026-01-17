//! Monster attacks monster combat (mhitm.c)
//!
//! Handles all combat between monsters (including pets).

use super::{Attack, CombatResult};
use crate::monster::Monster;
use crate::rng::GameRng;

/// Monster melee attack against another monster
pub fn monster_attack_monster(
    _attacker: &mut Monster,
    defender: &mut Monster,
    attack: &Attack,
    rng: &mut GameRng,
) -> CombatResult {
    // TODO: Check if monsters can fight (same square, etc.)

    // Calculate to-hit
    // TODO: Get attacker level
    let attacker_level = 1i32; // TODO: From permonst
    let to_hit = attacker_level + 10;

    // TODO: Get defender AC from permonst
    let defender_ac = 10i8;

    // Roll to hit
    let roll = rng.rnd(20) as i32;
    if roll + to_hit <= defender_ac as i32 + 10 {
        return CombatResult::MISS;
    }

    // Calculate base damage
    let damage = rng.dice(attack.dice_num as u32, attack.dice_sides as u32) as i32;

    // Apply damage to defender
    defender.hp -= damage;

    // TODO: Apply special effects
    // TODO: Handle attacker death (cockatrice corpse, etc.)

    CombatResult {
        hit: true,
        defender_died: defender.hp <= 0,
        attacker_died: false,
        damage,
        special_effect: None,
    }
}
