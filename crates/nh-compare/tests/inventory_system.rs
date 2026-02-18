//! Inventory system behavioral tests
//!
//! Tests for inventory management: adding, removing, searching,
//! filtering, counting, weight, capacity, and display functions.

use nh_core::object::*;

// ============================================================================
// Helpers
// ============================================================================

fn make_weapon(letter: char) -> Object {
    let mut obj = Object::default();
    obj.class = ObjectClass::Weapon;
    obj.inv_letter = letter;
    obj.weight = 30;
    obj
}

fn make_armor(letter: char) -> Object {
    let mut obj = Object::default();
    obj.class = ObjectClass::Armor;
    obj.inv_letter = letter;
    obj.weight = 150;
    obj
}

fn make_potion(letter: char) -> Object {
    let mut obj = Object::default();
    obj.class = ObjectClass::Potion;
    obj.inv_letter = letter;
    obj.weight = 20;
    obj
}

fn make_scroll(letter: char) -> Object {
    let mut obj = Object::default();
    obj.class = ObjectClass::Scroll;
    obj.inv_letter = letter;
    obj.weight = 5;
    obj
}

fn make_gold(amount: i32) -> Object {
    let mut obj = Object::default();
    obj.class = ObjectClass::Coin;
    obj.quantity = amount;
    obj.weight = 1;
    obj
}

fn sample_inventory() -> Vec<Object> {
    vec![
        make_weapon('a'),
        make_armor('b'),
        make_potion('c'),
        make_scroll('d'),
    ]
}

// ============================================================================
// find_by_letter / find_by_id
// ============================================================================

#[test]
fn test_find_by_letter_found() {
    let inv = sample_inventory();
    assert_eq!(inventory::find_by_letter(&inv, 'a'), Some(0));
}

#[test]
fn test_find_by_letter_not_found() {
    let inv = sample_inventory();
    assert_eq!(inventory::find_by_letter(&inv, 'z'), None);
}

#[test]
fn test_find_by_letter_middle() {
    let inv = sample_inventory();
    assert_eq!(inventory::find_by_letter(&inv, 'c'), Some(2));
}

#[test]
fn test_find_by_id_found() {
    let mut inv = sample_inventory();
    inv[1].id = ObjectId(42);
    assert_eq!(inventory::find_by_id(&inv, ObjectId(42)), Some(1));
}

#[test]
fn test_find_by_id_not_found() {
    let inv = sample_inventory();
    assert_eq!(inventory::find_by_id(&inv, ObjectId(999)), None);
}

// ============================================================================
// is_full / slot_count / item_count
// ============================================================================

#[test]
fn test_is_full_empty() {
    let inv: Vec<Object> = vec![];
    assert!(!inventory::is_full(&inv));
}

#[test]
fn test_is_full_few_items() {
    let inv = sample_inventory();
    assert!(!inventory::is_full(&inv));
}

#[test]
fn test_slot_count() {
    let inv = sample_inventory();
    assert_eq!(inventory::slot_count(&inv), 4);
}

#[test]
fn test_slot_count_empty() {
    let inv: Vec<Object> = vec![];
    assert_eq!(inventory::slot_count(&inv), 0);
}

#[test]
fn test_item_count_with_quantities() {
    let mut inv = sample_inventory();
    inv[2].quantity = 5; // 5 potions
    let count = inventory::item_count(&inv);
    assert_eq!(count, 8); // 1+1+5+1
}

// ============================================================================
// total_weight
// ============================================================================

#[test]
fn test_total_weight_empty() {
    let inv: Vec<Object> = vec![];
    assert_eq!(inventory::total_weight(&inv), 0);
}

#[test]
fn test_total_weight_single() {
    let inv = vec![make_weapon('a')];
    assert_eq!(inventory::total_weight(&inv), 30);
}

#[test]
fn test_total_weight_multiple() {
    let inv = sample_inventory();
    // 30 + 150 + 20 + 5 = 205
    assert_eq!(inventory::total_weight(&inv), 205);
}

#[test]
fn test_total_weight_with_quantity() {
    let mut inv = vec![make_potion('a')];
    inv[0].quantity = 3;
    assert_eq!(inventory::total_weight(&inv), 60); // 20 * 3
}

// ============================================================================
// gold_count
// ============================================================================

#[test]
fn test_gold_count_no_gold() {
    let inv = sample_inventory();
    assert_eq!(inventory::gold_count(&inv), 0);
}

