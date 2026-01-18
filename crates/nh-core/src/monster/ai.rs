//! Monster AI (mon.c, monmove.c)
//!
//! Handles monster movement, pathfinding, and decision-making.

use crate::dungeon::Level;
use crate::player::You;
use crate::rng::GameRng;

use super::MonsterId;

/// AI action result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiAction {
    /// No action taken
    None,
    /// Moved to new position
    Moved(i8, i8),
    /// Attacked player
    AttackedPlayer,
    /// Waited/rested
    Waited,
}

/// Process monster AI for a single turn
///
/// Returns true if the monster took an action that consumed time
pub fn process_monster_ai(
    monster_id: MonsterId,
    level: &mut Level,
    player: &You,
    rng: &mut GameRng,
) -> AiAction {
    let monster = match level.monster_mut(monster_id) {
        Some(m) => m,
        None => return AiAction::None,
    };

    // Sleeping monsters have a chance to wake up if player is near
    if monster.state.sleeping {
        let dist_sq = monster.distance_sq(player.pos.x, player.pos.y);
        if dist_sq <= 16 {
            // Within 4 squares
            let wake_chance = 100 - (dist_sq as u32 * 5);
            if rng.percent(wake_chance) {
                monster.state.sleeping = false;
            } else {
                return AiAction::Waited;
            }
        } else {
            return AiAction::Waited;
        }
    }

    // Can't act if incapacitated (but not sleeping, that's handled above)
    if !monster.can_act() {
        return AiAction::Waited;
    }

    // Peaceful/tame monsters - pets follow player, others wander
    if monster.is_peaceful() {
        if monster.is_pet() {
            return pet_ai(monster_id, level, player, rng);
        }
        return wander_randomly(monster_id, level, rng);
    }

    // Hostile monster - pursue player
    let monster = level.monster(monster_id).unwrap();
    let px = player.pos.x;
    let py = player.pos.y;

    // If adjacent to player, attack
    if monster.is_adjacent(px, py) {
        return AiAction::AttackedPlayer;
    }

    // Move towards player
    move_towards(monster_id, level, px, py, rng)
}

/// Move monster towards a target position
fn move_towards(
    monster_id: MonsterId,
    level: &mut Level,
    target_x: i8,
    target_y: i8,
    rng: &mut GameRng,
) -> AiAction {
    let monster = level.monster(monster_id).unwrap();
    let mx = monster.x;
    let my = monster.y;

    // Calculate direction to target
    let dx = (target_x - mx).signum();
    let dy = (target_y - my).signum();

    // Confused monsters move randomly
    let (move_dx, move_dy) = if monster.state.confused {
        random_direction(rng)
    } else {
        (dx, dy)
    };

    let new_x = mx + move_dx;
    let new_y = my + move_dy;

    // Check if target position is valid and walkable
    if level.is_valid_pos(new_x, new_y) && level.is_walkable(new_x, new_y) {
        // Check if there's another monster there
        if level.monster_at(new_x, new_y).is_some() {
            // Can't move there, try alternative direction
            let alt_action = try_alternative_move(monster_id, level, dx, dy, rng);
            return alt_action;
        }

        // Move the monster
        level.move_monster(monster_id, new_x, new_y);
        AiAction::Moved(new_x, new_y)
    } else {
        // Can't move in desired direction, try alternative
        try_alternative_move(monster_id, level, dx, dy, rng)
    }
}

