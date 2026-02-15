//! Phase 28: Naming, Identification, and Container Behavioral Tests
//!
//! Verifies artifact definitions, object naming, BUC identification,
//! discovery tracking, container operations, and object name formatting.

use nh_core::data::artifacts::{ARTIFACTS, Alignment, get_artifact};
use nh_core::magic::identification::{
    IdentificationKnowledge, IdentificationLevel, identify_from_scroll,
};
use nh_core::object::{
    BucStatus, DiscoveryState, Object, ObjectClass, ObjectId,
    container_weight, doname, put_in_container, xname,
};

// ============================================================================
// Helpers
// ============================================================================

fn make_container_obj(object_type: i16) -> Object {
    let mut obj = Object::default();
    obj.id = ObjectId(1);
    obj.class = ObjectClass::Tool;
    obj.object_type = object_type;
    obj.weight = 15;
    obj.quantity = 1;
    obj
}

fn make_item_for_container(id: u32) -> Object {
    let mut obj = Object::default();
    obj.id = ObjectId(id);
    obj.class = ObjectClass::Weapon;
    obj.object_type = 1;
    obj.weight = 10;
    obj.quantity = 1;
    obj
}

// ============================================================================
// Test 1: Artifact list exists with substantial entries
// ============================================================================

#[test]
fn test_artifact_list_exists() {
    // NetHack 3.6.7 has at least 20 artifacts (both regular and quest)
    assert!(
        ARTIFACTS.len() >= 20,
        "Expected at least 20 artifacts, found {}",
        ARTIFACTS.len()
    );

    // Known artifacts should be present
    let names: Vec<&str> = ARTIFACTS.iter().map(|a| a.name).collect();
    assert!(names.contains(&"Excalibur"), "Excalibur should be in artifact list");
    assert!(names.contains(&"Stormbringer"), "Stormbringer should be in artifact list");
    assert!(names.contains(&"Mjollnir"), "Mjollnir should be in artifact list");
    assert!(names.contains(&"Sting"), "Sting should be in artifact list");
    assert!(names.contains(&"Vorpal Blade"), "Vorpal Blade should be in artifact list");
}

// ============================================================================
// Test 2: Artifacts have name fields and valid base types
// ============================================================================

#[test]
fn test_artifact_has_name() {
    for art in ARTIFACTS.iter() {
        // Every artifact must have a non-empty name
        assert!(
            !art.name.is_empty(),
            "Artifact should have a non-empty name"
        );

        // Every artifact must have a cost > 0
        assert!(
            art.cost > 0,
            "Artifact '{}' should have a positive cost, got {}",
            art.name,
            art.cost
        );
    }

    // Verify get_artifact can find by name
    let excal = get_artifact("Excalibur");
    assert!(excal.is_some(), "Should find Excalibur by name");
    assert_eq!(excal.unwrap().name, "Excalibur");

    let sting = get_artifact("Sting");
    assert!(sting.is_some(), "Should find Sting by name");
    assert_eq!(sting.unwrap().name, "Sting");
}

// ============================================================================
// Test 3: Artifacts have alignment restrictions
// ============================================================================

#[test]
fn test_artifact_alignment() {
    // Excalibur is Lawful
    let excal = get_artifact("Excalibur").expect("Excalibur must exist");
    assert_eq!(
        excal.alignment,
        Alignment::Lawful,
        "Excalibur should be Lawful"
    );

    // Stormbringer is Chaotic
    let storm = get_artifact("Stormbringer").expect("Stormbringer must exist");
    assert_eq!(
        storm.alignment,
        Alignment::Chaotic,
        "Stormbringer should be Chaotic"
    );

    // Mjollnir is Neutral
    let mjol = get_artifact("Mjollnir").expect("Mjollnir must exist");
    assert_eq!(
        mjol.alignment,
        Alignment::Neutral,
        "Mjollnir should be Neutral"
    );

    // Frost Brand is unaligned (None)
    let fb = get_artifact("Frost Brand").expect("Frost Brand must exist");
    assert_eq!(
        fb.alignment,
        Alignment::None,
        "Frost Brand should be unaligned (None)"
    );
}

// ============================================================================
// Test 4: Objects can be named by player
// ============================================================================

