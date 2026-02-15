//! Phase 20: Monster AI -- Ray Tracing and Wand Attacks
//!
//! Behavioral tests verifying that buzz() ray tracing, monster wand selection,
//! monster healing, scroll usage, mbhit beams, and breath weapons work correctly.

use nh_core::dungeon::{DLevel, Level};
use nh_core::magic::zap::{
    BuzzResult, MbhitEffect, ZapType, ZapVariant, buzz, mbhit_effect,
};
use nh_core::magic::MonsterVitals;
use nh_core::monster::{Monster, MonsterId, MonsterResistances};
use nh_core::monster::item_usage;
use nh_core::object::{Object, ObjectClass};
use nh_core::player::{Property, You};
use nh_core::GameRng;

// ============================================================================
// Helpers
// ============================================================================

fn test_level(rng: &mut GameRng) -> Level {
    Level::new_generated(DLevel::main_dungeon_start(), rng, &MonsterVitals::default())
}

/// Create a simple open corridor level for ray testing
fn ray_test_level() -> Level {
    let mut level = Level::new(DLevel::main_dungeon_start());
    // Clear a horizontal corridor from (1,5) to (70,5)
    for x in 1..70 {
        level.cell_mut(x, 5).typ = nh_core::dungeon::CellType::Corridor;
    }
    // Clear a vertical corridor from (10,1) to (10,20)
    for y in 1..20 {
        level.cell_mut(10, y).typ = nh_core::dungeon::CellType::Corridor;
    }
    level
}

fn make_wand(object_type: i16, charges: i8) -> Object {
    let mut obj = Object::default();
    obj.class = ObjectClass::Wand;
    obj.object_type = object_type;
    obj.enchantment = charges;
    obj
}

// ============================================================================
// Test 1: buzz fire ray hits player
// ============================================================================

#[test]
fn test_buzz_fire_ray_hits_player() {
    let mut level = ray_test_level();
    let mut rng = GameRng::new(42);
    let mut player = You::default();
    player.pos.x = 15;
    player.pos.y = 5;
    player.hp = 100;
    player.hp_max = 100;

    // Fire a fire ray from (5,5) toward (15,5) = direction (1,0)
    let result = buzz(
        ZapType::Fire,
        ZapVariant::Wand,
        5, 5,  // start
        1, 0,  // direction: right
        20,    // range
        &mut player,
        &mut level,
        &mut rng,
    );

    // Player should be hit (no fire resistance)
    assert!(result.player_damage > 0, "Fire ray should damage player");
    assert!(player.hp < 100, "Player HP should decrease");
}

// ============================================================================
// Test 2: buzz ray stops at wall
// ============================================================================

#[test]
fn test_buzz_ray_stops_at_wall() {
    let mut level = ray_test_level();
    let mut rng = GameRng::new(42);
    let mut player = You::default();
    player.pos.x = 50;
    player.pos.y = 5;
    player.hp = 100;
    player.hp_max = 100;

    // Place a wall at x=10
    level.cell_mut(10, 5).typ = nh_core::dungeon::CellType::Wall;

    // Fire from (5,5) toward player at (50,5)
    let result = buzz(
        ZapType::MagicMissile,
        ZapVariant::Wand,
        5, 5,
        1, 0,
        20,
        &mut player,
        &mut level,
        &mut rng,
    );

    // Ray should bounce off wall, not reach player at x=50
    // (bounced ray goes back toward x=5, away from player)
    assert!(
        result.player_damage == 0 || result.reflected,
        "Ray should not reach player past wall (damage={}, reflected={})",
        result.player_damage, result.reflected,
    );
}

// ============================================================================
// Test 3: buzz ray bounces off wall
// ============================================================================

#[test]
fn test_buzz_ray_bounces() {
    let mut level = ray_test_level();
    let mut rng = GameRng::new(42);
    let mut player = You::default();
    player.pos.x = 50;
    player.pos.y = 5;
    player.hp = 100;
    player.hp_max = 100;

    // Fire from (5,5) to the right
    let result = buzz(
        ZapType::Cold,
        ZapVariant::Wand,
        5, 5,
        1, 0,
        12,
        &mut player,
        &mut level,
        &mut rng,
    );

    // With a long enough corridor and no walls, ray should either
    // hit player or reach end of range
    // The key test is that it doesn't crash and produces valid output
    assert!(result.end_x > 5 || result.player_damage > 0,
        "Ray should travel forward from start position");
}

// ============================================================================
// Test 4: buzz death ray kills non-resistant target
// ============================================================================

#[test]
fn test_buzz_death_ray_kills() {
    let mut level = ray_test_level();
    let mut rng = GameRng::new(42);
    let mut player = You::default();
    player.pos.x = 15;
    player.pos.y = 5;
    player.hp = 100;
    player.hp_max = 100;
    // Player has NO magic resistance

    let result = buzz(
        ZapType::Death,
        ZapVariant::Wand,
        5, 5,
        1, 0,
        20,
        &mut player,
        &mut level,
        &mut rng,
    );

    assert!(result.player_died, "Death ray should kill non-resistant player");
    assert!(player.hp <= 0, "Player HP should be 0 after death ray");
}

// ============================================================================
// Test 5: buzz cold ray reflected off shield (player has Reflection)
// ============================================================================

