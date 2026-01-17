//! Scroll reading effects (read.c)
//!
//! Handles reading scrolls and their effects.

use crate::dungeon::Level;
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
        return ScrollResult::new()
            .with_message("You can't read while blind!");
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
        ScrollType::Destroy => scroll_destroy(player, level, cursed, rng),
        ScrollType::Confuse => scroll_confuse(player, confused, rng),
        ScrollType::Scare => scroll_scare(level, player, cursed, rng),
        ScrollType::RemoveCurse => scroll_remove_curse(player, blessed),
        ScrollType::EnchantWeapon => scroll_enchant_weapon(player, blessed, cursed, confused),
        ScrollType::Create => scroll_create(level, player, cursed, rng),
        ScrollType::Taming => scroll_taming(level, player, cursed),
        ScrollType::Genocide => scroll_genocide(blessed, cursed),
        ScrollType::Light => scroll_light(level, player, cursed),
        ScrollType::Teleportation => scroll_teleportation(player, level, cursed, rng),
        ScrollType::Gold => scroll_gold(player, blessed, rng),
        ScrollType::Food => scroll_food(player, blessed, rng),
        ScrollType::Identify => scroll_identify(player, blessed),
        ScrollType::MagicMapping => scroll_magic_mapping(level, player, confused),
        ScrollType::Amnesia => scroll_amnesia(player, cursed),
        ScrollType::Fire => scroll_fire(player, level, cursed, rng),
        ScrollType::Earth => scroll_earth(player, level, cursed, rng),
        ScrollType::Punishment => scroll_punishment(player, cursed),
        ScrollType::Charging => scroll_charging(player, blessed, cursed),
        ScrollType::StinkingCloud => scroll_stinking_cloud(level, player, cursed, rng),
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
        result.messages.push("Your armor glows brightly!".to_string());
        // Increase armor protection by 2-3 (lower AC = better)
        player.armor_class = player.armor_class.saturating_sub(3);
    } else {
        result.messages.push("Your armor glows!".to_string());
        // Increase armor protection by 1
        player.armor_class = player.armor_class.saturating_sub(1);
    }

    result
}

fn scroll_destroy(player: &mut You, level: &mut Level, cursed: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result.messages.push("You feel like you need a new weapon.".to_string());
        // Destroy wielded weapon - reduce weapon bonus
        player.weapon_bonus = player.weapon_bonus.saturating_sub(3);
    } else {
        // Destroy armor on nearby monsters
        result.messages.push("You hear crashing and tearing sounds!".to_string());
        // Damage armor on nearby monsters
        for monster in &mut level.monsters {
            if rng.one_in(3) {
                monster.ac = monster.ac.saturating_add(2); // Worse AC
            }
        }
    }

    result
}

fn scroll_confuse(player: &mut You, confused: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    if confused {
        // Reading while confused grants confusion touch
        result.messages.push("Your hands begin to glow purple!".to_string());
        // Confusion touch lasts for a while
        player.confused_timeout = player.confused_timeout.saturating_add(20);
    } else {
        let duration = rng.dice(4, 4) as u16;
        player.confused_timeout = player.confused_timeout.saturating_add(duration);
        result.messages.push("Your head spins!".to_string());
    }

    result
}

fn scroll_scare(level: &mut Level, player: &You, cursed: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result.messages.push("You hear maniacal laughter!".to_string());
    } else {
        result.messages.push("You hear a terrifying shriek!".to_string());
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
        result.messages.push("You feel like someone is helping you.".to_string());
        // Blessed removes all curses (inventory handling done by caller)
    } else {
        result.messages.push("You feel less encumbered.".to_string());
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
        result.messages.push("Your weapon glows brightly blue!".to_string());
        // Increase weapon enchantment by 2-3
        player.weapon_bonus = player.weapon_bonus.saturating_add(3);
    } else {
        result.messages.push("Your weapon glows blue!".to_string());
        // Increase weapon enchantment by 1
        player.weapon_bonus = player.weapon_bonus.saturating_add(1);
    }

    result
}

fn scroll_create(level: &mut Level, player: &You, cursed: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result.messages.push("You read the scroll and nothing happens.".to_string());
    } else {
        let count = rng.rnd(4) + 1;
        result.messages.push(format!("You create {} monster(s)!", count));
        // Monster spawning requires monster creation infrastructure
        // For now, just acknowledge the effect
        let _ = (level, player, count);
    }

    result
}

fn scroll_taming(level: &mut Level, player: &You, cursed: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result.messages.push("You feel that reading this was a mistake.".to_string());
        // Aggravate nearby monsters - wake them up and make hostile
        for monster in &mut level.monsters {
            monster.state.sleeping = false;
            monster.state.peaceful = false;
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
        result.messages.push("A thunderous voice booms: FOOL!".to_string());
        // Reverse genocide - spawn many monsters (requires UI input)
    } else if blessed {
        result.messages.push("What class of monsters do you wish to genocide?".to_string());
        // Genocide entire monster class (requires UI input for class selection)
    } else {
        result.messages.push("What monster do you wish to genocide?".to_string());
        // Genocide single monster type (requires UI input for monster selection)
    }

    result
}

