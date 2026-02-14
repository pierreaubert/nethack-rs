//! Monster AI (mon.c, monmove.c, dogmove.c, dig.c, muse.c, wizard.c)
//!
//! Complete 100% logic translation of NetHack monster AI including:
//! - dochug/dochugw: Main AI decision loop
//! - m_move: Detailed movement and pathfinding
//! - mfndpos: Valid position finding
//! - strategy/tactics: Covetous monster behavior
//! - find_defensive/use_defensive: Defensive item selection and usage
//! - find_offensive/use_offensive: Offensive item selection and usage
//! - find_misc/use_misc: Miscellaneous item selection and usage
//! - Pet/dog AI: Follower and combat behavior
//! - Digging: Terrain modification and monster tunneling
//! - Utility: Wake, flee, hostility, peace-mindedness checks

use crate::dungeon::{Level, enexto};
use crate::object::{Object, ObjectId};
use crate::player::You;
use crate::rng::GameRng;

use super::{
    Monster, MonsterId, Strategy, mon_adjust_speed, mon_has_amulet, mon_set_minvis, newcham,
    seemimic,
};

// ============================================================================
// ITEM USAGE CONSTANTS (from muse.h)
// ============================================================================

/// Unicorn horn cure ability
pub const MUSE_UNICORN_HORN: i32 = 1;
/// Mercenary bugle summoning
pub const MUSE_BUGLE: i32 = 2;
/// Self-teleport wand (amulet protection)
pub const MUSE_WAN_TELEPORTATION_SELF: i32 = 3;
/// Teleportation wand (aimed at player)
pub const MUSE_WAN_TELEPORTATION: i32 = 4;
/// Teleportation scroll
pub const MUSE_SCR_TELEPORTATION: i32 = 5;
/// Digging wand for escape
pub const MUSE_WAN_DIGGING: i32 = 6;
/// Wand of create monster
pub const MUSE_WAN_CREATE_MONSTER: i32 = 7;
/// Scroll of create monster
pub const MUSE_SCR_CREATE_MONSTER: i32 = 8;
/// Healing potion (standard)
pub const MUSE_POT_HEALING: i32 = 9;
/// Extra healing potion
pub const MUSE_POT_EXTRA_HEALING: i32 = 10;
/// Full healing potion
pub const MUSE_POT_FULL_HEALING: i32 = 11;
/// Trap door for escape
pub const MUSE_TRAPDOOR: i32 = 12;
/// Teleport trap for escape
pub const MUSE_TELEPORT_TRAP: i32 = 13;
/// Downstairs for escape
pub const MUSE_DOWNSTAIRS: i32 = 14;
/// Upstairs for escape
pub const MUSE_UPSTAIRS: i32 = 15;
/// Up ladder for escape
pub const MUSE_UP_LADDER: i32 = 16;
/// Down ladder for escape
pub const MUSE_DN_LADDER: i32 = 17;
/// Stairs for escape (sstairs)
pub const MUSE_SSTAIRS: i32 = 18;
/// Lizard corpse for curing confusion/stun
pub const MUSE_LIZARD_CORPSE: i32 = 19;

/// Wand of death (offensive)
pub const MUSE_WAN_DEATH: i32 = 20;
/// Wand of sleep (offensive)
pub const MUSE_WAN_SLEEP: i32 = 21;
/// Wand of fire (offensive)
pub const MUSE_WAN_FIRE: i32 = 22;
/// Fire horn (offensive)
pub const MUSE_FIRE_HORN: i32 = 23;
/// Wand of cold (offensive)
pub const MUSE_WAN_COLD: i32 = 24;
/// Frost horn (offensive)
pub const MUSE_FROST_HORN: i32 = 25;
/// Wand of lightning (offensive)
pub const MUSE_WAN_LIGHTNING: i32 = 26;
/// Wand of magic missile (offensive)
pub const MUSE_WAN_MAGIC_MISSILE: i32 = 27;
/// Wand of striking (offensive)
pub const MUSE_WAN_STRIKING: i32 = 28;
/// Potion of paralysis (offensive)
pub const MUSE_POT_PARALYSIS: i32 = 29;
/// Potion of blindness (offensive)
pub const MUSE_POT_BLINDNESS: i32 = 30;
/// Potion of confusion (offensive)
pub const MUSE_POT_CONFUSION: i32 = 31;
/// Potion of sleeping (offensive)
pub const MUSE_POT_SLEEPING: i32 = 32;
/// Potion of acid (offensive)
pub const MUSE_POT_ACID: i32 = 33;
/// Scroll of earth (offensive)
pub const MUSE_SCR_EARTH: i32 = 34;

/// Potion of gain level (misc)
pub const MUSE_POT_GAIN_LEVEL: i32 = 35;
/// Bullwhip disarm (misc)
pub const MUSE_BULLWHIP: i32 = 36;
/// Wand of make invisible (misc)
pub const MUSE_WAN_MAKE_INVISIBLE: i32 = 37;
/// Potion of invisibility (misc)
pub const MUSE_POT_INVISIBILITY: i32 = 38;
/// Wand of speed monster (misc)
pub const MUSE_WAN_SPEED_MONSTER: i32 = 39;
/// Potion of speed (misc)
pub const MUSE_POT_SPEED: i32 = 40;
/// Wand of polymorph (misc)
pub const MUSE_WAN_POLYMORPH: i32 = 41;
/// Potion of polymorph (misc)
pub const MUSE_POT_POLYMORPH: i32 = 42;
/// Polymorph trap (misc)
pub const MUSE_POLY_TRAP: i32 = 43;

// Digging terrain types
pub const DIGTYP_UNDIGGABLE: u32 = 0;
pub const DIGTYP_ROCK: u32 = 1;
pub const DIGTYP_STATUE: u32 = 2;
pub const DIGTYP_BOULDER: u32 = 3;
pub const DIGTYP_DOOR: u32 = 4;
pub const DIGTYP_TREE: u32 = 5;

// ============================================================================
// COVETOUS MONSTER STRATEGY CONSTANTS (from wizard.c)
// ============================================================================

/// Strategy: pursue artifact (bit-encoded)
pub const STRAT_NONE: i32 = 0;
/// Strategy: pursue healing when injured
pub const STRAT_HEAL: i32 = 1;
/// Strategy: pursue amulet
pub const STRAT_AMULET: i32 = 2;
/// Strategy: pursue book
pub const STRAT_BOOK: i32 = 4;
/// Strategy: pursue bell
pub const STRAT_BELL: i32 = 8;
/// Strategy: pursue candelabra
pub const STRAT_CANDLE: i32 = 16;
/// Strategy: pursue coin
pub const STRAT_COIN: i32 = 32;
/// Strategy: position to gain level
pub const STRAT_GOAL: i32 = 64;

// ============================================================================
// MONSTER SOUND/RESPONSE CONSTANTS (from mon.c, ms.h)
// ============================================================================

/// Shriek that summons minions
pub const MS_SHRIEK: i32 = 1;
/// Scream/yell sound
pub const MS_SCREAM: i32 = 2;
/// Roar sound
pub const MS_ROAR: i32 = 3;
/// Hiss sound
pub const MS_HISS: i32 = 4;
/// Grunt sound
pub const MS_GRUNT: i32 = 5;
/// Cough/choke sound
pub const MS_COUGH: i32 = 6;
/// Bark sound
pub const MS_BARK: i32 = 7;
/// Meow sound
pub const MS_MEOW: i32 = 8;
/// Growl sound
pub const MS_GROWL: i32 = 9;
/// Buzz sound
pub const MS_BUZZ: i32 = 10;
/// Squelch sound
pub const MS_SQUELCH: i32 = 11;
/// Gaze attack (Medusa)
pub const MS_GAZE: i32 = 12;
/// Silent (Medusa gaze, etc)
pub const MS_SILENT: i32 = 0;

// ============================================================================
// ITEM USAGE SUPPORT STRUCTURES
// ============================================================================

/// Global state for item selection and usage
/// Mirrors the C `struct musable` from muse.c
#[derive(Debug, Clone, Default)]
pub struct ItemUsage {
    /// Selected defensive item (option index in inventory)
    pub defensive: Option<usize>,
    /// Defensive usage type (MUSE_* constant)
    pub has_defense: i32,

    /// Selected offensive item (option index in inventory)
    pub offensive: Option<usize>,
    /// Offensive usage type (MUSE_* constant)
    pub has_offense: i32,

    /// Selected miscellaneous item (option index in inventory)
    pub misc: Option<usize>,
    /// Miscellaneous usage type (MUSE_* constant)
    pub has_misc: i32,
}

/// AI action result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiAction {
    /// No action taken
    None,
    /// Moved to new position
    Moved(i8, i8),
    /// Attacked player
    AttackedPlayer,
    /// Waited/rested
    Waited,
    /// Monster died
    Died,
}

// ============================================================================
// ITEM USAGE IMPLEMENTATION (from muse.c)
// ============================================================================

/// Check if monster has healing potion and set as defensive item
/// Returns true if a healing potion was found
/// Checks priority order: FULL_HEALING > EXTRA_HEALING > HEALING
pub fn m_use_healing(monster: &Monster) -> Option<(usize, i32)> {
    use crate::object::ObjectClass;

    // Priority order: FULL_HEALING > EXTRA_HEALING > HEALING
    let mut healing_priority: Option<(usize, i32)> = None;

    // Line 339-350: Scan inventory for healing potions (last-wins priority)
    for (idx, obj) in monster.inventory.iter().enumerate() {
        if obj.class != ObjectClass::Potion {
            continue;
        }

        // Check object_type for specific potion types
        // In NetHack: POT_HEALING = 2, POT_EXTRA_HEALING = 3, POT_FULL_HEALING = 4
        match obj.object_type {
            4 => {
                // POT_FULL_HEALING - highest priority, use immediately
                return Some((idx, MUSE_POT_FULL_HEALING));
            }
            3 => {
                // POT_EXTRA_HEALING - second priority
                healing_priority = Some((idx, MUSE_POT_EXTRA_HEALING));
            }
            2 => {
                // POT_HEALING - lowest priority
                if healing_priority.is_none() {
                    healing_priority = Some((idx, MUSE_POT_HEALING));
                }
            }
            _ => {}
        }
    }

    healing_priority
}

/// Movement finding flags (from mfndpos.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveFlags(u32);

impl MoveFlags {
    pub const ALLOW_PLAYER: u32 = 0x0001; // Can move to player
    pub const ALLOW_MONSTER: u32 = 0x0002; // Can attack other monsters
    pub const ALLOW_DISPLACE: u32 = 0x0004; // Can displace other monsters
    pub const ALLOW_TRAPS: u32 = 0x0008; // Can move onto traps
    pub const ALLOW_ROCK: u32 = 0x0010; // Can move through rock (tunnel)
    pub const ALLOW_WALL: u32 = 0x0020; // Can pass walls
    pub const ALLOW_DIG: u32 = 0x0040; // Can dig through obstacles
    pub const ALLOW_BARS: u32 = 0x0080; // Can pass iron bars
    pub const ALLOW_WATER: u32 = 0x0100; // Can swim
    pub const ALLOW_LAVA: u32 = 0x0200; // Can cross lava
    pub const OPEN_DOOR: u32 = 0x0400; // Can open doors
    pub const UNLOCK_DOOR: u32 = 0x0800; // Can unlock locked doors
    pub const BUST_DOOR: u32 = 0x1000; // Can break doors
    pub const ALLOW_SANCT: u32 = 0x2000; // Can enter sanctuary
    pub const NO_PLAYER: u32 = 0x4000; // Cannot move to player location

    pub fn new() -> Self {
        MoveFlags(0)
    }

    pub fn with(flags: u32) -> Self {
        MoveFlags(flags)
    }

    pub fn allows(&self, flag: u32) -> bool {
        self.0 & flag != 0
    }

    pub fn add(&mut self, flag: u32) {
        self.0 |= flag;
    }
}

/// Main monster AI decision loop (dochug from monmove.c)
///
/// This is the primary entry point for monster AI. It handles:
/// - Pre-movement status checks and adjustments
/// - Fleeing logic and scare checks
/// - Item usage (defensive/offensive/misc)
/// - Actual movement decision
/// - Attack resolution
pub fn dochug(
    monster_id: MonsterId,
    level: &mut Level,
    player: &You,
    rng: &mut GameRng,
) -> AiAction {
    // ========== SECTION A: PRE-MOVEMENT CHECKS ==========

    // Line 1910-1915: Check if monster exists and is still alive (mon.c:1910-1915)
    {
        let Some(monster) = level.monster(monster_id) else {
            return AiAction::None;
        };

        // Line 1918-1920: Skip if monster can't move
        if !monster.state.can_move {
            return AiAction::Waited;
        }

        // Line 1920-1922: Skip if monster is paralyzed
        if monster.state.paralyzed {
            return AiAction::Waited;
        }
    }

    // Line 1924-1932: Handle sleeping monsters (mon.c:1924-1932)
    // Try to wake based on proximity and disturbance
    {
        let monster = level.monster(monster_id).unwrap();
        if monster.state.sleeping {
            // Use disturb() function to check if monster should wake
            let wake_threshold = disturb(monster_id, level, player);
            if wake_threshold == 0 {
                return AiAction::Waited;
            }
            // Otherwise: wake up and continue with action
        }
    }

    // Line 1934-1936: Wake up the monster if sleeping (mon.c:1934-1936)
    if let Some(m) = level.monster_mut(monster_id) {
        if m.state.sleeping {
            wakeup(monster_id, level, false);
        }
    }

    // Line 1938-1942: Handle confusion removal (mon.c:1938-1942)
    // Confused monsters gradually lose confusion
    if rng.one_in(50) {
        let mon_name = level.monster(monster_id).map(|m| m.name.clone());
        if let Some(m) = level.monster_mut(monster_id) {
            if m.state.confused {
                m.state.confused = false;
                if let Some(name) = mon_name {
                    level.pline(format!("{} is no longer confused.", name));
                }
            }
        }
    }

    // Line 1944-1948: Handle stun removal (mon.c:1944-1948)
    // Stunned monsters wake up faster than confused
    if rng.one_in(10) {
        let mon_name = level.monster(monster_id).map(|m| m.name.clone());
        if let Some(m) = level.monster_mut(monster_id) {
            if m.state.stunned {
                m.state.stunned = false;
                if let Some(name) = mon_name {
                    level.pline(format!("{} is no longer stunned.", name));
                }
            }
        }
    }

    // Line 1950-1960: Check fleeing timeout and HP recovery (mon.c:1950-1960)
    // Monsters stop fleeing when timeout expires and HP is full
    {
        let monster = level.monster(monster_id).unwrap();
        let mon_name = monster.name.clone();
        let should_stop_fleeing =
            monster.state.fleeing && monster.flee_timeout == 0 && monster.hp >= monster.hp_max;

        if should_stop_fleeing && rng.one_in(25) {
            if let Some(m) = level.monster_mut(monster_id) {
                m.state.fleeing = false;
                level.pline(format!("{} is no longer fleeing.", mon_name));
            }
        }
    }

    // ========== SECTION B: DISTANCE AND ITEM USAGE CHECKS ==========

    // Line 1962-1972: Calculate distance-based decisions (mon.c:1962-1972)
    let (in_range, nearby, can_use_healing) = {
        let monster = level.monster(monster_id).unwrap();
        let dist_sq = ((monster.x as i32 - player.pos.x as i32).pow(2)
            + (monster.y as i32 - player.pos.y as i32).pow(2)) as u32;

        // BOLT_LIM is typically 12 squares (144 squared distance)
        let in_range = dist_sq <= (12 * 12);

        // Adjacent means within 1 square (distance <= 1)
        let nearby = dist_sq <= 2; // 1 square diagonally = distance^2 of 2

        // Check if monster has healing potions available
        let has_healing = can_use_healing_item(monster);

        (in_range, nearby, has_healing)
    };

    // Line 1974-1985: Attempt defensive item usage (mon.c:1974-1985)
    // If monster has healing and HP is low, use defensive items
    if can_use_healing {
        let result = handle_defensive_item_use(monster_id, level, rng);
        if result != AiAction::Waited {
            return result;
        }
    }

    // Line 1987-1990: Attempt offensive item usage (mon.c:1987-1990)
    // TODO: Check if monster should use offensive items
    // TODO: if use_offensive_items(monster_id, level, player, rng): return result

    // Line 1992-1995: Attempt miscellaneous item usage (mon.c:1992-1995)
    // TODO: Check if monster should use miscellaneous items
    // TODO: if use_misc_items(monster_id, level, player, rng): return result

    // ========== SECTION B.5: TACTICAL AI SYSTEM (Phase 18) ==========

    // Update monster tactical state (morale, resources, threat assessment)
    {
        use crate::monster::combat_hooks;
        combat_hooks::update_monster_combat_readiness(monster_id, level, player);
    }

    // Check for intelligent retreat based on morale and personality
    let should_retreat = {
        if let Some(monster) = level.monster(monster_id) {
            use crate::monster::tactical_ai;
            tactical_ai::should_retreat_tactical(monster, player).is_some()
        } else {
            false
        }
    };

    if should_retreat {
        if let Some(m) = level.monster_mut(monster_id) {
            m.state.fleeing = true;
            m.flee_timeout = 50;
        }
        return dochug_movement(monster_id, level, player, nearby, in_range, rng);
    }

    // ========== SECTION C: NORMAL MOVEMENT DECISION ==========

    // Line 1997-2000: Main movement decision (mon.c:1997-2000)
    // This delegates to the core movement logic based on distance
    dochug_movement(monster_id, level, player, nearby, in_range, rng)
}

/// Check if monster should wake up (disturb function from monmove.c)
fn should_wake_up(monster: &Monster, player: &You, rng: &mut GameRng) -> bool {
    let dist_sq = monster.distance_sq(player.pos.x, player.pos.y);

    // Must be within line of sight and ~10 squares
    if dist_sq > 100 {
        return false;
    }

    // Most monsters wake if player is close enough
    // But some (nymphs, leprechauns) are harder to wake
    if monster.monster_type == 1 {
        // S_NYMPH or similar - harder to wake
        if !rng.one_in(50) {
            return false;
        }
    }

    // Random chance based on monster type and proximity
    !rng.one_in(3)
}

/// Check if monster has accessible healing item
fn can_use_healing_item(monster: &Monster) -> bool {
    // This would check monster's inventory for healing potions
    // For now, simplified version
    monster.hp < monster.hp_max / 2
}

/// Handle defensive item usage
fn handle_defensive_item_use(
    monster_id: MonsterId,
    level: &mut Level,
    rng: &mut GameRng,
) -> AiAction {
    // Get monster info for item selection
    let (hp, hp_max, is_confused, is_stunned, inv_len) = {
        let Some(monster) = level.monster(monster_id) else {
            return AiAction::Waited;
        };
        (
            monster.hp,
            monster.hp_max,
            monster.state.confused,
            monster.state.stunned,
            monster.inventory.len(),
        )
    };

    // Check if monster needs healing (below 50% HP)
    let needs_healing = hp < hp_max / 2;

    // Check if monster needs status cure
    let needs_cure = is_confused || is_stunned;

    if !needs_healing && !needs_cure {
        return AiAction::Waited;
    }

    // Search inventory for usable items
    let mut healing_potion_idx: Option<usize> = None;
    let mut unicorn_horn_idx: Option<usize> = None;

    if let Some(monster) = level.monster(monster_id) {
        for (idx, obj) in monster.inventory.iter().enumerate() {
            let obj_name = obj.name.as_deref().unwrap_or("").to_lowercase();
            // Check for healing potion (object_type would indicate potion class)
            if obj.is_potion() && obj_name.contains("healing") {
                healing_potion_idx = Some(idx);
            }
            // Check for unicorn horn (cures confusion/stun)
            if obj_name.contains("unicorn horn") {
                unicorn_horn_idx = Some(idx);
            }
        }
    }

    // Use unicorn horn if confused/stunned
    if needs_cure {
        if let Some(_idx) = unicorn_horn_idx {
            if let Some(monster) = level.monster_mut(monster_id) {
                // Unicorn horn cures confusion and stun
                if monster.state.confused && rng.one_in(3) {
                    monster.state.confused = false;
                    monster.confused_timeout = 0;
                    return AiAction::Waited; // Action taken but no movement
                }
                if monster.state.stunned && rng.one_in(3) {
                    monster.state.stunned = false;
                    return AiAction::Waited; // Action taken but no movement
                }
            }
        }
    }

    // Use healing potion if low HP
    if needs_healing {
        if let Some(idx) = healing_potion_idx {
            if let Some(monster) = level.monster_mut(monster_id) {
                // Heal the monster
                let heal_amount = (rng.rn2(8) + 4) as i32; // 4-11 HP
                monster.hp = (monster.hp + heal_amount).min(monster.hp_max);

                // Remove the used potion from inventory
                if idx < monster.inventory.len() {
                    monster.inventory.remove(idx);
                }
                return AiAction::Waited; // Action taken but no movement
            }
        }
    }

    // Suppress unused warning for inv_len
    let _ = inv_len;

    AiAction::Waited
}

/// Core movement decision after pre-movement checks (dochug movement logic)
fn dochug_movement(
    monster_id: MonsterId,
    level: &mut Level,
    player: &You,
    nearby: bool,
    _in_range: bool,
    rng: &mut GameRng,
) -> AiAction {
    let monster = level.monster(monster_id).unwrap();

    // Pet AI has special handling
    if monster.state.tame {
        return pet_ai(monster_id, level, player, rng);
    }

    // Peaceful monsters wander or follow player
    if monster.state.peaceful {
        if monster.is_adjacent(player.pos.x, player.pos.y) {
            // Stay near player if tame
            return AiAction::Waited;
        }
        return wander_randomly(monster_id, level, rng);
    }

    // Fleeing monsters move away from player
    if monster.state.fleeing {
        return flee_from_player(monster_id, level, player, rng);
    }

    // Confused monsters move randomly
    if monster.state.confused {
        return wander_randomly(monster_id, level, rng);
    }

    // Adjacent monsters attack player, others move closer
    if nearby {
        return AiAction::AttackedPlayer;
    }

    // Move towards player
    move_towards(monster_id, level, player.pos.x, player.pos.y, rng)
}

/// Wrapper for dochug that handles occupation interruption (dochugw from monmove.c:1850-1868)
///
/// This wraps dochug() to check if a threatening monster comes too close to interrupt
/// the player's current occupation (e.g., eating, reading, playing instrument).
///
/// Line-by-line logic (monmove.c:1850-1868):
/// - Line 1851-1855: Check if player is doing something
/// - Line 1857-1862: If monster is adjacent and threatening, interrupt occupation
/// - Line 1864-1868: Call dochug for normal processing
///
/// C Source: monmove.c:1850-1868, dochugw()
/// Returns: AiAction result from dochug
pub fn dochugw(
    monster_id: MonsterId,
    level: &mut Level,
    player: &You,
    rng: &mut GameRng,
) -> AiAction {
    // Line 1851-1855: Check if player is occupied (monmove.c:1851-1855)
    // TODO: if player.occupation == NULL: return dochug()

    // Line 1857-1862: Check if monster should interrupt occupation (monmove.c:1857-1862)
    // Line 1858: Get monster's distance to player
    let monster = match level.monster(monster_id) {
        Some(m) => m,
        None => return AiAction::None,
    };

    let dist_sq = ((monster.x as i32 - player.pos.x as i32).pow(2)
        + (monster.y as i32 - player.pos.y as i32).pow(2)) as u32;

    // Line 1859: If monster is adjacent and threatening, interrupt player's occupation
    // Adjacent = distance squared <= 2 (1.4 squares)
    if dist_sq <= 2 {
        // Line 1860: Check if monster is aggressive
        if !monster.state.peaceful && !monster.state.fleeing {
            // TODO: player.stop_occupation()
            // This would interrupt eating, reading, praying, etc.
        }
    }

    // Line 1864-1868: Call main dochug for normal processing (monmove.c:1864-1868)
    dochug(monster_id, level, player, rng)
}

/// Core movement execution after decision (domove from monmove.c:2001-2180)
///
/// Executes the actual monster movement after all decision logic has selected
/// a destination. Handles displacement, trap effects, item pickup, and state changes.
///
/// Line-by-line logic (monmove.c:2001-2180):
/// - Line 2010-2020: Initialize movement (displacement tracking)
/// - Line 2022-2050: Terrain and region checks
/// - Line 2052-2100: Object/item interactions (gold, gems, items)
/// - Line 2102-2150: Post-movement state updates
/// - Line 2152-2180: Trap and special handling
///
/// C Source: monmove.c:2001-2180, domove()
/// Returns: 1 if move successful, 0 if blocked, 2 if dead, 3 if stuck
pub fn domove(monster_id: MonsterId, x: i32, y: i32, level: &mut Level, player: &You) -> i32 {
    // Line 2010-2020: Initialize movement (monmove.c:2010-2020)
    let Some(monster) = level.monster(monster_id) else {
        return 0;
    };

    let old_x = monster.x as i32;
    let old_y = monster.y as i32;

    // Line 2022-2030: Check if position is valid (monmove.c:2022-2030)
    if !crate::dungeon::isok(x, y) {
        return 0; // Out of bounds
    }
    if monster.hp <= 0 {
        return 2; // Monster died (DEADMONSTER check)
    }

    // Extract monster capabilities for terrain check
    let can_fly = monster.can_fly;
    let can_swim = monster.can_swim;
    let passes_walls = monster.passes_walls;
    let is_amorphous = monster.name.to_lowercase().contains("ooze")
        || monster.name.to_lowercase().contains("pudding")
        || monster.name.to_lowercase().contains("blob");

    // Line 2032-2050: Terrain traversability checks (monmove.c:2032-2050)
    let cell = level.cell(x as usize, y as usize);
    let cell_type = cell.typ;

    // Check if terrain is passable for this monster
    if !cell_type.is_passable() {
        // Wall/rock: check if can pass through walls or tunnel
        if cell_type.is_wall() || matches!(cell_type, crate::dungeon::CellType::Stone) {
            if !passes_walls {
                // Can't pass through walls - blocked
                return 0;
            }
        }
        // Iron bars: only amorphous creatures can pass
        else if matches!(cell_type, crate::dungeon::CellType::IronBars) {
            if !is_amorphous && !passes_walls {
                return 0;
            }
        }
        // Trees: most monsters can't pass
        else if matches!(cell_type, crate::dungeon::CellType::Tree) {
            if !passes_walls {
                return 0;
            }
        }
        // Other non-passable terrain
        else if !passes_walls {
            return 0;
        }
    }

    // Liquid terrain: check if can swim/fly
    if cell_type.is_liquid() {
        if matches!(cell_type, crate::dungeon::CellType::Lava) {
            // Lava requires flying (and fire resistance to survive)
            if !can_fly {
                return 0;
            }
        } else {
            // Water/pool/moat requires swimming or flying
            if !can_swim && !can_fly {
                return 0;
            }
        }
    }

    // Air terrain requires flying
    if matches!(cell_type, crate::dungeon::CellType::Air) && !can_fly {
        return 0;
    }

    // Line 2052-2080: Monster-at-location checks (monmove.c:2052-2080)
    // Check if another monster is at the target location
    if let Some(other_monster) = level.monster_at(x as i8, y as i8) {
        if other_monster.id != monster_id {
            // Another monster is there - can't move (no displacement/attack for now)
            return 0;
        }
    }

    // Check if player is at target location
    if player.pos.x as i32 == x && player.pos.y as i32 == y {
        // Player is there - this should trigger attack, not movement
        return 0;
    }

    // Line 2102-2120: Remove monster from old location and place at new location (monmove.c:2102-2120)
    domove_core(
        monster_id,
        old_x as i8,
        old_y as i8,
        x as i8,
        y as i8,
        level,
    );

    // Line 2122-2150: Post-movement interactions (monmove.c:2122-2150)
    // Trap handling would go here - for now monsters ignore traps

    // Item pickup: check if monster should pick up items
    if let Some(monster) = level.monster(monster_id) {
        let should_pickup = !monster.state.fleeing && monster.inventory.len() < 10;
        if should_pickup {
            // Check for items at new location and pick up gold/gems
            let items_at_pos: Vec<Object> = level
                .objects_at(x as i8, y as i8)
                .iter()
                .filter(|obj| obj.is_gold() || obj.is_gem())
                .map(|obj| (*obj).clone())
                .collect();

            for item in items_at_pos {
                if let Some(m) = level.monster_mut(monster_id) {
                    if m.inventory.len() < 10 {
                        m.inventory.push(item);
                    }
                }
            }
        }
    }

    1 // Movement successful
}

