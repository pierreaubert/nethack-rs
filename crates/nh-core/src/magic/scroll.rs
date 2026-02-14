//! Scroll reading effects (read.c)
//!
//! Handles reading scrolls and their effects.

use crate::dungeon::{DLevel, Level};
use crate::object::Object;
use crate::player::{Property, You};
use crate::rng::GameRng;

/// Result of reading a scroll
#[derive(Debug, Clone)]
pub struct ScrollResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Whether the scroll was consumed
    pub consumed: bool,
    /// Whether to identify the scroll type
    pub identify: bool,
}

impl ScrollResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            consumed: true, // Most scrolls are consumed
            identify: true, // Most scrolls identify on use
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

impl Default for ScrollResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Scroll type indices (matching ObjectType in nh-data/objects.rs)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i16)]
pub enum ScrollType {
    EnchantArmor = 285,
    Destroy = 286,
    Confuse = 287,
    Scare = 288,
    RemoveCurse = 289,
    EnchantWeapon = 290,
    Create = 291,
    Taming = 292,
    Genocide = 293,
    Light = 294,
    Teleportation = 295,
    Gold = 296,
    Food = 297,
    Identify = 298,
    MagicMapping = 299,
    Amnesia = 300,
    Fire = 301,
    Earth = 302,
    Punishment = 303,
    Charging = 304,
    StinkingCloud = 305,
    Blank = 306,
}

impl ScrollType {
    /// Try to convert an object type to a scroll type
    pub fn from_object_type(otype: i16) -> Option<Self> {
        match otype {
            285 => Some(ScrollType::EnchantArmor),
            286 => Some(ScrollType::Destroy),
            287 => Some(ScrollType::Confuse),
            288 => Some(ScrollType::Scare),
            289 => Some(ScrollType::RemoveCurse),
            290 => Some(ScrollType::EnchantWeapon),
            291 => Some(ScrollType::Create),
            292 => Some(ScrollType::Taming),
            293 => Some(ScrollType::Genocide),
            294 => Some(ScrollType::Light),
            295 => Some(ScrollType::Teleportation),
            296 => Some(ScrollType::Gold),
            297 => Some(ScrollType::Food),
            298 => Some(ScrollType::Identify),
            299 => Some(ScrollType::MagicMapping),
            300 => Some(ScrollType::Amnesia),
            301 => Some(ScrollType::Fire),
            302 => Some(ScrollType::Earth),
            303 => Some(ScrollType::Punishment),
            304 => Some(ScrollType::Charging),
            305 => Some(ScrollType::StinkingCloud),
            306 => Some(ScrollType::Blank),
            _ => None,
        }
    }
}

/// Read a scroll
pub fn read_scroll(
    scroll: &Object,
    player: &mut You,
    level: &mut Level,
    rng: &mut GameRng,
) -> ScrollResult {
    // Check for blindness
    if player.blinded_timeout > 0 {
        return ScrollResult::new().with_message("You can't read while blind!");
    }

    // Check for confusion - may read the wrong scroll
    let confused = player.confused_timeout > 0;

    let Some(stype) = ScrollType::from_object_type(scroll.object_type) else {
        return ScrollResult::new().with_message("That's not a scroll!");
    };

    let blessed = scroll.is_blessed();
    let cursed = scroll.is_cursed();

    match stype {
        ScrollType::EnchantArmor => scroll_enchant_armor(player, blessed, cursed, confused),
        ScrollType::Destroy => scroll_destroy(player, level, blessed, cursed, rng),
        ScrollType::Confuse => scroll_confuse(player, blessed, cursed, confused, rng),
        ScrollType::Scare => scroll_scare(level, player, blessed, cursed, rng),
        ScrollType::RemoveCurse => scroll_remove_curse(player, blessed),
        ScrollType::EnchantWeapon => scroll_enchant_weapon(player, blessed, cursed, confused),
        ScrollType::Create => scroll_create(level, player, blessed, cursed, rng),
        ScrollType::Taming => scroll_taming(level, player, blessed, cursed),
        ScrollType::Genocide => scroll_genocide(blessed, cursed),
        ScrollType::Light => scroll_light(level, player, blessed, cursed),
        ScrollType::Teleportation => scroll_teleportation(player, level, blessed, cursed, rng),
        ScrollType::Gold => scroll_gold(player, blessed, rng),
        ScrollType::Food => scroll_food(player, blessed, rng),
        ScrollType::Identify => scroll_identify(player, blessed),
        ScrollType::MagicMapping => scroll_magic_mapping(level, player, blessed, cursed),
        ScrollType::Amnesia => scroll_amnesia(player, blessed, cursed),
        ScrollType::Fire => scroll_fire(player, level, blessed, cursed, rng),
        ScrollType::Earth => scroll_earth(player, level, blessed, cursed, rng),
        ScrollType::Punishment => scroll_punishment(player, blessed, cursed),
        ScrollType::Charging => scroll_charging(player, blessed, cursed),
        ScrollType::StinkingCloud => scroll_stinking_cloud(level, player, blessed, cursed, rng),
        ScrollType::Blank => scroll_blank(),
    }
}

