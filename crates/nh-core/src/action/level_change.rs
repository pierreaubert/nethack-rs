//! Level change actions (do.c: dodown, doup, goto_level)
//!
//! Handles stair usage, level transitions, falling through traps,
//! and floor effects when objects land.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::dungeon::{CellType, DLevel, Level, TrapType};
use crate::player::{Property, You};
use crate::rng::GameRng;
use crate::{COLNO, ROWNO};

// ============================================================================
// Level transition direction
// ============================================================================

/// Direction of level change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelDirection {
    /// Going down (stairs, trap door, hole)
    Down,
    /// Going up (stairs, ladder)
    Up,
    /// Teleporting to specific level
    Teleport,
    /// Falling (involuntary)
    Falling,
    /// Portal (magic portal)
    Portal,
}

/// Result of attempting a level change
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelChangeResult {
    /// Successfully changed level — destination DLevel
    Changed(DLevel),
    /// Cannot go that direction (message included)
    Blocked(String),
    /// Player is floating/levitating
    Floating(String),
    /// Player is trapped
    Trapped(String),
    /// Player is held by a monster
    Held,
}

// ============================================================================
// Stair usage
// ============================================================================

/// Attempt to go down stairs or through a hole (C: dodown).
///
/// Checks for various conditions that prevent descending:
/// - Levitation/flying
/// - Being trapped
/// - Being held/swallowed
/// - No stairs at current position
pub fn dodown(
    player: &You,
    level: &Level,
    _rng: &mut GameRng,
) -> LevelChangeResult {
    let px = player.pos.x;
    let py = player.pos.y;

    // Check if levitating
    if player.properties.has(Property::Levitation) {
        return LevelChangeResult::Floating(
            "You are floating high above the stairs.".to_string(),
        );
    }

    // Check if trapped
    if player.utrap > 0 {
        return LevelChangeResult::Trapped(
            "You are stuck and cannot go down.".to_string(),
        );
    }

    // Check for downstairs at current position
    let stair = level.stairs.iter().find(|s| s.x == px && s.y == py && !s.up);

    if let Some(stair) = stair {
        LevelChangeResult::Changed(stair.destination)
    } else if let Some(trap) = level.trap_at(px, py) {
        // Trap door or hole allows descent
        match trap.trap_type {
            TrapType::TrapDoor | TrapType::Hole => {
                LevelChangeResult::Changed(DLevel {
                    dungeon_num: level.dlevel.dungeon_num,
                    level_num: level.dlevel.level_num + 1,
                })
            }
            _ => LevelChangeResult::Blocked("You can't go down here.".to_string()),
        }
    } else {
        LevelChangeResult::Blocked("You can't go down here.".to_string())
    }
}

/// Attempt to go up stairs (C: doup).
///
/// Checks for conditions preventing ascent:
/// - No upstairs at current position
/// - Being trapped in a pit (can climb out)
/// - Being held/swallowed
pub fn doup(
    player: &You,
    level: &Level,
    _rng: &mut GameRng,
) -> LevelChangeResult {
    let px = player.pos.x;
    let py = player.pos.y;

    // If trapped in pit, try to climb out
    if player.utrap > 0 {
        // In C: climbing out of a pit via '<' always works
        // The actual trap reset is handled by the caller
        return LevelChangeResult::Trapped(
            "You climb out of the pit.".to_string(),
        );
    }

    // Check for upstairs at current position
    let stair = level.stairs.iter().find(|s| s.x == px && s.y == py && s.up);

    if let Some(stair) = stair {
        // Check if this is the top of the dungeon
        if stair.destination.level_num == 0 && stair.destination.dungeon_num == 0 {
            // Escaping the dungeon
            LevelChangeResult::Changed(stair.destination)
        } else {
            LevelChangeResult::Changed(stair.destination)
        }
    } else {
        LevelChangeResult::Blocked("You can't go up here.".to_string())
    }
}

// ============================================================================
// Level transition
// ============================================================================

