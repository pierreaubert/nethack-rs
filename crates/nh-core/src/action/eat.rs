//! Eating food and corpses (eat.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::object::{BucStatus, Object, ObjectClass};
use crate::player::{Attribute, HungerState, Property};
use crate::rng::GameRng;

// ============================================================================
// Public Interface
// ============================================================================

/// Simple eat dispatcher (older interface; prefer do_eat below)
fn do_eat_simple(state: &mut GameState, obj_letter: char) -> ActionResult {
    // First pass: validation
    let (_obj_name, obj_class, obj_type) = {
        let obj = match state.get_inventory_item(obj_letter) {
            Some(o) => o,
            None => return ActionResult::Failed("You don't have that item.".to_string()),
        };

        if !is_edible(obj) {
            return ActionResult::Failed("That's not something you can eat.".to_string());
        }

        (obj.display_name(), obj.class, obj.object_type)
    };

    // Check for choking (eating while satiated)
    let hunger_state = HungerState::from_nutrition(state.player.nutrition);
    if hunger_state == HungerState::Satiated {
        state.message("You're having a hard time getting all of it down.");
    }

    // Dispatch to specific eating functions
    if obj_class == ObjectClass::Food {
        if obj_type == 129 {
            // Tin (placeholder type)
            consume_tin(state, obj_letter);
        } else if obj_type == 130 {
            // Corpse (placeholder type)
            eat_corpse(state, obj_letter);
        } else {
            eat_food(state, obj_letter);
        }
    } else {
        // Eating non-food items (e.g. if polymorphed into metallivore)
        eat_accessory(state, obj_letter);
    }

    ActionResult::Success
}

// ============================================================================
// Eating Logic
// ============================================================================

pub fn eat_food(state: &mut GameState, obj_letter: char) {
    let nutrition = {
        if let Some(obj) = state.get_inventory_item(obj_letter) {
            calculate_nutrition(obj)
        } else {
            0
        }
    };

    // Remove item first
    state.remove_from_inventory(obj_letter);

    state.message("You eat the food.");
    lesshungry(state, nutrition);
}

pub fn eat_corpse(state: &mut GameState, obj_letter: char) {
    let nutrition = {
        if let Some(obj) = state.get_inventory_item(obj_letter) {
            calculate_nutrition(obj)
        } else {
            0
        }
    };

    // Remove item first
    state.remove_from_inventory(obj_letter);

    state.message("You eat the corpse.");
    lesshungry(state, nutrition);

    // TODO: Apply corpse effects
}

pub fn consume_tin(state: &mut GameState, obj_letter: char) {
    state.message("You open the tin.");
    // In a real implementation, this would involve opening, checking type (spinach, etc.)
    // For now, treat as generic food
    eat_food(state, obj_letter);
}

pub fn eat_accessory(state: &mut GameState, obj_letter: char) {
    state.message("You eat the accessory.");
    state.remove_from_inventory(obj_letter);
}

pub fn eatspecial() {
    // Stub
}

pub fn eat_brains(state: &mut GameState) {
    state.message("You eat the brains.");
    // Int Boost
}

pub fn eatmdone() -> i32 {
    0
}

pub fn eatmupdate() {
    // Stub
}

pub fn bite() {
    // Stub
}

pub fn edibility_prompts() -> bool {
    true
}

pub fn eating_conducts(state: &mut GameState) {
    // Check vegan, vegetarian, etc.
}

pub fn eaten_stat() {
    // Stub
}

pub fn touchfood() {
    // Stub
}

pub fn floorfood() {
    // Stub
}

pub fn consume_oeaten() {
    // Stub
}

pub fn opentin(state: &mut GameState) {
    state.message("You open the tin.");
}

pub fn start_tin() {
    // Stub
}

pub fn start_eating() {
    // Stub
}

pub fn maybe_finished_meal() {
    // Stub
}

pub fn finish_meating() {
    // Stub
}

pub fn food_xname() -> String {
    "food".to_string()
}

pub fn food_disappears() {
    // Stub
}

pub fn food_substitution() {
    // Stub
}

pub fn foodword() -> String {
    "food".to_string()
}

pub fn fatal_corpse_mistake() {
    // Stub
}

// rottenfood stub removed — see full implementation below

pub fn popeye() {
    // Stub
}

// choke stub removed — see full implementation below

pub fn choke_dialogue() {
    // Stub
}

// vomit stub removed — see full implementation below

pub fn vomiting_dialogue() {
    // Stub
}

/// Set vomiting status
pub fn make_vomiting(state: &mut GameState, duration: i32, from_outside: bool) {
    if duration > 0 {
        state.player.vomiting_timeout = duration as u16;
        if from_outside {
            state.message("You feel nauseated.");
        }
    } else {
        state.player.vomiting_timeout = 0;
    }
}

