//! Tool application (apply.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::object::ObjectClass;

/// Apply a tool from inventory
pub fn do_apply(state: &mut GameState, obj_letter: char) -> ActionResult {
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o.clone(),
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    if obj.class != ObjectClass::Tool {
        return ActionResult::Failed("That's not something you can apply.".to_string());
    }

    let obj_name = obj.name.clone().unwrap_or_else(|| "tool".to_string());

    // Handle different tool types based on object_type
    // Tool object types from objects.c (approximate ranges)
    match obj.object_type {
        // Pickaxe (176) and mattock (177)
        176 | 177 => {
            apply_pickaxe(state, &obj_name)
        }
        // Lamp (188) and lantern (189)
        188 | 189 => {
            apply_light(state, obj_letter, &obj_name)
        }
        // Whistle (190) and magic whistle (191)
        190 => {
            apply_whistle(state)
        }
        191 => {
            apply_magic_whistle(state)
        }
        // Horn (195) - tooled horn
        195 => {
            state.message("You produce a loud noise!");
            // Wake up nearby monsters
            for monster in &mut state.current_level.monsters {
                monster.state.sleeping = false;
            }
            ActionResult::Success
        }
        // Horn of plenty (196)
        196 => {
            apply_horn_of_plenty(state, obj_letter)
        }
        // Stethoscope (200)
        200 => {
            apply_stethoscope(state)
        }
        // Mirror (201)
        201 => {
            apply_mirror(state)
        }
        // Tinning kit (203)
        203 => {
            state.message("You need a corpse to tin.");
            ActionResult::NoTime
        }
        // Skeleton key (205) and lock pick (206)
        205 | 206 => {
            state.message("You need to apply this to a door or container.");
            ActionResult::NoTime
        }
        // Unicorn horn (213)
        213 => {
            apply_unicorn_horn(state)
        }
        // Candles (221-222)
        221 | 222 => {
            apply_light(state, obj_letter, &obj_name)
        }
        _ => {
            state.message(format!("You apply the {}.", obj_name));
            ActionResult::Success
        }
    }
}

/// Apply a pickaxe or mattock for digging
fn apply_pickaxe(state: &mut GameState, obj_name: &str) -> ActionResult {
    state.message(format!("You swing the {}.", obj_name));
    // Digging requires direction selection - for now just message
    state.message("In what direction do you want to dig?");
    ActionResult::NoTime
}

/// Apply a light source (lamp, lantern, candle)
fn apply_light(state: &mut GameState, obj_letter: char, obj_name: &str) -> ActionResult {
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        if obj.lit {
            obj.lit = false;
            state.message(format!("You extinguish the {}.", obj_name));
        } else {
            obj.lit = true;
            state.message(format!("The {} is now lit.", obj_name));
        }
    }
    ActionResult::Success
}

/// Apply horn of plenty - creates food
fn apply_horn_of_plenty(state: &mut GameState, obj_letter: char) -> ActionResult {
    if let Some(obj) = state.get_inventory_item_mut(obj_letter) {
        if obj.enchantment > 0 {
            obj.enchantment -= 1;
            state.player.nutrition += 600;
            state.message("Food spills out of the horn!");
            ActionResult::Success
        } else {
            state.message("The horn is empty.");
            ActionResult::NoTime
        }
    } else {
        ActionResult::NoTime
    }
}

/// Apply stethoscope - examine monster or self
fn apply_stethoscope(state: &mut GameState) -> ActionResult {
    // Check adjacent squares for monsters
    let px = state.player.pos.x;
    let py = state.player.pos.y;
    
    for dy in -1i8..=1 {
        for dx in -1i8..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            if let Some(monster) = state.current_level.monster_at(px + dx, py + dy) {
                state.message(format!(
                    "{}: HP {}/{}, AC {}, Level {}",
                    monster.name, monster.hp, monster.hp_max, monster.ac, monster.level
                ));
                return ActionResult::Success;
            }
        }
    }
    
    // No monster found, examine self
    state.message(format!(
        "You: HP {}/{}, AC {}, Level {}",
        state.player.hp, state.player.hp_max,
        state.calculate_armor_class(), state.player.exp_level
    ));
    ActionResult::Success
}

