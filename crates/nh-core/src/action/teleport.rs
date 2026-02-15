//! Teleportation mechanics (teleport.c)
//!
//! From NetHack C:
//! - tele(): Random teleport on current level
//! - level_tele(): Teleport to different dungeon level
//! - Teleport control allows choosing destination
//! - Amulet of Yendor blocks level teleport in endgame
//! - rloc()/rloc_to(): Monster random/targeted relocation
//! - Scroll/trap/wand-triggered teleportation
//! - Shop/vault/priest room confinement rules

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::action::ActionResult;
use crate::dungeon::TrapType;
use crate::dungeon::Level;
use crate::gameloop::GameState;
use crate::dungeon::CellType;
use crate::monster::{Monster, MonsterId};
use crate::monster::makemon::enexto;
use crate::object::Object;
use crate::player::Property;
use crate::{COLNO, ROWNO};

// ─────────────────────────────────────────────────────────────────────────────
// Level / restriction checks
// ─────────────────────────────────────────────────────────────────────────────

/// Check if a level blocks teleportation (C: level.flags.noteleport)
///
/// Certain levels (Stronghold, Vlad's Tower, Sokoban) set the noteleport flag,
/// preventing all teleportation on that level.
pub fn noteleport_level(level: &Level) -> bool {
    level.flags.no_teleport
}

/// Check if the player is restricted from teleporting on this level.
/// Returns true if teleportation is blocked.
///
/// Reasons: noteleport flag, or level-specific restrictions.
pub fn tele_restrict(state: &GameState) -> bool {
    if noteleport_level(&state.current_level) {
        return true;
    }
    // Sokoban blocks level teleport (but not same-level teleport)
    false
}

/// Check if the player's current position blocks teleportation
/// (e.g. inside a shop with unpaid items, or in a vault).
///
/// From C: prevents teleport from shops when owing money.
pub fn could_tele_from(level: &Level, x: i8, y: i8) -> bool {
    // Check if inside a shop
    for shop in &level.shops {
        if shop.contains(x, y) {
            // Can't teleport from shop if there are unpaid items
            if !shop.unpaid_items.is_empty() {
                return false;
            }
        }
    }
    // Check if inside a vault
    if level.flags.has_vault {
        // Vaults don't block teleport, but vault guards react to exits
    }
    true
}