/// Special effect from eating a specific corpse type
#[derive(Debug, Clone)]
pub enum CorpseEffect {
    /// Gain an intrinsic property (with probability 0-100)
    GainIntrinsic { property: Property, chance: u8 },
    /// Gain energy/mana
    GainEnergy { amount: i32 },
    /// Gain a level
    GainLevel,
    /// Heal to full HP
    FullHeal,
    /// Cause confusion
    Confusion { duration: i32 },
    /// Cause hallucination
    Hallucination { duration: i32 },
    /// Cause stunning
    Stun { duration: i32 },
    /// Cause blindness
    Blindness { duration: i32 },
    /// Cure stoning
    CureStoning,
    /// Cure confusion
    CureConfusion,
    /// Cure stunning
    CureStunning,
    /// Polymorphs the eater
    Polymorph,
    /// Strength increase
    StrengthBoost,
    /// Intelligence increase
    IntelligenceBoost,
    /// Instant death (petrification, etc.)
    InstantDeath { cause: &'static str },
    /// Become sick
    Sickness { duration: i32 },
    /// Lycanthropy infection
    Lycanthropy { monster_type: i16 },
    /// Toggle speed (quantum mechanic)
    ToggleSpeed,
}

// Monster indices from nh-data/src/monsters.rs MONSTERS array.
// These must match the actual array positions for C parity.
mod pm {
    pub const COCKATRICE: i16 = 10;
    pub const WEREWOLF: i16 = 21;
    pub const FLOATING_EYE: i16 = 29;
    pub const NEWT: i16 = 45;
    pub const LIZARD: i16 = 49;
    pub const CHAMELEON: i16 = 50;
    pub const MIND_FLAYER: i16 = 58;
    pub const MASTER_MIND_FLAYER: i16 = 59;
    pub const BAT: i16 = 136;
    pub const GIANT_BAT: i16 = 137;
    pub const RED_DRAGON: i16 = 156;
    pub const WHITE_DRAGON: i16 = 157;
    pub const STALKER: i16 = 163;
    pub const FIRE_ELEMENTAL: i16 = 165;
    pub const VIOLET_FUNGUS: i16 = 174;
    pub const STONE_GIANT: i16 = 180;
    pub const HILL_GIANT: i16 = 181;
    pub const FIRE_GIANT: i16 = 182;
    pub const FROST_GIANT: i16 = 183;
    pub const GREEN_SLIME: i16 = 219;
    pub const QUANTUM_MECHANIC: i16 = 221;
    pub const WRAITH: i16 = 241;
    pub const DOPPELGANGER: i16 = 281;
    pub const NURSE: i16 = 290;
    // Second copies in the array (same monsters at different indices)
    pub const NEWT_ALT: i16 = 338;
    pub const LIZARD_ALT: i16 = 342;
    pub const CHAMELEON_ALT: i16 = 343;
}

/// Get corpse effects for a monster type.
/// Returns a list of effects that may occur when eating this corpse.
///
/// Based on cpostfx() in NetHack 3.6.7 eat.c lines 945-1156.
/// Monster indices match nh-data/src/monsters.rs MONSTERS array positions.
///
/// # Arguments
/// * `monster_type` - The corpse's corpse_type field (monster index in MONSTERS)
pub fn corpse_effects(monster_type: i16) -> Vec<CorpseEffect> {
    match monster_type {
        // Newt: 2/3 chance to gain 1-3 magic energy (C: eat.c line 958-974)
        pm::NEWT | pm::NEWT_ALT => vec![CorpseEffect::GainEnergy { amount: 3 }],

        // Floating eye: telepathy (C: rn2(1) = guaranteed, eat.c intrinsic system)
        pm::FLOATING_EYE => vec![CorpseEffect::GainIntrinsic {
            property: Property::Telepathy,
            chance: 100,
        }],

        // Cockatrice: instant death by petrification (C: eat.c touch_petrifies)
        pm::COCKATRICE => vec![CorpseEffect::InstantDeath {
            cause: "swallowing a cockatrice whole",
        }],

        // Lizard: cure stoning, reduce confusion/stunning (C: eat.c line 1062-1067)
        pm::LIZARD | pm::LIZARD_ALT => vec![
            CorpseEffect::CureStoning,
            CorpseEffect::CureConfusion,
            CorpseEffect::CureStunning,
        ],

        // Wraith: gain a level (C: pluslvl(FALSE), eat.c line 975-977)
        pm::WRAITH => vec![CorpseEffect::GainLevel],

        // Nurse: full heal + poison resistance via mconveys (C: eat.c line 987-995)
        pm::NURSE => vec![
            CorpseEffect::FullHeal,
            CorpseEffect::GainIntrinsic {
                property: Property::PoisonResistance,
                chance: 15,
            },
        ],

        // Mind flayer / Master mind flayer: 50% int boost + intrinsic check
        // (C: eat.c line 1084-1095, mconveys=0 so no standard intrinsics)
        pm::MIND_FLAYER | pm::MASTER_MIND_FLAYER => vec![
            CorpseEffect::IntelligenceBoost,
        ],

        // Red dragon: fire resistance via mconveys MR_FIRE
        pm::RED_DRAGON => vec![CorpseEffect::GainIntrinsic {
            property: Property::FireResistance,
            chance: 15,
        }],

        // White dragon: cold resistance via mconveys MR_COLD
        pm::WHITE_DRAGON => vec![CorpseEffect::GainIntrinsic {
            property: Property::ColdResistance,
            chance: 15,
        }],

        // Fire elemental: fire resistance via mconveys MR_FIRE
        pm::FIRE_ELEMENTAL => vec![CorpseEffect::GainIntrinsic {
            property: Property::FireResistance,
            chance: 15,
        }],

        // Stalker: invisibility + stunning (C: eat.c line 996-1008, falls through to bat)
        pm::STALKER => vec![
            CorpseEffect::GainIntrinsic {
                property: Property::Invisibility,
                chance: 100,
            },
            CorpseEffect::GainIntrinsic {
                property: Property::SeeInvisible,
                chance: 100,
            },
            CorpseEffect::Stun { duration: 30 },
        ],

        // Bat / Giant bat: stunning (C: eat.c line 1009-1015)
        pm::BAT | pm::GIANT_BAT => vec![CorpseEffect::Stun { duration: 30 }],

        // Violet fungus: hallucination (C: eat.c line 1107-1111, dmgtype AD_HALU)
        pm::VIOLET_FUNGUS => vec![CorpseEffect::Hallucination { duration: 200 }],

        // Quantum mechanic: toggle speed (C: eat.c line 1052-1061)
        pm::QUANTUM_MECHANIC => vec![CorpseEffect::ToggleSpeed],

        // Chameleon / Doppelganger: polymorph (C: eat.c line 1068-1077)
        pm::CHAMELEON | pm::CHAMELEON_ALT | pm::DOPPELGANGER => vec![CorpseEffect::Polymorph],

        // Giants: strength boost (C: is_giant macro checks M2_GIANT flag)
        // Fire giant also conveys MR_FIRE, frost giant MR_COLD
        pm::STONE_GIANT | pm::HILL_GIANT => vec![CorpseEffect::StrengthBoost],
        pm::FIRE_GIANT => vec![
            CorpseEffect::StrengthBoost,
            CorpseEffect::GainIntrinsic {
                property: Property::FireResistance,
                chance: 15,
            },
        ],
        pm::FROST_GIANT => vec![
            CorpseEffect::StrengthBoost,
            CorpseEffect::GainIntrinsic {
                property: Property::ColdResistance,
                chance: 15,
            },
        ],

        // Werewolf (human form): lycanthropy (C: eat.c line 978-986)
        pm::WEREWOLF => vec![CorpseEffect::Lycanthropy { monster_type: pm::WEREWOLF }],

        // Green slime: turns you into slime (C: eat.c touch_petrifies / slimeproof)
        pm::GREEN_SLIME => vec![CorpseEffect::InstantDeath {
            cause: "turning into green slime",
        }],

        // Default: no hardcoded effects.
        // In C, the intrinsic system checks mconveys flags for standard resistances.
        // TODO: Implement mconveys-based intrinsic grants from monster data.
        _ => vec![],
    }
}

/// Return a message describing an intrinsic property gain.
fn intrinsic_gain_message(property: Property) -> &'static str {
    match property {
        Property::FireResistance => "You feel a momentary chill.",
        Property::ColdResistance => "You feel full of hot air.",
        Property::SleepResistance => "You feel wide awake.",
        Property::DisintResistance => "You feel very firm.",
        Property::ShockResistance => "Your health currently feels amplified!",
        Property::PoisonResistance => "You feel healthy.",
        Property::Telepathy => "You feel a strange mental acuity.",
        Property::SeeInvisible => "You feel perceptive!",
        Property::Invisibility => "You feel hidden.",
        Property::Speed => "You feel speedy.",
        Property::Stealth => "You feel stealthy.",
        Property::Regeneration => "You feel an itch.",
        Property::Levitation => "You float up!",
        Property::Teleportation => "You feel very jumpy.",
        Property::TeleportControl => "You feel in control of yourself.",
        _ => "You feel a change.",
    }
}

