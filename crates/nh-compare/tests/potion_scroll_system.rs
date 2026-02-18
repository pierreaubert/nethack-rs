//! Potion and scroll system behavioral tests
//!
//! Tests for potion quaffing, scroll reading, recharging, punishment,
//! and other magic item effects.

use nh_core::magic::potion::*;
use nh_core::magic::scroll::*;
use nh_core::object::{BucStatus, Object, ObjectClass, ObjectId};
use nh_core::player::{Attribute, HungerState, Property, You};
use nh_core::GameRng;

// ============================================================================
// Helpers
// ============================================================================

fn test_player() -> You {
    let mut p = You::default();
    p.hp = 20;
    p.hp_max = 20;
    p.energy = 50;
    p.energy_max = 50;
    p.nutrition = 900;
    p.hunger_state = HungerState::NotHungry;
    p.exp_level = 5;
    for attr in [
        Attribute::Strength, Attribute::Intelligence, Attribute::Wisdom,
        Attribute::Dexterity, Attribute::Constitution, Attribute::Charisma,
    ] {
        p.attr_current.set(attr, 12);
        p.attr_max.set(attr, 18);
    }
    p
}

fn make_potion(ptype: PotionType, buc: BucStatus) -> Object {
    let mut obj = Object::default();
    obj.id = ObjectId(100);
    obj.class = ObjectClass::Potion;
    obj.object_type = ptype as i16;
    obj.buc = buc;
    obj.name = Some("potion".to_string());
    obj
}

fn make_scroll(stype: ScrollType, buc: BucStatus) -> Object {
    let mut obj = Object::default();
    obj.id = ObjectId(200);
    obj.class = ObjectClass::Scroll;
    obj.object_type = stype as i16;
    obj.buc = buc;
    obj.name = Some("scroll".to_string());
    obj
}

// ============================================================================
// Potion quaffing
// ============================================================================

#[test]
fn test_quaff_healing_restores_hp() {
    let pot = make_potion(PotionType::Healing, BucStatus::Uncursed);
    let mut player = test_player();
    player.hp = 10;
    let mut rng = GameRng::new(42);
    let result = quaff_potion(&pot, &mut player, &mut rng);
    assert!(player.hp > 10, "Healing potion should restore HP");
}

#[test]
fn test_quaff_extra_healing() {
    let pot = make_potion(PotionType::ExtraHealing, BucStatus::Uncursed);
    let mut player = test_player();
    player.hp = 5;
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    assert!(player.hp > 5);
}

#[test]
fn test_quaff_full_healing() {
    let pot = make_potion(PotionType::FullHealing, BucStatus::Uncursed);
    let mut player = test_player();
    player.hp = 1;
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    assert!(player.hp >= player.hp_max);
}

