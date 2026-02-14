//! Inventory management (invent.c)
//!
//! Functions for managing player inventory.

use crate::object::{Object, ObjectClass, ObjectId};

/// Maximum number of inventory slots (a-z, A-Z)
pub const MAX_INVENTORY_SLOTS: usize = 52;

/// Gold symbol for inventory
pub const GOLD_SYM: char = '$';

/// No inventory symbol (overflow)
pub const NOINVSYM: char = '#';

/// Inventory letter order (lowercase before uppercase)
#[allow(dead_code)]
const INV_ORDER: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Assign an inventory letter to an object
pub fn assign_invlet(obj: &mut Object, inventory: &[Object]) {
    // Gold always gets the gold symbol
    if obj.class == ObjectClass::Coin {
        obj.inv_letter = GOLD_SYM;
        return;
    }

    // Track which letters are in use
    let mut in_use = [false; 52];
    for item in inventory {
        if item.id != obj.id {
            let c = item.inv_letter;
            if c.is_ascii_lowercase() {
                in_use[(c as usize) - ('a' as usize)] = true;
            } else if c.is_ascii_uppercase() {
                in_use[(c as usize) - ('A' as usize) + 26] = true;
            }
        }
    }

    // If object already has a valid letter that's not in use, keep it
    let c = obj.inv_letter;
    if c.is_ascii_lowercase() || c.is_ascii_uppercase() {
        let idx = if c.is_ascii_lowercase() {
            (c as usize) - ('a' as usize)
        } else {
            (c as usize) - ('A' as usize) + 26
        };
        if !in_use[idx] {
            return;
        }
    }

    // Find first available letter
    for (i, &used) in in_use.iter().enumerate() {
        if !used {
            obj.inv_letter = if i < 26 {
                (b'a' + i as u8) as char
            } else {
                (b'A' + (i - 26) as u8) as char
            };
            return;
        }
    }

    // No letter available, use overflow symbol
    obj.inv_letter = NOINVSYM;
}

/// Find an object in inventory by its letter
pub fn find_by_letter(inventory: &[Object], letter: char) -> Option<usize> {
    inventory.iter().position(|obj| obj.inv_letter == letter)
}

/// Find an object in inventory by its ID
pub fn find_by_id(inventory: &[Object], id: ObjectId) -> Option<usize> {
    inventory.iter().position(|obj| obj.id == id)
}

/// Check if inventory is full (all 52 slots used)
pub fn is_full(inventory: &[Object]) -> bool {
    // Count unique letters in use
    let mut count = 0;
    for obj in inventory {
        let c = obj.inv_letter;
        if c.is_ascii_lowercase() || c.is_ascii_uppercase() {
            count += 1;
        }
    }
    count >= MAX_INVENTORY_SLOTS
}

/// Get the number of items in inventory (counting stacks as 1)
pub fn slot_count(inventory: &[Object]) -> usize {
    inventory.len()
}

/// Get the total number of items in inventory (counting quantities)
pub fn item_count(inventory: &[Object]) -> i32 {
    inventory.iter().map(|obj| obj.quantity).sum()
}

/// Get total weight of inventory (weight × quantity for each item)
pub fn total_weight(inventory: &[Object]) -> u32 {
    inventory
        .iter()
        .map(|obj| obj.weight * obj.quantity.max(1) as u32)
        .sum()
}

/// Get total gold in inventory
pub fn gold_count(inventory: &[Object]) -> i32 {
    inventory
        .iter()
        .filter(|obj| obj.class == ObjectClass::Coin)
        .map(|obj| obj.quantity)
        .sum()
}

/// Find an object that can merge with the given object
pub fn find_mergeable(inventory: &[Object], obj: &Object) -> Option<usize> {
    inventory.iter().position(|item| item.can_merge(obj))
}

/// Add an object to inventory, merging if possible
/// Returns the index where the object was added/merged
pub fn add_to_inventory(inventory: &mut Vec<Object>, mut obj: Object) -> usize {
    // Try to merge with existing item
    if let Some(idx) = find_mergeable(inventory, &obj) {
        inventory[idx].merge(obj);
        return idx;
    }

    // Assign inventory letter
    assign_invlet(&mut obj, inventory);

    // Add to inventory
    inventory.push(obj);
    let idx = inventory.len() - 1;

    // Sort inventory by letter
    sort_inventory(inventory);

    // Find new position after sort
    find_by_id(inventory, inventory[idx].id).unwrap_or(idx)
}