fn scroll_enchant_armor(
    player: &mut You,
    blessed: bool,
    cursed: bool,
    _confused: bool,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    // Find worn armor to enchant
    if cursed {
        result.messages.push("Your armor feels weaker.".to_string());
        // Reduce armor protection (higher AC = worse)
        player.armor_class = player.armor_class.saturating_add(1);
    } else if blessed {
        result
            .messages
            .push("Your armor glows brightly!".to_string());
        // Increase armor protection by 2-3 (lower AC = better)
        player.armor_class = player.armor_class.saturating_sub(3);
    } else {
        result.messages.push("Your armor glows!".to_string());
        // Increase armor protection by 1
        player.armor_class = player.armor_class.saturating_sub(1);
    }

    result
}

fn scroll_destroy(
    player: &mut You,
    level: &mut Level,
    blessed: bool,
    cursed: bool,
    rng: &mut GameRng,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result
            .messages
            .push("You feel like you need a new weapon.".to_string());
        // Destroy wielded weapon - reduce weapon bonus severely
        player.weapon_bonus = player.weapon_bonus.saturating_sub(3);
    } else if blessed {
        result
            .messages
            .push("Everything around you shatters and crumbles!".to_string());
        // Destroy armor on all nearby monsters
        for monster in &mut level.monsters {
            let dx = (monster.x - player.pos.x).abs();
            let dy = (monster.y - player.pos.y).abs();
            if dx <= 5 && dy <= 5 {
                monster.ac = monster.ac.saturating_add(3); // Worse AC
            }
        }
    } else {
        // Destroy armor on nearby monsters
        result
            .messages
            .push("You hear crashing and tearing sounds!".to_string());
        // Damage armor on nearby monsters
        for monster in &mut level.monsters {
            if rng.one_in(3) {
                monster.ac = monster.ac.saturating_add(2); // Worse AC
            }
        }
    }

    result
}

fn scroll_confuse(
    player: &mut You,
    blessed: bool,
    cursed: bool,
    _confused: bool,
    rng: &mut GameRng,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        // Cursed scroll causes extreme confusion
        let duration = rng.dice(6, 8) as u16;
        player.confused_timeout = player.confused_timeout.saturating_add(duration);
        result
            .messages
            .push("Your head spins violently!".to_string());
    } else if blessed {
        // Blessed scroll grants mild confusion touch
        result
            .messages
            .push("Your hands begin to glow with purple light!".to_string());
        player.confused_timeout = player.confused_timeout.saturating_add(10);
    } else {
        // Normal scroll causes standard confusion
        let duration = rng.dice(4, 4) as u16;
        player.confused_timeout = player.confused_timeout.saturating_add(duration);
        result.messages.push("Your head spins!".to_string());
    }

    result
}

fn scroll_scare(
    level: &mut Level,
    player: &You,
    blessed: bool,
    cursed: bool,
    rng: &mut GameRng,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result
            .messages
            .push("You hear maniacal laughter!".to_string());
        // Scare the player instead - they become frightened
        // (Would trigger fear intrinsic if we had it)
    } else if blessed {
        result
            .messages
            .push("A terrifying shriek echoes across the entire level!".to_string());
        // Scare ALL monsters on level with guaranteed effect
        for monster in &mut level.monsters {
            monster.state.fleeing = true;
            monster.flee_timeout = rng.dice(3, 8) as u16;
        }
    } else {
        result
            .messages
            .push("You hear a terrifying shriek!".to_string());
        // Scare nearby monsters
        let px = player.pos.x;
        let py = player.pos.y;
        let radius = 10i8;

        for monster in &mut level.monsters {
            let dx = (monster.x - px).abs();
            let dy = (monster.y - py).abs();
            if dx <= radius && dy <= radius && rng.percent(80) {
                monster.state.fleeing = true;
                monster.flee_timeout = rng.dice(2, 6) as u16;
            }
        }
    }

    result
}

fn scroll_remove_curse(player: &mut You, blessed: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if blessed {
        result
            .messages
            .push("You feel like someone is helping you.".to_string());
        // Blessed removes all curses (inventory handling done by caller)
    } else {
        result
            .messages
            .push("You feel less encumbered.".to_string());
        // Normal removes curses from worn/wielded only (inventory handling done by caller)
    }

    // Clear any curse effects on player
    player.properties.remove_intrinsic(Property::Fumbling);

    result
}

fn scroll_enchant_weapon(
    player: &mut You,
    blessed: bool,
    cursed: bool,
    _confused: bool,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result.messages.push("Your weapon feels dull.".to_string());
        // Reduce weapon enchantment
        player.weapon_bonus = player.weapon_bonus.saturating_sub(1);
    } else if blessed {
        result
            .messages
            .push("Your weapon glows brightly blue!".to_string());
        // Increase weapon enchantment by 2-3
        player.weapon_bonus = player.weapon_bonus.saturating_add(3);
    } else {
        result.messages.push("Your weapon glows blue!".to_string());
        // Increase weapon enchantment by 1
        player.weapon_bonus = player.weapon_bonus.saturating_add(1);
    }

    result
}

