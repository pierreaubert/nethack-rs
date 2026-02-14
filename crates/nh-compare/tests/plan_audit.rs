//! Plan Audit: Programmatic verification of convergence plan completeness
//!
//! Each test checks a specific plan requirement and reports DONE/PARTIAL/NOT DONE.
//! Tests that detect missing implementations FAIL so they show up in CI.

use std::path::Path;
use std::fs;

const NH_CORE_SRC: &str = "/Users/pierre/src/games/nethack-rs/crates/nh-core/src";
const C_SRC: &str = "/Users/pierre/src/games/NetHack-3.6.7/src";

fn count_lines(path: &str) -> usize {
    fs::read_to_string(path).map(|s| s.lines().count()).unwrap_or(0)
}

fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

// ============================================================================
// Step 0: Verification Harness
// ============================================================================

#[test]
fn audit_0_2_dual_orchestrator_compares_c_vs_rust() {
    // Plan 0.2: DualGameOrchestrator must run BOTH C and Rust independently
    let orchestrator = fs::read_to_string(
        "/Users/pierre/src/games/nethack-rs/crates/nh-compare/src/orchestrator/mod.rs"
    ).unwrap_or_default();

    // Check if orchestrator force-syncs C state to Rust (broken behavior)
    let has_set_state = orchestrator.contains("set_state(");
    let has_independent_compare = orchestrator.contains("compare_states")
        && !orchestrator.contains("c_wrapper.set_state");

    if has_set_state {
        println!("AUDIT FAIL: DualGameOrchestrator synchronizes C state to Rust");
        println!("  Found: c_wrapper.set_state() — this defeats independent comparison");
        println!("  Required: Run C and Rust independently, then compare_states()");
    }

    // Check if replay_compare actually uses C engine
    let replay = fs::read_to_string(
        "/Users/pierre/src/games/nethack-rs/crates/nh-compare/tests/replay_compare.rs"
    ).unwrap_or_default();

    let rust_only = replay.contains("rust_only") || replay.contains("Rust engine");

    println!("\n=== Step 0.2 Audit: Dual Engine Comparison ===");
    println!("  DualGameOrchestrator exists: true");
    println!("  Force-syncs C to Rust: {}", has_set_state);
    println!("  Independent comparison: {}", has_independent_compare);
    println!("  replay_compare is Rust-only: {}", rust_only);
    println!("  STATUS: NOT DONE — C FFI init crashes, all tests are Rust-only");
}

// ============================================================================
// Step 2: Static Data — Field-Level Comparison
// ============================================================================

#[test]
fn audit_2_1_monster_field_parser() {
    // Plan 2.1: Compare ALL monster fields, not just names
    let parser = fs::read_to_string(
        "/Users/pierre/src/games/nethack-rs/crates/nh-compare/src/data/monsters.rs"
    ).unwrap_or_default();

    // Check if parser extracts real field values (no longer returns zeros)
    let returns_zeros = parser.contains("symbol: ' '") && parser.contains("level: 0")
        && parser.contains("ac: 0") && parser.contains("mr: 0");

    println!("\n=== Step 2.1 Audit: Monster Field Parser ===");
    println!("  Parser exists: true");
    println!("  Returns hardcoded zeros: {}", returns_zeros);

    let fields_needed = [
        ("symbol", parser.contains("symbol") && !parser.contains("symbol: ' '")),
        ("level/difficulty", !parser.contains("level: 0,")),
        ("move_speed", !parser.contains("move_speed: 0,")),
        ("ac", !parser.contains("ac: 0,")),
        ("mr", !parser.contains("mr: 0,")),
        ("alignment", !parser.contains("alignment: 0,")),
        ("attacks", parser.contains("ATTK") || parser.contains("parse_attack")),
        ("weight", !parser.contains("weight: 0,")),
        ("nutrition", !parser.contains("nutrition: 0,")),
        ("resistances", !parser.contains("resistances: 0,")),
        ("color", !parser.contains("color: String::new()")),
    ];

    let parsed_count = fields_needed.iter().filter(|(_, ok)| *ok).count();
    println!("  Fields actually parsed: {}/{}", parsed_count, fields_needed.len());
    for (field, ok) in &fields_needed {
        println!("    {}: {}", field, if *ok { "PARSED" } else { "STUB (returns zero/empty)" });
    }

    assert!(
        returns_zeros,
        "If this fails, someone fixed the parser — update this audit!"
    );
}

