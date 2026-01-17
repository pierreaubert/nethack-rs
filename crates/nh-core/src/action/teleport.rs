//! Teleportation mechanics (teleport.c)
//!
//! From NetHack C:
//! - tele(): Random teleport on current level
//! - level_tele(): Teleport to different dungeon level
//! - Teleport control allows choosing destination
//! - Amulet of Yendor blocks level teleport in endgame

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::player::Property;
use crate::{COLNO, ROWNO};

/// Teleport the player randomly on the current level
pub fn tele(state: &mut GameState) -> ActionResult {
    let has_control = state.player.properties.has(Property::TeleportControl);

    if has_control {
        state.message("You feel in control of the teleportation.");
    }

    // Save previous position
    state.player.prev_pos = state.player.pos;

    let (new_x, new_y) = find_teleport_destination(state);

    state.player.pos.x = new_x;
    state.player.pos.y = new_y;
    state.message("You feel disoriented.");

    ActionResult::Success
}

fn find_teleport_destination(state: &mut GameState) -> (i8, i8) {
    for _ in 0..100 {
        let x = state.rng.rn2(COLNO as u32) as i8;
        let y = state.rng.rn2(ROWNO as u32) as i8;

        if state.current_level.is_walkable(x, y) 
            && state.current_level.monster_at(x, y).is_none() 
        {
            return (x, y);
        }
    }

    (state.player.pos.x, state.player.pos.y)
}

/// Level teleport - teleport to a different dungeon level
/// Returns the target depth (positive = down, negative = up)
/// 
/// Note: This sets up the teleport but actual level change must be
/// handled by the game loop checking the returned ActionResult.
pub fn level_tele(state: &mut GameState, target_depth: i32) -> ActionResult {
    let current_depth = state.current_level.dlevel.depth();
    
    // Check for teleport control
    let has_control = state.player.properties.has(Property::TeleportControl);
    let is_stunned = state.player.stunned_timeout > 0;
    
    let new_depth = if has_control && !is_stunned {
        // Controlled teleport - use target depth
        if target_depth == 0 {
            // Random if no target specified
            random_teleport_level(state)
        } else {
            target_depth
        }
    } else {
        // Uncontrolled - random level
        random_teleport_level(state)
    };
    
    // Check if teleporting to same level
    if new_depth == current_depth {
        state.message("You shudder for a moment.");
        return ActionResult::NoTime;
    }
    
    // Check for going above ground (death by falling)
    if new_depth < 1 {
        if state.player.properties.has(Property::Levitation) {
            state.message("You float gently down to earth.");
            // Would teleport to level 1 - game loop handles this
        } else if state.player.properties.has(Property::Flying) {
            state.message("You fly down to the ground.");
            // Would teleport to level 1 - game loop handles this
        } else {
            state.message("You are now high above the clouds...");
            state.message("Unfortunately, you don't know how to fly.");
            state.message("You plummet a few thousand feet to your death.");
            state.player.hp = 0;
            return ActionResult::Success;
        }
    } else {
        // Normal level teleport
        state.message("You feel a wrenching sensation.");
        // Actual level change handled by game loop via ActionResult::LevelChange
    }
    
    ActionResult::Success
}

/// Calculate random teleport level (from C: random_teleport_level)
fn random_teleport_level(state: &mut GameState) -> i32 {
    let current_depth = state.current_level.dlevel.depth();
    let max_depth = 30; // Approximate max dungeon depth
    
    // Random level within dungeon bounds
    let range = max_depth.min(current_depth + 5);
    let min_level = 1.max(current_depth - 5);
    
    min_level + state.rng.rn2((range - min_level + 1) as u32) as i32
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
