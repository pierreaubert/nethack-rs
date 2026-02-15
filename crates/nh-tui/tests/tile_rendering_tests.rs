use nh_core::data::tile::{Tile, DungeonTile};

#[test]
fn test_tile_to_tui_char() {
    // This test will verify that nh-tui logic (which we will implement)
    // correctly converts a Tile to the expected character.
    // For now, we can just test the Tile's own to_ascii() as a base.
    
    let floor = Tile::Dungeon(DungeonTile::Floor);
    assert_eq!(floor.to_ascii(), '.');
    
    let kobold = Tile::Monster("kobold".to_string());
    assert_eq!(kobold.to_ascii(), 'k');
}
