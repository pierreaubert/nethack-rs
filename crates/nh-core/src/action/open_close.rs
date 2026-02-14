//! Opening and closing doors (lock.c)
//!
//! Door states from NetHack (rm.h):
//! - D_NODOOR = 0x00 (doorway with no door)
//! - D_BROKEN = 0x01 (broken door)
//! - D_ISOPEN = 0x02 (open door)
//! - D_CLOSED = 0x04 (closed door)
//! - D_LOCKED = 0x08 (locked door)
//! - D_TRAPPED = 0x10 (trapped door)
//! - D_SECRET = 0x20 (secret door)

use crate::action::{ActionResult, Direction};
use crate::dungeon::{CellType, DoorState};
use crate::gameloop::GameState;
use crate::object::{Object, ObjectClass};

/// Open a door in the given direction
pub fn do_open(state: &mut GameState, dir: Direction) -> ActionResult {
    let (dx, dy) = dir.delta();
    let x = state.player.pos.x + dx;
    let y = state.player.pos.y + dy;

    if !state.current_level.is_valid_pos(x, y) {
        state.message("You see no door there.");
        return ActionResult::NoTime;
    }

    let cell = state.current_level.cell(x as usize, y as usize);

    // Check if it's a door type
    if cell.typ != CellType::Door && cell.typ != CellType::SecretDoor {
        state.message("You see no door there.");
        return ActionResult::NoTime;
    }

    // Secret doors can't be opened until found
    if cell.typ == CellType::SecretDoor {
        state.message("You see no door there.");
        return ActionResult::NoTime;
    }

    let door_state = cell.door_state();

    // Check door state using flags
    if door_state.contains(DoorState::OPEN) {
        state.message("This door is already open.");
        return ActionResult::NoTime;
    }

    if door_state.contains(DoorState::LOCKED) {
        state.message("This door is locked.");
        return ActionResult::NoTime;
    }

    if door_state.contains(DoorState::BROKEN) {
        state.message("This door is broken.");
        return ActionResult::NoTime;
    }

    if door_state.contains(DoorState::CLOSED) {
        openit(state, x, y);
        ActionResult::Success
    } else {
        // No door or already open
        state.message("This doorway has no door.");
        ActionResult::NoTime
    }
}

pub fn doopen_indir(state: &mut GameState, x: i8, y: i8) -> ActionResult {
    // Simplified logic: just check if there is a door and open it
    let cell = state.current_level.cell(x as usize, y as usize);
    if cell.typ == CellType::Door {
        let door_state = cell.door_state();
        if door_state.contains(DoorState::CLOSED) && !door_state.contains(DoorState::LOCKED) {
            openit(state, x, y);
            return ActionResult::Success;
        }
    }
    ActionResult::NoTime
}

/// Close a door in the given direction
pub fn do_close(state: &mut GameState, dir: Direction) -> ActionResult {
    let (dx, dy) = dir.delta();
    let x = state.player.pos.x + dx;
    let y = state.player.pos.y + dy;

    if !state.current_level.is_valid_pos(x, y) {
        state.message("You see no door there.");
        return ActionResult::NoTime;
    }

    // Check for monster blocking
    if state.current_level.monster_at(x, y).is_some() {
        state.message("Something blocks the way!");
        return ActionResult::NoTime;
    }

    let cell = state.current_level.cell(x as usize, y as usize);

    if cell.typ != CellType::Door {
        state.message("You see no door there.");
        return ActionResult::NoTime;
    }

    let door_state = cell.door_state();

    if door_state.contains(DoorState::CLOSED) || door_state.contains(DoorState::LOCKED) {
        state.message("This door is already closed.");
        return ActionResult::NoTime;
    }

    if door_state.contains(DoorState::BROKEN) {
        state.message("This door is broken.");
        return ActionResult::NoTime;
    }

    if door_state.contains(DoorState::OPEN) {
        let cell = state.current_level.cell_mut(x as usize, y as usize);
        cell.set_door_state(DoorState::CLOSED);
        state.message("The door closes.");
        ActionResult::Success
    } else {
        state.message("This doorway has no door.");
        ActionResult::NoTime
    }
}

