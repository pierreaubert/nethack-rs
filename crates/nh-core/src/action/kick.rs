//! Kicking (dokick.c)
//!
//! Kick damage formula from C:
//! dmg = (ACURRSTR + ACURR(A_DEX) + ACURR(A_CON)) / 15
//! If clumsy: dmg /= 2
//! If kicking boots: dmg += 5
//! Final damage: rnd(dmg) + martial bonus

use crate::action::{ActionResult, Direction};
use crate::dungeon::{CellType, DoorState};
use crate::gameloop::GameState;
use crate::player::Attribute;

/// Kick in a direction
pub fn do_kick(state: &mut GameState, dir: Direction) -> ActionResult {
    let (dx, dy) = dir.delta();
    let x = state.player.pos.x + dx;
    let y = state.player.pos.y + dy;

    if !state.current_level.is_valid_pos(x, y) {
        state.message("You kick at empty space.");
        return ActionResult::NoTime;
    }

    // Check for monster at target
    if let Some(monster_id) = state.current_level.monster_at(x, y).map(|m| m.id) {
        return kick_monster(state, monster_id);
    }

    // Check cell type for doors
    let cell = state.current_level.cell(x as usize, y as usize);
    if cell.typ == CellType::Door {
        return kick_door(state, x, y);
    }

    // Check for objects
    if !state.current_level.objects_at(x, y).is_empty() {
        return kick_object(state, x, y);
    }

    state.message("You kick at nothing.");
    ActionResult::Success
}

/// Kick a monster - deals damage based on stats
pub fn kick_monster(state: &mut GameState, monster_id: crate::monster::MonsterId) -> ActionResult {
    let damage = kickdmg(state, 0); // 0 = martial arts bonus if applicable (stub)

    // Get monster name before mutating
    let monster_name = state
        .current_level
        .monster(monster_id)
        .map(|m| m.name.clone())
        .unwrap_or_else(|| "monster".to_string());

    // Apply damage to monster
    if let Some(monster) = state.current_level.monster_mut(monster_id) {
        monster.hp -= damage;

        // Anger the monster
        monster.state.peaceful = false;

        if monster.hp <= 0 {
            state.message(format!("You kick the {} to death!", monster_name));
            // Monster death handled elsewhere (should be, but here we just leave it for the loop to clean up)
        } else {
            state.message(format!(
                "You kick the {} for {} damage!",
                monster_name, damage
            ));
        }
    }

    ActionResult::Success
}

/// Kick a door - may break it open
fn kick_door(state: &mut GameState, x: i8, y: i8) -> ActionResult {
    let cell = state.current_level.cell(x as usize, y as usize);
    let door_state = cell.door_state();

    if door_state.contains(DoorState::OPEN) {
        state.message("You kick at the open door.");
        return ActionResult::Success;
    }

    if door_state.contains(DoorState::BROKEN) {
        state.message("You kick at the broken door.");
        return ActionResult::Success;
    }

    let kick_power = kickstr(state);

    // Locked doors are harder to break
    let difficulty = if door_state.contains(DoorState::LOCKED) {
        30
    } else {
        20
    };

    let roll = state.rng.rn2(difficulty);

    if roll < kick_power {
        // Success - break the door
        let cell = state.current_level.cell_mut(x as usize, y as usize);

        if door_state.contains(DoorState::LOCKED) {
            // Breaking a locked door
            cell.set_door_state(DoorState::BROKEN);
            state.message("You break open the door!");
        } else {
            // Opening a closed door with a kick
            cell.set_door_state(DoorState::OPEN);
            state.message("The door crashes open!");
        }
    } else {
        // Failed
        if door_state.contains(DoorState::LOCKED) {
            state.message("The door shudders but remains locked.");
        } else {
            state.message("The door resists your kick.");
        }
    }

    ActionResult::Success
}

pub fn kick_object(state: &mut GameState, x: i8, y: i8) -> ActionResult {
    state.message("You kick the object.");
    // Stub: move object logic would go here
    ActionResult::Success
}

pub fn kick_steed(state: &mut GameState) -> ActionResult {
    state.message("You kick your steed.");
    ActionResult::Success
}

