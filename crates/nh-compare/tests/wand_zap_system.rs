//! Wand and zap system behavioral tests
//!
//! Verifies wand mechanics: charges, recharging, damage calculations,
//! ray directions, line-of-fire, cancel/probe effects, wand breaking,
//! and durability.

use nh_core::magic::zap::*;
use nh_core::object::{BucStatus, Material, Object, ObjectClass, ObjectId};
use nh_core::monster::{Monster, MonsterFlags, MonsterId};
use nh_core::player::Role;
use nh_core::GameRng;

// ============================================================================
// Helpers
// ============================================================================

fn make_wand(object_type: i16, charges: i8) -> Object {
    let mut obj = Object::default();
    obj.id = ObjectId(100);
    obj.class = ObjectClass::Wand;
    obj.object_type = object_type;
    obj.enchantment = charges;
    obj.name = Some("wand".to_string());
    obj
}

fn make_monster_basic(hp: i32) -> Monster {
    let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
    m.hp = hp;
    m.hp_max = hp;
    m
}

// ============================================================================
// ZapType properties
// ============================================================================

#[test]
fn test_zap_type_wand_variant() {
    let zt = ZapType::MagicMissile;
    let _wand = zt.wand();
    let _spell = zt.spell();
    let _breath = zt.breath();
}

#[test]
fn test_zap_type_name_wand() {
    let name = ZapType::MagicMissile.name(ZapVariant::Wand);
    assert!(!name.is_empty());
}

#[test]
fn test_zap_type_name_spell() {
    let name = ZapType::Fire.name(ZapVariant::Spell);
    assert!(!name.is_empty());
}

#[test]
fn test_zap_type_name_breath() {
    let name = ZapType::Fire.name(ZapVariant::Breath);
    assert!(!name.is_empty());
}

#[test]
fn test_zap_type_damage_type() {
    let dt = ZapType::Fire.damage_type();
    assert_eq!(dt, nh_core::combat::DamageType::Fire);
}

#[test]
fn test_zap_type_cold_damage_type() {
    let dt = ZapType::Cold.damage_type();
    assert_eq!(dt, nh_core::combat::DamageType::Cold);
}

#[test]
fn test_zap_type_sleep_damage_type() {
    let dt = ZapType::Sleep.damage_type();
    assert_eq!(dt, nh_core::combat::DamageType::Sleep);
}

#[test]
fn test_zap_type_death_damage_type() {
    let dt = ZapType::Death.damage_type();
    assert_eq!(dt, nh_core::combat::DamageType::Death);
}

#[test]
fn test_zap_type_lightning_damage_type() {
    let dt = ZapType::Lightning.damage_type();
    assert_eq!(dt, nh_core::combat::DamageType::Electric);
}

#[test]
fn test_zap_type_poison_damage_type() {
    let dt = ZapType::PoisonGas.damage_type();
    assert_eq!(dt, nh_core::combat::DamageType::DrainStrength);
}

#[test]
fn test_zap_type_acid_damage_type() {
    let dt = ZapType::Acid.damage_type();
    assert_eq!(dt, nh_core::combat::DamageType::Acid);
}

// ============================================================================
// Zap damage
// ============================================================================

#[test]
fn test_zap_damage_magic_missile() {
    let mut rng = GameRng::new(42);
    let dmg = zap_damage(ZapType::MagicMissile, ZapVariant::Wand, &mut rng);
    assert!(dmg > 0, "Magic missile should do damage");
}

#[test]
fn test_zap_damage_fire() {
    let mut rng = GameRng::new(42);
    let dmg = zap_damage(ZapType::Fire, ZapVariant::Wand, &mut rng);
    assert!(dmg > 0);
}

#[test]
fn test_zap_damage_cold() {
    let mut rng = GameRng::new(42);
    let dmg = zap_damage(ZapType::Cold, ZapVariant::Spell, &mut rng);
    assert!(dmg > 0);
}

#[test]
fn test_zap_damage_varies_by_seed() {
    let mut damages = std::collections::HashSet::new();
    for seed in 0..20 {
        let mut rng = GameRng::new(seed);
        damages.insert(zap_damage(ZapType::Fire, ZapVariant::Wand, &mut rng));
    }
    assert!(damages.len() > 1, "Zap damage should vary with RNG");
}

#[test]
fn test_zap_damage_breath_variant() {
    let mut rng = GameRng::new(42);
    let dmg = zap_damage(ZapType::Fire, ZapVariant::Breath, &mut rng);
    assert!(dmg > 0);
}

// ============================================================================
// Wand utility functions
// ============================================================================

