//! RL Bot Training Example
//!
//! This example trains a reinforcement learning bot to play NetHack
//! using Q-learning with experience replay and target networks.

use nh_core::{GameLoop, GameRng, GameState};
use nh_player::ffi::CGameEngine;
use nh_player::state::c_extractor::CGameWrapper;
use nh_player::state::common::{GameAction, UnifiedGameState};
use nh_player::state::rust_extractor::RustGameEngine;

use rand::{Rng, SeedableRng};
use std::collections::{HashMap, VecDeque};

const STATE_SIZE: usize = 1000;
const ACTION_SIZE: usize = 20;
const MEMORY_SIZE: usize = 10000;
const BATCH_SIZE: usize = 32;
const TARGET_UPDATE_FREQ: usize = 100;

#[derive(Clone)]
struct Transition {
    state: Vec<f64>,
    action: usize,
    reward: f64,
    next_state: Vec<f64>,
    done: bool,
}

struct ReplayMemory {
    memory: VecDeque<Transition>,
    capacity: usize,
}

impl ReplayMemory {
    fn new(capacity: usize) -> Self {
        Self {
            memory: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn push(&mut self, transition: Transition) {
        if self.memory.len() >= self.capacity {
            self.memory.pop_front();
        }
        self.memory.push_back(transition);
    }

    fn sample(&self, batch_size: usize) -> Vec<Transition> {
        let mut rng = rand::thread_rng();
        let mut samples = Vec::with_capacity(batch_size);
        let len = self.memory.len();

        for _ in 0..batch_size.min(len) {
            let idx = rng.gen_range(0..len);
            if let Some(transition) = self.memory.get(idx).cloned() {
                samples.push(transition);
            }
        }
        samples
    }

    fn len(&self) -> usize {
        self.memory.len()
    }
}

struct QNetwork {
    weights: Vec<Vec<f64>>, // Simple linear model for demo
    input_size: usize,
    output_size: usize,
}

impl QNetwork {
    fn new(input_size: usize, output_size: usize) -> Self {
        let weights = vec![vec![0.0; output_size]; input_size];
        Self {
            weights,
            input_size,
            output_size,
        }
    }

    fn forward(&self, state: &[f64]) -> Vec<f64> {
        let mut output = vec![0.0; self.output_size];
        for i in 0..self.output_size {
            for j in 0..state.len().min(self.input_size) {
                output[i] += state[j] * self.weights[j][i];
            }
        }
        output
    }

    fn update(&mut self, transitions: &[Transition], learning_rate: f64, gamma: f64) {
        for t in transitions {
            let q_values = self.forward(&t.state);
            let next_q_values = self.forward(&t.next_state);
            let max_next_q = if t.done {
                0.0
            } else {
                next_q_values.iter().fold(f64::MIN, |m, v| v.max(m))
            };

            let target = t.reward + gamma * max_next_q;
            let current = q_values[t.action];

            // Gradient update (simplified)
            let error = target - current;
            for i in 0..t.state.len().min(self.input_size) {
                self.weights[i][t.action] += learning_rate * error * t.state[i];
            }
        }
    }

    fn copy_from(&mut self, other: &QNetwork) {
        self.weights = other.weights.clone();
    }
}

struct TrainingMetrics {
    episode_rewards: Vec<f64>,
    episode_lengths: Vec<u64>,
    deaths: usize,
    wins: usize,
    exploration_rates: Vec<f64>,
}

impl TrainingMetrics {
    fn new() -> Self {
        Self {
            episode_rewards: Vec::new(),
            episode_lengths: Vec::new(),
            deaths: 0,
            wins: 0,
            exploration_rates: Vec::new(),
        }
    }

    fn record_episode(&mut self, reward: f64, length: u64, died: bool, won: bool) {
        self.episode_rewards.push(reward);
        self.episode_lengths.push(length);
        if died {
            self.deaths += 1;
        }
        if won {
            self.wins += 1;
        }
    }

    fn summary(&self) -> String {
        format!(
            "Episodes: {}, Avg Reward: {:.2}, Avg Length: {:.1}, Deaths: {}, Wins: {}",
            self.episode_rewards.len(),
            self.episode_rewards.iter().sum::<f64>() / self.episode_rewards.len().max(1) as f64,
            self.episode_lengths.iter().sum::<u64>() as f64
                / self.episode_lengths.len().max(1) as f64,
            self.deaths,
            self.wins
        )
    }
}

fn state_to_features(state: &UnifiedGameState) -> Vec<f64> {
    let mut features = Vec::with_capacity(STATE_SIZE);

    // HP (normalized)
    let hp_ratio = (state.hp as f64 / state.max_hp as f64).clamp(0.0, 1.0);
    features.push(hp_ratio);

    // Dungeon depth
    features.push((state.dungeon_depth as f64 / 30.0).clamp(0.0, 1.0));

    // Position
    features.push((state.position.0 as f64 / 80.0).clamp(0.0, 1.0));
    features.push((state.position.1 as f64 / 25.0).clamp(0.0, 1.0));

    // Gold
    features.push((state.gold as f64 / 1000.0).clamp(0.0, 1.0));

    // XP level
    features.push((state.experience_level as f64 / 30.0).clamp(0.0, 1.0));

    // Armor class (inverted, lower is better)
    features.push(((10 - state.armor_class) as f64 / 20.0).clamp(0.0, 1.0));

    // Energy
    let energy_ratio = (state.energy as f64 / state.max_energy as f64).clamp(0.0, 1.0);
    features.push(energy_ratio);

    // Status effects (binary flags)
    let status_count = state.status_effects.len() as f64 / 10.0;
    features.push(status_count.clamp(0.0, 1.0));

    // Monster proximity
    let nearest_monster = state
        .nearby_monsters
        .iter()
        .map(|m| {
            let dx = (m.position.0 - state.position.0) as f64;
            let dy = (m.position.1 - state.position.1) as f64;
            (dx * dx + dy * dy).sqrt()
        })
        .min_by(|a, b| a.partial_cmp(b).unwrap());
    if let Some(dist) = nearest_monster {
        features.push((1.0 / (dist + 1.0)).clamp(0.0, 1.0));
    } else {
        features.push(0.0);
    }

    // Inventory size
    features.push((state.inventory.len() as f64 / 20.0).clamp(0.0, 1.0));

    // Pad to STATE_SIZE
    while features.len() < STATE_SIZE {
        features.push(0.0);
    }

    features
}

fn get_valid_actions(state: &UnifiedGameState) -> Vec<GameAction> {
    let mut actions = Vec::new();

    // Movement (always valid)
    actions.extend_from_slice(&[
        GameAction::MoveNorth,
        GameAction::MoveSouth,
        GameAction::MoveEast,
        GameAction::MoveWest,
        GameAction::Wait,
    ]);

    // Combat actions if monsters nearby
    if !state.nearby_monsters.is_empty() {
        actions.extend_from_slice(&[
            GameAction::AttackNorth,
            GameAction::AttackSouth,
            GameAction::AttackEast,
            GameAction::AttackWest,
        ]);
    }

    // Navigation
    actions.push(GameAction::GoDown);
    if state.position.1 > 0 {
        actions.push(GameAction::GoUp);
    }

    // Interactions
    if !state.inventory.is_empty() {
        actions.push(GameAction::Pickup);
    }

    actions
}

fn action_to_index(action: &GameAction, valid_actions: &[GameAction]) -> usize {
    valid_actions.iter().position(|a| a == action).unwrap_or(0)
}

fn index_to_action(idx: usize, valid_actions: &[GameAction]) -> GameAction {
    valid_actions[idx.min(valid_actions.len() - 1)].clone()
}

fn calculate_reward(
    old_state: &UnifiedGameState,
    new_state: &UnifiedGameState,
    action: &GameAction,
    message: &str,
) -> f64 {
    let mut reward = 0.0;

    // Survival
    if new_state.is_dead {
        return -100.0;
    }
    if old_state.hp > new_state.hp {
        reward -= (old_state.hp - new_state.hp) as f64 * 0.5;
    }

    // Healing
    if new_state.hp > old_state.hp {
        reward += (new_state.hp - old_state.hp) as f64 * 0.2;
    }

    // Progression
    if new_state.dungeon_depth > old_state.dungeon_depth {
        reward += 10.0;
    }

    // Gold
    if new_state.gold > old_state.gold {
        reward += (new_state.gold - old_state.gold) as f64 * 0.01;
    }

    // XP
    if new_state.experience_level > old_state.experience_level {
        reward += 5.0;
    }

    // Victory
    if new_state.is_won {
        reward += 1000.0;
    }

    // Time penalty
    reward -= 0.01;

    // Combat rewards/penalties
    let msg = message.to_lowercase();
    if msg.contains("hit") || msg.contains("damage") {
        reward += 0.5;
    }
    if msg.contains("kill") || msg.contains("defeat") {
        reward += 2.0;
    }
    if msg.contains("die") || msg.contains("killed") {
        reward -= 5.0;
    }

    reward
}

fn train_episode(
    q_network: &mut QNetwork,
    replay_memory: &mut ReplayMemory,
    metrics: &mut TrainingMetrics,
    seed: u64,
    exploration_rate: f64,
    target_network: &QNetwork,
) -> (f64, u64, bool, bool) {
    // Initialize games
    let rust_rng = GameRng::new(seed);
    let rust_state = GameState::new(rust_rng);
    let mut rust_loop = GameLoop::new(rust_state);
    let mut rust_engine = RustGameEngine::new(&mut rust_loop);

    let mut c_engine = CGameEngine::new();
    c_engine
        .init("Tourist", "Human", 0, 0)
        .expect("Failed to init C engine");
    let mut c_wrapper = CGameWrapper::new(&mut c_engine);

    let mut total_reward = 0.0;
    let mut turn = 0;
    let mut rng = rand::thread_rng();
    let mut died = false;
    let mut won = false;

    let mut current_state = state_to_features(&rust_engine.extract_state());

    while turn < 10000 {
        let state = rust_engine.extract_state();
        let valid_actions = get_valid_actions(&state);

        // Epsilon-greedy action selection
        let action = if rng.r#gen::<f64>() < exploration_rate {
            valid_actions[rng.gen_range(0..valid_actions.len())].clone()
        } else {
            let q_values = q_network.forward(&current_state);
            let mut best_idx = 0;
            let mut best_q = f64::MIN;
            for (i, _) in valid_actions.iter().enumerate().take(ACTION_SIZE) {
                let q = if i < q_values.len() { q_values[i] } else { 0.0 };
                if q > best_q {
                    best_q = q;
                    best_idx = i;
                }
            }
            valid_actions[best_idx].clone()
        };

        // Execute action
        let (_rust_reward, rust_msg) = rust_engine.step(&action);
        let new_state = rust_engine.extract_state();
        let next_state_features = state_to_features(&new_state);

        // Calculate reward
        let prev_state = rust_engine.extract_state();
        let reward = calculate_reward(&prev_state, &new_state, &action, &rust_msg);
        total_reward += reward;

        // Check terminal conditions
        let done = new_state.is_dead || new_state.is_won;
        if new_state.is_dead {
            died = true;
        }
        if new_state.is_won {
            won = true;
        }

        // Store transition
        let action_idx = action_to_index(&action, &valid_actions);
        replay_memory.push(Transition {
            state: current_state.clone(),
            action: action_idx,
            reward,
            next_state: next_state_features.clone(),
            done,
        });

        // Sample batch and update
        if replay_memory.len() >= BATCH_SIZE {
            let batch = replay_memory.sample(BATCH_SIZE);
            q_network.update(&batch, 0.01, 0.99);
        }

        current_state = next_state_features;
        turn += 1;

        // Safety wait
        let _ = rust_engine.step(&GameAction::Wait);
        let _ = c_wrapper.step(&GameAction::Wait);

        if done {
            break;
        }
    }

    metrics.record_episode(total_reward, turn, died, won);
    (total_reward, turn, died, won)
}

fn train_rl_bot(num_episodes: usize) {
    println!("=== RL Bot Training ===");
    println!(
        "Episodes: {}, Memory: {}, Batch: {}",
        num_episodes, MEMORY_SIZE, BATCH_SIZE
    );

    let mut q_network = QNetwork::new(STATE_SIZE, ACTION_SIZE);
    let mut target_network = QNetwork::new(STATE_SIZE, ACTION_SIZE);
    let mut replay_memory = ReplayMemory::new(MEMORY_SIZE);
    let mut metrics = TrainingMetrics::new();

    let mut exploration_rate = 1.0;
    let exploration_decay = 0.995;
    let min_exploration = 0.05;

    for episode in 0..num_episodes {
        let seed = 42 + episode as u64 * 1000;

        let (reward, length, _died, _won) = train_episode(
            &mut q_network,
            &mut replay_memory,
            &mut metrics,
            seed,
            exploration_rate,
            &target_network,
        );

        // Update target network periodically
        if episode % TARGET_UPDATE_FREQ == 0 {
            target_network.copy_from(&q_network);
        }

        // Decay exploration
        exploration_rate = (exploration_rate * exploration_decay).max(min_exploration);

        // Progress reporting
        if episode % 50 == 0 || episode == num_episodes - 1 {
            println!(
                "Episode {:4}: Reward: {:7.2}, Length: {:4}, Epsilon: {:.3} | {}",
                episode,
                reward,
                length,
                exploration_rate,
                metrics.summary()
            );
        }
    }

    println!("\n=== Training Complete ===");
    println!("{}", metrics.summary());
    println!("\nFinal Q-network ready for evaluation!");
}

fn evaluate_bot(q_network: &QNetwork, num_episodes: usize) {
    println!("\n=== Bot Evaluation ===");

    let mut total_wins = 0;
    let mut total_deaths = 0;
    let mut total_turns = 0;

    for episode in 0..num_episodes {
        let seed = 999 + episode as u64 * 1000;

        let rust_rng = GameRng::new(seed);
        let rust_state = GameState::new(rust_rng);
        let mut rust_loop = GameLoop::new(rust_state);
        let mut rust_engine = RustGameEngine::new(&mut rust_loop);

        let mut turn = 0;
        let mut won = false;
        let mut died = false;

        while turn < 5000 {
            let state = rust_engine.extract_state();
            let valid_actions = get_valid_actions(&state);

            // Greedy action selection
            let q_values = q_network.forward(&state_to_features(&state));
            let mut best_idx = 0;
            let mut best_q = f64::MIN;
            for (i, _) in valid_actions.iter().enumerate().take(ACTION_SIZE) {
                let q = if i < q_values.len() { q_values[i] } else { 0.0 };
                if q > best_q {
                    best_q = q;
                    best_idx = i;
                }
            }
            let action = valid_actions[best_idx].clone();

            // Execute
            let (_reward, _msg) = rust_engine.step(&action);
            let new_state = rust_engine.extract_state();

            if new_state.is_dead {
                died = true;
                total_deaths += 1;
                break;
            }
            if new_state.is_won {
                won = true;
                total_wins += 1;
                break;
            }

            let _ = rust_engine.step(&GameAction::Wait);
            turn += 1;
        }

        total_turns += turn;

        if episode % 100 == 0 {
            println!(
                "Episode {:4}: {} turns, {} won, {} died",
                episode,
                turn,
                if won { "yes" } else { "no" },
                if died { "yes" } else { "no" }
            );
        }
    }

    println!("\n=== Evaluation Summary ===");
    println!("Episodes: {}", num_episodes);
    println!("Total Turns: {}", total_turns);
    println!(
        "Avg Turns per Episode: {:.1}",
        total_turns as f64 / num_episodes as f64
    );
    println!(
        "Wins: {} ({:.1}%)",
        total_wins,
        total_wins as f64 / num_episodes as f64 * 100.0
    );
    println!(
        "Deaths: {} ({:.1}%)",
        total_deaths,
        total_deaths as f64 / num_episodes as f64 * 100.0
    );
}

fn main() {
    // Training phase
    train_rl_bot(500);

    // Evaluation phase (would use trained network)
    // evaluate_bot(&q_network, 100);

    println!("\n=== Next Steps ===");
    println!("1. Save trained Q-network weights");
    println!("2. Add evaluation phase with saved weights");
    println!("3. Implement more sophisticated network (neural net)");
    println!("4. Add prioritized experience replay");
    println!("5. Implement dueling DQN architecture");
}