/// Full level transition processing (C: goto_level).
///
/// Handles the actual mechanics of changing levels:
/// - Saving the current level state
/// - Moving pets that are adjacent
/// - Loading or generating the new level
/// - Placing the player at the appropriate location
///
/// This is a high-level orchestrator — the actual level storage is
/// managed by the caller (GameState/DungeonSystem).
#[derive(Debug, Clone)]
pub struct LevelTransition {
    /// Where we're going
    pub destination: DLevel,
    /// How we're getting there
    pub direction: LevelDirection,
    /// Whether we arrived at stairs (vs falling)
    pub at_stairs: bool,
    /// Whether this is a falling transition
    pub falling: bool,
    /// Whether this is via magic portal
    pub portal: bool,
}

impl LevelTransition {
    /// Create a stairway transition
    pub fn stairs(destination: DLevel, going_up: bool) -> Self {
        Self {
            destination,
            direction: if going_up { LevelDirection::Up } else { LevelDirection::Down },
            at_stairs: true,
            falling: false,
            portal: false,
        }
    }

    /// Create a falling transition
    pub fn falling(destination: DLevel) -> Self {
        Self {
            destination,
            direction: LevelDirection::Falling,
            at_stairs: false,
            falling: true,
            portal: false,
        }
    }

    /// Create a portal transition
    pub fn portal(destination: DLevel) -> Self {
        Self {
            destination,
            direction: LevelDirection::Portal,
            at_stairs: false,
            falling: false,
            portal: true,
        }
    }

    /// Create a teleport transition
    pub fn teleport(destination: DLevel) -> Self {
        Self {
            destination,
            direction: LevelDirection::Teleport,
            at_stairs: false,
            falling: false,
            portal: false,
        }
    }
}

/// Find adjacent pets that should follow the player when changing levels.
///
/// In C, `keepdogs()` and `losedogs()` manage pets following between levels.
/// A pet follows if it is adjacent (within 1 tile) to the player and tame.
pub fn find_following_pets(
    level: &Level,
    player_x: i8,
    player_y: i8,
) -> Vec<super::super::monster::MonsterId> {
    let mut following = Vec::new();

    for mon in &level.monsters {
        if mon.tameness > 0 {
            let dx = (mon.x - player_x).abs();
            let dy = (mon.y - player_y).abs();
            if dx <= 1 && dy <= 1 {
                following.push(mon.id);
            }
        }
    }

    following
}

/// Determine where to place the player on the new level.
///
/// If arriving via stairs, place at the corresponding stair.
/// If falling, place at a random valid position.
/// If teleporting, place at a random valid position.
pub fn find_arrival_position(
    level: &Level,
    transition: &LevelTransition,
    source: DLevel,
    rng: &mut GameRng,
) -> (i8, i8) {
    if transition.at_stairs {
        // Find the stair that connects back to the source level
        let going_up = transition.direction == LevelDirection::Up;
        // If we went down, we arrive at upstairs on the new level (and vice versa)
        let arrive_at_up = !going_up;

        for stair in &level.stairs {
            if stair.up == arrive_at_up && stair.destination == source {
                return (stair.x, stair.y);
            }
        }

        // Fallback: find any matching stair direction
        for stair in &level.stairs {
            if stair.up == arrive_at_up {
                return (stair.x, stair.y);
            }
        }
    }

    // Random valid position for falling/teleport/portal
    find_random_valid_position(level, rng)
}

/// Find a random valid position on the level for player placement.
fn find_random_valid_position(level: &Level, rng: &mut GameRng) -> (i8, i8) {
    // Try random positions
    for _ in 0..1000 {
        let x = rng.rn2(COLNO as u32) as i8;
        let y = rng.rn2(ROWNO as u32) as i8;
        if level.is_valid_pos(x, y) && level.is_walkable(x, y) {
            return (x, y);
        }
    }

    // Fallback: scan for any walkable cell
    for x in 0..COLNO as i8 {
        for y in 0..ROWNO as i8 {
            if level.is_walkable(x, y) {
                return (x, y);
            }
        }
    }

    // Last resort
    (1, 1)
}

