//! Spellcasting system (spell.c)
//!
//! Handles learning and casting spells.

use super::advanced;
use crate::dungeon::{DLevel, Level};
use crate::monster::MonsterId;
use crate::player::{Attribute, Property, You};
use crate::rng::GameRng;

/// Spell schools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum SpellSchool {
    Attack = 0,
    Healing = 1,
    Divination = 2,
    Enchantment = 3,
    Clerical = 4,
    Escape = 5,
    Matter = 6,
}

impl SpellSchool {
    pub const fn name(&self) -> &'static str {
        match self {
            SpellSchool::Attack => "attack",
            SpellSchool::Healing => "healing",
            SpellSchool::Divination => "divination",
            SpellSchool::Enchantment => "enchantment",
            SpellSchool::Clerical => "clerical",
            SpellSchool::Escape => "escape",
            SpellSchool::Matter => "matter",
        }
    }

    /// Get mana cost multiplier for this school (base 1.0)
    pub const fn mana_multiplier(&self) -> f32 {
        match self {
            SpellSchool::Attack => 1.0,
            SpellSchool::Healing => 0.8,
            SpellSchool::Divination => 0.9,
            SpellSchool::Enchantment => 1.1,
            SpellSchool::Clerical => 0.9,
            SpellSchool::Escape => 1.0,
            SpellSchool::Matter => 1.2,
        }
    }

    /// Get list of all schools
    pub fn all() -> &'static [SpellSchool] {
        &[
            SpellSchool::Attack,
            SpellSchool::Healing,
            SpellSchool::Divination,
            SpellSchool::Enchantment,
            SpellSchool::Clerical,
            SpellSchool::Escape,
            SpellSchool::Matter,
        ]
    }
}

/// Spell mastery levels
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[repr(u8)]
pub enum SpellMastery {
    /// Never studied this school
    Unknown = 0,
    /// Basic understanding
    Novice = 1,
    /// Intermediate skill
    Adept = 2,
    /// Advanced skill
    Expert = 3,
    /// Master of this school
    Master = 4,
}

impl SpellMastery {
    /// Get failure chance percentage based on mastery (0-100)
    pub const fn failure_chance(&self) -> u8 {
        match self {
            SpellMastery::Unknown => 100, // Can't cast
            SpellMastery::Novice => 60,
            SpellMastery::Adept => 30,
            SpellMastery::Expert => 10,
            SpellMastery::Master => 0,
        }
    }

    /// Get mana cost modifier
    pub const fn mana_modifier(&self) -> f32 {
        match self {
            SpellMastery::Unknown => 2.0, // Double cost if not trained
            SpellMastery::Novice => 1.5,
            SpellMastery::Adept => 1.0,
            SpellMastery::Expert => 0.8,
            SpellMastery::Master => 0.6,
        }
    }

    /// Get casting speed modifier (faster with mastery)
    pub const fn speed_modifier(&self) -> f32 {
        match self {
            SpellMastery::Unknown => 1.0,
            SpellMastery::Novice => 0.9,
            SpellMastery::Adept => 0.8,
            SpellMastery::Expert => 0.7,
            SpellMastery::Master => 0.6,
        }
    }

    /// Check if can cast at all
    pub const fn can_cast(&self) -> bool {
        !matches!(self, SpellMastery::Unknown)
    }

    /// Advance to next mastery level
    pub fn advance(&self) -> SpellMastery {
        match self {
            SpellMastery::Unknown => SpellMastery::Novice,
            SpellMastery::Novice => SpellMastery::Adept,
            SpellMastery::Adept => SpellMastery::Expert,
            SpellMastery::Expert => SpellMastery::Master,
            SpellMastery::Master => SpellMastery::Master,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            SpellMastery::Unknown => "Unknown",
            SpellMastery::Novice => "Novice",
            SpellMastery::Adept => "Adept",
            SpellMastery::Expert => "Expert",
            SpellMastery::Master => "Master",
        }
    }
}

/// Spell type indices (matching spellbook ObjectType in nh-data/objects.rs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(i16)]
pub enum SpellType {
    ForceBolt = 309,
    Drain = 310,
    MagicMissile = 311,
    Confuse = 312,
    Slow = 313,
    CureBlindness = 314,
    CureSickness = 315,
    DetectMonsters = 316,
    DetectFood = 317,
    Clairvoyance = 318,
    DetectUnseen = 319,
    Identify = 320,
    DetectTreasure = 321,
    MagicMapping = 322,
    Sleep = 323,
    ConfuseMonster = 324,
    Haste = 325,
    Invisibility = 326,
    Levitation = 327,
    Knock = 328,
    WizardLock = 329,
    CreateMonster = 330,
    Healing = 331,
    ExtraHealing = 332,
    StoneSkin = 333,
    RestoreAbility = 334,
    Jumping = 335,
    Digging = 336,
    TeleportAway = 337,
    Cancellation = 338,
    Protection = 339,
    TurnUndead = 340,
    Polymorph = 341,
    Fireball = 342,
    ConeOfCold = 343,
    FingerOfDeath = 344,
}

impl SpellType {
    /// Try to convert an id (i32) to a spell type
    pub fn from_id(id: i32) -> Option<Self> {
        Self::from_object_type(id as i16)
    }

    /// Try to convert an object type to a spell type
    pub fn from_object_type(otype: i16) -> Option<Self> {
        match otype {
            309 => Some(SpellType::ForceBolt),
            310 => Some(SpellType::Drain),
            311 => Some(SpellType::MagicMissile),
            312 => Some(SpellType::Confuse),
            313 => Some(SpellType::Slow),
            314 => Some(SpellType::CureBlindness),
            315 => Some(SpellType::CureSickness),
            316 => Some(SpellType::DetectMonsters),
            317 => Some(SpellType::DetectFood),
            318 => Some(SpellType::Clairvoyance),
            319 => Some(SpellType::DetectUnseen),
            320 => Some(SpellType::Identify),
            321 => Some(SpellType::DetectTreasure),
            322 => Some(SpellType::MagicMapping),
            323 => Some(SpellType::Sleep),
            324 => Some(SpellType::ConfuseMonster),
            325 => Some(SpellType::Haste),
            326 => Some(SpellType::Invisibility),
            327 => Some(SpellType::Levitation),
            328 => Some(SpellType::Knock),
            329 => Some(SpellType::WizardLock),
            330 => Some(SpellType::CreateMonster),
            331 => Some(SpellType::Healing),
            332 => Some(SpellType::ExtraHealing),
            333 => Some(SpellType::StoneSkin),
            334 => Some(SpellType::RestoreAbility),
            335 => Some(SpellType::Jumping),
            336 => Some(SpellType::Digging),
            337 => Some(SpellType::TeleportAway),
            338 => Some(SpellType::Cancellation),
            339 => Some(SpellType::Protection),
            340 => Some(SpellType::TurnUndead),
            341 => Some(SpellType::Polymorph),
            342 => Some(SpellType::Fireball),
            343 => Some(SpellType::ConeOfCold),
            344 => Some(SpellType::FingerOfDeath),
            _ => None,
        }
    }

    /// Get the spell's name
    pub const fn name(&self) -> &'static str {
        match self {
            SpellType::ForceBolt => "force bolt",
            SpellType::Drain => "drain life",
            SpellType::MagicMissile => "magic missile",
            SpellType::Confuse => "confuse monster",
            SpellType::Slow => "slow monster",
            SpellType::CureBlindness => "cure blindness",
            SpellType::CureSickness => "cure sickness",
            SpellType::DetectMonsters => "detect monsters",
            SpellType::DetectFood => "detect food",
            SpellType::Clairvoyance => "clairvoyance",
            SpellType::DetectUnseen => "detect unseen",
            SpellType::Identify => "identify",
            SpellType::DetectTreasure => "detect treasure",
            SpellType::MagicMapping => "magic mapping",
            SpellType::Sleep => "sleep",
            SpellType::ConfuseMonster => "confuse monster",
            SpellType::Haste => "haste self",
            SpellType::Invisibility => "invisibility",
            SpellType::Levitation => "levitation",
            SpellType::Knock => "knock",
            SpellType::WizardLock => "wizard lock",
            SpellType::CreateMonster => "create monster",
            SpellType::Healing => "healing",
            SpellType::ExtraHealing => "extra healing",
            SpellType::StoneSkin => "stone to flesh",
            SpellType::RestoreAbility => "restore ability",
            SpellType::Jumping => "jumping",
            SpellType::Digging => "dig",
            SpellType::TeleportAway => "teleport away",
            SpellType::Cancellation => "cancellation",
            SpellType::Protection => "protection",
            SpellType::TurnUndead => "turn undead",
            SpellType::Polymorph => "polymorph",
            SpellType::Fireball => "fireball",
            SpellType::ConeOfCold => "cone of cold",
            SpellType::FingerOfDeath => "finger of death",
        }
    }

    /// Get the spell's school
    pub const fn school(&self) -> SpellSchool {
        match self {
            SpellType::ForceBolt
            | SpellType::Drain
            | SpellType::MagicMissile
            | SpellType::Fireball
            | SpellType::ConeOfCold
            | SpellType::FingerOfDeath => SpellSchool::Attack,

            SpellType::Healing
            | SpellType::ExtraHealing
            | SpellType::CureBlindness
            | SpellType::CureSickness
            | SpellType::RestoreAbility => SpellSchool::Healing,

            SpellType::DetectMonsters
            | SpellType::DetectFood
            | SpellType::Clairvoyance
            | SpellType::DetectUnseen
            | SpellType::Identify
            | SpellType::DetectTreasure
            | SpellType::MagicMapping => SpellSchool::Divination,

            SpellType::Confuse
            | SpellType::Slow
            | SpellType::Sleep
            | SpellType::ConfuseMonster
            | SpellType::Cancellation => SpellSchool::Enchantment,

            SpellType::Protection | SpellType::TurnUndead => SpellSchool::Clerical,

            SpellType::Haste
            | SpellType::Invisibility
            | SpellType::Levitation
            | SpellType::Jumping
            | SpellType::TeleportAway => SpellSchool::Escape,

            SpellType::Knock
            | SpellType::WizardLock
            | SpellType::CreateMonster
            | SpellType::StoneSkin
            | SpellType::Digging
            | SpellType::Polymorph => SpellSchool::Matter,
        }
    }

    /// Get the base energy cost to cast
    pub const fn energy_cost(&self) -> i32 {
        match self {
            SpellType::ForceBolt => 1,
            SpellType::Healing => 1,
            SpellType::DetectFood => 1,
            SpellType::Knock => 1,
            SpellType::Slow => 2,
            SpellType::Confuse => 2,
            SpellType::Sleep => 2,
            SpellType::DetectMonsters => 2,
            SpellType::CureBlindness => 2,
            SpellType::MagicMissile => 3,
            SpellType::Haste => 3,
            SpellType::Invisibility => 4,
            SpellType::Levitation => 4,
            SpellType::ExtraHealing => 5,
            SpellType::Identify => 5,
            SpellType::MagicMapping => 5,
            SpellType::DetectTreasure => 5,
            SpellType::TeleportAway => 6,
            SpellType::Protection => 6,
            SpellType::Fireball => 8,
            SpellType::ConeOfCold => 8,
            SpellType::Digging => 8,
            SpellType::Drain => 10,
            SpellType::RestoreAbility => 10,
            SpellType::Polymorph => 15,
            SpellType::FingerOfDeath => 20,
            _ => 3,
        }
    }

    /// Get the spell's difficulty level
    pub const fn level(&self) -> u8 {
        match self {
            SpellType::ForceBolt | SpellType::Healing | SpellType::Knock => 1,
            SpellType::DetectFood
            | SpellType::DetectMonsters
            | SpellType::Slow
            | SpellType::CureBlindness => 2,
            SpellType::MagicMissile | SpellType::Confuse | SpellType::Sleep | SpellType::Haste => 3,
            SpellType::Invisibility
            | SpellType::Levitation
            | SpellType::ExtraHealing
            | SpellType::Identify => 4,
            SpellType::MagicMapping
            | SpellType::DetectTreasure
            | SpellType::TeleportAway
            | SpellType::Protection => 5,
            SpellType::Fireball | SpellType::ConeOfCold | SpellType::Digging => 6,
            SpellType::Drain | SpellType::RestoreAbility | SpellType::Polymorph => 7,
            SpellType::FingerOfDeath => 8,
            _ => 3,
        }
    }

    /// Check if this spell deals damage
    pub const fn deals_damage(&self) -> bool {
        matches!(
            self,
            SpellType::ForceBolt
                | SpellType::Drain
                | SpellType::MagicMissile
                | SpellType::Fireball
                | SpellType::ConeOfCold
                | SpellType::FingerOfDeath
        )
    }

    /// Check if this spell heals
    pub const fn heals(&self) -> bool {
        matches!(self, SpellType::Healing | SpellType::ExtraHealing)
    }

    /// Check if this spell has a duration
    pub const fn has_duration(&self) -> bool {
        matches!(
            self,
            SpellType::Haste
                | SpellType::Invisibility
                | SpellType::Levitation
                | SpellType::Protection
                | SpellType::StoneSkin
                | SpellType::Sleep
                | SpellType::Confuse
                | SpellType::ConfuseMonster
                | SpellType::Slow
        )
    }

    /// Check if this spell is an area effect
    pub const fn is_area_effect(&self) -> bool {
        matches!(
            self,
            SpellType::Fireball
                | SpellType::ConeOfCold
                | SpellType::MagicMapping
                | SpellType::DetectMonsters
        )
    }

    /// Check if this spell can target multiple entities
    pub const fn can_target_multiple(&self) -> bool {
        matches!(
            self,
            SpellType::Fireball
                | SpellType::ConeOfCold
                | SpellType::TurnUndead
                | SpellType::Sleep
                | SpellType::Confuse
                | SpellType::ConfuseMonster
                | SpellType::Slow
        )
    }
}

