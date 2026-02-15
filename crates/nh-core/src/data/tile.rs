use serde::{Serialize, Deserialize};

/// A unique identifier for a graphical tile in frontends like Bevy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileId(pub u32);

/// Categories of dungeon features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DungeonTile {
    Floor,
    VerticalWall,
    HorizontalWall,
    DoorClosed,
    DoorOpen,
    StairsUp,
    StairsDown,
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
                DungeonTile::VerticalWall => '|',
                DungeonTile::HorizontalWall => '-',
                DungeonTile::DoorClosed => '+',
                DungeonTile::DoorOpen => '/',
                DungeonTile::StairsUp => '<',
                DungeonTile::StairsDown => '>',
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
                DungeonTile::VerticalWall => TileId(10),
                DungeonTile::HorizontalWall => TileId(11),
                DungeonTile::DoorClosed => TileId(20),
                DungeonTile::DoorOpen => TileId(21),
                DungeonTile::StairsUp => TileId(30),
                DungeonTile::StairsDown => TileId(31),
            },
            Tile::Monster(_) => TileId(100),
            Tile::Object(_) => TileId(200),
        }
    }
}
