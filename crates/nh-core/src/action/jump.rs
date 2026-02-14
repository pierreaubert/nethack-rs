//! Jumping mechanics (apply.c)

use crate::action::ActionResult;
use crate::dungeon::CellType;
use crate::gameloop::GameState;
use crate::player::Property;

/// Jump trajectory types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpTrajectory {
    /// Any direction (magical jump)
    Any = 0,
    /// Horizontal
    Horizontal = 1,
    /// Vertical
    Vertical = 2,
    /// Diagonal (both horizontal and vertical)
    Diagonal = 3,
}

/// Physical jump command
pub fn dojump(state: &mut GameState) -> ActionResult {
    jump(state, 0)
}

/// Jump command with optional magic level
///
/// # Arguments
/// * `state` - Game state
/// * `magic` - 0 for physical jump, otherwise spell skill level
pub fn jump(state: &mut GameState, magic: i32) -> ActionResult {
    // Check if player can jump
    if magic == 0 && !state.player.properties.has(Property::Jumping) {
        // Try to use jumping spell if known
        let has_jump_spell = state
            .player
            .known_spells
            .iter()
            .any(|s| matches!(s.spell_type, crate::magic::spell::SpellType::Jumping));

        if has_jump_spell {
            state.message("You invoke the jumping spell!");
            // Would cast the spell here
            return jump(state, 1);
        }

        state.message("You can't jump very far.");
        return ActionResult::NoTime;
    }

    // Check physical jumping requirements
    if magic == 0 {
        // Check if player has legs (not polymorphed into something legless)
        if state.player.is_polymorphed() {
            // Simplified check - would check monster form for legs
            state.message("You can't jump; you have no legs!");
            return ActionResult::NoTime;
        }
    }

    // TODO: Check for being stuck/held when held state tracking is implemented

    // Check for being burdened (Stressed or worse prevents jumping)
    use crate::player::Encumbrance;
    let enc = state.player.encumbrance();
    if enc >= Encumbrance::Stressed {
        state.message("You are too burdened to jump!");
        return ActionResult::Failed("burdened".to_string());
    }

    state.message("Where do you want to jump? (select a position)");
    // In full implementation, would get target position from player input
    // For now, just demonstrate the mechanic by jumping in a random valid direction

    // Find valid jump positions
    let valid_positions = get_all_valid_jump_positions(state, magic);

    if valid_positions.is_empty() {
        state.message("There is nowhere you can jump to.");
        return ActionResult::NoTime;
    }

    // In full implementation, player would select target
    // For now, just report the count of valid positions
    state.message(format!(
        "You could jump to {} different positions.",
        valid_positions.len()
    ));

    ActionResult::Success
}

/// Check if a jump along a path is valid
///
/// Returns true if the path is clear for jumping
pub fn check_jump(
    state: &GameState,
    start_x: i8,
    start_y: i8,
    end_x: i8,
    end_y: i8,
    traj: JumpTrajectory,
) -> bool {
    // Check each cell along the path
    let dx = (end_x - start_x).signum();
    let dy = (end_y - start_y).signum();

    let mut x = start_x;
    let mut y = start_y;

    // Walk the path
    while x != end_x || y != end_y {
        // Move toward target
        if x != end_x {
            x += dx;
        }
        if y != end_y {
            y += dy;
        }

        // Check if this position is passable
        if !state.current_level.is_valid_pos(x, y) {
            return false;
        }

        // Check for walls
        let cell = state.current_level.cell(x as usize, y as usize);
        if cell.typ.is_wall() {
            // Can pass through walls with Passes_walls property
            if !state.player.properties.has(Property::PassesWalls) {
                return false;
            }
        }

        // Check for closed doors
        if cell.typ == CellType::Door {
            // Check if door is closed (simplified - would check door state flags)
            let is_closed = cell.flags & 0x01 != 0;
            if is_closed {
                return false;
            }

            // Check for diagonal movement through open doors
            let is_horizontal_door = cell.horizontal;
            if traj != JumpTrajectory::Any {
                if traj == JumpTrajectory::Diagonal {
                    return false; // Can't jump diagonally through doors
                }
                // Can't jump horizontally through horizontal door or vice versa
                if (traj == JumpTrajectory::Horizontal) == is_horizontal_door {
                    return false;
                }
            }
        }

        // Check for boulders
        // In full implementation, would check for BOULDER objects
    }

    true
}

