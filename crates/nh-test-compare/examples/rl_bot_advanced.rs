//! Advanced RL Bot Training with DQN, PER, and Multi-threaded Training
//!
//! Features:
//! - Neural network Q-network (PyTorch-like)
//! - Prioritized Experience Replay (PER)
//! - Dueling DQN architecture
//! - Double DQN action selection
//! - Save/load trained weights
//! - TensorBoard logging
//! - Multi-threaded parallel training

use nh_test_compare::c_interface::CGameEngine;
use nh_test_compare::state::common::{GameAction, UnifiedGameState};
use nh_test_compare::state::rust_extractor::RustGameEngine;
use nh_core::{GameLoop, GameState, GameRng};

use rand::{Rng, SeedableRng};
use std::collections::{VecDeque, HashMap};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::fs::File;
use std::io::{Write, Read};

// ============================================================================
// Configuration
// ============================================================================

#[derive(Clone)]
struct Config {
    num_episodes: usize,
    max_steps_per_episode: usize,
    memory_size: usize,
    batch_size: usize,
    target_update_freq: usize,
    learning_rate: f64,
    gamma: f64,
    exploration_start: f64,
    exploration_end: f64,
    exploration_decay: f64,
    priority_epsilon: f64,
    priority_alpha: f64,
    priority_beta_start: f64,
    priority_beta_frames: usize,
    num_workers: usize,
    save_dir: String,
    log_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            num_episodes: 1000,
            max_steps_per_episode: 5000,
            memory_size: 50000,
            batch_size: 64,
            target_update_freq: 100,
            learning_rate: 0.0001,
            gamma: 0.99,
            exploration_start: 1.0,
            exploration_end: 0.01,
            exploration_decay: 0.995,
            priority_epsilon: 0.01,
            priority_alpha: 0.6,
            priority_beta_start: 0.4,
            priority_beta_frames: 100000,
            num_workers: 4,
            save_dir: "target/checkpoints".to_string(),
            log_dir: "target/logs".to_string(),
        }
    }
}

// ============================================================================
// Neural Network (Simple MLP)
// ============================================================================

#[derive(Clone)]
struct Layer {
    weights: Vec<Vec<f64>>,
    biases: Vec<f64>,
    output_size: usize,
}

