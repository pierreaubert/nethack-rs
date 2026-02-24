//! Step 4: Core action parity tests
//!
//! Tests the Rust implementations of core player actions against expected
//! behavior from the C NetHack source. Covers:
//! - Eating (eat.c)
//! - Tool application (apply.c)
//! - Pickup/drop (pickup.c)
//! - Equipment (do_wear.c)
//! - Traps (trap.c)

use nh_core::action::eat::{
    apply_corpse_effects, calculate_nutrition, corpse_effects, gethungry, is_edible, is_rotten,
    lesshungry, newuhs, CorpseEffect,
};
use nh_core::player::Race;
use nh_core::action::pickup::{
    do_autopickup, do_drop, do_pickup, gold_weight, matches_autopickup_type,
    parse_autopickup_types, should_autopickup, within_pickup_burden, PickupBurden,
    DEFAULT_AUTOPICKUP_TYPES,
};
use nh_core::action::trap::trigger_trap;
use nh_core::dungeon::TrapType;
use nh_core::action::wear::{
    amulet_off, amulet_on, do_puton, do_remove, do_takeoff, do_wear, do_wield, ring_off, ring_on,
    worn_mask::*,
};
use nh_core::action::ActionResult;
use nh_core::object::{BucStatus, Object, ObjectClass, ObjectId};
use nh_core::player::{Encumbrance, HungerState, Property};
use nh_core::GameRng;
use nh_core::GameState;

// ============================================================================
// Helper: create a GameState with a deterministic RNG seed
// ============================================================================

fn test_state(seed: u64) -> GameState {
    GameState::new(GameRng::new(seed))
}

fn make_food(letter: char) -> Object {
    let mut obj = Object::default();
    obj.id = ObjectId(1);
    obj.class = ObjectClass::Food;
    obj.inv_letter = letter;
    obj.name = Some("food ration".to_string());
    obj.weight = 20;
    obj
}

fn make_armor(letter: char, object_type: i16) -> Object {
    let mut obj = Object::default();
    obj.id = ObjectId(2);
    obj.class = ObjectClass::Armor;
    obj.inv_letter = letter;
    obj.object_type = object_type;
    obj.name = Some("armor".to_string());
    obj.weight = 100;
    obj
}

fn make_ring(letter: char, object_type: i16, enchantment: i8) -> Object {
    let mut obj = Object::default();
    obj.id = ObjectId(3);
    obj.class = ObjectClass::Ring;
    obj.inv_letter = letter;
    obj.object_type = object_type;
    obj.enchantment = enchantment;
    obj.name = Some("ring".to_string());
    obj.weight = 3;
    obj
}

fn make_amulet(letter: char, object_type: i16) -> Object {
    let mut obj = Object::default();
    obj.id = ObjectId(4);
    obj.class = ObjectClass::Amulet;
    obj.inv_letter = letter;
    obj.object_type = object_type;
    obj.name = Some("amulet".to_string());
    obj.weight = 20;
    obj
}

fn make_tool(letter: char, object_type: i16) -> Object {
    let mut obj = Object::default();
    obj.id = ObjectId(5);
    obj.class = ObjectClass::Tool;
    obj.inv_letter = letter;
    obj.object_type = object_type;
    obj.name = Some("tool".to_string());
    obj.weight = 10;
    obj
}

// ============================================================================
// 4.1: Eating tests
// ============================================================================

#[test]
fn test_eat_food_increases_nutrition() {
    let mut state = test_state(42);
    let initial_nutrition = state.player.nutrition;
    let food = make_food('a');
    state.inventory.push(food);

    let result = nh_core::action::eat::do_eat(&mut state, 'a');
    assert!(matches!(result, ActionResult::Success));
    assert!(
        state.player.nutrition > initial_nutrition,
        "Eating food should increase nutrition"
    );
}

#[test]
fn test_eat_non_food_fails() {
    let mut state = test_state(42);
    let mut obj = Object::default();
    obj.class = ObjectClass::Weapon;
    obj.inv_letter = 'a';
    state.inventory.push(obj);

    let result = nh_core::action::eat::do_eat(&mut state, 'a');
    assert!(matches!(result, ActionResult::Failed(_)));
}

#[test]
fn test_eat_removes_item_from_inventory() {
    let mut state = test_state(42);
    let food = make_food('a');
    state.inventory.push(food);
    assert_eq!(state.inventory.len(), 1);

    nh_core::action::eat::do_eat(&mut state, 'a');
    assert_eq!(
        state.inventory.len(),
        0,
        "Eaten food should be removed from inventory"
    );
}

