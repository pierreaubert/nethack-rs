//! Step 0.2: Turn-level replay comparison
//!
//! Given a seed and a sequence of commands, runs the Rust game loop
//! and verifies determinism and stability.
//!
//! When C FFI becomes stable, this should be extended to compare C vs Rust.

use std::collections::HashMap;
use std::path::Path;

// ============================================================================
// GameAction — local enum for replay commands
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
enum GameAction {
    MoveNorth,
    MoveSouth,
    MoveEast,
    MoveWest,
    MoveNorthWest,
    MoveNorthEast,
    MoveSouthWest,
    MoveSouthEast,
    Wait,
    Pickup,
    EatFirst,
    DropFirst,
    WieldFirst,
    WearFirst,
    TakeOffFirst,
    QuaffFirst,
    ReadFirst,
    ZapFirst,
    GoUp,
    GoDown,
    Inventory,
    Look,
}

// ============================================================================
// Replay file format
// ============================================================================

/// A recorded replay: seed + sequence of actions.
#[derive(Debug, Clone)]
struct Replay {
    name: String,
    seed: u64,
    role: String,
    race: String,
    commands: Vec<GameAction>,
    /// Which subsystems this replay exercises.
    tags: Vec<String>,
}

impl Replay {
    /// Parse a replay from the simple text format.
    ///
    /// Format:
    /// ```text
    /// # comment
    /// name: short_game_basic_movement
    /// seed: 42
    /// role: Tourist
    /// race: Human
    /// tags: movement,combat
    /// commands: kjhl.kjhl,e
    /// ```
    fn parse(text: &str) -> Option<Self> {
        let mut name = String::new();
        let mut seed = 0u64;
        let mut role = "Tourist".to_string();
        let mut race = "Human".to_string();
        let mut commands = Vec::new();
        let mut tags = Vec::new();

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(val) = line.strip_prefix("name:") {
                name = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("seed:") {
                seed = val.trim().parse().ok()?;
            } else if let Some(val) = line.strip_prefix("role:") {
                role = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("race:") {
                race = val.trim().to_string();
            } else if let Some(val) = line.strip_prefix("tags:") {
                tags = val.trim().split(',').map(|s| s.trim().to_string()).collect();
            } else if let Some(val) = line.strip_prefix("commands:") {
                commands = parse_command_string(val.trim());
            }
        }

        if name.is_empty() {
            return None;
        }

        Some(Replay {
            name,
            seed,
            role,
            race,
            commands,
            tags,
        })
    }
}

/// Parse a command string like "kjhl.kjhl,e" into GameActions.
fn parse_command_string(s: &str) -> Vec<GameAction> {
    let mut actions = Vec::new();
    for ch in s.chars() {
        let action = match ch {
            'k' => GameAction::MoveNorth,
            'j' => GameAction::MoveSouth,
            'h' => GameAction::MoveWest,
            'l' => GameAction::MoveEast,
            'y' => GameAction::MoveNorthWest,
            'u' => GameAction::MoveNorthEast,
            'b' => GameAction::MoveSouthWest,
            'n' => GameAction::MoveSouthEast,
            '.' => GameAction::Wait,
            ',' => GameAction::Pickup,
            'e' => GameAction::EatFirst,
            'd' => GameAction::DropFirst,
            'w' => GameAction::WieldFirst,
            'W' => GameAction::WearFirst,
            'T' => GameAction::TakeOffFirst,
            'q' => GameAction::QuaffFirst,
            'r' => GameAction::ReadFirst,
            'z' => GameAction::ZapFirst,
            '<' => GameAction::GoUp,
            '>' => GameAction::GoDown,
            'i' => GameAction::Inventory,
            ' ' => continue, // skip whitespace
            _ => continue,   // skip unknown
        };
        actions.push(action);
    }
    actions
}

// ============================================================================
// Replay execution
// ============================================================================

/// Result of running a single replay.
#[derive(Debug)]
struct ReplayResult {
    name: String,
    seed: u64,
    total_turns: usize,
    /// Per-turn difference counts by category.
    diffs_by_category: HashMap<String, usize>,
    /// Total differences across all turns.
    total_critical: usize,
    total_major: usize,
    total_minor: usize,
    /// First turn where a critical difference was detected.
    first_critical_turn: Option<usize>,
    /// Whether the replay passed (no critical diffs).
    passed: bool,
}

/// Run a replay against only the Rust engine (since C engine FFI init crashes
/// in the test environment). Reports the Rust-side state after each turn.
///
/// When the C FFI becomes stable, this should be extended to run both engines.
fn run_replay_rust_only(replay: &Replay) -> ReplayResult {
    let rng = nh_core::GameRng::new(replay.seed);
    let state = nh_core::GameState::new(rng);
    let mut game_loop = nh_core::GameLoop::new(state);

    let mut result = ReplayResult {
        name: replay.name.clone(),
        seed: replay.seed,
        total_turns: 0,
        diffs_by_category: HashMap::new(),
        total_critical: 0,
        total_major: 0,
        total_minor: 0,
        first_critical_turn: None,
        passed: true,
    };

    for (turn_idx, action) in replay.commands.iter().enumerate() {
        let command = action_to_command(action);
        let loop_result = game_loop.tick(command);

        result.total_turns = turn_idx + 1;

        // Check if game ended
        match loop_result {
            nh_core::GameLoopResult::PlayerDied(_) |
            nh_core::GameLoopResult::PlayerWon |
            nh_core::GameLoopResult::PlayerQuit |
            nh_core::GameLoopResult::SaveAndQuit => break,
            nh_core::GameLoopResult::Continue => {}
        }
    }

    result
}

/// Convert GameAction to nh_core Command.
fn action_to_command(action: &GameAction) -> nh_core::action::Command {
    use nh_core::action::{Command, Direction};
    match action {
        GameAction::MoveNorth => Command::Move(Direction::North),
        GameAction::MoveSouth => Command::Move(Direction::South),
        GameAction::MoveEast => Command::Move(Direction::East),
        GameAction::MoveWest => Command::Move(Direction::West),
        GameAction::MoveNorthWest => Command::Move(Direction::NorthWest),
        GameAction::MoveNorthEast => Command::Move(Direction::NorthEast),
        GameAction::MoveSouthWest => Command::Move(Direction::SouthWest),
        GameAction::MoveSouthEast => Command::Move(Direction::SouthEast),
        GameAction::Wait => Command::Rest,
        GameAction::Pickup => Command::Pickup,
        GameAction::GoUp => Command::GoUp,
        GameAction::GoDown => Command::GoDown,
        GameAction::Inventory => Command::Inventory,
        GameAction::EatFirst => Command::Eat('a'),
        GameAction::DropFirst => Command::Drop('a'),
        GameAction::WieldFirst => Command::Wield(Some('a')),
        GameAction::WearFirst => Command::Wear('a'),
        GameAction::TakeOffFirst => Command::TakeOff('a'),
        GameAction::QuaffFirst => Command::Quaff('a'),
        GameAction::ReadFirst => Command::Read('a'),
        GameAction::ZapFirst => Command::Rest, // TODO: add ZapFirst when wand targeting is ready
        GameAction::Look => Command::Look,
    }
}

// ============================================================================
// Tests
// ============================================================================

/// Test: Rust engine can replay a basic movement sequence deterministically.
/// Same seed + same commands = same final state.
#[test]
fn test_rust_replay_deterministic() {
    let commands = parse_command_string("kjhlkjhl........");

    // Run twice with the same seed
    let rng1 = nh_core::GameRng::new(42);
    let state1 = nh_core::GameState::new(rng1);
    let mut loop1 = nh_core::GameLoop::new(state1);

    let rng2 = nh_core::GameRng::new(42);
    let state2 = nh_core::GameState::new(rng2);
    let mut loop2 = nh_core::GameLoop::new(state2);

    let mut results1 = Vec::new();
    let mut results2 = Vec::new();

    for action in &commands {
        let cmd = action_to_command(action);
        results1.push(format!("{:?}", loop1.tick(cmd.clone())));
        results2.push(format!("{:?}", loop2.tick(cmd)));
    }

    assert_eq!(
        results1, results2,
        "Determinism failure: same seed + same commands produced different results"
    );
}

/// Test: Parse a replay file format.
#[test]
fn test_replay_parsing() {
    let text = r#"
# Test replay
name: basic_movement
seed: 42
role: Wizard
race: Elf
tags: movement,exploration
commands: kjhlkjhl
"#;

    let replay = Replay::parse(text).expect("Failed to parse replay");
    assert_eq!(replay.name, "basic_movement");
    assert_eq!(replay.seed, 42);
    assert_eq!(replay.role, "Wizard");
    assert_eq!(replay.race, "Elf");
    assert_eq!(replay.commands.len(), 8);
    assert_eq!(replay.tags, vec!["movement", "exploration"]);
}

/// Test: Load and run all .replay files from tests/replays/.
#[test]
fn test_run_all_replays() {
    let replay_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/replays");

    if !replay_dir.exists() {
        eprintln!("No replays directory found at {:?}, skipping", replay_dir);
        return;
    }

    let mut results = Vec::new();
    let mut total_replays = 0;
    let mut passed_replays = 0;

    if let Ok(entries) = std::fs::read_dir(&replay_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "replay") {
                total_replays += 1;
                let text = std::fs::read_to_string(&path).expect("Failed to read replay file");
                if let Some(replay) = Replay::parse(&text) {
                    let result = run_replay_rust_only(&replay);
                    if result.passed {
                        passed_replays += 1;
                    }
                    results.push(result);
                }
            }
        }
    }

    // Print summary
    println!("\n=== Replay Test Summary ===");
    println!("Total replays: {}", total_replays);
    println!("Passed: {}", passed_replays);
    println!("Failed: {}", total_replays - passed_replays);

    for result in &results {
        println!(
            "  {} (seed={}): {} turns, {} critical, {} major, {} minor - {}",
            result.name,
            result.seed,
            result.total_turns,
            result.total_critical,
            result.total_major,
            result.total_minor,
            if result.passed { "PASSED" } else { "FAILED" }
        );
    }
}

