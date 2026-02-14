//! Fountain interactions (fountain.c)

use crate::action::ActionResult;
use crate::dungeon::CellType;
use crate::gameloop::GameState;
use crate::player::Property;

/// Drink from a fountain at the player's current position
pub fn drinkfountain(state: &mut GameState) -> ActionResult {
    let player_x = state.player.pos.x;
    let player_y = state.player.pos.y;

    // Check if actually at a fountain
    let cell_type = state
        .current_level
        .cell(player_x as usize, player_y as usize)
        .typ;
    if cell_type != CellType::Fountain {
        state.message("There is no fountain here.");
        return ActionResult::NoTime;
    }

    // Check for levitation
    if state.player.properties.has(Property::Levitation) {
        state.message("You are floating high above the fountain.");
        return ActionResult::NoTime;
    }

    // Check if fountain is blessed (magic fountain)
    let is_magic = state
        .current_level
        .cell(player_x as usize, player_y as usize)
        .flags
        & 0x01
        != 0;

    // Roll fate
    let fate = state.rng.rnd(30);

    // Magic fountain with good luck has special effects
    if is_magic && state.player.luck >= 0 && fate >= 10 {
        state.message("Wow! This makes you feel great!");

        // Restore all attributes to maximum
        for attr in [
            crate::player::Attribute::Strength,
            crate::player::Attribute::Dexterity,
            crate::player::Attribute::Constitution,
            crate::player::Attribute::Intelligence,
            crate::player::Attribute::Wisdom,
            crate::player::Attribute::Charisma,
        ] {
            let max_val = state.player.attr_max.get(attr);
            state.player.attr_current.set(attr, max_val);
        }

        // Gain a random attribute point
        let attr_idx = state.rng.rn2(6);
        let attr = match attr_idx {
            0 => crate::player::Attribute::Strength,
            1 => crate::player::Attribute::Dexterity,
            2 => crate::player::Attribute::Constitution,
            3 => crate::player::Attribute::Intelligence,
            4 => crate::player::Attribute::Wisdom,
            _ => crate::player::Attribute::Charisma,
        };
        let current = state.player.attr_current.get(attr);
        let max_val = state.player.attr_max.get(attr);
        if current < max_val {
            state.player.attr_current.set(attr, current + 1);
        }

        state.message("A wisp of vapor escapes the fountain...");

        // Remove magic from fountain
        state
            .current_level
            .cell_mut(player_x as usize, player_y as usize)
            .flags &= !0x01;
        return ActionResult::Success;
    }

    // Normal fountain effects based on fate
    if fate < 10 {
        state.message("The cool draught refreshes you.");
        state.player.nutrition += state.rng.rnd(10) as i32;
        return ActionResult::Success;
    }

    match fate {
        10..=18 => {
            state.message("The cool draught refreshes you.");
            state.player.nutrition += state.rng.rnd(10) as i32;
        }
        19 => {
            // Self-knowledge
            state.message("You feel self-knowledgeable...");
            state.message("You know yourself better now.");
        }
        20 => {
            // Foul water
            state.message("The water is foul! You gag and vomit.");
            // rn1(20, 11) = 11 + rn2(20), gives 11-30
            state.player.nutrition -= (11 + state.rng.rn2(20)) as i32;
        }
        21 => {
            // Poisonous water
            state.message("The water is contaminated!");
            if state.player.properties.has(Property::PoisonResistance) {
                state.message("Perhaps it is runoff from a nearby farm.");
                state.player.take_damage(state.rng.rnd(4) as i32);
            } else {
                // rn1(4, 3) = 3 + rn2(4), gives 3-6
                let str_loss = (3 + state.rng.rn2(4)) as i8;
                let current_str = state
                    .player
                    .attr_current
                    .get(crate::player::Attribute::Strength);
                state.player.attr_current.set(
                    crate::player::Attribute::Strength,
                    current_str.saturating_sub(str_loss),
                );
                state.player.take_damage(state.rng.rnd(10) as i32);
            }
        }
        22 => {
            // Fountain of snakes
            dowatersnakes(state);
        }
        23 => {
            // Water demon
            dowaterdemon(state);
        }
        24 => {
            // Curse items
            state.message("This water's no good!");
            // rn1(20, 11) = 11 + rn2(20), gives 11-30
            state.player.nutrition -= (11 + state.rng.rn2(20)) as i32;
            // Would curse some inventory items
            for obj in &mut state.inventory {
                if state.rng.rn2(5) == 0 {
                    obj.buc = crate::object::BucStatus::Cursed;
                }
            }
        }
        25 => {
            // See invisible
            if state.player.is_blind() {
                state.message("You feel very self-conscious.");
            } else {
                state.message("You see an image of someone stalking you.");
                state.message("But it disappears.");
            }
            state
                .player
                .properties
                .grant_intrinsic(Property::SeeInvisible);
        }
        26 => {
            // See monsters
            state.message("You sense the presence of monsters!");
            // Would reveal monsters on the level
        }
        27 => {
            // Find gem
            let looted = state
                .current_level
                .cell(player_x as usize, player_y as usize)
                .flags
                & 0x02
                != 0;
            if !looted {
                dofindgem(state);
            } else {
                dowaternymph(state);
            }
        }
        28 => {
            // Water nymph
            dowaternymph(state);
        }
        29 => {
            // Scare monsters
            state.message("This water gives you bad breath!");
            for mon in state.current_level.monsters.iter_mut() {
                mon.state.fleeing = true;
                mon.flee_timeout = 20;
            }
        }
        _ => {
            // Gush
            dogushforth(state, true);
        }
    }

    // Chance to dry up fountain
    dryup(state, player_x, player_y, true);

    ActionResult::Success
}

