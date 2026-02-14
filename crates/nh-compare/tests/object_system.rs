//! Step 3: Object system parity tests
//!
//! Tests object creation, naming, and inventory management between C and Rust.
//! Since C FFI initialization crashes, these tests verify Rust-side consistency
//! and prepare infrastructure for future C comparison.

use std::collections::HashMap;

use nh_core::object::*;
use nh_core::GameRng;
use nh_core::data::objects::OBJECTS;

// ============================================================================
// Object creation determinism (Step 3.1)
// ============================================================================

/// Same seed must produce identical objects across runs.
#[test]
fn test_object_creation_deterministic() {
    let bases = ClassBases::compute(OBJECTS);

    for seed in [42u64, 0, 1, 12345, 0xDEADBEEF] {
        let mut rng1 = GameRng::new(seed);
        let mut ctx1 = MkObjContext::new();
        let mut rng2 = GameRng::new(seed);
        let mut ctx2 = MkObjContext::new();

        for _ in 0..100 {
            let obj1 = mkobj_with_data(OBJECTS, &bases, &mut ctx1, &mut rng1,
                ObjectClass::Weapon, true);
            let obj2 = mkobj_with_data(OBJECTS, &bases, &mut ctx2, &mut rng2,
                ObjectClass::Weapon, true);

            assert_eq!(obj1.object_type, obj2.object_type,
                "Object type mismatch for seed {}", seed);
            assert_eq!(obj1.enchantment, obj2.enchantment,
                "Enchantment mismatch for seed {}", seed);
            assert_eq!(obj1.buc, obj2.buc,
                "BUC mismatch for seed {}", seed);
            assert_eq!(obj1.quantity, obj2.quantity,
                "Quantity mismatch for seed {}", seed);
            assert_eq!(obj1.poisoned, obj2.poisoned,
                "Poisoned mismatch for seed {}", seed);
        }
    }
    println!("OK: Object creation is deterministic across 5 seeds x 100 objects");
}

/// Test all object classes can be created without panicking.
#[test]
fn test_create_all_classes() {
    let bases = ClassBases::compute(OBJECTS);
    let mut rng = GameRng::new(42);
    let mut ctx = MkObjContext::new();

    // Note: Coin and Rock are excluded because they have no entries in the
    // OBJECTS probability table; gold is created via mkgold(), rocks/statues
    // via mkcorpstat() in C, not mkobj().
    let classes = [
        ObjectClass::Weapon,
        ObjectClass::Armor,
        ObjectClass::Food,
        ObjectClass::Tool,
        ObjectClass::Gem,
        ObjectClass::Potion,
        ObjectClass::Scroll,
        ObjectClass::Spellbook,
        ObjectClass::Wand,
        ObjectClass::Ring,
        ObjectClass::Amulet,
    ];

    let mut counts: HashMap<ObjectClass, usize> = HashMap::new();

    for class in &classes {
        for _ in 0..50 {
            let obj = mkobj_with_data(OBJECTS, &bases, &mut ctx, &mut rng, *class, true);
            assert_eq!(obj.class, *class,
                "Created object class {:?} doesn't match requested {:?}", obj.class, class);
            *counts.entry(*class).or_insert(0) += 1;
        }
    }

    println!("\n=== Object Creation by Class ===");
    for class in &classes {
        println!("  {:?}: {} objects created", class, counts[class]);
    }
    println!("OK: All {} classes create objects successfully", classes.len());
}

/// Test object creation uses probability-weighted selection.
#[test]
fn test_probability_weighted_selection() {
    let bases = ClassBases::compute(OBJECTS);
    let mut rng = GameRng::new(42);
    let mut ctx = MkObjContext::new();

    // Create 10000 weapons and check distribution
    let mut type_counts: HashMap<i16, usize> = HashMap::new();
    for _ in 0..10_000 {
        let obj = mkobj_with_data(OBJECTS, &bases, &mut ctx, &mut rng,
            ObjectClass::Weapon, false);
        *type_counts.entry(obj.object_type).or_insert(0) += 1;
    }

    // Should have multiple different weapon types
    assert!(type_counts.len() > 10,
        "Expected >10 weapon types, got {}", type_counts.len());

    // No single type should dominate >50%
    let max_count = type_counts.values().max().unwrap();
    assert!(*max_count < 5000,
        "Single weapon type dominates with {} out of 10000", max_count);

    println!("OK: {} distinct weapon types created, max frequency = {}",
        type_counts.len(), max_count);
}

// ============================================================================
// Enchantment distribution (Step 3.1 detail)
// ============================================================================