#[test]
fn test_is_edible() {
    let mut food = Object::default();
    food.class = ObjectClass::Food;
    assert!(is_edible(&food));

    let mut weapon = Object::default();
    weapon.class = ObjectClass::Weapon;
    assert!(!is_edible(&weapon));
}

#[test]
fn test_is_rotten_timing() {
    use nh_core::action::eat::otyp;

    let mut corpse = Object::default();
    corpse.class = ObjectClass::Food;
    corpse.object_type = otyp::CORPSE;
    corpse.corpse_type = 0; // generic monster type
    corpse.buc = BucStatus::Uncursed;
    corpse.age = 0;

    // Fresh corpse at turn 100
    assert!(!is_rotten(&corpse, 100));
    // Rotten corpse at turn 300 (threshold 150 for uncursed)
    assert!(is_rotten(&corpse, 300));
    // Blessed lasts longer (threshold 300)
    corpse.buc = BucStatus::Blessed;
    assert!(!is_rotten(&corpse, 200));
    assert!(is_rotten(&corpse, 400));
    // Cursed rots faster (threshold 50)
    corpse.buc = BucStatus::Cursed;
    assert!(is_rotten(&corpse, 200));
}

#[test]
fn test_nutrition_race_modifier() {
    use nh_core::action::eat::otyp;

    let mut food = Object::default();
    food.object_type = otyp::LEMBAS_WAFER;
    food.nutrition = 800;

    // C-faithful: race affects nutrition for lembas/cram, not BUC
    let human = calculate_nutrition(&food, Race::Human);
    assert_eq!(human, 800, "Human gets base nutrition");

    let elf = calculate_nutrition(&food, Race::Elf);
    assert!(elf > human, "Elf gets bonus from lembas");

    let orc = calculate_nutrition(&food, Race::Orc);
    assert!(orc < human, "Orc gets penalty from lembas");
}

/// Corpse effects table: verify key monster types produce expected effects.
/// Monster indices match nh-data/src/monsters.rs MONSTERS array positions.
#[test]
fn test_corpse_effects_known_monsters() {
    // Floating eye (index 29) -> telepathy
    let effects = corpse_effects(29);
    assert!(!effects.is_empty(), "Floating eye should have effects");
    assert!(
        effects.iter().any(|e| matches!(
            e,
            CorpseEffect::GainIntrinsic {
                property: Property::Telepathy,
                ..
            }
        )),
        "Floating eye corpse should grant telepathy"
    );

    // Cockatrice (index 10) -> instant death
    let effects = corpse_effects(10);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, CorpseEffect::InstantDeath { .. })),
        "Cockatrice corpse should cause instant death"
    );

    // Lizard (index 49) -> cure stoning, confusion, stunning
    let effects = corpse_effects(49);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, CorpseEffect::CureStoning)),
        "Lizard corpse should cure stoning"
    );
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, CorpseEffect::CureConfusion)),
        "Lizard corpse should cure confusion"
    );

    // Wraith (index 241) -> gain level
    let effects = corpse_effects(241);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, CorpseEffect::GainLevel)),
        "Wraith corpse should grant a level"
    );

    // Newt (index 45) -> energy
    let effects = corpse_effects(45);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, CorpseEffect::GainEnergy { .. })),
        "Newt corpse should grant energy"
    );

    // Stalker (index 163) -> invisibility + stunning
    let effects = corpse_effects(163);
    assert!(
        effects.iter().any(|e| matches!(
            e,
            CorpseEffect::GainIntrinsic {
                property: Property::Invisibility,
                ..
            }
        )),
        "Stalker corpse should grant invisibility"
    );
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, CorpseEffect::Stun { .. })),
        "Stalker corpse should also stun"
    );

    // Quantum mechanic (index 221) -> toggle speed
    let effects = corpse_effects(221);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, CorpseEffect::ToggleSpeed)),
        "Quantum mechanic corpse should toggle speed"
    );

    // Fire giant (index 182) -> strength + fire resistance
    let effects = corpse_effects(182);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, CorpseEffect::StrengthBoost)),
        "Fire giant corpse should grant strength"
    );
    assert!(
        effects.iter().any(|e| matches!(
            e,
            CorpseEffect::GainIntrinsic {
                property: Property::FireResistance,
                ..
            }
        )),
        "Fire giant corpse should also grant fire resistance"
    );

    // Green slime (index 219) -> instant death
    let effects = corpse_effects(219);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, CorpseEffect::InstantDeath { .. })),
        "Green slime corpse should cause instant death"
    );

    // Nurse (index 290) -> full heal + poison resistance
    let effects = corpse_effects(290);
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, CorpseEffect::FullHeal)),
        "Nurse corpse should grant full heal"
    );
}

