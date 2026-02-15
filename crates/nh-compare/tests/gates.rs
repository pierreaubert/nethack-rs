//! Phase Gate Tests: One ignored test per convergence phase.
//!
//! Un-ignore each test as its phase completes. Run with:
//!   cargo test -p nh-compare --test phase_gates -- --include-ignored --nocapture

use std::fs;
use std::path::Path;

const NH_CORE_SRC: &str = "/Users/pierre/src/games/nethack-rs/crates/nh-core/src";

fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

fn count_lines(path: &str) -> usize {
    fs::read_to_string(path)
        .map(|s| s.lines().count())
        .unwrap_or(0)
}

// ============================================================================
// Phase 0: Verification Infrastructure
// ============================================================================

#[test]
fn test_phase0_verification_infra() {
    // 0.1: Registry exists and has entries
    let registry_path =
        "/Users/pierre/src/games/nethack-rs/crates/nh-compare/data/c_function_registry.json";
    assert!(
        file_exists(registry_path),
        "Phase 0.1: c_function_registry.json missing"
    );

    let registry_data = fs::read_to_string(registry_path).unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&registry_data).unwrap();
    assert!(
        entries.len() > 1000,
        "Phase 0.1: registry has too few entries ({})",
        entries.len()
    );

    // 0.2: extensions feature gate compiles (checked externally via cargo check)

    // 0.3: This file exists (self-referential)

    // 0.4: stub_audit.rs exists
    assert!(
        file_exists(
            "/Users/pierre/src/games/nethack-rs/crates/nh-compare/tests/stub_audit.rs"
        ),
        "Phase 0.4: stub_audit.rs missing"
    );

    println!("Phase 0: PASSED");
}

// ============================================================================
// Phase 1: Deepen Eating, Wearing, Applying
// ============================================================================

#[test]
#[ignore]
fn test_phase1_eat_wear_apply_deepened() {
    // eat.rs should reach ~70% of eat.c (3,352 lines → ~2,300 Rust lines)
    let eat_lines = count_lines(&format!("{}/action/eat.rs", NH_CORE_SRC));
    assert!(
        eat_lines >= 2000,
        "Phase 1.1: eat.rs has {} lines, need ~2000+",
        eat_lines
    );

    // wear.rs should reach ~70% of do_wear.c (2,846 lines → ~2,000 Rust lines)
    let wear_lines = count_lines(&format!("{}/action/wear.rs", NH_CORE_SRC));
    assert!(
        wear_lines >= 1800,
        "Phase 1.2: wear.rs has {} lines, need ~1800+",
        wear_lines
    );

    // apply.rs should reach ~85%
    let apply_lines = count_lines(&format!("{}/action/apply.rs", NH_CORE_SRC));
    assert!(
        apply_lines >= 3200,
        "Phase 1.3: apply.rs has {} lines, need ~3200+",
        apply_lines
    );

    println!("Phase 1: PASSED — eat/wear/apply deepened");
}

// ============================================================================
// Phase 2: Deepen Reading, Potions, Zapping
// ============================================================================

#[test]
#[ignore]
fn test_phase2_magic_items_deepened() {
    // read.rs + scroll.rs combined should reach ~60% of read.c
    let read_lines = count_lines(&format!("{}/action/read.rs", NH_CORE_SRC));
    let scroll_lines = count_lines(&format!("{}/magic/scroll.rs", NH_CORE_SRC));
    assert!(
        read_lines + scroll_lines >= 1500,
        "Phase 2.1: read+scroll has {} lines, need ~1500+",
        read_lines + scroll_lines
    );

    // potion.rs + quaff.rs combined should reach ~60% of potion.c
    let potion_lines = count_lines(&format!("{}/magic/potion.rs", NH_CORE_SRC));
    let quaff_lines = count_lines(&format!("{}/action/quaff.rs", NH_CORE_SRC));
    assert!(
        potion_lines + quaff_lines >= 1700,
        "Phase 2.2: potion+quaff has {} lines, need ~1700+",
        potion_lines + quaff_lines
    );

    // zap.rs (magic) + zap.rs (action) should reach ~70% of zap.c
    let zap_magic_lines = count_lines(&format!("{}/magic/zap.rs", NH_CORE_SRC));
    let zap_action_lines = count_lines(&format!("{}/action/zap.rs", NH_CORE_SRC));
    assert!(
        zap_magic_lines + zap_action_lines >= 4000,
        "Phase 2.3: zap combined has {} lines, need ~4000+",
        zap_magic_lines + zap_action_lines
    );

    println!("Phase 2: PASSED — magic items deepened");
}

