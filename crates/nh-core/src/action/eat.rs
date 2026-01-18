//! Eating food and corpses (eat.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::object::{BucStatus, Object, ObjectClass};
use crate::player::{Attribute, HungerState, Property};
use crate::rng::GameRng;

// ============================================================================
// Intrinsic gain messages
// ============================================================================

/// Get the message when gaining an intrinsic property from food
pub fn intrinsic_gain_message(prop: Property) -> &'static str {
    match prop {
        Property::FireResistance => "You feel a momentary chill.",
        Property::ColdResistance => "You feel full of hot air.",
        Property::SleepResistance => "You feel wide awake.",
        Property::DisintResistance => "You feel very firm.",
        Property::ShockResistance => "Your health currently feels amplified!",
        Property::PoisonResistance => "You feel healthy.",
        Property::Telepathy => "You feel a strange mental acuity.",
        Property::Teleportation => "You feel very jumpy.",
        Property::TeleportControl => "You feel in control of yourself.",
        Property::Invisibility => "You feel hidden!",
        Property::SeeInvisible => "You see an image of someone stalking you.",
        Property::Speed => "You feel yourself speed up.",
        Property::VeryFast => "You feel yourself speed up a lot!",
        _ => "You feel different.",
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

/// Get corpse effects for a monster type.
/// Returns a list of effects that may occur when eating this corpse.
///
/// # Arguments
/// * `monster_type` - The corpse's corpse_type field (monster index)
pub fn corpse_effects(monster_type: i16) -> Vec<CorpseEffect> {
    // These monster indices should match nh-data monster definitions
    // For now, use approximate values based on common monster names
    match monster_type {
        // Newt - minor energy boost
        0 => vec![CorpseEffect::GainEnergy { amount: 3 }],

        // Floating eye - telepathy
        38 => vec![CorpseEffect::GainIntrinsic {
            property: Property::Telepathy,
            chance: 100, // Guaranteed from floating eye
        }],

        // Cockatrice - instant death by petrification
        69 => vec![CorpseEffect::InstantDeath {
            cause: "swallowing a cockatrice whole",
        }],

        // Lizard - cure stoning and reduce confusion/stunning
        81 => vec![
            CorpseEffect::CureStoning,
            CorpseEffect::CureConfusion,
            CorpseEffect::CureStunning,
        ],

        // Wraith - gain a level
        86 => vec![CorpseEffect::GainLevel],

        // Nurse - full heal
        110 => vec![
            CorpseEffect::FullHeal,
            CorpseEffect::GainIntrinsic {
                property: Property::PoisonResistance,
                chance: 15,
            },
        ],

        // Mind flayer - intelligence or telepathy
        120 => vec![
            CorpseEffect::IntelligenceBoost,
            CorpseEffect::GainIntrinsic {
                property: Property::Telepathy,
                chance: 50,
            },
        ],

        // Fire elemental / red dragon - fire resistance
        150..=155 => vec![CorpseEffect::GainIntrinsic {
            property: Property::FireResistance,
            chance: 15,
        }],

        // Ice elemental / white dragon - cold resistance
        160..=165 => vec![CorpseEffect::GainIntrinsic {
            property: Property::ColdResistance,
            chance: 15,
        }],

        // Stalker - invisibility
        186 => vec![
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

        // Giant bat / bat - stunning
        200..=201 => vec![CorpseEffect::Stun { duration: 30 }],

        // Violet fungus - hallucination
        220 => vec![CorpseEffect::Hallucination { duration: 200 }],

        // Quantum mechanic - toggle speed
        230 => vec![CorpseEffect::ToggleSpeed],

        // Chameleon / doppelganger - polymorph
        240..=241 => vec![CorpseEffect::Polymorph],

        // Giants - strength
        250..=260 => vec![CorpseEffect::StrengthBoost],

        // Werewolf (human form)
        280 => vec![CorpseEffect::Lycanthropy { monster_type: 281 }],

        // Green slime - turns you into slime (fatal)
        300 => vec![CorpseEffect::InstantDeath {
            cause: "turning into green slime",
        }],

        // Disenchanter - lose a random intrinsic
        310 => vec![], // Special handling needed

        // Default - check for standard resistances based on monster flags
        _ => vec![],
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
                if rng.rn2(100) < *chance as u32 {
                    if !state.player.properties.has_intrinsic(*property) {
                        state.player.properties.grant_intrinsic(*property);
                        messages.push(intrinsic_gain_message(*property).to_string());
                    }
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
                // Stoning would need to be added to player - using StoneResistance as proxy
                if state.player.properties.has(Property::StoneResistance) {
                    // Already resistant, no effect
                } else {
                    // Would cure stoning in progress
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
                // Would need polymorph implementation
                messages.push("You feel a change coming over you.".to_string());
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
                messages.push(format!("You die from {}.", cause));
                state.player.hp = 0;
                // Would trigger death handling
            }

            CorpseEffect::Sickness { duration: _ } => {
                // Would need sickness tracking
                messages.push("You feel deathly sick.".to_string());
            }

            CorpseEffect::Lycanthropy { monster_type: _ } => {
                // Would need lycanthropy implementation
                messages.push("You feel feverish.".to_string());
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

// HungerState is imported from crate::player::HungerState
// Threshold constants are available via HungerState::threshold()

/// Tin preparation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TinType {
    Rotten,
    Homemade,
    Soup,
    FrenchFried,
    Pickled,
    Boiled,
    Smoked,
    Dried,
    DeepFried,
    Szechuan,
    Broiled,
    StirFried,
    Sauteed,
    Candied,
    Pureed,
    Spinach,
}

impl TinType {
    /// Get nutrition modifier for tin type
    pub fn nutrition(&self) -> i32 {
        match self {
            TinType::Rotten => -50,
            TinType::Homemade => 50,
            TinType::Soup => 20,
            TinType::FrenchFried => 40,
            TinType::Pickled => 40,
            TinType::Boiled => 50,
            TinType::Smoked => 50,
            TinType::Dried => 55,
            TinType::DeepFried => 60,
            TinType::Szechuan => 70,
            TinType::Broiled => 80,
            TinType::StirFried => 80,
            TinType::Sauteed => 95,
            TinType::Candied => 100,
            TinType::Pureed => 500,
            TinType::Spinach => 600,
        }
    }

    /// Get description for tin type
    pub fn description(&self) -> &'static str {
        match self {
            TinType::Rotten => "rotten",
            TinType::Homemade => "homemade",
            TinType::Soup => "soup made from",
            TinType::FrenchFried => "french fried",
            TinType::Pickled => "pickled",
            TinType::Boiled => "boiled",
            TinType::Smoked => "smoked",
            TinType::Dried => "dried",
            TinType::DeepFried => "deep fried",
            TinType::Szechuan => "szechuan",
            TinType::Broiled => "broiled",
            TinType::StirFried => "stir fried",
            TinType::Sauteed => "sauteed",
            TinType::Candied => "candied",
            TinType::Pureed => "pureed",
            TinType::Spinach => "spinach",
        }
    }
}

/// Base nutrition values for common foods
pub fn base_nutrition(object_type: i16) -> i32 {
    // These should match nh-data object definitions
    // For now, provide reasonable defaults
    match object_type {
        // Corpses vary by monster
        _ if object_type < 100 => 100, // Generic food
        _ => 50,
    }
}

/// Check if food is edible
pub fn is_edible(obj: &Object) -> bool {
    obj.class == ObjectClass::Food
}

/// Check if food is rotten (based on age and type)
pub fn is_rotten(obj: &Object, current_turn: i64) -> bool {
    // Corpses rot after ~250 turns
    // Blessed food lasts longer, cursed food rots faster
    let age = current_turn - obj.age;
    let rot_time = match obj.buc {
        BucStatus::Blessed => 350,
        BucStatus::Uncursed => 250,
        BucStatus::Cursed => 150,
    };
    age > rot_time
}

/// Calculate nutrition from eating an object
pub fn calculate_nutrition(obj: &Object) -> i32 {
    let base = base_nutrition(obj.object_type);

    // Blessed food gives more nutrition
    let buc_modifier = match obj.buc {
        BucStatus::Blessed => 1.5,
        BucStatus::Uncursed => 1.0,
        BucStatus::Cursed => 0.75,
    };

    (base as f32 * buc_modifier) as i32
}

/// Eat food from inventory
pub fn do_eat(state: &mut GameState, obj_letter: char) -> ActionResult {
    // First pass: validation
    let (obj_name, is_cursed, nutrition) = {
        let obj = match state.get_inventory_item(obj_letter) {
            Some(o) => o,
            None => return ActionResult::Failed("You don't have that item.".to_string()),
        };

        if obj.class != ObjectClass::Food {
            return ActionResult::Failed("That's not something you can eat.".to_string());
        }

        let name = obj.name.clone().unwrap_or_else(|| "food".to_string());
        let nutrition = calculate_nutrition(obj);

        (name, obj.is_cursed(), nutrition)
    };

    // Check for choking (eating while satiated)
    let hunger_state = HungerState::from_nutrition(state.player.nutrition);
    if hunger_state == HungerState::Satiated {
        state.message("You're having a hard time getting all of it down.");
        // Could potentially choke - for now just warn
    }

    // Eating message
    state.message(format!("You eat the {}.", obj_name));

    // Apply nutrition
    state.player.nutrition += nutrition;

    // Cursed food might cause problems
    if is_cursed {
        state.message("Ulch - that food was tainted!");
        // Could cause sickness, vomiting, etc.
        state.player.nutrition -= nutrition / 2; // Lose some nutrition
    }

    // Update hunger state
    state.player.update_hunger();

    // Check new hunger state and give feedback
    let new_state = HungerState::from_nutrition(state.player.nutrition);
    match new_state {
        HungerState::Satiated => {
            state.message("You're having a hard time getting all of it down.");
        }
        HungerState::NotHungry => {
            // No message needed
        }
        _ => {
            // Still hungry
        }
    }

    // Remove the food item
    state.remove_from_inventory(obj_letter);

    ActionResult::Success
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

/// Choke on food (potentially fatal)
pub fn choke(state: &mut GameState, food_name: &str) {
    state.message(format!("You choke on the {}!", food_name));
    // In full implementation, this could be fatal
    // For now, just cause vomiting
    vomit(state);
}

// ============================================================================
// Hunger state management (newuhs, gethungry, lesshungry from NetHack)
// ============================================================================

/// Update hunger status with messages when state changes.
/// This is the Rust equivalent of NetHack's newuhs() function.
///
/// Called after nutrition changes to determine if the hunger state
/// has changed and produce appropriate messages/effects.
///
/// # Arguments
/// * `state` - The game state
/// * `incr` - Whether nutrition increased (true) or decreased (false)
///
/// # Returns
/// Messages about hunger state changes
pub fn newuhs(state: &mut GameState, incr: bool) -> Vec<String> {
    let mut messages = Vec::new();
    let old_state = state.player.hunger_state;
    let new_state = HungerState::from_nutrition(state.player.nutrition);

    // Only update if state actually changed
    if old_state == new_state {
        return messages;
    }

    state.player.hunger_state = new_state;

    // Generate messages for state transitions
    if incr {
        // Getting less hungry (eating)
        match (old_state, new_state) {
            (HungerState::Fainted | HungerState::Fainting, HungerState::Weak | HungerState::Hungry | HungerState::NotHungry | HungerState::Satiated) => {
                messages.push("You regain consciousness.".to_string());
            }
            (HungerState::Weak, HungerState::Hungry | HungerState::NotHungry | HungerState::Satiated) => {
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
        // Getting more hungry
        match new_state {
            HungerState::Hungry => {
                if !matches!(old_state, HungerState::Weak | HungerState::Fainting | HungerState::Fainted) {
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
                    // In NetHack, this can cause the player to faint
                }
            }
            HungerState::Fainted => {
                messages.push("You faint from lack of food.".to_string());
                // Paralyzed for a duration based on level
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
/// This is the Rust equivalent of NetHack's gethungry() function.
///
/// Decrements nutrition based on:
/// - Base metabolism (1 point per move)
/// - Encumbrance (more if heavily burdened)
/// - Ring of Hunger (doubles hunger rate)
/// - Slow Digestion property (reduces hunger rate)
/// - Regeneration property (increases hunger rate)
///
/// # Arguments
/// * `state` - The game state
/// * `rng` - Random number generator
///
/// # Returns
/// Messages about hunger state changes
pub fn gethungry(state: &mut GameState, rng: &mut GameRng) -> Vec<String> {
    // Don't get hungry if already dead
    if state.player.hp <= 0 {
        return Vec::new();
    }

    // Calculate base hunger rate
    let mut hunger_rate: i32 = 1;

    // Ring of Hunger doubles hunger rate
    // Check for Hunger property (which includes ring of hunger)
    if state.player.properties.has(Property::Hunger) {
        hunger_rate += 1;
    }

    // Regeneration increases hunger
    if state.player.properties.has(Property::Regeneration) {
        hunger_rate += 1;
    }

    // Encumbrance affects hunger
    let encumbrance = state.player.encumbrance();
    match encumbrance {
        crate::player::Encumbrance::Unencumbered => {}
        crate::player::Encumbrance::Burdened => {
            // Burdened: 50% chance of extra hunger
            if rng.rn2(2) == 0 {
                hunger_rate += 1;
            }
        }
        crate::player::Encumbrance::Stressed => {
            // Stressed: always extra hunger
            hunger_rate += 1;
        }
        crate::player::Encumbrance::Strained => {
            // Strained: double extra hunger
            hunger_rate += 2;
        }
        crate::player::Encumbrance::Overtaxed => {
            // Overtaxed: triple extra hunger
            hunger_rate += 3;
        }
        crate::player::Encumbrance::Overloaded => {
            // Overloaded: massive hunger
            hunger_rate += 4;
        }
    }

    // Slow Digestion negates all hunger
    if state.player.properties.has(Property::SlowDigestion) {
        hunger_rate = 0;
    }

    // Apply hunger
    if hunger_rate > 0 {
        state.player.nutrition = state.player.nutrition.saturating_sub(hunger_rate);
    }

    // Update hunger state and get messages
    newuhs(state, false)
}

/// Add nutrition from eating food.
/// This is the Rust equivalent of NetHack's lesshungry() function.
///
/// # Arguments
/// * `state` - The game state
/// * `nutrition` - Amount of nutrition to add
///
/// # Returns
/// Messages about hunger state changes
pub fn lesshungry(state: &mut GameState, nutrition: i32) -> Vec<String> {
    // Add nutrition
    state.player.nutrition = state.player.nutrition.saturating_add(nutrition);

    // Cap nutrition at a reasonable maximum (prevents overflow issues)
    const MAX_NUTRITION: i32 = 5000;
    if state.player.nutrition > MAX_NUTRITION {
        state.player.nutrition = MAX_NUTRITION;
    }

    // Update hunger state and get messages
    newuhs(state, true)
}

/// Calculate hunger timeout for weak/fainting states.
/// Used for determining when the player might faint from hunger.
///
/// # Arguments
/// * `state` - The game state
/// * `rng` - Random number generator
///
/// # Returns
/// Number of turns before potential fainting (0 means no fainting risk)
pub fn hunger_timeout(state: &GameState, rng: &mut GameRng) -> i32 {
    match state.player.hunger_state {
        HungerState::Weak => {
            // Random chance to faint when weak
            if rng.rn2(20) < 3 {
                rng.rnd(10) as i32
            } else {
                0
            }
        }
        HungerState::Fainting => {
            // High chance to faint when fainting
            if rng.rn2(10) < 4 {
                rng.rnd(5) as i32
            } else {
                0
            }
        }
        _ => 0,
    }
}

/// Check if the player should faint from hunger this turn.
/// Called during game tick to potentially cause fainting.
///
/// # Arguments
/// * `state` - The game state
/// * `rng` - Random number generator
///
/// # Returns
/// True if the player faints, false otherwise
pub fn check_faint_from_hunger(state: &mut GameState, rng: &mut GameRng) -> bool {
    let timeout = hunger_timeout(state, rng);
    if timeout > 0 && state.player.paralyzed_timeout == 0 {
        state.message("You faint from lack of food.");
        state.player.paralyzed_timeout = timeout as u16;
        state.player.hunger_state = HungerState::Fainted;
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{Object, ObjectClass, ObjectId};
    use crate::rng::GameRng;

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
    fn test_eat_missing_item_fails() {
        let mut state = GameState::new(GameRng::from_entropy());
        let result = do_eat(&mut state, 'z');
        assert!(matches!(result, ActionResult::Failed(_)));
    }

    #[test]
    fn test_eat_food_increases_nutrition() {
        let mut state = GameState::new(GameRng::from_entropy());
        let initial_nutrition = state.player.nutrition;
        
        let mut obj = Object::default();
        obj.id = ObjectId(1);
        obj.class = ObjectClass::Food;
        obj.inv_letter = 'a';
        state.inventory.push(obj);

        let result = do_eat(&mut state, 'a');
        assert!(matches!(result, ActionResult::Success));
        assert!(state.player.nutrition > initial_nutrition);
    }
}
