//! Potion effects (potion.c)
//!
//! Handles drinking potions and their effects.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::monster::Monster;
use crate::monster::MonsterResistances;
use crate::object::{BucStatus, Object};
use crate::player::Attribute;
use crate::player::{Property, You};
use crate::rng::GameRng;

// ============================================================================
// Glow Effects (artifact.c + read.c)
// ============================================================================

/// Get the strength level of a glow (0-3) based on count
/// Used for artifact warning intensity (Sting, Orcrist, Grimtooth)
pub fn glow_strength(count: i32) -> usize {
    // glow strength proportional to nearby orc count
    if count > 12 {
        3
    } else if count > 4 {
        2
    } else if count > 0 {
        1
    } else {
        0
    }
}

/// Get the verb for glow effect
/// Intensity 0: "quiver" (for blind)
/// Intensity 1: "flicker"
/// Intensity 2: "glimmer"
/// Intensity 3: "gleam"
pub fn glow_verb(count: i32, with_ing_suffix: bool) -> String {
    let verbs = ["quiver", "flicker", "glimmer", "gleam"];
    let strength = glow_strength(count);
    let verb = verbs[strength];

    if with_ing_suffix {
        format!("{}ing", verb)
    } else {
        verb.to_string()
    }
}

/// Get glow color name (e.g., for artifact)
/// In original: uses artifact color from artilist array
pub fn glow_color(artifact_index: usize) -> &'static str {
    // Simplified: hardcoded common artifact colors
    match artifact_index {
        0..=5 => "blue",     // Most artifacts
        6..=10 => "white",   // Holy artifacts
        11..=15 => "yellow", // Gold/treasure
        _ => "blue",         // Default
    }
}

/// Print glow message (simple glow without color)
/// Used when blinded or in simple situations
pub fn p_glow1(object_name: &str) -> String {
    format!("{} briefly.", object_name)
}

/// Print glow message with color
/// Used when player can see the color
pub fn p_glow2(object_name: &str, color: &str) -> String {
    format!("{} {} for a moment.", object_name, color)
}

// ============================================================================
// Potion Result Structures
// ============================================================================

/// Result of drinking/using a potion
#[derive(Debug, Clone)]
pub struct PotionResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Whether the potion was consumed
    pub consumed: bool,
    /// Whether player died
    pub player_died: bool,
    /// Whether to identify the potion type
    pub identify: bool,
    /// Change dungeon level (positive = up, negative = down, 0 = no change)
    pub level_change: i32,
}

impl PotionResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            consumed: true, // Most potions are consumed
            player_died: false,
            identify: true, // Most potions identify on use
            level_change: 0,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

impl Default for PotionResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Potion type indices (matching ObjectType in nh-data/objects.rs)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i16)]
pub enum PotionType {
    GainAbility = 257,
    Restore = 258,
    Confusion = 259,
    Blindness = 260,
    Paralysis = 261,
    Speed = 262,
    Levitation = 263,
    Hallucination = 264,
    Invisibility = 265,
    SeeInvisible = 266,
    Healing = 267,
    ExtraHealing = 268,
    GainLevel = 269,
    Enlightenment = 270,
    MonsterDetection = 271,
    ObjectDetection = 272,
    GainEnergy = 273,
    Sleeping = 274,
    FullHealing = 275,
    Polymorph = 276,
    Booze = 277,
    Sickness = 278,
    FruitJuice = 279,
    Acid = 280,
    Oil = 281,
    Water = 282,
    Poison = 283, // Added for thrown potion handling
}

impl PotionType {
    /// Try to convert an object type to a potion type
    pub fn from_object_type(otype: i16) -> Option<Self> {
        match otype {
            257 => Some(PotionType::GainAbility),
            258 => Some(PotionType::Restore),
            259 => Some(PotionType::Confusion),
            260 => Some(PotionType::Blindness),
            261 => Some(PotionType::Paralysis),
            262 => Some(PotionType::Speed),
            263 => Some(PotionType::Levitation),
            264 => Some(PotionType::Hallucination),
            265 => Some(PotionType::Invisibility),
            266 => Some(PotionType::SeeInvisible),
            267 => Some(PotionType::Healing),
            268 => Some(PotionType::ExtraHealing),
            269 => Some(PotionType::GainLevel),
            270 => Some(PotionType::Enlightenment),
            271 => Some(PotionType::MonsterDetection),
            272 => Some(PotionType::ObjectDetection),
            273 => Some(PotionType::GainEnergy),
            274 => Some(PotionType::Sleeping),
            275 => Some(PotionType::FullHealing),
            276 => Some(PotionType::Polymorph),
            277 => Some(PotionType::Booze),
            278 => Some(PotionType::Sickness),
            279 => Some(PotionType::FruitJuice),
            280 => Some(PotionType::Acid),
            281 => Some(PotionType::Oil),
            282 => Some(PotionType::Water),
            283 => Some(PotionType::Poison),
            _ => None,
        }
    }
}

/// Quaff (drink) a potion
pub fn quaff_potion(potion: &Object, player: &mut You, rng: &mut GameRng) -> PotionResult {
    let Some(ptype) = PotionType::from_object_type(potion.object_type) else {
        return PotionResult::new().with_message("That's not a potion!");
    };

    let blessed = potion.is_blessed();
    let cursed = potion.is_cursed();

    match ptype {
        PotionType::Healing => potion_healing(player, blessed, cursed, rng),
        PotionType::ExtraHealing => potion_extra_healing(player, blessed, cursed, rng),
        PotionType::FullHealing => potion_full_healing(player, blessed, cursed),
        PotionType::GainAbility => potion_gain_ability(player, blessed, rng),
        PotionType::Restore => potion_restore(player, blessed),
        PotionType::Confusion => potion_confusion(player, blessed, rng),
        PotionType::Blindness => potion_blindness(player, blessed, cursed, rng),
        PotionType::Paralysis => potion_paralysis(player, blessed, cursed, rng),
        PotionType::Speed => potion_speed(player, blessed, rng),
        PotionType::Levitation => potion_levitation(player, blessed, cursed, rng),
        PotionType::Hallucination => potion_hallucination(player, cursed, rng),
        PotionType::Invisibility => potion_invisibility(player, blessed, rng),
        PotionType::SeeInvisible => potion_see_invisible(player, blessed),
        PotionType::GainLevel => potion_gain_level(player, cursed),
        PotionType::Enlightenment => potion_enlightenment(player),
        PotionType::MonsterDetection => potion_monster_detection(player),
        PotionType::ObjectDetection => potion_object_detection(player),
        PotionType::GainEnergy => potion_gain_energy(player, blessed, rng),
        PotionType::Sleeping => potion_sleeping(player, rng),
        PotionType::Polymorph => potion_polymorph(player, rng),
        PotionType::Booze => potion_booze(player, rng),
        PotionType::Sickness => potion_sickness(player, blessed),
        PotionType::FruitJuice => potion_fruit_juice(player),
        PotionType::Acid => potion_acid(player, rng),
        PotionType::Oil => potion_oil(player),
        PotionType::Water => potion_water(player, &potion.buc),
        PotionType::Poison => potion_poison(player, blessed, rng),
    }
}

