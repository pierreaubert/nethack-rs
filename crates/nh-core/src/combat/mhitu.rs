//! Monster attacks player combat (mhitu.c)
//!
//! Handles all combat initiated by monsters against the player.

use super::{Attack, CombatEffect, CombatResult, DamageType};
use crate::monster::Monster;
use crate::player::You;
use crate::rng::GameRng;

/// Monster melee attack against player
pub fn monster_attack_player(
    _attacker: &Monster,
    player: &mut You,
    attack: &Attack,
    rng: &mut GameRng,
) -> CombatResult {
    // TODO: Check if monster can reach player (distance, engulfed, etc.)

    // Calculate to-hit
    // TODO: Get monster level and other modifiers
    let monster_level = 1i32; // TODO: From permonst
    let to_hit = monster_level + 10;

    // Roll to hit
    let roll = rng.rnd(20) as i32;
    if roll + to_hit <= player.armor_class as i32 + 10 {
        return CombatResult::MISS;
    }

    // Calculate base damage
    let damage = rng.dice(attack.dice_num as u32, attack.dice_sides as u32) as i32;

    // Apply special damage effects based on damage type
    let special_effect = apply_damage_effect(attack.damage_type, player, damage, rng);

    // Apply damage to player
    player.hp -= damage;

    CombatResult {
        hit: true,
        defender_died: player.hp <= 0,
        attacker_died: false,
        damage,
        special_effect,
    }
}

/// Apply special effects based on damage type
fn apply_damage_effect(
    damage_type: DamageType,
    _player: &mut You,
    _damage: i32,
    rng: &mut GameRng,
) -> Option<CombatEffect> {
    match damage_type {
        DamageType::Physical => None,

        DamageType::Fire => {
            // TODO: Check fire resistance
            // TODO: Burn inventory items
            None
        }

        DamageType::Cold => {
            // TODO: Check cold resistance
            // TODO: Freeze potions
            None
        }

        DamageType::Sleep => {
            // TODO: Check sleep resistance
            if rng.one_in(3) {
                // TODO: Actually put player to sleep
                Some(CombatEffect::Paralyzed)
            } else {
                None
            }
        }

        DamageType::DrainLife => {
            // TODO: Check drain resistance
            // TODO: Drain experience level
            Some(CombatEffect::Drained)
        }

        DamageType::Stone => {
            // TODO: Check stone resistance
            Some(CombatEffect::Petrifying)
        }

        DamageType::DrainStrength => {
            // TODO: Check poison resistance
            // TODO: Actually drain strength
            Some(CombatEffect::Poisoned)
        }

        DamageType::Confuse => {
            // TODO: Actually confuse player
            Some(CombatEffect::Confused)
        }

        DamageType::Stun => {
            // TODO: Actually stun player
            Some(CombatEffect::Stunned)
        }

        DamageType::Blind => {
            // TODO: Actually blind player
            Some(CombatEffect::Blinded)
        }

        DamageType::Paralyze => {
            // TODO: Check free action
            Some(CombatEffect::Paralyzed)
        }

        DamageType::StealGold => {
            // TODO: Actually steal gold
            Some(CombatEffect::GoldStolen)
        }

        DamageType::StealItem => {
            // TODO: Actually steal item
            Some(CombatEffect::ItemStolen)
        }

        DamageType::Teleport => {
            // TODO: Actually teleport player
            Some(CombatEffect::Teleported)
        }

        DamageType::Digest => Some(CombatEffect::Engulfed),

        DamageType::Wrap | DamageType::Stick => Some(CombatEffect::Grabbed),

        _ => None,
    }
}