/// Calculate kick damage
pub fn kickdmg(state: &mut GameState, martial_bonus: i32) -> i32 {
    // Calculate kick damage: (STR + DEX + CON) / 15
    let str_val = state.player.attr_current.get(Attribute::Strength) as i32;
    let dex = state.player.attr_current.get(Attribute::Dexterity) as i32;
    let con = state.player.attr_current.get(Attribute::Constitution) as i32;

    let mut dmg = (str_val + dex + con) / 15;

    // Check for clumsy kick (encumbered)
    let encumbrance = state.player.encumbrance();
    let clumsy = encumbrance >= crate::player::Encumbrance::Stressed;
    if clumsy {
        dmg /= 2;
    }

    // Minimum 1 damage
    if dmg < 1 {
        dmg = 1;
    }

    // Roll for actual damage
    state.rng.rnd(dmg as u32) as i32 + martial_bonus
}

/// Calculate kick strength (for doors, etc.)
pub fn kickstr(state: &mut GameState) -> u32 {
    let str_val = state.player.attr_current.get(Attribute::Strength) as u32;
    let dex = state.player.attr_current.get(Attribute::Dexterity) as u32;
    let con = state.player.attr_current.get(Attribute::Constitution) as u32;
    (str_val + dex + con) / 3
}

pub fn maybe_kick_monster(state: &mut GameState, monster_id: crate::monster::MonsterId) -> bool {
    kick_monster(state, monster_id);
    true
}

pub fn really_kick_object(state: &mut GameState, x: i8, y: i8) {
    kick_object(state, x, y);
}

// ============================================================================
// Hurtle (Knockback) System
// ============================================================================

/// Player is thrown through the air (knockback effect)
/// Called when player is hit by an explosion, pushed by a monster, etc.
pub fn hurtle(state: &mut GameState, dx: i8, dy: i8, range: i32) {
    if range <= 0 {
        return;
    }

    state.message("You are thrown through the air!");

    let mut moved = 0;
    let start_x = state.player.pos.x;
    let start_y = state.player.pos.y;

    // Move the player step by step
    for _ in 0..range {
        if !hurtle_step(state, dx, dy) {
            break;
        }
        moved += 1;
    }

    // If we moved, show how far
    if moved > 0 {
        let end_x = state.player.pos.x;
        let end_y = state.player.pos.y;

        // Calculate damage if we hit something (stopped early)
        if moved < range {
            // Hit a wall or obstacle
            let damage = (range - moved) / 2;
            if damage > 0 {
                state.message(format!("You slam into something! ({} damage)", damage));
                state.player.hp -= damage;
            }
        }

        // Check if we fell into water, lava, or a pit
        let cell = state.current_level.cell(end_x as usize, end_y as usize);
        match cell.typ {
            CellType::Pool | CellType::Moat => {
                state.message("You fall into the water!");
                // Would trigger drown check
            }
            CellType::Lava => {
                state.message("You fall into the lava!");
                // Would trigger lava damage
            }
            _ => {}
        }
    }
}

/// Move player one step in hurtle direction
/// Returns true if step was successful, false if blocked
pub fn hurtle_step(state: &mut GameState, dx: i8, dy: i8) -> bool {
    let new_x = state.player.pos.x + dx;
    let new_y = state.player.pos.y + dy;

    // Check if new position is valid
    if !state.current_level.is_valid_pos(new_x, new_y) {
        return false;
    }

    // Check if blocked by terrain
    let cell = state.current_level.cell(new_x as usize, new_y as usize);
    if !cell.typ.is_passable() && cell.typ != CellType::Door {
        return false;
    }

    // Check for closed door
    if cell.typ == CellType::Door {
        let door_state = cell.door_state();
        if !door_state.contains(DoorState::OPEN) && !door_state.contains(DoorState::BROKEN) {
            // Hit the closed door
            return false;
        }
    }

    // Check for monster at destination
    if state.current_level.monster_at(new_x, new_y).is_some() {
        // Collide with monster - stop here and possibly damage both
        return false;
    }

    // Move the player
    state.player.prev_pos = state.player.pos;
    state.player.pos.x = new_x;
    state.player.pos.y = new_y;

    true
}