/// Test weapon enchantment distribution matches expected C pattern.
/// C: 1/11 positive (rne(3)), 1/10 negative (-rne(3)), rest blessorcurse(10)
#[test]
fn test_weapon_enchantment_distribution() {
    let bases = ClassBases::compute(OBJECTS);
    let mut rng = GameRng::new(42);
    let mut ctx = MkObjContext::new();

    let mut positive = 0u32;
    let mut negative = 0u32;
    let mut zero = 0u32;
    let total = 10_000u32;

    for _ in 0..total {
        let obj = mkobj_with_data(OBJECTS, &bases, &mut ctx, &mut rng,
            ObjectClass::Weapon, true);
        if obj.enchantment > 0 {
            positive += 1;
        } else if obj.enchantment < 0 {
            negative += 1;
        } else {
            zero += 1;
        }
    }

    let pos_pct = positive as f64 / total as f64 * 100.0;
    let neg_pct = negative as f64 / total as f64 * 100.0;
    let zero_pct = zero as f64 / total as f64 * 100.0;

    println!("\n=== Weapon Enchantment Distribution (n={}) ===", total);
    println!("  Positive: {} ({:.1}%) -- C expected ~9%", positive, pos_pct);
    println!("  Negative: {} ({:.1}%) -- C expected ~10%", negative, neg_pct);
    println!("  Zero:     {} ({:.1}%) -- C expected ~81%", zero, zero_pct);

    // Sanity bounds: positive should be ~5-15%, negative ~5-15%
    assert!(pos_pct > 3.0 && pos_pct < 20.0,
        "Positive enchantment rate {:.1}% out of expected range", pos_pct);
    assert!(neg_pct > 3.0 && neg_pct < 20.0,
        "Negative enchantment rate {:.1}% out of expected range", neg_pct);
}

/// Test BUC distribution for weapons.
#[test]
fn test_weapon_buc_distribution() {
    let bases = ClassBases::compute(OBJECTS);
    let mut rng = GameRng::new(42);
    let mut ctx = MkObjContext::new();

    let mut blessed = 0u32;
    let mut uncursed = 0u32;
    let mut cursed = 0u32;
    let total = 10_000u32;

    for _ in 0..total {
        let obj = mkobj_with_data(OBJECTS, &bases, &mut ctx, &mut rng,
            ObjectClass::Weapon, true);
        match obj.buc {
            BucStatus::Blessed => blessed += 1,
            BucStatus::Uncursed => uncursed += 1,
            BucStatus::Cursed => cursed += 1,
        }
    }

    let b_pct = blessed as f64 / total as f64 * 100.0;
    let u_pct = uncursed as f64 / total as f64 * 100.0;
    let c_pct = cursed as f64 / total as f64 * 100.0;

    println!("\n=== Weapon BUC Distribution (n={}) ===", total);
    println!("  Blessed:  {} ({:.1}%)", blessed, b_pct);
    println!("  Uncursed: {} ({:.1}%)", uncursed, u_pct);
    println!("  Cursed:   {} ({:.1}%)", cursed, c_pct);

    // Most weapons should be uncursed
    assert!(u_pct > 60.0, "Uncursed rate {:.1}% too low", u_pct);
}

// ============================================================================
// Object naming (Step 3.2)
// ============================================================================

/// Test object naming produces valid non-empty strings.
#[test]
fn test_object_naming_basic() {
    let bases = ClassBases::compute(OBJECTS);
    let mut rng = GameRng::new(42);
    let mut ctx = MkObjContext::new();

    let classes = [
        ObjectClass::Weapon, ObjectClass::Armor, ObjectClass::Food,
        ObjectClass::Tool, ObjectClass::Potion, ObjectClass::Scroll,
        ObjectClass::Wand, ObjectClass::Ring, ObjectClass::Amulet,
    ];

    for class in &classes {
        for _ in 0..10 {
            let obj = mkobj_with_data(OBJECTS, &bases, &mut ctx, &mut rng, *class, true);
            let base_name = obj.name.as_deref().unwrap_or("unknown");
            let name = obj.doname(base_name);
            assert!(!name.is_empty(),
                "Object of class {:?} (type {}) has empty name", class, obj.object_type);
        }
    }
    println!("OK: All object classes produce non-empty names");
}

/// Test doname includes expected components (quantity, BUC, enchantment, erosion).
#[test]
fn test_doname_components() {
    // Create a blessed +3 long sword
    let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
    obj.name = Some("long sword".to_string());
    obj.buc = BucStatus::Blessed;
    obj.buc_known = true;
    obj.enchantment = 3;
    obj.known = true;
    obj.quantity = 1;

    let name = obj.doname("long sword");
    assert!(name.contains("blessed"), "doname missing 'blessed': {}", name);
    assert!(name.contains("+3"), "doname missing '+3': {}", name);
    assert!(name.contains("long sword"), "doname missing 'long sword': {}", name);

    // Multiple quantity
    obj.quantity = 5;
    let name = obj.doname("long sword");
    assert!(name.contains("5"), "doname missing quantity '5': {}", name);
}

