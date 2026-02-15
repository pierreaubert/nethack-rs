//! Movement system (hack.c)
//!
//! Implements boulder pushing, sink interactions, movement validation,
//! and terrain effects from the C source's domove_core/moverock/dosinkfall.

use crate::consts::BOULDER;
use crate::dungeon::{CellType, Level, TrapType};
use crate::gameloop::GameState;
use crate::object::ObjectId;
use crate::player::{Encumbrance, Property};

/// Result of attempting to push a boulder (hack.c:moverock)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveRockResult {
    /// Boulder moved, player can advance to the vacated position
    Moved,
    /// Boulder blocked, player cannot advance
    Blocked,
    /// Player squeezed past the boulder
    SqueezedPast,
}

/// Try to push a boulder at (sx,sy) in direction (dx,dy).
///
/// Port of hack.c:moverock(). Returns whether the player can advance.
///
/// The boulder at (sx,sy) is pushed to (rx,ry) = (sx+dx, sy+dy) if possible.
pub fn moverock(
    state: &mut GameState,
    sx: i8,
    sy: i8,
    dx: i8,
    dy: i8,
) -> MoveRockResult {
    // Check if there's a boulder at the source
    let boulder_id = match find_boulder(&state.current_level, sx, sy) {
        Some(id) => id,
        None => return MoveRockResult::Moved, // no boulder, free to pass
    };

    let rx = sx + dx; // boulder destination
    let ry = sy + dy;

    // Can't push while levitating — no leverage (hack.c:121)
    if state.player.properties.has(Property::Levitation) {
        state.message("You don't have enough leverage to push the boulder.");
        return MoveRockResult::Blocked;
    }

    // Check destination is valid and passable for boulder
    if !state.current_level.is_valid_pos(rx, ry) {
        state.message("You try to push the boulder, but it won't budge.");
        return MoveRockResult::Blocked;
    }

    let dest_cell = state.current_level.cell(rx as usize, ry as usize);
    let dest_typ = dest_cell.typ;

    // Boulder can't be pushed into walls, iron bars, other boulders
    if !can_receive_boulder(dest_typ) {
        state.message("You try to push the boulder, but it won't budge.");
        return MoveRockResult::Blocked;
    }

    // Check for another boulder at destination
    if find_boulder(&state.current_level, rx, ry).is_some() {
        state.message("There is a boulder in the way.");
        return MoveRockResult::Blocked;
    }

    // Check for monster at destination
    if state.current_level.monster_at(rx, ry).is_some() {
        state.message("There's a monster on the other side.");
        state.message("Perhaps that's why you cannot move it.");
        return MoveRockResult::Blocked;
    }

    // Check for closed door at destination
    if state.current_level.cell(rx as usize, ry as usize).is_closed_door() {
        state.message("You try to push the boulder, but it won't budge.");
        return MoveRockResult::Blocked;
    }

    // Handle traps at destination (hack.c:188-279)
    if let Some(trap) = state.current_level.trap_at(rx, ry) {
        let trap_type = trap.trap_type;
        match trap_type {
            TrapType::Pit | TrapType::SpikedPit => {
                // Boulder fills the pit
                state.message("The boulder fills a pit!");
                state.current_level.remove_object(boulder_id);
                state.current_level.remove_trap(rx, ry);
                return if find_boulder(&state.current_level, sx, sy).is_some() {
                    MoveRockResult::Blocked
                } else {
                    MoveRockResult::Moved
                };
            }
            TrapType::Hole | TrapType::TrapDoor => {
                // Boulder plugs the hole
                state.message("The boulder plugs the hole!");
                state.current_level.remove_object(boulder_id);
                state.current_level.remove_trap(rx, ry);
                return if find_boulder(&state.current_level, sx, sy).is_some() {
                    MoveRockResult::Blocked
                } else {
                    MoveRockResult::Moved
                };
            }
            TrapType::LandMine => {
                // 90% chance landmine detonates
                if state.rng.rn2(10) != 0 {
                    state.message("KAABLAMM!!! The boulder triggers a land mine.");
                    state.current_level.remove_trap(rx, ry);
                    // Boulder may survive — move it there
                    move_boulder(&mut state.current_level, boulder_id, rx, ry);
                    return if find_boulder(&state.current_level, sx, sy).is_some() {
                        MoveRockResult::Blocked
                    } else {
                        MoveRockResult::Moved
                    };
                }
                // 10% chance: landmine doesn't fire, push normally
            }
            TrapType::Teleport => {
                // Boulder teleports randomly
                state.message("You push the boulder and suddenly it disappears!");
                state.current_level.remove_object(boulder_id);
                // In full implementation, would relocate boulder randomly
                return if find_boulder(&state.current_level, sx, sy).is_some() {
                    MoveRockResult::Blocked
                } else {
                    MoveRockResult::Moved
                };
            }
            TrapType::LevelTeleport => {
                state.message("You push the boulder and suddenly it disappears!");
                state.current_level.remove_object(boulder_id);
                return if find_boulder(&state.current_level, sx, sy).is_some() {
                    MoveRockResult::Blocked
                } else {
                    MoveRockResult::Moved
                };
            }
            _ => {
                // Other traps don't affect boulders
            }
        }
    }

    // Boulder goes into water/lava/pool (hack.c:283 boulder_hits_pool)
    match dest_typ {
        CellType::Pool | CellType::Moat | CellType::Water => {
            state.message("The boulder falls into the water!");
            state.current_level.remove_object(boulder_id);
            // Pool is filled — change to floor
            state.current_level.cell_mut(rx as usize, ry as usize).typ = CellType::Room;
            return if find_boulder(&state.current_level, sx, sy).is_some() {
                MoveRockResult::Blocked
            } else {
                MoveRockResult::Moved
            };
        }
        CellType::Lava => {
            state.message("The boulder falls into the lava and is consumed!");
            state.current_level.remove_object(boulder_id);
            // Lava is cooled
            state.current_level.cell_mut(rx as usize, ry as usize).typ = CellType::Room;
            return if find_boulder(&state.current_level, sx, sy).is_some() {
                MoveRockResult::Blocked
            } else {
                MoveRockResult::Moved
            };
        }
        _ => {}
    }

    // Normal push — move boulder to destination (hack.c:302-325)
    state.message("With great effort you move the boulder.");
    move_boulder(&mut state.current_level, boulder_id, rx, ry);

    MoveRockResult::Moved
}