/// Dip an item into a fountain
pub fn dipfountain(state: &mut GameState, obj_letter: char) -> ActionResult {
    let player_x = state.player.pos.x;
    let player_y = state.player.pos.y;

    // Check if actually at a fountain
    let cell_type = state
        .current_level
        .cell(player_x as usize, player_y as usize)
        .typ;
    if cell_type != CellType::Fountain {
        state.message("There is no fountain here.");
        return ActionResult::NoTime;
    }

    // Check for levitation
    if state.player.properties.has(Property::Levitation) {
        state.message("You are floating high above the fountain.");
        return ActionResult::NoTime;
    }

    // Find the object
    let obj_idx = match state
        .inventory
        .iter()
        .position(|o| o.inv_letter == obj_letter)
    {
        Some(idx) => idx,
        None => {
            state.message("You don't have that item.");
            return ActionResult::NoTime;
        }
    };

    let obj = &state.inventory[obj_idx];
    let obj_name = obj.display_name().to_string();
    let obj_class = obj.class;

    state.message(format!("You dip {} into the fountain.", obj_name));

    // Different effects based on object type
    use crate::object::ObjectClass;
    match obj_class {
        ObjectClass::Potion => {
            // Potion dilution or fountain effect
            let fate = state.rng.rn2(30);
            if fate < 10 {
                state.message("The potion dilutes!");
                // Would dilute the potion
            } else if fate < 20 {
                state.message("The potion becomes water!");
                // Would convert to potion of water
            } else {
                state.message("A geyser of steam erupts from the fountain!");
                state.player.take_damage(state.rng.rnd(10) as i32);
            }
        }
        ObjectClass::Weapon | ObjectClass::Armor => {
            // Rust or bless/curse
            let fate = state.rng.rn2(10);
            if fate < 3 {
                state.message("The water glows for a moment.");
                // Would affect blessed/cursed status
            } else {
                state.message("The water washes over your item.");
            }
        }
        _ => {
            state.message("You wash the item.");
        }
    }

    // Chance to dry up
    dryup(state, player_x, player_y, true);

    ActionResult::Success
}

/// Water gushes forth from the fountain
pub fn dogushforth(state: &mut GameState, drinking: bool) {
    let player_x = state.player.pos.x;
    let player_y = state.player.pos.y;

    let mut made_pool = false;

    // Create pools in a radius
    for dx in -7..=7i8 {
        for dy in -7..=7i8 {
            let x = player_x as i32 + dx as i32;
            let y = player_y as i32 + dy as i32;

            if x < 0 || y < 0 || x > 127 || y > 127 {
                continue;
            }

            made_pool |= gush(state, x as i8, y as i8);
        }
    }

    if !made_pool {
        if drinking {
            state.message("Your thirst is quenched.");
        } else {
            state.message("Water sprays all over you.");
        }
    }
}

