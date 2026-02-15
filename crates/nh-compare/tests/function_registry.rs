//! Function Registry: Validates c_function_registry.json against actual Rust source
//!
//! Loads the registry and checks that Rust files referenced actually exist,
//! reports coverage percentages per C file and overall.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

const REGISTRY_PATH: &str =
    "/Users/pierre/src/games/nethack-rs/crates/nh-compare/data/c_function_registry.json";
const NH_CORE_SRC: &str = "/Users/pierre/src/games/nethack-rs/crates/nh-core/src";

#[derive(Debug)]
struct RegistryEntry {
    c_file: String,
    c_func: String,
    rust_file: Option<String>,
    rust_func: Option<String>,
    status: String,
    phase: Option<u32>,
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
            phase: v["phase"].as_u64().map(|n| n as u32),
        })
        .collect()
}

#[test]
fn registry_json_is_valid() {
    let entries = load_registry();
    assert!(
        !entries.is_empty(),
        "Registry is empty — run the extraction script"
    );
    println!("Registry contains {} entries", entries.len());

    // Every entry must have c_file and c_func
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

#[test]
fn registry_rust_files_exist() {
    let entries = load_registry();
    let mut missing_files = Vec::new();

    let rust_files: Vec<&str> = entries
        .iter()
        .filter_map(|e| e.rust_file.as_deref())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    for rf in &rust_files {
        let full_path = format!("{}/{}", NH_CORE_SRC, rf);
        if !Path::new(&full_path).exists() {
            missing_files.push(rf.to_string());
        }
    }

    println!("\n=== Rust File Existence Check ===");
    println!("Referenced Rust files: {}", rust_files.len());
    println!("Missing Rust files: {}", missing_files.len());
    for mf in &missing_files {
        println!("  MISSING: {}", mf);
    }
    assert!(
        missing_files.is_empty(),
        "{} referenced Rust files do not exist",
        missing_files.len()
    );
}

#[test]
fn registry_coverage_report() {
    let entries = load_registry();

    // Per-C-file stats
    let mut per_file: HashMap<String, (usize, usize, usize, usize)> = HashMap::new();
    for e in &entries {
        let entry = per_file.entry(e.c_file.clone()).or_default();
        entry.0 += 1; // total
        match e.status.as_str() {
            "ported" => entry.1 += 1,
            "stub" => entry.2 += 1,
            "missing" => entry.3 += 1,
            _ => {}
        }
    }

    println!("\n=== C Function Registry Coverage ===");
    println!(
        "{:<20} {:>6} {:>6} {:>6} {:>6} {:>6}",
        "C File", "Total", "Ported", "Stub", "Missing", "Port%"
    );
    println!("{}", "-".repeat(70));

    let mut files: Vec<_> = per_file.iter().collect();
    files.sort_by_key(|(k, _)| k.clone());

    let mut total_all = 0;
    let mut ported_all = 0;
    let mut stub_all = 0;
    let mut missing_all = 0;

    for (file, (total, ported, stub, missing)) in &files {
        let pct = if *total > 0 {
            *ported * 100 / *total
        } else {
            0
        };
        println!(
            "{:<20} {:>6} {:>6} {:>6} {:>6} {:>5}%",
            file, total, ported, stub, missing, pct
        );
        total_all += total;
        ported_all += ported;
        stub_all += stub;
        missing_all += missing;
    }

    println!("{}", "-".repeat(70));
    let pct_all = if total_all > 0 {
        ported_all * 100 / total_all
    } else {
        0
    };
    println!(
        "{:<20} {:>6} {:>6} {:>6} {:>6} {:>5}%",
        "TOTAL", total_all, ported_all, stub_all, missing_all, pct_all
    );

    // Per-phase stats
    let mut per_phase: HashMap<u32, (usize, usize)> = HashMap::new();
    for e in &entries {
        if let Some(phase) = e.phase {
            let entry = per_phase.entry(phase).or_default();
            entry.0 += 1;
            if e.status == "ported" {
                entry.1 += 1;
            }
        }
    }

    println!("\n=== Per-Phase Coverage ===");
    println!("{:<10} {:>6} {:>6} {:>6}", "Phase", "Total", "Ported", "Port%");
    println!("{}", "-".repeat(35));

    let mut phases: Vec<_> = per_phase.iter().collect();
    phases.sort_by_key(|(k, _)| *k);
    for (phase, (total, ported)) in &phases {
        let pct = if *total > 0 {
            *ported * 100 / *total
        } else {
            0
        };
        println!("{:<10} {:>6} {:>6} {:>5}%", phase, total, ported, pct);
    }

    println!(
        "\nBaseline: {}% ported ({}/{})",
        pct_all, ported_all, total_all
    );
}