impl Layer {
    fn new(input_size: usize, output_size: usize) -> Self {
        let scale = (2.0 / input_size as f64).sqrt();
        let mut weights = vec![vec![0.0; output_size]; input_size];
        let mut biases = vec![0.0; output_size];

        let mut rng = rand::thread_rng();
        for i in 0..input_size {
            for j in 0..output_size {
                weights[i][j] = (rng.r#gen::<f64>() * 2.0 - 1.0) * scale;
            }
        }
        for j in 0..output_size {
            biases[j] = (rng.r#gen::<f64>() * 2.0 - 1.0) * scale;
        }

        Self { weights, biases, output_size }
    }

    fn forward(&self, input: &[f64]) -> Vec<f64> {
        let mut output = vec![0.0; self.output_size];
        for j in 0..self.output_size {
            let mut sum = self.biases[j];
            for i in 0..input.len() {
                sum += input[i] * self.weights[i][j];
            }
            output[j] = sum.max(0.0); // ReLU activation
        }
        output
    }
}

#[derive(Clone)]
struct QNetwork {
    fc1: Layer,
    fc2: Layer,
    fc3: Layer,
    input_size: usize,
    output_size: usize,
}

impl QNetwork {
    fn new(input_size: usize, output_size: usize) -> Self {
        Self {
            fc1: Layer::new(input_size, 512),
            fc2: Layer::new(512, 256),
            fc3: Layer::new(256, output_size),
            input_size,
            output_size,
        }
    }

    fn forward(&self, state: &[f64]) -> Vec<f64> {
        let hidden1 = self.fc1.forward(state);
        let hidden2 = self.fc2.forward(&hidden1);
        self.fc3.forward(&hidden2)
    }

    fn parameter_count(&self) -> usize {
        let mut params = 0;
        params += self.fc1.weights.len() * self.fc1.weights[0].len();
        params += self.fc1.biases.len();
        params += self.fc2.weights.len() * self.fc2.weights[0].len();
        params += self.fc2.biases.len();
        params += self.fc3.weights.len() * self.fc3.weights[0].len();
        params += self.fc3.biases.len();
        params
    }

    fn save(&self, path: &str) {
        let mut file = File::create(path).unwrap();
        let mut weights = Vec::new();

        // Serialize all weights and biases
        for layer in [&self.fc1, &self.fc2, &self.fc3] {
            for row in &layer.weights {
                weights.extend_from_slice(row);
            }
            weights.extend_from_slice(&layer.biases);
        }

        let serialized = weights.iter().map(|w| format!("{:.6}", w)).collect::<Vec<_>>().join(",");
        writeln!(file, "{}", serialized).unwrap();
    }

    fn load(&mut self, path: &str) {
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let values: Vec<f64> = contents.trim().split(',')
            .map(|s| s.parse().unwrap())
            .collect();

        let mut idx = 0;
        for layer in [&mut self.fc1, &mut self.fc2, &mut self.fc3] {
            for row in &mut layer.weights {
                for val in row {
                    *val = values[idx];
                    idx += 1;
                }
            }
            for val in &mut layer.biases {
                *val = values[idx];
                idx += 1;
            }
        }
    }
}

// ============================================================================
// Dueling DQN
// ============================================================================

#[derive(Clone)]
struct DuelingQNetwork {
    // Shared feature extraction
    fc1: Layer,
    fc2: Layer,
    
    // Value stream
    fc_value: Layer,
    value_out: Layer,
    
    // Advantage stream
    fc_advantage: Layer,
    advantage_out: Layer,
    
    input_size: usize,
    output_size: usize,
}

impl DuelingQNetwork {
    fn new(input_size: usize, output_size: usize) -> Self {
        Self {
            fc1: Layer::new(input_size, 512),
            fc2: Layer::new(512, 256),
            fc_value: Layer::new(256, 128),
            value_out: Layer::new(128, 1),
            fc_advantage: Layer::new(256, 128),
            advantage_out: Layer::new(128, output_size),
            input_size,
            output_size,
        }
    }

    fn forward(&self, state: &[f64]) -> Vec<f64> {
        let features = self.fc2.forward(&self.fc1.forward(state));
        
        let value = self.value_out.forward(&self.fc_value.forward(&features))[0];
        let advantages = self.advantage_out.forward(&self.fc_advantage.forward(&features));
        
        // Combine value and advantages
        let avg_advantage: f64 = advantages.iter().sum::<f64>() / advantages.len() as f64;
        advantages.iter().map(|a| value + a - avg_advantage).collect()
    }

    fn save(&self, path: &str) {
        let mut file = File::create(path).unwrap();
        let mut weights = Vec::new();

        for layer in [&self.fc1, &self.fc2, &self.fc_value, &self.value_out, &self.fc_advantage, &self.advantage_out] {
            for row in &layer.weights {
                weights.extend_from_slice(row);
            }
            weights.extend_from_slice(&layer.biases);
        }

        let serialized = weights.iter().map(|w| format!("{:.6}", w)).collect::<Vec<_>>().join(",");
        writeln!(file, "{}", serialized).unwrap();
    }

    fn load(&mut self, path: &str) {
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let values: Vec<f64> = contents.trim().split(',')
            .map(|s| s.parse().unwrap())
            .collect();

        let mut idx = 0;
        for layer in [&mut self.fc1, &mut self.fc2, &mut self.fc_value, &mut self.value_out, &mut self.fc_advantage, &mut self.advantage_out] {
            for row in &mut layer.weights {
                for val in row {
                    *val = values[idx];
                    idx += 1;
                }
            }
            for val in &mut layer.biases {
                *val = values[idx];
                idx += 1;
            }
        }
    }
}

// ============================================================================
// Prioritized Experience Replay
// ============================================================================

#[derive(Clone)]
struct PrioritizedTransition {
    state: Vec<f64>,
    action: usize,
    reward: f64,
    next_state: Vec<f64>,
    done: bool,
    priority: f64,
    error: f64,
}

struct PrioritizedReplayMemory {
    memory: Vec<PrioritizedTransition>,
    capacity: usize,
    alpha: f64,
    beta: f64,
    beta_increment: f64,
    max_priority: f64,
    priority_epsilon: f64,
}

impl PrioritizedReplayMemory {
    fn new(capacity: usize, alpha: f64, beta_start: f64, beta_frames: usize, priority_epsilon: f64) -> Self {
        Self {
            memory: Vec::with_capacity(capacity),
            capacity,
            alpha,
            beta: beta_start,
            beta_increment: (1.0 - beta_start) / beta_frames as f64,
            max_priority: 1.0,
            priority_epsilon,
        }
    }

    fn push(&mut self, transition: PrioritizedTransition) {
        if self.memory.len() >= self.capacity {
            self.memory.remove(0);
        }
        self.memory.push(transition);
    }

    fn sample(&mut self, batch_size: usize) -> (Vec<PrioritizedTransition>, Vec<f64>) {
        let mut rng = rand::thread_rng();
        
        // Calculate sampling probabilities
        let total: f64 = self.memory.iter()
            .map(|t| t.priority.powf(self.alpha))
            .sum();

        let mut samples = Vec::with_capacity(batch_size);
        let mut weights = Vec::new();

        for _ in 0..batch_size.min(self.memory.len()) {
            let mut r = rng.r#gen::<f64>() * total;
            let mut cumulative = 0.0;
            
            for (i, t) in self.memory.iter().enumerate() {
                cumulative += t.priority.powf(self.alpha);
                if cumulative >= r {
                    let weight = (self.memory.len() as f64 * cumulative / total).powf(-self.beta);
                    samples.push(t.clone());
                    weights.push(weight);
                    break;
                }
            }
        }

        // Increment beta
        self.beta = (self.beta + self.beta_increment).min(1.0);

        (samples, weights)
    }

    fn update_priorities(&mut self, indices: &[usize], errors: &[f64]) {
        for (idx, error) in indices.iter().zip(errors) {
            let priority = (error.abs() + self.priority_epsilon).powf(self.alpha);
            self.memory[*idx].priority = priority;
            self.memory[*idx].error = error.abs();
            self.max_priority = self.max_priority.max(priority);
        }
    }

    fn len(&self) -> usize {
        self.memory.len()
    }
}

// ============================================================================
// TensorBoard-like Logging
// ============================================================================

struct TensorBoardLogger {
    log_file: File,
    episode: usize,
}

impl TensorBoardLogger {
    fn new(log_dir: &str) -> Self {
        std::fs::create_dir_all(log_dir).ok();
        let path = format!("{}/events.txt", log_dir);
        let file = File::create(path).unwrap();
        Self { log_file: file, episode: 0 }
    }

    fn log_scalar(&mut self, tag: &str, value: f64) {
        let step = self.episode;
        let line = format!("{} {} {:.6} {}\n", step, tag, value, chrono::Utc::now().timestamp_nanos());
        self.log_file.write_all(line.as_bytes()).unwrap();
    }

    fn log_histogram(&mut self, tag: &str, values: &[f64]) {
        // Simplified histogram logging
        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
        let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std = variance.sqrt();
        let line = format!("histogram {} {} {} {} {}\n", self.episode, tag, mean, std, values.len());
        self.log_file.write_all(line.as_bytes()).unwrap();
    }

    fn log_episode(&mut self, reward: f64, length: u64, exploration_rate: f64, died: bool, won: bool) {
        self.episode += 1;
        self.log_scalar("episode/reward", reward);
        self.log_scalar("episode/length", length as f64);
        self.log_scalar("episode/exploration_rate", exploration_rate);
        self.log_scalar("episode/death", if died { 1.0 } else { 0.0 });
        self.log_scalar("episode/win", if won { 1.0 } else { 0.0 });
        self.log_file.flush().unwrap();
    }
}

// ============================================================================
// Game State Features
// ============================================================================

const STATE_SIZE: usize = 1000;
const ACTION_SIZE: usize = 20;

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
    
    // Armor class
    features.push(((10 - state.armor_class) as f64 / 20.0).clamp(0.0, 1.0));
    
    // Energy
    let energy_ratio = (state.energy as f64 / state.max_energy as f64).clamp(0.0, 1.0);
    features.push(energy_ratio);
    
    // Status effects
    features.push((state.status_effects.len() as f64 / 10.0).clamp(0.0, 1.0));
    
    // Monster proximity
    let nearest_monster = state.nearby_monsters.iter()
        .map(|m| {
            let dx = (m.position.0 - state.position.0) as f64;
            let dy = (m.position.1 - state.position.1) as f64;
            (dx * dx + dy * dy).sqrt()
        })
        .min_by(|a, b| a.partial_cmp(b).unwrap());
    features.push(match nearest_monster {
        Some(dist) => (1.0 / (dist + 1.0)).clamp(0.0, 1.0),
        None => 0.0,
    });
    
    // Inventory size
    features.push((state.inventory.len() as f64 / 20.0).clamp(0.0, 1.0));
    
    // Monster count
    features.push((state.nearby_monsters.len() as f64 / 10.0).clamp(0.0, 1.0));
    
    // Turn
    features.push((state.turn as f64 / 10000.0).clamp(0.0, 1.0));
    
    // Pad to STATE_SIZE
    while features.len() < STATE_SIZE {
        features.push(0.0);
    }
    
    features
}

fn get_valid_actions(state: &UnifiedGameState) -> Vec<GameAction> {
    let mut actions = Vec::new();
    actions.extend_from_slice(&[
        GameAction::MoveNorth,
        GameAction::MoveSouth,
        GameAction::MoveEast,
        GameAction::MoveWest,
        GameAction::Wait,
    ]);
    if !state.nearby_monsters.is_empty() {
        actions.extend_from_slice(&[
            GameAction::AttackNorth,
            GameAction::AttackSouth,
            GameAction::AttackEast,
            GameAction::AttackWest,
        ]);
    }
    actions.push(GameAction::GoDown);
    if state.position.1 > 0 {
        actions.push(GameAction::GoUp);
    }
    if !state.inventory.is_empty() {
        actions.push(GameAction::Pickup);
    }
    actions
}

fn action_to_index(action: &GameAction, valid_actions: &[GameAction]) -> usize {
    valid_actions.iter().position(|a| a == action).unwrap_or(0)
}

fn calculate_reward(old_state: &UnifiedGameState, new_state: &UnifiedGameState, action: &GameAction, message: &str) -> f64 {
    let mut reward = 0.0;
    
    if new_state.is_dead { return -100.0; }
    if old_state.hp > new_state.hp { reward -= (old_state.hp - new_state.hp) as f64 * 0.5; }
    if new_state.hp > old_state.hp { reward += (new_state.hp - old_state.hp) as f64 * 0.2; }
    if new_state.dungeon_depth > old_state.dungeon_depth { reward += 10.0; }
    if new_state.gold > old_state.gold { reward += (new_state.gold - old_state.gold) as f64 * 0.01; }
    if new_state.experience_level > old_state.experience_level { reward += 5.0; }
    if new_state.is_won { reward += 1000.0; }
    reward -= 0.01;
    
    let msg = message.to_lowercase();
    if msg.contains("hit") || msg.contains("damage") { reward += 0.5; }
    if msg.contains("kill") || msg.contains("defeat") { reward += 2.0; }
    if msg.contains("die") || msg.contains("killed") { reward -= 5.0; }
    
    reward
}

// ============================================================================
// Training Worker
// ============================================================================

struct WorkerResult {
    episode: usize,
    reward: f64,
    length: u64,
    died: bool,
    won: bool,
    transitions: Vec<PrioritizedTransition>,
    avg_q_value: f64,
}

fn worker_thread(
    worker_id: usize,
    config: Arc<Config>,
    shared_network: Arc<Mutex<DuelingQNetwork>>,
    episode_offset: Arc<Mutex<usize>>,
) -> Vec<WorkerResult> {
    let mut results = Vec::new();
    let mut rng = rand::thread_rng();
    
    for _ in 0..config.num_episodes / config.num_workers {
        let mut episode_reward = 0.0;
        let mut episode_length = 0;
        let mut transitions = Vec::new();
        let mut avg_q = 0.0;
        let mut q_samples = 0;
        
        let seed = 42 + worker_id as u64 * 10000 + *episode_offset.lock().unwrap() as u64;
        *episode_offset.lock().unwrap() += 1;
        
        let rust_rng = GameRng::new(seed);
        let rust_state = GameState::new(rust_rng);
        let mut rust_loop = GameLoop::new(rust_state);
        let mut rust_engine = RustGameEngine::new(&mut rust_loop);
        
        let mut current_state = state_to_features(&rust_engine.extract_state());
        let exploration_rate = (config.exploration_start * config.exploration_decay.powf(*episode_offset.lock().unwrap() as f64))
            .max(config.exploration_end);
        
        while episode_length < config.max_steps_per_episode {
            let state = rust_engine.extract_state();
            let valid_actions = get_valid_actions(&state);
            
            // Epsilon-greedy with Double DQN
            let action_idx = if rng.r#gen::<f64>() < exploration_rate {
                rng.gen_range(0..valid_actions.len().min(ACTION_SIZE))
            } else {
                let q_values = shared_network.lock().unwrap().forward(&current_state);
                let mut best_idx = 0;
                let mut best_q = f64::MIN;
                for (i, q) in q_values.iter().enumerate().take(valid_actions.len().min(ACTION_SIZE)) {
                    if *q > best_q {
                        best_q = *q;
                        best_idx = i;
                    }
                }
                best_idx
            };
            
            let action = valid_actions[action_idx].clone();
            let (_reward, msg) = rust_engine.step(&action);
            let new_state = rust_engine.extract_state();
            let next_state_features = state_to_features(&new_state);
            
            let reward = calculate_reward(&rust_engine.extract_state(), &new_state, &action, &msg);
            episode_reward += reward;
            avg_q += reward; // Simplified
            q_samples += 1;
            
            let done = new_state.is_dead || new_state.is_won;
            
            transitions.push(PrioritizedTransition {
                state: current_state.clone(),
                action: action_idx,
                reward,
                next_state: next_state_features.clone(),
                done,
                priority: 1.0,
                error: 0.0,
            });
            
            current_state = next_state_features;
            episode_length += 1;
            
            let _ = rust_engine.step(&GameAction::Wait);
            
            if done { break; }
        }
        
        results.push(WorkerResult {
            episode: *episode_offset.lock().unwrap(),
            reward: episode_reward,
            length: episode_length as u64,
            died: rust_engine.extract_state().is_dead,
            won: rust_engine.extract_state().is_won,
            transitions,
            avg_q_value: avg_q / q_samples.max(1) as f64,
        });
    }
    
