//! Unsynchronized drift tests — run both engines WITHOUT syncing state between turns.
//!
//! These tests let drift accumulate naturally, producing convergence reports
//! instead of panicking on first difference. This reveals systematic divergences
//! that synchronized tests mask.

use nh_compare::diff::{diff_snapshots, Severity};
use nh_compare::report::ConvergenceReport;
use nh_compare::snapshot::{GameSnapshot, ItemSnapshot, MonsterSnapshot, PlayerSnapshot};
use nh_core::action::Command;
use nh_core::player::{Attribute, Gender, Race, Role};
use nh_core::{GameLoop, GameRng, GameState};
use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use serial_test::serial;

/// Extract a snapshot from the Rust GameState.
fn rust_snapshot(gs: &GameState, turn: u64) -> GameSnapshot {
    let p = &gs.player;
    GameSnapshot {
        turn,
        player: PlayerSnapshot {
            x: p.pos.x as i32,
            y: p.pos.y as i32,
            hp: p.hp,
            hp_max: p.hp_max,
            energy: p.energy,
            energy_max: p.energy_max,
            armor_class: p.armor_class as i32,
            gold: p.gold,
            exp_level: p.exp_level,
            nutrition: p.nutrition,
            strength: p.attr_current.get(Attribute::Strength) as i32,
            intelligence: p.attr_current.get(Attribute::Intelligence) as i32,
            wisdom: p.attr_current.get(Attribute::Wisdom) as i32,
            dexterity: p.attr_current.get(Attribute::Dexterity) as i32,
            constitution: p.attr_current.get(Attribute::Constitution) as i32,
            charisma: p.attr_current.get(Attribute::Charisma) as i32,
            alive: p.hp > 0,
            dungeon_level: p.level.level_num as i32,
            dungeon_num: p.level.dungeon_num as i32,
            status_effects: collect_status_effects(p),
        },
        inventory: gs
            .inventory
            .iter()
            .map(|o| ItemSnapshot {
                object_type: o.object_type,
                class: format!("{:?}", o.class),
                quantity: o.quantity,
                enchantment: o.enchantment,
                buc: format!("{:?}", o.buc),
                weight: o.weight,
            })
            .collect(),
        monsters: gs
            .current_level
            .monsters
            .iter()
            .map(|m| MonsterSnapshot {
                monster_type: m.monster_type,
                x: m.x as i32,
                y: m.y as i32,
                hp: m.hp,
                hp_max: m.hp_max,
                peaceful: m.state.peaceful,
                sleeping: m.state.sleeping,
                alive: m.state.alive,
            })
            .collect(),
        source: "rust".into(),
    }
}

fn collect_status_effects(p: &nh_core::player::You) -> Vec<String> {
    let mut effects = Vec::new();
    if p.confused_timeout > 0 {
        effects.push("confused".into());
    }
    if p.stunned_timeout > 0 {
        effects.push("stunned".into());
    }
    if p.blinded_timeout > 0 {
        effects.push("blind".into());
    }
    if p.hallucinating_timeout > 0 {
        effects.push("hallucinating".into());
    }
    effects
}

/// Extract a snapshot from the C engine via FFI subprocess.
fn c_snapshot(engine: &CGameEngine, turn: u64) -> GameSnapshot {
    let (x, y) = engine.position();
    let attrs_json: serde_json::Value =
        serde_json::from_str(&engine.attributes_json()).unwrap_or_default();

    let c_inv_str = engine.inventory_json();
    let c_inv: serde_json::Value = serde_json::from_str(&c_inv_str).unwrap_or_default();
    let inventory = c_inv
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|item| ItemSnapshot {
                    object_type: item["otyp"].as_i64().unwrap_or(0) as i16,
                    class: "Unknown".into(),
                    quantity: item["quantity"].as_i64().unwrap_or(1) as i32,
                    enchantment: item["enchantment"].as_i64().unwrap_or(0) as i8,
                    buc: match item["buc"].as_i64().unwrap_or(0) {
                        1 => "Blessed".into(),
                        -1 => "Cursed".into(),
                        _ => "Uncursed".into(),
                    },
                    weight: item["weight"].as_u64().unwrap_or(0) as u32,
                })
                .collect()
        })
        .unwrap_or_default();

    let c_mon_str = engine.monsters_json();
    let c_mons: serde_json::Value = serde_json::from_str(&c_mon_str).unwrap_or_default();
    let monsters = c_mons
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|m| MonsterSnapshot {
                    monster_type: m["mnum"].as_i64().unwrap_or(-1) as i16,
                    x: m["x"].as_i64().unwrap_or(0) as i32,
                    y: m["y"].as_i64().unwrap_or(0) as i32,
                    hp: m["hp"].as_i64().unwrap_or(0) as i32,
                    hp_max: m["hp_max"].as_i64().unwrap_or(0) as i32,
                    peaceful: m["peaceful"].as_i64().unwrap_or(0) != 0,
                    sleeping: m["asleep"].as_i64().unwrap_or(0) != 0,
                    alive: true,
                })
                .collect()
        })
        .unwrap_or_default();

    GameSnapshot {
        turn,
        player: PlayerSnapshot {
            x,
            y,
            hp: engine.hp(),
            hp_max: engine.max_hp(),
            energy: engine.energy(),
            energy_max: engine.max_energy(),
            armor_class: engine.armor_class(),
            gold: engine.gold(),
            exp_level: engine.experience_level(),
            nutrition: engine.nutrition(),
            strength: attrs_json["str"].as_i64().unwrap_or(0) as i32,
            intelligence: attrs_json["int"].as_i64().unwrap_or(0) as i32,
            wisdom: attrs_json["wis"].as_i64().unwrap_or(0) as i32,
            dexterity: attrs_json["dex"].as_i64().unwrap_or(0) as i32,
            constitution: attrs_json["con"].as_i64().unwrap_or(0) as i32,
            charisma: attrs_json["cha"].as_i64().unwrap_or(0) as i32,
            alive: !engine.is_dead(),
            dungeon_level: engine.current_level(),
            dungeon_num: 0, // C FFI doesn't expose dnum separately
            status_effects: vec![], // C FFI doesn't expose these yet
        },
        inventory,
        monsters,
        source: "c".into(),
    }
}