/// Remove an object from inventory by index
pub fn remove_from_inventory(inventory: &mut Vec<Object>, index: usize) -> Option<Object> {
    if index < inventory.len() {
        Some(inventory.remove(index))
    } else {
        None
    }
}

/// Remove an object from inventory by letter
pub fn remove_by_letter(inventory: &mut Vec<Object>, letter: char) -> Option<Object> {
    find_by_letter(inventory, letter).map(|idx| inventory.remove(idx))
}

/// Sort inventory by letter (lowercase before uppercase)
pub fn sort_inventory(inventory: &mut [Object]) {
    inventory.sort_by(|a, b| {
        let rank_a = inv_rank(a.inv_letter);
        let rank_b = inv_rank(b.inv_letter);
        rank_a.cmp(&rank_b)
    });
}

/// Get inventory rank for sorting (lowercase before uppercase)
fn inv_rank(c: char) -> u8 {
    match c {
        'a'..='z' => (c as u8) - b'a',
        'A'..='Z' => (c as u8) - b'A' + 26,
        '$' => 52, // Gold at end
        '#' => 53, // Overflow at very end
        _ => 54,
    }
}

/// Get objects of a specific class
pub fn objects_of_class(inventory: &[Object], class: ObjectClass) -> Vec<&Object> {
    inventory.iter().filter(|obj| obj.class == class).collect()
}

/// Get worn/wielded objects
pub fn worn_objects(inventory: &[Object]) -> Vec<&Object> {
    inventory.iter().filter(|obj| obj.worn_mask != 0).collect()
}

/// Check if carrying an object of a specific type
pub fn carrying(inventory: &[Object], object_type: i16) -> bool {
    inventory.iter().any(|obj| obj.object_type == object_type)
}

/// Count objects of a specific type
pub fn count_type(inventory: &[Object], object_type: i16) -> i32 {
    inventory
        .iter()
        .filter(|obj| obj.object_type == object_type)
        .map(|obj| obj.quantity)
        .sum()
}

/// Get a summary of inventory by class
pub fn inventory_summary(inventory: &[Object]) -> Vec<(ObjectClass, usize, i32)> {
    let mut summary: Vec<(ObjectClass, usize, i32)> = Vec::new();

    for obj in inventory {
        if let Some(entry) = summary.iter_mut().find(|(c, _, _)| *c == obj.class) {
            entry.1 += 1;
            entry.2 += obj.quantity;
        } else {
            summary.push((obj.class, 1, obj.quantity));
        }
    }

    summary
}

/// Default inventory order for display
pub const DEFAULT_INV_ORDER: &[ObjectClass] = &[
    ObjectClass::Coin,
    ObjectClass::Amulet,
    ObjectClass::Ring,
    ObjectClass::Wand,
    ObjectClass::Potion,
    ObjectClass::Scroll,
    ObjectClass::Spellbook,
    ObjectClass::Gem,
    ObjectClass::Food,
    ObjectClass::Tool,
    ObjectClass::Weapon,
    ObjectClass::Armor,
    ObjectClass::Rock,
    ObjectClass::Ball,
    ObjectClass::Chain,
];

/// Sort inventory by class order (for display)
pub fn sort_by_class(inventory: &mut [Object], class_order: &[ObjectClass]) {
    inventory.sort_by(|a, b| {
        let pos_a = class_order
            .iter()
            .position(|&c| c == a.class)
            .unwrap_or(usize::MAX);
        let pos_b = class_order
            .iter()
            .position(|&c| c == b.class)
            .unwrap_or(usize::MAX);

        match pos_a.cmp(&pos_b) {
            std::cmp::Ordering::Equal => {
                // Within same class, sort by letter
                inv_rank(a.inv_letter).cmp(&inv_rank(b.inv_letter))
            }
            other => other,
        }
    });
}

// ============================================================================
// Item selection / filtering (C: getobj)
// ============================================================================