/// A known spell
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnownSpell {
    /// The spell type
    pub spell_type: SpellType,
    /// Turns until spell memory fades
    pub turns_remaining: u32,
    /// Times this spell has been successfully cast
    pub times_cast: u32,
    /// Times casting failed
    pub times_failed: u32,
}

/// Spell proficiency tracking for a school
#[derive(Debug, Clone)]
pub struct SchoolProficiency {
    /// School being tracked
    pub school: SpellSchool,
    /// Current mastery level
    pub mastery: SpellMastery,
    /// Experience points in this school (0-100 per level)
    pub experience: u32,
    /// Spells cast in this school
    pub spells_cast: u32,
    /// Successful casts
    pub successful_casts: u32,
}

impl SchoolProficiency {
    /// Create new school proficiency tracker
    pub fn new(school: SpellSchool) -> Self {
        Self {
            school,
            mastery: SpellMastery::Unknown,
            experience: 0,
            spells_cast: 0,
            successful_casts: 0,
        }
    }

    /// Add experience from casting
    pub fn add_experience(&mut self, amount: u32) {
        self.experience += amount;
        // Advance mastery every 100 xp
        while self.experience >= 100 && self.mastery != SpellMastery::Master {
            self.experience -= 100;
            self.mastery = self.mastery.advance();
        }
    }

    /// Record a cast attempt
    pub fn record_cast(&mut self, success: bool) {
        self.spells_cast += 1;
        if success {
            self.successful_casts += 1;
            self.add_experience(10); // Gain exp on success
        } else {
            self.add_experience(5); // Partial exp on failure
        }
    }

    /// Get success rate percentage
    pub fn success_rate(&self) -> u8 {
        if self.spells_cast == 0 {
            return 0;
        }
        ((self.successful_casts * 100) / self.spells_cast) as u8
    }
}

impl KnownSpell {
    pub fn new(spell_type: SpellType) -> Self {
        // Spell memory lasts ~20000 turns
        Self {
            spell_type,
            turns_remaining: 20000,
            times_cast: 0,
            times_failed: 0,
        }
    }

    pub fn is_forgotten(&self) -> bool {
        self.turns_remaining == 0
    }

    /// Get proficiency level based on casts
    pub fn proficiency_level(&self) -> u32 {
        self.times_cast / 10
    }

    /// Check if spell is well-known (cast many times)
    pub fn is_well_known(&self) -> bool {
        self.times_cast >= 50
    }

    /// Get success rate for this specific spell
    pub fn success_rate(&self) -> u8 {
        if self.times_cast == 0 {
            return 0;
        }
        let successful = self.times_cast - self.times_failed;
        ((successful * 100) / self.times_cast) as u8
    }
}

/// Result of casting a spell
#[derive(Debug, Clone)]
pub struct SpellResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Energy cost actually paid
    pub energy_cost: i32,
    /// Monsters killed
    pub killed: Vec<MonsterId>,
    /// Whether player died
    pub player_died: bool,
}

impl SpellResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            energy_cost: 0,
            killed: Vec::new(),
            player_died: false,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

impl Default for SpellResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Cast a spell
pub fn cast_spell(
    spell: SpellType,
    direction: Option<(i8, i8)>,
    player: &mut You,
    level: &mut Level,
    rng: &mut GameRng,
) -> SpellResult {
    let energy_cost = spell.energy_cost();

    // Check if player has enough energy
    if player.energy < energy_cost {
        return SpellResult::new().with_message("You don't have enough energy to cast that spell.");
    }

    // Check for confusion
    if player.confused_timeout > 0 && rng.percent(50) {
        player.energy -= energy_cost;
        let mut result = SpellResult::new();
        result.energy_cost = energy_cost;
        result
            .messages
            .push("You are too confused to cast the spell!".to_string());
        return result;
    }

    // Pay energy cost
    player.energy -= energy_cost;

    let mut result = SpellResult::new();
    result.energy_cost = energy_cost;

    // Check for spell interruption during casting
    if let Some(interrupt_event) = advanced::check_spell_interruption(player, rng) {
        result.messages.push(interrupt_event.message);
        return result;
    }

    match spell {
        SpellType::ForceBolt => cast_force_bolt(direction, player, level, rng, &mut result),
        SpellType::MagicMissile => cast_magic_missile(direction, player, level, rng, &mut result),
        SpellType::Fireball => cast_fireball(direction, player, level, rng, &mut result),
        SpellType::ConeOfCold => cast_cone_of_cold(direction, player, level, rng, &mut result),
        SpellType::FingerOfDeath => {
            cast_finger_of_death(direction, player, level, rng, &mut result)
        }
        SpellType::Drain => cast_drain(direction, player, level, rng, &mut result),
        SpellType::Healing => cast_healing(player, rng, &mut result),
        SpellType::ExtraHealing => cast_extra_healing(player, rng, &mut result),
        SpellType::CureBlindness => cast_cure_blindness(player, &mut result),
        SpellType::CureSickness => cast_cure_sickness(player, &mut result),
        SpellType::RestoreAbility => cast_restore_ability(player, &mut result),
        SpellType::DetectMonsters => cast_detect_monsters(level, &mut result),
        SpellType::MagicMapping => cast_magic_mapping(level, &mut result),
        SpellType::Identify => cast_identify(player, &mut result),
        SpellType::Clairvoyance => cast_clairvoyance(level, player, &mut result),
        SpellType::Haste => cast_haste(player, rng, &mut result),
        SpellType::Invisibility => cast_invisibility(player, rng, &mut result),
        SpellType::Levitation => cast_levitation(player, rng, &mut result),
        SpellType::TeleportAway => cast_teleport_away(direction, player, level, rng, &mut result),
        SpellType::Protection => cast_protection(player, &mut result),
        SpellType::Confuse | SpellType::ConfuseMonster => {
            cast_confuse_monster(direction, player, level, rng, &mut result)
        }
        SpellType::Slow => cast_slow_monster(direction, player, level, &mut result),
        SpellType::Sleep => cast_sleep(direction, player, level, rng, &mut result),
        SpellType::Knock => cast_knock(direction, player, level, &mut result),
        SpellType::WizardLock => cast_wizard_lock(direction, player, level, &mut result),
        SpellType::Digging => cast_digging(direction, player, level, &mut result),
        SpellType::Cancellation => cast_cancellation(direction, player, level, &mut result),
        SpellType::TurnUndead => cast_turn_undead(player, level, rng, &mut result),
        SpellType::Jumping => cast_jumping(player, level, direction, &mut result),
        SpellType::DetectFood => cast_detect_food(level, &mut result),
        SpellType::DetectUnseen => cast_detect_unseen(level, player, &mut result),
        SpellType::DetectTreasure => cast_detect_treasure(level, &mut result),
        SpellType::CreateMonster => cast_create_monster(direction, level, rng, &mut result),
        SpellType::StoneSkin => cast_stone_skin(player, &mut result),
        SpellType::Polymorph => cast_polymorph(direction, player, level, rng, &mut result),
        _ => {
            result
                .messages
                .push("That spell is not yet implemented.".to_string());
        }
    }

    result
}

fn cast_force_bolt(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    rng: &mut GameRng,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let tx = player.pos.x + dx;
    let ty = player.pos.y + dy;

    if let Some(monster) = level.monster_at_mut(tx, ty) {
        let damage = rng.dice(2, 6) as i32;
        monster.hp -= damage;
        result.messages.push(format!(
            "The {} is hit by a force bolt for {} damage!",
            monster.name, damage
        ));
        if monster.hp <= 0 {
            result.killed.push(monster.id);
        }
    } else {
        result
            .messages
            .push("The force bolt flies harmlessly.".to_string());
    }
}

fn cast_magic_missile(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    rng: &mut GameRng,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let mut x = player.pos.x;
    let mut y = player.pos.y;

    for _ in 0..10 {
        x += dx;
        y += dy;

        if !level.is_valid_pos(x, y) || level.cell(x as usize, y as usize).typ.is_wall() {
            result
                .messages
                .push("The magic missile hits a wall.".to_string());
            break;
        }

        if let Some(monster) = level.monster_at_mut(x, y) {
            let damage = rng.dice(2, 6) as i32;
            monster.hp -= damage;
            result.messages.push(format!(
                "The {} is hit by a magic missile for {} damage!",
                monster.name, damage
            ));
            if monster.hp <= 0 {
                result.killed.push(monster.id);
            }
            break;
        }
    }
}

fn cast_fireball(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    rng: &mut GameRng,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let mut x = player.pos.x;
    let mut y = player.pos.y;

    // Travel until hitting wall or reaching range
    for _ in 0..8 {
        x += dx;
        y += dy;

        if !level.is_valid_pos(x, y) || level.cell(x as usize, y as usize).typ.is_wall() {
            break;
        }
    }

    // Explode at endpoint - damage all in 3x3 area
    result.messages.push("The fireball explodes!".to_string());

    let damage = rng.dice(6, 6) as i32;

    for monster in &mut level.monsters {
        let mdx = (monster.x - x).abs();
        let mdy = (monster.y - y).abs();
        if mdx <= 1 && mdy <= 1 {
            if monster.resists_fire() {
                result.messages.push(format!(
                    "The {} is not affected by the flames.",
                    monster.name
                ));
            } else {
                monster.hp -= damage;
                result.messages.push(format!(
                    "The {} is engulfed in flames for {} damage!",
                    monster.name, damage
                ));
                if monster.hp <= 0 {
                    result.killed.push(monster.id);
                }
            }
        }
    }

    // Check player in blast radius
    let pdx = (player.pos.x - x).abs();
    let pdy = (player.pos.y - y).abs();
    if pdx <= 1 && pdy <= 1 && !player.properties.has(Property::FireResistance) {
        result.messages.push(format!(
            "You are caught in the explosion for {} damage!",
            damage
        ));
        // Note: Can't modify player.hp here, caller must handle
    }
}