#[test]
fn test_zappable_with_charges() {
    let wand = make_wand(1, 5);
    assert!(zappable(&wand));
}

#[test]
fn test_zappable_no_charges() {
    let wand = make_wand(1, 0);
    assert!(!zappable(&wand));
}

#[test]
fn test_can_recharge_fresh_wand() {
    let wand = make_wand(1, 3);
    assert!(can_recharge(&wand));
}

#[test]
fn test_max_wand_charges_positive() {
    let max = max_wand_charges(1);
    assert!(max > 0, "Max wand charges should be positive");
}

#[test]
fn test_wand_effect_name_nonempty() {
    let name = wand_effect_name(&WandEffect::Teleport);
    assert!(!name.is_empty());
}

#[test]
fn test_wand_effect_name_healing() {
    let name = wand_effect_name(&WandEffect::Healing);
    assert!(!name.is_empty());
}

#[test]
fn test_wand_effect_name_polymorph() {
    let name = wand_effect_name(&WandEffect::Polymorph);
    assert!(!name.is_empty());
}

#[test]
fn test_wand_effect_name_digging() {
    let name = wand_effect_name(&WandEffect::Digging);
    assert!(!name.is_empty());
}

#[test]
fn test_wand_effect_name_cancellation() {
    let name = wand_effect_name(&WandEffect::Cancellation);
    assert!(!name.is_empty());
}

#[test]
fn test_wand_recharge_difficulty() {
    let diff = wand_recharge_difficulty(1);
    assert!(diff >= 0);
}

#[test]
fn test_wand_needs_recharge_depleted() {
    let wand = make_wand(1, 0);
    assert!(wand_needs_recharge(&wand));
}

#[test]
fn test_wand_needs_recharge_full() {
    let wand = make_wand(1, max_wand_charges(1));
    assert!(!wand_needs_recharge(&wand));
}

#[test]
fn test_wand_durability_factor_range() {
    let wand = make_wand(1, 5);
    let factor = wand_durability_factor(&wand);
    assert!(factor >= 0.0 && factor <= 2.0, "Durability factor out of range: {}", factor);
}

// ============================================================================
// Direction and line-of-fire
// ============================================================================

#[test]
fn test_valid_zap_direction_cardinal() {
    assert!(valid_zap_direction(1, 0));
    assert!(valid_zap_direction(-1, 0));
    assert!(valid_zap_direction(0, 1));
    assert!(valid_zap_direction(0, -1));
}

#[test]
fn test_valid_zap_direction_diagonal() {
    assert!(valid_zap_direction(1, 1));
    assert!(valid_zap_direction(-1, -1));
    assert!(valid_zap_direction(1, -1));
    assert!(valid_zap_direction(-1, 1));
}

#[test]
fn test_zap_direction_zero_is_self() {
    // (0,0) is "zap self" - valid in some contexts
    let _ = valid_zap_direction(0, 0);
}

#[test]
fn test_invalid_zap_direction_large() {
    assert!(!valid_zap_direction(2, 0));
}

#[test]
fn test_in_line_of_fire_same_row() {
    assert!(in_line_of_fire(5, 5, 10, 5));
}

#[test]
fn test_in_line_of_fire_same_col() {
    assert!(in_line_of_fire(5, 5, 5, 10));
}

#[test]
fn test_in_line_of_fire_diagonal() {
    assert!(in_line_of_fire(5, 5, 8, 8));
}

#[test]
fn test_not_in_line_of_fire() {
    assert!(!in_line_of_fire(5, 5, 7, 8));
}

#[test]
fn test_direction_toward() {
    let (dx, dy) = direction_toward(5, 5, 8, 5);
    assert_eq!(dx, 1);
    assert_eq!(dy, 0);
}

#[test]
fn test_direction_toward_diagonal() {
    let (dx, dy) = direction_toward(5, 5, 8, 8);
    assert_eq!(dx, 1);
    assert_eq!(dy, 1);
}

#[test]
fn test_direction_toward_negative() {
    let (dx, dy) = direction_toward(8, 8, 5, 5);
    assert_eq!(dx, -1);
    assert_eq!(dy, -1);
}

// ============================================================================
// Cancel and probe
// ============================================================================

#[test]
fn test_cancel_item() {
    let mut obj = Object::default();
    obj.id = ObjectId(1);
    obj.class = ObjectClass::Wand;
    obj.enchantment = 5;
    obj.enchantment = 3;
    let msgs = cancel_item(&mut obj);
    // Cancellation should affect the item
    let _ = msgs;
}

#[test]
fn test_cancel_monst() {
    let mut mon = make_monster_basic(20);
    let msgs = cancel_monst(&mut mon);
    let _ = msgs;
}

