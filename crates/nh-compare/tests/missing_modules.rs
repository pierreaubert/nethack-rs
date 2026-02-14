//! Gap verification: Missing modules and implementation depth
//!
//! Checks that key modules referenced in the convergence plan exist
//! and measures their line counts against C targets.

use std::path::Path;

const RUST_SRC: &str = "/Users/pierre/src/games/nethack-rs/crates/nh-core/src";

// ============================================================================
// Module existence checks
// ============================================================================

#[test]
fn test_artifact_module_exists() {
    let path = Path::new(RUST_SRC).join("combat/artifact.rs");
    assert!(
        path.exists(),
        "Plan Step 5.5: artifact module should exist at {:?}",
        path
    );
}

#[test]
fn test_makemon_module_missing() {
    let path = Path::new(RUST_SRC).join("monster/makemon.rs");
    // This documents a known gap — makemon.rs is MISSING (Plan Step 6.2)
    if path.exists() {
        println!("PROGRESS: makemon.rs now exists!");
    } else {
        println!("GAP: monster/makemon.rs MISSING (C: makemon.c = 2,318 lines)");
    }
}

#[test]
fn test_polymorph_module_missing() {
    let path = Path::new(RUST_SRC).join("magic/polymorph.rs");
    if path.exists() {
        println!("PROGRESS: polymorph.rs now exists!");
    } else {
        println!("GAP: magic/polymorph.rs MISSING (C: polyself.c = 1,907 lines)");
    }
}

#[test]
fn test_detect_module_missing() {
    let path = Path::new(RUST_SRC).join("magic/detect.rs");
    if path.exists() {
        println!("PROGRESS: detect.rs now exists!");
    } else {
        println!("GAP: magic/detect.rs MISSING (C: detect.c = 2,032 lines)");
    }
}

#[test]
fn test_death_module_missing() {
    let path = Path::new(RUST_SRC).join("player/death.rs");
    if path.exists() {
        println!("PROGRESS: death.rs now exists!");
    } else {
        println!("GAP: player/death.rs MISSING (C: end.c = 2,292 lines)");
    }
}

// ============================================================================
// Implementation depth checks (line counts vs C targets)
// ============================================================================

struct ModuleCheck {
    rust_path: &'static str,
    c_file: &'static str,
    c_lines: usize,
    plan_step: &'static str,
}

const MODULE_CHECKS: &[ModuleCheck] = &[
    ModuleCheck {
        rust_path: "action/eat.rs",
        c_file: "eat.c",
        c_lines: 3352,
        plan_step: "4.1",
    },
    ModuleCheck {
        rust_path: "action/apply.rs",
        c_file: "apply.c",
        c_lines: 3811,
        plan_step: "4.2",
    },
    ModuleCheck {
        rust_path: "action/pickup.rs",
        c_file: "pickup.c",
        c_lines: 3272,
        plan_step: "4.3",
    },
    ModuleCheck {
        rust_path: "action/wear.rs",
        c_file: "do_wear.c",
        c_lines: 2846,
        plan_step: "4.4",
    },
    ModuleCheck {
        rust_path: "action/trap.rs",
        c_file: "trap.c",
        c_lines: 5476,
        plan_step: "4.5",
    },
    ModuleCheck {
        rust_path: "action/pray.rs",
        c_file: "pray.c",
        c_lines: 2302,
        plan_step: "5.4",
    },
    ModuleCheck {
        rust_path: "magic/zap.rs",
        c_file: "zap.c",
        c_lines: 5354,
        plan_step: "5.1",
    },
    ModuleCheck {
        rust_path: "magic/scroll.rs",
        c_file: "read.c",
        c_lines: 2652,
        plan_step: "5.2",
    },
    ModuleCheck {
        rust_path: "magic/potion.rs",
        c_file: "potion.c",
        c_lines: 2412,
        plan_step: "5.3",
    },
    ModuleCheck {
        rust_path: "magic/artifacts.rs",
        c_file: "artifact.c",
        c_lines: 2205,
        plan_step: "5.5",
    },
    ModuleCheck {
        rust_path: "dungeon/shop.rs",
        c_file: "shk.c",
        c_lines: 4973,
        plan_step: "5.6",
    },
    ModuleCheck {
        rust_path: "monster/monst.rs",
        c_file: "mon.c",
        c_lines: 4264,
        plan_step: "6.1",
    },
    ModuleCheck {
        rust_path: "object/inventory.rs",
        c_file: "invent.c",
        c_lines: 4479,
        plan_step: "3.3",
    },
    ModuleCheck {
        rust_path: "object/obj.rs",
        c_file: "objnam.c",
        c_lines: 4300,
        plan_step: "3.2",
    },
    ModuleCheck {
        rust_path: "object/mkobj.rs",
        c_file: "mkobj.c",
        c_lines: 2969,
        plan_step: "3.1",
    },
];

fn count_lines(path: &Path) -> Option<usize> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.lines().count())
}