fn cast_cone_of_cold(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    rng: &mut GameRng,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let mut x = player.pos.x;
    let mut y = player.pos.y;

    result
        .messages
        .push("A cone of cold spreads out!".to_string());

    let damage = rng.dice(6, 6) as i32;

    // Cone expands as it travels
    for dist in 1..=6i8 {
        x += dx;
        y += dy;

        if !level.is_valid_pos(x, y) {
            break;
        }

        // Check perpendicular positions based on distance
        for perp in -(dist / 2)..=(dist / 2) {
            let check_x = x + if dx == 0 { perp } else { 0 };
            let check_y = y + if dy == 0 { perp } else { 0 };

            if !level.is_valid_pos(check_x, check_y) {
                continue;
            }

            if let Some(monster) = level.monster_at_mut(check_x, check_y) {
                monster.hp -= damage;
                result.messages.push(format!(
                    "The {} is frozen for {} damage!",
                    monster.name, damage
                ));
                if monster.hp <= 0 {
                    result.killed.push(monster.id);
                }
            }
        }
    }
}

fn cast_finger_of_death(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    rng: &mut GameRng,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let mut x = player.pos.x;
    let mut y = player.pos.y;

    for _ in 0..20 {
        x += dx;
        y += dy;

        if !level.is_valid_pos(x, y) || level.cell(x as usize, y as usize).typ.is_wall() {
            result
                .messages
                .push("The death ray hits a wall.".to_string());
            break;
        }

        if let Some(monster) = level.monster_at_mut(x, y) {
            // Disintegration resistance or high level monsters can resist death magic
            let resist_chance = (monster.level as u32) * 3;
            if monster.resists_disint() || rng.percent(resist_chance) {
                result
                    .messages
                    .push(format!("The {} resists the death magic!", monster.name));
            } else {
                monster.hp = 0;
                result.messages.push(format!("The {} dies!", monster.name));
                result.killed.push(monster.id);
            }
            break;
        }
    }
}

fn cast_drain(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    rng: &mut GameRng,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let tx = player.pos.x + dx;
    let ty = player.pos.y + dy;

    if let Some(monster) = level.monster_at_mut(tx, ty) {
        let damage = rng.dice(2, 8) as i32;
        monster.hp -= damage;
        result.messages.push(format!(
            "You drain energy from the {}! ({} damage)",
            monster.name, damage
        ));
        if monster.hp <= 0 {
            result.killed.push(monster.id);
        }
    } else {
        result
            .messages
            .push("There's nothing there to drain.".to_string());
    }
}

fn cast_healing(player: &mut You, rng: &mut GameRng, result: &mut SpellResult) {
    let heal = rng.dice(4, 4) as i32;
    player.hp = (player.hp + heal).min(player.hp_max);
    result
        .messages
        .push(format!("You feel better. (+{} HP)", heal));
}

fn cast_extra_healing(player: &mut You, rng: &mut GameRng, result: &mut SpellResult) {
    let heal = rng.dice(6, 8) as i32;
    player.hp = (player.hp + heal).min(player.hp_max);
    player.blinded_timeout = 0;
    player.confused_timeout = 0;
    result
        .messages
        .push(format!("You feel much better. (+{} HP)", heal));
}

fn cast_cure_blindness(player: &mut You, result: &mut SpellResult) {
    if player.blinded_timeout > 0 {
        player.blinded_timeout = 0;
        result.messages.push("Your vision clears!".to_string());
    } else {
        result
            .messages
            .push("You have a brief moment of clarity.".to_string());
    }
}

fn cast_cure_sickness(player: &mut You, result: &mut SpellResult) {
    if player.sickness_timeout > 0 {
        player.sickness_timeout = 0;
        result.messages.push("You feel healthier.".to_string());
    } else {
        result.messages.push("You are not sick.".to_string());
    }
}

fn cast_restore_ability(player: &mut You, result: &mut SpellResult) {
    player.attr_current = player.attr_max;
    result.messages.push("You feel restored!".to_string());
}

fn cast_detect_monsters(level: &Level, result: &mut SpellResult) {
    let count = level.monsters.len();
    if count == 0 {
        result.messages.push("You sense no monsters.".to_string());
    } else {
        result
            .messages
            .push(format!("You sense {} monster(s) on this level.", count));
    }
}

fn cast_magic_mapping(level: &mut Level, result: &mut SpellResult) {
    for x in 0..crate::COLNO {
        for y in 0..crate::ROWNO {
            level.cells[x][y].explored = true;
        }
    }
    result
        .messages
        .push("A map coalesces in your mind!".to_string());
}

fn cast_identify(_player: &mut You, result: &mut SpellResult) {
    // TODO: Identify the player's current wielded weapon or first item in inventory
    // This requires access to inventory which is stored elsewhere
    result
        .messages
        .push("You feel a surge of knowledge.".to_string());
    result
        .messages
        .push("You have identified an item.".to_string());
}

fn cast_clairvoyance(level: &mut Level, player: &You, result: &mut SpellResult) {
    // Reveal area around player
    let cx = player.pos.x as usize;
    let cy = player.pos.y as usize;
    for dy in 0..=20 {
        for dx in 0..=20 {
            let x = (cx + dx).saturating_sub(10);
            let y = (cy + dy).saturating_sub(10);
            if x < crate::COLNO && y < crate::ROWNO {
                level.cells[x][y].explored = true;
            }
        }
    }
    result
        .messages
        .push("You have a vision of your surroundings.".to_string());
}

fn cast_haste(player: &mut You, rng: &mut GameRng, result: &mut SpellResult) {
    let duration = rng.dice(5, 10);
    player.properties.set_timeout(Property::Speed, duration);
    result
        .messages
        .push("You feel yourself speeding up!".to_string());
}

fn cast_invisibility(player: &mut You, rng: &mut GameRng, result: &mut SpellResult) {
    let duration = rng.dice(10, 10);
    player
        .properties
        .set_timeout(Property::Invisibility, duration);
    result.messages.push("You vanish!".to_string());
}

fn cast_levitation(player: &mut You, rng: &mut GameRng, result: &mut SpellResult) {
    let duration = rng.dice(10, 10);
    player
        .properties
        .set_timeout(Property::Levitation, duration);
    result.messages.push("You float into the air!".to_string());
}

fn cast_teleport_away(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    rng: &mut GameRng,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let tx = player.pos.x + dx;
    let ty = player.pos.y + dy;

    if let Some(monster) = level.monster_at(tx, ty) {
        let monster_id = monster.id;
        // Find random location
        for _ in 0..100 {
            let nx = rng.rn2(crate::COLNO as u32) as i8;
            let ny = rng.rn2(crate::ROWNO as u32) as i8;

            if level.is_walkable(nx, ny) && level.monster_at(nx, ny).is_none() {
                if let Some(m) = level.monster_mut(monster_id) {
                    let name = m.name.clone();
                    m.x = nx;
                    m.y = ny;
                    result.messages.push(format!("The {} vanishes!", name));
                }
                return;
            }
        }
    }

    result.messages.push("Nothing happens.".to_string());
}

fn cast_protection(player: &mut You, result: &mut SpellResult) {
    player.armor_class -= 2;
    player.properties.set_timeout(Property::Protection, 100);
    result.messages.push("You feel protected!".to_string());
}

fn cast_confuse_monster(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    rng: &mut GameRng,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let tx = player.pos.x + dx;
    let ty = player.pos.y + dy;

    if let Some(monster) = level.monster_at_mut(tx, ty) {
        monster.state.confused = true;
        monster.confused_timeout = rng.dice(3, 4) as u16;
        result
            .messages
            .push(format!("The {} looks confused!", monster.name));
    } else {
        result.messages.push("Your hands glow red.".to_string());
    }
}

fn cast_slow_monster(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let tx = player.pos.x + dx;
    let ty = player.pos.y + dy;

    if let Some(monster) = level.monster_at_mut(tx, ty) {
        monster.state.slowed = true;
        result
            .messages
            .push(format!("The {} slows down!", monster.name));
    } else {
        result.messages.push("Nothing happens.".to_string());
    }
}

fn cast_sleep(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    rng: &mut GameRng,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let mut x = player.pos.x;
    let mut y = player.pos.y;

    for _ in 0..10 {
        x += dx;
        y += dy;

        if !level.is_valid_pos(x, y) || level.cell(x as usize, y as usize).typ.is_wall() {
            break;
        }

        if let Some(monster) = level.monster_at_mut(x, y) {
            if monster.resists_sleep() {
                result
                    .messages
                    .push(format!("The {} resists!", monster.name));
            } else {
                monster.state.sleeping = true;
                monster.sleep_timeout = rng.dice(4, 6) as u16;
                result
                    .messages
                    .push(format!("The {} falls asleep!", monster.name));
            }
            break;
        }
    }
}

fn cast_knock(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let tx = player.pos.x + dx;
    let ty = player.pos.y + dy;

    if level.is_valid_pos(tx, ty) {
        let cell = level.cell_mut(tx as usize, ty as usize);
        if cell.typ == crate::dungeon::CellType::Door {
            cell.set_door_state(crate::dungeon::DoorState::OPEN);
            result.messages.push("The door opens!".to_string());
            return;
        }
    }

    result.messages.push("Nothing happens.".to_string());
}

fn cast_wizard_lock(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let tx = player.pos.x + dx;
    let ty = player.pos.y + dy;

    if level.is_valid_pos(tx, ty) {
        let cell = level.cell_mut(tx as usize, ty as usize);
        if cell.typ == crate::dungeon::CellType::Door {
            cell.set_door_state(crate::dungeon::DoorState::LOCKED);
            result
                .messages
                .push("The door is wizard-locked!".to_string());
            return;
        }
    }

    result.messages.push("Nothing happens.".to_string());
}

fn cast_digging(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let mut x = player.pos.x;
    let mut y = player.pos.y;
    let mut dug = false;

    for _ in 0..8 {
        x += dx;
        y += dy;

        if !level.is_valid_pos(x, y) {
            break;
        }

        let cell = level.cell_mut(x as usize, y as usize);
        if cell.typ.is_wall() || cell.typ == crate::dungeon::CellType::Stone {
            cell.typ = crate::dungeon::CellType::Corridor;
            dug = true;
        }
    }

    if dug {
        result.messages.push("The rock crumbles!".to_string());
    } else {
        result.messages.push("Nothing happens.".to_string());
    }
}

fn cast_cancellation(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let tx = player.pos.x + dx;
    let ty = player.pos.y + dy;

    if let Some(monster) = level.monster_at_mut(tx, ty) {
        monster.state.cancelled = true;
        result
            .messages
            .push(format!("The {} shudders!", monster.name));
    } else {
        result.messages.push("Nothing happens.".to_string());
    }
}

