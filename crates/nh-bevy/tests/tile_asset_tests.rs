use nh_core::data::tile::{Tile, DungeonTile, TileId};

#[test]
fn test_tile_to_bevy_id() {
    let floor = Tile::Dungeon(DungeonTile::Floor);
    assert_eq!(floor.to_tile_id(), TileId(1));
    
    let wall = Tile::Dungeon(DungeonTile::VerticalWall);
    assert_eq!(wall.to_tile_id(), TileId(10));
}
