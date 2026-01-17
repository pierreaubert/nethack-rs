//! Zapping wands (zap.c)

use crate::action::{ActionResult, Direction};
use crate::gameloop::GameState;
use crate::magic::zap::{zap_wand, ZapDirection};
use crate::object::ObjectClass;

/// Zap a wand from inventory
pub fn do_zap(state: &mut GameState, obj_letter: char, direction: Option<Direction>) -> ActionResult {
    // Find the wand index in inventory
    let wand_idx = match state.inventory.iter().position(|o| o.inv_letter == obj_letter) {
        Some(idx) => idx,
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    if state.inventory[wand_idx].class != ObjectClass::Wand {
        return ActionResult::Failed("That's not something you can zap.".to_string());
    }

    // Convert Direction to ZapDirection using delta
    let dir = direction.unwrap_or(Direction::East);
    let zap_dir = match dir {
        Direction::Up => ZapDirection::Up,
        Direction::Down => ZapDirection::Down,
        Direction::Self_ => ZapDirection::Self_,
        _ => {
            let (dx, dy) = dir.delta();
            ZapDirection::Direction(dx, dy)
        }
    };

    // Apply zap effects - need to temporarily remove wand to satisfy borrow checker
    let mut wand = state.inventory.remove(wand_idx);
    
    let result = zap_wand(
        &mut wand,
        zap_dir,
        &mut state.player,
        &mut state.current_level,
        &mut state.rng,
    );

    // Put wand back in inventory
    state.inventory.insert(wand_idx, wand);

    // Display messages
    for msg in result.messages {
        state.message(msg);
    }

    // Remove killed monsters
    for monster_id in result.killed {
        state.current_level.remove_monster(monster_id);
    }

    ActionResult::Success
}