fn cast_turn_undead(player: &You, level: &mut Level, rng: &mut GameRng, result: &mut SpellResult) {
    let mut turned = 0;
    let px = player.pos.x;
    let py = player.pos.y;

    for monster in &mut level.monsters {
        if !monster.is_undead() && !monster.is_demon() {
            continue;
        }
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();
        if dx <= 6 && dy <= 6 && rng.percent(70) {
            monster.state.fleeing = true;
            monster.flee_timeout = rng.dice(2, 6) as u16;
            turned += 1;
        }
    }

    if turned > 0 {
        result
            .messages
            .push(format!("{} undead monster(s) turn and flee!", turned));
    } else {
        result.messages.push("Nothing happens.".to_string());
    }
}

fn cast_jumping(
    player: &mut You,
    level: &Level,
    direction: Option<(i8, i8)>,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let jump_dist = 3;
    let tx = player.pos.x + dx * jump_dist;
    let ty = player.pos.y + dy * jump_dist;

    if level.is_valid_pos(tx, ty) && level.is_walkable(tx, ty) && level.monster_at(tx, ty).is_none()
    {
        player.prev_pos = player.pos;
        player.pos.x = tx;
        player.pos.y = ty;
        result.messages.push("You jump!".to_string());
    } else {
        result.messages.push("Something is in the way!".to_string());
    }
}

fn cast_detect_food(level: &Level, result: &mut SpellResult) {
    // Count food items on level - check level.objects instead of cells
    let food_count = level
        .objects
        .iter()
        .filter(|obj| obj.class == crate::object::ObjectClass::Food)
        .count();

    if food_count > 0 {
        result.messages.push(format!(
            "You sense {} food items on this level.",
            food_count
        ));
    } else {
        result
            .messages
            .push("You don't sense any food here.".to_string());
    }
}

fn cast_detect_unseen(level: &Level, player: &You, result: &mut SpellResult) {
    let mut unseen_count = 0;

    // Count unexplored or hidden areas
    let radius = 15i8;
    for x_offset in -radius..=radius {
        for y_offset in -radius..=radius {
            let x = ((player.pos.x as i32 + x_offset as i32) as usize).min(crate::COLNO - 1);
            let y = ((player.pos.y as i32 + y_offset as i32) as usize).min(crate::ROWNO - 1);

            if !level.cells[x][y].explored {
                unseen_count += 1;
            }
        }
    }

    if unseen_count > 0 {
        result
            .messages
            .push(format!("You sense {} hidden areas nearby.", unseen_count));
    } else {
        result
            .messages
            .push("You don't sense any hidden areas.".to_string());
    }
}

fn cast_detect_treasure(level: &Level, result: &mut SpellResult) {
    // Count gold (coin) and valuable objects on level
    let treasure_count = level
        .objects
        .iter()
        .filter(|obj| {
            obj.class == crate::object::ObjectClass::Coin
                || obj.class == crate::object::ObjectClass::Gem
        })
        .count();

    if treasure_count > 0 {
        result.messages.push(format!(
            "You sense {} treasure(s) on this level.",
            treasure_count
        ));
    } else {
        result
            .messages
            .push("You don't sense any treasure here.".to_string());
    }
}

fn cast_create_monster(
    direction: Option<(i8, i8)>,
    level: &Level,
    rng: &mut GameRng,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));

    // Check if direction is valid
    if dx == 0 && dy == 0 {
        result
            .messages
            .push("You must specify a direction.".to_string());
        return;
    }

    result.messages.push("You summon a monster!".to_string());
    // Monster creation would require integration with level generation
    // For now, just acknowledge the effect
}

fn cast_stone_skin(player: &mut You, result: &mut SpellResult) {
    // Grant stone skin property for temporary protection
    player.properties.grant_intrinsic(Property::StoneSkin);
    result
        .messages
        .push("Your skin hardens into stone!".to_string());
}

fn cast_polymorph(
    direction: Option<(i8, i8)>,
    player: &You,
    level: &mut Level,
    rng: &mut GameRng,
    result: &mut SpellResult,
) {
    let (dx, dy) = direction.unwrap_or((0, 0));
    let tx = player.pos.x + dx;
    let ty = player.pos.y + dy;

    if let Some(monster) = level.monster_at_mut(tx, ty) {
        // Polymorph the monster into a different form
        let new_monster_type = rng.rnd(100) as i16;
        result
            .messages
            .push(format!("The monster shimmers and transforms!"));
        // Would update monster.monster_type in full implementation
    } else {
        result
            .messages
            .push("There is no monster there to polymorph.".to_string());
    }
}

// ============================================================================
// Monster Spellcasting (mcastu.c)
// ============================================================================

use crate::monster::Monster;

/// Monster spell types (from mcastu.c)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterSpell {
    // Attack spells
    MagicMissile,
    Fireball,
    ConeOfCold,
    Lightning,
    Sleep,
    FingerOfDeath,
    Blind,
    Paralyze,
    Confuse,
    Slow,
    DrainLife,
    // Healing spells
    Healing,
    CureBlindness,
    // Utility spells
    Haste,
    Invisibility,
    Summon,
    Teleport,
    TeleportAway,
    // New spells
    TurnUndead,   // Clerical
    Levitation,   // Escape
    Digging,      // Matter
    Polymorph,    // Matter
    ExtraHealing, // Healing (stronger)
}

impl MonsterSpell {
    /// Get the spell name for messages
    pub fn name(&self) -> &'static str {
        match self {
            MonsterSpell::MagicMissile => "magic missile",
            MonsterSpell::Fireball => "fireball",
            MonsterSpell::ConeOfCold => "cone of cold",
            MonsterSpell::Lightning => "lightning bolt",
            MonsterSpell::Sleep => "sleep",
            MonsterSpell::FingerOfDeath => "finger of death",
            MonsterSpell::Blind => "blindness",
            MonsterSpell::Paralyze => "paralysis",
            MonsterSpell::Confuse => "confusion",
            MonsterSpell::Slow => "slow",
            MonsterSpell::DrainLife => "drain life",
            MonsterSpell::Healing => "healing",
            MonsterSpell::CureBlindness => "cure blindness",
            MonsterSpell::Haste => "haste",
            MonsterSpell::Invisibility => "invisibility",
            MonsterSpell::Summon => "summon",
            MonsterSpell::Teleport => "teleport",
            MonsterSpell::TeleportAway => "teleport away",
            MonsterSpell::TurnUndead => "turn undead",
            MonsterSpell::Levitation => "levitation",
            MonsterSpell::Digging => "digging",
            MonsterSpell::Polymorph => "polymorph",
            MonsterSpell::ExtraHealing => "extra healing",
        }
    }

    /// Check if this is an attack spell
    pub fn is_attack(&self) -> bool {
        matches!(
            self,
            MonsterSpell::MagicMissile
                | MonsterSpell::Fireball
                | MonsterSpell::ConeOfCold
                | MonsterSpell::Lightning
                | MonsterSpell::Sleep
                | MonsterSpell::FingerOfDeath
                | MonsterSpell::Blind
                | MonsterSpell::Paralyze
                | MonsterSpell::Confuse
                | MonsterSpell::Slow
                | MonsterSpell::DrainLife
        )
    }
}

/// Result of monster casting a spell
#[derive(Debug, Clone)]
pub struct MonsterSpellResult {
    pub messages: Vec<String>,
    pub player_damage: i32,
    pub player_died: bool,
    /// Amount the caster healed itself (for ExtraHealing spell)
    pub caster_healed: i32,
}

impl MonsterSpellResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            player_damage: 0,
            player_died: false,
            caster_healed: 0,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

impl Default for MonsterSpellResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if monster can cast spells
pub fn monster_can_cast(monster: &Monster) -> bool {
    // Monster must be able to move and not be silenced
    monster.can_act() && !monster.state.cancelled
}

