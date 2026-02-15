//! Monster item usage bridge functions (muse.c equivalent)
//!
//! Implements monster-specific wrappers around item usage mechanics like wand zapping,
//! horn playing, scroll reading, and potion quaffing. These functions bridge the gap
//! between monster AI decisions and the core game item effects.
//!
//! This module covers the execution side of monster item usage. Item *selection*
//! (find_offensive, find_defensive, find_misc) lives in `monster/ai.rs`.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::dungeon::Level;
use crate::magic::zap::{
    BuzzResult, MbhitEffect, ZapResult, ZapType, ZapVariant, buzz, check_wand_breakage,
    degrade_wand, direction_toward, mbhit_effect, muse_horn_to_zap_type, muse_wand_to_zap_type,
};
use crate::object::Object;
use crate::player::You;
use crate::rng::GameRng;

use super::{Monster, MonsterId};

// ============================================================================
// Wand/horn charge management
// ============================================================================

/// Monster zaps a wand, consuming charges and checking for breakage (C: mzapwand)
///
/// # Returns
/// true if wand still exists and can be used, false if broke
pub fn mzapwand(_monster: &mut Monster, wand: &mut Object, rng: &mut GameRng) -> bool {
    if wand.enchantment <= 0 {
        return false;
    }

    wand.enchantment -= 1;
    wand.wand_use_count += 1;

    let broke = check_wand_breakage(wand, rng);
    if broke {
        wand.enchantment = 0;
        return false;
    }

    degrade_wand(wand, rng);
    wand.enchantment > 0
}

/// Horn effect types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HornEffect {
    FireBeam(i32),
    FrostBeam(i32),
    BugleSummon,
    NoEffect,
}

/// Monster plays a horn, handling charge consumption and degradation (C: mplayhorn)
pub fn mplayhorn(_monster: &mut Monster, horn: &mut Object, rng: &mut GameRng) -> HornEffect {
    let effect = match horn.object_type {
        2100 => {
            let range = rng.rnd(6) as i32 + 6;
            HornEffect::FireBeam(range)
        }
        2101 => {
            let range = rng.rnd(6) as i32 + 6;
            HornEffect::FrostBeam(range)
        }
        2102 => HornEffect::BugleSummon,
        _ => HornEffect::NoEffect,
    };

    // 1/100 chance horn breaks
    if rng.rn2(100) == 0 {
        horn.enchantment = 0;
    }

    effect
}

// ============================================================================
// Object type mapping
// ============================================================================

/// Map object type to wand zap type for monster wand attacks
pub fn object_to_zap_type(object_type: i16) -> Option<ZapType> {
    match object_type {
        2000 => Some(ZapType::MagicMissile),
        2001 => Some(ZapType::Fire),
        2002 => Some(ZapType::Cold),
        2003 => Some(ZapType::Sleep),
        2004 => Some(ZapType::Death),
        2005 => Some(ZapType::Lightning),
        2006 => Some(ZapType::PoisonGas),
        2007 => Some(ZapType::Acid),
        _ => None,
    }
}

/// Map breath weapon attack damage type to ZapType
pub fn breath_damage_to_zap_type(damage_type: crate::combat::DamageType) -> Option<ZapType> {
    crate::magic::zap::damage_type_to_zap_type(damage_type)
}

// ============================================================================
// Monster wand/horn ray execution (calls into buzz())
// ============================================================================

/// Monster fires a wand ray using buzz() (C: use_offensive wand cases)
///
/// Traces a ray from monster toward the player using the full buzz() system
/// with wall bounces, resistance checks, and item destruction.
///
/// # Arguments
/// * `monster_x`, `monster_y` - Monster position
/// * `muse_type` - MUSE_WAN_* constant identifying the wand
/// * `range` - Maximum range of the bolt
/// * `player` - Player state
/// * `level` - Level state
/// * `rng` - Random number generator
#[allow(clippy::too_many_arguments)]
pub fn monster_fire_wand_ray(
    monster_x: i8,
    monster_y: i8,
    muse_type: i32,
    range: i32,
    player: &mut You,
    level: &mut Level,
    rng: &mut GameRng,
) -> BuzzResult {
    let Some(zap_type) = muse_wand_to_zap_type(muse_type) else {
        return BuzzResult::new();
    };

    let (dx, dy) = direction_toward(monster_x, monster_y, player.pos.x, player.pos.y);
    if dx == 0 && dy == 0 {
        return BuzzResult::new();
    }

    buzz(
        zap_type,
        ZapVariant::Wand,
        monster_x,
        monster_y,
        dx,
        dy,
        range,
        player,
        level,
        rng,
    )
}

