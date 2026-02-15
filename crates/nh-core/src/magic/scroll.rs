//! Scroll reading effects (read.c)
//!
//! Handles reading scrolls and their effects.

use crate::data::objects::OBJECTS;
use crate::dungeon::{DLevel, Level};
use crate::object::{Object, ObjectClass, DirectionType};
use crate::player::{Property, You};
use crate::rng::GameRng;

/// Wand of wishing object type index
const WAN_WISHING: i16 = 369;

/// Simple visibility check: monster is within sight range of player
fn cansee_monster(monster: &crate::monster::Monster, player: &You) -> bool {
    let dx = (monster.x - player.pos.x).abs();
    let dy = (monster.y - player.pos.y).abs();
    dx <= 15 && dy <= 15
}

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
        ScrollType::Destroy => scroll_destroy(player, level, blessed, cursed, confused, rng),
        ScrollType::Confuse => scroll_confuse(player, blessed, cursed, confused, rng),
        ScrollType::Scare => scroll_scare(level, player, blessed, cursed, confused, rng),
        ScrollType::RemoveCurse => scroll_remove_curse(player, blessed, cursed, confused),
        ScrollType::EnchantWeapon => scroll_enchant_weapon(player, blessed, cursed, confused),
        ScrollType::Create => scroll_create(level, player, blessed, cursed, confused, rng),
        ScrollType::Taming => scroll_taming(level, player, blessed, cursed, confused),
        ScrollType::Genocide => scroll_genocide(blessed, cursed),
        ScrollType::Light => scroll_light(level, player, blessed, cursed, confused, rng),
        ScrollType::Teleportation => scroll_teleportation(player, level, blessed, cursed, confused, rng),
        ScrollType::Gold => scroll_gold_detection(level, player, cursed, rng),
        ScrollType::Food => scroll_food_detection(level, player, rng),
        ScrollType::Identify => scroll_identify(player, blessed, confused),
        ScrollType::MagicMapping => scroll_magic_mapping(level, player, blessed, cursed),
        ScrollType::Amnesia => scroll_amnesia(player, level, blessed, cursed, rng),
        ScrollType::Fire => scroll_fire(player, level, blessed, cursed, confused, rng),
        ScrollType::Earth => scroll_earth(player, level, blessed, cursed, rng),
        ScrollType::Punishment => scroll_punishment(player, blessed, cursed, confused),
        ScrollType::Charging => scroll_charging(player, blessed, cursed, confused, rng),
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
    confused: bool,
    rng: &mut GameRng,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    if confused {
        // Confused: make armor erodeproof instead of destroying it
        // C: otmp->oerodeproof = !scursed
        if !cursed {
            result.messages.push("Your armor glows purple for a moment.".to_string());
            // Would set erodeproof on a random armor piece
        } else {
            result.messages.push("Your armor vibrates.".to_string());
            // Cursed+confused: remove erodeproof
        }
    } else if cursed {
        // Cursed: destroy player's armor (AC penalty)
        result.messages.push("You feel like you need some new armor.".to_string());
        player.armor_class = player.armor_class.saturating_add(3);
    } else if blessed {
        result.messages.push("Everything around you shatters and crumbles!".to_string());
        for monster in &mut level.monsters {
            let dx = (monster.x - player.pos.x).abs();
            let dy = (monster.y - player.pos.y).abs();
            if dx <= 5 && dy <= 5 {
                monster.ac = monster.ac.saturating_add(3);
            }
        }
    } else {
        result.messages.push("You hear crashing and tearing sounds!".to_string());
        for monster in &mut level.monsters {
            if rng.one_in(3) {
                monster.ac = monster.ac.saturating_add(2);
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
    _blessed: bool,
    cursed: bool,
    confused: bool,
    rng: &mut GameRng,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    if confused || cursed {
        // C: confused/cursed wakes and unfreezes monsters instead of scaring
        result.messages.push("You hear sad wailing close by.".to_string());
        for monster in &mut level.monsters {
            if cansee_monster(monster, player) {
                monster.state.fleeing = false;
                monster.state.sleeping = false;
            }
        }
    } else {
        // Normal/blessed: scare visible monsters
        let mut ct = 0;
        for monster in &mut level.monsters {
            if cansee_monster(monster, player) {
                monster.state.fleeing = true;
                monster.flee_timeout = rng.dice(2, 6) as u16;
                ct += 1;
            }
        }
        if ct > 0 {
            result.messages.push("You hear maniacal laughter close by.".to_string());
        } else {
            result.messages.push("You hear maniacal laughter in the distance.".to_string());
        }
    }

    result
}

fn scroll_remove_curse(player: &mut You, _blessed: bool, cursed: bool, confused: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if confused {
        result.messages.push("You feel like you need some help.".to_string());
        // Confused: randomly bless/curse worn items instead of uncursing
    } else {
        result.messages.push("You feel like someone is helping you.".to_string());
    }

    if cursed {
        result.messages.push("The scroll disintegrates.".to_string());
        return result;
    }

    // Uncurse worn/wielded items (blessed uncurses all inventory)
    // Full inventory iteration requires caller support; we handle player-level effects
    player.properties.remove_intrinsic(Property::Fumbling);

    // Remove punishment if not confused
    if player.punishment.punished && !confused {
        unpunish(player);
        result.messages.push("Your punishment is over!".to_string());
    }

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
    confused: bool,
    rng: &mut GameRng,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    // C: create_critters(1 + ((confused || cursed) ? 12 : 0) + ((blessed || rn2(73)) ? 0 : rnd(4)),
    //                    confused ? &mons[PM_ACID_BLOB] : NULL, FALSE)
    let base_count = 1;
    let extra = if confused || cursed { 12 } else { 0 };
    let bonus = if blessed || rng.rn2(73) != 0 { 0 } else { rng.rnd(4) };
    let count = base_count + extra + bonus;

    if confused {
        result.messages.push(format!("{} acid blobs appear around you!", count));
    } else {
        result.messages.push(format!("{} monster(s) appear around you!", count));
    }
    // Actual monster spawning requires makemon infrastructure
    let _ = (level, player);

    result
}

fn scroll_taming(level: &mut Level, player: &You, _blessed: bool, cursed: bool, confused: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    // C: confused -> 5x5 radius, normal -> adjacent (1x1)
    let bd: i8 = if confused { 5 } else { 1 };
    let px = player.pos.x;
    let py = player.pos.y;
    let mut results = 0i32;

    for monster in &mut level.monsters {
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();
        if dx <= bd && dy <= bd {
            if cursed {
                // Cursed: anger monsters
                monster.state.sleeping = false;
                monster.state.peaceful = false;
                results -= 1;
            } else {
                // Normal/blessed: tame monsters
                monster.state.peaceful = true;
                monster.state.tame = true;
                results += 1;
            }
        }
    }

    if results == 0 {
        result.messages.push("Nothing interesting seems to happen.".to_string());
    } else if results > 0 {
        result.messages.push("The neighborhood is friendlier.".to_string());
    } else {
        result.messages.push("The neighborhood is unfriendlier.".to_string());
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

fn scroll_light(level: &mut Level, player: &You, blessed: bool, cursed: bool, confused: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    if confused && rng.rn2(5) == 0 {
        // C: confused + 1/5 chance -> create yellow/black light monster instead
        if cursed {
            result.messages.push("A black light appears!".to_string());
        } else {
            result.messages.push("A yellow light appears!".to_string());
        }
        // Actual monster creation requires makemon infrastructure
        return result;
    }

    // litroom(!confused && !scursed, sobj) -> light if not confused and not cursed
    let do_light = !confused && !cursed;

    if do_light {
        if blessed {
            result.messages.push("The entire level is illuminated!".to_string());
            for x in 0..crate::COLNO {
                for y in 0..crate::ROWNO {
                    level.cells[x][y].lit = true;
                }
            }
        } else {
            result.messages.push("A light shines!".to_string());
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
    } else {
        // Darken
        result.messages.push("Darkness surrounds you!".to_string());
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
    }

    result
}

fn scroll_teleportation(
    player: &mut You,
    level: &Level,
    blessed: bool,
    cursed: bool,
    confused: bool,
    rng: &mut GameRng,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    if confused || cursed {
        // C: confused/cursed -> level_tele() (change dungeon level)
        result.messages.push("You feel very disoriented for a moment.".to_string());
        // Level teleport requires dungeon infrastructure
        // For now, mark the effect
        result.messages.push("You are yanked in a new direction!".to_string());
        return result;
    }

    // Normal/blessed: position teleport on current level
    if blessed {
        result.messages.push("You feel in control of where you teleport.".to_string());
    }

    // Find random walkable position
    for _ in 0..200 {
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

/// Scroll of gold detection - detects gold on the level (NOT creates gold)
/// C: gold_detect(sobj) for normal, trap_detect(sobj) for confused/cursed
fn scroll_gold_detection(level: &Level, player: &You, cursed: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    if cursed {
        // C: cursed -> trap_detect() - reveal traps instead of gold
        let mut trap_count = 0;
        for trap in &level.traps {
            if !trap.seen {
                trap_count += 1;
            }
        }
        if trap_count > 0 {
            result.messages.push(format!("You sense {} trap(s) on this level.", trap_count));
        } else {
            result.messages.push("You feel a strange sense of loss.".to_string());
            result.identify = false;
        }
    } else {
        // Normal/blessed: detect gold on the level
        let mut gold_found = false;
        for x in 0..crate::COLNO {
            for y in 0..crate::ROWNO {
                // Check floor objects for coins
                for obj in &level.objects_at(x as i8, y as i8) {
                    if obj.class == ObjectClass::Coin {
                        gold_found = true;
                    }
                }
            }
        }
        // Also check monster inventories for gold
        for monster in &level.monsters {
            for obj in &monster.inventory {
                if obj.class == ObjectClass::Coin {
                    gold_found = true;
                }
            }
        }
        if gold_found {
            result.messages.push("You sense the presence of gold.".to_string());
        } else {
            result.messages.push("You feel materially poor.".to_string());
            result.identify = false;
        }
    }

    let _ = (player, rng);
    result
}

/// Scroll of food detection - detects food on the level (NOT creates food)
/// C: food_detect(sobj)
fn scroll_food_detection(level: &Level, player: &You, _rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    let mut food_found = false;
    for x in 0..crate::COLNO {
        for y in 0..crate::ROWNO {
            for obj in &level.objects_at(x as i8, y as i8) {
                if obj.class == ObjectClass::Food {
                    food_found = true;
                }
            }
        }
    }
    if food_found {
        result.messages.push("You sense the presence of food.".to_string());
    } else {
        result.messages.push("You feel a strange sense of loss.".to_string());
        result.identify = false;
    }

    let _ = player;
    result
}

fn scroll_identify(player: &mut You, blessed: bool, confused: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    if confused {
        // C: "You identify this as an identify scroll."
        result.messages.push("You identify this as an identify scroll.".to_string());
    } else if blessed {
        // C: blessed -> identify all items (cval=0 means identify_pack(0))
        result.messages.push("All your possessions glow briefly!".to_string());
        // Full identify_pack requires inventory infrastructure
    } else {
        // C: normal -> identify 1 item (sometimes more with luck)
        result.messages.push("This is an identify scroll.".to_string());
        // identify_pack(cval) requires UI interaction from caller
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

/// Scroll of amnesia - C: forget() with howmuch flags
/// Always forgets: traps, 6/7 of map
/// Flags: ALL_SPELLS = forget all spells, ALL_MAP = forget whole map
fn scroll_amnesia(player: &mut You, level: &mut Level, blessed: bool, cursed: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    // C: forget((!sblessed ? ALL_SPELLS : 0) | (!confused || scursed ? ALL_MAP : 0))
    let forget_spells = !blessed;
    let forget_whole_map = !blessed || cursed; // (not confused is always true since seffects checks)

    // Forget traps (C: forget_traps)
    for trap in &mut level.traps {
        // Forget all traps except the one player is standing on
        if trap.x != player.pos.x || trap.y != player.pos.y {
            trap.seen = false;
        }
    }

    // Forget map (C: forget_map)
    if forget_whole_map {
        // Forget entire map
        for x in 0..crate::COLNO {
            for y in 0..crate::ROWNO {
                level.cells[x][y].explored = false;
            }
        }
    } else {
        // Forget 6/7 of map
        for x in 0..crate::COLNO {
            for y in 0..crate::ROWNO {
                if rng.rn2(7) != 0 {
                    level.cells[x][y].explored = false;
                }
            }
        }
    }

    // Forget spells (C: losespells)
    if forget_spells {
        player.known_spells.clear();
    }

    // 1 in 3 chance of forgetting some levels
    if rng.rn2(3) == 0 {
        // forget_levels(rn2(25)) - would require dungeon-level tracking
    }

    // 1 in 3 chance of forgetting some objects
    if rng.rn2(3) == 0 {
        // forget_objects(rn2(25)) - would require object discovery tracking
    }

    // Messages (C: various)
    if rng.rn2(2) != 0 {
        result.messages.push("Who was that Maud person anyway?".to_string());
    } else {
        result.messages.push("Thinking of Maud you forget everything else.".to_string());
    }

    result
}

fn scroll_fire(
    player: &mut You,
    level: &mut Level,
    blessed: bool,
    cursed: bool,
    confused: bool,
    rng: &mut GameRng,
) -> ScrollResult {
    let mut result = ScrollResult::new();

    if confused {
        // C: confused -> scroll catches fire, burn your hands
        if player.properties.has(Property::FireResistance) {
            result.messages.push("Oh, look, what a pretty fire in your hands.".to_string());
        } else {
            result.messages.push("The scroll catches fire and you burn your hands.".to_string());
            player.hp -= 1;
        }
        return result;
    }

    // C: dam = (2 * (rn1(3, 3) + 2 * cval) + 1) / 3
    // where cval = bcsign(sobj) -> blessed=1, uncursed=0, cursed=-1
    // rn1(3,3) = rnd(3) + 2 = [3,5]
    let cval = if blessed { 1i32 } else if cursed { -1 } else { 0 };
    let base_roll = rng.rnd(3) as i32 + 2; // [3,5]
    let dam = (2 * (base_roll + 2 * cval) + 1) / 3;
    let dam = dam.max(1);

    if blessed {
        // C: blessed -> player chooses target, 5x damage
        // Without targeting UI, center on player but protect player
        result.messages.push("The scroll erupts in a tower of flame!".to_string());
        let actual_dam = dam * 5;
        // Damage nearby monsters with explosion
        for monster in &mut level.monsters {
            let dx = (monster.x - player.pos.x).abs();
            let dy = (monster.y - player.pos.y).abs();
            if dx <= 3 && dy <= 3 && !monster.resists_fire() {
                monster.hp -= actual_dam;
            }
        }
    } else {
        result.messages.push("The scroll erupts in a tower of flame!".to_string());
        // Damage player if not fire resistant
        if !player.properties.has(Property::FireResistance) {
            player.hp -= dam;
            result.messages.push(format!("You are burned for {} damage!", dam));
        }
        // Damage nearby monsters
        for monster in &mut level.monsters {
            let dx = (monster.x - player.pos.x).abs();
            let dy = (monster.y - player.pos.y).abs();
            if dx <= 2 && dy <= 2 && !monster.resists_fire() {
                monster.hp -= dam;
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

fn scroll_punishment(player: &mut You, blessed: bool, _cursed: bool, confused: bool) -> ScrollResult {
    let mut result = ScrollResult::new();

    // C: confused || sblessed -> "You feel guilty." and nothing else
    if confused || blessed {
        result.messages.push("You feel guilty.".to_string());
        return result;
    }

    // Normal/cursed: punish the player
    let msgs = punish(player);
    result.messages.extend(msgs);
    result
}

/// Scroll of charging
/// C: confused -> energy restore/drain; normal -> recharge an item
fn scroll_charging(player: &mut You, blessed: bool, cursed: bool, confused: bool, rng: &mut GameRng) -> ScrollResult {
    let mut result = ScrollResult::new();

    if confused {
        // C: confused+cursed -> drain all energy; confused+normal/blessed -> restore energy
        if cursed {
            result.messages.push("You feel discharged.".to_string());
            player.energy = 0;
        } else {
            result.messages.push("You feel charged up!".to_string());
            let gain = if blessed {
                rng.dice(6, 4) as i32
            } else {
                rng.dice(4, 4) as i32
            };
            player.energy += gain;
            if player.energy > player.energy_max {
                // If current exceeds max, raise max
                player.energy_max = player.energy;
            } else {
                // Otherwise restore to max
                player.energy = player.energy_max;
            }
        }
        return result;
    }

    // Not confused: recharge an item
    // This requires UI interaction to pick an item, so signal the caller
    result.messages.push("This is a scroll of charging.".to_string());
    // The actual recharge() will be called from the action layer with a selected item
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
// Recharge System (recharge() from read.c)
// ============================================================================

/// Recharge an object (wand, ring, or tool).
/// `curse_bless`: -1 if cursed scroll, +1 if blessed, 0 otherwise.
///
/// C: recharge(obj, curse_bless) from read.c lines 466-714
pub fn recharge(obj: &mut Object, curse_bless: i32, rng: &mut GameRng) -> Vec<String> {
    let mut messages = Vec::new();
    let is_cursed = curse_bless < 0;
    let is_blessed = curse_bless > 0;

    if obj.class == ObjectClass::Wand {
        // Wand recharging
        let obj_def = &OBJECTS[obj.object_type as usize];
        let is_wishing = obj.object_type == WAN_WISHING;
        let lim: i32 = if is_wishing {
            3
        } else if obj_def.direction != DirectionType::NonDirectional {
            8  // Directional wands (ray/immediate)
        } else {
            15 // Non-directional wands
        };

        // Undo prior cancellation
        if obj.enchantment == -1 {
            obj.enchantment = 0;
        }

        // Explosion risk based on recharge count
        // C: n * n * n > rn2(7 * 7 * 7) where n = recharged count
        let n = obj.recharged as i32;
        if n > 0 && (is_wishing || (n * n * n > rng.rn2(343) as i32)) {
            // Wand explodes!
            let explode_dam = rng.rnd(lim as u32) as i32;
            messages.push(format!("Your wand explodes with force {}!", explode_dam));
            obj.enchantment = -1; // Mark as destroyed
            return messages;
        }

        // Increment recharge count
        obj.recharged = obj.recharged.saturating_add(1);

        if is_cursed {
            // Cursed: strip charges to 0
            if is_blessed || obj.enchantment <= 0 {
                messages.push("Nothing happens.".to_string());
            } else {
                messages.push("Your wand vibrates briefly.".to_string());
                obj.enchantment = 0;
            }
        } else {
            // Normal/blessed: add charges
            // C: n = (lim == 3) ? 3 : rn1(5, lim + 1 - 5)
            // rn1(5, lim+1-5) = rnd(5) + (lim-4) - 1 = rnd(5) + lim - 5
            let new_charges = if lim == 3 {
                3i8
            } else {
                let base = rng.rnd(5) as i8 + (lim as i8 - 5); // [lim-4, lim]
                if is_blessed { base } else { rng.rnd(base.max(1) as u32) as i8 }
            };

            if obj.enchantment < new_charges {
                obj.enchantment = new_charges;
            } else {
                obj.enchantment += 1;
            }

            // Wand of wishing max 3 charges
            if is_wishing && obj.enchantment > 3 {
                messages.push(format!("Your wand explodes!"));
                obj.enchantment = -1;
                return messages;
            }

            if obj.enchantment >= lim as i8 {
                messages.push("Your wand glows blue for a moment.".to_string());
            } else {
                messages.push("Your wand glows briefly.".to_string());
            }
        }
    } else if obj.class == ObjectClass::Ring {
        // Ring recharging
        let s: i8 = if is_blessed {
            rng.rnd(3) as i8
        } else if is_cursed {
            -(rng.rnd(2) as i8)
        } else {
            1
        };

        // Destruction check: if spe > rn2(7) or spe <= -5
        if obj.enchantment > rng.rn2(7) as i8 || obj.enchantment <= -5 {
            let dam = rng.rnd((3 * obj.enchantment.unsigned_abs() as u32).max(1));
            messages.push(format!("Your ring pulsates momentarily, then explodes! ({} damage)", dam));
            obj.enchantment = -128; // Mark as destroyed
        } else {
            if s < 0 {
                messages.push("Your ring spins counterclockwise for a moment.".to_string());
            } else {
                messages.push("Your ring spins clockwise for a moment.".to_string());
            }
            obj.enchantment = obj.enchantment.saturating_add(s);
        }
    } else if obj.class == ObjectClass::Tool {
        // Tool recharging
        if obj.recharged < 7 {
            obj.recharged += 1;
        }

        if is_cursed {
            if is_blessed || obj.enchantment <= 0 {
                messages.push("Nothing happens.".to_string());
            } else {
                messages.push("Your tool vibrates briefly.".to_string());
                obj.enchantment = 0;
            }
        } else if is_blessed {
            obj.enchantment = obj.enchantment.saturating_add(rng.rnd(3) as i8);
            messages.push("Your tool glows blue for a moment.".to_string());
        } else {
            obj.enchantment = obj.enchantment.saturating_add(1);
            messages.push("Your tool glows briefly.".to_string());
        }
    } else {
        messages.push("You have a feeling of loss.".to_string());
    }

    messages
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
        let _result = scroll_enchant_weapon(&mut player, false, false, false);
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

        for x in 0..crate::COLNO {
            for y in 0..crate::ROWNO {
                level.cells[x][y].lit = false;
            }
        }

        let result = scroll_light(&mut level, &player, true, false, false, &mut rng);
        let all_lit = (0..crate::COLNO).all(|x| (0..crate::ROWNO).all(|y| level.cells[x][y].lit));
        assert!(all_lit);
        assert!(result.messages[0].contains("entire"));
    }

    #[test]
    fn test_teleportation_normal() {
        let mut player = You::default();
        player.pos = crate::player::Position { x: 5, y: 5 };
        let level = Level::new(DLevel::main_dungeon_start());
        let mut rng = crate::rng::GameRng::new(12345);

        let _ = scroll_teleportation(&mut player, &level, false, false, false, &mut rng);
    }

    #[test]
    fn test_teleportation_confused_level_teleport() {
        let mut player = You::default();
        player.pos = crate::player::Position { x: 5, y: 5 };
        let level = Level::new(DLevel::main_dungeon_start());
        let mut rng = crate::rng::GameRng::new(12345);

        let result = scroll_teleportation(&mut player, &level, false, false, true, &mut rng);
        assert!(result.messages.iter().any(|m| m.contains("disoriented") || m.contains("yanked")));
    }

    #[test]
    fn test_scare_normal_scares_visible_monsters() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();
        let mut rng = crate::rng::GameRng::new(12345);

        for i in 0..5 {
            let mut monster = crate::monster::Monster::new(crate::monster::MonsterId(0), 0, 0, 0);
            monster.x = (i * 2) as i8;
            monster.y = (i * 2) as i8;
            monster.state.fleeing = false;
            level.monsters.push(monster);
        }

        let result = scroll_scare(&mut level, &player, false, false, false, &mut rng);
        let fleeing_count = level.monsters.iter().filter(|m| m.state.fleeing).count();
        assert!(fleeing_count > 0);
        assert!(result.messages[0].contains("laughter"));
    }

    #[test]
    fn test_scare_confused_wakes_monsters() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();
        let mut rng = crate::rng::GameRng::new(12345);

        for i in 0..5 {
            let mut monster = crate::monster::Monster::new(crate::monster::MonsterId(0), 0, 0, 0);
            monster.x = (i * 2) as i8;
            monster.y = (i * 2) as i8;
            monster.state.sleeping = true;
            level.monsters.push(monster);
        }

        let result = scroll_scare(&mut level, &player, false, false, true, &mut rng);
        let awake_count = level.monsters.iter().filter(|m| !m.state.sleeping).count();
        assert!(awake_count > 0);
        assert!(result.messages[0].contains("sad wailing"));
    }

    #[test]
    fn test_taming_blessed_tames_all() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();

        for i in 0..5 {
            let mut monster = crate::monster::Monster::new(crate::monster::MonsterId(0), 0, 0, 0);
            monster.x = i as i8;
            monster.y = i as i8;
            monster.state.peaceful = false;
            monster.state.tame = false;
            level.monsters.push(monster);
        }

        // Confused=true gives bd=5. Monsters at (0,0)...(4,4) are all within range.
        let result = scroll_taming(&mut level, &player, true, false, true);
        let all_tame = level.monsters.iter().all(|m| m.state.peaceful && m.state.tame);
        assert!(all_tame);
        assert!(result.messages[0].contains("friendlier"));
    }

    #[test]
    fn test_punishment_confused_just_feels_guilty() {
        let mut player = You::default();

        let result = scroll_punishment(&mut player, false, false, true);
        assert!(result.messages[0].contains("guilty"));
        assert!(!player.punishment.punished);
    }

    #[test]
    fn test_punishment_normal_punishes() {
        let mut player = You::default();
        player.pos = crate::player::Position { x: 10, y: 10 };

        let result = scroll_punishment(&mut player, false, false, false);
        assert!(player.punishment.punished);
        assert!(result.messages.iter().any(|m| m.contains("punished") || m.contains("iron ball")));
    }

    #[test]
    fn test_destroy_blessed_affects_all_nearby() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut player = You::default();
        player.pos = crate::player::Position { x: 10, y: 10 };
        let mut rng = crate::rng::GameRng::new(12345);

        for i in 0..3 {
            let mut monster = crate::monster::Monster::new(crate::monster::MonsterId(0), 0, 0, 0);
            monster.x = (10 + i) as i8;
            monster.y = (10 + i) as i8;
            monster.ac = 5;
            level.monsters.push(monster);
        }

        let initial_ac: Vec<i8> = level.monsters.iter().map(|m| m.ac).collect();
        let _ = scroll_destroy(&mut player, &mut level, true, false, false, &mut rng);

        for (i, monster) in level.monsters.iter().enumerate() {
            if i < initial_ac.len() {
                assert!(i32::from(monster.ac) > i32::from(initial_ac[i]));
            }
        }
    }

    #[test]
    fn test_magic_mapping_blessed_reveals_all() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();

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

        let result = scroll_fire(&mut player, &mut level, true, false, false, &mut rng);
        assert_eq!(player.hp, 100); // Blessed protects player
        assert!(result.messages[0].contains("tower of flame") || result.messages[0].contains("erupts"));

        // Normal fire should damage non-fire-resistant player
        player.hp = 100;
        player.properties.remove_intrinsic(Property::FireResistance);
        let _ = scroll_fire(&mut player, &mut level, false, false, false, &mut rng);
        assert!(player.hp < 100);
    }

    #[test]
    fn test_fire_scroll_confused_burns_hands() {
        let mut player = You::default();
        let mut level = Level::new(DLevel::main_dungeon_start());
        player.hp = 100;
        let mut rng = crate::rng::GameRng::new(12345);

        let result = scroll_fire(&mut player, &mut level, false, false, true, &mut rng);
        assert!(result.messages[0].contains("burn") || result.messages[0].contains("fire"));
    }

    #[test]
    fn test_earth_scroll_blessed_has_larger_radius() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut player = You::default();
        player.pos = crate::player::Position { x: 10, y: 10 };
        let mut rng = crate::rng::GameRng::new(12345);

        for i in 0..5 {
            let mut monster = crate::monster::Monster::new(crate::monster::MonsterId(0), 0, 0, 0);
            monster.x = (10 + i) as i8;
            monster.y = (10 + i) as i8;
            monster.hp = 100;
            level.monsters.push(monster);
        }

        let initial_hp: Vec<i32> = level.monsters.iter().map(|m| m.hp).collect();
        let _ = scroll_earth(&mut player, &mut level, true, false, &mut rng);

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

        for i in 0..10 {
            let mut monster = crate::monster::Monster::new(crate::monster::MonsterId(0), 0, 0, 0);
            monster.x = (i * 2) as i8;
            monster.y = (i * 2) as i8;
            monster.state.confused = false;
            level.monsters.push(monster);
        }

        let result = scroll_stinking_cloud(&mut level, &player, true, false, &mut rng);
        let confused_count = level.monsters.iter().filter(|m| m.state.confused).count();
        assert!(confused_count > 0);
        assert!(result.messages[0].contains("fragrance"));
    }

    #[test]
    fn test_amnesia_forgets_map() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut player = You::default();
        let mut rng = crate::rng::GameRng::new(12345);

        // Explore entire map
        for x in 0..crate::COLNO {
            for y in 0..crate::ROWNO {
                level.cells[x][y].explored = true;
            }
        }

        // Normal amnesia should forget most of the map
        let _ = scroll_amnesia(&mut player, &mut level, false, false, &mut rng);
        let explored_count = (0..crate::COLNO)
            .flat_map(|x| (0..crate::ROWNO).map(move |y| (x, y)))
            .filter(|&(x, y)| level.cells[x][y].explored)
            .count();
        // Should have forgotten most cells (6/7 of map)
        let total = crate::COLNO * crate::ROWNO;
        assert!(explored_count < total / 2);
    }

    #[test]
    fn test_amnesia_forgets_spells() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut player = You::default();
        let mut rng = crate::rng::GameRng::new(12345);

        // Give player some spells
        player.known_spells.push(crate::magic::spell::KnownSpell {
            spell_type: crate::magic::spell::SpellType::ForceBolt,
            turns_remaining: 10000,
            times_cast: 0,
            times_failed: 0,
        });
        assert!(!player.known_spells.is_empty());

        // Non-blessed amnesia should clear spells
        let _ = scroll_amnesia(&mut player, &mut level, false, false, &mut rng);
        assert!(player.known_spells.is_empty());
    }

    #[test]
    fn test_charging_confused_restores_energy() {
        let mut player = You::default();
        player.energy = 10;
        player.energy_max = 50;
        let mut rng = crate::rng::GameRng::new(12345);

        let result = scroll_charging(&mut player, false, false, true, &mut rng);
        assert!(result.messages[0].contains("charged up"));
        assert!(player.energy > 10); // Energy increased
    }

    #[test]
    fn test_charging_confused_cursed_drains_energy() {
        let mut player = You::default();
        player.energy = 30;
        player.energy_max = 50;
        let mut rng = crate::rng::GameRng::new(12345);

        let result = scroll_charging(&mut player, false, true, true, &mut rng);
        assert!(result.messages[0].contains("discharged"));
        assert_eq!(player.energy, 0);
    }

    #[test]
    fn test_identify_confused() {
        let mut player = You::default();
        let result = scroll_identify(&mut player, false, true);
        assert!(result.messages[0].contains("identify this as an identify scroll"));
    }

    #[test]
    fn test_recharge_wand() {
        let mut wand = Object::new(crate::object::ObjectId(1), 365, ObjectClass::Wand); // wand of light
        wand.enchantment = 2;
        wand.recharged = 0;
        let mut rng = crate::rng::GameRng::new(12345);

        let msgs = recharge(&mut wand, 0, &mut rng);
        assert!(wand.enchantment > 2);
        assert_eq!(wand.recharged, 1);
        assert!(!msgs.is_empty());
    }

    #[test]
    fn test_recharge_wand_cursed_strips() {
        let mut wand = Object::new(crate::object::ObjectId(1), 365, ObjectClass::Wand);
        wand.enchantment = 5;
        wand.recharged = 0;
        let mut rng = crate::rng::GameRng::new(12345);

        let msgs = recharge(&mut wand, -1, &mut rng);
        assert_eq!(wand.enchantment, 0);
        assert!(msgs.iter().any(|m| m.contains("vibrate")));
    }

    #[test]
    fn test_recharge_wand_of_wishing_cap() {
        let mut wand = Object::new(crate::object::ObjectId(1), WAN_WISHING, ObjectClass::Wand);
        wand.enchantment = 0;
        wand.recharged = 0;
        let mut rng = crate::rng::GameRng::new(12345);

        let _msgs = recharge(&mut wand, 1, &mut rng);
        // Should get 3 charges max for wand of wishing
        assert!(wand.enchantment <= 3 || wand.enchantment == -1); // -1 = exploded
    }

    #[test]
    fn test_gold_detection_not_creation() {
        let level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();
        let mut rng = crate::rng::GameRng::new(12345);

        let result = scroll_gold_detection(&level, &player, false, &mut rng);
        // Should NOT add gold to player (old bug)
        assert_eq!(player.gold, 0);
        // Should report detection result
        assert!(!result.messages.is_empty());
    }

    #[test]
    fn test_food_detection_not_creation() {
        let level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();
        let mut rng = crate::rng::GameRng::new(12345);
        let initial_nutrition = player.nutrition;

        let result = scroll_food_detection(&level, &player, &mut rng);
        // Should NOT add nutrition to player (old bug)
        assert_eq!(player.nutrition, initial_nutrition);
        assert!(!result.messages.is_empty());
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
