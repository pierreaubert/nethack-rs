//! Wand and ray zapping effects (zap.c)
//!
//! Handles wand zapping, spell rays, and breath weapons.

use crate::combat::DamageType;
use crate::dungeon::Level;
use crate::monster::MonsterId;
use crate::object::Object;
use crate::player::You;
use crate::rng::GameRng;

/// Ray/zap type indices (matches ZT_* from zap.c)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

impl ZapResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            consumed: false,
            killed: Vec::new(),
            player_died: false,
            player_damage: 0,
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
        ZapDirection::Down => {
            zap_down(zap_type, ZapVariant::Wand, player, level, rng, &mut result)
        }
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
            result.messages.push("A light shines around you.".to_string());
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
                result.messages.push(format!(
                    "The {} disappears!",
                    monster.name
                ));
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
                result.messages.push(format!(
                    "The {} seems to move more slowly.",
                    monster.name
                ));
            }
        }
        360 => {
            // WandOfSpeed
            if direction == ZapDirection::Self_ {
                player
                    .properties
                    .set_timeout(crate::player::Property::Speed, 50);
                result.messages.push("You feel yourself moving faster.".to_string());
            } else if let Some(monster) = level.monster_at_mut(tx, ty) {
                monster.state.hasted = true;
                result.messages.push(format!("The {} seems to speed up.", monster.name));
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
                player.attr_current.modify(crate::player::Attribute::Strength, stat_change);
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
            result.messages.push("You may wish for an object.".to_string());
            // Wishing requires UI interaction to get player's wish
            // The caller should prompt the player and create the wished-for object
        }
        363 => {
            // WandOfCancellation
            if let Some(monster) = level.monster_at_mut(tx, ty) {
                monster.state.cancelled = true;
                result.messages.push(format!("The {} shudders!", monster.name));
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
            result.messages.push("A cloud of smoke descends.".to_string());
        }
        ZapType::Cold => {
            result.messages.push("Ice shards fall on you.".to_string());
            let damage = rng.dice(2, 6) as i32;
            if !player.properties.has(crate::player::Property::ColdResistance) {
                player.hp -= damage;
                result.player_damage = damage;
                result.messages.push(format!("You take {} cold damage.", damage));
            }
        }
        ZapType::Lightning => {
            result.messages.push("The ceiling crackles.".to_string());
        }
        ZapType::Death | ZapType::Sleep | ZapType::MagicMissile | ZapType::PoisonGas | ZapType::Acid => {
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
            result
                .messages
                .push("The floor is scorched.".to_string());
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
                result
                    .messages
                    .push(format!("You are hit by a magic missile for {} damage!", damage));
            } else {
                result.messages.push("The magic missile bounces off!".to_string());
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
                result
                    .messages
                    .push("You shudder momentarily.".to_string());
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
                result
                    .messages
                    .push("You feel a mild tingle.".to_string());
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
                    player.attr_current.set(crate::player::Attribute::Strength, str - 1);
                    result
                        .messages
                        .push("You feel weaker.".to_string());
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
                result.messages.push("The acid doesn't affect you.".to_string());
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

    // Trace the ray
    for _ in 0..20 {
        // Max range
        x += dx;
        y += dy;

        if !level.is_valid_pos(x, y) {
            break;
        }

        // Check for walls
        let cell = level.cell(x as usize, y as usize);
        if cell.typ.is_wall() {
            result.messages.push(format!(
                "The {} hits the wall.",
                zap_type.name(variant)
            ));
            break;
        }

        // Check for monsters
        if let Some(monster) = level.monster_at_mut(x, y) {
            hit_monster_with_ray(monster, zap_type, variant, rng, result);
            if result.killed.contains(&monster.id) {
                // Monster died, ray continues (for some effects)
            }
            // Most rays stop at first monster
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
                result.messages.push(format!(
                    "The {} is not affected by the flames.",
                    name
                ));
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
                result.messages.push(format!(
                    "The {} is not affected by the cold.",
                    name
                ));
            } else {
                monster.hp -= damage;
                result.messages.push(format!(
                    "The {} is frozen for {} damage!",
                    name, damage
                ));
            }
        }
        ZapType::Sleep => {
            monster.state.sleeping = true;
            result.messages.push(format!("The {} falls asleep!", name));
        }
        ZapType::Death => {
            // Monsters with disintegration resistance are immune to death ray
            // Higher level monsters have a chance to resist based on level
            let resist_chance = (monster.level as u32) * 3;
            if monster.resists_disint() || rng.percent(resist_chance) {
                result.messages.push(format!("The {} resists!", name));
            } else {
                monster.hp = 0;
                result.messages.push(format!("The {} dies!", name));
            }
        }
        ZapType::Lightning => {
            let damage = rng.dice(6, 6) as i32;
            monster.hp -= damage;
            result.messages.push(format!(
                "The {} is shocked for {} damage!",
                name, damage
            ));
        }
        ZapType::PoisonGas => {
            let damage = rng.dice(2, 6) as i32;
            if monster.resists_poison() {
                result.messages.push(format!(
                    "The {} is not affected by the poison.",
                    name
                ));
            } else {
                monster.hp -= damage;
                result.messages.push(format!(
                    "The {} is poisoned for {} damage!",
                    name, damage
                ));
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
fn dig_ray(
    start_x: i8,
    start_y: i8,
    dx: i8,
    dy: i8,
    level: &mut Level,
    result: &mut ZapResult,
) {
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
        if cell.typ.is_wall() || cell.typ == crate::dungeon::CellType::Stone || cell.typ == crate::dungeon::CellType::Door {
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
                result.messages.push(format!("The {} vanishes!", monster.name));
            }
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
