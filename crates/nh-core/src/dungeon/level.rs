//! Level structure (dlevel_t from rm.h)

use serde::{Deserialize, Serialize};

use super::{Cell, CellType, DLevel};
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
    Dust = 0, // Written in dust (easily erased)
    Engrave = 1,    // Engraved (permanent)
    Burn = 2,       // Burned (permanent)
    Mark = 3,       // Marked with marker
    BloodStain = 4, // Written in blood
    Headstone = 5,  // Grave inscription
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

/// Light source type - what the light source is attached to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightSourceType {
    /// Light source attached to an object
    Object,
    /// Light source attached to a monster
    Monster,
}

/// Light source flags
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LightSourceFlags {
    /// Should be displayed/considered
    pub show: bool,
    /// Needs object/monster ID fixup after restore
    pub needs_fixup: bool,
}

/// Mobile light source (from light.c)
///
/// Light sources are "things" that have a physical position and range.
/// They can be attached to objects (lamps, candles) or monsters (fire vortex).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightSource {
    /// X position on the level
    pub x: i8,
    /// Y position on the level
    pub y: i8,
    /// Light radius (1-15, MAX_RADIUS in NetHack)
    pub range: i32,
    /// Type of light source
    pub source_type: LightSourceType,
    /// ID of the object or monster this is attached to
    pub id: u32,
    /// Flags
    pub flags: LightSourceFlags,
}

impl LightSource {
    /// Create a new light source attached to an object
    pub fn from_object(x: i8, y: i8, range: i32, object_id: ObjectId) -> Self {
        Self {
            x,
            y,
            range: range.clamp(1, 15),
            source_type: LightSourceType::Object,
            id: object_id.0,
            flags: LightSourceFlags::default(),
        }
    }

    /// Create a new light source attached to a monster
    pub fn from_monster(x: i8, y: i8, range: i32, monster_id: MonsterId) -> Self {
        Self {
            x,
            y,
            range: range.clamp(1, 15),
            source_type: LightSourceType::Monster,
            id: monster_id.0,
            flags: LightSourceFlags::default(),
        }
    }

    /// Check if this light source is attached to a specific object
    pub fn is_for_object(&self, object_id: ObjectId) -> bool {
        self.source_type == LightSourceType::Object && self.id == object_id.0
    }

    /// Check if this light source is attached to a specific monster
    pub fn is_for_monster(&self, monster_id: MonsterId) -> bool {
        self.source_type == LightSourceType::Monster && self.id == monster_id.0
    }
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

    /// Mobile light sources (lamps, candles, fire vortexes, etc.)
    pub light_sources: Vec<LightSource>,

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

    /// Phase 19: Terrain modifications tracker
    pub terrain_modifications: crate::magic::terrain_modification::TerrainModificationTracker,

    /// Phase 19: Persistent spell effects tracker
    pub persistent_effects: crate::magic::spell_persistence::PersistentEffectTracker,

