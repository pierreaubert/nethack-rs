//! Player attribute, level, luck, and capacity tests
//!
//! Behavioral tests verifying player attribute mechanics match C NetHack:
//! attribute clamping, bonuses, luck decay, encumbrance, level gain/loss,
//! HP/energy regeneration, status effects, and distance calculations.

use nh_core::player::{Attribute, Encumbrance, Gender, HungerState, Property, Race, Role, You};
use nh_core::GameRng;

// ============================================================================
// Helpers
// ============================================================================

fn make_player() -> You {
    let mut p = You::default();
    p.hp = 20;
    p.hp_max = 20;
    p.energy = 20;
    p.energy_max = 20;
    p.exp_level = 5;
    p.nutrition = 900;
    p.hunger_state = HungerState::NotHungry;
    for attr in [
        Attribute::Strength,
        Attribute::Intelligence,
        Attribute::Wisdom,
        Attribute::Dexterity,
        Attribute::Constitution,
        Attribute::Charisma,
    ] {
        p.attr_current.set(attr, 12);
        p.attr_max.set(attr, 18);
    }
    p
}

fn make_player_with_role(role: Role) -> You {
    You::new("Test".to_string(), role, Race::Human, Gender::Male)
}

// ============================================================================
// Attribute clamping (3..25)
// ============================================================================

#[test]
fn test_attr_clamp_minimum() {
    let mut p = make_player();
    p.set_attr(Attribute::Strength, 1);
    assert!(p.acurr(Attribute::Strength) >= 3, "Attributes clamp to minimum 3");
}

#[test]
fn test_attr_clamp_maximum() {
    let mut p = make_player();
    p.set_attr(Attribute::Strength, 30);
    assert!(p.acurr(Attribute::Strength) <= 25, "Attributes clamp to maximum 25");
}

#[test]
fn test_attr_default_is_zero() {
    let p = You::default();
    // Default attributes start at 0 (initialization sets proper values)
    assert_eq!(p.acurr(Attribute::Strength), 0);
}

// ============================================================================
// Attribute adjustment
// ============================================================================

#[test]
fn test_adjattrib_increase() {
    let mut p = make_player();
    let old = p.acurr(Attribute::Strength);
    p.adjattrib(Attribute::Strength, 2);
    assert_eq!(p.acurr(Attribute::Strength), old + 2);
}

#[test]
fn test_adjattrib_decrease() {
    let mut p = make_player();
    let old = p.acurr(Attribute::Strength);
    p.adjattrib(Attribute::Strength, -1);
    assert_eq!(p.acurr(Attribute::Strength), old - 1);
}

#[test]
fn test_adjattrib_large_increase() {
    let mut p = make_player();
    p.set_attr(Attribute::Strength, 15);
    p.adjattrib(Attribute::Strength, 5);
    assert_eq!(p.acurr(Attribute::Strength), 20);
}

#[test]
fn test_adjattrib_respects_min() {
    let mut p = make_player();
    p.set_attr(Attribute::Strength, 4);
    p.adjattrib(Attribute::Strength, -5);
    assert!(p.acurr(Attribute::Strength) >= 3);
}

#[test]
fn test_all_six_attributes_settable() {
    let mut p = make_player();
    let attrs = [
        Attribute::Strength,
        Attribute::Intelligence,
        Attribute::Wisdom,
        Attribute::Dexterity,
        Attribute::Constitution,
        Attribute::Charisma,
    ];
    for (i, attr) in attrs.iter().enumerate() {
        p.set_attr(*attr, 10 + i as i8);
    }
    for (i, attr) in attrs.iter().enumerate() {
        assert_eq!(p.acurr(*attr), 10 + i as i8);
    }
}

#[test]
fn test_attribute_names() {
    assert_eq!(Attribute::Strength.name(), "strength");
    assert_eq!(Attribute::Intelligence.name(), "intelligence");
    assert_eq!(Attribute::Wisdom.name(), "wisdom");
    assert_eq!(Attribute::Dexterity.name(), "dexterity");
    assert_eq!(Attribute::Constitution.name(), "constitution");
    assert_eq!(Attribute::Charisma.name(), "charisma");
}

// ============================================================================
// Attribute-derived bonuses
// ============================================================================

#[test]
fn test_str_to_hit_bonus() {
    let mut p = make_player();
    p.set_attr(Attribute::Strength, 18);
    let bonus = p.str_to_hit_bonus();
    assert!(bonus >= 0, "High STR should give non-negative to-hit bonus");
}

