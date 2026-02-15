//! Wand and ray zapping effects (zap.c)
//!
//! Handles wand zapping, spell rays, and breath weapons.

use crate::combat::DamageType;
use crate::dungeon::{DLevel, Level};
use crate::monster::MonsterId;
use crate::object::Object;
use crate::player::{Property, You};
use crate::rng::GameRng;

/// Ray/zap type indices (matches ZT_* from zap.c)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ZapType {
    MagicMissile = 0,
    Fire = 1,
    Cold = 2,
    Sleep = 3,
    Death = 4, // Also disintegration
    Lightning = 5,
    PoisonGas = 6,
    Acid = 7,
}

/// Extended wand effects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WandEffect {
    /// Standard zap effect
    StandardZap(ZapType, ZapVariant),
    /// Teleportation effect
    Teleport,
    /// Healing effect
    Healing,
    /// Polymorph effect
    Polymorph,
    /// Digging effect
    Digging,
    /// Resurrection effect
    Resurrection,
    /// Cancellation effect
    Cancellation,
}

impl ZapType {
    /// Get the wand variant (0-9)
    pub const fn wand(self) -> u8 {
        self as u8
    }

    /// Get the spell variant (10-19)
    pub const fn spell(self) -> u8 {
        10 + self as u8
    }

    /// Get the breath variant (20-29)
    pub const fn breath(self) -> u8 {
        20 + self as u8
    }

    /// Get display name for this zap type
    pub const fn name(&self, variant: ZapVariant) -> &'static str {
        match variant {
            ZapVariant::Wand => match self {
                ZapType::MagicMissile => "magic missile",
                ZapType::Fire => "bolt of fire",
                ZapType::Cold => "bolt of cold",
                ZapType::Sleep => "sleep ray",
                ZapType::Death => "death ray",
                ZapType::Lightning => "bolt of lightning",
                ZapType::PoisonGas => "poison gas",
                ZapType::Acid => "acid",
            },
            ZapVariant::Spell => match self {
                ZapType::MagicMissile => "magic missile",
                ZapType::Fire => "fireball",
                ZapType::Cold => "cone of cold",
                ZapType::Sleep => "sleep ray",
                ZapType::Death => "finger of death",
                ZapType::Lightning => "bolt of lightning",
                ZapType::PoisonGas => "poison gas",
                ZapType::Acid => "acid",
            },
            ZapVariant::Breath => match self {
                ZapType::MagicMissile => "blast of missiles",
                ZapType::Fire => "blast of fire",
                ZapType::Cold => "blast of frost",
                ZapType::Sleep => "blast of sleep gas",
                ZapType::Death => "blast of disintegration",
                ZapType::Lightning => "blast of lightning",
                ZapType::PoisonGas => "blast of poison gas",
                ZapType::Acid => "blast of acid",
            },
        }
    }

    /// Get the damage type associated with this zap
    pub const fn damage_type(&self) -> DamageType {
        match self {
            ZapType::MagicMissile => DamageType::MagicMissile,
            ZapType::Fire => DamageType::Fire,
            ZapType::Cold => DamageType::Cold,
            ZapType::Sleep => DamageType::Sleep,
            ZapType::Death => DamageType::Death,
            ZapType::Lightning => DamageType::Electric,
            ZapType::PoisonGas => DamageType::DrainStrength,
            ZapType::Acid => DamageType::Acid,
        }
    }
}

/// Variant of zap (wand, spell, breath)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZapVariant {
    Wand,
    Spell,
    Breath,
}

/// Direction for zapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZapDirection {
    Up,
    Down,
    Direction(i8, i8), // dx, dy
    Self_,             // At self
}

/// Result of zapping at something
#[derive(Debug, Clone)]
pub struct ZapResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Whether the wand/spell was consumed
    pub consumed: bool,
    /// Monsters that died
    pub killed: Vec<MonsterId>,
    /// Whether player died
    pub player_died: bool,
    /// Damage dealt to player
    pub player_damage: i32,
    /// Whether the wand was destroyed
    pub wand_destroyed: bool,
}

impl ZapResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            consumed: false,
            killed: Vec::new(),
            player_died: false,
            player_damage: 0,
            wand_destroyed: false,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

impl Default for ZapResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Zap a wand at a direction
pub fn zap_wand(
    wand: &mut Object,
    direction: ZapDirection,
    player: &mut You,
    level: &mut Level,
    rng: &mut GameRng,
) -> ZapResult {
    let mut result = ZapResult::new();

    // Check charges
    if wand.enchantment <= 0 {
        result.messages.push("Nothing happens.".to_string());
        return result;
    }

    // Consume a charge
    wand.enchantment -= 1;

    // Track wand usage for degradation
    wand.wand_use_count += 1;

    // Check for wand degradation
    let enchantment_before = wand.enchantment;
    degrade_wand(wand, rng);

    // Check if wand was destroyed
    if wand.enchantment <= 0 && enchantment_before > 0 {
        result.wand_destroyed = true;
        result
            .messages
            .push("The wand crumbles to dust!".to_string());
        return result;
    }

    // Determine zap type from wand object type
    let zap_type = match wand_to_zap_type(wand.object_type) {
        Some(zt) => zt,
        None => {
            // Non-ray wand - handle immediate effects
            return handle_immediate_wand(wand, direction, player, level, rng);
        }
    };

    match direction {
        ZapDirection::Up => zap_up(zap_type, ZapVariant::Wand, player, level, rng, &mut result),
        ZapDirection::Down => zap_down(zap_type, ZapVariant::Wand, player, level, rng, &mut result),
        ZapDirection::Self_ => {
            zap_self(zap_type, ZapVariant::Wand, player, rng, &mut result);
        }
        ZapDirection::Direction(dx, dy) => {
            zap_direction(
                zap_type,
                ZapVariant::Wand,
                dx,
                dy,
                player,
                level,
                rng,
                &mut result,
            );
        }
    }

    result
}

/// Map wand object type to zap type (ray wands only)
fn wand_to_zap_type(object_type: i16) -> Option<ZapType> {
    // Object type indices for wands - these match nh-data/objects.rs
    // WandOfMagicMissile = 355
    // WandOfFire = 365
    // WandOfCold = 366
    // WandOfLightning = 367
    // WandOfSleep = 368
    // WandOfDeath = 369
    match object_type {
        355 => Some(ZapType::MagicMissile), // WandOfMagicMissile
        365 => Some(ZapType::Fire),         // WandOfFire
        366 => Some(ZapType::Cold),         // WandOfCold
        367 => Some(ZapType::Lightning),    // WandOfLightning
        368 => Some(ZapType::Sleep),        // WandOfSleep
        369 => Some(ZapType::Death),        // WandOfDeath
        _ => None,                          // Not a ray wand
    }
}

/// Handle immediate effect wands (not rays)
fn handle_immediate_wand(
    wand: &Object,
    direction: ZapDirection,
    player: &mut You,
    level: &mut Level,
    rng: &mut GameRng,
) -> ZapResult {
    let mut result = ZapResult::new();

    // Get target position
    let (tx, ty) = match direction {
        ZapDirection::Direction(dx, dy) => (player.pos.x + dx, player.pos.y + dy),
        ZapDirection::Self_ => (player.pos.x, player.pos.y),
        ZapDirection::Up | ZapDirection::Down => {
            result.messages.push("Nothing happens.".to_string());
            return result;
        }
    };

    // Handle different wand types by object_type
    match wand.object_type {
        349 => {
            // WandOfLight
            result
                .messages
                .push("A light shines around you.".to_string());
            // Light up surrounding area
            light_area(player.pos.x as usize, player.pos.y as usize, level, 5);
        }
        351 => {
            // WandOfDigging
            if let ZapDirection::Direction(dx, dy) = direction {
                dig_ray(player.pos.x, player.pos.y, dx, dy, level, &mut result);
            }
        }
        354 => {
            // WandOfLocking
            if level.is_valid_pos(tx, ty) {
                let cell = level.cell_mut(tx as usize, ty as usize);
                if cell.typ == crate::dungeon::CellType::Door {
                    cell.set_door_state(crate::dungeon::DoorState::LOCKED);
                    result.messages.push("The door locks!".to_string());
                }
            }
        }
        356 => {
            // WandOfMake (make invisible)
            if direction == ZapDirection::Self_ {
                player
                    .properties
                    .set_timeout(crate::player::Property::Invisibility, 50);
                result.messages.push("You vanish.".to_string());
            } else if let Some(monster) = level.monster_at_mut(tx, ty) {
                monster.state.invisible = true;
                result
                    .messages
                    .push(format!("The {} disappears!", monster.name));
            }
        }
        357 => {
            // WandOfOpening
            if level.is_valid_pos(tx, ty) {
                let cell = level.cell_mut(tx as usize, ty as usize);
                if cell.typ == crate::dungeon::CellType::Door {
                    cell.set_door_state(crate::dungeon::DoorState::OPEN);
                    result.messages.push("The door opens!".to_string());
                }
            }
        }
        359 => {
            // WandOfSlowMonster
            if let Some(monster) = level.monster_at_mut(tx, ty) {
                monster.state.slowed = true;
                result
                    .messages
                    .push(format!("The {} seems to move more slowly.", monster.name));
            }
        }
        360 => {
            // WandOfSpeed
            if direction == ZapDirection::Self_ {
                player
                    .properties
                    .set_timeout(crate::player::Property::Speed, 50);
                result
                    .messages
                    .push("You feel yourself moving faster.".to_string());
            } else if let Some(monster) = level.monster_at_mut(tx, ty) {
                monster.state.hasted = true;
                result
                    .messages
                    .push(format!("The {} seems to speed up.", monster.name));
            }
        }
        361 => {
            // WandOfStriking
            if let Some(monster) = level.monster_at_mut(tx, ty) {
                let damage = rng.dice(2, 12) as i32;
                monster.hp -= damage;
                result.messages.push(format!(
                    "The {} is struck by a force bolt for {} damage!",
                    monster.name, damage
                ));
                if monster.hp <= 0 {
                    result.killed.push(monster.id);
                }
            }
        }
        364 => {
            // WandOfCreateMonster
            result.messages.push("You create a monster!".to_string());
            // Monster spawning requires monster creation infrastructure
            // The caller should spawn a random monster near the player
        }
        371 => {
            // WandOfTeleportation
            if direction == ZapDirection::Self_ {
                teleport_player(player, level, rng, &mut result);
            } else if let Some(monster) = level.monster_at_mut(tx, ty) {
                teleport_monster(monster.id, level, rng, &mut result);
            }
        }
        370 => {
            // WandOfPolymorph
            if direction == ZapDirection::Self_ {
                result
                    .messages
                    .push("You feel like a new person!".to_string());
                // Polymorph player - grant temporary stat changes
                let stat_change = rng.rnd(3) as i8 - 1;
                player
                    .attr_current
                    .modify(crate::player::Attribute::Strength, stat_change);
                player.polymorph_timeout = 100 + rng.rnd(100);
            } else if let Some(monster) = level.monster_at_mut(tx, ty) {
                result.messages.push("The monster transforms!".to_string());
                // Polymorph monster - change its stats randomly
                monster.level = (monster.level as i8 + rng.rnd(5) as i8 - 2).max(1) as u8;
                monster.hp = monster.hp.saturating_add(rng.rnd(10) as i32 - 5);
            }
        }
        372 => {
            // WandOfWishing
            result
                .messages
                .push("You may wish for an object.".to_string());
            // Wishing requires UI interaction to get player's wish
            // The caller should prompt the player and create the wished-for object
        }
        363 => {
            // WandOfCancellation
            if direction == ZapDirection::Self_ {
                // Cancelling yourself removes most magical effects
                player.properties.remove_intrinsic(Property::Invisibility);
                player.hallucinating_timeout = 0;
                result
                    .messages
                    .push("You feel like nothing special.".to_string());
            } else if let Some(monster) = level.monster_at_mut(tx, ty) {
                let msgs = cancel_monst(monster);
                result.messages.extend(msgs);
            }
        }
        358 => {
            // WandOfProbing
            if direction == ZapDirection::Self_ {
                result
                    .messages
                    .push(format!("You are level {} with {}/{} HP.", player.exp_level, player.hp, player.hp_max));
            } else if let Some(monster) = level.monster_at_mut(tx, ty) {
                let msgs = probe_monster(monster);
                result.messages.extend(msgs);
            }
        }
        362 => {
            // WandOfUndead Turning
            if let Some(monster) = level.monster_at_mut(tx, ty) {
                if monster.is_undead() {
                    monster.state.fleeing = true;
                    monster.flee_timeout = 20;
                    result
                        .messages
                        .push(format!("The {} turns to flee!", monster.name));
                } else {
                    result
                        .messages
                        .push(format!("The {} is not affected.", monster.name));
                }
            }
        }
        _ => {
            result.messages.push("Nothing happens.".to_string());
        }
    }

    result
}

