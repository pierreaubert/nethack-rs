//! Combat model comparison between Rust and C NetHack
//!
//! This example runs a focused comparison of combat mechanics:
//! - Player attacks on monsters
//! - Monster attacks on player
//! - Damage calculation verification
//! - Combat message comparison

use nh_core::{GameLoop, GameRng, GameState};
use nh_player::ffi::CGameEngine;
use nh_player::state::c_extractor::CGameWrapper;
use nh_player::state::common::{GameAction, UnifiedGameState};
use nh_player::state::rust_extractor::RustGameEngine;

/// Find a monster adjacent to the player and attack it
fn find_and_attack(state: &UnifiedGameState) -> Option<GameAction> {
    for monster in &state.nearby_monsters {
        let dx = monster.position.0 - state.position.0;
        let dy = monster.position.1 - state.position.1;

        // Check if monster is adjacent (not diagonal)
        if dx.abs() + dy.abs() == 1 {
            return Some(match (dx, dy) {
                (0, -1) => GameAction::AttackNorth,
                (0, 1) => GameAction::AttackSouth,
                (-1, 0) => GameAction::AttackWest,
                (1, 0) => GameAction::AttackEast,
                _ => return None,
            });
        }
    }
    None
}

/// Get a random attack direction when no monsters are nearby
fn random_attack_direction(rng: u64) -> GameAction {
    match rng % 4 {
        0 => GameAction::AttackNorth,
        1 => GameAction::AttackSouth,
        2 => GameAction::AttackWest,
        _ => GameAction::AttackEast,
    }
}

fn run_combat_session(seed: u64, max_turns: u64) {
    println!("=== Combat Comparison Session (seed={}) ===", seed);

    // Initialize Rust game
    let rust_rng = GameRng::new(seed);
    let rust_state = GameState::new(rust_rng);
    let mut rust_loop = GameLoop::new(rust_state);
    let mut rust_engine = RustGameEngine::new(&mut rust_loop);

    // Initialize C game
    let mut c_engine = CGameEngine::new();
    c_engine
        .init("Tourist", "Human", 0, 0)
        .expect("Failed to init C engine");
    let mut c_wrapper = CGameWrapper::new(&mut c_engine);

    let mut combat_stats = CombatStats::default();

    for turn in 1..=max_turns {
        let rust_state = rust_engine.extract_state();
        let c_state = c_wrapper.extract_state();

        // Check if player is dead
        if rust_state.is_dead || c_state.is_dead {
            println!("Player died at turn {}", turn);
            break;
        }

        // Select combat action
        let action = if let Some(attack) = find_and_attack(&rust_state) {
            println!("[Turn {}] Attacking monster!", turn);
            attack
        } else {
            let action = random_attack_direction(seed + turn as u64);
            let direction_str = match action {
                GameAction::AttackNorth => "North",
                GameAction::AttackSouth => "South",
                GameAction::AttackWest => "West",
                GameAction::AttackEast => "East",
                _ => "unknown",
            };
            println!(
                "[Turn {}] No monsters nearby, attacking {}",
                turn, direction_str
            );
            action
        };

        // Execute step - returns (reward, message)
        let (rust_reward, rust_msg) = rust_engine.step(&action);
        let (c_reward, c_msg) = c_wrapper.step(&action);

        // Record combat events
        combat_stats.turns += 1;

        // Get new states
        let rust_new_state = rust_engine.extract_state();
        let c_new_state = c_wrapper.extract_state();

        // Look for combat messages
        let rust_damage = rust_msg.to_lowercase().contains("hit")
            || rust_msg.to_lowercase().contains("damage")
            || rust_msg.to_lowercase().contains("kill");
        let c_damage = c_msg.to_lowercase().contains("hit")
            || c_msg.to_lowercase().contains("damage")
            || c_msg.to_lowercase().contains("kill");

        if rust_damage {
            combat_stats.rust_attacks += 1;
        }
        if c_damage {
            combat_stats.c_attacks += 1;
        }

        if !rust_msg.is_empty() && rust_msg != "You move." {
            println!("  Rust: {}", rust_msg);
        }
        if !c_msg.is_empty() && c_msg != "You move." {
            println!("  C:    {}", c_msg);
        }

        // Check for HP differences
        let hp_diff = (rust_new_state.hp - c_new_state.hp).abs();
        if hp_diff > 0 {
            combat_stats.hp_differences += 1;
            println!(
                "  HP DIFF: Rust={}, C={} (diff={})",
                rust_new_state.hp, c_new_state.hp, hp_diff
            );
        }

        // Check monster count
        let rust_monsters = rust_new_state.nearby_monsters.len();
        let c_monsters = c_new_state.nearby_monsters.len();
        if rust_monsters != c_monsters {
            println!(
                "  MONSTER COUNT DIFF: Rust={}, C={}",
                rust_monsters, c_monsters
            );
            combat_stats.monster_diffs += 1;
        }

        // Safety wait step
        let _ = rust_engine.step(&GameAction::Wait);
        let _ = c_wrapper.step(&GameAction::Wait);

        // Progress indicator
        if turn % 25 == 0 && combat_stats.turns > 0 {
            println!(
                "[Turn {}] Progress: {} turns, {} HP diffs, {} monster diffs",
                turn, combat_stats.turns, combat_stats.hp_differences, combat_stats.monster_diffs
            );
        }
    }

    println!("\n=== Combat Session Summary ===");
    println!("Total turns: {}", combat_stats.turns);
    println!("Rust attacks detected: {}", combat_stats.rust_attacks);
    println!("C attacks detected: {}", combat_stats.c_attacks);
    println!("HP differences: {}", combat_stats.hp_differences);
    println!("Monster count differences: {}", combat_stats.monster_diffs);
}

#[derive(Default)]
struct CombatStats {
    turns: u64,
    rust_attacks: u32,
    c_attacks: u32,
    hp_differences: u32,
    monster_diffs: u32,
}

fn main() {
    println!("NetHack Rust vs C Combat Model Comparison\n");

    // Run combat-focused sessions
    for seed in [42, 12345, 99999] {
        run_combat_session(seed, 100);
        println!();
    }

    println!("\n=== Analysis ===");
    println!("This comparison tests:");
    println!("1. Damage calculation matching between Rust and C");
    println!("2. Monster spawning and tracking");
    println!("3. Combat message consistency");
    println!("4. HP change synchronization");
}