    /// Message buffer for AI/monster actions (pline equivalent)
    /// These messages are collected and transferred to GameState after monster turns
    #[serde(skip, default)]
    pub pending_messages: Vec<String>,
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
            light_sources: Vec::new(),
            stairs: Vec::new(),
            flags: LevelFlags::default(),
            explored: default_explored(),
            visible: default_visible(),
            next_object_id: 1,
            next_monster_id: 1,
            terrain_modifications:
                crate::magic::terrain_modification::TerrainModificationTracker::new(),
            persistent_effects: crate::magic::spell_persistence::PersistentEffectTracker::new(),
            pending_messages: Vec::new(),
        }
    }

    /// Add a message to the pending message buffer (pline equivalent for AI/monster code)
    /// These messages will be transferred to GameState after monster turns
    pub fn pline(&mut self, msg: impl Into<String>) {
        self.pending_messages.push(msg.into());
    }

    /// Take all pending messages, clearing the buffer
    pub fn take_pending_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending_messages)
    }

    /// Create a new level with generated content
    pub fn new_generated(
        dlevel: DLevel,
        rng: &mut crate::rng::GameRng,
        monster_vitals: &crate::magic::MonsterVitals,
    ) -> Self {
        let mut level = Self::new(dlevel);
        super::generation::generate_rooms_and_corridors(&mut level, rng, monster_vitals);
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

    /// Get an iterator over all monster IDs on this level
    pub fn monster_ids(&self) -> impl Iterator<Item = MonsterId> + '_ {
        self.monsters.iter().map(|m| m.id)
    }

    /// Add a monster to the level
    pub fn add_monster(&mut self, mut monster: Monster) -> MonsterId {
        let id = MonsterId(self.next_monster_id);
        self.next_monster_id += 1;
        monster.id = id;

        // Phase 18: Initialize personality and combat systems on monster spawn
        {
            use crate::monster::{assign_personality, combat_hooks, monster_intelligence};

            // Assign personality based on intelligence
            let intelligence = monster_intelligence(monster.monster_type);
            monster.personality = assign_personality(intelligence, id.0);

            // Initialize combat resources based on level
            monster.resources.initialize(monster.level);
        }

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

        // Phase 18: Notify nearby monsters of ally death for morale tracking
        {
            use crate::monster::combat_hooks;
            let dead_x = monster.x;
            let dead_y = monster.y;

            // Find all nearby witnesses and trigger morale events
            for other in self.monsters.iter_mut() {
                let dist_sq =
                    ((other.x - dead_x) as i32).pow(2) + ((other.y - dead_y) as i32).pow(2);
                // Notify if within ~10 squares (100 distance squared)
                if dist_sq <= 100 {
                    // Same type allies feel more impact
                    if other.monster_type == monster.monster_type {
                        use crate::monster::morale::MoraleEvent;
                        other.morale.add_event(MoraleEvent::AlliedDeath);
                        other.morale.ally_deaths_witnessed =
                            other.morale.ally_deaths_witnessed.saturating_add(1);
                    } else if dist_sq <= 25 {
                        // Nearby death of any creature still affects morale
                        use crate::monster::morale::MoraleEvent;
                        other.morale.add_event(MoraleEvent::AlliedDeath);
                        other.morale.ally_deaths_witnessed =
                            other.morale.ally_deaths_witnessed.saturating_add(1);
                    }
                }
            }
        }

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
        self.traps.iter().find(|t| t.x == x && t.y == y)
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
        self.stairs.iter().find(|s| s.up).map(|s| (s.x, s.y))
    }

    /// Find downstairs
    pub fn find_downstairs(&self) -> Option<(i8, i8)> {
        self.stairs.iter().find(|s| !s.up).map(|s| (s.x, s.y))
    }

    /// Check if level has upstairs (has_upstairs equivalent)
    pub fn has_upstairs(&self) -> bool {
        self.stairs.iter().any(|s| s.up)
    }

    /// Check if level has downstairs (has_dnstairs equivalent)
    pub fn has_dnstairs(&self) -> bool {
        self.stairs.iter().any(|s| !s.up)
    }

    /// Check if level has a ceiling (has_ceiling equivalent)
    ///
    /// Most levels have ceilings, except for Plane of Air
    pub fn has_ceiling(&self) -> bool {
        // Plane of Air (air levels) don't have ceilings
        !matches!(self.dlevel.dungeon_num, 7) // 7 = Planes (assuming air plane has no ceiling)
    }

    /// Check if we're on the specified dungeon level (on_level equivalent)
    pub fn on_level(&self, other: &DLevel) -> bool {
        self.dlevel == *other
    }

    /// Check if position is on stairs (On_stairs equivalent)
    pub fn on_stairs(&self, x: i8, y: i8) -> bool {
        self.stairway_at(x, y).is_some()
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

    // ========================================================================
    // Position validation functions (from do_move.c / hack.c)
    // ========================================================================

    /// Check if a position is valid for placement (goodpos equivalent)
    ///
    /// Returns true if position is valid and can contain a creature.
    /// # Arguments
    /// * `x`, `y` - Position to check
    /// * `check_monster` - If true, also check that no monster is there
    pub fn goodpos(&self, x: i8, y: i8, check_monster: bool) -> bool {
        // Must be valid position
        if !self.is_valid_pos(x, y) {
            return false;
        }

        // Must be walkable terrain
        if !self.is_walkable(x, y) {
            return false;
        }

        // Optionally check for existing monster
        if check_monster && self.monster_grid[x as usize][y as usize].is_some() {
            return false;
        }

        true
    }

    /// Check if position can be passed through (passable)
    pub fn passable(&self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return false;
        }
        let cell = &self.cells[x as usize][y as usize];
        cell.is_walkable() || cell.is_door()
    }

    /// Check if position is accessible from adjacent position
    pub fn accessible(&self, x: i8, y: i8) -> bool {
        self.is_valid_pos(x, y) && self.is_walkable(x, y)
    }

    /// Check if position is occupied by a monster or player
    pub fn occupied(&self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return false;
        }
        self.monster_grid[x as usize][y as usize].is_some()
    }

    /// Find a good position near a given location
    ///
    /// Searches in expanding rings around the given position.
    /// Returns None if no valid position found within range.
    pub fn find_goodpos(&self, x: i8, y: i8, range: i32) -> Option<(i8, i8)> {
        // Check the position itself first
        if self.goodpos(x, y, true) {
            return Some((x, y));
        }

        // Search in expanding rings
        for dist in 1..=range {
            for dx in -dist..=dist {
                for dy in -dist..=dist {
                    // Only check positions on the ring perimeter
                    if dx.abs() != dist && dy.abs() != dist {
                        continue;
                    }
                    let nx = x + dx as i8;
                    let ny = y + dy as i8;
                    if self.goodpos(nx, ny, true) {
                        return Some((nx, ny));
                    }
                }
            }
        }

        None
    }

    /// Check if position has a closed door
    pub fn closed_door(&self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return false;
        }
        self.cells[x as usize][y as usize].is_closed_door()
    }

    /// Check if a boulder blocks the position
    pub fn boulder_at(&self, x: i8, y: i8) -> bool {
        // Check floor objects for a boulder
        self.objects_at(x, y).iter().any(|obj| obj.is_boulder())
    }

    /// Check if position blocks movement due to terrain or objects
    pub fn blocked(&self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return true;
        }
        let cell = &self.cells[x as usize][y as usize];
        !cell.is_walkable() || self.boulder_at(x, y)
    }

    /// Count monsters adjacent to a position
    pub fn count_adjacent_monsters(&self, x: i8, y: i8) -> i32 {
        let mut count = 0;
        for dx in -1..=1i8 {
            for dy in -1..=1i8 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if self.is_valid_pos(nx, ny)
                    && self.monster_grid[nx as usize][ny as usize].is_some()
                {
                    count += 1;
                }
            }
        }
        count
    }

    /// Get engraving at position
    pub fn engr_at(&self, x: i8, y: i8) -> Option<&Engraving> {
        self.engravings.iter().find(|e| e.x == x && e.y == y)
    }

    /// Get mutable engraving at position
    pub fn engr_at_mut(&mut self, x: i8, y: i8) -> Option<&mut Engraving> {
        self.engravings.iter_mut().find(|e| e.x == x && e.y == y)
    }

    /// Remove engraving at position
    pub fn del_engr_at(&mut self, x: i8, y: i8) {
        self.engravings.retain(|e| e.x != x || e.y != y);
    }

    /// Check if position is inside a room
    pub fn in_room(&self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return false;
        }
        self.cells[x as usize][y as usize].is_room()
    }

    /// Check if position is a corridor
    pub fn in_corridor(&self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return false;
        }
        self.cells[x as usize][y as usize].is_corridor()
    }

    /// Check for clear path between two points (no obstacles)
    pub fn clear_path(&self, x1: i8, y1: i8, x2: i8, y2: i8) -> bool {
        self.has_line_of_sight(x1, y1, x2, y2)
    }

    /// Check if position is bad for rock-movers (bad_rock equivalent)
    ///
    /// Returns true if the position is rock/wall that a rock-mover
    /// (xorn, earth elemental) would have trouble passing through,
    /// such as iron bars or special walls.
    pub fn bad_rock(&self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return true;
        }
        let cell = &self.cells[x as usize][y as usize];
        // Iron bars are bad even for rock-movers
        cell.typ == CellType::IronBars
    }

    /// Check if digging at position is valid (dig_check equivalent)
    ///
    /// Returns true if digging can proceed at the given location.
    /// Some positions (like special walls, level boundaries) can't be dug.
    pub fn dig_check(&self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return false;
        }

        // Can't dig at boundaries
        if x <= 0 || x >= COLNO as i8 - 1 || y <= 0 || y >= ROWNO as i8 - 1 {
            return false;
        }

        let cell = &self.cells[x as usize][y as usize];

        // Check if the terrain can be dug
        if cell.typ.is_wall() || cell.typ == CellType::Stone {
            // Can dig through walls/stone unless flagged as non-diggable
            return cell.can_dig;
        }

        match cell.typ {
            CellType::SecretDoor | CellType::SecretCorridor => {
                // Can dig through secret passages unless flagged as non-diggable
                cell.can_dig
            }
            CellType::Door => {
                // Can dig through doors
                true
            }
            CellType::DrawbridgeUp | CellType::DrawbridgeDown => {
                // Can't dig through drawbridges
                false
            }
            _ => false,
        }
    }

    /// Check if a position can be tunneled through by a monster
    pub fn can_tunnel(&self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return false;
        }
        let cell = &self.cells[x as usize][y as usize];
        cell.typ.is_wall() && cell.can_dig
    }

    /// Find random empty position (for monster placement, etc.)
    pub fn rnd_goodpos(&self, rng: &mut crate::GameRng) -> Option<(i8, i8)> {
        // Try random positions
        for _ in 0..1000 {
            let x = rng.rn2(crate::COLNO as u32) as i8;
            let y = rng.rn2(crate::ROWNO as u32) as i8;
            if self.goodpos(x, y, true) {
                return Some((x, y));
            }
        }
        None
    }

    // ========================================================================
    // Vision/Perception functions (detect.c, see_monsters, see_objects, etc.)
    // ========================================================================

    /// Reveal all monsters on the level (see_monsters equivalent)
    /// Called by detect monster scroll, wand, spell, etc.
    /// Returns the number of monsters revealed.
    pub fn see_monsters(&mut self) -> i32 {
        let mut count = 0;
        for monster in &self.monsters {
            if !monster.is_dead() {
                let x = monster.x as usize;
                let y = monster.y as usize;
                // Mark the monster's position as explored
                self.explored[x][y] = true;
                count += 1;
            }
        }
        count
    }

    /// Reveal all objects on the level (see_objects equivalent)
    /// Called by detect objects scroll, wand, spell, etc.
    /// Returns the number of objects revealed.
    pub fn see_objects(&mut self) -> i32 {
        let mut count = 0;
        for obj in &self.objects {
            let x = obj.x as usize;
            let y = obj.y as usize;
            if x < COLNO && y < ROWNO {
                self.explored[x][y] = true;
                count += 1;
            }
        }
        count
    }

    /// Reveal all traps on the level (see_traps equivalent)
    /// Called by detect traps spell, scroll, etc.
    /// Returns the number of traps revealed.
    pub fn see_traps(&mut self) -> i32 {
        let mut count = 0;
        for trap in &mut self.traps {
            trap.seen = true;
            let x = trap.x as usize;
            let y = trap.y as usize;
            if x < COLNO && y < ROWNO {
                self.explored[x][y] = true;
                count += 1;
            }
        }
        count
    }

    /// Detect gold on the level (gold_detect equivalent)
    /// Returns the number of gold piles detected.
    pub fn gold_detect(&mut self) -> i32 {
        let mut count = 0;
        for obj in &self.objects {
            if obj.class == crate::object::ObjectClass::Coin {
                let x = obj.x as usize;
                let y = obj.y as usize;
                if x < COLNO && y < ROWNO {
                    self.explored[x][y] = true;
                    count += 1;
                }
            }
        }
        // Also check monster inventories for gold
        for monster in &self.monsters {
            if monster.gold_amount() > 0 {
                let x = monster.x as usize;
                let y = monster.y as usize;
                if x < COLNO && y < ROWNO {
                    self.explored[x][y] = true;
                    count += 1;
                }
            }
        }
        count
    }

    /// Detect food on the level (food_detect equivalent)
    /// Returns the number of food items detected.
    pub fn food_detect(&mut self) -> i32 {
        let mut count = 0;
        for obj in &self.objects {
            if obj.class == crate::object::ObjectClass::Food {
                let x = obj.x as usize;
                let y = obj.y as usize;
                if x < COLNO && y < ROWNO {
                    self.explored[x][y] = true;
                    count += 1;
                }
            }
        }
        count
    }

    /// Light up the entire level (mapping/enlightenment)
    pub fn map_entire_level(&mut self) {
        for x in 0..COLNO {
            for y in 0..ROWNO {
                self.explored[x][y] = true;
            }
        }
    }

    /// Unmap/forget part of the level (amnesia effect)
    pub fn forget_map(&mut self, rng: &mut crate::GameRng) {
        for x in 0..COLNO {
            for y in 0..ROWNO {
                // Randomly forget explored cells
                if rng.percent(50) {
                    self.explored[x][y] = false;
                }
            }
        }
    }

    /// Forget all traps (amnesia effect)
    pub fn forget_traps(&mut self) {
        for trap in &mut self.traps {
            trap.seen = false;
        }
    }

    /// Check if a monster is in the player's line of sight
    pub fn monster_in_sight(&self, player_x: i8, player_y: i8, monster_id: MonsterId) -> bool {
        if let Some(monster) = self.monster(monster_id) {
            self.is_visible(monster.x, monster.y)
                && self.has_line_of_sight(player_x, player_y, monster.x, monster.y)
        } else {
            false
        }
    }

    /// Get all visible monsters from player's position
    pub fn visible_monsters(&self, player_x: i8, player_y: i8) -> Vec<&Monster> {
        self.monsters
            .iter()
            .filter(|m| {
                !m.is_dead()
                    && self.is_visible(m.x, m.y)
                    && self.has_line_of_sight(player_x, player_y, m.x, m.y)
            })
            .collect()
    }

    /// Get all visible objects from player's position
    pub fn visible_objects(&self, player_x: i8, player_y: i8) -> Vec<&Object> {
        self.objects
            .iter()
            .filter(|o| {
                self.is_visible(o.x, o.y) && self.has_line_of_sight(player_x, player_y, o.x, o.y)
            })
            .collect()
    }

    /// Check if any hostile monsters are in sight (monster_nearby equivalent)
    pub fn hostile_monster_nearby(&self, player_x: i8, player_y: i8, range: i32) -> bool {
        self.monsters.iter().any(|m| {
            !m.is_dead()
                && !m.is_peaceful()
                && m.distance_sq(player_x, player_y) <= range * range
                && self.has_line_of_sight(player_x, player_y, m.x, m.y)
        })
    }

    /// Count visible hostile monsters (for tension calculation)
    pub fn count_visible_hostiles(&self, player_x: i8, player_y: i8) -> i32 {
        self.monsters
            .iter()
            .filter(|m| {
                !m.is_dead()
                    && !m.is_peaceful()
                    && self.is_visible(m.x, m.y)
                    && self.has_line_of_sight(player_x, player_y, m.x, m.y)
            })
            .count() as i32
    }

    /// Sense traps near a position (trap_detect equivalent)
    /// Returns traps within the given range.
    pub fn sense_traps_near(&self, x: i8, y: i8, range: i32) -> Vec<&Trap> {
        self.traps
            .iter()
            .filter(|t| {
                let dx = (t.x - x) as i32;
                let dy = (t.y - y) as i32;
                dx * dx + dy * dy <= range * range
            })
            .collect()
    }

    /// Mark a trap as seen
    pub fn mark_trap_seen(&mut self, x: i8, y: i8) {
        if let Some(trap) = self.traps.iter_mut().find(|t| t.x == x && t.y == y) {
            trap.seen = true;
        }
    }

    // ========================================================================
    // Light source functions (light.c, vision.c)
    // ========================================================================

    /// Light up an area from a position (litroom equivalent concept)
    /// Makes all floor cells connected to (x,y) lit/visible.
    /// Uses flood fill to find connected room cells.
    /// Returns true if any cells were lit.
    pub fn litroom(&mut self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return false;
        }

        // If not in a room, just light this cell
        if !self.cells[x as usize][y as usize].is_room() {
            self.cells[x as usize][y as usize].lit = true;
            self.explored[x as usize][y as usize] = true;
            return true;
        }

        // Flood fill to light all connected room cells
        let mut lit_count = 0;
        let mut stack = vec![(x, y)];
        let mut visited = vec![vec![false; ROWNO]; COLNO];

        while let Some((cx, cy)) = stack.pop() {
            if !self.is_valid_pos(cx, cy) {
                continue;
            }
            let ux = cx as usize;
            let uy = cy as usize;

            if visited[ux][uy] {
                continue;
            }
            visited[ux][uy] = true;

            let cell = &self.cells[ux][uy];
            if !cell.is_room() && !cell.is_door() {
                continue;
            }

            self.cells[ux][uy].lit = true;
            self.explored[ux][uy] = true;
            self.visible[ux][uy] = true;
            lit_count += 1;

            // Add adjacent cells
            for dx in -1..=1i8 {
                for dy in -1..=1i8 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    stack.push((cx + dx, cy + dy));
                }
            }
        }

        lit_count > 0
    }

    /// Darken an area from a position (unlit equivalent concept)
    /// Makes all floor cells connected to (x,y) dark.
    pub fn unlitroom(&mut self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return false;
        }

        // Flood fill to darken all connected room cells
        let mut dark_count = 0;
        let mut stack = vec![(x, y)];
        let mut visited = vec![vec![false; ROWNO]; COLNO];

        while let Some((cx, cy)) = stack.pop() {
            if !self.is_valid_pos(cx, cy) {
                continue;
            }
            let ux = cx as usize;
            let uy = cy as usize;

            if visited[ux][uy] {
                continue;
            }
            visited[ux][uy] = true;

            let cell = &self.cells[ux][uy];
            if !cell.is_room() && !cell.is_door() {
                continue;
            }

            self.cells[ux][uy].lit = false;
            self.visible[ux][uy] = false;
            dark_count += 1;

            for dx in -1..=1i8 {
                for dy in -1..=1i8 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    stack.push((cx + dx, cy + dy));
                }
            }
        }

        dark_count > 0
    }

    /// Check if a position is lit
    pub fn is_lit(&self, x: i8, y: i8) -> bool {
        if !self.is_valid_pos(x, y) {
            return false;
        }
        self.cells[x as usize][y as usize].lit
    }

    /// Set a position's lit status
    pub fn set_lit(&mut self, x: i8, y: i8, lit: bool) {
        if self.is_valid_pos(x, y) {
            self.cells[x as usize][y as usize].lit = lit;
        }
    }

    /// Add a light source at a position (new_light_source equivalent concept)
    /// Lights up cells within radius.
    pub fn add_light_source(&mut self, x: i8, y: i8, radius: i32) {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let nx = x + dx as i8;
                let ny = y + dy as i8;

                if !self.is_valid_pos(nx, ny) {
                    continue;
                }

                // Check within radius
                if dx * dx + dy * dy <= radius * radius {
                    // Check line of sight from light source
                    if self.has_line_of_sight(x, y, nx, ny) {
                        self.cells[nx as usize][ny as usize].lit = true;
                    }
                }
            }
        }
    }

    /// Remove a light source at a position
    /// Note: This is simplified - real implementation tracks multiple light sources
    pub fn remove_light_source(&mut self, x: i8, y: i8, radius: i32) {
        // In the real game, this would recalculate lighting from all sources
        // For now, just mark cells as unlit
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let nx = x + dx as i8;
                let ny = y + dy as i8;

                if !self.is_valid_pos(nx, ny) {
                    continue;
                }

                if dx * dx + dy * dy <= radius * radius {
                    self.cells[nx as usize][ny as usize].lit = false;
                }
            }
        }
    }

    /// Calculate light radius for an object (obj_light_radius equivalent concept)
    pub fn light_radius(obj: &Object) -> i32 {
        if !obj.lit {
            return 0;
        }

        // Different light sources have different radii
        // This would normally be based on object type
        match obj.class {
            crate::object::ObjectClass::Tool => {
                // Lamps, candles, etc.
                if obj.enchantment > 0 {
                    3 // Magic lamp/lantern with charges
                } else {
                    2 // Regular lamp/candle
                }
            }
            _ => 1, // Other lit objects (like lit potions)
        }
    }

    /// Check if darkness spell/effect should work here
    pub fn can_darken(&self, x: i8, y: i8) -> bool {
        // Can't darken if there's a permanent light source
        // Would check for light sources in the area
        self.is_valid_pos(x, y)
    }

    /// Apply darkness effect to an area
    pub fn apply_darkness(&mut self, x: i8, y: i8, radius: i32) {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let nx = x + dx as i8;
                let ny = y + dy as i8;

                if !self.is_valid_pos(nx, ny) {
                    continue;
                }

                if dx * dx + dy * dy <= radius * radius {
                    self.cells[nx as usize][ny as usize].lit = false;
                    self.visible[nx as usize][ny as usize] = false;
                }
            }
        }
    }
}

