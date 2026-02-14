//! Trap mechanics (trap.c)
//!
//! This module provides player actions for trap interaction:
//! - Triggering traps when stepping on them
//! - Setting traps (future)
//! - Searching for traps
//! - Disarming traps

use crate::action::ActionResult;
use crate::dungeon::TrapType;
use crate::dungeon::trap::{
    self, is_holding_trap, roll_trap_damage, trap_name,
};
use crate::dungeon::{CellType, Trap};
use crate::gameloop::GameState;
use crate::monster::{Monster, MonsterResistances};
use crate::object::Object;
use crate::player::PlayerTrapType;
use crate::rng::GameRng;

/// Convert a dungeon TrapType to the player's TrapType for utrap tracking.
pub fn to_player_trap_type(tt: TrapType) -> PlayerTrapType {
    match tt {
        TrapType::BearTrap => PlayerTrapType::BearTrap,
        TrapType::Pit => PlayerTrapType::Pit,
        TrapType::SpikedPit => PlayerTrapType::SpikedPit,
        TrapType::Web => PlayerTrapType::Web,
        _ => PlayerTrapType::InFloor, // fallback for other holding traps
    }
}

/// Check for and trigger traps at a position
pub fn check_trap(state: &mut GameState, x: i8, y: i8) -> ActionResult {
    let trap_type = match state.current_level.trap_at(x, y) {
        Some(t) => t.trap_type,
        None => return ActionResult::NoTime,
    };

    // Build player resistances for dotrap
    let dex = state.player.attr_current.get(crate::player::Attribute::Dexterity);
    let resistances = trap::resistances_from_properties(
        |prop| state.player.properties.has(prop),
        dex,
    );

    // Trigger the trap
    if let Some(trap) = state.current_level.trap_at_mut(x, y) {
        let result = trap::dotrap(&mut state.rng, trap, &resistances, false);

        for msg in &result.messages {
            state.message(msg.clone());
        }

        if result.damage > 0 {
            state.player.take_damage(result.damage);
        }

        if result.held_turns > 0 {
            state.player.utrap = result.held_turns as u32;
            state.player.utrap_type = to_player_trap_type(trap_type);
        }

        // Apply status effect if any
        if let Some(status) = result.status {
            use crate::dungeon::trap::StatusEffect;
            match status {
                StatusEffect::Poisoned => {
                    state.message("You are poisoned!");
                }
                StatusEffect::Asleep => {
                    state.player.sleeping_timeout = result.held_turns as u16;
                }
                StatusEffect::Confused => {
                    state.player.confused_timeout = result.held_turns as u16;
                }
                StatusEffect::Blind => {
                    state.player.blinded_timeout = result.held_turns as u16;
                }
                StatusEffect::Stunned => {
                    state.player.stunned_timeout =
                        (result.held_turns as u16).max(state.player.stunned_timeout);
                }
                StatusEffect::Paralyzed => {
                    state.player.stunned_timeout =
                        (result.held_turns as u16).max(state.player.stunned_timeout);
                }
                StatusEffect::Rusted => {
                    state.message("Your equipment rusts!");
                    // TODO: apply rust damage to worn armor
                }
            }
        }

        if result.trap_destroyed {
            state.current_level.remove_trap(x, y);
        }
    }

    if state.player.hp <= 0 {
        ActionResult::Died("killed by a trap".to_string())
    } else {
        ActionResult::Success
    }
}

/// Trigger a specific trap type on the player (convenience wrapper for tests/gameloop)
pub fn trigger_trap(state: &mut GameState, trap_type: TrapType) -> ActionResult {
    let mut temp_trap = crate::dungeon::trap::create_trap(
        state.player.pos.x,
        state.player.pos.y,
        trap_type,
    );
    let effect = crate::dungeon::trap::trigger_trap(&mut state.rng, &mut temp_trap);

    match effect {
        crate::dungeon::trap::TrapEffect::Damage(d) => {
            state.player.take_damage(d);
            if state.player.hp <= 0 {
                ActionResult::Died("killed by a trap".to_string())
            } else {
                ActionResult::Success
            }
        }
        crate::dungeon::trap::TrapEffect::Status(s) => {
            state.message(format!("You are affected by {:?}!", s));
            ActionResult::Success
        }
        crate::dungeon::trap::TrapEffect::Trapped { turns } => {
            state.player.utrap = turns as u32;
            state.player.utrap_type = to_player_trap_type(trap_type);
            ActionResult::Success
        }
        crate::dungeon::trap::TrapEffect::Teleport { x, y } => {
            state.player.pos.x = x;
            state.player.pos.y = y;
            ActionResult::Success
        }
        crate::dungeon::trap::TrapEffect::Fall { damage, .. } => {
            state.player.take_damage(damage);
            if state.player.hp <= 0 {
                ActionResult::Died("killed by a fall".to_string())
            } else {
                ActionResult::Success
            }
        }
        _ => ActionResult::Success,
    }
}