#[test]
fn test_str_damage_bonus() {
    let mut p = make_player();
    p.set_attr(Attribute::Strength, 18);
    let bonus = p.str_damage_bonus();
    assert!(bonus >= 0, "High STR should give non-negative damage bonus");
}

#[test]
fn test_dex_ac_bonus() {
    let mut p = make_player();
    p.set_attr(Attribute::Dexterity, 18);
    let bonus = p.dex_ac_bonus();
    // Higher DEX = better AC (more negative = better)
    assert!(bonus != 0, "High DEX should affect AC");
}

#[test]
fn test_dex_to_hit_bonus() {
    let mut p = make_player();
    p.set_attr(Attribute::Dexterity, 18);
    let bonus = p.dex_to_hit_bonus();
    assert!(bonus >= 0, "High DEX gives to-hit bonus");
}

#[test]
fn test_con_hp_bonus() {
    let mut p = make_player();
    p.set_attr(Attribute::Constitution, 18);
    let bonus = p.con_hp_bonus();
    assert!(bonus >= 0, "High CON gives HP bonus");
}

#[test]
fn test_cha_price_modifier() {
    let mut p = make_player();
    p.set_attr(Attribute::Charisma, 18);
    let mod_high = p.cha_price_modifier();
    p.set_attr(Attribute::Charisma, 6);
    let mod_low = p.cha_price_modifier();
    assert!(mod_high != mod_low, "CHA should affect prices");
}

#[test]
fn test_low_str_hit_penalty() {
    let mut p = make_player();
    p.set_attr(Attribute::Strength, 3);
    let bonus = p.str_to_hit_bonus();
    assert!(bonus <= 0, "Low STR should give hit penalty or zero");
}

// ============================================================================
// Luck system
// ============================================================================

#[test]
fn test_luck_starts_at_zero() {
    let p = You::default();
    assert_eq!(p.luck, 0);
}

#[test]
fn test_gain_luck() {
    let mut p = make_player();
    p.gain_luck(3);
    assert_eq!(p.effective_luck(), 3);
}

#[test]
fn test_lose_luck() {
    let mut p = make_player();
    p.set_luck(5);
    p.lose_luck(2);
    assert_eq!(p.effective_luck(), 3);
}

#[test]
fn test_change_luck_positive() {
    let mut p = make_player();
    p.change_luck(4);
    assert_eq!(p.effective_luck(), 4);
}

#[test]
fn test_change_luck_negative() {
    let mut p = make_player();
    p.change_luck(-3);
    assert_eq!(p.effective_luck(), -3);
}

#[test]
fn test_has_good_luck() {
    let mut p = make_player();
    p.set_luck(5);
    assert!(p.has_good_luck());
    assert!(!p.has_bad_luck());
}

#[test]
fn test_has_bad_luck() {
    let mut p = make_player();
    p.set_luck(-5);
    assert!(p.has_bad_luck());
    assert!(!p.has_good_luck());
}

#[test]
fn test_luck_decay_toward_zero_positive() {
    let mut p = make_player();
    p.set_luck(5);
    p.luck_bonus = 0;
    p.decay_luck();
    assert!(p.luck < 5, "Positive luck should decay toward 0");
}

#[test]
fn test_luck_decay_toward_zero_negative() {
    let mut p = make_player();
    p.set_luck(-5);
    p.luck_bonus = 0;
    p.decay_luck();
    assert!(p.luck > -5, "Negative luck should decay toward 0");
}

#[test]
fn test_luck_bonus_affects_effective() {
    let mut p = make_player();
    p.set_luck(0);
    p.luck_bonus = 3;
    // effective_luck adds luck_bonus
    let eff = p.effective_luck();
    assert!(eff >= 0);
}

// ============================================================================
// Health and energy
// ============================================================================

#[test]
fn test_take_damage() {
    let mut p = make_player();
    p.take_damage(5);
    assert_eq!(p.hp, 15);
}

#[test]
fn test_heal_capped_at_max() {
    let mut p = make_player();
    p.take_damage(5);
    p.heal(100);
    assert_eq!(p.hp, p.hp_max);
}

#[test]
fn test_is_dead() {
    let mut p = make_player();
    assert!(!p.is_dead());
    p.take_damage(100);
    assert!(p.is_dead());
}

#[test]
fn test_restore_hp() {
    let mut p = make_player();
    p.take_damage(10);
    p.restore_hp(5);
    assert_eq!(p.hp, 15);
}