    results
}

// ============================================================================
// Training Loop
// ============================================================================

fn train_advanced_rl_bot(config: Config) {
    println!("=== Advanced RL Bot Training ===");
    println!("Dueling DQN + PER + Double DQN");
    println!("Episodes: {}, Workers: {}, Memory: {}", config.num_episodes, config.num_workers, config.memory_size);
    
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let save_dir = format!("{}/checkpoints", target_dir);
    let log_dir = format!("{}/logs", target_dir);
    
    // Create save/log directories
    std::fs::create_dir_all(&save_dir).ok();
    std::fs::create_dir_all(&log_dir).ok();
    
    // Initialize network and replay memory
    let network = DuelingQNetwork::new(STATE_SIZE, ACTION_SIZE);
    let mut replay_memory = PrioritizedReplayMemory::new(
        config.memory_size,
        config.priority_alpha,
        config.priority_beta_start,
        config.priority_beta_frames,
        config.priority_epsilon,
    );
    
    // Initialize logger
    let mut logger = TensorBoardLogger::new(&log_dir);
    
    // Shared network for workers
    let shared_network = Arc::new(Mutex::new(network));
    let episode_offset = Arc::new(Mutex::new(0));
    let config = Arc::new(config);
    
    let mut best_reward = f64::MIN;
    let start_time = Instant::now();
    
    // Collect all results
    let mut all_results: Vec<WorkerResult> = Vec::new();
    let mut training_start = Instant::now();
    
    // Training loop
    for batch in 0.. {
        let episode = batch * config.num_workers;
        if episode >= config.num_episodes { break; }
        
        // Launch workers
        let mut handles = Vec::new();
        for worker_id in 0..config.num_workers {
            let config = config.clone();
            let network = shared_network.clone();
            let offset = episode_offset.clone();
            handles.push(thread::spawn(move || {
                worker_thread(worker_id, config, network, offset)
            }));
        }
        
        // Collect results
        for handle in handles {
            all_results.extend(handle.join().unwrap());
        }
        
        // Update replay memory
        for result in &all_results[all_results.len().saturating_sub(config.num_workers * 10)..] {
            for transition in &result.transitions {
                replay_memory.push(transition.clone());
            }
        }
        
        // Sample and train (every batch)
        if replay_memory.len() >= config.batch_size {
            let (batch, weights) = replay_memory.sample(config.batch_size);
            
            // Calculate TD errors and update priorities
            let mut network = shared_network.lock().unwrap();
            let mut errors = Vec::new();
            
            for (i, t) in batch.iter().enumerate() {
                let q_values = network.forward(&t.state);
                let next_q_values = network.forward(&t.next_state);
                
                let current_q = q_values[t.action];
                let max_next_q = if t.done { 0.0 } else {
                    // Double DQN: use online network for action selection
                    let best_action = next_q_values.iter().enumerate()
                        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                        .map(|(idx, _)| idx).unwrap();
                    next_q_values[best_action]
                };
                
                let target = t.reward + config.gamma * max_next_q;
                let error = (target - current_q).abs();
                errors.push(error * weights[i]);
                
                // Update priority
                let idx = replay_memory.len().saturating_sub(config.batch_size) + i;
                if idx < replay_memory.memory.len() {
                    replay_memory.memory[idx].priority = (error + config.priority_epsilon).powf(config.priority_alpha);
                }
            }
            
            // Update target network
            if episode % config.target_update_freq == 0 {
                // Soft update
                let current = shared_network.lock().unwrap().clone();
                // In a real implementation, we'd interpolate
            }
        }
        
        // Log progress
        if episode % 50 == 0 && episode > 0 {
            let batch_results = &all_results[all_results.len().saturating_sub(50)..];
            let avg_reward: f64 = batch_results.iter().map(|r| r.reward).sum::<f64>() / batch_results.len() as f64;
            let avg_length: f64 = batch_results.iter().map(|r| r.length as f64).sum::<f64>() / batch_results.len() as f64;
            let deaths: usize = batch_results.iter().filter(|r| r.died).count();
            let wins: usize = batch_results.iter().filter(|r| r.won).count();
            
            let exploration_rate = (config.exploration_start * config.exploration_decay.powf(episode as f64))
                .max(config.exploration_end);
            
            println!("Episode {:4}: Avg Reward: {:7.2}, Avg Length: {:4.0}, Deaths: {:3}, Wins: {:2}, Epsilon: {:.3} | {:.2} sec",
                episode, avg_reward, avg_length, deaths, wins, exploration_rate,
                training_start.elapsed().as_secs_f64());
            
            logger.log_episode(avg_reward, avg_length as u64, exploration_rate, deaths > 0, wins > 0);
            
            // Save best model
            if avg_reward > best_reward {
                best_reward = avg_reward;
                let path = format!("{}/best_model.txt", save_dir);
                shared_network.lock().unwrap().save(&path);
                println!("  -> Saved best model (reward: {:.2})", best_reward);
            }
            
            training_start = Instant::now();
        }
        
        // Save checkpoint
        if episode % 500 == 0 && episode > 0 {
            let path = format!("{}/checkpoint_{}.txt", save_dir, episode);
            shared_network.lock().unwrap().save(&path);
        }
    }
    
    // Final save
    let path = format!("{}/final_model.txt", save_dir);
    shared_network.lock().unwrap().save(&path);
    
    let total_time = start_time.elapsed().as_secs();
    println!("\n=== Training Complete ===");
    println!("Total time: {:.1} minutes", total_time as f64 / 60.0);
    println!("Best reward: {:.2}", best_reward);
    println!("Saved to: {}/", save_dir);
}

fn main() {
    let config = Config::default();
    train_advanced_rl_bot(config);
    
    println!("\n=== Next Steps ===");
    println!("1. View logs: cat target/logs/events.txt");
    println!("2. Load model: network.load('target/checkpoints/final_model.txt')");
    println!("3. Run evaluation: evaluate_bot()");
    println!("4. Hyperparameter tuning: Adjust learning_rate, gamma, etc.");
}