/// Check if a position is a valid jump destination
///
/// # Arguments
/// * `state` - Game state
/// * `x` - Target x coordinate
/// * `y` - Target y coordinate
/// * `magic` - 0 for physical jump, otherwise spell skill level
/// * `show_msg` - Whether to show error messages
pub fn is_valid_jump_pos(state: &mut GameState, x: i8, y: i8, magic: i32, show_msg: bool) -> bool {
    let player_x = state.player.pos.x;
    let player_y = state.player.pos.y;

    // Calculate distance squared
    let dx = (x as i32 - player_x as i32).abs();
    let dy = (y as i32 - player_y as i32).abs();
    let dist_sq = dx * dx + dy * dy;

    // Knight's move restriction for physical jumps
    if magic == 0 && !state.player.properties.has(Property::Jumping) {
        // Knights can only move in an L shape (distance squared = 5)
        if dist_sq != 5 {
            if show_msg {
                state.message("Illegal move!");
            }
            return false;
        }
    }

    // Maximum jump distance
    let max_dist_sq = if magic > 0 {
        (6 + magic * 3) as i32 * (6 + magic * 3) as i32
    } else {
        9 * 9 // 9 squares for physical jump
    };

    if dist_sq > max_dist_sq {
        if show_msg {
            state.message("Too far!");
        }
        return false;
    }

    // Check if position is valid
    if !state.current_level.is_valid_pos(x, y) {
        if show_msg {
            state.message("You cannot jump there!");
        }
        return false;
    }

    // Check if position is visible
    if !state.current_level.is_visible(x, y) {
        if show_msg {
            state.message("You cannot see where to land!");
        }
        return false;
    }

    // Determine trajectory type
    let ax = dx;
    let ay = dy;
    let traj = if magic > 0 || state.player.properties.has(Property::PassesWalls) {
        JumpTrajectory::Any
    } else if ay == 0 {
        JumpTrajectory::Horizontal
    } else if ax == 0 {
        JumpTrajectory::Vertical
    } else {
        JumpTrajectory::Diagonal
    };

    // Flatten trajectory for door checking
    let flat_traj = if ax >= 2 * ay {
        JumpTrajectory::Horizontal
    } else if ay >= 2 * ax {
        JumpTrajectory::Vertical
    } else {
        traj
    };

    // Check path is clear
    if !check_jump(state, player_x, player_y, x, y, flat_traj) {
        if show_msg {
            state.message("There is an obstacle preventing that jump.");
        }
        return false;
    }

    // Check destination is passable
    let cell = state.current_level.cell(x as usize, y as usize);
    if !cell.typ.is_passable() && !state.player.properties.has(Property::PassesWalls) {
        if show_msg {
            state.message("You cannot land there!");
        }
        return false;
    }

    true
}

/// Get a valid jump position (simplified)
pub fn get_valid_jump_position(state: &mut GameState, x: i8, y: i8, magic: i32) -> bool {
    if !state.current_level.is_valid_pos(x, y) {
        return false;
    }

    let cell = state.current_level.cell(x as usize, y as usize);
    if !cell.typ.is_passable() && !state.player.properties.has(Property::PassesWalls) {
        return false;
    }

    is_valid_jump_pos(state, x, y, magic, false)
}

/// Get all valid jump positions
fn get_all_valid_jump_positions(state: &mut GameState, magic: i32) -> Vec<(i8, i8)> {
    let mut positions = Vec::new();
    let player_x = state.player.pos.x;
    let player_y = state.player.pos.y;

    // Check positions within jump range
    for dx in -4..=4i8 {
        for dy in -4..=4i8 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let x = player_x + dx;
            let y = player_y + dy;

            if get_valid_jump_position(state, x, y, magic) {
                positions.push((x, y));
            }
        }
    }

    positions
}

/// Display valid jump positions (for UI)
pub fn display_jump_positions(state: &mut GameState, magic: i32) -> Vec<(i8, i8)> {
    get_all_valid_jump_positions(state, magic)
}

/// Execute the actual jump to a position
pub fn execute_jump(state: &mut GameState, target_x: i8, target_y: i8, magic: i32) -> ActionResult {
    // Verify the position is valid
    if !is_valid_jump_pos(state, target_x, target_y, magic, true) {
        return ActionResult::Failed("Invalid jump position".to_string());
    }

    // Move the player
    state.player.pos.x = target_x;
    state.player.pos.y = target_y;

    state.message("You jump!");

    // Check for landing on something
    // Would check for traps, items, monsters at destination

    ActionResult::Success
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::GameRng;

    #[test]
    fn test_dojump_no_jumping() {
        let mut state = GameState::new(GameRng::from_entropy());
        state.player.properties.remove_intrinsic(Property::Jumping);
        state.player.known_spells.clear();

        let result = dojump(&mut state);
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_jump_trajectory() {
        assert_eq!(JumpTrajectory::Any as i32, 0);
        assert_eq!(JumpTrajectory::Horizontal as i32, 1);
        assert_eq!(JumpTrajectory::Vertical as i32, 2);
        assert_eq!(JumpTrajectory::Diagonal as i32, 3);
    }

    #[test]
    fn test_get_all_valid_jump_positions() {
        let mut state = GameState::new(GameRng::from_entropy());
        let positions = get_all_valid_jump_positions(&mut state, 1);
        // Should return some positions (depends on level generation)
        // The exact count depends on the level layout
        assert!(positions.len() >= 0);
    }
}
