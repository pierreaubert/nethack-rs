//! Spellcasting system (spell.c)
//!
//! Handles learning and casting spells.

use crate::dungeon::Level;
use crate::monster::MonsterId;
use crate::player::{Property, You};
use crate::rng::GameRng;

/// Spell schools
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

/// Spell type indices (matching spellbook ObjectType in nh-data/objects.rs)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            SpellType::MagicMissile
            | SpellType::Confuse
            | SpellType::Sleep
            | SpellType::Haste => 3,
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
}

/// A known spell
#[derive(Debug, Clone)]
pub struct KnownSpell {
    /// The spell type
    pub spell_type: SpellType,
    /// Turns until spell memory fades
    pub turns_remaining: u32,
}

impl KnownSpell {
    pub fn new(spell_type: SpellType) -> Self {
        // Spell memory lasts ~20000 turns
        Self {
            spell_type,
            turns_remaining: 20000,
        }
    }

    pub fn is_forgotten(&self) -> bool {
        self.turns_remaining == 0
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
        result.messages.push("You are too confused to cast the spell!".to_string());
        return result;
    }

    // Pay energy cost
    player.energy -= energy_cost;

    let mut result = SpellResult::new();
    result.energy_cost = energy_cost;

    match spell {
        SpellType::ForceBolt => cast_force_bolt(direction, player, level, rng, &mut result),
        SpellType::MagicMissile => cast_magic_missile(direction, player, level, rng, &mut result),
        SpellType::Fireball => cast_fireball(direction, player, level, rng, &mut result),
        SpellType::ConeOfCold => cast_cone_of_cold(direction, player, level, rng, &mut result),
        SpellType::FingerOfDeath => cast_finger_of_death(direction, player, level, rng, &mut result),
        SpellType::Drain => cast_drain(direction, player, level, rng, &mut result),
        SpellType::Healing => cast_healing(player, rng, &mut result),
        SpellType::ExtraHealing => cast_extra_healing(player, rng, &mut result),
        SpellType::CureBlindness => cast_cure_blindness(player, &mut result),
        SpellType::CureSickness => cast_cure_sickness(player, &mut result),
        SpellType::RestoreAbility => cast_restore_ability(player, &mut result),
        SpellType::DetectMonsters => cast_detect_monsters(level, &mut result),
        SpellType::MagicMapping => cast_magic_mapping(level, &mut result),
        SpellType::Identify => cast_identify(&mut result),
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
        result.messages.push("The force bolt flies harmlessly.".to_string());
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
            result.messages.push("The magic missile hits a wall.".to_string());
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

    result.messages.push("A cone of cold spreads out!".to_string());

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
            result.messages.push("The death ray hits a wall.".to_string());
            break;
        }

        if let Some(monster) = level.monster_at_mut(x, y) {
            // Disintegration resistance or high level monsters can resist death magic
            let resist_chance = (monster.level as u32) * 3;
            if monster.resists_disint() || rng.percent(resist_chance) {
                result.messages.push(format!(
                    "The {} resists the death magic!",
                    monster.name
                ));
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
        result.messages.push("There's nothing there to drain.".to_string());
    }
}

fn cast_healing(player: &mut You, rng: &mut GameRng, result: &mut SpellResult) {
    let heal = rng.dice(4, 4) as i32;
    player.hp = (player.hp + heal).min(player.hp_max);
    result.messages.push(format!("You feel better. (+{} HP)", heal));
}

fn cast_extra_healing(player: &mut You, rng: &mut GameRng, result: &mut SpellResult) {
    let heal = rng.dice(6, 8) as i32;
    player.hp = (player.hp + heal).min(player.hp_max);
    player.blinded_timeout = 0;
    player.confused_timeout = 0;
    result.messages.push(format!("You feel much better. (+{} HP)", heal));
}

fn cast_cure_blindness(player: &mut You, result: &mut SpellResult) {
    if player.blinded_timeout > 0 {
        player.blinded_timeout = 0;
        result.messages.push("Your vision clears!".to_string());
    } else {
        result.messages.push("You have a brief moment of clarity.".to_string());
    }
}

fn cast_cure_sickness(player: &mut You, result: &mut SpellResult) {
    // TODO: Track sickness status properly
    result.messages.push("You feel healthier.".to_string());
    let _ = player;
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
        result.messages.push(format!(
            "You sense {} monster(s) on this level.",
            count
        ));
    }
}

fn cast_magic_mapping(level: &mut Level, result: &mut SpellResult) {
    for x in 0..crate::COLNO {
        for y in 0..crate::ROWNO {
            level.cells[x][y].explored = true;
        }
    }
    result.messages.push("A map coalesces in your mind!".to_string());
}

fn cast_identify(result: &mut SpellResult) {
    result.messages.push("You may identify an item.".to_string());
    // TODO: Open item selection menu
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
    result.messages.push("You have a vision of your surroundings.".to_string());
}

