//! Zapping wands (zap.c)

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::action::{ActionResult, Direction};
use crate::gameloop::GameState;
use crate::magic::zap::{ZapDirection, zap_wand};
use crate::object::{Object, ObjectClass};

/// Zap a wand from inventory
pub fn do_zap(
    state: &mut GameState,
    obj_letter: char,
    direction: Option<Direction>,
) -> ActionResult {
    // Find the wand index in inventory
    let wand_idx = match state
        .inventory
        .iter()
        .position(|o| o.inv_letter == obj_letter)
    {
        Some(idx) => idx,
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    if state.inventory[wand_idx].class != ObjectClass::Wand {
        return ActionResult::Failed("That's not something you can zap.".to_string());
    }

    // Convert Direction to ZapDirection using delta
    let dir = direction.unwrap_or(Direction::East);
    let zap_dir = match dir {
        Direction::Up => ZapDirection::Up,
        Direction::Down => ZapDirection::Down,
        Direction::Self_ => ZapDirection::Self_,
        _ => {
            let (dx, dy) = dir.delta();
            ZapDirection::Direction(dx, dy)
        }
    };

    // Apply zap effects - need to temporarily remove wand to satisfy borrow checker
    let mut wand = state.inventory.remove(wand_idx);

    let result = zap_wand(
        &mut wand,
        zap_dir,
        &mut state.player,
        &mut state.current_level,
        &mut state.rng,
    );

    // Put wand back in inventory
    state.inventory.insert(wand_idx, wand);

    // Display messages
    for msg in result.messages {
        state.message(msg);
    }

    // Remove killed monsters
    for monster_id in result.killed {
        state.current_level.remove_monster(monster_id);
    }

    ActionResult::Success
}

pub fn dozap(
    state: &mut GameState,
    obj_letter: char,
    direction: Option<Direction>,
) -> ActionResult {
    do_zap(state, obj_letter, direction)
}

pub fn zapsetup() {
    // Stub
}

pub fn zapwrapup() {
    // Stub
}

pub fn zapyourself(state: &mut GameState, obj: &Object) {
    // Apply wand effect to self
    state.message("You zap yourself!");
    // Stub: need to call appropriate effect
}

pub fn zap_hit(state: &mut GameState, x: i8, y: i8) -> i32 {
    // Stub
    0
}

pub fn zap_over_floor(state: &mut GameState, x: i8, y: i8, type_: i32, shop_check: bool) {
    // Stub
}

pub fn zap_updown(state: &mut GameState, obj: &Object) -> bool {
    // Zap up/down logic
    false
}

pub fn zap_dig(state: &mut GameState) {
    // Digging zap logic
}

pub fn zap_steed(state: &mut GameState, obj: &Object) -> bool {
    false
}

/// Beam/ray type constants
pub mod beam_type {
    pub const AD_MAGM: i32 = 0; // Magic missile
    pub const AD_FIRE: i32 = 1; // Fire
    pub const AD_COLD: i32 = 2; // Cold
    pub const AD_SLEE: i32 = 3; // Sleep
    pub const AD_DISN: i32 = 4; // Disintegration
    pub const AD_ELEC: i32 = 5; // Lightning
    pub const AD_DRST: i32 = 6; // Drain strength
    pub const AD_ACID: i32 = 7; // Acid
    pub const AD_BLND: i32 = 8; // Blindness
    pub const AD_STUN: i32 = 9; // Stun
    pub const AD_SLOW: i32 = 10; // Slow
    pub const AD_PLYS: i32 = 11; // Paralysis
    pub const AD_DRLI: i32 = 12; // Drain life
    pub const AD_DETH: i32 = 13; // Death
    pub const AD_PEST: i32 = 14; // Pestilence
    pub const AD_FAMN: i32 = 15; // Famine
}

/// Project a beam/ray in a direction
pub fn buzz(state: &mut GameState, type_: i32, distance: i32, dx: i8, dy: i8) {
    let start_x = state.player.pos.x;
    let start_y = state.player.pos.y;

    dobuzz(state, type_, distance, start_x, start_y, dx, dy);
}

/// Monster zaps player with a beam
pub fn buzzmu(state: &mut GameState, monster_id: u32, obj: &Object) {
    use crate::monster::MonsterId;

    // Get monster position
    let (start_x, start_y, monster_name) = {
        if let Some(mon) = state.current_level.monster(MonsterId(monster_id)) {
            (mon.x, mon.y, mon.name.clone())
        } else {
            return;
        }
    };

    // Calculate direction toward player
    let dx = (state.player.pos.x as i32 - start_x as i32).signum() as i8;
    let dy = (state.player.pos.y as i32 - start_y as i32).signum() as i8;

    state.message(format!("The {} zaps at you!", monster_name));

    // Determine beam type from wand/object
    let beam_type = wand_to_beam_type(obj.object_type);
    let distance = 8;

    dobuzz(state, beam_type, distance, start_x, start_y, dx, dy);
}

/// Helper to convert wand type to beam type
fn wand_to_beam_type(wand_type: i16) -> i32 {
    // Simplified mapping - in full implementation would use wand definitions
    match wand_type {
        100..=105 => beam_type::AD_FIRE,
        106..=110 => beam_type::AD_COLD,
        111..=115 => beam_type::AD_ELEC,
        116..=120 => beam_type::AD_SLEE,
        121..=125 => beam_type::AD_DETH,
        126..=130 => beam_type::AD_MAGM,
        _ => beam_type::AD_MAGM,
    }
}

/// Internal buzz implementation with starting position
pub fn dobuzz(
    state: &mut GameState,
    type_: i32,
    distance: i32,
    start_x: i8,
    start_y: i8,
    dx: i8,
    dy: i8,
) {
    let beam_name = beam_type_name(type_);
    let mut x = start_x;
    let mut y = start_y;

    // Trace the beam
    for _ in 0..distance {
        x += dx;
        y += dy;

        if !state.current_level.is_valid_pos(x, y) {
            break;
        }

        // Check for player
        if x == state.player.pos.x && y == state.player.pos.y {
            let damage = beam_base_damage(type_, &mut state.rng);
            zhitu(state, type_, damage, &beam_name, x, y);
            break; // Beam stops at player
        }

        // Check for monster
        if let Some(monster) = state.current_level.monster_at(x, y) {
            let monster_id = monster.id;
            let damage = beam_base_damage(type_, &mut state.rng);
            let killed = zhitm(state, monster_id.0, type_, damage);
            if killed > 0 {
                state.current_level.remove_monster(monster_id);
            }
            break; // Beam stops at monster
        }

        // Check for walls
        if !state.current_level.is_walkable(x, y) {
            state.message(format!("The {} hits the wall.", beam_name));
            break;
        }
    }
}

/// Calculate base damage for a beam type
fn beam_base_damage(type_: i32, rng: &mut crate::rng::GameRng) -> i32 {
    use beam_type::*;
    match type_ {
        AD_MAGM => rng.dice(2, 6) as i32,
        AD_FIRE => rng.dice(6, 6) as i32,
        AD_COLD => rng.dice(6, 6) as i32,
        AD_ELEC => rng.dice(6, 6) as i32,
        AD_ACID => rng.dice(4, 6) as i32,
        AD_DRLI => rng.dice(2, 8) as i32,
        AD_DETH => 127, // Instant death (if not resistant)
        _ => rng.dice(2, 6) as i32,
    }
}

/// Get display name for a beam type
fn beam_type_name(type_: i32) -> String {
    use beam_type::*;
    match type_ {
        AD_MAGM => "magic missile".to_string(),
        AD_FIRE => "bolt of fire".to_string(),
        AD_COLD => "bolt of cold".to_string(),
        AD_SLEE => "sleep ray".to_string(),
        AD_DISN => "disintegration beam".to_string(),
        AD_ELEC => "bolt of lightning".to_string(),
        AD_DRST => "weakening ray".to_string(),
        AD_ACID => "splash of acid".to_string(),
        AD_BLND => "blinding light".to_string(),
        AD_STUN => "stunning ray".to_string(),
        AD_SLOW => "slowing ray".to_string(),
        AD_PLYS => "paralyzing ray".to_string(),
        AD_DRLI => "life drain".to_string(),
        AD_DETH => "death ray".to_string(),
        AD_PEST => "plague".to_string(),
        AD_FAMN => "famine".to_string(),
        _ => "ray".to_string(),
    }
}

/// Hit a monster with a wand effect
pub fn bhitm(state: &mut GameState, monster_id: u32, obj: &Object) -> i32 {
    let beam_type = wand_to_beam_type(obj.object_type);
    let damage = beam_base_damage(beam_type, &mut state.rng);
    zhitm(state, monster_id, beam_type, damage)
}

/// Hit an object with a wand effect
pub fn bhito(state: &mut GameState, obj: &mut Object, wand: &Object) -> i32 {
    // Transform object based on wand type
    let beam_type = wand_to_beam_type(wand.object_type);

    match beam_type {
        beam_type::AD_FIRE => {
            // Fire can burn scrolls, wooden items
            if obj.class == ObjectClass::Scroll {
                state.message("The scroll burns!");
                return 1; // Object destroyed
            }
        }
        beam_type::AD_COLD => {
            // Cold can freeze potions
            if obj.class == ObjectClass::Potion {
                state.message("The potion freezes and shatters!");
                return 1; // Object destroyed
            }
        }
        _ => {}
    }
    0
}

/// Hit a pile of objects at a position
pub fn bhitpile(state: &mut GameState, wand: &Object, x: i8, y: i8) -> i32 {
    let mut destroyed = 0;
    // In full implementation, would iterate through objects at position
    // and apply bhito to each one
    destroyed
}

/// Monster beam hits monster
pub fn mbhit(state: &mut GameState, attacker_id: u32, target_id: u32, obj: &Object) -> i32 {
    bhitm(state, target_id, obj)
}

/// Monster beam hits monster (simplified)
pub fn mbhitm(state: &mut GameState, monster_id: u32, obj: &Object) -> i32 {
    bhitm(state, monster_id, obj)
}

/// Generic hit monster
pub fn ghitm(state: &mut GameState, monster_id: u32, obj: &Object) -> i32 {
    bhitm(state, monster_id, obj)
}

/// Zap effect hits a monster
/// Returns 1 if monster was killed, 0 otherwise
pub fn zhitm(state: &mut GameState, monster_id: u32, type_: i32, damage: i32) -> i32 {
    use crate::monster::{MonsterId, MonsterResistances};
    use beam_type::*;

    // Get monster info
    let (monster_name, has_resistance) = {
        if let Some(mon) = state.current_level.monster(MonsterId(monster_id)) {
            let resist = match type_ {
                AD_FIRE => mon.resistances.contains(MonsterResistances::FIRE),
                AD_COLD => mon.resistances.contains(MonsterResistances::COLD),
                AD_ELEC => mon.resistances.contains(MonsterResistances::ELEC),
                AD_SLEE => mon.resistances.contains(MonsterResistances::SLEEP),
                AD_DETH => false, // Death magic can't be resisted by standard resistances
                _ => false,
            };
            (mon.name.clone(), resist)
        } else {
            return 0;
        }
    };

    // Check resistance
    if has_resistance {
        state.message(format!("The {} resists!", monster_name));
        return 0;
    }

    // Apply damage or effect
    match type_ {
        AD_SLEE => {
            state.message(format!("The {} falls asleep!", monster_name));
            // Would set monster sleep counter here
            return 0;
        }
        AD_BLND => {
            state.message(format!("The {} is blinded!", monster_name));
            // Would set monster blind counter here
            return 0;
        }
        AD_STUN => {
            state.message(format!("The {} staggers!", monster_name));
            // Would set monster stun counter here
            return 0;
        }
        AD_SLOW => {
            state.message(format!("The {} slows down!", monster_name));
            // Would set monster slow flag here
            return 0;
        }
        AD_PLYS => {
            state.message(format!("The {} is paralyzed!", monster_name));
            // Would set monster paralysis counter here
            return 0;
        }
        AD_DETH => {
            state.message(format!("The {} is killed!", monster_name));
            return 1;
        }
        _ => {
            // Damage-dealing beam
            let killed = if let Some(mon) = state
                .current_level
                .monster_mut(crate::monster::MonsterId(monster_id))
            {
                mon.hp -= damage;
                mon.hp <= 0
            } else {
                false
            };
            state.message(format!("The {} is hit by the beam!", monster_name));
            if killed {
                state.message(format!("The {} is killed!", monster_name));
                return 1;
            }
        }
    }

    0
}

/// Zap effect hits the player
pub fn zhitu(state: &mut GameState, type_: i32, damage: i32, beam_name: &str, _x: i8, _y: i8) {
    use crate::player::{Attribute, Property};
    use beam_type::*;

    state.message(format!("The {} hits you!", beam_name));

    // Check resistance
    let resisted = match type_ {
        AD_FIRE => state.player.properties.has(Property::FireResistance),
        AD_COLD => state.player.properties.has(Property::ColdResistance),
        AD_ELEC => state.player.properties.has(Property::ShockResistance),
        AD_SLEE => state.player.properties.has(Property::SleepResistance),
        AD_DISN => state.player.properties.has(Property::DisintResistance),
        AD_DETH => state.player.properties.has(Property::MagicResistance),
        AD_ACID => state.player.properties.has(Property::AcidResistance),
        _ => false,
    };

    if resisted {
        state.message("You resist!");
        return;
    }

    // Apply effect
    match type_ {
        AD_SLEE => {
            state.message("You fall asleep!");
            state.player.sleeping_timeout = 30;
        }
        AD_BLND => {
            state.message("You are blinded!");
            state.player.blinded_timeout = 50;
        }
        AD_STUN => {
            state.message("You stagger...");
            state.player.stunned_timeout = 20;
        }
        AD_SLOW => {
            state.message("You slow down...");
            // Remove Speed property if present
            state.player.properties.remove_intrinsic(Property::Speed);
        }
        AD_PLYS => {
            state.message("You can't move!");
            state.player.stunned_timeout = 40; // Use stun as proxy
        }
        AD_DRLI => {
            state.message("You feel weaker...");
            state.player.take_damage(damage);
            state.player.losexp(true);
        }
        AD_DETH => {
            state.message("You die...");
            state.player.hp = 0;
        }
        AD_DRST => {
            state.message("You feel weak!");
            let current_str = state.player.attr_current.get(Attribute::Strength);
            state
                .player
                .attr_current
                .set(Attribute::Strength, current_str.saturating_sub(1));
        }
        _ => {
            // Damage-dealing beam
            state.player.take_damage(damage);
        }
    }
}

pub fn recharge(state: &mut GameState, obj: &mut Object, val: i32) {
    state.message("You recharge the item.");
    // Stub
}

pub fn is_chargeable(obj: &Object) -> bool {
    obj.class == ObjectClass::Wand
}

pub fn drain_item(state: &mut GameState, obj: &mut Object) {
    // Drain charges
}

pub fn weffects(state: &mut GameState, obj: &Object) {
    // Wand effects
}

pub fn boomhit(state: &mut GameState, x: i8, y: i8) {
    // Boom effect
}
