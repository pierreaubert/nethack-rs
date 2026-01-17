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

    state.message("You kick at nothing.");
    ActionResult::Success
}

/// Kick a monster - deals damage based on stats
fn kick_monster(state: &mut GameState, monster_id: crate::monster::MonsterId) -> ActionResult {
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
    let actual_dmg = state.rng.rnd(dmg as u32) as i32;
    
    // Get monster name before mutating
    let monster_name = state.current_level.monster(monster_id)
        .map(|m| m.name.clone())
        .unwrap_or_else(|| "monster".to_string());
    
    // Apply damage to monster
    if let Some(monster) = state.current_level.monster_mut(monster_id) {
        monster.hp -= actual_dmg;
        
        // Anger the monster
        monster.state.peaceful = false;
        
        if monster.hp <= 0 {
            state.message(format!("You kick the {} to death!", monster_name));
            // Monster death handled elsewhere
        } else if clumsy {
            state.message(format!("Your clumsy kick hits the {} for {} damage.", monster_name, actual_dmg));
        } else {
            state.message(format!("You kick the {} for {} damage!", monster_name, actual_dmg));
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
    
    // Calculate kick strength: (STR + DEX + CON) / 3
    let str_val = state.player.attr_current.get(Attribute::Strength) as u32;
    let dex = state.player.attr_current.get(Attribute::Dexterity) as u32;
    let con = state.player.attr_current.get(Attribute::Constitution) as u32;
    let kick_power = (str_val + dex + con) / 3;
    
    // Locked doors are harder to break
    let difficulty = if door_state.contains(DoorState::LOCKED) { 30 } else { 20 };
    
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