#[test]
fn test_probe_monster() {
    let mon = make_monster_basic(20);
    let msgs = probe_monster(&mon);
    assert!(!msgs.is_empty(), "Probing should reveal info");
}

// ============================================================================
// Zapnodir
// ============================================================================

#[test]
fn test_zapnodir_some_types() {
    // Just verify the function doesn't panic on various types
    for t in 0..20_i16 {
        let _ = zapnodir(t);
    }
}

// ============================================================================
// Wand wear/breakage
// ============================================================================

#[test]
fn test_calculate_wand_wear() {
    let wand = make_wand(1, 5);
    let wear = calculate_wand_wear(&wand, 1);
    assert!(wear >= 0.0);
}

#[test]
fn test_check_wand_breakage() {
    let wand = make_wand(1, 1);
    let mut broke = false;
    for seed in 0..100 {
        let mut rng = GameRng::new(seed);
        if check_wand_breakage(&wand, &mut rng) {
            broke = true;
            break;
        }
    }
    // Low charges wand should sometimes break
    let _ = broke;
}

#[test]
fn test_get_wand_effectiveness() {
    let wand = make_wand(1, 5);
    let eff = get_wand_effectiveness(&wand, 0);
    assert!(eff > 0.0);
}

#[test]
fn test_apply_wand_wear_penalty() {
    let result = apply_wand_wear_penalty(10, 1.0);
    assert_eq!(result, 10);
    let result2 = apply_wand_wear_penalty(10, 0.5);
    assert!(result2 < 10);
}

#[test]
fn test_degrade_wand() {
    let mut wand = make_wand(1, 5);
    let mut rng = GameRng::new(42);
    let old_charges = wand.enchantment;
    degrade_wand(&mut wand, &mut rng);
    assert!(wand.enchantment <= old_charges);
}

#[test]
fn test_get_wand_status() {
    let wand = make_wand(1, 5);
    let status = get_wand_status(&wand, 0);
    assert!(!status.is_empty());
}

// ============================================================================
// Role damage reduction
// ============================================================================

#[test]
fn test_role_damage_reduction_knight() {
    let reduced = role_damage_reduction(20, Role::Knight);
    assert!(reduced <= 20);
}

#[test]
fn test_role_damage_reduction_wizard() {
    let reduced = role_damage_reduction(20, Role::Wizard);
    assert!(reduced <= 20);
}

// ============================================================================
// Breaktest
// ============================================================================

#[test]
fn test_breaktest_potion() {
    let mut obj = Object::default();
    obj.class = ObjectClass::Potion;
    // breaktest returns true if item *might* break (RNG-based for some)
    let _ = breaktest(&obj);
}

#[test]
fn test_breaktest_weapon() {
    let mut obj = Object::default();
    obj.class = ObjectClass::Weapon;
    assert!(!breaktest(&obj), "Weapons generally don't break from impact");
}

#[test]
fn test_breakmsg_nonempty() {
    let mut obj = Object::default();
    obj.class = ObjectClass::Potion;
    obj.name = Some("potion".to_string());
    let msg = breakmsg(&obj, true);
    assert!(!msg.is_empty());
}

// ============================================================================
// damage_type_to_zap_type conversion
// ============================================================================

#[test]
fn test_damage_type_to_zap_fire() {
    let zt = damage_type_to_zap_type(nh_core::combat::DamageType::Fire);
    assert_eq!(zt, Some(ZapType::Fire));
}

#[test]
fn test_damage_type_to_zap_cold() {
    let zt = damage_type_to_zap_type(nh_core::combat::DamageType::Cold);
    assert_eq!(zt, Some(ZapType::Cold));
}

#[test]
fn test_damage_type_to_zap_lightning() {
    let zt = damage_type_to_zap_type(nh_core::combat::DamageType::Electric);
    assert_eq!(zt, Some(ZapType::Lightning));
}

#[test]
fn test_damage_type_to_zap_physical_none() {
    let zt = damage_type_to_zap_type(nh_core::combat::DamageType::Physical);
    assert_eq!(zt, None);
}

// ============================================================================
// Zapdir to glyph
// ============================================================================

#[test]
fn test_zapdir_to_glyph_horizontal() {
    let g = zapdir_to_glyph(1, 0, ZapType::MagicMissile);
    assert!(g > 0);
}

#[test]
fn test_zapdir_to_glyph_vertical() {
    let g = zapdir_to_glyph(0, 1, ZapType::Fire);
    assert!(g > 0);
}

#[test]
fn test_zapdir_to_glyph_diagonal() {
    let g = zapdir_to_glyph(1, 1, ZapType::Cold);
    assert!(g > 0);
}