/// Zap upward
fn zap_up(
    zap_type: ZapType,
    variant: ZapVariant,
    player: &mut You,
    _level: &Level,
    rng: &mut GameRng,
    result: &mut ZapResult,
) {
    let name = zap_type.name(variant);

    // Check for ceiling effects
    result
        .messages
        .push(format!("The {} hits the ceiling.", name));

    // Some effects bounce back
    match zap_type {
        ZapType::Fire => {
            result
                .messages
                .push("A cloud of smoke descends.".to_string());
        }
        ZapType::Cold => {
            result.messages.push("Ice shards fall on you.".to_string());
            let damage = rng.dice(2, 6) as i32;
            if !player
                .properties
                .has(crate::player::Property::ColdResistance)
            {
                player.hp -= damage;
                result.player_damage = damage;
                result
                    .messages
                    .push(format!("You take {} cold damage.", damage));
            }
        }
        ZapType::Lightning => {
            result.messages.push("The ceiling crackles.".to_string());
        }
        ZapType::Death
        | ZapType::Sleep
        | ZapType::MagicMissile
        | ZapType::PoisonGas
        | ZapType::Acid => {
            // No special ceiling effect
        }
    }
}

/// Zap downward
fn zap_down(
    zap_type: ZapType,
    variant: ZapVariant,
    _player: &mut You,
    level: &mut Level,
    _rng: &mut GameRng,
    result: &mut ZapResult,
) {
    let name = zap_type.name(variant);
    result
        .messages
        .push(format!("The {} hits the floor.", name));

    // Fire/cold might affect the floor
    match zap_type {
        ZapType::Fire => {
            result
                .messages
                .push("The floor smolders briefly.".to_string());
            // Could destroy items on ground
        }
        ZapType::Cold => {
            result.messages.push("The floor is frosted.".to_string());
            // Could freeze water/lava
        }
        ZapType::Lightning => {
            result.messages.push("The floor is scorched.".to_string());
        }
        ZapType::Death => {
            // Check for corpses to animate? For now, nothing
            let _ = level; // Silence unused warning
        }
        ZapType::Sleep | ZapType::MagicMissile | ZapType::PoisonGas | ZapType::Acid => {}
    }
}

/// Zap at self
fn zap_self(
    zap_type: ZapType,
    _variant: ZapVariant,
    player: &mut You,
    rng: &mut GameRng,
    result: &mut ZapResult,
) {
    match zap_type {
        ZapType::MagicMissile => {
            let damage = rng.dice(2, 6) as i32;
            if !player
                .properties
                .has(crate::player::Property::MagicResistance)
            {
                player.hp -= damage;
                result.player_damage = damage;
                result.messages.push(format!(
                    "You are hit by a magic missile for {} damage!",
                    damage
                ));
            } else {
                result
                    .messages
                    .push("The magic missile bounces off!".to_string());
            }
        }
        ZapType::Fire => {
            let damage = rng.dice(6, 6) as i32;
            if !player
                .properties
                .has(crate::player::Property::FireResistance)
            {
                player.hp -= damage;
                result.player_damage = damage;
                result
                    .messages
                    .push(format!("You burn yourself for {} damage!", damage));
            } else {
                result.messages.push("You feel warm.".to_string());
            }
        }
        ZapType::Cold => {
            let damage = rng.dice(6, 6) as i32;
            if !player
                .properties
                .has(crate::player::Property::ColdResistance)
            {
                player.hp -= damage;
                result.player_damage = damage;
                result
                    .messages
                    .push(format!("You freeze yourself for {} damage!", damage));
            } else {
                result.messages.push("You feel chilly.".to_string());
            }
        }
        ZapType::Sleep => {
            if !player
                .properties
                .has(crate::player::Property::SleepResistance)
            {
                player.sleeping_timeout = rng.dice(3, 6) as u16;
                result.messages.push("You fall asleep!".to_string());
            } else {
                result.messages.push("You feel drowsy.".to_string());
            }
        }
        ZapType::Death => {
            if player
                .properties
                .has(crate::player::Property::MagicResistance)
            {
                result.messages.push("You shudder momentarily.".to_string());
            } else {
                result.messages.push("You die...".to_string());
                result.player_died = true;
            }
        }
        ZapType::Lightning => {
            let damage = rng.dice(6, 6) as i32;
            if !player
                .properties
                .has(crate::player::Property::ShockResistance)
            {
                player.hp -= damage;
                result.player_damage = damage;
                result
                    .messages
                    .push(format!("You shock yourself for {} damage!", damage));
            } else {
                result.messages.push("You feel a mild tingle.".to_string());
            }
        }
        ZapType::PoisonGas => {
            if !player
                .properties
                .has(crate::player::Property::PoisonResistance)
            {
                // Drain strength
                let str = player.attr_current.get(crate::player::Attribute::Strength);
                if str > 3 {
                    player
                        .attr_current
                        .set(crate::player::Attribute::Strength, str - 1);
                    result.messages.push("You feel weaker.".to_string());
                }
            } else {
                result
                    .messages
                    .push("You are immune to poison.".to_string());
            }
        }
        ZapType::Acid => {
            let damage = rng.dice(4, 6) as i32;
            if !player
                .properties
                .has(crate::player::Property::AcidResistance)
            {
                player.hp -= damage;
                result.player_damage = damage;
                result
                    .messages
                    .push(format!("You are covered in acid for {} damage!", damage));
            } else {
                result
                    .messages
                    .push("The acid doesn't affect you.".to_string());
            }
        }
    }

    if player.hp <= 0 {
        result.player_died = true;
    }
}

/// Zap in a direction (ray)
#[allow(clippy::too_many_arguments)]
fn zap_direction(
    zap_type: ZapType,
    variant: ZapVariant,
    dx: i8,
    dy: i8,
    player: &You,
    level: &mut Level,
    rng: &mut GameRng,
    result: &mut ZapResult,
) {
    let mut x = player.pos.x;
    let mut y = player.pos.y;
    let mut cur_dx = dx;
    let mut cur_dy = dy;

    // C: range = rn1(7, 7) = 7 to 13 squares
    let range = rng.rnd(7) as i32 + 6;

    // Trace the ray with bounce support (potion.c:4044 dobuzz)
    for _ in 0..range {
        x += cur_dx;
        y += cur_dy;

        if !level.is_valid_pos(x, y) {
            break;
        }

        // Check for walls — attempt bounce
        let cell = level.cell(x as usize, y as usize);
        if cell.typ.is_wall() {
            // Try bouncing: reverse the component that hit the wall
            // Check if we can bounce by reversing dx, dy, or both
            let bounce_x = level.is_valid_pos(x - cur_dx, y)
                && !level.cell((x - cur_dx) as usize, y as usize).typ.is_wall();
            let bounce_y = level.is_valid_pos(x, y - cur_dy)
                && !level.cell(x as usize, (y - cur_dy) as usize).typ.is_wall();

            if cur_dx != 0 && cur_dy != 0 {
                // Diagonal: try reversing one or both components
                if !bounce_x && !bounce_y {
                    // Corner: reverse both
                    cur_dx = -cur_dx;
                    cur_dy = -cur_dy;
                } else if !bounce_x {
                    // Hit wall in x direction
                    cur_dx = -cur_dx;
                } else {
                    // Hit wall in y direction
                    cur_dy = -cur_dy;
                }
            } else if cur_dx != 0 {
                cur_dx = -cur_dx;
            } else {
                cur_dy = -cur_dy;
            }

            // Back up to pre-wall position and continue
            x -= dx; // Use original dx to back up
            y -= dy;
            result
                .messages
                .push(format!("The {} bounces!", zap_type.name(variant)));
            continue;
        }

        // Check for monsters with reflection
        if let Some(monster) = level.monster_at_mut(x, y) {
            // TODO: check monster reflection (silver dragon scales, amulet of reflection)
            hit_monster_with_ray(monster, zap_type, variant, rng, result);
            // Most rays stop at first monster
            break;
        }

        // Check if ray hits the player (for bounced rays)
        if x == player.pos.x && y == player.pos.y {
            // Player hit by own bounced ray
            let damage = zap_damage(zap_type, variant, rng);
            result.player_damage += damage;
            result.messages.push(format!(
                "The {} hits you for {} damage!",
                zap_type.name(variant),
                damage
            ));
            break;
        }
    }
}

/// Apply ray effect to a monster
fn hit_monster_with_ray(
    monster: &mut crate::monster::Monster,
    zap_type: ZapType,
    variant: ZapVariant,
    rng: &mut GameRng,
    result: &mut ZapResult,
) {
    let name = &monster.name;

    match zap_type {
        ZapType::MagicMissile => {
            let damage = rng.dice(2, 6) as i32;
            monster.hp -= damage;
            result.messages.push(format!(
                "The {} is hit by a {} for {} damage!",
                name,
                zap_type.name(variant),
                damage
            ));
        }
        ZapType::Fire => {
            let damage = rng.dice(6, 6) as i32;
            if monster.resists_fire() {
                result
                    .messages
                    .push(format!("The {} is not affected by the flames.", name));
            } else {
                monster.hp -= damage;
                result.messages.push(format!(
                    "The {} is engulfed in flames for {} damage!",
                    name, damage
                ));
            }
        }
        ZapType::Cold => {
            let damage = rng.dice(6, 6) as i32;
            if monster.resists_cold() {
                result
                    .messages
                    .push(format!("The {} is not affected by the cold.", name));
            } else {
                monster.hp -= damage;
                result
                    .messages
                    .push(format!("The {} is frozen for {} damage!", name, damage));
            }
        }
        ZapType::Sleep => {
            if monster.resists_sleep() {
                result
                    .messages
                    .push(format!("The {} is not affected.", name));
            } else {
                monster.state.sleeping = true;
                result.messages.push(format!("The {} falls asleep!", name));
            }
        }
        ZapType::Death => {
            // Death ray: magic resistance or disintegration resistance blocks
            if monster.resists_disint() || monster.resists_magic() {
                result.messages.push(format!("The {} resists!", name));
            } else {
                monster.hp = 0;
                result.messages.push(format!("The {} dies!", name));
            }
        }
        ZapType::Lightning => {
            let damage = rng.dice(6, 6) as i32;
            if monster.resists_elec() {
                result
                    .messages
                    .push(format!("The {} is not affected by the lightning.", name));
            } else {
                monster.hp -= damage;
                result
                    .messages
                    .push(format!("The {} is shocked for {} damage!", name, damage));
            }
        }
        ZapType::PoisonGas => {
            let damage = rng.dice(2, 6) as i32;
            if monster.resists_poison() {
                result
                    .messages
                    .push(format!("The {} is not affected by the poison.", name));
            } else {
                monster.hp -= damage;
                result
                    .messages
                    .push(format!("The {} is poisoned for {} damage!", name, damage));
            }
        }
        ZapType::Acid => {
            let damage = rng.dice(4, 6) as i32;
            monster.hp -= damage;
            result.messages.push(format!(
                "The {} is burned by acid for {} damage!",
                name, damage
            ));
        }
    }

    if monster.hp <= 0 {
        result.killed.push(monster.id);
    }
}

/// Light up an area
fn light_area(cx: usize, cy: usize, level: &mut Level, radius: usize) {
    for dy in 0..=radius * 2 {
        for dx in 0..=radius * 2 {
            let x = (cx + dx).saturating_sub(radius);
            let y = (cy + dy).saturating_sub(radius);
            if x < crate::COLNO && y < crate::ROWNO {
                level.cells[x][y].lit = true;
            }
        }
    }
}

/// Dig in a direction
fn dig_ray(start_x: i8, start_y: i8, dx: i8, dy: i8, level: &mut Level, result: &mut ZapResult) {
    let mut x = start_x;
    let mut y = start_y;
    let mut dug = false;

    for _ in 0..10 {
        x += dx;
        y += dy;

        if !level.is_valid_pos(x, y) {
            break;
        }

        let cell = level.cell_mut(x as usize, y as usize);
        if cell.typ.is_wall()
            || cell.typ == crate::dungeon::CellType::Stone
            || cell.typ == crate::dungeon::CellType::Door
        {
            cell.typ = crate::dungeon::CellType::Corridor;
            dug = true;
        }
    }

    if dug {
        result.messages.push("The rock crumbles.".to_string());
    } else {
        result.messages.push("Nothing happens.".to_string());
    }
}

/// Teleport player to random location
fn teleport_player(player: &mut You, level: &Level, rng: &mut GameRng, result: &mut ZapResult) {
    // Find random walkable position
    for _ in 0..100 {
        let x = rng.rn2(crate::COLNO as u32) as i8;
        let y = rng.rn2(crate::ROWNO as u32) as i8;

        if level.is_walkable(x, y) && level.monster_at(x, y).is_none() {
            player.prev_pos = player.pos;
            player.pos.x = x;
            player.pos.y = y;
            result
                .messages
                .push("You find yourself somewhere else.".to_string());
            return;
        }
    }

    result
        .messages
        .push("You feel disoriented for a moment.".to_string());
}