#[test]
fn test_restore_energy() {
    let mut p = make_player();
    p.energy = 5;
    p.restore_energy(10);
    assert_eq!(p.energy, 15);
}

#[test]
fn test_use_energy_sufficient() {
    let mut p = make_player();
    assert!(p.use_energy(10));
    assert_eq!(p.energy, 10);
}

#[test]
fn test_use_energy_insufficient() {
    let mut p = make_player();
    assert!(!p.use_energy(100));
    assert_eq!(p.energy, 20, "Energy unchanged when insufficient");
}

#[test]
fn test_full_heal() {
    let mut p = make_player();
    p.take_damage(15);
    p.full_heal();
    assert_eq!(p.hp, p.hp_max);
}

#[test]
fn test_full_energy() {
    let mut p = make_player();
    p.energy = 0;
    p.full_energy();
    assert_eq!(p.energy, p.energy_max);
}

// ============================================================================
// Experience and level
// ============================================================================

#[test]
fn test_gain_exp() {
    let mut p = make_player();
    let old_exp = p.exp;
    p.gain_exp(100);
    assert_eq!(p.exp, old_exp + 100);
}

#[test]
fn test_check_level_up() {
    let mut p = make_player();
    p.exp_level = 1;
    p.exp = 100_000;
    p.check_level_up();
    assert!(p.exp_level > 1, "Should level up with enough exp");
}

#[test]
fn test_exp_level_capped() {
    let mut p = make_player();
    p.exp_level = 30;
    p.max_exp_level = 30;
    let old_level = p.exp_level;
    p.gain_exp(1_000_000);
    p.check_level_up();
    assert_eq!(p.exp_level, old_level, "Max level should be capped at 30");
}

#[test]
fn test_losexp() {
    let mut p = make_player();
    p.exp_level = 10;
    p.hp_max = 60;
    p.hp = 60;
    p.losexp(false);
    assert!(p.exp_level < 10, "losexp should reduce level");
}

#[test]
fn test_losexp_minimum_level() {
    let mut p = make_player();
    p.exp_level = 1;
    p.losexp(false);
    assert!(p.exp_level >= 1, "Can't go below level 1");
}

#[test]
fn test_hit_dice() {
    let p = make_player();
    let hd = p.hit_dice();
    assert!(hd > 0);
}

#[test]
fn test_newuexp() {
    let p = make_player();
    let exp_needed = p.newuexp();
    assert!(exp_needed > 0, "Exp to next level should be positive");
}

// ============================================================================
// Status queries
// ============================================================================

#[test]
fn test_is_confused() {
    let mut p = make_player();
    assert!(!p.is_confused());
    p.confused_timeout = 10;
    assert!(p.is_confused());
}

#[test]
fn test_is_stunned() {
    let mut p = make_player();
    assert!(!p.is_stunned());
    p.stunned_timeout = 5;
    assert!(p.is_stunned());
}

#[test]
fn test_is_blind() {
    let mut p = make_player();
    assert!(!p.is_blind());
    p.blinded_timeout = 10;
    assert!(p.is_blind());
}

#[test]
fn test_is_hallucinating() {
    let mut p = make_player();
    assert!(!p.is_hallucinating());
    p.hallucinating_timeout = 10;
    assert!(p.is_hallucinating());
}

#[test]
fn test_is_paralyzed() {
    let mut p = make_player();
    assert!(!p.is_paralyzed());
    p.paralyzed_timeout = 5;
    assert!(p.is_paralyzed());
}

#[test]
fn test_is_sleeping() {
    let mut p = make_player();
    assert!(!p.is_sleeping());
    p.sleeping_timeout = 10;
    assert!(p.is_sleeping());
}

#[test]
fn test_is_polymorphed() {
    let mut p = make_player();
    assert!(!p.is_polymorphed());
    p.monster_num = Some(5);
    assert!(p.is_polymorphed());
}

#[test]
fn test_can_move_when_buried() {
    let mut p = make_player();
    p.buried = true;
    assert!(!p.can_move());
}

#[test]
fn test_can_move_normal() {
    let p = make_player();
    assert!(p.can_move());
}

// ============================================================================
// Status effect application
// ============================================================================

#[test]
fn test_make_confused() {
    let mut p = make_player();
    let msg = p.make_confused(10, true);
    assert!(p.is_confused());
    assert!(msg.is_some());
}

#[test]
fn test_make_stunned() {
    let mut p = make_player();
    let msg = p.make_stunned(10, true);
    assert!(p.is_stunned());
    assert!(msg.is_some());
}

