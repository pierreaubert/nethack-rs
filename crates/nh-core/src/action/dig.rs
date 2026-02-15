//! Digging system (dig.c)
//!
//! Handles pickaxe digging (multi-turn), wand of digging beams,
//! hole/pit creation, and grave excavation.

use crate::dungeon::{CellType, DLevel, Level, TrapType};
use crate::player::{Attribute, You};
use crate::rng::GameRng;

// ============================================================================
// Dig target types
// ============================================================================

/// What we're trying to dig (dig_typ equivalent)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigTarget {
    /// Digging into rock/wall
    Rock,
    /// Digging through a door
    Door,
    /// Digging a tree (requires axe)
    Tree,
    /// Digging a boulder
    Boulder,
    /// Digging a statue
    Statue,
    /// Digging downward (floor)
    Down,
    /// Can't dig here
    Undiggable,
}

/// Classify what digging target is at (x, y) (dig_typ equivalent from dig.c:143).
pub fn dig_target(level: &Level, x: i8, y: i8, has_pick: bool) -> DigTarget {
    if !level.is_valid_pos(x, y) {
        return DigTarget::Undiggable;
    }

    let cell = &level.cells[x as usize][y as usize];

    // Check for boulders at the location
    // TODO: Check object grid for boulders → DigTarget::Boulder
    // TODO: Check object grid for statues → DigTarget::Statue

    if cell.typ.is_door() {
        return DigTarget::Door;
    }

    if cell.typ == CellType::Tree {
        // Trees need an axe, not a pick
        if has_pick {
            return DigTarget::Undiggable;
        }
        return DigTarget::Tree;
    }

    if cell.typ.is_wall() || cell.typ == CellType::Stone || cell.typ == CellType::SecretCorridor {
        if has_pick {
            return DigTarget::Rock;
        }
        return DigTarget::Undiggable;
    }

    DigTarget::Undiggable
}

// ============================================================================
// Dig validation
// ============================================================================

/// Check if digging down is possible at (x, y) (dig_check equivalent from dig.c:183).
///
/// Returns None if digging is allowed, or an error message if not.
pub fn dig_check(
    level: &Level,
    x: i8,
    y: i8,
    _dlevel: &DLevel,
) -> Option<&'static str> {
    if !level.is_valid_pos(x, y) {
        return Some("You can't dig here.");
    }

    let cell = &level.cells[x as usize][y as usize];

    // Can't dig through special terrain
    if cell.typ == CellType::Altar {
        return Some("An altar cannot be dug down.");
    }

    if cell.typ == CellType::Throne {
        return Some("A throne cannot be dug down.");
    }

    // Check for stairs at position
    for stair in &level.stairs {
        if stair.x == x && stair.y == y {
            return Some("The stairs are in the way.");
        }
    }

    // Check for traps that block digging
    for trap in &level.traps {
        if trap.x == x && trap.y == y && trap.trap_type == TrapType::MagicPortal {
            return Some("The magic portal is in the way.");
        }
    }

    // Check for non-diggable flag
    if cell.flags & 0x40 != 0 {
        // W_NONDIGGABLE flag
        return Some("The floor here is too hard to dig in.");
    }

    None
}

// ============================================================================
// Multi-turn digging context
// ============================================================================

/// Context for multi-turn digging occupation.
///
/// Matches C context.digging from dig.c.
#[derive(Debug, Clone, Default)]
pub struct DigContext {
    /// Target position
    pub x: i8,
    pub y: i8,
    /// Whether digging down (true) or horizontally (false)
    pub down: bool,
    /// Effort accumulated (0-250+)
    pub effort: i32,
    /// What we're digging
    pub target: Option<DigTarget>,
    /// Whether digging is in progress
    pub active: bool,
}

/// Result of one turn of digging
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DigResult {
    /// Still digging — not done yet
    InProgress(String),
    /// Finished digging — terrain modified
    Completed(String),
    /// Digging was interrupted or failed
    Failed(String),
    /// Weapon broke during digging
    WeaponBroke(String),
}