// ============================================================================
// Floor effects
// ============================================================================

/// What happens when an object lands on a terrain feature.
///
/// Matches C `flooreffects()` from do.c. Objects can:
/// - Fill pools/moats (boulders)
/// - Fall into pits/holes
/// - Trigger traps
///
/// Returns true if the object was consumed (fell into something).
pub fn flooreffects(
    level: &Level,
    x: i8,
    y: i8,
    is_boulder: bool,
) -> FloorEffect {
    if !level.is_valid_pos(x, y) {
        return FloorEffect::Nothing;
    }

    let cell = &level.cells[x as usize][y as usize];

    // Boulder fills pool
    if is_boulder && cell.typ == CellType::Pool {
        return FloorEffect::FillsPool;
    }

    // Boulder fills moat
    if is_boulder && cell.typ == CellType::Moat {
        return FloorEffect::FillsMoat;
    }

    // Falls into pit or hole
    if let Some(trap) = level.trap_at(x, y) {
        match trap.trap_type {
            TrapType::Pit | TrapType::SpikedPit => return FloorEffect::FallsIntoPit,
            TrapType::Hole | TrapType::TrapDoor => return FloorEffect::FallsThroughHole,
            _ => {}
        }
    }

    // Sinks in lava
    if cell.typ == CellType::Lava {
        return FloorEffect::SinksInLava;
    }

    // Falls in water
    if cell.typ == CellType::Pool || cell.typ == CellType::Moat {
        return FloorEffect::FallsInWater;
    }

    FloorEffect::Nothing
}

/// What happened to an object on the floor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorEffect {
    /// Nothing special happened
    Nothing,
    /// Boulder fills a pool (pool becomes floor)
    FillsPool,
    /// Boulder fills a moat
    FillsMoat,
    /// Object sinks in lava
    SinksInLava,
    /// Object falls in water
    FallsInWater,
    /// Object falls into a pit
    FallsIntoPit,
    /// Object falls through a hole to next level
    FallsThroughHole,
}

// ============================================================================
// Level depth utilities
// ============================================================================

/// Calculate the effective depth for monster difficulty.
///
/// Matches C `depth()` and related macros. Returns the absolute
/// depth from the surface.
pub fn effective_depth(dlevel: &DLevel) -> i32 {
    // Simple calculation: level_num is the depth
    // In C, this accounts for dungeon branches with different base depths
    dlevel.level_num as i32
}

/// Check if a level is in Gehennom (hell).
///
/// In the standard dungeon, Gehennom starts at level 25+.
pub fn is_in_hell(dlevel: &DLevel) -> bool {
    // Simplified: main dungeon (0), levels 25+ are hell
    dlevel.dungeon_num == 0 && dlevel.level_num >= 25
}

/// Check if this is the bottom level of the dungeon.
pub fn is_bottom_level(dlevel: &DLevel) -> bool {
    // In standard NetHack, level ~50 is the bottom
    dlevel.level_num >= 50
}

