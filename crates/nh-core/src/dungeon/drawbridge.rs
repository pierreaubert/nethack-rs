//! Drawbridge mechanics (dbridge.c)
//!
//! Implements drawbridge creation, manipulation, and entity handling.
//! Drawbridges can be opened, closed, or destroyed, affecting entities
//! (players and monsters) standing on them.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::monster::Monster;
use crate::rng::GameRng;
use crate::{COLNO, ROWNO};

use super::cell::CellType;
use super::level::Level;

/// Drawbridge direction constants
pub mod direction {
    pub const DB_NORTH: u8 = 0;
    pub const DB_SOUTH: u8 = 1;
    pub const DB_EAST: u8 = 2;
    pub const DB_WEST: u8 = 3;
    pub const DB_DIR: u8 = 3; // Mask for direction bits
}

/// Drawbridge under-type constants (what's beneath the drawbridge)
pub mod under_type {
    pub const DB_MOAT: u8 = 0;
    pub const DB_LAVA: u8 = 4;
    pub const DB_ICE: u8 = 8;
    pub const DB_UNDER: u8 = 0x1C; // Mask for under bits
}

use direction::*;
use under_type::*;

/// Entity on a drawbridge (player or monster)
#[derive(Debug, Clone)]
pub struct Entity {
    /// Monster reference (None for player)
    pub monster_id: Option<usize>,
    /// Whether this is the player
    pub is_player: bool,
    /// Entity x position
    pub ex: i8,
    /// Entity y position
    pub ey: i8,
    /// Whether entity data is valid
    pub valid: bool,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            monster_id: None,
            is_player: false,
            ex: 0,
            ey: 0,
            valid: false,
        }
    }
}

impl Entity {
    /// Create a new entity for the player
    pub fn player(x: i8, y: i8) -> Self {
        Self {
            monster_id: None,
            is_player: true,
            ex: x,
            ey: y,
            valid: true,
        }
    }

    /// Create a new entity for a monster
    pub fn monster(monster_idx: usize, x: i8, y: i8) -> Self {
        Self {
            monster_id: Some(monster_idx),
            is_player: false,
            ex: x,
            ey: y,
            valid: true,
        }
    }
}

/// Get the underlying terrain type for a raised drawbridge
/// Matches C's db_under_typ()
///
/// # Arguments
/// * `mask` - The drawbridge mask value
///
/// # Returns
/// The CellType beneath the drawbridge
pub fn db_under_typ(mask: u8) -> CellType {
    match mask & DB_UNDER {
        DB_ICE => CellType::Ice,
        DB_LAVA => CellType::Lava,
        DB_MOAT => CellType::Moat,
        _ => CellType::Stone,
    }
}

/// Check if a wall position is a drawbridge wall and get direction
/// Matches C's is_drawbridge_wall()
///
/// # Arguments
/// * `level` - The level to check
/// * `x` - X coordinate
/// * `y` - Y coordinate
///
/// # Returns
/// Some(direction) if this is a drawbridge wall, None otherwise
pub fn is_drawbridge_wall(level: &Level, x: usize, y: usize) -> Option<u8> {
    if x >= COLNO || y >= ROWNO {
        return None;
    }

    let cell = &level.cells[x][y];
    if cell.typ != CellType::Door && cell.typ != CellType::DBWall {
        return None;
    }

    // Check adjacent cells for drawbridge
    if x + 1 < COLNO {
        let adj = &level.cells[x + 1][y];
        if is_drawbridge_type(adj.typ) && (adj.flags & DB_DIR) == DB_WEST {
            return Some(DB_WEST);
        }
    }
    if x > 0 {
        let adj = &level.cells[x - 1][y];
        if is_drawbridge_type(adj.typ) && (adj.flags & DB_DIR) == DB_EAST {
            return Some(DB_EAST);
        }
    }
    if y > 0 {
        let adj = &level.cells[x][y - 1];
        if is_drawbridge_type(adj.typ) && (adj.flags & DB_DIR) == DB_SOUTH {
            return Some(DB_SOUTH);
        }
    }
    if y + 1 < ROWNO {
        let adj = &level.cells[x][y + 1];
        if is_drawbridge_type(adj.typ) && (adj.flags & DB_DIR) == DB_NORTH {
            return Some(DB_NORTH);
        }
    }

    None
}

/// Check if a cell type is a drawbridge
fn is_drawbridge_type(typ: CellType) -> bool {
    matches!(typ, CellType::DrawbridgeUp | CellType::DrawbridgeDown)
}

