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
        // Try to open - check strength (simplified from C)
        let str_val = state.player.attr_current.get(crate::player::Attribute::Strength) as u32;
        let dex = state.player.attr_current.get(crate::player::Attribute::Dexterity) as u32;
        let con = state.player.attr_current.get(crate::player::Attribute::Constitution) as u32;
        
        // In C: rnl(20) < (ACURRSTR + ACURR(A_DEX) + ACURR(A_CON)) / 3
        let threshold = (str_val + dex + con) / 3;
        let roll = state.rng.rn2(20);
        
        if roll < threshold {
            // Success - open the door
            let cell = state.current_level.cell_mut(x as usize, y as usize);
            cell.set_door_state(DoorState::OPEN);
            state.message("The door opens.");
            ActionResult::Success
        } else {
            state.message("The door resists!");
            ActionResult::Success // Still takes a turn
        }
    } else {
        // No door or already open
        state.message("This doorway has no door.");
        ActionResult::NoTime
    }
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
        // Try to close - check strength (simplified from C)
        let str_val = state.player.attr_current.get(crate::player::Attribute::Strength) as u32;
        let dex = state.player.attr_current.get(crate::player::Attribute::Dexterity) as u32;
        let con = state.player.attr_current.get(crate::player::Attribute::Constitution) as u32;
        
        // In C: rn2(25) < (ACURRSTR + ACURR(A_DEX) + ACURR(A_CON)) / 3
        let threshold = (str_val + dex + con) / 3;
        let roll = state.rng.rn2(25);
        
        if roll < threshold {
            // Success - close the door
            let cell = state.current_level.cell_mut(x as usize, y as usize);
            cell.set_door_state(DoorState::CLOSED);
            state.message("The door closes.");
            ActionResult::Success
        } else {
            state.message("The door resists!");
            ActionResult::Success // Still takes a turn
        }
    } else {
        state.message("This doorway has no door.");
        ActionResult::NoTime
    }
}