/// Filter specification for selecting items from inventory.
///
/// This is the Rust equivalent of C NetHack's `getobj()` filter string.
/// Instead of a char-based filter string with "ugly checks", we use a
/// type-safe struct with explicit fields.
#[derive(Debug, Clone)]
pub struct ItemFilter {
    /// Allowed object classes (empty = all classes allowed)
    pub classes: Vec<ObjectClass>,
    /// Allow gold/coins to be selected
    pub allow_gold: bool,
    /// Allow selecting "nothing" (e.g., wielding bare hands)
    pub allow_none: bool,
    /// Allow quantity selection (for partial stacks)
    pub allow_count: bool,
    /// Action-specific filter: if set, items must satisfy this predicate
    /// in addition to class matching
    pub action_filter: Option<ActionFilter>,
}

/// Action-specific filtering rules (C: "ugly checks")
///
/// Each variant encodes the rules that C's getobj applies based on the
/// `word` parameter (e.g., "eat", "drink", "wield").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionFilter {
    /// For eating: item must be food class
    Eat,
    /// For drinking: item must be potion class
    Drink,
    /// For reading: scrolls and spellbooks
    Read,
    /// For zapping: wands only
    Zap,
    /// For wielding: weapons and wep-tools, exclude currently worn
    Wield,
    /// For wearing: armor only, exclude already-worn
    Wear,
    /// For putting on: rings, amulets, and accessory tools
    PutOn,
    /// For taking off: must be currently worn
    TakeOff,
    /// For throwing: weapons, gems, tools
    Throw,
    /// For applying: applicable tools
    Apply,
    /// For dipping: exclude inaccessible items
    Dip,
    /// For invoking: artifacts and special items only
    Invoke,
    /// For rubbing: lamps and gemstones
    Rub,
    /// For tipping: containers only
    Tip,
}

impl ItemFilter {
    /// Create a filter that accepts only the given classes
    pub fn classes(classes: &[ObjectClass]) -> Self {
        Self {
            classes: classes.to_vec(),
            allow_gold: false,
            allow_none: false,
            allow_count: false,
            action_filter: None,
        }
    }

    /// Create a filter that accepts all classes
    pub fn all() -> Self {
        Self {
            classes: Vec::new(),
            allow_gold: false,
            allow_none: false,
            allow_count: false,
            action_filter: None,
        }
    }

    /// Builder: set allow_gold
    pub fn with_gold(mut self) -> Self {
        self.allow_gold = true;
        self
    }

    /// Builder: set allow_none
    pub fn with_none(mut self) -> Self {
        self.allow_none = true;
        self
    }

    /// Builder: set allow_count
    pub fn with_count(mut self) -> Self {
        self.allow_count = true;
        self
    }

    /// Builder: set action filter
    pub fn with_action(mut self, action: ActionFilter) -> Self {
        self.action_filter = Some(action);
        self
    }

    /// Check if an object passes this filter
    pub fn matches(&self, obj: &Object) -> bool {
        // Gold check
        if obj.class == ObjectClass::Coin && !self.allow_gold {
            return false;
        }

        // Class check (empty classes = all allowed)
        if !self.classes.is_empty() && !self.classes.contains(&obj.class) {
            return false;
        }

        // Action-specific checks
        if self
            .action_filter
            .is_some_and(|action| !action_matches(action, obj))
        {
            return false;
        }

        true
    }
}

/// Apply action-specific filtering rules (C: "ugly checks")
fn action_matches(action: ActionFilter, obj: &Object) -> bool {
    match action {
        ActionFilter::Eat => obj.class == ObjectClass::Food,
        ActionFilter::Drink => obj.class == ObjectClass::Potion,
        ActionFilter::Read => {
            matches!(obj.class, ObjectClass::Scroll | ObjectClass::Spellbook)
        }
        ActionFilter::Zap => obj.class == ObjectClass::Wand,
        ActionFilter::Wield => {
            // Weapons are always wieldable; some tools are wep-tools
            matches!(obj.class, ObjectClass::Weapon | ObjectClass::Tool)
        }
        ActionFilter::Wear => {
            // Must be armor and not already worn
            obj.class == ObjectClass::Armor && obj.worn_mask == 0
        }
        ActionFilter::PutOn => {
            // Rings, amulets, and accessory tools; not already worn
            matches!(
                obj.class,
                ObjectClass::Ring | ObjectClass::Amulet | ObjectClass::Tool
            ) && obj.worn_mask == 0
        }
        ActionFilter::TakeOff => {
            // Must be currently worn
            obj.worn_mask != 0
        }
        ActionFilter::Throw => {
            matches!(
                obj.class,
                ObjectClass::Weapon | ObjectClass::Gem | ObjectClass::Tool | ObjectClass::Potion
            )
        }
        ActionFilter::Apply => {
            matches!(
                obj.class,
                ObjectClass::Tool | ObjectClass::Weapon | ObjectClass::Food | ObjectClass::Potion
            )
        }
        ActionFilter::Dip => {
            // Most items can be dipped, exclude worn items
            obj.worn_mask == 0
        }
        ActionFilter::Invoke => {
            // Artifacts or unique items
            obj.artifact != 0
        }
        ActionFilter::Rub => {
            // Lamps and gemstones
            matches!(obj.class, ObjectClass::Tool | ObjectClass::Gem)
        }
        ActionFilter::Tip => {
            // Containers
            obj.is_container()
        }
    }
}