#[test]
fn test_gold_count_with_gold() {
    let mut inv = sample_inventory();
    inv.push(make_gold(100));
    assert_eq!(inventory::gold_count(&inv), 100);
}

// ============================================================================
// add_to_inventory / remove
// ============================================================================

#[test]
fn test_add_to_inventory() {
    let mut inv = Vec::new();
    let obj = make_weapon('a');
    let idx = inventory::add_to_inventory(&mut inv, obj);
    assert_eq!(inv.len(), 1);
    let _ = idx;
}

#[test]
fn test_remove_from_inventory() {
    let mut inv = sample_inventory();
    let removed = inventory::remove_from_inventory(&mut inv, 1);
    assert!(removed.is_some());
    assert_eq!(inv.len(), 3);
}

#[test]
fn test_remove_from_inventory_out_of_bounds() {
    let mut inv = sample_inventory();
    let removed = inventory::remove_from_inventory(&mut inv, 99);
    assert!(removed.is_none());
}

#[test]
fn test_remove_by_letter() {
    let mut inv = sample_inventory();
    let removed = inventory::remove_by_letter(&mut inv, 'c');
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().class, ObjectClass::Potion);
}

#[test]
fn test_remove_by_letter_not_found() {
    let mut inv = sample_inventory();
    let removed = inventory::remove_by_letter(&mut inv, 'z');
    assert!(removed.is_none());
}

// ============================================================================
// sort_inventory
// ============================================================================

#[test]
fn test_sort_inventory_by_letter() {
    let mut inv = vec![make_potion('c'), make_weapon('a'), make_armor('b')];
    inventory::sort_inventory(&mut inv);
    assert_eq!(inv[0].inv_letter, 'a');
    assert_eq!(inv[1].inv_letter, 'b');
    assert_eq!(inv[2].inv_letter, 'c');
}

// ============================================================================
// objects_of_class / carrying / count_type
// ============================================================================

#[test]
fn test_objects_of_class_weapon() {
    let inv = sample_inventory();
    let weapons = inventory::objects_of_class(&inv, ObjectClass::Weapon);
    assert_eq!(weapons.len(), 1);
}

#[test]
fn test_objects_of_class_none() {
    let inv = sample_inventory();
    let wands = inventory::objects_of_class(&inv, ObjectClass::Wand);
    assert_eq!(wands.len(), 0);
}

#[test]
fn test_carrying_true() {
    let mut inv = sample_inventory();
    inv[0].object_type = 42;
    assert!(inventory::carrying(&inv, 42));
}

#[test]
fn test_carrying_false() {
    let inv = sample_inventory();
    assert!(!inventory::carrying(&inv, 999));
}

#[test]
fn test_count_type_multiple() {
    let mut inv = sample_inventory();
    inv[0].object_type = 5;
    inv.push({
        let mut o = make_weapon('e');
        o.object_type = 5;
        o
    });
    assert_eq!(inventory::count_type(&inv, 5), 2);
}

// ============================================================================
// carrying_artifact / carrying_blessed / carrying_cursed
// ============================================================================

#[test]
fn test_carrying_artifact_false() {
    let inv = sample_inventory();
    assert!(!inventory::carrying_artifact(&inv));
}

#[test]
fn test_carrying_artifact_true() {
    let mut inv = sample_inventory();
    inv[0].artifact = 1;
    assert!(inventory::carrying_artifact(&inv));
}

#[test]
fn test_carrying_blessed_false() {
    let inv = sample_inventory();
    assert!(!inventory::carrying_blessed(&inv));
}

#[test]
fn test_carrying_blessed_true() {
    let mut inv = sample_inventory();
    inv[1].buc = BucStatus::Blessed;
    assert!(inventory::carrying_blessed(&inv));
}

#[test]
fn test_carrying_cursed_false() {
    let inv = sample_inventory();
    assert!(!inventory::carrying_cursed(&inv));
}

#[test]
fn test_carrying_cursed_true() {
    let mut inv = sample_inventory();
    inv[2].buc = BucStatus::Cursed;
    assert!(inventory::carrying_cursed(&inv));
}

// ============================================================================
// Class-specific filters
// ============================================================================

#[test]
fn test_weapons_filter() {
    let inv = sample_inventory();
    assert_eq!(inventory::weapons(&inv).len(), 1);
}

#[test]
fn test_armor_filter() {
    let inv = sample_inventory();
    assert_eq!(inventory::armor(&inv).len(), 1);
}