/// Apply unicorn horn - cure ailments
fn apply_unicorn_horn(state: &mut GameState) -> ActionResult {
    let mut cured = false;
    
    if state.player.confused_timeout > 0 {
        state.player.confused_timeout = 0;
        state.message("Your head clears.");
        cured = true;
    }
    if state.player.stunned_timeout > 0 {
        state.player.stunned_timeout = 0;
        state.message("You feel steadier.");
        cured = true;
    }
    if state.player.blinded_timeout > 0 {
        state.player.blinded_timeout = 0;
        state.message("Your vision clears.");
        cured = true;
    }
    if state.player.hallucinating_timeout > 0 {
        state.player.hallucinating_timeout = 0;
        state.message("Everything looks normal again.");
        cured = true;
    }
    
    if !cured {
        state.message("You feel healthy.");
    }
    
    ActionResult::Success
}

/// Apply a whistle - wakes nearby monsters (from C: use_whistle)
fn apply_whistle(state: &mut GameState) -> ActionResult {
    state.message("You produce a high-pitched humming noise.");
    
    // Wake up nearby monsters (within ~10 squares)
    let px = state.player.pos.x;
    let py = state.player.pos.y;
    
    for monster in &mut state.current_level.monsters {
        let dx = (monster.x - px).abs();
        let dy = (monster.y - py).abs();
        if dx <= 10 && dy <= 10 {
            monster.state.sleeping = false;
        }
    }
    
    ActionResult::Success
}

/// Apply a magic whistle - pets teleport to you (from C: use_magic_whistle)
fn apply_magic_whistle(state: &mut GameState) -> ActionResult {
    state.message("You produce a strange whistling sound.");
    
    let px = state.player.pos.x;
    let py = state.player.pos.y;
    let mut pets_moved = 0;
    
    // Find tame monsters and move them adjacent to player
    let tame_ids: Vec<_> = state.current_level.monsters
        .iter()
        .filter(|m| m.state.tame)
        .map(|m| m.id)
        .collect();
    
    for monster_id in tame_ids {
        // Find an adjacent empty square
        for dy in -1i8..=1 {
            for dx in -1i8..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = px + dx;
                let ny = py + dy;
                
                if state.current_level.is_walkable(nx, ny) 
                    && state.current_level.monster_at(nx, ny).is_none() 
                {
                    // Move the pet here
                    if let Some(monster) = state.current_level.monster_mut(monster_id) {
                        monster.x = nx;
                        monster.y = ny;
                        monster.state.sleeping = false;
                        pets_moved += 1;
                    }
                    break;
                }
            }
        }
    }
    
    if pets_moved > 0 {
        state.message(format!("{} pet(s) come to you.", pets_moved));
    }
    
    ActionResult::Success
}

/// Apply a mirror - look at yourself or scare monsters (from C: use_mirror)
fn apply_mirror(state: &mut GameState) -> ActionResult {
    // Looking at self (no direction specified)
    let is_sick = state.player.hp < state.player.hp_max / 4;
    let is_hungry = state.player.nutrition < 150;
    let is_hallucinating = state.player.hallucinating_timeout > 0;
    
    if is_hallucinating {
        state.message("You look groovy.");
    } else if is_sick {
        state.message("You look peaked.");
    } else if is_hungry {
        state.message("You look undernourished.");
    } else {
        state.message("You look as beautiful as ever.");
    }
    
    ActionResult::Success
}

/// Apply a mirror in a direction - can scare monsters
pub fn apply_mirror_at(state: &mut GameState, x: i8, y: i8) -> ActionResult {
    // Check for monster at target
    if let Some(monster) = state.current_level.monster_at_mut(x, y) {
        // Monsters with eyes can be scared by mirrors
        // Nymphs and other vain monsters are especially affected
        let monster_name = monster.name.clone();
        
        if state.rng.one_in(3) {
            monster.state.fleeing = true;
            monster.flee_timeout = state.rng.dice(2, 6) as u16;
            state.message(format!("The {} is frightened by its reflection!", monster_name));
        } else {
            state.message(format!("The {} ignores the mirror.", monster_name));
        }
    } else {
        state.message("You reflect the empty space.");
    }
    
    ActionResult::Success
}
