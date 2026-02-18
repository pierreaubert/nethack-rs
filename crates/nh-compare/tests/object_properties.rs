//! Object property and erosion behavioral tests
//!
//! Tests for object system: creation, BUC status, erosion, materials,
//! object classes, weight, naming, and identification.

use nh_core::object::*;

// ============================================================================
// Object creation
// ============================================================================

#[test]
fn test_object_default() {
    let obj = Object::default();
    assert_eq!(obj.id, ObjectId(0));
    assert_eq!(obj.quantity, 1);
}

#[test]
fn test_object_new() {
    let obj = Object::new(ObjectId(42), 16, ObjectClass::Weapon);
    assert_eq!(obj.id, ObjectId(42));
    assert_eq!(obj.class, ObjectClass::Weapon);
}

#[test]
fn test_object_id_none() {
    assert_eq!(ObjectId::NONE, ObjectId(0));
}

#[test]
fn test_object_id_next() {
    let id = ObjectId(5);
    assert_eq!(id.next(), ObjectId(6));
}

// ============================================================================
// BUC status
// ============================================================================

#[test]
fn test_buc_blessed_str() {
    assert_eq!(BucStatus::Blessed.as_str(), "blessed");
}

#[test]
fn test_buc_uncursed_str() {
    assert_eq!(BucStatus::Uncursed.as_str(), "uncursed");
}

#[test]
fn test_buc_cursed_str() {
    assert_eq!(BucStatus::Cursed.as_str(), "cursed");
}

#[test]
fn test_buc_sign_blessed() {
    assert_eq!(BucStatus::Blessed.sign(), 1);
}

#[test]
fn test_buc_sign_uncursed() {
    assert_eq!(BucStatus::Uncursed.sign(), 0);
}

#[test]
fn test_buc_sign_cursed() {
    assert_eq!(BucStatus::Cursed.sign(), -1);
}

#[test]
fn test_buc_default_is_uncursed() {
    let obj = Object::default();
    assert_eq!(obj.buc, BucStatus::Uncursed);
}

// ============================================================================
// Object classes
// ============================================================================

#[test]
fn test_object_class_weapon() {
    let mut obj = Object::default();
    obj.class = ObjectClass::Weapon;
    assert_eq!(obj.class, ObjectClass::Weapon);
}

#[test]
fn test_object_class_armor() {
    let mut obj = Object::default();
    obj.class = ObjectClass::Armor;
    assert_eq!(obj.class, ObjectClass::Armor);
}

#[test]
fn test_object_class_potion() {
    let mut obj = Object::default();
    obj.class = ObjectClass::Potion;
    assert_eq!(obj.class, ObjectClass::Potion);
}

#[test]
fn test_object_class_scroll() {
    let mut obj = Object::default();
    obj.class = ObjectClass::Scroll;
    assert_eq!(obj.class, ObjectClass::Scroll);
}

#[test]
fn test_object_class_wand() {
    let mut obj = Object::default();
    obj.class = ObjectClass::Wand;
    assert_eq!(obj.class, ObjectClass::Wand);
}

#[test]
fn test_object_class_ring() {
    let mut obj = Object::default();
    obj.class = ObjectClass::Ring;
    assert_eq!(obj.class, ObjectClass::Ring);
}

#[test]
fn test_object_class_amulet() {
    let mut obj = Object::default();
    obj.class = ObjectClass::Amulet;
    assert_eq!(obj.class, ObjectClass::Amulet);
}

#[test]
fn test_object_class_food() {
    let mut obj = Object::default();
    obj.class = ObjectClass::Food;
    assert_eq!(obj.class, ObjectClass::Food);
}

#[test]
fn test_object_class_gem() {
    let mut obj = Object::default();
    obj.class = ObjectClass::Gem;
    assert_eq!(obj.class, ObjectClass::Gem);
}

#[test]
fn test_object_class_tool() {
    let mut obj = Object::default();
    obj.class = ObjectClass::Tool;
    assert_eq!(obj.class, ObjectClass::Tool);
}

// ============================================================================
// Erosion
// ============================================================================

#[test]
fn test_erosion_default_zero() {
    let obj = Object::default();
    assert_eq!(obj.erosion1, 0);
    assert_eq!(obj.erosion2, 0);
}

#[test]
fn test_erosion_proof_default() {
    let obj = Object::default();
    assert!(!obj.erosion_proof);
}

#[test]
fn test_erosion1_range() {
    let mut obj = Object::default();
    obj.erosion1 = 3;
    assert_eq!(obj.erosion1, 3);
}

#[test]
fn test_erosion2_range() {
    let mut obj = Object::default();
    obj.erosion2 = 3;
    assert_eq!(obj.erosion2, 3);
}

#[test]
fn test_erosion_proof_blocks() {
    let mut obj = Object::default();
    obj.erosion_proof = true;
    assert!(obj.erosion_proof);
}

// ============================================================================
// Material enum
// ============================================================================

#[test]
fn test_material_iron_is_metallic() {
    assert!(Material::Iron.is_metallic());
}

#[test]
fn test_material_wood_not_metallic() {
    assert!(!Material::Wood.is_metallic());
}

#[test]
fn test_material_silver_is_metallic() {
    assert!(Material::Silver.is_metallic());
}

#[test]
fn test_material_gold_is_metallic() {
    assert!(Material::Gold.is_metallic());
}

#[test]
fn test_material_leather_not_metallic() {
    assert!(!Material::Leather.is_metallic());
}

#[test]
fn test_material_cloth_not_metallic() {
    assert!(!Material::Cloth.is_metallic());
}

#[test]
fn test_material_glass_not_metallic() {
    assert!(!Material::Glass.is_metallic());
}

