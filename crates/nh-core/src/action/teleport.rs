//! Teleportation mechanics (teleport.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::player::Property;
use crate::{COLNO, ROWNO};

/// Teleport the player randomly on the current level
pub fn tele(state: &mut GameState) -> ActionResult {
    let has_control = state.player.properties.has(Property::TeleportControl);

    if has_control {
        state.message("You feel in control of the teleportation.");
    }

    let (new_x, new_y) = find_teleport_destination(state);

    state.player.pos.x = new_x;
    state.player.pos.y = new_y;
    state.message("You feel disoriented.");

    ActionResult::Success
}

fn find_teleport_destination(state: &mut GameState) -> (i8, i8) {
    for _ in 0..100 {
        let x = state.rng.rn2(COLNO as u32) as i8;
        let y = state.rng.rn2(ROWNO as u32) as i8;

        if state.current_level.is_walkable(x, y) {
            return (x, y);
        }
    }

    (state.player.pos.x, state.player.pos.y)
}

/// Level teleport
pub fn level_tele(state: &mut GameState, _target_depth: i32) -> ActionResult {
    state.message("You feel a wrenching sensation.");
    ActionResult::Success
}
