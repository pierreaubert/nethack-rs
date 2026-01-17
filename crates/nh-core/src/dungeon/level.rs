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
}

/// Trap on the level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trap {
    pub x: i8,
    pub y: i8,
    pub trap_type: TrapType,
    pub activated: bool,
    pub seen: bool,
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

    /// Stairways
    pub stairs: Vec<Stairway>,

    /// Level flags
    pub flags: LevelFlags,

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
            stairs: Vec::new(),
            flags: LevelFlags::default(),
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

    /// Add a trap
    pub fn add_trap(&mut self, x: i8, y: i8, trap_type: TrapType) {
        self.traps.push(Trap {
            x,
            y,
            trap_type,
            activated: false,
            seen: false,
        });
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
}
