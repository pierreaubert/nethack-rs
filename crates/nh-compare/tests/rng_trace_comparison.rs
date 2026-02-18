//! RNG trace comparison tests — compare Rust and C RNG call sequences
//! to pinpoint exactly where code paths diverge.

use nh_compare::diff::compare_rng_traces;
use nh_compare::snapshot::RngTraceEntry;
use nh_core::player::{Gender, Race, Role};
use nh_core::{GameLoop, GameRng, GameState};
use nh_core::action::Command;
use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use serial_test::serial;

/// Parse C RNG trace JSON into our RngTraceEntry format.
fn parse_c_trace(json: &str) -> Vec<RngTraceEntry> {
    let entries: Vec<serde_json::Value> = serde_json::from_str(json).unwrap_or_default();
    entries
        .iter()
        .map(|e| RngTraceEntry {
            seq: e["seq"].as_u64().unwrap_or(0),
            func: e["func"].as_str().unwrap_or("").to_string(),
            arg: e["arg"].as_u64().unwrap_or(0),
            result: e["result"].as_u64().unwrap_or(0),
        })
        .collect()
}

/// Convert nh_rng trace entries to our comparison format.
fn convert_rust_trace(entries: &[nh_rng::RngTraceEntry]) -> Vec<RngTraceEntry> {
    entries
        .iter()
        .map(|e| RngTraceEntry {
            seq: e.seq,
            func: e.func.to_string(),
            arg: e.arg,
            result: e.result,
        })
        .collect()
}

/// Per-turn RNG trace comparison for rest-only scenario.
/// Enables tracing on both engines, executes one turn, dumps traces,
/// and compares them.
#[test]
#[serial]
fn test_rng_trace_per_turn_rest() {
    let seed = 42u64;

    // Initialize C engine
    let mut c_engine = CGameEngine::new();
    c_engine.init("Valkyrie", "Human", 1, 0).expect("C init failed");
    c_engine.reset(seed).expect("C reset failed");

    // Initialize Rust engine
    let (cx, cy) = c_engine.position();
    let rust_rng = GameRng::new(seed);
    let mut rust_state = GameState::new_with_identity(
        rust_rng,
        "Hero".into(),
        Role::Valkyrie,
        Race::Human,
        Gender::Female,
        Role::Valkyrie.default_alignment(),
    );
    rust_state.player.pos.x = cx as i8;
    rust_state.player.pos.y = cy as i8;
    let mut rust_loop = GameLoop::new(rust_state);

    // Enable tracing on both
    rust_loop.state_mut().rng.start_tracing();
    c_engine.enable_rng_tracing();

    let num_turns = 10;
    let mut divergence_found = false;

    for turn in 0..num_turns {
        // Clear traces for this turn
        c_engine.clear_rng_trace();
        // Rust trace is cumulative, so we snapshot before/after

        let rust_trace_before = rust_loop.state().rng.get_trace().len();

        // Execute rest on both
        rust_loop.tick(Command::Rest);
        c_engine.exec_cmd('.').expect("C rest failed");

        // Get traces
        let rust_full_trace = rust_loop.state().rng.get_trace();
        let rust_turn_trace = &rust_full_trace[rust_trace_before..];
        let rust_entries = convert_rust_trace(rust_turn_trace);

        let c_trace_json = c_engine.rng_trace_json();
        let c_entries = parse_c_trace(&c_trace_json);

        println!(
            "Turn {}: Rust made {} RNG calls, C made {} RNG calls",
            turn,
            rust_entries.len(),
            c_entries.len()
        );

        // Compare traces
        if let Some(divergence) = compare_rng_traces(&rust_entries, &c_entries) {
            println!("  DIVERGENCE at call {}: {}", divergence.call_index, divergence.description);
            println!("  Rust context:");
            for e in &divergence.rust_context {
                println!("    seq={} {}({}) -> {}", e.seq, e.func, e.arg, e.result);
            }
            println!("  C context:");
            for e in &divergence.c_context {
                println!("    seq={} {}({}) -> {}", e.seq, e.func, e.arg, e.result);
            }
            divergence_found = true;
        }

        // Print first few entries for debugging
        if turn == 0 {
            let show = rust_entries.len().min(5);
            if show > 0 {
                println!("  Rust first {} calls:", show);
                for e in &rust_entries[..show] {
                    println!("    {}({}) -> {}", e.func, e.arg, e.result);
                }
            }
            let show = c_entries.len().min(5);
            if show > 0 {
                println!("  C first {} calls:", show);
                for e in &c_entries[..show] {
                    println!("    {}({}) -> {}", e.func, e.arg, e.result);
                }
            }
        }
    }

    if !divergence_found {
        println!("\nNo RNG divergence found across {} turns of resting!", num_turns);
    } else {
        println!("\nRNG divergence detected — this identifies where code paths differ.");
    }

    // This test reports but doesn't fail — divergence is expected until convergence improves.
}

/// Smoke test: verify that C RNG tracing infrastructure works.
#[test]
#[serial]
fn test_c_rng_tracing_smoke() {
    let mut c_engine = CGameEngine::new();
    c_engine.init("Valkyrie", "Human", 0, 0).expect("C init failed");

    // Enable tracing
    c_engine.enable_rng_tracing();

    // Make some RNG calls via the FFI
    let r1 = c_engine.rng_rn2(100);
    let r2 = c_engine.rng_rn2(6);

    // Get trace
    let trace_json = c_engine.rng_trace_json();
    let entries = parse_c_trace(&trace_json);

    println!("rn2(100)={}, rn2(6)={}", r1, r2);
    println!("Trace entries: {}", entries.len());
    for e in &entries {
        println!("  seq={} {}({}) -> {}", e.seq, e.func, e.arg, e.result);
    }

    assert_eq!(entries.len(), 2, "Expected 2 trace entries");
    assert_eq!(entries[0].func, "rn2");
    assert_eq!(entries[0].arg, 100);
    assert_eq!(entries[0].result, r1 as u64);
    assert_eq!(entries[1].func, "rn2");
    assert_eq!(entries[1].arg, 6);
    assert_eq!(entries[1].result, r2 as u64);

    // Test clear
    c_engine.clear_rng_trace();
    let trace_json = c_engine.rng_trace_json();
    let entries = parse_c_trace(&trace_json);
    assert!(entries.is_empty(), "Trace should be empty after clear");
}
