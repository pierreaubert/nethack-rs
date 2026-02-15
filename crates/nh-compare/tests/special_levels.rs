//! Phase 24: Special Level Generation
//!
//! Behavioral tests verifying that special levels (Sokoban, Mines, Gehennom,
//! Quest, Endgame planes) generate correctly with expected terrain features,
//! level flags, and structural properties.

use nh_core::dungeon::{
    CellType, DLevel, Level, Plane, QuestInfo, SpecialLevelId, TrapType,
    generate_plane, generate_quest_home, generate_special_level, get_special_level,
};
use nh_core::player::Role;
use nh_core::GameRng;

// ============================================================================
// Helpers
// ============================================================================

/// Count cells of a given type on a level.
fn count_cell_type(level: &Level, cell_type: CellType) -> usize {
    level
        .cells
        .iter()
        .flat_map(|col| col.iter())
        .filter(|c| c.typ == cell_type)
        .count()
}

/// Count traps of a given type on a level.
fn count_trap_type(level: &Level, trap_type: TrapType) -> usize {
    level
        .traps
        .iter()
        .filter(|t| t.trap_type == trap_type)
        .count()
}

// ============================================================================
// Test 1: Sokoban special level ID exists and generates
// ============================================================================

#[test]
fn test_sokoban_level_exists() {
    // Sokoban level 1 should be recognized at dungeon 3, level 1
    let dlevel = DLevel::new(3, 1);
    let special = get_special_level(&dlevel);
    assert_eq!(
        special,
        Some(SpecialLevelId::Sokoban1a),
        "Sokoban level 1 should be recognized as special"
    );

    // Generate it and verify basic structure
    let mut rng = GameRng::new(42);
    let mut level = Level::new(dlevel);
    generate_special_level(&mut level, SpecialLevelId::Sokoban1a, &mut rng);

    // Sokoban should have room cells (the puzzle area)
    let room_count = count_cell_type(&level, CellType::Room);
    assert!(
        room_count > 50,
        "Sokoban should have a significant puzzle area, got {} room cells",
        room_count
    );

    // Sokoban should have hole traps (part of the puzzle)
    let hole_count = count_trap_type(&level, TrapType::Hole);
    assert!(
        hole_count > 0,
        "Sokoban should have hole traps as part of the puzzle"
    );

    // Sokoban should have stairs (entry and exit)
    assert!(
        level.stairs.len() >= 2,
        "Sokoban should have at least entry and exit stairs, got {}",
        level.stairs.len()
    );

    // Verify the display name
    assert_eq!(SpecialLevelId::Sokoban1a.name(), "Sokoban Level 1");
}

// ============================================================================
// Test 2: Sokoban level 4 has a prize room at the top
// ============================================================================

#[test]
fn test_sokoban_prize_level() {
    // Sokoban level 4 is the prize level
    let dlevel = DLevel::new(3, 4);
    let special = get_special_level(&dlevel);
    assert_eq!(
        special,
        Some(SpecialLevelId::Sokoban4a),
        "Sokoban level 4 should be recognized"
    );

    let mut rng = GameRng::new(99);
    let mut level = Level::new(dlevel);
    generate_special_level(&mut level, SpecialLevelId::Sokoban4a, &mut rng);

    // Prize level should have a substantial puzzle area plus prize room
    let room_count = count_cell_type(&level, CellType::Room);
    assert!(
        room_count > 100,
        "Sokoban prize level should have puzzle area + prize room, got {} room cells",
        room_count
    );

    // All four Sokoban levels should exist
    assert!(get_special_level(&DLevel::new(3, 1)).is_some());
    assert!(get_special_level(&DLevel::new(3, 2)).is_some());
    assert!(get_special_level(&DLevel::new(3, 3)).is_some());
    assert!(get_special_level(&DLevel::new(3, 4)).is_some());
}

// ============================================================================
// Test 3: Mines Town generation with shops and temple
// ============================================================================

#[test]
fn test_mines_town_generation() {
    let dlevel = DLevel::new(2, 3);
    assert_eq!(
        get_special_level(&dlevel),
        Some(SpecialLevelId::MinesTown),
        "Mines Town should be recognized"
    );

    let mut rng = GameRng::new(42);
    let mut level = Level::new(dlevel);
    generate_special_level(&mut level, SpecialLevelId::MinesTown, &mut rng);

    // Minetown has a temple with an altar
    let altar_count = count_cell_type(&level, CellType::Altar);
    assert!(
        altar_count >= 1,
        "Minetown should have at least one altar (temple), got {}",
        altar_count
    );

    // Minetown has fountains in the town square
    let fountain_count = count_cell_type(&level, CellType::Fountain);
    assert!(
        fountain_count >= 2,
        "Minetown should have at least two fountains, got {}",
        fountain_count
    );

    // Minetown has corridors (the main street)
    let corridor_count = count_cell_type(&level, CellType::Corridor);
    assert!(
        corridor_count > 10,
        "Minetown should have a main street corridor, got {} corridor cells",
        corridor_count
    );

    // Should have doors connecting shops to the street
    let door_count = count_cell_type(&level, CellType::Door);
    assert!(
        door_count >= 3,
        "Minetown should have shop doors, got {}",
        door_count
    );
}