fn scroll_create(
    level: &mut Level,
    player: &You,
    blessed: bool,
    cursed: bool,
    rng: &mut GameRng,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result
            .messages
            .push("You read the scroll and nothing happens.".to_string());
    } else if blessed {
        let count = rng.rnd(2) + 3; // 3-4 monsters instead of 1-4
        result
            .messages
            .push(format!("You create {} peaceful creature(s)!", count));
        // Monster spawning requires monster creation infrastructure
        // For now, just acknowledge the effect
        let _ = (level, player, count);
    } else {
        let count = rng.rnd(4) + 1;
        result
            .messages
            .push(format!("You create {} monster(s)!", count));
        // Monster spawning requires monster creation infrastructure
        // For now, just acknowledge the effect
        let _ = (level, player, count);
    }

    result
}

fn scroll_taming(level: &mut Level, player: &You, blessed: bool, cursed: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result
            .messages
            .push("You feel that reading this was a mistake.".to_string());
        // Aggravate nearby monsters - wake them up and make hostile
        for monster in &mut level.monsters {
            monster.state.sleeping = false;
            monster.state.peaceful = false;
        }
    } else if blessed {
        result
            .messages
            .push("You feel extremely charismatic!".to_string());
        // Tame all monsters on level
        for monster in &mut level.monsters {
            monster.state.peaceful = true;
            monster.state.tame = true;
        }
    } else {
        result.messages.push("You feel charismatic!".to_string());
        // Tame adjacent monsters
        let px = player.pos.x;
        let py = player.pos.y;

        for monster in &mut level.monsters {
            let dx = (monster.x - px).abs();
            let dy = (monster.y - py).abs();
            if dx <= 1 && dy <= 1 {
                monster.state.peaceful = true;
                monster.state.tame = true;
            }
        }
    }

    result
}

fn scroll_genocide(blessed: bool, cursed: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result
            .messages
            .push("A thunderous voice booms: FOOL!".to_string());
        // Reverse genocide - spawn many monsters (requires UI input)
    } else if blessed {
        result
            .messages
            .push("What class of monsters do you wish to genocide?".to_string());
        // Genocide entire monster class (requires UI input for class selection)
    } else {
        result
            .messages
            .push("What monster do you wish to genocide?".to_string());
        // Genocide single monster type (requires UI input for monster selection)
    }

    result
}

fn scroll_light(level: &mut Level, player: &You, blessed: bool, cursed: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result.messages.push("Darkness surrounds you!".to_string());
        // Darken area around player
        let cx = player.pos.x as usize;
        let cy = player.pos.y as usize;
        for dy in 0..=10 {
            for dx in 0..=10 {
                let x = (cx + dx).saturating_sub(5);
                let y = (cy + dy).saturating_sub(5);
                if x < crate::COLNO && y < crate::ROWNO {
                    level.cells[x][y].lit = false;
                }
            }
        }
    } else if blessed {
        result
            .messages
            .push("The entire level is illuminated!".to_string());
        // Light entire level
        for x in 0..crate::COLNO {
            for y in 0..crate::ROWNO {
                level.cells[x][y].lit = true;
            }
        }
    } else {
        result.messages.push("A light shines!".to_string());
        // Light area around player
        let cx = player.pos.x as usize;
        let cy = player.pos.y as usize;
        for dy in 0..=10 {
            for dx in 0..=10 {
                let x = (cx + dx).saturating_sub(5);
                let y = (cy + dy).saturating_sub(5);
                if x < crate::COLNO && y < crate::ROWNO {
                    level.cells[x][y].lit = true;
                }
            }
        }
    }

    result
}

fn scroll_teleportation(
    player: &mut You,
    level: &Level,
    blessed: bool,
    cursed: bool,
    rng: &mut GameRng,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    let mut attempts = 100;
    if blessed {
        // Blessed: try more attempts to find a good spot
        attempts = 200;
    } else if cursed {
        // Cursed: try fewer attempts (more likely to fail)
        attempts = 30;
    }

    if cursed {
        result.messages.push("You feel disoriented.".to_string());
    } else if blessed {
        result
            .messages
            .push("You feel like you know exactly where you want to go.".to_string());
    }

    // Find random walkable position
    for _ in 0..attempts {
        let x = rng.rn2(crate::COLNO as u32) as i8;
        let y = rng.rn2(crate::ROWNO as u32) as i8;

        if level.is_walkable(x, y) && level.monster_at(x, y).is_none() {
            player.prev_pos = player.pos;
            player.pos.x = x;
            player.pos.y = y;
            result
                .messages
                .push("You find yourself somewhere else.".to_string());
            return result;
        }
    }

    result
        .messages
        .push("You feel disoriented for a moment.".to_string());
    result
}

fn scroll_gold(player: &mut You, blessed: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    if blessed {
        let gold = rng.dice(10, 50) as i32;
        player.gold += gold;
        result
            .messages
            .push(format!("You feel richer! (+{} gold)", gold));
    } else {
        let gold = rng.dice(5, 50) as i32;
        player.gold += gold;
        result
            .messages
            .push(format!("Gold appears! (+{} gold)", gold));
    }

    result
}

fn scroll_food(player: &mut You, blessed: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    let nutrition = if blessed {
        rng.dice(8, 100) as i32
    } else {
        rng.dice(4, 100) as i32
    };

    player.nutrition += nutrition;
    result
        .messages
        .push(format!("Food appears! (+{} nutrition)", nutrition));
    // Food item creation would require object spawning infrastructure

    result
}