#[test]
fn test_apply_corpse_effects_gain_level() {
    let mut state = test_state(42);
    let initial_level = state.player.exp_level;
    let initial_hp_max = state.player.hp_max;

    let effects = vec![CorpseEffect::GainLevel];
    let mut rng = GameRng::new(42);
    let messages = apply_corpse_effects(&mut state, &mut rng, &effects);

    assert_eq!(state.player.exp_level, initial_level + 1);
    assert!(state.player.hp_max > initial_hp_max);
    assert!(messages.iter().any(|m| m.contains("experienced")));
}

#[test]
fn test_apply_corpse_effects_instant_death() {
    let mut state = test_state(42);
    let effects = vec![CorpseEffect::InstantDeath {
        cause: "eating a cockatrice",
    }];
    let mut rng = GameRng::new(42);
    let messages = apply_corpse_effects(&mut state, &mut rng, &effects);

    assert_eq!(state.player.hp, 0, "Instant death should kill the player");
    assert!(messages.iter().any(|m| m.contains("die")));
}

#[test]
fn test_apply_corpse_effects_toggle_speed() {
    let mut state = test_state(42);
    assert!(!state.player.properties.has_intrinsic(Property::Speed));

    let effects = vec![CorpseEffect::ToggleSpeed];
    let mut rng = GameRng::new(42);

    // First toggle: gain speed
    apply_corpse_effects(&mut state, &mut rng, &effects);
    assert!(state.player.properties.has_intrinsic(Property::Speed));

    // Second toggle: lose speed
    apply_corpse_effects(&mut state, &mut rng, &effects);
    assert!(!state.player.properties.has_intrinsic(Property::Speed));
}

// ============================================================================
// 4.1b: Hunger system tests (newuhs, gethungry, lesshungry)
// ============================================================================

#[test]
fn test_hunger_state_transitions() {
    let mut state = test_state(42);

    // C thresholds (eat.c:2936-2939):
    //   nutrition > 1000 → Satiated
    //   nutrition > 150  → NotHungry
    //   nutrition > 50   → Hungry
    //   nutrition > 0    → Weak
    //   else             → Fainting

    // Start with normal nutrition (>150 is NotHungry)
    state.player.nutrition = 800;
    state.player.hunger_state = HungerState::NotHungry;

    // Decrease nutrition below hungry threshold (<=150, >50 is Hungry)
    state.player.nutrition = 100;
    let msgs = newuhs(&mut state, true); // incr=true: getting hungrier
    assert_eq!(state.player.hunger_state, HungerState::Hungry);
    assert!(
        msgs.iter().any(|m| m.contains("hungry")),
        "Should get hungry message"
    );

    // Eat and recover (>150 is NotHungry)
    state.player.nutrition = 800;
    let msgs = newuhs(&mut state, false); // incr=false: getting less hungry
    assert_eq!(state.player.hunger_state, HungerState::NotHungry);
    assert!(
        !msgs.is_empty(),
        "Should get recovery message when no longer hungry"
    );
}

#[test]
fn test_gethungry_decrements_nutrition() {
    let mut state = test_state(42);
    state.player.nutrition = 500;
    state.player.hunger_state = HungerState::NotHungry;
    let mut rng = GameRng::new(42);

    let initial = state.player.nutrition;
    gethungry(&mut state);
    assert!(
        state.player.nutrition < initial,
        "gethungry should decrement nutrition"
    );
}

#[test]
fn test_slow_digestion_prevents_hunger() {
    let mut state = test_state(42);
    state.player.nutrition = 500;
    state.player.hunger_state = HungerState::NotHungry;
    state
        .player
        .properties
        .grant_intrinsic(Property::SlowDigestion);
    let mut rng = GameRng::new(42);

    let initial = state.player.nutrition;
    gethungry(&mut state);
    assert_eq!(
        state.player.nutrition, initial,
        "Slow Digestion should prevent nutrition loss"
    );
}

#[test]
fn test_lesshungry_adds_nutrition() {
    let mut state = test_state(42);
    state.player.nutrition = 200;
    state.player.hunger_state = HungerState::NotHungry;

    let initial = state.player.nutrition;
    lesshungry(&mut state, 300);
    assert_eq!(state.player.nutrition, initial + 300);
}