fn cast_haste(player: &mut You, rng: &mut GameRng, result: &mut SpellResult) {
    let duration = rng.dice(5, 10);
    player.properties.set_timeout(Property::Speed, duration);
    result.messages.push("You feel yourself speeding up!".to_string());
}

fn cast_invisibility(player: &mut You, rng: &mut GameRng, result: &mut SpellResult) {
    let duration = rng.dice(10, 10);
    player.properties.set_timeout(Property::Invisibility, duration);
    result.messages.push("You vanish!".to_string());
}

fn cast_levitation(player: &mut You, rng: &mut GameRng, result: &mut SpellResult) {
    let duration = rng.dice(10, 10);
    player.properties.set_timeout(Property::Levitation, duration);
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
        result.messages.push(format!("The {} looks confused!", monster.name));
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
        result.messages.push(format!("The {} slows down!", monster.name));
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
                result.messages.push(format!("The {} resists!", monster.name));
            } else {
                monster.state.sleeping = true;
                monster.sleep_timeout = rng.dice(4, 6) as u16;
                result.messages.push(format!("The {} falls asleep!", monster.name));
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
            result.messages.push("The door is wizard-locked!".to_string());
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
        result.messages.push(format!("The {} shudders!", monster.name));
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
        result.messages.push(format!("{} monster(s) turn and flee!", turned));
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
}

impl MonsterSpellResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            player_damage: 0,
            player_died: false,
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

    result.messages.push(format!(
        "The {} casts {}!",
        caster.name,
        spell.name()
    ));

    match spell {
        MonsterSpell::MagicMissile => {
            if player.properties.has(Property::MagicResistance) {
                result.messages.push("The missiles bounce off!".to_string());
            } else {
                let damage = rng.dice(2, 6) as i32 + (caster.level / 2) as i32;
                player.hp -= damage;
                result.player_damage = damage;
                result.messages.push(format!("You are hit by magic missiles! ({} damage)", damage));
            }
        }
        MonsterSpell::Fireball => {
            if player.properties.has(Property::FireResistance) {
                result.messages.push("You are unaffected by the fire.".to_string());
            } else {
                let damage = rng.dice(6, 6) as i32;
                player.hp -= damage;
                result.player_damage = damage;
                result.messages.push(format!("You are engulfed in flames! ({} damage)", damage));
            }
        }
        MonsterSpell::ConeOfCold => {
            if player.properties.has(Property::ColdResistance) {
                result.messages.push("You are unaffected by the cold.".to_string());
            } else {
                let damage = rng.dice(6, 6) as i32;
                player.hp -= damage;
                result.player_damage = damage;
                result.messages.push(format!("You are frozen! ({} damage)", damage));
            }
        }
        MonsterSpell::Lightning => {
            if player.properties.has(Property::ShockResistance) {
                result.messages.push("You are unaffected by the lightning.".to_string());
            } else {
                let damage = rng.dice(6, 6) as i32;
                player.hp -= damage;
                result.player_damage = damage;
                result.messages.push(format!("You are struck by lightning! ({} damage)", damage));
            }
        }
        MonsterSpell::Sleep => {
            if player.properties.has(Property::SleepResistance) {
                result.messages.push("You resist the sleep spell.".to_string());
            } else {
                let duration = rng.dice(2, 6) as u16;
                player.sleeping_timeout = player.sleeping_timeout.saturating_add(duration);
                result.messages.push("You feel very drowsy...".to_string());
            }
        }
        MonsterSpell::FingerOfDeath => {
            if player.properties.has(Property::MagicResistance) {
                result.messages.push("You resist the death magic!".to_string());
            } else if rng.percent(50) {
                // 50% chance to resist even without MR
                result.messages.push("You feel drained but survive.".to_string());
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
                result.messages.push("You resist the paralysis.".to_string());
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
                result.messages.push("You feel your life force draining away!".to_string());
            }
        }
        MonsterSpell::Healing | MonsterSpell::CureBlindness => {
            // These are self-healing spells for the monster
            result.messages.push(format!("The {} looks healthier.", caster.name));
        }
        MonsterSpell::Haste | MonsterSpell::Invisibility => {
            // Self-buff spells
            result.messages.push(format!("The {} seems faster.", caster.name));
        }
        MonsterSpell::Summon => {
            // Summon allies - would need to create new monsters
            result.messages.push("Monsters appear around you!".to_string());
        }
        MonsterSpell::Teleport => {
            // Teleport self away
            result.messages.push(format!("The {} vanishes!", caster.name));
        }
        MonsterSpell::TeleportAway => {
            // Teleport player away
            if player.properties.has(Property::TeleportControl) && rng.one_in(3) {
                result.messages.push("You resist the teleportation.".to_string());
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
}
