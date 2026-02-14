//! Level structure (dlevel_t from rm.h)

use serde::{Deserialize, Serialize};

use super::{Cell, DLevel};
use crate::monster::{Monster, MonsterId};
use crate::object::{Object, ObjectId};
use crate::{COLNO, ROWNO};

/// Create default cells grid
fn default_cells() -> Vec<Vec<Cell>> {
    vec![vec![Cell::stone(); ROWNO]; COLNO]
}

/// Create default object grid
fn default_object_grid() -> Vec<Vec<Vec<ObjectId>>> {
    vec![vec![Vec::new(); ROWNO]; COLNO]
}

/// Create default monster grid
fn default_monster_grid() -> Vec<Vec<Option<MonsterId>>> {
    vec![vec![None; ROWNO]; COLNO]
}

/// Create default explored grid (all false)
fn default_explored() -> Vec<Vec<bool>> {
    vec![vec![false; ROWNO]; COLNO]
}

/// Create default visible grid (all false)
fn default_visible() -> Vec<Vec<bool>> {
    vec![vec![false; ROWNO]; COLNO]
}

/// Engraving types (from engrave.c)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum EngravingType {
    #[default]
    Dust = 0,      // Written in dust (easily erased)
    Engrave = 1,   // Engraved (permanent)
    Burn = 2,      // Burned (permanent)
    Mark = 3,      // Marked with marker
    BloodStain = 4, // Written in blood
    Headstone = 5, // Grave inscription
}

/// An engraving on the floor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engraving {
    pub x: i8,
    pub y: i8,
    pub text: String,
    pub engr_type: EngravingType,
    pub time: i64,
}

impl Engraving {
    pub fn new(x: i8, y: i8, text: String, engr_type: EngravingType) -> Self {
        Self {
            x,
            y,
            text,
            engr_type,
            time: 0,
        }
    }
}

/// Level flags
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct LevelFlags {
    pub fountain_count: u8,
    pub sink_count: u8,
    pub has_shop: bool,
    pub has_vault: bool,
    pub has_zoo: bool,
    pub has_court: bool,
    pub has_morgue: bool,
    pub has_beehive: bool,
    pub has_barracks: bool,
    pub has_temple: bool,
    pub has_swamp: bool,
    pub no_teleport: bool,
    pub hard_floor: bool,
    pub no_magic_map: bool,
    pub hero_memory: bool,
    pub shortsighted: bool,
    pub graveyard: bool,
    pub sokoban_rules: bool,
    pub is_maze: bool,
    pub is_cavernous: bool,
    pub arboreal: bool,
    pub wizard_bones: bool,
    pub corridor_maze: bool,
    pub has_branch: bool,
}

/// Trap on the level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trap {
    pub x: i8,
    pub y: i8,
    pub trap_type: TrapType,
    pub activated: bool,
    pub seen: bool,
    /// One-shot trap (destroyed after triggering, e.g. land mine)
    pub once: bool,
    /// Trap was set by the player
    pub madeby_u: bool,
    /// Object ID for traps that launch objects (rolling boulder)
    pub launch_oid: Option<u32>,
}

/// Trap types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrapType {
    Arrow,
    Dart,
    RockFall,
    Squeaky,
    BearTrap,
    LandMine,
    RollingBoulder,
    SleepingGas,
    RustTrap,
    FireTrap,
    Pit,
    SpikedPit,
    Hole,
    TrapDoor,
    Teleport,
    LevelTeleport,
    MagicPortal,
    Web,
    Statue,
    MagicTrap,
    AntiMagic,
    Polymorph,
}

/// Stairway
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Stairway {
    pub x: i8,
    pub y: i8,
    pub destination: DLevel,
    pub up: bool,
}