/// Filter inventory items according to a filter specification.
///
/// Returns indices of matching items (C: getobj enumeration phase).
pub fn filter_inventory(inventory: &[Object], filter: &ItemFilter) -> Vec<usize> {
    inventory
        .iter()
        .enumerate()
        .filter(|(_, obj)| filter.matches(obj))
        .map(|(i, _)| i)
        .collect()
}

/// Convenience: get references to matching items
pub fn matching_items<'a>(inventory: &'a [Object], filter: &ItemFilter) -> Vec<&'a Object> {
    inventory
        .iter()
        .filter(|obj| filter.matches(obj))
        .collect()
}

// ============================================================================
// Pre-built filters for common actions
// ============================================================================

/// Filter for eating (food only)
pub fn eat_filter() -> ItemFilter {
    ItemFilter::classes(&[ObjectClass::Food]).with_action(ActionFilter::Eat)
}

/// Filter for drinking (potions only)
pub fn drink_filter() -> ItemFilter {
    ItemFilter::classes(&[ObjectClass::Potion]).with_action(ActionFilter::Drink)
}

/// Filter for reading (scrolls and spellbooks)
pub fn read_filter() -> ItemFilter {
    ItemFilter::classes(&[ObjectClass::Scroll, ObjectClass::Spellbook])
        .with_action(ActionFilter::Read)
}

/// Filter for zapping (wands only)
pub fn zap_filter() -> ItemFilter {
    ItemFilter::classes(&[ObjectClass::Wand]).with_action(ActionFilter::Zap)
}

/// Filter for wielding (weapons and tools, with bare-hands option)
pub fn wield_filter() -> ItemFilter {
    ItemFilter::all()
        .with_none()
        .with_action(ActionFilter::Wield)
}

/// Filter for wearing armor
pub fn wear_filter() -> ItemFilter {
    ItemFilter::classes(&[ObjectClass::Armor]).with_action(ActionFilter::Wear)
}

/// Filter for putting on accessories (rings, amulets, tools like blindfold)
pub fn puton_filter() -> ItemFilter {
    ItemFilter::classes(&[ObjectClass::Ring, ObjectClass::Amulet, ObjectClass::Tool])
        .with_action(ActionFilter::PutOn)
}

/// Filter for taking off worn items
pub fn takeoff_filter() -> ItemFilter {
    ItemFilter::all().with_action(ActionFilter::TakeOff)
}