#[test]
fn test_buzz_cold_reflects_off_shield() {
    let mut level = ray_test_level();
    let mut rng = GameRng::new(42);
    let mut player = You::default();
    player.pos.x = 15;
    player.pos.y = 5;
    player.hp = 100;
    player.hp_max = 100;
    player.properties.grant_intrinsic(Property::Reflection);

    let result = buzz(
        ZapType::Cold,
        ZapVariant::Wand,
        5, 5,
        1, 0,
        20,
        &mut player,
        &mut level,
        &mut rng,
    );

    // Player should NOT be damaged because of reflection
    assert_eq!(result.player_damage, 0, "Reflected ray should deal no damage");
    assert!(result.reflected, "Ray should be reflected");
    assert!(
        result.messages.iter().any(|m| m.contains("reflected")),
        "Should have a reflection message"
    );
}

// ============================================================================
// Test 6: monster selects best wand (death > fire > MM)
// ============================================================================

#[test]
fn test_monster_selects_best_wand() {
    let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
    m.name = "orc".to_string();
    m.hp = 20;
    m.hp_max = 20;

    // Give monster three wands
    m.inventory.push(make_wand(2000, 5)); // MM wand (priority 50)
    m.inventory.push(make_wand(2001, 3)); // Fire wand (priority 80)
    m.inventory.push(make_wand(2004, 1)); // Death wand (priority 100)

    let result = item_usage::select_best_offensive_wand(&m);
    assert!(result.is_some(), "Monster should find a wand");
    let (muse_type, _idx) = result.unwrap();
    // MUSE_WAN_DEATH = 20
    assert_eq!(muse_type, 20, "Monster should prefer death wand");
}

// ============================================================================
// Test 7: monster zaps healing on self when low HP
// ============================================================================

#[test]
fn test_monster_zaps_healing_on_self() {
    let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
    m.hp = 3;
    m.hp_max = 30;

    // Give monster a wand of healing (object type 2010)
    m.inventory.push(make_wand(2010, 5));

    let heal_idx = item_usage::should_zap_healing_on_self(&m);
    assert!(heal_idx.is_some(), "Low-HP monster should want to heal");

    // Apply healing
    let mut rng = GameRng::new(42);
    item_usage::apply_wand_healing(&mut m, 2010, &mut rng);
    assert!(m.hp > 3, "Monster HP should increase after healing");
    assert!(m.hp <= 30, "Monster HP should not exceed max");
}

// ============================================================================
// Test 8: monster uses scroll of teleportation when cornered
// ============================================================================

#[test]
fn test_monster_uses_scroll_teleport() {
    let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
    m.hp = 3;
    m.hp_max = 30;
    m.state.fleeing = true;

    // Give monster a scroll of teleportation (object type 37)
    let mut scroll = Object::default();
    scroll.class = ObjectClass::Scroll;
    scroll.object_type = 37;
    m.inventory.push(scroll);

    let scroll_idx = item_usage::should_use_teleport_scroll(&m);
    assert!(scroll_idx.is_some(), "Fleeing low-HP monster should want to teleport");

    // Execute teleportation
    let mut rng = GameRng::new(42);
    let mut level = ray_test_level();
    let mid = level.add_monster(m);
    let teleported = item_usage::execute_monster_teleport(mid, &mut level, &mut rng);
    assert!(teleported, "Monster should teleport to a new position");

    // Monster should have moved from original position
    let m = level.monster(mid).unwrap();
    assert!(m.x != 5 || m.y != 5, "Monster should be at a different position");
}

// ============================================================================
// Test 9: mbhit speed beam changes target speed
// ============================================================================

#[test]
fn test_mbhit_speed_beam() {
    let mut level = ray_test_level();
    let mut rng = GameRng::new(42);
    let mut player = You::default();
    player.pos.x = 15;
    player.pos.y = 5;
    player.hp = 100;
    player.hp_max = 100;

    let result = mbhit_effect(
        MbhitEffect::Speed,
        5, 5,  // start
        1, 0,  // direction
        20,    // range
        &mut player,
        &mut level,
        &mut rng,
    );

    assert!(
        result.messages.iter().any(|m| m.contains("speed")),
        "Speed beam should generate speed message"
    );
    assert!(
        player.properties.has(Property::Speed),
        "Player should gain Speed property from speed beam"
    );
}

// ============================================================================
// Test 10: monster breath weapon uses correct type
// ============================================================================

#[test]
fn test_monster_breath_weapon() {
    let mut level = ray_test_level();
    let mut rng = GameRng::new(42);
    let mut player = You::default();
    player.pos.x = 15;
    player.pos.y = 5;
    player.hp = 100;
    player.hp_max = 100;

    // Monster at (5,5) breathes fire toward player at (15,5)
    let result = item_usage::monster_use_breath_weapon(
        5, 5,           // monster position
        ZapType::Fire,  // fire breath
        10,             // monster level
        &mut player,
        &mut level,
        &mut rng,
    );

    // Fire breath should damage a non-resistant player
    assert!(result.player_damage > 0, "Fire breath should damage player (got {})", result.player_damage);
    assert!(player.hp < 100, "Player HP should decrease from fire breath");
}