#[test]
fn test_potions_filter() {
    let inv = sample_inventory();
    assert_eq!(inventory::potions(&inv).len(), 1);
}

#[test]
fn test_scrolls_filter() {
    let inv = sample_inventory();
    assert_eq!(inventory::scrolls(&inv).len(), 1);
}

#[test]
fn test_wands_filter_empty() {
    let inv = sample_inventory();
    assert_eq!(inventory::wands(&inv).len(), 0);
}

#[test]
fn test_rings_filter_empty() {
    let inv = sample_inventory();
    assert_eq!(inventory::rings(&inv).len(), 0);
}

#[test]
fn test_food_filter_empty() {
    let inv = sample_inventory();
    assert_eq!(inventory::food(&inv).len(), 0);
}

#[test]
fn test_gems_filter_empty() {
    let inv = sample_inventory();
    assert_eq!(inventory::gems(&inv).len(), 0);
}

// ============================================================================
// inventory_summary
// ============================================================================

#[test]
fn test_inventory_summary() {
    let inv = sample_inventory();
    let summary = inventory::inventory_summary(&inv);
    assert!(!summary.is_empty());
}

// ============================================================================
// inv_cnt / inv_weight
// ============================================================================

#[test]
fn test_inv_cnt_no_gold() {
    let inv = sample_inventory();
    assert_eq!(inventory::inv_cnt(&inv, false), 4);
}

#[test]
fn test_inv_cnt_with_gold() {
    let mut inv = sample_inventory();
    inv.push(make_gold(50));
    // inv_cnt with gold includes gold quantity
    let cnt = inventory::inv_cnt(&inv, true);
    assert!(cnt > 4, "Should include gold in count, got {}", cnt);
}

#[test]
fn test_inv_weight() {
    let inv = sample_inventory();
    assert_eq!(inventory::inv_weight(&inv), 205);
}

// ============================================================================
// weight_cap_bonus / calc_capacity / encumbrance_name
// ============================================================================

#[test]
fn test_weight_cap_bonus_low_str() {
    let bonus = inventory::weight_cap_bonus(10);
    assert!(bonus > 0);
}

#[test]
fn test_weight_cap_bonus_high_str() {
    let low = inventory::weight_cap_bonus(10);
    let high = inventory::weight_cap_bonus(18);
    assert!(high >= low);
}

#[test]
fn test_calc_capacity_unencumbered() {
    let level = inventory::calc_capacity(100, 1000);
    assert_eq!(level, 0); // unencumbered
}

#[test]
fn test_calc_capacity_overloaded() {
    let level = inventory::calc_capacity(5000, 1000);
    assert!(level > 0);
}

#[test]
fn test_encumbrance_name_unencumbered() {
    let name = inventory::encumbrance_name(0);
    assert!(!name.is_empty());
}

#[test]
fn test_encumbrance_name_burdened() {
    let name = inventory::encumbrance_name(1);
    assert!(!name.is_empty());
}

// ============================================================================
// count_buc / count_unpaid / count_obj / count_worn_stuff
// ============================================================================

#[test]
fn test_count_buc() {
    let mut inv = sample_inventory();
    inv[0].buc = BucStatus::Blessed;
    inv[1].buc = BucStatus::Cursed;
    let (blessed, _uncursed, cursed) = inventory::count_buc(&inv);
    assert_eq!(blessed, 1);
    assert_eq!(cursed, 1);
}

#[test]
fn test_count_unpaid() {
    let inv = sample_inventory();
    assert_eq!(inventory::count_unpaid(&inv), 0);
}

#[test]
fn test_count_obj() {
    let inv = sample_inventory();
    assert_eq!(inventory::count_obj(&inv), 4);
}

#[test]
fn test_count_worn_stuff() {
    let inv = sample_inventory();
    let worn = inventory::count_worn_stuff(&inv);
    assert_eq!(worn, 0);
}

// ============================================================================
// ItemFilter
// ============================================================================

#[test]
fn test_item_filter_all() {
    let filter = inventory::ItemFilter::all();
    let inv = sample_inventory();
    assert!(filter.matches(&inv[0]));
    assert!(filter.matches(&inv[1]));
}

#[test]
fn test_item_filter_classes() {
    let filter = inventory::ItemFilter::classes(&[ObjectClass::Weapon, ObjectClass::Armor]);
    let inv = sample_inventory();
    assert!(filter.matches(&inv[0])); // weapon
    assert!(filter.matches(&inv[1])); // armor
    assert!(!filter.matches(&inv[2])); // potion
}