fn potion_healing(
    player: &mut You,
    blessed: bool,
    cursed: bool,
    rng: &mut GameRng,
) -> PotionResult {
    let mut result = PotionResult::new();

    let heal_amount = if blessed {
        rng.dice(8, 4) as i32 + 8
    } else if cursed {
        rng.dice(4, 4) as i32
    } else {
        rng.dice(6, 4) as i32
    };

    player.hp = (player.hp + heal_amount).min(player.hp_max);
    result
        .messages
        .push(format!("You feel better. (+{} HP)", heal_amount));

    // Cure blindness
    if player.blinded_timeout > 0 && !cursed {
        player.blinded_timeout = 0;
        result.messages.push("Your vision clears.".to_string());
    }

    result
}

fn potion_extra_healing(
    player: &mut You,
    blessed: bool,
    cursed: bool,
    rng: &mut GameRng,
) -> PotionResult {
    let mut result = PotionResult::new();

    let heal_amount = if blessed {
        rng.dice(8, 8) as i32 + 16
    } else if cursed {
        rng.dice(4, 8) as i32
    } else {
        rng.dice(6, 8) as i32
    };

    player.hp = (player.hp + heal_amount).min(player.hp_max);
    result
        .messages
        .push(format!("You feel much better. (+{} HP)", heal_amount));

    // Cure blindness and confusion
    if player.blinded_timeout > 0 && !cursed {
        player.blinded_timeout = 0;
        result.messages.push("Your vision clears.".to_string());
    }
    if player.confused_timeout > 0 && !cursed {
        player.confused_timeout = 0;
        result.messages.push("Your confusion clears.".to_string());
    }

    // Blessed can increase max HP
    if blessed && player.hp_max < 500 {
        player.hp_max += 1;
        player.hp += 1;
    }

    result
}

fn potion_full_healing(player: &mut You, blessed: bool, cursed: bool) -> PotionResult {
    let mut result = PotionResult::new();

    if cursed {
        player.hp = (player.hp + player.hp_max / 2).min(player.hp_max);
        result
            .messages
            .push("You feel somewhat better.".to_string());
    } else {
        player.hp = player.hp_max;
        result
            .messages
            .push("You feel completely healed.".to_string());
    }

    // Cure everything
    player.blinded_timeout = 0;
    player.confused_timeout = 0;
    player.stunned_timeout = 0;
    player.hallucinating_timeout = 0;

    // Blessed increases max HP
    if blessed && player.hp_max < 500 {
        let gain = 4 + (player.hp_max / 10).min(10);
        player.hp_max += gain;
        player.hp = player.hp_max;
        result
            .messages
            .push(format!("Your max HP increases by {}!", gain));
    }

    result
}

fn potion_gain_ability(player: &mut You, blessed: bool, rng: &mut GameRng) -> PotionResult {
    let mut result = PotionResult::new();

    if blessed {
        // Increase all stats by 1
        for attr in [
            Attribute::Strength,
            Attribute::Dexterity,
            Attribute::Constitution,
            Attribute::Intelligence,
            Attribute::Wisdom,
            Attribute::Charisma,
        ] {
            let new_val = (player.attr_current.get(attr) + 1).min(player.attr_max.get(attr));
            player.attr_current.set(attr, new_val);
        }
        result.messages.push("You feel great!".to_string());
    } else {
        // Increase random stat by 1
        let stat = rng.rn2(6);
        let (attr, msg) = match stat {
            0 => (Attribute::Strength, "You feel strong!"),
            1 => (Attribute::Dexterity, "You feel agile!"),
            2 => (Attribute::Constitution, "You feel tough!"),
            3 => (Attribute::Intelligence, "You feel smart!"),
            4 => (Attribute::Wisdom, "You feel wise!"),
            _ => (Attribute::Charisma, "You feel charismatic!"),
        };
        let new_val = (player.attr_current.get(attr) + 1).min(player.attr_max.get(attr));
        player.attr_current.set(attr, new_val);
        result.messages.push(msg.to_string());
    }

    result
}

fn potion_restore(player: &mut You, blessed: bool) -> PotionResult {
    let mut result = PotionResult::new();

    // Restore all stats to max
    player.attr_current = player.attr_max;

    if blessed {
        result
            .messages
            .push("You feel like a new person!".to_string());
    } else {
        result.messages.push("You feel restored.".to_string());
    }

    result
}

fn potion_confusion(player: &mut You, blessed: bool, rng: &mut GameRng) -> PotionResult {
    let mut result = PotionResult::new();

    if blessed {
        // Blessed clears confusion
        if player.confused_timeout > 0 {
            player.confused_timeout = 0;
            result.messages.push("Your head feels clear.".to_string());
        } else {
            result
                .messages
                .push("You feel mildly disoriented for a moment.".to_string());
        }
    } else {
        let duration = rng.dice(3, 6) as u16;
        player.confused_timeout = player.confused_timeout.saturating_add(duration);
        result.messages.push("Huh, what? Where am I?".to_string());
    }

    result
}

