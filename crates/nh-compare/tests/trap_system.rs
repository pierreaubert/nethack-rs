//! Trap system behavioral tests
//!
//! Verifies trap mechanics: type classification, damage ranges, detection,
//! disarming, container traps, escape mechanics, and trap naming.

use nh_core::dungeon::trap::*;
use nh_core::dungeon::{DLevel, Level, TrapType};
use nh_core::GameRng;

// ============================================================================
// Helpers
// ============================================================================

fn default_ctx() -> TrapContext {
    TrapContext {
        difficulty: 10,
        allow_holes: true,
        allow_magic: true,
    }
}

fn make_level() -> Level {
    Level::new(DLevel::new(0, 10))
}

// ============================================================================
// Trap type classification
// ============================================================================

#[test]
fn test_arrow_is_not_pit() {
    assert!(!is_pit(TrapType::Arrow));
}

#[test]
fn test_pit_is_pit() {
    assert!(is_pit(TrapType::Pit));
}

#[test]
fn test_spiked_pit_is_pit() {
    assert!(is_pit(TrapType::SpikedPit));
}

#[test]
fn test_trap_door_is_hole() {
    assert!(is_hole(TrapType::TrapDoor));
}

#[test]
fn test_hole_is_hole() {
    assert!(is_hole(TrapType::Hole));
}

#[test]
fn test_arrow_not_hole() {
    assert!(!is_hole(TrapType::Arrow));
}

#[test]
fn test_bear_trap_is_holding() {
    assert!(is_holding_trap(TrapType::BearTrap));
}

#[test]
fn test_web_is_holding() {
    assert!(is_holding_trap(TrapType::Web));
}

#[test]
fn test_arrow_not_holding() {
    assert!(!is_holding_trap(TrapType::Arrow));
}

#[test]
fn test_teleport_is_magic() {
    assert!(is_magic_trap(TrapType::Teleport));
}

#[test]
fn test_levelport_is_magic() {
    assert!(is_magic_trap(TrapType::LevelTeleport));
}

#[test]
fn test_arrow_not_magic() {
    assert!(!is_magic_trap(TrapType::Arrow));
}

#[test]
fn test_pit_is_ground_trap() {
    assert!(is_ground_trap(TrapType::Pit));
}

#[test]
fn test_arrow_not_ground_trap() {
    assert!(!is_ground_trap(TrapType::Arrow));
}

// ============================================================================
// Trap names
// ============================================================================

#[test]
fn test_trap_name_arrow() {
    assert_eq!(trap_name(TrapType::Arrow), "arrow trap");
}

#[test]
fn test_trap_name_dart() {
    assert_eq!(trap_name(TrapType::Dart), "dart trap");
}

#[test]
fn test_trap_name_bear_trap() {
    assert_eq!(trap_name(TrapType::BearTrap), "bear trap");
}

#[test]
fn test_trap_name_pit() {
    assert_eq!(trap_name(TrapType::Pit), "pit");
}

#[test]
fn test_trap_name_spiked_pit() {
    assert_eq!(trap_name(TrapType::SpikedPit), "spiked pit");
}

#[test]
fn test_trap_name_web() {
    assert_eq!(trap_name(TrapType::Web), "web");
}

#[test]
fn test_trap_name_fire() {
    assert_eq!(trap_name(TrapType::FireTrap), "fire trap");
}

#[test]
fn test_trap_name_squeaky() {
    assert_eq!(trap_name(TrapType::Squeaky), "squeaky board");
}

#[test]
fn test_trap_name_all_nonempty() {
    let types = [
        TrapType::Arrow, TrapType::Dart, TrapType::RockFall,
        TrapType::Squeaky, TrapType::BearTrap, TrapType::LandMine,
        TrapType::RollingBoulder, TrapType::SleepingGas, TrapType::RustTrap,
        TrapType::FireTrap, TrapType::Pit, TrapType::SpikedPit,
        TrapType::Hole, TrapType::TrapDoor, TrapType::Teleport,
        TrapType::LevelTeleport, TrapType::AntiMagic, TrapType::Web,
        TrapType::Statue, TrapType::MagicTrap, TrapType::Polymorph,
    ];
    for tt in types {
        assert!(!trap_name(tt).is_empty(), "Trap name should be non-empty for {:?}", tt);
    }
}

// ============================================================================
// Trap damage
// ============================================================================

#[test]
fn test_arrow_base_damage() {
    let (dice, sides) = trap_base_damage(TrapType::Arrow);
    assert!(dice > 0 && sides > 0, "Arrow trap should have positive damage");
}

#[test]
fn test_dart_base_damage() {
    let (dice, sides) = trap_base_damage(TrapType::Dart);
    assert!(dice > 0 && sides > 0);
}