// ============================================================================
// Test 3.1: Rest-only unsynchronized drift
// ============================================================================

/// Run both engines for N turns of resting with NO state synchronization.
/// This test doesn't need matching levels — both players just rest in place.
/// Reports drift in HP regen, hunger, and turn counter.
#[test]
#[serial]
fn test_unsynchronized_rest_drift_1000_turns() {
    let seed = 42u64;
    let role = Role::Valkyrie;
    let race = Race::Human;
    let gender = Gender::Female;
    let num_turns = 1000;

    // Initialize C engine
    let mut c_engine = CGameEngine::new();
    c_engine
        .init("Valkyrie", "Human", 1, 0)
        .expect("C engine init failed");
    c_engine.reset(seed).expect("C engine reset failed");

    // Initialize Rust engine — match C starting position
    let (cx, cy) = c_engine.position();
    let rust_rng = GameRng::new(seed);
    let mut rust_state = GameState::new_with_identity(
        rust_rng,
        "Hero".into(),
        role,
        race,
        gender,
        role.default_alignment(),
    );
    rust_state.player.pos.x = cx as i8;
    rust_state.player.pos.y = cy as i8;
    let mut rust_loop = GameLoop::new(rust_state);

    let mut report = ConvergenceReport::new(
        format!("rest-only unsync seed={} turns={}", seed, num_turns),
        seed,
    );

    let mut first_critical = false;

    for turn in 0..num_turns {
        // Execute rest in both engines — NO sync_stats_to_c!
        rust_loop.tick(Command::Rest);
        c_engine.exec_cmd('.').expect("C rest failed");

        // Snapshot every 10 turns (and turn 0, 1 for early drift)
        if turn < 5 || turn % 10 == 0 || turn == num_turns - 1 {
            let rs = rust_snapshot(rust_loop.state(), turn as u64);
            let cs = c_snapshot(&c_engine, turn as u64);
            let diffs = diff_snapshots(&rs, &cs);

            if !diffs.is_empty() {
                let has_critical = diffs.iter().any(|d| d.severity == Severity::Critical);
                report.add_turn(turn as u64, diffs);

                // Stop on first critical diff — position divergence makes
                // all subsequent comparison meaningless
                if has_critical && !first_critical {
                    first_critical = true;
                    println!(
                        "First critical divergence at turn {}. Continuing to collect data.",
                        turn
                    );
                }
            }
        }
    }

    report.print_summary();

    // Write JSON report to data directory
    let report_json = report.to_json();
    let report_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    std::fs::create_dir_all(&report_dir).ok();
    let report_path = report_dir.join(format!(
        "drift_rest_seed{}_{}turns.json",
        seed, num_turns
    ));
    std::fs::write(&report_path, &report_json).ok();
    println!("Report written to {}", report_path.display());

    // The test doesn't fail on diffs — it reports them.
    // CI gate tests (Phase 5) will enforce thresholds.
    println!(
        "Rest-only drift summary: {} critical, {} major, {} minor diffs across {} turns",
        report.critical_count, report.major_count, report.minor_count, num_turns
    );
}

/// Multi-seed rest drift — run across several seeds for broader coverage.
#[test]
#[serial]
fn test_unsynchronized_rest_drift_multi_seed() {
    let seeds = [1u64, 42, 12345, 99999, 314159];
    let num_turns = 200;
    let role = Role::Valkyrie;
    let race = Race::Human;
    let gender = Gender::Female;

    let mut total_critical = 0u64;
    let mut total_major = 0u64;
    let mut total_minor = 0u64;

    for &seed in &seeds {
        let mut c_engine = CGameEngine::new();
        c_engine
            .init("Valkyrie", "Human", 1, 0)
            .expect("C engine init failed");
        c_engine.reset(seed).expect("C engine reset failed");

        let (cx, cy) = c_engine.position();
        let rust_rng = GameRng::new(seed);
        let mut rust_state = GameState::new_with_identity(
            rust_rng,
            "Hero".into(),
            role,
            race,
            gender,
            role.default_alignment(),
        );
        rust_state.player.pos.x = cx as i8;
        rust_state.player.pos.y = cy as i8;
        let mut rust_loop = GameLoop::new(rust_state);

        let mut report = ConvergenceReport::new(
            format!("rest-only unsync seed={} turns={}", seed, num_turns),
            seed,
        );

        for turn in 0..num_turns {
            rust_loop.tick(Command::Rest);
            c_engine.exec_cmd('.').expect("C rest failed");

            // Snapshot every 50 turns
            if turn % 50 == 0 || turn == num_turns - 1 {
                let rs = rust_snapshot(rust_loop.state(), turn as u64);
                let cs = c_snapshot(&c_engine, turn as u64);
                let diffs = diff_snapshots(&rs, &cs);
                report.add_turn(turn as u64, diffs);
            }
        }

        println!(
            "Seed {}: {} critical, {} major, {} minor",
            seed, report.critical_count, report.major_count, report.minor_count
        );

        total_critical += report.critical_count;
        total_major += report.major_count;
        total_minor += report.minor_count;
    }

    println!(
        "\nMulti-seed total: {} critical, {} major, {} minor across {} seeds x {} turns",
        total_critical,
        total_major,
        total_minor,
        seeds.len(),
        num_turns
    );
}