/// Check if there is a boulder at a position (public convenience).
pub fn find_boulder_at(level: &Level, x: i8, y: i8) -> bool {
    find_boulder(level, x, y).is_some()
}

/// Find the first boulder object at a position on the level.
fn find_boulder(level: &Level, x: i8, y: i8) -> Option<ObjectId> {
    level
        .objects_at(x, y)
        .iter()
        .find(|obj| obj.object_type == BOULDER)
        .map(|obj| obj.id)
}

/// Move a boulder from its current position to (nx, ny).
fn move_boulder(level: &mut Level, boulder_id: ObjectId, nx: i8, ny: i8) {
    if let Some(boulder) = level.remove_object(boulder_id) {
        level.add_object(boulder, nx, ny);
    }
}

/// Check if a cell type can receive a pushed boulder.
fn can_receive_boulder(typ: CellType) -> bool {
    match typ {
        // Passable terrain that can hold a boulder
        CellType::Room
        | CellType::Corridor
        | CellType::Door
        | CellType::Altar
        | CellType::Throne
        | CellType::Grave
        | CellType::Ice
        | CellType::Fountain
        | CellType::Sink
        // Water/lava — boulder will fall in (handled separately)
        | CellType::Pool
        | CellType::Moat
        | CellType::Water
        | CellType::Lava => true,
        // Everything else (walls, stone, etc.) blocks
        _ => false,
    }
}

