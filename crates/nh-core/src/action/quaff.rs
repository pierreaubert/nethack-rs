//! Quaffing potions (potion.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::magic::potion::quaff_potion;
use crate::object::ObjectClass;

/// Quaff a potion from inventory
pub fn do_quaff(state: &mut GameState, obj_letter: char) -> ActionResult {
    // Get the potion from inventory
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o.clone(),
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    if obj.class != ObjectClass::Potion {
        return ActionResult::Failed("That's not something you can drink.".to_string());
    }

    // Apply potion effects
    let result = quaff_potion(&obj, &mut state.player, &mut state.rng);

    // Display messages
    for msg in result.messages {
        state.message(msg);
    }

    // Consume the potion if it was used
    if result.consumed {
        state.remove_from_inventory(obj_letter);
    }

    if result.player_died {
        return ActionResult::Died("poisoned".to_string());
    }

    ActionResult::Success
}