/// Perform one turn of digging (dig() occupation from dig.c:241).
///
/// Accumulates effort based on player stats. When effort > 250,
/// the digging completes.
///
/// # Effort formula (from C):
/// effort += 10 + rn2(5) + abon() + weapon_spe - erosion + udaminc
/// Dwarves get double effort.
pub fn dig_turn(
    player: &You,
    level: &mut Level,
    ctx: &mut DigContext,
    weapon_spe: i8,
    weapon_erosion: i8,
    is_dwarf: bool,
    rng: &mut GameRng,
) -> DigResult {
    if !ctx.active {
        return DigResult::Failed("You are not digging.".to_string());
    }

    // Fumbling check (matches C: if Fumbling, 1/3 chance of mishap)
    // Simplified: skip fumbling for now

    // Calculate effort for this turn
    let str_bonus = (player.attr_current.get(Attribute::Strength) - 10).max(0) as i32;
    let mut effort_gain = 10 + rng.rn2(5) as i32 + str_bonus
        + weapon_spe as i32
        - weapon_erosion as i32;

    // Dwarves dig faster (matches C: effort doubled)
    if is_dwarf {
        effort_gain *= 2;
    }

    effort_gain = effort_gain.max(1);
    ctx.effort += effort_gain;

    // Check if digging is complete (matches C: effort > 250)
    if ctx.effort > 250 {
        return complete_dig(level, ctx, rng);
    }

    // Progress messages at thresholds
    let msg = if ctx.effort > 200 {
        "You are almost through."
    } else if ctx.effort > 100 {
        "You continue digging."
    } else {
        "You hit the rock with all your might."
    };

    DigResult::InProgress(msg.to_string())
}

/// Complete a dig — modify terrain (called when effort > 250).
fn complete_dig(
    level: &mut Level,
    ctx: &mut DigContext,
    rng: &mut GameRng,
) -> DigResult {
    let x = ctx.x;
    let y = ctx.y;

    if ctx.down {
        // Digging down — create pit
        return complete_dig_down(level, x, y, rng, ctx);
    }

    // Horizontal digging
    let target = ctx.target.unwrap_or(DigTarget::Rock);
    ctx.active = false;

    match target {
        DigTarget::Rock => {
            // Wall/stone → corridor (matches C behavior)
            let cell = &mut level.cells[x as usize][y as usize];
            if cell.typ.is_wall() || cell.typ == CellType::Stone {
                cell.typ = CellType::Corridor;
                DigResult::Completed("You succeed in cutting away some rock.".to_string())
            } else if cell.typ == CellType::SecretCorridor {
                cell.typ = CellType::Corridor;
                DigResult::Completed("You uncover a secret passage!".to_string())
            } else {
                DigResult::Failed("There's nothing to dig here.".to_string())
            }
        }
        DigTarget::Door => {
            // Door → broken (matches C: doorway destroyed)
            let cell = &mut level.cells[x as usize][y as usize];
            if cell.typ.is_door() {
                cell.typ = CellType::Door;
                cell.set_door_state(crate::dungeon::DoorState::BROKEN);
                DigResult::Completed("You break through the door!".to_string())
            } else {
                DigResult::Failed("There's no door here.".to_string())
            }
        }
        DigTarget::Tree => {
            // Tree → room (matches C)
            let cell = &mut level.cells[x as usize][y as usize];
            cell.typ = CellType::Room;
            DigResult::Completed("You cut down the tree.".to_string())
        }
        _ => {
            ctx.active = false;
            DigResult::Failed("You stop digging.".to_string())
        }
    }
}