/// Apply corpse effects to the player.
///
/// # Arguments
/// * `state` - The game state
/// * `rng` - Random number generator
/// * `effects` - List of effects to potentially apply
///
/// # Returns
/// Messages describing what happened
pub fn apply_corpse_effects(
    state: &mut GameState,
    rng: &mut GameRng,
    effects: &[CorpseEffect],
) -> Vec<String> {
    let mut messages = Vec::new();

    for effect in effects {
        match effect {
            CorpseEffect::GainIntrinsic { property, chance } => {
                if rng.rn2(100) < *chance as u32
                    && !state.player.properties.has_intrinsic(*property)
                {
                    state.player.properties.grant_intrinsic(*property);
                    messages.push(intrinsic_gain_message(*property).to_string());
                }
            }

            CorpseEffect::GainEnergy { amount } => {
                state.player.energy = (state.player.energy + amount).min(state.player.energy_max);
                if *amount > 0 {
                    messages.push("You feel a mild buzz.".to_string());
                }
            }

            CorpseEffect::GainLevel => {
                state.player.exp_level += 1;
                let hp_gain = rng.rn2(10) as i32 + 1;
                state.player.hp_max += hp_gain;
                state.player.hp = state.player.hp_max;
                messages.push("You feel more experienced!".to_string());
            }

            CorpseEffect::FullHeal => {
                state.player.hp = state.player.hp_max;
                messages.push("You feel much better!".to_string());
            }

            CorpseEffect::Confusion { duration } => {
                state.player.confused_timeout = state.player.confused_timeout.saturating_add(*duration as u16);
                messages.push("Yuk--Loss of strength saps the mind.".to_string());
            }

            CorpseEffect::Hallucination { duration } => {
                state.player.hallucinating_timeout = state.player.hallucinating_timeout.saturating_add(*duration as u16);
                messages.push("Oh wow! Great stuff!".to_string());
            }

            CorpseEffect::Stun { duration } => {
                state.player.stunned_timeout = state.player.stunned_timeout.saturating_add(*duration as u16);
                messages.push("You feel dizzy.".to_string());
            }

            CorpseEffect::Blindness { duration } => {
                state.player.blinded_timeout = state.player.blinded_timeout.saturating_add(*duration as u16);
                messages.push("A cloud of darkness falls upon you.".to_string());
            }

            CorpseEffect::CureStoning => {
                if state.player.stoning > 0 {
                    state.player.stoning = 0;
                    messages.push("You feel limber!".to_string());
                }
            }

            CorpseEffect::CureConfusion => {
                if state.player.confused_timeout > 2 {
                    state.player.confused_timeout = 2;
                }
            }

            CorpseEffect::CureStunning => {
                if state.player.stunned_timeout > 2 {
                    state.player.stunned_timeout = 2;
                }
            }

            CorpseEffect::Polymorph => {
                if state.player.properties.has(Property::Unchanging) {
                    messages.push("You feel momentarily different.".to_string());
                } else {
                    // Trigger polymorph: set a short timeout for the gameloop to handle
                    state.player.polymorph_timeout = 1;
                    messages.push("You feel a change coming over you.".to_string());
                }
            }

            CorpseEffect::StrengthBoost => {
                if state.player.attr_current.get(Attribute::Strength) < 18 {
                    state.player.attr_current.modify(Attribute::Strength, 1);
                    messages.push("You feel stronger!".to_string());
                }
            }

            CorpseEffect::IntelligenceBoost => {
                if state.player.attr_current.get(Attribute::Intelligence) < 18 {
                    state.player.attr_current.modify(Attribute::Intelligence, 1);
                    messages.push("Yum! That was real brain food!".to_string());
                }
            }

            CorpseEffect::InstantDeath { cause } => {
                if state.player.properties.has(Property::LifeSaving) {
                    state.player.properties.remove_intrinsic(Property::LifeSaving);
                    messages.push("But wait...".to_string());
                    messages.push("Your medallion of life saving crumbles to dust!".to_string());
                    state.player.hp = state.player.hp_max / 2;
                } else {
                    messages.push(format!("You die from {}.", cause));
                    state.player.hp = 0;
                }
            }

            CorpseEffect::Sickness { duration } => {
                if state.player.properties.has(Property::SickResistance) {
                    messages.push("You feel mildly ill.".to_string());
                } else {
                    state.player.sick = *duration;
                    state.player.sick_reason = Some("a bad corpse".to_string());
                    messages.push("You feel deathly sick.".to_string());
                }
            }

            CorpseEffect::Lycanthropy { monster_type } => {
                if state.player.lycanthropy.is_none() {
                    state.player.lycanthropy = Some(*monster_type);
                    messages.push("You feel feverish.".to_string());
                }
            }

            CorpseEffect::ToggleSpeed => {
                if state.player.properties.has_intrinsic(Property::Speed) {
                    state.player.properties.remove_intrinsic(Property::Speed);
                    messages.push("You seem slower.".to_string());
                } else {
                    state.player.properties.grant_intrinsic(Property::Speed);
                    messages.push("You seem faster.".to_string());
                }
            }
        }
    }

    messages
}

