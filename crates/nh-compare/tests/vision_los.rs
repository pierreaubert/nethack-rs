//! Phase 25: Vision & Line-of-Sight
//!
//! Behavioral tests verifying cell lighting defaults, corridor darkness,
//! LightSource creation with range clamping, and player vision properties
//! (infravision, telepathy, see invisible).

use nh_core::dungeon::{Cell, CellType, DLevel, Level, LightSource, Room, RoomType};
use nh_core::object::ObjectId;
use nh_core::player::{Property, You};

// ============================================================================
// Test 1: Lit room has lit=true on cells
// ============================================================================

#[test]
fn test_lit_room_cells() {
    // Cell::floor() creates a room-type cell with lit=true by default,
    // matching NetHack's behavior where ordinary rooms are lit on shallow levels.
    let mut level = Level::new(DLevel::default());

    // Carve out a small lit room (5x5 interior)
    for x in 10..15 {
        for y in 5..10 {
            level.cells[x][y] = Cell::floor();
        }
    }

    // Every cell in the room interior should be lit
    for x in 10..15 {
        for y in 5..10 {
            let cell = level.cell(x, y);
            assert!(cell.lit, "Room cell at ({},{}) should be lit", x, y);
            assert_eq!(
                cell.typ,
                CellType::Room,
                "Room cell should have CellType::Room"
            );
        }
    }
}

// ============================================================================
// Test 2: Dark (unlit) room has lit=false
// ============================================================================

#[test]
fn test_dark_room_not_lit() {
    // Manually create cells with lit=false to simulate a dark room,
    // as happens on deeper dungeon levels or in morgues/vaults.
    let mut level = Level::new(DLevel::default());

    for x in 10..15 {
        for y in 5..10 {
            let mut cell = Cell::floor();
            cell.lit = false; // make it dark
            level.cells[x][y] = cell;
        }
    }

    for x in 10..15 {
        for y in 5..10 {
            let cell = level.cell(x, y);
            assert!(!cell.lit, "Dark room cell at ({},{}) should NOT be lit", x, y);
            assert_eq!(cell.typ, CellType::Room);
        }
    }

    // Also verify that Room::with_type for Morgue defaults to unlit
    let morgue_room = Room::with_type(10, 5, 5, 5, RoomType::Morgue);
    assert!(!morgue_room.lit, "Morgue room should default to unlit");

    let vault_room = Room::with_type(10, 5, 5, 5, RoomType::Vault);
    assert!(!vault_room.lit, "Vault room should default to unlit");
}

// ============================================================================
// Test 3: Corridor cells default to not lit
// ============================================================================

#[test]
fn test_corridor_default_dark() {
    // Cell::corridor() creates a corridor cell that is not lit,
    // matching NetHack's behavior where corridors are always dark.
    let corridor_cell = Cell::corridor();
    assert!(!corridor_cell.lit, "Corridor cell should default to not lit");
    assert_eq!(corridor_cell.typ, CellType::Corridor);
    assert!(!corridor_cell.was_lit, "Corridor should not have was_lit set");
    assert_eq!(corridor_cell.seen_from, 0, "Fresh corridor should have no seen_from");

    // Verify on an actual level
    let mut level = Level::new(DLevel::default());
    // Carve a corridor path
    for x in 5..20 {
        level.cells[x][10] = Cell::corridor();
    }

    for x in 5..20 {
        let cell = level.cell(x, 10);
        assert!(!cell.lit, "Corridor cell at ({},10) should not be lit", x);
    }
}

// ============================================================================
// Test 4: Player can have infravision property
// ============================================================================