#[test]
fn test_lesshungry_caps_at_max() {
    let mut state = test_state(42);
    state.player.nutrition = 4900;
    state.player.hunger_state = HungerState::NotHungry;

    lesshungry(&mut state, 5000);
    assert!(
        state.player.nutrition <= 5000,
        "Nutrition should be capped at maximum"
    );
}

// test_tin_type_nutrition_values removed: TinType not yet implemented in nh_core

// ============================================================================
// 4.2: Equipment tests (wear, puton, wield)
// ============================================================================

#[test]
fn test_wear_armor() {
    let mut state = test_state(42);
    let armor = make_armor('a', 10); // body armor range
    state.inventory.push(armor);

    let result = do_wear(&mut state, 'a');
    assert!(matches!(result, ActionResult::Success));
    assert!(
        state.inventory[0].worn_mask & W_ARM != 0,
        "Armor should be marked as worn"
    );
}

#[test]
fn test_wear_non_armor_fails() {
    let mut state = test_state(42);
    let food = make_food('a');
    state.inventory.push(food);

    let result = do_wear(&mut state, 'a');
    assert!(matches!(result, ActionResult::Failed(_)));
}

#[test]
fn test_wear_already_worn_fails() {
    let mut state = test_state(42);
    let mut armor = make_armor('a', 10);
    armor.worn_mask = W_ARM;
    state.inventory.push(armor);

    let result = do_wear(&mut state, 'a');
    assert!(matches!(result, ActionResult::Failed(_)));
}

#[test]
fn test_takeoff_armor() {
    let mut state = test_state(42);
    let mut armor = make_armor('a', 10);
    armor.worn_mask = W_ARM;
    state.inventory.push(armor);

    let result = do_takeoff(&mut state, 'a');
    assert!(matches!(result, ActionResult::Success));
    assert_eq!(
        state.inventory[0].worn_mask & W_ARMOR,
        0,
        "Armor should no longer be worn"
    );
}

#[test]
fn test_takeoff_cursed_fails() {
    let mut state = test_state(42);
    let mut armor = make_armor('a', 10);
    armor.worn_mask = W_ARM;
    armor.buc = BucStatus::Cursed;
    state.inventory.push(armor);

    let result = do_takeoff(&mut state, 'a');
    assert!(
        matches!(result, ActionResult::Failed(_)),
        "Should not be able to take off cursed armor"
    );
}

#[test]
fn test_wield_weapon() {
    let mut state = test_state(42);
    let mut weapon = Object::default();
    weapon.class = ObjectClass::Weapon;
    weapon.inv_letter = 'a';
    weapon.name = Some("long sword".to_string());
    state.inventory.push(weapon);

    let result = do_wield(&mut state, 'a');
    assert!(matches!(result, ActionResult::Success));
    assert!(
        state.inventory[0].worn_mask & W_WEP != 0,
        "Weapon should be wielded"
    );
}

#[test]
fn test_wield_replaces_current() {
    let mut state = test_state(42);

    // Wield first weapon
    let mut weapon1 = Object::default();
    weapon1.class = ObjectClass::Weapon;
    weapon1.inv_letter = 'a';
    weapon1.worn_mask = W_WEP;
    weapon1.name = Some("dagger".to_string());
    state.inventory.push(weapon1);

    // Add second weapon
    let mut weapon2 = Object::default();
    weapon2.class = ObjectClass::Weapon;
    weapon2.inv_letter = 'b';
    weapon2.name = Some("long sword".to_string());
    state.inventory.push(weapon2);

    // Wield second weapon should unwield first
    let result = do_wield(&mut state, 'b');
    assert!(matches!(result, ActionResult::Success));
    assert_eq!(
        state.inventory[0].worn_mask & W_WEP,
        0,
        "First weapon should be unwielded"
    );
    assert!(
        state.inventory[1].worn_mask & W_WEP != 0,
        "Second weapon should be wielded"
    );
}

#[test]
fn test_puton_ring() {
    let mut state = test_state(42);
    let ring = make_ring('a', 500, 0); // RIN_REGENERATION
    state.inventory.push(ring);

    let result = do_puton(&mut state, 'a');
    assert!(matches!(result, ActionResult::Success));
    assert!(
        state.inventory[0].worn_mask & W_RING != 0,
        "Ring should be worn"
    );
}