/// Complete level structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    /// Level identifier
    pub dlevel: DLevel,

    /// Map cells
    #[serde(default = "default_cells")]
    pub cells: Vec<Vec<Cell>>,

    /// Object grid (object IDs at each position)
    #[serde(skip, default = "default_object_grid")]
    pub object_grid: Vec<Vec<Vec<ObjectId>>>,

    /// Monster grid (monster ID at each position)
    #[serde(skip, default = "default_monster_grid")]
    pub monster_grid: Vec<Vec<Option<MonsterId>>>,

    /// All objects on this level
    pub objects: Vec<Object>,

    /// Buried objects
    pub buried_objects: Vec<Object>,

    /// All monsters on this level
    pub monsters: Vec<Monster>,

    /// Traps
    pub traps: Vec<Trap>,

    /// Engravings
    pub engravings: Vec<Engraving>,

    /// Shops on this level
    pub shops: Vec<crate::special::shk::Shop>,

    /// Stairways
    pub stairs: Vec<Stairway>,

    /// Level flags
    pub flags: LevelFlags,

    /// Explored cells (player has seen at some point)
    #[serde(default = "default_explored")]
    pub explored: Vec<Vec<bool>>,

    /// Currently visible cells (in player's field of view)
    #[serde(skip, default = "default_visible")]
    pub visible: Vec<Vec<bool>>,

    /// Next object ID to assign
    next_object_id: u32,

    /// Next monster ID to assign
    next_monster_id: u32,
}

impl Default for Level {
    fn default() -> Self {
        Self::new(DLevel::default())
    }
}

impl Level {
    /// Create a new empty level
    pub fn new(dlevel: DLevel) -> Self {
        Self {
            dlevel,
            cells: default_cells(),
            object_grid: default_object_grid(),
            monster_grid: default_monster_grid(),
            objects: Vec::new(),
            buried_objects: Vec::new(),
            monsters: Vec::new(),
            traps: Vec::new(),
            engravings: Vec::new(),
            shops: Vec::new(),
            stairs: Vec::new(),
            flags: LevelFlags::default(),
            explored: default_explored(),
            visible: default_visible(),
            next_object_id: 1,
            next_monster_id: 1,
        }
    }

    /// Create a new level with generated content
    pub fn new_generated(dlevel: DLevel, rng: &mut crate::rng::GameRng) -> Self {
        let mut level = Self::new(dlevel);
        super::generation::generate_rooms_and_corridors(&mut level, rng);
        level
    }

    /// Get cell at position
    pub fn cell(&self, x: usize, y: usize) -> &Cell {
        &self.cells[x][y]
    }