#[test]
fn test_infravision_property() {
    let mut player = You::default();

    // Initially no infravision
    assert!(
        !player.properties.has(Property::Infravision),
        "Player should not start with infravision"
    );

    // Grant infravision as intrinsic (e.g., elf racial ability)
    player.properties.grant_intrinsic(Property::Infravision);
    assert!(
        player.properties.has(Property::Infravision),
        "Player should have infravision after granting intrinsic"
    );
    assert!(
        player.properties.has_intrinsic(Property::Infravision),
        "Infravision should be intrinsic"
    );

    // Remove it
    player.properties.remove_intrinsic(Property::Infravision);
    assert!(
        !player.properties.has(Property::Infravision),
        "Player should not have infravision after removal"
    );

    // Infravision is classified as a vision property
    assert!(Property::Infravision.is_vision(), "Infravision should be a vision property");
}

// ============================================================================
// Test 5: Player can have telepathy property
// ============================================================================

#[test]
fn test_telepathy_property() {
    let mut player = You::default();

    assert!(
        !player.properties.has(Property::Telepathy),
        "Player should not start with telepathy"
    );

    // Grant telepathy (e.g., from eating a floating eye)
    player.properties.grant_intrinsic(Property::Telepathy);
    assert!(
        player.properties.has(Property::Telepathy),
        "Player should have telepathy after granting"
    );

    // Telepathy is a vision property
    assert!(Property::Telepathy.is_vision(), "Telepathy should be a vision property");

    // Can also be timed (e.g., from potion of ESP)
    let mut player2 = You::default();
    player2.properties.set_timeout(Property::Telepathy, 100);
    assert!(
        player2.properties.has(Property::Telepathy),
        "Player should have telepathy from timeout"
    );
}

// ============================================================================
// Test 6: LightSource can be created with range
// ============================================================================

#[test]
fn test_light_source_creation() {
    // Create a light source from an object (e.g., a lamp)
    let ls = LightSource::from_object(10, 10, 3, ObjectId(42));
    assert_eq!(ls.x, 10);
    assert_eq!(ls.y, 10);
    assert_eq!(ls.range, 3, "Light source should have range 3");
    assert!(ls.is_for_object(ObjectId(42)));
    assert!(!ls.is_for_object(ObjectId(99)));

    // Range is clamped to 1..=15
    let ls_big = LightSource::from_object(5, 5, 100, ObjectId(1));
    assert_eq!(ls_big.range, 15, "Range should be clamped to max 15");

    let ls_zero = LightSource::from_object(5, 5, 0, ObjectId(1));
    assert_eq!(ls_zero.range, 1, "Range should be clamped to min 1");

    let ls_neg = LightSource::from_object(5, 5, -5, ObjectId(1));
    assert_eq!(ls_neg.range, 1, "Negative range should be clamped to 1");

    // Light sources can be added to a level
    let mut level = Level::new(DLevel::default());
    assert!(level.light_sources.is_empty());
    level.light_sources.push(LightSource::from_object(10, 10, 5, ObjectId(1)));
    assert_eq!(level.light_sources.len(), 1);
    assert_eq!(level.light_sources[0].range, 5);
}

// ============================================================================
// Test 7: Player can have see invisible property
// ============================================================================

#[test]
fn test_see_invisible_property() {
    let mut player = You::default();

    assert!(
        !player.properties.has(Property::SeeInvisible),
        "Player should not start with see invisible"
    );

    // Grant see invisible as intrinsic (e.g., from potion of see invisible)
    player.properties.grant_intrinsic(Property::SeeInvisible);
    assert!(
        player.properties.has(Property::SeeInvisible),
        "Player should have see invisible after granting"
    );

    // SeeInvisible is a vision property
    assert!(
        Property::SeeInvisible.is_vision(),
        "SeeInvisible should be a vision property"
    );

    // Verify it can be removed
    player.properties.remove_intrinsic(Property::SeeInvisible);
    assert!(
        !player.properties.has(Property::SeeInvisible),
        "Player should not have see invisible after removal"
    );

    // Verify that a timed version also works
    player.properties.set_timeout(Property::SeeInvisible, 50);
    assert!(
        player.properties.has(Property::SeeInvisible),
        "Timed see invisible should be active"
    );
}