fn scroll_light(level: &mut Level, player: &You, cursed: bool) -> ScrollResult {
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

fn scroll_teleportation(player: &mut You, level: &Level, cursed: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result.messages.push("You feel disoriented.".to_string());
        // Teleport to a bad location
    }

    // Find random walkable position
    for _ in 0..100 {
        let x = rng.rn2(crate::COLNO as u32) as i8;
        let y = rng.rn2(crate::ROWNO as u32) as i8;

        if level.is_walkable(x, y) && level.monster_at(x, y).is_none() {
            player.prev_pos = player.pos;
            player.pos.x = x;
            player.pos.y = y;
            result.messages.push("You find yourself somewhere else.".to_string());
            return result;
        }
    }

    result.messages.push("You feel disoriented for a moment.".to_string());
    result
}

fn scroll_gold(player: &mut You, blessed: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    if blessed {
        let gold = rng.dice(10, 50) as i32;
        player.gold += gold;
        result.messages.push(format!("You feel richer! (+{} gold)", gold));
    } else {
        let gold = rng.dice(5, 50) as i32;
        player.gold += gold;
        result.messages.push(format!("Gold appears! (+{} gold)", gold));
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
    result.messages.push(format!("Food appears! (+{} nutrition)", nutrition));
    // Food item creation would require object spawning infrastructure

    result
}

fn scroll_identify(player: &mut You, blessed: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if blessed {
        result.messages.push("All your possessions glow briefly!".to_string());
        // Identify all inventory items (requires inventory access from caller)
    } else {
        result.messages.push("This is an identify scroll.".to_string());
        // Let player pick item to identify (requires UI interaction from caller)
    }

    let _ = player;
    result
}

fn scroll_magic_mapping(level: &mut Level, player: &You, confused: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if confused {
        result.messages.push("Your head spins as the world around you shifts!".to_string());
        // Scramble map memory - unexplore random cells
        for x in 0..crate::COLNO {
            for y in 0..crate::ROWNO {
                if level.cells[x][y].explored {
                    // 50% chance to forget each explored cell
                    level.cells[x][y].explored = (x + y) % 2 == 0;
                }
            }
        }
    } else {
        result.messages.push("A map coalesces in your mind!".to_string());
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

fn scroll_amnesia(player: &mut You, cursed: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result.messages.push("You forget everything you knew!".to_string());
        // Forget all identified items (requires discovery tracking from caller)
    } else {
        result.messages.push("You forget your surroundings.".to_string());
        // Forget current level map (requires level access from caller)
    }

    let _ = player;
    result
}

fn scroll_fire(player: &mut You, level: &mut Level, cursed: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    result.messages.push("Flames erupt around you!".to_string());

    // Damage player if not fire resistant
    if !player.properties.has(Property::FireResistance) {
        let damage = rng.dice(3, 6) as i32;
        player.hp -= damage;
        result.messages.push(format!("You are burned for {} damage!", damage));
    }

    if cursed {
        // Larger blast radius
        result.messages.push("The flames spread wildly!".to_string());
    }

    // Damage nearby monsters with fire
    let px = player.pos.x;
    let py = player.pos.y;
    let radius = if cursed { 3i8 } else { 2i8 };
    
    for monster in &mut level.monsters {
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();
        if dx <= radius && dy <= radius && !monster.resists_fire() {
            let damage = rng.dice(2, 6) as i32;
            monster.hp -= damage;
        }
    }

    result
}

fn scroll_earth(player: &mut You, level: &mut Level, cursed: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    result.messages.push("The earth shakes!".to_string());

    if cursed {
        // Drop boulders on player
        let damage = rng.dice(4, 6) as i32;
        player.hp -= damage;
        result.messages.push(format!("A boulder falls on you for {} damage!", damage));
    } else {
        // Create boulders around player - damage nearby monsters
        result.messages.push("Boulders fall around you!".to_string());
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

fn scroll_punishment(player: &mut You, cursed: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result.messages.push("You are punished even more severely!".to_string());
        // Extra punishment - more fumbling and slower
        player.properties.grant_intrinsic(Property::Fumbling);
        player.movement_points = player.movement_points.saturating_sub(4);
    } else {
        result.messages.push("You are punished for your misbehavior!".to_string());
        // Attach ball and chain effect - grant fumbling
        player.properties.grant_intrinsic(Property::Fumbling);
    }
    result
}

fn scroll_charging(player: &mut You, blessed: bool, cursed: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        result.messages.push("You feel momentarily disoriented.".to_string());
        // Drain wand charges (requires inventory wand access from caller)
    } else if blessed {
        result.messages.push("This is a blessed scroll of charging!".to_string());
        // Full charge wand + uncurse (requires inventory wand access from caller)
    } else {
        result.messages.push("This is a scroll of charging.".to_string());
        // Let player choose wand to recharge (requires UI interaction from caller)
    }

    let _ = player;
    result
}

fn scroll_stinking_cloud(level: &mut Level, player: &You, cursed: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    result.messages.push("A stinking cloud billows out!".to_string());

    // Sicken nearby monsters
    let px = player.pos.x;
    let py = player.pos.y;
    let radius = if cursed { 3i8 } else { 6i8 };

    for monster in &mut level.monsters {
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();
        if dx <= radius && dy <= radius && !monster.resists_poison() {
            monster.state.confused = true;
            monster.confused_timeout = rng.dice(2, 4) as u16;
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
}
