//! Inventory management (invent.c)
//!
//! Functions for managing player inventory.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::object::{BucStatus, Object, ObjectClass, ObjectId};

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

/// Check if carrying a corpse or statue of a specific monster type (have_corpsenm equivalent)
pub fn have_corpsenm(inventory: &[Object], monster_type: i16) -> bool {
    inventory.iter().any(|obj| {
        (obj.is_corpse() || obj.is_statue() || obj.is_figurine()) && obj.corpse_type == monster_type
    })
}

/// Get a corpse or statue of a specific monster type
pub fn get_corpsenm(inventory: &[Object], monster_type: i16) -> Option<&Object> {
    inventory.iter().find(|obj| {
        (obj.is_corpse() || obj.is_statue() || obj.is_figurine()) && obj.corpse_type == monster_type
    })
}

/// Check if carrying any artifact
pub fn carrying_artifact(inventory: &[Object]) -> bool {
    inventory.iter().any(|obj| obj.is_artifact())
}

/// Get artifact by index
pub fn get_artifact(inventory: &[Object], artifact_id: u8) -> Option<&Object> {
    inventory.iter().find(|obj| obj.artifact == artifact_id)
}

/// Check if carrying any blessed object
pub fn carrying_blessed(inventory: &[Object]) -> bool {
    inventory.iter().any(|obj| obj.is_blessed())
}

/// Check if carrying any cursed object
pub fn carrying_cursed(inventory: &[Object]) -> bool {
    inventory.iter().any(|obj| obj.is_cursed())
}

/// Check if carrying any lit light source
pub fn carrying_lit(inventory: &[Object]) -> bool {
    inventory.iter().any(|obj| obj.is_lit())
}

/// Find object by inventory letter
pub fn get_by_letter(inventory: &[Object], letter: char) -> Option<&Object> {
    inventory.iter().find(|obj| obj.inv_letter == letter)
}

/// Find mutable object by inventory letter
pub fn get_by_letter_mut(inventory: &mut [Object], letter: char) -> Option<&mut Object> {
    inventory.iter_mut().find(|obj| obj.inv_letter == letter)
}

/// Check if carrying an object of a specific class
pub fn carrying_class(inventory: &[Object], class: ObjectClass) -> bool {
    inventory.iter().any(|obj| obj.class == class)
}

/// Count objects of a specific class
pub fn count_class(inventory: &[Object], class: ObjectClass) -> i32 {
    inventory
        .iter()
        .filter(|obj| obj.class == class)
        .map(|obj| obj.quantity)
        .sum()
}

/// Get all weapons in inventory
pub fn weapons(inventory: &[Object]) -> Vec<&Object> {
    inventory.iter().filter(|obj| obj.is_weapon()).collect()
}

/// Get all armor in inventory
pub fn armor(inventory: &[Object]) -> Vec<&Object> {
    inventory.iter().filter(|obj| obj.is_armor()).collect()
}

/// Get all food in inventory
pub fn food(inventory: &[Object]) -> Vec<&Object> {
    inventory.iter().filter(|obj| obj.is_food()).collect()
}

/// Get all potions in inventory
pub fn potions(inventory: &[Object]) -> Vec<&Object> {
    inventory.iter().filter(|obj| obj.is_potion()).collect()
}

/// Get all scrolls in inventory
pub fn scrolls(inventory: &[Object]) -> Vec<&Object> {
    inventory.iter().filter(|obj| obj.is_scroll()).collect()
}

/// Get all wands in inventory
pub fn wands(inventory: &[Object]) -> Vec<&Object> {
    inventory.iter().filter(|obj| obj.is_wand()).collect()
}

/// Get all rings in inventory
pub fn rings(inventory: &[Object]) -> Vec<&Object> {
    inventory.iter().filter(|obj| obj.is_ring()).collect()
}

/// Get all amulets in inventory
pub fn amulets(inventory: &[Object]) -> Vec<&Object> {
    inventory.iter().filter(|obj| obj.is_amulet()).collect()
}

/// Get all tools in inventory
pub fn tools(inventory: &[Object]) -> Vec<&Object> {
    inventory.iter().filter(|obj| obj.is_tool()).collect()
}

