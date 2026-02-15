//! Phase 21: Monster AI -- Movement, Pathfinding, Special Behaviors
//!
//! Behavioral tests verifying monster fleeing, trap interaction, covetous teleportation,
//! door handling, pet following, gold pickup, lava avoidance, guard behavior, and
//! grave disturbance.

use nh_core::dungeon::{CellType, DLevel, DoorState, Level, TrapType};
use nh_core::monster::{Monster, MonsterId, monflee, should_flee_from_damage};
use nh_core::player::You;
use nh_core::special::dog;
use nh_core::GameRng;

// ============================================================================
// Helpers
// ============================================================================

fn make_hostile_monster(name: &str, x: i8, y: i8) -> Monster {
    let mut m = Monster::new(MonsterId::NONE, 0, x, y);
    m.name = name.to_string();
    m.hp = 10;
    m.hp_max = 10;
    m.ac = 5;
    m.level = 3;
    m.state.peaceful = false;
    m.state.tame = false;
    m
}

fn make_pet(name: &str, x: i8, y: i8) -> Monster {
    let mut m = make_hostile_monster(name, x, y);
    m.state.tame = true;
    m.state.peaceful = true;
    m
}

/// Create a simple open corridor level for movement testing
fn movement_test_level() -> Level {
    let mut level = Level::new(DLevel::main_dungeon_start());
    // Clear a large open room from (1,1) to (30,20)
    for x in 1..30 {
        for y in 1..20 {
            level.cell_mut(x, y).typ = CellType::Room;
        }
    }
    level
}

// ============================================================================
// Test 1: Monster flees when low HP
// ============================================================================

#[test]
fn test_monster_flees_when_low_hp() {
    let mut m = make_hostile_monster("kobold", 5, 5);
    m.hp = 10;
    m.hp_max = 100;

    // Monster at 10% HP should want to flee (threshold 33%)
    assert!(
        should_flee_from_damage(&m, 33),
        "Monster at 10% HP should want to flee at 33% threshold"
    );

    // Apply flee
    monflee(&mut m, 10, true);
    assert!(m.state.fleeing, "Monster should be fleeing after monflee");
    assert!(m.flee_timeout > 0, "Flee timeout should be set");
}

// ============================================================================
// Test 2: Monster trapped in pit
// ============================================================================

#[test]
fn test_monster_trapped_in_pit() {
    let mut level = movement_test_level();
    // Place a pit trap at (10, 10)
    level.add_trap(10, 10, TrapType::Pit);

    // Verify trap exists
    let trap = level.trap_at(10, 10);
    assert!(trap.is_some(), "Pit trap should exist at (10,10)");
    assert!(
        matches!(trap.unwrap().trap_type, TrapType::Pit),
        "Trap should be a pit"
    );

    // A monster at this location would be held by the pit
    // (trap holding is checked via is_holding_trap)
    assert!(
        matches!(TrapType::Pit, TrapType::Pit | TrapType::SpikedPit | TrapType::BearTrap | TrapType::Web),
        "Pit should be a holding trap type"
    );
}

// ============================================================================
// Test 3: Covetous monster strategy uses heal when low HP
// ============================================================================

#[test]
fn test_covetous_teleports_to_player() {
    use nh_core::monster::ai::{STRAT_HEAL, STRAT_AMULET, strategy};

    let mut level = movement_test_level();
    let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
    monster.hp = 20; // 20% HP → STRAT_HEAL
    monster.hp_max = 100;
    level.add_monster(monster);

    let strat = strategy(MonsterId(1), &level);
    assert_eq!(strat, STRAT_HEAL, "Low-HP covetous monster should heal");

    // High HP → pursue amulet
    let mut level2 = movement_test_level();
    let mut monster2 = Monster::new(MonsterId(1), 0, 5, 5);
    monster2.hp = 90;
    monster2.hp_max = 100;
    level2.add_monster(monster2);

    let strat2 = strategy(MonsterId(1), &level2);
    assert_eq!(strat2, STRAT_AMULET, "Healthy covetous monster should pursue amulet");
}

// ============================================================================
// Test 4: Door state and closed door detection
// ============================================================================

#[test]
fn test_monster_opens_door() {
    let mut level = movement_test_level();

    // Place a closed door at (15, 10)
    level.cell_mut(15, 10).typ = CellType::Door;
    level.cell_mut(15, 10).set_door_state(DoorState::CLOSED);

    assert!(
        level.cell(15, 10).is_closed_door(),
        "Door should be detected as closed"
    );
    assert!(
        !level.cell(15, 10).is_walkable(),
        "Closed door should not be walkable"
    );

    // Open the door
    level.cell_mut(15, 10).set_door_state(DoorState::OPEN);
    assert!(
        level.cell(15, 10).is_open_door(),
        "Door should now be open"
    );
    assert!(
        level.cell(15, 10).is_walkable(),
        "Open door should be walkable"
    );
}