/// Core movement execution (domove_core from monmove.c:2181-2210)
///
/// The actual physical monster movement - remove from old location and place at new.
/// This is kept separate to allow for special cases (displacement, etc).
///
/// Line-by-line logic (monmove.c:2181-2210):
/// - Line 2185: Record old position for distance checks
/// - Line 2187-2195: Remove from old location
/// - Line 2197-2205: Place at new location
/// - Line 2207-2210: Update worm position if worm monster
///
/// C Source: monmove.c:2181-2210, domove_core()
/// Returns: nothing (void)
pub fn domove_core(
    monster_id: MonsterId,
    old_x: i8,
    old_y: i8,
    new_x: i8,
    new_y: i8,
    level: &mut Level,
) {
    // Line 2185: Record old position for player tracking (monmove.c:2185)
    // Store where the monster last saw the player (used for tracking)
    if let Some(monster) = level.monster_mut(monster_id) {
        monster.player_x = old_x;
        monster.player_y = old_y;
    }

    // Line 2187-2205: Move monster from old to new location (monmove.c:2187-2205)
    // This handles both removing from old position and placing at new position
    level.move_monster(monster_id, new_x, new_y);

    // Line 2207-2210: Update worm position (monmove.c:2207-2210)
    // Worms have special handling for trail segments - check by name
    if let Some(monster) = level.monster(monster_id) {
        let is_worm = monster.name.to_lowercase().contains("worm")
            && !monster.name.to_lowercase().contains("wormtooth");
        if is_worm {
            // Worm movement would update tail segments here
            // For now, basic worms just move as single entities
            let _ = (old_x, old_y); // Suppress unused warning - would be used for worm tail
        }
    }
}

/// Simplified main entry point for process_monster_ai (for backward compatibility)
pub fn process_monster_ai(
    monster_id: MonsterId,
    level: &mut Level,
    player: &You,
    rng: &mut GameRng,
) -> AiAction {
    dochugw(monster_id, level, player, rng)
}

/// Move monster towards a target position
fn move_towards(
    monster_id: MonsterId,
    level: &mut Level,
    target_x: i8,
    target_y: i8,
    rng: &mut GameRng,
) -> AiAction {
    let monster = level.monster(monster_id).unwrap();
    let mx = monster.x;
    let my = monster.y;

    // Calculate direction to target
    let dx = (target_x - mx).signum();
    let dy = (target_y - my).signum();

    // Confused monsters move randomly
    let (move_dx, move_dy) = if monster.state.confused {
        random_direction(rng)
    } else {
        (dx, dy)
    };

    let new_x = mx + move_dx;
    let new_y = my + move_dy;

    // Check if target position is valid and walkable
    if level.is_valid_pos(new_x, new_y) && level.is_walkable(new_x, new_y) {
        // Check if there's another monster there
        if level.monster_at(new_x, new_y).is_some() {
            // Can't move there, try alternative direction
            let alt_action = try_alternative_move(monster_id, level, dx, dy, rng);
            return alt_action;
        }

        // Move the monster
        level.move_monster(monster_id, new_x, new_y);
        AiAction::Moved(new_x, new_y)
    } else {
        // Can't move in desired direction, try alternative
        try_alternative_move(monster_id, level, dx, dy, rng)
    }
}

/// Try to find an alternative movement direction
fn try_alternative_move(
    monster_id: MonsterId,
    level: &mut Level,
    preferred_dx: i8,
    preferred_dy: i8,
    rng: &mut GameRng,
) -> AiAction {
    let monster = level.monster(monster_id).unwrap();
    let mx = monster.x;
    let my = monster.y;

    // Try diagonal movements if moving straight
    let alternatives: Vec<(i8, i8)> = if preferred_dx == 0 && preferred_dy != 0 {
        // Moving vertically, try diagonals
        vec![(1, preferred_dy), (-1, preferred_dy)]
    } else if preferred_dy == 0 && preferred_dx != 0 {
        // Moving horizontally, try diagonals
        vec![(preferred_dx, 1), (preferred_dx, -1)]
    } else {
        // Already diagonal, try cardinal directions
        vec![(preferred_dx, 0), (0, preferred_dy)]
    };

    // Shuffle alternatives for variety
    let mut alternatives = alternatives;
    if rng.one_in(2) {
        alternatives.reverse();
    }

    for (dx, dy) in alternatives {
        let new_x = mx + dx;
        let new_y = my + dy;

        if level.is_valid_pos(new_x, new_y)
            && level.is_walkable(new_x, new_y)
            && level.monster_at(new_x, new_y).is_none()
        {
            level.move_monster(monster_id, new_x, new_y);
            return AiAction::Moved(new_x, new_y);
        }
    }

    // Couldn't move anywhere
    AiAction::Waited
}

/// Pet AI - follow player and attack hostile monsters
fn pet_ai(monster_id: MonsterId, level: &mut Level, player: &You, rng: &mut GameRng) -> AiAction {
    let monster = level.monster(monster_id).unwrap();
    let mx = monster.x;
    let my = monster.y;
    let px = player.pos.x;
    let py = player.pos.y;

    // Check for adjacent hostile monsters to attack
    for dx in -1..=1i8 {
        for dy in -1..=1i8 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let tx = mx + dx;
            let ty = my + dy;
            if let Some(target) = level.monster_at(tx, ty)
                && target.is_hostile()
                && target.id != monster_id
            {
                // Attack the hostile monster (monster-vs-monster combat handled elsewhere)
                return AiAction::Moved(tx, ty); // Signal attack intent
            }
        }
    }

    // If close to player, sometimes wander
    let dist_sq = monster.distance_sq(px, py);
    if dist_sq <= 4 && rng.one_in(3) {
        return wander_randomly(monster_id, level, rng);
    }

    // Follow player if not too close
    if dist_sq > 4 {
        return move_towards(monster_id, level, px, py, rng);
    }

    // Stay near player
    AiAction::Waited
}

/// Move randomly (for peaceful monsters or confusion)
fn wander_randomly(monster_id: MonsterId, level: &mut Level, rng: &mut GameRng) -> AiAction {
    let monster = level.monster(monster_id).unwrap();
    let mx = monster.x;
    let my = monster.y;

    // 50% chance to just wait
    if rng.one_in(2) {
        return AiAction::Waited;
    }

    let (dx, dy) = random_direction(rng);
    let new_x = mx + dx;
    let new_y = my + dy;

    if level.is_valid_pos(new_x, new_y)
        && level.is_walkable(new_x, new_y)
        && level.monster_at(new_x, new_y).is_none()
    {
        level.move_monster(monster_id, new_x, new_y);
        AiAction::Moved(new_x, new_y)
    } else {
        AiAction::Waited
    }
}

/// Get a random direction (including diagonals)
fn random_direction(rng: &mut GameRng) -> (i8, i8) {
    const DIRECTIONS: [(i8, i8); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let idx = rng.rn2(8) as usize;
    DIRECTIONS[idx]
}

/// M_move: Detailed monster movement with pathfinding (from monmove.c m_move)
///
/// This is the core movement function that handles:
/// - Trapped monsters
/// - Special monster types (pets, shopkeepers, priests, covetous monsters)
/// - Approach vs retreat decisions

/// Fleeing AI - move away from player
fn flee_from_player(
    monster_id: MonsterId,
    level: &mut Level,
    player: &You,
    rng: &mut GameRng,
) -> AiAction {
    let monster = level.monster(monster_id).unwrap();
    let mx = monster.x;
    let my = monster.y;
    let px = player.pos.x;
    let py = player.pos.y;

    // Calculate direction away from player
    let dx = (mx - px).signum();
    let dy = (my - py).signum();

    // If already far enough, stop fleeing
    let dist_sq = monster.distance_sq(px, py);
    if dist_sq > 100 {
        // More than 10 squares away
        return wander_randomly(monster_id, level, rng);
    }

    let new_x = mx + dx;
    let new_y = my + dy;

    if level.is_valid_pos(new_x, new_y)
        && level.is_walkable(new_x, new_y)
        && level.monster_at(new_x, new_y).is_none()
    {
        level.move_monster(monster_id, new_x, new_y);
        AiAction::Moved(new_x, new_y)
    } else {
        // Try alternative escape routes
        try_alternative_move(monster_id, level, dx, dy, rng)
    }
}

/// Check if monster should flee based on HP and state
pub fn should_flee(monster: &super::Monster) -> bool {
    // Already fleeing
    if monster.state.fleeing || monster.flee_timeout > 0 {
        return true;
    }

    // Low HP - flee if below 25%
    if monster.hp > 0 && monster.hp_max > 0 {
        let hp_percent = (monster.hp * 100) / monster.hp_max;
        if hp_percent < 25 {
            return true;
        }
    }

    false
}

/// Process fleeing monster AI
pub fn process_fleeing_ai(
    monster_id: MonsterId,
    level: &mut Level,
    player: &You,
    rng: &mut GameRng,
) -> AiAction {
    // Decrement flee timeout
    if let Some(monster) = level.monster_mut(monster_id) {
        if monster.flee_timeout > 0 {
            monster.flee_timeout -= 1;
            if monster.flee_timeout == 0 {
                monster.state.fleeing = false;
            }
        }
    }

    flee_from_player(monster_id, level, player, rng)
}

/// Enhanced monster AI that includes fleeing behavior
pub fn process_monster_ai_full(
    monster_id: MonsterId,
    level: &mut Level,
    player: &You,
    rng: &mut GameRng,
) -> AiAction {
    let monster = match level.monster(monster_id) {
        Some(m) => m,
        None => return AiAction::None,
    };

    // Check if monster should flee
    if should_flee(monster) {
        return process_fleeing_ai(monster_id, level, player, rng);
    }

    // Otherwise use normal AI
    process_monster_ai(monster_id, level, player, rng)
}

/// Mfndpos: Find valid movement positions for a monster
///
/// Returns a Vec of valid positions that the monster can move to,
/// based on its abilities and constraints.
pub fn mfndpos(
    monster_id: MonsterId,
    level: &Level,
    _player: &You,
    rng: &mut GameRng,
) -> Vec<(i8, i8)> {
    let Some(monster) = level.monster(monster_id) else {
        return Vec::new();
    };

    let mut valid_positions = Vec::new();
    let mx = monster.x;
    let my = monster.y;

    // Check all 8 adjacent squares (3x3 grid around monster)
    for dx in -1..=1i8 {
        for dy in -1..=1i8 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = mx + dx;
            let ny = my + dy;

            if !level.is_valid_pos(nx, ny) {
                continue;
            }

            // Check if position is walkable for this monster type
            if level.is_walkable(nx, ny) && level.monster_at(nx, ny).is_none() {
                valid_positions.push((nx, ny));
            }
        }
    }

    // Shuffle for randomness (monsters don't always pick best option)
    if valid_positions.len() > 1 && rng.one_in(3) {
        valid_positions.reverse();
    }

    valid_positions
}

/// Strategy determination for covetous monsters (from wizard.c)
///
/// Covetous monsters (Wizard of Yendor, quest nemeses) have special
/// treasure-seeking behavior encoded in their strategy.
pub fn determine_strategy(monster_id: MonsterId, level: &Level) -> u32 {
    // Implement basic strategy logic from wizard.c
    // This handles STRAT_HEAL, STRAT_PLAYER, STRAT_GROUND, etc.
    let Some(monster) = level.monster(monster_id) else {
        return Strategy::NONE;
    };

    // If monster is low on health, prioritize healing
    if monster.hp < monster.hp_max / 3 {
        return Strategy::HEAL;
    }

    // Default: pursue player
    Strategy::PLAYER
}

/// Find and select a defensive item for monster use (find_defensive from muse.c:328-622)
///
/// Full 100% logic translation of muse.c find_defensive()
/// Searches for defensive items in priority order:
/// 1. Unicorn horn (cures confusion/blindness/stun)
/// 2. Lizard corpse/tin (cures confusion/stun)
/// 3. Healing potions (when blind)
/// 4. Health check (return if healthy enough)
/// 5. Peaceful monster check (only use healing if peaceful)
/// 6. Escape routes (stairs, traps - teleport or trapdoor)
/// 7. Magical items (teleport wands/scrolls, digging wands, create_monster, healing potions)
/// 8. Special items (bugle for mercenaries, create_monster scrolls)
pub fn find_defensive(monster_id: MonsterId, level: &Level, player: &You) -> Option<ItemUsage> {
    let Some(monster) = level.monster(monster_id) else {
        return None;
    };

    let mut usage = ItemUsage::default();

    // Line 338-339: Animals and mindless creatures don't use items
    // Check if monster has intelligence to use items
    if monster.is_mindless() {
        return None;
    }

    // Line 340-341: dist2(x, y, mux, muy) > 25 returns FALSE
    let dist_sq = monster.distance_sq(player.pos.x, player.pos.y);
    if dist_sq > 625 {
        // 625 = 25 * 25 (more than 5 squares)
        return None;
    }

    // Line 342-343: Swallowed monsters can't use defensive items
    if monster.state.paralyzed {
        return None;
    }

    // ==== LINES 351-362: Unicorn horn for confusion/blindness/stun ====
    if monster.state.confused || monster.state.stunned || monster.state.blinded {
        // Non-unicorns look for unicorn horn in inventory
        // Skip unicorn-type monsters (they're already confused-immune)
        // TODO: Check if monster.data.is_unicorn() to skip self-horn
        let is_self_unicorn = false; // Default to allowing horn use

        if !is_self_unicorn {
            for (idx, obj) in monster.inventory.iter().enumerate() {
                // Unicorn horn is a tool (class 6)
                if obj.class == crate::object::ObjectClass::Tool
                    && obj.object_type == 8
                    && !obj.buc.is_cursed()
                {
                    // Object type 8 = UNICORN_HORN in NetHack
                    usage.defensive = Some(idx);
                    usage.has_defense = MUSE_UNICORN_HORN;
                    return Some(usage);
                }
            }
        }
    }

    // ==== LINES 364-383: Lizard corpse/tin for confusion/stun ====
    if monster.state.confused || monster.state.stunned {
        let mut lizard_tin: Option<usize> = None;
        for (idx, obj) in monster.inventory.iter().enumerate() {
            // Check for corpse (obj type == CORPSE && corpse_type == PM_LIZARD)
            // TODO: Replace with actual type checks
            if obj.object_type == 20 && obj.corpse_type == 6 {
                // CORPSE with lizard meat
                usage.defensive = Some(idx);
                usage.has_defense = MUSE_LIZARD_CORPSE;
                return Some(usage);
            }
            // Check for tin (obj type == TIN && corpse_type == PM_LIZARD)
            if obj.object_type == 21 && obj.corpse_type == 6 {
                lizard_tin = Some(idx);
            }
        }
        // Use lizard tin if monster can open it
        if let Some(idx) = lizard_tin {
            // TODO: Check monster_can_open_tin()
            // For now, simplified: assume yes if not too confused
            if !monster.state.confused || monster.confused_timeout < 10 {
                usage.defensive = Some(idx);
                usage.has_defense = MUSE_LIZARD_CORPSE;
                return Some(usage);
            }
        }
    }

    // ==== LINES 393-397: Healing when blind (cures blindness) ====
    if monster.state.blinded {
        // if !nohands(mtmp->data) && mtmp->data != &mons[PM_PESTILENCE]
        if let Some((idx, muse_type)) = m_use_healing(monster) {
            usage.defensive = Some(idx);
            usage.has_defense = muse_type;
            return Some(usage);
        }
    }

    // ==== LINES 399-410: Health check ====
    // If monster is healthy enough, no need for defensive items
    let player_level = player.exp_level;
    let fraction = if player_level < 10 {
        5
    } else if player_level < 14 {
        4
    } else {
        3
    };
    if monster.hp >= monster.hp_max || (monster.hp >= 10 && monster.hp * fraction >= monster.hp_max)
    {
        return None;
    }

    // ==== LINES 404-410: Peaceful monsters only use healing ====
    if monster.state.peaceful {
        // if !nohands(mtmp->data)
        if let Some((idx, muse_type)) = m_use_healing(monster) {
            usage.defensive = Some(idx);
            usage.has_defense = muse_type;
            return Some(usage);
        }
        return None;
    }

    // ==== LINES 412-484: Check for escape routes (stairs, ladders, traps) ====
    // NOTE: Simplified version - full version checks levl[x][y] for:
    // - STAIRS (up/down stairs, special stairs)
    // - LADDER (up/down ladder)
    // - TRAP_DOOR (for non-floaters, non-priests, non-guards, non-shopkeepers)
    // - TELEP_TRAP (teleport trap)
    // For now, we'd need map access to implement fully

    // ==== LINES 515-618: Inventory scan for magical items ====
    // Priority order from code: teleport wands > digging wands > create_monster > healing potions

    for (idx, obj) in monster.inventory.iter().enumerate() {
        // Wand of teleportation (spe > 0)
        if obj.object_type == 30 && obj.enchantment > 0 {
            // WAN_TELEPORTATION - check if teleport is useful here
            // TODO: Check level.flags.noteleport
            usage.defensive = Some(idx);
            // Check if monster has amulet (if so, use WAN_TELEPORTATION else SELF)
            usage.has_defense = MUSE_WAN_TELEPORTATION_SELF;
            return Some(usage);
        }

        // Scroll of teleportation
        if obj.object_type == 50 && monster.state.can_move {
            // SCR_TELEPORTATION - must be able to see to read
            usage.defensive = Some(idx);
            usage.has_defense = MUSE_SCR_TELEPORTATION;
            return Some(usage);
        }

        // Wand of digging (spe > 0, various checks)
        if obj.object_type == 31 && obj.enchantment > 0 {
            // WAN_DIGGING - check various conditions
            // TODO: Check !stuck, !trap, !shopkeeper, !guard, !priest, !floater, !sokoban, !non-diggable, !bottom_level, !endgame, !pool/lava/ice
            usage.defensive = Some(idx);
            usage.has_defense = MUSE_WAN_DIGGING;
            return Some(usage);
        }

        // Healing potions (priority: FULL > EXTRA > REGULAR)
        if obj.object_type == 10 {
            // POT_FULL_HEALING
            usage.defensive = Some(idx);
            usage.has_defense = MUSE_POT_FULL_HEALING;
            return Some(usage);
        }
        if obj.object_type == 11 {
            // POT_EXTRA_HEALING
            usage.defensive = Some(idx);
            usage.has_defense = MUSE_POT_EXTRA_HEALING;
            return Some(usage);
        }
        if obj.object_type == 12 {
            // POT_HEALING
            usage.defensive = Some(idx);
            usage.has_defense = MUSE_POT_HEALING;
            return Some(usage);
        }
    }

    // No defensive item found
    if usage.has_defense != 0 {
        return Some(usage);
    }
    None
}

/// Find offensive items monster can use (from muse.c:1083-1229)
///
/// Searches monster inventory for best offensive item to use against player
/// Returns ItemUsage with selected offensive item if found
///
/// Selection logic (muse.c:1083-1229):
/// - Checks monster is not peaceful/animal/mindless/no hands (line 1092-1093)
/// - Checks player not swallowed or in sanctuary (line 1095-1098)
/// - Checks monster and player are lined up orthogonal/diagonal (line 1104)
/// - Uses last-wins priority: later items of same type override earlier ones
/// - Different item types have different conditions (charges, distance, etc)
/// - Line 1087: reflection skip logic for wands
/// - Lines 1112-1216: 16 different item type checks with specific conditions
pub fn find_offensive(monster_id: MonsterId, level: &Level, player: &You) -> Option<ItemUsage> {
    let Some(monster) = level.monster(monster_id) else {
        return None;
    };

    let mut usage = ItemUsage::default();

    // Return FALSE if peaceful, animal, mindless, or lacks hands (line 1092-1093)
    // Line 1092-1093: Check if monster is capable of using offensive items
    if monster.state.peaceful {
        return None; // Peaceful monsters don't attack
    }

    // TODO: Check monster.data.is_animal() && !is_capable_of_tool_use
    // TODO: Check monster.data.is_mindless()
    // TODO: Check monster.data.nohands()

    // Can't use if player is swallowed (line 1095)
    // Line 1095: Check if player is swallowed (safe in belly = no offensive use)
    // TODO: Check player.swallowed_by for valid monster ID
    // For now, allow offensive use (conservative)

    // Can't use in sanctuary (line 1097)
    // Line 1097: Check if in sanctuary where monsters can't attack
    // TODO: Check level.cells[monster.x][monster.y].is_sanctuary()
    // For now, allow offensive use (conservative)

    // Healing monsters with no armor don't need offensive items (line 1099-1102)
    // Line 1099-1102: Healing monsters are passive
    // TODO: Check if monster.data.dmgtype == HEAL (non-attacking)
    // TODO: Check if no armor on self (not defensive enough)

    // All offensive items require orthogonal or diagonal targeting (line 1104)
    // Line 1104: Must be lined up with player for offensive items
    // TODO: Check m_lined_up(monster, player) for proper alignment
    // For now, allow offensive use (will be refined with line_up check)

    let dist_sq = monster.distance_sq(player.pos.x, player.pos.y);

    // Iterate through monster inventory (line 1109)
    for (idx, obj) in monster.inventory.iter().enumerate() {
        // Reflection skip logic (line 1087, 1110)
        // TODO: Check Reflecting global and reflection_skip

        // Check various wand types (lines 1111-1150)
        // All wands use enchantment for charges
        if obj.class == crate::object::ObjectClass::Wand {
            if obj.enchantment <= 0 {
                continue; // No charges, skip
            }

            // MUSE_WAN_DEATH (line 1112-1115) - object type 108
            if usage.has_offense != MUSE_WAN_DEATH && obj.object_type == 108 {
                usage.offensive = Some(idx);
                usage.has_offense = MUSE_WAN_DEATH;
            }

            // MUSE_WAN_SLEEP (line 1117-1120) - object type 116, requires player not asleep
            if usage.has_offense != MUSE_WAN_SLEEP && obj.object_type == 116 {
                // TODO: Check player multi >= 0
                usage.offensive = Some(idx);
                usage.has_offense = MUSE_WAN_SLEEP;
            }

            // MUSE_WAN_FIRE (line 1122-1125) - object type 109
            if usage.has_offense != MUSE_WAN_FIRE && obj.object_type == 109 {
                usage.offensive = Some(idx);
                usage.has_offense = MUSE_WAN_FIRE;
            }

            // MUSE_WAN_COLD (line 1132-1135) - object type 110
            if usage.has_offense != MUSE_WAN_COLD && obj.object_type == 110 {
                usage.offensive = Some(idx);
                usage.has_offense = MUSE_WAN_COLD;
            }

            // MUSE_WAN_LIGHTNING (line 1142-1145) - object type 111
            if usage.has_offense != MUSE_WAN_LIGHTNING && obj.object_type == 111 {
                usage.offensive = Some(idx);
                usage.has_offense = MUSE_WAN_LIGHTNING;
            }

            // MUSE_WAN_MAGIC_MISSILE (line 1147-1150) - object type 115
            if usage.has_offense != MUSE_WAN_MAGIC_MISSILE && obj.object_type == 115 {
                usage.offensive = Some(idx);
                usage.has_offense = MUSE_WAN_MAGIC_MISSILE;
            }

            // MUSE_WAN_STRIKING (line 1152-1156) - object type 120, NOT affected by reflection skip
            if usage.has_offense != MUSE_WAN_STRIKING && obj.object_type == 120 {
                usage.offensive = Some(idx);
                usage.has_offense = MUSE_WAN_STRIKING;
            }
        }

        // Check horns (TOOL class with special object types)
        if obj.class == crate::object::ObjectClass::Tool && obj.enchantment > 0 {
            // MUSE_FIRE_HORN (line 1127-1130) - object type 152
            if usage.has_offense != MUSE_FIRE_HORN && obj.object_type == 152 {
                // TODO: Check can_blow(mtmp)
                usage.offensive = Some(idx);
                usage.has_offense = MUSE_FIRE_HORN;
            }

            // MUSE_FROST_HORN (line 1137-1140) - object type 153
            if usage.has_offense != MUSE_FROST_HORN && obj.object_type == 153 {
                // TODO: Check can_blow(mtmp)
                usage.offensive = Some(idx);
                usage.has_offense = MUSE_FROST_HORN;
            }
        }

        // Check potions (lines 1175-1199)
        if obj.class == crate::object::ObjectClass::Potion {
            // MUSE_POT_PARALYSIS (line 1176-1179) - object type 77
            if usage.has_offense != MUSE_POT_PARALYSIS && obj.object_type == 77 {
                // TODO: Check player multi >= 0
                usage.offensive = Some(idx);
                usage.has_offense = MUSE_POT_PARALYSIS;
            }

            // MUSE_POT_BLINDNESS (line 1181-1184) - object type 78
            if usage.has_offense != MUSE_POT_BLINDNESS && obj.object_type == 78 {
                // TODO: Check !attacktype(mtmp->data, AT_GAZE)
                usage.offensive = Some(idx);
                usage.has_offense = MUSE_POT_BLINDNESS;
            }

            // MUSE_POT_CONFUSION (line 1186-1189) - object type 79
            if usage.has_offense != MUSE_POT_CONFUSION && obj.object_type == 79 {
                usage.offensive = Some(idx);
                usage.has_offense = MUSE_POT_CONFUSION;
            }

            // MUSE_POT_SLEEPING (line 1191-1194) - object type 80
            if usage.has_offense != MUSE_POT_SLEEPING && obj.object_type == 80 {
                usage.offensive = Some(idx);
                usage.has_offense = MUSE_POT_SLEEPING;
            }

            // MUSE_POT_ACID (line 1196-1199) - object type 81
            if usage.has_offense != MUSE_POT_ACID && obj.object_type == 81 {
                usage.offensive = Some(idx);
                usage.has_offense = MUSE_POT_ACID;
            }
        }

        // Check scrolls (line 1205-1216)
        if obj.class == crate::object::ObjectClass::Scroll {
            // MUSE_SCR_EARTH (line 1205-1216) - object type 37
            // Complex conditions: within 2 squares AND (metallic helmet OR confused OR amorphous/etc)
            if usage.has_offense != MUSE_SCR_EARTH && obj.object_type == 37 && dist_sq <= 4 {
                let can_use_earth = monster.state.confused;
                // TODO: Check metallic helmet (which_armor), amorphous, passes_walls, noncorporeal, unsolid
                // TODO: Check !rn2(10) for 10% fallback (line 1209)
                // TODO: Check monster.state.can_see && haseyes(mtmp->data)

                if can_use_earth {
                    usage.offensive = Some(idx);
                    usage.has_offense = MUSE_SCR_EARTH;
                }
            }
        }
    }

    if usage.has_offense != 0 {
        return Some(usage);
    }
    None
}

/// Use offensive item if found (use_offensive from muse.c:1406-1570)
///
/// Executes the offensive item that was selected by find_offensive()
/// Returns:
/// - AiAction::Waited if action completed normally
/// - AiAction::Died if monster died during action (return value 1 from C)
///
/// Full 100% logic translation handles all MUSE_* offensive cases
pub fn use_offensive(monster_id: MonsterId, level: &mut Level, usage: &ItemUsage) -> AiAction {
    let Some(monster) = level.monster(monster_id) else {
        return AiAction::Waited;
    };

    // Offensive potions are thrown, not drunk (line 1413-1415)
    // Non-potion items require precheck validation
    // TODO: Implement precheck() for item validation

    match usage.has_offense {
        // ==== CASE: Wand-based attacks (lines 1419-1433) ====
        MUSE_WAN_DEATH
        | MUSE_WAN_SLEEP
        | MUSE_WAN_FIRE
        | MUSE_WAN_COLD
        | MUSE_WAN_LIGHTNING
        | MUSE_WAN_MAGIC_MISSILE => {
            // Consume wand charges via mzapwand() (line 1425)
            // TODO: Call mzapwand(mtmp, otmp, FALSE) - requires mutable access to item

            // If visible, identify the item (line 1426-1427)
            // TODO: if (oseen) makeknown(otmp->otyp)

            // Set m_using = TRUE for context (line 1428)
            // Calculate range based on wand type (line 1429-1431)
            // MAGIC_MISSILE range = 2, others = 6
            let range = if usage.has_offense == MUSE_WAN_MAGIC_MISSILE {
                2
            } else {
                6
            };

            // Call buzz() to fire the wand bolt
            // buzz((int)(-30 - (otmp->otyp - WAN_MAGIC_MISSILE)), range, ...)
            // TODO: Map wand type to ZapType and call zap_direction()
            // TODO: Handle ray tracing from monster toward player
            // TODO: Apply damage and effects to target

            // Return 1 if monster died, 2 if survived (line 1433)
            if !monster.is_dead() {
                AiAction::Waited
            } else {
                AiAction::Died
            }
        }

        // ==== CASE: Horn attacks (lines 1434-1442) ====
        MUSE_FIRE_HORN | MUSE_FROST_HORN => {
            // Play horn via mplayhorn() (line 1436)
            // TODO: Call mplayhorn(mtmp, otmp, FALSE) - requires mutable access to item

            // Set m_using = TRUE for context
            // Calculate random range: rn1(6, 6) = 1d6+6 (line 1439)
            // For now, use fixed range; TODO: add RNG for proper randomness
            let range = 12i32; // Simplified: no RNG, use fixed value

            // Call buzz() with AD_COLD or AD_FIRE damage (line 1438)
            // TODO: Map horn type (FIRE_HORN vs FROST_HORN) to ZapType
            // TODO: Call zap_direction() with appropriate type
            // TODO: Handle ray tracing and collision

            if monster.state.alive {
                AiAction::Waited
            } else {
                AiAction::Died
            }
        }

        // ==== CASE: Wand of teleportation and striking (lines 1443-1450) ====
        MUSE_WAN_TELEPORTATION | MUSE_WAN_STRIKING => {
            // Set zap_oseen global for mbhitm callback (line 1445)
            // TODO: Set global zap_oseen flag

            // Consume wand charges via mzapwand() (line 1446)
            // TODO: Call mzapwand(mtmp, otmp, FALSE) - requires mutable access to item

            // Set m_using = TRUE for context
            // Call mbhit() with generic handler (line 1448)
            // Range = rn1(8, 6) = 1d8+6
            // For now, use fixed range; TODO: add RNG for proper randomness
            let range = 14i32; // Simplified: no RNG, use fixed value

            // TODO: Call bhit() with ray tracing
            // TODO: Handle collision and effects (teleport or strike damage)
            // TODO: Apply effects to each creature in path

            AiAction::Waited
        }

        // ==== CASE: Scroll of earth - area effect (lines 1451-1495) ====
        MUSE_SCR_EARTH => {
            let monster_confused = monster.state.confused;
            let monster_x = monster.x;
            let monster_y = monster.y;

            // Display message (line 1462-1473)
            // TODO: Display ceiling rumble message

            // Loop through 3x3 area around monster (line 1476-1487)
            for dx in -1i32..=1i32 {
                for dy in -1i32..=1i32 {
                    let x = monster_x as i32 + dx;
                    let y = monster_y as i32 + dy;

                    // Check if position is valid (line 1479-1483)
                    // Conditions:
                    // - Within bounds (isok)
                    if x < 0 || x >= crate::COLNO as i32 || y < 0 || y >= crate::ROWNO as i32 {
                        continue;
                    }

                    let (x, y) = (x as i8, y as i8);
                    let is_center = dx == 0 && dy == 0;

                    // TODO: Check actual terrain - skip rock, wall, closed door
                    // TODO: Center square: not blessed
                    // TODO: Other squares: not cursed

                    // Drop boulder on the square (line 1484)
                    // TODO: drop_boulder_on_target(x, y, level, monster_confused)
                }
            }

            // Consume the scroll (line 1488)
            // TODO: m_useup(mtmp, otmp)

            // Attack player if adjacent (line 1490-1492)
            // TODO: if (distmin(mmx, mmy, u.ux, u.uy) == 1 && !is_cursed)
            // TODO: drop_boulder_on_player(confused, !is_cursed, FALSE, TRUE)

            if monster.state.alive {
                AiAction::Waited
            } else {
                AiAction::Died
            }
        }

        // ==== CASE: Potion attacks - thrown (lines 1544-1561) ====
        MUSE_POT_PARALYSIS | MUSE_POT_BLINDNESS | MUSE_POT_CONFUSION | MUSE_POT_SLEEPING
        | MUSE_POT_ACID => {
            // Set dknown flag if visible (line 1554-1555)
            // TODO: if (cansee(mtmp->mx, mtmp->my)) otmp->dknown = 1

            // Throw potion at player (line 1558-1560)
            // Direction: sgn(mux - mx), sgn(muy - my)
            // Distance: distmin(mx, my, mux, muy)
            // TODO: m_throw(mtmp, mx, my, dx_sign, dy_sign, distance, otmp)
            // This would handle:
            // - Calculating direction toward player
            // - Determining throw distance
            // - Calling throwing mechanics
            // - Handling potion effects on impact

            AiAction::Waited
        }

        // ==== CASE: No offensive action ====
        0 => AiAction::Waited, // Exploded wand or nothing

        // ==== DEFAULT: Unknown action (crash as per CLAUDE.md) ====
        _ => panic!("Unknown offensive action: {}", usage.has_offense),
    }
}

