//! Picking up and dropping items (pickup.c)

#[cfg(not(feature = "std"))]
use crate::compat::*;

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
pub fn can_pickup(obj: &Object, state: &GameState) -> bool {
    match obj.class {
        ObjectClass::Ball | ObjectClass::Chain => return false, // Punishment items
        _ => {}
    }

    // Cockatrice corpse without gloves -> petrification risk
    if obj.class == ObjectClass::Food && obj.corpse_type == 10 /* PM_COCKATRICE */ {
        let wearing_gloves = state.inventory.iter().any(|o| o.worn_mask & crate::action::wear::worn_mask::W_ARMG != 0);
        if !wearing_gloves && !state.player.properties.has(crate::player::Property::StoneResistance) {
            return false;
        }
    }

    // Loadstones are cursed and cannot be dropped once picked up;
    // warn the player by refusing pickup if the stone is known-cursed.
    if obj.class == ObjectClass::Gem && obj.is_cursed() && obj.buc_known {
        return false;
    }

    true
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
    let object_ids: Vec<_> = state
        .current_level
        .objects_at(x, y)
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
            // Gold goes to player.gold, not inventory (C: dopickup / pickup_object)
            if obj.class == crate::object::ObjectClass::Coin {
                let amount = obj.quantity;
                state.player.gold += amount;
                picked_up.push(format!(
                    "{} gold piece{}",
                    amount,
                    if amount == 1 { "" } else { "s" }
                ));
            } else {
                let name = obj.display_name();
                state.add_to_inventory(obj);
                picked_up.push(name);
            }
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

pub fn dopickup(state: &mut GameState) -> ActionResult {
    do_pickup(state)
}

pub fn pickup(state: &mut GameState) {
    do_pickup(state);
}

pub fn pickup_object(state: &mut GameState, obj: Object) {
    state.add_to_inventory(obj);
}

pub fn pickup_checks() -> bool {
    true
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

pub fn dodrop(state: &mut GameState, obj_letter: char) -> ActionResult {
    do_drop(state, obj_letter)
}

pub fn drop(state: &mut GameState, obj: Object) {
    let x = state.player.pos.x;
    let y = state.player.pos.y;
    state.current_level.add_object(obj, x, y);
}

pub fn dropp(state: &mut GameState, obj_letter: char) {
    do_drop(state, obj_letter);
}

/// Get list of inventory items that can be dropped
pub fn droppables(state: &GameState) -> Vec<char> {
    state
        .inventory
        .iter()
        .filter(|obj| obj.worn_mask == 0 && !obj.is_wielded())
        .map(|obj| obj.inv_letter)
        .collect()
}

pub fn drop_to(state: &mut GameState, obj: Object, x: i8, y: i8) {
    state.current_level.add_object(obj, x, y);
}

pub fn drop_upon_death(state: &mut GameState) {
    // Drop all inventory
    while let Some(obj) = state.inventory.pop() {
        drop(state, obj);
    }
}

pub fn dropx(state: &mut GameState, obj: Object) {
    drop(state, obj);
}

pub fn dropy(state: &mut GameState, obj: Object) {
    drop(state, obj);
}

pub fn dropz(state: &mut GameState, obj: Object, with_message: bool) {
    if with_message {
        state.message(format!("You drop {}.", obj.display_name()));
    }
    drop(state, obj);
}

pub fn menu_drop() {
    // Stub
}

pub fn doddrop() {
    // Stub: Drop multiple
}

// ============================================================================
// Container Looting (doloot from NetHack)
// ============================================================================

/// Loot items from a container in the player's inventory
pub fn do_loot_container(state: &mut GameState, obj_letter: char) -> ActionResult {
    // Get container from inventory
    let container = match state.get_inventory_item(obj_letter) {
        Some(o) => o,
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    // Check if it's actually a container (Tool class with contents)
    if !matches!(container.class, ObjectClass::Tool) {
        return ActionResult::Failed("That's not a container.".to_string());
    }

    // For now, show what's inside
    if container.contents.is_empty() {
        state.message(format!("The {} is empty.", container.display_name()));
        return ActionResult::NoTime;
    }

    // List contents
    let contents_str = container
        .contents
        .iter()
        .map(|obj| obj.display_name())
        .collect::<Vec<_>>()
        .join(", ");

    state.message(format!(
        "The {} contains: {}",
        container.display_name(),
        contents_str
    ));

    ActionResult::Success
}

pub fn doloot(state: &mut GameState, obj_letter: char) -> ActionResult {
    do_loot_container(state, obj_letter)
}

pub fn able_to_loot() -> bool {
    true
}

pub fn do_loot_cont() {
    // Stub
}

pub fn loot_mon() {
    // Stub
}

pub fn loot_xname() -> String {
    "item".to_string()
}

pub fn menu_loot() {
    // Stub
}

pub fn traditional_loot() {
    // Stub
}

pub fn reverse_loot() {
    // Stub
}

/// Put an item from inventory into a container
pub fn in_container(
    state: &mut GameState,
    container_letter: char,
    item_letter: char,
) -> ActionResult {
    // Get the item
    let item_idx = state
        .inventory
        .iter()
        .position(|o| o.inv_letter == item_letter);

    let item_idx = match item_idx {
        Some(i) => i,
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    // Check if item is worn/wielded
    if state.inventory[item_idx].worn_mask != 0 || state.inventory[item_idx].is_wielded() {
        return ActionResult::Failed("You'll have to take it off first.".to_string());
    }

    // Get the container
    let container_idx = state
        .inventory
        .iter()
        .position(|o| o.inv_letter == container_letter);

    let container_idx = match container_idx {
        Some(i) => i,
        None => return ActionResult::Failed("You don't have that container.".to_string()),
    };

    // Can't put container in itself
    if container_idx == item_idx {
        return ActionResult::Failed("You can't put something inside itself.".to_string());
    }

    // Remove item and add to container
    let item = state.inventory.remove(item_idx);
    let item_name = item.display_name();

    // Need to re-find container index after removal
    let container_idx = state
        .inventory
        .iter()
        .position(|o| o.inv_letter == container_letter)
        .unwrap();

    let container = &mut state.inventory[container_idx];
    let container_name = container.display_name();
    container.contents.push(item);

    state.message(format!(
        "You put the {} in the {}.",
        item_name, container_name
    ));
    ActionResult::Success
}

/// Take an item out of a container into inventory
pub fn out_container(
    state: &mut GameState,
    container_letter: char,
    item_index: usize,
) -> ActionResult {
    extract_from_container(state, container_letter, item_index)
}

pub fn explain_container_prompt() {
    // Stub
}

pub fn tipcontainer() {
    // Stub
}

pub fn picked_container() {
    // Stub
}

pub fn dropped_container() {
    // Stub
}

pub fn container_at(state: &GameState, x: i8, y: i8) -> bool {
    state
        .current_level
        .objects_at(x, y)
        .iter()
        .any(|o| matches!(o.class, ObjectClass::Tool))
}

pub fn container_gone() {
    // Stub
}

pub fn container_impact_dmg() {
    // Stub
}

pub fn get_container_location() {
    // Stub
}

pub fn unknwn_contnr_contents() {
    // Stub
}

pub fn contained_stats() {
    // Stub
}

/// Count items in a container
pub fn count_contents(container: &Object) -> usize {
    container.contents.len()
}

/// Extract an item from a container
pub fn extract_from_container(
    state: &mut GameState,
    container_letter: char,
    item_index: usize,
) -> ActionResult {
    let container = match state.get_inventory_item_mut(container_letter) {
        Some(o) => o,
        None => return ActionResult::Failed("You don't have that container.".to_string()),
    };

    if item_index >= container.contents.len() {
        return ActionResult::Failed("That item is not in the container.".to_string());
    }

    // Extract item and add to inventory
    let extracted = container.contents.remove(item_index);
    let item_name = extracted.display_name();

    state.add_to_inventory(extracted);
    state.message(format!("You extract the {} from the container.", item_name));

    ActionResult::Success
}

// ============================================================================
// Autopickup system
// ============================================================================

pub fn autopick() {
    // Stub
}

pub fn autopick_testobj() {
    // Stub
}

pub fn autoquiver() {
    // Stub
}

pub fn check_autopickup_exceptions() {
    // Stub
}

pub fn add_autopickup_exception() {
    // Stub
}

pub fn remove_autopickup_exception() {
    // Stub
}

pub fn free_autopickup_exceptions() {
    // Stub
}

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
            PickupBurden::Burdened => {
                matches!(enc, Encumbrance::Unencumbered | Encumbrance::Burdened)
            }
            PickupBurden::Stressed => matches!(
                enc,
                Encumbrance::Unencumbered | Encumbrance::Burdened | Encumbrance::Stressed
            ),
            PickupBurden::Strained => matches!(
                enc,
                Encumbrance::Unencumbered
                    | Encumbrance::Burdened
                    | Encumbrance::Stressed
                    | Encumbrance::Strained
            ),
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
pub fn within_pickup_burden(obj: &Object, state: &GameState, pickup_burden: PickupBurden) -> bool {
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
    let object_ids: Vec<_> = state
        .current_level
        .objects_at(x, y)
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
            messages.push(format!(
                "{} - {}.",
                state.inventory.last().map(|o| o.inv_letter).unwrap_or('?'),
                picked_up[0]
            ));
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
pub fn has_autopickup_items(state: &GameState, autopickup_types: &str) -> bool {
    if !state.flags.autopickup || state.context.nopick {
        return false;
    }

    let x = state.player.pos.x;
    let y = state.player.pos.y;

    state
        .current_level
        .objects_at(x, y)
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
    types
        .chars()
        .filter(|c| valid_symbols.contains(*c))
        .collect()
}

/// Default autopickup types (gold, scrolls, potions, wands, amulets, rings)
pub const DEFAULT_AUTOPICKUP_TYPES: &str = "$?!/\"=";

pub fn can_reach_floor() -> bool {
    true
}

pub fn could_reach_item() -> bool {
    true
}

pub fn collect_obj_classes() {
    // Stub
}

pub fn add_valid_menu_class() {
    // Stub
}

pub fn allow_all() {
    // Stub
}

pub fn allow_cat_no_uchain() {
    // Stub
}

pub fn allow_category() {
    // Stub
}

pub fn ckvalidcat() -> bool {
    true
}

pub fn query_category() {
    // Stub
}

pub fn query_objlist() {
    // Stub
}

pub fn askchain() {
    // Stub
}

pub fn ggetobj() {
    // Stub
}

pub fn getobj() {
    // Stub
}

pub fn oselect() {
    // Stub
}

pub fn this_type_only() {
    // Stub
}

pub fn n_or_more() {
    // Stub
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