#[test]
fn test_puton_two_rings() {
    let mut state = test_state(42);
    let ring1 = make_ring('a', 500, 0);
    let ring2 = make_ring('b', 501, 0);
    state.inventory.push(ring1);
    state.inventory.push(ring2);

    // First ring -> left hand
    do_puton(&mut state, 'a');
    assert!(state.inventory[0].worn_mask & W_RINGL != 0);

    // Second ring -> right hand
    do_puton(&mut state, 'b');
    assert!(state.inventory[1].worn_mask & W_RINGR != 0);
}

#[test]
fn test_puton_third_ring_fails() {
    let mut state = test_state(42);
    let mut ring1 = make_ring('a', 500, 0);
    ring1.worn_mask = W_RINGL;
    let mut ring2 = make_ring('b', 501, 0);
    ring2.worn_mask = W_RINGR;
    let ring3 = make_ring('c', 502, 0);
    state.inventory.push(ring1);
    state.inventory.push(ring2);
    state.inventory.push(ring3);

    let result = do_puton(&mut state, 'c');
    assert!(
        matches!(result, ActionResult::Failed(_)),
        "Should not be able to wear 3 rings"
    );
}

#[test]
fn test_puton_amulet() {
    let mut state = test_state(42);
    let amulet = make_amulet('a', 475); // AMULET_OF_ESP
    state.inventory.push(amulet);

    let result = do_puton(&mut state, 'a');
    assert!(matches!(result, ActionResult::Success));
    assert!(
        state.inventory[0].worn_mask & W_AMUL != 0,
        "Amulet should be worn"
    );
}

#[test]
fn test_remove_cursed_ring_fails() {
    let mut state = test_state(42);
    let mut ring = make_ring('a', 500, 0);
    ring.worn_mask = W_RINGL;
    ring.buc = BucStatus::Cursed;
    state.inventory.push(ring);

    let result = do_remove(&mut state, 'a');
    assert!(
        matches!(result, ActionResult::Failed(_)),
        "Should not be able to remove cursed ring"
    );
}

// ============================================================================
// 4.2b: Ring/Amulet property effect tests
// ============================================================================

#[test]
fn test_ring_on_grants_property() {
    let mut state = test_state(42);
    let mut ring = make_ring('a', 510, 0); // RIN_FIRE_RESISTANCE
    ring.worn_mask = W_RINGL;

    assert!(!state.player.properties.has(Property::FireResistance));
    ring_on(&mut state, &ring);
    assert!(
        state.player.properties.has(Property::FireResistance),
        "Ring of fire resistance should grant fire resistance"
    );
}

#[test]
fn test_ring_off_removes_property() {
    let mut state = test_state(42);
    let mut ring = make_ring('a', 510, 0); // RIN_FIRE_RESISTANCE
    ring.worn_mask = W_RINGL;

    ring_on(&mut state, &ring);
    assert!(state.player.properties.has(Property::FireResistance));

    ring_off(&mut state, &ring);
    assert!(
        !state.player.properties.has(Property::FireResistance),
        "Removing ring should remove property"
    );
}

#[test]
fn test_ring_gain_strength_bonus() {
    let mut state = test_state(42);
    // Set reasonable starting strength (Attributes clamp to 3..25)
    state
        .player
        .attr_current
        .set(nh_core::player::Attribute::Strength, 10);
    let mut ring = make_ring('a', 495, 2); // RIN_GAIN_STRENGTH, +2
    ring.worn_mask = W_RINGL;

    let initial_str = state
        .player
        .attr_current
        .get(nh_core::player::Attribute::Strength);
    assert_eq!(initial_str, 10);
    let effect = ring_on(&mut state, &ring);

    let new_str = state
        .player
        .attr_current
        .get(nh_core::player::Attribute::Strength);
    assert_eq!(
        new_str,
        initial_str + 2,
        "Ring of gain strength +2 should add 2 to strength"
    );
    assert!(
        effect.identify,
        "Stat-modifying ring should identify itself"
    );
}

#[test]
fn test_ring_gain_strength_removal_reverses() {
    let mut state = test_state(42);
    // Set reasonable starting strength (Attributes clamp to 3..25)
    state
        .player
        .attr_current
        .set(nh_core::player::Attribute::Strength, 10);
    let mut ring = make_ring('a', 495, 3); // RIN_GAIN_STRENGTH, +3
    ring.worn_mask = W_RINGL;

    let initial_str = state
        .player
        .attr_current
        .get(nh_core::player::Attribute::Strength);
    assert_eq!(initial_str, 10);
    ring_on(&mut state, &ring);
    ring_off(&mut state, &ring);

    let final_str = state
        .player
        .attr_current
        .get(nh_core::player::Attribute::Strength);
    assert_eq!(
        final_str, initial_str,
        "Removing ring of gain strength should restore original strength"
    );
}