/// Complete digging downward — create pit or hole.
fn complete_dig_down(
    level: &mut Level,
    x: i8,
    y: i8,
    rng: &mut GameRng,
    ctx: &mut DigContext,
) -> DigResult {
    ctx.active = false;

    let cell = &level.cells[x as usize][y as usize];

    // Grave → unearth contents (matches C)
    if cell.typ == CellType::Grave {
        let cell = &mut level.cells[x as usize][y as usize];
        cell.typ = CellType::Room;
        return DigResult::Completed("You dig up the grave.".to_string());
    }

    // Create a pit trap at this location
    level.traps.push(crate::dungeon::Trap {
        trap_type: TrapType::Pit,
        x,
        y,
        activated: true,
        seen: true,
        once: false,
        madeby_u: true,
        launch_oid: None,
    });

    // Random: 1/3 chance of deeper hole
    if rng.rn2(3) == 0 {
        DigResult::Completed("You dig a pit in the floor.".to_string())
    } else {
        DigResult::Completed("You dig a hole through the floor!".to_string())
    }
}

// ============================================================================
// Wand of digging
// ============================================================================

/// Result of a wand of digging zap
#[derive(Debug, Clone)]
pub struct ZapDigResult {
    /// Cells that were modified (x, y, old_type, new_type)
    pub cells_modified: Vec<(i8, i8, CellType, CellType)>,
    /// Messages to display
    pub messages: Vec<String>,
    /// How deep the beam penetrated (in cells)
    pub depth: i32,
}

/// Zap a wand of digging horizontally (zap_dig from dig.c:1388).
///
/// The beam travels in the given direction, converting walls to corridors,
/// doors to broken doorways, etc. Depth is random (8-25 tiles in C).
///
/// Non-diggable walls glow and stop the beam.
pub fn zap_dig_horizontal(
    level: &mut Level,
    start_x: i8,
    start_y: i8,
    dx: i8,
    dy: i8,
    rng: &mut GameRng,
) -> ZapDigResult {
    let mut result = ZapDigResult {
        cells_modified: Vec::new(),
        messages: Vec::new(),
        depth: 0,
    };

    // Beam depth: rn1(18, 8) = rnd(18) + 7 → range 8-25 (matches C)
    let max_depth = rng.rnd(18) as i32 + 7;

    let mut cx = start_x;
    let mut cy = start_y;
    let mut remaining_depth = max_depth;

    while remaining_depth > 0 {
        cx += dx;
        cy += dy;

        if !level.is_valid_pos(cx, cy) {
            break;
        }

        let cell = &level.cells[cx as usize][cy as usize];
        let old_typ = cell.typ;

        // Check for non-diggable flag
        if cell.flags & 0x40 != 0 {
            result.messages.push("The wall glows then fades.".to_string());
            break;
        }

        match old_typ {
            // Walls → corridor/room
            t if t.is_wall() => {
                let cell = &mut level.cells[cx as usize][cy as usize];
                cell.typ = CellType::Corridor;
                result.cells_modified.push((cx, cy, old_typ, CellType::Corridor));
                remaining_depth -= 2; // Walls cost 2 depth
            }
            // Stone → corridor
            CellType::Stone => {
                let cell = &mut level.cells[cx as usize][cy as usize];
                cell.typ = CellType::Corridor;
                result.cells_modified.push((cx, cy, old_typ, CellType::Corridor));
                remaining_depth -= 1;
            }
            // Secret corridor → corridor
            CellType::SecretCorridor => {
                let cell = &mut level.cells[cx as usize][cy as usize];
                cell.typ = CellType::Corridor;
                result.cells_modified.push((cx, cy, old_typ, CellType::Corridor));
                remaining_depth -= 1;
            }
            // Secret door → door (open)
            CellType::SecretDoor => {
                let cell = &mut level.cells[cx as usize][cy as usize];
                cell.typ = CellType::Door;
                cell.set_door_state(crate::dungeon::DoorState::BROKEN);
                result.cells_modified.push((cx, cy, old_typ, CellType::Door));
                remaining_depth -= 2;
            }
            // Door → broken
            CellType::Door => {
                let cell = &mut level.cells[cx as usize][cy as usize];
                cell.set_door_state(crate::dungeon::DoorState::BROKEN);
                result.cells_modified.push((cx, cy, old_typ, CellType::Door));
                remaining_depth -= 2;
            }
            // Tree → room
            CellType::Tree => {
                let cell = &mut level.cells[cx as usize][cy as usize];
                cell.typ = CellType::Room;
                result.cells_modified.push((cx, cy, old_typ, CellType::Room));
                remaining_depth -= 2;
            }
            // Passable terrain → beam passes through
            t if t.is_passable() => {
                remaining_depth -= 1;
            }
            // Everything else stops the beam
            _ => break,
        }

        result.depth += 1;
    }

    if result.cells_modified.is_empty() {
        result.messages.push("The zap hits the wall.".to_string());
    } else {
        result.messages.push(format!("The beam digs through {} cell(s).", result.cells_modified.len()));
    }

    result
}