/// Sink fall effect when levitating over a sink (hack.c:dosinkfall)
///
/// In NetHack, levitating over a kitchen sink causes you to crash down.
/// The damage is based on Constitution.
pub fn dosinkfall(state: &mut GameState) {
    let has_flying = state.player.properties.has(Property::Flying);

    if has_flying {
        // Flying prevents the crash
        state.message("You wobble unsteadily for a moment.");
        return;
    }

    // Remove levitation and crash
    state.message("You crash to the floor!");

    let con = state
        .player
        .attr_current
        .get(crate::player::Attribute::Constitution) as i32;
    // rn1(8, 25 - CON) = (25 - CON) + rn2(8), range: (17-CON) to (32-CON)
    let dmg = (25 - con + state.rng.rn2(8) as i32).max(1);
    state.player.take_damage(dmg);

    if state.player.is_dead() {
        state.message("You die from the fall!");
    }

    // Check for weapons on the floor that could hurt
    let px = state.player.pos.x;
    let py = state.player.pos.y;
    let weapon_on_floor = state
        .current_level
        .objects_at(px, py)
        .iter()
        .any(|obj| {
            obj.class == crate::object::ObjectClass::Weapon
        });

    if weapon_on_floor {
        state.message("You fell on a weapon!");
        let extra_dmg = state.rng.rnd(3) as i32;
        state.player.take_damage(extra_dmg);
    }
}