/// Create a pool at a location if possible
fn gush(state: &mut GameState, x: i8, y: i8) -> bool {
    if !state.current_level.is_valid_pos(x, y) {
        return false;
    }

    // Various conditions that prevent pool creation
    let cell = state.current_level.cell(x as usize, y as usize);

    // Only create pools in rooms
    if cell.typ != CellType::Room {
        return false;
    }

    // Skip player position
    if x == state.player.pos.x && y == state.player.pos.y {
        return false;
    }

    // Random chance based on distance
    let player_x = state.player.pos.x;
    let player_y = state.player.pos.y;
    let dist = ((x - player_x).abs() + (y - player_y).abs()) as u32;
    if state.rng.rn2(1 + dist) != 0 {
        return false;
    }

    // Create pool
    state.current_level.cell_mut(x as usize, y as usize).typ = CellType::Pool;
    state.message("Water gushes forth from the overflowing fountain!");

    true
}

/// Fountain dries up
pub fn dryup(state: &mut GameState, x: i8, y: i8, is_player: bool) {
    if !state.current_level.is_valid_pos(x, y) {
        return;
    }

    let cell = state.current_level.cell(x as usize, y as usize);
    if cell.typ != CellType::Fountain {
        return;
    }

    // Check if warned already (flag bit 0x04)
    let warned = cell.flags & 0x04 != 0;

    // 1/3 chance to dry up, or always if warned
    if state.rng.rn2(3) != 0 && !warned {
        return;
    }

    // First warning if in town
    if is_player && !warned {
        // Would check if in town and warn
        state.current_level.cell_mut(x as usize, y as usize).flags |= 0x04;
        state.message("The flow reduces to a trickle.");
        return;
    }

    // Replace fountain with floor
    state.current_level.cell_mut(x as usize, y as usize).typ = CellType::Room;
    state.current_level.cell_mut(x as usize, y as usize).flags = 0;
    state.message("The fountain dries up!");
}

/// Spawn water snakes from fountain
pub fn dowatersnakes(state: &mut GameState) {
    // rn1(5, 2) = 2 + rn2(5), gives 2-6 snakes
    let num_snakes = 2 + state.rng.rn2(5);

    if state.player.is_blind() {
        state.message("You hear hissing!");
    } else {
        state.message("An endless stream of snakes pours forth!");
    }

    // Would spawn water moccasins here
    for _ in 0..num_snakes {
        state.message("A snake appears!");
        // In full implementation, would create monster
    }
}

/// Spawn water demon from fountain
pub fn dowaterdemon(state: &mut GameState) {
    if state.player.is_blind() {
        state.message("You feel the presence of evil.");
    } else {
        state.message("You unleash a water demon!");
    }

    // Low level characters might get a wish
    let difficulty = state.player.exp_level as i32;
    if state.rng.rnd(100) as i32 > 80 + difficulty {
        state.message("Grateful for its release, the demon grants you a wish!");
        // Would handle wish granting
    } else {
        state.message("The water demon attacks!");
        // Would create hostile water demon
    }
}

/// Spawn water nymph from fountain
pub fn dowaternymph(state: &mut GameState) {
    if state.player.is_blind() {
        state.message("You hear a seductive voice.");
    } else {
        state.message("You attract a water nymph!");
    }

    // Would create water nymph monster
}

/// Find a gem in the fountain
fn dofindgem(state: &mut GameState) {
    let player_x = state.player.pos.x;
    let player_y = state.player.pos.y;

    if state.player.is_blind() {
        state.message("You feel a gem here!");
    } else {
        state.message("You spot a gem in the sparkling waters!");
    }

    // Would create random gem at player position
    // Mark fountain as looted
    state
        .current_level
        .cell_mut(player_x as usize, player_y as usize)
        .flags |= 0x02;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::GameRng;

    #[test]
    fn test_drinkfountain_no_fountain() {
        let mut state = GameState::new(GameRng::from_entropy());
        // Player starts at position without fountain
        let result = drinkfountain(&mut state);
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_dryup_not_fountain() {
        let mut state = GameState::new(GameRng::from_entropy());
        // dryup on non-fountain should do nothing
        dryup(&mut state, 5, 5, true);
        // No assertion needed, just shouldn't panic
    }
}