/// Try to find an alternative movement direction
fn try_alternative_move(
    monster_id: MonsterId,
    level: &mut Level,
    preferred_dx: i8,
    preferred_dy: i8,
    rng: &mut GameRng,
) -> AiAction {
    let monster = level.monster(monster_id).unwrap();
    let mx = monster.x;
    let my = monster.y;

    // Try diagonal movements if moving straight
    let alternatives: Vec<(i8, i8)> = if preferred_dx == 0 && preferred_dy != 0 {
        // Moving vertically, try diagonals
        vec![(1, preferred_dy), (-1, preferred_dy)]
    } else if preferred_dy == 0 && preferred_dx != 0 {
        // Moving horizontally, try diagonals
        vec![(preferred_dx, 1), (preferred_dx, -1)]
    } else {
        // Already diagonal, try cardinal directions
        vec![(preferred_dx, 0), (0, preferred_dy)]
    };

    // Shuffle alternatives for variety
    let mut alternatives = alternatives;
    if rng.one_in(2) {
        alternatives.reverse();
    }

    for (dx, dy) in alternatives {
        let new_x = mx + dx;
        let new_y = my + dy;

        if level.is_valid_pos(new_x, new_y)
            && level.is_walkable(new_x, new_y)
            && level.monster_at(new_x, new_y).is_none()
        {
            level.move_monster(monster_id, new_x, new_y);
            return AiAction::Moved(new_x, new_y);
        }
    }

    // Couldn't move anywhere
    AiAction::Waited
}

/// Pet AI - follow player and attack hostile monsters
fn pet_ai(
    monster_id: MonsterId,
    level: &mut Level,
    player: &You,
    rng: &mut GameRng,
) -> AiAction {
    let monster = level.monster(monster_id).unwrap();
    let mx = monster.x;
    let my = monster.y;
    let px = player.pos.x;
    let py = player.pos.y;

    // Check for adjacent hostile monsters to attack
    for dx in -1..=1i8 {
        for dy in -1..=1i8 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let tx = mx + dx;
            let ty = my + dy;
            if let Some(target) = level.monster_at(tx, ty)
                && target.is_hostile()
                && target.id != monster_id
            {
                // Attack the hostile monster (monster-vs-monster combat handled elsewhere)
                return AiAction::Moved(tx, ty); // Signal attack intent
            }
        }
    }

    // If close to player, sometimes wander
    let dist_sq = monster.distance_sq(px, py);
    if dist_sq <= 4 && rng.one_in(3) {
        return wander_randomly(monster_id, level, rng);
    }

    // Follow player if not too close
    if dist_sq > 4 {
        return move_towards(monster_id, level, px, py, rng);
    }

    // Stay near player
    AiAction::Waited
}

/// Move randomly (for peaceful monsters or confusion)
fn wander_randomly(monster_id: MonsterId, level: &mut Level, rng: &mut GameRng) -> AiAction {
    let monster = level.monster(monster_id).unwrap();
    let mx = monster.x;
    let my = monster.y;

    // 50% chance to just wait
    if rng.one_in(2) {
        return AiAction::Waited;
    }

    let (dx, dy) = random_direction(rng);
    let new_x = mx + dx;
    let new_y = my + dy;

    if level.is_valid_pos(new_x, new_y)
        && level.is_walkable(new_x, new_y)
        && level.monster_at(new_x, new_y).is_none()
    {
        level.move_monster(monster_id, new_x, new_y);
        AiAction::Moved(new_x, new_y)
    } else {
        AiAction::Waited
    }
}