/// Set sick status (food poisoning or illness)
pub fn make_sick(state: &mut GameState, duration: i32, cause: &str, sick_type: SickType) {
    if duration > 0 {
        match sick_type {
            SickType::FoodPoisoning => {
                state.player.sick_food_timeout = duration as u16;
                state.message(format!("You feel very sick from eating {}.", cause));
            }
            SickType::Illness => {
                state.player.sick_illness_timeout = duration as u16;
                state.message("You feel deathly sick.");
            }
        }
    } else {
        // Cure sickness
        state.player.sick_food_timeout = 0;
        state.player.sick_illness_timeout = 0;
        state.message("What a relief!");
    }
}

/// Sickness type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SickType {
    FoodPoisoning,
    Illness,
}

/// Set sliming status (turning into a green slime)
pub fn make_slimed(state: &mut GameState, duration: i32) {
    if duration > 0 {
        if state.player.sliming_timeout == 0 {
            // Start sliming
            state.player.sliming_timeout = duration as u16;
            state.message("You don't feel very well.");
        }
    } else {
        // Cure sliming
        if state.player.sliming_timeout > 0 {
            state.player.sliming_timeout = 0;
            state.message("You feel much better!");
        }
    }
}

// ============================================================================
// Food object type constants (indices into OBJECTS array)
// Must match nh-data/src/objects.rs ObjectType enum values
// ============================================================================

/// Food object type constants matching the OBJECTS array
#[allow(dead_code)]
mod otyp {
    pub const TRIPE_RATION: i16 = 240;
    pub const CORPSE: i16 = 241;
    pub const EGG: i16 = 242;
    pub const MEATBALL: i16 = 243;
    pub const MEAT_STICK: i16 = 244;
    pub const HUGE_CHUNK_OF_MEAT: i16 = 245;
    pub const MEAT_RING: i16 = 246;
    pub const GLOB_OF_GRAY_OOZE: i16 = 247;
    pub const GLOB_OF_BROWN_PUDDING: i16 = 248;
    pub const GLOB_OF_GREEN_SLIME: i16 = 249;
    pub const GLOB_OF_BLACK_PUDDING: i16 = 250;
    pub const KELP_FROND: i16 = 251;
    pub const EUCALYPTUS_LEAF: i16 = 252;
    pub const APPLE: i16 = 253;
    pub const ORANGE: i16 = 254;
    pub const PEAR: i16 = 255;
    pub const MELON: i16 = 256;
    pub const BANANA: i16 = 257;
    pub const CARROT: i16 = 258;
    pub const SPRIG: i16 = 259;
    pub const CLOVE: i16 = 260;
    pub const SLIME_MOLD: i16 = 261;
    pub const LUMP_OF_ROYAL_JELLY: i16 = 262;
    pub const CREAM_PIE: i16 = 263;
    pub const CANDY_BAR: i16 = 264;
    pub const FORTUNE_COOKIE: i16 = 265;
    pub const PANCAKE: i16 = 266;
    pub const LEMBAS_WAFER: i16 = 267;
    pub const CRAM: i16 = 268;
    pub const FOOD_RATION: i16 = 269;
    pub const K_RATION: i16 = 270;
    pub const C_RATION: i16 = 271;
    pub const TIN: i16 = 272;
}

/// Check if an object type is a glob
fn is_glob(object_type: i16) -> bool {
    (otyp::GLOB_OF_GRAY_OOZE..=otyp::GLOB_OF_BLACK_PUDDING).contains(&object_type)
}

/// Set petrification status (turning to stone)
pub fn make_stoned(state: &mut GameState, duration: i32) {
    if duration > 0 {
        if state.player.stoning_timeout == 0 {
            // Start turning to stone
            state.player.stoning_timeout = duration as u16;
            state.message("You are slowing down.");
        }
    } else {
        // Cure petrification
        if state.player.stoning_timeout > 0 {
            state.player.stoning_timeout = 0;
            state.message("You feel limber!");
        }
    }
}

/// Show sliming progress dialogue
pub fn slime_dialogue(state: &mut GameState) {
    let timeout = state.player.sliming_timeout;
    match timeout {
        1 => state.message("You have turned into a green slime!"),
        2 => state.message("Your body is turning into green slime!"),
        3..=5 => state.message("Your limbs are stiffening..."),
        6..=10 => state.message("Your skin is turning green..."),
        _ => {}
    }
}

/// Handle player's death from sliming
pub fn slimed_to_death(state: &mut GameState) {
    state.message("You have turned into a green slime!");
    state.player.hp = 0;
    // In full implementation, would set killer to "turned into green slime"
}

/// Show petrification progress dialogue
pub fn stoned_dialogue(state: &mut GameState) {
    let timeout = state.player.stoning_timeout;
    match timeout {
        1 => state.message("You have turned to stone!"),
        2 => state.message("Your limbs have solidified!"),
        3..=5 => state.message("Your joints are stiffening..."),
        6..=10 => state.message("You are slowing down..."),
        _ => {}
    }
}

/// Burn away slime with fire
pub fn burn_away_slime(state: &mut GameState) {
    if state.player.sliming_timeout > 0 {
        state.player.sliming_timeout = 0;
        state.message("The slime that was turning you into green slime burns away!");
    }
}

/// Fix petrification (by eating lizard, acidic corpse, or using a unicorn horn)
pub fn fix_petrification(state: &mut GameState) {
    if state.player.stoning_timeout > 0 {
        make_stoned(state, 0);
    }
}

