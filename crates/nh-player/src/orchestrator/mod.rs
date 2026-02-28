//! Orchestrator for running parallel comparison sessions.
//!
//! This module provides the main entry point for running comparison tests
//! between the Rust and C implementations using the virtual player.

use crate::agent::{SessionResult, VirtualPlayer, VirtualPlayerConfig};
use crate::ffi::CGameEngine;
use nh_core::CGameEngineTrait;
use crate::compare::compare_states;
use crate::state::c_extractor::CGameWrapper;
use crate::state::common::*;
use crate::state::rust_extractor::RustGameEngine;
use nh_core::player::Position;

/// Main orchestrator for dual-game comparison
pub struct DualGameOrchestrator {
    player: VirtualPlayer,
    config: OrchestratorConfig,
}

impl DualGameOrchestrator {
    /// Create a new orchestrator
    pub fn new(config: OrchestratorConfig) -> Self {
        let player_config = VirtualPlayerConfig {
            max_turns: config.max_turns_per_session,
            exploration_rate: config.initial_exploration_rate,
            report_interval: config.report_interval,
            ..Default::default()
        };

        Self {
            player: VirtualPlayer::new(player_config),
            config,
        }
    }

    /// Run a single comparison session
    pub fn run_session<E: CGameEngineTrait>(
        &mut self,
        rust_loop: &mut nh_core::GameLoop,
        c_engine: &mut E,
        seed: u64,
    ) -> SessionResult {
        let mut result = SessionResult::default();
        result.seed = seed;

        // Initialize wrappers
        // Need to do this before creating c_wrapper which takes a mutable borrow
        let level_json = c_engine.export_level();

        if let Ok(fixture) = serde_json::from_str::<nh_core::dungeon::LevelFixture>(&level_json) {
            rust_loop.state_mut().current_level = nh_core::dungeon::Level::from_fixture(&fixture);
        }

        let mut c_wrapper = CGameWrapper::new(c_engine);

        // Get initial C state to find where C placed the player
        let c_state_initial = c_wrapper.extract_state();
        
        // SYNC PLAYER POSITION: Move Rust player to where C engine placed them
        rust_loop.state_mut().player.pos = Position::new(
            c_state_initial.position.0 as i8,
            c_state_initial.position.1 as i8,
        );

        // Initialize wrappers
        let mut rust_wrapper = RustGameEngine::new(rust_loop);

        // Get initial states for comparison loop
        let mut rust_state = rust_wrapper.extract_state();
        let mut c_state = c_wrapper.extract_state();

        // Ensure C position and stats match Rust's initial (potentially modified by map sync)
        c_wrapper.set_state(
            rust_state.hp,
            rust_state.max_hp,
            rust_state.position.0,
            rust_state.position.1,
            rust_state.armor_class,
            rust_state.turn as i64,
        );

        // Set exploration rate
        self.player
            .set_exploration_rate(self.config.initial_exploration_rate);

        for turn in 0..self.config.max_turns_per_session {
            // Select action based on Rust's view
            let action = self.player.select_action(&rust_state);

            // Execute on Rust
            let (rust_reward, rust_message) = rust_wrapper.step(&action);
            let rust_messages = rust_wrapper.last_messages();

            // RE-SYNC State before C step to ensure C is testing the SAME situation
            // Use rust_state (which was used to select the action)
            c_wrapper.set_state(
                rust_state.hp,
                rust_state.max_hp,
                rust_state.position.0,
                rust_state.position.1,
                rust_state.armor_class,
                rust_state.turn as i64,
            );

            // Execute on C
            let (c_reward, c_message) = c_wrapper.step(&action);
            let c_messages = c_wrapper.last_messages();

            // Get new states
            let new_rust_state = rust_wrapper.extract_state();
            let new_c_state = c_wrapper.extract_state();

            // Calculate combined reward (average of both)
            let combined_reward = (rust_reward + c_reward) / 2.0;

            // Update player Q-values
            self.player
                .update(&rust_state, &action, combined_reward, &new_rust_state);

            // Compare states
            let differences = compare_states(&new_rust_state, &new_c_state);

            // Record critical/major differences
            for diff in &differences {
                if diff.severity == DifferenceSeverity::Critical {
                    result.critical_differences.push((turn + 1, diff.clone()));
                } else if diff.severity == DifferenceSeverity::Major {
                    result.major_differences.push((turn + 1, diff.clone()));
                }
            }

            // Check for game over conditions
            if new_rust_state.is_dead {
                result.rust_death_turn = Some(turn + 1);
            }
            if new_c_state.is_dead {
                result.c_death_turn = Some(turn + 1);
            }
            if new_rust_state.is_won {
                result.rust_won = true;
            }
            if new_c_state.is_won {
                result.c_won = true;
            }

            // Update states
            rust_state = new_rust_state;
            c_state = new_c_state;
            result.total_reward += combined_reward;
            result.total_turns = turn + 1;

            // Collect messages
            if !rust_messages.is_empty() {
                result.messages.push(format!(
                    "[Turn {}] Rust: {}",
                    turn + 1,
                    rust_messages.join("; ")
                ));
            }
            if !c_messages.is_empty() {
                result
                    .messages
                    .push(format!("[Turn {}] C: {}", turn + 1, c_messages.join("; ")));
            }

            // Early exit if both died or both won
            if (result.rust_death_turn.is_some() && result.c_death_turn.is_some())
                || (result.rust_won && result.c_won)
            {
                break;
            }

            // Decay exploration periodically
            if turn > 0 && turn % self.config.report_interval == 0 {
                self.player.decay_exploration(0.95);

                // Print progress
                if self.config.verbose {
                    println!(
                        "Turn {}: reward={:.2}, exploration={:.2}, diffs={}/{}",
                        turn + 1,
                        result.total_reward,
                        self.player.exploration_rate(),
                        result.critical_differences.len(),
                        result.major_differences.len()
                    );
                }
            }
        }

        result
    }

