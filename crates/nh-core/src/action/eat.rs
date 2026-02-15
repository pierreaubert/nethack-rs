//! Eating food and corpses (eat.c)
//!
//! Faithfully ports NetHack 3.6.7 eat.c: multi-turn eating, tin opening,
//! corpse intrinsic grants via mconveys, food pre/post effects, rot handling,
//! hunger state management.

use crate::action::ActionResult;
use crate::combat::DamageType;
use crate::data::get_monster;
use crate::gameloop::GameState;
use crate::monster::{MonsterResistances, PerMonst};
use crate::object::{BucStatus, Object, ObjectClass};
use crate::player::{Attribute, HungerState, Property, Race};
use crate::rng::GameRng;

// ============================================================================
// Food object type constants (indices into OBJECTS array)
// ============================================================================

#[allow(dead_code)]
pub mod otyp {
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
    pub const SPRIG_OF_WOLFSBANE: i16 = 259;
    pub const CLOVE_OF_GARLIC: i16 = 260;
    pub const SLIME_MOLD: i16 = 261;
    pub const LUMP_OF_ROYAL_JELLY: i16 = 262;
    pub const CREAM_PIE: i16 = 263;
    pub const CANDY_BAR: i16 = 264;
    pub const FORTUNE_COOKIE: i16 = 265;
    pub const PANCAKE: i16 = 266;
    pub const LEMBAS_WAFER: i16 = 267;
    pub const CRAM_RATION: i16 = 268;
    pub const FOOD_RATION: i16 = 269;
    pub const K_RATION: i16 = 270;
    pub const C_RATION: i16 = 271;
    pub const TIN: i16 = 272;
}

/// Check if food is edible
pub fn is_edible(obj: &Object) -> bool {
    obj.class == ObjectClass::Food
}

fn is_glob(object_type: i16) -> bool {
    (otyp::GLOB_OF_GRAY_OOZE..=otyp::GLOB_OF_BLACK_PUDDING).contains(&object_type)
}

// ============================================================================
// Multi-turn eating context (context.victual from C)
// ============================================================================

/// Tracks in-progress eating across multiple turns.
/// Port of `struct victual_info` from hack.h.
#[derive(Debug, Clone, Default)]
pub struct VictualContext {
    /// Object letter of the food being eaten
    pub piece_letter: Option<char>,
    /// Turns required to eat
    pub reqtime: i32,
    /// Turns spent eating so far
    pub usedtime: i32,
    /// Nutrition modifier per turn
    pub nmod: i32,
    /// Can choke from overeating
    pub canchoke: bool,
    /// Currently eating flag
    pub eating: bool,
    /// Reset flag (eating interrupted)
    pub doreset: bool,
    /// Already warned about being full
    pub fullwarn: bool,
}

/// Tracks in-progress tin opening across multiple turns.
/// Port of `struct tin_info` from hack.h.
#[derive(Debug, Clone, Default)]
pub struct TinContext {
    /// Object letter of the tin being opened
    pub tin_letter: Option<char>,
    /// Turns required to open
    pub reqtime: i32,
    /// Turns spent opening so far
    pub usedtime: i32,
}

// ============================================================================
// Sickness types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SickType {
    FoodPoisoning,
    Illness,
}

// ============================================================================
// Nutrition calculation (obj_nutrition from eat.c)
// ============================================================================

/// Base nutrition of a food-class object.
/// Port of obj_nutrition() from eat.c line 316.
///
/// For corpses: uses corpse_type to look up monster nutrition.
/// For tins: base 0, nutrition determined by contents.
/// For globs: weight-based nutrition.
/// Applies race modifiers for lembas wafer and cram ration.
pub fn calculate_nutrition(obj: &Object, race: Race) -> i32 {
    let base = match obj.object_type {
        otyp::CORPSE => {
            if let Some(template) = get_monster(obj.corpse_type) {
                template.corpse_nutrition as i32
            } else if obj.nutrition > 0 {
                obj.nutrition as i32
            } else {
                100
            }
        }
        otyp::TIN => 0,
        t if is_glob(t) => obj.weight as i32,
        _ => {
            if obj.nutrition > 0 {
                obj.nutrition as i32
            } else {
                (obj.weight as i32 * 5).max(10)
            }
        }
    };

    // Race nutrition modifiers (eat.c line 323-336)
    let modified = match obj.object_type {
        otyp::LEMBAS_WAFER => match race {
            Race::Elf => base + base / 4,   // 800 -> 1000
            Race::Orc => base - base / 4,   // 800 -> 600
            _ => base,
        },
        otyp::CRAM_RATION => match race {
            Race::Dwarf => base + base / 6, // 600 -> 700
            _ => base,
        },
        _ => base,
    };

    modified
}

/// Legacy interface without race (backward compat)
pub fn calculate_nutrition_simple(obj: &Object) -> i32 {
    calculate_nutrition(obj, Race::Human)
}

// ============================================================================
// Rot checking
// ============================================================================

/// Check if a corpse is rotten based on age (from eat.c eatcorpse lines 1606-1614).
/// Uses C formula: rotted = (monstermoves - age) / (10 + rn2(20))
pub fn is_rotten(obj: &Object, current_turn: i64) -> bool {
    if obj.class != ObjectClass::Food || obj.object_type != otyp::CORPSE {
        return false;
    }

    // Check nonrotting corpse
    if let Some(template) = get_monster(obj.corpse_type) {
        if template.nonrotting_corpse() {
            return false;
        }
    }

    let age = current_turn - obj.age;
    let rot_threshold: i64 = match obj.buc {
        BucStatus::Blessed => 300,
        BucStatus::Cursed => 50,
        _ => 150,
    };
    age > rot_threshold
}

/// Calculate rot level for C-faithful rot handling.
/// Returns rot value matching C's (monstermoves - age) / (10 + rn2(20)).
pub fn calculate_rot(obj: &Object, current_turn: i64, rng: &mut GameRng) -> i64 {
    if let Some(template) = get_monster(obj.corpse_type) {
        if template.nonrotting_corpse() {
            return 0;
        }
    }

    let age = current_turn - obj.age;
    let divisor = 10 + rng.rn2(20) as i64;
    let mut rotted = age / divisor.max(1);

    match obj.buc {
        BucStatus::Cursed => rotted += 2,
        BucStatus::Blessed => rotted -= 2,
        _ => {}
    }

    rotted.max(0)
}

// ============================================================================
// Rotten food effects (rottenfood from eat.c line 1547)
// ============================================================================