/// Search for traps and secret doors
pub fn do_search(state: &mut GameState) -> ActionResult {
    let px = state.player.pos.x;
    let py = state.player.pos.y;

    // Check adjacent cells for hidden traps
    for dx in -1..=1i8 {
        for dy in -1..=1i8 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let tx = px + dx;
            let ty = py + dy;
            if let Some(trap) = state.current_level.trap_at_mut(tx, ty)
                && !trap.seen
            {
                // Search skill check
                let search_bonus = state.player.exp_level;
                if state.rng.rn2(20) < (search_bonus as u32).min(18) {
                    trap.seen = true;
                    let name = trap_name(trap.trap_type);
                    state.message(format!("You find a {}.", name));
                }
            }
        }
    }

    state.message("You search for traps and secret doors.");
    ActionResult::Success
}

// ─────────────────────────────────────────────────────────────────────────────
// Gitea stubs for trap-related functions
// ─────────────────────────────────────────────────────────────────────────────

pub fn dotrap_action(state: &mut GameState, trap: &mut Trap) {
    check_trap(state, trap.x, trap.y);
}

pub fn dountrap(state: &mut GameState, trap: &mut Trap) -> ActionResult {
    do_disarm(state, trap.x, trap.y)
}

pub fn doidtrap(state: &mut GameState) {
    state.message("You identify the trap.");
}

pub fn find_trap(_state: &mut GameState, trap: &mut Trap) {
    trap.seen = true;
}

pub fn t_at(state: &GameState, x: i8, y: i8) -> Option<&Trap> {
    state.current_level.trap_at(x, y)
}

pub fn maketrap(state: &mut GameState, x: i8, y: i8, trap_type: TrapType) {
    let trap = Trap {
        x,
        y,
        trap_type,
        activated: false,
        seen: false,
        once: false,
        madeby_u: false,
        launch_oid: None,
    };
    state.current_level.traps.push(trap);
}

pub fn mktrap(_state: &mut GameState, _trap_type: i32, _count: i32, _flags: i32) {
    // Make trap logic
}

pub fn deltrap(state: &mut GameState, trap: &Trap) {
    if let Some(idx) = state
        .current_level
        .traps
        .iter()
        .position(|t| t.x == trap.x && t.y == trap.y)
    {
        state.current_level.traps.remove(idx);
    }
}

pub fn delfloortrap(state: &mut GameState, x: i8, y: i8) {
    if let Some(idx) = state
        .current_level
        .traps
        .iter()
        .position(|t| t.x == x && t.y == y)
    {
        state.current_level.traps.remove(idx);
    }
}

pub fn steedintrap(_state: &mut GameState, _trap: &Trap) -> bool {
    false
}

pub fn move_into_trap(state: &mut GameState, trap: &Trap) -> bool {
    check_trap(state, trap.x, trap.y);
    true
}

pub fn sense_trap(_state: &mut GameState, _trap: &Trap, _x: i8, _y: i8, _src_x: i8, _src_y: i8) -> bool {
    false
}

pub fn set_trap(state: &mut GameState, _obj: &Object, _x: i8, _y: i8) {
    state.message("You set a trap.");
}

/// Effects of being at a spot (pool, lava, trap, etc.)
pub fn spoteffects(state: &mut GameState, x: i8, y: i8) {
    let cell = state.current_level.cell(x as usize, y as usize);

    match cell.typ {
        CellType::Pool | CellType::Moat => {
            pooleffects(state, false);
        }
        CellType::Lava => {
            lava_effects(state);
        }
        CellType::Sink => {
            state.message("You hear a gurgling noise.");
        }
        CellType::Fountain => {
            if state.player.confused_timeout > 0 && state.rng.one_in(10) {
                state.message("Oops! You drank from the fountain.");
                crate::action::quaff::drinkfountain(state);
            }
        }
        _ => {}
    }

    if let Some(_trap) = state.current_level.trap_at(x, y) {
        check_trap(state, x, y);
    }
}