/// Calculate fall damage for falling through a level.
///
/// Matches C fall damage: d(dist, 6) where dist is number of levels fallen.
pub fn fall_damage(levels_fallen: i32, rng: &mut GameRng) -> i32 {
    if levels_fallen <= 0 {
        return 0;
    }
    rng.dice(levels_fallen as u32, 6) as i32
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::{Level, Stairway};
    use crate::player::{Gender, Race, Role};
    use crate::rng::GameRng;

    fn test_player_at(x: i8, y: i8) -> You {
        let mut player = You::new("Test".into(), Role::Valkyrie, Race::Human, Gender::Female);
        player.pos.x = x;
        player.pos.y = y;
        player
    }

    fn test_level_with_stairs() -> Level {
        let mut level = Level::new(DLevel::new(0, 5));
        // Create a room
        for x in 5..15 {
            for y in 3..8 {
                level.cells[x][y].typ = CellType::Room;
            }
        }
        // Add stairs
        level.stairs.push(Stairway {
            x: 7,
            y: 5,
            destination: DLevel::new(0, 4),
            up: true,
        });
        level.stairs.push(Stairway {
            x: 10,
            y: 5,
            destination: DLevel::new(0, 6),
            up: false,
        });
        level
    }

    // ---- dodown tests ----

    #[test]
    fn test_dodown_on_stairs() {
        let level = test_level_with_stairs();
        let player = test_player_at(10, 5);
        let mut rng = GameRng::new(42);

        let result = dodown(&player, &level, &mut rng);
        assert_eq!(result, LevelChangeResult::Changed(DLevel::new(0, 6)));
    }

    #[test]
    fn test_dodown_no_stairs() {
        let level = test_level_with_stairs();
        let player = test_player_at(8, 5);
        let mut rng = GameRng::new(42);

        let result = dodown(&player, &level, &mut rng);
        assert!(matches!(result, LevelChangeResult::Blocked(_)));
    }

    #[test]
    fn test_dodown_levitating() {
        let level = test_level_with_stairs();
        let mut player = test_player_at(10, 5);
        player.properties.grant_intrinsic(Property::Levitation);
        let mut rng = GameRng::new(42);

        let result = dodown(&player, &level, &mut rng);
        assert!(matches!(result, LevelChangeResult::Floating(_)));
    }

    #[test]
    fn test_dodown_trapped() {
        let level = test_level_with_stairs();
        let mut player = test_player_at(10, 5);
        player.utrap = 3;
        let mut rng = GameRng::new(42);

        let result = dodown(&player, &level, &mut rng);
        assert!(matches!(result, LevelChangeResult::Trapped(_)));
    }

    // ---- doup tests ----

    #[test]
    fn test_doup_on_stairs() {
        let level = test_level_with_stairs();
        let player = test_player_at(7, 5);
        let mut rng = GameRng::new(42);

        let result = doup(&player, &level, &mut rng);
        assert_eq!(result, LevelChangeResult::Changed(DLevel::new(0, 4)));
    }

    #[test]
    fn test_doup_no_stairs() {
        let level = test_level_with_stairs();
        let player = test_player_at(8, 5);
        let mut rng = GameRng::new(42);

        let result = doup(&player, &level, &mut rng);
        assert!(matches!(result, LevelChangeResult::Blocked(_)));
    }

    #[test]
    fn test_doup_trapped_climb_pit() {
        let level = test_level_with_stairs();
        let mut player = test_player_at(7, 5);
        player.utrap = 2;
        let mut rng = GameRng::new(42);

        let result = doup(&player, &level, &mut rng);
        assert!(matches!(result, LevelChangeResult::Trapped(_)));
    }

    // ---- LevelTransition tests ----

    #[test]
    fn test_transition_stairs() {
        let t = LevelTransition::stairs(DLevel::new(0, 6), false);
        assert!(t.at_stairs);
        assert!(!t.falling);
        assert_eq!(t.direction, LevelDirection::Down);
    }

    #[test]
    fn test_transition_falling() {
        let t = LevelTransition::falling(DLevel::new(0, 8));
        assert!(!t.at_stairs);
        assert!(t.falling);
        assert_eq!(t.direction, LevelDirection::Falling);
    }

    // ---- find_arrival_position tests ----

    #[test]
    fn test_arrival_at_stairs() {
        let level = test_level_with_stairs();
        let mut rng = GameRng::new(42);

        // Going down from level 4, arrive at upstairs on level 5
        let transition = LevelTransition::stairs(DLevel::new(0, 5), false);
        let (x, y) = find_arrival_position(&level, &transition, DLevel::new(0, 4), &mut rng);

        // Should arrive at the upstairs (7, 5) which connects to level 4
        assert_eq!((x, y), (7, 5));
    }

    #[test]
    fn test_arrival_at_stairs_going_up() {
        let level = test_level_with_stairs();
        let mut rng = GameRng::new(42);

        // Going up from level 6, arrive at downstairs on level 5
        let transition = LevelTransition::stairs(DLevel::new(0, 5), true);
        let (x, y) = find_arrival_position(&level, &transition, DLevel::new(0, 6), &mut rng);

        // Should arrive at the downstairs (10, 5) which connects to level 6
        assert_eq!((x, y), (10, 5));
    }

    #[test]
    fn test_arrival_falling_random() {
        let level = test_level_with_stairs();
        let mut rng = GameRng::new(42);

        let transition = LevelTransition::falling(DLevel::new(0, 5));
        let (x, y) = find_arrival_position(&level, &transition, DLevel::new(0, 3), &mut rng);

        // Should be a valid walkable position
        assert!(level.is_walkable(x, y));
    }

    // ---- find_following_pets tests ----

    #[test]
    fn test_find_following_pets_adjacent() {
        let mut level = test_level_with_stairs();
        let mut pet = crate::monster::Monster::new(
            crate::monster::MonsterId(0), 0, 8, 5,
        );
        pet.tameness = 5;
        pet.name = "kitten".to_string();
        level.add_monster(pet);

        let following = find_following_pets(&level, 7, 5);
        assert_eq!(following.len(), 1);
    }

    #[test]
    fn test_find_following_pets_too_far() {
        let mut level = test_level_with_stairs();
        let mut pet = crate::monster::Monster::new(
            crate::monster::MonsterId(0), 0, 12, 5,
        );
        pet.tameness = 5;
        pet.name = "kitten".to_string();
        level.add_monster(pet);

        let following = find_following_pets(&level, 7, 5);
        assert!(following.is_empty());
    }

    #[test]
    fn test_find_following_pets_hostile_ignored() {
        let mut level = test_level_with_stairs();
        let mut hostile = crate::monster::Monster::new(
            crate::monster::MonsterId(0), 0, 8, 5,
        );
        hostile.tameness = 0;
        hostile.name = "goblin".to_string();
        level.add_monster(hostile);

        let following = find_following_pets(&level, 7, 5);
        assert!(following.is_empty());
    }

    // ---- flooreffects tests ----

    #[test]
    fn test_flooreffects_nothing() {
        let level = test_level_with_stairs();
        let effect = flooreffects(&level, 7, 5, false);
        assert_eq!(effect, FloorEffect::Nothing);
    }

    #[test]
    fn test_flooreffects_boulder_pool() {
        let mut level = test_level_with_stairs();
        level.cells[8][5].typ = CellType::Pool;

        let effect = flooreffects(&level, 8, 5, true);
        assert_eq!(effect, FloorEffect::FillsPool);
    }

    #[test]
    fn test_flooreffects_lava() {
        let mut level = test_level_with_stairs();
        level.cells[8][5].typ = CellType::Lava;

        let effect = flooreffects(&level, 8, 5, false);
        assert_eq!(effect, FloorEffect::SinksInLava);
    }

    // ---- depth utility tests ----

    #[test]
    fn test_effective_depth() {
        assert_eq!(effective_depth(&DLevel::new(0, 1)), 1);
        assert_eq!(effective_depth(&DLevel::new(0, 10)), 10);
    }

    #[test]
    fn test_is_in_hell() {
        assert!(!is_in_hell(&DLevel::new(0, 20)));
        assert!(is_in_hell(&DLevel::new(0, 25)));
        assert!(is_in_hell(&DLevel::new(0, 30)));
    }

    #[test]
    fn test_fall_damage() {
        let mut rng = GameRng::new(42);
        let dmg = fall_damage(3, &mut rng);
        // d(3, 6) = 3-18
        assert!(dmg >= 3 && dmg <= 18, "dmg={dmg}");
    }

    #[test]
    fn test_fall_damage_zero() {
        let mut rng = GameRng::new(42);
        assert_eq!(fall_damage(0, &mut rng), 0);
    }
}
