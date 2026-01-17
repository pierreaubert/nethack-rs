//! Throwing objects (dothrow.c)

use crate::action::{ActionResult, Direction};
use crate::gameloop::GameState;
use crate::object::ObjectClass;

/// Throw an object from inventory
pub fn do_throw(state: &mut GameState, obj_letter: char, direction: Direction) -> ActionResult {
    // Get the object from inventory
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o.clone(),
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    // Calculate throw range based on strength and object weight
    let str_bonus = state.player.attr_current.get(crate::player::Attribute::Strength) as i32;
    let weight_penalty = (obj.weight as i32 / 40).min(10);
    let max_range = ((str_bonus / 2) - weight_penalty).clamp(1, 10) as i8;

    let (dx, dy) = direction.delta();
    
    // Can't throw at self
    if dx == 0 && dy == 0 {
        state.message("You can't throw something at yourself.");
        return ActionResult::NoTime;
    }

    let obj_name = obj.name.clone().unwrap_or_else(|| "object".to_string());

    // Remove from inventory (or reduce quantity)
    if obj.quantity > 1 {
        if let Some(inv_obj) = state.get_inventory_item_mut(obj_letter) {
            inv_obj.quantity -= 1;
        }
    } else {
        state.remove_from_inventory(obj_letter);
    }

    // Trace the projectile path
    let mut x = state.player.pos.x;
    let mut y = state.player.pos.y;
    let mut hit_monster = false;
    let mut final_x = x;
    let mut final_y = y;

    let mut monster_hit_info: Option<(crate::monster::MonsterId, String, bool, i32)> = None;
    
    for _ in 0..max_range {
        x += dx;
        y += dy;

        if !state.current_level.is_valid_pos(x, y) {
            break;
        }

        // Check for monster at this position
        if let Some(monster) = state.current_level.monster_at(x, y) {
            // Calculate hit chance
            let to_hit = 10 + state.player.exp_level - monster.ac as i32;
            let roll = state.rng.rnd(20) as i32;
            let monster_name = monster.name.clone();
            let monster_id = monster.id;

            if roll <= to_hit {
                // Hit!
                let damage = calculate_throw_damage(&obj, &mut state.rng);
                monster_hit_info = Some((monster_id, monster_name, true, damage));
            } else {
                monster_hit_info = Some((monster_id, monster_name, false, 0));
            }
            final_x = x;
            final_y = y;
            hit_monster = true;
            break;
        }

        // Check for walls
        if !state.current_level.is_walkable(x, y) {
            state.message(format!("The {} hits the wall.", obj_name));
            // Object lands at previous position
            final_x = x - dx;
            final_y = y - dy;
            break;
        }

        final_x = x;
        final_y = y;
    }

    // Apply monster hit effects after the loop
    if let Some((monster_id, monster_name, did_hit, damage)) = monster_hit_info {
        if did_hit {
            if let Some(monster) = state.current_level.monster_mut(monster_id) {
                monster.hp -= damage;
                let killed = monster.hp <= 0;
                state.message(format!("The {} hits the {}!", obj_name, monster_name));
                if killed {
                    state.message(format!("You kill the {}!", monster_name));
                    state.current_level.remove_monster(monster_id);
                }
            }
        } else {
            state.message(format!("The {} misses the {}.", obj_name, monster_name));
        }
    }

    // If didn't hit anything, object lands at final position
    if !hit_monster {
        // Create a copy of the thrown object on the ground
        let mut thrown_obj = obj;
        thrown_obj.quantity = 1;
        thrown_obj.x = final_x;
        thrown_obj.y = final_y;
        state.current_level.add_object(thrown_obj, final_x, final_y);
        state.message(format!("The {} lands on the ground.", obj_name));
    }

    ActionResult::Success
}

/// Calculate damage for thrown object
fn calculate_throw_damage(obj: &crate::object::Object, rng: &mut crate::rng::GameRng) -> i32 {
    let base_damage = match obj.class {
        ObjectClass::Weapon => {
            // Use weapon damage dice
            let dice = obj.damage_dice.max(1);
            let sides = obj.damage_sides.max(4);
            rng.dice(dice as u32, sides as u32) as i32
        }
        ObjectClass::Gem | ObjectClass::Rock => {
            // Rocks and gems do 1d3
            rng.dice(1, 3) as i32
        }
        ObjectClass::Potion => {
            // Potions shatter for 1 damage
            1
        }
        _ => {
            // Other objects do 1d2
            rng.dice(1, 2) as i32
        }
    };

    // Add enchantment bonus
    base_damage + obj.enchantment.max(0) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{Object, ObjectId};

    #[test]
    fn test_throw_damage_weapon() {
        let mut rng = crate::rng::GameRng::new(12345);
        let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        obj.damage_dice = 2;
        obj.damage_sides = 6;
        obj.enchantment = 1;

        let damage = calculate_throw_damage(&obj, &mut rng);
        assert!(damage >= 3 && damage <= 13); // 2d6 + 1
    }

    #[test]
    fn test_throw_damage_rock() {
        let mut rng = crate::rng::GameRng::new(12345);
        let obj = Object::new(ObjectId(1), 0, ObjectClass::Rock);

        let damage = calculate_throw_damage(&obj, &mut rng);
        assert!(damage >= 1 && damage <= 3); // 1d3
    }
}