fn potion_blindness(
    player: &mut You,
    blessed: bool,
    cursed: bool,
    rng: &mut GameRng,
) -> PotionResult {
    let mut result = PotionResult::new();

    if blessed {
        if player.blinded_timeout > 0 {
            player.blinded_timeout = 0;
            result.messages.push("Your vision clears.".to_string());
        } else {
            result
                .messages
                .push("It looks dark for a moment.".to_string());
        }
    } else {
        let duration = if cursed {
            rng.dice(5, 50) as u16
        } else {
            rng.dice(3, 25) as u16
        };
        player.blinded_timeout = player.blinded_timeout.saturating_add(duration);
        result
            .messages
            .push("A cloud of darkness falls upon you.".to_string());
    }

    result
}

fn potion_paralysis(
    player: &mut You,
    blessed: bool,
    cursed: bool,
    rng: &mut GameRng,
) -> PotionResult {
    let mut result = PotionResult::new();

    if blessed {
        result.messages.push("You stiffen momentarily.".to_string());
    } else {
        let duration = if cursed {
            rng.dice(3, 6) as u16
        } else {
            rng.dice(2, 4) as u16
        };
        player.paralyzed_timeout = player.paralyzed_timeout.saturating_add(duration);
        result.messages.push("Your limbs freeze!".to_string());
    }

    result
}

fn potion_speed(player: &mut You, blessed: bool, rng: &mut GameRng) -> PotionResult {
    let mut result = PotionResult::new();

    let duration = if blessed {
        rng.dice(10, 10)
    } else {
        rng.dice(5, 10)
    };

    player.properties.set_timeout(Property::Speed, duration);
    result
        .messages
        .push("You feel yourself moving faster.".to_string());

    result
}

fn potion_levitation(
    player: &mut You,
    blessed: bool,
    cursed: bool,
    rng: &mut GameRng,
) -> PotionResult {
    let mut result = PotionResult::new();

    let duration = if cursed {
        rng.dice(20, 10) // Long duration, can't control
    } else if blessed {
        rng.dice(5, 10)
    } else {
        rng.dice(10, 10)
    };

    player
        .properties
        .set_timeout(Property::Levitation, duration);
    result.messages.push("You float into the air!".to_string());

    if cursed {
        result
            .messages
            .push("You have no control over your levitation!".to_string());
    }

    result
}

fn potion_hallucination(player: &mut You, cursed: bool, rng: &mut GameRng) -> PotionResult {
    let mut result = PotionResult::new();

    let duration = if cursed {
        rng.dice(5, 50) as u16
    } else {
        rng.dice(3, 25) as u16
    };

    player.hallucinating_timeout = player.hallucinating_timeout.saturating_add(duration);
    result
        .messages
        .push("Oh wow! Everything seems so cosmic!".to_string());

    result
}

fn potion_invisibility(player: &mut You, blessed: bool, rng: &mut GameRng) -> PotionResult {
    let mut result = PotionResult::new();

    let duration = if blessed {
        rng.dice(15, 10)
    } else {
        rng.dice(10, 10)
    };

    player
        .properties
        .set_timeout(Property::Invisibility, duration);
    result
        .messages
        .push("Gee! All of a sudden, you can't see yourself.".to_string());

    result
}

fn potion_see_invisible(player: &mut You, blessed: bool) -> PotionResult {
    let mut result = PotionResult::new();

    if blessed {
        player.properties.grant_intrinsic(Property::SeeInvisible);
        result.messages.push("You feel perceptive!".to_string());
    } else {
        player.properties.set_timeout(Property::SeeInvisible, 750);
        result
            .messages
            .push("You can see the invisible.".to_string());
    }

    result
}

fn potion_gain_level(player: &mut You, cursed: bool) -> PotionResult {
    let mut result = PotionResult::new();

    if cursed {
        result
            .messages
            .push("You rise up, through the ceiling!".to_string());
        // Go up one dungeon level
        result.level_change = 1;
    } else {
        player.exp_level += 1;
        let hp_gain = 5 + player.attr_current.get(Attribute::Constitution) as i32 / 3;
        player.hp_max += hp_gain;
        player.hp += hp_gain;
        result
            .messages
            .push(format!("Welcome to experience level {}!", player.exp_level));
    }

    result
}

fn potion_enlightenment(player: &You) -> PotionResult {
    let mut result = PotionResult::new();
    result
        .messages
        .push("You feel self-knowledgeable...".to_string());

    // Display player stats summary
    result.messages.push(format!(
        "You are at experience level {} with {} HP.",
        player.exp_level, player.hp
    ));

    result
}

fn potion_monster_detection(_player: &You) -> PotionResult {
    let mut result = PotionResult::new();
    result
        .messages
        .push("You sense the presence of monsters.".to_string());
    // Monster detection reveals all monsters on the current level
    // The caller should mark all monsters as detected/visible
    result
}

fn potion_object_detection(_player: &You) -> PotionResult {
    let mut result = PotionResult::new();
    result
        .messages
        .push("You sense the presence of objects.".to_string());
    // Object detection reveals all objects on the current level
    // The caller should mark all object positions as known
    result
}

fn potion_gain_energy(player: &mut You, blessed: bool, rng: &mut GameRng) -> PotionResult {
    let mut result = PotionResult::new();

    let gain = if blessed {
        rng.dice(4, 10) as i32 + 10
    } else {
        rng.dice(3, 6) as i32
    };

    player.energy = (player.energy + gain).min(player.energy_max);
    result
        .messages
        .push(format!("You feel magical energy! (+{} Pw)", gain));

    // Blessed can increase max energy
    if blessed && player.energy_max < 500 {
        player.energy_max += 1;
    }

    result
}

fn potion_sleeping(player: &mut You, rng: &mut GameRng) -> PotionResult {
    let mut result = PotionResult::new();

    if player.properties.has(Property::SleepResistance) {
        result.messages.push("You yawn.".to_string());
    } else {
        player.sleeping_timeout = rng.dice(4, 6) as u16;
        result.messages.push("You fall asleep!".to_string());
    }

    result
}