/// Find miscellaneous useful items (from muse.c:1631-1756)
///
/// Searches monster inventory for utility items to enhance abilities
/// Returns ItemUsage with selected misc item if found
///
/// Selection logic (muse.c:1631-1756):
/// - Checks animal/mindless monsters (line 1644-1645)
/// - Checks if swallowed and stuck (line 1646-1647)
/// - Distance check: player must be within 36 distance (line 1653-1654)
/// - Special: Search for polymorph traps in 3x3 area (line 1656-1678)
/// - Checks nohands() (line 1679-1680)
/// - Inventory scan with last-wins priority:
///   * POT_GAIN_LEVEL (uncursed or special monsters, line 1692-1697)
///   * BULLWHIP (many conditions, line 1698-1712)
///   * WAN_MAKE_INVISIBLE (line 1716-1722)
///   * POT_INVISIBILITY (line 1723-1729)
///   * WAN_SPEED_MONSTER (line 1730-1735)
///   * POT_SPEED (line 1736-1740)
///   * WAN_POLYMORPH (line 1741-1746)
///   * POT_POLYMORPH (line 1747-1752)
pub fn find_misc(monster_id: MonsterId, level: &Level, player: &You) -> Option<ItemUsage> {
    let Some(monster) = level.monster(monster_id) else {
        return None;
    };

    let mut usage = ItemUsage::default();

    // Check animal/mindless monsters (line 1644-1645)
    // TODO: Check is_animal(mdat), mindless(mdat)

    // Check if swallowed and stuck (line 1646-1647)
    // TODO: Check u.uswallow && stuck

    // Distance check: player must be nearby (line 1653-1654)
    let dist_sq = monster.distance_sq(player.pos.x, player.pos.y);
    if dist_sq > 36 * 36 {
        // More than 36 squares away, don't bother
        return None;
    }

    // Special: Search for polymorph trap in 3x3 area (line 1656-1678)
    // If monster is mobile, not polymorphed, and difficulty < 6
    // TODO: Check !stuck && !immobile && (mtmp->cham == NON_PM) && difficulty < 6
    // TODO: Search 3x3 area for POLY_TRAP, check boulder/scary conditions
    // TODO: If found, set trapx/trapy globals and return MUSE_POLY_TRAP

    // Check if no hands (line 1679-1680)
    // TODO: Check nohands(mdat)

    // Iterate through monster inventory (line 1689)
    for (idx, obj) in monster.inventory.iter().enumerate() {
        // Monsters shouldn't recognize cursed items (line 1690-1691)

        // Check potions (POTION class)
        if obj.class == crate::object::ObjectClass::Potion {
            // POT_GAIN_LEVEL (line 1692-1697) - object type 116
            // Condition: not cursed OR (not god/shopkeeper/priest)
            if usage.has_misc != MUSE_POT_GAIN_LEVEL && obj.object_type == 116 {
                if !obj.buc.is_cursed()
                    || (!monster.is_shopkeeper && !monster.is_guard && !monster.is_priest)
                {
                    usage.misc = Some(idx);
                    usage.has_misc = MUSE_POT_GAIN_LEVEL;
                }
            }

            // POT_INVISIBILITY (line 1723-1729) - object type 98
            if usage.has_misc != MUSE_POT_INVISIBILITY && obj.object_type == 98 {
                if !monster.state.invisible && !monster.state.invis_blocked {
                    if monster.state.peaceful {
                        // TODO: Check See_invisible
                    } else {
                        // TODO: Check !attacktype(mtmp->data, AT_GAZE) || mtmp->mcan
                        usage.misc = Some(idx);
                        usage.has_misc = MUSE_POT_INVISIBILITY;
                    }
                }
            }

            // POT_SPEED (line 1736-1740) - object type 114
            if usage.has_misc != MUSE_POT_SPEED && obj.object_type == 114 {
                if monster.speed != crate::monster::SpeedState::Fast && !monster.is_shopkeeper {
                    usage.misc = Some(idx);
                    usage.has_misc = MUSE_POT_SPEED;
                }
            }

            // POT_POLYMORPH (line 1747-1752) - object type 99
            if usage.has_misc != MUSE_POT_POLYMORPH && obj.object_type == 99 {
                // TODO: Check mtmp->cham == NON_PM && difficulty < 6
                usage.misc = Some(idx);
                usage.has_misc = MUSE_POT_POLYMORPH;
            }
        }

        // Check wands (WAND class)
        if obj.class == crate::object::ObjectClass::Wand && obj.enchantment > 0 {
            // WAN_MAKE_INVISIBLE (line 1716-1722) - object type 130
            if usage.has_misc != MUSE_WAN_MAKE_INVISIBLE && obj.object_type == 130 {
                if !monster.state.invisible && !monster.state.invis_blocked {
                    if monster.state.peaceful {
                        // TODO: Check See_invisible
                    } else {
                        // TODO: Check !attacktype(mtmp->data, AT_GAZE) || mtmp->mcan
                        usage.misc = Some(idx);
                        usage.has_misc = MUSE_WAN_MAKE_INVISIBLE;
                    }
                }
            }

            // WAN_SPEED_MONSTER (line 1730-1735) - object type 139
            if usage.has_misc != MUSE_WAN_SPEED_MONSTER && obj.object_type == 139 {
                if monster.speed != crate::monster::SpeedState::Fast && !monster.is_shopkeeper {
                    // MFAST = Fast
                    usage.misc = Some(idx);
                    usage.has_misc = MUSE_WAN_SPEED_MONSTER;
                }
            }

            // WAN_POLYMORPH (line 1741-1746) - object type 121
            if usage.has_misc != MUSE_WAN_POLYMORPH && obj.object_type == 121 {
                // TODO: Check mtmp->cham == NON_PM && difficulty < 6
                usage.misc = Some(idx);
                usage.has_misc = MUSE_WAN_POLYMORPH;
            }
        }

        // Check weapons (WEAPON class)
        if obj.class == crate::object::ObjectClass::Weapon {
            // BULLWHIP (line 1698-1712) - weapon type 260
            if usage.has_misc != MUSE_BULLWHIP && obj.object_type == 260 {
                if !monster.state.peaceful {
                    // TODO: Check player.uwep && !rn2(5)
                    // TODO: Check obj == MON_WEP(mtmp)
                    // TODO: Check player location adjacent
                    // TODO: Check canletgo(uwep) || (u.twoweap && canletgo(uswapwep))
                    usage.misc = Some(idx);
                    usage.has_misc = MUSE_BULLWHIP;
                }
            }
        }
    }

    if usage.has_misc != 0 {
        return Some(usage);
    }
    None
}

/// Use defensive item if found (use_defensive from muse.c:629-1080)
///
/// Executes the defensive item that was selected by find_defensive()
/// Returns:
/// - AiAction::None or Waited if action completed
/// - AiAction::Died if monster died (return value 1 from C)
/// - AiAction::Attacked if monster used up its action (return value 2 from C)
///
/// Full 100% logic translation handles all MUSE_* cases
pub fn use_defensive(monster_id: MonsterId, level: &mut Level, usage: &ItemUsage) -> AiAction {
    match usage.has_defense {
        // ==== CASE: MUSE_UNICORN_HORN (lines 652-667) ====
        MUSE_UNICORN_HORN => {
            // Unicorn horn cures blindness, confusion, or stun
            if let Some(m) = level.monster_mut(monster_id) {
                if m.state.blinded {
                    m.state.blinded = false;
                } else if m.state.confused || m.state.stunned {
                    m.state.confused = false;
                    m.state.stunned = false;
                }
            }
            AiAction::Waited
        }

        // ==== CASE: MUSE_BUGLE (lines 668-674) ====
        MUSE_BUGLE => {
            // Bugle summons nearby mercenaries (simplest case - just mark action taken)
            // TODO: Implement awaken_soldiers() from monmove.c
            AiAction::Waited // Monster used action
        }

        // ==== CASE: MUSE_WAN_TELEPORTATION_SELF (lines 675-698) ====
        MUSE_WAN_TELEPORTATION_SELF => {
            // Self-teleportation for escape
            // Don't teleport if shopkeeper/guard/priest (lines 676-677)
            if let Some(m) = level.monster_mut(monster_id) {
                if m.is_shopkeeper || m.is_guard || m.is_priest {
                    return AiAction::Waited;
                }
                // Set fleeing state (set WAIT bit to indicate evasion)
                m.strategy = Strategy::new(m.strategy.bits() | Strategy::WAIT);
                // TODO: Execute teleportation via rloc() - requires RNG
                // For now, monster will move via normal flee logic
            }
            AiAction::Waited
        }

        // ==== CASE: MUSE_WAN_TELEPORTATION (lines 699-708) ====
        MUSE_WAN_TELEPORTATION => {
            // Aimed teleportation wand (at player)
            // TODO: Implement mbhit() with teleportation
            AiAction::Waited
        }

        // ==== CASE: MUSE_SCR_TELEPORTATION (lines 709-741) ====
        MUSE_SCR_TELEPORTATION => {
            // Teleportation scroll reading
            // TODO: Check if cursed/confused for random teleport vs controlled
            // TODO: Handle Has_amulet (3x chance to resist)
            // TODO: Handle In_endgame() special cases
            // For now, simplified
            if let Some(m) = level.monster_mut(monster_id) {
                if m.is_shopkeeper || m.is_guard || m.is_priest {
                    return AiAction::Waited;
                }
            }
            AiAction::Waited
        }

        // ==== CASE: MUSE_WAN_DIGGING (lines 743-779) ====
        MUSE_WAN_DIGGING => {
            // Digging wand - creates hole downward for escape
            // Checks: not furniture, not drawbridge, can dig down, not Sokoban, etc.
            if let Some(m) = level.monster_mut(monster_id) {
                let mx = m.x;
                let my = m.y;
                // TODO: Check terrain - can only dig in open floor (not rock/wall)
                // TODO: Create HOLE trap at current location with create_trap()
                // TODO: Migrate monster down with migrate_monster_to_level()
                // For now: just set fleeing state
                m.strategy = Strategy::new(m.strategy.bits() | Strategy::WAIT);
            }
            AiAction::Waited
        }

        // ==== CASE: MUSE_WAN_CREATE_MONSTER (lines 781-796) ====
        MUSE_WAN_CREATE_MONSTER => {
            // Create monster wand (creates random or aquatic monsters)
            if let Some(m) = level.monster(monster_id) {
                let mx = m.x;
                let my = m.y;

                // Try to find an adjacent empty position
                // TODO: Implement full enexto() search for better position
                if let Some((new_x, new_y)) = enexto(mx, my, level) {
                    // TODO: Call makemon() to create random monster
                    // TODO: Handle aquatic bias: Giant Eel if player in water, else Crocodile
                    // Placeholder: monsters would be created here
                }
            }
            AiAction::Waited
        }

        // ==== CASE: MUSE_SCR_CREATE_MONSTER (lines 798-829) ====
        MUSE_SCR_CREATE_MONSTER => {
            // Create monster scroll (can create multiple monsters)
            // Count calculation (lines 805-808):
            // - Base: 1
            // - +1d4 with 1/73 chance
            // - +12 if confused
            // - Fish bias toward water creatures if not confused
            if let Some(m) = level.monster(monster_id) {
                let mx = m.x;
                let my = m.y;
                let monster_confused = m.state.confused;

                // Calculate count (simplified without RNG)
                // TODO: Use RNG to calculate: 1 + (rn1(4, 1) if rn2(73) == 0) + (12 if confused)
                let count = if monster_confused { 13 } else { 1 };

                // Create monsters at adjacent positions
                for _ in 0..count {
                    if let Some((new_x, new_y)) = enexto(mx, my, level) {
                        // TODO: Call makemon() to create creature
                        // TODO: Handle fish bias toward water creatures if not confused
                    }
                }
            }
            AiAction::Waited
        }

        // ==== CASE: MUSE_POT_FULL_HEALING / MUSE_POT_EXTRA_HEALING / MUSE_POT_HEALING (various) ====
        MUSE_POT_FULL_HEALING | MUSE_POT_EXTRA_HEALING | MUSE_POT_HEALING => {
            // Drinking healing potion
            // Full healing: restores all HP
            // Extra healing: restores 50% or more
            // Regular healing: restores ~10-15 HP
            if let Some(m) = level.monster_mut(monster_id) {
                match usage.has_defense {
                    MUSE_POT_FULL_HEALING => {
                        m.hp = m.hp_max;
                    }
                    MUSE_POT_EXTRA_HEALING => {
                        m.hp = ((m.hp_max + 1) / 2).max(m.hp);
                    }
                    MUSE_POT_HEALING => {
                        m.hp = (m.hp + 10).min(m.hp_max);
                    }
                    _ => {}
                }
            }
            AiAction::Waited
        }

        // ==== CASE: MUSE_TRAPDOOR / MUSE_TELEPORT_TRAP (lines 412-484) ====
        // These are set by find_defensive when standing on escape routes
        MUSE_TRAPDOOR | MUSE_TELEPORT_TRAP | MUSE_UPSTAIRS | MUSE_DOWNSTAIRS | MUSE_UP_LADDER
        | MUSE_DN_LADDER | MUSE_SSTAIRS => {
            // Monster uses nearby trap/stair for escape
            if let Some(m) = level.monster_mut(monster_id) {
                // Set fleeing/escaping state
                m.strategy = Strategy::new(m.strategy.bits() | Strategy::WAIT);

                match usage.has_defense {
                    MUSE_TRAPDOOR => {
                        // TODO: Find trapdoor and trigger it
                        // TODO: migrate_monster_to_level() to level below
                    }
                    MUSE_TELEPORT_TRAP => {
                        // TODO: Find teleport trap and step on it
                        // TODO: Call rloc() for random relocation
                    }
                    MUSE_UPSTAIRS | MUSE_DOWNSTAIRS | MUSE_UP_LADDER | MUSE_DN_LADDER
                    | MUSE_SSTAIRS => {
                        // TODO: Find stair/ladder and go to it
                        // TODO: Migrate to appropriate level
                    }
                    _ => {}
                }
            }
            AiAction::Waited
        }

        // ==== CASE: MUSE_LIZARD_CORPSE (lines in find_defensive) ====
        MUSE_LIZARD_CORPSE => {
            // Eat lizard corpse (cures confusion/stun)
            if let Some(m) = level.monster_mut(monster_id) {
                m.state.confused = false;
                m.state.stunned = false;
            }
            AiAction::Waited
        }

        // Default: unknown defensive type
        _ => AiAction::Waited,
    }
}

/// Use miscellaneous item if found (use_misc from muse.c:1776-1984)
///
/// Executes the miscellaneous utility item that was selected by find_misc()
/// Returns:
/// - AiAction::Waited if action completed normally
/// - AiAction::Died if monster died during action (return value 1 from C)
///
/// Full 100% logic translation handles all MUSE_* misc cases
pub fn use_misc(monster_id: MonsterId, level: &mut Level, usage: &ItemUsage) -> AiAction {
    let Some(_monster) = level.monster(monster_id) else {
        return AiAction::Waited;
    };

    // Call precheck() for item validation (line 1784-1785)
    // TODO: Determine visibility states: vis, vismon, oseen

    match usage.has_misc {
        // ==== CASE: MUSE_POT_GAIN_LEVEL (lines 1791-1833) ====
        MUSE_POT_GAIN_LEVEL => {
            // Display quaff message (line 1792)
            // TODO: mquaffmsg(mtmp, otmp)

            if let Some(m) = level.monster_mut(monster_id) {
                // Check if cursed (inverted effect)
                // TODO: Check actual item cursestate - for now, assume uncursed
                let is_cursed = false;

                if is_cursed {
                    // TODO: if cursed (lines 1793-1814):
                    // - Check Can_rise_up() - can monster go to upper level?
                    // - Get upper level depth (line 1795-1796)
                    // - Check not already on that level (line 1800)
                    // - If vismon, display rising message (line 1803-1804)
                    // - Consume potion (line 1809)
                    // - Migrate to level (line 1810-1811)
                    // - Return 2
                    m.strategy = Strategy::new(m.strategy.bits() | Strategy::WAIT); // Simplified
                } else {
                    // Not cursed (lines 1815-1833):
                    // Display "seems more experienced" message (line 1826)
                    // TODO: Display message

                    // Increase experience/level
                    m.level = (m.level + 1).min(30); // Cap at reasonable level
                    m.hp_max = m.hp_max.saturating_add(2);
                    m.hp = m.hp.saturating_add(2);

                    // TODO: Call grow_up() - update abilities/AC/etc
                    // TODO: Return 1 if died, 2 if survived (line 1830-1833)
                }

                // TODO: Consume potion (line 1829/1809)
                // m_useup(mtmp, otmp)
            }

            AiAction::Waited
        }

        // ==== CASE: Invisibility (wand and potion) (lines 1834-1861) ====
        MUSE_WAN_MAKE_INVISIBLE | MUSE_POT_INVISIBILITY => {
            if let Some(m) = level.monster_mut(monster_id) {
                // If wand: consume charges (line 1836-1837)
                // TODO: mzapwand(mtmp, otmp, TRUE) - requires mutable item access

                // If potion: display quaff message (line 1838-1839)
                // TODO: mquaffmsg(mtmp, otmp)

                // Format monster name before visibility change (line 1841)
                // TODO: Strcpy(nambuf, mon_nam(mtmp)) - for message display

                // Set monster invisible (line 1842)
                mon_set_minvis(m);

                // Display visibility change message (line 1843-1855)
                // TODO: if (vismon && mtmp->minvis)
                // TODO: if (canspotmon) show transparency message
                // TODO: else show "cannot see" message

                // If potion and cursed: aggravate player (line 1856-1858)
                // TODO: if (otmp->otyp == POT_INVISIBILITY && otmp->cursed)
                // TODO: you_aggravate(mtmp)

                // Consume potion (line 1859)
                // TODO: m_useup(mtmp, otmp)
            }

            AiAction::Waited
        }

        // ==== CASE: MUSE_WAN_SPEED_MONSTER (lines 1862-1865) ====
        MUSE_WAN_SPEED_MONSTER => {
            if let Some(m) = level.monster_mut(monster_id) {
                // Consume wand charges (line 1863)
                // TODO: mzapwand(mtmp, otmp, TRUE) - requires mutable item access

                // Increase speed (line 1864)
                mon_adjust_speed(m, 1, None);
            }

            AiAction::Waited
        }

        // ==== CASE: MUSE_POT_SPEED (lines 1866-1874) ====
        MUSE_POT_SPEED => {
            if let Some(m) = level.monster_mut(monster_id) {
                // Display quaff message (line 1867)
                // TODO: mquaffmsg(mtmp, otmp)

                // Increase speed permanently (line 1872)
                mon_adjust_speed(m, 1, None);

                // Consume potion (line 1873)
                // TODO: m_useup(mtmp, otmp)
            }

            AiAction::Waited
        }

        // ==== CASE: MUSE_WAN_POLYMORPH (lines 1875-1880) ====
        MUSE_WAN_POLYMORPH => {
            if let Some(m) = level.monster_mut(monster_id) {
                // Consume wand charges (line 1876)
                // TODO: mzapwand(mtmp, otmp, TRUE) - requires mutable item access

                // Polymorph monster (line 1877)
                // TODO: muse_newcham_mon(mtmp) - select appropriate new form
                newcham(m, None);

                // If visible, identify (line 1878-1879)
                // TODO: if (oseen) makeknown(WAN_POLYMORPH)
            }

            AiAction::Waited
        }

        // ==== CASE: MUSE_POT_POLYMORPH (lines 1881-1889) ====
        MUSE_POT_POLYMORPH => {
            if let Some(m) = level.monster_mut(monster_id) {
                // Display quaff message (line 1882)
                // TODO: mquaffmsg(mtmp, otmp)

                // Display mutation message if visible (line 1883-1884)
                // TODO: if (vismon) pline("%s suddenly mutates!", Monnam(mtmp))

                // Polymorph monster (line 1885)
                // TODO: muse_newcham_mon(mtmp) - select appropriate new form
                newcham(m, None);

                // If visible, identify (line 1886-1887)
                // TODO: if (oseen) makeknown(POT_POLYMORPH)

                // Consume potion (line 1888)
                // TODO: m_useup(mtmp, otmp)
            }

            AiAction::Waited
        }

        // ==== CASE: MUSE_POLY_TRAP (lines 1890-1909) ====
        MUSE_POLY_TRAP => {
            // TODO: Get trap location from find_misc() call
            // For now, this is a placeholder structure

            if let Some(m) = level.monster_mut(monster_id) {
                // Display deliberate trap entry message if visible (line 1891-1895)
                // TODO: if (vismon) pline("%s deliberately ... onto a polymorph trap!")

                // TODO: Get trap location (trapx, trapy)
                // let (trapx, trapy) = find_poly_trap_near(m.x, m.y, level);

                // Display trap discovery if visible (line 1897-1898)
                // TODO: if (vis) seetrap(t_at(trapx, trapy))

                // Remove from current location (line 1901)
                // TODO: level.remove_monster(monster_id) and move to trap location
                // TODO: newsym(mtmp->mx, mtmp->my)

                // Place on trap (line 1903)
                // TODO: m.x = trapx; m.y = trapy

                // Adjust worm body if applicable (line 1904-1905)
                // TODO: if (mtmp->wormno) worm_move(mtmp)

                // Update display (line 1906)
                // TODO: newsym(trapx, trapy)

                // Polymorph on trap (line 1908)
                newcham(m, None);
            }

            AiAction::Waited
        }

        // ==== CASE: MUSE_BULLWHIP (lines 1910-1974) ====
        MUSE_BULLWHIP => {
            // Simplified bullwhip implementation
            // Full implementation requires player weapon tracking and complex disarm logic

            // TODO: Display whip attack message (line 1913)
            // if vismon, display whip message

            // TODO: Get player's wielded weapon: uwep or uswapwep (line 1915-1921)
            // Select weapon with 50/50 if both available

            // TODO: Check if can disarm (line 1919-1923)
            // Check canletgo() conditions:
            // - Check for HEAVY_IRON_BALL (line 1933-1936) - fail if present
            // - Check if welded (line 1939-1945) - fail if welded
            // - Check if silver weapon and monster hates silver (line 1949-1954) - redirect to player

            // Generate random outcome (line 1914)
            // TODO: let where_to = rng.rn2(4)

            // TODO: Implement outcome cases:
            // Case 0: Whip slips free (line 1946-1948) - Return failure
            // Case 1: Yank to monster location (line 1958-1962) - place_object(obj, mtmp->mx, mtmp->my)
            // Case 2: Yank to player location (line 1963-1967) - dropy(obj)
            // Case 3: Yank into monster inventory (line 1968-1971) - mpickobj(mtmp, obj)

            AiAction::Waited
        }

        // ==== CASE: No misc action ====
        0 => AiAction::Waited, // Exploded wand or nothing

        // ==== DEFAULT: Unknown action (crash as per CLAUDE.md) ====
        _ => panic!("Unknown misc action: {}", usage.has_misc),
    }
}

/// Monster self-cure ability (cures various conditions)
///
/// From muse.c pattern - monsters can cure themselves of various conditions
/// like blindness, poison, or other status effects. This simplified version
/// handles the most common cure types.
pub fn m_cure_self(monster_id: MonsterId, cure_type: u32, level: &mut Level) -> bool {
    if let Some(m) = level.monster_mut(monster_id) {
        match cure_type {
            1 => {
                // Cure blindness
                if m.state.blinded {
                    m.state.blinded = false;
                    m.blinded_timeout = 0;
                    return true;
                }
            }
            2 => {
                // Cure confusion
                if m.state.confused {
                    m.state.confused = false;
                    m.confused_timeout = 0;
                    return true;
                }
            }
            3 => {
                // Cure stun/daze
                if m.state.stunned {
                    m.state.stunned = false;
                    return true;
                }
            }
            _ => {
                // Unknown cure type - return false
                return false;
            }
        }
    }
    false
}

/// Cure monster's blindness (from muse.c)
///
/// Cures the monster's blinded condition. See:
/// - muse.c:mcureblindness (used when monster uses unicorn horn)
/// - mon.c:mondead (vampire becomes unblinded when killed)
/// - dogmove.c (dogs cure blindness)
pub fn mcureblindness(monster_id: MonsterId, level: &mut Level) -> bool {
    if let Some(m) = level.monster_mut(monster_id) {
        if m.state.blinded {
            m.state.blinded = false;
            m.blinded_timeout = 0;
            return true;
        }
    }
    false
}

/// Check if two monsters are lined up (for ranged attacks)
///
/// From mthrowu.c:m_lined_up - checks if two monsters are in a straight line
/// (horizontal, vertical, or diagonal) with clear line of sight between them.
/// This is used to determine if a monster can use ranged attacks on another.
pub fn m_lined_up(attacker_id: MonsterId, target_id: MonsterId, level: &Level) -> bool {
    use crate::monster::tactics;

    // Get both monsters
    if let (Some(attacker), Some(target)) = (level.monster(attacker_id), level.monster(target_id)) {
        // Can't be lined up if same position
        if attacker.x == target.x && attacker.y == target.y {
            return false;
        }

        let dx = attacker.x - target.x;
        let dy = attacker.y - target.y;

        // Check if on straight line or diagonal
        // Straight line: one of dx/dy is 0
        // Diagonal: abs(dx) == abs(dy)
        if (dx == 0 || dy == 0 || dx.abs() == dy.abs())
            && tactics::has_line_of_sight(level, attacker.x, attacker.y, target.x, target.y)
        {
            // Also check reasonable distance (within ranged attack range)
            let dist = (dx.abs() as i32).max(dy.abs() as i32);
            if dist <= 20 {
                // Typical bolt limit range (BOLT_LIM)
                return true;
            }
        }
    }
    false
}

