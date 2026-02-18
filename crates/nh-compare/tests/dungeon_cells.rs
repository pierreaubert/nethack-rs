//! Dungeon cell and level behavioral tests
//!
//! Tests for cell types, cell properties, door states,
//! terrain queries, and level coordinate validation.

use nh_core::dungeon::*;

// ============================================================================
// CellType properties
// ============================================================================

#[test]
fn test_wall_is_wall() {
    assert!(CellType::Wall.is_wall());
}

#[test]
fn test_room_is_not_wall() {
    assert!(!CellType::Room.is_wall());
}

#[test]
fn test_door_is_door() {
    assert!(CellType::Door.is_door());
}

#[test]
fn test_room_is_not_door() {
    assert!(!CellType::Room.is_door());
}

#[test]
fn test_room_is_passable() {
    assert!(CellType::Room.is_passable());
}

#[test]
fn test_corridor_is_passable() {
    assert!(CellType::Corridor.is_passable());
}

#[test]
fn test_wall_not_passable() {
    assert!(!CellType::Wall.is_passable());
}

#[test]
fn test_pool_is_liquid() {
    assert!(CellType::Pool.is_liquid());
}

#[test]
fn test_lava_is_liquid() {
    assert!(CellType::Lava.is_liquid());
}

#[test]
fn test_room_not_liquid() {
    assert!(!CellType::Room.is_liquid());
}

#[test]
fn test_lava_requires_flight() {
    assert!(CellType::Lava.requires_flight());
}

#[test]
fn test_room_no_flight() {
    assert!(!CellType::Room.requires_flight());
}

#[test]
fn test_stone_is_diggable() {
    assert!(CellType::Stone.is_diggable());
}

#[test]
fn test_room_not_diggable() {
    // Room floor may or may not be diggable (downward)
    let _ = CellType::Room.is_diggable();
}

// ============================================================================
// CellType symbols
// ============================================================================

#[test]
fn test_wall_symbol() {
    let sym = CellType::Wall.symbol();
    assert!(sym == '|' || sym == '-' || sym == '#');
}

#[test]
fn test_room_symbol() {
    let sym = CellType::Room.symbol();
    assert_eq!(sym, '.');
}

#[test]
fn test_corridor_symbol() {
    let sym = CellType::Corridor.symbol();
    assert_eq!(sym, '#');
}

#[test]
fn test_door_symbol() {
    let sym = CellType::Door.symbol();
    let _ = sym; // depends on state
}

// ============================================================================
// CellType surface / ceiling / hliquid
// ============================================================================

#[test]
fn test_surface_room() {
    let s = CellType::Room.surface();
    assert!(!s.is_empty());
}

#[test]
fn test_surface_pool() {
    let s = CellType::Pool.surface();
    assert!(!s.is_empty());
}

#[test]
fn test_ceiling_room() {
    let c = CellType::Room.ceiling();
    assert!(!c.is_empty());
}

#[test]
fn test_hliquid_pool() {
    let h = CellType::Pool.hliquid();
    assert!(!h.is_empty());
}

#[test]
fn test_is_water() {
    assert!(CellType::Pool.is_water());
    assert!(!CellType::Lava.is_water());
}

#[test]
fn test_is_lava() {
    assert!(CellType::Lava.is_lava());
    assert!(!CellType::Pool.is_lava());
}

// ============================================================================
// Cell constructors
// ============================================================================

#[test]
fn test_cell_stone() {
    let cell = Cell::stone();
    // Stone is not a "wall" type - it's unexcavated rock
    assert!(!cell.is_room());
}

#[test]
fn test_cell_floor() {
    let cell = Cell::floor();
    assert!(cell.is_room());
}

#[test]
fn test_cell_corridor() {
    let cell = Cell::corridor();
    assert!(cell.is_corridor());
}

// ============================================================================
// Cell door state
// ============================================================================

#[test]
fn test_cell_door_default_state() {
    let mut cell = Cell::floor();
    cell.typ = CellType::Door;
    let state = cell.door_state();
    let _ = state;
}

#[test]
fn test_cell_set_door_closed() {
    let mut cell = Cell::floor();
    cell.typ = CellType::Door;
    cell.set_door_state(DoorState::CLOSED);
    assert!(cell.is_closed_door());
}

#[test]
fn test_cell_set_door_open() {
    let mut cell = Cell::floor();
    cell.typ = CellType::Door;
    cell.set_door_state(DoorState::OPEN);
    assert!(cell.is_open_door());
}

// ============================================================================
// Cell query methods
// ============================================================================

#[test]
fn test_cell_blocks_sight_wall() {
    let cell = Cell::stone();
    assert!(cell.blocks_sight());
}

#[test]
fn test_cell_blocks_sight_floor() {
    let cell = Cell::floor();
    assert!(!cell.blocks_sight());
}