// ============================================================================
// Test 4: Mines End generation (luckstone level)
// ============================================================================

#[test]
fn test_mines_end_generation() {
    let dlevel = DLevel::new(2, 8);
    assert_eq!(
        get_special_level(&dlevel),
        Some(SpecialLevelId::MinesEnd1),
        "Mines End should be recognized"
    );

    let mut rng = GameRng::new(42);
    let mut level = Level::new(dlevel);
    generate_special_level(&mut level, SpecialLevelId::MinesEnd1, &mut rng);

    // Mines End has a central lit treasure room
    let lit_room_count = level
        .cells
        .iter()
        .flat_map(|col| col.iter())
        .filter(|c| c.typ == CellType::Room && c.lit)
        .count();
    assert!(
        lit_room_count > 10,
        "Mines End should have a lit central treasure room, got {} lit room cells",
        lit_room_count
    );

    // Mines End has corridors connecting cavern rooms
    let corridor_count = count_cell_type(&level, CellType::Corridor);
    assert!(
        corridor_count > 0,
        "Mines End should have corridors connecting rooms"
    );

    // Should have stairs
    assert!(
        !level.stairs.is_empty(),
        "Mines End should have stairs"
    );

    // Verify the display name
    assert_eq!(SpecialLevelId::MinesEnd1.name(), "Mines End");
}

// ============================================================================
// Test 5: Sanctum generation (Moloch's temple)
// ============================================================================

#[test]
fn test_sanctum_generation() {
    let dlevel = DLevel::new(1, 20);
    assert_eq!(
        get_special_level(&dlevel),
        Some(SpecialLevelId::Sanctum),
        "Sanctum should be recognized"
    );

    let mut rng = GameRng::new(42);
    let mut level = Level::new(dlevel);
    generate_special_level(&mut level, SpecialLevelId::Sanctum, &mut rng);

    // Sanctum has a lava moat surrounding the structure
    let lava_count = count_cell_type(&level, CellType::Lava);
    assert!(
        lava_count > 30,
        "Sanctum should have a significant lava moat, got {} lava cells",
        lava_count
    );

    // Sanctum has exactly one high altar (to Moloch)
    let altar_count = count_cell_type(&level, CellType::Altar);
    assert_eq!(
        altar_count, 1,
        "Sanctum should have exactly one high altar"
    );

    // Sanctum has iron bars separating chambers
    let bars_count = count_cell_type(&level, CellType::IronBars);
    assert!(
        bars_count > 0,
        "Sanctum should have iron bars separating chambers"
    );

    // Sanctum has a drawbridge entrance
    let drawbridge_count = count_cell_type(&level, CellType::DrawbridgeDown);
    assert!(
        drawbridge_count > 0,
        "Sanctum should have a drawbridge entrance"
    );

    // Critical flags must be set
    assert!(level.flags.no_teleport, "Sanctum must have no_teleport flag");
    assert!(level.flags.hard_floor, "Sanctum must have hard_floor flag");
    assert!(
        level.flags.no_magic_map,
        "Sanctum must have no_magic_map flag"
    );
}

// ============================================================================
// Test 6: Wizard Tower generation
// ============================================================================