fn potion_polymorph(player: &mut You, rng: &mut GameRng) -> PotionResult {
    let mut result = PotionResult::new();
    result
        .messages
        .push("You feel like a new person!".to_string());
    // Polymorph changes player's form temporarily
    // For now, grant random stat changes as a simplified effect
    let stat_change = rng.rnd(3) as i8 - 1; // -1, 0, or +1
    player.attr_current.modify(Attribute::Strength, stat_change);
    player
        .attr_current
        .modify(Attribute::Dexterity, stat_change);
    // Set polymorph timeout
    player.polymorph_timeout = 100 + rng.rnd(100);
    result
}

fn potion_booze(player: &mut You, rng: &mut GameRng) -> PotionResult {
    let mut result = PotionResult::new();

    let confusion = rng.dice(3, 6) as u16;
    player.confused_timeout = player.confused_timeout.saturating_add(confusion);

    // Small healing effect
    player.hp = (player.hp + 1).min(player.hp_max);

    result
        .messages
        .push("Ooph! This tastes like liquid fire!".to_string());

    result
}

fn potion_sickness(player: &mut You, blessed: bool) -> PotionResult {
    let mut result = PotionResult::new();

    if blessed {
        result.messages.push("It tastes terrible.".to_string());
    } else if player.properties.has(Property::PoisonResistance) {
        result
            .messages
            .push("It tastes terrible, but you resist!".to_string());
    } else {
        // Lose stats
        player.attr_current.modify(Attribute::Constitution, -1);
        player.attr_current.modify(Attribute::Strength, -1);
        result.messages.push("You feel very sick.".to_string());
    }

    result
}

fn potion_fruit_juice(player: &mut You) -> PotionResult {
    let mut result = PotionResult::new();

    // Provides nutrition
    player.nutrition += 50;
    result
        .messages
        .push("This tastes like fruit juice.".to_string());

    result
}

fn potion_acid(player: &mut You, rng: &mut GameRng) -> PotionResult {
    let mut result = PotionResult::new();

    if player.properties.has(Property::AcidResistance) {
        result.messages.push("This tastes sour.".to_string());
    } else {
        let damage = rng.dice(2, 6) as i32;
        player.hp -= damage;
        result
            .messages
            .push(format!("This burns! You take {} acid damage!", damage));
        result.player_died = player.hp <= 0;
    }

    result
}

fn potion_oil(player: &mut You) -> PotionResult {
    let mut result = PotionResult::new();

    player.nutrition += 10;
    result
        .messages
        .push("That was smooth and greasy.".to_string());

    result
}

fn potion_water(player: &mut You, buc: &BucStatus) -> PotionResult {
    let mut result = PotionResult::new();

    match buc {
        BucStatus::Blessed => {
            // Holy water - remove curse effects
            result.messages.push("This feels blessed.".to_string());
            player.properties.remove_intrinsic(Property::Fumbling);
            // Additional curse removal would be handled by caller with inventory access
        }
        BucStatus::Cursed => {
            // Unholy water
            result.messages.push("This water is foul!".to_string());
            player.hp -= 1;
        }
        BucStatus::Uncursed => {
            result.messages.push("This tastes like water.".to_string());
        }
    }

    result
}

fn potion_poison(player: &mut You, blessed: bool, rng: &mut GameRng) -> PotionResult {
    let mut result = PotionResult::new();

    if blessed {
        result.messages.push("This tastes terrible.".to_string());
    } else if player.properties.has(Property::PoisonResistance) {
        result
            .messages
            .push("It tastes terrible, but you resist!".to_string());
    } else {
        // Damage and weaken player
        let damage = rng.dice(2, 6) as i32;
        player.hp -= damage;
        player.attr_current.modify(Attribute::Constitution, -1);
        player.attr_current.modify(Attribute::Strength, -1);
        result
            .messages
            .push("You feel very sick and weakened.".to_string());
        result.player_died = player.hp <= 0;
    }

    result
}

// ============================================================================
// Thrown Potion Handling (potionhit from potion.c)
// ============================================================================

/// Result of a thrown potion hitting a target
#[derive(Debug, Clone)]
pub struct PotionHitResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Damage dealt to target
    pub damage: i32,
    /// Whether potion broke
    pub broke: bool,
    /// Effects applied to target
    pub effects_applied: Vec<String>,
    /// Whether player is the target
    pub hit_player: bool,
    /// Whether the target died
    pub player_died: bool,
}

impl PotionHitResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            damage: 0,
            broke: true,
            effects_applied: Vec::new(),
            hit_player: false,
            player_died: false,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