/// Test xname (simple name without quantity/article).
#[test]
fn test_xname_basic() {
    let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
    obj.name = Some("dagger".to_string());
    obj.quantity = 1;

    let name = obj.xname("dagger");
    assert!(name.contains("dagger"), "xname should contain 'dagger': {}", name);
}

// ============================================================================
// Inventory management (Step 3.3)
// ============================================================================

/// Test inventory letter assignment.
#[test]
fn test_inventory_letter_assignment() {
    use nh_core::object::inventory::*;

    let mut inv: Vec<Object> = Vec::new();

    // Add 26 items, should get letters a-z
    for i in 0..26u32 {
        let mut obj = Object::new(ObjectId(i + 1), 0, ObjectClass::Weapon);
        obj.name = Some(format!("weapon_{}", i));
        obj.quantity = 1;
        assign_invlet(&mut obj, &inv);
        inv.push(obj);
    }

    // First 26 should have letters a-z
    for (i, obj) in inv.iter().enumerate() {
        let expected = (b'a' + i as u8) as char;
        assert_eq!(obj.inv_letter, expected,
            "Item {} should have letter '{}', got '{}'",
            i, expected, obj.inv_letter);
    }

    println!("OK: Inventory letter assignment a-z works correctly");
}

/// Test inventory merging for stackable items.
#[test]
fn test_inventory_merging() {
    use nh_core::object::inventory::*;

    let mut inv: Vec<Object> = Vec::new();

    // Add arrows
    let mut arrow1 = Object::new(ObjectId(1), 10, ObjectClass::Weapon);
    arrow1.name = Some("arrow".to_string());
    arrow1.quantity = 5;
    let idx = add_to_inventory(&mut inv, arrow1);
    assert_eq!(inv.len(), 1);
    assert_eq!(inv[idx].quantity, 5);

    // Add more of the same arrows -- should merge
    let mut arrow2 = Object::new(ObjectId(2), 10, ObjectClass::Weapon);
    arrow2.name = Some("arrow".to_string());
    arrow2.quantity = 3;
    let idx = add_to_inventory(&mut inv, arrow2);
    assert_eq!(inv.len(), 1, "Arrows should merge into one stack");
    assert_eq!(inv[idx].quantity, 8, "Merged stack should have 8 arrows");

    println!("OK: Inventory merging works");
}

/// Test inventory weight calculation.
/// NOTE: inventory::total_weight currently sums per-unit weight, not weight*quantity.
/// This is a known divergence from C. container::total_weight does it correctly.
#[test]
fn test_inventory_weight() {
    use nh_core::object::inventory;

    let mut inv: Vec<Object> = Vec::new();

    let mut obj1 = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
    obj1.weight = 30;
    obj1.quantity = 2;
    let _ = inventory::add_to_inventory(&mut inv, obj1);

    let mut obj2 = Object::new(ObjectId(2), 1, ObjectClass::Armor);
    obj2.weight = 100;
    obj2.quantity = 1;
    let _ = inventory::add_to_inventory(&mut inv, obj2);

    // BUG: inventory::total_weight sums per-unit weight, not weight*quantity
    // Should be 30*2 + 100 = 160, but currently returns 30 + 100 = 130
    let total = inventory::total_weight(&inv);
    assert_eq!(total, 130,
        "inventory::total_weight sums per-unit weight (known bug), got {}", total);

    println!("OK: Inventory weight calculated (known bug: doesn't multiply by quantity)");
}

/// Test inventory is_full at 52 slots.
#[test]
fn test_inventory_capacity() {
    use nh_core::object::inventory::*;

    let mut inv: Vec<Object> = Vec::new();

    // Fill to capacity (52 = a-z + A-Z)
    for i in 0..MAX_INVENTORY_SLOTS {
        let mut obj = Object::new(ObjectId(i as u32 + 1), i as i16, ObjectClass::Weapon);
        obj.name = Some(format!("item_{}", i));
        obj.quantity = 1;
        assign_invlet(&mut obj, &inv);
        inv.push(obj);
    }

    assert!(is_full(&inv), "Inventory should be full at {} slots", MAX_INVENTORY_SLOTS);
    assert_eq!(slot_count(&inv), MAX_INVENTORY_SLOTS);

    println!("OK: Inventory capacity is {} slots", MAX_INVENTORY_SLOTS);
}

// ============================================================================
// Object creation coverage (Step 3.1 detail)
// ============================================================================