/// Effects of stepping into pool/water
pub fn pooleffects(state: &mut GameState, by_magic: bool) {
    use crate::player::Property;

    if state.player.properties.has(Property::Swimming) {
        state.message("You swim through the water.");
        return;
    }
    if state.player.properties.has(Property::Levitation) {
        state.message("You float above the water.");
        return;
    }
    if state.player.properties.has(Property::Flying) {
        state.message("You fly over the water.");
        return;
    }

    state.message("You fall into the water!");

    if state.player.properties.has(Property::MagicBreathing) {
        state.message("But you can breathe underwater!");
        return;
    }

    let str_val = state.player.attr_current.get(crate::player::Attribute::Strength) as i32;
    let swim_chance = 20 + str_val;
    let roll = state.rng.rnd(100) as i32;

    if roll <= swim_chance {
        state.message("You struggle to the shore.");
    } else if by_magic {
        state.message("You are sinking!");
    } else {
        drown(state);
    }
}

/// Effects of stepping into lava
pub fn lava_effects(state: &mut GameState) {
    use crate::player::Property;

    if state.player.properties.has(Property::Levitation) {
        state.message("You float above the lava.");
        return;
    }
    if state.player.properties.has(Property::Flying) {
        state.message("You fly over the lava.");
        return;
    }

    if state.player.properties.has(Property::FireResistance) {
        state.message("The lava feels warm!");
        lava_damage(state);
    } else {
        state.message("You fall into the lava!");
        sink_into_lava(state);
    }
}

pub fn lava_damage(state: &mut GameState) {
    use crate::player::Property;
    let base_damage = state.rng.dice(2, 10) as i32;
    let damage = if state.player.properties.has(Property::FireResistance) {
        base_damage / 4
    } else {
        base_damage
    };
    state.message(format!("The lava burns you for {} damage!", damage));
    state.player.take_damage(damage);
}

pub fn sink_into_lava(state: &mut GameState) {
    use crate::player::Property;
    if state.player.properties.has(Property::FireResistance) {
        state.message("You sink into the lava but your fire resistance protects you!");
        lava_damage(state);
        state.message("You manage to crawl out of the lava.");
    } else {
        state.message("You sink into the lava and are incinerated!");
        state.player.hp = 0;
    }
}

pub fn flooreffects(state: &mut GameState, x: i8, y: i8, touch: bool) {
    let cell = state.current_level.cell(x as usize, y as usize);
    if !touch { return; }
    match cell.typ {
        CellType::Lava => { state.message("The floor is extremely hot!"); }
        CellType::Ice => {
            state.message("The floor is slippery!");
            if state.rng.one_in(10) {
                state.message("You slip and fall!");
                state.player.stunned_timeout = 2;
            }
        }
        _ => {}
    }
}

pub fn trapmove(_state: &mut GameState, _x: i8, _y: i8, _dest_x: i8, _dest_y: i8) {}
pub fn trapnote(_state: &mut GameState, _trap: &Trap, _boolean: bool) {}

pub fn trapped_chest_at(_state: &GameState, _trap_type: i32, _x: i8, _y: i8) -> bool { false }

pub fn trapped_door_at(state: &GameState, x: i8, y: i8) -> bool {
    if let Some(cell) = state.current_level.cells.get(x as usize).and_then(|col| col.get(y as usize)) {
        cell.typ == CellType::Door && cell.door_state().contains(crate::dungeon::DoorState::TRAPPED)
    } else {
        false
    }
}

pub fn chest_trap(_state: &mut GameState, _chest: &mut Object, _force: i32, _web: bool) {}
pub fn dofiretrap(_state: &mut GameState, _box_obj: &mut Object) {}
pub fn domagictrap(_state: &mut GameState, _box_obj: &mut Object) {}
pub fn domagicportal(_state: &mut GameState, _trap: &mut Trap) {}
pub fn fall_through(_state: &mut GameState, _trap_door: bool, _dropped_objects: u32) {}