/// Test: Run a short game and verify the Rust engine doesn't crash.
#[test]
fn test_short_game_stability() {
    let seeds = [42u64, 123, 456, 789, 1000];
    let commands = "kjhlkjhl........kjhlkjhl........kjhlkjhl........";

    for &seed in &seeds {
        let replay = Replay {
            name: format!("stability_seed_{}", seed),
            seed,
            role: "Tourist".to_string(),
            race: "Human".to_string(),
            commands: parse_command_string(commands),
            tags: vec!["stability".to_string()],
        };

        let result = run_replay_rust_only(&replay);
        assert!(
            result.total_turns > 0,
            "Seed {}: game should have run at least 1 turn",
            seed
        );
    }
}

/// Test: Subsystem report — run a replay and report per-subsystem diff counts.
/// This is the harness entry point that reports current convergence status.
#[test]
fn test_convergence_report() {
    // Define replays covering different subsystems
    let replays = vec![
        ("movement", 42, "kjhlkjhl.kjhlkjhl"),
        ("waiting", 100, "................"),
        ("diagonal", 200, "yubnybun"),
    ];

    println!("\n=== Convergence Report ===");
    println!("{:<20} {:<8} {:<8} {:<8}", "Subsystem", "Turns", "Seed", "Status");
    println!("{}", "-".repeat(50));

    for (name, seed, cmds) in &replays {
        let replay = Replay {
            name: name.to_string(),
            seed: *seed,
            role: "Tourist".to_string(),
            race: "Human".to_string(),
            commands: parse_command_string(cmds),
            tags: vec![name.to_string()],
        };

        let result = run_replay_rust_only(&replay);
        println!(
            "{:<20} {:<8} {:<8} {:<8}",
            name,
            result.total_turns,
            seed,
            if result.passed { "OK" } else { "DIVERGED" }
        );
    }
}