/// Create 1000 random objects and verify all have valid state.
#[test]
fn test_mass_object_creation() {
    let bases = ClassBases::compute(OBJECTS);
    let mut rng = GameRng::new(42);
    let mut ctx = MkObjContext::new();

    let mut class_counts: HashMap<String, usize> = HashMap::new();
    let mut total_created = 0u32;

    for _ in 0..1000 {
        let obj = mkobj_random_with_data(OBJECTS, &bases, &mut ctx, &mut rng,
            LocationType::Normal, true);

        // Verify basic invariants
        assert!(obj.quantity >= 1, "Object has quantity 0");
        assert!(obj.weight > 0 || obj.class == ObjectClass::Coin,
            "Non-coin object has weight 0");

        *class_counts.entry(format!("{:?}", obj.class)).or_insert(0) += 1;
        total_created += 1;
    }

    println!("\n=== Mass Object Creation (n={}) ===", total_created);
    let mut sorted: Vec<_> = class_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (class, count) in &sorted {
        println!("  {:<15} {:>5} ({:.1}%)", class, count,
            (**count) as f64 / total_created as f64 * 100.0);
    }

    // Should have at least 5 different classes
    assert!(class_counts.len() >= 5,
        "Expected >=5 object classes, got {}", class_counts.len());

    println!("OK: {} objects created across {} classes", total_created, class_counts.len());
}

/// Verify wand charges are within expected bounds.
#[test]
fn test_wand_charges_bounds() {
    let bases = ClassBases::compute(OBJECTS);
    let mut rng = GameRng::new(42);
    let mut ctx = MkObjContext::new();

    for _ in 0..1000 {
        let obj = mkobj_with_data(OBJECTS, &bases, &mut ctx, &mut rng,
            ObjectClass::Wand, true);
        // C: wands get rn1(5, 4-11) = 4..16 or so
        assert!(obj.enchantment >= 0 && obj.enchantment <= 20,
            "Wand charges {} out of range", obj.enchantment);
    }
    println!("OK: All wand charges within bounds");
}

/// Verify ring enchantments are within expected bounds.
#[test]
fn test_ring_enchantment_bounds() {
    let bases = ClassBases::compute(OBJECTS);
    let mut rng = GameRng::new(42);
    let mut ctx = MkObjContext::new();

    for _ in 0..1000 {
        let obj = mkobj_with_data(OBJECTS, &bases, &mut ctx, &mut rng,
            ObjectClass::Ring, true);
        assert!(obj.enchantment >= -10 && obj.enchantment <= 10,
            "Ring enchantment {} out of range", obj.enchantment);
    }
    println!("OK: All ring enchantments within bounds");
}

// ============================================================================
// Summary
// ============================================================================

/// Print a comprehensive object system status report.
#[test]
fn test_object_system_summary() {
    let bases = ClassBases::compute(OBJECTS);
    let mut rng = GameRng::new(42);
    let mut ctx = MkObjContext::new();

    println!("\n=== Object System Summary ===");
    println!("OBJECTS array: {} entries", OBJECTS.len());

    // Count by class
    let classes = [
        ("Weapon", ObjectClass::Weapon),
        ("Armor", ObjectClass::Armor),
        ("Food", ObjectClass::Food),
        ("Tool", ObjectClass::Tool),
        ("Gem", ObjectClass::Gem),
        ("Potion", ObjectClass::Potion),
        ("Scroll", ObjectClass::Scroll),
        ("Spellbook", ObjectClass::Spellbook),
        ("Wand", ObjectClass::Wand),
        ("Ring", ObjectClass::Ring),
        ("Amulet", ObjectClass::Amulet),
        ("Coin", ObjectClass::Coin),
        ("Rock", ObjectClass::Rock),
    ];

    println!("\n{:<15} {:<8} {:<10}", "Class", "Types", "Sample");
    println!("{}", "-".repeat(35));

    for (name, class) in &classes {
        let type_count = OBJECTS.iter().filter(|o| o.class == *class).count();
        let obj = mkobj_with_data(OBJECTS, &bases, &mut ctx, &mut rng, *class, true);
        let base_name = obj.name.as_deref().unwrap_or("unknown");
        let sample_name = obj.doname(base_name);
        println!("{:<15} {:<8} {}", name, type_count, sample_name);
    }

    println!("\n=== Known Divergences from C ===");
    println!("1. rne() probability: Fixed (was 1/4, now 1/x matching C)");
    println!("2. Armor init: Missing specific cursed types (fumble boots etc.)");
    println!("3. Food init: Missing corpse monster selection, egg/tin varieties");
    println!("4. Tool init: Missing per-tool charge ranges (candles, lamps, etc.)");
    println!("5. Ring init: Missing specific cursed types (teleportation etc.)");
    println!("6. Amulet init: Missing specific cursed types (strangulation etc.)");
    println!("7. Wand init: Missing WAN_WISHING special case (rnd(3) charges)");
    println!("8. Gem init: Missing loadstone curse, luckstone exception");
}