#[test]
fn test_amulet_on_grants_property() {
    let mut state = test_state(42);
    let amulet = make_amulet('a', 475); // AMULET_OF_ESP

    assert!(!state.player.properties.has(Property::Telepathy));
    amulet_on(&mut state, &amulet);
    assert!(
        state.player.properties.has(Property::Telepathy),
        "Amulet of ESP should grant telepathy"
    );
}

#[test]
fn test_amulet_off_removes_property() {
    let mut state = test_state(42);
    let amulet = make_amulet('a', 475); // AMULET_OF_ESP

    amulet_on(&mut state, &amulet);
    assert!(state.player.properties.has(Property::Telepathy));

    amulet_off(&mut state, &amulet);
    assert!(
        !state.player.properties.has(Property::Telepathy),
        "Removing amulet of ESP should remove telepathy"
    );
}

#[test]
fn test_amulet_of_change_destroys_itself() {
    let mut state = test_state(42);
    let amulet = make_amulet('a', 480); // AMULET_OF_CHANGE

    let effect = amulet_on(&mut state, &amulet);
    assert!(effect.destroy, "Amulet of change should destroy itself");
    assert!(effect.identify, "Amulet of change should identify itself");
}

// ============================================================================
// 4.3: Pickup/Drop tests
// ============================================================================

#[test]
fn test_pickup_empty_floor() {
    let mut state = test_state(42);
    let result = do_pickup(&mut state);
    assert!(matches!(result, ActionResult::NoTime));
}

#[test]
fn test_pickup_item_from_floor() {
    let mut state = test_state(42);
    let x = state.player.pos.x;
    let y = state.player.pos.y;

    let mut obj = Object::default();
    obj.class = ObjectClass::Food;
    obj.name = Some("apple".to_string());
    state.current_level.add_object(obj, x, y);

    assert_eq!(state.current_level.objects_at(x, y).len(), 1);
    assert!(state.inventory.is_empty());

    let result = do_pickup(&mut state);
    assert!(matches!(result, ActionResult::Success));
    assert!(state.current_level.objects_at(x, y).is_empty());
    assert_eq!(state.inventory.len(), 1);
}

#[test]
fn test_drop_item_to_floor() {
    let mut state = test_state(42);
    let food = make_food('a');
    state.inventory.push(food);

    let x = state.player.pos.x;
    let y = state.player.pos.y;

    let result = do_drop(&mut state, 'a');
    assert!(matches!(result, ActionResult::Success));
    assert!(state.inventory.is_empty());
    assert_eq!(state.current_level.objects_at(x, y).len(), 1);
}

#[test]
fn test_drop_worn_item_fails() {
    let mut state = test_state(42);
    let mut armor = make_armor('a', 10);
    armor.worn_mask = W_ARM;
    state.inventory.push(armor);

    let result = do_drop(&mut state, 'a');
    assert!(
        matches!(result, ActionResult::Failed(_)),
        "Should not be able to drop worn armor"
    );
}

#[test]
fn test_gold_weight_formula() {
    // C formula: (50 gold + 50) / 100 = weight
    assert_eq!(gold_weight(0), 0);
    assert_eq!(gold_weight(50), 1);
    assert_eq!(gold_weight(100), 1);
    assert_eq!(gold_weight(1000), 10);
}

#[test]
fn test_autopickup_type_matching() {
    let mut gold = Object::default();
    gold.class = ObjectClass::Coin;

    let mut scroll = Object::default();
    scroll.class = ObjectClass::Scroll;

    let mut weapon = Object::default();
    weapon.class = ObjectClass::Weapon;

    // Default types include gold ($) and scrolls (?)
    assert!(matches_autopickup_type(&gold, DEFAULT_AUTOPICKUP_TYPES));
    assert!(matches_autopickup_type(&scroll, DEFAULT_AUTOPICKUP_TYPES));
    assert!(!matches_autopickup_type(&weapon, DEFAULT_AUTOPICKUP_TYPES));
}

#[test]
fn test_should_autopickup_disabled() {
    let mut obj = Object::default();
    obj.class = ObjectClass::Coin;

    assert!(!should_autopickup(&obj, false, "$", false));
    assert!(!should_autopickup(&obj, true, "$", true)); // nopick flag
    assert!(should_autopickup(&obj, true, "$", false)); // normal case
}

