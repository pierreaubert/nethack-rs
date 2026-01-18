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
            if ('a'..='z').contains(&c) {
                in_use[(c as usize) - ('a' as usize)] = true;
            } else if ('A'..='Z').contains(&c) {
                in_use[(c as usize) - ('A' as usize) + 26] = true;
            }
        }
    }

    // If object already has a valid letter that's not in use, keep it
    let c = obj.inv_letter;
    if ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) {
        let idx = if ('a'..='z').contains(&c) {
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
        if ('a'..='z').contains(&c) || ('A'..='Z').contains(&c) {
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

/// Get total weight of inventory
pub fn total_weight(inventory: &[Object]) -> u32 {
    inventory.iter().map(|obj| obj.weight).sum()
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
    if let Some(idx) = find_by_letter(inventory, letter) {
        Some(inventory.remove(idx))
    } else {
        None
    }
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