/// Filter for throwing
pub fn throw_filter() -> ItemFilter {
    ItemFilter::all()
        .with_count()
        .with_action(ActionFilter::Throw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::BucStatus;

    fn make_obj(id: u32, class: ObjectClass, letter: char) -> Object {
        let mut obj = Object::default();
        obj.id = ObjectId(id);
        obj.class = class;
        obj.inv_letter = letter;
        obj.quantity = 1;
        obj
    }

    #[test]
    fn test_assign_invlet() {
        let inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Armor, 'b'),
        ];

        let mut new_obj = make_obj(3, ObjectClass::Food, '\0');
        assign_invlet(&mut new_obj, &inventory);
        assert_eq!(new_obj.inv_letter, 'c');
    }

    #[test]
    fn test_assign_invlet_gold() {
        let inventory = vec![];
        let mut gold = make_obj(1, ObjectClass::Coin, '\0');
        assign_invlet(&mut gold, &inventory);
        assert_eq!(gold.inv_letter, GOLD_SYM);
    }

    #[test]
    fn test_find_by_letter() {
        let inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Armor, 'b'),
            make_obj(3, ObjectClass::Food, 'c'),
        ];

        assert_eq!(find_by_letter(&inventory, 'b'), Some(1));
        assert_eq!(find_by_letter(&inventory, 'z'), None);
    }

    #[test]
    fn test_is_full() {
        let mut inventory = Vec::new();
        assert!(!is_full(&inventory));

        // Fill with 52 items
        for i in 0..52 {
            let letter = if i < 26 {
                (b'a' + i as u8) as char
            } else {
                (b'A' + (i - 26) as u8) as char
            };
            inventory.push(make_obj(i as u32, ObjectClass::Weapon, letter));
        }
        assert!(is_full(&inventory));
    }

    #[test]
    fn test_item_count() {
        let mut inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Weapon, 'b'),
        ];
        inventory[0].quantity = 5;
        inventory[1].quantity = 3;

        assert_eq!(slot_count(&inventory), 2);
        assert_eq!(item_count(&inventory), 8);
    }

    #[test]
    fn test_total_weight_with_quantity() {
        let mut inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Weapon, 'b'),
        ];
        // Stack of 5 arrows at weight 1 each
        inventory[0].weight = 1;
        inventory[0].quantity = 5;
        // Single item at weight 10
        inventory[1].weight = 10;
        inventory[1].quantity = 1;

        assert_eq!(total_weight(&inventory), 15); // 5*1 + 1*10
    }

    #[test]
    fn test_gold_count() {
        let mut inventory = vec![
            make_obj(1, ObjectClass::Coin, '$'),
            make_obj(2, ObjectClass::Weapon, 'a'),
        ];
        inventory[0].quantity = 100;

        assert_eq!(gold_count(&inventory), 100);
    }

    #[test]
    fn test_find_mergeable() {
        let mut inventory = vec![make_obj(1, ObjectClass::Weapon, 'a')];
        inventory[0].object_type = 5;
        inventory[0].buc = BucStatus::Uncursed;

        let mut new_obj = make_obj(2, ObjectClass::Weapon, '\0');
        new_obj.object_type = 5;
        new_obj.buc = BucStatus::Uncursed;

        // Should find mergeable (same type, same BUC)
        assert!(find_mergeable(&inventory, &new_obj).is_some());

        // Different BUC should not merge
        new_obj.buc = BucStatus::Cursed;
        assert!(find_mergeable(&inventory, &new_obj).is_none());
    }

    #[test]
    fn test_sort_inventory() {
        let mut inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'c'),
            make_obj(2, ObjectClass::Armor, 'a'),
            make_obj(3, ObjectClass::Food, 'B'),
        ];

        sort_inventory(&mut inventory);

        assert_eq!(inventory[0].inv_letter, 'a');
        assert_eq!(inventory[1].inv_letter, 'c');
        assert_eq!(inventory[2].inv_letter, 'B');
    }

    #[test]
    fn test_objects_of_class() {
        let inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Armor, 'b'),
            make_obj(3, ObjectClass::Weapon, 'c'),
        ];

        let weapons = objects_of_class(&inventory, ObjectClass::Weapon);
        assert_eq!(weapons.len(), 2);
    }

    #[test]
    fn test_worn_objects() {
        let mut inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Armor, 'b'),
        ];
        inventory[1].worn_mask = 1;

        let worn = worn_objects(&inventory);
        assert_eq!(worn.len(), 1);
        assert_eq!(worn[0].inv_letter, 'b');
    }

    // ================================================================
    // Item filter tests
    // ================================================================

    #[test]
    fn test_eat_filter_accepts_food() {
        let food = make_obj(1, ObjectClass::Food, 'a');
        let weapon = make_obj(2, ObjectClass::Weapon, 'b');
        let filter = eat_filter();
        assert!(filter.matches(&food));
        assert!(!filter.matches(&weapon));
    }

    #[test]
    fn test_drink_filter_accepts_potions() {
        let potion = make_obj(1, ObjectClass::Potion, 'a');
        let scroll = make_obj(2, ObjectClass::Scroll, 'b');
        let filter = drink_filter();
        assert!(filter.matches(&potion));
        assert!(!filter.matches(&scroll));
    }

    #[test]
    fn test_read_filter_accepts_scrolls_and_books() {
        let scroll = make_obj(1, ObjectClass::Scroll, 'a');
        let book = make_obj(2, ObjectClass::Spellbook, 'b');
        let wand = make_obj(3, ObjectClass::Wand, 'c');
        let filter = read_filter();
        assert!(filter.matches(&scroll));
        assert!(filter.matches(&book));
        assert!(!filter.matches(&wand));
    }

    #[test]
    fn test_zap_filter_accepts_wands() {
        let wand = make_obj(1, ObjectClass::Wand, 'a');
        let potion = make_obj(2, ObjectClass::Potion, 'b');
        let filter = zap_filter();
        assert!(filter.matches(&wand));
        assert!(!filter.matches(&potion));
    }

    #[test]
    fn test_wield_filter_accepts_weapons_and_tools() {
        let weapon = make_obj(1, ObjectClass::Weapon, 'a');
        let tool = make_obj(2, ObjectClass::Tool, 'b');
        let food = make_obj(3, ObjectClass::Food, 'c');
        let filter = wield_filter();
        assert!(filter.matches(&weapon));
        assert!(filter.matches(&tool));
        assert!(!filter.matches(&food));
    }

    #[test]
    fn test_wear_filter_excludes_worn() {
        let mut armor = make_obj(1, ObjectClass::Armor, 'a');
        let filter = wear_filter();
        assert!(filter.matches(&armor));
        armor.worn_mask = 1; // now wearing it
        assert!(!filter.matches(&armor));
    }

    #[test]
    fn test_takeoff_filter_requires_worn() {
        let mut armor = make_obj(1, ObjectClass::Armor, 'a');
        let filter = takeoff_filter();
        assert!(!filter.matches(&armor)); // not worn
        armor.worn_mask = 1;
        assert!(filter.matches(&armor)); // worn
    }

    #[test]
    fn test_filter_excludes_gold_by_default() {
        let mut gold = make_obj(1, ObjectClass::Coin, '$');
        gold.quantity = 100;
        let filter = ItemFilter::all();
        assert!(!filter.matches(&gold));

        let filter_with_gold = ItemFilter::all().with_gold();
        assert!(filter_with_gold.matches(&gold));
    }

    #[test]
    fn test_filter_inventory_returns_indices() {
        let inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Food, 'b'),
            make_obj(3, ObjectClass::Food, 'c'),
            make_obj(4, ObjectClass::Potion, 'd'),
        ];
        let indices = filter_inventory(&inventory, &eat_filter());
        assert_eq!(indices, vec![1, 2]); // two food items
    }

    #[test]
    fn test_matching_items() {
        let inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Wand, 'b'),
            make_obj(3, ObjectClass::Potion, 'c'),
        ];
        let items = matching_items(&inventory, &zap_filter());
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].class, ObjectClass::Wand);
    }

    #[test]
    fn test_puton_filter_rings_and_amulets() {
        let ring = make_obj(1, ObjectClass::Ring, 'a');
        let amulet = make_obj(2, ObjectClass::Amulet, 'b');
        let mut worn_ring = make_obj(3, ObjectClass::Ring, 'c');
        worn_ring.worn_mask = 1;
        let filter = puton_filter();
        assert!(filter.matches(&ring));
        assert!(filter.matches(&amulet));
        assert!(!filter.matches(&worn_ring)); // already worn
    }

    #[test]
    fn test_inventory_summary() {
        let mut inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Weapon, 'b'),
            make_obj(3, ObjectClass::Armor, 'c'),
        ];
        inventory[0].quantity = 5;
        inventory[1].quantity = 3;
        inventory[2].quantity = 1;

        let summary = inventory_summary(&inventory);
        assert_eq!(summary.len(), 2);

        let weapon_entry = summary.iter().find(|(c, _, _)| *c == ObjectClass::Weapon);
        assert!(weapon_entry.is_some());
        let (_, slots, total) = weapon_entry.unwrap();
        assert_eq!(*slots, 2);
        assert_eq!(*total, 8);
    }
}