/// Find the drawbridge at or adjacent to a position
/// Matches C's find_drawbridge()
///
/// # Arguments
/// * `level` - The level to search
/// * `x` - X coordinate (modified to point to drawbridge)
/// * `y` - Y coordinate (modified to point to drawbridge)
///
/// # Returns
/// Some((db_x, db_y)) if a drawbridge was found, None otherwise
pub fn find_drawbridge(level: &Level, x: usize, y: usize) -> Option<(usize, usize)> {
    if x >= COLNO || y >= ROWNO {
        return None;
    }

    // Check if this position is already a drawbridge
    if is_drawbridge_type(level.cells[x][y].typ) {
        return Some((x, y));
    }

    // Check if this is a drawbridge wall
    if let Some(dir) = is_drawbridge_wall(level, x, y) {
        let (nx, ny) = match dir {
            DB_NORTH => (x, y + 1),
            DB_SOUTH => (x, y.saturating_sub(1)),
            DB_EAST => (x.saturating_sub(1), y),
            DB_WEST => (x + 1, y),
            _ => return None,
        };
        if nx < COLNO && ny < ROWNO {
            return Some((nx, ny));
        }
    }

    None
}

/// Get the wall position for a drawbridge
/// Matches C's get_wall_for_db()
fn get_wall_for_db(level: &Level, x: usize, y: usize) -> Option<(usize, usize)> {
    if x >= COLNO || y >= ROWNO {
        return None;
    }

    let dir = level.cells[x][y].flags & DB_DIR;
    let (wx, wy) = match dir {
        DB_NORTH => (x, y.saturating_sub(1)),
        DB_SOUTH => (x, y + 1),
        DB_EAST => (x + 1, y),
        DB_WEST => (x.saturating_sub(1), y),
        _ => return None,
    };

    if wx < COLNO && wy < ROWNO {
        Some((wx, wy))
    } else {
        None
    }
}

/// Create a drawbridge at a position
/// Matches C's create_drawbridge()
///
/// # Arguments
/// * `level` - The level to modify
/// * `x` - X coordinate for drawbridge span
/// * `y` - Y coordinate for drawbridge span
/// * `dir` - Direction the drawbridge faces
/// * `open` - Whether to create it in open (lowered) state
///
/// # Returns
/// true if drawbridge was created successfully
pub fn create_drawbridge(level: &mut Level, x: usize, y: usize, dir: u8, open: bool) -> bool {
    if x >= COLNO || y >= ROWNO {
        return false;
    }

    // Calculate wall position
    let (x2, y2, horiz) = match dir {
        DB_NORTH => (x, y.saturating_sub(1), true),
        DB_SOUTH => (x, y + 1, true),
        DB_EAST => (x + 1, y, false),
        DB_WEST => (x.saturating_sub(1), y, false),
        _ => return false,
    };

    if x2 >= COLNO || y2 >= ROWNO {
        return false;
    }

    // Wall position must be a wall
    if !level.cells[x2][y2].typ.is_wall() {
        return false;
    }

    // Check if current position is lava
    let is_lava = level.cells[x][y].typ == CellType::Lava;

    if open {
        // Lowered drawbridge
        level.cells[x][y].typ = CellType::DrawbridgeDown;
        level.cells[x2][y2].typ = CellType::Door;
        level.cells[x2][y2].flags = 0; // D_NODOOR equivalent
    } else {
        // Raised drawbridge
        level.cells[x][y].typ = CellType::DrawbridgeUp;
        level.cells[x2][y2].typ = CellType::DBWall;
        level.cells[x2][y2].can_dig = false; // Non-diggable
    }

    level.cells[x][y].horizontal = !horiz;
    level.cells[x2][y2].horizontal = horiz;
    level.cells[x][y].flags = dir;
    if is_lava {
        level.cells[x][y].flags |= DB_LAVA;
    }

    true
}

/// Open (lower) a drawbridge
/// Matches C's open_drawbridge()
///
/// # Arguments
/// * `level` - The level containing the drawbridge
/// * `x` - X coordinate of the drawbridge
/// * `y` - Y coordinate of the drawbridge
///
/// # Returns
/// true if drawbridge was opened successfully
pub fn open_drawbridge(level: &mut Level, x: usize, y: usize) -> bool {
    if x >= COLNO || y >= ROWNO {
        return false;
    }

    // Must be a raised drawbridge
    if level.cells[x][y].typ != CellType::DrawbridgeUp {
        return false;
    }

    // Get the wall position
    let (wx, wy) = match get_wall_for_db(level, x, y) {
        Some(pos) => pos,
        None => return false,
    };

    // Lower the drawbridge
    level.cells[x][y].typ = CellType::DrawbridgeDown;
    level.cells[wx][wy].typ = CellType::Door;
    level.cells[wx][wy].flags = 0; // Open door

    // Remove any traps at these locations
    level.traps.retain(|t| {
        !((t.x as usize == x && t.y as usize == y) || (t.x as usize == wx && t.y as usize == wy))
    });

    true
}

