//! Monster tactical AI (monmove.c, mthrowu.c)
//!
//! Advanced monster behaviors: ranged attacks, special abilities, group tactics.

use crate::dungeon::Level;
use crate::player::You;
use crate::rng::GameRng;

use super::{Monster, MonsterId};

/// Tactical action a monster can take
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TacticalAction {
    /// No special action
    None,
    /// Use ranged attack (throw, breath, spell)
    RangedAttack { target_x: i8, target_y: i8 },
    /// Use special ability
    SpecialAbility(SpecialAbility),
    /// Retreat to heal
    Retreat,
    /// Call for help (wake nearby monsters)
    CallForHelp,
    /// Pick up item
    PickupItem,
    /// Use item (wand, potion)
    UseItem,
    /// Open door
    OpenDoor { x: i8, y: i8 },
    /// Hide/ambush
    Hide,
}

/// Special abilities monsters can use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialAbility {
    /// Breath weapon (dragon)
    BreathWeapon,
    /// Gaze attack (floating eye, medusa)
    GazeAttack,
    /// Spit venom
    SpitVenom,
    /// Cast spell
    CastSpell,
    /// Summon allies
    Summon,
    /// Teleport self
    TeleportSelf,
    /// Steal item
    Steal,
    /// Seduce/charm
    Seduce,
}

/// Monster intelligence level affects tactical decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Intelligence {
    /// Mindless (golems, oozes)
    Mindless = 0,
    /// Animal intelligence
    Animal = 1,
    /// Low intelligence (orcs, goblins)
    Low = 2,
    /// Average intelligence
    Average = 3,
    /// High intelligence (liches, dragons)
    High = 4,
    /// Genius (arch-liches, demon lords)
    Genius = 5,
}

/// Get monster intelligence based on type
pub fn monster_intelligence(monster_type: i16) -> Intelligence {
    // Simplified - in real NetHack this comes from permonst data
    match monster_type {
        0..=5 => Intelligence::Animal,      // Basic creatures
        6..=10 => Intelligence::Low,        // Orcs, goblins
        11..=15 => Intelligence::Average,   // Humanoids
        16..=20 => Intelligence::High,      // Dragons, liches
        _ => Intelligence::Average,
    }
}