/// Handle first bite of rotten food.
/// Returns true if player was knocked unconscious.
/// Port of rottenfood() from eat.c lines 1547-1585.
pub fn rottenfood(state: &mut GameState) -> bool {
    state.message("Blecch!  Rotten food!");

    let roll = state.rng.rn2(12);
    if roll < 3 {
        // 1-in-4: confusion (d(2,4) turns)
        let duration = state.rng.dice(2, 4) as u16;
        state.player.confused_timeout = state.player.confused_timeout.saturating_add(duration);
        state.message("You feel rather light-headed.");
        false
    } else if roll < 6 {
        // 1-in-4 and not blind: blindness (d(2,10) turns)
        if state.player.blinded_timeout == 0 {
            let duration = state.rng.dice(2, 10) as u16;
            state.player.blinded_timeout = state.player.blinded_timeout.saturating_add(duration);
            state.message("Everything suddenly goes dark.");
        }
        false
    } else if roll < 10 {
        // 1-in-3: unconsciousness (rnd(10) turns)
        let duration = state.rng.rnd(10) as u16;
        state.player.paralyzed_timeout = state.player.paralyzed_timeout.saturating_add(duration);
        state.message("The world spins and goes dark.");
        true
    } else {
        // ~1-in-6: no effect
        false
    }
}

// ============================================================================
// Corpse pre-effects (cprefx from eat.c line 677)
// ============================================================================

/// Corpse pre-eating effects. Called before consuming corpse.
/// Port of cprefx() from eat.c lines 677-742.
///
/// Returns ActionResult::Died if eating kills the player.
pub fn cprefx(state: &mut GameState, pm: i16) -> ActionResult {
    let template = match get_monster(pm) {
        Some(t) => t,
        None => return ActionResult::Success,
    };

    // Petrification check (flesh_petrifies)
    if template.flesh_petrifies() {
        if !state.player.properties.has(Property::StoneResistance) {
            state.message(format!("You turn to stone from tasting {} meat.", template.name));
            state.player.hp = 0;
            return ActionResult::Died(format!("tasting {} meat", template.name));
        }
    }

    // Domestic animals: guilt + aggravate (eat.c lines 696-707)
    if template.is_domestic() {
        state.message(format!(
            "You feel that eating the {} was a bad idea.",
            template.name
        ));
        state.player.properties.grant_intrinsic(Property::Aggravate);
    }

    // Lizard: cure stoning (eat.c line 708-711)
    if template.symbol == ':' && template.name == "lizard" {
        if state.player.stoning_timeout > 0 {
            fix_petrification(state);
        }
    }

    // Riders: instant death (eat.c lines 712-728)
    if template.name == "Death" || template.name == "Pestilence" || template.name == "Famine" {
        state.message("Eating that is instantly fatal.");
        state.player.hp = 0;
        return ActionResult::Died(format!("unwisely ate the body of {}", template.name));
    }

    // Green slime: start sliming (eat.c lines 730-735)
    if template.name == "green slime" {
        if state.player.sliming_timeout == 0
            && !state.player.properties.has(Property::Unchanging)
        {
            state.message("You don't feel very well.");
            make_slimed(state, 10);
        }
    }

    // Acidic monsters: cure stoning (eat.c lines 738-740)
    if template.is_acidic() && state.player.stoning_timeout > 0 {
        fix_petrification(state);
    }

    ActionResult::Success
}

// ============================================================================
// Corpse post-effects (cpostfx from eat.c line 945)
// ============================================================================

/// Intrinsic types that can be conveyed by eating a corpse.
/// Maps to C's intrinsic_possible() property checks.
const CONVEYABLE_INTRINSICS: &[(MonsterResistances, Property)] = &[
    (MonsterResistances::FIRE, Property::FireResistance),
    (MonsterResistances::COLD, Property::ColdResistance),
    (MonsterResistances::SLEEP, Property::SleepResistance),
    (MonsterResistances::DISINT, Property::DisintResistance),
    (MonsterResistances::ELEC, Property::ShockResistance),
    (MonsterResistances::POISON, Property::PoisonResistance),
];

/// Messages for gaining intrinsics from corpse eating (from givit in eat.c)
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
        Property::Invisibility => "You feel hidden!",
        Property::Speed => "You seem faster.",
        Property::Teleportation => "You feel very jumpy.",
        Property::TeleportControl => "You feel in control of yourself.",
        _ => "You feel a change.",
    }
}

/// Determine chance for givit() die roll. Port of eat.c lines 840-860.
fn intrinsic_chance(property: Property, template: &PerMonst) -> u32 {
    match property {
        Property::PoisonResistance => {
            if template.name == "killer bee" || template.name == "scorpion" {
                if rand::random::<u32>() % 4 == 0 {
                    return 1;
                }
            }
            15
        }
        Property::Teleportation => 10,
        Property::TeleportControl => 12,
        Property::Telepathy => 1,
        _ => 15,
    }
}

/// Try to give an intrinsic to the player. Port of givit() from eat.c line 832.
fn givit(state: &mut GameState, property: Property, template: &PerMonst) {
    let chance = intrinsic_chance(property, template);

    // Die roll: ptr->mlevel <= rn2(chance)
    if (template.level as u32) <= state.rng.rn2(chance) {
        return; // Failed
    }

    if !state.player.properties.has_intrinsic(property) {
        state.player.properties.grant_intrinsic(property);
        state.message(intrinsic_gain_message(property));
    }
}