/// Close (raise) a drawbridge
/// Matches C's close_drawbridge()
///
/// # Arguments
/// * `level` - The level containing the drawbridge
/// * `x` - X coordinate of the drawbridge
/// * `y` - Y coordinate of the drawbridge
///
/// # Returns
/// true if drawbridge was closed successfully
pub fn close_drawbridge(level: &mut Level, x: usize, y: usize) -> bool {
    if x >= COLNO || y >= ROWNO {
        return false;
    }

    // Must be a lowered drawbridge
    if level.cells[x][y].typ != CellType::DrawbridgeDown {
        return false;
    }

    // Get the wall position
    let (wx, wy) = match get_wall_for_db(level, x, y) {
        Some(pos) => pos,
        None => return false,
    };

    // Raise the drawbridge
    level.cells[x][y].typ = CellType::DrawbridgeUp;
    level.cells[wx][wy].typ = CellType::DBWall;
    level.cells[wx][wy].can_dig = false;

    // Remove any traps at these locations
    level.traps.retain(|t| {
        !((t.x as usize == x && t.y as usize == y) || (t.x as usize == wx && t.y as usize == wy))
    });

    true
}

/// Destroy a drawbridge
/// Matches C's destroy_drawbridge()
///
/// # Arguments
/// * `level` - The level containing the drawbridge
/// * `x` - X coordinate of the drawbridge
/// * `y` - Y coordinate of the drawbridge
///
/// # Returns
/// true if drawbridge was destroyed successfully
pub fn destroy_drawbridge(level: &mut Level, x: usize, y: usize) -> bool {
    if x >= COLNO || y >= ROWNO {
        return false;
    }

    let cell_type = level.cells[x][y].typ;
    if !is_drawbridge_type(cell_type) {
        return false;
    }

    // Get the wall position
    let wall_pos = get_wall_for_db(level, x, y);

    // Determine what terrain to replace with
    let under_type = if cell_type == CellType::DrawbridgeUp {
        db_under_typ(level.cells[x][y].flags)
    } else {
        // For lowered bridge, check what's stored
        let mask = level.cells[x][y].flags;
        db_under_typ(mask)
    };

    // Replace drawbridge with underlying terrain
    level.cells[x][y].typ = under_type;
    level.cells[x][y].flags = 0;

    // Replace wall with floor/door opening
    if let Some((wx, wy)) = wall_pos {
        level.cells[wx][wy].typ = CellType::Room;
        level.cells[wx][wy].flags = 0;
    }

    // Remove any traps
    level
        .traps
        .retain(|t| t.x as usize != x || t.y as usize != y);
    if let Some((wx, wy)) = wall_pos {
        level
            .traps
            .retain(|t| t.x as usize != wx || t.y as usize != wy);
    }

    true
}

/// Lower the drawbridge gate (alternative to open)
/// Matches C's down_gate()
pub fn down_gate(level: &mut Level, x: usize, y: usize) -> bool {
    open_drawbridge(level, x, y)
}

/// Set up an entity structure for a position
/// Matches C's set_entity()
///
/// # Arguments
/// * `level` - The level to check
/// * `x` - X coordinate
/// * `y` - Y coordinate
/// * `player_x` - Player's x position
/// * `player_y` - Player's y position
///
/// # Returns
/// Some(Entity) if there's a player or monster at the position
pub fn set_entity(level: &Level, x: i8, y: i8, player_x: i8, player_y: i8) -> Option<Entity> {
    // Check for player
    if x == player_x && y == player_y {
        return Some(Entity::player(x, y));
    }

    // Check for monster
    for (idx, monster) in level.monsters.iter().enumerate() {
        if monster.x == x && monster.y == y {
            return Some(Entity::monster(idx, x, y));
        }
    }

    None
}

/// Get the name of an entity
/// Matches C's e_nam()
pub fn e_nam(entity: &Entity, monsters: &[Monster]) -> String {
    if entity.is_player {
        "you".to_string()
    } else if let Some(idx) = entity.monster_id {
        if let Some(monster) = monsters.get(idx) {
            monster.name.clone()
        } else {
            "something".to_string()
        }
    } else {
        "something".to_string()
    }
}