pub fn fall_asleep(state: &mut GameState, how_long: i32, _wakeup_msg: bool) {
    state.message("You fall asleep.");
    state.player.sleeping_timeout = how_long as u16;
}

pub fn drown(state: &mut GameState) {
    state.message("You drown.");
    state.player.hp = 0;
}

pub fn climb_pit(state: &mut GameState) { state.message("You climb out of the pit."); }
pub fn fill_pit(state: &mut GameState, _x: i8, _y: i8) { state.message("You fill the pit."); }
pub fn pit_flow(_state: &mut GameState, _trap: &mut Trap, _dist: i32) {}
pub fn conjoined_pits(_state: &mut GameState, _trap: &mut Trap, _trap2: &mut Trap, _boolean: bool) {}
pub fn adj_nonconjoined_pit(_state: &mut GameState, _trap: &mut Trap) -> bool { false }
pub fn adj_pit_checks(_state: &mut GameState, _trap: &mut Trap, _trap2: &mut Trap) -> bool { false }
pub fn clear_conjoined_pits(_state: &mut GameState, _trap: &mut Trap) {}
pub fn join_adjacent_pits(_state: &mut GameState, _trap: &mut Trap, _boolean: bool) {}

pub fn reward_untrap(_state: &mut GameState, _trap: &mut Trap, _monster: &mut crate::monster::Monster) {}
pub fn untrap_prob(_state: &mut GameState, _trap: &mut Trap) -> i32 { 50 }

pub fn cnv_trap_obj(_state: &mut GameState, _otyp: i32, _count: i32, _trap: &mut Trap, _boolean: bool) -> bool { false }
pub fn holetime() -> i32 { 0 }
pub fn t_warn(_state: &mut GameState, _trap: &mut Trap) {}
pub fn t_missile(_state: &mut GameState, _trap_type: i32, _trap: &mut Trap) {}

/// Launch style constants
pub const ROLL: i32 = 0x01;
pub const FUSE: i32 = 0x02;
pub const NEED_PICK: i32 = 0x04;

/// Launch an object from one position toward another
pub fn launch_obj(state: &mut GameState, obj: &mut Object, x: i8, y: i8, x2: i8, y2: i8, style: i32) {
    let obj_name = obj.display_name();
    let dx = (x2 - x).signum();
    let dy = (y2 - y).signum();
    let range = if style & ROLL != 0 { 20 } else { 10 };
    let mut cur_x = x;
    let mut cur_y = y;

    for _ in 0..range {
        cur_x += dx;
        cur_y += dy;
        if !state.current_level.is_valid_pos(cur_x, cur_y) { break; }
        if cur_x == state.player.pos.x && cur_y == state.player.pos.y {
            state.message(format!("The {} hits you!", obj_name));
            let damage = if style & ROLL != 0 { state.rng.dice(2, 10) as i32 + 10 } else { state.rng.dice(1, 6) as i32 };
            state.player.take_damage(damage);
            launch_drop_spot(state, obj, cur_x, cur_y);
            return;
        }
        if let Some(monster) = state.current_level.monster_at(cur_x, cur_y) {
            let monster_id = monster.id;
            let monster_name = monster.name.clone();
            state.message(format!("The {} hits the {}!", obj_name, monster_name));
            let damage = if style & ROLL != 0 { state.rng.dice(2, 10) as i32 + 10 } else { state.rng.dice(1, 6) as i32 };
            if let Some(mon) = state.current_level.monster_mut(monster_id) {
                mon.hp -= damage;
                if mon.hp <= 0 {
                    state.message(format!("The {} is killed!", monster_name));
                    state.current_level.remove_monster(monster_id);
                }
            }
            launch_drop_spot(state, obj, cur_x, cur_y);
            return;
        }
        if !state.current_level.is_walkable(cur_x, cur_y) {
            state.message(format!("The {} crashes into a wall!", obj_name));
            launch_drop_spot(state, obj, cur_x - dx, cur_y - dy);
            return;
        }
    }
    launch_drop_spot(state, obj, cur_x, cur_y);
}

pub fn launch_drop_spot(state: &mut GameState, obj: &mut Object, x: i8, y: i8) {
    let mut dropped = obj.clone();
    dropped.x = x;
    dropped.y = y;
    state.current_level.add_object(dropped, x, y);
}