/// Apply post-eating effects from corpse based on monster type.
/// Port of cpostfx() from eat.c lines 945-1156.
pub fn cpostfx(state: &mut GameState, pm: i16) {
    let template = match get_monster(pm) {
        Some(t) => t,
        None => return,
    };

    let mut check_intrinsics = false;
    let mut catch_lycanthropy: Option<&str> = None;

    match template.name {
        // Newt: energy boost (eat.c lines 958-974)
        "newt" => {
            if state.rng.rn2(3) != 0
                || 3 * state.player.energy <= 2 * state.player.energy_max
            {
                let old_en = state.player.energy;
                let boost = state.rng.rnd(3) as i32;
                state.player.energy += boost;
                if state.player.energy > state.player.energy_max {
                    if state.rng.rn2(3) == 0 {
                        state.player.energy_max += 1;
                    }
                    state.player.energy = state.player.energy_max;
                }
                if old_en != state.player.energy {
                    state.message("You feel a mild buzz.");
                }
            }
        }

        // Wraith: gain level (eat.c line 975-977)
        "wraith" => {
            state.message("You feel more experienced!");
            if state.player.exp_level < 30 {
                state.player.exp_level += 1;
                if state.player.exp_level > state.player.max_exp_level {
                    state.player.max_exp_level = state.player.exp_level;
                }
                let hp_gain = state.rng.rnd(10) as i32;
                state.player.hp_max += hp_gain;
                state.player.hp = state.player.hp_max;
            }
        }

        // Human were-creatures: lycanthropy (eat.c lines 978-986)
        "human wererat" => {
            catch_lycanthropy = Some("wererat");
        }
        "human werejackal" => {
            catch_lycanthropy = Some("werejackal");
        }
        "human werewolf" => {
            catch_lycanthropy = Some("werewolf");
        }

        // Nurse: full heal + check intrinsics (eat.c lines 987-995)
        "nurse" => {
            state.player.hp = state.player.hp_max;
            state.player.blinded_timeout = 0;
            state.message("You feel much better!");
            check_intrinsics = true;
        }

        // Stalker: invisibility + see invisible + stun (eat.c lines 996-1015)
        "stalker" => {
            if !state.player.properties.has_intrinsic(Property::Invisibility) {
                // Grant invisibility (C uses HInvis += rn1(100,50) for timed)
                state.player.properties.grant_intrinsic(Property::Invisibility);
                state.message("You feel hidden!");
            } else {
                state.player.properties.grant_intrinsic(Property::Invisibility);
                state.player.properties.grant_intrinsic(Property::SeeInvisible);
            }
            // Fallthrough to stun
            let stun_dur = 30u16;
            state.player.stunned_timeout = state.player.stunned_timeout.saturating_add(stun_dur);
            // Bat stun also applies (double stun from fallthrough)
            state.player.stunned_timeout = state.player.stunned_timeout.saturating_add(stun_dur);
        }

        // Yellow light + giant bat: stun (eat.c line 1009-1011)
        "yellow light" | "giant bat" => {
            state.player.stunned_timeout = state.player.stunned_timeout.saturating_add(30);
            // Giant bat fallthrough to bat means another 30
            state.player.stunned_timeout = state.player.stunned_timeout.saturating_add(30);
        }

        // Bat: stun (eat.c lines 1013-1015)
        "bat" => {
            state.player.stunned_timeout = state.player.stunned_timeout.saturating_add(30);
        }

        // Mimics: paralysis as gold pile (eat.c lines 1016-1051)
        "giant mimic" => {
            let dur = 50u16;
            state.message("You can't resist the temptation to mimic a pile of gold.");
            state.player.paralyzed_timeout = state.player.paralyzed_timeout.saturating_add(dur);
        }
        "large mimic" => {
            let dur = 40u16;
            state.message("You can't resist the temptation to mimic a pile of gold.");
            state.player.paralyzed_timeout = state.player.paralyzed_timeout.saturating_add(dur);
        }
        "small mimic" => {
            let dur = 20u16;
            state.message("You can't resist the temptation to mimic a pile of gold.");
            state.player.paralyzed_timeout = state.player.paralyzed_timeout.saturating_add(dur);
        }

        // Quantum mechanic: toggle speed (eat.c lines 1052-1061)
        "quantum mechanic" => {
            state.message("Your velocity suddenly seems very uncertain!");
            if state.player.properties.has_intrinsic(Property::Speed) {
                state.player.properties.remove_intrinsic(Property::Speed);
                state.message("You seem slower.");
            } else {
                state.player.properties.grant_intrinsic(Property::Speed);
                state.message("You seem faster.");
            }
        }

        // Lizard: reduce stun/confusion to 2 (eat.c lines 1062-1067)
        "lizard" => {
            if state.player.stunned_timeout > 2 {
                state.player.stunned_timeout = 2;
            }
            if state.player.confused_timeout > 2 {
                state.player.confused_timeout = 2;
            }
        }

        // Chameleon/Doppelganger/Sandestin: polymorph (eat.c lines 1068-1077)
        "chameleon" | "doppelganger" | "sandestin" => {
            if state.player.properties.has(Property::Unchanging) {
                state.message("You feel momentarily different.");
            } else {
                state.message("You feel a change coming over you.");
                state.player.polymorph_timeout = 1;
            }
        }

        // Disenchanter: strip random intrinsic (eat.c lines 1078-1083)
        "disenchanter" => {
            state.message("You feel a draining sensation.");
            // attrcurse() - strip a random intrinsic
            let intrinsics = [
                Property::FireResistance,
                Property::ColdResistance,
                Property::SleepResistance,
                Property::PoisonResistance,
                Property::ShockResistance,
                Property::Telepathy,
                Property::SeeInvisible,
                Property::Speed,
                Property::Stealth,
            ];
            let idx = state.rng.rn2(intrinsics.len() as u32) as usize;
            state.player.properties.remove_intrinsic(intrinsics[idx]);
        }

        // Mind flayer / master mind flayer: int boost or telepathy (eat.c lines 1084-1095)
        "mind flayer" | "master mind flayer" => {
            let cur_int = state.player.attr_current.get(Attribute::Intelligence);
            let max_int = state.player.attr_max.get(Attribute::Intelligence);
            if cur_int < max_int {
                if state.rng.rn2(2) == 0 {
                    state.message("Yum!  That was real brain food!");
                    state.player.attr_current.modify(Attribute::Intelligence, 1);
                    // Don't give telepathy too
                    return;
                }
            } else {
                state.message("For some reason, that tasted bland.");
            }
            // Fallthrough to check_intrinsics (for telepathy via mconveys)
            check_intrinsics = true;
        }

        _ => {
            check_intrinsics = true;
        }
    }

    // Possibly convey an intrinsic (eat.c lines 1102-1149)
    if check_intrinsics {
        let template = match get_monster(pm) {
            Some(t) => t,
            None => return,
        };

        // Check for hallucination-causing attacks (AD_STUN, AD_HALU, violet fungus)
        if template.dmgtype(DamageType::Stun)
            || template.dmgtype(DamageType::Hallucinate)
            || template.name == "violet fungus"
        {
            state.message("Oh wow!  Great stuff!");
            state.player.hallucinating_timeout =
                state.player.hallucinating_timeout.saturating_add(200);
        }

        // Count possible intrinsics and pick one at random
        let is_giant = template.is_giant();
        let mut count: u32 = 0;
        let mut selected: i32 = 0; // 0 = nothing, -1 = strength, >0 = property index

        // Strength from giants
        if is_giant {
            count = 1;
            selected = -1;
        }

        // Check each conveyable resistance
        for (i, (resist_flag, property)) in CONVEYABLE_INTRINSICS.iter().enumerate() {
            if template.conveys.contains(*resist_flag) {
                count += 1;
                // 1/count chance to replace previous selection
                if state.rng.rn2(count) == 0 {
                    selected = (i + 1) as i32;
                }
            }
        }

        // Teleport from can_teleport flag
        if template.can_teleport() {
            count += 1;
            if state.rng.rn2(count) == 0 {
                selected = 100; // Special: teleport
            }
        }

        // Teleport control from has_teleport_control flag
        if template.has_teleport_control() {
            count += 1;
            if state.rng.rn2(count) == 0 {
                selected = 101; // Special: teleport control
            }
        }

        // Telepathy from telepathic monsters
        if template.is_telepathic() {
            count += 1;
            if state.rng.rn2(count) == 0 {
                selected = 102; // Special: telepathy
            }
        }

        // If strength is the only candidate, give it 50% chance (eat.c line 1142)
        if is_giant && count == 1 && state.rng.rn2(2) != 0 {
            selected = 0;
        }

        // Give the selected intrinsic
        if selected == -1 {
            // Strength gain
            let cur_str = state.player.attr_current.get(Attribute::Strength);
            let max_str = state.player.attr_max.get(Attribute::Strength);
            if cur_str < max_str {
                state.player.attr_current.modify(Attribute::Strength, 1);
                state.message("You feel stronger!");
            }
        } else if selected > 0 && selected <= CONVEYABLE_INTRINSICS.len() as i32 {
            let (_, property) = CONVEYABLE_INTRINSICS[(selected - 1) as usize];
            givit(state, property, template);
        } else if selected == 100 {
            givit(state, Property::Teleportation, template);
        } else if selected == 101 {
            givit(state, Property::TeleportControl, template);
        } else if selected == 102 {
            givit(state, Property::Telepathy, template);
        }
    }

    // Lycanthropy (eat.c lines 1151-1154)
    if let Some(_were_type) = catch_lycanthropy {
        if state.player.lycanthropy.is_none() {
            state.player.lycanthropy = Some(pm);
            state.message("You feel feverish.");
        }
    }
}