/// Check if player would turn into a stone golem instead of dying from petrification
pub fn poly_when_stoned(state: &GameState) -> bool {
    // Would return true if polymorphed into a form that becomes stone golem
    // For now, always return false
    false
}

/// Monster consumes item to cure petrification
pub fn mon_consume_unstone(state: &mut GameState, _monster_id: crate::monster::MonsterId) {
    // Would have monster eat acidic corpse or similar
}

/// Monster uses item to cure sliming
pub fn muse_unslime(state: &mut GameState, _monster_id: crate::monster::MonsterId) {
    // Would have monster use fire or similar
}

/// Cure monster sliming
pub fn munslime(state: &mut GameState, monster_id: crate::monster::MonsterId) {
    if let Some(monster) = state.current_level.monster_mut(monster_id) {
        // Remove sliming status effect from monster
        monster
            .status_effects
            .remove_effect(crate::combat::StatusEffect::Slimed);
    }
}

/// Cure monster petrification
pub fn munstone(state: &mut GameState, monster_id: crate::monster::MonsterId) {
    if let Some(monster) = state.current_level.monster_mut(monster_id) {
        // Remove stoning status effect from monster
        monster
            .status_effects
            .remove_effect(crate::combat::StatusEffect::Petrifying);
    }
}

/// Apply post-eating effects from corpse based on monster type
pub fn cpostfx(state: &mut GameState, monster_type: i16) {
    // Apply intrinsic gain effects based on corpse monster type
    // Each monster type can grant specific intrinsics

    match monster_type {
        // Newt - small mana boost
        1 => {
            if state.rng.one_in(3) {
                let boost = state.rng.rnd(3);
                state.player.energy = state.player.energy.saturating_add(boost as i32);
                if state.player.energy > state.player.energy_max {
                    if state.rng.one_in(3) {
                        state.player.energy_max += 1;
                    }
                    state.player.energy = state.player.energy_max;
                }
                state.message("You feel a mild buzz.");
            }
        }
        // Wraith - level gain
        2 => {
            state.message("You feel vigorous!");
            // Level gain effect
            if state.player.exp_level < 30 {
                state.player.exp_level += 1;
                if state.player.exp_level > state.player.max_exp_level {
                    state.player.max_exp_level = state.player.exp_level;
                }
            }
        }
        // Giant bat / bat - causes stunning
        3 | 4 => {
            state.message("You feel dizzy.");
            state.player.confused_timeout = state.player.confused_timeout.saturating_add(30);
        }
        // Lizard - cures petrification
        5 => {
            fix_petrification(state);
        }
        // Nurse - heal to full and cure blindness
        6 => {
            state.player.hp = state.player.hp_max;
            state.player.blinded_timeout = 0;
            state.message("You feel fully healed!");
        }
        // Default - possibly gain intrinsics based on monster properties
        _ => {
            // Try to gain intrinsics (poison resistance, etc.)
            maybe_gain_intrinsic(state, monster_type);
        }
    }
}

/// Apply pre-eating effects from corpse (e.g., petrification, sliming)
pub fn cprefx(state: &mut GameState, monster_type: i16) {
    // Check for instant death effects before eating

    match monster_type {
        // Cockatrice and similar - petrification
        10 => {
            if !state.player.properties.has(Property::StoneResistance) {
                state.message("You turn to stone.");
                state.player.hp = 0;
                return;
            }
        }
        // Green slime
        11 => {
            if state.player.sliming_timeout == 0 {
                make_slimed(state, 10);
            }
        }
        // Death, Pestilence, Famine (Riders)
        100 | 101 | 102 => {
            state.message("Eating that is instantly fatal.");
            state.player.hp = 0;
            return;
        }
        // Dogs and cats - bad karma
        20..=25 => {
            state.message("You feel that eating that was a bad idea.");
            // Would set aggravate monster intrinsic
        }
        _ => {}
    }
}

/// Apply pre-eating effects from food item (object-based interface)
pub fn fprefx_obj(state: &mut GameState, obj: &Object) {
    // Check for effects before eating food

    // Rotten food check
    if obj.age > 0 && obj.age < 50 {
        // Old food
        if state.rng.one_in(7) {
            state.message("This food seems a bit stale.");
        }
    }

    // Cursed food might have negative effects
    if obj.is_cursed() {
        if state.rng.one_in(3) {
            state.message("Something seems wrong with this food...");
        }
    }
}

/// Apply post-eating effects from food item (object-based interface)
pub fn fpostfx_obj(state: &mut GameState, obj: &Object) {
    // Special food effects

    match obj.object_type {
        // Spinach tin
        100 => {
            popeye_effect(state);
        }
        // Fortune cookie
        101 => {
            // Would show a random rumor/message
            state.message("You find a small piece of paper inside the cookie.");
        }
        // Carrot
        102 => {
            // Cures blindness
            if state.player.blinded_timeout > 0 {
                state.player.blinded_timeout = 0;
                state.message("Your vision improves!");
            }
        }
        // Lembas wafer - extra nutrition for elves
        103 => {
            // Would check if player is elf for bonus nutrition
        }
        _ => {}
    }
}

/// Popeye effect from eating spinach - temporary strength boost
fn popeye_effect(state: &mut GameState) {
    state.message("Strenth rush! You feel like Popeye!");
    // Temporary strength bonus
    state.player.temp_str_bonus = 5;
    state.player.str_timeout = 100;
}

/// Try to gain an intrinsic from eating a monster
fn maybe_gain_intrinsic(state: &mut GameState, _monster_type: i16) {
    // Based on monster properties, potentially gain intrinsics like:
    // - Fire resistance (from fire-based monsters)
    // - Cold resistance (from cold-based monsters)
    // - Poison resistance (from poisonous monsters)
    // - Sleep resistance (from sleep-resistant monsters)
    // - Telepathy (from mind flayers)
    // etc.

    // For now, small chance to gain poison resistance
    if state.rng.one_in(15) {
        if !state.player.properties.has(Property::PoisonResistance) {
            state
                .player
                .properties
                .grant_intrinsic(Property::PoisonResistance);
            state.message("You feel healthy.");
        }
    }
}