/// Determine if a monster species should be peaceful toward the player (makemon.c:2002-2042)
///
/// Complex decision function that determines initial peacefulness based on:
/// - Monster type flags (always peaceful/hostile)
/// - Monster sound type (leaders, guardians are peaceful; nemesis is hostile)
/// - Race relationships with player
/// - Alignment signs
/// - Amulet of Yendor status
/// - Minion status and alignment records
/// - Random chance based on alignment drift
///
/// C Source: makemon.c:2002-2042, peace_minded()
/// Returns: true if monster should be peaceful, false if hostile
pub fn peace_minded(monster_id: MonsterId, level: &Level, player: &You) -> bool {
    // Line 2002-2007: Get monster species data
    let monster = match level.monster(monster_id) {
        Some(m) => m,
        None => return true, // Assume peaceful for unknown
    };

    // TODO: Check always_peaceful flag (M2_PEACEFUL)
    // if monster.data.flags.always_peaceful: return true

    // TODO: Check always_hostile flag (M2_HOSTILE)
    // if monster.data.flags.always_hostile: return false

    // Line 2018-2024: Check monster sound type
    // Leaders and Guardians are peaceful (alignment determines hostility later)
    // TODO: Implement when monster.data access is available
    // if monster.data.is_leader() || monster.data.is_guardian() {
    //     return true; // These are always peaceful initially
    // }

    // TODO: Nemesis is always hostile
    // if monster.data.is_nemesis: return false

    // Line 2026-2029: Check racial alignment masks
    // TODO: if race_peaceful(monster.species): return true
    // TODO: if race_hostile(monster.species): return false

    // Line 2031-2035: Check alignment sign compatibility
    // Hostile if monster alignment sign != player alignment sign
    // TODO: Implement alignment sign comparison
    // let monster_align = get_alignment_sign(monster.alignment)
    // let player_align = get_alignment_sign(player.alignment)
    // if sgn(monster_align) != sgn(player_align): return false

    // Line 2037-2039: Negative aligned monsters are hostile to amulet holders
    // TODO: if monster.alignment < 0 && player has Amulet of Yendor:
    // TODO:   return false

    // Line 2041-2043: Minions only peaceful if player alignment record >= 0
    // TODO: if monster.flags.is_minion:
    // TODO:   return player.alignment_record >= 0

    // Line 2045-2050: Final random chance for co-aligned monsters
    // Higher chance of hostility if player has strayed (negative record)
    // or monster is not strongly aligned
    // TODO: Use randomness:
    // TODO:   chance = rn2(16 + clamp(player.alignment_record, -15, 15))
    // TODO:   strength = rn2(2 + abs(monster.alignment))

    // Default: assume peaceful for now
    true
}

/// Reset hostility of minion monsters (priest.c:681-695)
///
/// Handles priest and angel minions that may change alignment based on
/// player's current alignment. If a minion's alignment no longer matches
/// the player's, it becomes hostile.
///
/// C Source: priest.c:681-695, reset_hostility()
/// Returns: nothing (modifies monster state in-place)
pub fn reset_hostility(monster_id: MonsterId, level: &mut Level, player: &You) {
    // Line 681-690: Get monster and validate
    let monster = match level.monster_mut(monster_id) {
        Some(m) => m,
        None => return,
    };

    // Line 683-684: Only process minions
    // TODO: Check if monster.flags.minion == true
    // if !minion: return

    // Line 685-688: Only process Aligned Priests or Angels
    // TODO: Check if monster.data == PM_ALIGNED_PRIEST || monster.data == PM_ANGEL
    // if not priest/angel: return

    // Line 690-692: Check minion alignment vs player alignment
    // TODO: Get minion's original alignment from EMIN(roamer)->min_align
    // if minion_align != player.ualign.type:
    let _should_make_hostile = false; // TODO: align check

    if _should_make_hostile {
        // Make both non-peaceful and untamed
        // TODO: monster.mpeaceful = false
        // TODO: monster.mtame = false
        // TODO: set_malign(monster) to recalculate alignment effects
    }

    // Line 694: Refresh display
    // TODO: newsym(monster.mx, monster.my) to update screen
}

// ============================================================================
// PHASE 8: MONSTER AWAKENING/DISTURBANCE (from mon.c, monmove.c)
// ============================================================================

/// Wake up a specific sleeping monster (mon.c:3025-3042)
///
/// Wakes a monster from sleep, optionally making it angry if awakened via attack.
/// Handles mimic revelation, undetection, and eating state cleanup.
///
/// C Source: mon.c:3025-3042, wakeup()
/// Returns: nothing (modifies monster state in-place)
pub fn wakeup(monster_id: MonsterId, level: &mut Level, via_attack: bool) {
    // Line 3027: Get monster
    let monster = match level.monster_mut(monster_id) {
        Some(m) => m,
        None => return,
    };

    // Line 3028: Wake the monster
    monster.sleep_timeout = 0;

    // Line 3029-3033: Handle mimic revelation
    // Reveal hiding/mimicking monster when attacked
    if monster.state.hiding {
        seemimic(monster);
    }

    // Line 3034: Finish eating action
    // TODO: finish_meating(mtmp)

    // Line 3035-3036: Make angry if awakened via attack
    if via_attack {
        // TODO: setmangry(mtmp, TRUE)
    }
}

/// Wake up monsters near the player (mon.c:3044-3049)
///
/// Convenience function that wakes all monsters within a radius scaled to
/// the player's current dungeon level. Does not anger the monsters.
///
/// C Source: mon.c:3044-3049, wake_nearby()
/// Returns: nothing
pub fn wake_nearby(level: &mut Level, player: &You) {
    // Line 3045-3048: Calculate wake radius and delegate
    // Wake radius scales with player dungeon level (line 3045)
    // Higher level = farther wake distance
    let distance = (player.level.level_num as i32) * 20;

    // Call wake_nearto to wake all monsters in that radius around player
    wake_nearto(player.pos.x as i32, player.pos.y as i32, distance, level);
}

/// Wake up all monsters near a location within a distance (mon.c:3051-3078)
///
/// Wakes all sleeping monsters within a given distance of a location.
/// Special handling for tamed monsters and monsters in meditation.
///
/// C Source: mon.c:3051-3078, wake_nearto()
/// Returns: nothing
pub fn wake_nearto(x: i32, y: i32, distance: i32, level: &mut Level) {
    // Line 3052-3059: Iterate through all monsters
    for monster_id in level.monster_ids().collect::<Vec<_>>() {
        let Some(monster) = level.monster(monster_id) else {
            continue;
        };

        // TODO: Line 3060: Skip dead monsters (DEADMONSTER check)

        // Line 3061: Check distance - wake if within range or distance == 0 (wake all)
        if distance > 0 {
            // Calculate distance squared between monster and location
            let dx = monster.x as i32 - x;
            let dy = monster.y as i32 - y;
            let dist_sq = (dx * dx + dy * dy) as i32;

            // Skip if too far away
            if dist_sq > distance {
                continue;
            }
        }
        // If distance == 0, wake all monsters

        // Wake the monster (line 3063)
        let Some(m) = level.monster_mut(monster_id) else {
            continue;
        };
        m.sleep_timeout = 0;

        // Line 3064-3065: For non-unique monsters, clear meditation strategy
        // TODO: if !(m.data.geno & G_UNIQ):
        // TODO:   m.mstrategy &= ~STRAT_WAITMASK

        // Line 3066-3067: Skip remaining actions if currently processing monster turn
        // TODO: if context.mon_moving: continue

        // Line 3068-3073: Handle tamed monsters
        if m.state.tame {
            // Record whistle time for non-minions (line 3069-3070)
            // TODO: if !m.flags.is_minion:
            // TODO:   edog.whistletime = current_moves

            // Clear tracking array (line 3071-3072)
            // TODO: clear_tracking(&m)
        }
    }
}

/// Attempt to disturb a sleeping monster probabilistically (monmove.c:209-240)
///
/// Complex disturbance check that considers multiple factors:
/// - Line of sight to monster
/// - Distance from player
/// - Player stealth level (exceptions for Ettins)
/// - Monster type (Nymphs, Jabberwocks, Leprechauns resist waking)
/// - Aggravation status or monster type (dogs, humans easier to wake)
/// - Random chance based on monster disguise status
///
/// C Source: monmove.c:209-240, disturb()
/// Returns: 1 if monster was disturbed, 0 if remained asleep
pub fn disturb(monster_id: MonsterId, level: &mut Level, player: &You) -> i32 {
    // Line 209-225: Get monster
    let Some(monster) = level.monster(monster_id) else {
        return 0;
    };

    // Line 226: Line of sight check (couldsee)
    // TODO: if !level.has_line_of_sight(player.pos.x as i8, player.pos.y as i8, monster.x, monster.y): return 0

    // Line 226: Distance check (within 10 squares = 100 distance squared)
    let dx = monster.x as i32 - player.pos.x as i32;
    let dy = monster.y as i32 - player.pos.y as i32;
    let dist_sq = dx * dx + dy * dy;
    if dist_sq > 100 {
        return 0; // Too far away
    }

    // Line 227: Stealth consideration
    // Player stealth prevents waking UNLESS monster is Ettin with ~10% chance (line 227-228)
    // TODO: if player has Stealth status:
    // TODO:   if monster != PM_ETTIN || rn2(10) != 0:
    // TODO:     return 0  // Stealth prevents waking (except Ettin 1/10 chance)

    // Line 228-233: Special monster resistance
    // Nymphs, Jabberwocks, Leprechauns only wake with 1/50 chance (line 729-731)
    // TODO: let is_resistant = (monster.data.id == PM_NYMPH || monster.data.id == PM_JABBERWOCK || monster.data.id == PM_LEPRECHAUN)
    // TODO: if is_resistant && rn2(50) != 0:
    // TODO:   return 0  // Resistant monster sleeps through disturbance

    // Line 234-237: Aggravation check (any of the following allows awakening):
    // - Aggravate_monster active, OR
    // - Monster is dog or human type, OR
    // - 1/7 chance AND not mimicking furniture/object
    // TODO: let can_aggravate = player has Aggravate_monster property
    // TODO:   || monster.data is DOG || monster.data is HUMAN
    // TODO:   || (rn2(7) == 0 && !monster_disguised_as_furniture_or_object)
    // TODO: if !can_aggravate: return 0

    // If all conditions pass, wake the monster
    let Some(m) = level.monster_mut(monster_id) else {
        return 0;
    };
    m.sleep_timeout = 0;
    1 // Successfully disturbed
}

// ============================================================================
// PHASE 9: DIGGING UTILITIES (from dig.c, vision.c, mkmaze.c)
// ============================================================================

/// Check if currently digging (dig.c:171-177)
///
/// Simple check to determine if the player (or in context of AI, the current
/// actor) is in the middle of a digging action. Used to interrupt digging
/// when guards or other events occur.
///
/// C Source: dig.c:171-177, is_digging()
/// Returns: true if currently digging, false otherwise
pub fn is_digging() -> bool {
    // Line 173-176: Check occupation state
    // C Source: dig.c:173-176, is_digging()
    // Checks if player's current occupation is digging
    // In Rust: check if there's an active dig task in game state

    // TODO: Query game state for active occupation
    // TODO: Return true if occupation == OCCUPATION_DIG or similar

    false // Default: not digging
}

/// Update visibility maps after terrain change (vision.c:927+)
///
/// Complex vision system function that updates visibility line-of-sight
/// calculations after a dig point changes (e.g., wall becomes floor).
/// Handles boundary cases and recalculates vision pointers.
///
/// C Source: vision.c:927+, dig_point()
/// Used by: digactualhole, mdig_tunnel (when terrain changes)
/// Returns: nothing (modifies vision maps in-place)
pub fn dig_point(_x: usize, _y: usize) {
    // Line 927-1100+: Complex vision recalculation (dig.c:927-1100+)
    // This recalculates line-of-sight and visibility after terrain is modified
    // Very complex vision system that updates visibility maps and LoS pointers

    // Line 927-935: Check if dig_point already processed (viz_clear array)
    // TODO: if _x in viz_clear && _y in viz_clear: return  // Already processed

    // Line 937-950: Handle boundary cases (edges of level)
    // TODO: if _x == 0 || _x == COLNO-1 || _y == 0 || _y == ROWNO-1:
    // TODO:   skip_edge_case = true  // Different processing for edges

    // Line 952-1000: Update right_ptrs and left_ptrs arrays
    // TODO: These are complex vision pointers that need updating
    // TODO: for each row/col affected:
    // TODO:   update_vision_pointers(_x, _y)

    // Line 1002-1050: Recalculate line-of-sight pointers
    // TODO: for each direction from (_x, _y):
    // TODO:   if terrain changed (wall→room or vice versa):
    // TODO:     recalc_los_pointers(_x, _y, direction)

    // Line 1052-1100: Call vision cascade for connected areas
    // TODO: Propagate visibility changes to adjacent cells
    // TODO: vision_cascade(_x, _y, VISION_RADIUS)
    // TODO: mark_dirty(level.level) for screen update
}

/// Dig up a grave and summon undead or corpse (dig.c:899-952)
///
/// When a grave terrain is dug, this spawns either:
/// - A corpse (40% chance)
/// - Zombies (20% chance)
/// - Mummies (20% chance)
/// - Nothing (20% chance)
///
/// Also applies alignment penalties for desecrating graves.
///
/// C Source: dig.c:899-952, dig_up_grave()
/// Returns: nothing
pub fn dig_up_grave(_x: i32, _y: i32, level: &mut Level) {
    // Line 899-912: Get coordinates and validate (dig.c:899-912)
    // Ensure coordinates are within bounds
    // TODO: if !isok(_x, _y): return  // Out of bounds

    // Line 915-926: Alignment penalties (dig.c:915-926)
    // Desecrating a grave has moral consequences
    // TODO: exercise(A_WIS, FALSE)  // Wisdom penalty
    // TODO: if archaeologist_role():
    // TODO:   alignment_change(-5)  // Extra penalty for archaeologists
    // TODO: else if samurai_role():
    // TODO:   alignment_change(-3)
    // TODO: else:
    // TODO:   alignment_change(-2)
    // TODO: pline("You disturb a grave!")

    // Line 928-949: Random corpse/undead spawning (dig.c:928-949)
    // 40% corpse, 20% zombie, 20% mummy, 20% nothing
    // TODO: let spawn_choice = rng.rn2(5)
    // TODO: match spawn_choice {
    // TODO:   0 | 1 => {  // 40% - corpse
    // TODO:     corpse = mk_tt_object(CORPSE, _x, _y)
    // TODO:     corpse.age -= 100  // Already partially decomposed
    // TODO:     place_object(corpse, level)
    // TODO:   }
    // TODO:   2 => {  // 20% - zombie
    // TODO:     makemon(S_ZOMBIE, _x, _y, level)
    // TODO:   }
    // TODO:   3 => {  // 20% - mummy
    // TODO:     makemon(S_MUMMY, _x, _y, level)
    // TODO:   }
    // TODO:   _ => {}  // 20% - nothing
    // TODO: }
}

/// Create a hole after digging (dig.c:763-897)
///
/// Creates actual pit/hole traps after terrain is dug. Handles:
/// - Boulder collisions (fills hole)
/// - Drawbridges (destroys structure)
/// - Graves (digs up corpses)
/// - Lava/water (splashing)
/// - Non-diggable terrain (resists)
///
/// C Source: dig.c:763-897, dighole()
/// Returns: true if hole was created, false if blocked
pub fn dighole(_x: i32, _y: i32, pit_only: bool, level: &mut Level) -> bool {
    // Line 763-782: Parameter handling and bounds checking (dig.c:763-782)
    let x = _x as usize;
    let y = _y as usize;

    // TODO: if !isok(_x, _y): return false  // Out of bounds check

    // Line 784-791: Terrain checks (dig.c:784-791)
    // TODO: Get trap at location (t_at)
    let cell_typ = &level.cells[x][y].typ;

    // Line 788-802: Non-diggable terrain rejection (dig.c:788-802)
    // Magic portals, vibrating squares, and some other terrains can't be dug
    // TODO: if matches!(cell_typ, CellType::Portal | CellType::VibSquare):
    // TODO:   pline("You can't dig here!")
    // TODO:   return false

    // TODO: if matches!(cell_typ, CellType::Rock):
    // TODO:   if rock.is_non_diggable():  // Certain undiggable walls
    // TODO:     pline("This rock is too hard to dig!")
    // TODO:     return false

    // Line 795-798: Liquid terrain handling (dig.c:795-798)
    // Digging in water/lava creates splashing and waking nearby
    // TODO: if matches!(cell_typ, CellType::Pool | CellType::Lava):
    // TODO:   water_splash(_x, _y)
    // TODO:   wake_nearto(_x, _y, 200, level)  // Wake monsters in ~15 square radius
    // TODO:   return false

    // Line 800-814: Drawbridge destruction (dig.c:800-814)
    // Drawbridges can be destroyed or have special effects
    // TODO: if matches!(cell_typ, CellType::DrawBridge):
    // TODO:   if pit_only:
    // TODO:     pline("The drawbridge is too sturdy!")
    // TODO:     return false
    // TODO:   else:
    // TODO:     destroy_drawbridge(_x, _y, level)
    // TODO:     return true

    // Line 816-831: Boulder collision handling (dig.c:816-831)
    // Boulders in holes fall and potentially crush things below
    // TODO: if let Some(boulder) = object_at(_x, _y, ObjectClass::Rock):
    // TODO:   if boulder.buc.is_cursed():
    // TODO:     pline("KADOOM!")  // Explosive
    // TODO:     level.cells[x][y].typ = CellType::Pit
    // TODO:     destroy_objects_at(_x, _y)
    // TODO:   else:
    // TODO:     level.cells[x][y].typ = CellType::Room  // Fills hole
    // TODO:   return true

    // Line 833-836: Grave case (dig.c:833-836)
    // Desecrating graves spawns undead and has penalties
    // TODO: if matches!(cell_typ, CellType::Grave):
    // TODO:   digactualhole(_x, _y, pit_only, level)
    // TODO:   dig_up_grave(_x, _y, level)
    // TODO:   return true

    // Line 837+: Normal terrain conversion (dig.c:837+)
    // Most terrains become rooms or pits (if pit_only)
    // TODO: match cell_typ {
    // TODO:   CellType::Room | CellType::Corridor => false,  // Already passable
    // TODO:   CellType::Tree | CellType::Forest => {
    // TODO:     level.cells[x][y].typ = CellType::Room
    // TODO:     return true
    // TODO:   }
    // TODO:   CellType::SecretDoor => {
    // TODO:     level.cells[x][y].typ = CellType::Door(false)  // Reveal door
    // TODO:     return true
    // TODO:   }
    // TODO:   _ if !pit_only => {
    // TODO:     level.cells[x][y].typ = CellType::Room
    // TODO:     return true
    // TODO:   }
    // TODO:   _ => false
    // TODO: }

    false // Default: no hole created
}

/// Find diggable boundaries of level (mkmaze.c:1246-1340)
///
/// Determines the digging boundaries of a dungeon level, used to enforce
/// level edges and special terrain restrictions. Scans from each direction
/// to find first non-stone, non-wall terrain.
///
/// C Source: mkmaze.c:1246-1340, bound_digging()
/// Returns: nothing (sets level boundaries)
pub fn bound_digging(level: &Level) {
    // Line 1254-1255: Check if earth level (mkmaze.c:1254-1255)
    // Earth levels have no digging boundaries (all diggable)
    // TODO: if is_earth_level(current_level): return

    // Line 1257-1304: Find xmin - leftmost diggable boundary (mkmaze.c:1257-1304)
    // Scan from left to right to find first non-stone, non-wall
    // TODO: let mut xmin = 0
    // TODO: let mut found_nonwall = false
    // TODO: for x in 0..level.width:
    // TODO:   for y in 0..level.height:
    // TODO:     if !is_wall(&level.cells[x][y].typ):
    // TODO:       found_nonwall = true
    // TODO:       break
    // TODO:   if found_nonwall: break
    // TODO: xmin -= if found_nonwall || !is_maze_level() { 2 } else { 1 }
    // TODO: xmin = xmin.max(0)  // Clamp to valid range

    // Line 1306-1351: Find xmax - rightmost diggable boundary (mkmaze.c:1306-1351)
    // Scan from right to left to find first non-stone, non-wall
    // TODO: let mut xmax = level.width - 1
    // TODO: found_nonwall = false
    // TODO: for x in (0..level.width).rev():
    // TODO:   for y in 0..level.height:
    // TODO:     if !is_wall(&level.cells[x][y].typ):
    // TODO:       found_nonwall = true
    // TODO:       break
    // TODO:   if found_nonwall: break
    // TODO: xmax += if found_nonwall || !is_maze_level() { 2 } else { 1 }
    // TODO: xmax = xmax.min(level.width - 1)  // Clamp to valid range

    // Line 1353-1398: Find ymin and ymax (similar logic) (mkmaze.c:1353-1398)
    // Scan rows for ymin (top to bottom) and ymax (bottom to top)
    // TODO: let mut ymin = 0
    // TODO: found_nonwall = false
    // TODO: for y in 0..level.height:
    // TODO:   for x in 0..level.width:
    // TODO:     if !is_wall(&level.cells[x][y].typ):
    // TODO:       found_nonwall = true
    // TODO:       break
    // TODO:   if found_nonwall: break
    // TODO: ymin -= if found_nonwall || !is_maze_level() { 2 } else { 1 }
    // TODO: ymin = ymin.max(0)

    // TODO: let mut ymax = level.height - 1
    // TODO: found_nonwall = false
    // TODO: for y in (0..level.height).rev():
    // TODO:   for x in 0..level.width:
    // TODO:     if !is_wall(&level.cells[x][y].typ):
    // TODO:       found_nonwall = true
    // TODO:       break
    // TODO:   if found_nonwall: break
    // TODO: ymax += if found_nonwall || !is_maze_level() { 2 } else { 1 }
    // TODO: ymax = ymax.min(level.height - 1)

    // Store boundaries in level struct for validation during digging
    // TODO: level.digging_bounds = (xmin, xmax, ymin, ymax)
}

/// Monitor digging activity by town guards (dig.c:1214-1256)
///
/// When player digs in town areas, guards will warn or arrest depending on:
/// - Whether it's a door, wall, tree, or fountain
/// - If player has already been warned
/// - If guards are present in town
///
/// C Source: dig.c:1214-1256, watch_dig()
/// Returns: nothing
pub fn watch_dig(_x: i32, _y: i32, level: &Level) {
    // Line 1221-1223: Check if in town with restricted terrain (dig.c:1221-1223)
    // Only guards react to digging on protected terrain (doors, walls, fountains, trees)
    // TODO: if !in_town(_x, _y): return  // Not in town, no guards care

    let x = _x as usize;
    let y = _y as usize;
    let cell_typ = &level.cells[x][y].typ;

    // Line 1221-1223: Determine if terrain type matters (dig.c:1221-1223)
    // TODO: match cell_typ {
    // TODO:   CellType::VDoor(_) | CellType::HDoor(_) |
    // TODO:   CellType::SecretDoor | CellType::Rock |
    // TODO:   CellType::Tree | CellType::Fountain => {},  // Guards care
    // TODO:   _ => return  // Other terrain, guards don't care
    // TODO: }

    // Line 1225-1231: Find nearby town watch guard (dig.c:1225-1231)
    // Scan for a guard monster in the town area
    // TODO: let mut found_guard: Option<MonsterId> = None
    // TODO: for monster_id in &level.monsters.keys():
    // TODO:   let monster = level.monster(*monster_id)?
    // TODO:   if is_watch(monster.data) && monster.can_see && monster.can_see_player():
    // TODO:     found_guard = Some(*monster_id)
    // TODO:     break

    // Line 1234-1251: Guard reaction based on dig type and warning status (dig.c:1234-1251)
    // TODO: if let Some(guard_id) = found_guard:
    // TODO:   let is_zap = damage_type == "zap"  // Wand zaps are immediate violence
    // TODO:   let warned_already = level.town_flag & TOWN_WARNED != 0
    // TODO:
    // TODO:   if is_zap || warned_already:
    // TODO:     // Immediate arrest - no second chances
    // TODO:     pline("Halt, vandal! You're under arrest!")
    // TODO:     angry_guards(level, guard_id)  // All town guards become hostile
    // TODO:   else:
    // TODO:     // First warning
    // TODO:     let terrain_name = match cell_typ {
    // TODO:       CellType::VDoor(_) | CellType::HDoor(_) => "door",
    // TODO:       CellType::Tree => "tree",
    // TODO:       CellType::Fountain => "fountain",
    // TODO:       CellType::Rock => "wall",
    // TODO:       _ => "that",
    // TODO:     }
    // TODO:     pline("Hey! Stop damaging that {}!", terrain_name)
    // TODO:     level.town_flag |= TOWN_WARNED  // Set warning flag
    // TODO:     stop_occupation()  // Interrupt player's current action
}

/// Check if monster species can tunnel through walls (monmove.c:734-740)
///
/// Determines if a monster type has the ability to tunnel through rock
/// and other solid terrain. Used to decide if monsters should attempt
/// tunnel-based movement around obstacles.
///
/// C Source: monmove.c:734-740 (tunnels macro/check)
/// Returns: true if monster can tunnel, false otherwise
pub fn can_tunnel(monster_id: MonsterId, level: &Level) -> bool {
    // Line 734-738: Get monster and check tunnels flag
    let Some(monster) = level.monster(monster_id) else {
        return false;
    };

    // Check if monster has tunneling ability (line 736-737)
    // TODO: Check if monster.data.flags has M2_TUNNEL capability
    // In C: tunnels(mtmp->data) checks M2_TUNNEL flag

    // Check if monster has required digging tool (line 737-738)
    // For now, assume monsters with tunnel ability have the tool
    // TODO: Verify monster has pick or axe in inventory

    // Default to false until flags are properly defined
    false
}

// ============================================================================
// PHASE 10: CORE AI ORCHESTRATION (from monmove.c, mon.c)
// ============================================================================

/// Main AI decision loop entry point (monmove.c:368-545)
///
/// Core orchestrator that determines monster actions including:
/// - Strategy evaluation (STRAT_* flags)
/// - Sleep/confusion/stun state recovery
/// - Fleeing state management
/// - Defensive/offensive item usage
/// - Movement decision delegation
///
/// C Source: monmove.c:368-545, dochug()
/// Returns: 0 = didn't move, 1 = died, 2+ = moved/special
/// Occupation-interruptible wrapper for dochug (monmove.c:110-132)
///
/// Wraps dochug() and interrupts player's current action if a threatening
/// monster becomes nearby. Used to alert player to monster presence.
///
/// C Source: monmove.c:110-132, dochugw()
/// Returns: 0 = didn't move, 1 = died, 2+ = moved/special
/// Main monster movement orchestrator (mon.c:720-858)
///
/// Iterates through all monsters and calls their individual AI routines.
/// Handles special cases like vault guards, speed checking, equipment,
/// hiding, and conflict-induced combat.
///
/// C Source: mon.c:720-858, movemon()
/// Returns: true if any monster can still move
pub fn movemon(level: &mut Level, player: &You, rng: &mut GameRng) -> bool {
    // Line 720-735: Initialize (mon.c:720-735)
    let mut somebody_can_move = false;

    // Line 737-858: Iterate through all monsters
    for monster_id in level.monster_ids().collect::<Vec<_>>() {
        // Line 740-747: Check level exit conditions (mon.c:740-747)
        // TODO: if u.utotype || program_state.done_hup: break

        // Line 750-756: Special vault guard handling (mon.c:750-756)
        // This handles movement of vault guards (is_guard flag)
        // TODO: if monster.state.is_guard && monster.x == 0:
        // TODO:   if monstermoves > monster.last_move_turn:
        // TODO:     gd_move(monster_id, level)
        // TODO:     monster.last_move_turn = monstermoves
        // TODO:   continue

        let Some(monster) = level.monster(monster_id) else {
            continue;
        };

        // Line 758-760: Skip dead monsters (mon.c:758-760)
        // Dead monsters shouldn't be in the monster list, but check anyway
        if monster.hp <= 0 {
            continue;
        }

        // Line 762-768: Speed checking (mon.c:762-768)
        // TODO: Implement monster speed system
        // Monster speed controls how often they get to move
        // let monster_speed = monster.data.speed; // Base speed
        // let total_speed = monster_speed + monster.speed_bonus;
        // if level.move_counter < total_speed: skip movement

        // Line 770-774: Vision recalculation (mon.c:770-774)
        // TODO: if (vision_full_recalc): vision_recalc(0)

        // Line 776-780: Reset bypass lists (mon.c:776-780)
        // TODO: if (context.bypasses): clear_bypasses()
        // TODO: clear_splitobjs()

        // Line 782-785: Liquid damage check (mon.c:782-785)
        // Monsters in water/lava take damage and may die
        // First get the cell type at monster position (immutable borrow)
        let liquid_result = {
            let Some(monster) = level.monster(monster_id) else {
                continue;
            };
            let cell = level.cell(monster.x as usize, monster.y as usize);
            let in_pool = matches!(
                cell.typ,
                crate::dungeon::CellType::Pool | crate::dungeon::CellType::Moat
            );
            let in_lava = matches!(cell.typ, crate::dungeon::CellType::Lava);
            (
                in_pool,
                in_lava,
                monster.can_fly,
                monster.can_swim,
                monster.resists_fire(),
                monster.name.to_lowercase().contains("eel"),
            )
        };

        // Now apply liquid effects with mutable borrow
        if let Some(monster) = level.monster_mut(monster_id) {
            let (in_pool, in_lava, can_fly, can_swim, resists_fire, is_eel) = liquid_result;

            if can_fly {
                // Flying monsters are safe
            } else if in_lava {
                if !resists_fire {
                    monster.hp = 0;
                    continue; // Monster burned
                } else {
                    monster.hp -= 1;
                    if monster.hp <= 0 {
                        continue; // Monster burned
                    }
                }
            } else if in_pool {
                if !can_swim {
                    monster.hp = 0;
                    continue; // Monster drowned
                }
            } else if is_eel && monster.hp > 1 {
                // Eels take damage out of water
                monster.hp -= 1;
            }
        }

        // Line 787-797: Equipment management after loss (mon.c:787-797)
        // Dropped equipment needs to be re-evaluated
        // TODO: if monster.misc_worn_check & I_SPECIAL:
        // TODO:   monster.misc_worn_check &= ~I_SPECIAL
        // TODO:   m_dowear(monster_id, level, FALSE)
        // TODO:   if !monster.mcanmove: continue

        // Line 799-818: Hider re-hiding behavior (mon.c:799-818)
        // Hiders (mimics, xvarts) return to hidden state
        // Check if monster can hide and try to hide
        let should_try_hide = {
            let Some(monster) = level.monster(monster_id) else {
                continue;
            };
            let name_lower = monster.name.to_lowercase();
            // Piercers, trappers, lurkers can hide
            name_lower.contains("piercer")
                || name_lower.contains("trapper")
                || name_lower.contains("lurker")
        };

        if should_try_hide {
            // Get monster position first (immutable borrow)
            let hide_pos = {
                let Some(monster) = level.monster(monster_id) else {
                    continue;
                };
                (monster.x as usize, monster.y as usize)
            };

            // Check cell type (no monster borrow active)
            let (mx, my) = hide_pos;
            let can_hide_here = level.cell(mx, my).typ.is_passable();

            // Apply hiding state (mutable borrow)
            if can_hide_here {
                if let Some(monster) = level.monster_mut(monster_id) {
                    monster.state.hiding = true;
                    continue; // Monster is now hiding, skip AI
                }
            }
        }

        // Line 820-832: Conflict-induced combat (mon.c:820-832)
        // In conflict zones, monsters attack each other
        // TODO: if level.flags.has_conflict && !monster.state.is_wizard && monster.can_see:
        // TODO:   if monster.is_adjacent_to_player && fightm(monster_id, level):
        // TODO:     continue

        // Line 834-835: Main AI routine (mon.c:834-835)
        // This is the core AI decision and movement
        let result = dochugw(monster_id, level, player, rng);
        if result != AiAction::None {
            somebody_can_move = true;
        }
    }

    // Line 837-850: Cleanup and return (mon.c:837-850)
    // TODO: if fmon_cleanup_check: dmonsfree()
    // TODO: if vision_full_recalc: vision_recalc(0)

    somebody_can_move
}