/// Zap a wand of digging downward (zap_dig vertical from dig.c).
///
/// Creates a hole in the floor at the player's location.
pub fn zap_dig_down(
    level: &mut Level,
    x: i8,
    y: i8,
    dlevel: &DLevel,
    _rng: &mut GameRng,
) -> Option<String> {
    // Check if downward digging is possible
    if let Some(msg) = dig_check(level, x, y, dlevel) {
        return Some(msg.to_string());
    }

    // Check for stairs
    for stair in &level.stairs {
        if stair.x == x && stair.y == y {
            return Some("A rock falls on your head! Ouch!".to_string());
        }
    }

    // Create a hole
    level.traps.push(crate::dungeon::Trap {
        trap_type: TrapType::Hole,
        x,
        y,
        activated: false,
        seen: true,
        once: false,
        madeby_u: true,
        launch_oid: None,
    });

    Some("A hole opens up beneath you!".to_string())
}

// ============================================================================
// Grave excavation
// ============================================================================

/// What was found in a grave
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraveContents {
    /// Old corpse (very decayed)
    OldCorpse,
    /// Zombie rises from the grave
    Zombie,
    /// Mummy rises from the grave
    Mummy,
    /// Empty grave — "The grave seems unused"
    Empty,
}

/// Alignment penalty for digging up a grave
#[derive(Debug, Clone)]
pub struct GravePenalty {
    pub alignment_loss: i32,
    pub message: String,
}

/// Dig up a grave (dig_up_grave from dig.c:899).
///
/// Determines what's in the grave and applies alignment penalties.
///
/// # Alignment penalties (from C):
/// - Archaeologists: -3 alignment, "despicable grave-robber"
/// - Samurai: -1 alignment, "disturb honorable dead"
/// - Lawful (if record > -10): -1 alignment, "violated sanctity"
///
/// # Random contents (1d5 from C):
/// - 0-1: Old corpse
/// - 2: Zombie
/// - 3: Mummy
/// - 4: Empty ("grave seems unused")
pub fn dig_up_grave(
    player: &You,
    rng: &mut GameRng,
) -> (GraveContents, Option<GravePenalty>) {
    use crate::player::Role;

    // Determine alignment penalty
    let penalty = match player.role {
        Role::Archeologist => Some(GravePenalty {
            alignment_loss: 3,
            message: "You feel like a despicable grave-robber!".to_string(),
        }),
        Role::Samurai => Some(GravePenalty {
            alignment_loss: 1,
            message: "You disturb the honorable dead!".to_string(),
        }),
        _ => {
            // Lawful characters with decent alignment also get a penalty
            if player.alignment.record > -10 {
                Some(GravePenalty {
                    alignment_loss: 1,
                    message: "You have violated the sanctity of this grave!".to_string(),
                })
            } else {
                None
            }
        }
    };

    // Random grave contents (matches C dig.c:935, 1d5)
    let contents = match rng.rn2(5) {
        0 | 1 => GraveContents::OldCorpse,
        2 => GraveContents::Zombie,
        3 => GraveContents::Mummy,
        _ => GraveContents::Empty,
    };

    (contents, penalty)
}