#[test]
fn test_filter_inventory_all() {
    let inv = sample_inventory();
    let filter = inventory::ItemFilter::all();
    let indices = inventory::filter_inventory(&inv, &filter);
    assert_eq!(indices.len(), 4);
}

#[test]
fn test_matching_items() {
    let inv = sample_inventory();
    let filter = inventory::ItemFilter::classes(&[ObjectClass::Potion]);
    let items = inventory::matching_items(&inv, &filter);
    assert_eq!(items.len(), 1);
}

// ============================================================================
// Predefined filters
// ============================================================================

#[test]
fn test_eat_filter() {
    let filter = inventory::eat_filter();
    let food = {
        let mut o = Object::default();
        o.class = ObjectClass::Food;
        o
    };
    assert!(filter.matches(&food));
}

#[test]
fn test_drink_filter() {
    let filter = inventory::drink_filter();
    let potion = make_potion('a');
    assert!(filter.matches(&potion));
}

#[test]
fn test_read_filter() {
    let filter = inventory::read_filter();
    let scroll = make_scroll('a');
    assert!(filter.matches(&scroll));
}

#[test]
fn test_zap_filter() {
    let filter = inventory::zap_filter();
    let mut wand = Object::default();
    wand.class = ObjectClass::Wand;
    assert!(filter.matches(&wand));
}

#[test]
fn test_wield_filter() {
    let filter = inventory::wield_filter();
    let weapon = make_weapon('a');
    assert!(filter.matches(&weapon));
}

// ============================================================================
// Display functions
// ============================================================================

#[test]
fn test_display_inventory() {
    let inv = sample_inventory();
    let display = inventory::display_inventory(&inv);
    assert!(!display.is_empty());
}

#[test]
fn test_display_inventory_empty() {
    let inv: Vec<Object> = vec![];
    let display = inventory::display_inventory(&inv);
    let _ = display;
}

#[test]
fn test_dotypeinv() {
    let inv = sample_inventory();
    let display = inventory::dotypeinv(&inv, ObjectClass::Weapon);
    let _ = display;
}

#[test]
fn test_display_packed_inventory() {
    let inv = sample_inventory();
    let display = inventory::display_packed_inventory(&inv);
    let _ = display;
}

#[test]
fn test_doprwep() {
    let inv = sample_inventory();
    let display = inventory::doprwep(&inv);
    let _ = display;
}

#[test]
fn test_doprarm() {
    let inv = sample_inventory();
    let display = inventory::doprarm(&inv);
    let _ = display;
}

#[test]
fn test_ddoinv() {
    let inv = sample_inventory();
    let display = inventory::ddoinv(&inv);
    let _ = display;
}

// ============================================================================
// get_by_letter / carrying_class / count_class
// ============================================================================

#[test]
fn test_get_by_letter() {
    let inv = sample_inventory();
    let obj = inventory::get_by_letter(&inv, 'b');
    assert!(obj.is_some());
    assert_eq!(obj.unwrap().class, ObjectClass::Armor);
}

#[test]
fn test_get_by_letter_missing() {
    let inv = sample_inventory();
    assert!(inventory::get_by_letter(&inv, 'z').is_none());
}

#[test]
fn test_carrying_class_true() {
    let inv = sample_inventory();
    assert!(inventory::carrying_class(&inv, ObjectClass::Weapon));
}

#[test]
fn test_carrying_class_false() {
    let inv = sample_inventory();
    assert!(!inventory::carrying_class(&inv, ObjectClass::Wand));
}

#[test]
fn test_count_class() {
    let inv = sample_inventory();
    assert_eq!(inventory::count_class(&inv, ObjectClass::Weapon), 1);
    assert_eq!(inventory::count_class(&inv, ObjectClass::Wand), 0);
}

// ============================================================================
// sort_by_class
// ============================================================================

#[test]
fn test_sort_by_class() {
    let mut inv = vec![make_scroll('d'), make_weapon('a'), make_potion('c')];
    let order = [ObjectClass::Weapon, ObjectClass::Potion, ObjectClass::Scroll];
    inventory::sort_by_class(&mut inv, &order);
    assert_eq!(inv[0].class, ObjectClass::Weapon);
    assert_eq!(inv[1].class, ObjectClass::Potion);
    assert_eq!(inv[2].class, ObjectClass::Scroll);
}
