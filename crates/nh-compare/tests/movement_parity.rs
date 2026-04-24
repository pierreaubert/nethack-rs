//! Movement command parity tests — verify Rust and C produce identical state
//! when executing movement commands on shared level fixtures.
//!
//! Also includes the automated divergence diagnosis pipeline (Task 5):
//! runs turns until RNG delta != 0, prints divergent RNG traces with
//! function tags, and stops.

use nh_core::action::{Command, Direction};
use nh_core::dungeon::{Level, LevelFixture};
use nh_core::player::{Gender, Race, Role};
use nh_core::{CGameEngineTrait, GameLoop, GameRng, GameState};
use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use serial_test::serial;

/// Helper: init both engines on the same C-exported level, skip movemon, reseed.
fn setup_synced_engines(
    seed: u64,
    rng_reseed: u64,
) -> (GameLoop, CGameEngine, LevelFixture) {
    let mut c_engine = CGameEngine::new();
    c_engine
        .init("Valkyrie", "Human", 1, 0)
        .expect("C engine init failed");
    c_engine.reset(seed).expect("C engine reset failed");
    c_engine
        .generate_and_place()
        .expect("C generate_and_place failed");

    let level_json = c_engine.export_level();
    let fixture: LevelFixture = serde_json::from_str(&level_json)
        .unwrap_or_else(|e| panic!("Failed to parse C level fixture: {}", e));

    let (cx, cy) = c_engine.position();

    // Skip monster AI to isolate player command parity
    c_engine.set_skip_movemon(true);
    c_engine.reset_rng(rng_reseed).expect("C RNG reseed failed");

    let rust_rng = GameRng::new(rng_reseed);
    let mut rust_state = GameState::new_with_identity(
        rust_rng,
        "Hero".into(),
        Role::Valkyrie,
        Race::Human,
        Gender::Female,
        Role::Valkyrie.default_alignment(),
    );
    rust_state.current_level = Level::from_fixture(&fixture);
    rust_state.player.pos.x = cx as i8;
    rust_state.player.pos.y = cy as i8;
    rust_state.context.skip_movemon = true;
    rust_state.skip_invariant_checks = true;
    let rust_loop = GameLoop::new(rust_state);

    // Sync HP/position/AC/moves
    let rs = rust_loop.state();
    c_engine.set_state(
        rs.player.hp,
        rs.player.hp_max,
        rs.player.pos.x as i32,
        rs.player.pos.y as i32,
        rs.player.armor_class as i32,
        rs.turns as i64,
    );

    (rust_loop, c_engine, fixture)
}

/// Map Direction to C movement command character
fn direction_to_cmd(dir: Direction) -> char {
    match dir {
        Direction::North => 'k',
        Direction::South => 'j',
        Direction::East => 'l',
        Direction::West => 'h',
        Direction::NorthEast => 'u',
        Direction::NorthWest => 'y',
        Direction::SouthEast => 'n',
        Direction::SouthWest => 'b',
        other => panic!("unsupported direction {:?}", other),
    }
}

// ============================================================================
// Task 4: Movement command parity tests
// ============================================================================

/// Test: cardinal movement on shared level — position + RNG parity.
#[test]
#[serial]
fn test_movement_cardinal_parity() {
    let (mut rust_loop, c_engine, _) = setup_synced_engines(42, 7777);

    let directions = [
        Direction::East,
        Direction::East,
        Direction::South,
        Direction::South,
        Direction::West,
        Direction::North,
    ];

    for (i, &dir) in directions.iter().enumerate() {
        let c_before = c_engine.rng_call_count();
        let rust_before = rust_loop.state().rng.call_count();

        let rs = rust_loop.state();
        c_engine.set_state(
            rs.player.hp,
            rs.player.hp_max,
            rs.player.pos.x as i32,
            rs.player.pos.y as i32,
            rs.player.armor_class as i32,
            rs.turns as i64,
        );

        // Execute movement in both engines
        rust_loop.tick(Command::Move(dir));
        let cmd = direction_to_cmd(dir);
        match c_engine.exec_cmd(cmd) {
            Ok(()) => {}
            Err(e) if e.contains("Player died") => {
                println!("C player died at step {} — stopping", i);
                break;
            }
            Err(e) => panic!("C command failed at step {}: {}", i, e),
        }

        let rs = rust_loop.state();
        let (cx, cy) = c_engine.position();

        let c_after = c_engine.rng_call_count();
        let rust_after = rust_loop.state().rng.call_count();
        let c_consumed = c_after - c_before;
        let rust_consumed = rust_after - rust_before;
        let delta = (c_consumed as i64 - rust_consumed as i64).abs();

        println!(
            "Step {} ({:?}): Rust({},{}) C({},{}) RNG delta={}",
            i, dir, rs.player.pos.x, rs.player.pos.y, cx, cy, delta
        );

        // Position should match
        assert_eq!(
            rs.player.pos.x, cx as i8,
            "X position desync at step {} ({:?})",
            i, dir
        );
        assert_eq!(
            rs.player.pos.y, cy as i8,
            "Y position desync at step {} ({:?})",
            i, dir
        );

        // RNG should match (no movemon, same level)
        assert_eq!(
            delta, 0,
            "RNG delta at step {} ({:?}): C consumed {}, Rust consumed {}",
            i, dir, c_consumed, rust_consumed
        );
    }

    println!("Cardinal movement parity test passed!");
}

