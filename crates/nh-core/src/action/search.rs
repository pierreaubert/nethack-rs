//! Searching for hidden things (detect.c)

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::action::ActionResult;
use crate::dungeon::{CellType, TrapType};
use crate::gameloop::GameState;

/// The 's' command - explicit searching for hidden doors, traps, monsters
///
/// Searches all adjacent squares for hidden features.
pub fn dosearch(state: &mut GameState) -> ActionResult {
    dosearch0(state, false)
}

/// Search implementation with autosearch flag
///
/// # Arguments
/// * `state` - The game state
/// * `autosearch` - True if this is intrinsic autosearch vs explicit searching
pub fn dosearch0(state: &mut GameState, autosearch: bool) -> ActionResult {
    if state.player.swallowed {
        return ActionResult::NoTime; // Can't search while engulfed
    }

    // Calculate search bonus from equipment
    let mut search_bonus = 0i32;

    // Check for lenses (wearing lenses helps searching)
    // Object type 9000+ is the tool category, lenses are typically around 9080
    for obj in &state.inventory {
        if obj.class == crate::object::ObjectClass::Tool && obj.worn_mask != 0 {
            // Simplified check - in full implementation would check for LENSES specifically
            search_bonus += 1;
        }
    }
    search_bonus = search_bonus.min(5);

    let player_x = state.player.pos.x;
    let player_y = state.player.pos.y;
    let is_blind = state.player.is_blind();
    let luck = state.player.luck;

    // Search all adjacent squares
    for dx in -1..=1i8 {
        for dy in -1..=1i8 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let x = player_x + dx;
            let y = player_y + dy;

            if !state.current_level.is_valid_pos(x, y) {
                continue;
            }

            // Feel location if blind
            if is_blind && !autosearch {
                // Would call feel_location here
            }

            let cell_type = state.current_level.cell(x as usize, y as usize).typ;

            // Check for secret door
            if cell_type == CellType::SecretDoor {
                // Chance to find based on luck and search bonus
                let find_chance = 7 - search_bonus;
                if state.rng.rnl(find_chance as u32, luck) == 0 {
                    state.current_level.cell_mut(x as usize, y as usize).typ = CellType::Door;
                    state.message("You find a hidden door.");
                }
            }
            // Check for secret corridor
            else if cell_type == CellType::SecretCorridor {
                let find_chance = 7 - search_bonus;
                if state.rng.rnl(find_chance as u32, luck) == 0 {
                    state.current_level.cell_mut(x as usize, y as usize).typ = CellType::Corridor;
                    state.message("You find a hidden passage.");
                }
            }
            // Check for hidden monsters and traps
            else {
                // Check for hidden monster
                if !autosearch {
                    if let Some(monster) = state.current_level.monster_at(x, y) {
                        if monster.state.hiding {
                            let monster_id = monster.id;
                            let monster_name = monster.name.clone();
                            // Try to find the hidden monster
                            if let Some(mon) = state.current_level.monster_mut(monster_id) {
                                mon.state.hiding = false;
                                state.message(format!("You find {} hiding there!", monster_name));
                            }
                        }
                    }
                }

                // Check for hidden trap
                if let Some(trap) = state.current_level.trap_at(x, y) {
                    if !trap.seen {
                        let find_chance = 8;
                        if state.rng.rnl(find_chance, luck) == 0 {
                            find_trap(state, x, y);
                        }
                    }
                }
            }
        }
    }

    ActionResult::Success
}

/// Find a trap at the given location and reveal it
pub fn find_trap(state: &mut GameState, x: i8, y: i8) {
    // Find trap index first
    let trap_idx = state
        .current_level
        .traps
        .iter()
        .position(|t| t.x == x && t.y == y);

    if let Some(idx) = trap_idx {
        let trap = &mut state.current_level.traps[idx];
        if !trap.seen {
            trap.seen = true;
            let name = trap_name(trap.trap_type);
            state.message(format!("You find a {}.", name));
        }
    }
}

/// Get the display name for a trap type
fn trap_name(trap_type: TrapType) -> &'static str {
    match trap_type {
        TrapType::Arrow => "arrow trap",
        TrapType::Dart => "dart trap",
        TrapType::RockFall => "falling rock trap",
        TrapType::Squeaky => "squeaky board",
        TrapType::BearTrap => "bear trap",
        TrapType::LandMine => "land mine",
        TrapType::RollingBoulder => "rolling boulder trap",
        TrapType::SleepingGas => "sleeping gas trap",
        TrapType::RustTrap => "rust trap",
        TrapType::FireTrap => "fire trap",
        TrapType::Pit => "pit",
        TrapType::SpikedPit => "spiked pit",
        TrapType::Hole => "hole",
        TrapType::TrapDoor => "trap door",
        TrapType::Teleport => "teleportation trap",
        TrapType::LevelTeleport => "level teleporter",
        TrapType::MagicPortal => "magic portal",
        TrapType::Web => "web",
        TrapType::Statue => "statue trap",
        TrapType::MagicTrap => "magic trap",
        TrapType::AntiMagic => "anti-magic field",
        TrapType::Polymorph => "polymorph trap",
    }
}