/// Generate a phrase for entity action
/// Matches C's E_phrase()
///
/// # Arguments
/// * `entity` - The entity performing the action
/// * `verb` - The verb to use
/// * `monsters` - Monster list for name lookup
///
/// # Returns
/// A phrase like "You fall" or "The goblin falls"
pub fn e_phrase(entity: &Entity, verb: &str, monsters: &[Monster]) -> String {
    if entity.is_player {
        format!("You {}", verb)
    } else if let Some(idx) = entity.monster_id {
        if let Some(monster) = monsters.get(idx) {
            // Convert verb to third person (simple -s suffix)
            let verb_3rd = if verb.ends_with('s')
                || verb.ends_with('x')
                || verb.ends_with("ch")
                || verb.ends_with("sh")
            {
                format!("{}es", verb)
            } else if verb.ends_with('y')
                && !verb.ends_with("ay")
                && !verb.ends_with("ey")
                && !verb.ends_with("oy")
                && !verb.ends_with("uy")
            {
                format!("{}ies", &verb[..verb.len() - 1])
            } else {
                format!("{}s", verb)
            };
            format!("The {} {}", monster.name, verb_3rd)
        } else {
            format!("Something {}s", verb)
        }
    } else {
        format!("Something {}s", verb)
    }
}

/// Check if an entity can survive at a position
/// Matches C's e_survives_at()
pub fn e_survives_at(
    _entity: &Entity,
    level: &Level,
    x: usize,
    y: usize,
    can_fly: bool,
    can_levitate: bool,
    can_swim: bool,
    passes_walls: bool,
) -> bool {
    if x >= COLNO || y >= ROWNO {
        return false;
    }

    let cell_type = level.cells[x][y].typ;

    // Water/pool survival
    if cell_type.is_water() {
        return can_fly || can_levitate || can_swim;
    }

    // Lava survival
    if cell_type == CellType::Lava {
        return can_fly || can_levitate;
    }

    // Wall survival (raised drawbridge wall)
    if cell_type == CellType::DBWall {
        return passes_walls;
    }

    true
}

/// Handle entity death from drawbridge
/// Matches C's e_died()
pub fn e_died(entity: &Entity, how: &str) -> String {
    if entity.is_player {
        format!("You are killed by {}!", how)
    } else {
        format!("killed by {}", how)
    }
}

/// Check if entity automatically misses drawbridge effects
/// Matches C's automiss()
pub fn automiss(passes_walls: bool, noncorporeal: bool) -> bool {
    passes_walls || noncorporeal
}

/// Check if a falling drawbridge misses an entity
/// Matches C's e_missed()
///
/// # Arguments
/// * `rng` - Random number generator
/// * `chunks` - Whether this is crushing chunks (true) or portcullis (false)
/// * `entity_ac` - Entity's armor class
///
/// # Returns
/// true if the drawbridge missed the entity
pub fn e_missed(rng: &mut GameRng, chunks: bool, entity_ac: i32) -> bool {
    if chunks {
        // Chunks: random chance based on AC
        let roll = rng.rn2(8) as i32;
        roll < entity_ac / 3
    } else {
        // Portcullis: straight random chance
        rng.rn2(5) != 0
    }
}

/// Check if entity jumps out of the way
/// Matches C's e_jumps()
pub fn e_jumps(rng: &mut GameRng, dexterity: i32) -> bool {
    // Chance based on dexterity
    (rng.rn2(20) as i32) < dexterity
}

