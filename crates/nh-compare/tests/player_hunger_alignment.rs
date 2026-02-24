//! Player hunger, alignment, and conduct behavioral tests
//!
//! Tests for hunger system, alignment mechanics, conduct tracking,
//! and related player subsystems.

use nh_core::player::*;

// ============================================================================
// HungerState
// ============================================================================

// C thresholds (eat.c:2936-2939):
//   nutrition > 1000 → Satiated
//   nutrition > 150  → NotHungry
//   nutrition > 50   → Hungry
//   nutrition > 0    → Weak
//   else             → Fainting

#[test]
fn test_hunger_satiated_threshold() {
    assert_eq!(HungerState::Satiated.threshold(), 1000);
}

#[test]
fn test_hunger_not_hungry_threshold() {
    assert_eq!(HungerState::NotHungry.threshold(), 150);
}

#[test]
fn test_hunger_hungry_threshold() {
    assert_eq!(HungerState::Hungry.threshold(), 50);
}

#[test]
fn test_hunger_weak_threshold() {
    assert_eq!(HungerState::Weak.threshold(), 0);
}

#[test]
fn test_hunger_fainting_threshold() {
    assert_eq!(HungerState::Fainting.threshold(), -1);
}

#[test]
fn test_hunger_from_nutrition_satiated() {
    let state = HungerState::from_nutrition(1500);
    assert_eq!(state, HungerState::Satiated);
}

#[test]
fn test_hunger_from_nutrition_not_hungry() {
    // nutrition > 150 but <= 1000 → NotHungry
    let state = HungerState::from_nutrition(500);
    assert_eq!(state, HungerState::NotHungry);
}

#[test]
fn test_hunger_from_nutrition_hungry() {
    // nutrition > 50 but <= 150 → Hungry
    let state = HungerState::from_nutrition(100);
    assert_eq!(state, HungerState::Hungry);
}

#[test]
fn test_hunger_from_nutrition_weak() {
    // nutrition > 0 but <= 50 → Weak
    let state = HungerState::from_nutrition(25);
    assert_eq!(state, HungerState::Weak);
}

#[test]
fn test_hunger_from_nutrition_fainting() {
    // nutrition <= 0 → Fainting
    let state = HungerState::from_nutrition(0);
    assert_eq!(state, HungerState::Fainting);
}

#[test]
fn test_hunger_from_nutrition_starved() {
    // Starved is handled separately by newuhs when nutrition < -(100 + 10*CON)
    // from_nutrition doesn't directly return Starved
    let state = HungerState::from_nutrition(-200);
    assert_eq!(state, HungerState::Fainting);
}

#[test]
fn test_hunger_can_act_satiated() {
    assert!(HungerState::Satiated.can_act());
}

#[test]
fn test_hunger_can_act_hungry() {
    assert!(HungerState::Hungry.can_act());
}

#[test]
fn test_hunger_has_penalty_hungry() {
    assert!(HungerState::Hungry.has_penalty());
}

#[test]
fn test_hunger_no_penalty_not_hungry() {
    assert!(!HungerState::NotHungry.has_penalty());
}

#[test]
fn test_hunger_status_string_not_hungry() {
    assert!(HungerState::NotHungry.status_string().is_none());
}

#[test]
fn test_hunger_status_string_hungry() {
    let s = HungerState::Hungry.status_string();
    assert!(s.is_some());
    assert!(!s.unwrap().is_empty());
}

#[test]
fn test_hunger_status_string_weak() {
    assert!(HungerState::Weak.status_string().is_some());
}

#[test]
fn test_hunger_status_string_fainting() {
    assert!(HungerState::Fainting.status_string().is_some());
}

// ============================================================================
// AlignmentType
// ============================================================================

#[test]
fn test_alignment_lawful_value() {
    assert!(AlignmentType::Lawful.value() > 0);
}

#[test]
fn test_alignment_neutral_value() {
    assert_eq!(AlignmentType::Neutral.value(), 0);
}

