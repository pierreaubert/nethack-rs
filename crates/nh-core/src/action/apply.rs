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
            state.message("You produce a high-pitched humming noise.");
            ActionResult::Success
        }
        191 => {
            // Magic whistle - pets come to you
            state.message("You produce a strange whistling sound.");
            ActionResult::Success
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
            state.message("You see your reflection.");
            ActionResult::Success
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