/// Player jumps in a direction (jumping boots, etc.)
pub fn hurtle_jump(state: &mut GameState, dx: i8, dy: i8) {
    // Jump is like a controlled hurtle with range 2
    state.message("You jump!");

    let range = 2; // Standard jump range
    for _ in 0..range {
        if !hurtle_step(state, dx, dy) {
            break;
        }
    }
}

/// Monster is thrown through the air (knockback effect)
pub fn mhurtle(
    state: &mut GameState,
    monster_id: crate::monster::MonsterId,
    dx: i8,
    dy: i8,
    range: i32,
) {
    if range <= 0 {
        return;
    }

    // Get monster name for messages
    let monster_name = state
        .current_level
        .monster(monster_id)
        .map(|m| m.name.clone())
        .unwrap_or_else(|| "creature".to_string());

    // Check if player can see the monster
    let (mx, my) = state
        .current_level
        .monster(monster_id)
        .map(|m| (m.x, m.y))
        .unwrap_or((0, 0));

    if state.current_level.visible[mx as usize][my as usize] {
        state.message(format!("The {} is thrown through the air!", monster_name));
    }

    let mut moved = 0;

    // Move the monster step by step
    for _ in 0..range {
        if !mhurtle_step(state, monster_id, dx, dy) {
            break;
        }
        moved += 1;
    }

    // Calculate collision damage if stopped early
    if moved < range {
        let damage = (range - moved) / 2;
        if damage > 0 {
            if let Some(monster) = state.current_level.monster_mut(monster_id) {
                monster.hp -= damage;
            }
        }
    }

    // Check if monster ended up in hazardous terrain
    if let Some(monster) = state.current_level.monster(monster_id) {
        let cell = state
            .current_level
            .cell(monster.x as usize, monster.y as usize);
        match cell.typ {
            CellType::Pool | CellType::Moat => {
                // Check if monster can swim
                // For now, just note it
            }
            CellType::Lava => {
                // Check if monster is fire resistant
            }
            _ => {}
        }
    }
}