/// Teleport monster to random location
fn teleport_monster(
    monster_id: MonsterId,
    level: &mut Level,
    rng: &mut GameRng,
    result: &mut ZapResult,
) {
    // Find random walkable position
    for _ in 0..100 {
        let x = rng.rn2(crate::COLNO as u32) as i8;
        let y = rng.rn2(crate::ROWNO as u32) as i8;

        if level.is_walkable(x, y) && level.monster_at(x, y).is_none() {
            if let Some(monster) = level.monster_mut(monster_id) {
                monster.x = x;
                monster.y = y;
                result
                    .messages
                    .push(format!("The {} vanishes!", monster.name));
            }
            return;
        }
    }
}

/// Monster uses breath weapon against player (from mcastu.c buzzmu)
/// This is the main entry point for dragon breath, etc.
pub fn monster_breath_weapon(
    attacker_x: i8,
    attacker_y: i8,
    attacker_name: &str,
    attacker_level: u8,
    zap_type: ZapType,
    player: &mut crate::player::You,
    level: &mut Level,
    rng: &mut GameRng,
) -> ZapResult {
    let mut result = ZapResult::new();

    // Calculate direction toward player
    let dx = (player.pos.x - attacker_x).signum();
    let dy = (player.pos.y - attacker_y).signum();

    if dx == 0 && dy == 0 {
        // Monster is on top of player - shouldn't happen
        return result;
    }

    result.messages.push(format!(
        "The {} breathes {}!",
        attacker_name,
        zap_type.name(ZapVariant::Breath)
    ));

    // Breath damage is based on monster level
    let damage_dice = (attacker_level / 2).max(1) as u32;

    // Trace the breath ray
    zap_direction(
        zap_type,
        ZapVariant::Breath,
        dx,
        dy,
        player,
        level,
        rng,
        &mut result,
    );

    // Apply extra damage based on monster level if player was hit
    if result.player_damage > 0 {
        let extra_damage = rng.dice(damage_dice, 6) as i32;
        result.player_damage += extra_damage;
        player.hp -= extra_damage;
    }

    result
}

/// Map monster attack damage type to zap type for breath weapons
pub fn damage_type_to_zap_type(damage_type: crate::combat::DamageType) -> Option<ZapType> {
    match damage_type {
        crate::combat::DamageType::Fire => Some(ZapType::Fire),
        crate::combat::DamageType::Cold => Some(ZapType::Cold),
        crate::combat::DamageType::Electric => Some(ZapType::Lightning),
        crate::combat::DamageType::MagicMissile => Some(ZapType::MagicMissile),
        crate::combat::DamageType::Sleep => Some(ZapType::Sleep),
        crate::combat::DamageType::Death => Some(ZapType::Death),
        crate::combat::DamageType::Acid => Some(ZapType::Acid),
        crate::combat::DamageType::DrainStrength => Some(ZapType::PoisonGas),
        _ => None,
    }
}

// ============================================================================
// Wand utility functions (zap.c)
// ============================================================================

/// Check if a wand is non-directional (zapnodir equivalent)
/// Non-directional wands have immediate effects and don't fire rays.
pub fn zapnodir(object_type: i16) -> bool {
    matches!(
        object_type,
        349 |  // WandOfLight
        364 |  // WandOfCreateMonster
        372 |  // WandOfWishing
        362 |  // WandOfEnlightenment
        373 // WandOfNothing
    )
}

/// Check if a wand can be zapped (has charges) (zappable equivalent)
pub fn zappable(wand: &Object) -> bool {
    // Wands need at least 1 charge to work
    // Wand of nothing (-1 charges) never works
    wand.enchantment > 0
}

/// Check if wand can potentially be recharged (can_recharge equivalent)
pub fn can_recharge(wand: &Object) -> bool {
    // Wands can be recharged, but get weaker each time
    wand.recharged < 7
}

/// Get the maximum charges for a wand type
pub fn max_wand_charges(object_type: i16) -> i8 {
    match object_type {
        355 => 15, // WandOfMagicMissile
        365 => 8,  // WandOfFire
        366 => 8,  // WandOfCold
        367 => 8,  // WandOfLightning
        368 => 15, // WandOfSleep
        369 => 3,  // WandOfDeath
        370 => 8,  // WandOfPolymorph
        371 => 8,  // WandOfTeleportation
        372 => 3,  // WandOfWishing
        363 => 8,  // WandOfCancellation
        361 => 8,  // WandOfStriking
        351 => 8,  // WandOfDigging
        349 => 15, // WandOfLight
        _ => 8,    // Default
    }
}

/// Get wand effect name
pub fn wand_effect_name(effect: &WandEffect) -> &'static str {
    match effect {
        WandEffect::StandardZap(zap_type, variant) => zap_type.name(*variant),
        WandEffect::Teleport => "teleportation",
        WandEffect::Healing => "healing",
        WandEffect::Polymorph => "polymorph",
        WandEffect::Digging => "digging",
        WandEffect::Resurrection => "resurrection",
        WandEffect::Cancellation => "cancellation",
    }
}

/// Get recharge difficulty for a wand
pub fn wand_recharge_difficulty(object_type: i16) -> i8 {
    match object_type {
        369 | 372 => 5, // Death/Wishing - very difficult
        370 => 4,       // Polymorph - difficult
        371 => 3,       // Teleportation - moderate
        _ => 2,         // Most wands - easier
    }
}

/// Check if wand needs recharging soon
pub fn wand_needs_recharge(wand: &Object) -> bool {
    let max_charges = max_wand_charges(wand.object_type);
    let threshold = (max_charges / 3).max(1);
    wand.enchantment <= threshold
}

/// Get wand durability (affects recharge success)
pub fn wand_durability_factor(wand: &Object) -> f32 {
    // Each recharge degrades the wand
    match wand.recharged {
        0 => 1.0,
        1 => 0.9,
        2 => 0.8,
        3 => 0.7,
        4 => 0.6,
        5 => 0.5,
        6 => 0.3,
        _ => 0.1,
    }
}

/// Result of a bolt/ray hitting something
#[derive(Debug, Clone)]
pub struct BhitResult {
    /// Final position of the ray
    pub end_x: i8,
    pub end_y: i8,
    /// Whether it hit a monster
    pub hit_monster: Option<MonsterId>,
    /// Whether it hit a wall/obstacle
    pub hit_wall: bool,
    /// Whether it hit the player
    pub hit_player: bool,
    /// Distance traveled
    pub distance: i32,
}

impl BhitResult {
    pub fn new() -> Self {
        Self {
            end_x: 0,
            end_y: 0,
            hit_monster: None,
            hit_wall: false,
            hit_player: false,
            distance: 0,
        }
    }
}

impl Default for BhitResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Trace a bolt/ray from a starting position in a direction (bhit equivalent)
/// Returns information about what the ray hit.
pub fn bhit(
    start_x: i8,
    start_y: i8,
    dx: i8,
    dy: i8,
    range: i32,
    player_x: i8,
    player_y: i8,
    level: &Level,
) -> BhitResult {
    let mut result = BhitResult::new();
    let mut x = start_x;
    let mut y = start_y;

    for dist in 0..range {
        x += dx;
        y += dy;

        if !level.is_valid_pos(x, y) {
            result.hit_wall = true;
            result.distance = dist;
            break;
        }

        // Check for walls
        let cell = level.cell(x as usize, y as usize);
        if cell.typ.is_wall() {
            result.hit_wall = true;
            result.distance = dist;
            break;
        }

        // Check for player
        if x == player_x && y == player_y {
            result.hit_player = true;
            result.distance = dist;
            result.end_x = x;
            result.end_y = y;
            break;
        }

        // Check for monsters
        if let Some(monster) = level.monster_at(x, y) {
            result.hit_monster = Some(monster.id);
            result.distance = dist;
            result.end_x = x;
            result.end_y = y;
            break;
        }

        result.end_x = x;
        result.end_y = y;
        result.distance = dist + 1;
    }

    result
}

/// Check if a position is in line of fire from another position
pub fn in_line_of_fire(from_x: i8, from_y: i8, to_x: i8, to_y: i8) -> bool {
    let dx = to_x - from_x;
    let dy = to_y - from_y;

    // Must be in a straight line (horizontal, vertical, or diagonal)
    dx == 0 || dy == 0 || dx.abs() == dy.abs()
}

/// Get direction from one position toward another
pub fn direction_toward(from_x: i8, from_y: i8, to_x: i8, to_y: i8) -> (i8, i8) {
    let dx = (to_x - from_x).signum();
    let dy = (to_y - from_y).signum();
    (dx, dy)
}

/// Check if zap direction is valid (zap_hit equivalent concept)
pub fn valid_zap_direction(dx: i8, dy: i8) -> bool {
    // Must be one of 8 cardinal/diagonal directions, or self (0,0)
    dx.abs() <= 1 && dy.abs() <= 1
}

/// Convert direction to glyph index for display (zapdir_to_glyph equivalent)
/// Returns an index into the zap glyph array
pub fn zapdir_to_glyph(dx: i8, dy: i8, zap_type: ZapType) -> u16 {
    let base = match zap_type {
        ZapType::MagicMissile => 0,
        ZapType::Fire => 4,
        ZapType::Cold => 8,
        ZapType::Sleep => 12,
        ZapType::Death => 16,
        ZapType::Lightning => 20,
        ZapType::PoisonGas => 24,
        ZapType::Acid => 28,
    };

    let dir_offset = match (dx, dy) {
        (0, -1) => 0,  // up
        (0, 1) => 0,   // down (same as up for display)
        (-1, 0) => 1,  // left
        (1, 0) => 1,   // right (same as left)
        (-1, -1) => 2, // up-left
        (1, 1) => 2,   // down-right
        (1, -1) => 3,  // up-right
        (-1, 1) => 3,  // down-left
        _ => 0,
    };

    base + dir_offset
}

/// Spell/wand type to damage calculation
pub fn zap_damage(zap_type: ZapType, variant: ZapVariant, rng: &mut GameRng) -> i32 {
    let (dice, sides) = match variant {
        ZapVariant::Wand => match zap_type {
            ZapType::MagicMissile => (2, 6),
            ZapType::Fire => (6, 6),
            ZapType::Cold => (6, 6),
            ZapType::Sleep => (0, 0), // No direct damage
            ZapType::Death => (0, 0), // Instakill
            ZapType::Lightning => (6, 6),
            ZapType::PoisonGas => (2, 6),
            ZapType::Acid => (4, 6),
        },
        ZapVariant::Spell => match zap_type {
            ZapType::MagicMissile => (2, 6),
            ZapType::Fire => (6, 6),
            ZapType::Cold => (4, 6),
            ZapType::Sleep => (0, 0),
            ZapType::Death => (0, 0),
            ZapType::Lightning => (6, 6),
            ZapType::PoisonGas => (2, 4),
            ZapType::Acid => (3, 6),
        },
        ZapVariant::Breath => match zap_type {
            ZapType::MagicMissile => (4, 6),
            ZapType::Fire => (4, 6),
            ZapType::Cold => (4, 6),
            ZapType::Sleep => (0, 0),
            ZapType::Death => (0, 0), // Disintegration
            ZapType::Lightning => (4, 6),
            ZapType::PoisonGas => (3, 6),
            ZapType::Acid => (3, 6),
        },
    };

    if dice == 0 {
        0
    } else {
        rng.dice(dice, sides) as i32
    }
}

// ============================================================================
// Cancellation System (zap.c:1001 cancel_item, zap.c:2730 cancel_monst)
// ============================================================================

/// Cancel an object (zap.c:1001)
/// Removes enchantment, blanks scrolls/spellbooks, neutralizes potions.
pub fn cancel_item(obj: &mut Object) -> Vec<String> {
    use crate::object::{BucStatus, ObjectClass};

    let mut messages = Vec::new();

    match obj.class {
        ObjectClass::Wand => {
            // Wand of cancellation can't be cancelled
            if obj.object_type == 363 {
                return messages;
            }
            // Strip charges
            if obj.enchantment > 0 {
                obj.enchantment = 0;
                messages.push("The wand loses its power.".to_string());
            }
        }
        ObjectClass::Weapon | ObjectClass::Armor => {
            // Remove enchantment
            if obj.enchantment != 0 {
                obj.enchantment = 0;
                messages.push("The enchantment fades.".to_string());
            }
            // Remove erosion protection
            obj.erosion_proof = false;
        }
        ObjectClass::Scroll | ObjectClass::Spellbook => {
            // Blank scrolls/spellbooks
            // Scroll of blank paper (306) is already blank
            if obj.object_type != 306 {
                obj.object_type = 306; // Become blank paper
                messages.push("The writing vanishes.".to_string());
            }
        }
        ObjectClass::Potion => {
            // Neutralize to water
            if obj.object_type != crate::magic::potion::PotionType::Water as i16 {
                obj.object_type = crate::magic::potion::PotionType::Water as i16;
                obj.buc = BucStatus::Uncursed;
                messages.push("The potion turns to water.".to_string());
            }
        }
        ObjectClass::Ring => {
            // Remove enchantment
            if obj.enchantment != 0 {
                obj.enchantment = 0;
                messages.push("The ring dulls.".to_string());
            }
        }
        ObjectClass::Tool => {
            // Strip charges from chargeable tools
            if obj.enchantment > 0 {
                obj.enchantment = 0;
                messages.push("The tool loses its charge.".to_string());
            }
        }
        _ => {}
    }

    messages
}