/// Check if monster has line of sight to target
pub fn has_line_of_sight(level: &Level, from_x: i8, from_y: i8, to_x: i8, to_y: i8) -> bool {
    // Bresenham's line algorithm to check for obstacles
    let dx = (to_x - from_x).abs();
    let dy = (to_y - from_y).abs();
    let sx = if from_x < to_x { 1 } else { -1 };
    let sy = if from_y < to_y { 1 } else { -1 };

    let mut err = dx - dy;
    let mut x = from_x;
    let mut y = from_y;

    while x != to_x || y != to_y {
        // Check if current position blocks sight
        if x != from_x || y != from_y {
            if !level.is_valid_pos(x, y) {
                return false;
            }
            let cell = &level.cells[x as usize][y as usize];
            if cell.blocks_sight() {
                return false;
            }
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }

    true
}

/// Calculate distance between two points
pub fn distance(x1: i8, y1: i8, x2: i8, y2: i8) -> i32 {
    let dx = (x2 - x1) as i32;
    let dy = (y2 - y1) as i32;
    ((dx * dx + dy * dy) as f64).sqrt() as i32
}

/// Check if monster can use ranged attacks
pub fn can_use_ranged(monster: &Monster) -> bool {
    // Check for ranged capability based on monster type
    // Simplified - real NetHack checks for projectiles, breath weapons, spells
    monster.monster_type >= 10 // Higher level monsters have ranged
}

/// Check if monster should use ranged attack
pub fn should_use_ranged(
    monster: &Monster,
    player: &You,
    level: &Level,
    rng: &mut GameRng,
) -> bool {
    if !can_use_ranged(monster) {
        return false;
    }

    let dist = distance(monster.x, monster.y, player.pos.x, player.pos.y);

    // Too close - prefer melee
    if dist <= 1 {
        return false;
    }

    // Too far - can't hit
    if dist > 8 {
        return false;
    }

    // Need line of sight
    if !has_line_of_sight(level, monster.x, monster.y, player.pos.x, player.pos.y) {
        return false;
    }

    // Intelligence affects decision
    let intelligence = monster_intelligence(monster.monster_type);
    let use_chance = match intelligence {
        Intelligence::Mindless => 0,
        Intelligence::Animal => 10,
        Intelligence::Low => 30,
        Intelligence::Average => 50,
        Intelligence::High => 70,
        Intelligence::Genius => 90,
    };

    rng.percent(use_chance)
}

/// Determine if monster should retreat
pub fn should_retreat(monster: &Monster, _rng: &mut GameRng) -> bool {
    // Low HP monsters with intelligence may retreat
    if monster.hp <= 0 || monster.hp_max <= 0 {
        return false;
    }

    let hp_percent = (monster.hp * 100) / monster.hp_max;
    let intelligence = monster_intelligence(monster.monster_type);

    match intelligence {
        Intelligence::Mindless | Intelligence::Animal => false,
        Intelligence::Low => hp_percent < 10,
        Intelligence::Average => hp_percent < 20,
        Intelligence::High => hp_percent < 30,
        Intelligence::Genius => hp_percent < 40,
    }
}

/// Check if monster should call for help
pub fn should_call_for_help(
    monster: &Monster,
    level: &Level,
    rng: &mut GameRng,
) -> bool {
    let intelligence = monster_intelligence(monster.monster_type);

    // Only intelligent monsters call for help
    if intelligence < Intelligence::Low {
        return false;
    }

    // Check if there are allies nearby to wake
    let mut sleeping_allies = 0;
    for other in &level.monsters {
        if other.id == monster.id {
            continue;
        }
        if other.state.sleeping && other.monster_type == monster.monster_type {
            let dist = distance(monster.x, monster.y, other.x, other.y);
            if dist <= 5 {
                sleeping_allies += 1;
            }
        }
    }

    if sleeping_allies == 0 {
        return false;
    }

    // Chance based on intelligence and HP
    let hp_percent = if monster.hp_max > 0 {
        (monster.hp * 100) / monster.hp_max
    } else {
        100
    };

    let call_chance = match intelligence {
        Intelligence::Low => 10,
        Intelligence::Average => 20,
        Intelligence::High => 40,
        Intelligence::Genius => 60,
        _ => 0,
    };

    // More likely to call when hurt
    let adjusted_chance = if hp_percent < 50 {
        call_chance * 2
    } else {
        call_chance
    };

    rng.percent(adjusted_chance)
}

/// Wake nearby monsters of same type
pub fn wake_nearby_allies(monster_id: MonsterId, level: &mut Level) -> i32 {
    let monster = match level.monster(monster_id) {
        Some(m) => m,
        None => return 0,
    };

    let mx = monster.x;
    let my = monster.y;
    let mtype = monster.monster_type;

    let mut woken = 0;

    // Find and wake sleeping allies
    let ally_ids: Vec<MonsterId> = level
        .monsters
        .iter()
        .filter(|m| {
            m.id != monster_id
                && m.state.sleeping
                && m.monster_type == mtype
                && distance(mx, my, m.x, m.y) <= 5
        })
        .map(|m| m.id)
        .collect();

    for ally_id in ally_ids {
        if let Some(ally) = level.monster_mut(ally_id) {
            ally.state.sleeping = false;
            woken += 1;
        }
    }

    woken
}

/// Determine tactical action for a monster
pub fn determine_tactics(
    monster: &Monster,
    player: &You,
    level: &Level,
    rng: &mut GameRng,
) -> TacticalAction {
    let intelligence = monster_intelligence(monster.monster_type);

    // Mindless creatures don't use tactics
    if intelligence == Intelligence::Mindless {
        return TacticalAction::None;
    }

    // Check retreat first (self-preservation)
    if should_retreat(monster, rng) {
        return TacticalAction::Retreat;
    }

    // Check ranged attack
    if should_use_ranged(monster, player, level, rng) {
        return TacticalAction::RangedAttack {
            target_x: player.pos.x,
            target_y: player.pos.y,
        };
    }

    // Check call for help
    if should_call_for_help(monster, level, rng) {
        return TacticalAction::CallForHelp;
    }

    TacticalAction::None
}

/// Group behavior - monsters of same type coordinate
pub fn group_behavior(
    monster_id: MonsterId,
    level: &Level,
    player: &You,
) -> Option<(i8, i8)> {
    let monster = level.monster(monster_id)?;

    // Find allies of same type
    let allies: Vec<&Monster> = level
        .monsters
        .iter()
        .filter(|m| {
            m.id != monster_id
                && m.monster_type == monster.monster_type
                && !m.state.sleeping
                && distance(monster.x, monster.y, m.x, m.y) <= 10
        })
        .collect();

    if allies.is_empty() {
        return None;
    }

    // Calculate center of mass of group
    let mut sum_x: i32 = monster.x as i32;
    let mut sum_y: i32 = monster.y as i32;
    for ally in &allies {
        sum_x += ally.x as i32;
        sum_y += ally.y as i32;
    }
    let count = (allies.len() + 1) as i32;
    let center_x = (sum_x / count) as i8;
    let center_y = (sum_y / count) as i8;

    // If far from group center, move towards it
    let dist_to_center = distance(monster.x, monster.y, center_x, center_y);
    let dist_to_player = distance(monster.x, monster.y, player.pos.x, player.pos.y);

    // Stay with group if player is far
    if dist_to_player > 5 && dist_to_center > 3 {
        Some((center_x, center_y))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::DLevel;
    use crate::player::Position;

    #[test]
    fn test_distance() {
        assert_eq!(distance(0, 0, 3, 4), 5);
        assert_eq!(distance(0, 0, 0, 0), 0);
        assert_eq!(distance(1, 1, 4, 5), 5);
    }

    #[test]
    fn test_monster_intelligence() {
        assert!(monster_intelligence(0) < monster_intelligence(15));
        assert!(monster_intelligence(20) > monster_intelligence(5));
    }

    #[test]
    fn test_has_line_of_sight() {
        let mut level = Level::new(DLevel::main_dungeon_start());

        // Create open area
        for x in 0..10 {
            for y in 0..10 {
                level.cells[x][y].typ = crate::dungeon::CellType::Room;
            }
        }

        // Should have LOS in open area
        assert!(has_line_of_sight(&level, 1, 1, 5, 5));

        // Add a wall
        level.cells[3][3].typ = crate::dungeon::CellType::VWall;

        // LOS through wall should be blocked
        assert!(!has_line_of_sight(&level, 1, 1, 5, 5));
    }

    #[test]
    fn test_should_retreat() {
        let mut rng = GameRng::new(42);

        // Healthy monster shouldn't retreat
        let mut monster = Monster::new(MonsterId(1), 15, 5, 5);
        monster.hp = 100;
        monster.hp_max = 100;
        assert!(!should_retreat(&monster, &mut rng));

        // Badly hurt intelligent monster should retreat
        monster.hp = 10;
        assert!(should_retreat(&monster, &mut rng));
    }

    #[test]
    fn test_determine_tactics() {
        let mut rng = GameRng::new(42);
        let level = Level::new(DLevel::main_dungeon_start());
        let mut player = You::default();
        player.pos = Position { x: 10, y: 10 };

        // Low-level monster
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        let action = determine_tactics(&monster, &player, &level, &mut rng);
        // Animal intelligence - limited tactics
        assert!(matches!(action, TacticalAction::None | TacticalAction::Retreat));
    }

    #[test]
    fn test_wake_nearby_allies() {
        let mut level = Level::new(DLevel::main_dungeon_start());

        // Add a monster
        let mut m1 = Monster::new(MonsterId(1), 5, 5, 5);
        m1.state.sleeping = false;
        level.add_monster(m1);

        // Add sleeping ally nearby
        let mut m2 = Monster::new(MonsterId(2), 5, 6, 6);
        m2.state.sleeping = true;
        level.add_monster(m2);

        // Add sleeping ally far away (but within level bounds)
        let mut m3 = Monster::new(MonsterId(3), 5, 15, 15);
        m3.state.sleeping = true;
        level.add_monster(m3);

        let woken = wake_nearby_allies(MonsterId(1), &mut level);
        assert_eq!(woken, 1, "Should wake one nearby ally");

        // Check the nearby ally is awake
        let ally = level.monster(MonsterId(2)).unwrap();
        assert!(!ally.state.sleeping);

        // Far ally should still be sleeping
        let far_ally = level.monster(MonsterId(3)).unwrap();
        assert!(far_ally.state.sleeping);
    }
}