/// Move monster one step in hurtle direction
/// Returns true if step was successful, false if blocked
pub fn mhurtle_step(
    state: &mut GameState,
    monster_id: crate::monster::MonsterId,
    dx: i8,
    dy: i8,
) -> bool {
    // Get monster's current position
    let (cur_x, cur_y) = match state.current_level.monster(monster_id) {
        Some(m) => (m.x, m.y),
        None => return false,
    };

    let new_x = cur_x + dx;
    let new_y = cur_y + dy;

    // Check if new position is valid
    if !state.current_level.is_valid_pos(new_x, new_y) {
        return false;
    }

    // Check if blocked by terrain
    let cell = state.current_level.cell(new_x as usize, new_y as usize);
    if !cell.typ.is_passable() && cell.typ != CellType::Door {
        return false;
    }

    // Check for closed door
    if cell.typ == CellType::Door {
        let door_state = cell.door_state();
        if !door_state.contains(DoorState::OPEN) && !door_state.contains(DoorState::BROKEN) {
            return false;
        }
    }

    // Check for another monster at destination
    if state.current_level.monster_at(new_x, new_y).is_some() {
        return false;
    }

    // Check for player at destination
    if new_x == state.player.pos.x && new_y == state.player.pos.y {
        return false;
    }

    // Move the monster
    if let Some(monster) = state.current_level.monster_mut(monster_id) {
        monster.x = new_x;
        monster.y = new_y;
    }

    // Update monster grid
    state.current_level.monster_grid[cur_x as usize][cur_y as usize] = None;
    state.current_level.monster_grid[new_x as usize][new_y as usize] = Some(monster_id);

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Encumbrance;

    #[test]
    fn test_kickdmg() {
        let mut state = GameState::default();
        // Set stats for predictable damage base
        state.player.attr_current.set(Attribute::Strength, 15);
        state.player.attr_current.set(Attribute::Dexterity, 15);
        state.player.attr_current.set(Attribute::Constitution, 15);

        // (15+15+15)/15 = 3 base damage

        let dmg = kickdmg(&mut state, 0);
        assert!(dmg >= 1 && dmg <= 3); // rnd(3)
    }

    #[test]
    fn test_hurtle_step_moves_player() {
        let mut state = GameState::default();
        // Position player in middle of open room
        state.player.pos.x = 10;
        state.player.pos.y = 10;

        // Mark cells as passable room floor
        state.current_level.cell_mut(10, 10).typ = CellType::Room;
        state.current_level.cell_mut(11, 10).typ = CellType::Room;

        // Step east
        let success = hurtle_step(&mut state, 1, 0);
        assert!(success);
        assert_eq!(state.player.pos.x, 11);
        assert_eq!(state.player.pos.y, 10);
    }

    #[test]
    fn test_hurtle_step_blocked_by_wall() {
        let mut state = GameState::default();
        state.player.pos.x = 10;
        state.player.pos.y = 10;

        // Mark current cell as room, destination as wall
        state.current_level.cell_mut(10, 10).typ = CellType::Room;
        state.current_level.cell_mut(11, 10).typ = CellType::HWall;

        // Try to step into wall
        let success = hurtle_step(&mut state, 1, 0);
        assert!(!success);
        // Player should not have moved
        assert_eq!(state.player.pos.x, 10);
        assert_eq!(state.player.pos.y, 10);
    }

    #[test]
    fn test_hurtle_step_blocked_by_closed_door() {
        let mut state = GameState::default();
        state.player.pos.x = 10;
        state.player.pos.y = 10;

        state.current_level.cell_mut(10, 10).typ = CellType::Room;
        let door_cell = state.current_level.cell_mut(11, 10);
        door_cell.typ = CellType::Door;
        door_cell.set_door_state(DoorState::CLOSED);

        // Hurtle step should be blocked by closed door
        let success = hurtle_step(&mut state, 1, 0);
        assert!(!success);
        assert_eq!(state.player.pos.x, 10);
    }

    #[test]
    fn test_hurtle_step_through_open_door() {
        let mut state = GameState::default();
        state.player.pos.x = 10;
        state.player.pos.y = 10;

        state.current_level.cell_mut(10, 10).typ = CellType::Room;
        let door_cell = state.current_level.cell_mut(11, 10);
        door_cell.typ = CellType::Door;
        door_cell.set_door_state(DoorState::OPEN);

        // Should pass through open door
        let success = hurtle_step(&mut state, 1, 0);
        assert!(success);
        assert_eq!(state.player.pos.x, 11);
    }

    #[test]
    fn test_hurtle_full_range() {
        let mut state = GameState::default();
        state.player.pos.x = 10;
        state.player.pos.y = 10;

        // Create a corridor of passable cells
        for i in 10..=15 {
            state.current_level.cell_mut(i, 10).typ = CellType::Room;
        }

        // Hurtle 3 cells east
        hurtle(&mut state, 1, 0, 3);

        // Should have moved 3 cells
        assert_eq!(state.player.pos.x, 13);
        assert_eq!(state.player.pos.y, 10);
    }

    #[test]
    fn test_hurtle_stopped_by_obstacle() {
        let mut state = GameState::default();
        state.player.pos.x = 10;
        state.player.pos.y = 10;

        // Create passable cells then a wall
        state.current_level.cell_mut(10, 10).typ = CellType::Room;
        state.current_level.cell_mut(11, 10).typ = CellType::Room;
        state.current_level.cell_mut(12, 10).typ = CellType::HWall; // Wall

        // Try to hurtle 5 cells but wall is at 2
        hurtle(&mut state, 1, 0, 5);

        // Should have stopped at cell 11 (just before wall)
        assert_eq!(state.player.pos.x, 11);
    }

    #[test]
    fn test_kickstr() {
        let mut state = GameState::default();
        state.player.attr_current.set(Attribute::Strength, 18);
        state.player.attr_current.set(Attribute::Dexterity, 15);
        state.player.attr_current.set(Attribute::Constitution, 12);

        // (18+15+12)/3 = 15
        let str = kickstr(&mut state);
        assert_eq!(str, 15);
    }
}