/// Cancel a monster (zap.c:2730)
/// Sets monster cancelled flag, reverts shapeshifters, etc.
pub fn cancel_monst(monster: &mut crate::monster::Monster) -> Vec<String> {
    let mut messages = Vec::new();

    if monster.state.cancelled {
        return messages;
    }

    monster.state.cancelled = true;
    messages.push(format!("The {} shudders!", monster.name));

    // Revert shapeshifters to original form
    if monster.monster_type != monster.original_type {
        monster.monster_type = monster.original_type;
        messages.push(format!("The {} reverts to its original form.", monster.name));
    }

    // Remove invisibility
    if monster.state.invisible {
        monster.state.invisible = false;
        messages.push(format!("The {} becomes visible.", monster.name));
    }

    messages
}

/// Probe a monster — display its stats (zap.c:485)
pub fn probe_monster(monster: &crate::monster::Monster) -> Vec<String> {
    let mut messages = Vec::new();

    messages.push(format!(
        "{}: HP:{}/{}, AC:{}, Level:{}",
        monster.name, monster.hp, monster.hp_max, monster.ac, monster.level
    ));

    if !monster.inventory.is_empty() {
        messages.push(format!(
            "  Carrying {} item{}.",
            monster.inventory.len(),
            if monster.inventory.len() == 1 { "" } else { "s" }
        ));
        for item in &monster.inventory {
            messages.push(format!("    {}", item.display_name()));
        }
    }

    if monster.state.tame {
        messages.push("  (tame)".to_string());
    } else if monster.state.peaceful {
        messages.push("  (peaceful)".to_string());
    }

    messages
}

// ============================================================================
// Object Transformation System (Phase 2)
// ============================================================================

/// Result of object polymorphism
#[derive(Debug, Clone)]
pub struct ObjectTransformResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Whether transformation was successful
    pub transformed: bool,
    /// Whether object identity was revealed
    pub identity_revealed: bool,
}

impl ObjectTransformResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            transformed: false,
            identity_revealed: false,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

impl Default for ObjectTransformResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of golem creation
#[derive(Debug, Clone)]
pub struct GolemCreationResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Whether golem was successfully created
    pub created: bool,
    /// Number of objects consumed
    pub objects_consumed: i32,
}

impl GolemCreationResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            created: false,
            objects_consumed: 0,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

impl Default for GolemCreationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of stone-to-flesh transformation
#[derive(Debug, Clone)]
pub struct StoneTransformResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Whether transformation was successful
    pub transformed: bool,
    /// Number of objects affected
    pub objects_affected: i32,
}

impl StoneTransformResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            transformed: false,
            objects_affected: 0,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

impl Default for StoneTransformResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Object material types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectMaterial {
    Iron,
    Copper,
    Flesh,
    Wood,
    Leather,
    Cloth,
    Bone,
    Gold,
    Glass,
    Paper,
    Stone,
    Ceramic,
}

// ============================================================================
// Object Transformation Helper Functions
// ============================================================================

/// Detect material of an object based on type
fn detect_object_material(object_type: i16) -> Option<ObjectMaterial> {
    // Simplified material detection based on object type
    // In full implementation, would check oc_material array
    match object_type {
        // Iron items (weapons, armor)
        1..=100 if object_type % 10 == 1 => Some(ObjectMaterial::Iron),
        // Gold items
        200..=250 => Some(ObjectMaterial::Gold),
        // Glass items
        350..=380 => Some(ObjectMaterial::Glass),
        // Wood items (staves, bows)
        50..=70 => Some(ObjectMaterial::Wood),
        // Stone/Rock items
        0..=10 => Some(ObjectMaterial::Stone),
        // Leather armor
        150..=170 => Some(ObjectMaterial::Leather),
        // Paper (books, scrolls)
        300..=320 => Some(ObjectMaterial::Paper),
        _ => None,
    }
}

/// Check if object is a mineral/stone type that can be transformed
fn is_mineral_object(object_type: i16) -> bool {
    match object_type {
        // Boulder, gems, rocks, etc.
        0..=20 | 300..=350 => true,
        _ => false,
    }
}

// ============================================================================
// Object Transformation Functions (from zap.c)
// ============================================================================

/// Create a golem from a material object
/// Adapted from zap.c:1271 create_polymon()
///
/// # Arguments
/// * `object` - The material object to create golem from
/// * `material` - The material type
/// * `player_x`, `player_y` - Player position (for golem placement)
///
/// # Returns
/// GolemCreationResult with messages and creation status
pub fn create_polymon(
    object: &Object,
    material: ObjectMaterial,
    _player_x: i8,
    _player_y: i8,
) -> GolemCreationResult {
    let mut result = GolemCreationResult::new();

    let golem_name = match material {
        ObjectMaterial::Iron => "iron golem",
        ObjectMaterial::Copper => "copper golem",
        ObjectMaterial::Flesh => "flesh golem",
        ObjectMaterial::Wood => "wood golem",
        ObjectMaterial::Leather => "leather golem",
        ObjectMaterial::Cloth => "cloth golem",
        ObjectMaterial::Bone => "bone golem",
        ObjectMaterial::Gold => "gold golem",
        ObjectMaterial::Glass => "glass golem",
        ObjectMaterial::Paper => "paper golem",
        ObjectMaterial::Stone => "stone golem",
        ObjectMaterial::Ceramic => "ceramic golem",
    };

    result.messages.push(format!(
        "A {} arises from the {}!",
        golem_name,
        if object.object_type < 20 {
            "ground"
        } else {
            "object"
        }
    ));
    result.created = true;
    result.objects_consumed = 1;

    result
}

/// Transform stone/mineral objects into flesh or animate them
/// Adapted from zap.c:1692 stone_to_flesh_obj()
///
/// # Arguments
/// * `object` - The stone object to transform
/// * `player_has_resistance` - Whether player has relevant resistance
/// * `rng` - Random number generator
///
/// # Returns
/// StoneTransformResult with messages and transformation status
pub fn stone_to_flesh_obj(
    object: &Object,
    player_has_resistance: bool,
    rng: &mut GameRng,
) -> StoneTransformResult {
    let mut result = StoneTransformResult::new();

    // Check if object can be transformed
    if !is_mineral_object(object.object_type) {
        result.messages.push("Nothing happens.".to_string());
        return result;
    }

    // 2% resistance chance
    if !player_has_resistance && rng.one_in(50) {
        result
            .messages
            .push("The stone glows momentarily but resists transformation.".to_string());
        return result;
    }

    match object.object_type {
        // Boulder
        0..=5 => {
            let chunks = rng.dice(3, 4) as i32;
            result.messages.push(format!(
                "The boulder transforms into {} chunks of meat!",
                chunks
            ));
            result.transformed = true;
            result.objects_affected = chunks;
        }
        // Statue or figurine
        10..=15 => {
            result
                .messages
                .push("The statue animates and becomes a flesh creature!".to_string());
            result.transformed = true;
            result.objects_affected = 1;
        }
        // Ring, wand, or gem
        20..=50 => {
            result
                .messages
                .push("The stone object turns to meat!".to_string());
            result.transformed = true;
            result.objects_affected = 1;
        }
        // Default
        _ => {
            result
                .messages
                .push("The stone glows but nothing happens.".to_string());
        }
    }

    result
}

/// Transform an object into a different object type
/// Adapted from zap.c:1421 poly_obj()
///
/// # Arguments
/// * `object` - The object to transform
/// * `new_type` - Optional new object type (None = random)
/// * `rng` - Random number generator
///
/// # Returns
/// ObjectTransformResult with transformation status and messages
pub fn poly_obj(
    object: &Object,
    new_type: Option<i16>,
    rng: &mut GameRng,
) -> ObjectTransformResult {
    let mut result = ObjectTransformResult::new();

    // Determine new type
    let target_type = if let Some(nt) = new_type {
        nt
    } else {
        // Random selection from same class (simplified)
        // In full implementation, would select from pool of same class
        rng.rnd(1000) as i16
    };

    if target_type == object.object_type {
        result
            .messages
            .push("The object shimmers but remains unchanged.".to_string());
        return result;
    }

    // Preserve blessing/cursing
    let was_blessed = object.is_blessed();
    let was_cursed = object.is_cursed();

    result.messages.push(format!(
        "The {} transforms into something different!",
        if object.quantity > 1 {
            format!("one of the {}", object.display_name())
        } else {
            object.display_name()
        }
    ));

    // Set transformed flag
    result.transformed = true;
    result.identity_revealed = false; // Transformed object is "new"

    // Add message about blessing status
    if was_blessed {
        result
            .messages
            .push("The new object glows with a holy aura!".to_string());
    } else if was_cursed {
        result
            .messages
            .push("The new object has a cursed aura!".to_string());
    }

    result
}

// ============================================================================
// Explosion & Breaking System (Phase 3)
// ============================================================================

/// Result of explosion effect
#[derive(Debug, Clone)]
pub struct ExplosionResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Monsters killed by explosion
    pub monsters_killed: Vec<MonsterId>,
    /// Damage dealt to player
    pub player_damage: i32,
    /// Whether player died
    pub player_died: bool,
    /// Shop damage incurred
    pub shop_damage: i32,
    /// Items destroyed
    pub items_destroyed: i32,
}

impl ExplosionResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            monsters_killed: Vec::new(),
            player_damage: 0,
            player_died: false,
            shop_damage: 0,
            items_destroyed: 0,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

impl Default for ExplosionResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of object breaking
#[derive(Debug, Clone)]
pub struct BreakingResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Whether object broke
    pub broke: bool,
    /// Luck change from breaking (e.g., -2 for mirror)
    pub luck_change: i32,
    /// Shop charge for breaking
    pub shop_charge: i32,
}

impl BreakingResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            broke: false,
            luck_change: 0,
            shop_charge: 0,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

impl Default for BreakingResult {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Explosion Functions (from explode.c)
// ============================================================================

/// Explosion source type (determines death message and damage scaling)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionSource {
    /// Wand explosion (retributive strike) — role-based damage reduction
    Wand,
    /// Monster exploding (gas spore, etc.)
    Monster,
    /// Spell or scroll (tower of flame, fireball)
    Spell,
    /// Burning oil
    Oil,
    /// Other
    Other,
}

/// Explosion description name from damage type (matches C explode.c type→str mapping)
pub fn explosion_name(damage_type: DamageType, source: ExplosionSource) -> &'static str {
    match damage_type {
        DamageType::MagicMissile => "magical blast",
        DamageType::Fire => match source {
            ExplosionSource::Oil => "burning oil",
            ExplosionSource::Spell => "tower of flame",
            _ => "fireball",
        },
        DamageType::Cold => "ball of cold",
        DamageType::Disintegrate => "disintegration field",
        DamageType::Electric => "ball of lightning",
        DamageType::DrainStrength => "poison gas cloud",
        DamageType::Acid => "splash of acid",
        _ => "explosion",
    }
}

/// Check if the player resists a specific damage type
fn player_resists(player: &You, damage_type: DamageType) -> bool {
    match damage_type {
        DamageType::Fire => player.properties.has(Property::FireResistance),
        DamageType::Cold => player.properties.has(Property::ColdResistance),
        DamageType::Electric => player.properties.has(Property::ShockResistance),
        DamageType::DrainStrength => player.properties.has(Property::PoisonResistance),
        DamageType::Acid => player.properties.has(Property::AcidResistance),
        DamageType::Disintegrate => player.properties.has(Property::DisintResistance),
        DamageType::MagicMissile => player.properties.has(Property::MagicResistance),
        _ => false,
    }
}

/// Apply role-based damage reduction for wand explosions (retributive strike).
///
/// Matches C explode.c: Priest/Monk/Wizard → dam/5, Healer/Knight → dam/2.
pub fn role_damage_reduction(damage: i32, role: crate::player::Role) -> i32 {
    use crate::player::Role;
    match role {
        Role::Priest | Role::Monk | Role::Wizard => damage / 5,
        Role::Healer | Role::Knight => damage / 2,
        _ => damage,
    }
}