impl Default for PotionHitResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle thrown potion hitting the player (potion.c:1313)
pub fn potionhit_player(potion: &Object, player: &mut You, rng: &mut GameRng) -> PotionHitResult {
    let mut result = PotionHitResult::new();
    let Some(ptype) = PotionType::from_object_type(potion.object_type) else {
        return result.with_message("The potion crashes but nothing happens.");
    };

    result.hit_player = true;
    result.damage = rng.rnd(2) as i32;
    result
        .messages
        .push("The bottle crashes on your head and breaks into shards.".to_string());

    let blessed = potion.is_blessed();
    let cursed = potion.is_cursed();

    match ptype {
        PotionType::Oil => {
            if potion.is_lit() {
                result
                    .messages
                    .push("The burning oil splashes all over you!".to_string());
                if !player.properties.has(Property::FireResistance) {
                    result.damage = rng.dice(3, 6) as i32;
                    result.effects_applied.push("burning_oil".to_string());
                }
            }
        }
        PotionType::Polymorph => {
            result
                .messages
                .push("You feel a little strange!".to_string());
            if !player.properties.has(Property::Unchanging) {
                result.effects_applied.push("polymorph".to_string());
            }
        }
        PotionType::Acid => {
            if !player.properties.has(Property::AcidResistance) {
                let nd = if cursed { 2 } else { 1 };
                let nsides = if blessed { 4 } else { 8 };
                result.damage = rng.dice(nd, nsides) as i32;
                result.messages.push(format!(
                    "This burns{}!",
                    if blessed {
                        " a little"
                    } else if cursed {
                        " a lot"
                    } else {
                        ""
                    }
                ));
            }
        }
        PotionType::Healing | PotionType::ExtraHealing | PotionType::FullHealing => {
            // Healing potions heal the player when thrown at them
            let heal = match ptype {
                PotionType::FullHealing => player.hp_max - player.hp,
                PotionType::ExtraHealing => rng.dice(6, 8) as i32,
                _ => rng.dice(6, 4) as i32,
            };
            player.hp = (player.hp + heal).min(player.hp_max);
            result
                .messages
                .push("You feel better.".to_string());
            // Cure blindness for blessed healing / extra healing / full healing
            let cureblind = matches!(ptype, PotionType::FullHealing)
                || (matches!(ptype, PotionType::ExtraHealing) && !cursed)
                || blessed;
            if cureblind && player.blinded_timeout > 0 {
                player.blinded_timeout = 0;
                result.messages.push("Your vision clears.".to_string());
            }
        }
        PotionType::Sleeping => {
            if !player.properties.has(Property::SleepResistance) {
                player.sleeping_timeout = rng.rnd(12) as u16;
                result
                    .messages
                    .push("You fall asleep!".to_string());
            }
        }
        PotionType::Paralysis => {
            if !player.properties.has(Property::FreeAction) {
                player.paralyzed_timeout =
                    player.paralyzed_timeout.saturating_add(rng.rnd(25) as u16);
                result
                    .messages
                    .push("Your limbs freeze!".to_string());
            } else {
                result
                    .messages
                    .push("You stiffen momentarily.".to_string());
            }
        }
        PotionType::Confusion | PotionType::Booze => {
            player.confused_timeout = player.confused_timeout.saturating_add(rng.rnd(5) as u16);
            result
                .messages
                .push("You feel somewhat dizzy.".to_string());
        }
        PotionType::Blindness => {
            let duration = 64 + rng.rn2(32) as u16;
            player.blinded_timeout = player.blinded_timeout.saturating_add(duration);
            result
                .messages
                .push("It suddenly gets dark.".to_string());
        }
        PotionType::Speed => {
            let duration = rng.rnd(5);
            player.properties.set_timeout(Property::Speed, duration);
            result
                .messages
                .push("Your knees seem more flexible now.".to_string());
        }
        PotionType::Invisibility => {
            let duration = rng.rnd(15) + 31;
            player.properties.set_timeout(Property::Invisibility, duration);
            result
                .messages
                .push("For an instant you couldn't see yourself.".to_string());
        }
        PotionType::Water => {
            if blessed {
                // Holy water splashed on player
                result
                    .messages
                    .push("You feel a sense of purity.".to_string());
            } else if cursed {
                // Unholy water
                result
                    .messages
                    .push("You feel contaminated.".to_string());
            }
        }
        PotionType::Sickness => {
            if !player.properties.has(Property::PoisonResistance) {
                let hp_loss = if player.hp > 2 { player.hp / 2 } else { 1 };
                player.hp -= hp_loss;
                result
                    .messages
                    .push("You feel rather ill.".to_string());
                result.player_died = player.hp <= 0;
            }
        }
        _ => {
            // Other potions: just splash with no special effect
        }
    }

    result.broke = true;
    result
}

/// Handle thrown potion hitting a monster (potion.c:1416)
pub fn potionhit_monster(
    potion: &Object,
    monster: &mut Monster,
    rng: &mut GameRng,
) -> PotionHitResult {
    let mut result = PotionHitResult::new();
    let Some(ptype) = PotionType::from_object_type(potion.object_type) else {
        result.messages.push("Crash!".to_string());
        return result;
    };

    let blessed = potion.is_blessed();
    let cursed = potion.is_cursed();

    // Physical damage from the bottle
    if rng.rn2(5) != 0 && monster.hp > 1 {
        monster.hp -= 1;
        result.damage = 1;
    }
    result.messages.push("The bottle crashes and breaks into shards.".to_string());

    match ptype {
        PotionType::Healing
        | PotionType::ExtraHealing
        | PotionType::FullHealing
        | PotionType::Restore
        | PotionType::GainAbility => {
            // Healing potions heal the monster
            if monster.hp < monster.hp_max {
                monster.hp = monster.hp_max;
                result
                    .messages
                    .push(format!("The monster looks sound and hale again."));
            }
            // Cure blindness for healing potions
            let cureblind = matches!(ptype, PotionType::FullHealing)
                || (matches!(ptype, PotionType::ExtraHealing) && !cursed)
                || blessed;
            if cureblind {
                monster.state.blinded = false;
            }
            result.effects_applied.push("healing".to_string());
        }
        PotionType::Sickness => {
            // Sickness halves monster HP
            if !monster.resistances.contains(MonsterResistances::POISON) {
                if monster.hp_max > 3 {
                    monster.hp_max /= 2;
                }
                if monster.hp > 2 {
                    monster.hp /= 2;
                }
                if monster.hp > monster.hp_max {
                    monster.hp = monster.hp_max;
                }
                result
                    .messages
                    .push("The monster looks rather ill.".to_string());
            }
        }
        PotionType::Confusion | PotionType::Booze => {
            monster.state.confused = true;
            result
                .messages
                .push("The monster looks confused.".to_string());
        }
        PotionType::Invisibility => {
            monster.state.invisible = true;
            result
                .messages
                .push("The monster vanishes!".to_string());
        }
        PotionType::Sleeping => {
            if !monster.resistances.contains(MonsterResistances::SLEEP) {
                monster.state.sleeping = true;
                result
                    .messages
                    .push("The monster falls asleep.".to_string());
            }
        }
        PotionType::Paralysis => {
            if monster.state.can_move {
                monster.state.can_move = false;
                monster.state.paralyzed = true;
                result
                    .messages
                    .push("The monster is frozen in place!".to_string());
            }
        }
        PotionType::Speed => {
            monster.state.hasted = true;
            result
                .messages
                .push("The monster seems to move faster.".to_string());
        }
        PotionType::Blindness => {
            monster.state.blinded = true;
            result
                .messages
                .push("The monster is blinded.".to_string());
        }
        PotionType::Acid => {
            if !monster.resistances.contains(MonsterResistances::ACID) {
                let nd = if cursed { 2 } else { 1 };
                let nsides = if blessed { 4 } else { 8 };
                let dmg = rng.dice(nd, nsides) as i32;
                monster.hp -= dmg;
                result.damage += dmg;
                result
                    .messages
                    .push("The monster writhes in pain!".to_string());
            }
        }
        PotionType::Water => {
            // Holy water damages undead/demons
            if blessed && (monster.is_undead() || monster.is_demon()) {
                let dmg = rng.dice(2, 6) as i32;
                monster.hp -= dmg;
                result.damage += dmg;
                result
                    .messages
                    .push("The monster shrieks in pain!".to_string());
            } else if cursed && (monster.is_undead() || monster.is_demon()) {
                // Unholy water heals undead/demons
                monster.hp = (monster.hp + rng.dice(2, 6) as i32).min(monster.hp_max);
                result
                    .messages
                    .push("The monster looks healthier.".to_string());
            }
        }
        PotionType::Oil => {
            if potion.is_lit() {
                let dmg = rng.dice(3, 6) as i32;
                monster.hp -= dmg;
                result.damage += dmg;
                result
                    .messages
                    .push("The burning oil splashes the monster!".to_string());
            }
        }
        PotionType::Polymorph => {
            result.effects_applied.push("polymorph".to_string());
            result
                .messages
                .push("The monster shimmers and changes!".to_string());
        }
        _ => {
            // Other potions: splash with no special effect on monsters
        }
    }

    // Wake up monster (unless calmed by healing/speed/invis/sleep)
    if !matches!(
        ptype,
        PotionType::Healing
            | PotionType::ExtraHealing
            | PotionType::FullHealing
            | PotionType::Restore
            | PotionType::GainAbility
            | PotionType::Speed
            | PotionType::Invisibility
            | PotionType::Sleeping
    ) {
        monster.state.sleeping = false;
    }

    result.broke = true;
    result
}

