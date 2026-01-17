//! Wearing and removing equipment (do_wear.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::object::ObjectClass;

/// Wear armor
pub fn do_wear(state: &mut GameState, obj_letter: char) -> ActionResult {
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o,
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    if obj.class != ObjectClass::Armor {
        return ActionResult::Failed("That's not something you can wear.".to_string());
    }

    if obj.is_worn() {
        return ActionResult::Failed("You're already wearing that.".to_string());
    }

    let obj_name = obj.name.clone().unwrap_or_else(|| "armor".to_string());
    state.message(format!("You put on the {}.", obj_name));

    ActionResult::Success
}

/// Take off armor
pub fn do_takeoff(state: &mut GameState, obj_letter: char) -> ActionResult {
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o,
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    if !obj.is_worn() {
        return ActionResult::Failed("You're not wearing that.".to_string());
    }

    if obj.is_cursed() {
        return ActionResult::Failed("You can't remove it, it's cursed!".to_string());
    }

    let obj_name = obj.name.clone().unwrap_or_else(|| "armor".to_string());
    state.message(format!("You take off the {}.", obj_name));

    ActionResult::Success
}

/// Wield a weapon
pub fn do_wield(state: &mut GameState, obj_letter: char) -> ActionResult {
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o,
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    let obj_name = obj.name.clone().unwrap_or_else(|| "weapon".to_string());
    state.message(format!("You wield the {}.", obj_name));

    ActionResult::Success
}

/// Put on an accessory (ring/amulet)
pub fn do_puton(state: &mut GameState, obj_letter: char) -> ActionResult {
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o,
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    if !matches!(obj.class, ObjectClass::Ring | ObjectClass::Amulet) {
        return ActionResult::Failed("That's not something you can put on.".to_string());
    }

    let obj_name = obj.name.clone().unwrap_or_else(|| "accessory".to_string());
    state.message(format!("You put on the {}.", obj_name));

    ActionResult::Success
}

/// Remove an accessory
pub fn do_remove(state: &mut GameState, obj_letter: char) -> ActionResult {
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o,
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    if !obj.is_worn() {
        return ActionResult::Failed("You're not wearing that.".to_string());
    }

    if obj.is_cursed() {
        return ActionResult::Failed("You can't remove it, it's cursed!".to_string());
    }

    let obj_name = obj.name.clone().unwrap_or_else(|| "accessory".to_string());
    state.message(format!("You remove the {}.", obj_name));

    ActionResult::Success
}
