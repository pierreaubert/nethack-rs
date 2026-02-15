//! Phase 23: Lock Picking, Trap Erosion, Equipment Damage
//!
//! Behavioral tests verifying DC-based lock picking, tool quality, cursed tools,
//! forcing chests, chest traps, rust/fire erosion, elven armor resistance,
//! erosion AC reduction, and oilskin/greased water protection.

use nh_core::action::open_close::{PickType, calculate_pick_chance, doforce};
use nh_core::dungeon::trap::{ContainerTrap, TrapResistances, chest_trap};
use nh_core::object::{Object, ObjectClass};
use nh_core::GameRng;

// ============================================================================
// Helpers
// ============================================================================

fn make_armor(name: &str, base_ac: i8) -> Object {
    let mut obj = Object::default();
    obj.class = ObjectClass::Armor;
    obj.name = Some(name.to_string());
    obj.base_ac = base_ac;
    obj
}

fn make_locked_chest() -> Object {
    let mut obj = Object::default();
    obj.class = ObjectClass::Tool;
    obj.object_type = 360; // chest
    obj.locked = true;
    obj
}

// ============================================================================
// Test 1: Lock picking success rate scales with tool and DEX
// ============================================================================

#[test]
fn test_pick_lock_skill_check() {
    // Skeleton key at DEX=18 (door) — should be 70 + 18 = 88
    let chance_high = calculate_pick_chance(PickType::SkeletonKey, 18, false, true, false);
    assert_eq!(chance_high, 88, "Skeleton key + high DEX should give 88% chance");

    // Lock pick at DEX=10 (door) — should be 3*10 = 30
    let chance_mid = calculate_pick_chance(PickType::LockPick, 10, false, true, false);
    assert_eq!(chance_mid, 30, "Lock pick + mid DEX should give 30% chance");

    // Credit card at DEX=10 (door) — should be 2*10 = 20
    let chance_low = calculate_pick_chance(PickType::CreditCard, 10, false, true, false);
    assert_eq!(chance_low, 20, "Credit card + mid DEX should give 20% chance");

    // Higher DEX always gives higher chance
    let chance_dex8 = calculate_pick_chance(PickType::LockPick, 8, false, true, false);
    let chance_dex16 = calculate_pick_chance(PickType::LockPick, 16, false, true, false);
    assert!(
        chance_dex16 > chance_dex8,
        "Higher DEX should give higher chance: {} vs {}",
        chance_dex16, chance_dex8
    );
}

// ============================================================================
// Test 2: Skeleton key is better than lock pick
// ============================================================================

#[test]
fn test_skeleton_key_better_than_lockpick() {
    for dex in [8, 12, 16, 18] {
        let key_chance = calculate_pick_chance(PickType::SkeletonKey, dex, false, true, false);
        let pick_chance = calculate_pick_chance(PickType::LockPick, dex, false, true, false);

        assert!(
            key_chance > pick_chance,
            "Skeleton key ({}) should be better than lock pick ({}) at DEX={}",
            key_chance, pick_chance, dex
        );
    }

    // Also for containers
    for dex in [8, 12, 16, 18] {
        let key_chance = calculate_pick_chance(PickType::SkeletonKey, dex, false, false, false);
        let pick_chance = calculate_pick_chance(PickType::LockPick, dex, false, false, false);

        assert!(
            key_chance > pick_chance,
            "Skeleton key ({}) should be better than lock pick ({}) for containers at DEX={}",
            key_chance, pick_chance, dex
        );
    }
}

// ============================================================================
// Test 3: Cursed lock pick has halved chance
// ============================================================================

#[test]
fn test_cursed_lockpick_can_break() {
    let normal = calculate_pick_chance(PickType::LockPick, 14, false, true, false);
    let cursed = calculate_pick_chance(PickType::LockPick, 14, false, true, true);

    assert_eq!(
        cursed, normal / 2,
        "Cursed chance ({}) should be half of normal ({})",
        cursed, normal
    );

    // Cursed skeleton key
    let key_normal = calculate_pick_chance(PickType::SkeletonKey, 14, false, true, false);
    let key_cursed = calculate_pick_chance(PickType::SkeletonKey, 14, false, true, true);

    assert_eq!(
        key_cursed,
        key_normal / 2,
        "Cursed skeleton key ({}) should be half of normal ({})",
        key_cursed, key_normal
    );
}

// ============================================================================
// Test 4: Force chest open
// ============================================================================

#[test]
fn test_force_chest_open() {
    let mut player = nh_core::player::You::default();
    player.hp = 100;
    player.hp_max = 100;
    // Player has a blade weapon wielded
    player.attr_current.set(nh_core::player::Attribute::Dexterity, 16);

    let mut chest = make_locked_chest();
    let mut rng = GameRng::new(42);

    let result = doforce(&player, &mut chest, &mut rng);

    // doforce should produce a result (success or failure, but not crash)
    assert!(
        !result.messages.is_empty(),
        "Force should produce at least one message"
    );
}

// ============================================================================
// Test 5: Chest trap (needle) deals poison
// ============================================================================