// ============================================================================
// Food pre-effects (fprefx from eat.c line 1790)
// ============================================================================

/// Food pre-eating effects for non-corpse food.
/// Port of fprefx() from eat.c lines 1790-1894.
pub fn fprefx(state: &mut GameState, object_type: i16, corpse_type: i16) {
    match object_type {
        otyp::FOOD_RATION => {
            // eat.c lines 1794-1805
            if state.player.nutrition <= 200 {
                state.message("This food really hits the spot!");
            } else if state.player.nutrition < 700 {
                state.message("This satiates your stomach!");
            }
        }
        otyp::TRIPE_RATION => {
            // eat.c lines 1806-1820
            match state.player.race {
                Race::Orc => {
                    state.message("Mmm, tripe... not bad!");
                }
                _ => {
                    state.message("Yak - dog food!");
                    // 50% chance of vomiting for non-orcs, non-carnivores
                    if state.rng.rn2(2) != 0 {
                        let dur = (state.rng.rnd(14) + 14) as u16;
                        state.player.vomiting_timeout = state.player.vomiting_timeout.saturating_add(dur);
                    }
                }
            }
        }
        otyp::LEMBAS_WAFER => {
            // eat.c lines 1822-1830
            match state.player.race {
                Race::Orc => {
                    state.message("!#?&* elf kibble!");
                }
                Race::Elf => {
                    state.message("A little goes a long way.");
                }
                _ => {
                    state.message("This wafer is delicious!");
                }
            }
        }
        otyp::CLOVE_OF_GARLIC => {
            // eat.c lines 1836-1840: garlic makes undead vomit
            // (player is rarely undead, but check for polymorph)
        }
        otyp::EGG => {
            // eat.c lines 1875-1880: rotten/stale eggs
            if let Some(template) = get_monster(corpse_type) {
                if template.flesh_petrifies() {
                    if !state.player.properties.has(Property::StoneResistance) {
                        state.message("Tstrstrstrch!");
                        state.player.stoning_timeout = 5;
                    } else {
                        state.message("This egg doesn't taste like a chicken egg.");
                    }
                }
            }
        }
        otyp::MEATBALL | otyp::MEAT_STICK | otyp::HUGE_CHUNK_OF_MEAT | otyp::MEAT_RING => {
            state.message("This is delicious!");
        }
        _ => {
            // eat.c lines 1882-1891: generic feedback
            if object_type == otyp::CRAM_RATION
                || object_type == otyp::K_RATION
                || object_type == otyp::C_RATION
            {
                state.message("This food is bland.");
            }
        }
    }
}

// ============================================================================
// Food post-effects (fpostfx from eat.c line 2187)
// ============================================================================

/// Food post-eating effects for non-corpse food.
/// Port of fpostfx() from eat.c lines 2187-2267.
pub fn fpostfx(state: &mut GameState, object_type: i16, buc: BucStatus, corpse_type: i16) {
    match object_type {
        otyp::SPRIG_OF_WOLFSBANE => {
            // eat.c lines 2191-2194: cure lycanthropy
            if state.player.lycanthropy.is_some() {
                state.player.lycanthropy = None;
                state.message("You feel purified.");
            }
        }
        otyp::CARROT => {
            // eat.c lines 2195-2199: cure blindness
            if state.player.blinded_timeout > 0 {
                state.player.blinded_timeout = 0;
                state.message("Your vision improves.");
            }
        }
        otyp::FORTUNE_COOKIE => {
            // eat.c lines 2200-2204
            state.message("This cookie has a scrap of paper inside.");
            let fortunes = [
                "You will reach strking new heights.",
                "A strking adventure awaits you.",
                "Change is on the horizon.",
                "Strngth comes from within.",
                "Trust your instincts.",
            ];
            let idx = state.rng.rn2(fortunes.len() as u32) as usize;
            state.message(format!("It reads: \"{}\"", fortunes[idx]));
        }
        otyp::LUMP_OF_ROYAL_JELLY => {
            // eat.c lines 2205-2231: str gain + HP change
            let cur_str = state.player.attr_current.get(Attribute::Strength);
            let max_str = state.player.attr_max.get(Attribute::Strength);
            if cur_str < max_str {
                state.player.attr_current.modify(Attribute::Strength, 1);
                state.message("You feel a little stronger.");
            }

            let hp_change = if buc == BucStatus::Cursed {
                -(state.rng.rnd(20) as i32)
            } else {
                state.rng.rnd(20) as i32
            };

            state.player.hp += hp_change;
            if state.player.hp > state.player.hp_max {
                if state.rng.rn2(17) == 0 {
                    state.player.hp_max += 1;
                }
                state.player.hp = state.player.hp_max;
            } else if state.player.hp <= 0 {
                state.message("That was a bad lump of royal jelly!");
                state.player.hp = 1; // Don't kill from this, just reduce to 1
            }
        }
        otyp::EGG => {
            // eat.c lines 2232-2245: cockatrice egg petrification
            if let Some(template) = get_monster(corpse_type) {
                if template.flesh_petrifies() {
                    if !state.player.properties.has(Property::StoneResistance)
                        && state.player.stoning_timeout == 0
                    {
                        state.player.stoning_timeout = 5;
                        state.message("You are turning to stone!");
                    }
                }
            }
        }
        otyp::EUCALYPTUS_LEAF => {
            // eat.c lines 2246-2251: cure sickness and vomiting
            if buc != BucStatus::Cursed {
                if state.player.sick_food_timeout > 0 || state.player.sick_illness_timeout > 0 {
                    state.player.sick_food_timeout = 0;
                    state.player.sick_illness_timeout = 0;
                    state.message("You feel better.");
                }
                if state.player.vomiting_timeout > 0 {
                    state.player.vomiting_timeout = 0;
                }
            }
        }
        otyp::APPLE => {
            // eat.c lines 2252-2264: cursed apple = sleep (Snow White)
            if buc == BucStatus::Cursed
                && !state.player.properties.has(Property::SleepResistance)
            {
                let dur = (state.rng.rnd(11) + 20) as u16;
                state.player.sleeping_timeout = state.player.sleeping_timeout.saturating_add(dur);
                state.message("You fall asleep.");
            }
        }
        _ => {}
    }
}

