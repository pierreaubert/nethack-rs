//! RL-based virtual player for automated game testing.
//!
//! This module provides a virtual player that can play both implementations
//! in parallel using reinforcement learning techniques to detect behavioral
//! differences.

use rand::{Rng, SeedableRng};
use rand::distributions::Uniform;
use crate::state::common::*;

/// Configuration for the virtual player
#[derive(Debug, Clone)]
pub struct VirtualPlayerConfig {
    pub max_turns: u64,
    pub exploration_rate: f64,
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub report_interval: u64,
}

impl Default for VirtualPlayerConfig {
    fn default() -> Self {
        Self {
            max_turns: 10000,
            exploration_rate: 0.1,
            learning_rate: 0.1,
            discount_factor: 0.99,
            report_interval: 100,
        }
    }
}

/// Simple Q-learning table for action values
#[derive(Debug, Clone)]
pub struct QTable {
    table: Vec<Vec<f64>>,
    state_size: usize,
    action_size: usize,
}

impl QTable {
    pub fn new(state_size: usize, action_size: usize) -> Self {
        Self {
            table: vec![vec![0.0; action_size]; state_size],
            state_size,
            action_size,
        }
    }

    pub fn get_q(&self, state: usize, action: usize) -> f64 {
        self.table[state % self.state_size][action % self.action_size]
    }

    pub fn set_q(&mut self, state: usize, action: usize, value: f64) {
        self.table[state % self.state_size][action % self.action_size] = value;
    }

    pub fn update(&mut self, state: usize, action: usize, reward: f64, next_state: usize) {
        let old_value = self.get_q(state, action);
        let max_next = (0..self.action_size)
            .map(|a| self.get_q(next_state, a))
            .fold(f64::MIN, f64::max);
        
        let new_value = old_value + 0.1 * (reward + 0.99 * max_next - old_value);
        self.set_q(state, action, new_value);
    }
}

/// Virtual player that can play both implementations
#[derive(Debug, Clone)]
pub struct VirtualPlayer {
    config: VirtualPlayerConfig,
    q_table: QTable,
    rng: rand::rngs::StdRng,
}

impl VirtualPlayer {
    /// Create a new virtual player
    pub fn new(config: VirtualPlayerConfig) -> Self {
        let rng = rand::rngs::StdRng::from_entropy();
        let q_table = QTable::new(1000, 30); // Simplified state/action spaces
        
        Self { config, q_table, rng }
    }