#[test]
fn test_object_naming() {
    use nh_core::action::name::{NamingResult, do_oname, oname};

    // Basic naming succeeds
    let mut obj = Object::default();
    let result = oname(&mut obj, "my sword");
    assert_eq!(result, NamingResult::Named("my sword".to_string()));
    assert_eq!(obj.name, Some("my sword".to_string()));

    // Naming an artifact is rejected
    let mut artifact_obj = Object::default();
    artifact_obj.artifact = 1;
    let result = oname(&mut artifact_obj, "new name");
    assert!(matches!(result, NamingResult::Rejected(_)));

    // Naming with an artifact name is rejected by do_oname
    let mut obj2 = Object::default();
    let result = do_oname(&mut obj2, "Excalibur");
    assert!(
        matches!(result, NamingResult::Rejected(_)),
        "Naming a normal object 'Excalibur' should be rejected"
    );

    // Empty name removes existing name
    let mut obj3 = Object::default();
    obj3.name = Some("old name".to_string());
    let result = oname(&mut obj3, "");
    assert_eq!(result, NamingResult::Named(String::new()));
    assert_eq!(obj3.name, None);
}

// ============================================================================
// Test 5: Objects track known/unknown/identified state
// ============================================================================

#[test]
fn test_object_identification_state() {
    let mut knowledge = IdentificationKnowledge::new();

    // Initially unknown
    assert!(!knowledge.is_known(42));

    // Identify via scroll
    let result = identify_from_scroll(&mut knowledge, 42);
    assert!(result.identified);
    assert!(result.new_knowledge);
    assert!(knowledge.is_known(42));

    // Verify the level is Identified
    let item_k = knowledge.get_knowledge(42).unwrap();
    assert_eq!(item_k.identification_level, IdentificationLevel::Identified);

    // Object-level known flag
    let mut obj = Object::default();
    assert!(!obj.known, "New object should not be known");
    obj.known = true;
    assert!(obj.known, "Object should now be known");

    // desc_known flag
    assert!(!obj.desc_known, "New object should not have desc_known");
    obj.desc_known = true;
    assert!(obj.desc_known);
}

// ============================================================================
// Test 6: BUC status tracking
// ============================================================================

#[test]
fn test_buc_identification() {
    // Default BUC is uncursed
    let obj = Object::default();
    assert_eq!(obj.buc, BucStatus::Uncursed);
    assert!(!obj.buc_known, "BUC should be unknown initially");

    // Set BUC and verify
    let mut blessed_obj = Object::default();
    blessed_obj.buc = BucStatus::Blessed;
    blessed_obj.buc_known = true;
    assert!(blessed_obj.is_blessed());
    assert!(!blessed_obj.is_cursed());
    assert_eq!(blessed_obj.buc.as_str(), "blessed");
    assert_eq!(blessed_obj.buc.sign(), 1);

    let mut cursed_obj = Object::default();
    cursed_obj.buc = BucStatus::Cursed;
    cursed_obj.buc_known = true;
    assert!(cursed_obj.is_cursed());
    assert!(!cursed_obj.is_blessed());
    assert_eq!(cursed_obj.buc.as_str(), "cursed");
    assert_eq!(cursed_obj.buc.sign(), -1);

    // BUC opposite
    assert_eq!(BucStatus::Blessed.opposite(), BucStatus::Cursed);
    assert_eq!(BucStatus::Cursed.opposite(), BucStatus::Blessed);
    assert_eq!(BucStatus::Uncursed.opposite(), BucStatus::Uncursed);

    // buc_prefix shows status only when buc_known is true
    let mut unknown_buc = Object::default();
    unknown_buc.buc = BucStatus::Blessed;
    unknown_buc.buc_known = false;
    assert_eq!(unknown_buc.buc_prefix(), "", "Unknown BUC should show empty prefix");

    unknown_buc.buc_known = true;
    assert_eq!(unknown_buc.buc_prefix(), "blessed ");
}

// ============================================================================
// Test 7: Discovery state tracking system
// ============================================================================