pub fn launch_in_progress() -> bool { false }
pub fn force_launch_placement() {}

pub fn mkroll_launch(state: &mut GameState, trap: &mut Trap, x: i8, y: i8, toward_player: bool, _unused: i64) {
    let mut boulder = Object::new(crate::object::ObjectId(state.rng.rn2(10000)), 0, crate::object::ObjectClass::Rock);
    boulder.weight = 6000;
    let (dx, dy) = if toward_player {
        ((state.player.pos.x - x).signum(), (state.player.pos.y - y).signum())
    } else {
        let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        dirs[state.rng.rn2(4) as usize]
    };
    let target_x = x + dx * 20;
    let target_y = y + dy * 20;
    state.message("Click! You hear a rumbling sound.");
    launch_obj(state, &mut boulder, x, y, target_x, target_y, ROLL);
    trap.activated = true;
}

// ============================================================================
// mintrap — monster triggers a trap (trap.c)
// ============================================================================

/// Result of a monster stepping into a trap
#[derive(Debug, Clone, Default)]
pub struct MintrapResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Damage dealt to the monster
    pub damage: i32,
    /// Monster is held in trap (turns)
    pub held_turns: i32,
    /// Monster was teleported to new position
    pub teleport: Option<(i8, i8)>,
    /// Monster fell through to a lower level
    pub fell_through: bool,
    /// Trap should be destroyed
    pub trap_destroyed: bool,
    /// Trap was avoided entirely
    pub avoided: bool,
}