// ============================================================================
// Phase 3: Trap System Deep Port
// ============================================================================

#[test]
#[ignore]
fn test_phase3_traps_complete() {
    let trap_action = count_lines(&format!("{}/action/trap.rs", NH_CORE_SRC));
    let trap_dungeon = count_lines(&format!("{}/dungeon/trap.rs", NH_CORE_SRC));
    assert!(
        trap_action + trap_dungeon >= 3800,
        "Phase 3: trap combined has {} lines, need ~3800+",
        trap_action + trap_dungeon
    );

    println!("Phase 3: PASSED — trap system complete");
}

// ============================================================================
// Phase 4: Monster AI Convergence
// ============================================================================

#[test]
#[ignore]
fn test_phase4_ai_no_todos() {
    let ai_content = fs::read_to_string(format!("{}/monster/ai.rs", NH_CORE_SRC)).unwrap();
    let todo_count = ai_content.matches("TODO").count();
    assert!(
        todo_count == 0,
        "Phase 4: ai.rs still has {} TODOs, need 0",
        todo_count
    );

    println!("Phase 4: PASSED — monster AI TODOs resolved");
}

// ============================================================================
// Phase 5: Inventory Deep Port
// ============================================================================

#[test]
#[ignore]
fn test_phase5_inventory_complete() {
    let inv_lines = count_lines(&format!("{}/object/inventory.rs", NH_CORE_SRC));
    assert!(
        inv_lines >= 2000,
        "Phase 5: inventory.rs has {} lines, need ~2000+",
        inv_lines
    );

    // Check for key functions
    let content = fs::read_to_string(format!("{}/object/inventory.rs", NH_CORE_SRC)).unwrap();
    assert!(
        content.contains("fn getobj") || content.contains("fn get_obj"),
        "Phase 5: getobj() not found in inventory.rs"
    );

    println!("Phase 5: PASSED — inventory deepened");
}

// ============================================================================
// Phase 6: Movement & Terrain
// ============================================================================

#[test]
#[ignore]
fn test_phase6_movement_complete() {
    let movement_path = format!("{}/action/movement.rs", NH_CORE_SRC);
    assert!(
        file_exists(&movement_path),
        "Phase 6: action/movement.rs missing"
    );
    let movement_lines = count_lines(&movement_path);
    assert!(
        movement_lines >= 800,
        "Phase 6: movement.rs has {} lines, need ~800+",
        movement_lines
    );

    println!("Phase 6: PASSED — movement complete");
}

// ============================================================================
// Phase 7: Weapons & Skills
// ============================================================================

#[test]
#[ignore]
fn test_phase7_weapons_skills() {
    let weapon_path = format!("{}/combat/weapon.rs", NH_CORE_SRC);
    assert!(
        file_exists(&weapon_path),
        "Phase 7: combat/weapon.rs missing"
    );

    println!("Phase 7: PASSED — weapons & skills complete");
}

// ============================================================================
// Phase 8: Shop System
// ============================================================================

#[test]
#[ignore]
fn test_phase8_shops() {
    let shk_lines = count_lines(&format!("{}/special/shk.rs", NH_CORE_SRC));
    assert!(
        shk_lines >= 2200,
        "Phase 8: shk.rs has {} lines, need ~2200+",
        shk_lines
    );

    let content = fs::read_to_string(format!("{}/special/shk.rs", NH_CORE_SRC)).unwrap();
    assert!(
        content.contains("fn dopay") || content.contains("fn do_pay"),
        "Phase 8: dopay() not found in shk.rs"
    );

    println!("Phase 8: PASSED — shops complete");
}