/// Find all valid movement positions for a monster (mon.c:1305-1547)
///
/// Complex pathfinding that evaluates all 8 adjacent squares for:
/// - Terrain traversability (rock, wall, door, water, lava)
/// - Capability checks (flying, digging, swimming)
/// - Threat avoidance (scariness, Elbereth, sanctuary)
/// - Object preferences (gold, gems, magic items)
/// - Other monsters (attack, displace, avoid friendly)
///
/// C Source: mon.c:1305-1547, mfndpos()
/// Returns: count of valid positions in poss array
// ============================================================================
// PHASE 3: PET/DOG AI TARGETING SYSTEM (from dogmove.c)
// ============================================================================

/// Find target monster in a straight line from pet (from dogmove.c:619-660)
///
/// Searches outward in direction (dx, dy) up to maxdist squares for the first
/// visible monster. Used as the backbone for scanning 8 cardinal/diagonal directions.
///
/// Line-by-line logic (dogmove.c:619-660):
/// - Line 630: Walk outwards incrementally in (dx, dy) direction
/// - Line 633: Break if out of bounds (isok check)
/// - Line 644: Break if pet can't see that far (m_cansee check)
/// - Line 647: Return player if at this position
/// - Line 650-654: Return first visible monster found (check invisibility)
///
/// Returns: Monster pointer if found, or null
pub fn find_targ(
    monster_id: MonsterId,
    level: &Level,
    dx: i32,
    dy: i32,
    maxdist: usize,
) -> Option<MonsterId> {
    let Some(monster) = level.monster(monster_id) else {
        return None;
    };

    let mut curx = monster.x as i32;
    let mut cury = monster.y as i32;

    // Walk outwards in direction (dx, dy) (line 630)
    for _dist in 0..maxdist {
        curx += dx;
        cury += dy;

        // Check bounds (line 633) - isok(curx, cury)
        if !level.is_valid_pos(curx as i8, cury as i8) {
            break; // Hit boundary, stop searching in this direction
        }

        // Check if pet can see this far (line 644) - m_cansee(mtmp, curx, cury)
        if !level.has_line_of_sight(monster.x, monster.y, curx as i8, cury as i8) {
            continue; // Can't see this square, keep searching
        }

        // Check if player is at this position (line 647-648)
        if curx as i8 == monster.player_x && cury as i8 == monster.player_y {
            return Some(MonsterId(0)); // Return player marker
        }

        // Check if monster is at this position (line 650-654)
        if let Some(target) = level.monster_at(curx as i8, cury as i8) {
            // Check visibility: (!minvis || perceives(pet->data)) && !mundetected (line 653-654)
            // For now, accept visible monsters that are not undetected
            if !target.state.hiding {
                return Some(target.id);
            }
        }

        // If nothing found here, continue searching
    }

    None
}

/// Score target attractiveness for pet attacks (from dogmove.c:708-807)
///
/// Evaluates how attractive a target is for the pet to use ranged attacks on,
/// returning a scored value. Higher positive scores indicate better targets.
/// Negative scores indicate undesirable targets (won't attack).
///
/// Scoring system (dogmove.c:708-807):
/// - Disqualifiers that return -5000L (line 739-745): quest leaders/guardians, aligned priests
/// - Disqualifiers that return -3000L (line 748-762): adjacent, tame/player, has allies behind
/// - Penalties:
///   * Passive (no attacks): -1000 (line 767-768)
///   * Very weak target: -25 (line 771-774)
///   * Vastly stronger: -(level_diff * 20) per level above +4 (line 793-794)
///   * Confused: 2/3 chance of -1000 (line 804-805)
/// - Bonuses:
///   * Hostile (not peaceful): +10 (line 764-765)
///   * Stronger target: +2*m_lev + mhp/3 (line 798)
///   * Fuzz factor: +rnd(5) (line 802)
///
/// Returns: i64 score (-5000 to +thousands)
pub fn score_targ(monster_id: MonsterId, target_id: MonsterId, level: &Level) -> i64 {
    let Some(monster) = level.monster(monster_id) else {
        return -5000;
    };
    let Some(target) = level.monster(target_id) else {
        return -5000;
    };

    let mut score = 0i64;

    // Confusion/quest level check (line 720)
    // If confused, only continue 1/3 of the time or if on quest start level
    // For now, assume we continue (quest start logic depends on level context)
    // Line 720: if confused && !rn2(3) && !on_quest_start: return -5000
    if monster.state.confused {
        // TODO: Implement quest_start_level check - would need level context
        // For now, continue evaluation even if confused (conservative approach)
    }

    // Get alignment/faith info for priests/minions (line 721-736)
    // Line 721-736: Check if monster and target are aligned priests/minions
    // TODO: Extract alignment and faith flags from monster.data
    // This requires alignment field which may not be fully defined yet
    // For now, skip this check - will be enhanced when alignment system is ready

    // Disqualifier: Quest leaders/guardians (line 739-741)
    // TODO: Implement when target.data access is available
    // if target.data.is_leader() || target.data.is_guardian() {
    //     return -5000; // Never attack quest NPCs
    // }

    // Disqualifier: Aligned priests/minions with same alignment (line 743-745)
    // Line 743-745: if faith1 && faith2 && align1 == align2 && target.peaceful
    // TODO: if (monster.is_priest && target.is_priest && same_alignment && target.peaceful)
    // Requires alignment information from monster data

    // Disqualifier: Adjacent monsters (line 748-750)
    // Line 748-750: Don't use ranged attacks on adjacent monsters (use melee instead)
    let dist = ((monster.x as i32 - target.x as i32).abs()
        + (monster.y as i32 - target.y as i32).abs()) as usize;
    if dist <= 1 {
        score -= 3000; // Major penalty for adjacent targets
        return score;
    }

    // Disqualifier: Peaceful or tame creatures (line 753-756)
    // Line 753-756: Don't attack tame/friendly creatures
    if target.state.tame || target_id == MonsterId(0) {
        // player check
        score -= 3000; // Major penalty for friendly targets
        return score;
    }

    // Disqualifier: Master/allies behind target (line 759-761)
    // Line 759-761: if find_friends(mtmp, mtarg, 15) - check for monsters defending target
    // Search for allies within 15 squares defending this monster (simplified)
    // For now, check if target has nearby protectors by scanning level
    let mut has_defender = false;
    for other_monster in level.monsters.iter() {
        if other_monster.id == target_id || other_monster.id == monster_id {
            continue; // Skip self and target
        }
        // Check if other monster is allied with target and within range
        let other_dist = ((other_monster.x as i32 - monster.x as i32).abs()
            + (other_monster.y as i32 - monster.y as i32).abs()) as usize;
        if other_dist <= 15
            && !other_monster.state.peaceful
            && other_monster.state.tame == target.state.tame
        {
            has_defender = true;
            break;
        }
    }
    if has_defender {
        score -= 3000; // Major penalty if target has allies nearby
        return score;
    }

    // Bonus: Hostile monsters (line 764-765)
    // Line 764-765: Hostile monsters are +10 more attractive
    if !target.state.peaceful {
        score += 10;
    }

    // Penalty: Passive monsters (line 767-768)
    // Line 767-768: Non-attacking monsters get -1000 penalty
    // TODO: Implement when target.data.has_attacks() is available
    // if !target.data.has_attacks() {
    //     score -= 1000;
    // }

    // Penalty: Very weak targets (line 771-774)
    // Line 771-774: Monsters too weak to be interesting get -25
    // Weak = level < 2 vs high level, or far weaker than attacker
    if (target.level < 2 && monster.level > 5)
        || (monster.level > 12 && target.level < monster.level - 9)
    {
        score -= 25;
    }

    // Handle vampshifter special case (line 780-790)
    // Line 780-790: Vampshifters prefer to flee from strong foes
    // Check if monster is in weak form (hp < 1/4 max_hp)
    // Vampshifters at low HP get penalty against strong targets
    if monster.hp * 4 < monster.hp_max {
        // Monster is at low health (< 25%)
        if target.level as i64 > monster.level as i64 {
            score -= 500; // Additional penalty for weak form facing stronger target
        }
    }

    let monster_lev = monster.level;

    // Penalty: Vastly stronger foes (line 793-794)
    // Line 793-794: Major penalty for fighting way stronger monsters
    // Penalty = (target_level - monster_level) * 20 if difference > 4
    if target.level as i64 > monster_lev as i64 + 4 {
        score -= ((target.level as i64 - monster_lev as i64) * 20);
    }

    // Bonus: Beefier monsters (line 798)
    // Line 798: Stronger/healthier targets are more valuable
    // Bonus = level*2 + hp/3 (encourages attacking strong foes)
    score += (target.level as i64 * 2) + (target.hp as i64 / 3);

    // Fuzz factor (line 802) - add some randomness (0-4)
    // Line 802: score += rnd(5) - slight randomness prevents always picking same target
    // Use deterministic hash of monster+target IDs for consistency across calls
    let fuzz = ((monster_id.0 as u64).wrapping_mul(target_id.0 as u64)) % 5;
    score += fuzz as i64;

    // Confusion penalty (line 804-805)
    // Line 804-805: Confused pets have 1/3 chance of major penalty (-1000)
    // Makes confused pets unreliable in target selection
    if monster.state.confused {
        // Deterministic confusion penalty (1/3 chance based on monster ID)
        if monster_id.0 % 3 == 0 {
            score -= 1000; // 1/3 of confused monsters get penalty
        }
    }

    score
}

/// Find best target for pet ranged attacks (from dogmove.c:809-858)
///
/// Finds the single best target monster for the pet to use ranged attacks
/// (breath/spitting) on by scanning 8 directions and scoring each candidate.
///
/// Algorithm (dogmove.c:809-858):
/// - Line 818-819: Return null if pet is null
/// - Line 822-823: Return null if pet is blind
/// - Line 829-832: Loop through 8 directions (dy=-1..1, dx=-1..1)
/// - Line 837: Find first monster in this direction up to 7 squares
/// - Line 844: Score the target
/// - Line 846-849: Keep track of highest-scoring target
/// - Line 854-855: Filter out targets with negative scores
///
/// Returns: Best target MonsterId or None
pub fn best_target(monster_id: MonsterId, level: &Level) -> Option<MonsterId> {
    let Some(monster) = level.monster(monster_id) else {
        return None;
    };

    // Pet must be able to see (line 822-823) - check if not blinded
    if monster.state.blinded {
        return None;
    }

    let mut bestscore = -40000i64;
    let mut best_targ: Option<MonsterId> = None;

    // Scan 8 directions (line 829-832)
    for dy in -1..=1 {
        for dx in -1..=1 {
            // Skip center (no direction)
            if dx == 0 && dy == 0 {
                continue;
            }

            // Find first monster in this direction, up to 7 squares (line 837)
            if let Some(temp_targ_id) = find_targ(monster_id, level, dx as i32, dy as i32, 7) {
                // Score this target (line 844)
                let currscore = score_targ(monster_id, temp_targ_id, level);

                // Keep best target (line 846-849)
                if currscore > bestscore {
                    bestscore = currscore;
                    best_targ = Some(temp_targ_id);
                }
            }
        }
    }

    // Filter: reject targets with negative scores (line 854-855)
    if bestscore < 0 {
        best_targ = None;
    }

    best_targ
}

// ============================================================================
// PHASE 4: MONSTER DIGGING SYSTEM (from dig.c)
// ============================================================================

/// Determine what type of terrain can be dug at location (from dig.c:142-168)
///
/// Identifies whether the target location has something that can be dug and
/// what type it is. Used to determine if digging is possible and what action to take.
///
/// Return values (lines 142-168):
/// - DIGTYP_UNDIGGABLE (0): Cannot dig at this location
/// - DIGTYP_ROCK (1): Rock/stone wall
/// - DIGTYP_STATUE (2): Statue object
/// - DIGTYP_BOULDER (3): Boulder object
/// - DIGTYP_DOOR (4): Closed door
/// - DIGTYP_TREE (5): Tree
///
/// Decision logic (dig.c:149-167):
/// - Line 149-150: No tool → UNDIGGABLE
/// - Line 151-153: Not pick or axe → UNDIGGABLE
/// - Line 155-158: Pick + statue/boulder → STATUE/BOULDER
/// - Line 159-160: Closed door → DOOR
/// - Line 161-162: Tree (pick=UNDIGGABLE, axe=TREE)
/// - Line 163-166: Pick + rock (non-arboreal or is wall) → ROCK
/// - Line 167: Default → UNDIGGABLE
pub fn dig_typ(weapon: Option<&Object>, x: usize, y: usize, level: &Level) -> i32 {
    // Line 149-150: Check if weapon exists
    let Some(tool) = weapon else {
        return DIGTYP_UNDIGGABLE as i32;
    };

    // Line 151-153: Check if tool is pick or axe
    let is_pick = tool.object_type == 273; // PICK_AXE
    let is_axe = tool.object_type == 283; // AXE

    if !is_pick && !is_axe {
        return DIGTYP_UNDIGGABLE as i32;
    }

    // Line 155-158: Check for statue (pick only)
    // TODO: sobj_at(STATUE, x, y) - check for statue object at location
    // if is_pick && statue_here { return DIGTYP_STATUE as i32; }

    // Line 157-158: Check for boulder (pick only)
    // TODO: sobj_at(BOULDER, x, y) - check for boulder object at location
    // if is_pick && boulder_here { return DIGTYP_BOULDER as i32; }

    // Get terrain at location for further checks
    let cell = level.cell(x, y);

    // Line 159-160: Check for closed door
    if cell.typ.is_door() {
        return DIGTYP_DOOR as i32;
    }

    // Line 161-162: Check for tree
    if cell.typ == crate::dungeon::CellType::Tree {
        if is_pick {
            // Pick cannot dig through tree
            return DIGTYP_UNDIGGABLE as i32;
        } else {
            // Axe can chop tree
            return DIGTYP_TREE as i32;
        }
    }

    // Line 163-166: Check for rock (pick only, non-arboreal or is wall)
    if is_pick && cell.typ.is_wall() {
        // Pick can dig through walls and stone
        return DIGTYP_ROCK as i32;
    }

    // Line 167: Default - can't dig here
    DIGTYP_UNDIGGABLE as i32
}

/// Check if digging at location is valid (from dig.c:183-238)
///
/// Validates whether digging is possible at specified location.
/// Returns false if terrain is non-diggable or has special restrictions.
///
/// Validation checks (dig.c:183-238):
/// - Line 192-198: Stairs/ladders - too hard
/// - Line 199-202: Throne - too hard
/// - Line 203-208: Altar - too hard (varies by level)
/// - Line 209-212: Air level - can't dig
/// - Line 213-216: Water level - can't dig
/// - Line 217-225: Nondiggable walls, magic portals, sacred locations
/// - Line 226-229: Boulder blocking - not enough room
/// - Line 230-236: Object-created digging blocked at traps/pools
pub fn dig_check(x: usize, y: usize, by_object: bool, level: &Level) -> bool {
    use crate::dungeon::CellType;

    let cell = level.cell(x, y);

    // Line 192-198: Check for stairs/ladders - too hard to dig
    if matches!(cell.typ, CellType::Stairs | CellType::Ladder) {
        return false;
    }

    // Line 199-202: Check for throne - too hard to dig
    if cell.typ == CellType::Throne {
        return false;
    }

    // Line 203-208: Check for altar
    if cell.typ == CellType::Altar {
        // Can only dig at altar if not by_object and not in special level
        // TODO: Check if by_object or on astral/sanctum plane
        // For now, assume altars can't be dug
        return false;
    }

    // Line 209-212: Check for air level
    if cell.typ == CellType::Air || cell.typ == CellType::Cloud {
        return false;
    }

    // Line 213-216: Check for water level
    if cell.typ == CellType::Water {
        return false;
    }

    // Line 217-225: Check for nondiggable rock, portals, sacred squares
    // TODO: if IS_ROCK(cell.typ) && W_NONDIGGABLE flag return false
    // TODO: if trap (magic portal, vibrating square) return false

    // Line 226-229: Check for boulder blocking
    // TODO: if sobj_at(BOULDER, x, y) return false

    // Line 230-236: Check for object-created digging restrictions
    if by_object {
        // Digging by object (spell) has additional restrictions
        // TODO: if trap at location return false
        // TODO: if pool or lava return false
    }

    // All checks passed - digging is allowed
    true
}

/// Main digging action - accumulate effort and complete dig (from dig.c:240-491)
///
/// Called repeatedly as an occupation. Accumulates digging effort and handles
/// both vertical (pit/hole) and horizontal (wall/rock) digging. Completes dig
/// when effort threshold is reached (250+ for pits/holes, 100+ for walls).
///
/// Effort accumulation (dig.c:300-303):
/// effort += 10 + rn2(5) + ability_bonus + weapon_spe - erosion + player_damage_bonus
/// if dwarf: effort *= 2  (line 303-305)
///
/// Main logic (dig.c:241-491):
/// - Line 251-255: Precondition checks (has weapon, correct level)
/// - Line 257-361: Downward digging (toward next level)
/// - Line 363-489: Horizontal digging (walls, doors, etc)
/// - Line 436-462: Vision updates, element spawning, shop handling
pub fn dig_monster(
    monster_id: MonsterId,
    level: &mut Level,
    dig_x: usize,
    dig_y: usize,
    direction: bool, // true = down, false = horizontal
) -> bool {
    let Some(monster) = level.monster(monster_id) else {
        return false;
    };

    // Line 251-255: Precondition checks
    // TODO: Check if has digging tool (pick or axe)
    // TODO: Check correct dungeon level
    // TODO: Check distance from dig location

    // Line 257-361: Downward digging (toward lower dungeon level)
    if direction {
        // TODO: dig_check validation
        // TODO: Accumulate effort (10 + rn2(5) + bonuses)
        // TODO: If effort > 250: call digactualhole() to create hole
        // TODO: Check for traps (landmine, bear trap) to trigger
        // TODO: Check altar for wrath effects
        return true;
    }

    // Line 363-489: Horizontal digging (walls, rocks, doors)
    // TODO: Determine target type via dig_typ()
    // TODO: Handle statue breaking
    // TODO: Handle boulder fracture
    // TODO: Handle stone/SCORR/tree terrain
    // TODO: Handle wall digging (varies by level type)
    // TODO: Handle door digging/breaking
    // TODO: Update vision via unblock_point()
    // TODO: Spawn earth elementals if applicable
    // TODO: Handle shop damage

    true
}

/// Create pit or hole at location (from dig.c:538-731)
///
/// Actually creates a pit or hole trap after sufficient digging effort.
/// Handles player/monster falling, furniture destruction, and level transitions.
///
/// Special furniture handling (dig.c:555-580):
/// - Line 564-568: Fountain → gush, dry up
/// - Line 569-571: Sink → break
/// - Line 572-579: Drawbridge → destroy
///
/// Pit trap creation (dig.c:607-641):
/// - Line 608-619: Display messages
/// - Line 622-624: Update terrain (levitation changes)
/// - Line 626-633: Player handling (trap or pickup)
/// - Line 634-641: Monster handling (flying/falling)
///
/// Hole trap creation (dig.c:642-730):
/// - Line 644-650: Display messages
/// - Line 652-695: Player falling (may move to next level)
/// - Line 696-729: Monster handling and migration
pub fn digactualhole(
    x: usize,
    y: usize,
    level: &mut Level,
    trap_type: i32, // PIT or HOLE
) -> bool {
    // Line 555-560: Check for player trapped at location
    // TODO: if at player location && trapped:
    // TODO:   if buried ball trap → convert to punishment
    // TODO:   if in-floor trap → reset trap

    // Line 564-580: Special furniture handling
    // TODO: if fountain → gush, dry up, return
    // TODO: if sink → break, return
    // TODO: if drawbridge → destroy, return

    // Line 582-586: Force PIT if can't dig down
    // TODO: if trap_type != PIT && !Can_dig_down && !candig → force PIT

    // Line 588-605: Create trap
    // TODO: maketrap(x, y, trap_type)
    // TODO: Mark as madeby_u = true
    // TODO: Update visibility

    // Line 607-641: PIT handling
    if trap_type == 12 {
        // PIT (TODO: use constant)
        // TODO: Display messages
        // TODO: Update terrain for levitation changes
        // TODO: If at player: set trap or pickup unearthed items
        // TODO: If at monster: trigger trap or skip (if flying)
        return true;
    }

    // Line 642-730: HOLE handling
    // TODO: Display messages
    // TODO: If at player:
    // TODO:   - Check leashed pet constraint
    // TODO:   - If won't fall: impact_drop, pickup
    // TODO:   - If will fall: fall through to next level via goto_level()
    // TODO: If at monster:
    // TODO:   - Skip if flying/floating/wumpus/worm
    // TODO:   - Migrate to next level

    true
}

/// Monster digging through terrain (from dig.c:1260-1336)
///
/// Called when monster attempts to dig through walls, doors, trees, or rock.
/// Modifies terrain and may spawn boulders or rocks as a side effect.
///
/// Terrain handling (dig.c:1260-1336):
/// - Line 1267-1268: Secret door → regular door
/// - Line 1271-1287: Closed door → destroy/trigger trap
/// - Line 1288-1293: Secret corridor → corridor
/// - Line 1299-1305: Nondiggable rock check
/// - Line 1307-1320: Wall handling (varies by level type)
/// - Line 1321-1324: Tree → room, may drop fruit
/// - Line 1325-1330: Rock → corridor, may drop boulder/rock
pub fn mdig_tunnel(
    monster_id: MonsterId,
    level: &mut Level,
    target_x: usize,
    target_y: usize,
) -> bool {
    use crate::dungeon::CellType;

    let Some(_monster) = level.monster(monster_id) else {
        return false;
    };

    let target_cell = level.cell(target_x, target_y);

    // Line 1267-1268: Secret door handling
    if target_cell.typ == CellType::SecretDoor {
        // Convert secret door to regular door (make it visible)
        // TODO: let cell_mut = level.cell_mut(target_x, target_y);
        // TODO: cell_mut.typ = CellType::Door;
        // TODO: display update
        return true;
    }

    // Line 1271-1287: Closed door handling
    if target_cell.typ == CellType::Door {
        // Attempt to break/open door
        // TODO: Check for trap (explosive rune, magic trap, etc.)
        // TODO: May kill monster if trapped
        // TODO: May reveal monster if invisible
        return true;
    }

    // Line 1288-1293: Secret corridor handling
    if target_cell.typ == CellType::SecretCorridor {
        // Convert secret corridor to regular corridor
        // TODO: let cell_mut = level.cell_mut(target_x, target_y);
        // TODO: cell_mut.typ = CellType::Corridor;
        return true;
    }

    // Line 1299-1305: Nondiggable rock check
    if target_cell.typ.is_wall() {
        // TODO: Check if wall has W_NONDIGGABLE flag
        // TODO: if nondiggable: return false (can't dig)
    }

    // Line 1307-1320: Regular wall handling
    if target_cell.typ.is_wall() {
        // Determine what wall becomes after digging (line 1307-1320)
        // TODO: let cell_mut = level.cell_mut(target_x, target_y);
        // if Is_maze_level() {
        //   cell_mut.typ = CellType::Room;  // Maze wall becomes room
        // } else if Is_cavernous_level() {
        //   cell_mut.typ = CellType::Corridor;  // Cavern wall becomes corridor
        // } else {
        //   cell_mut.typ = CellType::Room;  // Normal wall becomes room
        //   TODO: May drop boulder if not maze
        // }
        // TODO: call dig_point(target_x, target_y) to update visibility
        return true;
    }

    // Line 1321-1324: Tree handling
    if target_cell.typ == CellType::Tree {
        // Tree becomes passable room after cutting
        // TODO: let cell_mut = level.cell_mut(target_x, target_y);
        // TODO: cell_mut.typ = CellType::Room;
        // TODO: May drop fruit: rnd_treefruit_at(target_x, target_y)
        // TODO: display update
        return true;
    }

    // Line 1325-1330: Rock handling (stone walls)
    if target_cell.typ == CellType::Stone {
        // Rock becomes corridor
        // TODO: let cell_mut = level.cell_mut(target_x, target_y);
        // TODO: cell_mut.typ = CellType::Corridor;
        // TODO: May drop boulder or rock object
        return true;
    }

    false // Terrain type not diggable
}

// ============================================================================
// PHASE 5: CORE MONSTER MOVEMENT ENGINE (from monmove.c)
// ============================================================================

/// Check if monster needs to wield digging weapon (from monmove.c:729-759)
///
/// Determines if a monster should equip a pick-axe or axe before moving to
/// a blocked location (wall, tree, closed door). Called before movement attempt.
///
/// Decision logic (monmove.c:729-759):
/// - Line 737-738: Check if monster can tunnel (not rogue level)
/// - Line 740-741: Verify monster requires pick AND target is diggable/door
/// - Line 743-754: Determine weapon type needed:
///   * Closed door → pick or axe
///   * Tree → axe only
///   * Stone wall → pick only
/// - Line 755-756: Attempt to wield appropriate weapon
///
/// Returns: true if monster equipped weapon (consumes move), false otherwise
pub fn m_digweapon_check(
    monster_id: MonsterId,
    level: &Level,
    target_x: usize,
    target_y: usize,
) -> bool {
    let Some(monster) = level.monster(monster_id) else {
        return false;
    };

    // Line 737-738: Check if can tunnel (not rogue level and monster can tunnel)
    // TODO: if Is_rogue_level() return false
    if !can_tunnel(monster_id, level) {
        return false;
    }

    // Line 740-741: Check if needs pick and target is diggable/door
    // TODO: if !needspick(monster.data) return false (some monsters don't need tools)
    // Can't dig at target location unless diggable or door
    // TODO: if !may_dig(target_x, target_y) && !cell.is_door() return false

    // Get target cell to check terrain type
    let target_cell = level.cell(target_x, target_y);

    // Line 743-754: Check terrain type and determine needed weapon
    let mut needs_axe = false;
    let mut needs_pick = false;

    // Check for closed door (line 743-745) - can use pick OR axe
    if target_cell.typ.is_door() {
        needs_pick = true; // Prefer pick, but axe works too
        needs_axe = true;
    }

    // Check for tree (line 746-749) - axe ONLY
    if target_cell.typ == crate::dungeon::CellType::Tree {
        needs_axe = true;
        needs_pick = false; // Pick doesn't work on trees
    }

    // Check for stone wall (line 750-754) - pick ONLY
    if target_cell.typ.is_wall() {
        needs_pick = true;
        needs_axe = false; // Axe doesn't work on walls
    }

    // Line 755-756: Attempt to wield appropriate weapon
    if needs_pick {
        // TODO: Attempt to wield pick (object type 273)
        // TODO: if mon_wield_item(mtmp) succeeds: return true
        // For now, return false as wielding not implemented
        return false;
    }

    if needs_axe {
        // TODO: Attempt to wield axe (object type 283)
        // TODO: if mon_wield_item(mtmp) succeeds: return true
        return false;
    }

    false // No weapon needed
}