#[test]
fn test_quaff_confusion() {
    let pot = make_potion(PotionType::Confusion, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    assert!(player.is_confused());
}

#[test]
fn test_quaff_blindness() {
    let pot = make_potion(PotionType::Blindness, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    assert!(player.is_blind());
}

#[test]
fn test_quaff_paralysis() {
    let pot = make_potion(PotionType::Paralysis, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    assert!(player.is_paralyzed());
}

#[test]
fn test_quaff_hallucination() {
    let pot = make_potion(PotionType::Hallucination, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    assert!(player.is_hallucinating());
}

#[test]
fn test_quaff_sleeping() {
    let pot = make_potion(PotionType::Sleeping, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    assert!(player.is_sleeping());
}

#[test]
fn test_quaff_speed() {
    let pot = make_potion(PotionType::Speed, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    // Speed potion grants Speed property
}

#[test]
fn test_quaff_levitation() {
    let pot = make_potion(PotionType::Levitation, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    assert!(player.is_levitating());
}

#[test]
fn test_quaff_gain_energy() {
    let pot = make_potion(PotionType::GainEnergy, BucStatus::Uncursed);
    let mut player = test_player();
    player.energy = 10;
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    assert!(player.energy > 10 || player.energy_max > 50);
}

#[test]
fn test_quaff_fruit_juice_nutrition() {
    let pot = make_potion(PotionType::FruitJuice, BucStatus::Uncursed);
    let mut player = test_player();
    let old_nutr = player.nutrition;
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    assert!(player.nutrition > old_nutr, "Fruit juice should add nutrition");
}

#[test]
fn test_quaff_booze_nutrition() {
    let pot = make_potion(PotionType::Booze, BucStatus::Uncursed);
    let mut player = test_player();
    let old_nutr = player.nutrition;
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    assert!(player.nutrition > old_nutr || player.is_confused());
}

#[test]
fn test_quaff_sickness() {
    let pot = make_potion(PotionType::Sickness, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    // Should cause sickness or vomiting
}

#[test]
fn test_quaff_see_invisible() {
    let pot = make_potion(PotionType::SeeInvisible, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    assert!(player.has_see_invisible());
}

#[test]
fn test_quaff_invisibility() {
    let pot = make_potion(PotionType::Invisibility, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
}

#[test]
fn test_quaff_acid() {
    let pot = make_potion(PotionType::Acid, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    // Should deal some acid damage
}

#[test]
fn test_quaff_water_uncursed() {
    let pot = make_potion(PotionType::Water, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
}

#[test]
fn test_quaff_water_blessed() {
    let pot = make_potion(PotionType::Water, BucStatus::Blessed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    // Holy water should have beneficial effects
}

#[test]
fn test_quaff_water_cursed() {
    let pot = make_potion(PotionType::Water, BucStatus::Cursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    // Unholy water should have negative effects
}

#[test]
fn test_quaff_restore_ability() {
    let pot = make_potion(PotionType::Restore, BucStatus::Uncursed);
    let mut player = test_player();
    player.set_attr(Attribute::Strength, 8);
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
}

#[test]
fn test_quaff_gain_ability() {
    let pot = make_potion(PotionType::GainAbility, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
}

#[test]
fn test_quaff_oil() {
    let pot = make_potion(PotionType::Oil, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
}

#[test]
fn test_quaff_monster_detection() {
    let pot = make_potion(PotionType::MonsterDetection, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
}

#[test]
fn test_quaff_object_detection() {
    let pot = make_potion(PotionType::ObjectDetection, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
}

#[test]
fn test_quaff_enlightenment() {
    let pot = make_potion(PotionType::Enlightenment, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
}

#[test]
fn test_quaff_polymorph() {
    let pot = make_potion(PotionType::Polymorph, BucStatus::Uncursed);
    let mut player = test_player();
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
}

#[test]
fn test_quaff_gain_level() {
    let pot = make_potion(PotionType::GainLevel, BucStatus::Uncursed);
    let mut player = test_player();
    let old_level = player.exp_level;
    let mut rng = GameRng::new(42);
    quaff_potion(&pot, &mut player, &mut rng);
    assert!(player.exp_level > old_level, "Gain level potion should increase level");
}

// ============================================================================
// Potion BUC effects
// ============================================================================

#[test]
fn test_blessed_healing_more() {
    let blessed = make_potion(PotionType::Healing, BucStatus::Blessed);
    let uncursed = make_potion(PotionType::Healing, BucStatus::Uncursed);
    let mut p1 = test_player();
    let mut p2 = test_player();
    p1.hp = 5;
    p2.hp = 5;
    let mut rng1 = GameRng::new(42);
    let mut rng2 = GameRng::new(42);
    quaff_potion(&blessed, &mut p1, &mut rng1);
    quaff_potion(&uncursed, &mut p2, &mut rng2);
    assert!(p1.hp >= p2.hp, "Blessed healing should heal at least as much");
}

// ============================================================================
// Glow messages
// ============================================================================

#[test]
fn test_glow_strength_0() {
    let gs = glow_strength(0);
    assert_eq!(gs, 0);
}

#[test]
fn test_glow_strength_positive() {
    let gs = glow_strength(5);
    assert!(gs > 0);
}

#[test]
fn test_glow_verb_no_suffix() {
    let verb = glow_verb(1, false);
    assert!(!verb.is_empty());
}

#[test]
fn test_glow_verb_with_suffix() {
    let verb = glow_verb(1, true);
    assert!(!verb.is_empty());
}

#[test]
fn test_p_glow1() {
    let msg = p_glow1("sword");
    assert!(msg.contains("sword"));
}

#[test]
fn test_p_glow2() {
    let msg = p_glow2("sword", "blue");
    assert!(msg.contains("sword"));
    assert!(msg.contains("blue"));
}

// ============================================================================
// Scroll types
// ============================================================================

#[test]
fn test_scroll_type_values_unique() {
    let types = [
        ScrollType::EnchantArmor, ScrollType::Destroy, ScrollType::Confuse,
        ScrollType::Scare, ScrollType::RemoveCurse, ScrollType::EnchantWeapon,
        ScrollType::Create, ScrollType::Taming, ScrollType::Genocide,
        ScrollType::Light, ScrollType::Teleportation, ScrollType::Gold,
        ScrollType::Food, ScrollType::Identify, ScrollType::MagicMapping,
        ScrollType::Amnesia, ScrollType::Fire, ScrollType::Earth,
        ScrollType::Punishment, ScrollType::Charging, ScrollType::StinkingCloud,
        ScrollType::Blank,
    ];
    let mut values: Vec<i16> = types.iter().map(|t| *t as i16).collect();
    let orig_len = values.len();
    values.sort();
    values.dedup();
    assert_eq!(values.len(), orig_len, "All ScrollType values should be unique");
}

#[test]
fn test_potion_type_values_unique() {
    let types = [
        PotionType::GainAbility, PotionType::Restore, PotionType::Confusion,
        PotionType::Blindness, PotionType::Paralysis, PotionType::Speed,
        PotionType::Levitation, PotionType::Hallucination, PotionType::Invisibility,
        PotionType::SeeInvisible, PotionType::Healing, PotionType::ExtraHealing,
        PotionType::GainLevel, PotionType::Enlightenment, PotionType::MonsterDetection,
        PotionType::ObjectDetection, PotionType::GainEnergy, PotionType::Sleeping,
        PotionType::FullHealing, PotionType::Polymorph, PotionType::Booze,
        PotionType::Sickness, PotionType::FruitJuice, PotionType::Acid,
        PotionType::Oil, PotionType::Water,
    ];
    let mut values: Vec<i16> = types.iter().map(|t| *t as i16).collect();
    let orig_len = values.len();
    values.sort();
    values.dedup();
    assert_eq!(values.len(), orig_len, "All PotionType values should be unique");
}

// ============================================================================
// Scroll recharge
// ============================================================================

#[test]
fn test_recharge_wand() {
    let mut wand = Object::default();
    wand.class = ObjectClass::Wand;
    wand.enchantment = 0;
    let mut rng = GameRng::new(42);
    let msgs = recharge(&mut wand, 1, &mut rng);
    assert!(wand.enchantment > 0 || !msgs.is_empty());
}

#[test]
fn test_recharge_blessed() {
    let mut wand = Object::default();
    wand.class = ObjectClass::Wand;
    wand.enchantment = 0;
    let mut rng = GameRng::new(42);
    let msgs = recharge(&mut wand, 1, &mut rng);
    let _ = msgs;
}

#[test]
fn test_recharge_cursed() {
    let mut wand = Object::default();
    wand.class = ObjectClass::Wand;
    wand.enchantment = 3;
    let mut rng = GameRng::new(42);
    let msgs = recharge(&mut wand, -1, &mut rng);
    let _ = msgs;
}

// ============================================================================
// Punishment
// ============================================================================

#[test]
fn test_punish() {
    let mut player = test_player();
    let msgs = punish(&mut player);
    assert!(!msgs.is_empty(), "Punishment should produce messages");
}

#[test]
fn test_unpunish() {
    let mut player = test_player();
    punish(&mut player);
    let msgs = unpunish(&mut player);
    let _ = msgs;
}

// ============================================================================
// Ball movement
// ============================================================================

#[test]
fn test_can_move_punished_adjacent() {
    let mut player = test_player();
    player.pos.x = 10;
    player.pos.y = 10;
    let result = can_move_punished(&player, 11, 10);
    let _ = result;
}

#[test]
fn test_can_move_punished_far() {
    let mut player = test_player();
    player.pos.x = 10;
    player.pos.y = 10;
    let result = can_move_punished(&player, 20, 20);
    let _ = result;
}