/// Find an empty position using expanding square search (enexto equivalent)
///
/// Searches for a valid, empty position starting from the given coordinates
/// and expanding outward in a diamond/square pattern up to 10 squares away.
///
/// Returns the first valid position found, or None if no position is available.
///
/// # Arguments
/// * `x` - Starting X coordinate
/// * `y` - Starting Y coordinate
/// * `level` - The level to search on
pub fn enexto(x: i8, y: i8, level: &Level) -> Option<(i8, i8)> {
    // Try expanding square search up to distance 10
    for distance in 0i32..=10 {
        // For each distance, check all positions at that distance
        // Using Manhattan distance / diamond pattern
        for dx in -distance..=distance {
            for dy in -distance..=distance {
                // Skip if not on the current distance boundary
                if (dx.abs() != distance && dy.abs() != distance) && distance > 0 {
                    continue;
                }

                let nx = x + dx as i8;
                let ny = y + dy as i8;

                // Check if position is valid and walkable
                if level.is_valid_pos(nx, ny) && level.is_walkable(nx, ny) {
                    // Check that no monster is at this position
                    if level.monster_at(nx, ny).is_none() {
                        return Some((nx, ny));
                    }
                }
            }
        }
    }

    // No position found within 10 squares
    None
}

/// Randomly relocate a monster to a valid position on the same level (NetHack rloc)
///
/// Used by teleportation and escape mechanics. Finds a random valid position
/// using rnd_goodpos() and moves the monster there.
///
/// # Arguments
/// * `monster_id` - The monster to relocate
/// * `level` - The level to operate on
/// * `rng` - Random number generator
///
/// # Returns
/// true if successful, false if no valid position found
pub fn rloc_monster(monster_id: MonsterId, level: &mut Level, rng: &mut crate::GameRng) -> bool {
    // Find a random valid position on the level
    if let Some((x, y)) = level.rnd_goodpos(rng) {
        // Move the monster to the new position
        level.move_monster(monster_id, x, y)
    } else {
        false
    }
}