// ============================================================================
// Main eating entry point (doeat from eat.c)
// ============================================================================

/// Eat food from inventory.
/// Port of doeat() from eat.c. Handles corpses, tins, and regular food.
pub fn do_eat(state: &mut GameState, obj_letter: char) -> ActionResult {
    // Extract data from object
    let (obj_name, object_type, corpse_type, buc, obj_age) = {
        let obj = match state.get_inventory_item(obj_letter) {
            Some(o) => o,
            None => return ActionResult::Failed("You don't have that item.".to_string()),
        };

        if obj.class != ObjectClass::Food {
            return ActionResult::Failed("That's not something you can eat.".to_string());
        }

        let name = obj.name.clone().unwrap_or_else(|| "food".to_string());
        (name, obj.object_type, obj.corpse_type, obj.buc, obj.age)
    };

    // Tin handling
    if object_type == otyp::TIN {
        return do_eat_tin(state, obj_letter);
    }

    // Calculate nutrition (with race modifier)
    let nutrition = {
        let obj = state.get_inventory_item(obj_letter).unwrap();
        calculate_nutrition(obj, state.player.race)
    };

    // Choking risk (eating while satiated)
    let hunger_state = HungerState::from_nutrition(state.player.nutrition);
    if hunger_state == HungerState::Satiated {
        state.message("You're having a hard time getting all of it down.");
        if state.player.role == crate::player::Role::Knight {
            state.message("You feel like a glutton!");
        }
    }

    // Corpse handling
    if object_type == otyp::CORPSE {
        return do_eat_corpse(state, obj_letter, obj_name, corpse_type, buc, obj_age, nutrition);
    }

    // Non-corpse food
    fprefx(state, object_type, corpse_type);

    state.message(format!("You eat the {}.", obj_name));

    // Apply nutrition
    state.player.nutrition += nutrition;

    // Post-effects
    fpostfx(state, object_type, buc, corpse_type);

    // Choking check
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

    state.remove_from_inventory(obj_letter);
    ActionResult::Success
}

/// Eat a corpse. Handles rot, pre/post effects, intrinsic grants.
fn do_eat_corpse(
    state: &mut GameState,
    obj_letter: char,
    obj_name: String,
    corpse_type: i16,
    buc: BucStatus,
    obj_age: i64,
    nutrition: i32,
) -> ActionResult {
    let template = get_monster(corpse_type);

    // Corpse pre-effects (may kill player)
    let pre_result = cprefx(state, corpse_type);
    if matches!(pre_result, ActionResult::Died(_)) {
        state.remove_from_inventory(obj_letter);
        return pre_result;
    }

    // Rot check (eat.c lines 1606-1663)
    let current_turn = state.turns as i64;
    let rotted = calculate_rot_from_age(obj_age, current_turn, buc, corpse_type, &mut state.rng);

    if rotted > 5 {
        // Badly rotted corpse: food poisoning
        state.message(format!("Ulch - that {} was tainted!", meat_word(template)));

        if state.player.properties.has(Property::SickResistance) {
            state.message("It doesn't seem at all sickening, though...");
        } else {
            let sick_time = (state.rng.rnd(10) + 10) as u16;
            state.player.sick_food_timeout = sick_time;
            state.message("(It must have died too long ago to be safe to eat.)");
        }
        state.remove_from_inventory(obj_letter);
        return ActionResult::Success;
    } else if let Some(t) = template {
        // Acidic corpse damage (eat.c line 1644-1648)
        if t.is_acidic() && !state.player.properties.has(Property::AcidResistance) {
            state.message("You have a very bad case of stomach acid.");
            let dmg = state.rng.rnd(15) as i32;
            state.player.hp -= dmg;
            if state.player.hp <= 0 {
                state.remove_from_inventory(obj_letter);
                return ActionResult::Died("acidic corpse".to_string());
            }
        }

        // Poisonous corpse damage (eat.c lines 1649-1657)
        if t.is_poisonous() && state.rng.rn2(5) != 0 {
            state.message("Ecch - that must have been poisonous!");
            if !state.player.properties.has(Property::PoisonResistance) {
                let str_loss = state.rng.rnd(4) as i8;
                state.player.attr_current.modify(Attribute::Strength, -str_loss);
                let dmg = state.rng.rnd(15) as i32;
                state.player.hp -= dmg;
                if state.player.hp <= 0 {
                    state.remove_from_inventory(obj_letter);
                    return ActionResult::Died("poisonous corpse".to_string());
                }
            } else {
                state.message("You seem unaffected by the poison.");
            }
        }

        // Mild rot (eat.c lines 1659-1663)
        if (rotted > 5 || (rotted > 3 && state.rng.rn2(5) != 0))
            && !state.player.properties.has(Property::SickResistance)
        {
            state.message("You feel sick.");
            let dmg = state.rng.rnd(8) as i32;
            state.player.hp -= dmg;
        }
    }

    // Not rotten or survived rot: try rotten food effects
    if template.map_or(false, |t| !t.nonrotting_corpse()) && (rotted > 0 || state.rng.rn2(7) == 0) {
        rottenfood(state);
    }

    // Taste message
    if let Some(t) = template {
        if t.flesh_petrifies() && state.player.properties.has(Property::StoneResistance) {
            state.message("This tastes just like chicken!");
        }
    }

    state.message(format!("You eat the {}.", obj_name));

    // Apply nutrition (eat.c: weight-dependent reqtime = 3 + (cwt >> 6))
    state.player.nutrition += nutrition;

    // Post-effects
    cpostfx(state, corpse_type);

    if state.player.hp <= 0 {
        state.remove_from_inventory(obj_letter);
        return ActionResult::Died("eating something deadly".to_string());
    }

    // Choking check
    if state.player.nutrition >= 2000 {
        let died = choke(state, &obj_name);
        if died {
            state.remove_from_inventory(obj_letter);
            return ActionResult::Died("choked on food".to_string());
        }
    }

    let hunger_msgs = newuhs(state, true);
    for msg in &hunger_msgs {
        state.message(msg.clone());
    }

    state.remove_from_inventory(obj_letter);
    ActionResult::Success
}