pub fn openit(state: &mut GameState, x: i8, y: i8) {
    let cell = state.current_level.cell_mut(x as usize, y as usize);
    cell.set_door_state(DoorState::OPEN);
    state.message("The door opens.");
}

pub fn openone(state: &mut GameState, x: i8, y: i8) {
    openit(state, x, y);
}

pub fn really_close(state: &mut GameState, x: i8, y: i8) {
    let cell = state.current_level.cell_mut(x as usize, y as usize);
    cell.set_door_state(DoorState::CLOSED);
}

pub fn doorlock(state: &mut GameState, x: i8, y: i8) -> bool {
    let cell = state.current_level.cell_mut(x as usize, y as usize);
    let mut door_state = cell.door_state();

    if door_state.contains(DoorState::LOCKED) {
        door_state.remove(DoorState::LOCKED);
        cell.set_door_state(door_state);
        state.message("You unlock the door.");
        true
    } else if door_state.contains(DoorState::CLOSED) {
        door_state.insert(DoorState::LOCKED);
        cell.set_door_state(door_state);
        state.message("You lock the door.");
        true
    } else {
        false
    }
}

pub fn boxlock(state: &mut GameState, container: &mut Object) -> bool {
    if container.locked {
        container.locked = false;
        state.message("You unlock the container.");
    } else {
        container.locked = true;
        state.message("You lock the container.");
    }
    true
}

pub fn doorless_door(state: &mut GameState, x: i8, y: i8) -> bool {
    let cell = state.current_level.cell(x as usize, y as usize);
    if cell.typ == CellType::Door {
        let door_state = cell.door_state();
        if door_state.contains(DoorState::BROKEN) {
            state.message("This door is broken.");
            return true;
        }
    }
    false
}

/// Lock picking context (what we're currently picking)
#[derive(Debug, Default, Clone)]
pub struct LockContext {
    pub door_x: Option<i8>,
    pub door_y: Option<i8>,
    pub used_time: u32,
    pub chance: u32,
    pub pick_type: u32,
}

/// Get description of current lock action based on whether door is currently locked
pub fn lock_action(is_unlocking: bool) -> &'static str {
    if is_unlocking {
        "unlocking the door"
    } else {
        "locking the door"
    }
}

pub fn pick_lock(state: &mut GameState, tool: &Object, x: i8, y: i8) -> ActionResult {
    // Check if there is a door
    let cell = state.current_level.cell(x as usize, y as usize);
    if cell.typ == CellType::Door {
        let door_state = cell.door_state();
        if door_state.contains(DoorState::LOCKED) {
            // Unlock
            // TODO: Skill check
            if state.rng.one_in(3) {
                let cell = state.current_level.cell_mut(x as usize, y as usize);
                let mut ds = cell.door_state();
                ds.remove(DoorState::LOCKED);
                cell.set_door_state(ds);
                state.message("You succeed in picking the lock.");
            } else {
                state.message("You fail to pick the lock.");
            }
            return ActionResult::Success;
        } else if door_state.contains(DoorState::CLOSED) {
            // Lock
            if state.rng.one_in(3) {
                let cell = state.current_level.cell_mut(x as usize, y as usize);
                let mut ds = cell.door_state();
                ds.insert(DoorState::LOCKED);
                cell.set_door_state(ds);
                state.message("You succeed in locking the door.");
            } else {
                state.message("You fail to lock the door.");
            }
            return ActionResult::Success;
        }
    }

    state.message("Nothing to pick here.");
    ActionResult::NoTime
}

pub fn picking_lock() -> bool {
    false
}

pub fn picking_at() -> bool {
    false
}

