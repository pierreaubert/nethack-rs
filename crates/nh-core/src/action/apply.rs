//! Tool application (apply.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;

/// Apply a tool from inventory
pub fn do_apply(state: &mut GameState, obj_letter: char) -> ActionResult {
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o,
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    let obj_name = obj.name.clone().unwrap_or_else(|| "tool".to_string());
    state.message(format!("You apply the {}.", obj_name));

    ActionResult::Success
}