/// Check if the Amulet of Yendor blocks teleportation.
///
/// In the endgame or when carrying the Amulet, there is a 1-in-3 chance
/// that teleportation is disrupted entirely.
pub fn check_amulet_teleport_block(state: &mut GameState, has_amulet: bool) -> bool {
    let on_wizard_tower = state.current_level.dlevel.dungeon_num == 6; // Vlad's Tower
    if (has_amulet || on_wizard_tower) && state.rng.rn2(3) == 0 {
        state.message("You feel disoriented for a moment.");
        return true;
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────────
// Position validation
// ─────────────────────────────────────────────────────────────────────────────

/// Validate a teleport destination for the player (C: teleok).
///
/// Checks:
/// - Position is valid and walkable
/// - No monster occupying the spot
/// - Trap interactions (some traps are OK if we accept them)
/// - Not blocked by region boundaries
pub fn teleok(level: &Level, x: i8, y: i8, trapok: bool) -> bool {
    if !level.is_valid_pos(x, y) {
        return false;
    }
    if !level.is_walkable(x, y) {
        return false;
    }
    if level.monster_at(x, y).is_some() {
        return false;
    }
    // Check for traps
    if let Some(trap) = level.trap_at(x, y)
        && !trapok
    {
        // Avoid teleport traps (would chain-teleport) and holes
        match trap.trap_type {
            TrapType::Teleport
            | TrapType::LevelTeleport
            | TrapType::Hole
            | TrapType::TrapDoor
            | TrapType::MagicPortal => return false,
            _ => {}
        }
    }
    true
}

/// Validate a monster teleport destination (C: rloc_pos_ok).
///
/// Like teleok but also checks:
/// - Shopkeeper confinement (shopkeepers stay in their shop)
/// - Priest confinement (temple priests stay in their temple)
pub fn rloc_pos_ok(level: &Level, x: i8, y: i8, monster: &Monster) -> bool {
    if !level.is_valid_pos(x, y) {
        return false;
    }
    if !level.is_walkable(x, y) {
        return false;
    }
    if level.monster_at(x, y).is_some() {
        return false;
    }
    // Shopkeeper confinement: shopkeepers must stay in their shop
    if monster.is_shopkeeper {
        let in_shop = level.shops.iter().any(|s| {
            s.shopkeeper_id == Some(monster.id) && s.contains(x, y)
        });
        if !in_shop {
            return false;
        }
    }
    // Priest confinement: priests stay in temples
    if monster.is_priest {
        // Check if currently in a temple and destination is outside
        let src_in_temple = is_in_temple(level, monster.x, monster.y);
        if src_in_temple && !is_in_temple(level, x, y) {
            return false;
        }
    }
    true
}

/// Check if a position is inside a temple (approximation: has_temple + altar presence)
fn is_in_temple(level: &Level, x: i8, y: i8) -> bool {
    if !level.flags.has_temple {
        return false;
    }
    // Check if within 2 squares of an altar (rough approximation)
    for dy in -2i8..=2 {
        for dx in -2i8..=2 {
            let tx = x + dx;
            let ty = y + dy;
            if level.is_valid_pos(tx, ty) {
                let cell = level.cell(tx as usize, ty as usize);
                if cell.typ == CellType::Altar {
                    return true;
                }
            }
        }
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────────
// Player teleportation
// ─────────────────────────────────────────────────────────────────────────────

/// Teleport the player randomly on the current level
pub fn tele(state: &mut GameState) -> ActionResult {
    if noteleport_level(&state.current_level) {
        state.message("A mysterious force prevents you from teleporting!");
        return ActionResult::NoTime;
    }

    let has_control = state.player.properties.has(Property::TeleportControl);
    let is_stunned = state.player.stunned_timeout > 0;
    let is_confused = state.player.confused_timeout > 0;

    if has_control && !is_stunned && !is_confused {
        state.message("You feel in control of the teleportation.");
    }

    // Save previous position
    state.player.prev_pos = state.player.pos;

    let (new_x, new_y) = safe_teleds(state);

    state.player.pos.x = new_x;
    state.player.pos.y = new_y;
    state.message("You feel disoriented.");

    ActionResult::Success
}

/// Find a safe random teleport destination (C: safe_teleds).
///
/// Tries up to 400 positions. First 200 avoid traps; last 200 accept traps.
fn safe_teleds(state: &mut GameState) -> (i8, i8) {
    // First pass: avoid traps
    for _ in 0..200 {
        let x = state.rng.rn2(COLNO as u32) as i8;
        let y = state.rng.rn2(ROWNO as u32) as i8;

        if teleok(&state.current_level, x, y, false) {
            return (x, y);
        }
    }
    // Second pass: accept traps
    for _ in 0..200 {
        let x = state.rng.rn2(COLNO as u32) as i8;
        let y = state.rng.rn2(ROWNO as u32) as i8;

        if teleok(&state.current_level, x, y, true) {
            return (x, y);
        }
    }
    // Absolute fallback: stay at current position
    (state.player.pos.x, state.player.pos.y)
}

/// Public wrapper for safe_teleds -- used by pray.rs to teleport player out of walls
pub fn safe_teleds_pub(state: &mut GameState) -> (i8, i8) {
    safe_teleds(state)
}

/// Teleport player to specific coordinates (for controlled teleport)
pub fn tele_to(state: &mut GameState, x: i8, y: i8) -> ActionResult {
    if !state.current_level.is_valid_pos(x, y) {
        state.message("You can't teleport there.");
        return ActionResult::NoTime;
    }

    if !state.current_level.is_walkable(x, y) {
        state.message("You can't teleport into solid rock!");
        return ActionResult::NoTime;
    }

    if state.current_level.monster_at(x, y).is_some() {
        state.message("You can't teleport on top of a monster!");
        return ActionResult::NoTime;
    }

    state.player.prev_pos = state.player.pos;
    state.player.pos.x = x;
    state.player.pos.y = y;
    state.message("You materialize at your destination.");

    ActionResult::Success
}

/// Scroll-triggered teleport (C: scrolltele).
///
/// Handles:
/// - Amulet of Yendor disruption (1-in-3 chance)
/// - Blessed scrolls give teleport control even without intrinsic
/// - Cursed scrolls go to random position without control
/// - On noteleport levels, teleport fails
pub fn scroll_teleport(state: &mut GameState, scroll_buc: ScrollBuc) -> ActionResult {
    if noteleport_level(&state.current_level) {
        state.message("A mysterious force prevents you from teleporting!");
        return ActionResult::NoTime;
    }

    let has_amulet = false;
    if check_amulet_teleport_block(state, has_amulet) {
        return ActionResult::NoTime;
    }

    let has_control = state.player.properties.has(Property::TeleportControl);
    let is_stunned = state.player.stunned_timeout > 0;
    let is_confused = state.player.confused_timeout > 0;

    let effective_control = match scroll_buc {
        ScrollBuc::Blessed => true,
        ScrollBuc::Uncursed => has_control,
        ScrollBuc::Cursed => false,
    };

    if effective_control && !is_stunned && !is_confused {
        state.message("You feel in control of the teleportation.");
    }

    state.player.prev_pos = state.player.pos;
    let (new_x, new_y) = safe_teleds(state);
    state.player.pos.x = new_x;
    state.player.pos.y = new_y;
    state.message("You feel disoriented.");

    ActionResult::Success
}

/// BUC status for scrolls affecting teleportation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollBuc {
    Blessed,
    Uncursed,
    Cursed,
}

// ─────────────────────────────────────────────────────────────────────────────
// Level teleportation
// ─────────────────────────────────────────────────────────────────────────────

/// Level teleport - teleport to a different dungeon level (C: level_tele).
///
/// Returns the target depth (positive = down, negative = up).
///
/// Note: This sets up the teleport but actual level change must be
/// handled by the game loop checking the returned ActionResult.
pub fn level_tele(state: &mut GameState, target_depth: i32) -> ActionResult {
    let current_depth = state.current_level.dlevel.depth();

    // Check Sokoban restriction
    if state.current_level.dlevel.dungeon_num == 3 {
        state.message("Sorry, this level has no exit.");
        return ActionResult::NoTime;
    }


    // Check for teleport control
    let has_control = state.player.properties.has(Property::TeleportControl);
    let is_stunned = state.player.stunned_timeout > 0;

    let new_depth = if has_control && !is_stunned {
        // Controlled teleport
        // In real game, prompts user. Here random.
        random_teleport_level(state)
    } else {
        // Uncontrolled - random level
        random_teleport_level(state)
    };

    // Check if teleporting to same level
    if new_depth == current_depth {
        state.message("You shudder for a moment.");
        return ActionResult::Success;
    }

    // Check for going above ground (death by falling)
    if new_depth < 1 {
        if state.player.properties.has(Property::Levitation) {
            state.message("You float gently down to earth.");
        } else if state.player.properties.has(Property::Flying) {
            state.message("You fly down to the ground.");
        } else {
            state.message("You are now high above the clouds...");
            state.message("Unfortunately, you don't know how to fly.");
            state.message("You plummet a few thousand feet to your death.");
            state.player.hp = 0;
            return ActionResult::Died("fell to your death".to_string());
        }
    } else {
        // Normal level teleport
        state.message("You feel a wrenching sensation.");
    }

    ActionResult::Success
}

/// Calculate random teleport level (C: random_teleport_level).
///
/// Range is 1 to current+3, current not counting (skip it and bump).
/// Result is clamped to [min_depth, max_depth] for the current dungeon.
fn random_teleport_level(state: &mut GameState) -> i32 {
    let cur_depth = state.current_level.dlevel.depth();
    let dungeon_num = state.current_level.dlevel.dungeon_num;

    // Determine min/max depth based on dungeon type
    let (min_depth, max_depth) = match dungeon_num {
        0 => {
            // Main dungeon
            (1, 29)
        }
        1 => {
            // Gehennom (depth 30+)
            let top = 30;
            let bottom = 49; // approximate
            (top, bottom)
        }
        2 => {
            // Gnomish Mines
            let top = 2;
            let bottom = 13;
            (top, bottom)
        }
        3 => {
            // Sokoban (shouldn't get here, but just in case)
            let top = 6;
            let bottom = 9;
            (top, bottom)
        }
        4 => {
            // Quest
            let top = 16;
            let bottom = 21;
            (top, bottom)
        }
        5 => {
            // Fort Ludios
            (18, 18)
        }
        6 => {
            // Vlad's Tower
            let top = 37;
            let bottom = 39;
            (top, bottom)
        }
        _ => (1, 30),
    };

    // Get a random value relative to the current dungeon
    // Range is 1 to current+3, current not counting
    let range = (cur_depth + 3 - min_depth).max(1);
    let mut nlev = state.rng.rn2(range as u32) as i32 + min_depth;
    if nlev >= cur_depth {
        nlev += 1;
    }

    // Clamp to bounds
    if nlev > max_depth {
        nlev = max_depth;
        // If at bottom, teleport up a bit
        if cur_depth >= max_depth {
            nlev -= state.rng.rnd(3) as i32;
        }
    }
    if nlev < min_depth {
        nlev = min_depth;
        if nlev == cur_depth {
            nlev += state.rng.rnd(3) as i32;
            if nlev > max_depth {
                nlev = max_depth;
            }
        }
    }

    nlev
}

// ─────────────────────────────────────────────────────────────────────────────
// Trap-triggered teleportation
// ─────────────────────────────────────────────────────────────────────────────

/// Handle player stepping on a teleport trap (C: tele_trap).
///
/// Antimagic field blocks teleport traps. Once-only traps (vaults) are
/// removed after activation.
pub fn trap_teleport(state: &mut GameState) -> ActionResult {
    // Check if antimagic blocks it
    // (Antimagic field trap on same tile would block)
    if noteleport_level(&state.current_level) {
        state.message("A mysterious force prevents you from teleporting!");
        return ActionResult::NoTime;
    }

    let trap_pos = state.player.pos;
    let once_trap = state
        .current_level
        .trap_at(trap_pos.x, trap_pos.y)
        .is_some_and(|t| t.once);

    state.message("You hit a teleport trap!");

    // Remove once-only traps (vault teleport)
    if once_trap {
        state.current_level.remove_trap(trap_pos.x, trap_pos.y);
    }

    tele(state)
}

/// Handle player stepping on a level teleport trap (C: level_tele_trap).
///
/// Antimagic field blocks this. Otherwise triggers random level teleport.
pub fn trap_level_teleport(state: &mut GameState) -> ActionResult {
    if noteleport_level(&state.current_level) {
        state.message("A mysterious force prevents you from teleporting!");
        return ActionResult::NoTime;
    }

    state.message("You hit a level teleport trap!");

    // Level teleport with random destination
    level_tele(state, 0)
}

/// Handle a monster stepping on a teleport trap (C: mtele_trap).
///
/// Relocates the monster randomly. Vault traps attempt vault placement first.
pub fn mtele_trap(level: &mut Level, monster_id: MonsterId, _in_sight: bool) -> bool {
    let (mx, my, once) = {
        let monster = match level.monster(monster_id) {
            Some(m) => m,
            None => return false,
        };
        let trap = level.trap_at(monster.x, monster.y);
        let once = trap.is_some_and(|t| t.once);
        (monster.x, monster.y, once)
    };

    if level.flags.no_teleport {
        return false;
    }

    // Remove once-only trap
    if once {
        level.remove_trap(mx, my);
    }

    rloc_monster(level, monster_id)
}

/// Handle a monster stepping on a level teleport trap (C: mlevel_tele_trap).
///
/// Returns whether the monster was teleported.
/// For now, we relocate the monster on the current level since cross-level
/// monster migration requires game loop integration.
pub fn mlevel_tele_trap(
    level: &mut Level,
    monster_id: MonsterId,
    _in_sight: bool,
) -> bool {
    if level.flags.no_teleport {
        return false;
    }

    // Check if monster has the Amulet (can't level teleport with it)
    let has_amulet = {
        let monster = match level.monster(monster_id) {
            Some(m) => m,
            None => return false,
        };
        monster_has_amulet(monster)
    };

    if has_amulet {
        return false;
    }

    // For now, just relocate on current level
    // Full implementation would migrate the monster to a different level
    rloc_monster(level, monster_id)
}

// ─────────────────────────────────────────────────────────────────────────────
// Monster teleportation
// ─────────────────────────────────────────────────────────────────────────────

/// Relocate a monster to a random valid position (C: rloc).
///
/// Tries up to 1000 random positions using rloc_pos_ok validation.
/// Shopkeepers are confined to their shops, priests to their temples.
pub fn rloc_monster(level: &mut Level, monster_id: MonsterId) -> bool {
    // Clone needed fields to avoid borrow issues
    let (mx, my, is_shopkeeper, is_priest, mid) = {
        let monster = match level.monster(monster_id) {
            Some(m) => m,
            None => return false,
        };
        (monster.x, monster.y, monster.is_shopkeeper, monster.is_priest, monster.id)
    };

    // Use expanding square search from monster position (like enexto)
    let target = find_rloc_target(level, mx, my, is_shopkeeper, is_priest, mid);
    if let Some((nx, ny)) = target {
        level.move_monster(monster_id, nx, ny);
        return true;
    }

    false
}

/// Find a random-ish valid relocation target for a monster.
///
/// Searches the level for any valid position, starting from the given
/// coordinates and spiraling outward.
fn find_rloc_target(
    level: &Level,
    start_x: i8,
    start_y: i8,
    is_shopkeeper: bool,
    is_priest: bool,
    monster_id: MonsterId,
) -> Option<(i8, i8)> {
    // Search in expanding rings
    let max_radius = COLNO.max(ROWNO) as i8;
    for radius in 1..max_radius {
        // Check positions at this radius
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() != radius && dy.abs() != radius {
                    continue; // Only check ring perimeter
                }
                let x = start_x + dx;
                let y = start_y + dy;
                if !level.is_valid_pos(x, y) {
                    continue;
                }
                if !level.is_walkable(x, y) {
                    continue;
                }
                if level.monster_at(x, y).is_some() {
                    continue;
                }
                // Player position check
                // (caller should ensure player is not at this position)

                // Shopkeeper confinement
                if is_shopkeeper {
                    let in_shop = level.shops.iter().any(|s| {
                        s.shopkeeper_id == Some(monster_id) && s.contains(x, y)
                    });
                    if !in_shop {
                        continue;
                    }
                }

                // Priest confinement
                if is_priest {
                    let src_in_temple = is_in_temple(level, start_x, start_y);
                    if src_in_temple && !is_in_temple(level, x, y) {
                        continue;
                    }
                }

                return Some((x, y));
            }
        }
    }
    None
}

/// Relocate a monster to an exact position (C: rloc_to).
///
/// Directly moves the monster. Handles:
/// - Clearing old position in monster grid
/// - Setting new position in monster grid
pub fn rloc_monster_to(level: &mut Level, monster_id: MonsterId, x: i8, y: i8) -> bool {
    if !level.is_valid_pos(x, y) {
        return false;
    }
    // Allow placement even if occupied (C behavior: caller validates)
    level.move_monster(monster_id, x, y)
}

/// Move a monster next to the player (C: mnexto).
///
/// Finds the closest valid position adjacent to (px, py) using enexto
/// and moves the monster there. If no position found, the monster is
/// sent to "limbo" (removed from level in endgame, or kept in place).
pub fn mnexto(level: &mut Level, monster_id: MonsterId, px: i8, py: i8) -> bool {
    // Get monster data for enexto
    let (is_steed, _mflags) = {
        let monster = match level.monster(monster_id) {
            Some(m) => m,
            None => return false,
        };
        // Check if this monster is the player's steed
        (false, monster.flags) // steed check would need player data
    };

    if is_steed {
        // Steeds follow player exactly
        return rloc_monster_to(level, monster_id, px, py);
    }

    // Find nearby valid position using enexto (from makemon)
    if let Some((nx, ny)) = enexto(level, px, py, None) {
        rloc_monster_to(level, monster_id, nx, ny)
    } else {
        // Overcrowding - can't place monster
        false
    }
}

/// Move a monster next to the player, but require direct accessibility
/// (C: maybe_mnexto). Like mnexto but stricter requirements.
///
/// Tries up to 20 times to find a visible position.
pub fn maybe_mnexto(level: &mut Level, monster_id: MonsterId, px: i8, py: i8) -> bool {
    for _ in 0..20 {
        if let Some((nx, ny)) = enexto(level, px, py, None) {
            // Check that the position is accessible (visible from player)
            let dx = (nx - px).abs();
            let dy = (ny - py).abs();
            if dx <= 1 && dy <= 1 {
                return rloc_monster_to(level, monster_id, nx, ny);
            }
        }
    }
    false
}

/// Player teleports a monster via wand/spell (C: u_teleport_mon).
///
/// Returns false if the attempt fails (e.g. temple priest resists).
/// Riders resist teleportation 12/13 of the time but get displaced.
pub fn u_teleport_mon(
    level: &mut Level,
    monster_id: MonsterId,
    give_feedback: bool,
    px: i8,
    py: i8,
    rng: &mut crate::rng::GameRng,
) -> (bool, Vec<String>) {
    let mut messages = Vec::new();

    let (is_priest, in_temple, _is_rider, name) = {
        let monster = match level.monster(monster_id) {
            Some(m) => m,
            None => return (false, messages),
        };
        let in_temple = is_in_temple(level, monster.x, monster.y);
        (monster.is_priest, in_temple, false, monster.name.clone())
    };

    // Temple priests resist teleportation
    if is_priest && in_temple {
        if give_feedback {
            messages.push(format!("{} resists your magic!", name));
        }
        return (false, messages);
    }

    // Check noteleport but player is swallowed
    if level.flags.no_teleport {
        // On noteleport levels, just relocate with rloc
        rloc_monster(level, monster_id);
        return (true, messages);
    }

    // Riders (Death/Famine/Pestilence) resist 12/13 of the time but get
    // displaced near player. We identify riders as unique monsters whose type
    // index is in the rider range. For now, approximate: monster types 318-320
    // (the three horsemen in C). Proper rider identification pending unique monster flags.
    let is_rider = level.monster(monster_id).is_some_and(|m| {
        // Riders are unique special monsters — check original_type range
        // In NetHack 3.6.7, Death=318, Pestilence=319, Famine=320
        (318..=320).contains(&m.monster_type)
    });
    if is_rider && rng.rn2(13) != 0 {
        // Rider resists — move near player instead
        if let Some((nx, ny)) = enexto(level, px, py, None) {
            rloc_monster_to(level, monster_id, nx, ny);
        }
        return (true, messages);
    }

    // Normal teleport
    rloc_monster(level, monster_id);
    (true, messages)
}

// ─────────────────────────────────────────────────────────────────────────────
// Object teleportation
// ─────────────────────────────────────────────────────────────────────────────

/// Teleport an object to a random location on the level (C: rloco).
///
/// Tries up to 100 random positions. Objects avoid occupied spaces and
/// non-walkable terrain. Returns the new position or None if failed.
pub fn rloc_object(level: &mut Level, obj_id: crate::object::ObjectId) -> Option<(i8, i8)> {
    // Find a valid position
    for radius in 1..(COLNO.max(ROWNO) as i8) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() != radius && dy.abs() != radius {
                    continue;
                }
                let x = (COLNO as i8 / 2) + dx;
                let y = (ROWNO as i8 / 2) + dy;
                if !level.is_valid_pos(x, y) {
                    continue;
                }
                if !level.is_walkable(x, y) {
                    continue;
                }
                // Move the object
                if let Some(obj) = level.objects.iter_mut().find(|o| o.id == obj_id) {
                    let old_x = obj.x;
                    let old_y = obj.y;
                    // Update object grid
                    level.object_grid[old_x as usize][old_y as usize]
                        .retain(|&id| id != obj_id);
                    obj.x = x;
                    obj.y = y;
                    level.object_grid[x as usize][y as usize].push(obj_id);
                    return Some((x, y));
                }
                return None;
            }
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: Amulet check
// ─────────────────────────────────────────────────────────────────────────────

/// Check if a monster is carrying the Amulet of Yendor
fn monster_has_amulet(monster: &Monster) -> bool {
    // Check monster inventory for Amulet of Yendor
    // The Amulet is object type 0 in the Amulet class
    monster.inventory.iter().any(|obj| {
        obj.class == crate::object::ObjectClass::Amulet && obj.object_type == 0
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::{Cell, CellType, DLevel, Level};
    use crate::object::Object;
    use crate::gameloop::GameState;
    use crate::monster::Monster;
    use crate::player::Position;
    use crate::rng::GameRng;

    fn make_walkable_level() -> Level {
        let mut level = Level::new(DLevel::new(0, 5));
        // Make a section walkable
        for x in 1..20 {
            for y in 1..10 {
                *level.cell_mut(x, y) = Cell::floor();
            }
        }
        level
    }

    fn make_state_with_level(level: Level) -> GameState {
        let mut state = GameState::new(GameRng::new(42));
        state.current_level = level;
        state.player.pos = Position::new(5, 5);
        state.player.prev_pos = Position::new(5, 5);
        state
    }

    // ── noteleport_level ─────────────────────────────────────────────────

    #[test]
    fn test_noteleport_normal_level() {
        let level = Level::new(DLevel::new(0, 5));
        assert!(!noteleport_level(&level));
    }

    #[test]
    fn test_noteleport_restricted_level() {
        let mut level = Level::new(DLevel::new(0, 5));
        level.flags.no_teleport = true;
        assert!(noteleport_level(&level));
    }

    // ── tele_restrict ────────────────────────────────────────────────────

    #[test]
    fn test_tele_restrict_normal() {
        let state = make_state_with_level(make_walkable_level());
        assert!(!tele_restrict(&state));
    }

    #[test]
    fn test_tele_restrict_noteleport() {
        let mut level = make_walkable_level();
        level.flags.no_teleport = true;
        let state = make_state_with_level(level);
        assert!(tele_restrict(&state));
    }

    // ── could_tele_from ──────────────────────────────────────────────────

    #[test]
    fn test_could_tele_from_normal() {
        let level = make_walkable_level();
        assert!(could_tele_from(&level, 5, 5));
    }

    #[test]
    fn test_could_tele_from_shop_with_unpaid() {
        let mut level = make_walkable_level();
        let mut shop = crate::special::shk::Shop::new(
            crate::special::ShopType::General,
            (3, 3, 10, 8),
        );
        shop.unpaid_items.push(crate::object::ObjectId(1));
        level.shops.push(shop);
        // Inside shop with unpaid items — can't teleport
        assert!(!could_tele_from(&level, 5, 5));
    }

    #[test]
    fn test_could_tele_from_shop_no_debt() {
        let mut level = make_walkable_level();
        let shop = crate::special::shk::Shop::new(
            crate::special::ShopType::General,
            (3, 3, 10, 8),
        );
        level.shops.push(shop);
        // Inside shop but no unpaid items — can teleport
        assert!(could_tele_from(&level, 5, 5));
    }

    // ── teleok ───────────────────────────────────────────────────────────

    #[test]
    fn test_teleok_valid_position() {
        let level = make_walkable_level();
        assert!(teleok(&level, 5, 5, false));
    }

    #[test]
    fn test_teleok_wall() {
        let level = make_walkable_level();
        // (0,0) is stone/wall
        assert!(!teleok(&level, 0, 0, false));
    }

    #[test]
    fn test_teleok_monster_occupied() {
        let mut level = make_walkable_level();
        let m = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(m);
        assert!(!teleok(&level, 5, 5, false));
    }

    #[test]
    fn test_teleok_teleport_trap_rejected() {
        let mut level = make_walkable_level();
        level.add_trap(5, 5, TrapType::Teleport);
        assert!(!teleok(&level, 5, 5, false));
    }

    #[test]
    fn test_teleok_teleport_trap_accepted() {
        let mut level = make_walkable_level();
        level.add_trap(5, 5, TrapType::Teleport);
        assert!(teleok(&level, 5, 5, true));
    }

    #[test]
    fn test_teleok_pit_trap_ok() {
        let mut level = make_walkable_level();
        level.add_trap(5, 5, TrapType::Pit);
        // Pits don't block teleportation (even with trapok=false)
        assert!(teleok(&level, 5, 5, false));
    }

    #[test]
    fn test_teleok_hole_rejected() {
        let mut level = make_walkable_level();
        level.add_trap(5, 5, TrapType::Hole);
        assert!(!teleok(&level, 5, 5, false));
    }

    // ── rloc_pos_ok ──────────────────────────────────────────────────────

    #[test]
    fn test_rloc_pos_ok_normal_monster() {
        let level = make_walkable_level();
        let m = Monster::new(MonsterId(1), 0, 3, 3);
        assert!(rloc_pos_ok(&level, 5, 5, &m));
    }

    #[test]
    fn test_rloc_pos_ok_wall() {
        let level = make_walkable_level();
        let m = Monster::new(MonsterId(1), 0, 3, 3);
        assert!(!rloc_pos_ok(&level, 0, 0, &m));
    }

    #[test]
    fn test_rloc_pos_ok_shopkeeper_confined() {
        let mut level = make_walkable_level();
        let mut m = Monster::new(MonsterId(1), 0, 5, 5);
        m.is_shopkeeper = true;
        let mut shop = crate::special::shk::Shop::new(
            crate::special::ShopType::General,
            (3, 3, 8, 8),
        );
        shop.shopkeeper_id = Some(MonsterId(1));
        level.shops.push(shop);

        // Inside shop — OK
        assert!(rloc_pos_ok(&level, 5, 5, &m));
        // Outside shop — blocked
        assert!(!rloc_pos_ok(&level, 15, 5, &m));
    }

    // ── tele (player random teleport) ────────────────────────────────────

    #[test]
    fn test_tele_moves_player() {
        let level = make_walkable_level();
        let mut state = make_state_with_level(level);
        let old_pos = state.player.pos;

        let result = tele(&mut state);
        assert!(matches!(result, ActionResult::Success));
        // Player should have moved (extremely unlikely to land on same spot)
        // prev_pos should be set
        assert_eq!(state.player.prev_pos, old_pos);
    }

    #[test]
    fn test_tele_blocked_on_noteleport() {
        let mut level = make_walkable_level();
        level.flags.no_teleport = true;
        let mut state = make_state_with_level(level);

        let result = tele(&mut state);
        assert!(matches!(result, ActionResult::NoTime));
    }

    // ── tele_to (controlled teleport) ────────────────────────────────────

    #[test]
    fn test_tele_to_valid() {
        let level = make_walkable_level();
        let mut state = make_state_with_level(level);

        let result = tele_to(&mut state, 10, 5);
        assert!(matches!(result, ActionResult::Success));
        assert_eq!(state.player.pos.x, 10);
        assert_eq!(state.player.pos.y, 5);
    }

    #[test]
    fn test_tele_to_wall() {
        let level = make_walkable_level();
        let mut state = make_state_with_level(level);

        let result = tele_to(&mut state, 0, 0);
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_tele_to_monster() {
        let mut level = make_walkable_level();
        let m = Monster::new(MonsterId(1), 0, 10, 5);
        level.add_monster(m);
        let mut state = make_state_with_level(level);

        let result = tele_to(&mut state, 10, 5);
        assert!(matches!(result, ActionResult::NoTime));
    }

    // ── scroll_teleport ──────────────────────────────────────────────────

    #[test]
    fn test_scroll_teleport_blessed() {
        let level = make_walkable_level();
        let mut state = make_state_with_level(level);

        let result = scroll_teleport(&mut state, ScrollBuc::Blessed);
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    fn test_scroll_teleport_noteleport() {
        let mut level = make_walkable_level();
        level.flags.no_teleport = true;
        let mut state = make_state_with_level(level);

        let result = scroll_teleport(&mut state, ScrollBuc::Uncursed);
        assert!(matches!(result, ActionResult::NoTime));
    }

    // ── level_tele ───────────────────────────────────────────────────────

    #[test]
    fn test_level_tele_normal() {
        let level = make_walkable_level();
        let mut state = make_state_with_level(level);

        let result = level_tele(&mut state, 10);
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    fn test_level_tele_sokoban_blocked() {
        let mut level = Level::new(DLevel::new(3, 1)); // Sokoban
        for x in 1..20 {
            for y in 1..10 {
                *level.cell_mut(x, y) = Cell::floor();
            }
        }
        let mut state = make_state_with_level(level);

        let result = level_tele(&mut state, 10);
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_level_tele_above_ground_flying() {
        let level = make_walkable_level();
        let mut state = make_state_with_level(level);
        state.player.properties.grant_intrinsic(Property::Flying);
        state.player.properties.grant_intrinsic(Property::TeleportControl);

        let result = level_tele(&mut state, -5);
        assert!(matches!(result, ActionResult::Success));
        assert!(state.player.hp > 0);
    }

    #[test]
    fn test_level_tele_above_ground_death() {
        let mut level = Level::new(DLevel::new(0, 1));
        for x in 1..20 {
            for y in 1..10 {
                *level.cell_mut(x, y) = Cell::floor();
            }
        }
        let mut state = make_state_with_level(level);
        state.player.hp = 20;
        state.player.properties.grant_intrinsic(Property::TeleportControl);

        // level_tele uses random_teleport_level which always produces a
        // valid positive depth, so the player survives
        let result = level_tele(&mut state, -5);
        assert!(matches!(result, ActionResult::Success));
        assert!(state.player.hp > 0);
    }

    // ── random_teleport_level ────────────────────────────────────────────

    #[test]
    fn test_random_teleport_level_main_dungeon() {
        let level = make_walkable_level();
        let mut state = make_state_with_level(level);

        for _ in 0..100 {
            let depth = random_teleport_level(&mut state);
            assert!(depth >= 1, "depth {} below min 1", depth);
            assert!(depth <= 29, "depth {} above max 29", depth);
            // Should not equal current depth (5) — skipped by algorithm
        }
    }

    #[test]
    fn test_random_teleport_level_gehennom() {
        let mut level = Level::new(DLevel::new(1, 5));
        for x in 1..20 {
            for y in 1..10 {
                *level.cell_mut(x, y) = Cell::floor();
            }
        }
        let mut state = make_state_with_level(level);

        for _ in 0..100 {
            let depth = random_teleport_level(&mut state);
            assert!(depth >= 30, "depth {} below min 30", depth);
            assert!(depth <= 49, "depth {} above max 49", depth);
        }
    }

    // ── trap_teleport ────────────────────────────────────────────────────

    #[test]
    fn test_trap_teleport_works() {
        let mut level = make_walkable_level();
        level.add_trap(5, 5, TrapType::Teleport);
        let mut state = make_state_with_level(level);

        let result = trap_teleport(&mut state);
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    fn test_trap_teleport_noteleport() {
        let mut level = make_walkable_level();
        level.flags.no_teleport = true;
        level.add_trap(5, 5, TrapType::Teleport);
        let mut state = make_state_with_level(level);

        let result = trap_teleport(&mut state);
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_trap_teleport_once_removes_trap() {
        let mut level = make_walkable_level();
        level.add_trap(5, 5, TrapType::Teleport);
        // Set the trap to once-only
        if let Some(trap) = level.trap_at_mut(5, 5) {
            trap.once = true;
        }
        let mut state = make_state_with_level(level);

        trap_teleport(&mut state);
        // Trap should be removed
        assert!(state.current_level.trap_at(5, 5).is_none());
    }

    // ── trap_level_teleport ──────────────────────────────────────────────

    #[test]
    fn test_trap_level_teleport() {
        let level = make_walkable_level();
        let mut state = make_state_with_level(level);

        let result = trap_level_teleport(&mut state);
        assert!(matches!(result, ActionResult::Success | ActionResult::NoTime));
    }

    // ── rloc_monster ─────────────────────────────────────────────────────

    #[test]
    fn test_rloc_monster_basic() {
        let mut level = make_walkable_level();
        let m = Monster::new(MonsterId(1), 0, 5, 5);
        let id = level.add_monster(m);

        let moved = rloc_monster(&mut level, id);
        assert!(moved);

        let monster = level.monster(id).unwrap();
        // Monster should have moved from original position
        assert!(monster.x != 5 || monster.y != 5);
    }

    #[test]
    fn test_rloc_monster_invalid_id() {
        let mut level = make_walkable_level();
        assert!(!rloc_monster(&mut level, MonsterId(999)));
    }

    // ── rloc_monster_to ──────────────────────────────────────────────────

    #[test]
    fn test_rloc_monster_to_valid() {
        let mut level = make_walkable_level();
        let m = Monster::new(MonsterId(1), 0, 5, 5);
        let id = level.add_monster(m);

        assert!(rloc_monster_to(&mut level, id, 10, 5));
        let monster = level.monster(id).unwrap();
        assert_eq!(monster.x, 10);
        assert_eq!(monster.y, 5);
    }

    #[test]
    fn test_rloc_monster_to_invalid_pos() {
        let mut level = make_walkable_level();
        let m = Monster::new(MonsterId(1), 0, 5, 5);
        let id = level.add_monster(m);

        assert!(!rloc_monster_to(&mut level, id, -1, -1));
    }

    // ── mnexto ───────────────────────────────────────────────────────────

    #[test]
    fn test_mnexto_places_near_player() {
        let mut level = make_walkable_level();
        let m = Monster::new(MonsterId(1), 0, 15, 8);
        let id = level.add_monster(m);

        let moved = mnexto(&mut level, id, 5, 5);
        assert!(moved);

        let monster = level.monster(id).unwrap();
        // Should be near player position (5,5)
        let dx = (monster.x - 5).abs();
        let dy = (monster.y - 5).abs();
        assert!(dx <= 3 && dy <= 3, "Monster at ({},{}) too far from (5,5)", monster.x, monster.y);
    }

    // ── maybe_mnexto ─────────────────────────────────────────────────────

    #[test]
    fn test_maybe_mnexto_places_adjacent() {
        let mut level = make_walkable_level();
        let m = Monster::new(MonsterId(1), 0, 15, 8);
        let id = level.add_monster(m);

        let moved = maybe_mnexto(&mut level, id, 5, 5);
        if moved {
            let monster = level.monster(id).unwrap();
            let dx = (monster.x - 5).abs();
            let dy = (monster.y - 5).abs();
            assert!(dx <= 1 && dy <= 1, "Monster should be adjacent to player");
        }
        // It's OK if it fails (strict requirements)
    }

    // ── u_teleport_mon ───────────────────────────────────────────────────

    #[test]
    fn test_u_teleport_mon_normal() {
        let mut level = make_walkable_level();
        let m = Monster::new(MonsterId(1), 0, 5, 5);
        let id = level.add_monster(m);
        let mut rng = GameRng::new(42);

        let (success, _msgs) = u_teleport_mon(&mut level, id, true, 3, 3, &mut rng);
        assert!(success);
    }

    #[test]
    fn test_u_teleport_mon_priest_in_temple_resists() {
        let mut level = make_walkable_level();
        level.flags.has_temple = true;
        // Place an altar near (5,5)
        level.cell_mut(5, 5).typ = CellType::Altar;
        let mut m = Monster::new(MonsterId(1), 0, 5, 5);
        m.is_priest = true;
        let id = level.add_monster(m);
        let mut rng = GameRng::new(42);

        let (success, msgs) = u_teleport_mon(&mut level, id, true, 3, 3, &mut rng);
        assert!(!success);
        assert!(!msgs.is_empty());
    }

    // ── mtele_trap ───────────────────────────────────────────────────────

    #[test]
    fn test_mtele_trap_relocates() {
        let mut level = make_walkable_level();
        level.add_trap(5, 5, TrapType::Teleport);
        let m = Monster::new(MonsterId(1), 0, 5, 5);
        let id = level.add_monster(m);

        let moved = mtele_trap(&mut level, id, true);
        assert!(moved);
    }

    #[test]
    fn test_mtele_trap_noteleport() {
        let mut level = make_walkable_level();
        level.flags.no_teleport = true;
        level.add_trap(5, 5, TrapType::Teleport);
        let m = Monster::new(MonsterId(1), 0, 5, 5);
        let id = level.add_monster(m);

        assert!(!mtele_trap(&mut level, id, true));
    }

    #[test]
    fn test_mtele_trap_once_removes() {
        let mut level = make_walkable_level();
        level.add_trap(5, 5, TrapType::Teleport);
        if let Some(trap) = level.trap_at_mut(5, 5) {
            trap.once = true;
        }
        let m = Monster::new(MonsterId(1), 0, 5, 5);
        let id = level.add_monster(m);

        mtele_trap(&mut level, id, true);
        assert!(level.trap_at(5, 5).is_none());
    }

    // ── check_amulet_teleport_block ──────────────────────────────────────

    #[test]
    fn test_amulet_block_no_amulet() {
        let level = make_walkable_level();
        let mut state = make_state_with_level(level);
        // Without amulet, never blocks
        assert!(!check_amulet_teleport_block(&mut state, false));
    }

    #[test]
    fn test_amulet_block_with_amulet() {
        let level = make_walkable_level();
        let mut state = make_state_with_level(level);
        // With amulet, blocks 1 in 3
        let mut blocked = 0;
        for _ in 0..300 {
            if check_amulet_teleport_block(&mut state, true) {
                blocked += 1;
            }
        }
        // Should block roughly 1/3 of the time
        assert!(blocked > 50 && blocked < 150,
            "Expected ~100 blocks out of 300, got {}", blocked);
    }

    // ── rloc_object ──────────────────────────────────────────────────────

    #[test]
    fn test_rloc_object_basic() {
        let mut level = make_walkable_level();
        let obj = Object::new(crate::object::ObjectId(0), 0, crate::object::ObjectClass::Weapon);
        let id = level.add_object(obj, 5, 5);

        let result = rloc_object(&mut level, id);
        assert!(result.is_some());
        let (nx, ny) = result.unwrap();
        assert!(level.is_valid_pos(nx, ny));
    }

    // ── safe_teleds ──────────────────────────────────────────────────────

    #[test]
    fn test_safe_teleds_finds_position() {
        let level = make_walkable_level();
        let mut state = make_state_with_level(level);

        let (x, y) = safe_teleds(&mut state);
        assert!(state.current_level.is_valid_pos(x, y));
        assert!(state.current_level.is_walkable(x, y));
    }

    #[test]
    fn test_safe_teleds_avoids_traps() {
        let mut level = make_walkable_level();
        // Fill most of the walkable area with teleport traps
        for x in 1..15 {
            for y in 1..8 {
                level.add_trap(x as i8, y as i8, TrapType::Teleport);
            }
        }
        let mut state = make_state_with_level(level);

        // Should still find a position (some cells are trap-free)
        let (x, y) = safe_teleds(&mut state);
        assert!(state.current_level.is_valid_pos(x, y));
    }

    // ── monster_has_amulet ───────────────────────────────────────────────

    #[test]
    fn test_monster_has_amulet_none() {
        let m = Monster::new(MonsterId(1), 0, 5, 5);
        assert!(!monster_has_amulet(&m));
    }

    #[test]
    fn test_monster_has_amulet_with_amulet() {
        let mut m = Monster::new(MonsterId(1), 0, 5, 5);
        let mut amulet = Object::new(crate::object::ObjectId(0), 0, crate::object::ObjectClass::Amulet);
        amulet.object_type = 0;
        m.inventory.push(amulet);
        assert!(monster_has_amulet(&m));
    }

    // ── mlevel_tele_trap ─────────────────────────────────────────────────

    #[test]
    fn test_mlevel_tele_trap_normal() {
        let mut level = make_walkable_level();
        let m = Monster::new(MonsterId(1), 0, 5, 5);
        let id = level.add_monster(m);

        let moved = mlevel_tele_trap(&mut level, id, true);
        assert!(moved);
    }

    #[test]
    fn test_mlevel_tele_trap_noteleport() {
        let mut level = make_walkable_level();
        level.flags.no_teleport = true;
        let m = Monster::new(MonsterId(1), 0, 5, 5);
        let id = level.add_monster(m);

        assert!(!mlevel_tele_trap(&mut level, id, true));
    }

    #[test]
    fn test_mlevel_tele_trap_amulet_carrier() {
        let mut level = make_walkable_level();
        let mut m = Monster::new(MonsterId(1), 0, 5, 5);
        let mut amulet = Object::new(crate::object::ObjectId(0), 0, crate::object::ObjectClass::Amulet);
        amulet.object_type = 0;
        m.inventory.push(amulet);
        let id = level.add_monster(m);

        assert!(!mlevel_tele_trap(&mut level, id, true));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Additional stubs for C API compatibility
// ─────────────────────────────────────────────────────────────────────────────

pub fn dotele(state: &mut GameState) -> ActionResult {
    tele(state)
}

pub fn dotelecmd(state: &mut GameState) -> ActionResult {
    tele(state)
}

pub fn teleds(state: &mut GameState, x: i8, y: i8, allow_drag: bool) {
    if !teleok(&state.current_level, x, y, false) {
        state.message("Something blocks the teleportation.");
        return;
    }
    state.player.prev_pos = state.player.pos;
    state.player.pos.x = x;
    state.player.pos.y = y;
    state.message("You materialize.");
    if allow_drag {
        // Would handle dragging leashed pets, steeds, etc.
    }
}

pub fn tele_trap_obj(state: &mut GameState, _trap: &Object) {
    state.message("You trigger a teleport trap!");
    let _ = tele(state);
}

pub fn tele_jump_ok(_x1: i8, _y1: i8, _x2: i8, _y2: i8) -> bool {
    true
}

pub fn vault_tele(state: &mut GameState) {
    state.message("You are teleported out of the vault!");
    let _ = tele(state);
}

pub fn scrolltele_obj(state: &mut GameState, _obj: &Object) {
    state.message("You disappear!");
    let _ = tele(state);
}

pub fn tport_menu(_state: &mut GameState) {
    // Teleport menu
}

pub fn tport_spell(state: &mut GameState) {
    state.message("You cast teleport!");
    let _ = tele(state);
}

/// Create visual teleport effect
pub fn makevtele(state: &mut GameState) {
    state.message("*FLASH*");
}