// ============================================================================
// Phase 9: Prayer Deep Port
// ============================================================================

#[test]
#[ignore]
fn test_phase9_prayer() {
    let pray_lines = count_lines(&format!("{}/action/pray.rs", NH_CORE_SRC));
    assert!(
        pray_lines >= 2000,
        "Phase 9: pray.rs has {} lines, need ~2000+",
        pray_lines
    );

    let content = fs::read_to_string(format!("{}/action/pray.rs", NH_CORE_SRC)).unwrap();
    assert!(
        content.contains("fn dosacrifice") || content.contains("fn do_sacrifice"),
        "Phase 9: dosacrifice() not found in pray.rs"
    );

    println!("Phase 9: PASSED — prayer complete");
}

// ============================================================================
// Phase 10: Player Initialization
// ============================================================================

#[test]
#[ignore]
fn test_phase10_player_init() {
    let init_path = format!("{}/player/init.rs", NH_CORE_SRC);
    assert!(
        file_exists(&init_path),
        "Phase 10: player/init.rs missing"
    );
    let init_lines = count_lines(&init_path);
    assert!(
        init_lines >= 300,
        "Phase 10: init.rs has {} lines, need ~300+",
        init_lines
    );

    println!("Phase 10: PASSED — player init complete");
}

// ============================================================================
// Phase 11: Monster Lifecycle
// ============================================================================

#[test]
#[ignore]
fn test_phase11_monster_lifecycle() {
    let makemon_lines = count_lines(&format!("{}/monster/makemon.rs", NH_CORE_SRC));
    assert!(
        makemon_lines >= 1500,
        "Phase 11: makemon.rs has {} lines, need ~1500+",
        makemon_lines
    );

    let lifecycle_path = format!("{}/monster/lifecycle.rs", NH_CORE_SRC);
    assert!(
        file_exists(&lifecycle_path),
        "Phase 11: monster/lifecycle.rs missing"
    );

    println!("Phase 11: PASSED — monster lifecycle complete");
}

// ============================================================================
// Phase 12: Death & Game End
// ============================================================================

#[test]
#[ignore]
fn test_phase12_death() {
    let death_path = format!("{}/player/death.rs", NH_CORE_SRC);
    assert!(
        file_exists(&death_path),
        "Phase 12: player/death.rs missing"
    );
    let death_lines = count_lines(&death_path);
    assert!(
        death_lines >= 500,
        "Phase 12: death.rs has {} lines, need ~500+",
        death_lines
    );

    println!("Phase 12: PASSED — death system complete");
}

// ============================================================================
// Phase 13: Level Change
// ============================================================================

#[test]
#[ignore]
fn test_phase13_level_change() {
    let lc_path = format!("{}/action/level_change.rs", NH_CORE_SRC);
    assert!(
        file_exists(&lc_path),
        "Phase 13: action/level_change.rs missing"
    );
    let lc_lines = count_lines(&lc_path);
    assert!(
        lc_lines >= 400,
        "Phase 13: level_change.rs has {} lines, need ~400+",
        lc_lines
    );

    println!("Phase 13: PASSED — level change complete");
}

// ============================================================================
// Phase 14: Lock Picking
// ============================================================================

#[test]
#[ignore]
fn test_phase14_locks() {
    let lock_path = format!("{}/action/lock.rs", NH_CORE_SRC);
    assert!(
        file_exists(&lock_path),
        "Phase 14: action/lock.rs missing"
    );
    let lock_lines = count_lines(&lock_path);
    assert!(
        lock_lines >= 250,
        "Phase 14: lock.rs has {} lines, need ~250+",
        lock_lines
    );

    println!("Phase 14: PASSED — lock picking complete");
}

// ============================================================================
// Phase 15: Naming System
// ============================================================================

