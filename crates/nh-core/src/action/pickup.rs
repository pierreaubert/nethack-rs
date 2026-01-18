//! Picking up and dropping items (pickup.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::object::{Object, ObjectClass};
use crate::player::Encumbrance;

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

// ============================================================================
// Autopickup system
// ============================================================================

/// Pickup burden threshold (how much load to allow before stopping autopickup)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PickupBurden {
    /// Only autopickup if staying unencumbered
    #[default]
    Unencumbered,
    /// Autopickup up to burdened
    Burdened,
    /// Autopickup up to stressed
    Stressed,
    /// Autopickup up to strained
    Strained,
    /// Autopickup up to overtaxed (almost everything)
    Overtaxed,
    /// Always autopickup regardless of weight
    Overloaded,
}

impl PickupBurden {
    /// Check if the given encumbrance level is acceptable for autopickup
    pub fn allows_encumbrance(&self, enc: Encumbrance) -> bool {
        match self {
            PickupBurden::Unencumbered => enc == Encumbrance::Unencumbered,
            PickupBurden::Burdened => matches!(enc, Encumbrance::Unencumbered | Encumbrance::Burdened),
            PickupBurden::Stressed => matches!(enc, Encumbrance::Unencumbered | Encumbrance::Burdened | Encumbrance::Stressed),
            PickupBurden::Strained => matches!(enc, Encumbrance::Unencumbered | Encumbrance::Burdened | Encumbrance::Stressed | Encumbrance::Strained),
            PickupBurden::Overtaxed => enc != Encumbrance::Overloaded,
            PickupBurden::Overloaded => true,
        }
    }
}

/// Check if an object's class matches the autopickup types string.
///
/// The autopickup_types string contains class symbols like "$?!/\"=".
///
/// # Arguments
/// * `obj` - The object to check
/// * `autopickup_types` - String of class symbols to autopickup
///
/// # Returns
/// True if the object's class is in the autopickup_types
pub fn matches_autopickup_type(obj: &Object, autopickup_types: &str) -> bool {
    let symbol = obj.class.symbol();
    autopickup_types.contains(symbol)
}

/// Check if an object should be autopicked up based on game settings.
///
/// # Arguments
/// * `obj` - The object to consider
/// * `autopickup_enabled` - Whether autopickup is enabled at all
/// * `autopickup_types` - String of class symbols to autopickup
/// * `nopick` - Temporary flag to suppress autopickup (e.g., while running)
///
/// # Returns
/// True if the object should be autopicked up
pub fn should_autopickup(
    obj: &Object,
    autopickup_enabled: bool,
    autopickup_types: &str,
    nopick: bool,
) -> bool {
    // Autopickup must be enabled
    if !autopickup_enabled {
        return false;
    }

    // Temporary suppress flag (e.g., while running/travelling)
    if nopick {
        return false;
    }

    // Object must be pickable
    if !matches!(obj.class, ObjectClass::Ball | ObjectClass::Chain) {
        // Check if class matches the autopickup types
        matches_autopickup_type(obj, autopickup_types)
    } else {
        false
    }
}

/// Check if picking up an object would exceed the pickup burden threshold.
///
/// # Arguments
/// * `obj` - The object to potentially pick up
/// * `state` - Current game state
/// * `pickup_burden` - Maximum allowed encumbrance for autopickup
///
/// # Returns
/// True if picking up would NOT exceed the burden threshold (safe to pick up)
pub fn within_pickup_burden(
    obj: &Object,
    state: &GameState,
    pickup_burden: PickupBurden,
) -> bool {
    // Calculate what encumbrance would be after picking up
    let current_weight: u32 = state.inventory.iter().map(|o| o.weight).sum();
    let new_weight = current_weight + obj.weight;

    // Get the capacity and calculate new encumbrance
    let capacity = state.player.carrying_capacity as u32;
    let new_encumbrance = calculate_encumbrance(new_weight, capacity);

    pickup_burden.allows_encumbrance(new_encumbrance)
}