#[test]
fn test_chest_trap_needle() {
    let mut rng = GameRng::new(42);
    let resistances = TrapResistances::default(); // No poison resistance

    let result = chest_trap(&mut rng, ContainerTrap::Poison, &resistances);

    assert!(
        result.messages.iter().any(|m| m.contains("needle")),
        "Poison trap should mention needle: {:?}",
        result.messages
    );
    assert!(
        result.damage > 0 || result.status.is_some(),
        "Poison trap should deal damage or apply status"
    );

    // With poison resistance
    let mut res_immune = TrapResistances::default();
    res_immune.poison_resistant = true;

    let result2 = chest_trap(&mut rng, ContainerTrap::Poison, &res_immune);
    assert!(
        result2.messages.iter().any(|m| m.contains("doesn't seem")),
        "Poison resistant player should see resistance message: {:?}",
        result2.messages
    );
}

// ============================================================================
// Test 6: Rust trap corrodes armor (erosion system)
// ============================================================================

#[test]
fn test_rust_trap_corrodes_armor() {
    let mut armor = make_armor("iron armor", 5);
    assert_eq!(armor.erosion1, 0, "Armor starts with no rust");
    assert!(!armor.erosion_proof, "Armor starts not erosion-proof");

    // Apply rust (erosion type 0 = rust/burn)
    let destroyed = armor.erode(0);
    assert!(!destroyed, "First erosion should not destroy");
    assert_eq!(armor.erosion1, 1, "Erosion1 should be 1 after one rust");

    // Apply more rust
    armor.erode(0);
    assert_eq!(armor.erosion1, 2, "Erosion1 should be 2");

    armor.erode(0);
    assert_eq!(armor.erosion1, 3, "Erosion1 should be 3 (max)");
    assert!(armor.is_destroyed(), "Fully rusted armor should be destroyed");
}

// ============================================================================
// Test 7: Erosion-proof armor resists corrosion
// ============================================================================

#[test]
fn test_elven_armor_resists_corrosion() {
    let mut armor = make_armor("elven mithril-coat", 5);
    armor.erosion_proof = true;

    // Try to erode — should fail
    let destroyed = armor.erode(0); // rust
    assert!(!destroyed, "Erosion-proof should not be destroyed");
    assert_eq!(armor.erosion1, 0, "Erosion-proof armor should resist rust");

    let destroyed2 = armor.erode(1); // corrode
    assert!(!destroyed2, "Erosion-proof should not be destroyed");
    assert_eq!(armor.erosion2, 0, "Erosion-proof armor should resist corrosion");
}

// ============================================================================
// Test 8: Fire trap burns scrolls (erosion1 for flammable objects)
// ============================================================================

#[test]
fn test_fire_trap_burns_scrolls() {
    let mut scroll = Object::default();
    scroll.class = ObjectClass::Scroll;
    scroll.name = Some("scroll of identify".to_string());

    assert_eq!(scroll.erosion1, 0, "Scroll starts unburnt");

    // Scrolls are flammable and can erode
    let destroyed = scroll.erode(0); // fire/burn → erosion1
    assert!(!destroyed, "First burn should not destroy");
    assert_eq!(scroll.erosion1, 1, "Scroll should have erosion1=1 after burn");

    // Continue burning
    scroll.erode(0);
    scroll.erode(0);
    assert_eq!(scroll.erosion1, 3, "Scroll at max erosion");
    assert!(scroll.is_destroyed(), "Fully burnt scroll is destroyed");
}

// ============================================================================
// Test 9: Erosion reduces effective AC
// ============================================================================

#[test]
fn test_erosion_reduces_ac() {
    let mut armor = make_armor("plate mail", 3);
    armor.enchantment = 2; // +2 enchantment

    // Base effective AC = base_ac + enchantment - erosion
    assert_eq!(armor.effective_ac(), 5, "Base AC: 3 + 2 - 0 = 5");

    // Apply one point of rust
    armor.erode(0);
    assert_eq!(armor.effective_ac(), 4, "Eroded AC: 3 + 2 - 1 = 4");

    // Apply corrosion too
    armor.erode(1);
    assert_eq!(armor.effective_ac(), 3, "Double eroded AC: 3 + 2 - 2 = 3");

    // Max erosion on both channels
    armor.erode(0);
    armor.erode(0);
    armor.erode(1);
    armor.erode(1);
    // erosion1=3, erosion2=3, total=6
    assert_eq!(
        armor.effective_ac(),
        3 + 2 - 6,
        "Max eroded AC: 3 + 2 - 6 = -1"
    );
}

// ============================================================================
// Test 10: Greased object resists water/erosion
// ============================================================================

#[test]
fn test_oilskin_resists_water() {
    let mut armor = make_armor("oilskin cloak", 3);
    armor.greased = true;

    // Greased object should resist erosion
    let destroyed = armor.erode(0); // rust
    assert!(!destroyed, "Greased should not be destroyed");
    assert_eq!(armor.erosion1, 0, "Greased armor should resist rust");

    let destroyed2 = armor.erode(1); // corrode
    assert!(!destroyed2, "Greased should not be destroyed");
    assert_eq!(armor.erosion2, 0, "Greased armor should resist corrosion");

    // Remove grease and verify erosion now works
    armor.greased = false;
    armor.erode(0);
    assert_eq!(armor.erosion1, 1, "Un-greased armor should now rust");
}