/// Move a monster to a different dungeon level (NetHack migrate_to_level)
///
/// Removes the monster from the current level and returns it for placement on target level.
/// This is a simplified implementation that provides the basic infrastructure for
/// inter-level monster movement. Full implementation would require coordinating with
/// the dungeon level system to place the monster on the target level.
///
/// # Arguments
/// * `monster_id` - The monster to migrate
/// * `level` - The level to remove from
///
/// # Returns
/// The removed monster if found, None otherwise
///
/// # Note
/// This is currently a stub that removes the monster. Full inter-level coordination
/// requires dungeon-wide level management (TODO for future phases)
pub fn migrate_monster_to_level(monster_id: MonsterId, level: &mut Level) -> Option<Monster> {
    // Remove the monster from current level
    // The caller is responsible for placing it on the target level
    level.remove_monster(monster_id)
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

    #[test]
    fn test_enexto_finds_adjacent_empty() {
        let mut level = Level::new(DLevel::default());
        // Create a small room
        for x in 5..15 {
            for y in 5..15 {
                level.cells[x][y] = Cell::floor();
            }
        }

        // Find empty position starting from (10, 10)
        let result = enexto(10, 10, &level);
        assert!(result.is_some());
        let (x, y) = result.unwrap();

        // Should be adjacent or close to (10, 10)
        let distance = ((x - 10).abs() as i32).max((y - 10).abs() as i32);
        assert!(distance <= 10);

        // Found position should be walkable
        assert!(level.is_walkable(x, y));
    }

    #[test]
    fn test_enexto_avoids_monsters() {
        let mut level = Level::new(DLevel::default());
        // Create a small room
        for x in 5..15 {
            for y in 5..15 {
                level.cells[x][y] = Cell::floor();
            }
        }

        // Add a monster at one adjacent position
        let monster = crate::monster::Monster::new(crate::monster::MonsterId(1), 5, 10, 11);
        level.add_monster(monster);

        // Find position (should skip (10, 11) due to monster)
        let result = enexto(10, 10, &level);
        assert!(result.is_some());
        let (x, y) = result.unwrap();

        // Should not be at the monster's position
        assert!((x, y) != (10, 11));
    }

    #[test]
    fn test_enexto_returns_none_if_surrounded() {
        let level = Level::new(DLevel::default());
        // All cells are stone (default) - none are walkable
        let result = enexto(10, 10, &level);
        assert!(result.is_none());
    }

    #[test]
    fn test_pline_and_take_pending_messages() {
        let mut level = Level::new(DLevel::default());

        // Initially no messages
        assert!(level.pending_messages.is_empty());

        // Add messages via pline
        level.pline("The goblin is no longer confused.");
        level.pline("The orc attacks!");

        assert_eq!(level.pending_messages.len(), 2);

        // Take messages clears the buffer
        let messages = level.take_pending_messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], "The goblin is no longer confused.");
        assert_eq!(messages[1], "The orc attacks!");

        // Buffer is now empty
        assert!(level.pending_messages.is_empty());
    }
}