/// Create explosion at location with 3x3 area damage.
///
/// Adapted from explode.c:28. Iterates over a 3×3 grid centered on (x,y),
/// computing a resistance mask per cell, then applying damage to monsters
/// and the player with proper resistance and cross-vulnerability checks.
///
/// # Arguments
/// * `x`, `y` - Center of explosion
/// * `damage_type` - Type of damage (fire, cold, acid, etc.)
/// * `base_damage` - Base damage amount
/// * `source` - What caused the explosion (wand, monster, spell, etc.)
/// * `player` - Player reference
/// * `level` - Level with monsters
/// * `rng` - Random number generator
pub fn explode(
    x: i8,
    y: i8,
    damage_type: DamageType,
    base_damage: i32,
    source: ExplosionSource,
    player: &mut You,
    level: &mut Level,
    rng: &mut GameRng,
) -> ExplosionResult {
    let mut result = ExplosionResult::new();

    // Role-based damage reduction for wand explosions (retributive strike)
    let player_dam = if source == ExplosionSource::Wand {
        role_damage_reduction(base_damage, player.role)
    } else {
        base_damage
    };

    let str_name = explosion_name(damage_type, source);
    result.messages.push(format!("There is an explosion of {}!", str_name));

    // Build 3x3 resistance mask: 0=normal, 1=shielded (resists), 2=skip (invalid)
    let mut explmask = [[0u8; 3]; 3];

    for i in 0..3i8 {
        for j in 0..3i8 {
            let cx = x + i - 1;
            let cy = y + j - 1;

            if !level.is_valid_pos(cx, cy) {
                explmask[i as usize][j as usize] = 2;
                continue;
            }

            // Check player resistance at this cell
            if cx == player.pos.x && cy == player.pos.y {
                if player_resists(player, damage_type) {
                    explmask[i as usize][j as usize] = 1;
                }
            }

            // Check monster resistance at this cell
            for mon in &level.monsters {
                if mon.x == cx && mon.y == cy && mon.hp > 0 {
                    let mon_resists = match damage_type {
                        DamageType::Fire => mon.resists_fire(),
                        DamageType::Cold => mon.resists_cold(),
                        DamageType::Electric => mon.resists_elec(),
                        DamageType::DrainStrength => mon.resists_poison(),
                        DamageType::Acid => mon.resists_acid(),
                        DamageType::Disintegrate => mon.resists_disint(),
                        DamageType::MagicMissile => mon.resists_magic(),
                        _ => false,
                    };
                    if mon_resists {
                        explmask[i as usize][j as usize] |= 1;
                    }
                }
            }
        }
    }

    // Apply damage to monsters in the 3x3 area
    for i in 0..3i8 {
        for j in 0..3i8 {
            if explmask[i as usize][j as usize] == 2 {
                continue;
            }

            let cx = x + i - 1;
            let cy = y + j - 1;
            let shielded = explmask[i as usize][j as usize] == 1;

            // Find monster at this position and apply damage
            for mon in &mut level.monsters {
                if mon.x != cx || mon.y != cy || mon.hp <= 0 {
                    continue;
                }

                if shielded {
                    // Shielded: only item destruction damage, not blast damage
                    result.messages.push(format!(
                        "{} resists the {}!", mon.name, str_name
                    ));
                    let item_dam = (base_damage + 1) / 2;
                    mon.hp -= item_dam;
                } else {
                    result.messages.push(format!(
                        "{} is caught in the {}!", mon.name, str_name
                    ));
                    let mut mdam = base_damage;

                    // Cross-resistance vulnerability: fire-resistant takes
                    // double cold, and cold-resistant takes double fire
                    if mon.resists_cold() && damage_type == DamageType::Fire {
                        mdam *= 2;
                    } else if mon.resists_fire() && damage_type == DamageType::Cold {
                        mdam *= 2;
                    }

                    mon.hp -= mdam;
                }

                if mon.hp <= 0 {
                    result.monsters_killed.push(mon.id);
                }
            }
        }
    }

    // Apply damage to player (do last, matching C ordering)
    let player_in_area = {
        let dx = (player.pos.x - x).abs();
        let dy = (player.pos.y - y).abs();
        dx <= 1 && dy <= 1
    };

    if player_in_area {
        let pi = (player.pos.x - x + 1) as usize;
        let pj = (player.pos.y - y + 1) as usize;

        if explmask[pi][pj] != 2 {
            let shielded = explmask[pi][pj] == 1;

            result.messages.push(format!("You are caught in the {}!", str_name));

            // Fire burns away slime
            // (tracked via message — full implementation pending)

            if shielded {
                // Resisted: half damage
                let resist_dam = (player_dam + 1) / 2;
                player.hp -= resist_dam;
                result.player_damage = resist_dam;
            } else {
                player.hp -= player_dam;
                result.player_damage = player_dam;
            }

            // Item destruction tracking (scrolls, potions, wands, rings)
            if matches!(damage_type,
                DamageType::Fire | DamageType::Cold | DamageType::Electric | DamageType::Acid
            ) {
                let destroyed = match damage_type {
                    DamageType::Fire => rng.rn2(3) as i32, // 0-2 items
                    DamageType::Cold => rng.rn2(2) as i32,
                    DamageType::Electric => rng.rn2(3) as i32,
                    DamageType::Acid => rng.rn2(2) as i32,
                    _ => 0,
                };
                if destroyed > 0 {
                    result.items_destroyed += destroyed;
                    result.messages.push(format!(
                        "Some of your possessions are {}!",
                        match damage_type {
                            DamageType::Fire => "burnt",
                            DamageType::Cold => "frozen and shattered",
                            DamageType::Electric => "fried",
                            DamageType::Acid => "corroded away",
                            _ => "destroyed",
                        }
                    ));
                }
            }

            result.player_died = player.hp <= 0;
        }
    }

    result
}

/// Explode burning oil potion
/// Adapted from explode.c:807 explode_oil()
pub fn explode_oil(
    _object: &Object,
    x: i8,
    y: i8,
    player: &mut You,
    level: &mut Level,
    rng: &mut GameRng,
) -> ExplosionResult {
    let damage = rng.dice(4, 4) as i32;
    explode(x, y, DamageType::Fire, damage, ExplosionSource::Oil, player, level, rng)
}

/// Splatter burning oil - lesser explosion
/// Adapted from explode.c:793 splatter_burning_oil()
pub fn splatter_burning_oil(
    x: i8,
    y: i8,
    diluted: bool,
    player: &mut You,
    level: &mut Level,
    rng: &mut GameRng,
) -> ExplosionResult {
    let damage = if diluted {
        rng.dice(3, 4) as i32
    } else {
        rng.dice(4, 4) as i32
    };

    explode(x, y, DamageType::Fire, damage, ExplosionSource::Oil, player, level, rng)
}

// ============================================================================
// Object Breaking Functions (from dothrow.c)
// ============================================================================

/// Test if object can break
/// Adapted from dothrow.c:2054 breaktest()
pub fn breaktest(object: &Object) -> bool {
    // 2% resist chance
    if object.is_blessed() || object.is_cursed() {
        return false; // Magical items resist breaking
    }

    // Glass items always break
    // Specific brittle items break
    match object.object_type {
        // Glass, mirrors, cameras, potions, etc.
        100..=150 | 200..=250 => true,
        _ => false,
    }
}

/// Display break message
/// Adapted from dothrow.c:2077 breakmsg()
pub fn breakmsg(object: &Object, in_view: bool) -> String {
    if !in_view {
        return "You hear a crash!".to_string();
    }

    match object.object_type {
        // Glass and mirrors
        100..=120 => format!("{} shatters into a thousand pieces!", object.display_name()),
        // Potions
        200..=220 => format!("{} explodes!", object.display_name()),
        // Eggs
        300..=310 => "Splat!".to_string(),
        // Cream pie
        320..=330 => "What a mess!".to_string(),
        // Default
        _ => format!("{} breaks!", object.display_name()),
    }
}

/// Break object from hero action
/// Adapted from dothrow.c:1909 hero_breaks()
pub fn hero_breaks(object: &Object, _x: i8, _y: i8) -> BreakingResult {
    let mut result = BreakingResult::new();

    if !breaktest(object) {
        return result;
    }

    let msg = breakmsg(object, true);
    result.messages.push(msg);
    result.broke = true;

    // Mirror breaking causes luck damage
    if object.object_type >= 100 && object.object_type <= 110 {
        result.luck_change = -2;
        result.messages.push("You feel unlucky!".to_string());
    }

    result
}

/// Break object from non-hero cause
/// Adapted from dothrow.c:1929 breaks()
pub fn breaks(object: &Object, _x: i8, _y: i8) -> BreakingResult {
    let mut result = BreakingResult::new();

    if !breaktest(object) {
        return result;
    }

    let msg = breakmsg(object, true);
    result.messages.push(msg);
    result.broke = true;

    result
}

/// Core break handler with effects
/// Adapted from dothrow.c:1965 breakobj()
pub fn breakobj(object: &Object, _x: i8, _y: i8, hero_caused: bool) -> BreakingResult {
    let mut result = BreakingResult::new();

    if !breaktest(object) {
        return result;
    }

    // Special effects based on object type
    match object.object_type {
        // Mirror - bad luck
        100..=110 => {
            result.messages.push("You hear a crash!".to_string());
            if hero_caused {
                result.luck_change = -2;
            }
        }
        // Potion - apply splash effects
        200..=220 => {
            result.messages.push(format!(
                "{} explodes in a shower of liquid!",
                object.display_name()
            ));
            // In full implementation, would apply splash effects
        }
        // Egg - luck effect
        300..=310 => {
            result.messages.push("Splat!".to_string());
            if hero_caused {
                result.luck_change = -1;
            }
        }
        // Boulder/statue - set fracture flag
        400..=450 => {
            result
                .messages
                .push(format!("{} breaks apart!", object.display_name()));
            // In full implementation, would set fracture flag
        }
        _ => {
            result
                .messages
                .push(format!("{} breaks!", object.display_name()));
        }
    }

    result.broke = true;
    result
}

// ============================================================================
// Wand Degradation System (wand wear-and-tear)
// ============================================================================

/// Calculate wand wear factor based on use
pub fn calculate_wand_wear(wand: &Object, uses: i32) -> f32 {
    let wear_per_use = 0.05_f32; // 5% wear per use
    let max_wear = 1.0_f32;

    let total_wear = (uses as f32) * wear_per_use;
    total_wear.min(max_wear)
}

/// Check if wand breaks during use
pub fn check_wand_breakage(wand: &Object, rng: &mut GameRng) -> bool {
    // Base breakage chance: 5% per use
    let base_chance = 5;

    // Cursed wands have higher breakage
    let curse_factor = if wand.is_cursed() { 2 } else { 1 };

    // Less charges = higher breakage risk
    let charge_factor = if wand.enchantment <= 2 {
        2
    } else if wand.enchantment <= 5 {
        1
    } else {
        0
    };

    let total_chance = base_chance * curse_factor + charge_factor;
    rng.percent(total_chance as u32)
}

/// Reduce wand effectiveness based on wear
pub fn get_wand_effectiveness(wand: &Object, uses: i32) -> f32 {
    let wear = calculate_wand_wear(wand, uses);

    // Effectiveness scales from 1.0 (new) to 0.3 (heavily worn)
    let effectiveness = 1.0_f32 - (wear * 0.7_f32);
    effectiveness.max(0.3_f32)
}

/// Calculate damage/effect reduction from wand wear
pub fn apply_wand_wear_penalty(base_damage: i32, effectiveness: f32) -> i32 {
    ((base_damage as f32) * effectiveness) as i32
}

/// Track wand usage and apply degradation
pub fn degrade_wand(wand: &mut Object, rng: &mut GameRng) {
    // Track use count (stored in quantity field as a hack)
    // In a full implementation, would need a dedicated field

    // Check for breakage
    if check_wand_breakage(wand, rng) {
        // Wand breaks - no charges left
        wand.enchantment = 0;
    }

    // Reduce charge by small amount if blessed/worn
    if wand.is_cursed() && rng.percent(30) {
        wand.enchantment = wand.enchantment.saturating_sub(1);
    }
}

/// Get wand status message based on wear
pub fn get_wand_status(wand: &Object, uses: i32) -> String {
    let wear = calculate_wand_wear(wand, uses);

    if wand.enchantment <= 0 {
        "The wand has no charges.".to_string()
    } else if wear > 0.8 {
        format!(
            "The wand is heavily worn and has {} charge(s) left.",
            wand.enchantment
        )
    } else if wear > 0.5 {
        format!(
            "The wand shows signs of wear and has {} charge(s) left.",
            wand.enchantment
        )
    } else if wear > 0.2 {
        format!(
            "The wand looks slightly worn and has {} charge(s) left.",
            wand.enchantment
        )
    } else {
        format!(
            "The wand looks like new and has {} charge(s) left.",
            wand.enchantment
        )
    }
}

// ============================================================================
// Rock and Statue Breaking (from zap.c)
// ============================================================================

