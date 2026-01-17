//! Reading scrolls (read.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::magic::scroll::read_scroll;
use crate::object::ObjectClass;

/// Read a scroll from inventory
pub fn do_read(state: &mut GameState, obj_letter: char) -> ActionResult {
    // Get the scroll from inventory
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o.clone(),
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    if obj.class != ObjectClass::Scroll {
        return ActionResult::Failed("That's not something you can read.".to_string());
    }

    // Apply scroll effects
    let result = read_scroll(
        &obj,
        &mut state.player,
        &mut state.current_level,
        &mut state.rng,
    );

    // Display messages
    for msg in result.messages {
        state.message(msg);
    }

    // Consume the scroll if it was used
    if result.consumed {
        state.remove_from_inventory(obj_letter);
    }

    ActionResult::Success
}