#[test]
#[ignore]
fn test_phase15_naming() {
    let name_path = format!("{}/action/name.rs", NH_CORE_SRC);
    assert!(
        file_exists(&name_path),
        "Phase 15: action/name.rs missing"
    );

    println!("Phase 15: PASSED — naming complete");
}

// ============================================================================
// Phase 16: Digging System
// ============================================================================

#[test]
#[ignore]
fn test_phase16_digging() {
    let dig_path = format!("{}/action/dig.rs", NH_CORE_SRC);
    assert!(
        file_exists(&dig_path),
        "Phase 16: action/dig.rs missing"
    );
    let dig_lines = count_lines(&dig_path);
    assert!(
        dig_lines >= 400,
        "Phase 16: dig.rs has {} lines, need ~400+",
        dig_lines
    );

    println!("Phase 16: PASSED — digging complete");
}

// ============================================================================
// Phase 17: Missing Commands
// ============================================================================

#[test]
#[ignore]
fn test_phase17_commands() {
    let mod_content =
        fs::read_to_string(format!("{}/action/mod.rs", NH_CORE_SRC)).unwrap();

    let required = [
        "Loot",
        "Untrap",
        "Force",
        "SwapWeapon",
        "SelectQuiver",
        "TwoWeapon",
        "EnhanceSkill",
        "TurnUndead",
        "Jump",
        "Invoke",
        "Rub",
        "Tip",
        "Wipe",
        "Ride",
        "MonsterAbility",
        "ShowAttributes",
        "ShowEquipment",
        "ShowSpells",
        "ShowConduct",
        "DungeonOverview",
        "CountGold",
        "ClassDiscovery",
        "TypeInventory",
        "Organize",
        "Vanquished",
    ];

    let mut missing = Vec::new();
    for cmd in &required {
        if !mod_content.contains(cmd) {
            missing.push(*cmd);
        }
    }

    assert!(
        missing.is_empty(),
        "Phase 17: {} commands missing from enum: {:?}",
        missing.len(),
        missing
    );

    println!("Phase 17: PASSED — all tier 1+2 commands present");
}

// ============================================================================
// Phase 18: Remaining C Files
// ============================================================================

#[test]
#[ignore]
fn test_phase18_remaining_systems() {
    let required_files = [
        ("special/ball.rs", "ball & chain"),
        ("special/steed.rs", "steed/riding"),
        ("action/music.rs", "instruments"),
        ("monster/worm.rs", "worm segments"),
        ("dungeon/region.rs", "regions"),
        ("special/wizard.rs", "wizard AI"),
        ("monster/throw.rs", "monster ranged"),
        ("special/steal.rs", "stealing"),
    ];

    let mut missing = Vec::new();
    for (path, desc) in &required_files {
        let full = format!("{}/{}", NH_CORE_SRC, path);
        if !file_exists(&full) {
            missing.push(*desc);
        }
    }

    assert!(
        missing.is_empty(),
        "Phase 18: {} systems missing: {:?}",
        missing.len(),
        missing
    );

    println!("Phase 18: PASSED — remaining systems complete");
}

// ============================================================================
// Phase 19: Integration & Full Verification
// ============================================================================

#[test]
#[ignore]
fn test_phase19_integration() {
    // Check function registry for >95% ported
    let registry_data = fs::read_to_string(
        "/Users/pierre/src/games/nethack-rs/crates/nh-compare/data/c_function_registry.json",
    )
    .unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&registry_data).unwrap();

    let total = entries.len();
    let ported = entries
        .iter()
        .filter(|e| e["status"].as_str() == Some("ported"))
        .count();
    let not_needed = entries
        .iter()
        .filter(|e| e["status"].as_str() == Some("not_needed"))
        .count();

    let effective_total = total - not_needed;
    let pct = if effective_total > 0 {
        ported * 100 / effective_total
    } else {
        0
    };

    assert!(
        pct >= 95,
        "Phase 19: only {}% ported ({}/{}), need 95%+",
        pct,
        ported,
        effective_total
    );

    println!("Phase 19: PASSED — {}% function coverage", pct);
}