/// Inhale potion vapors (potion.c:1613 potionbreathe)
/// Called when a potion breaks near the player
pub fn potionbreathe(potion: &Object, player: &mut You, rng: &mut GameRng) -> PotionResult {
    let mut result = PotionResult::new();
    result.consumed = false; // Vapors don't consume the potion (already broken)

    let Some(ptype) = PotionType::from_object_type(potion.object_type) else {
        return result;
    };

    let blessed = potion.is_blessed();
    let cursed = potion.is_cursed();

    match ptype {
        PotionType::Restore | PotionType::GainAbility => {
            if cursed {
                result
                    .messages
                    .push("Ulch! That potion smells terrible!".to_string());
            } else {
                // Vapors restore one (or all if blessed) depleted stats
                let start = rng.rn2(6) as usize;
                let attrs = [
                    Attribute::Strength,
                    Attribute::Dexterity,
                    Attribute::Constitution,
                    Attribute::Intelligence,
                    Attribute::Wisdom,
                    Attribute::Charisma,
                ];
                for offset in 0..6 {
                    let idx = (start + offset) % 6;
                    let attr = attrs[idx];
                    let current = player.attr_current.get(attr);
                    let max = player.attr_max.get(attr);
                    if current < max {
                        player.attr_current.set(attr, current + 1);
                        if !blessed {
                            break; // Only first if not blessed
                        }
                    }
                }
            }
        }
        PotionType::Healing => {
            if player.hp < player.hp_max {
                player.hp += 1;
            }
            if blessed && player.blinded_timeout > 0 {
                player.blinded_timeout = 0;
                result.messages.push("Your vision clears.".to_string());
            }
        }
        PotionType::ExtraHealing => {
            if player.hp < player.hp_max {
                player.hp += 1;
            }
            // Extra healing vapors always heal one more
            if player.hp < player.hp_max {
                player.hp += 1;
            }
            if !cursed && player.blinded_timeout > 0 {
                player.blinded_timeout = 0;
                result.messages.push("Your vision clears.".to_string());
            }
        }
        PotionType::FullHealing => {
            if player.hp < player.hp_max {
                player.hp += 1;
            }
            if player.hp < player.hp_max {
                player.hp += 1;
            }
            if player.hp < player.hp_max {
                player.hp += 1;
            }
            player.blinded_timeout = 0;
            result.messages.push("Your vision clears.".to_string());
        }
        PotionType::Sickness => {
            // Vapors damage unless healer
            if player.hp <= 5 {
                player.hp = 1;
            } else {
                player.hp -= 5;
            }
        }
        PotionType::Hallucination => {
            result
                .messages
                .push("You have a momentary vision.".to_string());
        }
        PotionType::Confusion | PotionType::Booze => {
            if player.confused_timeout == 0 {
                result
                    .messages
                    .push("You feel somewhat dizzy.".to_string());
            }
            player.confused_timeout = player.confused_timeout.saturating_add(rng.rnd(5) as u16);
        }
        PotionType::Invisibility => {
            result
                .messages
                .push("For an instant you couldn't see yourself.".to_string());
        }
        PotionType::Paralysis => {
            if !player.properties.has(Property::FreeAction) {
                result
                    .messages
                    .push("Something seems to be holding you.".to_string());
                player.paralyzed_timeout =
                    player.paralyzed_timeout.saturating_add(rng.rnd(5) as u16);
            } else {
                result
                    .messages
                    .push("You stiffen momentarily.".to_string());
            }
        }
        PotionType::Sleeping => {
            if !player.properties.has(Property::SleepResistance) {
                result
                    .messages
                    .push("You feel rather tired.".to_string());
                player.sleeping_timeout =
                    player.sleeping_timeout.saturating_add(rng.rnd(5) as u16);
            } else {
                result.messages.push("You yawn.".to_string());
            }
        }
        PotionType::Speed => {
            result
                .messages
                .push("Your knees seem more flexible now.".to_string());
            let duration = rng.rnd(5);
            player.properties.set_timeout(Property::Speed, duration);
        }
        PotionType::Blindness => {
            if player.blinded_timeout == 0 {
                result
                    .messages
                    .push("It suddenly gets dark.".to_string());
            }
            player.blinded_timeout = player.blinded_timeout.saturating_add(rng.rnd(5) as u16);
        }
        _ => {
            // Other potions: no vapor effect
        }
    }

    result
}

