//! Fuzz Testing Tool for NetHack Comparison
//!
//! This tool runs the DualGameOrchestrator in a loop to find divergences
//! between the Rust and C implementations. It focuses on finding edge cases
//! by running many short sessions with random seeds.

use nh_core::{GameLoop, GameRng, GameState};
use nh_player::ffi::CGameEngine;
use nh_player::orchestrator::{DualGameOrchestrator, OrchestratorConfig};
use std::io::Write;
use std::time::Instant;

fn main() {
    println!("=== NetHack Comparison Fuzzer ===");
    println!("Running fuzz tests against real NetHack 3.6.7...");

    let config = OrchestratorConfig {
        max_turns_per_session: 200, // Short sessions to test init/early game stability
        initial_exploration_rate: 0.8, // High exploration to find edge cases
        report_interval: 1000,      // Quiet output
        verbose: false,
        save_differences: true,
        output_dir: Some("fuzz_output".to_string()),
    };

    let start_time = Instant::now();
    let mut sessions = 0;
    let mut failures = 0;
    let mut total_turns = 0;

    // Run until user interrupt
    loop {
        sessions += 1;
        let seed = rand::random();

        if sessions % 10 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            print!(
                "\rSessions: {} | Failures: {} | Turns: {} | Rate: {:.1} eps/sec ",
                sessions,
                failures,
                total_turns,
                sessions as f64 / elapsed
            );
            std::io::stdout().flush().unwrap();
        }

        // Initialize engines
        let rust_rng = GameRng::new(seed);
        let rust_state = GameState::new(rust_rng);
        let mut rust_loop = GameLoop::new(rust_state);

        let mut c_engine = CGameEngine::new();
        // Init with same role/race as default Rust state (usually Tourist/Human for now)
        if let Err(e) = c_engine.init("Tourist", "Human", 0, 0) {
            println!("\nFailed to init C engine: {}", e);
            continue;
        }

        let mut orchestrator = DualGameOrchestrator::new(config.clone());
        let result = orchestrator.run_session(&mut rust_loop, &mut c_engine, seed);

        total_turns += result.total_turns;

        if !result.critical_differences.is_empty() {
            failures += 1;
            println!(
                "\n\n[FAILURE] Seed {}: Found {} critical differences",
                seed,
                result.critical_differences.len()
            );
            for (turn, diff) in result.critical_differences {
                println!("  Turn {}: {} - {}", turn, diff.field, diff.description);
            }
            if failures >= 10 {
                println!("\nFailure limit reached.");
                break;
            }
        }

        // Stop after some time for CI/demo purposes (e.g., 60 seconds or 100 sessions)
        if start_time.elapsed().as_secs() > 60 {
            println!("\n\nTime limit reached.");
            break;
        }
    }

    println!("\n=== Fuzzing Complete ===");
    println!("Total Sessions: {}", sessions);
    println!("Total Failures: {}", failures);
    println!("Total Turns: {}", total_turns);

    if failures > 0 {
        std::process::exit(1);
    }
}
