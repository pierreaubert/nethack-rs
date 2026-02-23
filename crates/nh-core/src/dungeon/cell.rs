//! Map cell types (rm.h)

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Cell/terrain type
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum CellType {
    #[default]
    Stone = 0,
    VWall = 1,
    HWall = 2,
    TLCorner = 3,
    TRCorner = 4,
    BLCorner = 5,
    BRCorner = 6,
    CrossWall = 7,
    TUWall = 8,  // T-wall up
    TDWall = 9,  // T-wall down
    TLWall = 10, // T-wall left
    TRWall = 11, // T-wall right
    DBWall = 12, // Drawbridge wall
    Tree = 13,
    SecretDoor = 14,
    SecretCorridor = 15,
    Pool = 16,
    Moat = 17,
    Water = 18,
    DrawbridgeUp = 19,
    Lava = 20,
    IronBars = 21,
    Door = 22,
    Corridor = 23,
    Room = 24,
    Stairs = 25,
    Ladder = 26,
    Fountain = 27,
    Throne = 28,
    Sink = 29,
    Grave = 30,
    Altar = 31,
    Ice = 32,
    DrawbridgeDown = 33,
    Air = 34,
    Cloud = 35,
    Wall = 36,  // Generic wall
    Vault = 37, // Vault room floor
}

impl CellType {
    /// Check if this is a wall type
    pub const fn is_wall(&self) -> bool {
        ((*self as u8) >= 1 && (*self as u8) <= 12) || *self as u8 == 36
    }

    /// C's IS_ROCK(typ): typ < POOL — absolutely nonaccessible terrain
    /// Includes Stone, all wall types, DBWall, Tree, SecretDoor, SecretCorridor (0-15)
    pub const fn is_rock(&self) -> bool {
        (*self as u8) < 16
    }

    /// Check if this is a door
    pub const fn is_door(&self) -> bool {
        matches!(self, CellType::Door | CellType::SecretDoor)
    }

    /// Check if this is passable (can walk through)
    pub const fn is_passable(&self) -> bool {
        matches!(
            self,
            CellType::Room
                | CellType::Corridor
                | CellType::Door
                | CellType::Stairs
                | CellType::Ladder
                | CellType::Fountain
                | CellType::Throne
                | CellType::Sink
                | CellType::Grave
                | CellType::Altar
                | CellType::Ice
                | CellType::DrawbridgeDown
                | CellType::Air
                | CellType::Cloud
        )
    }

    /// C's IS_POOL: POOL through DRAWBRIDGE_UP
    pub const fn is_pool(&self) -> bool {
        matches!(
            self,
            CellType::Pool
                | CellType::Moat
                | CellType::Water
                | CellType::Lava
                | CellType::DrawbridgeUp
        )
    }

    /// C's IS_FURNITURE: STAIRS through ALTAR
    pub const fn is_furniture(&self) -> bool {
        matches!(
            self,
            CellType::Stairs
                | CellType::Ladder
                | CellType::Fountain
                | CellType::Throne
                | CellType::Sink
                | CellType::Grave
                | CellType::Altar
        )
    }

    /// Check if this is a liquid/water type
    pub const fn is_liquid(&self) -> bool {
        matches!(
            self,
            CellType::Pool | CellType::Moat | CellType::Water | CellType::Lava
        )
    }

    /// Check if flying is required to cross
    pub const fn requires_flight(&self) -> bool {
        matches!(
            self,
            CellType::Pool
                | CellType::Moat
                | CellType::Water
                | CellType::Lava
                | CellType::Air
                | CellType::DrawbridgeUp
        )
    }

    /// Check if this is diggable
    pub const fn is_diggable(&self) -> bool {
        matches!(
            self,
            CellType::Stone
                | CellType::VWall
                | CellType::HWall
                | CellType::TLCorner
                | CellType::TRCorner
                | CellType::BLCorner
                | CellType::BRCorner
                | CellType::CrossWall
                | CellType::TUWall
                | CellType::TDWall
                | CellType::TLWall
                | CellType::TRWall
                | CellType::Corridor
                | CellType::Room
        )
    }