/// Backward-compatible alias
pub fn potionhit(potion: &Object, player: &mut You, rng: &mut GameRng) -> PotionHitResult {
    potionhit_player(potion, player, rng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monster::MonsterId;

    fn make_potion(ptype: PotionType, buc: BucStatus) -> Object {
        let mut obj = Object::default();
        obj.object_type = ptype as i16;
        obj.class = crate::object::ObjectClass::Potion;
        obj.buc = buc;
        obj
    }

    #[test]
    fn test_potion_type_from_object() {
        assert_eq!(PotionType::from_object_type(267), Some(PotionType::Healing));
        assert_eq!(PotionType::from_object_type(999), None);
    }

    #[test]
    fn test_glow_strength() {
        assert_eq!(glow_strength(0), 0);
        assert_eq!(glow_strength(1), 1);
        assert_eq!(glow_strength(5), 2);
        assert_eq!(glow_strength(13), 3);
    }

    #[test]
    fn test_glow_verb() {
        let v0 = glow_verb(0, false);
        assert_eq!(v0, "quiver");

        let v1 = glow_verb(1, false);
        assert_eq!(v1, "flicker");

        let v2 = glow_verb(5, false);
        assert_eq!(v2, "glimmer");

        let v3 = glow_verb(13, false);
        assert_eq!(v3, "gleam");

        let v_ing = glow_verb(5, true);
        assert_eq!(v_ing, "glimmering");
    }

    #[test]
    fn test_glow_color() {
        let c = glow_color(3);
        assert_eq!(c, "blue");

        let c = glow_color(7);
        assert_eq!(c, "white");

        let c = glow_color(12);
        assert_eq!(c, "yellow");

        let c = glow_color(100);
        assert_eq!(c, "blue");
    }

    #[test]
    fn test_p_glow1() {
        let msg = p_glow1("Sting");
        assert!(msg.contains("briefly"));
    }

    #[test]
    fn test_p_glow2() {
        let msg = p_glow2("Excalibur", "white");
        assert!(msg.contains("white"));
        assert!(msg.contains("for a moment"));
    }

    #[test]
    fn test_potion_result() {
        let result = PotionResult::new();
        assert!(result.messages.is_empty());
        assert!(result.consumed);
        assert!(!result.player_died);
        assert!(result.identify);

        let result = result.with_message("Test message");
        assert_eq!(result.messages.len(), 1);
    }

    #[test]
    fn test_potion_hit_result() {
        let result = PotionHitResult::new();
        assert!(result.messages.is_empty());
        assert_eq!(result.damage, 0);
        assert!(result.broke);
        assert!(!result.hit_player);

        let result = result.with_message("Crash!");
        assert_eq!(result.messages.len(), 1);
    }

    // ==========================================================================
    // potionhit_player tests
    // ==========================================================================

    #[test]
    fn test_potionhit_player_healing() {
        let mut player = You::default();
        let mut rng = GameRng::new(42);
        player.hp = 10;
        player.hp_max = 50;
        let potion = make_potion(PotionType::Healing, BucStatus::Uncursed);
        let result = potionhit_player(&potion, &mut player, &mut rng);
        assert!(player.hp > 10, "Healing potion should heal player");
        assert!(result.broke);
        assert!(result.hit_player);
    }

    #[test]
    fn test_potionhit_player_acid() {
        let mut player = You::default();
        let mut rng = GameRng::new(42);
        player.hp = 50;
        player.hp_max = 50;
        let potion = make_potion(PotionType::Acid, BucStatus::Uncursed);
        let result = potionhit_player(&potion, &mut player, &mut rng);
        assert!(result.damage > 0, "Acid potion should deal damage");
        assert!(result.messages.iter().any(|m| m.contains("burns")));
    }

    #[test]
    fn test_potionhit_player_sleep() {
        let mut player = You::default();
        let mut rng = GameRng::new(42);
        let potion = make_potion(PotionType::Sleeping, BucStatus::Uncursed);
        let _result = potionhit_player(&potion, &mut player, &mut rng);
        assert!(
            player.sleeping_timeout > 0,
            "Sleep potion should put player to sleep"
        );
    }

    #[test]
    fn test_potionhit_player_paralysis_with_free_action() {
        let mut player = You::default();
        let mut rng = GameRng::new(42);
        player.properties.grant_intrinsic(Property::FreeAction);
        let potion = make_potion(PotionType::Paralysis, BucStatus::Uncursed);
        let result = potionhit_player(&potion, &mut player, &mut rng);
        assert_eq!(
            player.paralyzed_timeout, 0,
            "Free Action should prevent paralysis"
        );
        assert!(result.messages.iter().any(|m| m.contains("momentarily")));
    }

    #[test]
    fn test_potionhit_player_confusion() {
        let mut player = You::default();
        let mut rng = GameRng::new(42);
        let potion = make_potion(PotionType::Confusion, BucStatus::Uncursed);
        let _result = potionhit_player(&potion, &mut player, &mut rng);
        assert!(
            player.confused_timeout > 0,
            "Confusion potion should confuse player"
        );
    }

    #[test]
    fn test_potionhit_player_blindness() {
        let mut player = You::default();
        let mut rng = GameRng::new(42);
        let potion = make_potion(PotionType::Blindness, BucStatus::Uncursed);
        let _result = potionhit_player(&potion, &mut player, &mut rng);
        assert!(
            player.blinded_timeout > 0,
            "Blindness potion should blind player"
        );
    }

    // ==========================================================================
    // potionhit_monster tests
    // ==========================================================================

    #[test]
    fn test_potionhit_monster_healing() {
        let mut monster = Monster::new(MonsterId(0), 0, 5, 5);
        let mut rng = GameRng::new(42);
        monster.hp = 5;
        monster.hp_max = 20;
        let potion = make_potion(PotionType::Healing, BucStatus::Uncursed);
        let _result = potionhit_monster(&potion, &mut monster, &mut rng);
        assert_eq!(
            monster.hp, monster.hp_max,
            "Healing potion should fully heal monster"
        );
    }

    #[test]
    fn test_potionhit_monster_sickness() {
        let mut monster = Monster::new(MonsterId(0), 0, 5, 5);
        let mut rng = GameRng::new(42);
        monster.hp = 20;
        monster.hp_max = 20;
        monster.resistances = MonsterResistances::empty(); // No poison resistance
        let potion = make_potion(PotionType::Sickness, BucStatus::Uncursed);
        let _result = potionhit_monster(&potion, &mut monster, &mut rng);
        // Sickness halves HP and max HP; bottle hit may also do 1 damage
        assert_eq!(monster.hp_max, 10, "Sickness should halve monster max HP");
        assert!(
            monster.hp <= 10,
            "Sickness should halve monster HP (plus possible bottle damage)"
        );
    }

    #[test]
    fn test_potionhit_monster_sleeping() {
        let mut monster = Monster::new(MonsterId(0), 0, 5, 5);
        let mut rng = GameRng::new(42);
        monster.resistances = MonsterResistances::empty(); // No sleep resistance
        let potion = make_potion(PotionType::Sleeping, BucStatus::Uncursed);
        let _result = potionhit_monster(&potion, &mut monster, &mut rng);
        assert!(
            monster.state.sleeping,
            "Sleep potion should put monster to sleep"
        );
    }

    #[test]
    fn test_potionhit_monster_paralysis() {
        let mut monster = Monster::new(MonsterId(0), 0, 5, 5);
        let mut rng = GameRng::new(42);
        monster.state.can_move = true;
        let potion = make_potion(PotionType::Paralysis, BucStatus::Uncursed);
        let _result = potionhit_monster(&potion, &mut monster, &mut rng);
        assert!(
            monster.state.paralyzed,
            "Paralysis potion should paralyze monster"
        );
        assert!(
            !monster.state.can_move,
            "Paralyzed monster cannot move"
        );
    }

    #[test]
    fn test_potionhit_monster_speed() {
        let mut monster = Monster::new(MonsterId(0), 0, 5, 5);
        let mut rng = GameRng::new(42);
        let potion = make_potion(PotionType::Speed, BucStatus::Uncursed);
        let _result = potionhit_monster(&potion, &mut monster, &mut rng);
        assert!(
            monster.state.hasted,
            "Speed potion should haste monster"
        );
    }

    #[test]
    fn test_potionhit_monster_acid() {
        let mut monster = Monster::new(MonsterId(0), 0, 5, 5);
        let mut rng = GameRng::new(42);
        monster.hp = 50;
        monster.hp_max = 50;
        let potion = make_potion(PotionType::Acid, BucStatus::Uncursed);
        let result = potionhit_monster(&potion, &mut monster, &mut rng);
        assert!(result.damage > 0, "Acid should deal damage to monster");
        assert!(monster.hp < 50);
    }

    // ==========================================================================
    // potionbreathe tests
    // ==========================================================================

    #[test]
    fn test_potionbreathe_healing() {
        let mut player = You::default();
        let mut rng = GameRng::new(42);
        player.hp = 10;
        player.hp_max = 50;
        let potion = make_potion(PotionType::Healing, BucStatus::Uncursed);
        let _result = potionbreathe(&potion, &mut player, &mut rng);
        assert_eq!(player.hp, 11, "Healing vapors should heal 1 HP");
    }

    #[test]
    fn test_potionbreathe_full_healing_cures_blindness() {
        let mut player = You::default();
        let mut rng = GameRng::new(42);
        player.blinded_timeout = 50;
        let potion = make_potion(PotionType::FullHealing, BucStatus::Uncursed);
        let result = potionbreathe(&potion, &mut player, &mut rng);
        assert_eq!(
            player.blinded_timeout, 0,
            "Full healing vapors cure blindness"
        );
        assert!(result.messages.iter().any(|m| m.contains("vision clears")));
    }

    #[test]
    fn test_potionbreathe_sickness_damages() {
        let mut player = You::default();
        let mut rng = GameRng::new(42);
        player.hp = 20;
        player.hp_max = 50;
        let potion = make_potion(PotionType::Sickness, BucStatus::Uncursed);
        let _result = potionbreathe(&potion, &mut player, &mut rng);
        assert_eq!(player.hp, 15, "Sickness vapors should reduce HP by 5");
    }

    #[test]
    fn test_potionbreathe_confusion() {
        let mut player = You::default();
        let mut rng = GameRng::new(42);
        let potion = make_potion(PotionType::Confusion, BucStatus::Uncursed);
        let result = potionbreathe(&potion, &mut player, &mut rng);
        assert!(
            player.confused_timeout > 0,
            "Confusion vapors should confuse"
        );
        assert!(result.messages.iter().any(|m| m.contains("dizzy")));
    }

    #[test]
    fn test_potionbreathe_sleep_with_resistance() {
        let mut player = You::default();
        let mut rng = GameRng::new(42);
        player.properties.grant_intrinsic(Property::SleepResistance);
        let potion = make_potion(PotionType::Sleeping, BucStatus::Uncursed);
        let result = potionbreathe(&potion, &mut player, &mut rng);
        assert_eq!(
            player.sleeping_timeout, 0,
            "Sleep resistance prevents sleep from vapors"
        );
        assert!(result.messages.iter().any(|m| m.contains("yawn")));
    }

    #[test]
    fn test_potionbreathe_restore_ability_blessed() {
        let mut player = You::default();
        let mut rng = GameRng::new(42);
        // Reduce some stats below max
        player.attr_max.set(Attribute::Strength, 18);
        player.attr_max.set(Attribute::Dexterity, 16);
        player.attr_current.set(Attribute::Strength, 14);
        player.attr_current.set(Attribute::Dexterity, 12);
        let potion = make_potion(PotionType::Restore, BucStatus::Blessed);
        let _result = potionbreathe(&potion, &mut player, &mut rng);
        // Blessed should restore all depleted stats by 1
        assert!(
            player.attr_current.get(Attribute::Strength) >= 15
                || player.attr_current.get(Attribute::Dexterity) >= 13,
            "Blessed restore vapors should improve at least one stat"
        );
    }

    #[test]
    fn test_potionbreathe_speed() {
        let mut player = You::default();
        let mut rng = GameRng::new(42);
        let potion = make_potion(PotionType::Speed, BucStatus::Uncursed);
        let result = potionbreathe(&potion, &mut player, &mut rng);
        assert!(result.messages.iter().any(|m| m.contains("flexible")));
    }
}