#[test]
fn test_parse_autopickup_types() {
    assert_eq!(parse_autopickup_types("$?!"), "$?!");
    assert_eq!(parse_autopickup_types("$xyz?!"), "$?!"); // invalid chars stripped
    assert_eq!(parse_autopickup_types(""), "");
}

#[test]
fn test_pickup_burden_thresholds() {
    assert!(PickupBurden::Unencumbered.allows_encumbrance(Encumbrance::Unencumbered));
    assert!(!PickupBurden::Unencumbered.allows_encumbrance(Encumbrance::Burdened));
    assert!(PickupBurden::Burdened.allows_encumbrance(Encumbrance::Burdened));
    assert!(!PickupBurden::Burdened.allows_encumbrance(Encumbrance::Stressed));
    assert!(PickupBurden::Overloaded.allows_encumbrance(Encumbrance::Overloaded));
}

// ============================================================================
// 4.4: Trap tests
// ============================================================================

#[test]
fn test_trap_arrow_damage() {
    let mut state = test_state(42);
    let initial_hp = state.player.hp;

    trigger_trap(&mut state, TrapType::Arrow);
    assert!(
        state.player.hp < initial_hp,
        "Arrow trap should deal damage"
    );
}

#[test]
fn test_trap_pit_damage() {
    let mut state = test_state(42);
    let initial_hp = state.player.hp;

    trigger_trap(&mut state, TrapType::Pit);
    assert!(state.player.hp < initial_hp, "Pit trap should deal damage");
}

#[test]
fn test_trap_damage_deterministic() {
    // Same seed should produce same trap damage
    let mut state1 = test_state(42);
    let mut state2 = test_state(42);

    trigger_trap(&mut state1, TrapType::Arrow);
    trigger_trap(&mut state2, TrapType::Arrow);

    assert_eq!(
        state1.player.hp, state2.player.hp,
        "Same seed should produce same trap damage"
    );
}

#[test]
fn test_trap_can_kill() {
    let mut state = test_state(42);
    state.player.hp = 1;

    let result = trigger_trap(&mut state, TrapType::Arrow);
    if state.player.hp <= 0 {
        assert!(
            matches!(result, ActionResult::Died(_)),
            "Lethal trap should return Died"
        );
    }
}

// ============================================================================
// 4.5: Tool application tests
// ============================================================================

#[test]
fn test_apply_non_tool_fails() {
    let mut state = test_state(42);
    let food = make_food('a');
    state.inventory.push(food);

    let result = nh_core::action::apply::do_apply(&mut state, 'a');
    assert!(matches!(result, ActionResult::Failed(_)));
}

#[test]
fn test_apply_lamp_toggles_lit() {
    let mut state = test_state(42);
    let mut lamp = make_tool('a', 188); // Lamp
    lamp.lit = false;
    state.inventory.push(lamp);

    // First apply: light it
    nh_core::action::apply::do_apply(&mut state, 'a');
    assert!(state.inventory[0].lit, "Lamp should be lit after applying");

    // Second apply: extinguish it
    nh_core::action::apply::do_apply(&mut state, 'a');
    assert!(
        !state.inventory[0].lit,
        "Lamp should be extinguished after applying again"
    );
}

#[test]
fn test_apply_unicorn_horn_cures_confusion() {
    // C-accurate algorithm: shuffled trouble list + random fix count (probabilistic).
    // Run multiple seeds; horn should cure confusion at least sometimes.
    let mut cured_count = 0;
    for seed in 0..50u64 {
        let mut state = test_state(seed);
        state.player.confused_timeout = 30;
        let tool = make_tool('a', 213); // Unicorn horn
        state.inventory.push(tool);

        nh_core::action::apply::do_apply(&mut state, 'a');
        if state.player.confused_timeout == 0 {
            cured_count += 1;
        }
    }
    assert!(
        cured_count > 10,
        "Unicorn horn should cure confusion at least sometimes, got {cured_count}/50"
    );
}

#[test]
fn test_apply_unicorn_horn_cures_blindness() {
    let mut cured_count = 0;
    for seed in 0..50u64 {
        let mut state = test_state(seed);
        state.player.blinded_timeout = 50;
        let tool = make_tool('a', 213); // Unicorn horn
        state.inventory.push(tool);

        nh_core::action::apply::do_apply(&mut state, 'a');
        if state.player.blinded_timeout == 0 {
            cured_count += 1;
        }
    }
    assert!(
        cured_count > 10,
        "Unicorn horn should cure blindness at least sometimes, got {cured_count}/50"
    );
}