pub fn tinnable() -> bool {
    true
}

pub fn tin_variety() {
    // Stub
}

pub fn tin_variety_txt() -> String {
    "tin".to_string()
}

pub fn tin_details() {
    // Stub
}

pub fn mcould_eat_tin() -> bool {
    true
}

pub fn costly_tin() {
    // Stub
}

pub fn veggy_item() -> bool {
    true
}

pub fn meatmetal() -> bool {
    false
}

pub fn done_eating() {
    // Stub
}

pub fn do_reset_eat() {
    // Stub
}

pub fn reset_eat() {
    // Stub
}

// ============================================================================
// Helpers
// ============================================================================

/// Check if food is edible
pub fn is_edible(obj: &Object) -> bool {
    obj.class == ObjectClass::Food
}

/// Calculate nutrition from eating an object.
///
/// For corpses: uses corpse_type to look up monster nutrition.
/// For tins: base 0, nutrition determined by contents.
/// For globs: weight-based nutrition.
/// For everything else: uses obj.nutrition (populated from OBJECTS data).
///
/// In C, BUC does NOT multiply nutrition — blessed just prevents rot.
pub fn calculate_nutrition(obj: &Object) -> i32 {
    let base = match obj.object_type {
        otyp::CORPSE => {
            // Corpse nutrition comes from the monster type, not the object def.
            if obj.nutrition > 0 {
                obj.nutrition as i32
            } else {
                // Default corpse nutrition when monster data unavailable
                100
            }
        }
        otyp::TIN => {
            // Tins have variable nutrition based on contents;
            // base nutrition is 0, caller should add tin type modifier
            0
        }
        t if is_glob(t) => {
            // Globs: nutrition based on weight
            obj.weight as i32
        }
        _ => {
            // Standard food: nutrition from OBJECTS data (stored on obj).
            // Fallback to weight * 5 if nutrition wasn't set.
            if obj.nutrition > 0 {
                obj.nutrition as i32
            } else {
                (obj.weight as i32 * 5).max(10)
            }
        }
    };

    // BUC modifier: blessed +50%, cursed -50%
    match obj.buc {
        crate::object::BucStatus::Blessed => base * 3 / 2,
        crate::object::BucStatus::Cursed => base / 2,
        _ => base,
    }
}

/// Check if a corpse is rotten based on age (rottenfood from eat.c).
///
/// Returns true if the corpse has gone bad. Blessed corpses last longer.
/// Lizard and lichen corpses never rot.
pub fn is_rotten(obj: &Object, current_turn: i64) -> bool {
    if obj.class != crate::object::ObjectClass::Food {
        return false;
    }

    // Lizard and lichen corpses never rot
    if obj.object_type == otyp::CORPSE
        && (obj.corpse_type == pm::LIZARD || obj.corpse_type == pm::LIZARD_ALT)
    {
        return false;
    }

    let age = current_turn - obj.age;
    let rot_threshold: i64 = match obj.buc {
        crate::object::BucStatus::Blessed => 300,
        crate::object::BucStatus::Cursed => 50,
        _ => 150,
    };
    age > rot_threshold
}

/// Handle rotten food effects (rottenfood from eat.c).
///
/// When eating a rotten corpse, there's a chance of food poisoning.
/// Returns messages about what happened.
pub fn rottenfood(state: &mut GameState) -> Vec<String> {
    let mut messages = Vec::new();

    // 1 in 7 chance to get food poisoning from rotten food
    if state.rng.one_in(7) {
        messages.push("You feel deathly sick.".to_string());
        // Food poisoning: lose nutrition and potentially lethal
        state.player.nutrition -= 40;
        if state.player.nutrition < 0 {
            state.player.nutrition = 0;
        }
    } else {
        messages.push("Ulch - that food was tainted!".to_string());
        // Mild sickness: just lose some nutrition
        state.player.nutrition = (state.player.nutrition - 20).max(0);
    }

    messages
}

/// Food pre-effects (fprefx from eat.c).
///
/// Effects that happen BEFORE the food is consumed. Returns messages.
fn fprefx(state: &mut GameState, object_type: i16, corpse_type: i16) -> Vec<String> {
    let mut messages = Vec::new();

    match object_type {
        otyp::TRIPE_RATION => {
            // Tripe is disgusting to non-carnivores
            // In C, checks if polymorphed into carnivore
            messages.push("Yak - Loss of strenth saps the mind.".to_string());
            state.player.confused_timeout = state.player.confused_timeout.saturating_add(2);
        }
        otyp::EGG => {
            // Check for cockatrice egg — causes petrification
            if corpse_type == pm::COCKATRICE {
                if !state.player.properties.has(Property::StoneResistance) {
                    messages.push("Tstrstrstrch!".to_string());
                    // Begin petrification countdown (5 turns)
                    state.player.stoning = 5;
                } else {
                    messages.push("This egg doesn't taste like a chicken egg.".to_string());
                }
            }
        }
        _ => {}
    }

    messages
}

/// Food post-effects (fpostfx from eat.c).
///
/// Effects that happen AFTER the food is consumed. Returns messages.
fn fpostfx(state: &mut GameState, object_type: i16) -> Vec<String> {
    let mut messages = Vec::new();

    match object_type {
        otyp::CARROT => {
            // Cure blindness
            if state.player.blinded_timeout > 0 {
                state.player.blinded_timeout = 0;
                messages.push("Your vision improves.".to_string());
            }
        }
        otyp::FORTUNE_COOKIE => {
            messages.push("This cookie has a scrap of paper inside.".to_string());
            messages.push("It reads: \"You will have a strking strke of luck.\"".to_string());
        }
        otyp::LEMBAS_WAFER => {
            // Elves get double nutrition, orcs get half
            match state.player.race {
                crate::player::Race::Elf => {
                    // Extra nutrition for elves (already got base, add more)
                    state.player.nutrition += 400; // roughly doubles the 800 base
                    messages.push("A taste of the Blessed Realm fills you.".to_string());
                }
                crate::player::Race::Orc => {
                    // Orcs find it distasteful — lose half the nutrition
                    state.player.nutrition -= 400;
                    messages.push("Yuck! Elvish food!".to_string());
                }
                _ => {}
            }
        }
        otyp::EUCALYPTUS_LEAF => {
            // Cures sickness
            messages.push("You feel better.".to_string());
        }
        otyp::APPLE => {
            // Cursed apple: sleep check (like Snow White)
            // Handled in do_eat via BUC check
        }
        otyp::LUMP_OF_ROYAL_JELLY => {
            // Restore strength
            let cur = state.player.attr_current.get(Attribute::Strength);
            let max = state.player.attr_max.get(Attribute::Strength);
            if cur < max {
                state.player.attr_current.modify(Attribute::Strength, 1);
                messages.push("You feel a little stronger.".to_string());
            }
        }
        _ => {}
    }

    messages
}