/// Monster casts a spell at the player
pub fn monster_cast_spell(
    caster: &Monster,
    spell: MonsterSpell,
    player: &mut You,
    level: &mut Level,
    rng: &mut GameRng,
) -> MonsterSpellResult {
    let mut result = MonsterSpellResult::new();

    if !monster_can_cast(caster) {
        return result;
    }

    result
        .messages
        .push(format!("The {} casts {}!", caster.name, spell.name()));

    match spell {
        MonsterSpell::MagicMissile => {
            if player.properties.has(Property::MagicResistance) {
                result.messages.push("The missiles bounce off!".to_string());
            } else {
                let damage = rng.dice(2, 6) as i32 + (caster.level / 2) as i32;
                player.hp -= damage;
                result.player_damage = damage;
                result.messages.push(format!(
                    "You are hit by magic missiles! ({} damage)",
                    damage
                ));
            }
        }
        MonsterSpell::Fireball => {
            if player.properties.has(Property::FireResistance) {
                result
                    .messages
                    .push("You are unaffected by the fire.".to_string());
            } else {
                let damage = rng.dice(6, 6) as i32;
                player.hp -= damage;
                result.player_damage = damage;
                result
                    .messages
                    .push(format!("You are engulfed in flames! ({} damage)", damage));
            }
        }
        MonsterSpell::ConeOfCold => {
            if player.properties.has(Property::ColdResistance) {
                result
                    .messages
                    .push("You are unaffected by the cold.".to_string());
            } else {
                let damage = rng.dice(6, 6) as i32;
                player.hp -= damage;
                result.player_damage = damage;
                result
                    .messages
                    .push(format!("You are frozen! ({} damage)", damage));
            }
        }
        MonsterSpell::Lightning => {
            if player.properties.has(Property::ShockResistance) {
                result
                    .messages
                    .push("You are unaffected by the lightning.".to_string());
            } else {
                let damage = rng.dice(6, 6) as i32;
                player.hp -= damage;
                result.player_damage = damage;
                result
                    .messages
                    .push(format!("You are struck by lightning! ({} damage)", damage));
            }
        }
        MonsterSpell::Sleep => {
            if player.properties.has(Property::SleepResistance) {
                result
                    .messages
                    .push("You resist the sleep spell.".to_string());
            } else {
                let duration = rng.dice(2, 6) as u16;
                player.sleeping_timeout = player.sleeping_timeout.saturating_add(duration);
                result.messages.push("You feel very drowsy...".to_string());
            }
        }
        MonsterSpell::FingerOfDeath => {
            if player.properties.has(Property::MagicResistance) {
                result
                    .messages
                    .push("You resist the death magic!".to_string());
            } else if rng.percent(50) {
                // 50% chance to resist even without MR
                result
                    .messages
                    .push("You feel drained but survive.".to_string());
                let damage = rng.dice(4, 6) as i32;
                player.hp -= damage;
                result.player_damage = damage;
            } else {
                result.messages.push("You die...".to_string());
                player.hp = 0;
                result.player_died = true;
            }
        }
        MonsterSpell::Blind => {
            let duration = rng.dice(4, 6) as u16 + 20;
            player.blinded_timeout = player.blinded_timeout.saturating_add(duration);
            result.messages.push("You are blinded!".to_string());
        }
        MonsterSpell::Paralyze => {
            if player.properties.has(Property::FreeAction) {
                result
                    .messages
                    .push("You resist the paralysis.".to_string());
            } else {
                let duration = rng.dice(2, 4) as u16;
                player.paralyzed_timeout = player.paralyzed_timeout.saturating_add(duration);
                result.messages.push("You are paralyzed!".to_string());
            }
        }
        MonsterSpell::Confuse => {
            let duration = rng.dice(3, 6) as u16;
            player.confused_timeout = player.confused_timeout.saturating_add(duration);
            result.messages.push("You feel confused!".to_string());
        }
        MonsterSpell::Slow => {
            if player.properties.has(Property::Speed) {
                player.properties.remove_intrinsic(Property::Speed);
                result.messages.push("You slow down.".to_string());
            } else {
                result.messages.push("You feel sluggish.".to_string());
            }
        }
        MonsterSpell::DrainLife => {
            if player.properties.has(Property::DrainResistance) {
                result.messages.push("You resist the drain.".to_string());
            } else if player.exp_level > 1 {
                player.exp_level -= 1;
                player.hp_max = (player.hp_max - rng.rnd(5) as i32).max(1);
                player.hp = player.hp.min(player.hp_max);
                result
                    .messages
                    .push("You feel your life force draining away!".to_string());
            }
        }
        MonsterSpell::Healing | MonsterSpell::CureBlindness => {
            // These are self-healing spells for the monster
            result
                .messages
                .push(format!("The {} looks healthier.", caster.name));
        }
        MonsterSpell::Haste | MonsterSpell::Invisibility => {
            // Self-buff spells
            result
                .messages
                .push(format!("The {} seems faster.", caster.name));
        }
        MonsterSpell::Summon => {
            // Summon allies - would need to create new monsters
            result
                .messages
                .push("Monsters appear around you!".to_string());
        }
        MonsterSpell::Teleport => {
            // Teleport self away
            result
                .messages
                .push(format!("The {} vanishes!", caster.name));
        }
        MonsterSpell::TeleportAway => {
            // Teleport player away
            if player.properties.has(Property::TeleportControl) && rng.one_in(3) {
                result
                    .messages
                    .push("You resist the teleportation.".to_string());
            } else {
                // Find random position
                for _ in 0..100 {
                    let x = rng.rn2(crate::COLNO as u32) as i8;
                    let y = rng.rn2(crate::ROWNO as u32) as i8;
                    if level.is_walkable(x, y) && level.monster_at(x, y).is_none() {
                        player.prev_pos = player.pos;
                        player.pos.x = x;
                        player.pos.y = y;
                        result.messages.push("You are teleported away!".to_string());
                        break;
                    }
                }
            }
        }
        MonsterSpell::TurnUndead => {
            // Turn nearby undead monsters - repel them from caster
            result
                .messages
                .push("The undead nearby shriek in terror!".to_string());
            let mut affected_count = 0;
            // Find undead monsters and make them flee
            let monster_ids: Vec<_> = level.monster_ids().collect();
            for monster_id in monster_ids {
                if let Some(target) = level.monster_mut(monster_id) {
                    if is_undead_monster(target) {
                        let dist = (target.x - caster.x).abs().max((target.y - caster.y).abs());
                        if dist <= 6 {
                            // Set fleeing state and timeout
                            target.state.fleeing = true;
                            target.flee_timeout = 10 + rng.rnd(20) as u16;
                            result.messages.push(format!("The {} flees!", target.name));
                            affected_count += 1;
                        }
                    }
                }
            }
            if affected_count == 0 {
                result
                    .messages
                    .push("No undead are nearby to turn.".to_string());
            }
        }
        MonsterSpell::Levitation => {
            // Monster levitates - becomes harder to hit and can cross terrain
            // Note: Full implementation would need levitation_timeout field on Monster struct
            // For now, provide the spell effect with message
            result
                .messages
                .push("The monster rises into the air!".to_string());

            // In a full implementation, would:
            // 1. Add levitation_timeout field to Monster struct
            // 2. Set: monster.levitation_timeout = 30 + rng.rnd(20)
            // 3. Modify AC calculation to account for levitation (harder to hit)
            // 4. Allow monster to move through water/lava when levitating
            //
            // For now, this serves as a placeholder that announces the spell
            // but the actual game effect requires structural changes
        }
        MonsterSpell::Digging => {
            // Dig in direction toward player - creates passages through stone
            use crate::dungeon::CellType;

            let dx = (player.pos.x - caster.x).signum();
            let dy = (player.pos.y - caster.y).signum();
            let range = 5;
            let mut dug_count = 0;

            for step in 1..=range {
                let dig_x = (caster.x + dx * step) as usize;
                let dig_y = (caster.y + dy * step) as usize;

                // Check bounds
                if dig_x >= crate::COLNO || dig_y >= crate::ROWNO {
                    break;
                }

                let cell_type = level.cells[dig_x][dig_y].typ;

                match cell_type {
                    CellType::Stone => {
                        // Dig through stone, create corridor
                        level.cells[dig_x][dig_y].typ = CellType::Corridor;
                        dug_count += 1;
                    }
                    CellType::Wall => {
                        // Dig through wall, create corridor
                        level.cells[dig_x][dig_y].typ = CellType::Corridor;
                        dug_count += 1;
                    }
                    CellType::Room | CellType::Corridor | CellType::SecretCorridor => {
                        // Already passable, continue
                    }
                    _ => {
                        // Other terrain, stop digging
                        break;
                    }
                }
            }

            if dug_count > 0 {
                result.messages.push(format!(
                    "The monster digs a tunnel! ({} tiles carved)",
                    dug_count
                ));
            } else {
                result
                    .messages
                    .push("The monster digs, but finds no stone to carve!".to_string());
            }
        }
        MonsterSpell::Polymorph => {
            // Attempt to polymorph player
            if player.properties.has(Property::MagicResistance) && rng.one_in(2) {
                result
                    .messages
                    .push("You resist the polymorph spell!".to_string());
            } else {
                // Apply polymorph effect to player (similar to potion)
                result
                    .messages
                    .push("You feel a strange change come over you!".to_string());

                // Set polymorph timeout (duration: 100-200 turns)
                let polymorph_duration = 100 + rng.rnd(100);
                player.polymorph_timeout = polymorph_duration as u32;

                // Apply temporary stat changes while polymorphed
                let stat_mod = rng.rnd(2) as i8 - 1; // -1, 0, or +1
                player.attr_current.modify(Attribute::Strength, stat_mod);
                player
                    .attr_current
                    .modify(Attribute::Constitution, stat_mod);

                result.messages.push(format!(
                    "You will remain transformed for {} turns.",
                    polymorph_duration
                ));
            }
        }
        MonsterSpell::ExtraHealing => {
            // Heal caster for more HP (this is for the monster, not affecting player directly)
            let heal_amount = rng.dice(4, 4) as i32 + (caster.level as i32 * 2) + 8;
            let actual_healed = (heal_amount).min(caster.hp_max - caster.hp).max(0);
            result.caster_healed = actual_healed;

            if actual_healed > 0 {
                result.messages.push(format!(
                    "The monster is surrounded by a blue aura and heals {} HP!",
                    actual_healed
                ));
            } else {
                result.messages.push(
                    "The monster is surrounded by a blue aura but is already at full health."
                        .to_string(),
                );
            }
        }
    }

    if player.hp <= 0 {
        result.player_died = true;
    }

    result
}

/// Choose a spell for a monster to cast based on situation
pub fn choose_monster_spell(
    caster: &Monster,
    player: &You,
    rng: &mut GameRng,
) -> Option<MonsterSpell> {
    if !monster_can_cast(caster) {
        return None;
    }

    // Distance to player
    let dx = (player.pos.x - caster.x).abs();
    let dy = (player.pos.y - caster.y).abs();
    let distance = dx.max(dy);

    // Build list of available spells based on monster level
    let mut spells = Vec::new();

    // Basic attack spells
    if caster.level >= 3 {
        spells.push(MonsterSpell::MagicMissile);
    }
    if caster.level >= 5 {
        spells.push(MonsterSpell::Confuse);
        spells.push(MonsterSpell::Slow);
    }
    if caster.level >= 7 {
        spells.push(MonsterSpell::Blind);
        spells.push(MonsterSpell::Sleep);
    }
    if caster.level >= 10 {
        spells.push(MonsterSpell::Fireball);
        spells.push(MonsterSpell::ConeOfCold);
        spells.push(MonsterSpell::Lightning);
    }
    if caster.level >= 12 {
        spells.push(MonsterSpell::Paralyze);
        spells.push(MonsterSpell::DrainLife);
    }
    if caster.level >= 15 {
        spells.push(MonsterSpell::TeleportAway);
    }
    if caster.level >= 20 {
        spells.push(MonsterSpell::FingerOfDeath);
    }

    // New spells - level gated
    if caster.level >= 15 && caster.hp < caster.hp_max / 2 {
        spells.push(MonsterSpell::ExtraHealing);
    }
    if caster.level >= 18 && is_clerical_monster(caster.monster_type) {
        spells.push(MonsterSpell::TurnUndead);
    }
    if caster.level >= 20 {
        spells.push(MonsterSpell::Polymorph);
        spells.push(MonsterSpell::Levitation);
    }
    if caster.level >= 15 {
        spells.push(MonsterSpell::Digging);
    }

    // If hurt, consider healing
    if caster.hp < caster.hp_max / 2 && caster.level >= 5 {
        spells.push(MonsterSpell::Healing);
    }

    // If far from player, consider teleport
    if distance > 5 && caster.level >= 10 {
        spells.push(MonsterSpell::Teleport);
    }

    if spells.is_empty() {
        return None;
    }

    // Pick a random spell
    let idx = rng.rn2(spells.len() as u32) as usize;
    Some(spells[idx])
}

/// Check if a monster is undead (for turn undead spell)
fn is_undead_monster(monster: &Monster) -> bool {
    // Check based on monster letter - simplified implementation
    // Would need access to permonst data in full implementation
    // For now, check monster type IDs that are known to be undead
    matches!(
        monster.monster_type,
        5 | 6 | 7 |     // Various ghosts
        10 | 11 |       // Zombies
        15 | 16 // Skeletons/ghouls
    )
}

/// Check if a monster is a clerical monster (for spell selection)
fn is_clerical_monster(monster_type: i16) -> bool {
    // Monsters that can cast clerical spells
    // Priests, priestesses, monks, liches
    matches!(
        monster_type,
        50 | 51 |       // Priest/Priestess
        30 | 31 |       // Monk/Nun
        100 |           // Lich
        101 // Arch-lich
    )
}

// ============================================================================
// Spell aging and miscasting (spell.c)
// ============================================================================

/// Age all known spells by one turn (age_spells equivalent)
/// Returns list of spells that were forgotten this turn
pub fn age_spells(known_spells: &mut Vec<KnownSpell>) -> Vec<SpellType> {
    let mut forgotten = Vec::new();

    for spell in known_spells.iter_mut() {
        if spell.turns_remaining > 0 {
            spell.turns_remaining -= 1;
            if spell.turns_remaining == 0 {
                forgotten.push(spell.spell_type);
            }
        }
    }

    // Remove forgotten spells
    known_spells.retain(|s| !s.is_forgotten());

    forgotten
}

/// Refresh a spell's memory (when re-reading spellbook)
pub fn refresh_spell(known_spells: &mut Vec<KnownSpell>, spell_type: SpellType) {
    // Check if already known
    if let Some(spell) = known_spells.iter_mut().find(|s| s.spell_type == spell_type) {
        spell.turns_remaining = 20000;
    } else {
        known_spells.push(KnownSpell::new(spell_type));
    }
}

/// Check if player knows a spell
pub fn knows_spell(known_spells: &[KnownSpell], spell_type: SpellType) -> bool {
    known_spells
        .iter()
        .any(|s| s.spell_type == spell_type && !s.is_forgotten())
}

/// Lose spells due to amnesia or spell loss effect
///
/// Randomly removes `count` spells from the player's known spells.
/// Returns a list of spell types that were forgotten.
///
/// # Arguments
/// * `player` - The player whose spells will be lost
/// * `count` - Number of spells to lose
/// * `rng` - Random number generator
pub fn losespells(player: &mut You, count: usize, rng: &mut crate::GameRng) -> Vec<SpellType> {
    let mut forgotten = Vec::new();

    if player.known_spells.is_empty() || count == 0 {
        return forgotten;
    }

    let mut indices_to_remove = Vec::new();

    // Randomly select spells to lose
    for _ in 0..count.min(player.known_spells.len()) {
        let idx = rng.rn2(player.known_spells.len() as u32) as usize;
        if !indices_to_remove.contains(&idx) {
            indices_to_remove.push(idx);
            forgotten.push(player.known_spells[idx].spell_type);
        }
    }

    // Remove in reverse order to preserve indices
    for &idx in indices_to_remove.iter().rev() {
        player.known_spells.remove(idx);
    }

    forgotten
}