#[test]
fn test_material_iron_rusts() {
    assert!(Material::Iron.rusts());
}

#[test]
fn test_material_wood_no_rust() {
    assert!(!Material::Wood.rusts());
}

#[test]
fn test_material_copper_corrodes() {
    assert!(Material::Copper.corrodes());
}

#[test]
fn test_material_wood_burns() {
    assert!(Material::Wood.burns());
}

#[test]
fn test_material_iron_no_burn() {
    assert!(!Material::Iron.burns());
}

#[test]
fn test_material_leather_rots() {
    assert!(Material::Leather.rots());
}

#[test]
fn test_material_iron_no_rot() {
    assert!(!Material::Iron.rots());
}

#[test]
fn test_material_default_is_iron() {
    assert_eq!(Material::default(), Material::Iron);
}

#[test]
fn test_material_values() {
    assert_eq!(Material::Liquid as u8, 1);
    assert_eq!(Material::Iron as u8, 11);
    assert_eq!(Material::Glass as u8, 19);
    assert_eq!(Material::Mineral as u8, 21);
}

// ============================================================================
// Object location
// ============================================================================

#[test]
fn test_object_location_default() {
    let obj = Object::default();
    assert_eq!(obj.location, ObjectLocation::Free);
}

#[test]
fn test_object_location_floor() {
    let mut obj = Object::default();
    obj.location = ObjectLocation::Floor;
    assert_eq!(obj.location, ObjectLocation::Floor);
}

#[test]
fn test_object_location_inventory() {
    let mut obj = Object::default();
    obj.location = ObjectLocation::PlayerInventory;
    assert_eq!(obj.location, ObjectLocation::PlayerInventory);
}

#[test]
fn test_object_location_contained() {
    let mut obj = Object::default();
    obj.location = ObjectLocation::Contained;
    assert_eq!(obj.location, ObjectLocation::Contained);
}

#[test]
fn test_object_location_monster() {
    let mut obj = Object::default();
    obj.location = ObjectLocation::MonsterInventory;
    assert_eq!(obj.location, ObjectLocation::MonsterInventory);
}

// ============================================================================
// Object properties
// ============================================================================

#[test]
fn test_object_enchantment() {
    let mut obj = Object::default();
    obj.enchantment = 3;
    assert_eq!(obj.enchantment, 3);
}

#[test]
fn test_object_negative_enchantment() {
    let mut obj = Object::default();
    obj.enchantment = -2;
    assert_eq!(obj.enchantment, -2);
}

#[test]
fn test_object_weight() {
    let mut obj = Object::default();
    obj.weight = 50;
    assert_eq!(obj.weight, 50);
}

#[test]
fn test_object_quantity_default() {
    let obj = Object::default();
    assert_eq!(obj.quantity, 1);
}

#[test]
fn test_object_quantity_multiple() {
    let mut obj = Object::default();
    obj.quantity = 5;
    assert_eq!(obj.quantity, 5);
}

#[test]
fn test_object_locked() {
    let mut obj = Object::default();
    obj.locked = true;
    assert!(obj.locked);
}

#[test]
fn test_object_broken() {
    let mut obj = Object::default();
    obj.broken = true;
    assert!(obj.broken);
}

#[test]
fn test_object_greased() {
    let mut obj = Object::default();
    obj.greased = true;
    assert!(obj.greased);
}

#[test]
fn test_object_known() {
    let mut obj = Object::default();
    obj.known = true;
    assert!(obj.known);
}

#[test]
fn test_object_desc_known() {
    let mut obj = Object::default();
    obj.desc_known = true;
    assert!(obj.desc_known);
}

#[test]
fn test_object_buc_known() {
    let mut obj = Object::default();
    obj.buc_known = true;
    assert!(obj.buc_known);
}

// ============================================================================
// Object name
// ============================================================================

#[test]
fn test_object_name_none_default() {
    let obj = Object::default();
    assert!(obj.name.is_none());
}

#[test]
fn test_object_name_set() {
    let mut obj = Object::default();
    obj.name = Some("Excalibur".to_string());
    assert_eq!(obj.name.as_deref(), Some("Excalibur"));
}

// ============================================================================
// Object position
// ============================================================================

#[test]
fn test_object_position() {
    let mut obj = Object::default();
    obj.x = 10;
    obj.y = 15;
    assert_eq!(obj.x, 10);
    assert_eq!(obj.y, 15);
}

// ============================================================================
// Inv letter
// ============================================================================

#[test]
fn test_inv_letter_default() {
    let obj = Object::default();
    assert_eq!(obj.inv_letter, '\0');
}

#[test]
fn test_inv_letter_set() {
    let mut obj = Object::default();
    obj.inv_letter = 'a';
    assert_eq!(obj.inv_letter, 'a');
}

// ============================================================================
// Artifact
// ============================================================================

#[test]
fn test_artifact_default() {
    let obj = Object::default();
    assert_eq!(obj.artifact, 0);
}

#[test]
fn test_artifact_set() {
    let mut obj = Object::default();
    obj.artifact = 5;
    assert_eq!(obj.artifact, 5);
}

// ============================================================================
// Recharged
// ============================================================================

#[test]
fn test_recharged_default() {
    let obj = Object::default();
    assert_eq!(obj.recharged, 0);
}

#[test]
fn test_recharged_set() {
    let mut obj = Object::default();
    obj.recharged = 2;
    assert_eq!(obj.recharged, 2);
}

// ============================================================================
// Object equality
// ============================================================================

#[test]
fn test_object_id_equality() {
    assert_eq!(ObjectId(42), ObjectId(42));
    assert_ne!(ObjectId(1), ObjectId(2));
}