/// Calculate encumbrance level from weight and capacity.
fn calculate_encumbrance(weight: u32, capacity: u32) -> Encumbrance {
    if capacity == 0 {
        return Encumbrance::Overloaded;
    }

    if weight <= capacity / 4 {
        Encumbrance::Unencumbered
    } else if weight <= capacity / 2 {
        Encumbrance::Burdened
    } else if weight <= (capacity * 3) / 4 {
        Encumbrance::Stressed
    } else if weight <= (capacity * 9) / 10 {
        Encumbrance::Strained
    } else if weight <= capacity {
        Encumbrance::Overtaxed
    } else {
        Encumbrance::Overloaded
    }
}

/// Perform autopickup at the player's current position.
///
/// This picks up items that match the autopickup settings, respecting
/// the burden threshold and temporary nopick flag.
///
/// # Arguments
/// * `state` - Game state
/// * `autopickup_types` - String of class symbols to autopickup (e.g., "$?!/\"=")
/// * `pickup_burden` - Maximum encumbrance level for autopickup
///
/// # Returns
/// List of messages describing what was picked up
pub fn do_autopickup(
    state: &mut GameState,
    autopickup_types: &str,
    pickup_burden: PickupBurden,
) -> Vec<String> {
    let mut messages = Vec::new();

    // Check if autopickup is enabled
    if !state.flags.autopickup {
        return messages;
    }

    // Check if temporarily suppressed
    if state.context.nopick {
        return messages;
    }

    let x = state.player.pos.x;
    let y = state.player.pos.y;

    // Get IDs of objects to potentially pick up
    let object_ids: Vec<_> = state.current_level.objects_at(x, y)
        .iter()
        .filter(|obj| {
            should_autopickup(obj, true, autopickup_types, false)
                && within_pickup_burden(obj, state, pickup_burden)
        })
        .map(|o| o.id)
        .collect();

    if object_ids.is_empty() {
        return messages;
    }

    let mut picked_up = Vec::new();

    // Pick up each qualifying object
    for id in object_ids {
        if let Some(obj) = state.current_level.remove_object(id) {
            // Re-check burden after each pickup (weight may have changed)
            if !within_pickup_burden(&obj, state, pickup_burden) {
                // Put it back - would exceed burden
                state.current_level.add_object(obj, x, y);
                if picked_up.is_empty() {
                    messages.push("You cannot carry any more.".to_string());
                }
                break;
            }

            let name = obj.display_name();
            state.add_to_inventory(obj);
            picked_up.push(name);
        }
    }

    // Generate pickup messages
    if !picked_up.is_empty() {
        if picked_up.len() == 1 {
            messages.push(format!("{} - {}.", state.inventory.last().map(|o| o.inv_letter).unwrap_or('?'), picked_up[0]));
        } else {
            messages.push(format!("You pick up {} items.", picked_up.len()));
        }
    }

    messages
}

/// Check if there are items to autopickup at the current position.
///
/// This is useful for checking before movement to know if autopickup
/// should trigger after the move.
///
/// # Arguments
/// * `state` - Game state
/// * `autopickup_types` - String of class symbols to autopickup
///
/// # Returns
/// True if there are items that would be autopicked up
pub fn has_autopickup_items(
    state: &GameState,
    autopickup_types: &str,
) -> bool {
    if !state.flags.autopickup || state.context.nopick {
        return false;
    }

    let x = state.player.pos.x;
    let y = state.player.pos.y;

    state.current_level.objects_at(x, y)
        .iter()
        .any(|obj| matches_autopickup_type(obj, autopickup_types))
}

/// Configure autopickup types from a string like "$?!/\"=".
///
/// # Common symbols:
/// * `$` - Gold (coins)
/// * `?` - Scrolls
/// * `!` - Potions
/// * `/` - Wands
/// * `"` - Amulets
/// * `=` - Rings
/// * `%` - Food
/// * `(` - Tools
/// * `)` - Weapons
/// * `[` - Armor
/// * `+` - Spellbooks
/// * `*` - Gems/rocks
///
/// # Returns
/// A validated string with only valid class symbols
pub fn parse_autopickup_types(types: &str) -> String {
    let valid_symbols: &str = "$?!/\"=%()+[*";
    types.chars().filter(|c| valid_symbols.contains(*c)).collect()
}

/// Default autopickup types (gold, scrolls, potions, wands, amulets, rings)
pub const DEFAULT_AUTOPICKUP_TYPES: &str = "$?!/\"=";

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
