#[cfg(not(feature = "std"))]
use crate::compat::*;

use serde::{Serialize, Deserialize};
use crate::monster::PerMonst;
use crate::dungeon::CellType;
use crate::object::Object;
use super::objects::{get_object, ObjectType};

/// A unique identifier for a graphical tile in frontends like Bevy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileId(pub u32);

/// Categories of dungeon features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DungeonTile {
    Floor,
    Stone,
    VerticalWall,
    HorizontalWall,
    Corner,
    DoorClosed,
    DoorOpen,
    StairsUp,
    StairsDown,
    Pool,
    Lava,
}

impl From<CellType> for DungeonTile {
    fn from(cell_type: CellType) -> Self {
        match cell_type {
            CellType::Room | CellType::Corridor | CellType::Vault => DungeonTile::Floor,
            CellType::VWall => DungeonTile::VerticalWall,
            CellType::HWall => DungeonTile::HorizontalWall,
            CellType::TLCorner
            | CellType::TRCorner
            | CellType::BLCorner
            | CellType::BRCorner
            | CellType::CrossWall
            | CellType::TUWall
            | CellType::TDWall
            | CellType::TLWall
            | CellType::TRWall => DungeonTile::Corner,
            CellType::Door => DungeonTile::DoorClosed, // Need more state for open/closed
            CellType::Stairs => DungeonTile::StairsDown, // Need more state for up/down
            CellType::Pool | CellType::Moat | CellType::Water => DungeonTile::Pool,
            CellType::Lava => DungeonTile::Lava,
            _ => DungeonTile::Stone,
        }
    }
}

/// A unified representation of a game entity for rendering.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tile {
    Dungeon(DungeonTile),
    Monster(String), // For now, use the monster name as an identifier
    Object(String),  // For now, use the object name as an identifier
}

impl Tile {
    /// Returns the classic ASCII representation of the tile.
    pub fn to_ascii(&self) -> char {
        match self {
            Tile::Dungeon(dt) => match dt {
                DungeonTile::Floor => '.',
                DungeonTile::Stone => ' ',
                DungeonTile::VerticalWall => '|',
                DungeonTile::HorizontalWall => '-',
                DungeonTile::Corner => '+',
                DungeonTile::DoorClosed => '+',
                DungeonTile::DoorOpen => '/',
                DungeonTile::StairsUp => '<',
                DungeonTile::StairsDown => '>',
                DungeonTile::Pool => '}',
                DungeonTile::Lava => '}',
            },
            Tile::Monster(name) => {
                // Simplified mapping for the prototype
                if name.to_lowercase().contains("kobold") {
                    'k'
                } else {
                    'M'
                }
            }
            Tile::Object(_) => '?',
        }
    }

    /// Returns the graphical tile identifier for Bevy.
    pub fn to_tile_id(&self) -> TileId {
        match self {
            Tile::Dungeon(dt) => match dt {
                DungeonTile::Floor => TileId(1),
                DungeonTile::Stone => TileId(0),
                DungeonTile::VerticalWall => TileId(10),
                DungeonTile::HorizontalWall => TileId(11),
                DungeonTile::Corner => TileId(12),
                DungeonTile::DoorClosed => TileId(20),
                DungeonTile::DoorOpen => TileId(21),
                DungeonTile::StairsUp => TileId(30),
                DungeonTile::StairsDown => TileId(31),
                DungeonTile::Pool => TileId(40),
                DungeonTile::Lava => TileId(41),
            },
            Tile::Monster(_) => TileId(100),
            Tile::Object(_) => TileId(200),
        }
    }
}

/// Returns the Tile representation for a given monster definition.
pub fn get_tile_for_monster(monster: &PerMonst) -> Tile {
    Tile::Monster(monster.name.to_string())
}

/// Returns the Tile representation for a given object instance.
pub fn get_tile_for_object(obj: &Object) -> Tile {
    // If it has a custom name, we could use it, but usually we want the type name
    // For now, use the name from the object class definition
    let class_def = get_object(unsafe { std::mem::transmute::<i16, ObjectType>(obj.object_type) });
    Tile::Object(class_def.name.to_string())
}
