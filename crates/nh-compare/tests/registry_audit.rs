//! Phase 29: Function Registry Promotion Sprint
//!
//! Validates the registry after the bulk promotion pass:
//! - C functions with working Rust implementations are marked "ported"
//! - Display/TTY/platform-specific functions are marked "not_needed"
//! - sp_lev.c parsing functions are marked "not_needed"
//! - Convergence score meets the minimum threshold

use std::collections::HashMap;
use std::fs;

const REGISTRY_PATH: &str =
    "/Users/pierre/src/games/nethack-rs/crates/nh-compare/data/c_function_registry.json";

#[derive(Debug)]
struct RegistryEntry {
    c_file: String,
    c_func: String,
    #[allow(dead_code)]
    rust_file: Option<String>,
    rust_func: Option<String>,
    status: String,
}

fn load_registry() -> Vec<RegistryEntry> {
    let data = fs::read_to_string(REGISTRY_PATH).expect("Failed to read registry JSON");
    let raw: Vec<serde_json::Value> = serde_json::from_str(&data).expect("Failed to parse JSON");
    raw.into_iter()
        .map(|v| RegistryEntry {
            c_file: v["c_file"].as_str().unwrap_or("").to_string(),
            c_func: v["c_func"].as_str().unwrap_or("").to_string(),
            rust_file: v["rust_file"].as_str().map(|s| s.to_string()),
            rust_func: v["rust_func"].as_str().map(|s| s.to_string()),
            status: v["status"].as_str().unwrap_or("unknown").to_string(),
        })
        .collect()
}

fn count_by_status(entries: &[RegistryEntry]) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for e in entries {
        *counts.entry(e.status.clone()).or_insert(0) += 1;
    }
    counts
}

// ---- Test 1: Registry loads and has entries ----

#[test]
fn registry_loads_and_has_entries() {
    let entries = load_registry();
    assert!(
        entries.len() > 2500,
        "Registry should have >2500 entries, got {}",
        entries.len()
    );
    println!("Registry contains {} entries", entries.len());

    // Every entry must have valid status
    for (i, e) in entries.iter().enumerate() {
        assert!(!e.c_file.is_empty(), "Entry {} has empty c_file", i);
        assert!(!e.c_func.is_empty(), "Entry {} has empty c_func", i);
        assert!(
            ["ported", "stub", "missing", "not_needed"].contains(&e.status.as_str()),
            "Entry {} ({}/{}) has invalid status: {}",
            i,
            e.c_file,
            e.c_func,
            e.status
        );
    }
}

// ---- Test 2: Count ported/stub/missing/not_needed ----

#[test]
fn registry_status_counts() {
    let entries = load_registry();
    let counts = count_by_status(&entries);

    let ported = *counts.get("ported").unwrap_or(&0);
    let stub = *counts.get("stub").unwrap_or(&0);
    let missing = *counts.get("missing").unwrap_or(&0);
    let not_needed = *counts.get("not_needed").unwrap_or(&0);
    let total = entries.len();

    println!("\n=== Phase 29 Registry Status Counts ===");
    println!("  ported:     {}", ported);
    println!("  stub:       {}", stub);
    println!("  missing:    {}", missing);
    println!("  not_needed: {}", not_needed);
    println!("  TOTAL:      {}", total);

    // After stub audit, targets are significantly higher
    assert!(
        ported >= 2100,
        "Expected >= 2100 ported entries, got {}",
        ported
    );
    assert!(
        not_needed >= 800,
        "Expected >= 800 not_needed entries, got {}",
        not_needed
    );
    assert!(
        stub == 0,
        "Expected 0 stub entries, got {}",
        stub
    );
    assert!(
        missing == 0,
        "Expected 0 missing entries, got {}",
        missing
    );
}

// ---- Test 3: Key functions are marked "ported" ----