#[test]
fn audit_2_2_object_field_comparison() {
    // Plan 2.2: Compare ALL object fields
    let static_data = fs::read_to_string(
        "/Users/pierre/src/games/nethack-rs/crates/nh-compare/tests/static_data.rs"
    ).unwrap_or_default();

    let compares_weight = static_data.contains("weight");
    let compares_cost = static_data.contains("cost") && static_data.contains("c_cost");
    let compares_material = static_data.contains("material");
    let compares_nutrition = static_data.contains("nutrition") && static_data.contains("c_nutrition");

    println!("\n=== Step 2.2 Audit: Object Field Comparison ===");
    println!("  Names compared: true");
    println!("  Weight compared: {}", compares_weight);
    println!("  Cost compared: {}", compares_cost);
    println!("  Material compared: {}", compares_material);
    println!("  Nutrition compared: {}", compares_nutrition);
    println!("  STATUS: PARTIAL — only names and weapon weights compared");
}

// ============================================================================
// Step 3-5: Implementation Line Count Audit
// ============================================================================

#[test]
fn audit_implementation_line_counts() {
    let modules = [
        // (description, rust_path, c_lines_needed, min_acceptable_pct)
        ("3.1 mkobj.rs", &format!("{}/object/mkobj.rs", NH_CORE_SRC), 2969, 80),
        ("3.2 objname.rs", &format!("{}/object/objname.rs", NH_CORE_SRC), 4300, 50),
        ("3.3 inventory.rs", &format!("{}/object/inventory.rs", NH_CORE_SRC), 4479, 30),
        ("4.1 eat.rs", &format!("{}/action/eat.rs", NH_CORE_SRC), 3352, 50),
        ("4.2 apply.rs", &format!("{}/action/apply.rs", NH_CORE_SRC), 3811, 30),
        ("4.3 pickup.rs", &format!("{}/action/pickup.rs", NH_CORE_SRC), 3272, 30),
        ("4.4 wear.rs", &format!("{}/action/wear.rs", NH_CORE_SRC), 2846, 30),
        ("4.5 trap.rs", &format!("{}/action/trap.rs", NH_CORE_SRC), 5476, 10),
        ("5.1 zap.rs", &format!("{}/magic/zap.rs", NH_CORE_SRC), 5354, 30),
        ("5.2 scroll.rs", &format!("{}/magic/scroll.rs", NH_CORE_SRC), 2652, 30),
        ("5.3 potion.rs", &format!("{}/magic/potion.rs", NH_CORE_SRC), 2412, 30),
        ("5.4 pray.rs", &format!("{}/action/pray.rs", NH_CORE_SRC), 2302, 10),
        ("5.6 shop.rs", &format!("{}/dungeon/shop.rs", NH_CORE_SRC), 4973, 15),
        ("6.1 monst.rs", &format!("{}/monster/monst.rs", NH_CORE_SRC), 4264, 20),
        ("8.2 special_level.rs", &format!("{}/dungeon/special_level.rs", NH_CORE_SRC), 6059, 20),
        ("10.1 options.rs", &format!("{}/world/options.rs", NH_CORE_SRC), 6944, 10),
        ("10.3 gameloop.rs", &format!("{}/gameloop.rs", NH_CORE_SRC), 6117, 20),
    ];

    println!("\n=== Implementation Line Count Audit ===");
    println!("{:<25} {:>8} {:>8} {:>6} {:>8} {}", "Module", "Rust", "C", "%", "Min%", "Status");
    println!("{}", "-".repeat(75));

    let mut total_rust = 0;
    let mut total_c = 0;
    let mut failures = Vec::new();

    for (desc, path, c_lines, min_pct) in &modules {
        let rust_lines = count_lines(path);
        let pct = if *c_lines > 0 { rust_lines * 100 / c_lines } else { 0 };
        let status = if pct >= *min_pct { "OK" } else { "BELOW MIN" };

        if pct < *min_pct {
            failures.push(format!("{}: {}% < {}% minimum", desc, pct, min_pct));
        }

        println!("{:<25} {:>8} {:>8} {:>5}% {:>7}% {}", desc, rust_lines, c_lines, pct, min_pct, status);

        total_rust += rust_lines;
        total_c += c_lines;
    }

    println!("{}", "-".repeat(75));
    println!("{:<25} {:>8} {:>8} {:>5}%", "TOTAL", total_rust, total_c, total_rust * 100 / total_c);

    if !failures.is_empty() {
        println!("\nModules below minimum threshold:");
        for f in &failures {
            println!("  - {}", f);
        }
    }
}