/// Test: rest + movement mixed sequence on shared level.
#[test]
#[serial]
fn test_mixed_rest_movement_parity() {
    let (mut rust_loop, c_engine, _) = setup_synced_engines(12345, 5555);

    // Alternate rest and movement
    let commands: Vec<(Command, char)> = vec![
        (Command::Rest, '.'),
        (Command::Move(Direction::East), 'l'),
        (Command::Rest, '.'),
        (Command::Move(Direction::South), 'j'),
        (Command::Rest, '.'),
        (Command::Rest, '.'),
        (Command::Move(Direction::West), 'h'),
        (Command::Move(Direction::North), 'k'),
        (Command::Rest, '.'),
        (Command::Move(Direction::East), 'l'),
    ];

    let mut cumulative_delta: i64 = 0;

    for (i, (rust_cmd, c_cmd)) in commands.iter().enumerate() {
        let c_before = c_engine.rng_call_count();
        let rust_before = rust_loop.state().rng.call_count();

        let rs = rust_loop.state();
        c_engine.set_state(
            rs.player.hp,
            rs.player.hp_max,
            rs.player.pos.x as i32,
            rs.player.pos.y as i32,
            rs.player.armor_class as i32,
            rs.turns as i64,
        );

        rust_loop.tick(rust_cmd.clone());
        match c_engine.exec_cmd(*c_cmd) {
            Ok(()) => {}
            Err(e) if e.contains("Player died") => {
                println!("C player died at step {} — stopping", i);
                break;
            }
            Err(e) => panic!("C command failed at step {}: {}", i, e),
        }

        let c_consumed = c_engine.rng_call_count() - c_before;
        let rust_consumed = rust_loop.state().rng.call_count() - rust_before;
        let delta = c_consumed as i64 - rust_consumed as i64;
        cumulative_delta += delta.abs();

        let rs = rust_loop.state();
        let (cx, cy) = c_engine.position();

        println!(
            "Step {} ('{}'): Rust({},{}) C({},{}) RNG delta={}",
            i, c_cmd, rs.player.pos.x, rs.player.pos.y, cx, cy, delta
        );
    }

    println!(
        "Mixed rest+movement: cumulative |delta| = {}",
        cumulative_delta
    );
}

// ============================================================================
// Task 5: Automated divergence diagnosis pipeline
// ============================================================================