#[test]
fn key_functions_are_ported() {
    let entries = load_registry();
    let ported_set: HashMap<(String, String), &RegistryEntry> = entries
        .iter()
        .map(|e| ((e.c_file.clone(), e.c_func.clone()), e))
        .collect();

    // Critical game-logic functions that must be ported.
    // Each tuple is (c_file, c_func) matching the registry entries.
    let must_be_ported = vec![
        // Monster system
        ("makemon.c", "makemon"),
        ("mon.c", "mcalcmove"),
        ("mon.c", "mfndpos"),
        ("mon.c", "wakeup"),
        ("mon.c", "wake_nearby"),
        ("mon.c", "newcham"),
        ("mondata.c", "attacktype"),
        ("mondata.c", "dmgtype"),
        // Object system
        ("mkobj.c", "mkobj"),
        ("mkobj.c", "mksobj"),
        ("mkobj.c", "blessorcurse"),
        ("mkobj.c", "weight"),
        ("objnam.c", "xname"),
        ("objnam.c", "doname"),
        ("objnam.c", "an"),
        // Combat
        ("uhitm.c", "find_roll_to_hit"),
        ("mhitu.c", "mattacku"),
        ("mhitm.c", "mattackm"),
        ("weapon.c", "hitval"),
        ("weapon.c", "dmgval"),
        // Actions
        ("eat.c", "doeat"),
        ("zap.c", "dozap"),
        ("explode.c", "explode"),
        ("zap.c", "bhit"),
        ("zap.c", "buzz"),
        ("trap.c", "dotrap"),
        ("trap.c", "mintrap"),
        ("trap.c", "maketrap"),
        ("trap.c", "erode_obj"),
        ("apply.c", "doapply"),
        ("apply.c", "use_mirror"),
        ("read.c", "doread"),
        ("potion.c", "dodrink"),
        ("potion.c", "make_confused"),
        ("potion.c", "make_stunned"),
        ("pray.c", "dopray"),
        ("pray.c", "dosacrifice"),
        ("dig.c", "dig"),
        // Player
        ("attrib.c", "adjattrib"),
        ("attrib.c", "exercise"),
        ("attrib.c", "acurr"),
        ("polyself.c", "polyself"),
        ("polyself.c", "newman"),
        // Dungeon
        ("teleport.c", "enexto"),
        ("teleport.c", "goodpos"),
        // RNG
        ("rnd.c", "rn2"),
        ("rnd.c", "rnd"),
        // Shop
        ("shk.c", "costly_spot"),
        ("shk.c", "addtobill"),
    ];

    let mut failures = Vec::new();
    for (c_file, c_func) in &must_be_ported {
        let key = (c_file.to_string(), c_func.to_string());
        match ported_set.get(&key) {
            Some(entry) if entry.status == "ported" => {}
            Some(entry) => {
                failures.push(format!(
                    "  {}::{} is '{}' (expected 'ported')",
                    c_file, c_func, entry.status
                ));
            }
            None => {
                failures.push(format!("  {}::{} not found in registry", c_file, c_func));
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "The following key functions are NOT marked as ported:\n{}",
            failures.join("\n")
        );
    }

    println!(
        "All {} key functions verified as ported",
        must_be_ported.len()
    );
}

// ---- Test 4: Display/TTY/platform functions are not_needed ----

#[test]
fn display_tty_functions_are_not_needed() {
    let entries = load_registry();

    // Files that should be entirely not_needed or ported (no stubs or missing).
    // Some functions in these files may have been matched to Rust equivalents
    // (e.g., options.c has nh_getenv which exists in Rust), so we allow "ported"
    // but disallow "stub" and "missing".
    let resolved_files = vec![
        "sp_lev.c",    // Rust-native level generation
        "options.c",   // Rust-native options (some funcs have Rust matches)
        "hacklib.c",   // replaced by Rust std
        "drawing.c",   // display subsystem
        "vision.c",    // LOS done differently
        "pline.c",     // message display
        "display.c",   // windowing
        "save.c",      // C serialization
        "restore.c",   // C deserialization
        "topten.c",    // score display
    ];

    for nf_file in &resolved_files {
        let file_entries: Vec<&RegistryEntry> =
            entries.iter().filter(|e| e.c_file == *nf_file).collect();

        if file_entries.is_empty() {
            continue;
        }

        // No stubs or missing should remain in these files
        let unresolved: Vec<&&RegistryEntry> = file_entries
            .iter()
            .filter(|e| e.status == "stub" || e.status == "missing")
            .collect();

        assert!(
            unresolved.is_empty(),
            "{}: {} entries still have stub/missing status (e.g., {}::{}='{}')",
            nf_file,
            unresolved.len(),
            unresolved.first().map(|e| e.c_file.as_str()).unwrap_or("?"),
            unresolved.first().map(|e| e.c_func.as_str()).unwrap_or("?"),
            unresolved.first().map(|e| e.status.as_str()).unwrap_or("?"),
        );
    }

    println!(
        "All {} display/TTY/platform files verified as fully resolved",
        resolved_files.len()
    );
}

// ---- Test 5: Convergence score meets minimum threshold ----

#[test]
fn convergence_score_minimum() {
    let entries = load_registry();
    let counts = count_by_status(&entries);
    let total = entries.len() as f64;

    let ported = *counts.get("ported").unwrap_or(&0) as f64;
    let not_needed = *counts.get("not_needed").unwrap_or(&0) as f64;
    let stub = *counts.get("stub").unwrap_or(&0) as f64;
    let missing = *counts.get("missing").unwrap_or(&0) as f64;

    let convergence = (ported + not_needed) / total * 100.0;

    println!("\n=== Phase 29 Convergence Score ===");
    println!("  Ported:     {:.0} ({:.1}%)", ported, ported / total * 100.0);
    println!(
        "  Not needed: {:.0} ({:.1}%)",
        not_needed,
        not_needed / total * 100.0
    );
    println!("  Stub:       {:.0} ({:.1}%)", stub, stub / total * 100.0);
    println!(
        "  Missing:    {:.0} ({:.1}%)",
        missing,
        missing / total * 100.0
    );
    println!("  ---------------------------------");
    println!("  CONVERGENCE: {:.1}%", convergence);

    // After stub audit, convergence should be 100%
    assert!(
        convergence >= 99.0,
        "Convergence score {:.1}% is below the 99% threshold",
        convergence
    );
}

// ---- Test 6: No missing entries remain ----

#[test]
fn no_missing_entries() {
    let entries = load_registry();
    let missing: Vec<&RegistryEntry> = entries.iter().filter(|e| e.status == "missing").collect();

    println!(
        "\n=== Missing Entries: {} ===",
        missing.len()
    );
    for e in &missing {
        println!("  {}::{}", e.c_file, e.c_func);
    }

    assert!(
        missing.is_empty(),
        "{} entries still have 'missing' status; these should be promoted or marked not_needed",
        missing.len()
    );
}

// ---- Test 7: Per-file ported coverage for key game files ----

#[test]
fn key_game_files_have_ported_functions() {
    let entries = load_registry();

    // These key game-logic files should have at least some ported functions
    let key_files = vec![
        ("mon.c", 20),
        ("mondata.c", 10),
        ("invent.c", 15),
        ("eat.c", 15),
        ("zap.c", 15),
        ("trap.c", 20),
        ("apply.c", 20),
        ("shk.c", 10),
        ("pray.c", 5),
        ("hack.c", 5),
        ("mkobj.c", 10),
        ("weapon.c", 5),
        ("attrib.c", 5),
    ];

    println!("\n=== Key File Ported Counts ===");

    for (c_file, min_ported) in &key_files {
        let ported_count = entries
            .iter()
            .filter(|e| e.c_file == *c_file && e.status == "ported")
            .count();

        let total = entries.iter().filter(|e| e.c_file == *c_file).count();

        let pct = if total > 0 {
            ported_count * 100 / total
        } else {
            0
        };

        println!(
            "  {:<15} {:>3}/{:<3} ported ({}%)",
            c_file, ported_count, total, pct
        );

        assert!(
            ported_count >= *min_ported,
            "{}: expected >= {} ported, got {}",
            c_file,
            min_ported,
            ported_count
        );
    }
}

// ---- Test 8: Ported entries with rust_func have valid function names ----

#[test]
fn ported_entries_have_valid_rust_func() {
    let entries = load_registry();
    let ported_with_func: Vec<&RegistryEntry> = entries
        .iter()
        .filter(|e| e.status == "ported" && e.rust_func.is_some())
        .collect();

    println!(
        "\n=== Ported entries with rust_func: {} ===",
        ported_with_func.len()
    );

    // rust_func should be a valid Rust identifier (snake_case)
    let invalid: Vec<&RegistryEntry> = ported_with_func
        .iter()
        .filter(|e| {
            let rf = e.rust_func.as_ref().unwrap();
            !rf.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                || rf.is_empty()
        })
        .copied()
        .collect();

    for e in &invalid {
        println!(
            "  INVALID rust_func: {}::{} -> {:?}",
            e.c_file, e.c_func, e.rust_func
        );
    }

    assert!(
        invalid.is_empty(),
        "{} ported entries have invalid rust_func names",
        invalid.len()
    );
}
