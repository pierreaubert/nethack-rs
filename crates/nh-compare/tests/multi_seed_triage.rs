//! Multi-Seed Triage Test (Phase 2 of Parity Strategy)
//!
//! Runs level generation across many seeds in both C and Rust,
//! automatically categorizes divergences by mismatch type,
//! and reports which functions to fix first for maximum impact.

use nh_core::{GameRng, COLNO, ROWNO};
use nh_core::dungeon::{DLevel, Level};
use nh_core::magic::genocide::MonsterVitals;
use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use serial_test::serial;
use serde_json::Value;
use std::collections::HashMap;

/// A single cell mismatch between C and Rust
#[derive(Debug, Clone)]
struct CellMismatch {
    x: usize,
    y: usize,
    rust_type: String,
    c_type: String,
}

/// Classification of a mismatch into a likely source function
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum MismatchCategory {
    /// Stone↔Corridor or SecretCorridor: dig_corridor/join issue
    CorridorPlacement,
    /// Door/SecretDoor mismatch: finddpos/dodoor issue
    DoorPlacement,
    /// Wall type mismatch: room carving issue
    WallType,
    /// Room/Stone: room placement or vault issue
    RoomPlacement,
    /// Other type mismatch
    Other(String),
}

impl std::fmt::Display for MismatchCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MismatchCategory::CorridorPlacement => write!(f, "corridor_placement"),
            MismatchCategory::DoorPlacement => write!(f, "door_placement"),
            MismatchCategory::WallType => write!(f, "wall_type"),
            MismatchCategory::RoomPlacement => write!(f, "room_placement"),
            MismatchCategory::Other(s) => write!(f, "other({})", s),
        }
    }
}

/// Results for a single seed
#[derive(Debug)]
struct SeedResult {
    seed: u64,
    total_mismatches: usize,
    room_count_match: bool,
    room_positions_match: bool,
    categories: HashMap<MismatchCategory, usize>,
    /// First few mismatches for debugging
    sample_mismatches: Vec<CellMismatch>,
}

fn classify_mismatch(rust_type: &str, c_type: &str) -> MismatchCategory {
    match (rust_type, c_type) {
        // Corridor-related mismatches
        ("Stone", "Corridor") | ("Corridor", "Stone") => MismatchCategory::CorridorPlacement,
        ("Stone", "SecretCorridor") | ("SecretCorridor", "Stone") => {
            MismatchCategory::CorridorPlacement
        }
        ("Corridor", "SecretCorridor") | ("SecretCorridor", "Corridor") => {
            MismatchCategory::CorridorPlacement
        }

        // Door-related mismatches
        ("Stone", "Door") | ("Door", "Stone") => MismatchCategory::DoorPlacement,
        ("Stone", "SecretDoor") | ("SecretDoor", "Stone") => MismatchCategory::DoorPlacement,
        ("Door", "SecretDoor") | ("SecretDoor", "Door") => MismatchCategory::DoorPlacement,
        ("HWall", "Door") | ("Door", "HWall") => MismatchCategory::DoorPlacement,
        ("VWall", "Door") | ("Door", "VWall") => MismatchCategory::DoorPlacement,
        ("HWall", "SecretDoor") | ("SecretDoor", "HWall") => MismatchCategory::DoorPlacement,
        ("VWall", "SecretDoor") | ("SecretDoor", "VWall") => MismatchCategory::DoorPlacement,
        ("Corridor", "Door") | ("Door", "Corridor") => MismatchCategory::DoorPlacement,

        // Wall type mismatches
        (r, c)
            if (r.contains("Wall") || r.contains("Corner"))
                && (c.contains("Wall") || c.contains("Corner")) =>
        {
            MismatchCategory::WallType
        }

        // Room placement
        ("Stone", "Room") | ("Room", "Stone") => MismatchCategory::RoomPlacement,

        // Everything else
        _ => MismatchCategory::Other(format!("{}↔{}", rust_type, c_type)),
    }
}

fn normalize_c_type(s: &str) -> String {
    match s {
        "SDoor" => "SecretDoor".to_string(),
        "SCorr" => "SecretCorridor".to_string(),
        other => other.to_string(),
    }
}