#[test]
fn test_discovery_state() {
    let mut state = DiscoveryState::default();

    // Nothing discovered initially
    assert!(!state.is_discovered(100));
    assert_eq!(state.count(), 0);

    // Discover an object type
    let new = state.discover_object(100, 1);
    assert!(new, "First discovery should return true");
    assert!(state.is_discovered(100));
    assert_eq!(state.count(), 1);

    // Duplicate discovery returns false
    let dup = state.discover_object(100, 2);
    assert!(!dup, "Duplicate discovery should return false");
    assert_eq!(state.count(), 1);

    // Discover another type
    state.discover_object(200, 3);
    assert_eq!(state.count(), 2);
    assert!(state.knows_object(200));

    // Undiscover
    let removed = state.undiscover_object(100);
    assert!(removed);
    assert!(!state.is_discovered(100));
    assert_eq!(state.count(), 1);

    // Clear all
    state.clear();
    assert_eq!(state.count(), 0);
    assert!(!state.is_discovered(200));
}

// ============================================================================
// Test 8: Container operations (put in, take out, count)
// ============================================================================

#[test]
fn test_container_operations() {
    let mut chest = make_container_obj(361); // Chest
    assert!(chest.is_container(), "Chest should be a container");
    assert!(chest.contents.is_empty(), "New chest should be empty");

    // Put items in
    let item1 = make_item_for_container(100);
    let result = put_in_container(&mut chest, item1);
    assert!(
        matches!(result, nh_core::object::ContainerResult::Success),
        "Putting item in chest should succeed"
    );
    assert_eq!(chest.contents.len(), 1);

    // Put a different item type
    let mut item2 = make_item_for_container(101);
    item2.object_type = 2; // Different type so it does not merge
    let result2 = put_in_container(&mut chest, item2);
    assert!(matches!(result2, nh_core::object::ContainerResult::Success));
    assert_eq!(chest.contents.len(), 2);

    // Locked container rejects items
    let mut locked = make_container_obj(361);
    locked.locked = true;
    let item3 = make_item_for_container(102);
    let result3 = put_in_container(&mut locked, item3);
    assert!(matches!(result3, nh_core::object::ContainerResult::Locked));
}

// ============================================================================
// Test 9: Bag of Holding object type exists and weight reduction works
// ============================================================================

#[test]
fn test_bag_of_holding_exists() {
    // Bag of Holding = object_type 365
    let mut bag = make_container_obj(365);
    bag.buc = BucStatus::Uncursed;
    assert!(bag.is_container(), "Bag of Holding should be a container");

    // Put a heavy item in
    let mut heavy = make_item_for_container(200);
    heavy.weight = 200;
    put_in_container(&mut bag, heavy);

    // Uncursed BoH halves contained weight
    // Base 15 + 200/2 = 115
    let w = container_weight(&bag);
    assert_eq!(w, 115, "Uncursed BoH weight: base(15) + contents(200)/2 = 115, got {}", w);

    // Blessed BoH quarters contained weight
    bag.buc = BucStatus::Blessed;
    let w_blessed = container_weight(&bag);
    assert_eq!(
        w_blessed, 65,
        "Blessed BoH weight: base(15) + contents(200)/4 = 65, got {}",
        w_blessed
    );

    // Cursed BoH doubles contained weight
    bag.buc = BucStatus::Cursed;
    let w_cursed = container_weight(&bag);
    assert_eq!(
        w_cursed, 415,
        "Cursed BoH weight: base(15) + contents(200)*2 = 415, got {}",
        w_cursed
    );

    // Cannot put BoH inside BoH
    let inner_boh = make_container_obj(365);
    let result = put_in_container(&mut bag, inner_boh);
    assert!(
        !matches!(result, nh_core::object::ContainerResult::Success),
        "Should not allow BoH inside BoH"
    );
}

// ============================================================================
// Test 10: doname() produces readable name with modifiers
// ============================================================================

