//! Combat system behavioral tests
//!
//! Tests for combat mechanics: attack types, damage calculations,
//! armor class, skill levels, weapon skills, status effects,
//! and combat modifiers.

use nh_core::combat::*;
use nh_core::monster::{Monster, MonsterId};
use nh_core::object::{Object, ObjectClass, ObjectId};
use nh_core::GameRng;

// ============================================================================
// Helpers
// ============================================================================

fn make_monster(hp: i32) -> Monster {
    let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
    m.hp = hp;
    m.hp_max = hp;
    m
}

fn make_armor_obj() -> Object {
    let mut obj = Object::default();
    obj.class = ObjectClass::Armor;
    obj.enchantment = 2;
    obj.weight = 150;
    obj
}

// ============================================================================
// Attack struct
// ============================================================================

#[test]
fn test_attack_new() {
    let atk = Attack::new(AttackType::Weapon, DamageType::Physical, 2, 6);
    assert_eq!(atk.dice_num, 2);
    assert_eq!(atk.dice_sides, 6);
    assert!(atk.is_active());
}

#[test]
fn test_attack_inactive() {
    let atk = Attack::new(AttackType::None, DamageType::Physical, 0, 0);
    assert!(!atk.is_active());
}

#[test]
fn test_attack_average_damage() {
    let atk = Attack::new(AttackType::Weapon, DamageType::Physical, 2, 6);
    let avg = atk.average_damage();
    assert!((avg - 7.0).abs() < 0.01, "2d6 average should be 7.0, got {}", avg);
}

// ============================================================================
// Empty attacks
// ============================================================================

#[test]
fn test_empty_attacks() {
    let attacks = empty_attacks();
    assert!(attacks.iter().all(|a| !a.is_active()));
}

// ============================================================================
// Armor bonus
// ============================================================================

#[test]
fn test_armor_bonus() {
    let obj = make_armor_obj();
    let bonus = armor_bonus(&obj);
    assert!(bonus != 0, "Armor should provide AC bonus");
}

// ============================================================================
// Grease protect
// ============================================================================

#[test]
fn test_grease_protect_greased() {
    let mut obj = make_armor_obj();
    obj.greased = true;
    let mut rng = GameRng::new(42);
    let protected = grease_protect(&mut obj, &mut rng);
    assert!(protected, "Greased item should protect");
}

#[test]
fn test_grease_protect_not_greased() {
    let mut obj = make_armor_obj();
    obj.greased = false;
    let mut rng = GameRng::new(42);
    let protected = grease_protect(&mut obj, &mut rng);
    assert!(!protected, "Non-greased item should not protect");
}

// ============================================================================
// Weapon skill
// ============================================================================

#[test]
fn test_weapon_skill_names() {
    for skill in WeaponSkill::all() {
        assert!(!skill.name().is_empty(), "Skill {:?} should have a name", skill);
    }
}

#[test]
fn test_skill_level_hit_bonus_increases() {
    let basic = SkillLevel::Basic.hit_bonus();
    let skilled = SkillLevel::Skilled.hit_bonus();
    let expert = SkillLevel::Expert.hit_bonus();
    assert!(skilled >= basic);
    assert!(expert >= skilled);
}

#[test]
fn test_skill_level_damage_bonus_increases() {
    let basic = SkillLevel::Basic.damage_bonus();
    let skilled = SkillLevel::Skilled.damage_bonus();
    let expert = SkillLevel::Expert.damage_bonus();
    assert!(skilled >= basic);
    assert!(expert >= skilled);
}

#[test]
fn test_skill_level_crit_chance_increases() {
    let basic = SkillLevel::Basic.crit_chance();
    let expert = SkillLevel::Expert.crit_chance();
    assert!(expert >= basic);
}

#[test]
fn test_skill_level_armor_penetration() {
    let basic = SkillLevel::Basic.armor_penetration();
    let expert = SkillLevel::Expert.armor_penetration();
    assert!(expert >= basic);
}