    /// Get the display character for this cell type
    pub const fn symbol(&self) -> char {
        match self {
            CellType::Stone => ' ',
            CellType::VWall => '|',
            CellType::HWall => '-',
            CellType::TLCorner => '-',
            CellType::TRCorner => '-',
            CellType::BLCorner => '-',
            CellType::BRCorner => '-',
            CellType::CrossWall => '-',
            CellType::TUWall => '-',
            CellType::TDWall => '-',
            CellType::TLWall => '|',
            CellType::TRWall => '|',
            CellType::DBWall => '|',
            CellType::Tree => '#',
            CellType::SecretDoor => '#', // looks like wall
            CellType::SecretCorridor => '#',
            CellType::Pool => '}',
            CellType::Moat => '}',
            CellType::Water => '}',
            CellType::DrawbridgeUp => '#',
            CellType::Lava => '}',
            CellType::IronBars => '#',
            CellType::Door => '+',
            CellType::Corridor => '#',
            CellType::Room => '.',
            CellType::Stairs => '>',
            CellType::Ladder => '>',
            CellType::Fountain => '{',
            CellType::Throne => '\\',
            CellType::Sink => '#',
            CellType::Grave => '|',
            CellType::Altar => '_',
            CellType::Ice => '.',
            CellType::DrawbridgeDown => '.',
            CellType::Air => ' ',
            CellType::Cloud => '#',
            CellType::Wall => '#',  // Generic wall
            CellType::Vault => '.', // Vault floor
        }
    }

    /// Get the surface name for this terrain (surface equivalent)
    ///
    /// Returns what the player is standing "on"
    pub const fn surface(&self) -> &'static str {
        match self {
            CellType::Pool | CellType::Moat | CellType::Water => "water",
            CellType::Lava => "lava",
            CellType::Ice => "ice",
            CellType::Air | CellType::Cloud => "air",
            CellType::Grave => "grave",
            CellType::Altar => "altar",
            CellType::Throne => "throne",
            CellType::Fountain => "fountain",
            CellType::Sink => "sink",
            CellType::DrawbridgeUp | CellType::DrawbridgeDown => "drawbridge",
            _ => "floor",
        }
    }

    /// Get the ceiling name for this terrain (ceiling equivalent)
    ///
    /// Returns what's above the player
    pub const fn ceiling(&self) -> &'static str {
        match self {
            CellType::Air | CellType::Cloud => "sky",
            _ => "ceiling",
        }
    }

    /// Get the liquid name for this terrain (hliquid equivalent)
    ///
    /// Returns the name of the liquid, or "water" as default
    pub const fn hliquid(&self) -> &'static str {
        match self {
            CellType::Lava => "lava",
            CellType::Pool | CellType::Moat | CellType::Water => "water",
            _ => "water",
        }
    }

    /// Check if this is a body of water
    pub const fn is_water(&self) -> bool {
        matches!(self, CellType::Pool | CellType::Moat | CellType::Water)
    }

    /// Check if this is lava
    pub const fn is_lava(&self) -> bool {
        matches!(self, CellType::Lava)
    }
}

bitflags! {
    /// Door state flags
    #[derive(Debug, Clone, Copy, Default)]
    pub struct DoorState: u8 {
        const NO_DOOR = 0x00;
        const BROKEN = 0x01;
        const OPEN = 0x02;
        const CLOSED = 0x04;
        const LOCKED = 0x08;
        const TRAPPED = 0x10;
        const SECRET = 0x20;
    }
}

// Manual serde impl for DoorState
impl Serialize for DoorState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.bits().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DoorState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u8::deserialize(deserializer)?;
        Ok(DoorState::from_bits_truncate(bits))
    }
}

/// A single map cell
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Cell {
    /// What the player remembers seeing here
    pub glyph: i32,

    /// Actual terrain type
    pub typ: CellType,

    /// Visibility bitmask (directions seen from)
    pub seen_from: u8,

    /// Cell flags (door state, altar alignment, etc.)
    pub flags: u8,

    /// Horizontal orientation (for walls)
    pub horizontal: bool,

    /// Currently lit
    pub lit: bool,

    /// Was lit before
    pub was_lit: bool,

    /// Room number (0 = not in room)
    pub room_number: u8,

    /// Is edge of level
    pub edge: bool,

    /// Can dig here (exception to normal rules)
    pub can_dig: bool,

    /// Has been seen by player
    pub explored: bool,
}

impl Cell {
    /// Create a new stone cell
    pub const fn stone() -> Self {
        Self {
            glyph: 0,
            typ: CellType::Stone,
            seen_from: 0,
            flags: 0,
            horizontal: false,
            lit: false,
            was_lit: false,
            room_number: 0,
            edge: false,
            can_dig: true,
            explored: false,
        }
    }

    /// Create a floor cell
    pub const fn floor() -> Self {
        Self {
            glyph: 0,
            typ: CellType::Room,
            seen_from: 0,
            flags: 0,
            horizontal: false,
            lit: true,
            was_lit: false,
            room_number: 0,
            edge: false,
            can_dig: true,
            explored: false,
        }
    }