/// Monster triggers a trap (mintrap from trap.c).
///
/// Core logic for each trap type vs a monster:
/// - Flying/levitating monsters avoid ground traps
/// - Arrow/Dart: damage + dodge check
/// - BearTrap/Web: hold monster
/// - Pit/SpikedPit: damage + hold
/// - Teleport: random relocation
/// - FireTrap: fire damage (check resistance)
/// - SleepingGas: sleep monster
/// - LandMine: damage + destroy trap
///
/// Returns true if trap triggered, false if avoided.
pub fn mintrap(
    rng: &mut GameRng,
    monster: &Monster,
    trap_type: TrapType,
) -> MintrapResult {
    let mut result = MintrapResult::default();
    let mon_name = &monster.name;

    // Flying/levitating monsters avoid ground traps.
    // TODO: check MonsterFlags::FLY once PerMonst flags are available.
    let _is_flying = false;

    match trap_type {
        TrapType::Arrow => {
            // Dexterity-based dodge (simplified)
            let damage = roll_trap_damage(rng, trap_type);
            if rng.one_in(4) {
                result.messages.push(format!("An arrow misses the {}.", mon_name));
                result.avoided = true;
            } else {
                result.messages.push(format!("An arrow hits the {}!", mon_name));
                result.damage = damage;
            }
        }

        TrapType::Dart => {
            let damage = roll_trap_damage(rng, trap_type);
            if rng.one_in(4) {
                result.messages.push(format!("A dart misses the {}.", mon_name));
                result.avoided = true;
            } else {
                result.messages.push(format!("A dart hits the {}!", mon_name));
                result.damage = damage;
                // Poison check
                if rng.one_in(3) && !monster.resistances.contains(MonsterResistances::POISON) {
                    result.messages.push(format!("The {} is poisoned!", mon_name));
                    result.damage += rng.rnd(6) as i32;
                }
            }
        }

        TrapType::RockFall => {
            let damage = roll_trap_damage(rng, trap_type);
            result.messages.push(format!("A rock falls on the {}!", mon_name));
            result.damage = damage;
        }

        TrapType::Squeaky => {
            result.messages.push("A board beneath it squeaks loudly.".to_string());
            // Wakes up sleeping monsters nearby (handled by caller)
        }

        TrapType::BearTrap => {
            let damage = roll_trap_damage(rng, trap_type);
            result.messages.push(format!("The {} is caught in a bear trap!", mon_name));
            result.damage = damage;
            result.held_turns = (rng.rnd(5) + 3) as i32;
        }

        TrapType::LandMine => {
            let damage = roll_trap_damage(rng, trap_type);
            result.messages.push(format!("KAABLAMM!!! The {} triggers a land mine!", mon_name));
            result.damage = damage;
            result.trap_destroyed = true;
        }

        TrapType::RollingBoulder => {
            if rng.one_in(4) {
                result.messages.push(format!("A boulder misses the {}.", mon_name));
                result.avoided = true;
            } else {
                let damage = roll_trap_damage(rng, trap_type);
                result.messages.push(format!("A boulder hits the {}!", mon_name));
                result.damage = damage;
            }
        }

        TrapType::SleepingGas => {
            if monster.resistances.contains(MonsterResistances::SLEEP) {
                result.messages.push(format!("The {} resists the gas.", mon_name));
                result.avoided = true;
            } else {
                result.messages.push(format!("The {} falls asleep!", mon_name));
                result.held_turns = (rng.rnd(25) + 10) as i32;
            }
        }

        TrapType::RustTrap => {
            result.messages.push(format!("A gush of water hits the {}!", mon_name));
            // Rust damage to monster's iron equipment (erosion +1)
            result.damage = 0; // No HP damage, just equipment degradation
        }

        TrapType::FireTrap => {
            let damage = roll_trap_damage(rng, trap_type);
            if monster.resistances.contains(MonsterResistances::FIRE) {
                result.messages.push(format!("The {} is unaffected by the fire.", mon_name));
                result.avoided = true;
            } else {
                result.messages.push(format!("The {} is engulfed in flames!", mon_name));
                result.damage = damage;
            }
        }

        TrapType::Pit | TrapType::SpikedPit => {
            let damage = roll_trap_damage(rng, trap_type);
            let pit_name = trap_name(trap_type);
            result.messages.push(format!("The {} falls into a {}!", mon_name, pit_name));
            result.damage = damage;
            result.held_turns = (rng.rnd(6) + 2) as i32;

            if trap_type == TrapType::SpikedPit
                && rng.one_in(6)
                && !monster.resistances.contains(MonsterResistances::POISON)
            {
                result.messages.push("The spikes were poisoned!".to_string());
                result.damage += rng.rnd(8) as i32;
            }
        }

        TrapType::Hole | TrapType::TrapDoor => {
            result.messages.push(format!("The {} falls through!", mon_name));
            result.fell_through = true;
            result.damage = rng.rnd(6) as i32;
        }

        TrapType::Teleport => {
            result.messages.push(format!("The {} is teleported!", mon_name));
            let x = (rng.rn2(77) + 1) as i8;
            let y = (rng.rn2(19) + 1) as i8;
            result.teleport = Some((x, y));
        }

        TrapType::LevelTeleport => {
            if monster.resistances.contains(MonsterResistances::FIRE) {
                // Using FIRE as proxy for magic resistance check
                result.avoided = true;
            } else {
                result.messages.push(format!("The {} vanishes!", mon_name));
                result.fell_through = true;
            }
        }

        TrapType::Web => {
            result.messages.push(format!("The {} is caught in a web!", mon_name));
            result.held_turns = (rng.rnd(10) + 5) as i32;
        }

        TrapType::MagicTrap => {
            result.messages.push(format!("The {} is caught in a magical light!", mon_name));
            // Random minor effect
        }

        TrapType::AntiMagic => {
            // Drain energy from spellcasting monsters
            result.messages.push(format!("The {} shudders.", mon_name));
        }

        TrapType::Polymorph => {
            result.messages.push(format!("The {} undergoes a transformation!", mon_name));
            // Polymorph handled by caller
        }

        TrapType::MagicPortal | TrapType::Statue => {
            // Special handling by caller
            result.avoided = true;
        }
    }

    result
}

/// Trap perception: mark trap as seen if player has line of sight
pub fn seetrap(state: &mut GameState, x: i8, y: i8) {
    if state.player.blinded_timeout > 0 {
        return;
    }
    if let Some(trap) = state.current_level.trap_at_mut(x, y)
        && !trap.seen
    {
        trap.seen = true;
        let name = trap_name(trap.trap_type);
        state.message(format!("You see a {} here.", name));
    }
}

/// Trap perception when blind: feel the trap
pub fn feeltrap(state: &mut GameState, x: i8, y: i8) {
    if let Some(trap) = state.current_level.trap_at_mut(x, y)
        && !trap.seen
    {
        trap.seen = true;
        state.message("You feel a trap here.");
    }
}

