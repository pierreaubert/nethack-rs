//! Run a comparison session between Rust and C implementations
//!
//! This example demonstrates how to run the dual-game comparison system
//! to find behavioral differences between the Rust and C implementations.

use nh_core::{GameLoop, GameRng, GameState, CGameEngineTrait};
use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use nh_player::orchestrator::{DualGameOrchestrator, OrchestratorConfig};

fn run_comparison(seed: u64, max_turns: u64) {
    println!("=== Comparison Session (seed={}) ===", seed);

    // Initialize Rust game
    let state = GameState::new_with_identity(
        GameRng::new(seed),
        "Hero".to_string(),
        nh_core::player::Role::Tourist,
        nh_core::player::Race::Human,
        nh_core::player::Gender::Male,
        nh_core::player::AlignmentType::Neutral,
    );
    let mut rust_loop = GameLoop::new(state);

    // Initialize C game
    let mut c_engine = CGameEngine::new();
    c_engine
        .init("Tourist", "Human", 0, 0)
        .expect("Failed to init C engine");
    c_engine
        .reset(seed)
        .expect("Failed to reset C engine");
    c_engine
        .generate_and_place()
        .expect("Failed to generate C level");

    // Create orchestrator with short run for demo
    let config = OrchestratorConfig {
        max_turns_per_session: max_turns,
        initial_exploration_rate: 0.5, // High exploration for finding differences
        report_interval: 10,
        verbose: true,
        save_differences: false,
        output_dir: None,
    };

    let mut orchestrator = DualGameOrchestrator::new(config);

    // Run session
    let result = orchestrator.run_session(&mut rust_loop, &mut c_engine, seed);

    // Print results
    println!("\n=== Results ===");
    println!("Total turns: {}", result.total_turns);
    println!("Total reward: {:.2}", result.total_reward);
    println!("Rust died: {:?}", result.rust_death_turn);
    println!("C died: {:?}", result.c_death_turn);

    println!("\n=== Differences ===");
    if result.critical_differences.is_empty() && result.major_differences.is_empty() {
        println!("No significant differences found!");
    } else {
        println!(
            "Critical differences ({}):",
            result.critical_differences.len()
        );
        for (turn, diff) in &result.critical_differences {
            println!(
                "  Turn {}: {} = Rust: {:?} vs C: {:?} - {}",
                turn, diff.field, diff.rust_value, diff.c_value, diff.description
            );
        }
        println!("Major differences ({}):", result.major_differences.len());
        for (turn, diff) in &result.major_differences {
            println!(
                "  Turn {}: {} = Rust: {:?} vs C: {:?} - {}",
                turn, diff.field, diff.rust_value, diff.c_value, diff.description
            );
        }
    }

    println!("\n=== Messages ===");
    for msg in &result.messages[..result.messages.len().min(10)] {
        println!("  {}", msg);
    }
    if result.messages.len() > 10 {
        println!("  ... and {} more messages", result.messages.len() - 10);
    }
}

fn main() {
    println!("NetHack Rust vs C Comparison Tool\n");

    // Run a few short comparison sessions with different seeds
    for seed in [42, 12345, 99999, 123456789] {
        run_comparison(seed, 50);
        println!();
    }
}