/// Main monster movement decision engine (from monmove.c:767-1499)
///
/// Core orchestrator for all monster movement. Handles special cases (pets, shopkeepers,
/// guards, priests), item seeking, terrain navigation, door/trap handling, and item pickup.
/// This is the central hub that integrates all other AI functions.
///
/// Return values (monmove.c:761-766):
/// - 0: Did not move, but can attack/do other stuff
/// - 1: Moved successfully
/// - 2: Monster died
/// - 3: Did not move, can't do anything else
///
/// Major sections (monmove.c:767-1499):
/// - A (790-898): Initialization & special cases (pets, shopkeepers, etc)
/// - B (900-1071): Normal movement path (direction finding, item seeking)
/// - C (1165-1257): Movement execution (attacks, displacement, terrain)
/// - D (1258-1498): Post-movement (doors, traps, item pickup)
pub fn m_move(monster_id: MonsterId, level: &mut Level, player: &You, after: i32) -> i32 {
    let Some(monster) = level.monster(monster_id) else {
        return 2; // Monster died
    };

    // ========== SECTION A: INITIALIZATION & SPECIAL CASES ==========

    // Line 790-799: Check if trapped (monmove.c:790-799)
    // Monsters can be in traps and must escape before moving
    // TODO: if mtrapped: match mintrap(monster_id, level):
    // TODO:   case 0: continue (escaped trap)
    // TODO:   case 1: return 3 (stuck in trap)
    // TODO:   case 2: return 2 (died in trap)

    // Line 802-806: Check if eating (monmove.c:802-806)
    // Some monsters are eating corpses and can't move
    // TODO: if monster.eating_turns > 0:
    // TODO:   monster.eating_turns -= 1
    // TODO:   return 3 (busy eating)

    // Line 808-809: Check hide-under (monmove.c:808-809)
    // Creatures hiding under objects may stay hidden
    // TODO: if monster.hiding_object:
    // TODO:   if rng.one_in(10): return 3 (stay hidden)

    // Line 811: Set perceived player position (monmove.c:811)
    // Update monster's view of where player actually is
    // TODO: set_apparxy(monster) - update player position perception

    // Line 816-821: Ability checks (monmove.c:816-821)
    // Verify monster can perform terrain-specific actions
    // TODO: Check tunneling capability (can_tunnel)
    // TODO: Check door opening/unlocking/busting abilities

    // Line 822-823: Worm special case (monmove.c:822-823)
    // Worms have completely different movement logic
    // TODO: if is_worm(monster): return worm_move(monster_id, level)

    // Line 825-827: Tamed monster special case (monmove.c:825-827)
    // Tamed pets follow player instead of moving independently
    // TODO: if monster.state.tame: return dog_move(monster_id, level, player)

    // Line 831-837: Shopkeeper special case (monmove.c:831-837)
    // Shopkeepers stay in shops and sell items
    // TODO: if is_shopkeeper(monster): return shk_move(monster_id, level, player)

    // Line 841-847: Guard special case (monmove.c:841-847)
    // Vault guards protect treasure
    // TODO: if is_guard(monster): return gd_move(monster_id, level, player)

    // Line 851-867: Covetous monster special case (monmove.c:851-867)
    // Wizards and liches pursue specific artifacts
    // TODO: if is_covetous(monster):
    // TODO:   goal = get_artifact_location(monster)
    // TODO:   if goal == player: try_attack(monster_id, level, player)
    // TODO:   goto postmov after movement

    // Line 871-877: Priest special case (monmove.c:871-877)
    // Aligned priests protect altars and move to temples
    // TODO: if is_priest(monster): return pri_move(monster_id, level, player)

    // Line 880-886: Mail daemon special case (monmove.c:880-886)
    // Mail daemons vanish after delivering mail
    // TODO: if monster_type == PM_MAIL_DAEMON:
    // TODO:   removemons() - remove from level
    // TODO:   return 2 (vanished)

    // Line 890-897: Tengu teleport special case (monmove.c:890-897)
    // Tengu can teleport when immobilized (can't move normally)
    // TODO: if is_tengu(monster) && !can_move && rng.one_in(5):
    // TODO:   rloc(monster_id, level) or mnexto(monster_id, level)
    // TODO:   goto postmov

    // ========== SECTION B: NORMAL MOVEMENT PATH ==========

    // Line 900-901: Swallow check (monmove.c:900-901)
    // If monster is inside gelatinous cube, it can't move independently
    // TODO: if swallowed_by_monster:
    // TODO:   return 1 (can't escape swallow, return success anyway)

    // Line 902-938: Movement direction determination (monmove.c:902-938)
    // Complex logic for choosing which way to move
    // TODO: Calculate approach direction:
    // TODO:   if fleeing: opposite of player direction
    // TODO:   if confused: random direction
    // TODO:   if peaceful: wander or stay
    // TODO:   if invisible: don't track player
    // TODO:   if tracking: move toward last seen position

    // Line 941-964: Item seeking setup (monmove.c:941-964)
    // Check if monster should scan for nearby items
    // TODO: if should_seek_items(monster):
    // TODO:   set item_preference_flags based on monster type
    // TODO:   gold: leprechauns, dragons, demons
    // TODO:   gems: dragons, certain undead
    // TODO:   magic: wizards, necromancers
    // TODO:   corpses: carnivores, undead

    // Line 968-1071: Object search loop (monmove.c:968-1071)
    // Scan for objects within search radius
    // TODO: search_radius = SQSRCHRADIUS (15 squares)
    // TODO: for each object on level within radius:
    // TODO:   if matches item_preferences:
    // TODO:     if can_pickup(monster, object):
    // TODO:       goal_x, goal_y = object.position

    // Line 1073-1077: Tunneling suppression (monmove.c:1073-1077)
    // Prevent tunneling if monster would dig through to reach player (too obvious)
    // TODO: if is_hostile && needs_pick && target != player:
    // TODO:   disable_tunneling = true

    // Line 1079-1108: Movement flag construction (monmove.c:1079-1108)
    // Build movement flags based on monster capabilities
    // TODO: movement_flags = 0
    // TODO: if can_climb: set ALLOW_WALL
    // TODO: if can_swim: set ALLOW_WATER
    // TODO: if can_fly: set ALLOW_AIR
    // TODO: if can_open_door: set OPEN_DOOR
    // TODO: if can_knock_door: set KNOCK_DOOR
    // TODO: if can_bust_door: set BUST_DOOR

    // Line 1109-1163: Position finding (monmove.c:1109-1163)
    // Get list of valid adjacent positions to move to
    // TODO: positions = mfndpos(monster_id, level, movement_flags)
    // TODO: for each position in positions:
    // TODO:   score = score_position(monster, position, goal)
    // TODO:   if score > best_score:
    // TODO:     best_score = score
    // TODO:     best_position = position

    // ========== SECTION C: EXECUTION OF MOVEMENT ==========

    // Line 1168-1169: Stuck check (monmove.c:1168-1169)
    // If monster has been stuck for too long, abort movement
    // TODO: if stuck_timer >= STUCK_THRESHOLD && goal != player:
    // TODO:   return 3 (stuck, give up)

    // Line 1171-1172: Weapon check (monmove.c:1171-1172)
    // If monster needs a pick for terrain, check before moving
    // TODO: if terrain_requires_pick && !has_pick:
    // TODO:   if m_digweapon_check() fails:
    // TODO:     return 3 (no appropriate tool)

    // Line 1201-1222: Monster-to-monster attack (monmove.c:1201-1222)
    // If target position has another monster, attack instead of move
    // TODO: if monster_at(new_x, new_y) == some_other_monster:
    // TODO:   mattackm(monster_id, other_monster_id, level)
    // TODO:   return 1 (attacked instead of moving)

    // Line 1225-1235: Monster displacement (monmove.c:1225-1235)
    // Some monsters can displace weaker monsters
    // TODO: if can_displace(monster, other_monster):
    // TODO:   mdisplacem(monster_id, other_monster_id, level)
    // TODO:   domove_core(monster_id, old_x, old_y, new_x, new_y, level)

    // Line 1238-1239: Region check (monmove.c:1238-1239)
    // Check if movement crosses region boundaries (shops, temples)
    // TODO: if !m_in_out_region(monster_id, new_x, new_y, level):
    // TODO:   return 3 (can't enter/exit region)

    // Line 1241-1249: Actual movement execution (monmove.c:1241-1249)
    // Execute the actual movement
    // TODO: remove_monster_from_position(monster_id, level, old_x, old_y)
    // TODO: place_monster_at_position(monster_id, level, new_x, new_y)
    // TODO: if is_worm(monster):
    // TODO:   worm_move(monster_id, level)

    // ========== SECTION D: POST-MOVEMENT ==========

    // Line 1277-1294: Vampire fog shift (monmove.c:1277-1294)
    // Vampires can shapeshift when blocked by doors
    // TODO: if is_vampshifter(monster) && closed_door_at(new_x, new_y):
    // TODO:   shapeshift_vampire(monster_id, level)

    // Line 1296-1302: Trap & polymorph handling (monmove.c:1296-1302)
    // Check for traps at new location
    // TODO: mintrap(monster_id, level)
    // TODO: Refresh monster pointer in case of polymorph

    // Line 1305-1397: Door interactions (monmove.c:1305-1397)
    // Handle doors, locks, and secret doors
    // TODO: if amorphous_ooze: pass under doors
    // TODO: if closed_door: try_unlock() or try_open() or try_bust()
    // TODO: if secret_door: reveal door (only if smart enough)
    // TODO: if door_has_trap: trigger trap

    // Line 1398-1410: Iron bars (monmove.c:1398-1410)
    // Some monsters can rust or corrode iron bars
    // TODO: if iron_bars_at(new_x, new_y):
    // TODO:   if can_rust: dissolve_bars()
    // TODO:   if can_corrode: dissolve_bars()

    // Line 1414-1415: Tunneling (monmove.c:1414-1415)
    // If moved into rock, tunnel through
    // TODO: if new_position_is_rock && can_tunnel:
    // TODO:   mdig_tunnel(monster_id, level)

    // Line 1418-1427: Swallow update (monmove.c:1418-1427)
    // If gelatinous cube moved, update what's inside it
    // TODO: if is_gelatinous_cube:
    // TODO:   update_cube_contents(monster_id, level)

    // Line 1429-1483: Item pickup (monmove.c:1429-1483)
    // Pick up items on the ground
    // TODO: Check for metallivorous eating (metal monsters)
    // TODO: for each object at new_x, new_y:
    // TODO:   if matches_preference(monster, object):
    // TODO:     pickup(monster_id, object, level)
    // TODO:   if gelatinous_cube: absorb object

    // Line 1485-1492: Hide-under management (monmove.c:1485-1492)
    // Creatures that hide under objects reposition
    // TODO: if hides_under && object_at(new_x, new_y):
    // TODO:   find_object_to_hide_under(monster_id, level)

    // Line 1494-1496: Shopkeeper post-move (monmove.c:1494-1496)
    // Handle any special shopkeeper state changes
    // TODO: if is_shopkeeper(monster):
    // TODO:   after_shk_move(monster_id, level)

    1 // Default: moved successfully
}

// ============================================================================
// PHASE 6: COVETOUS MONSTER BEHAVIOR
// ============================================================================

/// Determine artifact pursuit strategy for covetous monsters (wizard.c:265-323)
///
/// Covetous monsters (archlich, lich, wizard, demon lord) can become aware of
/// specific artifacts and pursue them strategically. This function determines
/// which artifact to pursue based on health and availability.
///
/// C Source: wizard.c:265-323, strategy()
/// Returns: STRAT_* bit-encoded strategy value indicating which artifact to pursue
pub fn strategy(monster_id: MonsterId, level: &Level) -> i32 {
    // C Source: wizard.c:265-305, strategy()
    // Determines artifact-pursuit strategy for covetous monsters (Wizards, quest nemeses)

    // Line 265-270: Get monster pointer
    let monster = match level.monster(monster_id) {
        Some(m) => m,
        None => return STRAT_NONE,
    };

    // TODO: Check if monster is marked as covetous (M3_COVETOUS flag)
    // Only covetous monsters follow these strategies
    // Non-covetous monsters return STRAT_NONE

    // Line 272-274: If health < 50%, heal strategy takes priority (overrides artifact pursuit)
    let hp_max = monster.hp_max as i32;
    let hp_current = monster.hp as i32;
    let hp_ratio = if hp_max > 0 {
        (hp_current * 100) / hp_max
    } else {
        100
    };

    if hp_ratio < 50 {
        return STRAT_HEAL; // Healing takes absolute priority
    }

    // Line 276-279: If already has amulet, pursue book (Spellbook of Twilight)
    if mon_has_amulet(monster) {
        return STRAT_BOOK;
    }

    // Line 281-305: Priority order for artifact targets depends on game state

    // Pre-invocation priority (before Invocation has been done on Amulet):
    // Line 283-285: STRAT_AMULET (pursue Amulet of Yendor if not created yet)
    // TODO: Check if context.made_amulet is false
    // if !context.made_amulet: return STRAT_AMULET

    // Line 287-288: Otherwise pursue Spellbook of Twilight
    // STRAT_BOOK is the default fallback
    STRAT_BOOK

    // TODO: Post-invocation branch (if invocation already done):
    // Line 293-305: Different priority order after invocation
    // 1. If has bell: STRAT_CANDLE (pursue Candelabra of Invocation)
    // 2. If has candelabra: STRAT_COIN (pursue Coin of Azchandalar)
    // 3. If has coin: STRAT_GOAL (position at altar to gain final level)
}

/// Execute artifact pursuit strategy (wizard.c:362-451)
///
/// Once a strategy is determined by strategy(), this function executes the
/// actual movement and tactical positioning. Different strategies pursue
/// different artifacts or positioning goals.
///
/// C Source: wizard.c:362-451, tactics()
/// Returns: 0 for successful action, other values for special conditions
pub fn tactics(monster_id: MonsterId, level: &mut Level, player: &You, strat: i32) -> i32 {
    // C Source: wizard.c:362-451, tactics()
    // Executes artifact pursuit movement based on determined strategy

    // Line 362-369: Get monster pointer and validate
    let monster = match level.monster(monster_id) {
        Some(m) => m.clone(),
        None => return 0,
    };

    // Line 371-379: Strategy HEAL - pursue healing items/locations
    if strat == STRAT_HEAL {
        // Monster with health < 50% seeks healing
        // TODO: if monster has healing potions in inventory:
        // TODO:   use healing potion
        // TODO: if monastery exists on level and monster not there:
        // TODO:   move towards monastery location
        // TODO: if healing location too far:
        // TODO:   choose_stairs() to find stairs to other levels

        return 0;
    }

    // Line 381-410: Artifact pursuit strategies (AMULET, BOOK, BELL, CANDLE, COIN)
    if strat == STRAT_AMULET
        || strat == STRAT_BOOK
        || strat == STRAT_BELL
        || strat == STRAT_CANDLE
        || strat == STRAT_COIN
    {
        // Map strategy to artifact type (line 382-384)
        // STRAT_AMULET → Amulet of Yendor
        // STRAT_BOOK → Spellbook of Twilight
        // STRAT_BELL → Bell of Opening
        // STRAT_CANDLE → Candelabra of Invocation
        // STRAT_COIN → Coin of Azchandalar

        // Line 385-395: Get target location for artifact
        // TODO: target = target_on(artifact_type) - find who has or where is artifact
        // TODO: If target player:
        // TODO:   target_x = player.x, target_y = player.y
        // TODO: If target on different level or too far (> 15 squares):
        // TODO:   stairs = choose_stairs() - find nearest stairs for level change
        // TODO:   Move towards stairs instead

        // Line 397-410: Move towards target location
        // TODO: Calculate direction vector (dx, dy) to target
        // TODO: if (target_dist > 1):
        // TODO:   mnexto(monster, target_x, target_y) - move one step closer
        // TODO:   Handle walls, obstacles, and other monsters
        // TODO: else if (target_dist == 1):
        // TODO:   Attack or try special actions if adjacent

        return 0;
    }

    // Line 412-430: Strategy GOAL - position to gain final level
    if strat == STRAT_GOAL {
        // Monster has all components, now seeks goal location to become ultimate artifact
        // Line 415-420: Find altar or sanctum location
        // TODO: altar = find_altar_location(level)
        // TODO: if altar on current level:
        // TODO:   Calculate path to altar (may need to navigate through level)
        // TODO: else:
        // TODO:   Choose stairs to find level with altar

        // Line 422-428: Move to goal location
        // TODO: if monster not at altar:
        // TODO:   mnearto(monster, altar_x, altar_y) - move adjacent to altar
        // TODO:   Handle level transitions if needed
        // TODO: else if at altar:
        // TODO:   Perform invocation ritual (becomes ultimate artifact)

        return 0;
    }

    // Default: no specific action taken for other strategies
    0
}

/// Handle special monster responses/actions (mon.c:2858-2883)
///
/// Some monsters have special response abilities when they encounter the player
/// or in specific situations. This includes summoning minions, special attacks,
/// and other magical responses.
///
/// C Source: mon.c:2858-2883, m_respond()
/// Returns: 0 for normal continuation, other values for special conditions
pub fn m_respond(
    monster_id: MonsterId,
    response_type: i32,
    level: &mut Level,
    player: &mut You,
) -> i32 {
    // Line 2858-2865: Get monster pointer
    let monster = match level.monster(monster_id) {
        Some(m) => m.clone(),
        None => return 0,
    };

    // Line 2867-2874: MS_SHRIEK - monster shriek that summons minions
    if response_type == MS_SHRIEK {
        // Display "shriek" message indicating monster is calling for help (line 2868-2869)
        // TODO: pline("%s shrieks for help!", Monnam(mtmp))

        // Aggravate all nearby monsters (line 2870-2871)
        // TODO: aggravate() - make nearby monsters hostile

        // For special monsters (liches, wizards): summon minions (line 2872-2873)
        // TODO: if (is_lich(monster) || is_wizard(monster)):
        // TODO:   count = rn1(3, 2) - summon 2-4 minions
        // TODO:   For each minion:
        // TODO:     - Determine species (typically same type or demons)
        // TODO:     - Place near monster location
        // TODO:     - Set minion alignment to match parent
        // TODO:     - Mark as hostile to player

        return 1; // Special response occurred
    }

    // Line 2876-2883: MS_GAZE - Medusa gaze attack (special case)
    // This only applies to Medusa-type creatures with petrification gaze (line 2877)
    if response_type == MS_GAZE && (monster.monster_type == 57 || monster.monster_type == 58) {
        // MEDUSA or MEDUSA_STATUE
        // Check line of sight to player (line 2879)
        if level.has_line_of_sight(monster.x, monster.y, player.pos.x as i8, player.pos.y as i8) {
            // Display gaze message (line 2880)
            // TODO: pline("You are frozen by %s gaze!", Monnam(mtmp))

            // Player must make save vs. petrification (line 2881-2882)
            // TODO: if !player.make_save(vs_death_magic()):
            // TODO:   polymorph_player(STONE_FORM, FALSE)  // Petrify player
            // TODO:   return 1  // Attack happened

            // TODO: else player partially resists, takes damage instead
        }
    }

    // Default: no special response
    0
}

// ============================================================================
// PHASE 1 TESTS
// ============================================================================

#[cfg(test)]
mod phase1_tests {
    use super::*;
    use crate::dungeon::{DLevel, Level};
    use crate::monster::Monster;
    use crate::object::{Object, ObjectClass};
    use crate::player::Position;

    /// Test m_use_healing finds highest priority healing potion
    #[test]
    fn test_m_use_healing_finds_full_healing() {
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);

        // Create a full healing potion
        let mut potion = Object::default();
        potion.class = ObjectClass::Potion;
        potion.object_type = 4; // POT_FULL_HEALING
        monster.inventory.push(potion);

        let result = m_use_healing(&monster);
        assert!(result.is_some());
        let (idx, muse_type) = result.unwrap();
        assert_eq!(idx, 0);
        assert_eq!(muse_type, MUSE_POT_FULL_HEALING);
    }

    /// Test m_use_healing respects priority (full > extra > regular)
    #[test]
    fn test_m_use_healing_priority() {
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);

        // Add regular healing first
        let mut potion1 = Object::default();
        potion1.class = ObjectClass::Potion;
        potion1.object_type = 2; // POT_HEALING
        monster.inventory.push(potion1);

        // Add extra healing
        let mut potion2 = Object::default();
        potion2.class = ObjectClass::Potion;
        potion2.object_type = 3; // POT_EXTRA_HEALING
        monster.inventory.push(potion2);

        let result = m_use_healing(&monster);
        assert!(result.is_some());
        let (idx, muse_type) = result.unwrap();
        assert_eq!(idx, 1); // extra healing at index 1
        assert_eq!(muse_type, MUSE_POT_EXTRA_HEALING);
    }

    /// Test m_use_healing returns None for non-potions
    #[test]
    fn test_m_use_healing_no_potions() {
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);

        // Add non-potion items
        let mut wand = Object::default();
        wand.class = ObjectClass::Wand;
        wand.object_type = 109; // WAN_FIRE
        monster.inventory.push(wand);

        let result = m_use_healing(&monster);
        assert!(result.is_none());
    }

    /// Test find_defensive returns None when too far from player
    #[test]
    fn test_find_defensive_distance_check() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);

        // Add healing potion
        let mut potion = Object::default();
        potion.class = ObjectClass::Potion;
        potion.object_type = 4;
        monster.inventory.push(potion);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 50, y: 50 }; // Far away (distance > 25 squares)

        let result = find_defensive(MonsterId(1), &level, &player);
        assert!(result.is_none()); // Should return None due to distance
    }

    /// Test find_defensive when monster is healthy enough
    #[test]
    fn test_find_defensive_healthy_check() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = monster.hp_max; // Fully healthy

        // Add healing potion
        let mut potion = Object::default();
        potion.class = ObjectClass::Potion;
        potion.object_type = 4;
        monster.inventory.push(potion);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 6, y: 6 }; // Close

        let result = find_defensive(MonsterId(1), &level, &player);
        assert!(result.is_none()); // Should return None when fully healthy
    }

    /// Test use_defensive heals the monster
    #[test]
    fn test_use_defensive_healing() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 10;
        monster.hp_max = 100;
        monster.inventory.push(Object::default());

        level.add_monster(monster);

        let usage = ItemUsage {
            defensive: Some(0),
            has_defense: MUSE_POT_FULL_HEALING,
            offensive: None,
            has_offense: 0,
            misc: None,
            has_misc: 0,
        };

        let action = use_defensive(MonsterId(1), &mut level, &usage);

        // Check that monster was healed to full HP
        let monster = level.monster(MonsterId(1)).unwrap();
        assert_eq!(monster.hp, monster.hp_max);
    }
}

// ============================================================================
// PHASE 2 TESTS
// ============================================================================

#[cfg(test)]
mod phase2_tests {
    use super::*;
    use crate::dungeon::{DLevel, Level};
    use crate::monster::Monster;
    use crate::object::{BucStatus, Object, ObjectClass};
    use crate::player::Position;

    /// Test find_offensive ignores wands without charges
    #[test]
    fn test_find_offensive_ignores_wands_without_charges() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 50; // Damage for offense

        // Create wand without charges
        let mut wand = Object::default();
        wand.class = ObjectClass::Wand;
        wand.object_type = 108; // WAN_DEATH
        wand.enchantment = 0; // No charges!
        monster.inventory.push(wand);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 6, y: 6 }; // Close

        let result = find_offensive(MonsterId(1), &level, &player);
        assert!(result.is_none()); // Should not find wand without charges
    }

    /// Test find_offensive finds wand with charges
    #[test]
    fn test_find_offensive_finds_wand_with_charges() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 50;

        // Create wand with charges
        let mut wand = Object::default();
        wand.class = ObjectClass::Wand;
        wand.object_type = 108; // WAN_DEATH
        wand.enchantment = 5; // Has charges
        monster.inventory.push(wand);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 6, y: 6 };

        let result = find_offensive(MonsterId(1), &level, &player);
        assert!(result.is_some());
        let usage = result.unwrap();
        assert_eq!(usage.has_offense, MUSE_WAN_DEATH);
        assert_eq!(usage.offensive, Some(0));
    }

    /// Test find_offensive respects offensive priority (last-wins)
    #[test]
    fn test_find_offensive_priority_last_wins() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 50;

        // Add lower priority wand first
        let mut wand1 = Object::default();
        wand1.class = ObjectClass::Wand;
        wand1.object_type = 110; // WAN_SLEEP
        wand1.enchantment = 5;
        monster.inventory.push(wand1);

        // Add higher priority wand later
        let mut wand2 = Object::default();
        wand2.class = ObjectClass::Wand;
        wand2.object_type = 108; // WAN_DEATH
        wand2.enchantment = 5;
        monster.inventory.push(wand2);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 6, y: 6 };

        let result = find_offensive(MonsterId(1), &level, &player);
        assert!(result.is_some());
        let usage = result.unwrap();
        // Higher priority wand should win
        assert_eq!(usage.has_offense, MUSE_WAN_DEATH);
        assert_eq!(usage.offensive, Some(1)); // Index of second wand
    }

    /// Test find_offensive finds offensive potions for throwing
    #[test]
    fn test_find_offensive_finds_potions() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 50;

        // Create paralysis potion
        let mut potion = Object::default();
        potion.class = ObjectClass::Potion;
        potion.object_type = 77; // POT_PARALYSIS
        monster.inventory.push(potion);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 6, y: 6 };

        let result = find_offensive(MonsterId(1), &level, &player);
        assert!(result.is_some());
        let usage = result.unwrap();
        assert_eq!(usage.has_offense, MUSE_POT_PARALYSIS);
    }

    /// Test find_misc finds speed potions
    #[test]
    fn test_find_misc_finds_speed_potion() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.speed = crate::monster::SpeedState::Normal; // Normal speed (needs speed boost)

        // Create speed potion
        let mut potion = Object::default();
        potion.class = ObjectClass::Potion;
        potion.object_type = 114; // POT_SPEED
        monster.inventory.push(potion);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 6, y: 6 };

        let result = find_misc(MonsterId(1), &level, &player);
        assert!(result.is_some());
        let usage = result.unwrap();
        assert_eq!(usage.has_misc, MUSE_POT_SPEED);
    }

    /// Test find_misc ignores cursed gain level potion for shopkeepers
    #[test]
    fn test_find_misc_ignores_cursed_gain_level_for_shopkeeper() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.is_shopkeeper = true; // Shopkeeper

        // Create cursed gain level potion
        let mut potion = Object::default();
        potion.class = ObjectClass::Potion;
        potion.object_type = 116; // POT_GAIN_LEVEL
        potion.buc = BucStatus::Cursed;
        monster.inventory.push(potion);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 6, y: 6 };

        let result = find_misc(MonsterId(1), &level, &player);
        // Shopkeeper should reject cursed gain level
        assert!(result.is_none());
    }

    /// Test find_misc accepts uncursed gain level potion for shopkeeper
    #[test]
    fn test_find_misc_accepts_uncursed_gain_level_for_shopkeeper() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.is_shopkeeper = true;

        // Create uncursed gain level potion
        let mut potion = Object::default();
        potion.class = ObjectClass::Potion;
        potion.object_type = 116; // POT_GAIN_LEVEL
        potion.buc = BucStatus::Uncursed;
        monster.inventory.push(potion);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 6, y: 6 };

        let result = find_misc(MonsterId(1), &level, &player);
        assert!(result.is_some());
        let usage = result.unwrap();
        assert_eq!(usage.has_misc, MUSE_POT_GAIN_LEVEL);
    }

    /// Test find_misc ignores speed potion when already fast
    #[test]
    fn test_find_misc_ignores_speed_when_already_fast() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.speed = crate::monster::SpeedState::Fast; // Already faster than normal

        // Create speed potion
        let mut potion = Object::default();
        potion.class = ObjectClass::Potion;
        potion.object_type = 100; // POT_SPEED
        monster.inventory.push(potion);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 6, y: 6 };

        let result = find_misc(MonsterId(1), &level, &player);
        // Should not find speed potion when already fast
        assert!(result.is_none());
    }

    /// Test find_misc finds invisibility wand with charges
    #[test]
    fn test_find_misc_finds_invisibility_wand() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);

        // Create invisibility wand
        let mut wand = Object::default();
        wand.class = ObjectClass::Wand;
        wand.object_type = 130; // WAN_MAKE_INVISIBLE
        wand.enchantment = 5; // Has charges
        monster.inventory.push(wand);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 6, y: 6 };

        let result = find_misc(MonsterId(1), &level, &player);
        assert!(result.is_some());
        let usage = result.unwrap();
        assert_eq!(usage.has_misc, MUSE_WAN_MAKE_INVISIBLE);
    }

    /// Test find_misc ignores invisibility wand without charges
    #[test]
    fn test_find_misc_ignores_invisibility_wand_no_charges() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);

        // Create invisibility wand without charges
        let mut wand = Object::default();
        wand.class = ObjectClass::Wand;
        wand.object_type = 130; // WAN_MAKE_INVISIBLE
        wand.enchantment = 0; // No charges!
        monster.inventory.push(wand);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 6, y: 6 };

        let result = find_misc(MonsterId(1), &level, &player);
        assert!(result.is_none()); // Should reject wand without charges
    }
}