    /// Get mutable cell at position
    pub fn cell_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[x][y]
    }

    /// Check if position is valid
    pub const fn is_valid_pos(&self, x: i8, y: i8) -> bool {
        x >= 0 && y >= 0 && (x as usize) < COLNO && (y as usize) < ROWNO
    }

    /// Check if position is walkable
    pub fn is_walkable(&self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return false;
        }
        self.cells[x as usize][y as usize].is_walkable()
    }

    /// Get monster at position
    pub fn monster_at(&self, x: i8, y: i8) -> Option<&Monster> {
        if !self.is_valid_pos(x, y) {
            return None;
        }
        let id = self.monster_grid[x as usize][y as usize]?;
        self.monsters.iter().find(|m| m.id == id)
    }

    /// Get mutable monster at position
    pub fn monster_at_mut(&mut self, x: i8, y: i8) -> Option<&mut Monster> {
        if !self.is_valid_pos(x, y) {
            return None;
        }
        let id = self.monster_grid[x as usize][y as usize]?;
        self.monsters.iter_mut().find(|m| m.id == id)
    }

    /// Get monster by ID
    pub fn monster(&self, id: MonsterId) -> Option<&Monster> {
        self.monsters.iter().find(|m| m.id == id)
    }

    /// Get mutable monster by ID
    pub fn monster_mut(&mut self, id: MonsterId) -> Option<&mut Monster> {
        self.monsters.iter_mut().find(|m| m.id == id)
    }

    /// Add a monster to the level
    pub fn add_monster(&mut self, mut monster: Monster) -> MonsterId {
        let id = MonsterId(self.next_monster_id);
        self.next_monster_id += 1;
        monster.id = id;

        let x = monster.x as usize;
        let y = monster.y as usize;
        self.monster_grid[x][y] = Some(id);
        self.monsters.push(monster);
        id
    }

    /// Remove a monster from the level
    pub fn remove_monster(&mut self, id: MonsterId) -> Option<Monster> {
        let idx = self.monsters.iter().position(|m| m.id == id)?;
        let monster = self.monsters.remove(idx);
        self.monster_grid[monster.x as usize][monster.y as usize] = None;
        Some(monster)
    }

    /// Move a monster to a new position
    pub fn move_monster(&mut self, id: MonsterId, new_x: i8, new_y: i8) -> bool {
        let monster = self.monsters.iter_mut().find(|m| m.id == id);
        if let Some(monster) = monster {
            let old_x = monster.x as usize;
            let old_y = monster.y as usize;
            self.monster_grid[old_x][old_y] = None;
            monster.x = new_x;
            monster.y = new_y;
            self.monster_grid[new_x as usize][new_y as usize] = Some(id);
            true
        } else {
            false
        }
    }

    /// Get objects at position
    pub fn objects_at(&self, x: i8, y: i8) -> Vec<&Object> {
        if !self.is_valid_pos(x, y) {
            return Vec::new();
        }
        let ids = &self.object_grid[x as usize][y as usize];
        ids.iter()
            .filter_map(|id| self.objects.iter().find(|o| o.id == *id))
            .collect()
    }

    /// Add an object to the level
    pub fn add_object(&mut self, mut object: Object, x: i8, y: i8) -> ObjectId {
        let id = ObjectId(self.next_object_id);
        self.next_object_id += 1;
        object.id = id;
        object.x = x;
        object.y = y;

        self.object_grid[x as usize][y as usize].push(id);
        self.objects.push(object);
        id
    }

    /// Remove an object from the level
    pub fn remove_object(&mut self, id: ObjectId) -> Option<Object> {
        let idx = self.objects.iter().position(|o| o.id == id)?;
        let object = self.objects.remove(idx);
        let grid = &mut self.object_grid[object.x as usize][object.y as usize];
        grid.retain(|&oid| oid != id);
        Some(object)
    }

    /// Get trap at position
    pub fn trap_at(&self, x: i8, y: i8) -> Option<&Trap> {
        self.traps
            .iter()
            .find(|t| t.x == x && t.y == y)
    }

    /// Get shops on this level
    pub fn shops(&self) -> &[crate::special::shk::Shop] {
        &self.shops
    }

    /// Add a trap
    pub fn add_trap(&mut self, x: i8, y: i8, trap_type: TrapType) {
        self.traps.push(crate::dungeon::trap::create_trap(x, y, trap_type));
    }

    /// Get a mutable reference to a trap at position
    pub fn trap_at_mut(&mut self, x: i8, y: i8) -> Option<&mut Trap> {
        self.traps
            .iter_mut()
            .find(|t| t.x == x && t.y == y)
    }

    /// Remove a trap at the given position
    pub fn remove_trap(&mut self, x: i8, y: i8) {
        self.traps.retain(|t| t.x != x || t.y != y);
    }

    /// Find upstairs
    pub fn find_upstairs(&self) -> Option<(i8, i8)> {
        self.stairs
            .iter()
            .find(|s| s.up)
            .map(|s| (s.x, s.y))
    }

    /// Find downstairs
    pub fn find_downstairs(&self) -> Option<(i8, i8)> {
        self.stairs
            .iter()
            .find(|s| !s.up)
            .map(|s| (s.x, s.y))
    }

    /// Get stairway at position
    pub fn stairway_at(&self, x: i8, y: i8) -> Option<&Stairway> {
        self.stairs.iter().find(|s| s.x == x && s.y == y)
    }

    /// Check if a cell is explored (player has seen it before)
    pub fn is_explored(&self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return false;
        }
        self.explored[x as usize][y as usize]
    }

    /// Check if a cell is currently visible (in player's field of view)
    pub fn is_visible(&self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return false;
        }
        self.visible[x as usize][y as usize]
    }

    /// Mark a cell as explored
    pub fn set_explored(&mut self, x: i8, y: i8) {
        if self.is_valid_pos(x, y) {
            self.explored[x as usize][y as usize] = true;
        }
    }

    /// Update visibility from player position
    /// Uses simple raycasting for line of sight
    pub fn update_visibility(&mut self, player_x: i8, player_y: i8, sight_range: i32) {
        // Clear current visibility
        for col in &mut self.visible {
            for cell in col {
                *cell = false;
            }
        }

        // Player's position is always visible
        if self.is_valid_pos(player_x, player_y) {
            self.visible[player_x as usize][player_y as usize] = true;
            self.explored[player_x as usize][player_y as usize] = true;
        }

        // Cast rays in all directions
        let range = sight_range;
        for dx in -range..=range {
            for dy in -range..=range {
                // Skip if outside sight range (circular)
                if dx * dx + dy * dy > range * range {
                    continue;
                }

                let target_x = player_x + dx as i8;
                let target_y = player_y + dy as i8;

                if self.is_valid_pos(target_x, target_y) {
                    if self.has_line_of_sight(player_x, player_y, target_x, target_y) {
                        self.visible[target_x as usize][target_y as usize] = true;
                        self.explored[target_x as usize][target_y as usize] = true;
                    }
                }
            }
        }
    }

    /// Check if there's line of sight between two points (Bresenham's algorithm)
    pub fn has_line_of_sight(&self, x0: i8, y0: i8, x1: i8, y1: i8) -> bool {
        let mut x = x0 as i32;
        let mut y = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;

        let dx = (x1 - x).abs();
        let dy = -(y1 - y).abs();
        let sx = if x < x1 { 1 } else { -1 };
        let sy = if y < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            // Check if current position blocks sight (but allow seeing the blocking tile)
            if x != x0 as i32 || y != y0 as i32 {
                if !self.is_valid_pos(x as i8, y as i8) {
                    return false;
                }
                let cell = &self.cells[x as usize][y as usize];
                // Walls and closed doors block sight
                if cell.blocks_sight() {
                    // Can see the blocking tile itself, but not beyond
                    return x == x1 && y == y1;
                }
            }

            if x == x1 && y == y1 {
                return true;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::CellType;

    #[test]
    fn test_visibility_initial_state() {
        let level = Level::new(DLevel::default());
        // Initially nothing is explored or visible
        assert!(!level.is_explored(10, 10));
        assert!(!level.is_visible(10, 10));
    }

    #[test]
    fn test_visibility_update() {
        let mut level = Level::new(DLevel::default());
        // Create a simple room
        for x in 5..15 {
            for y in 5..15 {
                level.cells[x][y] = Cell::floor();
            }
        }

        // Update visibility from center of room
        level.update_visibility(10, 10, 5);

        // Player position should be visible and explored
        assert!(level.is_visible(10, 10));
        assert!(level.is_explored(10, 10));

        // Nearby cells should be visible
        assert!(level.is_visible(11, 10));
        assert!(level.is_visible(10, 11));

        // Far cells should not be visible
        assert!(!level.is_visible(0, 0));
        assert!(!level.is_explored(0, 0));
    }

    #[test]
    fn test_visibility_persists_explored() {
        let mut level = Level::new(DLevel::default());
        // Create a simple room
        for x in 5..25 {
            for y in 5..15 {
                level.cells[x][y] = Cell::floor();
            }
        }

        // Update visibility from one position
        level.update_visibility(10, 10, 5);
        assert!(level.is_explored(10, 10));
        assert!(level.is_explored(12, 10));

        // Move to new position
        level.update_visibility(20, 10, 5);

        // Old position should still be explored but not visible
        assert!(level.is_explored(10, 10));
        assert!(!level.is_visible(10, 10));

        // New position should be visible and explored
        assert!(level.is_visible(20, 10));
        assert!(level.is_explored(20, 10));
    }

    #[test]
    fn test_line_of_sight_blocked_by_wall() {
        let mut level = Level::new(DLevel::default());
        // Create a room with a wall in the middle
        for x in 5..15 {
            for y in 5..15 {
                level.cells[x][y] = Cell::floor();
            }
        }
        // Add a wall
        level.cells[10][10].typ = CellType::VWall;

        // Check line of sight
        assert!(level.has_line_of_sight(8, 10, 9, 10)); // Before wall
        assert!(level.has_line_of_sight(8, 10, 10, 10)); // Can see the wall itself
        assert!(!level.has_line_of_sight(8, 10, 11, 10)); // Blocked by wall
    }

    #[test]
    fn test_set_explored() {
        let mut level = Level::new(DLevel::default());
        assert!(!level.is_explored(10, 10));

        level.set_explored(10, 10);
        assert!(level.is_explored(10, 10));
    }
}
