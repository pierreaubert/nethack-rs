//! Kicking (dokick.c)

use crate::action::{ActionResult, Direction};
use crate::dungeon::CellType;
use crate::gameloop::GameState;

/// Kick in a direction
pub fn do_kick(state: &mut GameState, dir: Direction) -> ActionResult {
    let (dx, dy) = dir.delta();
    let x = state.player.pos.x + dx;
    let y = state.player.pos.y + dy;

    if !state.current_level.is_valid_pos(x, y) {
        state.message("You kick at empty space.");
        return ActionResult::NoTime;
    }

    // Check for monster at target
    if let Some(monster) = state.current_level.monster_at(x, y) {
        let monster_name = monster.name.clone();
        state.message(format!("You kick the {}!", monster_name));
        return ActionResult::Success;
    }

    // Check cell type for doors
    let cell = state.current_level.cell(x as usize, y as usize);
    if matches!(cell.typ, CellType::Door) {
        state.message("You kick the door.");
        return ActionResult::Success;
    }

    state.message("You kick at nothing.");
    ActionResult::Success
}