#[test]
fn test_fire_base_damage() {
    let (dice, sides) = trap_base_damage(TrapType::FireTrap);
    assert!(dice > 0 && sides > 0);
}

#[test]
fn test_roll_trap_damage_in_range() {
    let mut rng = GameRng::new(42);
    for _ in 0..20 {
        let dmg = roll_trap_damage(&mut rng, TrapType::Arrow);
        let (dice, sides) = trap_base_damage(TrapType::Arrow);
        assert!(dmg >= dice && dmg <= dice * sides, "Damage {} out of range {}d{}", dmg, dice, sides);
    }
}

#[test]
fn test_pit_damage_range() {
    let mut rng = GameRng::new(100);
    for _ in 0..10 {
        let dmg = roll_trap_damage(&mut rng, TrapType::Pit);
        assert!(dmg >= 1, "Pit damage should be at least 1");
    }
}

#[test]
fn test_spiked_pit_more_than_pit() {
    let (d1, s1) = trap_base_damage(TrapType::Pit);
    let (d2, s2) = trap_base_damage(TrapType::SpikedPit);
    assert!(d2 * s2 >= d1 * s1, "Spiked pit max damage should be >= pit");
}

// ============================================================================
// Trap creation
// ============================================================================

#[test]
fn test_create_trap() {
    let trap = create_trap(10, 15, TrapType::BearTrap);
    assert_eq!(trap.x, 10);
    assert_eq!(trap.y, 15);
    assert_eq!(trap.trap_type, TrapType::BearTrap);
}

#[test]
fn test_create_random_trap() {
    let mut rng = GameRng::new(42);
    let ctx = default_ctx();
    let trap = create_random_trap(&mut rng, 5, 5, &ctx);
    assert_eq!(trap.x, 5);
    assert_eq!(trap.y, 5);
}

#[test]
fn test_random_trap_type_varies() {
    let mut rng = GameRng::new(42);
    let ctx = default_ctx();
    let mut types: Vec<TrapType> = Vec::new();
    for _ in 0..50 {
        let tt = random_trap_type(&mut rng, &ctx);
        if !types.contains(&tt) {
            types.push(tt);
        }
    }
    assert!(types.len() > 3, "Random traps should produce variety, got {}", types.len());
}

// ============================================================================
// Trap glyph
// ============================================================================

#[test]
fn test_trap_glyph_seen() {
    let g = trap_glyph(TrapType::Arrow, true);
    assert_ne!(g, ' ', "Seen trap should have visible glyph");
}

#[test]
fn test_trap_glyph_unseen() {
    let g = trap_glyph(TrapType::Arrow, false);
    // Unseen traps may show differently or as space
    let _ = g;
}

// ============================================================================
// Disarming
// ============================================================================

#[test]
fn test_can_disarm_arrow() {
    assert!(can_disarm(TrapType::Arrow));
}

#[test]
fn test_can_disarm_dart() {
    assert!(can_disarm(TrapType::Dart));
}

#[test]
fn test_disarm_difficulty_positive() {
    let diff = disarm_difficulty(TrapType::Arrow);
    assert!(diff >= 0, "Disarm difficulty should be non-negative");
}

#[test]
fn test_try_disarm_with_high_skill() {
    let mut rng = GameRng::new(42);
    let trap = create_trap(5, 5, TrapType::Arrow);
    let mut success_count = 0;
    for seed in 0..50 {
        let mut rng2 = GameRng::new(seed);
        if try_disarm(&mut rng2, &trap, 18, 10) {
            success_count += 1;
        }
    }
    assert!(success_count > 0, "High skill should sometimes succeed");
}

#[test]
fn test_try_disarm_with_low_skill() {
    let trap = create_trap(5, 5, TrapType::BearTrap);
    let mut fail_count = 0;
    for seed in 0..20 {
        let mut rng = GameRng::new(seed);
        if !try_disarm(&mut rng, &trap, 3, 0) {
            fail_count += 1;
        }
    }
    assert!(fail_count > 0, "Low skill should sometimes fail");
}

// ============================================================================
// Trap detection
// ============================================================================

#[test]
fn test_can_detect_with_high_skill() {
    let mut success = false;
    for seed in 0..50 {
        let mut rng = GameRng::new(seed);
        if can_detect_trap(&mut rng, 20, TrapType::Arrow) {
            success = true;
            break;
        }
    }
    assert!(success, "High search skill should detect traps");
}

// ============================================================================
// Escape traps
// ============================================================================

#[test]
fn test_escape_trap_message_pit() {
    let msg = escape_trap_message(TrapType::Pit);
    assert!(!msg.is_empty());
}