    /// Create a corridor cell
    pub const fn corridor() -> Self {
        Self {
            glyph: 0,
            typ: CellType::Corridor,
            seen_from: 0,
            flags: 0,
            horizontal: false,
            lit: false,
            was_lit: false,
            room_number: 0,
            edge: false,
            can_dig: true,
            explored: false,
        }
    }

    /// Get door state from flags
    pub fn door_state(&self) -> DoorState {
        DoorState::from_bits_truncate(self.flags)
    }

    /// Set door state
    pub fn set_door_state(&mut self, state: DoorState) {
        self.flags = state.bits();
    }

    /// Check if this cell blocks line of sight
    pub const fn blocks_sight(&self) -> bool {
        match self.typ {
            CellType::Stone
            | CellType::VWall
            | CellType::HWall
            | CellType::TLCorner
            | CellType::TRCorner
            | CellType::BLCorner
            | CellType::BRCorner
            | CellType::CrossWall
            | CellType::TUWall
            | CellType::TDWall
            | CellType::TLWall
            | CellType::TRWall
            | CellType::DBWall
            | CellType::Tree
            | CellType::SecretDoor
            | CellType::SecretCorridor
            | CellType::Cloud => true,
            CellType::Door => {
                // Closed doors block sight
                let state = self.flags;
                state & DoorState::CLOSED.bits() != 0
            }
            _ => false,
        }
    }

    /// Check if walkable
    pub fn is_walkable(&self) -> bool {
        if !self.typ.is_passable() {
            return false;
        }
        if self.typ == CellType::Door {
            let state = self.door_state();
            // D_NODOOR (0) = doorway (passable), D_ISOPEN (2) = open door, D_BROKEN (1) = broken door
            // Closed (4) and Locked (8) doors block movement
            return state.is_empty()  // D_NODOOR = 0 = doorway (passable)
                || state.contains(DoorState::OPEN)
                || state.contains(DoorState::BROKEN);
        }
        true
    }

    /// Check if this is a door (any type)
    pub const fn is_door(&self) -> bool {
        self.typ.is_door()
    }

    /// Check if this is a closed door
    pub fn is_closed_door(&self) -> bool {
        if self.typ != CellType::Door {
            return false;
        }
        let state = self.door_state();
        state.contains(DoorState::CLOSED) || state.contains(DoorState::LOCKED)
    }

    /// Check if this is an open door
    pub fn is_open_door(&self) -> bool {
        if self.typ != CellType::Door {
            return false;
        }
        let state = self.door_state();
        state.contains(DoorState::OPEN) || state.contains(DoorState::BROKEN)
    }

    /// Check if this is room floor
    pub const fn is_room(&self) -> bool {
        matches!(self.typ, CellType::Room)
    }

    /// Check if this is a corridor
    pub const fn is_corridor(&self) -> bool {
        matches!(self.typ, CellType::Corridor)
    }

    /// Check if this is water/pool
    pub const fn is_water(&self) -> bool {
        matches!(self.typ, CellType::Pool | CellType::Moat | CellType::Water)
    }

    /// Check if this is lava
    pub const fn is_lava(&self) -> bool {
        matches!(self.typ, CellType::Lava)
    }

    /// Check if this is ice
    pub const fn is_ice(&self) -> bool {
        matches!(self.typ, CellType::Ice)
    }

    /// Check if this is a trap door
    pub const fn is_trap_door(&self) -> bool {
        // Trap doors are tracked separately in traps vector
        false
    }

    /// Check if this is a fountain
    pub const fn is_fountain(&self) -> bool {
        matches!(self.typ, CellType::Fountain)
    }

    /// Check if this is a sink
    pub const fn is_sink(&self) -> bool {
        matches!(self.typ, CellType::Sink)
    }

    /// Check if this is an altar
    pub const fn is_altar(&self) -> bool {
        matches!(self.typ, CellType::Altar)
    }

    /// Check if this is a grave
    pub const fn is_grave(&self) -> bool {
        matches!(self.typ, CellType::Grave)
    }

    /// Check if this is a throne
    pub const fn is_throne(&self) -> bool {
        matches!(self.typ, CellType::Throne)
    }

    /// Check if this is stairs (up or down)
    pub const fn is_stairs(&self) -> bool {
        matches!(self.typ, CellType::Stairs | CellType::Ladder)
    }

    /// Check if this is a tree
    pub const fn is_tree(&self) -> bool {
        matches!(self.typ, CellType::Tree)
    }

    /// Check if this is iron bars
    pub const fn is_bars(&self) -> bool {
        matches!(self.typ, CellType::IronBars)
    }