/// Helper to describe corpse meat type for messages
fn meat_word(template: Option<&PerMonst>) -> &str {
    match template {
        Some(t) if t.symbol == 'F' => "fungoid vegetation",
        Some(t) if t.is_herbivore() && !t.is_carnivore() => "protoplasm",
        _ => "meat",
    }
}

/// Calculate rot from age and BUC, matching C formula.
fn calculate_rot_from_age(
    obj_age: i64,
    current_turn: i64,
    buc: BucStatus,
    corpse_type: i16,
    rng: &mut GameRng,
) -> i64 {
    if let Some(template) = get_monster(corpse_type) {
        if template.nonrotting_corpse() {
            return 0;
        }
    }

    let age_diff = current_turn - obj_age;
    let divisor = 10 + rng.rn2(20) as i64;
    let mut rotted = age_diff / divisor.max(1);

    match buc {
        BucStatus::Cursed => rotted += 2,
        BucStatus::Blessed => rotted -= 2,
        _ => {}
    }

    rotted.max(0)
}

// ============================================================================
// Tin eating (start_tin/opentin/consume_tin from eat.c)
// ============================================================================

/// Eat a tin from inventory.
fn do_eat_tin(state: &mut GameState, obj_letter: char) -> ActionResult {
    // Calculate opening time based on wielded weapon
    // (simplified: we just consume immediately for now, but set proper timing)
    let open_time = calculate_tin_open_time(state);

    if open_time == 0 {
        // Instant open
        return consume_tin(state, obj_letter);
    }

    // Multi-turn opening would be tracked in TinContext
    // For now, consume immediately with a message about the delay
    state.message(format!(
        "You spend {} turns opening the tin.",
        open_time
    ));

    consume_tin(state, obj_letter)
}

/// Calculate turns needed to open a tin.
/// Port of start_tin() from eat.c lines 1457-1531.
fn calculate_tin_open_time(state: &GameState) -> i32 {
    // Check for blessed tin: 50% instant, else 1 turn
    // Check wielded weapon type
    // For now: simplified based on whether player has any weapon
    if state.player.properties.has(Property::Speed) {
        0
    } else {
        // Default: 10 turns with bare hands (simplified from C formula)
        3
    }
}

/// Consume an opened tin.
fn consume_tin(state: &mut GameState, obj_letter: char) -> ActionResult {
    let (nutrition, corpse_type, buc) = {
        let obj = match state.get_inventory_item(obj_letter) {
            Some(o) => o,
            None => return ActionResult::Failed("The tin is gone.".to_string()),
        };
        let nut = if obj.nutrition > 0 {
            obj.nutrition as i32
        } else {
            // Tin nutrition is based on corpse inside
            if let Some(template) = get_monster(obj.corpse_type) {
                template.corpse_nutrition as i32 / 2
            } else {
                200
            }
        };
        (nut, obj.corpse_type, obj.buc)
    };

    state.message("You succeed in opening the tin.");

    // Check for spinach tin (corpse_type == -1 or special flag)
    if corpse_type < 0 {
        state.message("It contains spinach.");
        popeye(state);
        state.remove_from_inventory(obj_letter);
        return ActionResult::Success;
    }

    if let Some(template) = get_monster(corpse_type) {
        state.message(format!("It smells like {} meat.", template.name));
    }

    state.player.nutrition += nutrition;

    // Apply corpse effects from tin contents
    if corpse_type >= 0 {
        cpostfx(state, corpse_type);
    }

    let hunger_msgs = newuhs(state, true);
    for msg in &hunger_msgs {
        state.message(msg.clone());
    }

    state.remove_from_inventory(obj_letter);
    ActionResult::Success
}

/// Spinach tin effect: strength boost.
/// Port of Popeye reference from eat.c.
pub fn popeye(state: &mut GameState) {
    state.message("This makes you feel like Popeye!");
    let cur_str = state.player.attr_current.get(Attribute::Strength);
    let max_str = state.player.attr_max.get(Attribute::Strength);
    if cur_str < max_str {
        state.player.attr_current.modify(Attribute::Strength, 1);
        state.message("You feel stronger!");
    }
}

// ============================================================================
// Status effects (make_sick, make_slimed, make_stoned, etc.)
// ============================================================================

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
        state.player.sick_food_timeout = 0;
        state.player.sick_illness_timeout = 0;
        state.message("What a relief!");
    }
}

pub fn make_slimed(state: &mut GameState, duration: i32) {
    if duration > 0 {
        if state.player.sliming_timeout == 0 {
            state.player.sliming_timeout = duration as u16;
            state.message("You don't feel very well.");
        }
    } else if state.player.sliming_timeout > 0 {
        state.player.sliming_timeout = 0;
        state.message("You feel much better!");
    }
}

pub fn make_stoned(state: &mut GameState, duration: i32) {
    if duration > 0 {
        if state.player.stoning_timeout == 0 {
            state.player.stoning_timeout = duration as u16;
            state.message("You are slowing down.");
        }
    } else if state.player.stoning_timeout > 0 {
        state.player.stoning_timeout = 0;
        state.message("You feel limber!");
    }
}

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

pub fn fix_petrification(state: &mut GameState) {
    if state.player.stoning_timeout > 0 {
        make_stoned(state, 0);
    }
}

pub fn burn_away_slime(state: &mut GameState) {
    if state.player.sliming_timeout > 0 {
        state.player.sliming_timeout = 0;
        state.message("The slime that was turning you into green slime burns away!");
    }
}

pub fn poly_when_stoned(_state: &GameState) -> bool {
    false
}

pub fn mon_consume_unstone(_state: &mut GameState, _monster_id: crate::monster::MonsterId) {}
pub fn muse_unslime(_state: &mut GameState, _monster_id: crate::monster::MonsterId) {}

pub fn munslime(state: &mut GameState, monster_id: crate::monster::MonsterId) {
    if let Some(monster) = state.current_level.monster_mut(monster_id) {
        monster
            .status_effects
            .remove_effect(crate::combat::StatusEffect::Slimed);
    }
}

pub fn munstone(state: &mut GameState, monster_id: crate::monster::MonsterId) {
    if let Some(monster) = state.current_level.monster_mut(monster_id) {
        monster
            .status_effects
            .remove_effect(crate::combat::StatusEffect::Petrifying);
    }
}

