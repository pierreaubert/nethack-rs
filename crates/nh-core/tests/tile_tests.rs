use nh_core::data::tile::{Tile, TileId, DungeonTile};

#[test]
fn test_tile_representations() {
    // Test a dungeon tile (e.g., a wall)
    let wall_tile = Tile::Dungeon(DungeonTile::VerticalWall);
    assert_eq!(wall_tile.to_ascii(), '|');
    assert_eq!(wall_tile.to_tile_id(), TileId(10)); // Arbitrary ID for now

    // Test a monster tile (e.g., a kobold)
    // We'll need a way to reference monsters, maybe by string or id
    let kobold_tile = Tile::Monster("kobold".to_string());
    assert_eq!(kobold_tile.to_ascii(), 'k');
}

#[test]
fn test_tile_registry() {
    use nh_core::data::tile::get_tile_for_monster;
    use nh_core::data::monsters::find_monster;

    let (_, monster) = find_monster("kobold").expect("Kobold should exist");
    println!("Monster name: {}", monster.name);
    let tile = get_tile_for_monster(&monster);
    assert_eq!(tile.to_ascii(), 'k');
}
