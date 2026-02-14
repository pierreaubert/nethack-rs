//! Monster item usage bridge functions
//!
//! Implements monster-specific wrappers around item usage mechanics like wand zapping
//! and horn playing. These functions bridge the gap between monster AI decisions
//! and the core game item effects.

use crate::dungeon::Level;
use crate::magic::zap::{ZapResult, ZapType, check_wand_breakage, degrade_wand};
use crate::object::Object;
use crate::rng::GameRng;

use super::Monster;

/// Monster zaps a wand, consuming charges and checking for breakage
///
/// Simplified implementation of mzapwand() from muse.c. Handles wand charge
/// consumption, degradation, and breakage detection.
///
/// # Arguments
/// * `monster` - The monster using the wand
/// * `wand` - The wand being zapped (modified in place)
/// * `rng` - Random number generator
///
/// # Returns
/// true if wand still exists and can be used, false if broke
pub fn mzapwand(_monster: &mut Monster, wand: &mut Object, rng: &mut GameRng) -> bool {
    // Check wand has charges
    if wand.enchantment <= 0 {
        return false;
    }

    // Decrement charges
    wand.enchantment -= 1;

    // Increment usage counter
    wand.wand_use_count += 1;

    // Check for breakage
    let broke = check_wand_breakage(wand, rng);
    if broke {
        // Wand broke - set enchantment to 0
        wand.enchantment = 0;
        return false;
    }

    // Degrade effectiveness
    degrade_wand(wand, rng);

    // Return true if wand still has charges
    wand.enchantment > 0
}

/// Horn effect types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HornEffect {
    /// Fire beam with specified range
    FireBeam(i32),
    /// Frost beam with specified range
    FrostBeam(i32),
    /// Bugle summon (awaken soldiers)
    BugleSummon,
    /// No effect (broke or incompatible)
    NoEffect,
}

/// Monster plays a horn, handling charge consumption and degradation
///
/// Simplified implementation of mplayhorn() from muse.c. Handles different horn types
/// and degradation of horn durability.
///
/// # Arguments
/// * `monster` - The monster playing the horn (unused in simplified version)
/// * `horn` - The horn being played (modified in place)
/// * `rng` - Random number generator
///
/// # Returns
/// The type of effect the horn produced
pub fn mplayhorn(_monster: &mut Monster, horn: &mut Object, rng: &mut GameRng) -> HornEffect {
    // Identify horn type by object_type
    // TODO: These should match ObjectType enum values for horn types
    let effect = match horn.object_type {
        // Fire horn - produces fire beam
        2100 => {
            // Random range: 1d6+6
            let range = rng.rnd(6) as i32 + 6;
            HornEffect::FireBeam(range)
        }
        // Frost horn - produces frost beam
        2101 => {
            // Random range: 1d6+6
            let range = rng.rnd(6) as i32 + 6;
            HornEffect::FrostBeam(range)
        }
        // Bugle of the Valkyries - summons soldiers
        2102 => HornEffect::BugleSummon,
        // Unknown horn type
        _ => HornEffect::NoEffect,
    };

    // Degrade horn durability
    // Simplified: 1/100 chance to break
    if rng.rn2(100) == 0 {
        horn.enchantment = 0;
    }

    effect
}

/// Map object type to wand zap type for monster wand attacks
///
/// Translates between object types and ZapType enum for use with the zap system.
/// Returns None if the object is not a valid wand.
pub fn object_to_zap_type(object_type: i16) -> Option<ZapType> {
    match object_type {
        // Magic missile wand
        2000 => Some(ZapType::MagicMissile),
        // Fire wand
        2001 => Some(ZapType::Fire),
        // Cold wand
        2002 => Some(ZapType::Cold),
        // Sleep wand
        2003 => Some(ZapType::Sleep),
        // Death wand
        2004 => Some(ZapType::Death),
        // Lightning wand
        2005 => Some(ZapType::Lightning),
        // Poison gas wand
        2006 => Some(ZapType::PoisonGas),
        // Acid wand
        2007 => Some(ZapType::Acid),
        // Unknown or non-wand type
        _ => None,
    }
}

/// Monster fires a wand bolt using the zap system
///
/// Wrapper that integrates monster wand usage with the existing zap damage system.
/// Handles direction calculation and calls into the core zap functions.
///
/// # Arguments
/// * `monster` - The monster firing the wand
/// * `wand` - The wand object
/// * `direction` - Direction to fire (dx, dy)
/// * `range` - Maximum range of the bolt
/// * `level` - The level to apply effects on
/// * `rng` - Random number generator
///
/// # Returns
/// Result of the zap attack
///
/// # Note
/// This is a simplified implementation. Full version would call buzz() from zap.rs
/// with appropriate direction and range parameters.
pub fn monster_zap_wand(
    _monster: &Monster,
    wand: &Object,
    _direction: (i8, i8),
    _range: i32,
    _level: &mut Level,
    _rng: &mut GameRng,
) -> ZapResult {
    let mut result = ZapResult::new();

    // Map wand to zap type
    if let Some(zap_type) = object_to_zap_type(wand.object_type) {
        // Use bhit for ray tracing from monster position toward target
        let bhit_result = crate::magic::zap::bhit(
            _monster.x,
            _monster.y,
            _direction.0,
            _direction.1,
            _range,
            0, // player_x - target position for hit detection
            0, // player_y - target position for hit detection
            _level,
        );

        // Calculate damage based on zap type
        let damage =
            crate::magic::zap::zap_damage(zap_type, crate::magic::zap::ZapVariant::Wand, _rng);

        // Report what was hit
        let hit_desc = if bhit_result.hit_player {
            "hit player"
        } else if bhit_result.hit_monster.is_some() {
            "hit monster"
        } else if bhit_result.hit_wall {
            "hit wall"
        } else {
            "missed"
        };

        result.messages.push(format!(
            "The {} zaps a wand! (damage: {}, {})",
            _monster.name, damage, hit_desc
        ));
    } else {
        result.messages.push("Nothing happens.".to_string());
    }

    result
}