/// Get all gems in inventory
pub fn gems(inventory: &[Object]) -> Vec<&Object> {
    inventory.iter().filter(|obj| obj.is_gem()).collect()
}

/// Get all spellbooks in inventory
pub fn spellbooks(inventory: &[Object]) -> Vec<&Object> {
    inventory.iter().filter(|obj| obj.is_spellbook()).collect()
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

// ============================================================================
// NetHack C function aliases
// ============================================================================

/// Get count of items in inventory (inv_cnt equivalent)
/// Returns the total quantity of all items
pub fn inv_cnt(inventory: &[Object], include_gold: bool) -> i32 {
    inventory
        .iter()
        .filter(|obj| include_gold || obj.class != ObjectClass::Coin)
        .map(|obj| obj.quantity)
        .sum()
}

/// Get total weight of inventory (inv_weight equivalent)
/// Returns the total weight of all items
pub fn inv_weight(inventory: &[Object]) -> u32 {
    total_weight(inventory)
}

/// Get weight capacity bonus based on strength (weight_cap_bonus equivalent)
/// Higher strength allows carrying more weight
pub fn weight_cap_bonus(strength: i8) -> u32 {
    // Carrying capacity formula from NetHack
    // Base is 25 * strength, with bonuses for high strength
    let base = 25 * strength as u32;
    let bonus = if strength >= 18 {
        // Extra capacity for exceptional strength
        (strength as u32 - 17) * 10
    } else {
        0
    };
    base + bonus
}

/// Calculate encumbrance level based on inventory weight and capacity (calc_capacity equivalent)
/// Returns encumbrance level: 0=unencumbered, 1=burdened, 2=stressed, 3=strained, 4=overtaxed, 5=overloaded
pub fn calc_capacity(inventory_weight: u32, weight_capacity: u32) -> u8 {
    if weight_capacity == 0 {
        return 5; // Overloaded
    }

    let ratio = (inventory_weight * 100) / weight_capacity;

    if ratio < 50 {
        0 // Unencumbered
    } else if ratio < 75 {
        1 // Burdened
    } else if ratio < 100 {
        2 // Stressed
    } else if ratio < 125 {
        3 // Strained
    } else if ratio < 150 {
        4 // Overtaxed
    } else {
        5 // Overloaded
    }
}

/// Get encumbrance name
pub const fn encumbrance_name(level: u8) -> &'static str {
    match level {
        0 => "Unencumbered",
        1 => "Burdened",
        2 => "Stressed",
        3 => "Strained",
        4 => "Overtaxed",
        _ => "Overloaded",
    }
}

// ============================================================================
// Count functions (count_* from invent.c)
// ============================================================================

/// Count items by BUC status (count_buc equivalent)
///
/// Returns counts of blessed, uncursed, and cursed items.
pub fn count_buc(inventory: &[Object]) -> (i32, i32, i32) {
    let mut blessed = 0;
    let mut uncursed = 0;
    let mut cursed = 0;

    for obj in inventory {
        match obj.buc {
            BucStatus::Blessed => blessed += obj.quantity,
            BucStatus::Uncursed => uncursed += obj.quantity,
            BucStatus::Cursed => cursed += obj.quantity,
        }
    }

    (blessed, uncursed, cursed)
}

/// Count unpaid items in inventory (count_unpaid equivalent)
pub fn count_unpaid(inventory: &[Object]) -> i32 {
    inventory
        .iter()
        .filter(|obj| obj.unpaid)
        .map(|obj| obj.quantity)
        .sum()
}

/// Count objects in inventory (count_obj equivalent)
///
/// Returns the total number of objects (counting stacks as 1)
pub fn count_obj(inventory: &[Object]) -> i32 {
    inventory.len() as i32
}

/// Count items with a specific property
pub fn count_with<F>(inventory: &[Object], predicate: F) -> i32
where
    F: Fn(&Object) -> bool,
{
    inventory
        .iter()
        .filter(|obj| predicate(obj))
        .map(|obj| obj.quantity)
        .sum()
}