fn run_seed(seed: u64, c_engine: &mut CGameEngine) -> SeedResult {
    let monster_vitals = MonsterVitals::default();

    // C: set dlevel, reset RNG, generate
    c_engine.set_dlevel(0, 14);
    c_engine.reset_rng(seed).expect("C RNG reset failed");
    c_engine.generate_level().expect("C level generation failed");

    let c_map_str = c_engine.map_json();
    let c_map: Value = serde_json::from_str(&c_map_str).unwrap();
    let c_rooms = c_map["rooms"].as_array().expect("C rooms missing");

    // Rust: generate with fresh RNG
    let mut fresh_rng = GameRng::new(seed);
    let mut fresh_level = Level::new(DLevel::new(0, 14));
    nh_core::dungeon::generate_rooms_and_corridors(&mut fresh_level, &mut fresh_rng, &monster_vitals);

    // Filter out vaults
    let rs_rooms: Vec<_> = fresh_level
        .rooms
        .iter()
        .filter(|r| r.room_type != nh_core::dungeon::RoomType::Vault)
        .collect();
    let c_rooms_filtered: Vec<_> = c_rooms
        .iter()
        .filter(|r| r["type"].as_i64().unwrap_or(0) != 4)
        .collect();

    let room_count_match = rs_rooms.len() == c_rooms_filtered.len();

    // Check room positions match
    let room_positions_match = if room_count_match {
        rs_rooms.iter().zip(c_rooms_filtered.iter()).all(|(rs, c)| {
            let rs_lx = rs.x as i64;
            let rs_hx = (rs.x + rs.width - 1) as i64;
            let rs_ly = rs.y as i64;
            let rs_hy = (rs.y + rs.height - 1) as i64;
            rs_lx == c["lx"].as_i64().unwrap()
                && rs_hx == c["hx"].as_i64().unwrap()
                && rs_ly == c["ly"].as_i64().unwrap()
                && rs_hy == c["hy"].as_i64().unwrap()
        })
    } else {
        false
    };

    // Compare cells
    let c_cells = &c_map["cells"];
    let mut mismatches = Vec::new();
    let mut categories: HashMap<MismatchCategory, usize> = HashMap::new();

    for x in 0..COLNO {
        for y in 0..ROWNO {
            let c_cell = &c_cells[x][y];
            let c_typ = normalize_c_type(c_cell["type"].as_str().unwrap_or("Unknown"));
            let rs_typ = format!("{:?}", fresh_level.cells[x][y].typ);

            if rs_typ != c_typ {
                let category = classify_mismatch(&rs_typ, &c_typ);
                *categories.entry(category).or_insert(0) += 1;
                mismatches.push(CellMismatch {
                    x,
                    y,
                    rust_type: rs_typ,
                    c_type: c_typ,
                });
            }
        }
    }

    let total_mismatches = mismatches.len();
    let sample_mismatches = mismatches.into_iter().take(5).collect();

    SeedResult {
        seed,
        total_mismatches,
        room_count_match,
        room_positions_match,
        categories,
        sample_mismatches,
    }
}

#[test]
#[serial]
fn multi_seed_triage_100() {
    let seed_range = 1..=100;
    let mut c_engine = CGameEngine::new();
    c_engine
        .init("Valkyrie", "Human", 0, 0)
        .expect("C engine init failed");

    let mut all_results: Vec<SeedResult> = Vec::new();
    let mut perfect_seeds = 0u64;
    let mut room_mismatch_seeds = 0u64;
    let mut global_categories: HashMap<MismatchCategory, usize> = HashMap::new();
    let mut global_category_seeds: HashMap<MismatchCategory, u64> = HashMap::new();

    for seed in seed_range {
        let result = run_seed(seed, &mut c_engine);

        if result.total_mismatches == 0 {
            perfect_seeds += 1;
            println!("  PERFECT: seed {}", seed);
        }
        if !result.room_count_match {
            room_mismatch_seeds += 1;
        }

        for (cat, count) in &result.categories {
            *global_categories.entry(cat.clone()).or_insert(0) += count;
            *global_category_seeds.entry(cat.clone()).or_insert(0) += 1;
        }

        all_results.push(result);
    }

    // === REPORT ===
    println!("\n========================================");
    println!("  MULTI-SEED TRIAGE REPORT (seeds 1-100)");
    println!("========================================\n");

    println!("Perfect seeds (0 mismatches): {}/100", perfect_seeds);
    println!(
        "Seeds with room count mismatch: {}/100",
        room_mismatch_seeds
    );
    println!();

    // Sort categories by total cell mismatches (descending)
    let mut cat_list: Vec<_> = global_categories.iter().collect();
    cat_list.sort_by(|a, b| b.1.cmp(a.1));

    println!("Mismatch categories (sorted by total cells affected):");
    println!("{:<30} {:>10} {:>10}", "Category", "TotalCells", "Seeds");
    println!("{:-<52}", "");
    for (cat, total_cells) in &cat_list {
        let seeds_affected = global_category_seeds.get(cat).unwrap_or(&0);
        println!("{:<30} {:>10} {:>10}", cat.to_string(), total_cells, seeds_affected);
    }
    println!();

    // Show worst seeds
    let mut sorted_results: Vec<_> = all_results.iter().collect();
    sorted_results.sort_by(|a, b| b.total_mismatches.cmp(&a.total_mismatches));

    println!("Top 10 worst seeds:");
    println!(
        "{:<8} {:>10} {:>12} {:>12}",
        "Seed", "Mismatches", "RoomCount", "RoomPos"
    );
    println!("{:-<44}", "");
    for result in sorted_results.iter().take(10) {
        println!(
            "{:<8} {:>10} {:>12} {:>12}",
            result.seed,
            result.total_mismatches,
            if result.room_count_match { "OK" } else { "MISMATCH" },
            if result.room_positions_match {
                "OK"
            } else {
                "MISMATCH"
            },
        );
    }
    println!();

    // Show sample mismatches from worst seed
    if let Some(worst) = sorted_results.first().filter(|w| !w.sample_mismatches.is_empty()) {
        {
            println!(
                "Sample mismatches from seed {} ({} total):",
                worst.seed, worst.total_mismatches
            );
            for m in &worst.sample_mismatches {
                println!(
                    "  ({},{}) Rust={} C={}",
                    m.x, m.y, m.rust_type, m.c_type
                );
            }
            println!();
            println!("Category breakdown for seed {}:", worst.seed);
            let mut cats: Vec<_> = worst.categories.iter().collect();
            cats.sort_by(|a, b| b.1.cmp(a.1));
            for (cat, count) in &cats {
                println!("  {}: {}", cat, count);
            }
        }
    }

    // Show near-perfect seeds (1-5 mismatches)
    let mut near_perfect: Vec<_> = all_results.iter()
        .filter(|r| r.total_mismatches > 0 && r.total_mismatches <= 5)
        .collect();
    near_perfect.sort_by_key(|r| r.total_mismatches);
    if !near_perfect.is_empty() {
        println!("\nNear-perfect seeds (1-5 mismatches):");
        for r in &near_perfect {
            let cats: Vec<_> = r.categories.iter().map(|(c, n)| format!("{}:{}", c, n)).collect();
            println!("  seed {}: {} mismatches [{}]", r.seed, r.total_mismatches, cats.join(", "));
        }
    }

    // Metric: perfect seeds should only increase over time
    println!("\n========================================");
    println!("  PRIMARY METRIC: {}/100 seeds with 0 mismatches", perfect_seeds);
    println!("========================================\n");

    // This test is informational — it should never fail.
    // The CI convergence gate handles regression thresholds.
    // But we do assert the test ran successfully.
    assert!(
        all_results.len() == 100,
        "Should have tested 100 seeds"
    );
}