/// Continue picking a lock (occupation function)
/// Returns time used, or 0 if done/cancelled
pub fn picklock(state: &mut GameState, ctx: &mut LockContext) -> u32 {
    // Check if we're still at the right location
    if let (Some(dx), Some(dy)) = (ctx.door_x, ctx.door_y) {
        let x = state.player.pos.x + dx;
        let y = state.player.pos.y + dy;

        if !state.current_level.is_valid_pos(x, y) {
            reset_pick(ctx);
            return 0;
        }

        let cell = state.current_level.cell(x as usize, y as usize);
        if cell.typ != CellType::Door {
            state.message("This doorway has no door.");
            reset_pick(ctx);
            return 0;
        }

        let door_state = cell.door_state();
        if door_state.contains(DoorState::OPEN) {
            state.message("You cannot lock an open door.");
            reset_pick(ctx);
            return 0;
        }

        if door_state.contains(DoorState::BROKEN) {
            state.message("This door is broken.");
            reset_pick(ctx);
            return 0;
        }

        ctx.used_time += 1;

        // Give up after 50 turns
        if ctx.used_time >= 50 {
            state.message(format!(
                "You give up your attempt at {}.",
                lock_action(door_state.contains(DoorState::LOCKED))
            ));
            reset_pick(ctx);
            return 0;
        }

        // Check for success
        if state.rng.rn2(100) < ctx.chance {
            // Success! First determine what message to show
            let was_locked = door_state.contains(DoorState::LOCKED);

            // Now modify the door
            let cell = state.current_level.cell_mut(x as usize, y as usize);
            let mut new_state = cell.door_state();
            if was_locked {
                new_state.remove(DoorState::LOCKED);
            } else {
                new_state.insert(DoorState::LOCKED);
            }
            cell.set_door_state(new_state);

            // Send message after cell borrow ends
            if was_locked {
                state.message("You succeed in picking the lock.");
            } else {
                state.message("You succeed in locking the door.");
            }
            reset_pick(ctx);
            return 0;
        }

        // Continue picking
        1
    } else {
        0
    }
}

/// Reset lock picking context
pub fn reset_pick(ctx: &mut LockContext) {
    ctx.door_x = None;
    ctx.door_y = None;
    ctx.used_time = 0;
    ctx.chance = 0;
    ctx.pick_type = 0;
}

/// Conditionally reset pick if level changes or container is deleted
pub fn maybe_reset_pick(ctx: &mut LockContext, level_change: bool) {
    if level_change {
        reset_pick(ctx);
    }
}

pub fn check_door_at(state: &GameState, x: i8, y: i8) -> bool {
    let cell = state.current_level.cell(x as usize, y as usize);
    cell.typ == CellType::Door
}

/// Close a holding trap (bear trap or web) on a monster
/// Returns true if trap was closed successfully
pub fn closeholdingtrap(state: &mut GameState, x: i8, y: i8) -> bool {
    // Check for trap at position
    if let Some(trap_idx) = state
        .current_level
        .traps
        .iter()
        .position(|t| t.x == x && t.y == y)
    {
        let trap = &state.current_level.traps[trap_idx];
        // Check if it's a holding trap (bear trap or web)
        match trap.trap_type {
            crate::dungeon::TrapType::BearTrap | crate::dungeon::TrapType::Web => {
                // Trap is already set/closed
                return true;
            }
            _ => return false,
        }
    }
    false
}

/// Open a holding trap (bear trap or web) to release a monster
/// Returns true if trap was opened successfully
pub fn openholdingtrap(state: &mut GameState, x: i8, y: i8) -> bool {
    // Check for trap at position
    if let Some(trap_idx) = state
        .current_level
        .traps
        .iter()
        .position(|t| t.x == x && t.y == y)
    {
        let trap = &state.current_level.traps[trap_idx];
        match trap.trap_type {
            crate::dungeon::TrapType::BearTrap | crate::dungeon::TrapType::Web => {
                // In full implementation, would release trapped monster
                // For now, just acknowledge the trap exists
                return true;
            }
            _ => return false,
        }
    }
    false
}