/// Count worn items (count_worn_stuff equivalent)
pub fn count_worn_stuff(inventory: &[Object]) -> i32 {
    inventory.iter().filter(|obj| obj.is_worn()).count() as i32
}

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
            core::cmp::Ordering::Equal => {
                // Within same class, sort by letter
                inv_rank(a.inv_letter).cmp(&inv_rank(b.inv_letter))
            }
            other => other,
        }
    });
}

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

// ============================================================================
// Display and UI functions (display_inventory, dotypeinv, etc.)
// ============================================================================

/// Display inventory to player (display_inventory equivalent)
///
/// Returns formatted string showing inventory contents with letters and quantities.
/// Used by the 'i' (inventory) command.
pub fn display_inventory(inventory: &[Object]) -> String {
    if inventory.is_empty() {
        return "You are not carrying anything.".to_string();
    }

    let mut result = String::new();
    for obj in inventory {
        let letter = obj.inv_letter;
        let name = format_object_name(obj);
        let line = format!("  {} - {}\n", letter, name);
        result.push_str(&line);
    }

    // Include weight info
    let weight = inv_weight(inventory);
    result.push_str(&format!("\nTotal weight: {} units", weight));

    result
}

/// Display inventory of specific object class (dotypeinv equivalent)
///
/// Returns formatted string showing inventory items of a specific class.
/// Used for type-specific inventory queries.
pub fn dotypeinv(inventory: &[Object], class: ObjectClass) -> String {
    let items: Vec<&Object> = inventory.iter().filter(|obj| obj.class == class).collect();

    if items.is_empty() {
        return format!("You are not carrying any {}s.", class);
    }

    let mut result = String::from("Your inventory:\n");
    for obj in items {
        let letter = obj.inv_letter;
        let name = format_object_name(obj);
        let line = format!("  {} - {}\n", letter, name);
        result.push_str(&line);
    }

    result
}

/// Display packed (detailed) inventory (dolook equivalent for inventory)
///
/// Returns inventory with more detail (charges, wear status, etc.)
pub fn display_packed_inventory(inventory: &[Object]) -> String {
    if inventory.is_empty() {
        return "You are not carrying anything.".to_string();
    }

    let mut result = String::new();
    for obj in inventory {
        let letter = obj.inv_letter;
        let name = format_object_detail(obj);
        let line = format!("  {} - {}\n", letter, name);
        result.push_str(&line);
    }

    result
}

/// Display equipped weapons (doprwep equivalent)
///
/// Shows the currently equipped/wielded weapon(s).
pub fn doprwep(inventory: &[Object]) -> String {
    let wielded: Vec<&Object> = inventory.iter().filter(|obj| obj.is_wielded()).collect();

    if wielded.is_empty() {
        return "You are not wielding any weapon.".to_string();
    }

    let mut result = String::from("Currently wielding:\n");
    for obj in wielded {
        let name = format_object_name(obj);
        result.push_str(&format!("  {}\n", name));
    }

    result
}

/// Display equipped armor (doprarm equivalent)
///
/// Shows all currently equipped armor pieces.
pub fn doprarm(inventory: &[Object]) -> String {
    let worn: Vec<&Object> = inventory
        .iter()
        .filter(|obj| obj.is_worn() && obj.is_armor())
        .collect();

    if worn.is_empty() {
        return "You are not wearing any armor.".to_string();
    }

    let mut result = String::from("Currently wearing:\n");
    for obj in worn {
        let name = format_object_name(obj);
        result.push_str(&format!("  {}\n", name));
    }

    result
}

/// Main inventory command handler (ddoinv equivalent)
///
/// Displays inventory and returns formatted inventory string.
/// This is the top-level inventory display function called by the game loop.
pub fn ddoinv(inventory: &[Object]) -> String {
    let encumbrance = calc_capacity(inv_weight(inventory), 300); // Placeholder capacity
    let (blessed, uncursed, cursed) = count_buc(inventory);

    let mut result = display_inventory(inventory);
    result.push_str(&format!(
        "\n\nBless status: {} blessed, {} uncursed, {} cursed",
        blessed, uncursed, cursed
    ));

    let encumb_name = encumbrance_name(encumbrance);
    result.push_str(&format!("\nEncumbrance: {}", encumb_name));

    result
}