#[test]
fn test_wizard_tower_generation() {
    // All three Wizard Tower levels should exist
    assert_eq!(
        get_special_level(&DLevel::new(1, 17)),
        Some(SpecialLevelId::WizardTower1)
    );
    assert_eq!(
        get_special_level(&DLevel::new(1, 18)),
        Some(SpecialLevelId::WizardTower2)
    );
    assert_eq!(
        get_special_level(&DLevel::new(1, 19)),
        Some(SpecialLevelId::WizardTower3)
    );

    // Generate the top floor (Wizard's lair)
    let mut rng = GameRng::new(42);
    let mut level = Level::new(DLevel::new(1, 19));
    generate_special_level(&mut level, SpecialLevelId::WizardTower3, &mut rng);

    // Top floor should be lit (powerful Wizard presence)
    let lit_room_count = level
        .cells
        .iter()
        .flat_map(|col| col.iter())
        .filter(|c| c.typ == CellType::Room && c.lit)
        .count();
    assert!(
        lit_room_count > 10,
        "Wizard Tower top floor should be lit, got {} lit room cells",
        lit_room_count
    );

    // Should have magic traps
    let magic_traps = count_trap_type(&level, TrapType::MagicTrap);
    assert!(
        magic_traps > 0,
        "Wizard Tower should have magic traps"
    );

    // Must have no_teleport flag
    assert!(
        level.flags.no_teleport,
        "Wizard Tower must have no_teleport flag"
    );

    // Tower gets smaller as you go up: verify floor 1 is larger than floor 3
    let mut rng2 = GameRng::new(42);
    let mut level1 = Level::new(DLevel::new(1, 17));
    generate_special_level(&mut level1, SpecialLevelId::WizardTower1, &mut rng2);
    let room_count_1 = count_cell_type(&level1, CellType::Room);
    let room_count_3 = count_cell_type(&level, CellType::Room);
    assert!(
        room_count_1 > room_count_3,
        "Wizard Tower floor 1 ({}) should be larger than floor 3 ({})",
        room_count_1,
        room_count_3
    );
}

// ============================================================================
// Test 7: Quest level IDs exist for all roles
// ============================================================================

#[test]
fn test_quest_level_exists() {
    let roles = [
        Role::Archeologist,
        Role::Barbarian,
        Role::Caveman,
        Role::Healer,
        Role::Knight,
        Role::Monk,
        Role::Priest,
        Role::Ranger,
        Role::Rogue,
        Role::Samurai,
        Role::Tourist,
        Role::Valkyrie,
        Role::Wizard,
    ];

    for role in roles {
        let info = QuestInfo::for_role(role);

        // Every role must have a quest leader, nemesis, and artifact
        assert!(
            !info.leader_name.is_empty(),
            "{:?} quest should have a leader name",
            role
        );
        assert!(
            !info.nemesis_name.is_empty(),
            "{:?} quest should have a nemesis name",
            role
        );
        assert!(
            !info.artifact_name.is_empty(),
            "{:?} quest should have an artifact name",
            role
        );
        assert!(
            !info.goal_name.is_empty(),
            "{:?} quest should have a goal name",
            role
        );
        assert!(
            !info.home_name.is_empty(),
            "{:?} quest should have a home name",
            role
        );
    }

    // Verify quest home level generates for a specific role
    let mut rng = GameRng::new(42);
    let mut level = Level::new(DLevel::new(4, 1));
    generate_quest_home(&mut level, Role::Valkyrie, &mut rng);

    // Valkyrie quest home should have a throne (Shrine of Destiny)
    let throne_count = count_cell_type(&level, CellType::Throne);
    assert_eq!(
        throne_count, 1,
        "Valkyrie quest home should have a throne"
    );

    // Should have stairs to quest and back to main dungeon
    assert!(
        level.stairs.len() >= 2,
        "Quest home should have entry and exit stairs"
    );
}

// ============================================================================
// Test 8: Astral Plane generation with three temples
// ============================================================================

#[test]
fn test_astral_plane_generation() {
    let mut rng = GameRng::new(42);
    let mut level = Level::new(Plane::Astral.dlevel());
    generate_plane(&mut level, Plane::Astral, &mut rng);

    // Astral Plane has exactly 3 altars (Lawful, Neutral, Chaotic temples)
    let altar_count = count_cell_type(&level, CellType::Altar);
    assert_eq!(
        altar_count, 3,
        "Astral Plane must have exactly 3 temple altars"
    );

    // Astral Plane is primarily Cloud terrain
    let cloud_count = count_cell_type(&level, CellType::Cloud);
    assert!(
        cloud_count > 100,
        "Astral Plane should be mostly clouds, got {} cloud cells",
        cloud_count
    );

    // Has a main corridor connecting the temples
    let room_count = count_cell_type(&level, CellType::Room);
    assert!(
        room_count > 50,
        "Astral Plane should have temple rooms and corridor, got {} room cells",
        room_count
    );

    // Must have no_teleport flag (no escaping the final challenge)
    assert!(
        level.flags.no_teleport,
        "Astral Plane must have no_teleport flag"
    );

    // Should have entry stairs from Water Plane
    assert!(
        !level.stairs.is_empty(),
        "Astral Plane should have entry stairs from Water Plane"
    );

    // Verify the Astral Plane DLevel
    assert_eq!(Plane::Astral.dlevel(), DLevel::new(7, 5));
    assert_eq!(Plane::Astral.name(), "The Astral Plane");
}