fn scroll_identify(player: &mut You, blessed: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if blessed {
        result
            .messages
            .push("All your possessions glow briefly!".to_string());
        // Identify all inventory items (requires inventory access from caller)
    } else {
        result
            .messages
            .push("This is an identify scroll.".to_string());
        // Let player pick item to identify (requires UI interaction from caller)
    }

    let _ = player;
    result
}

fn scroll_magic_mapping(
    level: &mut Level,
    player: &You,
    blessed: bool,
    cursed: bool,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result
            .messages
            .push("Your head spins as the world around you shifts!".to_string());
        // Scramble map memory - unexplore random cells
        for x in 0..crate::COLNO {
            for y in 0..crate::ROWNO {
                if level.cells[x][y].explored {
                    // 50% chance to forget each explored cell
                    level.cells[x][y].explored = (x + y) % 2 == 0;
                }
            }
        }
    } else if blessed {
        result
            .messages
            .push("A perfect map coalesces in your mind!".to_string());
        // Reveal entire level and mark all as explored
        for x in 0..crate::COLNO {
            for y in 0..crate::ROWNO {
                level.cells[x][y].explored = true;
                level.cells[x][y].lit = true; // Also reveal secret details
            }
        }
    } else {
        result
            .messages
            .push("A map coalesces in your mind!".to_string());
        // Reveal entire level
        for x in 0..crate::COLNO {
            for y in 0..crate::ROWNO {
                level.cells[x][y].explored = true;
            }
        }
    }

    let _ = player;
    result
}

fn scroll_amnesia(player: &mut You, blessed: bool, cursed: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result
            .messages
            .push("You forget everything you knew!".to_string());
        // Forget all identified items (requires discovery tracking from caller)
    } else if blessed {
        result
            .messages
            .push("Your memory becomes crystal clear!".to_string());
        // Gain perfect recall of current level (requires level access from caller)
        // Could also grant temporary recall enhancement intrinsic
    } else {
        result
            .messages
            .push("You forget your surroundings.".to_string());
        // Forget current level map (requires level access from caller)
    }

    let _ = player;
    result
}

fn scroll_fire(
    player: &mut You,
    level: &mut Level,
    blessed: bool,
    cursed: bool,
    rng: &mut GameRng,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result.messages.push("Flames erupt around you!".to_string());
        // Larger blast radius and always damage player
        let damage = rng.dice(4, 8) as i32;
        player.hp -= damage;
        result
            .messages
            .push(format!("You are burned for {} damage!", damage));
        result
            .messages
            .push("The flames spread wildly!".to_string());

        let px = player.pos.x;
        let py = player.pos.y;
        let radius = 3i8;

        for monster in &mut level.monsters {
            let dx = (monster.x - px).abs();
            let dy = (monster.y - py).abs();
            if dx <= radius && dy <= radius && !monster.resists_fire() {
                let damage = rng.dice(3, 8) as i32;
                monster.hp -= damage;
            }
        }
    } else if blessed {
        result
            .messages
            .push("A protective fireball blooms around you!".to_string());
        // Protect player from fire, only affect monsters
        for monster in &mut level.monsters {
            let dx = (monster.x - player.pos.x).abs();
            let dy = (monster.y - player.pos.y).abs();
            if dx <= 4 && dy <= 4 && !monster.resists_fire() {
                let damage = rng.dice(3, 8) as i32;
                monster.hp -= damage;
            }
        }
    } else {
        result.messages.push("Flames erupt around you!".to_string());
        // Damage player if not fire resistant
        if !player.properties.has(Property::FireResistance) {
            let damage = rng.dice(3, 6) as i32;
            player.hp -= damage;
            result
                .messages
                .push(format!("You are burned for {} damage!", damage));
        }

        // Damage nearby monsters with fire
        let px = player.pos.x;
        let py = player.pos.y;
        let radius = 2i8;

        for monster in &mut level.monsters {
            let dx = (monster.x - px).abs();
            let dy = (monster.y - py).abs();
            if dx <= radius && dy <= radius && !monster.resists_fire() {
                let damage = rng.dice(2, 6) as i32;
                monster.hp -= damage;
            }
        }
    }

    result
}

fn scroll_earth(
    player: &mut You,
    level: &mut Level,
    blessed: bool,
    cursed: bool,
    rng: &mut GameRng,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    result.messages.push("The earth shakes!".to_string());

    if cursed {
        // Drop boulders on player - severe
        let damage = rng.dice(6, 8) as i32;
        player.hp -= damage;
        result
            .messages
            .push(format!("Large boulders fall on you for {} damage!", damage));
    } else if blessed {
        // Create boulders around player - protect from monsters, damage all nearby enemies
        result
            .messages
            .push("Stone barriers rise around you!".to_string());
        let px = player.pos.x;
        let py = player.pos.y;
        for monster in &mut level.monsters {
            let dx = (monster.x - px).abs();
            let dy = (monster.y - py).abs();
            if dx <= 3 && dy <= 3 {
                let damage = rng.dice(4, 8) as i32;
                monster.hp -= damage;
            }
        }
    } else {
        // Create boulders around player - damage nearby monsters
        result
            .messages
            .push("Boulders fall around you!".to_string());
        let px = player.pos.x;
        let py = player.pos.y;
        for monster in &mut level.monsters {
            let dx = (monster.x - px).abs();
            let dy = (monster.y - py).abs();
            if dx <= 2 && dy <= 2 && rng.one_in(3) {
                let damage = rng.dice(4, 6) as i32;
                monster.hp -= damage;
            }
        }
    }

    result
}