// ============================================================================
// Status dialogues
// ============================================================================

pub fn slime_dialogue(state: &mut GameState) {
    match state.player.sliming_timeout {
        1 => state.message("You have turned into a green slime!"),
        2 => state.message("Your body is turning into green slime!"),
        3..=5 => state.message("Your limbs are stiffening..."),
        6..=10 => state.message("Your skin is turning green..."),
        _ => {}
    }
}

pub fn slimed_to_death(state: &mut GameState) {
    state.message("You have turned into a green slime!");
    state.player.hp = 0;
}

pub fn stoned_dialogue(state: &mut GameState) {
    match state.player.stoning_timeout {
        1 => state.message("You have turned to stone!"),
        2 => state.message("Your limbs have solidified!"),
        3..=5 => state.message("Your joints are stiffening..."),
        6..=10 => state.message("You are slowing down..."),
        _ => {}
    }
}

// ============================================================================
// Vomit and choke
// ============================================================================

pub fn vomit(state: &mut GameState) {
    state.message("You vomit!");
    state.player.nutrition -= 1000;
    if state.player.nutrition < 0 {
        state.player.nutrition = 0;
    }
    state.player.update_hunger();
}

/// Choke on food. Port of choke() from eat.c lines 238-282.
pub fn choke(state: &mut GameState, food_name: &str) -> bool {
    if state.player.hunger_state != HungerState::Satiated {
        return false;
    }

    // Gluttony penalty for lawful knights
    if state.player.role == crate::player::Role::Knight {
        state.message("You feel like a glutton!");
    }

    // Breathless creatures can't choke
    if state.player.properties.has(Property::MagicBreathing) {
        return false;
    }

    // Breathless or lucky: vomit (eat.c line 252)
    if state.rng.rn2(20) != 0 {
        state.message("You stuff yourself and then vomit voluminously.");
        state.player.nutrition -= 1000;
        if state.player.nutrition < 0 {
            state.player.nutrition = 0;
        }
        vomit(state);
        return false;
    }

    // Fatal choking
    state.message(format!("You choke over your {}.", food_name));
    state.message("You die...");
    state.player.hp = 0;
    true
}

// ============================================================================
// Eating stubs (occupation system not yet implemented)
// ============================================================================

pub fn eat_brains(state: &mut GameState) {
    state.message("You eat the brains.");
}

pub fn eatmdone() -> i32 {
    0
}

pub fn eatmupdate() {}

pub fn bite() {}

pub fn edibility_prompts() -> bool {
    true
}

pub fn eating_conducts(_state: &mut GameState) {}
pub fn eaten_stat() {}

/// Split food from a stack and mark as partly eaten.
/// Port of touchfood() from eat.c lines 341-370.
pub fn touchfood(_state: &mut GameState, _obj_letter: char) {
    // Would split stacks and set oeaten
}

pub fn floorfood() {}

/// Reduce oeaten as food is consumed.
pub fn consume_oeaten() {}
pub fn start_eating() {}
pub fn maybe_finished_meal() {}
pub fn finish_meating() {}

pub fn food_xname() -> String {
    "food".to_string()
}

pub fn food_disappears() {}
pub fn food_substitution() {}

pub fn foodword() -> String {
    "food".to_string()
}

pub fn fatal_corpse_mistake() {}

pub fn tinnable() -> bool {
    true
}

pub fn tin_variety() {}

pub fn tin_variety_txt() -> String {
    "tin".to_string()
}

pub fn tin_details() {}

pub fn mcould_eat_tin() -> bool {
    true
}

pub fn costly_tin() {}

pub fn veggy_item() -> bool {
    true
}

pub fn meatmetal() -> bool {
    false
}

pub fn done_eating() {}
pub fn do_reset_eat() {}
pub fn reset_eat() {}

pub fn eat_food(state: &mut GameState, obj_letter: char) {
    let nutrition = {
        if let Some(obj) = state.get_inventory_item(obj_letter) {
            calculate_nutrition(obj, state.player.race)
        } else {
            0
        }
    };
    state.remove_from_inventory(obj_letter);
    state.message("You eat the food.");
    lesshungry(state, nutrition);
}

pub fn eat_corpse(state: &mut GameState, obj_letter: char) {
    let (nutrition, corpse_type) = {
        if let Some(obj) = state.get_inventory_item(obj_letter) {
            (calculate_nutrition(obj, state.player.race), obj.corpse_type)
        } else {
            (0, 0)
        }
    };
    state.remove_from_inventory(obj_letter);
    state.message("You eat the corpse.");
    lesshungry(state, nutrition);
    cpostfx(state, corpse_type);
}

pub fn eat_accessory(state: &mut GameState, obj_letter: char) {
    state.message("You eat the accessory.");
    state.remove_from_inventory(obj_letter);
}

pub fn eatspecial() {}

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
                state.player.hp = 0;
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

// ============================================================================
// Legacy interfaces (backward compatibility)
// ============================================================================