/// Quick single-seed diagnostic — run with:
/// cargo test -p nh-compare --test multi_seed_triage -- triage_single_seed --nocapture
#[test]
#[serial]
fn triage_single_seed() {
    let seed: u64 = std::env::var("TRIAGE_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(123);

    let mut c_engine = CGameEngine::new();
    c_engine
        .init("Valkyrie", "Human", 0, 0)
        .expect("C engine init failed");

    let result = run_seed(seed, &mut c_engine);

    println!("\nSeed {} triage:", seed);
    println!("  Total mismatches: {}", result.total_mismatches);
    println!("  Room count match: {}", result.room_count_match);
    println!("  Room positions match: {}", result.room_positions_match);

    if !result.categories.is_empty() {
        println!("  Categories:");
        let mut cats: Vec<_> = result.categories.iter().collect();
        cats.sort_by(|a, b| b.1.cmp(a.1));
        for (cat, count) in &cats {
            println!("    {}: {}", cat, count);
        }
    }

    for m in &result.sample_mismatches {
        println!(
            "  ({},{}) Rust={} C={}",
            m.x, m.y, m.rust_type, m.c_type
        );
    }
}

/// Test state leakage: run seed 89 alone vs after running prior seeds
#[test]
#[serial]
fn test_state_leakage() {
    let target_seed: u64 = std::env::var("TRIAGE_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(89);

    // Test 1: Run target seed alone (fresh engine)
    let mut c_engine1 = CGameEngine::new();
    c_engine1.init("Valkyrie", "Human", 0, 0).expect("init failed");
    let result_alone = run_seed(target_seed, &mut c_engine1);

    // Test 2: Run target seed after running seeds 1..target_seed first
    let mut c_engine2 = CGameEngine::new();
    c_engine2.init("Valkyrie", "Human", 0, 0).expect("init failed");
    for prior_seed in 1..target_seed {
        let _ = run_seed(prior_seed, &mut c_engine2);
    }
    let result_after = run_seed(target_seed, &mut c_engine2);

    println!("\n=== State Leakage Test for seed {} ===", target_seed);
    println!("  Alone:      {} mismatches", result_alone.total_mismatches);
    println!("  After 1..{}: {} mismatches", target_seed - 1, result_after.total_mismatches);

    if result_alone.total_mismatches != result_after.total_mismatches {
        println!("  *** STATE LEAKAGE DETECTED! ***");

        // Binary search: find which prior seed introduces the leak
        let mut lo = 1u64;
        let mut hi = target_seed - 1;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let mut engine = CGameEngine::new();
            engine.init("Valkyrie", "Human", 0, 0).expect("init failed");
            for s in 1..=mid {
                let _ = run_seed(s, &mut engine);
            }
            let r = run_seed(target_seed, &mut engine);
            if r.total_mismatches == result_alone.total_mismatches {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        println!("  First leaking seed: {}", lo);

        // Confirm: run just the leaking seed then the target
        let mut engine = CGameEngine::new();
        engine.init("Valkyrie", "Human", 0, 0).expect("init failed");
        let _ = run_seed(lo, &mut engine);
        let r = run_seed(target_seed, &mut engine);
        println!("  After only seed {}: {} mismatches", lo, r.total_mismatches);
    } else {
        println!("  No state leakage detected!");
    }
}