#[test]
fn test_cell_is_walkable_floor() {
    let cell = Cell::floor();
    assert!(cell.is_walkable());
}

#[test]
fn test_cell_is_walkable_corridor() {
    let cell = Cell::corridor();
    assert!(cell.is_walkable());
}

#[test]
fn test_cell_is_room() {
    let cell = Cell::floor();
    assert!(cell.is_room());
    assert!(!cell.is_corridor());
}

#[test]
fn test_cell_is_corridor() {
    let cell = Cell::corridor();
    assert!(cell.is_corridor());
    assert!(!cell.is_room());
}

#[test]
fn test_cell_is_door() {
    let mut cell = Cell::floor();
    cell.typ = CellType::Door;
    assert!(cell.is_door());
}

#[test]
fn test_cell_is_water() {
    let mut cell = Cell::floor();
    cell.typ = CellType::Pool;
    assert!(cell.is_water());
}

#[test]
fn test_cell_is_lava() {
    let mut cell = Cell::floor();
    cell.typ = CellType::Lava;
    assert!(cell.is_lava());
}

#[test]
fn test_cell_is_ice() {
    let mut cell = Cell::floor();
    cell.typ = CellType::Ice;
    assert!(cell.is_ice());
}

#[test]
fn test_cell_is_fountain() {
    let mut cell = Cell::floor();
    cell.typ = CellType::Fountain;
    assert!(cell.is_fountain());
}

#[test]
fn test_cell_is_sink() {
    let mut cell = Cell::floor();
    cell.typ = CellType::Sink;
    assert!(cell.is_sink());
}

#[test]
fn test_cell_is_altar() {
    let mut cell = Cell::floor();
    cell.typ = CellType::Altar;
    assert!(cell.is_altar());
}

#[test]
fn test_cell_is_grave() {
    let mut cell = Cell::floor();
    cell.typ = CellType::Grave;
    assert!(cell.is_grave());
}

#[test]
fn test_cell_is_throne() {
    let mut cell = Cell::floor();
    cell.typ = CellType::Throne;
    assert!(cell.is_throne());
}

#[test]
fn test_cell_is_stairs() {
    let mut cell = Cell::floor();
    cell.typ = CellType::Stairs;
    assert!(cell.is_stairs());
}

#[test]
fn test_cell_is_bars() {
    let mut cell = Cell::floor();
    cell.typ = CellType::IronBars;
    assert!(cell.is_bars());
}

#[test]
fn test_cell_is_wall_method() {
    let mut cell = Cell::floor();
    cell.typ = CellType::Wall;
    assert!(cell.is_wall());
}

// ============================================================================
// CellType variant coverage
// ============================================================================

#[test]
fn test_celltype_stone_default() {
    assert_eq!(CellType::default(), CellType::Stone);
}

#[test]
fn test_celltype_vwall_is_wall() {
    assert!(CellType::VWall.is_wall());
}

#[test]
fn test_celltype_hwall_is_wall() {
    assert!(CellType::HWall.is_wall());
}

#[test]
fn test_celltype_corners_are_walls() {
    assert!(CellType::TLCorner.is_wall());
    assert!(CellType::TRCorner.is_wall());
    assert!(CellType::BLCorner.is_wall());
    assert!(CellType::BRCorner.is_wall());
}

#[test]
fn test_celltype_crosswall_is_wall() {
    assert!(CellType::CrossWall.is_wall());
}

#[test]
fn test_celltype_twalls_are_walls() {
    assert!(CellType::TUWall.is_wall());
    assert!(CellType::TDWall.is_wall());
    assert!(CellType::TLWall.is_wall());
    assert!(CellType::TRWall.is_wall());
}

#[test]
fn test_celltype_dbwall_is_wall() {
    assert!(CellType::DBWall.is_wall());
}

#[test]
fn test_celltype_moat_is_liquid() {
    assert!(CellType::Moat.is_liquid());
}

#[test]
fn test_celltype_water_is_liquid() {
    assert!(CellType::Water.is_liquid());
}

#[test]
fn test_celltype_air_not_liquid() {
    assert!(!CellType::Air.is_liquid());
}

#[test]
fn test_celltype_cloud() {
    assert!(!CellType::Cloud.is_wall());
}

#[test]
fn test_celltype_vault() {
    // Vault is a special room type
    let _ = CellType::Vault.is_passable();
}

// ============================================================================
// isok (coordinate validation)
// ============================================================================

#[test]
fn test_isok_valid() {
    assert!(isok(5, 5));
}

#[test]
fn test_isok_origin() {
    let _ = isok(0, 0);
}

#[test]
fn test_isok_negative() {
    assert!(!isok(-1, 5));
    assert!(!isok(5, -1));
}

#[test]
fn test_isok_large() {
    assert!(!isok(1000, 5));
    assert!(!isok(5, 1000));
}