/// Monster fires a horn ray using buzz() (C: use_offensive horn cases)
#[allow(clippy::too_many_arguments)]
pub fn monster_fire_horn_ray(
    monster_x: i8,
    monster_y: i8,
    muse_type: i32,
    range: i32,
    player: &mut You,
    level: &mut Level,
    rng: &mut GameRng,
) -> BuzzResult {
    let Some(zap_type) = muse_horn_to_zap_type(muse_type) else {
        return BuzzResult::new();
    };

    let (dx, dy) = direction_toward(monster_x, monster_y, player.pos.x, player.pos.y);
    if dx == 0 && dy == 0 {
        return BuzzResult::new();
    }

    buzz(
        zap_type,
        ZapVariant::Wand, // horns use wand damage tables
        monster_x,
        monster_y,
        dx,
        dy,
        range,
        player,
        level,
        rng,
    )
}

/// Monster fires a special beam (teleport/striking) using mbhit (C: use_offensive mbhit cases)
#[allow(clippy::too_many_arguments)]
pub fn monster_fire_special_beam(
    monster_x: i8,
    monster_y: i8,
    effect: MbhitEffect,
    range: i32,
    player: &mut You,
    level: &mut Level,
    rng: &mut GameRng,
) -> BuzzResult {
    let (dx, dy) = direction_toward(monster_x, monster_y, player.pos.x, player.pos.y);
    if dx == 0 && dy == 0 {
        return BuzzResult::new();
    }

    mbhit_effect(
        effect,
        monster_x,
        monster_y,
        dx,
        dy,
        range,
        player,
        level,
        rng,
    )
}

/// Monster uses breath weapon using buzz() (C: buzzmu)
#[allow(clippy::too_many_arguments)]
pub fn monster_use_breath_weapon(
    monster_x: i8,
    monster_y: i8,
    zap_type: ZapType,
    monster_level: u8,
    player: &mut You,
    level: &mut Level,
    rng: &mut GameRng,
) -> BuzzResult {
    let (dx, dy) = direction_toward(monster_x, monster_y, player.pos.x, player.pos.y);
    if dx == 0 && dy == 0 {
        return BuzzResult::new();
    }

    // Breath range = monster level / 2 + 6, capped at 13
    let range = ((monster_level as i32) / 2 + 6).min(13);

    buzz(
        zap_type,
        ZapVariant::Breath,
        monster_x,
        monster_y,
        dx,
        dy,
        range,
        player,
        level,
        rng,
    )
}

// ============================================================================
// Monster wand selection heuristics (muse.c scoring)
// ============================================================================

/// Wand priority for monster offensive use (C: find_offensive priority order)
///
/// Higher priority = used first. Monsters prefer the most lethal wand available.
/// Returns a priority value (higher = preferred).
pub fn wand_offensive_priority(muse_type: i32) -> i32 {
    match muse_type {
        20 => 100,  // MUSE_WAN_DEATH - highest priority
        22 => 80,   // MUSE_WAN_FIRE
        24 => 75,   // MUSE_WAN_COLD
        26 => 70,   // MUSE_WAN_LIGHTNING
        27 => 50,   // MUSE_WAN_MAGIC_MISSILE
        21 => 40,   // MUSE_WAN_SLEEP
        28 => 30,   // MUSE_WAN_STRIKING
        23 => 78,   // MUSE_FIRE_HORN
        25 => 73,   // MUSE_FROST_HORN
        _ => 0,
    }
}

/// Select best offensive wand from monster inventory (C: find_offensive wand scanning)
///
/// Scans the monster's inventory and returns the MUSE constant and inventory index
/// of the best offensive wand available. Returns None if no suitable wand found.
///
/// Priority: death > fire > cold > lightning > magic missile > sleep > striking
pub fn select_best_offensive_wand(monster: &Monster) -> Option<(i32, usize)> {
    let mut best_priority = 0;
    let mut best_muse = 0i32;
    let mut best_idx = 0;

    for (idx, obj) in monster.inventory.iter().enumerate() {
        if obj.class != crate::object::ObjectClass::Wand {
            continue;
        }
        if obj.enchantment <= 0 {
            continue;
        }

        let muse_type = match obj.object_type {
            2004 => 20, // WAN_DEATH
            2001 => 22, // WAN_FIRE
            2002 => 24, // WAN_COLD
            2005 => 26, // WAN_LIGHTNING
            2000 => 27, // WAN_MAGIC_MISSILE
            2003 => 21, // WAN_SLEEP
            _ => continue,
        };

        let priority = wand_offensive_priority(muse_type);
        if priority > best_priority {
            best_priority = priority;
            best_muse = muse_type;
            best_idx = idx;
        }
    }

    if best_priority > 0 {
        Some((best_muse, best_idx))
    } else {
        None
    }
}

