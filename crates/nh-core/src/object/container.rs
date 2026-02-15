//! Container operations (pickup.c, invent.c)
//!
//! Functions for manipulating containers: looting, stashing, etc.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use super::{MkObjContext, Object, ObjectId};

/// Result of a container operation
#[derive(Debug, Clone)]
pub enum ContainerResult {
    /// Operation succeeded
    Success,
    /// Container is locked
    Locked,
    /// Container is trapped (trap type)
    Trapped(TrapType),
    /// Container is broken (cannot be used)
    Broken,
    /// Container is empty
    Empty,
    /// Not a container
    NotContainer,
}

/// Types of container traps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapType {
    /// Explodes, destroying contents
    Explosion,
    /// Paralyzes opener
    Paralysis,
    /// Poisons opener
    Poison,
    /// Teleports contents away
    Teleport,
    /// Summons monsters
    Summon,
}

/// Check if an object can be placed in a container.
/// Some objects cannot be placed in containers (e.g., other containers in some cases).
pub fn can_put_in_container(obj: &Object, container: &Object) -> bool {
    // Can't put a container inside itself
    if obj.id == container.id {
        return false;
    }

    // Can't put the Amulet of Yendor in containers (special rule)
    // Using object_type check - would need proper constant
    // For now, allow most items

    // Bag of Holding has special rules about other bags
    // BagOfHolding = 365
    if container.object_type == 365 {
        // Can't put another Bag of Holding in it (causes explosion)
        if obj.object_type == 365 {
            return false;
        }
        // Can't put a Bag of Tricks either
        if obj.object_type == 366 {
            return false;
        }
    }

    true
}

/// Calculate the weight of a container including its contents.
/// Bag of Holding reduces contained weight.
pub fn container_weight(container: &Object) -> u32 {
    let base_weight = container.weight;

    if container.contents.is_empty() {
        return base_weight;
    }

    let contents_weight: u32 = container.contents.iter().map(total_weight).sum();

    // Bag of Holding reduces weight
    // BagOfHolding = 365
    if container.object_type == 365 {
        // Blessed: weight / 4
        // Uncursed: weight / 2
        // Cursed: weight * 2 (makes things heavier!)
        let modified_weight = match container.buc {
            super::BucStatus::Blessed => contents_weight / 4,
            super::BucStatus::Uncursed => contents_weight / 2,
            super::BucStatus::Cursed => contents_weight * 2,
        };
        base_weight + modified_weight
    } else {
        base_weight + contents_weight
    }
}

/// Calculate total weight of an object, including contents if it's a container.
pub fn total_weight(obj: &Object) -> u32 {
    if obj.is_container() {
        container_weight(obj)
    } else {
        obj.weight * obj.quantity as u32
    }
}

/// Put an object into a container.
///
/// # Arguments
/// * `container` - The container to put the object into
/// * `obj` - The object to put in (will be moved)
///
/// # Returns
/// Whether the operation succeeded
pub fn put_in_container(container: &mut Object, obj: Object) -> ContainerResult {
    if !container.is_container() {
        return ContainerResult::NotContainer;
    }

    if container.locked {
        return ContainerResult::Locked;
    }

    if container.broken {
        return ContainerResult::Broken;
    }

    if !can_put_in_container(&obj, container) {
        return ContainerResult::NotContainer; // Reusing for "can't put"
    }

    // Try to merge with existing item
    for existing in container.contents.iter_mut() {
        if existing.can_merge(&obj) {
            existing.merge(obj);
            return ContainerResult::Success;
        }
    }

    // Add as new item
    container.contents.push(obj);
    ContainerResult::Success
}

/// Remove an object from a container by index.
///
/// # Arguments
/// * `container` - The container to remove from
/// * `index` - Index of the item to remove
///
/// # Returns
/// The removed object, or None if index is invalid
pub fn take_from_container(container: &mut Object, index: usize) -> Option<Object> {
    if !container.is_container() {
        return None;
    }

    if index >= container.contents.len() {
        return None;
    }

    Some(container.contents.remove(index))
}

