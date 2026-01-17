//! Opening and closing doors (lock.c)

use crate::action::{ActionResult, Direction};
use crate::dungeon::CellType;
use crate::gameloop::GameState;

/// Open a door in the given direction
pub fn do_open(state: &mut GameState, dir: Direction) -> ActionResult {
    let (dx, dy) = dir.delta();
    let x = state.player.pos.x + dx;
    let y = state.player.pos.y + dy;

    if !state.current_level.is_valid_pos(x, y) {
        state.message("There's nothing there to open.");
        return ActionResult::NoTime;
    }

    let cell = state.current_level.cell(x as usize, y as usize);

    match cell.typ {
        CellType::Door => {
            state.message("This door is already open.");
            ActionResult::NoTime
        }
        _ => {
            state.message("There's nothing there to open.");
            ActionResult::NoTime
        }
    }
}

/// Close a door in the given direction
pub fn do_close(state: &mut GameState, dir: Direction) -> ActionResult {
    let (dx, dy) = dir.delta();
    let x = state.player.pos.x + dx;
    let y = state.player.pos.y + dy;

    if !state.current_level.is_valid_pos(x, y) {
        state.message("There's nothing there to close.");
        return ActionResult::NoTime;
    }

    let cell = state.current_level.cell(x as usize, y as usize);

    match cell.typ {
        CellType::Door => {
            state.message("You close the door.");
            ActionResult::Success
        }
        _ => {
            state.message("There's nothing there to close.");
            ActionResult::NoTime
        }
    }
}