/// Check if monster should zap healing wand on self (C: find_defensive healing logic)
///
/// Monster uses wand of healing on self when HP < 1/3 max HP.
/// Returns the inventory index of the healing wand if found and should use.
pub fn should_zap_healing_on_self(monster: &Monster) -> Option<usize> {
    // Only heal when below 1/3 HP
    if monster.hp >= monster.hp_max / 3 {
        return None;
    }

    for (idx, obj) in monster.inventory.iter().enumerate() {
        if obj.class != crate::object::ObjectClass::Wand {
            continue;
        }
        if obj.enchantment <= 0 {
            continue;
        }
        // Wand of healing object types
        // healing=2010, extra_healing=2011
        if obj.object_type == 2010 || obj.object_type == 2011 {
            return Some(idx);
        }
    }
    None
}

/// Apply wand healing effect to a monster
pub fn apply_wand_healing(monster: &mut Monster, wand_type: i16, rng: &mut GameRng) {
    match wand_type {
        2010 => {
            // Wand of healing: 2d8+2
            let heal = rng.dice(2, 8) as i32 + 2;
            monster.hp = (monster.hp + heal).min(monster.hp_max);
        }
        2011 => {
            // Wand of extra healing: 4d8+4
            let heal = rng.dice(4, 8) as i32 + 4;
            monster.hp = (monster.hp + heal).min(monster.hp_max);
        }
        _ => {}
    }
}

// ============================================================================
// Monster scroll usage
// ============================================================================

/// Check if monster should read teleportation scroll (C: find_defensive scroll logic)
///
/// Monster reads teleportation scroll when fleeing and low on HP.
/// Returns inventory index of the scroll if found and appropriate.
pub fn should_use_teleport_scroll(monster: &Monster) -> Option<usize> {
    // Only if fleeing or low HP
    if !monster.state.fleeing && monster.hp >= monster.hp_max / 4 {
        return None;
    }

    // Shopkeepers, guards, priests don't teleport away
    if monster.is_shopkeeper || monster.is_guard || monster.is_priest {
        return None;
    }

    for (idx, obj) in monster.inventory.iter().enumerate() {
        if obj.class != crate::object::ObjectClass::Scroll {
            continue;
        }
        // Scroll of teleportation (object type 37)
        if obj.object_type == 37 {
            return Some(idx);
        }
    }
    None
}

/// Execute monster teleportation from scroll reading
///
/// Teleports the monster to a random walkable position on the level.
pub fn execute_monster_teleport(
    monster_id: MonsterId,
    level: &mut Level,
    rng: &mut GameRng,
) -> bool {
    for _ in 0..100 {
        let nx = rng.rn2(crate::COLNO as u32) as i8;
        let ny = rng.rn2(crate::ROWNO as u32) as i8;

        if level.is_walkable(nx, ny) && level.monster_at(nx, ny).is_none() {
            if let Some(m) = level.monster_mut(monster_id) {
                m.x = nx;
                m.y = ny;
                return true;
            }
        }
    }
    false
}

// ============================================================================
// Monster potion throwing
// ============================================================================

/// Calculate direction and distance from monster to player for throwing
pub fn throw_direction(
    monster_x: i8,
    monster_y: i8,
    player_x: i8,
    player_y: i8,
) -> (i8, i8, i32) {
    let dx = (player_x - monster_x).signum();
    let dy = (player_y - monster_y).signum();
    let dist = ((player_x as i32 - monster_x as i32).abs())
        .max((player_y as i32 - monster_y as i32).abs());
    (dx, dy, dist)
}