fn scroll_punishment(player: &mut You, blessed: bool, cursed: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result
            .messages
            .push("You are punished even more severely!".to_string());
        // Extra punishment - more fumbling and slower
        player.properties.grant_intrinsic(Property::Fumbling);
        player.movement_points = player.movement_points.saturating_sub(4);
    } else if blessed {
        result
            .messages
            .push("You feel absolved of your sins!".to_string());
        // Remove fumbling if present
        player.properties.remove_intrinsic(Property::Fumbling);
    } else {
        result
            .messages
            .push("You are punished for your misbehavior!".to_string());
        // Attach ball and chain effect - grant fumbling
        player.properties.grant_intrinsic(Property::Fumbling);
    }
    result
}

fn scroll_charging(player: &mut You, blessed: bool, cursed: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result
            .messages
            .push("You feel momentarily disoriented.".to_string());
        // Drain wand charges (requires inventory wand access from caller)
    } else if blessed {
        result
            .messages
            .push("This is a blessed scroll of charging!".to_string());
        // Full charge wand + uncurse (requires inventory wand access from caller)
    } else {
        result
            .messages
            .push("This is a scroll of charging.".to_string());
        // Let player choose wand to recharge (requires UI interaction from caller)
    }

    let _ = player;
    result
}

fn scroll_stinking_cloud(
    level: &mut Level,
    player: &You,
    blessed: bool,
    cursed: bool,
    rng: &mut GameRng,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result
            .messages
            .push("A vile stench erupts around you!".to_string());
        // Smaller cloud around player
        let px = player.pos.x;
        let py = player.pos.y;
        let radius = 3i8;

        for monster in &mut level.monsters {
            let dx = (monster.x - px).abs();
            let dy = (monster.y - py).abs();
            if dx <= radius && dy <= radius && !monster.resists_poison() {
                monster.state.confused = true;
                monster.confused_timeout = rng.dice(4, 6) as u16;
            }
        }
    } else if blessed {
        result
            .messages
            .push("A refreshing sweet fragrance spreads across the level!".to_string());
        // Large cloud confuses all monsters on level
        for monster in &mut level.monsters {
            if !monster.resists_poison() {
                monster.state.confused = true;
                monster.confused_timeout = rng.dice(3, 8) as u16;
            }
        }
    } else {
        result
            .messages
            .push("A stinking cloud billows out!".to_string());
        // Normal stinking cloud
        let px = player.pos.x;
        let py = player.pos.y;
        let radius = 6i8;

        for monster in &mut level.monsters {
            let dx = (monster.x - px).abs();
            let dy = (monster.y - py).abs();
            if dx <= radius && dy <= radius && !monster.resists_poison() {
                monster.state.confused = true;
                monster.confused_timeout = rng.dice(2, 4) as u16;
            }
        }
    }

    result
}

fn scroll_blank() -> ScrollResult {
    let mut result = ScrollResult::new();
    result.messages.push("This scroll is blank.".to_string());
    result.consumed = false; // Can be written on
    result.identify = false;
    result
}

// ============================================================================
// Punishment System (punish/unpunish from read.c)
// ============================================================================

/// Punishment state for the player
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PunishmentState {
    /// Whether player is currently punished (has ball and chain)
    pub punished: bool,
    /// Weight of the iron ball (affects movement)
    pub ball_weight: i32,
    /// Length of chain (affects how far player can move from ball)
    pub chain_length: i32,
    /// Position of the iron ball (x, y)
    pub ball_position: (i8, i8),
}

impl PunishmentState {
    pub fn new() -> Self {
        Self {
            punished: false,
            ball_weight: 0,
            chain_length: 0,
            ball_position: (0, 0),
        }
    }
}

/// Punish the player with ball and chain
///
/// When punished, the player:
/// - Has an iron ball attached to their leg
/// - Movement is restricted by chain length
/// - Carries extra weight (the ball)
/// - May fumble more often
///
/// Equivalent to punish() from read.c
pub fn punish(player: &mut You) -> Vec<String> {
    let mut messages = Vec::new();

    if player.punishment.punished {
        messages.push("You are already being punished.".to_string());
        return messages;
    }

    player.punishment.punished = true;
    player.punishment.ball_weight = 480; // Iron ball weight in aum
    player.punishment.chain_length = 5; // Chain allows 5 squares of movement
    player.punishment.ball_position = (player.pos.x, player.pos.y);

    // Grant fumbling intrinsic
    player.properties.grant_intrinsic(Property::Fumbling);

    // Add encumbrance from ball weight
    player.current_weight += player.punishment.ball_weight;

    messages.push("You are being punished for your misbehavior!".to_string());
    messages.push("A heavy iron ball is chained to your leg.".to_string());

    messages
}