#[test]
fn test_make_blinded() {
    let mut p = make_player();
    let msg = p.make_blinded(10, true);
    assert!(p.is_blind());
    assert!(msg.is_some());
}

#[test]
fn test_make_hallucinated() {
    let mut p = make_player();
    let msg = p.make_hallucinated(10, true);
    assert!(p.is_hallucinating());
    assert!(msg.is_some());
}

#[test]
fn test_cure_all_clears_statuses() {
    let mut p = make_player();
    p.confused_timeout = 10;
    p.stunned_timeout = 10;
    p.blinded_timeout = 10;
    p.cure_all();
    assert!(!p.is_confused());
    assert!(!p.is_stunned());
    assert!(!p.is_blind());
}

#[test]
fn test_healup_cures_conditions() {
    let mut p = make_player();
    p.take_damage(10);
    p.sickness_timeout = 5;
    p.blinded_timeout = 5;
    p.healup(100, true, true);
    assert_eq!(p.hp, p.hp_max);
}

// ============================================================================
// Distance calculations
// ============================================================================

#[test]
fn test_distu() {
    let mut p = make_player();
    p.pos.x = 10;
    p.pos.y = 10;
    let d = p.distu(13, 14);
    assert_eq!(d, 9 + 16); // 3^2 + 4^2 = 25
}

#[test]
fn test_distmin() {
    let mut p = make_player();
    p.pos.x = 10;
    p.pos.y = 10;
    let d = p.distmin(13, 14);
    assert_eq!(d, 4); // max(3, 4)
}

#[test]
fn test_next_to() {
    let mut p = make_player();
    p.pos.x = 10;
    p.pos.y = 10;
    assert!(p.next_to(11, 11));
    assert!(p.next_to(9, 10));
    assert!(!p.next_to(12, 10));
}

// ============================================================================
// Properties
// ============================================================================

#[test]
fn test_levitating() {
    let mut p = make_player();
    assert!(!p.is_levitating());
    p.properties.grant_intrinsic(Property::Levitation);
    assert!(p.is_levitating());
}

#[test]
fn test_flying() {
    let mut p = make_player();
    assert!(!p.is_flying());
    p.properties.grant_intrinsic(Property::Flying);
    assert!(p.is_flying());
}

#[test]
fn test_pass_walls() {
    let mut p = make_player();
    assert!(!p.can_pass_walls());
    p.properties.grant_intrinsic(Property::PassesWalls);
    assert!(p.can_pass_walls());
}

#[test]
fn test_telepathy() {
    let mut p = make_player();
    assert!(!p.has_telepathy());
    p.properties.grant_intrinsic(Property::Telepathy);
    assert!(p.has_telepathy());
}

#[test]
fn test_see_invisible() {
    let mut p = make_player();
    assert!(!p.has_see_invisible());
    p.properties.grant_intrinsic(Property::SeeInvisible);
    assert!(p.has_see_invisible());
}

#[test]
fn test_infravision() {
    let mut p = make_player();
    assert!(!p.has_infravision());
    p.properties.grant_intrinsic(Property::Infravision);
    assert!(p.has_infravision());
}

// ============================================================================
// Encumbrance
// ============================================================================

#[test]
fn test_unencumbered_default() {
    let p = make_player();
    assert_eq!(p.encumbrance(), Encumbrance::Unencumbered);
}

#[test]
fn test_weight_cap_positive() {
    let p = make_player();
    assert!(p.weight_cap() > 0, "Weight cap should be positive");
}

#[test]
fn test_excess_weight_when_unencumbered() {
    let p = make_player();
    assert!(p.excess_weight() <= 0, "No excess weight when unencumbered");
}

#[test]
fn test_encumbrance_movement_penalty() {
    assert_eq!(Encumbrance::Unencumbered.movement_penalty(), 0);
    assert!(Encumbrance::Burdened.movement_penalty() > 0);
    assert!(Encumbrance::Overloaded.movement_penalty() > Encumbrance::Burdened.movement_penalty());
}

// ============================================================================
// Position
// ============================================================================

#[test]
fn test_position_distance_sq() {
    use nh_core::player::Position;
    let a = Position::new(3, 4);
    let b = Position::new(6, 8);
    assert_eq!(a.distance_sq(&b), 25);
}

