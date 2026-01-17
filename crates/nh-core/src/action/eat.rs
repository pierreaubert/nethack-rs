//! Eating food and corpses (eat.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::object::ObjectClass;

/// Eat food from inventory
pub fn do_eat(state: &mut GameState, obj_letter: char) -> ActionResult {
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o,
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    if obj.class != ObjectClass::Food {
        return ActionResult::Failed("That's not something you can eat.".to_string());
    }

    let obj_name = obj.name.clone().unwrap_or_else(|| "food".to_string());
    state.message(format!("You eat the {}.", obj_name));

    // Basic nutrition gain
    state.player.nutrition += 100;
    state.player.update_hunger();

    // Remove the food item
    state.remove_from_inventory(obj_letter);

    ActionResult::Success
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{Object, ObjectClass, ObjectId};
    use crate::rng::GameRng;

    #[test]
    fn test_eat_non_food_fails() {
        let mut state = GameState::new(GameRng::from_entropy());
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.inv_letter = 'a';
        state.inventory.push(obj);

        let result = do_eat(&mut state, 'a');
        assert!(matches!(result, ActionResult::Failed(_)));
    }

    #[test]
    fn test_eat_missing_item_fails() {
        let mut state = GameState::new(GameRng::from_entropy());
        let result = do_eat(&mut state, 'z');
        assert!(matches!(result, ActionResult::Failed(_)));
    }

    #[test]
    fn test_eat_food_increases_nutrition() {
        let mut state = GameState::new(GameRng::from_entropy());
        let initial_nutrition = state.player.nutrition;
        
        let mut obj = Object::default();
        obj.id = ObjectId(1);
        obj.class = ObjectClass::Food;
        obj.inv_letter = 'a';
        state.inventory.push(obj);

        let result = do_eat(&mut state, 'a');
        assert!(matches!(result, ActionResult::Success));
        assert!(state.player.nutrition > initial_nutrition);
    }
}