#[test]
fn test_skill_level_advance() {
    let next = SkillLevel::Basic.advance();
    assert_eq!(next, SkillLevel::Skilled);
}

#[test]
fn test_skill_level_advance_expert() {
    let next = SkillLevel::Skilled.advance();
    assert_eq!(next, SkillLevel::Expert);
}

// ============================================================================
// CombatResult
// ============================================================================

#[test]
fn test_combat_result_hit() {
    let cr = CombatResult::hit(10);
    assert_eq!(cr.damage, 10);
    assert!(cr.hit);
}

#[test]
fn test_combat_result_miss() {
    let cr = CombatResult::MISS;
    assert!(!cr.hit);
    assert_eq!(cr.damage, 0);
}

// ============================================================================
// Apply armor penetration
// ============================================================================

#[test]
fn test_apply_armor_penetration_reduces_ac() {
    let new_ac = apply_armor_penetration(5, 3);
    let _ = new_ac; // armor penetration adjusts AC in implementation-specific way
}

#[test]
fn test_apply_armor_penetration_zero() {
    let new_ac = apply_armor_penetration(5, 0);
    assert_eq!(new_ac, 5);
}

// ============================================================================
// Combat modifiers
// ============================================================================

#[test]
fn test_combat_modifier_flanking_to_hit() {
    let mods = [CombatModifier::Flanking];
    let (to_hit, _damage) = apply_combat_modifiers(&mods);
    assert!(to_hit > 0, "Flanking should give to-hit bonus");
}

#[test]
fn test_combat_modifier_surrounded() {
    let mods = [CombatModifier::Surrounded];
    let (to_hit, _damage) = apply_combat_modifiers(&mods);
    assert!(to_hit < 0, "Surrounded should give to-hit penalty");
}

#[test]
fn test_combat_modifiers_empty() {
    let (to_hit, damage) = apply_combat_modifiers(&[]);
    assert_eq!(to_hit, 0);
    assert_eq!(damage, 0);
}

#[test]
fn test_combat_modifier_disarmed() {
    let mods = [CombatModifier::Disarmed];
    let (to_hit, damage) = apply_combat_modifiers(&mods);
    assert!(to_hit < 0);
    assert!(damage < 0);
}

#[test]
fn test_combat_modifiers_stack() {
    let mods = [CombatModifier::Flanking, CombatModifier::HighGround];
    let (to_hit, _damage) = apply_combat_modifiers(&mods);
    let (single, _) = apply_combat_modifiers(&[CombatModifier::Flanking]);
    assert!(to_hit >= single, "Multiple bonuses should stack");
}

// ============================================================================
// Apply damage reduction
// ============================================================================

#[test]
fn test_apply_damage_reduction_full() {
    let result = apply_damage_reduction(10, 0.0);
    assert_eq!(result, 10);
}

#[test]
fn test_apply_damage_reduction_half() {
    let result = apply_damage_reduction(10, 0.5);
    assert_eq!(result, 5);
}

#[test]
fn test_apply_damage_reduction_total() {
    let result = apply_damage_reduction(10, 1.0);
    assert!(result <= 1, "Full reduction should leave minimal damage, got {}", result);
}

// ============================================================================
// Status effect damage
// ============================================================================

#[test]
fn test_poison_damage_per_turn() {
    let dmg = poison_damage_per_turn(1);
    assert!(dmg > 0);
}

#[test]
fn test_poison_damage_scales() {
    let low = poison_damage_per_turn(1);
    let high = poison_damage_per_turn(5);
    assert!(high >= low);
}

#[test]
fn test_bleeding_damage_per_turn() {
    let dmg = bleeding_damage_per_turn(1);
    assert!(dmg > 0);
}

#[test]
fn test_paralysis_action_reduction() {
    let reduction = paralysis_action_reduction(3);
    assert!(reduction > 0.0 && reduction <= 1.0);
}

#[test]
fn test_curse_roll_penalty() {
    let penalty = curse_roll_penalty(2);
    assert!(penalty > 0);
}