/// Display inventory and apply a function to selection (interact)
///
/// Used for commands like 'drop', 'equip', 'examine', etc.
pub fn doinv_obj<F>(inventory: &[Object], callback: F) -> String
where
    F: Fn(&Object) -> String,
{
    let mut result = String::new();
    for obj in inventory {
        let letter = obj.inv_letter;
        let name = format_object_name(obj);
        let action_result = callback(obj);
        let line = format!("  {} - {} {}\n", letter, name, action_result);
        result.push_str(&line);
    }
    result
}

// Helper function to format object name for display
fn format_object_name(obj: &Object) -> String {
    let mut name = String::new();

    // Add quantity if > 1
    if obj.quantity > 1 {
        name.push_str(&format!("{} ", obj.quantity));
    }

    // Add BUC prefix
    if obj.buc_known {
        name.push_str(obj.buc_prefix());
    }

    // Add erosion prefix
    name.push_str(&obj.erosion_prefix());

    // Add base name (would need obj typename from discovery)
    name.push_str("item");

    // Add enchantment
    name.push_str(&obj.enchantment_str());

    // Add wear status
    name.push_str(obj.worn_suffix());

    // Add charges for wands
    name.push_str(&obj.charges_suffix());

    name
}

// Helper function to format detailed object name
fn format_object_detail(obj: &Object) -> String {
    let mut name = format_object_name(obj);

    // Add greased status
    if obj.greased {
        name.push_str(" [greased]");
    }

    // Add locked/trapped status for containers
    if obj.is_container() {
        if obj.locked {
            name.push_str(" [locked]");
        }
        if obj.trapped {
            name.push_str(" [trapped]");
        }
        if !obj.contents.is_empty() {
            name.push_str(&format!(" ({} items)", obj.contents.len()));
        }
    }

    name
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

    // ========================================================================
    // Phase 3 Tests: Display and Command Handler Functions
    // ========================================================================

    #[test]
    fn test_display_inventory_empty() {
        let inventory: Vec<Object> = vec![];
        let display = display_inventory(&inventory);
        assert!(display.contains("not carrying anything"));
    }

    #[test]
    fn test_display_inventory() {
        let mut inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Armor, 'b'),
            make_obj(3, ObjectClass::Food, 'c'),
        ];
        inventory[0].quantity = 3;

        let display = display_inventory(&inventory);
        assert!(display.contains("a"));
        assert!(display.contains("b"));
        assert!(display.contains("c"));
        assert!(display.contains("Total weight"));
    }

    #[test]
    fn test_dotypeinv_weapon() {
        let inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Armor, 'b'),
            make_obj(3, ObjectClass::Weapon, 'c'),
        ];

        let display = dotypeinv(&inventory, ObjectClass::Weapon);
        assert!(display.contains("a"));
        assert!(display.contains("c"));
        assert!(!display.contains("b")); // Armor should not appear
    }

    #[test]
    fn test_dotypeinv_empty() {
        let inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Armor, 'b'),
        ];

        let display = dotypeinv(&inventory, ObjectClass::Food);
        assert!(display.contains("not carrying"));
    }

    #[test]
    fn test_display_packed_inventory() {
        let mut inventory = vec![make_obj(1, ObjectClass::Armor, 'a')];
        inventory[0].locked = true;

        let display = display_packed_inventory(&inventory);
        assert!(display.contains("a"));
    }

    #[test]
    fn test_doprwep() {
        let mut inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Weapon, 'b'),
        ];
        inventory[0].worn_mask = 0x8000; // W_WEP flag for wielded

        let display = doprwep(&inventory);
        assert!(display.contains("wielding"));
        assert!(!display.contains("not wielding")); // Has a wielded weapon
    }

    #[test]
    fn test_doprwep_empty() {
        let inventory = vec![make_obj(1, ObjectClass::Armor, 'a')];

        let display = doprwep(&inventory);
        assert!(display.contains("not wielding"));
    }

    #[test]
    fn test_doprarm() {
        let mut inventory = vec![
            make_obj(1, ObjectClass::Armor, 'a'),
            make_obj(2, ObjectClass::Armor, 'b'),
        ];
        inventory[0].worn_mask = 1; // Worn

        let display = doprarm(&inventory);
        assert!(display.contains("wearing"));
        assert!(!display.contains("not wearing"));
    }

    #[test]
    fn test_doprarm_empty() {
        let inventory = vec![make_obj(1, ObjectClass::Weapon, 'a')];

        let display = doprarm(&inventory);
        assert!(display.contains("not wearing"));
    }

    #[test]
    fn test_ddoinv() {
        let mut inventory = vec![
            make_obj(1, ObjectClass::Coin, '$'),
            make_obj(2, ObjectClass::Weapon, 'a'),
        ];
        inventory[0].buc = BucStatus::Blessed;

        let display = ddoinv(&inventory);
        assert!(display.contains("Total weight"));
        assert!(display.contains("Bless status"));
        assert!(display.contains("Encumbrance"));
    }

    #[test]
    fn test_count_buc_inventory() {
        let mut inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Weapon, 'b'),
            make_obj(3, ObjectClass::Weapon, 'c'),
        ];
        inventory[0].buc = BucStatus::Blessed;
        inventory[1].buc = BucStatus::Cursed;
        inventory[2].buc = BucStatus::Uncursed;

        let (blessed, uncursed, cursed) = count_buc(&inventory);
        assert_eq!(blessed, 1);
        assert_eq!(uncursed, 1);
        assert_eq!(cursed, 1);
    }

    #[test]
    fn test_count_unpaid() {
        let mut inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Weapon, 'b'),
            make_obj(3, ObjectClass::Weapon, 'c'),
        ];
        inventory[0].unpaid = true;
        inventory[0].quantity = 5;
        inventory[1].unpaid = true;
        inventory[1].quantity = 3;

        let unpaid = count_unpaid(&inventory);
        assert_eq!(unpaid, 8); // 5 + 3
    }

    #[test]
    fn test_inventory_weight() {
        let mut inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Armor, 'b'),
        ];
        inventory[0].weight = 50;
        inventory[0].quantity = 2;
        inventory[1].weight = 100;
        inventory[1].quantity = 1;

        let total = inv_weight(&inventory);
        assert_eq!(total, 200); // 50*2 + 100*1
    }

    #[test]
    fn test_calc_capacity() {
        // Unencumbered: < 50%
        assert_eq!(calc_capacity(25, 100), 0);

        // Burdened: 50-74%
        assert_eq!(calc_capacity(50, 100), 1);
        assert_eq!(calc_capacity(74, 100), 1);

        // Stressed: 75-99%
        assert_eq!(calc_capacity(75, 100), 2);

        // Strained: 100-124%
        assert_eq!(calc_capacity(100, 100), 3);

        // Overtaxed: 125-149%
        assert_eq!(calc_capacity(125, 100), 4);

        // Overloaded: 150%+
        assert_eq!(calc_capacity(150, 100), 5);
    }

    #[test]
    fn test_encumbrance_name() {
        assert_eq!(encumbrance_name(0), "Unencumbered");
        assert_eq!(encumbrance_name(1), "Burdened");
        assert_eq!(encumbrance_name(2), "Stressed");
        assert_eq!(encumbrance_name(3), "Strained");
        assert_eq!(encumbrance_name(4), "Overtaxed");
        assert_eq!(encumbrance_name(5), "Overloaded");
    }

    #[test]
    fn test_weight_cap_bonus() {
        // Low strength (3): reduced capacity
        let low = weight_cap_bonus(3);

        // High strength (18): increased capacity
        let high = weight_cap_bonus(18);

        // High should be > low
        assert!(high > low);
    }

    #[test]
    fn test_carrying_blessed_cursed() {
        let mut inventory = vec![
            make_obj(1, ObjectClass::Weapon, 'a'),
            make_obj(2, ObjectClass::Weapon, 'b'),
        ];
        inventory[0].buc = BucStatus::Blessed;
        inventory[1].buc = BucStatus::Cursed;

        assert!(carrying_blessed(&inventory));
        assert!(carrying_cursed(&inventory));

        let inventory2 = vec![make_obj(1, ObjectClass::Weapon, 'a')];
        assert!(!carrying_blessed(&inventory2));
        assert!(!carrying_cursed(&inventory2));
    }
}