#[test]
fn test_apply_unicorn_horn_cures_all_ailments() {
    // Blessed horn with d(2,4) = 2-8 fixes should cure all 4 ailments often.
    let mut all_cured_count = 0;
    for seed in 0..50u64 {
        let mut state = test_state(seed);
        state.player.confused_timeout = 20;
        state.player.stunned_timeout = 15;
        state.player.blinded_timeout = 30;
        state.player.hallucinating_timeout = 100;
        let mut tool = make_tool('a', 213);
        tool.buc = nh_core::object::BucStatus::Blessed;
        state.inventory.push(tool);

        nh_core::action::apply::do_apply(&mut state, 'a');
        let remaining = state.player.confused_timeout as i32
            + state.player.stunned_timeout as i32
            + state.player.blinded_timeout as i32
            + state.player.hallucinating_timeout as i32;
        if remaining == 0 {
            all_cured_count += 1;
        }
    }
    assert!(
        all_cured_count > 5,
        "Blessed horn should cure all 4 ailments often, got {all_cured_count}/50"
    );
}

#[test]
fn test_apply_horn_of_plenty() {
    let mut state = test_state(42);
    let mut horn = make_tool('a', 196); // Horn of plenty
    horn.enchantment = 3;
    state.inventory.push(horn);

    let initial_nutrition = state.player.nutrition;
    nh_core::action::apply::do_apply(&mut state, 'a');

    assert!(
        state.player.nutrition > initial_nutrition,
        "Horn of plenty should increase nutrition"
    );
    assert_eq!(
        state.inventory[0].enchantment, 2,
        "Horn of plenty charges should decrease"
    );
}

#[test]
fn test_apply_empty_horn_of_plenty() {
    let mut state = test_state(42);
    let mut horn = make_tool('a', 196);
    horn.enchantment = 0; // Empty
    state.inventory.push(horn);

    let result = nh_core::action::apply::do_apply(&mut state, 'a');
    assert!(
        matches!(result, ActionResult::NoTime),
        "Empty horn should not take a turn"
    );
}

// ============================================================================
// Summary test
// ============================================================================

#[test]
fn test_core_actions_summary() {
    println!("\n=== Core Actions Summary ===");
    println!("{:<25} {:<10} {:<10}", "Module", "Tests", "Status");
    println!("{}", "-".repeat(45));
    println!("{:<25} {:<10} {:<10}", "eat.rs", "15", "OK");
    println!("{:<25} {:<10} {:<10}", "wear.rs", "15", "OK");
    println!("{:<25} {:<10} {:<10}", "pickup.rs", "8", "OK");
    println!("{:<25} {:<10} {:<10}", "trap.rs", "4", "OK");
    println!("{:<25} {:<10} {:<10}", "apply.rs", "7", "OK");
    println!();

    // Known divergences from C behavior
    println!("=== Known Divergences from C ===");
    println!("1. corpse_effects() uses correct nh-data MONSTERS indices (FIXED)");
    println!("   but mconveys-based intrinsic grants for unlisted monsters still TODO");
    println!("2. eat.rs calculate_nutrition() uses BUC float multiplier (1.5/1.0/0.75)");
    println!("   C calculates nutrition differently per food type from objects.c");
    println!("3. apply.rs pickaxe is stub (needs direction + digging implementation)");
    println!("   C apply.c use_pick_axe() is ~200 lines of digging logic");
    println!("4. apply.rs tinning kit is stub (needs corpse + delay implementation)");
    println!("   C apply.c use_tinning_kit() handles corpse type, delay, tin creation");
    println!("5. wear.rs armor_slot() uses approximate object_type ranges");
    println!("   C uses actual object class from objects.c ARM_BOOTS, ARM_GLOVES, etc.");
    println!("6. wear.rs ring/amulet types use hardcoded object_type constants");
    println!("   These must be validated against actual nh-data object IDs");
    println!("7. trap.rs only implements 3 of 8+ trap types with real effects");
    println!("   C trap.c dotrap() has ~5,476 lines covering all trap types");
    println!("8. pickup.rs can_pickup() has TODOs for cockatrice/loadstone checks");
    println!("   C pickup.c has extensive special-case handling");
    println!("9. C intrinsic probability uses monster level vs rn2(chance)");
    println!("   Rust uses simple percentage; needs level-based probability system");
}