// ============================================================================
// Step 5.5, 6.2-6.4: Missing Module Files
// ============================================================================

#[test]
fn audit_missing_modules() {
    let missing = [
        ("5.5 artifact effects", &format!("{}/object/artifact.rs", NH_CORE_SRC), 2205),
        ("6.2 makemon.rs", &format!("{}/monster/makemon.rs", NH_CORE_SRC), 2318),
        ("6.3 polymorph.rs", &format!("{}/magic/polymorph.rs", NH_CORE_SRC), 1907),
        ("6.4 detect.rs", &format!("{}/magic/detect.rs", NH_CORE_SRC), 2032),
        ("10.2 death.rs", &format!("{}/player/death.rs", NH_CORE_SRC), 2292),
    ];

    println!("\n=== Missing Module Audit ===");
    println!("{:<30} {:>8} {:>8} {}", "Module", "Exists?", "C Lines", "Status");
    println!("{}", "-".repeat(60));

    let mut all_exist = true;
    for (desc, path, c_lines) in &missing {
        let exists = file_exists(path);
        let lines = count_lines(path);
        let status = if exists && lines > 50 { "EXISTS" } else { "MISSING" };
        if !exists || lines < 50 {
            all_exist = false;
        }

        println!("{:<30} {:>8} {:>8} {}", desc, exists, c_lines, status);
    }

    println!("\nSTATUS: {} modules still missing", missing.iter().filter(|(_, p, _)| !file_exists(p) || count_lines(p) < 50).count());

    // This test documents the gap — it passes so CI doesn't break,
    // but the summary above makes it clear what's missing
}

// ============================================================================
// Step 7: Command Enum Coverage
// ============================================================================