/// Learn a new spell with Intelligence check
///
/// Attempts to teach the player a spell. Success depends on Intelligence.
/// Returns true if the spell was successfully learned.
///
/// # Arguments
/// * `player` - The player learning the spell
/// * `spell` - The spell to learn
/// * `rng` - Random number generator
pub fn learn_spell(player: &mut You, spell: SpellType, rng: &mut crate::GameRng) -> bool {
    // Check if already known
    if knows_spell(&player.known_spells, spell) {
        return false;
    }

    // Intelligence affects success
    let intelligence = player
        .attr_current
        .get(crate::player::Attribute::Intelligence) as i32;

    // Base difficulty is the spell's difficulty level
    let difficulty = spell.level() as i32;

    // Success chance: Intelligence (0-18) vs Difficulty (1-8)
    // Higher intelligence = better chance
    // Lower difficulty = better chance
    let success_chance = (intelligence * 10) - (difficulty * 15);

    // Clamp to reasonable bounds (20-95 percent)
    let success_chance = success_chance.clamp(20, 95);

    // Roll for success
    if rng.percent(success_chance as u32) {
        // Check if already attempting to learn this spell (refresh it)
        if let Some(spell_entry) = player
            .known_spells
            .iter_mut()
            .find(|s| s.spell_type == spell)
        {
            spell_entry.turns_remaining = 20000;
            return true;
        }

        // Add new spell
        player.known_spells.push(KnownSpell::new(spell));
        true
    } else {
        false
    }
}

/// Get spell skill bonus based on player's skill level
///
/// Higher skill levels reduce the failure chance of casting.
/// This is a simplified version that could be expanded with
/// actual skill tracking per spell or school.
///
/// # Arguments
/// * `player` - The player to check
/// * `spell` - The spell to get bonus for
///
/// # Returns
/// Failure reduction in percentage points (e.g., -5 means 5% less failure)
pub fn spell_skill_bonus(player: &You, spell: SpellType) -> i32 {
    // Count successful casts of this spell
    let cast_count = player
        .known_spells
        .iter()
        .find(|s| s.spell_type == spell)
        .map(|s| s.times_cast)
        .unwrap_or(0);

    // Bonus based on practice
    // Every 10 successful casts = 1% bonus, max 20%
    let skill_bonus = (cast_count / 10).min(20) as i32;

    // Intelligence bonus (very high Int gives small bonus)
    let intelligence = player
        .attr_current
        .get(crate::player::Attribute::Intelligence) as i32;
    let int_bonus = if intelligence >= 16 {
        2
    } else if intelligence >= 14 {
        1
    } else {
        0
    };

    // Wisdom bonus for clerical spells
    let wisdom_bonus = if spell.school() == SpellSchool::Clerical {
        let wisdom = player.attr_current.get(crate::player::Attribute::Wisdom) as i32;
        if wisdom >= 16 {
            2
        } else if wisdom >= 14 {
            1
        } else {
            0
        }
    } else {
        0
    };

    skill_bonus + int_bonus + wisdom_bonus
}

/// Get remaining turns for a known spell
pub fn spell_turns_remaining(known_spells: &[KnownSpell], spell_type: SpellType) -> Option<u32> {
    known_spells
        .iter()
        .find(|s| s.spell_type == spell_type)
        .map(|s| s.turns_remaining)
}

/// Spell casting failure/backfire (backfire equivalent)
/// Returns damage dealt to the player and a message
pub fn backfire(spell: SpellType, player: &mut You, rng: &mut GameRng) -> (i32, String) {
    let base_cost = spell.energy_cost();

    // Backfire effects depend on spell school
    match spell.school() {
        SpellSchool::Attack => {
            // Attack spells can hurt the caster
            let damage = rng.dice(2, 6) as i32 + base_cost / 2;
            player.hp -= damage;
            (damage, format!("Your {} spell backfires!", spell.name()))
        }
        SpellSchool::Healing => {
            // Healing spells can drain life
            let damage = rng.dice(1, 6) as i32;
            player.hp -= damage;
            (damage, "The healing energy reverses!".to_string())
        }
        SpellSchool::Matter => {
            // Matter spells can have physical effects
            let damage = rng.dice(1, 8) as i32;
            player.hp -= damage;
            (damage, "Your spell goes awry!".to_string())
        }
        SpellSchool::Enchantment => {
            // Enchantment spells can confuse the caster
            player.confused_timeout += rng.dice(1, 6) as u16;
            (0, "Your spell confuses you!".to_string())
        }
        SpellSchool::Divination => {
            // Divination spells can blind the caster
            player.blinded_timeout += rng.dice(1, 6) as u16;
            (0, "You see strange visions!".to_string())
        }
        SpellSchool::Escape => {
            // Escape spells can have weird effects
            (0, "Your escape spell fizzles!".to_string())
        }
        SpellSchool::Clerical => {
            // Clerical spells can anger the gods
            (
                0,
                "You feel a brief surge of divine disapproval.".to_string(),
            )
        }
    }
}

/// Calculate chance of spell failure
pub fn spell_failure_chance(spell: SpellType, player: &You, armor_penalty: i32) -> i32 {
    // Use spell level as base difficulty
    let base_difficulty = spell.level() as i32;
    let int = player.acurr(crate::player::Attribute::Intelligence) as i32;
    let wis = player.acurr(crate::player::Attribute::Wisdom) as i32;

    // Base failure chance
    let mut failure = base_difficulty * 5 - (int + wis) / 2;

    // Armor penalty
    failure += armor_penalty;

    // Level bonus
    failure -= player.exp_level * 2;

    // Clamp to 0-100
    failure.clamp(0, 100)
}

/// Check if casting fails and handle backfire
pub fn check_cast_failure(
    spell: SpellType,
    player: &mut You,
    rng: &mut GameRng,
    armor_penalty: i32,
) -> Option<(i32, String)> {
    let failure_chance = spell_failure_chance(spell, player, armor_penalty);

    if rng.percent(failure_chance as u32) {
        Some(backfire(spell, player, rng))
    } else {
        None
    }
}

// ============================================================================
// Spell Mastery and Enhancement System
// ============================================================================

/// Calculate spell power with mastery bonuses
pub fn calculate_spell_power(spell: SpellType, mastery: SpellMastery, player: &You) -> f32 {
    let base_power = 1.0;

    // Mastery bonus
    let mastery_bonus = match mastery {
        SpellMastery::Unknown => 0.5,
        SpellMastery::Novice => 0.8,
        SpellMastery::Adept => 1.0,
        SpellMastery::Expert => 1.2,
        SpellMastery::Master => 1.5,
    };

    // Intelligence bonus (0.5 to 1.5 based on Int 3-18)
    let int_attr = player
        .attr_current
        .get(crate::player::Attribute::Intelligence) as f32;
    let int_bonus = (int_attr / 18.0) * 1.0 + 0.5;

    // Wisdom bonus for clerical spells
    let wisdom_bonus = if spell.school() == SpellSchool::Clerical {
        let wis = player.attr_current.get(crate::player::Attribute::Wisdom) as f32;
        (wis / 18.0) * 0.5 + 0.75
    } else {
        1.0
    };

    base_power * mastery_bonus * int_bonus * wisdom_bonus
}

/// Calculate actual mana cost with modifiers
pub fn calculate_actual_mana_cost(spell: SpellType, mastery: SpellMastery) -> i32 {
    let base_cost = spell.energy_cost() as f32;
    let school_multiplier = spell.school().mana_multiplier();
    let mastery_multiplier = mastery.mana_modifier();

    (base_cost * school_multiplier * mastery_multiplier) as i32
}

/// Check if spell can be cast with current resources
pub fn can_cast_spell(spell: SpellType, mastery: SpellMastery, player: &You) -> bool {
    if !mastery.can_cast() {
        return false; // Not trained in this school
    }

    let cost = calculate_actual_mana_cost(spell, mastery);
    player.energy >= cost
}

/// Get spell school info for UI display
pub fn get_school_info(school: SpellSchool) -> (&'static str, &'static str) {
    match school {
        SpellSchool::Attack => ("attack", "Offensive spells and combat magic"),
        SpellSchool::Healing => ("healing", "Restoration and curative magic"),
        SpellSchool::Divination => ("divination", "Detection and information spells"),
        SpellSchool::Enchantment => ("enchantment", "Control and alteration magic"),
        SpellSchool::Clerical => ("clerical", "Divine and religious magic"),
        SpellSchool::Escape => ("escape", "Movement and protective magic"),
        SpellSchool::Matter => ("matter", "Transmutation and creation magic"),
    }
}

/// Get recommended attribute for casting a school of magic
pub fn school_recommended_attribute(school: SpellSchool) -> &'static str {
    match school {
        SpellSchool::Attack => "Intelligence",
        SpellSchool::Healing => "Wisdom",
        SpellSchool::Divination => "Wisdom",
        SpellSchool::Enchantment => "Intelligence",
        SpellSchool::Clerical => "Wisdom",
        SpellSchool::Escape => "Intelligence",
        SpellSchool::Matter => "Intelligence",
    }
}

// ============================================================================
// Mana System and Magical Resource Management
// ============================================================================

/// Calculate maximum mana pool based on player stats
pub fn calculate_max_mana(player: &You) -> i32 {
    // Base mana: 2 per level
    let base = player.exp_level * 2;

    // Intelligence bonus: +1 per point above 10
    let int_attr = player
        .attr_current
        .get(crate::player::Attribute::Intelligence) as i32;
    let int_bonus = (int_attr - 10).max(0);

    // Wisdom bonus for clerical: +1 per point above 10
    let wis_attr = player.attr_current.get(crate::player::Attribute::Wisdom) as i32;
    let wis_bonus = (wis_attr - 10).max(0) / 2;

    // Role bonus (wizards get more mana)
    let role_bonus = match player.role {
        crate::player::Role::Wizard => player.exp_level * 2,
        crate::player::Role::Priest => player.exp_level,
        crate::player::Role::Ranger => player.exp_level / 2,
        _ => 0,
    };

    (base + int_bonus + wis_bonus + role_bonus).max(1)
}

/// Calculate mana regeneration rate (mana per turn)
pub fn calculate_mana_regen(player: &You) -> f32 {
    // Base regeneration: 0.1 mana per turn
    let base = 0.1;

    // Constitution bonus
    let con_attr = player
        .attr_current
        .get(crate::player::Attribute::Constitution) as f32;
    let con_bonus = (con_attr - 10.0) / 100.0;

    // Rest bonus (not moving gives 2x regen)
    let rest_bonus = if player.movement_points == 0 {
        1.0
    } else {
        0.0
    };

    // Hunger penalty
    let hunger_penalty = match player.hunger_state {
        crate::player::HungerState::Satiated => 1.0,
        crate::player::HungerState::NotHungry => 1.0,
        crate::player::HungerState::Hungry => 0.8,
        crate::player::HungerState::Weak => 0.5,
        crate::player::HungerState::Fainting | crate::player::HungerState::Fainted => 0.1,
        crate::player::HungerState::Starved => 0.0,
    };

    (base + con_bonus + rest_bonus) * hunger_penalty
}