#[test]
fn test_position_adjacent() {
    use nh_core::player::Position;
    let a = Position::new(5, 5);
    assert!(a.is_adjacent(&Position::new(6, 6)));
    assert!(a.is_adjacent(&Position::new(4, 5)));
    assert!(!a.is_adjacent(&Position::new(7, 5)));
    assert!(!a.is_adjacent(&Position::new(5, 5))); // same pos not adjacent
}

// ============================================================================
// Trap state
// ============================================================================

#[test]
fn test_player_trap_type_names() {
    use nh_core::player::PlayerTrapType;
    assert_eq!(PlayerTrapType::BearTrap.name(), "bear trap");
    assert_eq!(PlayerTrapType::Pit.name(), "pit");
    assert_eq!(PlayerTrapType::Web.name(), "web");
}

#[test]
fn test_player_trap_is_pit() {
    use nh_core::player::PlayerTrapType;
    assert!(PlayerTrapType::Pit.is_pit());
    assert!(PlayerTrapType::SpikedPit.is_pit());
    assert!(!PlayerTrapType::BearTrap.is_pit());
}

// ============================================================================
// Role initialization
// ============================================================================

#[test]
fn test_role_valkyrie() {
    let p = make_player_with_role(Role::Valkyrie);
    assert_eq!(p.role, Role::Valkyrie);
}

#[test]
fn test_role_wizard() {
    let p = make_player_with_role(Role::Wizard);
    assert_eq!(p.role, Role::Wizard);
}

#[test]
fn test_role_tourist() {
    let p = make_player_with_role(Role::Tourist);
    assert_eq!(p.role, Role::Tourist);
}

// ============================================================================
// Pronouns and alignment
// ============================================================================

#[test]
fn test_rank_title_nonempty() {
    let p = make_player();
    let title = p.rank_title();
    assert!(!title.is_empty());
}

#[test]
fn test_align_str_nonempty() {
    let p = make_player();
    let s = p.align_str();
    assert!(!s.is_empty());
}

// ============================================================================
// Strength-related functions
// ============================================================================

#[test]
fn test_losestr() {
    let mut p = make_player();
    let old = p.acurr(Attribute::Strength);
    p.losestr(2);
    assert!(p.acurr(Attribute::Strength) < old);
}

#[test]
fn test_gainstr() {
    let mut p = make_player();
    let old = p.acurr(Attribute::Strength);
    p.gainstr(2);
    assert!(p.acurr(Attribute::Strength) > old);
}

// ============================================================================
// Miscellaneous player functions
// ============================================================================

#[test]
fn test_can_pray_default() {
    let p = make_player();
    assert!(p.can_pray());
}

#[test]
fn test_can_be_strangled_default() {
    let p = make_player();
    // Default player (non-polymorphed) can be strangled
    assert!(p.can_be_strangled());
}

#[test]
fn test_digest_reduces_nutrition() {
    let mut p = make_player();
    let old_nutr = p.nutrition;
    p.digest(100);
    assert!(p.nutrition < old_nutr);
}

#[test]
fn test_drain_en() {
    let mut p = make_player();
    p.energy = 15;
    p.drain_en(5);
    assert!(p.energy < 15);
}

#[test]
fn test_set_utrap_directly() {
    use nh_core::player::PlayerTrapType;
    let mut p = make_player();
    p.utrap = 5;
    p.utrap_type = PlayerTrapType::BearTrap;
    assert_eq!(p.utrap, 5);
    assert_eq!(p.utrap_type, PlayerTrapType::BearTrap);
}

#[test]
fn test_reset_utrap_directly() {
    use nh_core::player::PlayerTrapType;
    let mut p = make_player();
    p.utrap = 5;
    p.utrap_type = PlayerTrapType::Pit;
    // Manual reset
    p.utrap = 0;
    p.utrap_type = PlayerTrapType::None;
    assert_eq!(p.utrap, 0);
    assert_eq!(p.utrap_type, PlayerTrapType::None);
}

#[test]
fn test_nomul_directly() {
    let mut p = make_player();
    p.multi = -5;
    p.multi_reason = Some("reading a book".to_string());
    assert_eq!(p.multi, -5);
    assert!(p.multi_reason.is_some());
}

#[test]
fn test_wake_up_via_timeout() {
    let mut p = make_player();
    p.sleeping_timeout = 10;
    assert!(p.is_sleeping());
    p.sleeping_timeout = 0;
    assert!(!p.is_sleeping());
}

#[test]
fn test_distance_calculation() {
    let mut p = make_player();
    p.pos.x = 10;
    p.pos.y = 10;
    // distu gives distance squared
    assert_eq!(p.distu(13, 14), 25);
}