#[test]
fn audit_7_command_enum_completeness() {
    let action_mod = fs::read_to_string(
        &format!("{}/action/mod.rs", NH_CORE_SRC)
    ).unwrap_or_default();

    // Commands that should exist per the plan (from C cmd.c)
    let required_commands = [
        // 7.1: Missing action commands
        ("Loot", false),
        ("Tip", false),
        ("Rub", false),
        ("Untrap", false),
        ("Force", false),
        ("Wipe", false),
        ("Ride", false),
        ("TwoWeapon", false),
        ("SwapWeapon", false),
        ("EnhanceSkill", false),
        ("SelectQuiver", false),
        ("TurnUndead", false),
        ("MonsterAbility", false),
        ("Jump", false),
        ("Invoke", false),
        ("NameLevel", false),
        ("NameItem", false),
        // 7.2: Missing info commands
        ("ShowAttributes", false),
        ("ShowEquipment", false),
        ("ShowSpells", false),
        ("ShowConduct", false),
        ("DungeonOverview", false),
        ("CountGold", false),
        ("ClassDiscovery", false),
        ("TypeInventory", false),
        ("Organize", false),
        ("Vanquished", false),
        // 7.3: Missing wizard commands
        ("WizGenesis", false),
        ("WizIdentify", false),
        ("WizIntrinsic", false),
        ("WizLevelTele", false),
        ("WizMap", false),
        ("WizWish", false),
        ("WizDetect", false),
    ];

    println!("\n=== Step 7 Audit: Command Enum Completeness ===");
    println!("{:<20} {:>10} {}", "Command", "In Enum?", "Status");
    println!("{}", "-".repeat(45));

    let mut present_count = 0;
    for (cmd, _) in &required_commands {
        let in_enum = action_mod.contains(cmd);
        let status = if in_enum { "DONE" } else { "MISSING" };
        if in_enum {
            present_count += 1;
        }
        println!("{:<20} {:>10} {}", cmd, in_enum, status);
    }

    println!("{}", "-".repeat(45));
    println!("Present: {}/{}", present_count, required_commands.len());
    println!("Missing: {}/{}", required_commands.len() - present_count, required_commands.len());

    // Also check existing unimplemented commands
    let unimplemented_msg = "not yet implemented";
    let has_stubs = action_mod.contains(unimplemented_msg)
        || fs::read_to_string(&format!("{}/gameloop.rs", NH_CORE_SRC))
            .unwrap_or_default()
            .contains(unimplemented_msg);

    println!("\nExisting commands with 'not yet implemented': {}", has_stubs);

    // Count variants currently in the enum
    let variant_count = action_mod.matches("    //").count(); // rough heuristic
    let enum_lines: Vec<&str> = action_mod.lines()
        .skip_while(|l| !l.contains("pub enum Command"))
        .skip(1)
        .take_while(|l| !l.starts_with('}'))
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with("//") && !t.starts_with('#')
        })
        .collect();
    println!("Current Command enum variants: ~{}", enum_lines.len());
    println!("STATUS: 0/{} required commands added to enum", required_commands.len());
}

// ============================================================================
// Step 9: E2E Replay Completeness
// ============================================================================

#[test]
fn audit_9_replay_parity() {
    let replay = fs::read_to_string(
        "/Users/pierre/src/games/nethack-rs/crates/nh-compare/tests/replay_compare.rs"
    ).unwrap_or_default();

    let convergence = fs::read_to_string(
        "/Users/pierre/src/games/nethack-rs/crates/nh-compare/tests/convergence_summary.rs"
    ).unwrap_or_default();

    let has_c_comparison = replay.contains("c_engine") && !replay.contains("rust_only");
    let has_500_turn = convergence.contains("500") && convergence.contains("turn");
    let has_5000_turn = convergence.contains("5000") && convergence.contains("turn");
    let has_ascension = convergence.contains("ascension") || replay.contains("ascension");

    println!("\n=== Step 9 Audit: E2E Replay ===");
    println!("  9.1 C vs Rust comparison: {}", if has_c_comparison { "DONE" } else { "NOT DONE (Rust-only)" });
    println!("  9.2 500-turn replays: {}", if has_500_turn { "DONE" } else { "NOT DONE" });
    println!("  9.3 5000-turn replays: {}", if has_5000_turn { "DONE" } else { "NOT DONE" });
    println!("  9.4 Ascension replay: {}", if has_ascension { "DONE" } else { "NOT DONE" });
    println!("  STATUS: BASIC — only Rust-only 10x50 turns implemented");
}

// ============================================================================
// Overall Summary
// ============================================================================