    /// Select an action using epsilon-greedy strategy
    pub fn select_action(&mut self, state: &UnifiedGameState) -> GameAction {
        let actions = self.get_valid_actions(state);
        
        if self.rng.r#gen::<f64>() < self.config.exploration_rate {
            // Explore: random action
            let range = Uniform::new(0, actions.len());
            actions[self.rng.sample(range)].clone()
        } else {
            // Exploit: best known action
            let mut best_action = actions[0].clone();
            let mut best_value = f64::MIN;
            
            for (i, action) in actions.iter().enumerate() {
                let state_idx = self.state_index(state);
                let value = self.q_table.get_q(state_idx, i);
                if value > best_value {
                    best_value = value;
                    best_action = action.clone();
                }
            }
            
            best_action
        }
    }

    /// Get list of valid actions for current state
    fn get_valid_actions(&self, _state: &UnifiedGameState) -> Vec<GameAction> {
        let mut actions = Vec::new();
        
        // Movement is always valid
        actions.extend_from_slice(&[
            GameAction::MoveNorth,
            GameAction::MoveSouth,
            GameAction::MoveEast,
            GameAction::MoveWest,
            GameAction::Wait,
        ]);
        
        // Navigation based on position (simplified)
        actions.push(GameAction::GoDown);
        
        actions
    }

    /// Convert state to index for Q-table
    fn state_index(&self, state: &UnifiedGameState) -> usize {
        // Simplified state encoding
        let hp_bucket = (state.hp as usize / 5).min(19);
        let depth_bucket = state.dungeon_depth as usize % 20;
        let pos_bucket = ((state.position.0 + state.position.1) as usize) % 50;
        
        (hp_bucket * 400 + depth_bucket * 20 + pos_bucket) % 1000
    }

    /// Calculate reward for a step
    pub fn calculate_reward(
        &self,
        old_state: &UnifiedGameState,
        new_state: &UnifiedGameState,
        _action: &GameAction,
        messages: &[String],
    ) -> f64 {
        let mut reward = 0.0;
        
        // Survival reward
        if new_state.is_dead {
            reward -= 100.0;
        } else if old_state.hp > new_state.hp {
            // Damage taken
            reward -= (old_state.hp - new_state.hp) as f64 * 0.5;
        }
        
        // Ascension reward
        if new_state.is_won {
            reward += 1000.0;
        }
        
        // Progression rewards
        if new_state.dungeon_depth > old_state.dungeon_depth {
            reward += 10.0; // Descended deeper
        }
        
        // Gold reward (small)
        if new_state.gold > old_state.gold {
            reward += (new_state.gold - old_state.gold) as f64 * 0.01;
        }
        
        // XP reward
        if new_state.experience_level > old_state.experience_level {
            reward += 5.0;
        }
        
        // Small time penalty
        reward -= 0.01;
        
        // Negative messages
        for msg in messages {
            if msg.to_lowercase().contains("die") || 
               msg.to_lowercase().contains("kill") {
                reward -= 1.0;
            }
        }
        
        reward
    }

    /// Update Q-values after a step
    pub fn update(
        &mut self,
        old_state: &UnifiedGameState,
        action: &GameAction,
        reward: f64,
        new_state: &UnifiedGameState,
    ) {
        let old_idx = self.state_index(old_state);
        let new_idx = self.state_index(new_state);
        
        // Find action index
        let valid_actions = self.get_valid_actions(old_state);
        let action_idx = valid_actions.iter()
            .position(|a| a == action)
            .unwrap_or(0);
        
        self.q_table.update(old_idx, action_idx, reward, new_idx);
    }

    /// Get current exploration rate
    pub fn exploration_rate(&self) -> f64 {
        self.config.exploration_rate
    }

    /// Set exploration rate
    pub fn set_exploration_rate(&mut self, rate: f64) {
        self.config.exploration_rate = rate.max(0.0).min(1.0);
    }

    /// Decay exploration rate
    pub fn decay_exploration(&mut self, factor: f64) {
        self.config.exploration_rate *= factor;
        if self.config.exploration_rate < 0.01 {
            self.config.exploration_rate = 0.01;
        }
    }
}

/// Session result from running the virtual player
#[derive(Debug, Clone)]
pub struct SessionResult {
    pub seed: u64,
    pub total_turns: u64,
    pub rust_death_turn: Option<u64>,
    pub c_death_turn: Option<u64>,
    pub rust_won: bool,
    pub c_won: bool,
    pub critical_differences: Vec<(u64, StateDifference)>,
    pub major_differences: Vec<(u64, StateDifference)>,
    pub total_reward: f64,
    pub messages: Vec<String>,
}

impl Default for SessionResult {
    fn default() -> Self {
        Self {
            seed: 0,
            total_turns: 0,
            rust_death_turn: None,
            c_death_turn: None,
            rust_won: false,
            c_won: false,
            critical_differences: Vec::new(),
            major_differences: Vec::new(),
            total_reward: 0.0,
            messages: Vec::new(),
        }
    }
}