#[test]
fn test_implementation_depth_summary() {
    println!("\n=== Module Implementation Depth ===");
    println!(
        "{:<25} {:<12} {:<8} {:<8} {:<8} {:<8}",
        "Module", "C File", "C Lines", "Rust", "Ratio", "Step"
    );
    println!("{}", "-".repeat(75));

    let mut total_c = 0usize;
    let mut total_rust = 0usize;
    let mut missing_count = 0usize;
    let mut below_50_pct = Vec::new();

    for check in MODULE_CHECKS {
        let rust_path = Path::new(RUST_SRC).join(check.rust_path);
        let rust_lines = count_lines(&rust_path);
        total_c += check.c_lines;

        match rust_lines {
            Some(lines) => {
                total_rust += lines;
                let pct = lines * 100 / check.c_lines;
                println!(
                    "{:<25} {:<12} {:<8} {:<8} {:<7}% {:<8}",
                    check.rust_path, check.c_file, check.c_lines, lines, pct, check.plan_step
                );
                if pct < 50 {
                    below_50_pct.push((check.rust_path, lines, check.c_lines, check.plan_step));
                }
            }
            None => {
                missing_count += 1;
                println!(
                    "{:<25} {:<12} {:<8} {:<8} {:<7}  {:<8}",
                    check.rust_path, check.c_file, check.c_lines, "MISSING", "0%", check.plan_step
                );
            }
        }
    }

    println!("\n=== Summary ===");
    let overall_pct = if total_c > 0 {
        total_rust * 100 / total_c
    } else {
        0
    };
    println!("Total C lines tracked: {}", total_c);
    println!("Total Rust lines: {}", total_rust);
    println!("Overall ratio: {}%", overall_pct);
    println!("Missing modules: {}", missing_count);

    if !below_50_pct.is_empty() {
        println!("\n=== Critical Gaps (below 50%) ===");
        for (path, rust, c, step) in &below_50_pct {
            println!(
                "  Step {}: {} ({}/{} = {}%)",
                step,
                path,
                rust,
                c,
                rust * 100 / c
            );
        }
    }
}

// ============================================================================
// Specific depth assertions
// ============================================================================

#[test]
fn test_eat_depth() {
    let path = Path::new(RUST_SRC).join("action/eat.rs");
    let lines = count_lines(&path).unwrap_or(0);
    println!("eat.rs: {} lines (C target: 3,352)", lines);
    // Current known state: ~773 lines
    assert!(
        lines > 100,
        "eat.rs should have substantial implementation, got {} lines",
        lines
    );
}

#[test]
fn test_trap_depth() {
    let path = Path::new(RUST_SRC).join("action/trap.rs");
    let lines = count_lines(&path).unwrap_or(0);
    println!("trap.rs: {} lines (C target: 5,476)", lines);
    assert!(
        lines > 50,
        "trap.rs should exist with some implementation, got {} lines",
        lines
    );
}

#[test]
fn test_pray_depth() {
    let path = Path::new(RUST_SRC).join("action/pray.rs");
    let lines = count_lines(&path).unwrap_or(0);
    println!("pray.rs: {} lines (C target: 2,302)", lines);
    assert!(
        lines > 50,
        "pray.rs should exist with some implementation, got {} lines",
        lines
    );
}

#[test]
fn test_inventory_depth() {
    let path = Path::new(RUST_SRC).join("object/inventory.rs");
    let lines = count_lines(&path).unwrap_or(0);
    println!("inventory.rs: {} lines (C target: 4,479)", lines);
    assert!(
        lines > 100,
        "inventory.rs should have substantial implementation, got {} lines",
        lines
    );
}

#[test]
fn test_zap_depth() {
    let path = Path::new(RUST_SRC).join("magic/zap.rs");
    let lines = count_lines(&path).unwrap_or(0);
    println!("zap.rs: {} lines (C target: 5,354)", lines);
    assert!(
        lines > 500,
        "zap.rs should have substantial implementation, got {} lines",
        lines
    );
}

// ============================================================================
// Missing module gap scores
// ============================================================================

#[test]
fn test_missing_modules_gap_score() {
    let missing_modules: &[(&str, &str, usize)] = &[
        ("monster/makemon.rs", "makemon.c", 2318),
        ("magic/polymorph.rs", "polyself.c", 1907),
        ("magic/detect.rs", "detect.c", 2032),
        ("player/death.rs", "end.c", 2292),
    ];

    let mut total_missing_lines = 0;
    let mut actually_missing = 0;

    println!("\n=== Missing Module Analysis ===");
    for (rust_path, c_file, c_lines) in missing_modules {
        let path = Path::new(RUST_SRC).join(rust_path);
        if !path.exists() {
            println!(
                "MISSING: {} (needs {} from {})",
                rust_path, c_lines, c_file
            );
            total_missing_lines += c_lines;
            actually_missing += 1;
        } else {
            let lines = count_lines(&path).unwrap_or(0);
            println!("EXISTS:  {} ({} lines, C target: {})", rust_path, lines, c_lines);
        }
    }

    println!("\nTotal missing C lines: {}", total_missing_lines);
    println!("Modules still missing: {}/{}", actually_missing, missing_modules.len());
}