/// Process drawbridge effects on an entity
/// Matches C's do_entity()
///
/// This handles the effects of a drawbridge opening or closing on an entity.
///
/// # Arguments
/// * `level` - The level
/// * `entity` - The entity being affected
/// * `db_x` - Drawbridge x position
/// * `db_y` - Drawbridge y position
/// * `opening` - true if opening, false if closing
/// * `rng` - Random number generator
///
/// # Returns
/// (survived: bool, message: Option<String>)
pub fn do_entity(
    level: &Level,
    entity: &Entity,
    db_x: usize,
    db_y: usize,
    opening: bool,
    rng: &mut GameRng,
) -> (bool, Option<String>) {
    if !entity.valid {
        return (true, None);
    }

    let ex = entity.ex as usize;
    let ey = entity.ey as usize;

    // Check if entity is on or adjacent to drawbridge
    let on_drawbridge = ex == db_x && ey == db_y;
    let _on_wall = if let Some((wx, wy)) = get_wall_for_db(level, db_x, db_y) {
        ex == wx && ey == wy
    } else {
        false
    };

    if !on_drawbridge {
        return (true, None);
    }

    if opening {
        // Drawbridge lowering - entity might fall into water/lava
        let under = db_under_typ(level.cells[db_x][db_y].flags);
        if under == CellType::Moat || under == CellType::Lava {
            // Entity might fall
            if e_jumps(rng, 12) {
                // Default dexterity
                return (
                    true,
                    Some("jumps out of the way of the lowering drawbridge".to_string()),
                );
            }
            // Falls in
            let surface = if under == CellType::Lava {
                "lava"
            } else {
                "water"
            };
            return (false, Some(format!("falls into the {}!", surface)));
        }
    } else {
        // Drawbridge closing - entity might be crushed
        if automiss(false, false) {
            return (true, None);
        }

        if e_missed(rng, false, 10) {
            return (
                true,
                Some("narrowly escapes being crushed by the drawbridge".to_string()),
            );
        }

        // Crushed
        return (
            false,
            Some("is crushed by the closing drawbridge!".to_string()),
        );
    }

    (true, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::DLevel;

    #[test]
    fn test_db_under_typ() {
        assert_eq!(db_under_typ(DB_ICE), CellType::Ice);
        assert_eq!(db_under_typ(DB_LAVA), CellType::Lava);
        assert_eq!(db_under_typ(DB_MOAT), CellType::Moat);
        // DB_MOAT is 0, so db_under_typ(0) also returns Moat
        assert_eq!(db_under_typ(0), CellType::Moat);
    }

    #[test]
    fn test_create_drawbridge() {
        let mut level = Level::new(DLevel::new(0, 1));

        // Set up a wall for the drawbridge
        level.cells[20][10].typ = CellType::HWall;

        // Create a closed drawbridge
        let result = create_drawbridge(&mut level, 20, 11, DB_NORTH, false);
        assert!(result);
        assert_eq!(level.cells[20][11].typ, CellType::DrawbridgeUp);
        assert_eq!(level.cells[20][10].typ, CellType::DBWall);
    }

    #[test]
    fn test_open_close_drawbridge() {
        let mut level = Level::new(DLevel::new(0, 1));

        // Set up a wall
        level.cells[20][10].typ = CellType::HWall;

        // Create closed drawbridge
        create_drawbridge(&mut level, 20, 11, DB_NORTH, false);

        // Open it
        let opened = open_drawbridge(&mut level, 20, 11);
        assert!(opened);
        assert_eq!(level.cells[20][11].typ, CellType::DrawbridgeDown);

        // Close it
        let closed = close_drawbridge(&mut level, 20, 11);
        assert!(closed);
        assert_eq!(level.cells[20][11].typ, CellType::DrawbridgeUp);
    }

    #[test]
    fn test_destroy_drawbridge() {
        let mut level = Level::new(DLevel::new(0, 1));

        // Set up with lava underneath
        level.cells[20][11].typ = CellType::Lava;
        level.cells[20][10].typ = CellType::HWall;

        // Create drawbridge
        create_drawbridge(&mut level, 20, 11, DB_NORTH, false);

        // Destroy it
        let destroyed = destroy_drawbridge(&mut level, 20, 11);
        assert!(destroyed);
        // Should reveal lava underneath
    }

    #[test]
    fn test_find_drawbridge() {
        let mut level = Level::new(DLevel::new(0, 1));

        level.cells[20][10].typ = CellType::HWall;
        create_drawbridge(&mut level, 20, 11, DB_NORTH, false);

        // Find from drawbridge position
        let found = find_drawbridge(&level, 20, 11);
        assert_eq!(found, Some((20, 11)));

        // Find from wall position
        let found_from_wall = find_drawbridge(&level, 20, 10);
        assert_eq!(found_from_wall, Some((20, 11)));
    }

    #[test]
    fn test_e_nam() {
        let player = Entity::player(5, 5);
        let monsters: Vec<Monster> = vec![];
        assert_eq!(e_nam(&player, &monsters), "you");
    }

    #[test]
    fn test_e_phrase() {
        let player = Entity::player(5, 5);
        let monsters: Vec<Monster> = vec![];
        assert_eq!(e_phrase(&player, "fall", &monsters), "You fall");
    }

    #[test]
    fn test_automiss() {
        assert!(automiss(true, false));
        assert!(automiss(false, true));
        assert!(!automiss(false, false));
    }

    #[test]
    fn test_e_missed() {
        let mut rng = GameRng::new(42);
        // Just verify it returns a boolean
        let _result = e_missed(&mut rng, true, 10);
        let _result2 = e_missed(&mut rng, false, 10);
    }
}