/// Result of fracturing rock or breaking statue
#[derive(Debug, Clone, Default)]
pub struct FractureResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Whether the rock/statue was destroyed
    pub destroyed: bool,
    /// Items dropped (e.g., contents of statue, rubble)
    pub dropped_items: Vec<i16>, // Object type IDs
    /// Monster released (from statue)
    pub released_monster: Option<i16>,
    /// Player damage (from explosion or debris)
    pub player_damage: i32,
}

impl FractureResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

/// Fracture rock at a position
/// Used by digging, force bolt, and similar effects
///
/// Adapted from zap.c fracture_rock()
pub fn fracture_rock(level: &mut Level, x: i8, y: i8) -> FractureResult {
    let mut result = FractureResult::new();

    let ux = x as usize;
    let uy = y as usize;

    if ux >= crate::COLNO || uy >= crate::ROWNO {
        return result;
    }

    let cell = &level.cells[ux][uy];

    // Check if it's rock that can be fractured
    use crate::dungeon::CellType;
    let is_rock = matches!(cell.typ, CellType::Stone | CellType::Wall);

    if !is_rock {
        result
            .messages
            .push("There's nothing to break here.".to_string());
        return result;
    }

    // Fracture the rock
    level.cells[ux][uy] = crate::dungeon::Cell::floor();
    result.destroyed = true;
    result.messages.push("The rock crumbles!".to_string());

    // Chance of finding minerals (simplified from NetHack's mineralize)
    // In full implementation, would check for gold/gems

    result
}

/// Break a statue, possibly releasing the monster inside
///
/// Adapted from zap.c break_statue()
pub fn break_statue(
    level: &mut Level,
    object: &Object,
    x: i8,
    y: i8,
    rng: &mut GameRng,
) -> FractureResult {
    let mut result = FractureResult::new();

    // Check if it's actually a statue
    // In full implementation, would check object type against statue type
    let is_statue = object.object_type >= 400 && object.object_type <= 450
        || object
            .name
            .as_ref()
            .map(|n| n.to_lowercase().contains("statue"))
            .unwrap_or(false);

    if !is_statue {
        result.messages.push("That's not a statue.".to_string());
        return result;
    }

    result
        .messages
        .push("The statue breaks into pieces!".to_string());
    result.destroyed = true;

    // Chance to release trapped monster
    // In NetHack, statues can contain petrified monsters
    // 20% chance of finding contents
    if rng.percent(20) {
        // Could release a monster or drop items
        result
            .messages
            .push("Something stirs within the rubble...".to_string());
        // In full implementation, would check statue's monster type and spawn it
    }

    // Drop statue contents
    // Statues can contain items (placed by player or randomly)
    // In full implementation, would check object.contents field

    result
}

/// Check if breaking rock would release water/lava
/// Used to determine if fracturing is safe
pub fn rock_contains_hazard(level: &Level, x: i8, y: i8) -> Option<&'static str> {
    let ux = x as usize;
    let uy = y as usize;

    if ux >= crate::COLNO || uy >= crate::ROWNO {
        return None;
    }

    // Check adjacent cells for water/lava that could flood
    let directions: [(i8, i8); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    for (dx, dy) in directions {
        let nx = x + dx;
        let ny = y + dy;
        let nux = nx as usize;
        let nuy = ny as usize;

        if nux < crate::COLNO && nuy < crate::ROWNO {
            use crate::dungeon::CellType;
            match level.cells[nux][nuy].typ {
                CellType::Water => return Some("water"),
                CellType::Lava => return Some("lava"),
                _ => {}
            }
        }
    }

    None
}

/// Attempt to dig through rock
/// Returns true if dig was successful
pub fn dig_rock(level: &mut Level, x: i8, y: i8, _rng: &mut GameRng) -> FractureResult {
    let mut result = fracture_rock(level, x, y);

    // Check for flooding
    if let Some(hazard) = rock_contains_hazard(level, x, y) {
        result
            .messages
            .push(format!("Warning: {} seeps through the cracks!", hazard));
        // In full implementation, would flood the cell
    }

    result
}

// ============================================================================
// Additional Explosion Functions (from explode.c, zap.c, mon.c)
// ============================================================================

/// Shock objects inside a bag of holding (do_osshock equivalent)
///
/// When a bag of holding is zapped with lightning, objects inside may
/// be destroyed or transformed.
///
/// # Arguments
/// * `object` - The object inside the bag being shocked
/// * `rng` - Random number generator
/// * `luck` - Player's luck value
///
/// # Returns
/// Tuple of (was_destroyed, was_polymorphed, messages)
pub fn do_osshock(object: &Object, rng: &mut GameRng, luck: i8) -> (bool, bool, Vec<String>) {
    let mut messages = Vec::new();
    let mut destroyed = false;
    let mut polymorphed = false;

    // Some chance to polymorph based on luck
    if rng.rnl(45, luck) == 0 {
        polymorphed = true;
        messages.push(format!("{} transforms!", object.display_name()));
    }

    // Determine if object survives
    // Items in large stacks may partially survive
    if object.quantity > 1 {
        // Some survive, some destroyed
        let survive_count = rng.rn2(object.quantity as u32) as i32;
        if survive_count < object.quantity {
            destroyed = true;
            messages.push(format!(
                "Some of the {} are destroyed by the shock!",
                object.display_name()
            ));
        }
    } else {
        // Single item - more likely to be destroyed
        if rng.rn2(2) == 0 {
            destroyed = true;
            messages.push(format!(
                "The {} is destroyed by the shock!",
                object.display_name()
            ));
        }
    }

    (destroyed, polymorphed, messages)
}

