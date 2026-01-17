//! Engraving system (engrave.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;

/// Engrave on the floor
pub fn do_engrave(state: &mut GameState, text: &str) -> ActionResult {
    if text.is_empty() {
        state.message("You write in the dust with your fingers.");
        return ActionResult::Success;
    }

    state.message(format!("You engrave \"{}\" on the floor.", text));
    ActionResult::Success
}

/// Read engraving at current location
pub fn read_engrave(_state: &mut GameState) -> Option<String> {
    None
}