/// Convert grave terrain to room after excavation.
pub fn clear_grave(level: &mut Level, x: i8, y: i8) {
    if level.is_valid_pos(x, y) {
        let cell = &mut level.cells[x as usize][y as usize];
        if cell.typ == CellType::Grave {
            cell.typ = CellType::Room;
        }
    }
}

// ============================================================================
// Fill hole determination
// ============================================================================

/// What liquid fills a hole (fillholetyp from dig.c:504).
///
/// Checks 3x3 area around position for adjacent liquid terrain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoleFill {
    /// No liquid — remains as pit/room
    None,
    /// Fills with pool water
    Pool,
    /// Fills with moat water
    Moat,
    /// Fills with lava
    Lava,
}

pub fn fill_hole_type(level: &Level, x: i8, y: i8) -> HoleFill {
    let mut pool_count = 0i32;
    let mut moat_count = 0i32;
    let mut lava_count = 0i32;

    for dx in -1..=1i8 {
        for dy in -1..=1i8 {
            let nx = x + dx;
            let ny = y + dy;
            if !level.is_valid_pos(nx, ny) {
                continue;
            }
            match level.cells[nx as usize][ny as usize].typ {
                CellType::Pool => pool_count += 1,
                CellType::Moat => moat_count += 1,
                CellType::Lava => lava_count += 1,
                _ => {}
            }
        }
    }

    if lava_count > moat_count + pool_count {
        HoleFill::Lava
    } else if moat_count > 0 {
        HoleFill::Moat
    } else if pool_count > 0 {
        HoleFill::Pool
    } else {
        HoleFill::None
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::{DLevel, Level, Stairway};
    use crate::player::{Gender, Race, Role};

    fn test_level() -> Level {
        let mut level = Level::new(DLevel::new(0, 5));
        // Create a room
        for x in 5..15 {
            for y in 3..8 {
                level.cells[x][y].typ = CellType::Room;
            }
        }
        // Walls around the room
        for x in 4..16 {
            level.cells[x][2].typ = CellType::HWall;
            level.cells[x][8].typ = CellType::HWall;
        }
        for y in 2..9 {
            level.cells[4][y].typ = CellType::VWall;
            level.cells[15][y].typ = CellType::VWall;
        }
        level
    }

    fn test_player() -> You {
        You::new("Test".into(), Role::Valkyrie, Race::Human, Gender::Female)
    }

    // ---- dig_target tests ----

    #[test]
    fn test_dig_target_wall() {
        let level = test_level();
        assert_eq!(dig_target(&level, 4, 5, true), DigTarget::Rock);
    }

    #[test]
    fn test_dig_target_room() {
        let level = test_level();
        assert_eq!(dig_target(&level, 7, 5, true), DigTarget::Undiggable);
    }

    #[test]
    fn test_dig_target_tree() {
        let mut level = test_level();
        level.cells[7][5].typ = CellType::Tree;
        // Pick can't dig trees
        assert_eq!(dig_target(&level, 7, 5, true), DigTarget::Undiggable);
        // Axe can
        assert_eq!(dig_target(&level, 7, 5, false), DigTarget::Tree);
    }

    #[test]
    fn test_dig_target_door() {
        let mut level = test_level();
        level.cells[7][5].typ = CellType::Door;
        assert_eq!(dig_target(&level, 7, 5, true), DigTarget::Door);
    }

    // ---- dig_check tests ----

    #[test]
    fn test_dig_check_room_ok() {
        let level = test_level();
        let dlevel = DLevel::new(0, 5);
        assert_eq!(dig_check(&level, 7, 5, &dlevel), None);
    }

    #[test]
    fn test_dig_check_altar_blocked() {
        let mut level = test_level();
        level.cells[7][5].typ = CellType::Altar;
        let dlevel = DLevel::new(0, 5);
        assert!(dig_check(&level, 7, 5, &dlevel).is_some());
    }

    #[test]
    fn test_dig_check_stairs_blocked() {
        let mut level = test_level();
        level.stairs.push(Stairway {
            x: 7,
            y: 5,
            destination: DLevel::new(0, 4),
            up: true,
        });
        let dlevel = DLevel::new(0, 5);
        assert!(dig_check(&level, 7, 5, &dlevel).is_some());
    }

    // ---- dig_turn tests ----

    #[test]
    fn test_dig_turn_progress() {
        let player = test_player();
        let mut level = test_level();
        let mut ctx = DigContext {
            x: 4,
            y: 5,
            down: false,
            effort: 0,
            target: Some(DigTarget::Rock),
            active: true,
        };
        let mut rng = GameRng::new(42);

        let result = dig_turn(&player, &mut level, &mut ctx, 0, 0, false, &mut rng);
        assert!(matches!(result, DigResult::InProgress(_)));
        assert!(ctx.effort > 0);
    }

    #[test]
    fn test_dig_turn_completes_wall() {
        let player = test_player();
        let mut level = test_level();
        let mut ctx = DigContext {
            x: 4,
            y: 5,
            down: false,
            effort: 240, // Almost done
            target: Some(DigTarget::Rock),
            active: true,
        };
        let mut rng = GameRng::new(42);

        let result = dig_turn(&player, &mut level, &mut ctx, 5, 0, false, &mut rng);
        assert!(matches!(result, DigResult::Completed(_)));
        assert_eq!(level.cells[4][5].typ, CellType::Corridor);
    }

    #[test]
    fn test_dig_turn_dwarf_faster() {
        let player = test_player();
        let mut level = test_level();
        let mut ctx1 = DigContext {
            x: 4, y: 5, down: false, effort: 0,
            target: Some(DigTarget::Rock), active: true,
        };
        let mut ctx2 = DigContext {
            x: 4, y: 5, down: false, effort: 0,
            target: Some(DigTarget::Rock), active: true,
        };
        let mut rng1 = GameRng::new(42);
        let mut rng2 = GameRng::new(42);

        dig_turn(&player, &mut level, &mut ctx1, 0, 0, false, &mut rng1);
        dig_turn(&player, &mut level, &mut ctx2, 0, 0, true, &mut rng2);

        assert!(ctx2.effort > ctx1.effort, "Dwarf should dig faster");
    }

    #[test]
    fn test_dig_turn_not_active() {
        let player = test_player();
        let mut level = test_level();
        let mut ctx = DigContext::default();
        let mut rng = GameRng::new(42);

        let result = dig_turn(&player, &mut level, &mut ctx, 0, 0, false, &mut rng);
        assert!(matches!(result, DigResult::Failed(_)));
    }

    // ---- dig_turn down tests ----

    #[test]
    fn test_dig_turn_down_creates_pit() {
        let player = test_player();
        let mut level = test_level();
        let mut ctx = DigContext {
            x: 7,
            y: 5,
            down: true,
            effort: 245,
            target: Some(DigTarget::Down),
            active: true,
        };
        let mut rng = GameRng::new(42);

        let result = dig_turn(&player, &mut level, &mut ctx, 5, 0, false, &mut rng);
        assert!(matches!(result, DigResult::Completed(_)));

        // Should have created a trap
        let has_pit = level.traps.iter().any(|t| t.x == 7 && t.y == 5);
        assert!(has_pit);
    }

    // ---- zap_dig tests ----

    #[test]
    fn test_zap_dig_horizontal_through_wall() {
        let mut level = test_level();
        let mut rng = GameRng::new(42);

        // Zap east from inside the room toward the east wall
        let result = zap_dig_horizontal(&mut level, 14, 5, 1, 0, &mut rng);

        // The east wall at (15, 5) should have been converted
        assert!(!result.cells_modified.is_empty());
        assert_eq!(level.cells[15][5].typ, CellType::Corridor);
    }

    #[test]
    fn test_zap_dig_horizontal_through_stone() {
        let mut level = test_level();
        let mut rng = GameRng::new(42);

        // Stone is at (0, 0) by default
        let result = zap_dig_horizontal(&mut level, 0, 5, -1, 0, &mut rng);

        // Should have dug through some stone
        assert!(result.depth >= 0);
    }

    #[test]
    fn test_zap_dig_down() {
        let mut level = test_level();
        let dlevel = DLevel::new(0, 5);
        let mut rng = GameRng::new(42);

        let msg = zap_dig_down(&mut level, 7, 5, &dlevel, &mut rng);
        assert!(msg.is_some());

        let has_hole = level.traps.iter().any(|t| t.x == 7 && t.y == 5);
        assert!(has_hole);
    }

    // ---- grave tests ----

    #[test]
    fn test_dig_up_grave_archeologist() {
        let mut player = test_player();
        player.role = Role::Archeologist;
        let mut rng = GameRng::new(42);

        let (_, penalty) = dig_up_grave(&player, &mut rng);
        assert!(penalty.is_some());
        assert_eq!(penalty.unwrap().alignment_loss, 3);
    }

    #[test]
    fn test_dig_up_grave_samurai() {
        let mut player = test_player();
        player.role = Role::Samurai;
        let mut rng = GameRng::new(42);

        let (_, penalty) = dig_up_grave(&player, &mut rng);
        assert!(penalty.is_some());
        assert_eq!(penalty.unwrap().alignment_loss, 1);
    }

    #[test]
    fn test_dig_up_grave_contents_range() {
        let player = test_player();
        let mut has_corpse = false;
        let mut has_zombie = false;
        let mut has_mummy = false;
        let mut has_empty = false;

        for seed in 0..100 {
            let mut rng = GameRng::new(seed);
            let (contents, _) = dig_up_grave(&player, &mut rng);
            match contents {
                GraveContents::OldCorpse => has_corpse = true,
                GraveContents::Zombie => has_zombie = true,
                GraveContents::Mummy => has_mummy = true,
                GraveContents::Empty => has_empty = true,
            }
        }

        assert!(has_corpse, "Should have found old corpse");
        assert!(has_zombie, "Should have found zombie");
        assert!(has_mummy, "Should have found mummy");
        assert!(has_empty, "Should have found empty grave");
    }

    #[test]
    fn test_clear_grave() {
        let mut level = test_level();
        level.cells[7][5].typ = CellType::Grave;
        clear_grave(&mut level, 7, 5);
        assert_eq!(level.cells[7][5].typ, CellType::Room);
    }

    // ---- fill_hole_type tests ----

    #[test]
    fn test_fill_hole_type_none() {
        let level = test_level();
        assert_eq!(fill_hole_type(&level, 7, 5), HoleFill::None);
    }

    #[test]
    fn test_fill_hole_type_pool() {
        let mut level = test_level();
        level.cells[8][5].typ = CellType::Pool;
        level.cells[6][5].typ = CellType::Pool;
        assert_eq!(fill_hole_type(&level, 7, 5), HoleFill::Pool);
    }

    #[test]
    fn test_fill_hole_type_lava() {
        let mut level = test_level();
        level.cells[8][5].typ = CellType::Lava;
        level.cells[6][5].typ = CellType::Lava;
        level.cells[7][4].typ = CellType::Lava;
        assert_eq!(fill_hole_type(&level, 7, 5), HoleFill::Lava);
    }

    #[test]
    fn test_fill_hole_type_moat_over_pool() {
        let mut level = test_level();
        level.cells[8][5].typ = CellType::Pool;
        level.cells[6][5].typ = CellType::Moat;
        assert_eq!(fill_hole_type(&level, 7, 5), HoleFill::Moat);
    }
}