/// Remove a specific quantity from a stacked item in a container.
///
/// # Arguments
/// * `container` - The container
/// * `index` - Index of the item
/// * `quantity` - How many to take
/// * `ctx` - Object creation context for generating new ID
///
/// # Returns
/// The split-off object, or None if invalid
pub fn take_quantity_from_container(
    container: &mut Object,
    index: usize,
    quantity: i32,
    ctx: &mut MkObjContext,
) -> Option<Object> {
    if !container.is_container() {
        return None;
    }

    if index >= container.contents.len() {
        return None;
    }

    let item = &mut container.contents[index];

    if quantity >= item.quantity {
        // Take the whole stack
        Some(container.contents.remove(index))
    } else if quantity > 0 {
        // Split the stack
        let mut taken = item.clone();
        taken.id = ctx.next_id();
        taken.quantity = quantity;
        item.quantity -= quantity;
        Some(taken)
    } else {
        None
    }
}

/// Find an item in a container by predicate.
pub fn find_in_container<F>(container: &Object, predicate: F) -> Option<usize>
where
    F: Fn(&Object) -> bool,
{
    container.contents.iter().position(predicate)
}

/// Find an item in a container by ID.
pub fn find_by_id(container: &Object, id: ObjectId) -> Option<usize> {
    find_in_container(container, |obj| obj.id == id)
}

/// Open a container, checking for traps.
///
/// # Arguments
/// * `container` - The container to open
///
/// # Returns
/// Result indicating success or what went wrong
pub fn open_container(container: &mut Object) -> ContainerResult {
    if !container.is_container() {
        return ContainerResult::NotContainer;
    }

    if container.broken {
        return ContainerResult::Broken;
    }

    if container.locked {
        return ContainerResult::Locked;
    }

    // Check for traps
    if container.trapped {
        container.trapped = false; // Trap is triggered, no longer trapped
        // Determine trap type based on some factor
        // For simplicity, return explosion trap
        return ContainerResult::Trapped(TrapType::Explosion);
    }

    if container.contents.is_empty() {
        return ContainerResult::Empty;
    }

    ContainerResult::Success
}

/// Lock a container (requires key/lock pick).
pub fn lock_container(container: &mut Object) {
    if container.is_container() && !container.broken {
        container.locked = true;
    }
}

/// Unlock a container (requires key/lock pick).
pub fn unlock_container(container: &mut Object) {
    if container.is_container() {
        container.locked = false;
    }
}

/// Force open a locked container (may break it).
///
/// # Returns
/// true if successfully forced, false if failed
pub fn force_container(container: &mut Object, success_chance: u32, roll: u32) -> bool {
    if !container.is_container() || !container.locked {
        return false;
    }

    if roll < success_chance {
        container.locked = false;
        true
    } else {
        // Failed to force - may break lock
        container.broken = true;
        false
    }
}

/// Get a list of container contents for display.
pub fn list_contents(container: &Object) -> Vec<(usize, String)> {
    container
        .contents
        .iter()
        .enumerate()
        .map(|(i, obj)| {
            let name = obj.name.as_deref().unwrap_or("item");
            let display = if obj.quantity > 1 {
                format!("{} {}", obj.quantity, name)
            } else {
                name.to_string()
            };
            (i, display)
        })
        .collect()
}

/// Count items in a container.
pub fn count_contents(container: &Object) -> usize {
    container.contents.len()
}

/// Count total quantity of items in a container (counting stacks).
pub fn count_total_items(container: &Object) -> i32 {
    container.contents.iter().map(|obj| obj.quantity).sum()
}

/// Empty a container, returning all contents.
pub fn empty_container(container: &mut Object) -> Vec<Object> {
    core::mem::take(&mut container.contents)
}

// ============================================================================
// Container UI and validation functions (Phase 4)
// ============================================================================

/// Check if a Bag of Holding would explode (ck_bag equivalent)
///
/// Bag of Holding explodes if you try to put another Bag of Holding or
/// Bag of Tricks inside it.
///
/// # Returns
/// true if attempting to put an invalid bag inside this one would cause explosion
pub fn ck_bag(container: &Object) -> bool {
    if container.object_type != 365 {
        // Not a Bag of Holding
        return false;
    }

    // Check contents for nested Bag of Holding or Bag of Tricks
    for item in &container.contents {
        if item.object_type == 365 || item.object_type == 366 {
            // BagOfHolding or BagOfTricks found inside
            return true;
        }
    }

    false
}