/// Validate whether the player can move at all (hack.c:domove_core encumbrance check)
///
/// Returns an error message if movement is blocked, or None if movement is allowed.
pub fn check_movement_capacity(state: &GameState) -> Option<&'static str> {
    let enc = state.player.encumbrance();

    match enc {
        Encumbrance::Overloaded => {
            Some("You collapse under your load.")
        }
        Encumbrance::Overtaxed | Encumbrance::Strained | Encumbrance::Stressed => {
            // Can still move when heavily encumbered but not at critical HP
            if state.player.hp < 10 && state.player.hp != state.player.hp_max {
                Some("You don't have enough stamina to move.")
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Apply confusion/stun direction randomization (hack.c:confdir)
///
/// When confused or stunned, the player's movement direction may change randomly.
/// Returns (dx, dy) after potential randomization.
pub fn confdir(state: &mut GameState, dx: i8, dy: i8) -> (i8, i8) {
    let confused = state.player.confused_timeout > 0;
    let stunned = state.player.stunned_timeout > 0;

    if !stunned && !confused {
        return (dx, dy);
    }

    // Stunned always randomizes; confused has 1/5 chance of keeping direction
    if stunned || state.rng.rn2(5) == 0 {
        // Pick a random direction
        let dirs: [(i8, i8); 8] = [
            (0, -1),
            (0, 1),
            (1, 0),
            (-1, 0),
            (1, -1),
            (-1, -1),
            (1, 1),
            (-1, 1),
        ];
        let idx = state.rng.rn2(8) as usize;
        return dirs[idx];
    }

    (dx, dy)
}

/// Check if a position has slippery ice (hack.c:1422)
///
/// Returns true if player should slip on the ice.
pub fn check_ice_slip(state: &mut GameState, x: i8, y: i8) -> bool {
    if !state.current_level.is_valid_pos(x, y) {
        return false;
    }

    let cell = state.current_level.cell(x as usize, y as usize);
    if !cell.is_ice() {
        return false;
    }

    // Levitation, flying, cold resistance help
    if state.player.properties.has(Property::Levitation)
        || state.player.properties.has(Property::Flying)
        || state.player.properties.has(Property::ColdResistance)
    {
        return false;
    }

    // 50% chance to slip (1/3 with cold resistance, but we already returned above)
    state.rng.rn2(2) == 0
}

/// Handle effects when player steps on ice
pub fn ice_slip_effects(state: &mut GameState) {
    state.message("You slip on the ice!");
    // Slipping costs a turn but doesn't deal damage in vanilla NetHack
    // (it causes fumbling which may lead to falling later)
}

/// Check special room effects at current position (hack.c:check_special_room)
pub fn check_special_room(state: &mut GameState) {
    let px = state.player.pos.x;
    let py = state.player.pos.y;

    if !state.current_level.is_valid_pos(px, py) {
        return;
    }

    let cell_typ = state.current_level.cell(px as usize, py as usize).typ;

    match cell_typ {
        CellType::Altar => {
            let alignment = state.player.alignment.typ;
            state.message(format!(
                "There is an altar to {} here.",
                alignment.default_god()
            ));
        }
        CellType::Grave => {
            state.message("You are standing on a grave.");
        }
        CellType::Throne => {
            state.message("There is a throne here.");
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::{CellType, Level, TrapType};
    use crate::object::{Object, ObjectClass, ObjectId, ObjectLocation};
    use crate::rng::GameRng;

    fn make_test_state() -> GameState {
        let mut state = GameState::new(GameRng::new(42));
        // Set up a small room area (3..=7), surrounded by walls at 2 and 8
        for x in 3..=7 {
            for y in 3..=7 {
                state.current_level.cell_mut(x, y).typ = CellType::Room;
            }
        }
        // Ensure walls at boundaries so boulder-blocking tests are deterministic
        for i in 2..=8 {
            state.current_level.cell_mut(i, 2).typ = CellType::HWall;
            state.current_level.cell_mut(i, 8).typ = CellType::HWall;
            state.current_level.cell_mut(2, i).typ = CellType::VWall;
            state.current_level.cell_mut(8, i).typ = CellType::VWall;
        }
        state.player.pos.x = 5;
        state.player.pos.y = 5;
        state
    }

    fn place_boulder(level: &mut Level, x: i8, y: i8) -> ObjectId {
        let mut boulder = Object::default();
        boulder.object_type = BOULDER;
        boulder.class = ObjectClass::Rock;
        boulder.weight = 6000;
        boulder.location = ObjectLocation::Floor;
        level.add_object(boulder, x, y)
    }

    #[test]
    fn test_moverock_no_boulder() {
        let mut state = make_test_state();
        let result = moverock(&mut state, 6, 5, 1, 0);
        assert_eq!(result, MoveRockResult::Moved);
    }

    #[test]
    fn test_moverock_push_into_room() {
        let mut state = make_test_state();
        let _bid = place_boulder(&mut state.current_level, 6, 5);

        let result = moverock(&mut state, 6, 5, 1, 0);
        assert_eq!(result, MoveRockResult::Moved);

        // Boulder should now be at (7,5)
        assert!(find_boulder(&state.current_level, 7, 5).is_some());
        // No longer at (6,5)
        assert!(find_boulder(&state.current_level, 6, 5).is_none());
    }

    #[test]
    fn test_moverock_blocked_by_wall() {
        let mut state = make_test_state();
        // Wall at (8,5) — default stone
        let _bid = place_boulder(&mut state.current_level, 7, 5);

        let result = moverock(&mut state, 7, 5, 1, 0);
        assert_eq!(result, MoveRockResult::Blocked);

        // Boulder still at (7,5)
        assert!(find_boulder(&state.current_level, 7, 5).is_some());
    }

    #[test]
    fn test_moverock_blocked_by_boulder() {
        let mut state = make_test_state();
        place_boulder(&mut state.current_level, 6, 5);
        place_boulder(&mut state.current_level, 7, 5);

        let result = moverock(&mut state, 6, 5, 1, 0);
        assert_eq!(result, MoveRockResult::Blocked);
    }

    #[test]
    fn test_moverock_into_pit() {
        let mut state = make_test_state();
        let _bid = place_boulder(&mut state.current_level, 6, 5);
        state.current_level.add_trap(7, 5, TrapType::Pit);

        let result = moverock(&mut state, 6, 5, 1, 0);
        assert_eq!(result, MoveRockResult::Moved);

        // Boulder consumed, pit removed
        assert!(find_boulder(&state.current_level, 6, 5).is_none());
        assert!(find_boulder(&state.current_level, 7, 5).is_none());
        assert!(state.current_level.trap_at(7, 5).is_none());
    }

    #[test]
    fn test_moverock_into_water() {
        let mut state = make_test_state();
        place_boulder(&mut state.current_level, 6, 5);
        state.current_level.cell_mut(7, 5).typ = CellType::Pool;

        let result = moverock(&mut state, 6, 5, 1, 0);
        assert_eq!(result, MoveRockResult::Moved);

        // Boulder consumed, pool filled
        assert!(find_boulder(&state.current_level, 6, 5).is_none());
        assert!(find_boulder(&state.current_level, 7, 5).is_none());
        assert_eq!(
            state.current_level.cell(7, 5).typ,
            CellType::Room
        );
    }

    #[test]
    fn test_moverock_into_lava() {
        let mut state = make_test_state();
        place_boulder(&mut state.current_level, 6, 5);
        state.current_level.cell_mut(7, 5).typ = CellType::Lava;

        let result = moverock(&mut state, 6, 5, 1, 0);
        assert_eq!(result, MoveRockResult::Moved);

        // Boulder consumed, lava cooled
        assert_eq!(
            state.current_level.cell(7, 5).typ,
            CellType::Room
        );
    }

    #[test]
    fn test_moverock_levitating_blocked() {
        let mut state = make_test_state();
        place_boulder(&mut state.current_level, 6, 5);
        state.player.properties.grant_intrinsic(Property::Levitation);

        let result = moverock(&mut state, 6, 5, 1, 0);
        assert_eq!(result, MoveRockResult::Blocked);
    }

    #[test]
    fn test_dosinkfall_flying() {
        let mut state = make_test_state();
        let hp_before = state.player.hp;
        state.player.properties.grant_intrinsic(Property::Flying);

        dosinkfall(&mut state);
        assert_eq!(state.player.hp, hp_before); // no damage
    }

    #[test]
    fn test_dosinkfall_damage() {
        let mut state = make_test_state();
        state.player.hp = 100;
        state.player.hp_max = 100;

        dosinkfall(&mut state);
        assert!(state.player.hp < 100); // took damage
    }

    #[test]
    fn test_check_movement_capacity_overloaded() {
        let state = make_test_state();
        // Default state should be unencumbered
        assert!(check_movement_capacity(&state).is_none());
    }

    #[test]
    fn test_confdir_not_confused() {
        let mut state = make_test_state();
        state.player.confused_timeout = 0;
        state.player.stunned_timeout = 0;

        let (dx, dy) = confdir(&mut state, 1, 0);
        assert_eq!((dx, dy), (1, 0)); // unchanged
    }

    #[test]
    fn test_can_receive_boulder() {
        assert!(can_receive_boulder(CellType::Room));
        assert!(can_receive_boulder(CellType::Corridor));
        assert!(can_receive_boulder(CellType::Pool));
        assert!(can_receive_boulder(CellType::Lava));
        assert!(!can_receive_boulder(CellType::Stone));
        assert!(!can_receive_boulder(CellType::Wall));
    }

    #[test]
    fn test_check_ice_slip_no_ice() {
        let mut state = make_test_state();
        assert!(!check_ice_slip(&mut state, 5, 5)); // Room, not ice
    }

    #[test]
    fn test_check_ice_slip_levitating() {
        let mut state = make_test_state();
        state.current_level.cell_mut(5, 5).typ = CellType::Ice;
        state.player.properties.grant_intrinsic(Property::Levitation);

        assert!(!check_ice_slip(&mut state, 5, 5)); // immune
    }
}