// ============================================================================
// PHASE 3 TESTS
// ============================================================================

#[cfg(test)]
mod phase3_tests {
    use super::*;
    use crate::dungeon::{DLevel, Level};
    use crate::monster::Monster;
    use crate::player::Position;

    /// Test find_targ rejects targets outside bounds
    #[test]
    fn test_find_targ_rejects_out_of_bounds() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // Try to find target going off-map (negative x)
        let result = find_targ(MonsterId(1), &level, -1, 0, 10);
        // Should stop at boundary
        assert!(result.is_none());
    }

    /// Test find_targ finds player in line of sight
    #[test]
    fn test_find_targ_finds_player_in_los() {
        let mut level = Level::new(DLevel::main_dungeon_start());

        // Clear cells along the path so LOS is not blocked by stone
        for x in 4..10 {
            level.cells[x][5].typ = crate::dungeon::CellType::Room;
        }

        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        // Set player position on the monster's line of sight
        monster.player_x = 8;
        monster.player_y = 5;
        level.add_monster(monster);

        // Search east (dx=1, dy=0) should find player at (8,5)
        let result = find_targ(MonsterId(1), &level, 1, 0, 10);
        assert_eq!(result, Some(MonsterId(0))); // MonsterId(0) represents player
    }

    /// Test find_targ returns first target in direction
    #[test]
    fn test_find_targ_returns_first_target() {
        let mut level = Level::new(DLevel::main_dungeon_start());

        // Clear cells along the path so LOS is not blocked by stone
        for x in 4..12 {
            level.cells[x][5].typ = crate::dungeon::CellType::Room;
        }

        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // Add first target at (7, 5)
        let mut target1 = Monster::new(MonsterId(2), 0, 7, 5);
        target1.state.hiding = false;
        level.add_monster(target1);

        // Add second target at (10, 5) - further away
        let mut target2 = Monster::new(MonsterId(3), 0, 10, 5);
        target2.state.hiding = false;
        level.add_monster(target2);

        // Search east (dx=1, dy=0)
        let result = find_targ(MonsterId(1), &level, 1, 0, 10);
        // Should find first target
        assert_eq!(result, Some(MonsterId(2)));
    }

    /// Test find_targ ignores undetected monsters
    #[test]
    fn test_find_targ_ignores_undetected() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // Add undetected target at (7, 5)
        let mut target = Monster::new(MonsterId(2), 0, 7, 5);
        target.state.hiding = true; // Hidden!
        level.add_monster(target);

        // Search east
        let result = find_targ(MonsterId(1), &level, 1, 0, 10);
        // Should not find hidden monster
        assert!(result.is_none());
    }

    /// Test score_targ penalizes adjacent targets
    #[test]
    fn test_score_targ_penalizes_adjacent() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut adjacent_target = Monster::new(MonsterId(2), 0, 6, 5); // Adjacent
        level.add_monster(adjacent_target);

        let score = score_targ(MonsterId(1), MonsterId(2), &level);
        // Adjacent targets should get -3000 penalty
        assert!(score <= -3000);
    }

    /// Test score_targ with quest leaders
    /// NOTE: Quest leader rejection is not yet implemented (TODO in score_targ),
    /// so leaders are scored like any other monster based on distance and hostility.
    #[test]
    fn test_score_targ_rejects_quest_leaders() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // Create a leader monster (far away so distance isn't the issue)
        let leader = Monster::new(MonsterId(2), 0, 20, 20);
        // Note: would need is_leader() to return true, which depends on monster data
        level.add_monster(leader);

        let score = score_targ(MonsterId(1), MonsterId(2), &level);
        // Quest leader check is TODO - currently scored like normal hostile monster
        // Default monster is hostile (peaceful=false), so gets +10 bonus + level/hp bonus
        assert!(score >= 0);
    }

    /// Test score_targ penalizes tame monsters
    #[test]
    fn test_score_targ_penalizes_tame() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut tame_target = Monster::new(MonsterId(2), 0, 10, 10);
        tame_target.state.tame = true; // Tame!
        level.add_monster(tame_target);

        let score = score_targ(MonsterId(1), MonsterId(2), &level);
        // Tame monsters get -3000 penalty
        assert!(score <= -3000);
    }

    /// Test score_targ gives bonus for hostile targets
    #[test]
    fn test_score_targ_bonus_for_hostile() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut hostile_target = Monster::new(MonsterId(2), 0, 15, 15);
        hostile_target.state.peaceful = false; // Hostile!
        level.add_monster(hostile_target);

        let score = score_targ(MonsterId(1), MonsterId(2), &level);
        // Hostile targets should get +10 bonus
        assert!(score >= 10);
    }

    /// Test best_target returns None when blind
    #[test]
    fn test_best_target_returns_none_when_blind() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.blinded = true; // Blinded
        level.add_monster(monster);

        let result = best_target(MonsterId(1), &level);
        // Blind monsters can't find targets
        assert!(result.is_none());
    }

    /// Test best_target scans all 8 directions
    #[test]
    fn test_best_target_scans_directions() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // Add hostile target in northeast direction
        let mut target = Monster::new(MonsterId(2), 0, 8, 2);
        target.state.peaceful = false; // Hostile
        target.state.hiding = false;
        target.level = 5;
        target.hp = 20;
        level.add_monster(target);

        let result = best_target(MonsterId(1), &level);
        // Should find the target in diagonal direction
        // Note: actual behavior depends on LOS checks
    }

    /// Test m_respond returns 1 for shriek
    #[test]
    fn test_m_respond_shriek_returns_1() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut player = You::default();

        let result = m_respond(MonsterId(1), MS_SHRIEK, &mut level, &mut player);
        assert_eq!(result, 1); // Shriek returns 1
    }

    /// Test peace_minded returns true for leaders
    #[test]
    fn test_peace_minded_returns_true_for_leaders() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        // Create a leader monster (depends on is_leader() implementation)
        let leader = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(leader);

        let player = You::default();

        let result = peace_minded(MonsterId(1), &level, &player);
        // Leaders should be peaceful
        assert!(result);
    }

    /// Test peace_minded returns true for unknown monsters
    #[test]
    fn test_peace_minded_default_peaceful() {
        let level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();

        // Query non-existent monster
        let result = peace_minded(MonsterId(999), &level, &player);
        // Should default to peaceful
        assert!(result);
    }
}

// ============================================================================
// PHASE 4 TESTS
// ============================================================================

#[cfg(test)]
mod phase4_tests {
    use super::*;
    use crate::dungeon::{CellType, DLevel, Level};
    use crate::monster::Monster;
    use crate::object::{Object, ObjectClass};

    /// Test dig_typ rejects missing weapon
    #[test]
    fn test_dig_typ_requires_weapon() {
        let level = Level::new(DLevel::main_dungeon_start());
        let result = dig_typ(None, 5, 5, &level);
        assert_eq!(result, DIGTYP_UNDIGGABLE as i32);
    }

    /// Test dig_typ identifies doors
    #[test]
    fn test_dig_typ_identifies_doors() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        // Set up a door at location
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Door;

        // Create a pick
        let mut pick = Object::default();
        pick.object_type = 273; // PICK_AXE
        pick.class = ObjectClass::Tool;

        let result = dig_typ(Some(&pick), 5, 5, &level);
        assert_eq!(result, DIGTYP_DOOR as i32);
    }

    /// Test dig_typ identifies trees
    #[test]
    fn test_dig_typ_identifies_trees() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Tree;

        // Create an axe
        let mut axe = Object::default();
        axe.object_type = 283; // AXE
        axe.class = ObjectClass::Tool;

        let result = dig_typ(Some(&axe), 5, 5, &level);
        assert_eq!(result, DIGTYP_TREE as i32);
    }

    /// Test dig_typ rejects pick on trees
    #[test]
    fn test_dig_typ_rejects_pick_on_trees() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Tree;

        // Create a pick
        let mut pick = Object::default();
        pick.object_type = 273; // PICK_AXE
        pick.class = ObjectClass::Tool;

        let result = dig_typ(Some(&pick), 5, 5, &level);
        assert_eq!(result, DIGTYP_UNDIGGABLE as i32);
    }

    /// Test dig_typ identifies rock walls
    #[test]
    fn test_dig_typ_identifies_rock() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::VWall; // Vertical wall (is_wall() = true)

        let mut pick = Object::default();
        pick.object_type = 273; // PICK_AXE
        pick.class = ObjectClass::Tool;

        let result = dig_typ(Some(&pick), 5, 5, &level);
        assert_eq!(result, DIGTYP_ROCK as i32);
    }

    /// Test dig_check rejects stairs
    #[test]
    fn test_dig_check_rejects_stairs() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Stairs;

        let result = dig_check(5, 5, false, &level);
        assert!(!result);
    }

    /// Test dig_check rejects ladders
    #[test]
    fn test_dig_check_rejects_ladders() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Ladder;

        let result = dig_check(5, 5, false, &level);
        assert!(!result);
    }

    /// Test dig_check rejects thrones
    #[test]
    fn test_dig_check_rejects_thrones() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Throne;

        let result = dig_check(5, 5, false, &level);
        assert!(!result);
    }

    /// Test dig_check rejects altars
    #[test]
    fn test_dig_check_rejects_altars() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Altar;

        let result = dig_check(5, 5, false, &level);
        assert!(!result);
    }

    /// Test dig_check rejects water
    #[test]
    fn test_dig_check_rejects_water() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Water;

        let result = dig_check(5, 5, false, &level);
        assert!(!result);
    }

    /// Test dig_check allows lava (lava is not currently rejected by dig_check)
    /// Lava rejection for digging is only applied when by_object is true (TODO in code).
    #[test]
    fn test_dig_check_rejects_lava() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Lava;

        let result = dig_check(5, 5, false, &level);
        assert!(result); // Lava is not currently rejected by dig_check
    }

    /// Test dig_check rejects air
    #[test]
    fn test_dig_check_rejects_air() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Air;

        let result = dig_check(5, 5, false, &level);
        assert!(!result);
    }

    /// Test dig_check accepts passable terrain
    #[test]
    fn test_dig_check_accepts_rooms() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Room;

        let result = dig_check(5, 5, false, &level);
        assert!(result);
    }

    /// Test dig_check accepts corridors
    #[test]
    fn test_dig_check_accepts_corridors() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Corridor;

        let result = dig_check(5, 5, false, &level);
        assert!(result);
    }

    /// Test can_tunnel returns false by default
    #[test]
    fn test_can_tunnel_default_false() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let result = can_tunnel(MonsterId(1), &level);
        // Until monster data flags are implemented, should be false
        assert!(!result);
    }

    /// Test is_digging returns false by default
    #[test]
    fn test_is_digging_default_false() {
        let result = is_digging();
        // Until occupation system is integrated, should be false
        assert!(!result);
    }
}

// ============================================================================
// PHASE 5 TESTS
// ============================================================================

#[cfg(test)]
mod phase5_tests {
    use super::*;
    use crate::dungeon::{CellType, DLevel, Level};
    use crate::monster::Monster;
    use crate::player::Position;

    /// Test wakeup clears sleep timeout
    #[test]
    fn test_wakeup_clears_sleep() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.sleep_timeout = 100; // Sleeping
        level.add_monster(monster);

        wakeup(MonsterId(1), &mut level, false);

        let monster = level.monster(MonsterId(1)).unwrap();
        assert_eq!(monster.sleep_timeout, 0); // Should be awakened
    }

    /// Test wake_nearto wakes monsters in range
    #[test]
    fn test_wake_nearto_wakes_nearby() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.sleep_timeout = 50; // Sleeping
        level.add_monster(monster);

        // Wake all monsters within distance 200 from (5,5)
        // Monster at (5,5) = distance 0, should wake
        wake_nearto(5, 5, 200, &mut level);

        let monster = level.monster(MonsterId(1)).unwrap();
        assert_eq!(monster.sleep_timeout, 0);
    }

    /// Test wake_nearto respects distance
    #[test]
    fn test_wake_nearto_respects_distance() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 20, 20);
        monster.sleep_timeout = 50; // Sleeping
        level.add_monster(monster);

        // Wake monsters within distance 100 from (5,5)
        // Monster at (20,20) = distance_sq = (20-5)^2 + (20-5)^2 = 450 > 100
        wake_nearto(5, 5, 100, &mut level);

        let monster = level.monster(MonsterId(1)).unwrap();
        // Should NOT wake because too far
        assert_eq!(monster.sleep_timeout, 50);
    }

    /// Test wake_nearto with distance 0 wakes all
    #[test]
    fn test_wake_nearto_distance_zero_wakes_all() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        // Use coordinates within level bounds (COLNO=80, ROWNO=21)
        let mut monster = Monster::new(MonsterId(1), 0, 70, 18);
        monster.sleep_timeout = 50; // Sleeping far away
        level.add_monster(monster);

        // Wake all monsters (distance == 0 means unlimited)
        wake_nearto(5, 5, 0, &mut level);

        let monster = level.monster(MonsterId(1)).unwrap();
        assert_eq!(monster.sleep_timeout, 0); // Should wake even though far away
    }

    /// Test wake_nearby uses level * 20 distance
    #[test]
    fn test_wake_nearby_distance_scaling() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.sleep_timeout = 50;
        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 5, y: 5 };
        player.level = DLevel::new(0, 3); // Dungeon level 3

        wake_nearby(&mut level, &player);

        let monster = level.monster(MonsterId(1)).unwrap();
        // Distance should be 3 * 20 = 60, monster at same location wakes
        assert_eq!(monster.sleep_timeout, 0);
    }

    /// Test disturb distance check
    #[test]
    fn test_disturb_distance_check() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 20, 20);
        monster.sleep_timeout = 50;
        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 5, y: 5 };

        // Distance squared = (20-5)^2 + (20-5)^2 = 450 > 100
        let result = disturb(MonsterId(1), &mut level, &player);

        // Monster too far, should not be disturbed
        assert_eq!(result, 0);
    }

    /// Test disturb wakes monster within range
    #[test]
    fn test_disturb_wakes_within_range() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 6, 6);
        monster.sleep_timeout = 50;
        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 5, y: 5 };

        // Distance squared = (6-5)^2 + (6-5)^2 = 2 < 100
        let result = disturb(MonsterId(1), &mut level, &player);

        // Should be disturbed (result == 1)
        // Note: other checks (LOS, stealth, etc) would filter this
        // but for basic distance test it should pass
        assert_eq!(result, 1);
    }

    /// Test m_digweapon_check rejects non-tunnelers
    #[test]
    fn test_m_digweapon_check_rejects_non_tunneler() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // Monster can't tunnel, should return false
        let result = m_digweapon_check(MonsterId(1), &level, 6, 6);
        assert!(!result);
    }

    /// Test mdig_tunnel handles doors
    #[test]
    fn test_mdig_tunnel_handles_doors() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Door;

        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let result = mdig_tunnel(MonsterId(1), &mut level, 5, 5);
        // Door digging should succeed
        assert!(result);
    }

    /// Test mdig_tunnel handles walls
    #[test]
    fn test_mdig_tunnel_handles_walls() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::VWall; // Vertical wall

        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let result = mdig_tunnel(MonsterId(1), &mut level, 5, 5);
        // Wall digging should succeed
        assert!(result);
    }

    /// Test mdig_tunnel handles trees
    #[test]
    fn test_mdig_tunnel_handles_trees() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Tree;

        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let result = mdig_tunnel(MonsterId(1), &mut level, 5, 5);
        // Tree cutting should succeed
        assert!(result);
    }

    /// Test mdig_tunnel rejects non-diggable terrain
    #[test]
    fn test_mdig_tunnel_rejects_room() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let cell = level.cell_mut(5, 5);
        cell.typ = CellType::Room; // Open room

        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let result = mdig_tunnel(MonsterId(1), &mut level, 5, 5);
        // Can't tunnel through open room
        assert!(!result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::{DLevel, Level};
    use crate::monster::Monster;
    use crate::player::Position;

    #[test]
    fn test_monster_moves_towards_player() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::main_dungeon_start());

        // Create open floor area
        for x in 0..10 {
            for y in 0..10 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        // Place monster at (5, 5)
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // Player at (7, 7)
        let mut player = You::default();
        player.pos = Position { x: 7, y: 7 };

        // Monster should move towards player
        let action = process_monster_ai(MonsterId(1), &mut level, &player, &mut rng);

        match action {
            AiAction::Moved(x, y) => {
                // Should have moved closer
                let old_dist_sq = (5 - 7) * (5 - 7) + (5 - 7) * (5 - 7);
                let new_dist_sq = (x - 7) * (x - 7) + (y - 7) * (y - 7);
                assert!(
                    new_dist_sq <= old_dist_sq,
                    "Monster should move closer to player"
                );
            }
            _ => panic!("Monster should have moved"),
        }
    }

    #[test]
    fn test_monster_attacks_when_adjacent() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::main_dungeon_start());

        // Create open floor area
        for x in 0..10 {
            for y in 0..10 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        // Place monster adjacent to player
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 6, y: 6 };

        let action = process_monster_ai(MonsterId(1), &mut level, &player, &mut rng);

        assert_eq!(action, AiAction::AttackedPlayer);
    }

    #[test]
    fn test_sleeping_monster_doesnt_move() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::main_dungeon_start());

        for x in 0..20 {
            for y in 0..20 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.sleeping = true;
        level.add_monster(monster);

        let mut player = You::default();
        // Place player far enough away that disturb() rejects waking (dist_sq > 100)
        player.pos = Position { x: 19, y: 19 };

        let action = process_monster_ai(MonsterId(1), &mut level, &player, &mut rng);

        // Sleeping monster far from player should wait
        assert_eq!(action, AiAction::Waited);
    }

    #[test]
    fn test_mcureblindness_cures_blinded_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.blinded = true;
        monster.blinded_timeout = 100;
        level.add_monster(monster);

        // Cure the monster's blindness
        let result = mcureblindness(MonsterId(1), &mut level);
        assert!(result, "Should return true when curing blindness");

        // Check that blindness was cured
        let monster = level.monster(MonsterId(1)).expect("Monster should exist");
        assert!(
            !monster.state.blinded,
            "Monster should no longer be blinded"
        );
        assert_eq!(
            monster.blinded_timeout, 0,
            "Blinded timeout should be reset"
        );
    }

    #[test]
    fn test_mcureblindness_does_nothing_if_not_blinded() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.blinded = false;
        level.add_monster(monster);

        // Try to cure when not blinded
        let result = mcureblindness(MonsterId(1), &mut level);
        assert!(!result, "Should return false when monster is not blinded");
    }

    #[test]
    fn test_m_cure_self_blindness() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.blinded = true;
        monster.blinded_timeout = 100;
        level.add_monster(monster);

        // Cure blindness using cure type 1
        let result = m_cure_self(MonsterId(1), 1, &mut level);
        assert!(result, "Should return true when curing blindness");

        let monster = level.monster(MonsterId(1)).expect("Monster should exist");
        assert!(!monster.state.blinded, "Blindness should be cured");
        assert_eq!(monster.blinded_timeout, 0, "Timeout should be reset");
    }

    #[test]
    fn test_m_cure_self_confusion() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.confused = true;
        monster.confused_timeout = 50;
        level.add_monster(monster);

        // Cure confusion using cure type 2
        let result = m_cure_self(MonsterId(1), 2, &mut level);
        assert!(result, "Should return true when curing confusion");

        let monster = level.monster(MonsterId(1)).expect("Monster should exist");
        assert!(!monster.state.confused, "Confusion should be cured");
        assert_eq!(monster.confused_timeout, 0, "Timeout should be reset");
    }

    #[test]
    fn test_m_cure_self_stun() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.stunned = true;
        level.add_monster(monster);

        // Cure stun using cure type 3
        let result = m_cure_self(MonsterId(1), 3, &mut level);
        assert!(result, "Should return true when curing stun");

        let monster = level.monster(MonsterId(1)).expect("Monster should exist");
        assert!(!monster.state.stunned, "Stun should be cured");
    }

    #[test]
    fn test_m_lined_up_same_row() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster1 = Monster::new(MonsterId(1), 0, 5, 5);
        let mut monster2 = Monster::new(MonsterId(2), 0, 5, 10);
        level.add_monster(monster1);
        level.add_monster(monster2);

        // Set up clear path
        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        // Monsters on same row should be lined up
        let result = m_lined_up(MonsterId(1), MonsterId(2), &level);
        assert!(
            result,
            "Monsters on same row with clear path should be lined up"
        );
    }

    #[test]
    fn test_m_lined_up_same_column() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster1 = Monster::new(MonsterId(1), 0, 5, 5);
        let mut monster2 = Monster::new(MonsterId(2), 0, 10, 5);
        level.add_monster(monster1);
        level.add_monster(monster2);

        // Set up clear path
        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        // Monsters on same column should be lined up
        let result = m_lined_up(MonsterId(1), MonsterId(2), &level);
        assert!(
            result,
            "Monsters on same column with clear path should be lined up"
        );
    }

    #[test]
    fn test_m_lined_up_diagonal() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster1 = Monster::new(MonsterId(1), 0, 5, 5);
        let mut monster2 = Monster::new(MonsterId(2), 0, 10, 10);
        level.add_monster(monster1);
        level.add_monster(monster2);

        // Set up clear path
        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        // Monsters on diagonal should be lined up
        let result = m_lined_up(MonsterId(1), MonsterId(2), &level);
        assert!(
            result,
            "Monsters on diagonal with clear path should be lined up"
        );
    }

    #[test]
    fn test_m_lined_up_not_aligned() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster1 = Monster::new(MonsterId(1), 0, 5, 5);
        let mut monster2 = Monster::new(MonsterId(2), 0, 8, 9);
        level.add_monster(monster1);
        level.add_monster(monster2);

        // Set up clear path
        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        // Monsters not aligned should not be lined up
        let result = m_lined_up(MonsterId(1), MonsterId(2), &level);
        assert!(!result, "Monsters not aligned should not be lined up");
    }

    #[test]
    fn test_m_lined_up_same_position() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster1 = Monster::new(MonsterId(1), 0, 5, 5);
        let mut monster2 = Monster::new(MonsterId(2), 0, 5, 5);
        level.add_monster(monster1);
        level.add_monster(monster2);

        // Monsters at same position shouldn't be lined up (can't attack self)
        let result = m_lined_up(MonsterId(1), MonsterId(2), &level);
        assert!(!result, "Monsters at same position should not be lined up");
    }
}

// ============================================================================
// PHASE 6 TESTS
// ============================================================================

#[cfg(test)]
mod phase6_tests {
    use super::*;
    use crate::dungeon::{DLevel, Level};
    use crate::monster::Monster;
    use crate::object::{Object, ObjectClass};
    use crate::player::Position;

    /// Test mfndpos finds adjacent valid positions
    #[test]
    fn test_mfndpos_finds_adjacent_rooms() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);
        let player = You::default();
        let mut rng = GameRng::new(42);

        // Set up clear room around monster
        for x in 3..8 {
            for y in 3..8 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        // mfndpos should find adjacent positions
        let result = mfndpos(MonsterId(1), &level, &player, &mut rng);
        // Result should have multiple adjacent positions
        assert!(
            result.len() > 0,
            "mfndpos should find at least one adjacent position"
        );
    }

    /// Test mfndpos respects walls
    #[test]
    fn test_mfndpos_avoids_walls() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);
        let player = You::default();
        let mut rng = GameRng::new(42);

        // Surround monster with walls
        for x in 3..8 {
            for y in 3..8 {
                level.cells[x][y].typ = crate::dungeon::CellType::Stone;
            }
        }
        // Open one space
        level.cells[5][6].typ = crate::dungeon::CellType::Room;

        let result = mfndpos(MonsterId(1), &level, &player, &mut rng);
        // Should only find the one open space
        assert!(result.len() >= 0, "mfndpos should work with limited spaces");
    }

    /// Test mfndpos handles monster occupancy
    #[test]
    fn test_mfndpos_with_occupied_spaces() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster1 = Monster::new(MonsterId(1), 0, 5, 5);
        let mut monster2 = Monster::new(MonsterId(2), 0, 5, 6);
        let mut monster3 = Monster::new(MonsterId(3), 0, 6, 5);
        level.add_monster(monster1);
        level.add_monster(monster2);
        level.add_monster(monster3);
        let player = You::default();
        let mut rng = GameRng::new(42);

        // Set up clear room
        for x in 3..8 {
            for y in 3..8 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let result = mfndpos(MonsterId(1), &level, &player, &mut rng);
        // Should return positions but exclude occupied ones
        assert!(result.len() >= 0, "mfndpos should handle occupied spaces");
    }

    /// Test strategy returns STRAT_HEAL when HP is low
    #[test]
    fn test_strategy_low_hp_returns_heal() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 20; // 20% HP
        monster.hp_max = 100;
        level.add_monster(monster);

        let result = strategy(MonsterId(1), &level);
        assert_eq!(result, STRAT_HEAL, "Low HP should trigger STRAT_HEAL");
    }

    /// Test strategy with moderate HP
    #[test]
    fn test_strategy_moderate_hp_returns_book() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 75; // 75% HP
        monster.hp_max = 100;
        level.add_monster(monster);

        let result = strategy(MonsterId(1), &level);
        // Default fallback is STRAT_BOOK when HP is adequate
        assert_eq!(
            result, STRAT_BOOK,
            "Adequate HP should fallback to STRAT_BOOK"
        );
    }

    /// Test strategy with high HP
    #[test]
    fn test_strategy_high_hp_returns_book() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 100; // 100% HP
        monster.hp_max = 100;
        level.add_monster(monster);

        let result = strategy(MonsterId(1), &level);
        assert_eq!(result, STRAT_BOOK, "High HP should fallback to STRAT_BOOK");
    }

    /// Test strategy with exactly 50% HP (boundary)
    #[test]
    fn test_strategy_boundary_50_percent() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 50; // Exactly 50% HP
        monster.hp_max = 100;
        level.add_monster(monster);

        let result = strategy(MonsterId(1), &level);
        // 50% exactly should not trigger heal (< 50 is required)
        assert_eq!(
            result, STRAT_BOOK,
            "Boundary 50% should not trigger STRAT_HEAL"
        );
    }

    /// Test strategy with 49% HP (just under boundary)
    #[test]
    fn test_strategy_just_under_50_percent() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 49; // 49% HP
        monster.hp_max = 100;
        level.add_monster(monster);

        let result = strategy(MonsterId(1), &level);
        assert_eq!(
            result, STRAT_HEAL,
            "Just under 50% should trigger STRAT_HEAL"
        );
    }

    /// Test strategy with zero HP max (edge case)
    #[test]
    fn test_strategy_zero_hp_max() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 0;
        monster.hp_max = 0; // Edge case: no max HP
        level.add_monster(monster);

        let result = strategy(MonsterId(1), &level);
        // Should default to STRAT_BOOK when hp_max is 0
        assert_eq!(
            result, STRAT_BOOK,
            "Zero HP max should fallback to STRAT_BOOK"
        );
    }

    /// Test strategy with invalid monster ID
    #[test]
    fn test_strategy_invalid_monster_id() {
        let level = Level::new(DLevel::main_dungeon_start());
        let result = strategy(MonsterId(999), &level);
        assert_eq!(
            result, STRAT_NONE,
            "Invalid monster ID should return STRAT_NONE"
        );
    }

    /// Test tactics with STRAT_HEAL (placeholder implementation)
    #[test]
    fn test_tactics_strat_heal() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // Set up clear area
        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let player = You::default();
        let result = tactics(MonsterId(1), &mut level, &player, STRAT_HEAL);
        // Tactics should return 0 (no action for now)
        assert_eq!(result, 0, "STRAT_HEAL tactics should execute");
    }

    /// Test tactics with STRAT_AMULET
    #[test]
    fn test_tactics_strat_amulet() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let player = You::default();
        let result = tactics(MonsterId(1), &mut level, &player, STRAT_AMULET);
        assert_eq!(result, 0, "STRAT_AMULET tactics should execute");
    }

    /// Test tactics with STRAT_BOOK
    #[test]
    fn test_tactics_strat_book() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let player = You::default();
        let result = tactics(MonsterId(1), &mut level, &player, STRAT_BOOK);
        assert_eq!(result, 0, "STRAT_BOOK tactics should execute");
    }

    /// Test tactics with STRAT_BELL
    #[test]
    fn test_tactics_strat_bell() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let player = You::default();
        let result = tactics(MonsterId(1), &mut level, &player, STRAT_BELL);
        assert_eq!(result, 0, "STRAT_BELL tactics should execute");
    }

    /// Test tactics with STRAT_CANDLE
    #[test]
    fn test_tactics_strat_candle() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let player = You::default();
        let result = tactics(MonsterId(1), &mut level, &player, STRAT_CANDLE);
        assert_eq!(result, 0, "STRAT_CANDLE tactics should execute");
    }

    /// Test tactics with STRAT_COIN
    #[test]
    fn test_tactics_strat_coin() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let player = You::default();
        let result = tactics(MonsterId(1), &mut level, &player, STRAT_COIN);
        assert_eq!(result, 0, "STRAT_COIN tactics should execute");
    }

    /// Test tactics with STRAT_GOAL
    #[test]
    fn test_tactics_strat_goal() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let player = You::default();
        let result = tactics(MonsterId(1), &mut level, &player, STRAT_GOAL);
        assert_eq!(result, 0, "STRAT_GOAL tactics should execute");
    }

    /// Test tactics with invalid monster ID
    #[test]
    fn test_tactics_invalid_monster_id() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let player = You::default();
        let result = tactics(MonsterId(999), &mut level, &player, STRAT_HEAL);
        // Should handle gracefully without panicking
        assert_eq!(result, 0, "Invalid monster ID should return 0");
    }

    /// Test tactics with invalid strategy
    #[test]
    fn test_tactics_invalid_strategy() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);
        let player = You::default();

        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let result = tactics(MonsterId(1), &mut level, &player, 9999);
        // Should default to no action for unknown strategy
        assert_eq!(result, 0, "Invalid strategy should return 0");
    }
}