// ============================================================================
// Test 9: Fire Plane generation with lava and fire traps
// ============================================================================

#[test]
fn test_fire_plane_generation() {
    let mut rng = GameRng::new(42);
    let mut level = Level::new(Plane::Fire.dlevel());
    generate_plane(&mut level, Plane::Fire, &mut rng);

    // Fire Plane should be primarily lava
    let lava_count = count_cell_type(&level, CellType::Lava);
    assert!(
        lava_count > 200,
        "Fire Plane should be mostly lava, got {} lava cells",
        lava_count
    );

    // Fire Plane should have stone islands (Room cells) for traversal
    let room_count = count_cell_type(&level, CellType::Room);
    assert!(
        room_count > 30,
        "Fire Plane should have traversable islands, got {} room cells",
        room_count
    );

    // Fire Plane should have fire traps
    let fire_traps = count_trap_type(&level, TrapType::FireTrap);
    assert!(
        fire_traps > 0,
        "Fire Plane should have fire traps"
    );

    // Must have no_teleport flag (all planes block teleport)
    assert!(
        level.flags.no_teleport,
        "Fire Plane must have no_teleport flag"
    );

    // Should have a portal to the next plane (Water) represented as stairs
    assert!(
        !level.stairs.is_empty(),
        "Fire Plane should have portal/stairs to next plane"
    );

    // Verify the Fire Plane DLevel
    assert_eq!(Plane::Fire.dlevel(), DLevel::new(7, 3));
    assert_eq!(Plane::Fire.name(), "The Plane of Fire");
}

// ============================================================================
// Test 10: Special level flags are correctly set
// ============================================================================

#[test]
fn test_special_level_flags() {
    let mut rng = GameRng::new(42);

    // --- Sanctum: no_teleport, hard_floor, no_magic_map ---
    let mut sanctum = Level::new(DLevel::new(1, 20));
    generate_special_level(&mut sanctum, SpecialLevelId::Sanctum, &mut rng);
    assert!(sanctum.flags.no_teleport, "Sanctum: no_teleport");
    assert!(sanctum.flags.hard_floor, "Sanctum: hard_floor");
    assert!(sanctum.flags.no_magic_map, "Sanctum: no_magic_map");

    // --- Wizard Tower: no_teleport ---
    let mut wiz = Level::new(DLevel::new(1, 17));
    generate_special_level(&mut wiz, SpecialLevelId::WizardTower1, &mut rng);
    assert!(wiz.flags.no_teleport, "Wizard Tower: no_teleport");

    // --- Vlad's Tower: no_teleport ---
    let mut vlad = Level::new(DLevel::new(6, 1));
    generate_special_level(&mut vlad, SpecialLevelId::VladsTower1, &mut rng);
    assert!(vlad.flags.no_teleport, "Vlad's Tower: no_teleport");

    // --- Valley of the Dead: graveyard flag ---
    let mut valley = Level::new(DLevel::new(0, 26));
    generate_special_level(&mut valley, SpecialLevelId::Valley, &mut rng);
    assert!(valley.flags.graveyard, "Valley: graveyard flag");
    let grave_count = count_cell_type(&valley, CellType::Grave);
    assert!(grave_count > 0, "Valley should have graves");

    // --- Rogue Level: corridor_maze flag ---
    let mut rogue = Level::new(DLevel::new(0, 15));
    generate_special_level(&mut rogue, SpecialLevelId::RogueLevel, &mut rng);
    assert!(rogue.flags.corridor_maze, "Rogue Level: corridor_maze flag");

    // --- Astral Plane: no_teleport (set by endgame fill_level) ---
    let mut astral = Level::new(Plane::Astral.dlevel());
    generate_plane(&mut astral, Plane::Astral, &mut rng);
    assert!(astral.flags.no_teleport, "Astral Plane: no_teleport");

    // --- Earth Plane: no_teleport ---
    let mut earth = Level::new(Plane::Earth.dlevel());
    generate_plane(&mut earth, Plane::Earth, &mut rng);
    assert!(earth.flags.no_teleport, "Earth Plane: no_teleport");

    // --- All endgame planes have no_teleport ---
    for plane in [Plane::Earth, Plane::Air, Plane::Fire, Plane::Water, Plane::Astral] {
        let mut lvl = Level::new(plane.dlevel());
        generate_plane(&mut lvl, plane, &mut rng);
        assert!(
            lvl.flags.no_teleport,
            "{:?} Plane must have no_teleport",
            plane
        );
    }
}