#[test]
fn test_object_doname() {
    // Plain weapon
    let obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
    let name = obj.doname("long sword");
    assert!(
        name.contains("long sword"),
        "doname should include base name, got: {}",
        name
    );

    // Known blessed +2 weapon
    let mut known_obj = Object::new(ObjectId(2), 1, ObjectClass::Weapon);
    known_obj.known = true;
    known_obj.buc = BucStatus::Blessed;
    known_obj.buc_known = true;
    known_obj.enchantment = 2;
    let name = known_obj.doname("long sword");
    assert!(name.contains("blessed"), "doname should show BUC, got: {}", name);
    assert!(name.contains("+2"), "doname should show enchantment, got: {}", name);
    assert!(name.contains("long sword"), "doname should show base name, got: {}", name);

    // Multiple quantity
    let mut stack = Object::new(ObjectId(3), 1, ObjectClass::Weapon);
    stack.quantity = 5;
    let name = stack.doname("arrow");
    assert!(name.contains("5"), "doname should show quantity, got: {}", name);
    // Module-level doname function also works
    let name2 = doname(&stack, "arrow");
    assert!(name2.contains("5"), "Module doname should show quantity, got: {}", name2);
}

// ============================================================================
// Test 11: xname() produces base name without BUC/enchantment
// ============================================================================

#[test]
fn test_object_xname() {
    // Object::xname does NOT include BUC or enchantment;
    // it includes poisoned, greased, erosion, and named suffix.
    let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
    obj.known = true;
    obj.buc = BucStatus::Blessed;
    obj.buc_known = true;
    obj.enchantment = 3;

    let xn = obj.xname("long sword");
    assert!(
        xn.contains("long sword"),
        "xname should include base name, got: {}",
        xn
    );

    // Module-level xname (from objname.rs) does include BUC if known
    let modxn = xname(&obj, "long sword");
    assert!(
        modxn.contains("blessed"),
        "Module xname should include BUC if known, got: {}",
        modxn
    );

    // Verify xname works for a simple unknown object
    let simple = Object::new(ObjectId(2), 1, ObjectClass::Weapon);
    let xn_simple = simple.xname("dagger");
    assert_eq!(
        xn_simple, "dagger",
        "Simple xname should just be the base name, got: {}",
        xn_simple
    );
}

// ============================================================================
// Test 12: Eroded objects show erosion in name
// ============================================================================

#[test]
fn test_object_name_with_erosion() {
    // Weapon with rust (erosion1)
    let mut rusty = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
    rusty.erosion1 = 1;
    let name = rusty.doname("long sword");
    assert!(
        name.contains("rusty"),
        "Eroded weapon should show 'rusty', got: {}",
        name
    );

    // Very rusty
    rusty.erosion1 = 2;
    let name2 = rusty.doname("long sword");
    assert!(
        name2.contains("very rusty"),
        "Very eroded weapon should show 'very rusty', got: {}",
        name2
    );

    // Thoroughly rusty
    rusty.erosion1 = 3;
    let name3 = rusty.doname("long sword");
    assert!(
        name3.contains("thoroughly rusty"),
        "Max eroded weapon should show 'thoroughly rusty', got: {}",
        name3
    );

    // Corroded weapon (erosion2)
    let mut corroded = Object::new(ObjectId(2), 1, ObjectClass::Weapon);
    corroded.erosion2 = 1;
    let name4 = corroded.doname("long sword");
    assert!(
        name4.contains("corroded"),
        "Corroded weapon should show 'corroded', got: {}",
        name4
    );

    // Non-metallic object uses "burnt" for erosion1
    let mut burnt = Object::new(ObjectId(3), 1, ObjectClass::Scroll);
    burnt.erosion1 = 1;
    let name5 = burnt.doname("scroll");
    assert!(
        name5.contains("burnt"),
        "Eroded scroll should show 'burnt', got: {}",
        name5
    );

    // Non-metallic object uses "rotted" for erosion2
    let mut rotted = Object::new(ObjectId(4), 1, ObjectClass::Scroll);
    rotted.erosion2 = 1;
    let name6 = rotted.doname("scroll");
    assert!(
        name6.contains("rotted"),
        "Rotted scroll should show 'rotted', got: {}",
        name6
    );

    // Erosion-proof + known shows "rustproof"
    let mut proof = Object::new(ObjectId(5), 1, ObjectClass::Weapon);
    proof.erosion_proof = true;
    proof.rust_known = true;
    let name7 = proof.doname("long sword");
    assert!(
        name7.contains("rustproof"),
        "Erosion-proof known weapon should show 'rustproof', got: {}",
        name7
    );
}