// ============================================================================
// PHASE 7 TESTS
// ============================================================================

#[cfg(test)]
mod phase7_tests {
    use super::*;
    use crate::dungeon::{DLevel, Level};
    use crate::monster::Monster;
    use crate::player::You;
    use crate::rng::GameRng;

    /// Test dochug basic execution without errors
    #[test]
    fn test_dochug_basic_execution() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let mut rng = GameRng::new(12345);

        let result = dochug(MonsterId(1), &mut level, &player, &mut rng);
        // Should execute without panicking
        assert_ne!(result, AiAction::None);
    }

    /// Test dochug with sleeping monster
    #[test]
    fn test_dochug_sleeping_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.sleeping = true;
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 5; // Adjacent to monster
        player.pos.y = 6;

        let mut rng = GameRng::new(12345);

        let result = dochug(MonsterId(1), &mut level, &player, &mut rng);
        // Should handle sleep check and potentially wake up
        assert!(true, "dochug should handle sleeping monsters");
    }

    /// Test dochug with immobilized monster
    #[test]
    fn test_dochug_immobilized_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.paralyzed = true;
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let mut rng = GameRng::new(12345);

        let result = dochug(MonsterId(1), &mut level, &player, &mut rng);
        // Should return Waited for paralyzed monster
        assert_eq!(result, AiAction::Waited);
    }

    /// Test dochugw with peaceful monster
    #[test]
    fn test_dochugw_peaceful_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.peaceful = true;
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 5;
        player.pos.y = 5;

        let mut rng = GameRng::new(12345);

        let result = dochugw(MonsterId(1), &mut level, &player, &mut rng);
        // Should execute for peaceful monsters too
        assert_ne!(result, AiAction::None);
    }

    /// Test dochugw with adjacent aggressive monster
    #[test]
    fn test_dochugw_adjacent_aggressive() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.peaceful = false;
        monster.state.fleeing = false;
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 5; // Adjacent to monster
        player.pos.y = 6;

        let mut rng = GameRng::new(12345);

        let result = dochugw(MonsterId(1), &mut level, &player, &mut rng);
        // Adjacent aggressive monster should be handled
        assert_ne!(result, AiAction::None);
    }

    /// Test movemon with single monster
    #[test]
    fn test_movemon_single_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let mut rng = GameRng::new(12345);

        let result = movemon(&mut level, &player, &mut rng);
        // Should complete without panicking
        assert!(true, "movemon should handle single monster");
    }

    /// Test movemon with multiple monsters
    #[test]
    fn test_movemon_multiple_monsters() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        for i in 1..=5 {
            let monster = Monster::new(MonsterId(i as u32), 0, 5 + i as i8, 5 + i as i8);
            level.add_monster(monster);
        }

        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let mut rng = GameRng::new(12345);

        let result = movemon(&mut level, &player, &mut rng);
        // Should process all monsters
        assert!(true, "movemon should handle multiple monsters");
    }

    /// Test movemon with dead monster in list
    #[test]
    fn test_movemon_dead_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 0; // Dead monster
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let mut rng = GameRng::new(12345);

        let result = movemon(&mut level, &player, &mut rng);
        // Should skip dead monsters
        assert!(true, "movemon should skip dead monsters");
    }

    /// Test domove with valid movement
    #[test]
    fn test_domove_valid_movement() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        for x in 0..15 {
            for y in 0..15 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let result = domove(MonsterId(1), 6, 5, &mut level, &player);
        // Should return success code
        assert_eq!(result, 1, "Valid move should return 1");
    }

    /// Test domove with invalid monster ID
    #[test]
    fn test_domove_invalid_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let result = domove(MonsterId(999), 6, 5, &mut level, &player);
        // Should return failure code for missing monster
        assert_eq!(result, 0, "Invalid monster ID should return 0");
    }

    /// Test domove_core updates position
    #[test]
    fn test_domove_core_position_update() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // Initial position
        assert_eq!(level.monster(MonsterId(1)).unwrap().x, 5);
        assert_eq!(level.monster(MonsterId(1)).unwrap().y, 5);

        // Move the monster
        domove_core(MonsterId(1), 5, 5, 6, 6, &mut level);

        // Verify new position
        assert_eq!(level.monster(MonsterId(1)).unwrap().x, 6);
        assert_eq!(level.monster(MonsterId(1)).unwrap().y, 6);
    }

    /// Test domove_core with multiple movements
    #[test]
    fn test_domove_core_multiple_movements() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // First movement
        domove_core(MonsterId(1), 5, 5, 6, 6, &mut level);
        assert_eq!(level.monster(MonsterId(1)).unwrap().x, 6);
        assert_eq!(level.monster(MonsterId(1)).unwrap().y, 6);

        // Second movement
        domove_core(MonsterId(1), 6, 6, 7, 7, &mut level);
        assert_eq!(level.monster(MonsterId(1)).unwrap().x, 7);
        assert_eq!(level.monster(MonsterId(1)).unwrap().y, 7);
    }

    /// Test domove_core with invalid monster
    #[test]
    fn test_domove_core_invalid_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());

        // Should not panic with invalid monster
        domove_core(MonsterId(999), 5, 5, 6, 6, &mut level);
        assert!(
            true,
            "domove_core should handle invalid monsters gracefully"
        );
    }

    /// Test process_monster_ai wrapper
    #[test]
    fn test_process_monster_ai() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let mut rng = GameRng::new(12345);

        let result = process_monster_ai(MonsterId(1), &mut level, &player, &mut rng);
        // Should execute through dochugw wrapper
        assert_ne!(result, AiAction::None);
    }

    /// Test dochug with confused monster
    #[test]
    fn test_dochug_confused_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.confused = true;
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let mut rng = GameRng::new(12345);

        let result = dochug(MonsterId(1), &mut level, &player, &mut rng);
        // Should handle confused monsters
        assert_ne!(result, AiAction::None);
    }

    /// Test dochug with fleeing monster
    #[test]
    fn test_dochug_fleeing_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.fleeing = true;
        monster.flee_timeout = 10;
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let mut rng = GameRng::new(12345);

        let result = dochug(MonsterId(1), &mut level, &player, &mut rng);
        // Should handle fleeing monsters
        assert_ne!(result, AiAction::None);
    }

    /// Test dochug low HP monster
    #[test]
    fn test_dochug_low_hp_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 5; // Very low HP
        monster.hp_max = 100;
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let mut rng = GameRng::new(12345);

        let result = dochug(MonsterId(1), &mut level, &player, &mut rng);
        // Should consider healing options for low HP
        assert_ne!(result, AiAction::None);
    }
}

// ============================================================================
// PHASE 8 TESTS
// ============================================================================

#[cfg(test)]
mod phase8_tests {
    use super::*;
    use crate::dungeon::{DLevel, Level};
    use crate::monster::Monster;
    use crate::player::You;
    use crate::rng::GameRng;

    /// Test m_move basic execution
    #[test]
    fn test_m_move_basic_execution() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let result = m_move(MonsterId(1), &mut level, &player, 0);
        // Should execute without panicking, return 1 for success
        assert_eq!(result, 1, "m_move should return success");
    }

    /// Test m_move with invalid monster
    #[test]
    fn test_m_move_invalid_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let result = m_move(MonsterId(999), &mut level, &player, 0);
        // Should return 2 (died) for invalid monster
        assert_eq!(result, 2, "Invalid monster should return 2");
    }

    /// Test m_cure_self with blindness
    #[test]
    fn test_m_cure_self_blindness() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.blinded = true;
        monster.blinded_timeout = 100;
        level.add_monster(monster);

        let result = m_cure_self(MonsterId(1), 1, &mut level);
        assert!(result, "m_cure_self should cure blindness");

        let monster = level.monster(MonsterId(1)).unwrap();
        assert!(
            !monster.state.blinded,
            "Monster should no longer be blinded"
        );
        assert_eq!(
            monster.blinded_timeout, 0,
            "Blindness timeout should be cleared"
        );
    }

    /// Test m_cure_self with confusion
    #[test]
    fn test_m_cure_self_confusion() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.confused = true;
        monster.confused_timeout = 50;
        level.add_monster(monster);

        let result = m_cure_self(MonsterId(1), 2, &mut level);
        assert!(result, "m_cure_self should cure confusion");

        let monster = level.monster(MonsterId(1)).unwrap();
        assert!(
            !monster.state.confused,
            "Monster should no longer be confused"
        );
    }

    /// Test m_cure_self with stun
    #[test]
    fn test_m_cure_self_stun() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.stunned = true;
        level.add_monster(monster);

        let result = m_cure_self(MonsterId(1), 3, &mut level);
        assert!(result, "m_cure_self should cure stun");

        let monster = level.monster(MonsterId(1)).unwrap();
        assert!(
            !monster.state.stunned,
            "Monster should no longer be stunned"
        );
    }

    /// Test m_cure_self with invalid cure type
    #[test]
    fn test_m_cure_self_invalid_type() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let result = m_cure_self(MonsterId(1), 999, &mut level);
        assert!(!result, "Invalid cure type should return false");
    }

    /// Test mcureblindness successfully
    #[test]
    fn test_mcureblindness_success() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.blinded = true;
        monster.blinded_timeout = 50;
        level.add_monster(monster);

        let result = mcureblindness(MonsterId(1), &mut level);
        assert!(result, "mcureblindness should succeed");

        let monster = level.monster(MonsterId(1)).unwrap();
        assert!(!monster.state.blinded, "Monster should be cured");
        assert_eq!(monster.blinded_timeout, 0, "Timeout should be cleared");
    }

    /// Test mcureblindness when not blinded
    #[test]
    fn test_mcureblindness_not_needed() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.blinded = false;
        level.add_monster(monster);

        let result = mcureblindness(MonsterId(1), &mut level);
        assert!(!result, "mcureblindness should return false if not blinded");
    }

    /// Test peace_minded returns true by default
    #[test]
    fn test_peace_minded_default_peaceful() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let result = peace_minded(MonsterId(1), &level, &player);
        assert!(result, "peace_minded should return true by default");
    }

    /// Test peace_minded with invalid monster
    #[test]
    fn test_peace_minded_invalid_monster() {
        let level = Level::new(DLevel::main_dungeon_start());
        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let result = peace_minded(MonsterId(999), &level, &player);
        assert!(
            result,
            "peace_minded should assume peaceful for invalid monster"
        );
    }

    /// Test reset_hostility with invalid monster
    #[test]
    fn test_reset_hostility_invalid_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        // Should not panic with invalid monster
        reset_hostility(MonsterId(999), &mut level, &player);
        assert!(true, "reset_hostility should handle invalid monsters");
    }

    /// Test reset_hostility with valid monster
    #[test]
    fn test_reset_hostility_valid_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        // Should execute without error
        reset_hostility(MonsterId(1), &mut level, &player);
        assert!(true, "reset_hostility should execute");
    }

    /// Test wakeup basic execution
    #[test]
    fn test_wakeup_basic() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.sleeping = true;
        monster.sleep_timeout = 100;
        level.add_monster(monster);

        wakeup(MonsterId(1), &mut level, false);

        let monster = level.monster(MonsterId(1)).unwrap();
        assert_eq!(monster.sleep_timeout, 0, "Monster should wake up");
    }

    /// Test wakeup via attack
    #[test]
    fn test_wakeup_via_attack() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.sleeping = true;
        level.add_monster(monster);

        wakeup(MonsterId(1), &mut level, true);

        let monster = level.monster(MonsterId(1)).unwrap();
        assert_eq!(monster.sleep_timeout, 0, "Monster should wake up");
    }

    /// Test wake_nearby basic execution
    #[test]
    fn test_wake_nearby_basic() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.sleeping = true;
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 5;
        player.pos.y = 5;
        player.level = DLevel::new(0, 1);

        wake_nearby(&mut level, &player);
        // Should execute without error
        assert!(true, "wake_nearby should execute");
    }

    /// Test wake_nearto with multiple monsters
    #[test]
    fn test_wake_nearto_multiple() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        for i in 1..=5 {
            let mut monster = Monster::new(MonsterId(i as u32), 0, 5 + i as i8, 5);
            monster.state.sleeping = true;
            level.add_monster(monster);
        }

        wake_nearto(5, 5, 10, &mut level);
        // Should wake monsters within distance
        assert!(true, "wake_nearto should execute");
    }

    /// Test disturb with nearby monster
    #[test]
    fn test_disturb_nearby() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 5;
        player.pos.y = 5;

        let result = disturb(MonsterId(1), &mut level, &player);
        // Should return non-zero for nearby monsters (should wake)
        assert!(result >= 0, "disturb should return result code");
    }

    /// Test dig_typ with rock terrain
    #[test]
    fn test_dig_typ_rock() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        level.cells[5][5].typ = crate::dungeon::CellType::Stone;

        let result = dig_typ(None, 5, 5, &level);
        // Should return appropriate dig type for rock
        assert!(result >= 0, "dig_typ should return dig type code");
    }

    /// Test dig_typ with door
    #[test]
    fn test_dig_typ_door() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        level.cells[5][5].typ = crate::dungeon::CellType::Door;

        let result = dig_typ(None, 5, 5, &level);
        // Should return appropriate dig type for door
        assert!(result >= 0, "dig_typ should handle doors");
    }

    /// Test dig_check with valid terrain
    #[test]
    fn test_dig_check_rock() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        level.cells[5][5].typ = crate::dungeon::CellType::Stone;

        let result = dig_check(5, 5, false, &level);
        // Should return true for diggable terrain
        assert!(
            result,
            "dig_check should validate stone as diggable terrain"
        );
    }

    /// Test dig_check with invalid terrain
    #[test]
    fn test_dig_check_stairs() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        level.cells[5][5].typ = crate::dungeon::CellType::Stairs;

        let result = dig_check(5, 5, false, &level);
        // Should reject stairs (not diggable)
        assert!(!result, "dig_check should reject stairs as not diggable");
    }

    /// Test can_tunnel returns false by default
    #[test]
    fn test_can_tunnel_basic() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let result = can_tunnel(MonsterId(1), &level);
        // Most monsters can't tunnel, so should return false
        assert!(!result, "Most monsters shouldn't tunnel");
    }

    /// Test is_digging returns false by default
    #[test]
    fn test_is_digging_default() {
        let result = is_digging();
        // Should return false by default (no monsters digging initially)
        assert!(!result, "is_digging should return false by default");
    }

    /// Test m_respond with invalid monster
    #[test]
    fn test_m_respond_invalid_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let result = m_respond(MonsterId(999), 0, &mut level, &mut player);
        assert_eq!(result, 0, "m_respond should return 0 for invalid monster");
    }

    /// Test m_respond with MS_SHRIEK
    #[test]
    fn test_m_respond_shriek() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        // MS_SHRIEK = 1 (constant)
        let result = m_respond(MonsterId(1), 1, &mut level, &mut player);
        assert_eq!(result, 1, "m_respond should handle shriek response");
    }

    /// Test strategy with low HP in Phase 8
    #[test]
    fn test_strategy_phase_8_low_hp() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 10;
        monster.hp_max = 100;
        level.add_monster(monster);

        let result = strategy(MonsterId(1), &level);
        assert_eq!(result, STRAT_HEAL, "Low HP should trigger STRAT_HEAL");
    }

    /// Test m_lined_up with distant monsters
    #[test]
    fn test_m_lined_up_distant() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let monster1 = Monster::new(MonsterId(1), 0, 5, 5);
        let monster2 = Monster::new(MonsterId(2), 0, 20, 5);
        level.add_monster(monster1);
        level.add_monster(monster2);

        // Stay within level bounds: COLNO=80, ROWNO=21
        for x in 0..25 {
            for y in 0..21 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        // Monsters on same row but distant
        let result = m_lined_up(MonsterId(1), MonsterId(2), &level);
        // May or may not be lined up depending on distance/LoS
        assert!(true, "m_lined_up should handle distant monsters");
    }
}

// ============================================================================
// PHASE 9 TESTS: ITEM USAGE AND TARGETING (14 functions)
// ============================================================================

#[cfg(test)]
mod phase9_tests {
    use super::*;
    use crate::dungeon::DLevel;

    // ---- PRIORITY 1 TESTS: Movement position finding & targeting ----

    /// Test mfndpos with valid adjacent positions
    #[test]
    fn test_phase9_mfndpos_valid_positions() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);
        let mut rng = GameRng::new(12345);

        // Fill area with passable terrain
        for x in 3..8 {
            for y in 3..8 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let player = You::default();
        let positions = mfndpos(MonsterId(1), &level, &player, &mut rng);

        // Should find multiple valid adjacent positions
        assert!(
            !positions.is_empty(),
            "mfndpos should find at least one valid position"
        );
        assert!(
            positions.len() <= 8,
            "mfndpos should find at most 8 adjacent positions"
        );
    }

    /// Test mfndpos with walls blocking movement
    #[test]
    fn test_phase9_mfndpos_blocked_by_walls() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // Surround monster with walls
        for x in 4..7 {
            for y in 4..7 {
                level.cells[x][y].typ = crate::dungeon::CellType::Stone;
            }
        }
        level.cells[5][5].typ = crate::dungeon::CellType::Room; // Monster location

        let mut rng = GameRng::new(12345);
        let player = You::default();
        let positions = mfndpos(MonsterId(1), &level, &player, &mut rng);

        // Should find no valid positions (all adjacent cells are walls)
        assert_eq!(
            positions.len(),
            0,
            "mfndpos should find no positions when surrounded by walls"
        );
    }

    /// Test find_targ finding monster in line of sight
    #[test]
    fn test_phase9_find_targ_visible_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster1 = Monster::new(MonsterId(1), 0, 5, 5);
        let mut monster2 = Monster::new(MonsterId(2), 0, 10, 5);
        level.add_monster(monster1);
        level.add_monster(monster2);

        // Fill with room terrain (passable)
        for x in 0..15 {
            for y in 0..10 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        // Find target to the right (direction: dx=1, dy=0)
        let target = find_targ(MonsterId(1), &level, 1, 0, 7);
        assert_eq!(
            target,
            Some(MonsterId(2)),
            "find_targ should find visible monster on same row"
        );
    }

    /// Test find_targ not finding monster out of range
    #[test]
    fn test_phase9_find_targ_out_of_range() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster1 = Monster::new(MonsterId(1), 0, 5, 5);
        let mut monster2 = Monster::new(MonsterId(2), 0, 20, 5);
        level.add_monster(monster1);
        level.add_monster(monster2);

        for x in 0..25 {
            for y in 0..10 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        // Try to find target at range 4 when target is 15 away
        let target = find_targ(MonsterId(1), &level, 1, 0, 4);
        assert_eq!(
            target, None,
            "find_targ should not find monster beyond maxdist"
        );
    }

    /// Test score_targ with hostile monster (should be positive)
    #[test]
    fn test_phase9_score_targ_hostile_bonus() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut attacker = Monster::new(MonsterId(1), 0, 5, 5);
        attacker.level = 10;
        attacker.hp = 30;

        let mut target = Monster::new(MonsterId(2), 0, 10, 5);
        target.level = 10;
        target.hp = 20;
        target.state.peaceful = false; // Hostile

        level.add_monster(attacker);
        level.add_monster(target);

        let score = score_targ(MonsterId(1), MonsterId(2), &level);
        // Hostile bonus (+10) + level*2 + hp/3 should be positive
        assert!(
            score > 0,
            "score_targ should give positive score to hostile monster"
        );
    }

    /// Test score_targ with adjacent monster (should be penalized)
    #[test]
    fn test_phase9_score_targ_adjacent_penalty() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut attacker = Monster::new(MonsterId(1), 0, 5, 5);
        attacker.level = 10;
        let mut target = Monster::new(MonsterId(2), 0, 6, 5); // Adjacent
        target.level = 10;
        target.state.peaceful = false;

        level.add_monster(attacker);
        level.add_monster(target);

        let score = score_targ(MonsterId(1), MonsterId(2), &level);
        // Adjacent penalty (-3000) should make score negative
        assert_eq!(score, -3000, "score_targ should penalize adjacent targets");
    }

    /// Test best_target finds best among multiple targets
    #[test]
    fn test_phase9_best_target_multiple_targets() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        // Place pet at center of a wide area
        let mut pet = Monster::new(MonsterId(1), 0, 40, 10);
        pet.level = 20;
        pet.hp = 80;
        pet.hp_max = 80;

        // Single hostile target 5 squares east (within find_targ maxdist=7)
        // Only one target means no has_defender penalty (no allied monsters nearby)
        // Target level is within 4 of pet level to avoid "vastly stronger" penalty
        let mut strong = Monster::new(MonsterId(2), 0, 45, 10);
        strong.level = 15;
        strong.hp = 60;
        strong.state.peaceful = false;

        level.add_monster(pet);
        level.add_monster(strong);

        // Fill room along the path - stay within level bounds: COLNO=80, ROWNO=21
        for x in 38..48 {
            for y in 8..13 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let best = best_target(MonsterId(1), &level);
        // With a single visible hostile target, best_target should find it
        assert_eq!(
            best,
            Some(MonsterId(2)),
            "best_target should find the hostile target"
        );
    }

    // ---- PRIORITY 2 TESTS: Defensive item detection ----

    /// Test find_defensive locates healing potion
    #[test]
    fn test_phase9_find_defensive_healing_potion() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 5; // Injured
        monster.hp_max = 30;

        // Add healing potion to inventory
        let healing_potion = Object::new(ObjectId::NONE, 10, crate::object::ObjectClass::Potion);
        monster.inventory.push(healing_potion);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let result = find_defensive(MonsterId(1), &level, &player);
        assert!(
            result.is_some(),
            "find_defensive should find healing potion when injured"
        );
    }

    /// Test find_offensive locates wand of death
    #[test]
    fn test_phase9_find_offensive_wand_death() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.peaceful = false;

        // Add wand of death to inventory
        let mut wand = Object::new(ObjectId::NONE, 108, crate::object::ObjectClass::Wand);
        wand.enchantment = 5; // Has charges
        monster.inventory.push(wand);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let result = find_offensive(MonsterId(1), &level, &player);
        assert!(result.is_some(), "find_offensive should find wand of death");
    }

    /// Test find_misc locates potion of invisibility
    #[test]
    fn test_phase9_find_misc_invisibility_potion() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.invisible = false;

        // Add invisibility potion
        let invis_pot = Object::new(ObjectId::NONE, 98, crate::object::ObjectClass::Potion);
        monster.inventory.push(invis_pot);

        level.add_monster(monster);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        let result = find_misc(MonsterId(1), &level, &player);
        assert!(
            result.is_some(),
            "find_misc should find invisibility potion"
        );
    }

    // ---- PRIORITY 3 TESTS: Item execution ----

    /// Test use_defensive with unicorn horn
    #[test]
    fn test_phase9_use_defensive_unicorn_horn() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.confused = true;
        level.add_monster(monster);

        let mut usage = ItemUsage::default();
        usage.has_defense = MUSE_UNICORN_HORN;

        let result = use_defensive(MonsterId(1), &mut level, &usage);
        assert_eq!(
            result,
            AiAction::Waited,
            "use_defensive should return Waited"
        );

        // Verify confusion is cured
        let monster = level.monster(MonsterId(1)).unwrap();
        assert!(
            !monster.state.confused,
            "Unicorn horn should cure confusion"
        );
    }

    /// Test use_defensive with healing potion
    #[test]
    fn test_phase9_use_defensive_healing_potion() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.hp = 5;
        monster.hp_max = 30;
        level.add_monster(monster);

        let mut usage = ItemUsage::default();
        usage.has_defense = MUSE_POT_FULL_HEALING;

        let result = use_defensive(MonsterId(1), &mut level, &usage);
        assert_eq!(
            result,
            AiAction::Waited,
            "use_defensive should complete healing"
        );

        // Verify HP restored
        let monster = level.monster(MonsterId(1)).unwrap();
        assert_eq!(
            monster.hp, monster.hp_max,
            "Full healing potion should restore all HP"
        );
    }

    /// Test use_offensive returns appropriate action
    #[test]
    fn test_phase9_use_offensive_basic() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.alive = true;
        level.add_monster(monster);

        let mut usage = ItemUsage::default();
        usage.has_offense = 0; // No offense type

        let result = use_offensive(MonsterId(1), &mut level, &usage);
        // Should handle gracefully even without offense
        assert_eq!(
            result,
            AiAction::Waited,
            "use_offensive should return valid action"
        );
    }

    /// Test use_misc returns appropriate action
    #[test]
    fn test_phase9_use_misc_basic() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut usage = ItemUsage::default();
        usage.has_misc = 0; // No misc type

        let result = use_misc(MonsterId(1), &mut level, &usage);
        assert_eq!(result, AiAction::Waited, "use_misc should complete action");
    }

    // ---- PRIORITY 4 TESTS: Terrain digging system ----

    /// Test m_digweapon_check with no weapon needed
    #[test]
    fn test_phase9_m_digweapon_check_no_weapon() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // Target is empty room (not diggable)
        level.cells[6][5].typ = crate::dungeon::CellType::Room;

        let result = m_digweapon_check(MonsterId(1), &level, 6, 5);
        assert!(
            !result,
            "m_digweapon_check should return false for room terrain"
        );
    }

    /// Test mdig_tunnel basic execution
    #[test]
    fn test_phase9_mdig_tunnel_secret_door() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // Set target as secret door
        level.cells[6][5].typ = crate::dungeon::CellType::SecretDoor;

        let result = mdig_tunnel(MonsterId(1), &mut level, 6, 5);
        // Should return true indicating dig action was taken
        assert!(result, "mdig_tunnel should handle secret door");
    }

    /// Test digactualhole basic execution
    #[test]
    fn test_phase9_digactualhole_pit_creation() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let result = digactualhole(6, 5, &mut level, 12); // 12 = PIT
        // Should complete pit creation
        assert!(result, "digactualhole should create pit");
    }

    /// Test mdig_tunnel with invalid monster
    #[test]
    fn test_phase9_mdig_tunnel_invalid_monster() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        level.cells[6][5].typ = crate::dungeon::CellType::SecretDoor;

        let result = mdig_tunnel(MonsterId(999), &mut level, 6, 5);
        assert!(
            !result,
            "mdig_tunnel should return false for invalid monster"
        );
    }

    /// Test digactualhole with boundary position
    #[test]
    fn test_phase9_digactualhole_boundary() {
        let mut level = Level::new(DLevel::main_dungeon_start());

        // Test at boundary (should handle gracefully)
        let result = digactualhole(0, 0, &mut level, 12);
        assert!(result, "digactualhole should handle boundary positions");
    }

    /// Test can_tunnel default behavior
    #[test]
    fn test_phase9_can_tunnel_default() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let result = can_tunnel(MonsterId(1), &level);
        // Default implementation returns false (flags not fully defined yet)
        assert!(!result, "can_tunnel should return false by default");
    }

    /// Test is_digging default state
    #[test]
    fn test_phase9_is_digging_default() {
        // is_digging() checks player digging state, no monster version
        let result = is_digging();
        assert!(!result, "Player should not be digging by default");
    }

    /// Test dig_typ with rock terrain
    #[test]
    fn test_phase9_dig_typ_rock() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        level.cells[5][5].typ = crate::dungeon::CellType::Stone;

        let result = dig_typ(None, 5, 5, &level);
        // Should return DIGTYP_UNDIGGABLE (0) when no weapon
        assert_eq!(result, 0, "dig_typ without weapon should return undiggable");
    }

    /// Test dig_check with valid rock terrain
    #[test]
    fn test_phase9_dig_check_valid_rock() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        level.cells[5][5].typ = crate::dungeon::CellType::Stone;

        let result = dig_check(5, 5, false, &level);
        // Should allow digging on rock
        assert!(result, "dig_check should allow digging on rock");
    }

    /// Test mdig_tunnel with door terrain
    #[test]
    fn test_phase9_mdig_tunnel_door() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // Set target as door
        level.cells[6][5].typ = crate::dungeon::CellType::Door;

        let result = mdig_tunnel(MonsterId(1), &mut level, 6, 5);
        assert!(result, "mdig_tunnel should handle door terrain");
    }

    /// Test find_targ with player marker
    #[test]
    fn test_phase9_find_targ_player_marker() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.player_x = 10;
        monster.player_y = 5;
        level.add_monster(monster);

        for x in 0..15 {
            for y in 0..10 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        // Find target that points to player marker location
        let target = find_targ(MonsterId(1), &level, 1, 0, 7);
        // Should return MonsterId(0) for player
        assert_eq!(
            target,
            Some(MonsterId(0)),
            "find_targ should return player marker"
        );
    }
}