    /// Run multiple sessions with different seeds
    pub fn run_multiple_sessions(
        &mut self,
        rust_loop: &mut nh_core::GameLoop,
        c_engine: &mut CGameEngine,
        seeds: &[u64],
    ) -> Vec<SessionResult> {
        seeds
            .iter()
            .map(|seed| {
                // Reset both games
                let state = nh_core::GameState::new_with_identity(
                    nh_core::GameRng::new(*seed),
                    "Hero".to_string(),
                    nh_core::player::Role::Tourist,
                    nh_core::player::Race::Human,
                    nh_core::player::Gender::Male,
                    nh_core::player::AlignmentType::Neutral,
                );
                let mut new_rust_loop = nh_core::GameLoop::new(state);
                let mut new_c_engine = CGameEngine::new();
                let _ = new_c_engine.init("Tourist", "Human", 0, 0);
                let _ = new_c_engine.reset(*seed);
                let _ = new_c_engine.generate_and_place();

                // Run session
                self.run_session(&mut new_rust_loop, &mut new_c_engine, *seed)
            })
            .collect()
    }
}

/// Orchestrator configuration
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub max_turns_per_session: u64,
    pub initial_exploration_rate: f64,
    pub report_interval: u64,
    pub verbose: bool,
    pub save_differences: bool,
    pub output_dir: Option<String>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_turns_per_session: 1000,
            initial_exploration_rate: 0.3,
            report_interval: 100,
            verbose: true,
            save_differences: false,
            output_dir: None,
        }
    }
}

/// Summary of multiple sessions
#[derive(Debug, Clone)]
pub struct MultiSessionSummary {
    pub total_sessions: usize,
    pub total_turns: u64,
    pub sessions_with_critical_diffs: usize,
    pub total_critical_differences: usize,
    pub total_major_differences: usize,
    pub average_reward: f64,
    pub common_differences: Vec<CommonDifference>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CommonDifference {
    pub category: DifferenceCategory,
    pub field: String,
    pub occurrences: usize,
    pub severity: DifferenceSeverity,
}

/// Run a quick comparison test
pub fn quick_comparison_test(
    rust_loop: &mut nh_core::GameLoop,
    c_engine: &mut CGameEngine,
    seed: u64,
    max_turns: u64,
) -> QuickTestResult {
    let config = OrchestratorConfig {
        max_turns_per_session: max_turns,
        verbose: false,
        ..Default::default()
    };

    let mut orchestrator = DualGameOrchestrator::new(config);
    let result = orchestrator.run_session(rust_loop, c_engine, seed);

    QuickTestResult {
        seed,
        total_turns: result.total_turns,
        rust_died: result.rust_death_turn.is_some(),
        c_died: result.c_death_turn.is_some(),
        critical_diffs: result.critical_differences.len(),
        major_diffs: result.major_differences.len(),
        total_reward: result.total_reward,
        passed: result.critical_differences.is_empty(),
    }
}

/// Result of a quick comparison test
#[derive(Debug, Clone)]
pub struct QuickTestResult {
    pub seed: u64,
    pub total_turns: u64,
    pub rust_died: bool,
    pub c_died: bool,
    pub critical_diffs: usize,
    pub major_diffs: usize,
    pub total_reward: f64,
    pub passed: bool,
}

impl QuickTestResult {
    pub fn summary(&self) -> String {
        format!(
            "Quick Test (seed={}): {} turns, reward={:.2}, diffs={}/{} ({})",
            self.seed,
            self.total_turns,
            self.total_reward,
            self.critical_diffs,
            self.major_diffs,
            if self.passed { "PASSED" } else { "FAILED" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::CGameEngine;
    use nh_core::{GameLoop, GameRng, GameState};

    #[test]
    fn test_orchestrator_config() {
        let config = OrchestratorConfig::default();
        assert_eq!(config.max_turns_per_session, 1000);
        assert_eq!(config.initial_exploration_rate, 0.3);
    }

    #[test]
    fn test_quick_test_result() {
        let result = QuickTestResult {
            seed: 12345,
            total_turns: 100,
            rust_died: false,
            c_died: false,
            critical_diffs: 0,
            major_diffs: 2,
            total_reward: 5.0,
            passed: true,
        };

        assert!(result.passed);
        assert_eq!(
            result.summary(),
            "Quick Test (seed=12345): 100 turns, reward=5.00, diffs=0/2 (PASSED)"
        );
    }

    #[test]
    fn test_virtual_player_config() {
        let config = VirtualPlayerConfig::default();
        assert_eq!(config.max_turns, 10000);
        assert_eq!(config.exploration_rate, 0.1);
        assert_eq!(config.report_interval, 100);
    }
}