/// Interactive container use (use_container equivalent)
///
/// Shows container contents in a menu and allows player to select items.
/// This is the main UI function for loot/stash operations.
///
/// Returns formatted string showing container contents ready for interaction.
pub fn use_container(container: &Object) -> String {
    if !container.is_container() {
        return "That is not a container.".to_string();
    }

    if container.locked {
        return "That container is locked.".to_string();
    }

    if container.trapped {
        return "That container is trapped!".to_string();
    }

    if container.contents.is_empty() {
        return "This container is empty.".to_string();
    }

    let mut result = String::from("Container contents:\n");
    for (idx, item) in container.contents.iter().enumerate() {
        let letter = ('a' as u8 + idx as u8) as char;
        let name = format_container_item(item);
        result.push_str(&format!("  {} - {}\n", letter, name));
    }

    result
}

/// Tip container contents onto the ground (tipcontainer equivalent)
///
/// Removes all items from container and returns them.
/// Used when spilling a container's contents.
pub fn tipcontainer(container: &mut Object) -> Vec<Object> {
    container.contents.drain(..).collect()
}

/// Display container contents in a list (in_container equivalent)
///
/// Generates a formatted string of all items in the container.
pub fn in_container(container: &Object) -> String {
    if container.contents.is_empty() {
        return "The container is empty.".to_string();
    }

    let mut result = String::new();
    for item in &container.contents {
        let name = format_container_item(item);
        result.push_str(&format!("  {}\n", name));
    }

    result
}

/// Check if a container is in use (locked or being accessed)
pub fn is_container_in_use(container: &Object) -> bool {
    container.locked || container.trapped
}

/// Validate container state and return error if invalid
pub fn validate_container(container: &Object) -> ContainerResult {
    if !container.is_container() {
        return ContainerResult::NotContainer;
    }

    if container.broken {
        return ContainerResult::Broken;
    }

    if container.locked {
        return ContainerResult::Locked;
    }

    if container.trapped {
        return ContainerResult::Trapped(TrapType::Explosion); // Default trap
    }

    ContainerResult::Success
}