/// Automated divergence finder: runs rest turns on a shared level (with movemon)
/// until RNG delta != 0. Prints the first divergent turn's RNG traces from both
/// engines with caller tags. This is the main debugging loop for future work.
#[test]
#[serial]
fn test_divergence_diagnosis_pipeline() {
    let seed = 42u64;
    let rng_reseed = 9999u64;

    let mut c_engine = CGameEngine::new();
    c_engine
        .init("Valkyrie", "Human", 1, 0)
        .expect("C engine init failed");
    c_engine.reset(seed).expect("C engine reset failed");
    c_engine
        .generate_and_place()
        .expect("C generate_and_place failed");

    let level_json = c_engine.export_level();
    let fixture: LevelFixture = serde_json::from_str(&level_json)
        .unwrap_or_else(|e| panic!("Failed to parse C level fixture: {}", e));

    let (cx, cy) = c_engine.position();

    // Skip movemon for now (enable later for deeper diagnosis)
    c_engine.set_skip_movemon(true);
    c_engine.reset_rng(rng_reseed).expect("C RNG reseed failed");

    let rust_rng = GameRng::new(rng_reseed);
    let mut rust_state = GameState::new_with_identity(
        rust_rng,
        "Hero".into(),
        Role::Valkyrie,
        Race::Human,
        Gender::Female,
        Role::Valkyrie.default_alignment(),
    );
    rust_state.current_level = Level::from_fixture(&fixture);
    rust_state.player.pos.x = cx as i8;
    rust_state.player.pos.y = cy as i8;
    rust_state.context.skip_movemon = true;
    rust_state.skip_invariant_checks = true;
    let mut rust_loop = GameLoop::new(rust_state);

    let rs = rust_loop.state();
    c_engine.set_state(
        rs.player.hp,
        rs.player.hp_max,
        rs.player.pos.x as i32,
        rs.player.pos.y as i32,
        rs.player.armor_class as i32,
        rs.turns as i64,
    );

    // Enable tracing on both engines
    c_engine.enable_rng_tracing();
    rust_loop.state_mut().rng.start_tracing();

    let max_turns = 100;
    let mut first_divergence: Option<usize> = None;

    for turn in 0..max_turns {
        c_engine.clear_rng_trace();
        let rust_trace_before = rust_loop.state().rng.get_trace().len();

        let c_before = c_engine.rng_call_count();
        let rust_before = rust_loop.state().rng.call_count();

        // Sync state
        let rs = rust_loop.state();
        c_engine.set_state(
            rs.player.hp,
            rs.player.hp_max,
            rs.player.pos.x as i32,
            rs.player.pos.y as i32,
            rs.player.armor_class as i32,
            rs.turns as i64,
        );

        // Execute rest on both
        match c_engine.exec_cmd('.') {
            Ok(()) => {}
            Err(e) if e.contains("Player died") => {
                println!("C player died at turn {} — stopping", turn);
                break;
            }
            Err(e) => panic!("C rest failed at turn {}: {}", turn, e),
        }
        rust_loop.tick(Command::Rest);

        let c_after = c_engine.rng_call_count();
        let rust_after = rust_loop.state().rng.call_count();
        let c_consumed = c_after - c_before;
        let rust_consumed = rust_after - rust_before;
        let delta = (c_consumed as i64 - rust_consumed as i64).abs();

        if delta != 0 {
            first_divergence = Some(turn);

            println!("\n========================================");
            println!("DIVERGENCE FOUND at turn {}", turn);
            println!("========================================");
            println!(
                "C consumed {} RNG calls, Rust consumed {} (delta={})",
                c_consumed, rust_consumed, delta
            );

            // Print C trace for this turn
            let c_trace = c_engine.rng_trace_json();
            println!("\n--- C RNG trace (turn {}) ---", turn);
            // Parse and pretty-print
            let c_entries: Vec<serde_json::Value> =
                serde_json::from_str(&c_trace).unwrap_or_default();
            for entry in &c_entries {
                let caller = entry["caller"].as_str().unwrap_or("");
                let func = entry["func"].as_str().unwrap_or("?");
                let arg = entry["arg"].as_u64().unwrap_or(0);
                let result = entry["result"].as_u64().unwrap_or(0);
                if caller.is_empty() {
                    println!("  {}({}) = {}", func, arg, result);
                } else {
                    println!("  {}:{}({}) = {}", caller, func, arg, result);
                }
            }

            // Print Rust trace for this turn
            let rust_full_trace = rust_loop.state().rng.get_trace();
            let rust_turn_trace = &rust_full_trace[rust_trace_before..];
            println!("\n--- Rust RNG trace (turn {}) ---", turn);
            for entry in rust_turn_trace {
                if entry.caller.is_empty() {
                    println!("  {}({}) = {}", entry.func, entry.arg, entry.result);
                } else {
                    println!(
                        "  {}:{}({}) = {}",
                        entry.caller, entry.func, entry.arg, entry.result
                    );
                }
            }

            // Side-by-side comparison
            let min_len = c_entries.len().min(rust_turn_trace.len());
            println!("\n--- Side-by-side (first {} calls) ---", min_len);
            for i in 0..min_len {
                let c_func = c_entries[i]["func"].as_str().unwrap_or("?");
                let c_arg = c_entries[i]["arg"].as_u64().unwrap_or(0);
                let c_res = c_entries[i]["result"].as_u64().unwrap_or(0);
                let c_caller = c_entries[i]["caller"].as_str().unwrap_or("");

                let r = &rust_turn_trace[i];
                let match_str = if c_func == r.func
                    && c_arg == r.arg
                    && c_res == r.result
                {
                    "OK"
                } else {
                    "DIFF"
                };

                println!(
                    "  [{}] {} C:{}:{}({})={} | Rust:{}:{}({})={}",
                    i, match_str,
                    c_caller, c_func, c_arg, c_res,
                    r.caller, r.func, r.arg, r.result,
                );
            }
            if c_entries.len() != rust_turn_trace.len() {
                println!(
                    "  ... C has {} more calls, Rust has {} more calls",
                    c_entries.len().saturating_sub(rust_turn_trace.len()),
                    rust_turn_trace.len().saturating_sub(c_entries.len()),
                );
            }

            break;
        }
    }

    match first_divergence {
        Some(turn) => {
            println!(
                "\nDiagnosis complete: first divergence at turn {}. See traces above.",
                turn
            );
        }
        None => {
            println!(
                "\nPerfect RNG parity across all {} turns! No divergence found.",
                max_turns
            );
        }
    }
}