/// Monster throws a potion at the player (C: m_throw for potions)
///
/// Simplified potion throw: traces path toward player, on hit applies
/// potion splash effect. Returns damage dealt to player (0 for status effects).
pub fn monster_throw_potion(
    monster_x: i8,
    monster_y: i8,
    potion_type: i16,
    player: &mut You,
    level: &Level,
    rng: &mut GameRng,
) -> (i32, Vec<String>) {
    let mut messages = Vec::new();
    let (dx, dy, dist) = throw_direction(monster_x, monster_y, player.pos.x, player.pos.y);

    if dx == 0 && dy == 0 {
        return (0, messages);
    }

    // Trace path toward player
    let mut x = monster_x;
    let mut y = monster_y;
    let mut hit_player = false;

    for _ in 0..dist.min(10) {
        x += dx;
        y += dy;

        if !level.is_valid_pos(x, y) {
            break;
        }

        let cell = level.cell(x as usize, y as usize);
        if cell.typ.is_wall() {
            messages.push("The potion shatters against the wall!".to_string());
            break;
        }

        if x == player.pos.x && y == player.pos.y {
            hit_player = true;
            break;
        }
    }

    if !hit_player {
        messages.push("The potion misses!".to_string());
        return (0, messages);
    }

    // Apply potion effect on hit
    apply_thrown_potion_effect(potion_type, player, rng, &mut messages)
}

/// Apply the effect of a thrown potion hitting the player
fn apply_thrown_potion_effect(
    potion_type: i16,
    player: &mut You,
    rng: &mut GameRng,
    messages: &mut Vec<String>,
) -> (i32, Vec<String>) {
    let damage = match potion_type {
        // POT_PARALYSIS (object type 108)
        108 => {
            messages.push("You can't move!".to_string());
            player.multi = -(rng.rnd(10) as i32);
            player.multi_reason = Some("paralyzed by a potion".to_string());
            0
        }
        // POT_BLINDNESS (object type 109)
        109 => {
            messages.push("It's all dark!".to_string());
            player.blinded_timeout = player.blinded_timeout.saturating_add(rng.rnd(25) as u16);
            0
        }
        // POT_CONFUSION (object type 110)
        110 => {
            messages.push("You feel confused!".to_string());
            player.confused_timeout = player.confused_timeout.saturating_add(rng.rnd(10) as u16);
            0
        }
        // POT_SLEEPING (object type 111)
        111 => {
            messages.push("You fall asleep!".to_string());
            player.sleeping_timeout = player.sleeping_timeout.saturating_add(rng.rnd(10) as u16);
            0
        }
        // POT_ACID (object type 107)
        107 => {
            let d = rng.dice(2, 6) as i32;
            if player.properties.has(crate::player::Property::AcidResistance) {
                messages.push("The acid doesn't affect you.".to_string());
                0
            } else {
                messages.push(format!("The acid burns for {} damage!", d));
                player.hp -= d;
                d
            }
        }
        _ => {
            messages.push("The potion splashes harmlessly.".to_string());
            0
        }
    };

    (damage, core::mem::take(messages))
}

// ============================================================================
// Combined monster item usage: fire wand bolt via buzz() (replaces old monster_zap_wand)
// ============================================================================