/// Float up: escape from holding traps when gaining levitation/flying
pub fn float_up(state: &mut GameState) {
    if state.player.utrap > 0
        && matches!(state.player.utrap_type,
            PlayerTrapType::BearTrap | PlayerTrapType::Pit |
            PlayerTrapType::SpikedPit | PlayerTrapType::Web)
    {
        state.message("You float up, out of the trap.");
        state.player.utrap = 0;
        state.player.utrap_type = PlayerTrapType::None;
    }
    state.message("You start to float in the air!");
}

/// Float down: check current tile for trap, pool, lava when losing levitation
pub fn float_down(state: &mut GameState) {
    let px = state.player.pos.x;
    let py = state.player.pos.y;

    state.message("You float gently to the ground.");

    // Check for trap at current position
    if let Some(trap_type) = state.current_level.trap_at(px, py).map(|t| t.trap_type) {
        let dex = state.player.attr_current.get(crate::player::Attribute::Dexterity);
        let resistances = trap::resistances_from_properties(
            |prop| state.player.properties.has(prop),
            dex,
        );
        if let Some(trap) = state.current_level.trap_at_mut(px, py) {
            let result = trap::dotrap(
                &mut state.rng,
                trap,
                &resistances,
                false,
            );

            for msg in &result.messages {
                state.message(msg.clone());
            }

            if result.damage > 0 {
                state.player.take_damage(result.damage);
            }

            if result.held_turns > 0 {
                state.player.utrap = result.held_turns as u32;
                state.player.utrap_type = to_player_trap_type(trap_type);
            }

            if result.trap_destroyed {
                state.current_level.remove_trap(px, py);
            }
        }
    }
}

/// Disarm a holding trap (bear trap -> iron chain)
pub fn disarm_holdingtrap(state: &mut GameState, x: i8, y: i8) -> bool {
    let dex = state.player.attr_current.get(crate::player::Attribute::Dexterity);
    let trap_type = match state.current_level.trap_at(x, y) {
        Some(t) => t.trap_type,
        None => return false,
    };

    let difficulty = trap::disarm_difficulty(trap_type);
    let chance = 50 + (dex as i32 - 10) * 3 - difficulty;
    let roll = state.rng.rn2(100) as i32;

    if roll < chance.clamp(5, 95) {
        state.message("You disarm the trap.");
        state.current_level.remove_trap(x, y);
        true
    } else {
        state.message("You fail to disarm the trap.");
        false
    }
}

/// Disarm a shooting trap (arrow/dart -> get projectiles)
pub fn disarm_shooting_trap(state: &mut GameState, x: i8, y: i8) -> bool {
    let dex = state.player.attr_current.get(crate::player::Attribute::Dexterity);
    let trap_type = match state.current_level.trap_at(x, y) {
        Some(t) => t.trap_type,
        None => return false,
    };

    let difficulty = trap::disarm_difficulty(trap_type);
    let chance = 50 + (dex as i32 - 10) * 3 - difficulty;
    let roll = state.rng.rn2(100) as i32;

    if roll < chance.clamp(5, 95) {
        let name = trap_name(trap_type);
        state.message(format!("You disarm the {}.", name));
        state.current_level.remove_trap(x, y);
        // Create projectile objects on the ground
        let quantity = (state.rng.rnd(5) + 1) as i32;
        let proj_name = if trap_type == TrapType::Arrow { "arrow" } else { "dart" };
        let mut proj = crate::object::Object::new(
            crate::object::ObjectId(state.rng.rn2(10000)),
            0,
            crate::object::ObjectClass::Weapon,
        );
        proj.name = Some(proj_name.to_string());
        proj.quantity = quantity;
        state.current_level.add_object(proj, x, y);
        state.message(format!("You find {} {}s.", quantity, proj_name));
        true
    } else {
        state.message("You fail to disarm the trap.");
        // Failing to disarm might trigger it
        if state.rng.one_in(3) {
            state.message("Oops! You triggered it!");
        }
        false
    }
}