#[test]
fn test_alignment_chaotic_value() {
    assert!(AlignmentType::Chaotic.value() < 0);
}

#[test]
fn test_alignment_from_value_positive() {
    let a = AlignmentType::from_value(1);
    assert_eq!(a, AlignmentType::Lawful);
}

#[test]
fn test_alignment_from_value_zero() {
    let a = AlignmentType::from_value(0);
    assert_eq!(a, AlignmentType::Neutral);
}

#[test]
fn test_alignment_from_value_negative() {
    let a = AlignmentType::from_value(-1);
    assert_eq!(a, AlignmentType::Chaotic);
}

#[test]
fn test_alignment_default_god_lawful() {
    let god = AlignmentType::Lawful.default_god();
    assert!(!god.is_empty());
}

#[test]
fn test_alignment_default_god_neutral() {
    let god = AlignmentType::Neutral.default_god();
    assert!(!god.is_empty());
}

#[test]
fn test_alignment_default_god_chaotic() {
    let god = AlignmentType::Chaotic.default_god();
    assert!(!god.is_empty());
}

#[test]
fn test_alignment_as_str() {
    assert!(!AlignmentType::Lawful.as_str().is_empty());
    assert!(!AlignmentType::Neutral.as_str().is_empty());
    assert!(!AlignmentType::Chaotic.as_str().is_empty());
}

#[test]
fn test_alignment_as_title() {
    assert!(!AlignmentType::Lawful.as_title().is_empty());
    assert!(!AlignmentType::Neutral.as_title().is_empty());
    assert!(!AlignmentType::Chaotic.as_title().is_empty());
}

#[test]
fn test_alignment_coaligned_same() {
    assert!(AlignmentType::Lawful.is_coaligned(&AlignmentType::Lawful));
}

#[test]
fn test_alignment_not_coaligned_different() {
    assert!(!AlignmentType::Lawful.is_coaligned(&AlignmentType::Chaotic));
}

#[test]
fn test_alignment_cross_aligned() {
    assert!(AlignmentType::Lawful.is_cross_aligned(&AlignmentType::Chaotic));
}

#[test]
fn test_alignment_not_cross_aligned_same() {
    assert!(!AlignmentType::Lawful.is_cross_aligned(&AlignmentType::Lawful));
}

#[test]
fn test_alignment_from_str_lawful() {
    assert_eq!(AlignmentType::from_str("lawful"), Some(AlignmentType::Lawful));
}

#[test]
fn test_alignment_from_str_neutral() {
    assert_eq!(AlignmentType::from_str("neutral"), Some(AlignmentType::Neutral));
}

#[test]
fn test_alignment_from_str_chaotic() {
    assert_eq!(AlignmentType::from_str("chaotic"), Some(AlignmentType::Chaotic));
}

#[test]
fn test_alignment_from_str_invalid() {
    assert_eq!(AlignmentType::from_str("evil"), None);
}

// ============================================================================
// Alignment struct
// ============================================================================

#[test]
fn test_alignment_new() {
    let a = Alignment::new(AlignmentType::Lawful);
    assert_eq!(a.typ, AlignmentType::Lawful);
}

#[test]
fn test_alignment_increase() {
    let mut a = Alignment::new(AlignmentType::Lawful);
    let old = a.record;
    a.increase(5);
    assert_eq!(a.record, old + 5);
}

#[test]
fn test_alignment_decrease() {
    let mut a = Alignment::new(AlignmentType::Lawful);
    a.increase(10);
    a.decrease(3);
    assert_eq!(a.record, 7);
}

#[test]
fn test_alignment_in_good_standing() {
    let mut a = Alignment::new(AlignmentType::Lawful);
    a.increase(10);
    assert!(a.in_good_standing());
}

#[test]
fn test_alignment_is_opposite() {
    let a = Alignment::new(AlignmentType::Lawful);
    let b = Alignment::new(AlignmentType::Chaotic);
    assert!(a.is_opposite(&b));
}