/// Magical detection of hidden things (findit equivalent)
///
/// Uses magical means to find all hidden things within bolt range.
/// Returns the number of things found.
pub fn findit(state: &mut GameState) -> i32 {
    if state.player.swallowed {
        return 0; // Can't detect things while engulfed
    }

    let player_x = state.player.pos.x;
    let player_y = state.player.pos.y;
    const BOLT_LIM: i32 = 8;

    let mut found = 0;

    // Search in a square area around the player
    for dx in -BOLT_LIM..=BOLT_LIM {
        for dy in -BOLT_LIM..=BOLT_LIM {
            let x = player_x as i32 + dx;
            let y = player_y as i32 + dy;

            if x < 0 || y < 0 || x > 127 || y > 127 {
                continue;
            }

            found += findone(state, x as i8, y as i8);
        }
    }

    found
}

/// Find hidden things at one location (findone equivalent)
///
/// Used for magical detection - finds all hidden things at a location.
/// Returns the count of things found.
pub fn findone(state: &mut GameState, x: i8, y: i8) -> i32 {
    if !state.current_level.is_valid_pos(x, y) {
        return 0;
    }

    let mut found = 0;

    // Check for secret door
    let cell_type = state.current_level.cell(x as usize, y as usize).typ;
    if cell_type == CellType::SecretDoor {
        state.current_level.cell_mut(x as usize, y as usize).typ = CellType::Door;
        found += 1;
    } else if cell_type == CellType::SecretCorridor {
        state.current_level.cell_mut(x as usize, y as usize).typ = CellType::Corridor;
        found += 1;
    }

    // Check for hidden trap
    let trap_idx =
        state.current_level.traps.iter().position(|t| {
            t.x == x && t.y == y && !t.seen && !matches!(t.trap_type, TrapType::Statue)
        });
    if let Some(idx) = trap_idx {
        state.current_level.traps[idx].seen = true;
        found += 1;
    }

    // Check for hidden monster
    if let Some(monster) = state.current_level.monster_at(x, y) {
        if monster.state.hiding {
            let monster_id = monster.id;
            if let Some(mon) = state.current_level.monster_mut(monster_id) {
                mon.state.hiding = false;
                found += 1;
            }
        }
    }

    found
}

/// Open hidden things at one location (for wand of opening, knock spell)
/// Returns count of things opened/revealed.
pub fn openone(state: &mut GameState, x: i8, y: i8) -> i32 {
    if !state.current_level.is_valid_pos(x, y) {
        return 0;
    }

    let mut opened = 0;

    // Check for secret or closed doors
    let cell_type = state.current_level.cell(x as usize, y as usize).typ;
    if cell_type == CellType::SecretDoor {
        state.current_level.cell_mut(x as usize, y as usize).typ = CellType::Door;
        opened += 1;
    } else if cell_type == CellType::Door {
        // Would set door to open state via flags
        opened += 1;
    } else if cell_type == CellType::SecretCorridor {
        state.current_level.cell_mut(x as usize, y as usize).typ = CellType::Corridor;
        opened += 1;
    }

    // Check for hidden traps
    let trap_idx = state
        .current_level
        .traps
        .iter()
        .position(|t| t.x == x && t.y == y && !t.seen);
    if let Some(idx) = trap_idx {
        state.current_level.traps[idx].seen = true;
        opened += 1;
    }

    opened
}

/// Open things in area (openit equivalent, for knock spell)
pub fn openit(state: &mut GameState) -> i32 {
    if state.player.swallowed {
        // Knock spell expels player from engulfer
        state.player.swallowed = false;
        return -1;
    }

    let player_x = state.player.pos.x;
    let player_y = state.player.pos.y;
    const BOLT_LIM: i32 = 8;

    let mut opened = 0;

    for dx in -BOLT_LIM..=BOLT_LIM {
        for dy in -BOLT_LIM..=BOLT_LIM {
            let x = player_x as i32 + dx;
            let y = player_y as i32 + dy;

            if x < 0 || y < 0 || x > 127 || y > 127 {
                continue;
            }

            opened += openone(state, x as i8, y as i8);
        }
    }

    opened
}

/// Reveal nearby monsters that are sources of warnings
pub fn warnreveal(state: &mut GameState) {
    let player_x = state.player.pos.x;
    let player_y = state.player.pos.y;

    // Check adjacent squares
    for dx in -1..=1i8 {
        for dy in -1..=1i8 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let x = player_x + dx;
            let y = player_y + dy;

            if !state.current_level.is_valid_pos(x, y) {
                continue;
            }

            if let Some(monster) = state.current_level.monster_at(x, y) {
                // If monster is hiding, reveal it (simplified - would check warning source)
                if monster.state.hiding {
                    let monster_id = monster.id;
                    let monster_name = monster.name.clone();
                    if let Some(mon) = state.current_level.monster_mut(monster_id) {
                        mon.state.hiding = false;
                        state.message(format!("You sense {} lurking nearby!", monster_name));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::GameRng;

    #[test]
    fn test_dosearch_basic() {
        let mut state = GameState::new(GameRng::from_entropy());

        let result = dosearch(&mut state);
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    fn test_findit_basic() {
        let mut state = GameState::new(GameRng::from_entropy());

        let found = findit(&mut state);
        assert!(found >= 0);
    }

    #[test]
    fn test_trap_name() {
        assert_eq!(trap_name(TrapType::Arrow), "arrow trap");
        assert_eq!(trap_name(TrapType::BearTrap), "bear trap");
        assert_eq!(trap_name(TrapType::Pit), "pit");
    }
}