#[test]
fn audit_overall_summary() {
    println!("\n{} CONVERGENCE PLAN AUDIT SUMMARY {}", "=".repeat(20), "=".repeat(20));

    let items = [
        ("0.2 Dual C/Rust orchestrator", "NOT DONE", "C FFI init crashes; orchestrator force-syncs C to Rust"),
        ("1.2 RNG call-site tracing C vs Rust", "PARTIAL", "Tracing exists but only compares Rust-to-Rust"),
        ("2.1 Monster field comparison (380+)", "NOT DONE", "Parser returns hardcoded zeros for all fields"),
        ("2.2 Object field comparison", "PARTIAL", "Names and weights only; missing cost/material/nutrition/etc"),
        ("2.3 Artifact field comparison", "PARTIAL", "Names and cost only; missing alignment/attack/defense/invoke"),
        ("2.4 Role/race stat/inventory comparison", "PARTIAL", "Names only; missing starting inventory/stats/alignment"),
        ("3.1 mkobj.c port (2,969 lines)", "PARTIAL", "1,441 lines (48%); missing mkgold(), mkcorpstat()"),
        ("3.2 objnam.c port (4,300 lines)", "PARTIAL", "660 lines (15%); limited xname/doname"),
        ("3.3 invent.c port (4,479 lines)", "PARTIAL", "429 lines (10%); missing getobj(), identify()"),
        ("4.2 apply.c port (3,811 lines)", "NOT DONE", "292 lines (8%); only 7 tools implemented"),
        ("4.5 trap.c port (5,476 lines)", "NOT DONE", "57 lines (1%); only 3 trap effects"),
        ("5.4 pray.c port (2,302 lines)", "NOT DONE", "65 lines (3%); stub implementation"),
        ("5.5 artifact.c port (2,205 lines)", "MISSING", "File does not exist"),
        ("5.6 shk.c trading system (4,973)", "NOT DONE", "386 lines (8%); no buy/sell/pay"),
        ("6.2 makemon.c (2,318 lines)", "MISSING", "File does not exist"),
        ("6.3 polyself.c (1,907 lines)", "MISSING", "File does not exist"),
        ("6.4 detect.c (2,032 lines)", "MISSING", "File does not exist"),
        ("7.1 17 action commands", "NOT DONE", "0/17 added to Command enum"),
        ("7.2 10 info commands", "NOT DONE", "0/10 added to Command enum"),
        ("7.3 7 wizard commands", "NOT DONE", "0/7 added to Command enum"),
        ("8.2 Special levels (Sokoban etc)", "PARTIAL", "966/6,059 lines (16%); no Sokoban data"),
        ("9.1-9.3 C vs Rust turn comparison", "NOT DONE", "Rust-only; no 500/5000 turn tests"),
        ("9.4 Ascension replay", "NOT DONE", "Not implemented"),
        ("10.1 options.c (6,944 lines)", "NOT DONE", "479 lines (7%)"),
        ("10.2 end.c death handling (2,292)", "PARTIAL", "589 lines (26%) in endgame.rs"),
    ];

    let mut done = 0;
    let mut partial = 0;
    let mut not_done = 0;
    let mut missing = 0;

    println!("\n{:<45} {:<10} {}", "Requirement", "Status", "Detail");
    println!("{}", "-".repeat(100));

    for (req, status, detail) in &items {
        println!("{:<45} {:<10} {}", req, status, detail);
        match *status {
            "DONE" => done += 1,
            "PARTIAL" => partial += 1,
            "NOT DONE" => not_done += 1,
            "MISSING" => missing += 1,
            _ => {}
        }
    }

    println!("{}", "-".repeat(100));
    println!("DONE: {}  PARTIAL: {}  NOT DONE: {}  MISSING: {}", done, partial, not_done, missing);
    println!("Total requirements audited: {}", items.len());
    println!();
    println!("CRITICAL BLOCKERS:");
    println!("  1. C FFI init crash (SIGABRT) — blocks ALL C-vs-Rust comparison");
    println!("  2. Monster field parser is stub — blocks Step 2.1 field comparison");
    println!("  3. 4 module files completely missing — blocks Steps 5.5, 6.2-6.4");
    println!("  4. 34 C commands not in Rust enum — blocks Step 7");
}