/// Disarm a squeaky board (always succeeds)
pub fn disarm_squeaky_board(state: &mut GameState, x: i8, y: i8) -> bool {
    if let Some(trap) = state.current_level.trap_at(x, y)
        && trap.trap_type == TrapType::Squeaky
    {
        state.message("You silence the squeaky board.");
        state.current_level.remove_trap(x, y);
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monster::{Monster, MonsterId, MonsterResistances};
    use crate::rng::GameRng;

    fn make_monster(name: &str) -> Monster {
        let mut m = Monster::new(MonsterId(1), 0, 5, 5);
        m.name = name.to_string();
        m
    }

    #[test]
    fn test_mintrap_arrow() {
        let mut rng = GameRng::from_entropy();
        let monster = make_monster("goblin");
        let result = mintrap(&mut rng, &monster, TrapType::Arrow);
        // Either hits or misses
        assert!(!result.messages.is_empty());
    }

    #[test]
    fn test_mintrap_bear_trap_holds() {
        let mut rng = GameRng::from_entropy();
        let monster = make_monster("gnome");
        let result = mintrap(&mut rng, &monster, TrapType::BearTrap);
        assert!(result.held_turns > 0);
        assert!(result.damage > 0);
    }

    #[test]
    fn test_mintrap_fire_resistant() {
        let mut rng = GameRng::from_entropy();
        let mut monster = make_monster("fire elemental");
        monster.resistances = MonsterResistances::FIRE;
        let result = mintrap(&mut rng, &monster, TrapType::FireTrap);
        assert!(result.avoided);
        assert_eq!(result.damage, 0);
    }

    #[test]
    fn test_mintrap_sleeping_gas_resistant() {
        let mut rng = GameRng::from_entropy();
        let mut monster = make_monster("elf");
        monster.resistances = MonsterResistances::SLEEP;
        let result = mintrap(&mut rng, &monster, TrapType::SleepingGas);
        assert!(result.avoided);
    }

    #[test]
    fn test_mintrap_land_mine_destroys() {
        let mut rng = GameRng::from_entropy();
        let monster = make_monster("kobold");
        let result = mintrap(&mut rng, &monster, TrapType::LandMine);
        assert!(result.trap_destroyed);
        assert!(result.damage > 0);
    }

    #[test]
    fn test_mintrap_teleport() {
        let mut rng = GameRng::from_entropy();
        let monster = make_monster("orc");
        let result = mintrap(&mut rng, &monster, TrapType::Teleport);
        assert!(result.teleport.is_some());
    }

    #[test]
    fn test_mintrap_pit_holds() {
        let mut rng = GameRng::from_entropy();
        let monster = make_monster("dwarf");
        let result = mintrap(&mut rng, &monster, TrapType::Pit);
        assert!(result.held_turns > 0);
        assert!(result.damage > 0);
    }
}

/// Attempt to disarm a trap at a location
pub fn do_disarm(state: &mut GameState, x: i8, y: i8) -> ActionResult {
    let trap = match state.current_level.trap_at(x, y) {
        Some(t) => t,
        None => return ActionResult::Failed("You see no trap there.".to_string()),
    };

    // Check if trap can be disarmed
    if !crate::dungeon::trap::can_disarm(trap.trap_type) {
        return ActionResult::Failed(format!(
            "You can't disarm that {}.",
            crate::dungeon::trap::trap_name(trap.trap_type)
        ));
    }

    let dex = state
        .player
        .attr_current
        .get(crate::player::Attribute::Dexterity) as i32;
    let skill = 0; // TODO: Get disarm skill from player

    let success = crate::dungeon::trap::try_disarm(&mut state.rng, &trap, dex, skill);

    if success {
        state.message(format!(
            "You successfully disarm the {}.",
            crate::dungeon::trap::trap_name(trap.trap_type)
        ));

        if let Some(idx) = state
            .current_level
            .traps
            .iter()
            .position(|t| t.x == x && t.y == y)
        {
            state.current_level.traps.remove(idx);
        }

        ActionResult::Success
    } else {
        state.message("You fail to disarm the trap.");
        ActionResult::Success // Takes a turn even if failed
    }
}

/// Attempt to escape from a trap holding the player
pub fn try_escape(state: &mut GameState, trap_type: crate::dungeon::TrapType) -> bool {
    let str_val = state
        .player
        .attr_current
        .get(crate::player::Attribute::Strength) as i8;
    crate::dungeon::trap::try_escape_trap(&mut state.rng, trap_type, str_val)
}