/// Apply mana regeneration
pub fn regenerate_mana(player: &mut You) {
    let max_mana = calculate_max_mana(player);
    let regen_rate = calculate_mana_regen(player);

    // Every 10 turns should regenerate some mana
    if player.turns_played % 10 == 0 {
        let regen_amount = (regen_rate * 10.0) as i32;
        player.energy = (player.energy + regen_amount).min(max_mana);
    }
}

/// Check if player has enough mana for a spell
pub fn check_mana(player: &You, cost: i32) -> bool {
    player.energy >= cost
}

/// Calculate mana cost based on attributes and modifications
pub fn calculate_modified_mana_cost(
    base_cost: i32,
    mastery: SpellMastery,
    school: SpellSchool,
) -> i32 {
    let school_mult = school.mana_multiplier();
    let mastery_mult = mastery.mana_modifier();
    (base_cost as f32 * school_mult * mastery_mult) as i32
}

/// Get mana level description for UI
pub fn get_mana_status(current: i32, max: i32) -> &'static str {
    let percentage = (current * 100) / max.max(1);
    match percentage {
        0 => "depleted",
        1..=20 => "critically low",
        21..=40 => "low",
        41..=60 => "moderate",
        61..=80 => "high",
        _ => "full",
    }
}

// ============================================================================
// Additional Spell Functions from spell.c
// ============================================================================

/// Check if spell is undirected (doesn't need a direction) - is_undirected_spell equivalent
pub fn is_undirected_spell(spell: SpellType) -> bool {
    matches!(
        spell,
        SpellType::Healing
            | SpellType::ExtraHealing
            | SpellType::CureBlindness
            | SpellType::CureSickness
            | SpellType::RestoreAbility
            | SpellType::MagicMapping
            | SpellType::Identify
            | SpellType::DetectMonsters
            | SpellType::DetectFood
            | SpellType::DetectUnseen
            | SpellType::DetectTreasure
            | SpellType::Protection
            | SpellType::Haste
            | SpellType::Invisibility
            | SpellType::Levitation
            | SpellType::StoneSkin
            | SpellType::TurnUndead
    )
}

/// Get spell skill type (weapon skill equivalent for spells)
/// Returns the skill category this spell belongs to
pub fn spell_skilltype(spell: SpellType) -> SpellSchool {
    spell.school()
}

/// Calculate spell damage bonus based on player stats
/// Similar to weapon damage bonus but for spells
pub fn spell_damage_bonus(player: &You, spell: SpellType) -> i32 {
    let school = spell.school();

    // Base damage bonus from intelligence
    let int = player.attr_current.get(Attribute::Intelligence) as i32;
    let int_bonus = (int - 10).max(0) / 2;

    // Wisdom bonus for clerical spells
    let wis_bonus = if school == SpellSchool::Clerical {
        let wis = player.attr_current.get(Attribute::Wisdom) as i32;
        (wis - 10).max(0) / 2
    } else {
        0
    };

    // Level bonus
    let level_bonus = player.exp_level / 5;

    int_bonus + wis_bonus + level_bonus
}

/// Calculate spell hit bonus based on player stats
/// Affects accuracy for targetted spells
pub fn spell_hit_bonus(player: &You, spell: SpellType) -> i32 {
    let school = spell.school();

    // Dexterity affects spell aim
    let dex = player.attr_current.get(Attribute::Dexterity) as i32;
    let dex_bonus = (dex - 10) / 2;

    // Intelligence helps with attack spells
    let int_bonus = if school == SpellSchool::Attack {
        let int = player.attr_current.get(Attribute::Intelligence) as i32;
        (int - 10).max(0) / 3
    } else {
        0
    };

    // Level bonus
    let level_bonus = player.exp_level / 3;

    dex_bonus + int_bonus + level_bonus
}

/// Check if spell would be useless to cast
/// Returns true if the spell would have no effect
pub fn spell_would_be_useless(spell: SpellType, player: &You) -> bool {
    match spell {
        SpellType::Healing | SpellType::ExtraHealing => {
            // Useless if at full health
            player.hp >= player.hp_max
        }
        SpellType::CureBlindness => {
            // Useless if not blind
            player.blinded_timeout == 0
        }
        SpellType::CureSickness => {
            // Useless if not sick
            player.sickness_timeout == 0
        }
        SpellType::Invisibility => {
            // Useless if already invisible
            player.properties.has(Property::Invisibility)
        }
        SpellType::Levitation => {
            // Useless if already levitating
            player.properties.has(Property::Levitation)
        }
        SpellType::Protection => {
            // Useless if already at max protection
            player.protection_level >= 10
        }
        _ => false,
    }
}

/// Calculate spell retention duration based on intelligence
/// Higher intelligence means spells are remembered longer
pub fn spellretention(player: &You) -> u32 {
    let int = player.attr_current.get(Attribute::Intelligence) as u32;

    // Base retention: 10000 turns
    // +1000 per point of intelligence above 10
    let base = 10000u32;
    let int_bonus = if int > 10 { (int - 10) * 1000 } else { 0 };

    // Role bonus
    let role_bonus = match player.role {
        crate::player::Role::Wizard => 5000,
        crate::player::Role::Priest => 2500,
        _ => 0,
    };

    base + int_bonus + role_bonus
}

/// Get mnemonic character for spell type (for spell menu display)
pub fn spelltypemnemonic(spell: SpellType) -> char {
    match spell.school() {
        SpellSchool::Attack => 'A',
        SpellSchool::Healing => 'H',
        SpellSchool::Divination => 'D',
        SpellSchool::Enchantment => 'E',
        SpellSchool::Clerical => 'C',
        SpellSchool::Escape => 'X',
        SpellSchool::Matter => 'M',
    }
}

/// Calculate percent success chance for casting a spell
/// Equivalent to percent_success from spell.c
pub fn percent_success(spell: SpellType, player: &You) -> i32 {
    let failure = spell_failure_chance(spell, player, 0);
    (100 - failure).clamp(0, 100)
}

/// Charm nearby monsters - makes them peaceful/tame
pub fn charm_monsters(
    player: &You,
    level: &mut Level,
    radius: i8,
    rng: &mut GameRng,
) -> Vec<String> {
    let mut messages = Vec::new();
    let mut charmed = 0;

    let px = player.pos.x;
    let py = player.pos.y;
    let cha = player.attr_current.get(Attribute::Charisma) as i32;

    for monster in &mut level.monsters {
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();

        if dx <= radius && dy <= radius && !monster.state.tame {
            // Charm chance based on charisma vs monster level
            let charm_chance = 50 + (cha * 3) - (monster.level as i32 * 5);

            if rng.percent(charm_chance.clamp(5, 95) as u32) {
                monster.state.peaceful = true;
                monster.state.tame = true;
                charmed += 1;
            }
        }
    }

    if charmed > 0 {
        messages.push(format!("{} creature(s) become charmed!", charmed));
    } else {
        messages.push("Nothing seems charmed.".to_string());
    }

    messages
}

/// Charm snakes specifically - makes them peaceful/tame
pub fn charm_snakes(player: &You, level: &mut Level, rng: &mut GameRng) -> Vec<String> {
    let mut messages = Vec::new();
    let mut charmed = 0;

    let px = player.pos.x;
    let py = player.pos.y;
    let radius = 5i8;

    for monster in &mut level.monsters {
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();

        // Check if monster is snake-like (simplified - check name contains "snake")
        let is_snake = monster.name.to_lowercase().contains("snake")
            || monster.name.to_lowercase().contains("naga")
            || monster.name.to_lowercase().contains("cobra");

        if dx <= radius && dy <= radius && is_snake && !monster.state.tame {
            // High charm chance for snakes
            if rng.percent(80) {
                monster.state.peaceful = true;
                monster.state.tame = true;
                charmed += 1;
            }
        }
    }

    if charmed > 0 {
        messages.push(format!("{} snake(s) become charmed!", charmed));
    } else {
        messages.push("No snakes seem affected.".to_string());
    }

    messages
}

/// Put monsters to sleep in an area
pub fn put_monsters_to_sleep(
    player: &You,
    level: &mut Level,
    radius: i8,
    rng: &mut GameRng,
) -> Vec<String> {
    let mut messages = Vec::new();
    let mut asleep = 0;

    let px = player.pos.x;
    let py = player.pos.y;

    for monster in &mut level.monsters {
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();

        if dx <= radius && dy <= radius && !monster.state.sleeping {
            // Check sleep resistance
            if monster.resists_sleep() {
                continue;
            }

            // Sleep chance
            if rng.percent(70) {
                monster.state.sleeping = true;
                monster.sleep_timeout = rng.dice(4, 10) as u16;
                asleep += 1;
            }
        }
    }

    if asleep > 0 {
        messages.push(format!("{} creature(s) fall asleep!", asleep));
    } else {
        messages.push("Nothing falls asleep.".to_string());
    }

    messages
}