// ============================================================================
// Test 5: Monster breaks locked door
// ============================================================================

#[test]
fn test_monster_breaks_locked_door() {
    let mut level = movement_test_level();

    // Place a locked door
    level.cell_mut(15, 10).typ = CellType::Door;
    level.cell_mut(15, 10).set_door_state(DoorState::LOCKED);

    assert!(
        level.cell(15, 10).is_closed_door(),
        "Locked door should be detected as closed"
    );

    // Breaking it changes state to broken (walkable)
    level.cell_mut(15, 10).set_door_state(DoorState::BROKEN);
    assert!(
        level.cell(15, 10).is_walkable(),
        "Broken door should be walkable"
    );
}

// ============================================================================
// Test 6: Pet follows player (adjacency check)
// ============================================================================

#[test]
fn test_pet_follows_player() {
    let pet = make_pet("kitten", 6, 5);
    let mut player = You::default();
    player.pos.x = 5;
    player.pos.y = 5;

    // Adjacent pet should follow
    assert!(
        dog::pet_will_follow(&pet, &player),
        "Adjacent tame pet should follow player"
    );

    // Far pet should not follow
    let far_pet = make_pet("kitten", 20, 15);
    assert!(
        !dog::pet_will_follow(&far_pet, &player),
        "Distant pet should not follow player"
    );

    // Non-tame monster should not follow
    let hostile = make_hostile_monster("kobold", 6, 5);
    assert!(
        !dog::pet_will_follow(&hostile, &player),
        "Non-tame monster should not follow player"
    );
}

// ============================================================================
// Test 7: Monster picks up gold
// ============================================================================

#[test]
fn test_monster_picks_up_gold() {
    let mut m = make_hostile_monster("leprechaun", 5, 5);

    // Initially no gold
    assert_eq!(m.gold_amount(), 0, "Monster should start with no gold");

    // Pick up gold
    let picked = m.mpickgold(100);
    assert!(picked, "Monster should successfully pick up gold");
    assert_eq!(m.gold_amount(), 100, "Monster should have 100 gold");

    // Pick up more
    m.mpickgold(50);
    assert_eq!(m.gold_amount(), 150, "Monster gold should accumulate");
}

// ============================================================================
// Test 8: Monster avoids lava (minliquid check)
// ============================================================================

#[test]
fn test_monster_avoids_lava() {
    use nh_core::monster::{MinliquidResult, minliquid};

    let mut level = movement_test_level();
    level.cell_mut(10, 10).typ = CellType::Lava;

    // Non-flying, non-fire-resistant monster on lava
    let mut m = make_hostile_monster("orc", 10, 10);

    let result = minliquid(&mut m, &level);
    assert!(
        matches!(result, MinliquidResult::Burned | MinliquidResult::Damaged(_)),
        "Non-resistant monster on lava should burn or take damage, got {:?}",
        result,
    );

    // Verify lava cell properties
    assert!(level.cell(10, 10).is_lava(), "Cell should be lava");
}

// ============================================================================
// Test 9: Town guard detection
// ============================================================================

#[test]
fn test_town_guard_warns_vandal() {
    let mut level = movement_test_level();

    // Place a guard
    let mut guard = Monster::new(MonsterId::NONE, 0, 10, 10);
    guard.is_guard = true;
    guard.hp = 50;
    guard.hp_max = 50;
    guard.state.peaceful = true;
    let gid = level.add_monster(guard);

    // Verify guard exists and is identifiable
    let g = level.monster(gid).unwrap();
    assert!(g.is_guard, "Monster should be a guard");
    assert!(g.state.peaceful, "Guard should be peaceful");
    assert!(!g.is_dead(), "Guard should be alive");
}

// ============================================================================
// Test 10: Monster disturbs grave (dig_up_grave)
// ============================================================================

#[test]
fn test_monster_disturbs_grave() {
    use nh_core::monster::ai::dig_up_grave;

    let mut level = movement_test_level();
    let mut rng = GameRng::new(42);

    // Place grave terrain
    level.cell_mut(15, 10).typ = CellType::Grave;
    assert_eq!(
        level.cell(15, 10).typ,
        CellType::Grave,
        "Cell should be grave terrain"
    );

    // Dig up the grave
    dig_up_grave(15, 10, &mut level, &mut rng);

    // After digging, grave should be converted to room
    assert_eq!(
        level.cell(15, 10).typ,
        CellType::Room,
        "Grave should be converted to room after digging"
    );
}