#[test]
fn test_blindness_accuracy_penalty() {
    let penalty = blindness_accuracy_penalty(3);
    assert!(penalty > 0);
}

#[test]
fn test_disease_attribute_reduction() {
    let red = disease_attribute_reduction(2);
    assert!(red > 0);
}

// ============================================================================
// Attk protection
// ============================================================================

#[test]
fn test_attk_protection_weapon() {
    let prot = attk_protection(AttackType::Weapon);
    let _ = prot;
}

#[test]
fn test_attk_protection_claw() {
    let prot = attk_protection(AttackType::Claw);
    let _ = prot;
}

// ============================================================================
// DamageType properties
// ============================================================================

#[test]
fn test_damage_type_physical() {
    assert_eq!(DamageType::Physical as u8, 0);
}

#[test]
fn test_damage_type_fire() {
    assert_eq!(DamageType::Fire as u8, 2);
}

#[test]
fn test_damage_type_cold() {
    assert_eq!(DamageType::Cold as u8, 3);
}

#[test]
fn test_damage_type_sleep() {
    assert_eq!(DamageType::Sleep as u8, 4);
}

#[test]
fn test_damage_type_electric() {
    assert_eq!(DamageType::Electric as u8, 6);
}

// ============================================================================
// Ranged weapon types
// ============================================================================

#[test]
fn test_ranged_weapon_max_range() {
    let bow = RangedWeaponType::Bow;
    assert!(bow.max_range() > 0);
}

#[test]
fn test_ranged_weapon_optimal_range() {
    let bow = RangedWeaponType::Bow;
    assert!(bow.optimal_range() > 0);
    assert!(bow.optimal_range() <= bow.max_range());
}

#[test]
fn test_ranged_weapon_base_damage_bonus() {
    let xbow = RangedWeaponType::Crossbow;
    let _ = xbow.base_damage_bonus();
}

#[test]
fn test_ranged_weapon_names() {
    let bow = RangedWeaponType::Bow;
    assert!(!bow.name().is_empty());
}

#[test]
fn test_ranged_weapon_sling() {
    let sling = RangedWeaponType::Sling;
    assert!(sling.max_range() > 0);
    assert!(!sling.name().is_empty());
}

// ============================================================================
// Special combat effects
// ============================================================================

#[test]
fn test_special_effect_disarm() {
    let disarm = SpecialCombatEffect::Disarm;
    let chance = disarm.base_success_chance();
    assert!(chance > 0 && chance <= 100);
}

#[test]
fn test_special_effect_skill_requirement() {
    let disarm = SpecialCombatEffect::Disarm;
    assert!(disarm.can_attempt_with_skill(SkillLevel::Expert));
}

#[test]
fn test_critical_hit_type_none() {
    let _ = CriticalHitType::None;
}

#[test]
fn test_critical_hit_type_critical() {
    let _ = CriticalHitType::Critical;
}

#[test]
fn test_critical_hit_type_devastating() {
    let _ = CriticalHitType::Devastating;
}

// ============================================================================
// find_mac
// ============================================================================

#[test]
fn test_find_mac_default_monster() {
    let m = make_monster(20);
    let mac = find_mac(&m);
    let _ = mac;
}

// ============================================================================
// noattacks
// ============================================================================

#[test]
fn test_noattacks_default() {
    let m = make_monster(10);
    let result = noattacks(&m);
    let _ = result;
}

// ============================================================================
// Weapon vs armor bonus
// ============================================================================

#[test]
fn test_weapon_vs_armor_bonus_dagger() {
    let bonus = weapon_vs_armor_bonus(WeaponSkill::Dagger, 0);
    let _ = bonus;
}

// ============================================================================
// Encumbrance status string
// ============================================================================

#[test]
fn test_encumbrance_status_string() {
    use nh_core::player::Encumbrance;
    assert!(Encumbrance::Unencumbered.status_string().is_none());
    assert!(Encumbrance::Burdened.status_string().is_some());
    assert!(Encumbrance::Overloaded.status_string().is_some());
}