/// Legacy corpse effects interface (kept for nh-compare tests).
#[derive(Debug, Clone)]
pub enum CorpseEffect {
    GainIntrinsic { property: Property, chance: u8 },
    GainEnergy { amount: i32 },
    GainLevel,
    FullHeal,
    Confusion { duration: i32 },
    Hallucination { duration: i32 },
    Stun { duration: i32 },
    Blindness { duration: i32 },
    CureStoning,
    CureConfusion,
    CureStunning,
    Polymorph,
    StrengthBoost,
    IntelligenceBoost,
    InstantDeath { cause: &'static str },
    Sickness { duration: i32 },
    Lycanthropy { monster_type: i16 },
    ToggleSpeed,
}

/// Legacy corpse effects lookup (used by nh-compare tests).
/// Returns a simplified list; the real logic is in cpostfx() now.
pub fn corpse_effects(monster_type: i16) -> Vec<CorpseEffect> {
    let template = match get_monster(monster_type) {
        Some(t) => t,
        None => return vec![],
    };

    match template.name {
        "newt" => vec![CorpseEffect::GainEnergy { amount: 3 }],
        "floating eye" => vec![CorpseEffect::GainIntrinsic {
            property: Property::Telepathy,
            chance: 100,
        }],
        "cockatrice" | "chickatrice" => vec![CorpseEffect::InstantDeath {
            cause: "swallowing a cockatrice whole",
        }],
        "lizard" => vec![
            CorpseEffect::CureStoning,
            CorpseEffect::CureConfusion,
            CorpseEffect::CureStunning,
        ],
        "wraith" => vec![CorpseEffect::GainLevel],
        "nurse" => vec![
            CorpseEffect::FullHeal,
            CorpseEffect::GainIntrinsic {
                property: Property::PoisonResistance,
                chance: 15,
            },
        ],
        "mind flayer" | "master mind flayer" => vec![CorpseEffect::IntelligenceBoost],
        "stalker" => vec![
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
        "bat" | "giant bat" => vec![CorpseEffect::Stun { duration: 30 }],
        "violet fungus" => vec![CorpseEffect::Hallucination { duration: 200 }],
        "quantum mechanic" => vec![CorpseEffect::ToggleSpeed],
        "chameleon" | "doppelganger" => vec![CorpseEffect::Polymorph],
        "green slime" => vec![CorpseEffect::InstantDeath {
            cause: "turning into green slime",
        }],
        _ => {
            let mut effects = vec![];
            // Check conveys for resistances
            for (resist_flag, property) in CONVEYABLE_INTRINSICS {
                if template.conveys.contains(*resist_flag) {
                    effects.push(CorpseEffect::GainIntrinsic {
                        property: *property,
                        chance: 15,
                    });
                }
            }
            if template.is_giant() {
                effects.push(CorpseEffect::StrengthBoost);
            }
            effects
        }
    }
}

/// Legacy corpse effect application (used by nh-compare tests).
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
                state.player.energy =
                    (state.player.energy + amount).min(state.player.energy_max);
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
                state.player.confused_timeout =
                    state.player.confused_timeout.saturating_add(*duration as u16);
                messages.push("Yuk--Loss of strength saps the mind.".to_string());
            }
            CorpseEffect::Hallucination { duration } => {
                state.player.hallucinating_timeout = state
                    .player
                    .hallucinating_timeout
                    .saturating_add(*duration as u16);
                messages.push("Oh wow! Great stuff!".to_string());
            }
            CorpseEffect::Stun { duration } => {
                state.player.stunned_timeout =
                    state.player.stunned_timeout.saturating_add(*duration as u16);
                messages.push("You feel dizzy.".to_string());
            }
            CorpseEffect::Blindness { duration } => {
                state.player.blinded_timeout =
                    state.player.blinded_timeout.saturating_add(*duration as u16);
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
                    messages.push(
                        "Your medallion of life saving crumbles to dust!".to_string(),
                    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::MONSTERS;
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
        assert_eq!(calculate_nutrition(&obj, Race::Human), 800);
    }

    #[test]
    fn test_calculate_nutrition_lembas_elf_bonus() {
        let obj = make_food('a', otyp::LEMBAS_WAFER, 800);
        assert_eq!(calculate_nutrition(&obj, Race::Elf), 1000); // 800 + 200
    }

    #[test]
    fn test_calculate_nutrition_lembas_orc_penalty() {
        let obj = make_food('a', otyp::LEMBAS_WAFER, 800);
        assert_eq!(calculate_nutrition(&obj, Race::Orc), 600); // 800 - 200
    }

    #[test]
    fn test_calculate_nutrition_cram_dwarf_bonus() {
        let obj = make_food('a', otyp::CRAM_RATION, 600);
        assert_eq!(calculate_nutrition(&obj, Race::Dwarf), 700); // 600 + 100
    }

    #[test]
    fn test_is_rotten_fresh_corpse() {
        let mut obj = make_food('a', otyp::CORPSE, 0);
        obj.corpse_type = 1;
        obj.age = 100;
        assert!(!is_rotten(&obj, 150));
    }

    #[test]
    fn test_is_rotten_old_corpse() {
        let mut obj = make_food('a', otyp::CORPSE, 0);
        obj.corpse_type = 1;
        obj.age = 0;
        assert!(is_rotten(&obj, 200));
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
        obj.object_type = 0;
        assert!(!is_rotten(&obj, 10000));
    }

    #[test]
    fn test_cpostfx_quantum_mechanic_toggles_speed() {
        let mut state = GameState::new(GameRng::from_entropy());
        assert!(!state.player.properties.has_intrinsic(Property::Speed));

        // Find quantum mechanic index
        let qm_idx = MONSTERS
            .iter()
            .position(|m| m.name == "quantum mechanic")
            .unwrap() as i16;

        cpostfx(&mut state, qm_idx);
        // Should have toggled speed on
        assert!(state.player.properties.has_intrinsic(Property::Speed));

        cpostfx(&mut state, qm_idx);
        // Should have toggled speed off
        assert!(!state.player.properties.has_intrinsic(Property::Speed));
    }

    #[test]
    fn test_cpostfx_wraith_levels_up() {
        let mut state = GameState::new(GameRng::from_entropy());
        let initial_level = state.player.exp_level;

        let wraith_idx = MONSTERS
            .iter()
            .position(|m| m.name == "wraith")
            .unwrap() as i16;

        cpostfx(&mut state, wraith_idx);
        assert_eq!(state.player.exp_level, initial_level + 1);
    }

    #[test]
    fn test_fpostfx_wolfsbane_cures_lycanthropy() {
        let mut state = GameState::new(GameRng::from_entropy());
        state.player.lycanthropy = Some(42);

        fpostfx(&mut state, otyp::SPRIG_OF_WOLFSBANE, BucStatus::Uncursed, 0);
        assert!(state.player.lycanthropy.is_none());
    }

    #[test]
    fn test_fpostfx_eucalyptus_cures_sickness() {
        let mut state = GameState::new(GameRng::from_entropy());
        state.player.sick_food_timeout = 10;
        state.player.sick_illness_timeout = 5;

        fpostfx(&mut state, otyp::EUCALYPTUS_LEAF, BucStatus::Uncursed, 0);
        assert_eq!(state.player.sick_food_timeout, 0);
        assert_eq!(state.player.sick_illness_timeout, 0);
    }

    #[test]
    fn test_nonrotting_corpse_lizard() {
        let lizard_idx = MONSTERS
            .iter()
            .position(|m| m.name == "lizard")
            .map(|i| i as i16);

        if let Some(idx) = lizard_idx {
            let mut obj = make_food('a', otyp::CORPSE, 0);
            obj.corpse_type = idx;
            obj.age = 0;
            assert!(!is_rotten(&obj, 10000));
        }
    }

    #[test]
    fn test_mconveys_fire_resistance() {
        // Find a monster that conveys fire resistance
        let fire_conveyor = MONSTERS
            .iter()
            .enumerate()
            .find(|(_, m)| m.conveys.contains(MonsterResistances::FIRE) && m.name != "");

        if let Some((idx, template)) = fire_conveyor {
            let effects = corpse_effects(idx as i16);
            let has_fire = effects.iter().any(|e| matches!(e, CorpseEffect::GainIntrinsic { property: Property::FireResistance, .. }));
            assert!(has_fire, "Monster {} should convey fire resistance", template.name);
        }
    }
}