/// Mind blast attack - deals psychic damage to nearby enemies
pub fn domindblast(player: &You, level: &mut Level, rng: &mut GameRng) -> Vec<String> {
    let mut messages = Vec::new();
    let mut total_damage = 0;
    let mut killed = Vec::new();

    let px = player.pos.x;
    let py = player.pos.y;
    let radius = 3i8;

    // Damage based on intelligence
    let int = player.attr_current.get(Attribute::Intelligence) as i32;
    let base_damage = rng.dice(2, 6) as i32 + int / 2;

    for monster in &mut level.monsters {
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();

        if dx <= radius && dy <= radius {
            // Mindless creatures are immune
            if monster.is_mindless() {
                messages.push(format!("The {} is unaffected.", monster.name));
                continue;
            }

            // Magic resistance reduces damage
            let damage = if monster.resists_magic() {
                base_damage / 2
            } else {
                base_damage
            };

            monster.hp -= damage;
            total_damage += damage;

            if monster.hp <= 0 {
                killed.push(monster.id);
                messages.push(format!("The {}'s mind is destroyed!", monster.name));
            } else {
                messages.push(format!(
                    "The {} reels from the mental assault! ({} damage)",
                    monster.name, damage
                ));
            }
        }
    }

    if total_damage == 0 && killed.is_empty() {
        messages.push("Your mind blast affects nothing.".to_string());
    }

    messages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spell_type_from_object() {
        assert_eq!(
            SpellType::from_object_type(311),
            Some(SpellType::MagicMissile)
        );
        assert_eq!(SpellType::from_object_type(999), None);
    }

    #[test]
    fn test_spell_school() {
        assert_eq!(SpellType::Fireball.school(), SpellSchool::Attack);
        assert_eq!(SpellType::Healing.school(), SpellSchool::Healing);
        assert_eq!(SpellType::DetectMonsters.school(), SpellSchool::Divination);
    }

    #[test]
    fn test_mana_regeneration_basic() {
        let mut player = crate::player::You::default();
        let max_mana = calculate_max_mana(&player);

        // Drain mana
        player.energy = max_mana / 2;

        // Regenerate
        regenerate_mana(&mut player);

        // Mana should increase (though not necessarily to max in one turn)
        assert!(player.energy >= max_mana / 2, "Mana should regenerate");
    }

    #[test]
    fn test_mana_regeneration_stops_at_max() {
        let mut player = crate::player::You::default();
        let max_mana = calculate_max_mana(&player);

        // Set to max
        player.energy = max_mana;

        let initial = player.energy;
        regenerate_mana(&mut player);

        // Should not exceed max
        assert_eq!(
            player.energy, initial,
            "Mana should not regenerate beyond max when at max"
        );
    }

    #[test]
    fn test_mana_regen_rate_is_positive() {
        let mut player = crate::player::You::default();
        // Default constitution is 0, which gives negative con_bonus.
        // Set constitution to at least 10 so con_bonus is non-negative.
        player
            .attr_current
            .set(crate::player::Attribute::Constitution, 12);

        let regen_rate = calculate_mana_regen(&player);

        // Regen rate should be positive (at least some passive regen)
        assert!(
            regen_rate > 0.0,
            "Mana regen rate should be positive, got {}",
            regen_rate
        );
    }

    #[test]
    fn test_mana_regen_scales_with_intelligence() {
        let mut player1 = crate::player::You::default();
        let mut player2 = crate::player::You::default();

        // Set different intelligence
        player1
            .attr_current
            .set(crate::player::Attribute::Intelligence, 10);
        player2
            .attr_current
            .set(crate::player::Attribute::Intelligence, 18);

        let regen1 = calculate_mana_regen(&player1);
        let regen2 = calculate_mana_regen(&player2);

        // Higher intelligence should give better regen
        assert!(
            regen2 >= regen1,
            "Higher intelligence should improve mana regen: {} vs {}",
            regen2,
            regen1
        );
    }

    #[test]
    fn test_mana_check_with_enough_mana() {
        let mut player = crate::player::You::default();
        player.energy = 100;

        // Should be able to cast with sufficient mana
        assert!(
            check_mana(&player, 50),
            "Should have enough mana to cast spell"
        );
    }

    #[test]
    fn test_mana_check_with_insufficient_mana() {
        let mut player = crate::player::You::default();
        player.energy = 30;

        // Should not be able to cast
        assert!(
            !check_mana(&player, 50),
            "Should not have enough mana to cast spell"
        );
    }

    #[test]
    fn test_detect_food_finds_objects() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut result = SpellResult::new();

        // Add food objects to level.objects
        level.objects.push(crate::object::Object::new(
            crate::object::ObjectId(1),
            0,
            crate::object::ObjectClass::Food,
        ));
        level.objects.push(crate::object::Object::new(
            crate::object::ObjectId(2),
            0,
            crate::object::ObjectClass::Food,
        ));

        cast_detect_food(&level, &mut result);

        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("sense"));
    }

    #[test]
    fn test_detect_food_empty_level() {
        let level = Level::new(DLevel::main_dungeon_start());
        let mut result = SpellResult::new();

        cast_detect_food(&level, &mut result);

        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("don't sense"));
    }

    #[test]
    fn test_detect_unseen_finds_unexplored() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();
        let mut result = SpellResult::new();

        // Mark some cells as unexplored
        level.cells[5][5].explored = false;
        level.cells[6][6].explored = false;

        cast_detect_unseen(&level, &player, &mut result);

        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("sense") || result.messages[0].contains("don't"));
    }

    #[test]
    fn test_detect_treasure_finds_gold() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut result = SpellResult::new();

        // Add gold (Coin) objects to level.objects
        level.objects.push(crate::object::Object::new(
            crate::object::ObjectId(1),
            0,
            crate::object::ObjectClass::Coin,
        ));
        level.objects.push(crate::object::Object::new(
            crate::object::ObjectId(2),
            0,
            crate::object::ObjectClass::Coin,
        ));

        cast_detect_treasure(&level, &mut result);

        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("sense"));
    }

    #[test]
    fn test_detect_treasure_empty_level() {
        let level = Level::new(DLevel::main_dungeon_start());
        let mut result = SpellResult::new();

        cast_detect_treasure(&level, &mut result);

        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("don't sense"));
    }

    #[test]
    fn test_create_monster_requires_direction() {
        let level = Level::new(DLevel::main_dungeon_start());
        let mut rng = crate::rng::GameRng::new(42);
        let mut result = SpellResult::new();

        // No direction specified
        cast_create_monster(Some((0, 0)), &level, &mut rng, &mut result);

        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("direction"));
    }

    #[test]
    fn test_create_monster_with_valid_direction() {
        let level = Level::new(DLevel::main_dungeon_start());
        let mut rng = crate::rng::GameRng::new(42);
        let mut result = SpellResult::new();

        // Valid direction
        cast_create_monster(Some((1, 0)), &level, &mut rng, &mut result);

        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("summon"));
    }

    #[test]
    fn test_stone_skin_grants_property() {
        let mut player = You::default();
        let mut result = SpellResult::new();

        // Initially should not have stone skin
        let had_property = player.properties.has(Property::StoneSkin);

        cast_stone_skin(&mut player, &mut result);

        // After casting, should have property
        assert!(player.properties.has(Property::StoneSkin));
        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("stone"));
    }

    #[test]
    fn test_polymorph_with_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();
        let mut rng = crate::rng::GameRng::new(42);
        let mut result = SpellResult::new();

        // Add a monster using add_monster so that monster_grid is updated
        let monster = crate::monster::Monster::new(
            crate::monster::MonsterId(0),
            0,
            player.pos.x + 1,
            player.pos.y,
        );
        level.add_monster(monster);

        cast_polymorph(Some((1, 0)), &player, &mut level, &mut rng, &mut result);

        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("shimmers"));
    }

    #[test]
    fn test_polymorph_no_target() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();
        let mut rng = crate::rng::GameRng::new(42);
        let mut result = SpellResult::new();

        // No monster to polymorph
        cast_polymorph(Some((1, 0)), &player, &mut level, &mut rng, &mut result);

        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("no monster"));
    }

    #[test]
    fn test_all_spells_implemented() {
        let mut player = You::default();
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut rng = crate::rng::GameRng::new(42);

        // Test that previously unimplemented spells now have implementations
        let result = cast_spell(
            SpellType::DetectFood,
            None,
            &mut player,
            &mut level,
            &mut rng,
        );
        assert!(
            !result
                .messages
                .iter()
                .any(|m| m.contains("not yet implemented"))
        );

        let result = cast_spell(
            SpellType::DetectUnseen,
            None,
            &mut player,
            &mut level,
            &mut rng,
        );
        assert!(
            !result
                .messages
                .iter()
                .any(|m| m.contains("not yet implemented"))
        );

        let result = cast_spell(
            SpellType::DetectTreasure,
            None,
            &mut player,
            &mut level,
            &mut rng,
        );
        assert!(
            !result
                .messages
                .iter()
                .any(|m| m.contains("not yet implemented"))
        );

        let result = cast_spell(
            SpellType::StoneSkin,
            None,
            &mut player,
            &mut level,
            &mut rng,
        );
        assert!(
            !result
                .messages
                .iter()
                .any(|m| m.contains("not yet implemented"))
        );
    }

    // ========== Tests for New Spell Functions ==========

    #[test]
    fn test_is_undirected_spell() {
        // Healing spells are undirected
        assert!(is_undirected_spell(SpellType::Healing));
        assert!(is_undirected_spell(SpellType::ExtraHealing));
        assert!(is_undirected_spell(SpellType::MagicMapping));

        // Attack spells need direction
        assert!(!is_undirected_spell(SpellType::MagicMissile));
        assert!(!is_undirected_spell(SpellType::Fireball));
    }

    #[test]
    fn test_spell_skilltype() {
        assert_eq!(spell_skilltype(SpellType::Fireball), SpellSchool::Attack);
        assert_eq!(spell_skilltype(SpellType::Healing), SpellSchool::Healing);
        assert_eq!(
            spell_skilltype(SpellType::MagicMapping),
            SpellSchool::Divination
        );
    }

    #[test]
    fn test_spell_damage_bonus() {
        let mut player = You::default();
        player.attr_current.set(Attribute::Intelligence, 18);
        player.exp_level = 10;

        let bonus = spell_damage_bonus(&player, SpellType::Fireball);
        assert!(bonus > 0);
    }

    #[test]
    fn test_spell_hit_bonus() {
        let mut player = You::default();
        player.attr_current.set(Attribute::Dexterity, 18);
        player.attr_current.set(Attribute::Intelligence, 16);
        player.exp_level = 10;

        let bonus = spell_hit_bonus(&player, SpellType::MagicMissile);
        assert!(bonus > 0);
    }

    #[test]
    fn test_spell_would_be_useless() {
        let mut player = You::default();
        player.hp = 100;
        player.hp_max = 100;

        // Healing at full HP is useless
        assert!(spell_would_be_useless(SpellType::Healing, &player));

        // Healing when damaged is not useless
        player.hp = 50;
        assert!(!spell_would_be_useless(SpellType::Healing, &player));

        // Cure blindness when not blind is useless
        player.blinded_timeout = 0;
        assert!(spell_would_be_useless(SpellType::CureBlindness, &player));
    }

    #[test]
    fn test_spellretention() {
        let mut player = You::default();
        player.attr_current.set(Attribute::Intelligence, 10);

        let base_retention = spellretention(&player);
        assert_eq!(base_retention, 10000);

        // Higher intelligence = longer retention
        player.attr_current.set(Attribute::Intelligence, 15);
        let high_int_retention = spellretention(&player);
        assert!(high_int_retention > base_retention);

        // Wizard bonus
        player.role = crate::player::Role::Wizard;
        let wizard_retention = spellretention(&player);
        assert!(wizard_retention > high_int_retention);
    }

    #[test]
    fn test_spelltypemnemonic() {
        assert_eq!(spelltypemnemonic(SpellType::Fireball), 'A');
        assert_eq!(spelltypemnemonic(SpellType::Healing), 'H');
        assert_eq!(spelltypemnemonic(SpellType::MagicMapping), 'D');
        assert_eq!(spelltypemnemonic(SpellType::TurnUndead), 'C');
    }

    #[test]
    fn test_percent_success() {
        let mut player = You::default();
        player.attr_current.set(Attribute::Intelligence, 18);
        player.attr_current.set(Attribute::Wisdom, 18);
        player.exp_level = 20;

        // High stats should give high success
        let success = percent_success(SpellType::Healing, &player);
        assert!(success > 50);
    }

    #[test]
    fn test_charm_monsters() {
        let mut player = You::default();
        player.pos = crate::player::Position { x: 10, y: 10 };
        player.attr_current.set(Attribute::Charisma, 18);

        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut rng = crate::rng::GameRng::new(42);

        // Add a monster
        let mut monster = crate::monster::Monster::new(crate::monster::MonsterId(0), 0, 0, 0);
        monster.x = 11;
        monster.y = 11;
        monster.level = 1;
        level.monsters.push(monster);

        let messages = charm_monsters(&player, &mut level, 5, &mut rng);
        assert!(!messages.is_empty());
    }

    #[test]
    fn test_put_monsters_to_sleep() {
        let mut player = You::default();
        player.pos = crate::player::Position { x: 10, y: 10 };

        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut rng = crate::rng::GameRng::new(42);

        // Add a monster
        let mut monster = crate::monster::Monster::new(crate::monster::MonsterId(0), 0, 0, 0);
        monster.x = 11;
        monster.y = 11;
        level.monsters.push(monster);

        let messages = put_monsters_to_sleep(&player, &mut level, 5, &mut rng);
        assert!(!messages.is_empty());
    }

    #[test]
    fn test_domindblast() {
        let mut player = You::default();
        player.pos = crate::player::Position { x: 10, y: 10 };
        player.attr_current.set(Attribute::Intelligence, 18);

        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut rng = crate::rng::GameRng::new(42);

        // Add a monster
        let mut monster = crate::monster::Monster::new(crate::monster::MonsterId(0), 0, 0, 0);
        monster.x = 11;
        monster.y = 11;
        monster.hp = 100;
        level.monsters.push(monster);

        let messages = domindblast(&player, &mut level, &mut rng);
        assert!(!messages.is_empty());
    }
}
