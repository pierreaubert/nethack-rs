//! Picking up and dropping items (pickup.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::object::{Object, ObjectClass};

/// Weight messages for lifting heavy objects
const MODERATE_LOAD_MSG: &str = "You have a little trouble lifting";
const NEAR_LOAD_MSG: &str = "You have much trouble lifting";
const OVERLOAD_MSG: &str = "You have extreme difficulty lifting";

/// Calculate gold weight (50 gold pieces = 1 unit)
pub const fn gold_weight(amount: i32) -> u32 {
    ((amount + 50) / 100) as u32
}

/// Check if an object can be picked up
pub fn can_pickup(obj: &Object, _state: &GameState) -> bool {
    // TODO: Check for cursed items stuck to floor
    // TODO: Check for cockatrice corpses without gloves
    // TODO: Check for loadstones
    match obj.class {
        ObjectClass::Ball | ObjectClass::Chain => false, // Punishment items
        _ => true,
    }
}

/// Check if picking up would exceed carrying capacity
pub fn would_overload(obj: &Object, state: &GameState, capacity: u32) -> bool {
    let current_weight: u32 = state.inventory.iter().map(|o| o.weight).sum();
    let obj_weight = obj.weight;

    current_weight + obj_weight > capacity
}

/// Get the load message based on weight ratio
pub fn load_message(current: u32, capacity: u32) -> Option<&'static str> {
    if capacity == 0 {
        return Some(OVERLOAD_MSG);
    }
    let ratio = (current * 100) / capacity;
    if ratio >= 100 {
        Some(OVERLOAD_MSG)
    } else if ratio >= 80 {
        Some(NEAR_LOAD_MSG)
    } else if ratio >= 60 {
        Some(MODERATE_LOAD_MSG)
    } else {
        None
    }
}

/// Pick up items at current location
pub fn do_pickup(state: &mut GameState) -> ActionResult {
    let x = state.player.pos.x;
    let y = state.player.pos.y;

    // Get IDs of objects at this position
    let object_ids: Vec<_> = state.current_level.objects_at(x, y)
        .iter()
        .map(|o| o.id)
        .collect();

    if object_ids.is_empty() {
        state.message("There is nothing here to pick up.");
        return ActionResult::NoTime;
    }

    let mut picked_up = Vec::new();

    // Remove each object from the level and add to inventory
    for id in object_ids {
        if let Some(obj) = state.current_level.remove_object(id) {
            let name = obj.display_name();
            state.add_to_inventory(obj);
            picked_up.push(name);
        }
    }

    if picked_up.is_empty() {
        state.message("There is nothing here to pick up.");
        return ActionResult::NoTime;
    }

    // Format pickup message
    if picked_up.len() == 1 {
        state.message(format!("You pick up {}.", picked_up[0]));
    } else {
        state.message(format!("You pick up {} items.", picked_up.len()));
    }

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

    let obj_name = obj.display_name();

    // Remove from inventory and place on floor
    if let Some(obj) = state.remove_from_inventory(obj_letter) {
        let x = state.player.pos.x;
        let y = state.player.pos.y;
        state.current_level.add_object(obj, x, y);
        state.message(format!("You drop {}.", obj_name));
        ActionResult::Success
    } else {
        ActionResult::Failed("Failed to drop item.".to_string())
    }
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

    #[test]
    fn test_pickup_item_from_floor() {
        let mut state = GameState::new(GameRng::from_entropy());
        
        // Place an item on the floor at player position
        let mut obj = Object::default();
        obj.class = ObjectClass::Food;
        obj.name = Some("apple".to_string());
        let x = state.player.pos.x;
        let y = state.player.pos.y;
        state.current_level.add_object(obj, x, y);
        
        // Verify item is on floor
        assert_eq!(state.current_level.objects_at(x, y).len(), 1);
        assert!(state.inventory.is_empty());
        
        // Pick up
        let result = do_pickup(&mut state);
        assert!(matches!(result, ActionResult::Success));
        
        // Verify item moved to inventory
        assert!(state.current_level.objects_at(x, y).is_empty());
        assert_eq!(state.inventory.len(), 1);
        assert_eq!(state.inventory[0].name, Some("apple".to_string()));
    }

    #[test]
    fn test_drop_item_to_floor() {
        let mut state = GameState::new(GameRng::from_entropy());
        
        // Add item to inventory
        let mut obj = Object::default();
        obj.class = ObjectClass::Food;
        obj.name = Some("bread".to_string());
        state.add_to_inventory(obj);
        
        let letter = state.inventory[0].inv_letter;
        let x = state.player.pos.x;
        let y = state.player.pos.y;
        
        // Verify item is in inventory
        assert_eq!(state.inventory.len(), 1);
        assert!(state.current_level.objects_at(x, y).is_empty());
        
        // Drop
        let result = do_drop(&mut state, letter);
        assert!(matches!(result, ActionResult::Success));
        
        // Verify item moved to floor
        assert!(state.inventory.is_empty());
        assert_eq!(state.current_level.objects_at(x, y).len(), 1);
    }
}