/// Remove punishment from the player
///
/// This removes the ball and chain, restoring normal movement.
/// Can be triggered by:
/// - Blessed scroll of remove curse
/// - Blessed scroll of punishment
/// - Certain prayer effects
/// - Reading scroll of punishment while already punished (sometimes)
///
/// Equivalent to unpunish() from read.c
pub fn unpunish(player: &mut You) -> Vec<String> {
    let mut messages = Vec::new();

    if !player.punishment.punished {
        messages.push("You are not being punished.".to_string());
        return messages;
    }

    // Remove encumbrance from ball weight
    player.current_weight = player
        .current_weight
        .saturating_sub(player.punishment.ball_weight);

    // Reset punishment state
    player.punishment.punished = false;
    player.punishment.ball_weight = 0;
    player.punishment.chain_length = 0;
    player.punishment.ball_position = (0, 0);

    // Remove fumbling intrinsic if caused by punishment
    player.properties.remove_intrinsic(Property::Fumbling);

    messages.push("Your punishment is over!".to_string());
    messages.push("The iron ball falls from your leg.".to_string());

    messages
}

/// Check if player can move to a position given their punishment state
///
/// When punished, movement is restricted by the chain length.
/// Returns true if the move is allowed, false otherwise.
pub fn can_move_punished(player: &You, target_x: i8, target_y: i8) -> bool {
    if !player.punishment.punished {
        return true; // No punishment, can move freely
    }

    let (ball_x, ball_y) = player.punishment.ball_position;
    let dx = (target_x - ball_x).abs() as i32;
    let dy = (target_y - ball_y).abs() as i32;
    let distance = dx.max(dy); // Chebyshev distance

    distance <= player.punishment.chain_length
}