/// Get a random direction (including diagonals)
fn random_direction(rng: &mut GameRng) -> (i8, i8) {
    const DIRECTIONS: [(i8, i8); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let idx = rng.rn2(8) as usize;
    DIRECTIONS[idx]
}

/// Fleeing AI - move away from player
fn flee_from_player(
    monster_id: MonsterId,
    level: &mut Level,
    player: &You,
    rng: &mut GameRng,
) -> AiAction {
    let monster = level.monster(monster_id).unwrap();
    let mx = monster.x;
    let my = monster.y;
    let px = player.pos.x;
    let py = player.pos.y;

    // Calculate direction away from player
    let dx = (mx - px).signum();
    let dy = (my - py).signum();

    // If already far enough, stop fleeing
    let dist_sq = monster.distance_sq(px, py);
    if dist_sq > 100 {
        // More than 10 squares away
        return wander_randomly(monster_id, level, rng);
    }

    let new_x = mx + dx;
    let new_y = my + dy;

    if level.is_valid_pos(new_x, new_y)
        && level.is_walkable(new_x, new_y)
        && level.monster_at(new_x, new_y).is_none()
    {
        level.move_monster(monster_id, new_x, new_y);
        AiAction::Moved(new_x, new_y)
    } else {
        // Try alternative escape routes
        try_alternative_move(monster_id, level, dx, dy, rng)
    }
}

/// Check if monster should flee based on HP and state
pub fn should_flee(monster: &super::Monster) -> bool {
    // Already fleeing
    if monster.state.fleeing || monster.flee_timeout > 0 {
        return true;
    }
    
    // Low HP - flee if below 25%
    if monster.hp > 0 && monster.hp_max > 0 {
        let hp_percent = (monster.hp * 100) / monster.hp_max;
        if hp_percent < 25 {
            return true;
        }
    }
    
    false
}

/// Process fleeing monster AI
pub fn process_fleeing_ai(
    monster_id: MonsterId,
    level: &mut Level,
    player: &You,
    rng: &mut GameRng,
) -> AiAction {
    // Decrement flee timeout
    if let Some(monster) = level.monster_mut(monster_id) {
        if monster.flee_timeout > 0 {
            monster.flee_timeout -= 1;
            if monster.flee_timeout == 0 {
                monster.state.fleeing = false;
            }
        }
    }
    
    flee_from_player(monster_id, level, player, rng)
}

/// Enhanced monster AI that includes fleeing behavior
pub fn process_monster_ai_full(
    monster_id: MonsterId,
    level: &mut Level,
    player: &You,
    rng: &mut GameRng,
) -> AiAction {
    let monster = match level.monster(monster_id) {
        Some(m) => m,
        None => return AiAction::None,
    };

    // Check if monster should flee
    if should_flee(monster) {
        return process_fleeing_ai(monster_id, level, player, rng);
    }

    // Otherwise use normal AI
    process_monster_ai(monster_id, level, player, rng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::{DLevel, Level};
    use crate::monster::Monster;
    use crate::player::Position;

    #[test]
    fn test_monster_moves_towards_player() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::main_dungeon_start());

        // Create open floor area
        for x in 0..10 {
            for y in 0..10 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        // Place monster at (5, 5)
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        // Player at (7, 7)
        let mut player = You::default();
        player.pos = Position { x: 7, y: 7 };

        // Monster should move towards player
        let action = process_monster_ai(MonsterId(1), &mut level, &player, &mut rng);

        match action {
            AiAction::Moved(x, y) => {
                // Should have moved closer
                let old_dist_sq = (5 - 7) * (5 - 7) + (5 - 7) * (5 - 7);
                let new_dist_sq = (x - 7) * (x - 7) + (y - 7) * (y - 7);
                assert!(
                    new_dist_sq <= old_dist_sq,
                    "Monster should move closer to player"
                );
            }
            _ => panic!("Monster should have moved"),
        }
    }

    #[test]
    fn test_monster_attacks_when_adjacent() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::main_dungeon_start());

        // Create open floor area
        for x in 0..10 {
            for y in 0..10 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        // Place monster adjacent to player
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 6, y: 6 };

        let action = process_monster_ai(MonsterId(1), &mut level, &player, &mut rng);

        assert_eq!(action, AiAction::AttackedPlayer);
    }

    #[test]
    fn test_sleeping_monster_doesnt_move() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::main_dungeon_start());

        for x in 0..10 {
            for y in 0..10 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.sleeping = true;
        level.add_monster(monster);

        let mut player = You::default();
        player.pos = Position { x: 9, y: 9 }; // Far away

        let action = process_monster_ai(MonsterId(1), &mut level, &player, &mut rng);

        // Sleeping monster far from player should wait
        assert_eq!(action, AiAction::Waited);
    }
}