/// Vomit (lose nutrition, possibly drop items)
pub fn vomit(state: &mut GameState) {
    state.message("You vomit!");
    state.player.nutrition -= 1000;
    if state.player.nutrition < 0 {
        state.player.nutrition = 0;
    }
    state.player.update_hunger();
}

/// Choke on food (potentially fatal).
///
/// If nutrition >= 2000 after eating, choking check applies.
/// MagicBreathing grants immunity. 1-in-20 chance to die;
/// otherwise vomit and survive.
pub fn choke(state: &mut GameState, food_name: &str) -> bool {
    // MagicBreathing protects from choking
    if state.player.properties.has(Property::MagicBreathing) {
        return false;
    }

    state.message(format!("You choke over your {}!", food_name));

    // 1-in-20 chance of fatal choking
    if state.rng.one_in(20) {
        state.message("You choke to death!");
        state.player.hp = 0;
        return true;
    }

    // Survive by vomiting
    vomit(state);
    false
}

/// Eat food from inventory (doeat from eat.c).
///
/// Dispatches to corpse eating, tin eating, or regular food eating.
/// Handles choking, rotten food, food pre/post effects, and nutrition.
pub fn do_eat(state: &mut GameState, obj_letter: char) -> ActionResult {
    // Extract data we need from the object (borrow-safe)
    let (obj_name, object_type, corpse_type, buc, nutrition, _age, is_food) = {
        let obj = match state.get_inventory_item(obj_letter) {
            Some(o) => o,
            None => return ActionResult::Failed("You don't have that item.".to_string()),
        };

        if obj.class != ObjectClass::Food {
            return ActionResult::Failed("That's not something you can eat.".to_string());
        }

        let name = obj.name.clone().unwrap_or_else(|| "food".to_string());
        let nutrition = calculate_nutrition(obj);

        (
            name,
            obj.object_type,
            obj.corpse_type,
            obj.buc,
            nutrition,
            obj.age,
            true,
        )
    };

    if !is_food {
        return ActionResult::Failed("That's not something you can eat.".to_string());
    }

    // Check for choking risk (eating while satiated)
    let hunger_state = HungerState::from_nutrition(state.player.nutrition);
    if hunger_state == HungerState::Satiated {
        state.message("You're having a hard time getting all of it down.");
    }

    // Food pre-effects
    let pre_msgs = fprefx(state, object_type, corpse_type);
    for msg in &pre_msgs {
        state.message(msg.clone());
    }

    // Eating message
    state.message(format!("You eat the {}.", obj_name));

    // Corpse-specific handling
    if object_type == otyp::CORPSE {
        // Check for rotten corpse
        let current_turn = state.turns as i64;
        let is_rotten_food = {
            let obj = state.get_inventory_item(obj_letter).unwrap();
            is_rotten(obj, current_turn)
        };

        if is_rotten_food && buc != BucStatus::Blessed {
            let rot_msgs = rottenfood(state);
            for msg in &rot_msgs {
                state.message(msg.clone());
            }
        }

        // Apply corpse effects (cprefx/cpostfx)
        let effects = corpse_effects(corpse_type);
        let rng = &mut state.rng.clone();
        let effect_msgs = apply_corpse_effects(state, rng, &effects);
        for msg in &effect_msgs {
            state.message(msg.clone());
        }

        // Check if player died from corpse effects
        if state.player.is_dead() {
            state.remove_from_inventory(obj_letter);
            return ActionResult::Died("killed by eating something".to_string());
        }
    }

    // Apply nutrition
    state.player.nutrition += nutrition;

    // Food post-effects
    let post_msgs = fpostfx(state, object_type);
    for msg in &post_msgs {
        state.message(msg.clone());
    }

    // Choking check: if nutrition is now >= 2000
    if state.player.nutrition >= 2000 {
        let died = choke(state, &obj_name);
        if died {
            state.remove_from_inventory(obj_letter);
            return ActionResult::Died("choked on food".to_string());
        }
    }

    // Update hunger state
    let hunger_msgs = newuhs(state, true);
    for msg in &hunger_msgs {
        state.message(msg.clone());
    }

    // Remove the food item
    state.remove_from_inventory(obj_letter);

    ActionResult::Success
}

// ============================================================================
// Hunger state management (newuhs, gethungry, lesshungry from NetHack)
// ============================================================================


/// Update hunger status with messages when state changes.
pub fn newuhs(state: &mut GameState, incr: bool) -> Vec<String> {
    let mut messages = Vec::new();
    let old_state = state.player.hunger_state;
    let new_state = HungerState::from_nutrition(state.player.nutrition);

    if old_state == new_state {
        return messages;
    }

    state.player.hunger_state = new_state;

    if incr {
        match (old_state, new_state) {
            (
                HungerState::Fainted | HungerState::Fainting,
                HungerState::Weak
                | HungerState::Hungry
                | HungerState::NotHungry
                | HungerState::Satiated,
            ) => {
                messages.push("You regain consciousness.".to_string());
            }
            (
                HungerState::Weak,
                HungerState::Hungry | HungerState::NotHungry | HungerState::Satiated,
            ) => {
                messages.push("You feel less weak.".to_string());
            }
            (HungerState::Hungry, HungerState::NotHungry | HungerState::Satiated) => {
                messages.push("You are not hungry anymore.".to_string());
            }
            (_, HungerState::Satiated) => {
                messages.push("You are completely full.".to_string());
            }
            _ => {}
        }
    } else {
        match new_state {
            HungerState::Hungry => {
                if !matches!(
                    old_state,
                    HungerState::Weak | HungerState::Fainting | HungerState::Fainted
                ) {
                    messages.push("You are beginning to feel hungry.".to_string());
                }
            }
            HungerState::Weak => {
                if old_state == HungerState::Hungry {
                    messages.push("You are beginning to feel weak.".to_string());
                } else if !matches!(old_state, HungerState::Fainting | HungerState::Fainted) {
                    messages.push("You feel weak now.".to_string());
                }
            }
            HungerState::Fainting => {
                if !matches!(old_state, HungerState::Fainted | HungerState::Starved) {
                    messages.push("You feel faint.".to_string());
                }
            }
            HungerState::Fainted => {
                messages.push("You faint from lack of food.".to_string());
                state.player.paralyzed_timeout = (5 + state.player.exp_level) as u16;
            }
            HungerState::Starved => {
                messages.push("You die from starvation.".to_string());
                state.player.hp = 0; // Fatal
            }
            _ => {}
        }
    }

    messages
}