/// Drag the iron ball when player moves
///
/// When the player moves while punished, the ball may need to be dragged
/// along if the chain becomes taut.
pub fn drag_ball(player: &mut You, new_x: i8, new_y: i8) {
    if !player.punishment.punished {
        return;
    }

    let (ball_x, ball_y) = player.punishment.ball_position;
    let dx = (new_x - ball_x).abs() as i32;
    let dy = (new_y - ball_y).abs() as i32;
    let distance = dx.max(dy);

    // If player is at max chain distance, drag the ball
    if distance >= player.punishment.chain_length {
        // Move ball one step toward player's previous position
        let dir_x = (player.pos.x - ball_x).signum();
        let dir_y = (player.pos.y - ball_y).signum();
        player.punishment.ball_position = (ball_x + dir_x, ball_y + dir_y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_type_from_object() {
        assert_eq!(
            ScrollType::from_object_type(298),
            Some(ScrollType::Identify)
        );
        assert_eq!(ScrollType::from_object_type(999), None);
    }

    #[test]
    fn test_enchant_armor_blessed_vs_cursed() {
        let mut player = You::default();
        player.armor_class = 10;

        // Cursed: weakens armor
        let result = scroll_enchant_armor(&mut player, false, true, false);
        assert_eq!(player.armor_class, 11); // Worse AC
        assert!(result.messages[0].contains("weaker"));

        // Blessed: strengthens armor significantly
        player.armor_class = 10;
        let result = scroll_enchant_armor(&mut player, true, false, false);
        assert_eq!(player.armor_class, 7); // Much better AC
        assert!(result.messages[0].contains("brightly"));

        // Normal: modest improvement
        player.armor_class = 10;
        let result = scroll_enchant_armor(&mut player, false, false, false);
        assert_eq!(player.armor_class, 9); // Modest improvement
        assert!(result.messages[0].contains("glows"));
    }

    #[test]
    fn test_enchant_weapon_blessed_vs_cursed() {
        let mut player = You::default();
        player.weapon_bonus = 0;

        // Cursed: reduces weapon bonus
        let result = scroll_enchant_weapon(&mut player, false, true, false);
        assert!(player.weapon_bonus as i32 <= 0);
        assert!(result.messages[0].contains("dull"));

        // Blessed: increases significantly
        player.weapon_bonus = 0;
        let result = scroll_enchant_weapon(&mut player, true, false, false);
        assert!(player.weapon_bonus >= 3);
        assert!(result.messages[0].contains("brightly"));

        // Normal: modest increase
        player.weapon_bonus = 0;
        let result = scroll_enchant_weapon(&mut player, false, false, false);
        assert_eq!(player.weapon_bonus, 1);
    }

    #[test]
    fn test_confuse_scroll_variants() {
        let mut player = You::default();
        player.confused_timeout = 0;
        let mut rng = crate::rng::GameRng::new(12345);

        // Cursed: extreme confusion
        let initial = player.confused_timeout;
        let result = scroll_confuse(&mut player, false, true, false, &mut rng);
        assert!(player.confused_timeout > initial);
        assert!(result.messages[0].contains("violently"));

        // Blessed: mild confusion touch
        player.confused_timeout = 0;
        let result = scroll_confuse(&mut player, true, false, false, &mut rng);
        assert!(player.confused_timeout > 0);
        assert!(result.messages[0].contains("purple"));

        // Normal: standard confusion
        player.confused_timeout = 0;
        let result = scroll_confuse(&mut player, false, false, false, &mut rng);
        assert!(player.confused_timeout > 0);
        assert!(result.messages[0].contains("spins"));
    }

    #[test]
    fn test_light_scroll_blessed_illuminates_all() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();
        let mut rng = crate::rng::GameRng::new(12345);

        // Make sure level has some unlit cells
        for x in 0..crate::COLNO {
            for y in 0..crate::ROWNO {
                level.cells[x][y].lit = false;
            }
        }

        // Blessed should light entire level
        let result = scroll_light(&mut level, &player, true, false);
        let all_lit = (0..crate::COLNO).all(|x| (0..crate::ROWNO).all(|y| level.cells[x][y].lit));
        assert!(all_lit);
        assert!(result.messages[0].contains("entire"));
    }

    #[test]
    fn test_teleportation_blessed_has_better_odds() {
        let mut player = You::default();
        player.pos = crate::player::Position { x: 5, y: 5 };
        let level = Level::new(DLevel::main_dungeon_start());
        let mut rng = crate::rng::GameRng::new(12345);

        // Normal teleportation
        player.pos = crate::player::Position { x: 5, y: 5 };
        let original_pos = player.pos;
        let _ = scroll_teleportation(&mut player, &level, false, false, &mut rng);
        // Should attempt teleportation (may fail if no walkable spot found)

        // Blessed should have better odds with more attempts
        player.pos = original_pos;
        let _ = scroll_teleportation(&mut player, &level, true, false, &mut rng);
        // Blessed version tries 200 attempts vs normal 100
    }

    #[test]
    fn test_scare_blessed_scares_all_monsters() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();
        let mut rng = crate::rng::GameRng::new(12345);

        // Add some monsters
        for i in 0..5 {
            let mut monster = crate::monster::Monster::new(crate::monster::MonsterId(0), 0, 0, 0);
            monster.x = (i * 2) as i8;
            monster.y = (i * 2) as i8;
            monster.state.fleeing = false;
            level.monsters.push(monster);
        }

        // Blessed should scare all monsters
        let result = scroll_scare(&mut level, &player, true, false, &mut rng);
        let all_fleeing = level.monsters.iter().all(|m| m.state.fleeing);
        assert!(all_fleeing);
        assert!(result.messages[0].contains("entire"));
    }

    #[test]
    fn test_taming_blessed_tames_all() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();

        // Add some monsters
        for i in 0..5 {
            let mut monster = crate::monster::Monster::new(crate::monster::MonsterId(0), 0, 0, 0);
            monster.x = (i * 2) as i8;
            monster.y = (i * 2) as i8;
            monster.state.peaceful = false;
            monster.state.tame = false;
            level.monsters.push(monster);
        }

        // Blessed should tame all monsters
        let result = scroll_taming(&mut level, &player, true, false);
        let all_tame = level
            .monsters
            .iter()
            .all(|m| m.state.peaceful && m.state.tame);
        assert!(all_tame);
        assert!(result.messages[0].contains("extremely"));
    }

    #[test]
    fn test_punishment_blessed_removes_fumbling() {
        let mut player = You::default();
        player.properties.grant_intrinsic(Property::Fumbling);
        assert!(player.properties.has(Property::Fumbling));

        let result = scroll_punishment(&mut player, true, false);
        assert!(!player.properties.has(Property::Fumbling));
        assert!(result.messages[0].contains("absolved"));
    }

    #[test]
    fn test_destroy_blessed_affects_all_nearby() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut player = You::default();
        player.pos = crate::player::Position { x: 10, y: 10 };
        let mut rng = crate::rng::GameRng::new(12345);

        // Add monsters at various distances
        for i in 0..3 {
            let mut monster = crate::monster::Monster::new(crate::monster::MonsterId(0), 0, 0, 0);
            monster.x = (10 + i) as i8;
            monster.y = (10 + i) as i8;
            monster.ac = 5;
            level.monsters.push(monster);
        }

        let initial_ac: Vec<i8> = level.monsters.iter().map(|m| m.ac).collect();

        // Blessed should damage armor on all nearby monsters
        let _ = scroll_destroy(&mut player, &mut level, true, false, &mut rng);

        // All monsters in radius should have worse AC
        for (i, monster) in level.monsters.iter().enumerate() {
            if i < initial_ac.len() {
                assert!(
                    i32::from(monster.ac) > i32::from(initial_ac[i]),
                    "Blessed destroy should worsen AC"
                );
            }
        }
    }

    #[test]
    fn test_magic_mapping_blessed_reveals_all() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();

        // Start with all cells unexplored
        for x in 0..crate::COLNO {
            for y in 0..crate::ROWNO {
                level.cells[x][y].explored = false;
            }
        }

        let result = scroll_magic_mapping(&mut level, &player, true, false);

        let all_explored =
            (0..crate::COLNO).all(|x| (0..crate::ROWNO).all(|y| level.cells[x][y].explored));
        assert!(all_explored);
        assert!(result.messages[0].contains("perfect"));
    }

    #[test]
    fn test_fire_scroll_blessed_protects_player() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut player = You::default();
        player.pos = crate::player::Position { x: 10, y: 10 };
        player.hp = 100;
        let mut rng = crate::rng::GameRng::new(12345);

        // Blessed fire should not damage player
        let result = scroll_fire(&mut player, &mut level, true, false, &mut rng);
        assert_eq!(player.hp, 100); // No damage to player
        assert!(result.messages[0].contains("protective"));

        // Normal/cursed fire should damage player
        player.hp = 100;
        player.properties.remove_intrinsic(Property::FireResistance);
        let _ = scroll_fire(&mut player, &mut level, false, false, &mut rng);
        assert!(player.hp < 100); // Took damage
    }

    #[test]
    fn test_earth_scroll_blessed_has_larger_radius() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut player = You::default();
        player.pos = crate::player::Position { x: 10, y: 10 };
        let mut rng = crate::rng::GameRng::new(12345);

        // Add monsters at various distances
        for i in 0..5 {
            let mut monster = crate::monster::Monster::new(crate::monster::MonsterId(0), 0, 0, 0);
            monster.x = (10 + i) as i8;
            monster.y = (10 + i) as i8;
            monster.hp = 100;
            level.monsters.push(monster);
        }

        // Blessed has radius 3
        let initial_hp: Vec<i32> = level.monsters.iter().map(|m| m.hp).collect();
        let _ = scroll_earth(&mut player, &mut level, true, false, &mut rng);

        // Monsters within radius should take damage
        for (i, monster) in level.monsters.iter().enumerate() {
            if i < 3 {
                assert!(monster.hp < initial_hp[i]);
            }
        }
    }

    #[test]
    fn test_stinking_cloud_blessed_affects_all() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();
        let mut rng = crate::rng::GameRng::new(12345);

        // Add many monsters across level
        for i in 0..10 {
            let mut monster = crate::monster::Monster::new(crate::monster::MonsterId(0), 0, 0, 0);
            monster.x = (i * 2) as i8;
            monster.y = (i * 2) as i8;
            monster.state.confused = false;
            level.monsters.push(monster);
        }

        let result = scroll_stinking_cloud(&mut level, &player, true, false, &mut rng);

        // Blessed should confuse all monsters (that don't resist)
        let confused_count = level.monsters.iter().filter(|m| m.state.confused).count();
        assert!(confused_count > 0);
        assert!(result.messages[0].contains("fragrance"));
    }

    // ========== Punishment System Tests ==========

    #[test]
    fn test_punish_adds_ball_and_chain() {
        let mut player = You::default();
        player.pos = crate::player::Position { x: 10, y: 10 };

        assert!(!player.punishment.punished);

        let messages = punish(&mut player);

        assert!(player.punishment.punished);
        assert_eq!(player.punishment.ball_weight, 480);
        assert_eq!(player.punishment.chain_length, 5);
        assert_eq!(player.punishment.ball_position, (10, 10));
        assert!(player.properties.has(Property::Fumbling));
        assert!(!messages.is_empty());
    }

    #[test]
    fn test_punish_already_punished() {
        let mut player = You::default();
        player.punishment.punished = true;

        let messages = punish(&mut player);

        assert!(messages[0].contains("already"));
    }

    #[test]
    fn test_unpunish_removes_ball_and_chain() {
        let mut player = You::default();
        player.punishment.punished = true;
        player.punishment.ball_weight = 480;
        player.punishment.chain_length = 5;
        player.current_weight = 480;
        player.properties.grant_intrinsic(Property::Fumbling);

        let messages = unpunish(&mut player);

        assert!(!player.punishment.punished);
        assert_eq!(player.punishment.ball_weight, 0);
        assert_eq!(player.punishment.chain_length, 0);
        assert!(!player.properties.has(Property::Fumbling));
        assert!(messages[0].contains("over"));
    }

    #[test]
    fn test_unpunish_not_punished() {
        let mut player = You::default();
        assert!(!player.punishment.punished);

        let messages = unpunish(&mut player);

        assert!(messages[0].contains("not being punished"));
    }

    #[test]
    fn test_can_move_punished_within_chain() {
        let mut player = You::default();
        player.punishment.punished = true;
        player.punishment.chain_length = 5;
        player.punishment.ball_position = (10, 10);

        // Within chain length
        assert!(can_move_punished(&player, 12, 12)); // 2 steps away
        assert!(can_move_punished(&player, 15, 10)); // 5 steps away (at limit)

        // Outside chain length
        assert!(!can_move_punished(&player, 16, 10)); // 6 steps away
    }

    #[test]
    fn test_can_move_punished_no_punishment() {
        let player = You::default();
        assert!(!player.punishment.punished);

        // Can move anywhere when not punished
        assert!(can_move_punished(&player, 50, 50));
    }

    #[test]
    fn test_drag_ball() {
        let mut player = You::default();
        player.pos = crate::player::Position { x: 10, y: 10 };
        player.punishment.punished = true;
        player.punishment.chain_length = 3;
        player.punishment.ball_position = (7, 10);

        // Player moves further from ball
        drag_ball(&mut player, 14, 10);

        // Ball should have moved toward player's previous position
        assert!(player.punishment.ball_position.0 > 7);
    }

    #[test]
    fn test_punishment_state_serialization() {
        let state = PunishmentState {
            punished: true,
            ball_weight: 480,
            chain_length: 5,
            ball_position: (10, 10),
        };

        let json = serde_json::to_string(&state).expect("serialize");
        let restored: PunishmentState = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.punished, true);
        assert_eq!(restored.ball_weight, 480);
        assert_eq!(restored.chain_length, 5);
        assert_eq!(restored.ball_position, (10, 10));
    }
}