/// Monster fires a wand bolt using the full buzz() ray tracing system
///
/// This replaces the old simplified `monster_zap_wand()` that used bhit().
/// Now properly calls buzz() for ray tracing with wall bounces and resistance.
#[allow(clippy::too_many_arguments)]
pub fn monster_zap_wand(
    monster: &Monster,
    wand: &Object,
    _direction: (i8, i8),
    range: i32,
    player: &mut You,
    level: &mut Level,
    rng: &mut GameRng,
) -> ZapResult {
    let mut result = ZapResult::new();

    if let Some(zap_type) = object_to_zap_type(wand.object_type) {
        let (dx, dy) = direction_toward(monster.x, monster.y, player.pos.x, player.pos.y);
        if dx == 0 && dy == 0 {
            result.messages.push("Nothing happens.".to_string());
            return result;
        }

        let buzz_result = buzz(
            zap_type,
            ZapVariant::Wand,
            monster.x,
            monster.y,
            dx,
            dy,
            range,
            player,
            level,
            rng,
        );

        // Convert BuzzResult to ZapResult
        result.messages = buzz_result.messages;
        result.player_damage = buzz_result.player_damage;
        result.player_died = buzz_result.player_died;
        result.killed = buzz_result.killed;
    } else {
        result.messages.push("Nothing happens.".to_string());
    }

    result
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::ObjectClass;

    fn make_wand(object_type: i16, charges: i8) -> Object {
        let mut obj = Object::default();
        obj.class = ObjectClass::Wand;
        obj.object_type = object_type;
        obj.enchantment = charges;
        obj
    }

    fn make_monster_with_wands() -> Monster {
        let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
        m.name = "orc".to_string();
        m.hp = 20;
        m.hp_max = 20;
        // Give it several wands with different priorities
        m.inventory.push(make_wand(2000, 5)); // MM wand (priority 50)
        m.inventory.push(make_wand(2001, 3)); // Fire wand (priority 80)
        m.inventory.push(make_wand(2004, 1)); // Death wand (priority 100)
        m
    }

    #[test]
    fn test_select_best_wand_prefers_death() {
        let m = make_monster_with_wands();
        let result = select_best_offensive_wand(&m);
        assert!(result.is_some());
        let (muse_type, idx) = result.unwrap();
        assert_eq!(muse_type, 20); // MUSE_WAN_DEATH
        assert_eq!(idx, 2); // Third item in inventory
    }

    #[test]
    fn test_select_best_wand_skips_empty() {
        let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
        m.inventory.push(make_wand(2004, 0)); // Death wand, no charges
        m.inventory.push(make_wand(2001, 3)); // Fire wand, has charges
        let result = select_best_offensive_wand(&m);
        assert!(result.is_some());
        let (muse_type, _) = result.unwrap();
        assert_eq!(muse_type, 22); // MUSE_WAN_FIRE (death has no charges)
    }

    #[test]
    fn test_select_best_wand_none_available() {
        let m = Monster::new(MonsterId::NONE, 0, 5, 5);
        assert!(select_best_offensive_wand(&m).is_none());
    }

    #[test]
    fn test_should_zap_healing_low_hp() {
        let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
        m.hp = 3;
        m.hp_max = 20;
        m.inventory.push(make_wand(2010, 3)); // Healing wand
        assert!(should_zap_healing_on_self(&m).is_some());
    }

    #[test]
    fn test_should_not_heal_when_healthy() {
        let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
        m.hp = 18;
        m.hp_max = 20;
        m.inventory.push(make_wand(2010, 3));
        assert!(should_zap_healing_on_self(&m).is_none());
    }

    #[test]
    fn test_object_to_zap_type_mapping() {
        assert_eq!(object_to_zap_type(2000), Some(ZapType::MagicMissile));
        assert_eq!(object_to_zap_type(2001), Some(ZapType::Fire));
        assert_eq!(object_to_zap_type(2004), Some(ZapType::Death));
        assert_eq!(object_to_zap_type(9999), None);
    }

    #[test]
    fn test_wand_priority_ordering() {
        assert!(wand_offensive_priority(20) > wand_offensive_priority(22)); // death > fire
        assert!(wand_offensive_priority(22) > wand_offensive_priority(27)); // fire > MM
        assert!(wand_offensive_priority(27) > wand_offensive_priority(21)); // MM > sleep
    }

    #[test]
    fn test_throw_direction_calculation() {
        let (dx, dy, dist) = throw_direction(5, 5, 10, 5);
        assert_eq!(dx, 1);
        assert_eq!(dy, 0);
        assert_eq!(dist, 5);

        let (dx, dy, dist) = throw_direction(5, 5, 5, 2);
        assert_eq!(dx, 0);
        assert_eq!(dy, -1);
        assert_eq!(dist, 3);
    }

    #[test]
    fn test_mzapwand_consumes_charge() {
        let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
        let mut wand = make_wand(2001, 3);
        let mut rng = GameRng::new(42);
        let result = mzapwand(&mut m, &mut wand, &mut rng);
        assert!(result);
        assert!(wand.enchantment < 3);
    }

    #[test]
    fn test_mzapwand_empty_fails() {
        let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
        let mut wand = make_wand(2001, 0);
        let mut rng = GameRng::new(42);
        assert!(!mzapwand(&mut m, &mut wand, &mut rng));
    }

    #[test]
    fn test_should_use_teleport_scroll_when_fleeing() {
        let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
        m.hp = 3;
        m.hp_max = 20;
        m.state.fleeing = true;
        let mut scroll = Object::default();
        scroll.class = ObjectClass::Scroll;
        scroll.object_type = 37;
        m.inventory.push(scroll);
        assert!(should_use_teleport_scroll(&m).is_some());
    }

    #[test]
    fn test_shopkeeper_wont_teleport() {
        let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
        m.hp = 3;
        m.hp_max = 20;
        m.state.fleeing = true;
        m.is_shopkeeper = true;
        let mut scroll = Object::default();
        scroll.class = ObjectClass::Scroll;
        scroll.object_type = 37;
        m.inventory.push(scroll);
        assert!(should_use_teleport_scroll(&m).is_none());
    }

    #[test]
    fn test_apply_wand_healing() {
        let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
        m.hp = 5;
        m.hp_max = 30;
        let mut rng = GameRng::new(42);
        apply_wand_healing(&mut m, 2010, &mut rng);
        assert!(m.hp > 5);
        assert!(m.hp <= 30);
    }
}
