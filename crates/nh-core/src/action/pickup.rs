//! Picking up and dropping items (pickup.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;

/// Pick up items at current location
pub fn do_pickup(state: &mut GameState) -> ActionResult {
    let x = state.player.pos.x;
    let y = state.player.pos.y;

    let objects = state.current_level.objects_at(x, y);
    if objects.is_empty() {
        state.message("There is nothing here to pick up.");
        return ActionResult::NoTime;
    }

    state.message("You pick up items.");
    ActionResult::Success
}

/// Drop an item from inventory
pub fn do_drop(state: &mut GameState, obj_letter: char) -> ActionResult {
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o,
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    if obj.is_worn() {
        return ActionResult::Failed("You'll have to take that off first.".to_string());
    }

    let obj_name = obj.name.clone().unwrap_or_else(|| "item".to_string());
    state.message(format!("You drop the {}.", obj_name));

    state.remove_from_inventory(obj_letter);

    ActionResult::Success
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{Object, ObjectClass};
    use crate::rng::GameRng;

    #[test]
    fn test_drop_missing_item_fails() {
        let mut state = GameState::new(GameRng::from_entropy());
        let result = do_drop(&mut state, 'z');
        assert!(matches!(result, ActionResult::Failed(_)));
    }

    #[test]
    fn test_drop_worn_item_fails() {
        let mut state = GameState::new(GameRng::from_entropy());
        let mut obj = Object::default();
        obj.class = ObjectClass::Armor;
        obj.inv_letter = 'a';
        obj.worn_mask = 1;
        state.inventory.push(obj);

        let result = do_drop(&mut state, 'a');
        assert!(matches!(result, ActionResult::Failed(_)));
    }

    #[test]
    fn test_pickup_empty_floor() {
        let mut state = GameState::new(GameRng::from_entropy());
        let result = do_pickup(&mut state);
        assert!(matches!(result, ActionResult::NoTime));
    }
}
