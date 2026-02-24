//! CI convergence regression gate — fixed scenarios with thresholds.
//!
//! These tests run predetermined scenarios and fail if convergence
//! worsens beyond defined thresholds. As convergence improves,
//! thresholds should be ratcheted down.

use nh_compare::diff::diff_snapshots;
use nh_compare::report::ConvergenceReport;
use nh_compare::snapshot::{GameSnapshot, ItemSnapshot, MonsterSnapshot, PlayerSnapshot};
use nh_core::action::Command;
use nh_core::player::{Attribute, Gender, Race, Role};
use nh_core::{GameLoop, GameRng, GameState};
use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use serial_test::serial;

/// Thresholds for convergence gate.
struct ConvergenceThreshold {
    max_critical_diffs: u64,
    max_major_diffs: u64,
}

/// CI scenario definition.
struct CiScenario {
    label: &'static str,
    seed: u64,
    role: &'static str,
    rust_role: Role,
    race: Race,
    gender: Gender,
    num_turns: usize,
    commands: fn(usize) -> Command,
    threshold: ConvergenceThreshold,
}

fn rest_command(_turn: usize) -> Command {
    Command::Rest
}

/// Get all CI scenarios.
fn ci_scenarios() -> Vec<CiScenario> {
    vec![
        CiScenario {
            label: "rest-only-valkyrie-seed42",
            seed: 42,
            role: "Valkyrie",
            rust_role: Role::Valkyrie,
            race: Race::Human,
            gender: Gender::Female,
            num_turns: 200,
            commands: rest_command,
            threshold: ConvergenceThreshold {
                // Baseline (Feb 2026): critical=0, major=168
                // As convergence improves, ratchet these down.
                max_critical_diffs: 5,
                max_major_diffs: 210,
            },
        },
        CiScenario {
            label: "rest-only-wizard-seed1",
            seed: 1,
            role: "Wizard",
            rust_role: Role::Wizard,
            race: Race::Human,
            gender: Gender::Male,
            num_turns: 200,
            commands: rest_command,
            threshold: ConvergenceThreshold {
                // Baseline (Feb 2026): critical=0, major=315
                max_critical_diffs: 5,
                max_major_diffs: 380,
            },
        },
        CiScenario {
            label: "rest-only-rogue-seed12345",
            seed: 12345,
            role: "Rogue",
            rust_role: Role::Rogue,
            race: Race::Human,
            gender: Gender::Male,
            num_turns: 200,
            commands: rest_command,
            threshold: ConvergenceThreshold {
                // Baseline (Feb 2026): critical=0, major=231
                max_critical_diffs: 5,
                max_major_diffs: 280,
            },
        },
    ]
}

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
            status_effects: vec![],
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
            dungeon_num: 0,
            status_effects: vec![],
        },
        inventory,
        monsters,
        source: "c".into(),
    }
}

fn run_scenario(scenario: &CiScenario) -> ConvergenceReport {
    let mut c_engine = CGameEngine::new();
    c_engine
        .init(scenario.role, "Human", 0, 0)
        .expect("C engine init failed");
    c_engine
        .reset(scenario.seed)
        .expect("C engine reset failed");

    let (cx, cy) = c_engine.position();
    let rust_rng = GameRng::new(scenario.seed);
    let mut rust_state = GameState::new_with_identity(
        rust_rng,
        "Hero".into(),
        scenario.rust_role,
        scenario.race,
        scenario.gender,
        scenario.rust_role.default_alignment(),
    );
    rust_state.player.pos.x = cx as i8;
    rust_state.player.pos.y = cy as i8;
    // C FFI init places player at (0,0) on Stone — skip invariant checks
    // since this state doesn't have proper Rust-side level generation
    rust_state.skip_invariant_checks = true;
    let mut rust_loop = GameLoop::new(rust_state);

    let mut report = ConvergenceReport::new(scenario.label.to_string(), scenario.seed);

    for turn in 0..scenario.num_turns {
        let cmd = (scenario.commands)(turn);

        let c_cmd = match &cmd {
            Command::Rest => '.',
            _ => '.',
        };

        rust_loop.tick(cmd);
        c_engine.exec_cmd(c_cmd).expect("C command failed");

        // Snapshot every 10 turns
        if turn % 10 == 0 || turn == scenario.num_turns - 1 {
            let rs = rust_snapshot(rust_loop.state(), turn as u64);
            let cs = c_snapshot(&c_engine, turn as u64);
            let diffs = diff_snapshots(&rs, &cs);
            report.add_turn(turn as u64, diffs);
        }
    }

    report
}

/// CI gate test — runs all scenarios and enforces thresholds.
/// This is the test that should be run in CI to prevent convergence regressions.
#[test]
#[serial]
fn test_ci_convergence_gate() {
    let scenarios = ci_scenarios();
    let mut all_passed = true;
    let mut summary_lines = Vec::new();

    let history_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("convergence_history");
    std::fs::create_dir_all(&history_dir).ok();

    for scenario in &scenarios {
        let report = run_scenario(scenario);

        let passed_critical = report.critical_count <= scenario.threshold.max_critical_diffs;
        let passed_major = report.major_count <= scenario.threshold.max_major_diffs;
        let passed = passed_critical && passed_major;

        let status = if passed { "PASS" } else { "FAIL" };
        let line = format!(
            "  [{}] {}: critical={}/{}, major={}/{}",
            status,
            scenario.label,
            report.critical_count,
            scenario.threshold.max_critical_diffs,
            report.major_count,
            scenario.threshold.max_major_diffs,
        );
        println!("{}", line);
        summary_lines.push(line);

        if !passed {
            all_passed = false;
            report.print_summary();
        }

        // Write history entry
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let history_path = history_dir.join(format!(
            "{}_{}.json",
            scenario.label, timestamp
        ));
        std::fs::write(&history_path, report.to_json()).ok();
    }

    println!("\n=== CI Convergence Gate Summary ===");
    for line in &summary_lines {
        println!("{}", line);
    }
    println!(
        "Overall: {}",
        if all_passed { "PASS" } else { "FAIL" }
    );

    assert!(
        all_passed,
        "Convergence regression detected! See details above."
    );
}