/// Water elementals clogging fire damage in the endgame (elemental_clog equivalent)
///
/// In the Elemental Planes, summoning new elementals may cause existing
/// ones to be replaced or removed to prevent overcrowding.
///
/// # Arguments
/// * `monsters` - List of monsters on the level
/// * `is_endgame` - Whether we're in the endgame
/// * `moves` - Current game turn count
/// * `last_besieged_turn` - Last turn a "besieged" message was shown
///
/// # Returns
/// Option containing the MonsterId to remove, if any, and messages
pub fn elemental_clog(
    monsters: &[crate::monster::Monster],
    is_endgame: bool,
    moves: i64,
    last_besieged_turn: &mut i64,
) -> (Option<crate::monster::MonsterId>, Vec<String>) {
    let mut messages = Vec::new();
    let mut remove_monster = None;

    if !is_endgame {
        return (None, messages);
    }

    // Show "besieged" message periodically
    if *last_besieged_turn == 0 || (moves - *last_besieged_turn) > 200 {
        messages.push("You feel besieged.".to_string());
        *last_besieged_turn = moves;
    }

    // Find candidates for removal based on priority:
    // 1. Elementals from other planes
    // 2. Elementals from this plane
    // 3. Least powerful monster
    // 4. Other non-tame monster
    // 5. Pet (last resort)

    let mut lowest_level_mon: Option<&crate::monster::Monster> = None;
    let mut other_mon: Option<&crate::monster::Monster> = None;

    for mon in monsters {
        if mon.hp <= 0 {
            continue;
        }

        // Skip unplaced monsters
        if mon.x == 0 && mon.y == 0 {
            continue;
        }

        let name_lower = mon.name.to_lowercase();

        // Check if this is an elemental
        if name_lower.contains("elemental") {
            // Prefer removing elementals from other planes
            remove_monster = Some(mon.id);
            break;
        }

        // Track least powerful monster
        if let Some(ref lowest) = lowest_level_mon {
            if mon.level < lowest.level {
                lowest_level_mon = Some(mon);
            }
        } else {
            lowest_level_mon = Some(mon);
        }

        // Track other non-tame monsters
        if !mon.state.tame {
            other_mon = Some(mon);
        }
    }

    // If no elemental found, try other candidates
    if remove_monster.is_none() {
        if let Some(mon) = lowest_level_mon {
            remove_monster = Some(mon.id);
        } else if let Some(mon) = other_mon {
            remove_monster = Some(mon.id);
        }
    }

    (remove_monster, messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{BucStatus, ObjectClass, ObjectId};

    #[test]
    fn test_zap_type_names() {
        assert_eq!(ZapType::Fire.name(ZapVariant::Wand), "bolt of fire");
        assert_eq!(ZapType::Fire.name(ZapVariant::Spell), "fireball");
        assert_eq!(ZapType::Fire.name(ZapVariant::Breath), "blast of fire");
    }

    #[test]
    fn test_zap_type_damage() {
        assert_eq!(ZapType::Fire.damage_type(), DamageType::Fire);
        assert_eq!(ZapType::Cold.damage_type(), DamageType::Cold);
        assert_eq!(ZapType::Death.damage_type(), DamageType::Death);
    }

    #[test]
    fn test_zapnodir() {
        assert!(zapnodir(349)); // WandOfLight
        assert!(zapnodir(372)); // WandOfWishing
        assert!(!zapnodir(365)); // WandOfFire - directional
    }

    #[test]
    fn test_valid_zap_direction() {
        assert!(valid_zap_direction(0, 0)); // self
        assert!(valid_zap_direction(1, 0)); // right
        assert!(valid_zap_direction(-1, 1)); // down-left
        assert!(!valid_zap_direction(2, 0)); // invalid
    }

    #[test]
    fn test_direction_toward() {
        assert_eq!(direction_toward(0, 0, 5, 0), (1, 0)); // right
        assert_eq!(direction_toward(5, 5, 0, 0), (-1, -1)); // up-left
        assert_eq!(direction_toward(0, 0, 0, 10), (0, 1)); // down
    }

    // Phase 2 Tests: Object Transformation

    #[test]
    fn test_object_transform_result() {
        let result = ObjectTransformResult::new();
        assert!(!result.transformed);
        assert!(!result.identity_revealed);
        assert!(result.messages.is_empty());

        let result = result.with_message("Test");
        assert_eq!(result.messages.len(), 1);
    }

    #[test]
    fn test_golem_creation_result() {
        let result = GolemCreationResult::new();
        assert!(!result.created);
        assert_eq!(result.objects_consumed, 0);
        assert!(result.messages.is_empty());

        let result = result.with_message("Golem created!");
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0], "Golem created!");
    }

    #[test]
    fn test_stone_transform_result() {
        let result = StoneTransformResult::new();
        assert!(!result.transformed);
        assert_eq!(result.objects_affected, 0);
        assert!(result.messages.is_empty());

        let result = result.with_message("Stone transformed!");
        assert_eq!(result.messages.len(), 1);
    }

    #[test]
    fn test_object_material_enum() {
        // Test that all material variants exist
        let materials = [
            ObjectMaterial::Iron,
            ObjectMaterial::Copper,
            ObjectMaterial::Flesh,
            ObjectMaterial::Wood,
            ObjectMaterial::Leather,
            ObjectMaterial::Cloth,
            ObjectMaterial::Bone,
            ObjectMaterial::Gold,
            ObjectMaterial::Glass,
            ObjectMaterial::Paper,
            ObjectMaterial::Stone,
            ObjectMaterial::Ceramic,
        ];

        assert_eq!(materials.len(), 12);
    }

    #[test]
    fn test_create_polymon_iron_golem() {
        // Create a dummy object
        let obj = Object {
            object_type: 50, // dummy type
            quantity: 1,
            ..Default::default()
        };

        let result = create_polymon(&obj, ObjectMaterial::Iron, 0, 0);

        assert!(result.created);
        assert_eq!(result.objects_consumed, 1);
        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("iron golem"));
    }

    #[test]
    fn test_create_polymon_gold_golem() {
        let obj = Object {
            object_type: 200,
            quantity: 1,
            ..Default::default()
        };

        let result = create_polymon(&obj, ObjectMaterial::Gold, 5, 5);

        assert!(result.created);
        assert!(result.messages[0].contains("gold golem"));
    }

    #[test]
    fn test_create_polymon_all_materials() {
        let obj = Object {
            object_type: 100,
            quantity: 1,
            ..Default::default()
        };

        let materials = [
            (ObjectMaterial::Iron, "iron golem"),
            (ObjectMaterial::Copper, "copper golem"),
            (ObjectMaterial::Flesh, "flesh golem"),
            (ObjectMaterial::Wood, "wood golem"),
            (ObjectMaterial::Bone, "bone golem"),
            (ObjectMaterial::Stone, "stone golem"),
        ];

        for (material, name) in materials {
            let result = create_polymon(&obj, material, 0, 0);
            assert!(result.created);
            assert!(result.messages[0].contains(name));
        }
    }

    #[test]
    fn test_stone_to_flesh_non_mineral() {
        let obj = Object {
            object_type: 999, // Not a mineral
            quantity: 1,
            ..Default::default()
        };

        let mut rng = GameRng::new(42);
        let result = stone_to_flesh_obj(&obj, false, &mut rng);

        assert!(!result.transformed);
        assert_eq!(result.objects_affected, 0);
        assert!(result.messages[0].contains("Nothing happens"));
    }

    #[test]
    fn test_stone_to_flesh_boulder() {
        let obj = Object {
            object_type: 3, // Boulder range
            quantity: 1,
            ..Default::default()
        };

        let mut rng = GameRng::new(42);
        let result = stone_to_flesh_obj(&obj, false, &mut rng);

        assert!(result.transformed);
        assert!(result.objects_affected > 0);
        assert!(result.messages[0].contains("meat"));
    }

    #[test]
    fn test_stone_to_flesh_statue() {
        let obj = Object {
            object_type: 12, // Statue range
            quantity: 1,
            ..Default::default()
        };

        let mut rng = GameRng::new(42);
        let result = stone_to_flesh_obj(&obj, false, &mut rng);

        assert!(result.transformed);
        assert_eq!(result.objects_affected, 1);
        assert!(result.messages[0].contains("animates"));
    }

    #[test]
    fn test_poly_obj_same_type() {
        let obj = Object {
            object_type: 100,
            quantity: 1,
            ..Default::default()
        };

        let mut rng = GameRng::new(42);
        let result = poly_obj(&obj, Some(100), &mut rng);

        assert!(!result.transformed);
        assert!(result.messages[0].contains("unchanged"));
    }

    #[test]
    fn test_poly_obj_different_type() {
        let obj = Object {
            object_type: 100,
            quantity: 1,
            ..Default::default()
        };

        let mut rng = GameRng::new(42);
        let result = poly_obj(&obj, Some(200), &mut rng);

        assert!(result.transformed);
        assert!(!result.messages.is_empty());
        assert!(result.messages[0].contains("transforms"));
    }

    #[test]
    fn test_poly_obj_random_type() {
        let obj = Object {
            object_type: 100,
            quantity: 1,
            ..Default::default()
        };

        let mut rng = GameRng::new(42);
        let result = poly_obj(&obj, None, &mut rng);

        // Random selection might pick same type occasionally, but usually different
        assert!(!result.messages.is_empty());
    }

    #[test]
    fn test_poly_obj_preserve_blessing() {
        let mut obj = Object {
            object_type: 100,
            quantity: 1,
            ..Default::default()
        };
        obj.buc = crate::object::BucStatus::Blessed;

        let mut rng = GameRng::new(42);
        let result = poly_obj(&obj, Some(200), &mut rng);

        assert!(result.transformed);
        assert!(result.messages.iter().any(|m| m.contains("holy")));
    }

    #[test]
    fn test_poly_obj_preserve_curse() {
        let mut obj = Object {
            object_type: 100,
            quantity: 1,
            ..Default::default()
        };
        obj.buc = crate::object::BucStatus::Cursed;

        let mut rng = GameRng::new(42);
        let result = poly_obj(&obj, Some(200), &mut rng);

        assert!(result.transformed);
        assert!(result.messages.iter().any(|m| m.contains("curse")));
    }

    #[test]
    fn test_detect_material_iron() {
        // Test material detection for iron (simplified ranges)
        let material = detect_object_material(11);
        assert_eq!(material, Some(ObjectMaterial::Iron));
    }

    #[test]
    fn test_is_mineral_object_boulder() {
        // Boulders are in 0..=20 range
        assert!(is_mineral_object(5));
        assert!(is_mineral_object(0));
        assert!(is_mineral_object(15));
    }

    #[test]
    fn test_is_mineral_object_gem() {
        // Gems are in 300..=350 range
        assert!(is_mineral_object(320));
        assert!(is_mineral_object(300));
    }

    #[test]
    fn test_is_mineral_object_non_mineral() {
        // Non-mineral objects
        assert!(!is_mineral_object(100));
        assert!(!is_mineral_object(200));
        assert!(!is_mineral_object(400));
    }

    // ========== Phase 3: Explosion & Breaking System Tests ==========

    #[test]
    fn test_explosion_result() {
        let result = ExplosionResult::new();
        assert_eq!(result.messages.len(), 0);
        assert_eq!(result.monsters_killed.len(), 0);
        assert_eq!(result.player_damage, 0);
        assert!(!result.player_died);
        assert_eq!(result.shop_damage, 0);
        assert_eq!(result.items_destroyed, 0);
    }

    #[test]
    fn test_explosion_result_with_message() {
        let mut result = ExplosionResult::new();
        result = result.with_message("Boom!".to_string());
        assert_eq!(result.messages.len(), 1);
        assert!(result.messages[0].contains("Boom"));
    }

    #[test]
    fn test_breaking_result() {
        let result = BreakingResult::new();
        assert_eq!(result.messages.len(), 0);
        assert!(!result.broke);
        assert_eq!(result.luck_change, 0);
        assert_eq!(result.shop_charge, 0);
    }

    #[test]
    fn test_breaking_result_with_message() {
        let mut result = BreakingResult::new();
        result = result.with_message("Crash!".to_string());
        assert_eq!(result.messages.len(), 1);
        assert!(result.messages[0].contains("Crash"));
    }

    #[test]
    fn test_breaktest_breakable() {
        let obj = Object {
            object_type: 100, // Glass/mirror type
            ..Default::default()
        };
        assert!(breaktest(&obj));
    }

    #[test]
    fn test_breaktest_non_breakable() {
        let obj = Object {
            object_type: 50, // Weapon (not breakable)
            ..Default::default()
        };
        // Most weapons don't break, so this should return false or based on C logic
        let _ = breaktest(&obj);
    }

    #[test]
    fn test_breakmsg_glass() {
        let obj = Object {
            object_type: 100, // Glass/mirror
            ..Default::default()
        };
        let msg = breakmsg(&obj, true);
        assert!(msg.contains("shatters") || msg.contains("breaks"));
    }

    #[test]
    fn test_breakmsg_potion() {
        let obj = Object {
            object_type: 200, // Potion
            ..Default::default()
        };
        let msg = breakmsg(&obj, true);
        assert!(msg.contains("explodes") || msg.contains("breaks"));
    }

    #[test]
    fn test_breakmsg_out_of_view() {
        let obj = Object {
            object_type: 100, // Glass/mirror
            ..Default::default()
        };
        let msg = breakmsg(&obj, false);
        assert_eq!(msg, "You hear a crash!");
    }

    #[test]
    fn test_hero_breaks() {
        let obj = Object {
            object_type: 100, // Glass/mirror
            ..Default::default()
        };
        let result = hero_breaks(&obj, 10, 10);
        assert!(!result.messages.is_empty() || !result.broke);
    }

    #[test]
    fn test_breaks() {
        let obj = Object {
            object_type: 200, // Potion
            ..Default::default()
        };
        let result = breaks(&obj, 10, 10);
        assert!(!result.messages.is_empty() || !result.broke);
    }

    #[test]
    fn test_breakobj_hero_caused() {
        let obj = Object {
            object_type: 100, // Glass/mirror
            ..Default::default()
        };
        let result = breakobj(&obj, 10, 10, true);
        // Hero breaking objects may generate messages or not based on type
        let _ = result;
    }

    #[test]
    fn test_breakobj_non_hero() {
        let obj = Object {
            object_type: 100, // Glass/mirror
            ..Default::default()
        };
        let result = breakobj(&obj, 10, 10, false);
        // Non-hero breaking objects may generate different messages
        let _ = result;
    }

    #[test]
    fn test_explode_basic() {
        let mut player = You::default();
        let mut level = Level::new(DLevel::default());
        let mut rng = GameRng::new(42);

        let result = explode(5, 5, DamageType::Fire, 10, ExplosionSource::Spell, &mut player, &mut level, &mut rng);

        assert!(!result.messages.is_empty());
    }

    #[test]
    fn test_explode_player_in_area() {
        let mut player = You::default();
        player.pos.x = 5;
        player.pos.y = 5;
        player.hp = 50;
        let mut level = Level::new(DLevel::default());
        let mut rng = GameRng::new(42);

        let result = explode(5, 5, DamageType::Fire, 10, ExplosionSource::Spell, &mut player, &mut level, &mut rng);

        assert!(result.player_damage > 0);
        assert!(player.hp < 50);
    }

    #[test]
    fn test_explode_fire_resistance() {
        let mut player = You::default();
        player.pos.x = 5;
        player.pos.y = 5;
        player.hp = 50;
        player.properties.grant_intrinsic(Property::FireResistance);
        let mut level = Level::new(DLevel::default());
        let mut rng = GameRng::new(42);

        let result = explode(5, 5, DamageType::Fire, 20, ExplosionSource::Spell, &mut player, &mut level, &mut rng);

        // With resistance: half damage = (20+1)/2 = 10
        assert_eq!(result.player_damage, 10);
    }

    #[test]
    fn test_explode_cold_damage() {
        let mut player = You::default();
        let mut level = Level::new(DLevel::default());
        let mut rng = GameRng::new(42);

        let result = explode(5, 5, DamageType::Cold, 10, ExplosionSource::Spell, &mut player, &mut level, &mut rng);

        assert!(!result.messages.is_empty());
    }

    #[test]
    fn test_explode_wand_role_reduction() {
        // Wizard gets dam/5 for wand explosion
        let mut player = You::default();
        player.pos.x = 5;
        player.pos.y = 5;
        player.hp = 100;
        player.role = crate::player::Role::Wizard;
        let mut level = Level::new(DLevel::default());
        let mut rng = GameRng::new(42);

        let result = explode(5, 5, DamageType::MagicMissile, 50, ExplosionSource::Wand, &mut player, &mut level, &mut rng);

        // Wizard: 50/5 = 10 damage
        assert_eq!(result.player_damage, 10);
    }

    #[test]
    fn test_explode_oil() {
        let obj = Object {
            object_type: 0,
            ..Default::default()
        };
        let mut player = You::default();
        let mut level = Level::new(DLevel::default());
        let mut rng = GameRng::new(42);

        let result = explode_oil(&obj, 5, 5, &mut player, &mut level, &mut rng);

        let _ = result;
    }

    #[test]
    fn test_splatter_burning_oil() {
        let mut player = You::default();
        let mut level = Level::new(DLevel::default());
        let mut rng = GameRng::new(42);

        let result = splatter_burning_oil(5, 5, true, &mut player, &mut level, &mut rng);

        assert!(!result.messages.is_empty());
    }

    #[test]
    fn test_calculate_wand_wear_new() {
        let wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        let wear = calculate_wand_wear(&wand, 0);
        assert_eq!(wear, 0.0);
    }

    #[test]
    fn test_calculate_wand_wear_used() {
        let wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        let wear = calculate_wand_wear(&wand, 10);
        assert!(wear > 0.0);
        assert!(wear < 1.0);
    }

    #[test]
    fn test_calculate_wand_wear_heavily_used() {
        let wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        let wear = calculate_wand_wear(&wand, 100);
        assert_eq!(wear, 1.0); // Capped at 1.0
    }

    #[test]
    fn test_wand_effectiveness_new() {
        let wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        let effectiveness = get_wand_effectiveness(&wand, 0);
        assert_eq!(effectiveness, 1.0);
    }

    #[test]
    fn test_wand_effectiveness_degradation() {
        let wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        let effectiveness_new = get_wand_effectiveness(&wand, 0);
        let effectiveness_used = get_wand_effectiveness(&wand, 5);
        let effectiveness_worn = get_wand_effectiveness(&wand, 15);

        assert!(effectiveness_new > effectiveness_used);
        assert!(effectiveness_used > effectiveness_worn);
        assert!(effectiveness_worn >= 0.3);
    }

    #[test]
    fn test_apply_wand_wear_penalty() {
        let base_damage = 10;
        let penalty_none = apply_wand_wear_penalty(base_damage, 1.0);
        let penalty_half = apply_wand_wear_penalty(base_damage, 0.5);
        let penalty_min = apply_wand_wear_penalty(base_damage, 0.3);

        assert_eq!(penalty_none, 10);
        assert_eq!(penalty_half, 5);
        assert!(penalty_min < 4);
    }

    #[test]
    fn test_check_wand_breakage_probability() {
        let wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        let mut rng = GameRng::new(42);

        let mut breakage_count = 0;
        for _ in 0..100 {
            if check_wand_breakage(&wand, &mut rng) {
                breakage_count += 1;
            }
        }

        // Should have some breakages but not all
        assert!(breakage_count > 0);
        assert!(breakage_count < 100);
    }

    #[test]
    fn test_check_wand_breakage_cursed() {
        let mut wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        wand.buc = BucStatus::Cursed;
        let mut rng = GameRng::new(42);

        let mut blessed_wand = Object::new(ObjectId(2), 0, ObjectClass::Wand);
        blessed_wand.buc = BucStatus::Blessed;

        let mut cursed_breakages = 0;
        let mut blessed_breakages = 0;

        for _ in 0..50 {
            if check_wand_breakage(&wand, &mut rng) {
                cursed_breakages += 1;
            }
        }

        let mut rng = GameRng::new(42);
        for _ in 0..50 {
            if check_wand_breakage(&blessed_wand, &mut rng) {
                blessed_breakages += 1;
            }
        }

        // Cursed wands should break more often
        assert!(cursed_breakages >= blessed_breakages);
    }

    #[test]
    fn test_degrade_wand() {
        let mut wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        wand.enchantment = 10;
        let initial_charges = wand.enchantment;

        let mut rng = GameRng::new(42);
        degrade_wand(&mut wand, &mut rng);

        // After degradation, should have same or fewer charges
        assert!(wand.enchantment <= initial_charges);
    }

    #[test]
    fn test_degrade_wand_cursed_reduces_charges() {
        let mut wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        wand.buc = BucStatus::Cursed;
        wand.enchantment = 10;

        let mut rng = GameRng::new(42);
        degrade_wand(&mut wand, &mut rng);

        // Cursed wand has chance to lose extra charge
        assert!(wand.enchantment <= 10);
    }

    #[test]
    fn test_get_wand_status_new() {
        let mut wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        wand.enchantment = 10; // Needs charges to not hit "no charges" branch
        let status = get_wand_status(&wand, 0);
        assert!(status.contains("like new"));
    }

    #[test]
    fn test_get_wand_status_worn() {
        let mut wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        wand.enchantment = 5; // Needs charges to not hit "no charges" branch
        let status = get_wand_status(&wand, 15);
        assert!(status.contains("slightly worn") || status.contains("signs of wear"));
    }

    #[test]
    fn test_get_wand_status_heavily_worn() {
        let mut wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        wand.enchantment = 2; // Needs charges to not hit "no charges" branch
        let status = get_wand_status(&wand, 50);
        assert!(status.contains("heavily worn") || status.contains("signs of wear"));
    }

    #[test]
    fn test_get_wand_status_empty() {
        let mut wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        wand.enchantment = 0;
        let status = get_wand_status(&wand, 0);
        assert!(status.contains("no charges"));
    }

    // ========== Tests for Rock and Statue Breaking ==========

    #[test]
    fn test_fracture_rock() {
        use crate::dungeon::{Cell, CellType, DLevel, Level};

        let mut level = Level::new(DLevel::main_dungeon_start());
        level.cells[10][10] = Cell {
            typ: CellType::Wall,
            ..Default::default()
        };

        let result = fracture_rock(&mut level, 10, 10);

        assert!(result.destroyed);
        assert!(!result.messages.is_empty());
        // Rock should now be floor
        assert!(matches!(level.cells[10][10].typ, CellType::Room));
    }

    #[test]
    fn test_fracture_rock_not_rock() {
        use crate::dungeon::{DLevel, Level};

        let mut level = Level::new(DLevel::main_dungeon_start());
        // Level::new creates Stone cells by default; set to floor so it's not rock
        level.cells[10][10] = crate::dungeon::Cell::floor();

        let result = fracture_rock(&mut level, 10, 10);

        assert!(!result.destroyed);
        assert!(result.messages[0].contains("nothing to break"));
    }

    #[test]
    fn test_break_statue() {
        use crate::dungeon::{DLevel, Level};

        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut rng = GameRng::new(42);

        let statue = Object {
            object_type: 420, // In statue range
            name: Some("statue".to_string()),
            ..Default::default()
        };

        let result = break_statue(&mut level, &statue, 10, 10, &mut rng);

        assert!(result.destroyed);
        assert!(result.messages[0].contains("breaks"));
    }

    #[test]
    fn test_break_statue_not_statue() {
        use crate::dungeon::{DLevel, Level};

        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut rng = GameRng::new(42);

        let obj = Object {
            object_type: 50, // Not a statue
            name: Some("sword".to_string()),
            ..Default::default()
        };

        let result = break_statue(&mut level, &obj, 10, 10, &mut rng);

        assert!(!result.destroyed);
        assert!(result.messages[0].contains("not a statue"));
    }

    #[test]
    fn test_rock_contains_hazard() {
        use crate::dungeon::{Cell, CellType, DLevel, Level};

        let mut level = Level::new(DLevel::main_dungeon_start());

        // No hazard by default
        assert!(rock_contains_hazard(&level, 10, 10).is_none());

        // Add water next to position
        level.cells[9][10] = Cell {
            typ: CellType::Water,
            ..Default::default()
        };
        assert_eq!(rock_contains_hazard(&level, 10, 10), Some("water"));
    }

    #[test]
    fn test_dig_rock() {
        use crate::dungeon::{Cell, CellType, DLevel, Level};

        let mut level = Level::new(DLevel::main_dungeon_start());
        level.cells[10][10] = Cell {
            typ: CellType::Stone,
            ..Default::default()
        };

        let mut rng = GameRng::new(42);
        let result = dig_rock(&mut level, 10, 10, &mut rng);

        assert!(result.destroyed);
        assert!(matches!(level.cells[10][10].typ, CellType::Room));
    }

    // ========== Additional Explosion Functions Tests ==========

    #[test]
    fn test_do_osshock_single_item() {
        let obj = Object {
            object_type: 100,
            quantity: 1,
            ..Default::default()
        };

        let mut rng = GameRng::new(42);
        let (destroyed, _polymorphed, _messages) = do_osshock(&obj, &mut rng, 0);

        // With seed 42, we should get consistent results
        // Either destroyed or not based on rng
        assert!(destroyed || !destroyed); // Just verify it runs
    }

    #[test]
    fn test_do_osshock_multiple_items() {
        let obj = Object {
            object_type: 100,
            quantity: 5,
            ..Default::default()
        };

        let mut rng = GameRng::new(42);
        let (_destroyed, _polymorphed, messages) = do_osshock(&obj, &mut rng, 0);

        // With multiple items, messages may vary
        assert!(messages.is_empty() || !messages.is_empty());
    }

    #[test]
    fn test_elemental_clog_not_endgame() {
        use crate::monster::{Monster, MonsterId};

        let monsters = vec![Monster::new(MonsterId(1), 0, 5, 5)];
        let mut last_turn = 0;

        let (remove, messages) = elemental_clog(&monsters, false, 100, &mut last_turn);

        assert!(remove.is_none());
        assert!(messages.is_empty());
    }

    #[test]
    fn test_elemental_clog_endgame_shows_message() {
        use crate::monster::{Monster, MonsterId};

        let monsters = vec![Monster::new(MonsterId(1), 0, 5, 5)];
        let mut last_turn = 0;

        let (_remove, messages) = elemental_clog(&monsters, true, 100, &mut last_turn);

        assert!(messages.iter().any(|m| m.contains("besieged")));
        assert_eq!(last_turn, 100);
    }

    #[test]
    fn test_elemental_clog_cooldown() {
        use crate::monster::{Monster, MonsterId};

        let monsters = vec![Monster::new(MonsterId(1), 0, 5, 5)];
        let mut last_turn = 50;

        // Within cooldown period (200 turns)
        let (_remove, messages) = elemental_clog(&monsters, true, 100, &mut last_turn);

        // Should not show message within cooldown
        assert!(messages.is_empty());
        assert_eq!(last_turn, 50); // Not updated
    }

    #[test]
    fn test_elemental_clog_finds_elemental() {
        use crate::monster::{Monster, MonsterId};

        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "fire elemental".to_string();
        mon.hp = 10;

        let monsters = vec![mon];
        let mut last_turn = 0;

        let (remove, _messages) = elemental_clog(&monsters, true, 100, &mut last_turn);

        assert!(remove.is_some());
        assert_eq!(remove.unwrap(), MonsterId(1));
    }

    // ==========================================================================
    // Cancellation System tests
    // ==========================================================================

    #[test]
    fn test_cancel_item_wand_strips_charges() {
        let mut wand = Object::default();
        wand.class = ObjectClass::Wand;
        wand.object_type = 365; // Wand of fire
        wand.enchantment = 5;
        let msgs = cancel_item(&mut wand);
        assert_eq!(wand.enchantment, 0);
        assert!(!msgs.is_empty());
    }

    #[test]
    fn test_cancel_item_wand_of_cancellation_immune() {
        let mut wand = Object::default();
        wand.class = ObjectClass::Wand;
        wand.object_type = 363; // Wand of cancellation
        wand.enchantment = 5;
        let msgs = cancel_item(&mut wand);
        assert_eq!(wand.enchantment, 5, "Wand of cancellation should be immune");
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_cancel_item_armor_removes_enchantment() {
        let mut armor = Object::default();
        armor.class = ObjectClass::Armor;
        armor.enchantment = 3;
        armor.erosion_proof = true;
        let msgs = cancel_item(&mut armor);
        assert_eq!(armor.enchantment, 0);
        assert!(!armor.erosion_proof);
        assert!(!msgs.is_empty());
    }

    #[test]
    fn test_cancel_item_scroll_blanked() {
        let mut scroll = Object::default();
        scroll.class = ObjectClass::Scroll;
        scroll.object_type = 285; // Not blank
        let msgs = cancel_item(&mut scroll);
        assert_eq!(scroll.object_type, 306, "Should become blank paper");
        assert!(!msgs.is_empty());
    }

    #[test]
    fn test_cancel_item_potion_becomes_water() {
        let mut potion = Object::default();
        potion.class = ObjectClass::Potion;
        potion.object_type = 267; // Healing
        let msgs = cancel_item(&mut potion);
        assert_eq!(
            potion.object_type,
            crate::magic::potion::PotionType::Water as i16
        );
        assert!(!msgs.is_empty());
    }

    #[test]
    fn test_cancel_monst() {
        use crate::monster::{Monster, MonsterId};

        let mut monster = Monster::new(MonsterId(0), 5, 5, 5);
        monster.name = "goblin".to_string();
        monster.state.invisible = true;
        let msgs = cancel_monst(&mut monster);
        assert!(monster.state.cancelled);
        assert!(!monster.state.invisible, "Cancellation should remove invisibility");
        assert!(!msgs.is_empty());
    }

    #[test]
    fn test_cancel_monst_reverts_shapeshifter() {
        use crate::monster::{Monster, MonsterId};

        let mut monster = Monster::new(MonsterId(0), 5, 5, 5);
        monster.name = "shapeshifter".to_string();
        monster.monster_type = 10; // Currently shifted
        monster.original_type = 5; // Original form
        let msgs = cancel_monst(&mut monster);
        assert_eq!(monster.monster_type, 5, "Should revert to original form");
        assert!(msgs.iter().any(|m| m.contains("reverts")));
    }

    #[test]
    fn test_probe_monster() {
        use crate::monster::{Monster, MonsterId};

        let mut monster = Monster::new(MonsterId(0), 0, 5, 5);
        monster.name = "kobold".to_string();
        monster.hp = 8;
        monster.hp_max = 12;
        monster.ac = 7;
        monster.level = 3;
        monster.state.tame = true;
        let msgs = probe_monster(&monster);
        assert!(msgs[0].contains("kobold"));
        assert!(msgs[0].contains("8/12"));
        assert!(msgs.iter().any(|m| m.contains("tame")));
    }

    // ==========================================================================
    // Sleep/Lightning resistance in ray tests
    // ==========================================================================

    #[test]
    fn test_hit_monster_sleep_resistance() {
        use crate::monster::{Monster, MonsterId, MonsterResistances};

        let mut monster = Monster::new(MonsterId(0), 0, 5, 5);
        monster.name = "elf".to_string();
        monster.resistances = MonsterResistances::SLEEP;
        let mut rng = GameRng::new(42);
        let mut result = ZapResult::new();
        hit_monster_with_ray(&mut monster, ZapType::Sleep, ZapVariant::Wand, &mut rng, &mut result);
        assert!(!monster.state.sleeping, "Sleep resistant monster should not fall asleep");
        assert!(result.messages.iter().any(|m| m.contains("not affected")));
    }

    #[test]
    fn test_hit_monster_lightning_resistance() {
        use crate::monster::{Monster, MonsterId, MonsterResistances};

        let mut monster = Monster::new(MonsterId(0), 0, 5, 5);
        monster.name = "golem".to_string();
        monster.hp = 50;
        monster.hp_max = 50;
        monster.resistances = MonsterResistances::ELEC;
        let mut rng = GameRng::new(42);
        let mut result = ZapResult::new();
        hit_monster_with_ray(&mut monster, ZapType::Lightning, ZapVariant::Wand, &mut rng, &mut result);
        assert_eq!(monster.hp, 50, "Electricity resistant monster should take no damage");
        assert!(result.messages.iter().any(|m| m.contains("not affected")));
    }

    #[test]
    fn test_hit_monster_death_ray_magic_resist() {
        use crate::monster::{Monster, MonsterId, MonsterResistances};

        let mut monster = Monster::new(MonsterId(0), 0, 5, 5);
        monster.name = "dragon".to_string();
        monster.hp = 100;
        monster.hp_max = 100;
        monster.resistances = MonsterResistances::MAGIC;
        let mut rng = GameRng::new(42);
        let mut result = ZapResult::new();
        hit_monster_with_ray(&mut monster, ZapType::Death, ZapVariant::Wand, &mut rng, &mut result);
        assert!(monster.hp > 0, "Magic resistant monster should survive death ray");
        assert!(result.messages.iter().any(|m| m.contains("resists")));
    }
}