// Helper function to format container item names
fn format_container_item(item: &Object) -> String {
    let mut name = String::new();

    if item.quantity > 1 {
        name.push_str(&format!("{} ", item.quantity));
    }

    // Add basic description
    name.push_str("item");

    if item.is_worn() {
        name.push_str(" (worn)");
    }

    if item.greased {
        name.push_str(" (greased)");
    }

    name
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{BucStatus, ObjectClass};

    fn make_container() -> Object {
        let mut container = Object::default();
        container.id = ObjectId(1);
        container.object_type = 361; // Chest
        container.class = ObjectClass::Tool;
        container.weight = 350;
        container
    }

    fn make_item() -> Object {
        let mut item = Object::default();
        item.id = ObjectId(100);
        item.object_type = 1;
        item.class = ObjectClass::Weapon;
        item.weight = 10;
        item.quantity = 1;
        item
    }

    #[test]
    fn test_is_container() {
        let container = make_container();
        assert!(container.is_container());

        let item = make_item();
        assert!(!item.is_container());
    }

    #[test]
    fn test_put_in_container() {
        let mut container = make_container();
        let item = make_item();

        let result = put_in_container(&mut container, item);
        assert!(matches!(result, ContainerResult::Success));
        assert_eq!(container.contents.len(), 1);
    }

    #[test]
    fn test_take_from_container() {
        let mut container = make_container();
        let item = make_item();
        put_in_container(&mut container, item);

        let taken = take_from_container(&mut container, 0);
        assert!(taken.is_some());
        assert_eq!(container.contents.len(), 0);
    }

    #[test]
    fn test_locked_container() {
        let mut container = make_container();
        container.locked = true;

        let item = make_item();
        let result = put_in_container(&mut container, item);
        assert!(matches!(result, ContainerResult::Locked));
    }

    #[test]
    fn test_container_weight() {
        let mut container = make_container();
        let mut item1 = make_item();
        item1.weight = 100;
        let mut item2 = make_item();
        item2.id = ObjectId(101);
        item2.object_type = 2; // Different type to prevent merging
        item2.weight = 50;

        put_in_container(&mut container, item1);
        put_in_container(&mut container, item2);

        // Base weight (350) + item weights (100 + 50) = 500
        assert_eq!(container_weight(&container), 500);
    }

    #[test]
    fn test_bag_of_holding_weight() {
        let mut bag = Object::default();
        bag.id = ObjectId(1);
        bag.object_type = 365; // Bag of Holding
        bag.class = ObjectClass::Tool;
        bag.weight = 15;
        bag.buc = BucStatus::Uncursed;

        let mut item = make_item();
        item.weight = 100;
        put_in_container(&mut bag, item);

        // Base (15) + contents (100) / 2 = 65
        assert_eq!(container_weight(&bag), 65);

        // Blessed reduces more
        bag.buc = BucStatus::Blessed;
        // Base (15) + contents (100) / 4 = 40
        assert_eq!(container_weight(&bag), 40);

        // Cursed increases!
        bag.buc = BucStatus::Cursed;
        // Base (15) + contents (100) * 2 = 215
        assert_eq!(container_weight(&bag), 215);
    }

    #[test]
    fn test_empty_container() {
        let mut container = make_container();
        let item1 = make_item();
        let mut item2 = make_item();
        item2.id = ObjectId(101);
        item2.object_type = 2; // Different type to prevent merging
        put_in_container(&mut container, item1);
        put_in_container(&mut container, item2);

        let contents = empty_container(&mut container);
        assert_eq!(contents.len(), 2);
        assert!(container.contents.is_empty());
    }

    #[test]
    fn test_trapped_container() {
        let mut container = make_container();
        container.trapped = true;

        let result = open_container(&mut container);
        assert!(matches!(result, ContainerResult::Trapped(_)));
        assert!(!container.trapped); // Trap was triggered
    }

    // ========================================================================
    // Phase 4 Tests: Container UI and Bag of Holding
    // ========================================================================

    #[test]
    fn test_ck_bag_safe() {
        let mut bag = make_container();
        bag.object_type = 365; // Bag of Holding

        // Safe contents
        let item = make_item();
        put_in_container(&mut bag, item);

        assert!(!ck_bag(&bag)); // No explosion
    }

    #[test]
    fn test_ck_bag_explodes_with_bag_of_holding() {
        let mut bag = make_container();
        bag.object_type = 365; // Bag of Holding

        let mut item = make_item();
        item.object_type = 365; // Another Bag of Holding
        // Directly insert (put_in_container rejects this combo)
        bag.contents.push(item);

        assert!(ck_bag(&bag)); // Would explode!
    }

    #[test]
    fn test_ck_bag_explodes_with_bag_of_tricks() {
        let mut bag = make_container();
        bag.object_type = 365; // Bag of Holding

        let mut item = make_item();
        item.object_type = 366; // Bag of Tricks
        // Directly insert (put_in_container rejects this combo)
        bag.contents.push(item);

        assert!(ck_bag(&bag)); // Would explode!
    }

    #[test]
    fn test_use_container_empty() {
        let container = make_container();
        let result = use_container(&container);
        assert!(result.contains("empty"));
    }

    #[test]
    fn test_use_container_locked() {
        let mut container = make_container();
        container.locked = true;

        let result = use_container(&container);
        assert!(result.contains("locked"));
    }

    #[test]
    fn test_use_container_with_items() {
        let mut container = make_container();
        let item = make_item();
        put_in_container(&mut container, item);

        let result = use_container(&container);
        assert!(result.contains("contents"));
        assert!(result.contains("a"));
    }

    #[test]
    fn test_tipcontainer() {
        let mut container = make_container();
        let item1 = make_item();
        let mut item2 = make_item();
        item2.id = ObjectId(101);
        item2.object_type = 2; // Different type so items don't merge
        put_in_container(&mut container, item1);
        put_in_container(&mut container, item2);

        let contents = tipcontainer(&mut container);
        assert_eq!(contents.len(), 2);
        assert!(container.contents.is_empty());
    }

    #[test]
    fn test_in_container() {
        let mut container = make_container();
        let item = make_item();
        put_in_container(&mut container, item);

        let result = in_container(&container);
        assert!(result.contains("item"));
    }

    #[test]
    fn test_validate_container_success() {
        let container = make_container();
        let result = validate_container(&container);
        assert!(matches!(result, ContainerResult::Success));
    }

    #[test]
    fn test_validate_container_broken() {
        let mut container = make_container();
        container.broken = true;

        let result = validate_container(&container);
        assert!(matches!(result, ContainerResult::Broken));
    }

    #[test]
    fn test_is_container_in_use() {
        let mut container = make_container();
        assert!(!is_container_in_use(&container));

        container.locked = true;
        assert!(is_container_in_use(&container));

        container.locked = false;
        container.trapped = true;
        assert!(is_container_in_use(&container));
    }
}