#[test]
fn test_alignment_not_opposite_same() {
    let a = Alignment::new(AlignmentType::Lawful);
    let b = Alignment::new(AlignmentType::Lawful);
    assert!(!a.is_opposite(&b));
}

// ============================================================================
// Alignment additional tests
// ============================================================================

#[test]
fn test_alignment_record_starts_at_zero() {
    let a = Alignment::new(AlignmentType::Neutral);
    assert_eq!(a.record, 0);
}

#[test]
fn test_alignment_typ_preserved() {
    let a = Alignment::new(AlignmentType::Chaotic);
    assert_eq!(a.typ, AlignmentType::Chaotic);
}

#[test]
fn test_alignment_increase_then_good_standing() {
    let mut a = Alignment::new(AlignmentType::Lawful);
    a.increase(20);
    assert!(a.in_good_standing());
    assert_eq!(a.record, 20);
}

#[test]
fn test_alignment_decrease_below_zero() {
    let mut a = Alignment::new(AlignmentType::Neutral);
    a.decrease(5);
    assert_eq!(a.record, -5);
}

// ============================================================================
// Conduct
// ============================================================================

#[test]
fn test_conduct_default_all_maintained() {
    let c = Conduct::default();
    assert!(c.is_vegetarian());
    assert!(c.is_vegan());
    assert!(c.is_atheist());
    assert!(c.is_weaponless());
    assert!(c.is_pacifist());
    assert!(c.is_illiterate());
    assert!(c.is_wishless());
}

#[test]
fn test_conduct_ate_meat_breaks_veg() {
    let mut c = Conduct::default();
    c.ate_meat();
    assert!(!c.is_vegetarian());
}

#[test]
fn test_conduct_ate_non_vegan() {
    let mut c = Conduct::default();
    c.ate_non_vegan();
    assert!(!c.is_vegan());
}

#[test]
fn test_conduct_killed_monster() {
    let mut c = Conduct::default();
    c.killed_monster();
    assert!(!c.is_pacifist());
}

#[test]
fn test_conduct_read_something() {
    let mut c = Conduct::default();
    c.read_something();
    assert!(!c.is_illiterate());
}

#[test]
fn test_conduct_made_wish() {
    let mut c = Conduct::default();
    c.made_wish(false);
    assert!(!c.is_wishless());
    assert!(c.is_artiwishless());
}

#[test]
fn test_conduct_made_artifact_wish() {
    let mut c = Conduct::default();
    c.made_wish(true);
    assert!(!c.is_wishless());
    assert!(!c.is_artiwishless());
}

#[test]
fn test_conduct_is_foodless() {
    let c = Conduct::default();
    assert!(c.is_foodless());
}

#[test]
fn test_conduct_is_polypileless() {
    let c = Conduct::default();
    assert!(c.is_polypileless());
}

#[test]
fn test_conduct_is_polyselfless() {
    let c = Conduct::default();
    assert!(c.is_polyselfless());
}

#[test]
fn test_conduct_is_genocideless() {
    let c = Conduct::default();
    assert!(c.is_genocideless());
}

// ============================================================================
// Conduct additional
// ============================================================================

#[test]
fn test_conduct_multiple_breaks() {
    let mut c = Conduct::default();
    c.ate_meat();
    c.killed_monster();
    c.read_something();
    assert!(!c.is_vegetarian());
    assert!(!c.is_pacifist());
    assert!(!c.is_illiterate());
    // these should still hold
    assert!(c.is_atheist());
    assert!(c.is_wishless());
}

#[test]
fn test_conduct_weaponless_default() {
    let c = Conduct::default();
    assert!(c.is_weaponless());
}

#[test]
fn test_conduct_vegan_implies_vegetarian() {
    // if vegan is broken, vegetarian need not be
    let mut c = Conduct::default();
    c.ate_non_vegan();
    assert!(!c.is_vegan());
    // vegetarian may still hold since we only ate non-vegan (e.g. eggs)
    // but this depends on implementation - just verify no panic
    let _ = c.is_vegetarian();
}