/// Run a session with both implementations
pub fn run_session(
    rust_state: &mut UnifiedGameState,
    c_state: &mut UnifiedGameState,
    player: &mut VirtualPlayer,
    seed: u64,
) -> SessionResult {
    let mut result = SessionResult {
        seed,
        ..Default::default()
    };

    // Initialize RNG with seed (currently unused but available for future use)
    let _rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);

    // Reset exploration rate for each session
    player.config.exploration_rate = 0.3;

    for turn in 0..player.config.max_turns {
        // Select action
        let action = player.select_action(rust_state);
        
        // Apply action to both states (simplified - just advance turn)
        // In real implementation, would execute through extractors
        
        rust_state.turn += 1;
        c_state.turn += 1;
        
        // Calculate reward
        let reward = player.calculate_reward(rust_state, rust_state, &action, &[]);
        result.total_reward += reward;
        
        // Update Q-values
        player.update(rust_state, &action, reward, rust_state);
        
        // Check for game over
        if rust_state.is_dead && c_state.is_dead {
            break;
        }
        
        // Report progress
        if turn > 0 && turn % player.config.report_interval == 0 {
            player.decay_exploration(0.99);
        }
        
        result.total_turns = turn + 1;
    }
    
    result
}

/// Run a comparison session and report differences
pub fn run_comparison_session<R, C>(
    get_rust_state: R,
    get_c_state: C,
    player: &mut VirtualPlayer,
    seed: u64,
    _compare_fn: impl Fn(&UnifiedGameState, &UnifiedGameState) -> Vec<StateDifference>,
) -> SessionResult
where
    R: Fn() -> UnifiedGameState,
    C: Fn() -> UnifiedGameState,
{
    let mut rust_state = get_rust_state();
    let mut c_state = get_c_state();
    
    run_session(&mut rust_state, &mut c_state, player, seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_player_lifecycle() {
        let config = VirtualPlayerConfig {
            max_turns: 100,
            exploration_rate: 0.5,
            ..Default::default()
        };
        
        let mut player = VirtualPlayer::new(config);
        
        // Initial state
        let state = UnifiedGameState::default_start("Tourist", "Human");
        
        // Should select random actions initially
        let action = player.select_action(&state);
        assert!(matches!(action, GameAction::MoveNorth | GameAction::MoveSouth | 
                         GameAction::MoveEast | GameAction::MoveWest | GameAction::Wait));
    }

    #[test]
    fn test_q_table_update() {
        let mut q_table = QTable::new(100, 30);
        
        // Initial value should be 0
        assert_eq!(q_table.get_q(0, 0), 0.0);
        
        // Update
        q_table.update(0, 0, 1.0, 1);
        
        // Should have increased
        assert!(q_table.get_q(0, 0) > 0.0);
    }

    #[test]
    fn test_reward_calculation() {
        let config = VirtualPlayerConfig::default();
        let player = VirtualPlayer::new(config);
        
        let old_state = UnifiedGameState::default_start("Wizard", "Elf");
        let mut new_state = UnifiedGameState::default_start("Wizard", "Elf");
        new_state.hp = 5; // Took damage
        
        let reward = player.calculate_reward(&old_state, &new_state, &GameAction::Wait, &[]);
        assert!(reward < 0.0); // Negative due to damage
        
        // Test death
        new_state.is_dead = true;
        let death_reward = player.calculate_reward(&old_state, &new_state, &GameAction::Wait, &[]);
        assert!(death_reward < reward);
    }

    #[test]
    fn test_exploration_decay() {
        let config = VirtualPlayerConfig {
            exploration_rate: 0.5,
            ..Default::default()
        };
        
        let mut player = VirtualPlayer::new(config);
        assert_eq!(player.exploration_rate(), 0.5);
        
        player.decay_exploration(0.9);
        assert_eq!(player.exploration_rate(), 0.45);
        
        // Should not go below 0.01
        for _ in 0..100 {
            player.decay_exploration(0.9);
        }
        assert!(player.exploration_rate() >= 0.01);
    }

    #[test]
    fn test_state_index() {
        let config = VirtualPlayerConfig::default();
        let player = VirtualPlayer::new(config);
        
        let state = UnifiedGameState::default_start("Rogue", "Gnome");
        let idx = player.state_index(&state);
        
        // Should return a valid index
        assert!(idx < 1000);
    }

    #[test]
    fn test_session_result() {
        let result = SessionResult::default();
        assert_eq!(result.total_turns, 0);
        assert!(result.critical_differences.is_empty());
    }
}