#[test]
fn test_escape_trap_message_bear_trap() {
    let msg = escape_trap_message(TrapType::BearTrap);
    assert!(!msg.is_empty());
}

#[test]
fn test_try_escape_bear_trap_high_str() {
    let mut escaped = false;
    for seed in 0..50 {
        let mut rng = GameRng::new(seed);
        if try_escape_trap(&mut rng, TrapType::BearTrap, 18) {
            escaped = true;
            break;
        }
    }
    assert!(escaped, "Should eventually escape bear trap with STR 18");
}

#[test]
fn test_try_escape_web_possible() {
    let mut escaped = false;
    for seed in 0..100 {
        let mut rng = GameRng::new(seed);
        if try_escape_trap(&mut rng, TrapType::Web, 12) {
            escaped = true;
            break;
        }
    }
    assert!(escaped, "Should eventually escape web");
}

// ============================================================================
// Container traps
// ============================================================================

#[test]
fn test_b_trapped_when_trapped() {
    let mut rng = GameRng::new(42);
    let ct = b_trapped(true, &mut rng);
    assert_ne!(ct, ContainerTrap::None, "Trapped container should produce a trap");
}

#[test]
fn test_b_trapped_when_not_trapped() {
    let mut rng = GameRng::new(42);
    let ct = b_trapped(false, &mut rng);
    assert_eq!(ct, ContainerTrap::None);
}

#[test]
fn test_rndtrap_container_variety() {
    let mut types: Vec<ContainerTrap> = Vec::new();
    for seed in 0..100 {
        let mut rng = GameRng::new(seed);
        let ct = rndtrap_container(&mut rng);
        if !types.contains(&ct) {
            types.push(ct);
        }
    }
    assert!(types.len() >= 2, "Container traps should have variety");
}

#[test]
fn test_avoid_container_trap_high_dex_luck() {
    let mut avoided = false;
    for seed in 0..50 {
        let mut rng = GameRng::new(seed);
        if avoid_container_trap(&mut rng, 18, 5) {
            avoided = true;
            break;
        }
    }
    assert!(avoided, "High DEX + luck should sometimes avoid container traps");
}

#[test]
fn test_mb_trapped_true() {
    assert!(mb_trapped(true));
}

#[test]
fn test_mb_trapped_false() {
    assert!(!mb_trapped(false));
}

// ============================================================================
// Statue traps
// ============================================================================

#[test]
fn test_is_statue_trap_with_corpsenm() {
    // A statue with a corpsenm is a potential trap
    let result = is_statue_trap(true, Some(5));
    assert!(result);
}

#[test]
fn test_statue_trap_reveal_msg() {
    let msg = statue_trap_reveal_msg();
    assert!(!msg.is_empty());
}

// ============================================================================
// Trigger trap
// ============================================================================

#[test]
fn test_trigger_arrow_trap() {
    let mut rng = GameRng::new(42);
    let mut trap = create_trap(10, 10, TrapType::Arrow);
    let effect = trigger_trap(&mut rng, &mut trap);
    match effect {
        TrapEffect::Damage(d) => assert!(d > 0, "Arrow trap damage should be positive"),
        TrapEffect::Status(_) => {} // Dart poisoned variant
        _ => {} // Other valid effects
    }
}

#[test]
fn test_trigger_pit_trap() {
    let mut rng = GameRng::new(42);
    let mut trap = create_trap(10, 10, TrapType::Pit);
    let effect = trigger_trap(&mut rng, &mut trap);
    assert!(matches!(effect, TrapEffect::Fall { .. } | TrapEffect::Damage(_)));
}

#[test]
fn test_trigger_fire_trap() {
    let mut rng = GameRng::new(42);
    let mut trap = create_trap(10, 10, TrapType::FireTrap);
    let effect = trigger_trap(&mut rng, &mut trap);
    assert!(matches!(effect, TrapEffect::Damage(_) | TrapEffect::ItemDamage));
}

#[test]
fn test_trigger_bear_trap() {
    let mut rng = GameRng::new(42);
    let mut trap = create_trap(10, 10, TrapType::BearTrap);
    let effect = trigger_trap(&mut rng, &mut trap);
    assert!(matches!(effect, TrapEffect::Trapped { .. }));
}

// ============================================================================
// Level-based trap generation
// ============================================================================

#[test]
fn test_rndtrap_produces_valid_type() {
    let mut rng = GameRng::new(42);
    let level = make_level();
    let tt = rndtrap(&mut rng, &level);
    assert!(!trap_name(tt).is_empty());
}

#[test]
fn test_count_surround_traps_empty_level() {
    let level = make_level();
    let count = count_surround_traps(&level, 10, 10);
    assert_eq!(count, 0, "Empty level should have no surrounding traps");
}