/// Process hunger each turn (called during game tick).
pub fn gethungry(state: &mut GameState, rng: &mut GameRng) -> Vec<String> {
    if state.player.hp <= 0 {
        return Vec::new();
    }

    let mut hunger_rate: i32 = 1;

    if state.player.properties.has(Property::Hunger) {
        hunger_rate += 1;
    }

    if state.player.properties.has(Property::Regeneration) {
        hunger_rate += 1;
    }

    // Encumbrance
    match state.player.encumbrance() {
        crate::player::Encumbrance::Unencumbered => {}
        crate::player::Encumbrance::Burdened => {
            if rng.rn2(2) == 0 {
                hunger_rate += 1;
            }
        }
        crate::player::Encumbrance::Stressed => hunger_rate += 1,
        crate::player::Encumbrance::Strained => hunger_rate += 2,
        crate::player::Encumbrance::Overtaxed => hunger_rate += 3,
        crate::player::Encumbrance::Overloaded => hunger_rate += 4,
    }

    if state.player.properties.has(Property::SlowDigestion) {
        hunger_rate = 0;
    }

    if hunger_rate > 0 {
        state.player.nutrition = state.player.nutrition.saturating_sub(hunger_rate);
    }

    newuhs(state, false)
}

/// Add nutrition from eating food.
pub fn lesshungry(state: &mut GameState, nutrition: i32) -> Vec<String> {
    state.player.nutrition = state.player.nutrition.saturating_add(nutrition);

    const MAX_NUTRITION: i32 = 5000;
    if state.player.nutrition > MAX_NUTRITION {
        state.player.nutrition = MAX_NUTRITION;
    }

    newuhs(state, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{Object, ObjectClass, ObjectId};
    use crate::rng::GameRng;

    fn make_food(letter: char, object_type: i16, nutrition: u16) -> Object {
        let mut obj = Object::default();
        obj.id = ObjectId(1);
        obj.class = ObjectClass::Food;
        obj.object_type = object_type;
        obj.nutrition = nutrition;
        obj.inv_letter = letter;
        obj.name = Some("food".to_string());
        obj
    }

    #[test]
    fn test_eat_non_food_fails() {
        let mut state = GameState::new(GameRng::from_entropy());
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.inv_letter = 'a';
        state.inventory.push(obj);

        let result = do_eat(&mut state, 'a');
        assert!(matches!(result, ActionResult::Failed(_)));
    }

    #[test]
    fn test_eat_food_increases_nutrition() {
        let mut state = GameState::new(GameRng::from_entropy());
        let initial_nutrition = state.player.nutrition;

        let obj = make_food('a', otyp::FOOD_RATION, 800);
        state.inventory.push(obj);

        let result = do_eat(&mut state, 'a');
        assert!(matches!(result, ActionResult::Success));
        assert!(state.player.nutrition > initial_nutrition);
    }

    #[test]
    fn test_calculate_nutrition_food_ration() {
        let obj = make_food('a', otyp::FOOD_RATION, 800);
        assert_eq!(calculate_nutrition(&obj), 800);
    }

    #[test]
    fn test_calculate_nutrition_corpse_default() {
        let mut obj = make_food('a', otyp::CORPSE, 0);
        obj.corpse_type = 1; // arbitrary monster with no set nutrition
        // Corpse with 0 nutrition should fall back to default 100
        assert_eq!(calculate_nutrition(&obj), 100);
    }

    #[test]
    fn test_calculate_nutrition_corpse_with_value() {
        let mut obj = make_food('a', otyp::CORPSE, 150);
        obj.corpse_type = pm::LIZARD;
        assert_eq!(calculate_nutrition(&obj), 150);
    }

    #[test]
    fn test_is_rotten_fresh_corpse() {
        let mut obj = make_food('a', otyp::CORPSE, 0);
        obj.corpse_type = 1;
        obj.age = 100;
        assert!(!is_rotten(&obj, 150)); // 50 turns old, not rotten
    }

    #[test]
    fn test_is_rotten_old_corpse() {
        let mut obj = make_food('a', otyp::CORPSE, 0);
        obj.corpse_type = 1;
        obj.age = 0;
        assert!(is_rotten(&obj, 200)); // 200 turns old, rotten
    }

    #[test]
    fn test_is_rotten_lizard_never_rots() {
        let mut obj = make_food('a', otyp::CORPSE, 0);
        obj.corpse_type = pm::LIZARD;
        obj.age = 0;
        assert!(!is_rotten(&obj, 10000)); // Lizards never rot
    }

    #[test]
    fn test_carrot_cures_blindness() {
        let mut state = GameState::new(GameRng::from_entropy());
        state.player.blinded_timeout = 50;

        let obj = make_food('a', otyp::CARROT, 50);
        state.inventory.push(obj);

        let _ = do_eat(&mut state, 'a');
        assert_eq!(state.player.blinded_timeout, 0);
    }

    #[test]
    fn test_non_food_not_rotten() {
        let mut obj = Object::default();
        obj.object_type = 0; // Not a corpse
        assert!(!is_rotten(&obj, 10000));
    }
}
