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
    TUWall = 8,    // T-wall up
    TDWall = 9,    // T-wall down
    TLWall = 10,   // T-wall left
    TRWall = 11,   // T-wall right
    DBWall = 12,   // Drawbridge wall
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
}

impl CellType {
    /// Check if this is a wall type
    pub const fn is_wall(&self) -> bool {
        (*self as u8) >= 1 && (*self as u8) <= 12
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
        }
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
            // Can walk through open or broken doors
            return state.contains(DoorState::OPEN) || state.contains(DoorState::BROKEN);
        }
        true
    }
}