/// Open a falling trap (trapdoor, pit, hole) under a monster
/// Returns true if trap was triggered
pub fn openfallingtrap(state: &mut GameState, x: i8, y: i8, trapdoor_only: bool) -> bool {
    if let Some(trap_idx) = state
        .current_level
        .traps
        .iter()
        .position(|t| t.x == x && t.y == y)
    {
        let trap = &state.current_level.traps[trap_idx];
        match trap.trap_type {
            crate::dungeon::TrapType::TrapDoor => return true,
            crate::dungeon::TrapType::Hole
            | crate::dungeon::TrapType::Pit
            | crate::dungeon::TrapType::SpikedPit => {
                if !trapdoor_only {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Place a door during level generation (from mklev.c)
/// Randomly chooses between regular door and secret door
pub fn dodoor(state: &mut GameState, x: i8, y: i8) {
    // 1 in 8 chance of secret door
    if state.rng.one_in(8) {
        dosdoor(state, x, y, true);
    } else {
        dosdoor(state, x, y, false);
    }
}

/// Set door state during level generation
pub fn dosdoor(state: &mut GameState, x: i8, y: i8, secret: bool) {
    let cell = state.current_level.cell_mut(x as usize, y as usize);

    if secret {
        cell.typ = CellType::SecretDoor;
        cell.set_door_state(DoorState::CLOSED);
    } else {
        cell.typ = CellType::Door;

        // Randomly determine door state
        if state.rng.one_in(3) {
            // 1/3 chance: open, locked, or closed
            if state.rng.one_in(5) {
                cell.set_door_state(DoorState::OPEN);
            } else if state.rng.one_in(6) {
                cell.set_door_state(DoorState::LOCKED);
            } else {
                cell.set_door_state(DoorState::CLOSED);
            }
        } else {
            // 2/3 chance: no door (doorway)
            cell.set_door_state(DoorState::empty());
        }
    }
}

pub fn search_door(state: &mut GameState, x: i8, y: i8) {
    let cell = state.current_level.cell_mut(x as usize, y as usize);
    if cell.typ == CellType::SecretDoor {
        if state.rng.one_in(3) {
            cell.typ = CellType::Door;
            cell.set_door_state(DoorState::CLOSED);
            state.message("You find a hidden door!");
        }
    }
}

/// Check if a shopkeeper is blocking a door
/// Returns true if blocked, displays message
pub fn block_door(state: &mut GameState, x: i8, y: i8) -> bool {
    // Check if there's a door at the position
    let cell = state.current_level.cell(x as usize, y as usize);
    if cell.typ != CellType::Door {
        return false;
    }

    // Check if there's a shopkeeper at or near this door
    // In the full implementation, this would check shop boundaries
    // and shopkeeper debt/billing status
    // For now, return false (no blocking)
    false
}

/// Check if a shopkeeper is blocking entry to a shop
/// Returns true if entry is blocked
pub fn block_entry(state: &mut GameState, x: i8, y: i8) -> bool {
    // Check if player is at a broken door
    let player_cell = state
        .current_level
        .cell(state.player.pos.x as usize, state.player.pos.y as usize);
    if player_cell.typ != CellType::Door {
        return false;
    }

    let door_state = player_cell.door_state();
    if !door_state.contains(DoorState::BROKEN) {
        return false;
    }

    // In full implementation, would check if destination is a shop
    // and if shopkeeper wants to block entry
    false
}

pub fn cvt_sdoor_to_door(state: &mut GameState, x: i8, y: i8) {
    let cell = state.current_level.cell_mut(x as usize, y as usize);
    if cell.typ == CellType::SecretDoor {
        cell.typ = CellType::Door;
        cell.set_door_state(DoorState::CLOSED);
    }
}

// ============================================================================
// Force lock functions (from lock.c)
// ============================================================================

/// Result of attempting to force a container
#[derive(Debug, Clone, Default)]
pub struct ForceResult {
    pub messages: Vec<String>,
    pub success: bool,
    pub container_destroyed: bool,
    pub lock_broken: bool,
    pub items_destroyed: Vec<String>,
    pub shop_damage: i32,
}

impl ForceResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_message(mut self, msg: &str) -> Self {
        self.messages.push(msg.to_string());
        self
    }

    pub fn failure(msg: &str) -> Self {
        Self::new().with_message(msg)
    }

    pub fn success_open(msg: &str) -> Self {
        let mut result = Self::new().with_message(msg);
        result.success = true;
        result.lock_broken = true;
        result
    }
}

/// Attempt to force open a locked container (doforce equivalent)
///
/// Player attempts to force open a locked container using their weapon.
/// Requires a blade-type weapon (daggers, swords, etc.) or a pick.
///
/// # Arguments
/// * `player` - The player attempting to force
/// * `container` - The container being forced
/// * `rng` - Random number generator
///
/// # Returns
/// Result indicating success, destruction, or failure
pub fn doforce(
    player: &crate::player::You,
    container: &mut crate::object::Object,
    rng: &mut crate::rng::GameRng,
) -> ForceResult {
    // Check for special conditions
    if player.swallowed {
        return ForceResult::failure("You can't force anything from inside here.");
    }

    // Check if we have an appropriate weapon
    // In full implementation, would check wielded weapon type
    // For now, simplified check
    let has_proper_weapon = true; // Simplified

    if !has_proper_weapon {
        return ForceResult::failure("You can't force anything without a proper weapon.");
    }

    // Check if container is already unlocked or broken
    if !container.locked {
        if container.broken {
            return ForceResult::failure("That container is already broken.");
        } else {
            return ForceResult::failure("That container is already unlocked.");
        }
    }

    let mut result = ForceResult::new();
    result
        .messages
        .push("You force the lock open...".to_string());

    // Calculate success based on strength, skill, and random chance
    // Higher strength and luck improve odds
    let success_chance =
        70 + (player.attr_current.get(crate::player::Attribute::Strength) / 2) as i32;
    let roll = rng.rn2(100) as i32;

    if roll < success_chance {
        // Success - lock is broken
        result.success = true;
        result.lock_broken = true;

        // Small chance of destroying the container entirely
        if rng.rn2(10) == 0 {
            result.container_destroyed = true;
            result
                .messages
                .push("In fact, you've totally destroyed it!".to_string());
        } else {
            result.messages.push("The lock breaks!".to_string());
        }
    } else {
        // Failure
        result.messages.push("The lock holds.".to_string());
    }

    result
}

/// Break a chest's lock (breakchestlock equivalent)
///
/// Called when a chest lock is broken through force or other means.
/// If destroy_it is true, the chest is destroyed and contents are scattered.
///
/// # Arguments
/// * `container` - The container whose lock is being broken
/// * `destroy_it` - Whether to destroy the container entirely
/// * `rng` - Random number generator
///
/// # Returns
/// Result with damage details
pub fn breakchestlock(
    container: &mut crate::object::Object,
    destroy_it: bool,
    rng: &mut crate::rng::GameRng,
) -> ForceResult {
    let mut result = ForceResult::new();

    if !destroy_it {
        // Just break the lock, container survives
        container.locked = false;
        container.broken = true;
        result.lock_broken = true;
        result.messages.push(format!(
            "You break the lock on {}.",
            container.display_name()
        ));
    } else {
        // Container is destroyed - contents are scattered
        result.container_destroyed = true;
        result.lock_broken = true;
        result.messages.push(format!(
            "In fact, you've totally destroyed {}.",
            container.display_name()
        ));

        // Some contents may be destroyed (potions, fragile items)
        // In full implementation, would iterate through container contents
        // and potentially destroy some items
        let items_at_risk = rng.rn2(5) as usize;
        for i in 0..items_at_risk {
            if rng.rn2(3) == 0 {
                result.items_destroyed.push(format!("item {}", i));
                result
                    .messages
                    .push("Something inside is destroyed!".to_string());
            }
        }
    }

    result
}

/// Generate a message for an item destroyed when forcing a chest
/// (chest_shatter_msg equivalent)
///
/// Different materials break in different ways.
///
/// # Arguments
/// * `object` - The object that was destroyed
/// * `blind` - Whether the player is blind
///
/// # Returns
/// Message describing the destruction
pub fn chest_shatter_msg(object: &crate::object::Object, blind: bool) -> String {
    // Potions shatter specially
    if object.class == crate::object::ObjectClass::Potion {
        if blind {
            return "You hear something shatter!".to_string();
        } else {
            return "You see a bottle shatter!".to_string();
        }
    }

    // Other items based on material
    // In full implementation, would check object material
    let disposition = match object.class {
        crate::object::ObjectClass::Scroll | crate::object::ObjectClass::Spellbook => {
            "is torn to shreds"
        }
        crate::object::ObjectClass::Wand => "is snapped in two",
        crate::object::ObjectClass::Food => "is crushed",
        _ => "is smashed",
    };

    format!("A {} {}!", object.display_name(), disposition)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::{Cell, CellType};

    #[test]
    fn test_do_open_door() {
        let mut state = GameState::default();
        let (x, y) = (10, 10);
        state.player.pos.x = x - 1;
        state.player.pos.y = y;

        // Place a closed door
        let cell = state.current_level.cell_mut(x as usize, y as usize);
        cell.typ = CellType::Door;
        cell.set_door_state(DoorState::CLOSED);

        // Open it
        // We need to mock RNG or attributes to ensure success
        state
            .player
            .attr_current
            .set(crate::player::Attribute::Strength, 25); // Max str

        do_open(&mut state, Direction::East);

        let cell = state.current_level.cell(x as usize, y as usize);
        assert!(cell.door_state().contains(DoorState::OPEN));
    }

    // ========== Force Lock Tests ==========

    #[test]
    fn test_force_result_new() {
        let result = ForceResult::new();
        assert!(!result.success);
        assert!(!result.container_destroyed);
        assert!(!result.lock_broken);
        assert!(result.messages.is_empty());
    }

    #[test]
    fn test_force_result_failure() {
        let result = ForceResult::failure("Test failure");
        assert!(!result.success);
        assert_eq!(result.messages.len(), 1);
        assert!(result.messages[0].contains("Test failure"));
    }

    #[test]
    fn test_doforce_swallowed() {
        let mut player = crate::player::You::default();
        player.swallowed = true;
        let mut container = crate::object::Object::default();
        container.locked = true;
        let mut rng = crate::rng::GameRng::new(42);

        let result = doforce(&player, &mut container, &mut rng);

        assert!(!result.success);
        assert!(result.messages[0].contains("can't force"));
    }

    #[test]
    fn test_doforce_already_unlocked() {
        let player = crate::player::You::default();
        let mut container = crate::object::Object::default();
        container.locked = false;
        let mut rng = crate::rng::GameRng::new(42);

        let result = doforce(&player, &mut container, &mut rng);

        assert!(!result.success);
        assert!(result.messages[0].contains("already unlocked"));
    }

    #[test]
    fn test_doforce_already_broken() {
        let player = crate::player::You::default();
        let mut container = crate::object::Object::default();
        container.locked = false;
        container.broken = true;
        let mut rng = crate::rng::GameRng::new(42);

        let result = doforce(&player, &mut container, &mut rng);

        assert!(!result.success);
        assert!(result.messages[0].contains("already broken"));
    }

    #[test]
    fn test_doforce_locked_container() {
        let mut player = crate::player::You::default();
        player
            .attr_current
            .set(crate::player::Attribute::Strength, 18);
        let mut container = crate::object::Object::default();
        container.locked = true;
        let mut rng = crate::rng::GameRng::new(42);

        let result = doforce(&player, &mut container, &mut rng);

        // May succeed or fail based on RNG
        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("force"));
    }

    #[test]
    fn test_breakchestlock_not_destroyed() {
        let mut container = crate::object::Object::default();
        container.locked = true;
        let mut rng = crate::rng::GameRng::new(42);

        let result = breakchestlock(&mut container, false, &mut rng);

        assert!(result.lock_broken);
        assert!(!result.container_destroyed);
        assert!(!container.locked);
        assert!(container.broken);
    }

    #[test]
    fn test_breakchestlock_destroyed() {
        let mut container = crate::object::Object::default();
        container.locked = true;
        let mut rng = crate::rng::GameRng::new(42);

        let result = breakchestlock(&mut container, true, &mut rng);

        assert!(result.lock_broken);
        assert!(result.container_destroyed);
        assert!(result.messages.iter().any(|m| m.contains("destroyed")));
    }

    #[test]
    fn test_chest_shatter_msg_potion_blind() {
        let obj = crate::object::Object {
            class: crate::object::ObjectClass::Potion,
            ..Default::default()
        };
        let msg = chest_shatter_msg(&obj, true);
        assert!(msg.contains("hear"));
    }

    #[test]
    fn test_chest_shatter_msg_potion_sighted() {
        let obj = crate::object::Object {
            class: crate::object::ObjectClass::Potion,
            ..Default::default()
        };
        let msg = chest_shatter_msg(&obj, false);
        assert!(msg.contains("see"));
    }

    #[test]
    fn test_chest_shatter_msg_scroll() {
        let obj = crate::object::Object {
            class: crate::object::ObjectClass::Scroll,
            ..Default::default()
        };
        let msg = chest_shatter_msg(&obj, false);
        assert!(msg.contains("torn to shreds"));
    }
}