    /// Check if this cell is a wall
    pub const fn is_wall(&self) -> bool {
        self.typ.is_wall()
    }
}

// ============================================================================
// Free functions (C-style API equivalents)
// ============================================================================

/// Get the surface name for terrain (surface equivalent)
pub const fn surface(cell_type: CellType) -> &'static str {
    cell_type.surface()
}

/// Get the ceiling name for terrain (ceiling equivalent)
pub const fn ceiling(cell_type: CellType) -> &'static str {
    cell_type.ceiling()
}

/// Get the liquid name for terrain (hliquid equivalent)
pub const fn hliquid(cell_type: CellType) -> &'static str {
    cell_type.hliquid()
}

/// Get the name of a body of water (waterbody_name equivalent)
///
/// Returns the appropriate name for different water-like terrain.
pub const fn waterbody_name(cell_type: CellType) -> &'static str {
    match cell_type {
        CellType::Pool => "pool of water",
        CellType::Moat => "moat",
        CellType::Water => "water",
        CellType::Lava => "lava",
        _ => "water",
    }
}

/// Check if cell is a pool of water (IS_POOL equivalent)
pub const fn is_pool(cell_type: CellType) -> bool {
    matches!(cell_type, CellType::Pool | CellType::Moat | CellType::Water)
}

/// Check if cell is lava (IS_LAVA equivalent)
pub const fn is_lava(cell_type: CellType) -> bool {
    matches!(cell_type, CellType::Lava)
}

/// Check if cell is ice (IS_ICE equivalent)
pub const fn is_ice(cell_type: CellType) -> bool {
    matches!(cell_type, CellType::Ice)
}

/// Check if this is any type of drawbridge wall (IS_DRAWBRIDGE_WALL equivalent)
pub const fn is_drawbridge_wall(cell_type: CellType) -> bool {
    matches!(
        cell_type,
        CellType::DBWall | CellType::DrawbridgeUp | CellType::DrawbridgeDown
    )
}

/// Set a cell as lit
pub fn set_lit(cell: &mut Cell, lit: bool) {
    cell.was_lit = cell.lit;
    cell.lit = lit;
}

/// Check if coordinates are within valid map bounds (isok from hack.h)
///
/// This is the Rust equivalent of NetHack's isok() macro.
/// Returns true if the coordinates are within the valid map area.
#[inline]
pub fn isok(x: i32, y: i32) -> bool {
    x >= 0 && x < crate::COLNO as i32 && y >= 0 && y < crate::ROWNO as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isok() {
        assert!(isok(0, 0));
        assert!(isok(40, 10));
        assert!(isok(79, 20));
        assert!(!isok(-1, 0));
        assert!(!isok(0, -1));
        assert!(!isok(80, 0));
        assert!(!isok(0, 21));
    }

    #[test]
    fn test_is_pool() {
        assert!(is_pool(CellType::Pool));
        assert!(is_pool(CellType::Moat));
        assert!(is_pool(CellType::Water));
        assert!(!is_pool(CellType::Lava));
        assert!(!is_pool(CellType::Room));
    }

    #[test]
    fn test_is_lava() {
        assert!(is_lava(CellType::Lava));
        assert!(!is_lava(CellType::Pool));
        assert!(!is_lava(CellType::Room));
    }

    #[test]
    fn test_is_ice() {
        assert!(is_ice(CellType::Ice));
        assert!(!is_ice(CellType::Pool));
        assert!(!is_ice(CellType::Room));
    }

    #[test]
    fn test_is_drawbridge_wall() {
        assert!(is_drawbridge_wall(CellType::DBWall));
        assert!(is_drawbridge_wall(CellType::DrawbridgeUp));
        assert!(is_drawbridge_wall(CellType::DrawbridgeDown));
        assert!(!is_drawbridge_wall(CellType::VWall));
    }

    #[test]
    fn test_set_lit() {
        let mut cell = Cell::stone();
        cell.lit = true;
        set_lit(&mut cell, false);
        assert!(!cell.lit);
        assert!(cell.was_lit);
    }

    #[test]
    fn test_surface() {
        assert_eq!(surface(CellType::Pool), "water");
        assert_eq!(surface(CellType::Lava), "lava");
        assert_eq!(surface(CellType::Ice), "ice");
        assert_eq!(surface(CellType::Room), "floor");
    }

    #[test]
    fn test_ceiling() {
        assert_eq!(ceiling(CellType::Air), "sky");
        assert_eq!(ceiling(CellType::Room), "ceiling");
    }
}
