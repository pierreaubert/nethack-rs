//! Multi-Threaded RL Bot Training with Candle DQN (Metal GPU)
//!
//! Usage:
//!   cargo run --example rl_bot_parallel -p nh-test-compare -- [OPTIONS]

use nh_core::{GameLoop, GameRng, GameState};
use nh_player::state::common::{GameAction, UnifiedGameState};
use nh_player::state::rust_extractor::RustGameEngine;

use clap::{Parser, ValueEnum};
use rand::Rng;
use std::fs::{self, File};
use std::io::Write;
use std::sync::mpsc;
use std::sync::{Arc, Mutex, atomic::AtomicUsize};
use std::thread;
use std::time::Instant;

use candle_core::{DType, Device, Result, Tensor};
use candle_nn::{Linear, Module, Optimizer, SGD, VarBuilder, VarMap, linear};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum LogLevel {
    Info,
    Debug,
    Quiet,
}

#[derive(Debug, Clone, Copy, Parser)]
#[command(name = "rl_bot_parallel")]
#[command(version = "1.0")]
#[command(about = "Multi-threaded RL training for NetHack bot with Metal GPU", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "2000")]
    episodes: usize,
    #[arg(short, long, default_value = "8")]
    workers: usize,
    #[arg(short, long, default_value = "5000")]
    steps: usize,
    #[arg(short, long, default_value = "500")]
    interval: usize,
    #[arg(long, default_value = "0.995")]
    decay: f64,
    #[arg(long, default_value = "1.0")]
    exploration_start: f64,
    #[arg(long, default_value = "0.01")]
    exploration_end: f64,
    #[arg(long, default_value = "0.99")]
    gamma: f64,
    #[arg(long, default_value = "0.001")]
    lr: f64,
    #[arg(long, value_enum, default_value = "info")]
    log_level: LogLevel,
}

#[derive(Clone)]
struct ConfigArgs {
    num_episodes: usize,
    max_steps: usize,
    num_workers: usize,
    save_interval: usize,
    exploration_start: f64,
    exploration_end: f64,
    exploration_decay: f64,
    gamma: f64,
    learning_rate: f64,
}

#[derive(Clone)]
struct Transition {
    state: Vec<f32>,
    action: usize,
    reward: f32,
    next_state: Vec<f32>,
    done: bool,
}

struct ReplayMemory {
    memory: Vec<Transition>,
    capacity: usize,
}

impl ReplayMemory {
    fn new(capacity: usize) -> Self {
        Self {
            memory: Vec::with_capacity(capacity),
            capacity,
        }
    }

    fn push(&mut self, t: Transition) {
        if self.memory.len() >= self.capacity {
            self.memory.remove(0);
        }
        self.memory.push(t);
    }

    fn sample(&self, batch_size: usize) -> Vec<&Transition> {
        let mut rng = rand::thread_rng();
        let mut samples = Vec::with_capacity(batch_size);
        let len = self.memory.len();
        for _ in 0..batch_size.min(len) {
            let idx = rng.gen_range(0..len);
            samples.push(&self.memory[idx]);
        }
        samples
    }
}

struct QNetwork {
    fc1: Linear,
    fc2: Linear,
    fc3: Linear,
    varmap: VarMap,
    device: Device,
}

impl QNetwork {
    fn new(input_size: usize, hidden_size: usize, output_size: usize, device: &Device) -> Self {
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, device);

        let fc1 = linear(input_size, hidden_size, vb.pp("fc1")).unwrap();
        let fc2 = linear(hidden_size, hidden_size, vb.pp("fc2")).unwrap();
        let fc3 = linear(hidden_size, output_size, vb.pp("fc3")).unwrap();

        Self {
            fc1,
            fc2,
            fc3,
            varmap,
            device: device.clone(),
        }
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x = self.fc1.forward(x)?.relu()?;
        let x = self.fc2.forward(&x)?.relu()?;
        self.fc3.forward(&x)
    }

    fn predict(&self, state: &[f32]) -> Result<Vec<f32>> {
        let input = Tensor::from_slice(state, (1, state.len()), &self.device)?;
        let output = self.forward(&input)?;
        let output = output.reshape(())?;
        output.to_vec1()
    }

    fn load(&mut self, path: &str) {
        let _ = self.varmap.load(path);
    }

    fn save(&self, path: &str) {
        let _ = self.varmap.save(path);
    }
}

const STATE_SIZE: usize = 512;
const ACTION_SIZE: usize = 16;

fn state_to_features(state: &UnifiedGameState) -> Vec<f32> {
    let mut features = vec![0.0f32; STATE_SIZE];
    features[0] = (state.hp as f32 / state.max_hp as f32).clamp(0.0, 1.0);
    features[1] = (state.dungeon_depth as f32 / 30.0).clamp(0.0, 1.0);
    features[2] = (state.position.0 as f32 / 80.0).clamp(0.0, 1.0);
    features[3] = (state.position.1 as f32 / 25.0).clamp(0.0, 1.0);
    features[4] = (state.gold as f32 / 1000.0).clamp(0.0, 1.0);
    features[5] = (state.experience_level as f32 / 30.0).clamp(0.0, 1.0);
    features[6] = ((10 - state.armor_class) as f32 / 20.0).clamp(0.0, 1.0);
    features[7] = (state.energy as f32 / state.max_energy as f32).clamp(0.0, 1.0);
    features[8] = (state.status_effects.len() as f32 / 10.0).clamp(0.0, 1.0);

    let nearest = state
        .nearby_monsters
        .iter()
        .map(|m| {
            let dx = (m.position.0 - state.position.0) as f32;
            let dy = (m.position.1 - state.position.1) as f32;
            (dx * dx + dy * dy).sqrt()
        })
        .min_by(|a, b| a.partial_cmp(b).unwrap());
    features[9] = match nearest {
        Some(d) => (1.0 / (d + 1.0)).clamp(0.0, 1.0),
        None => 0.0,
    };
    features[10] = (state.inventory.len() as f32 / 20.0).clamp(0.0, 1.0);
    features[11] = (state.nearby_monsters.len() as f32 / 10.0).clamp(0.0, 1.0);
    features[12] = (state.turn as f32 / 10000.0).clamp(0.0, 1.0);
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

fn calculate_reward(
    old_state: &UnifiedGameState,
    new_state: &UnifiedGameState,
    _action: &GameAction,
    message: &str,
) -> f32 {
    let mut reward = 0.0f32;
    if new_state.is_dead {
        return -10.0;
    }
    if old_state.hp > new_state.hp {
        reward -= (old_state.hp - new_state.hp) as f32 * 0.1;
    }
    if new_state.hp > old_state.hp {
        reward += (new_state.hp - old_state.hp) as f32 * 0.2;
    }
    if new_state.dungeon_depth > old_state.dungeon_depth {
        reward += 5.0;
    }
    if new_state.gold > old_state.gold {
        reward += (new_state.gold - old_state.gold) as f32 * 0.01;
    }
    if new_state.experience_level > old_state.experience_level {
        reward += 3.0;
    }
    if new_state.is_won {
        reward += 100.0;
    }
    reward -= 0.01;
    let msg = message.to_lowercase();
    if msg.contains("hit") || msg.contains("damage") {
        reward += 0.5;
    }
    if msg.contains("kill") || msg.contains("defeat") {
        reward += 1.0;
    }
    if msg.contains("die") {
        reward -= 1.0;
    }
    reward
}

fn run_episode(
    seed: u64,
    exploration_rate: f64,
    max_steps: usize,
    q_network: &Arc<Mutex<QNetwork>>,
    _replay_memory: &Arc<Mutex<ReplayMemory>>,
    gamma: f64,
) -> (f32, u64, Vec<Transition>) {
    let rust_rng = GameRng::new(seed);
    let rust_state = GameState::new(rust_rng);
    let mut rust_loop = GameLoop::new(rust_state);
    let mut rust_engine = RustGameEngine::new(&mut rust_loop);

    let mut total_reward = 0.0f32;
    let mut transitions = Vec::new();
    let mut rng = rand::thread_rng();
    let mut current_state = state_to_features(&rust_engine.extract_state());
    let mut step = 0;

    while step < max_steps {
        let state = rust_engine.extract_state();
        let valid_actions = get_valid_actions(&state);
        let action_idx = if rng.r#gen::<f64>() < exploration_rate {
            rng.gen_range(0..valid_actions.len().min(ACTION_SIZE))
        } else {
            let net = q_network.lock().unwrap();
            let q_values = net.predict(&current_state).unwrap();
            let mut best_idx = 0;
            let mut best_q = f64::MIN;
            for (i, &q) in q_values
                .iter()
                .enumerate()
                .take(valid_actions.len().min(ACTION_SIZE))
            {
                if q as f64 > best_q {
                    best_q = q as f64;
                    best_idx = i;
                }
            }
            best_idx
        };

        let action = valid_actions[action_idx].clone();
        let (_reward, msg) = rust_engine.step(&action);
        let new_state = rust_engine.extract_state();
        let next_state_features = state_to_features(&new_state);
        let r = calculate_reward(&rust_engine.extract_state(), &new_state, &action, &msg);
        total_reward += r;
        let done = new_state.is_dead || new_state.is_won;

        transitions.push(Transition {
            state: current_state.clone(),
            action: action_idx,
            reward: r,
            next_state: next_state_features.clone(),
            done,
        });

        current_state = next_state_features;
        step += 1;
        let _ = rust_engine.step(&GameAction::Wait);
        if done {
            break;
        }
    }

    (total_reward, step as u64, transitions)
}

fn train_multithreaded(config: ConfigArgs, args: &Args) {
    let log_level = args.log_level;

    if matches!(log_level, LogLevel::Info | LogLevel::Debug) {
        println!("=== Multi-Threaded DQN Training (Metal GPU) ===");
        println!(
            "Episodes: {}, Workers: {}, Steps: {}",
            config.num_episodes, config.num_workers, config.max_steps
        );
        println!("Gamma: {}, LR: {}", config.gamma, config.learning_rate);
        println!();
    }

    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let checkpoints_dir = format!("{}/checkpoints", target_dir);
    fs::create_dir_all(&checkpoints_dir).ok();

    let device = Device::new_metal(0).expect("Failed to create Metal device");
    println!("Using device: {:?}", device);

    let q_network = QNetwork::new(STATE_SIZE, 256, ACTION_SIZE, &device);
    let network = Arc::new(Mutex::new(q_network));
    let replay_memory = Arc::new(Mutex::new(ReplayMemory::new(100000)));

    let completed_episodes = Arc::new(AtomicUsize::new(0));
    let start_time = Instant::now();

    let (episodes_tx, episodes_rx) = mpsc::channel();

    let mut handles = Vec::new();
    for worker_id in 0..config.num_workers {
        let config = config.clone();
        let tx = episodes_tx.clone();
        let network = network.clone();
        let replay_memory = replay_memory.clone();
        let completed = completed_episodes.clone();

        handles.push(thread::spawn(move || {
            let mut local_episode = 0;
            loop {
                let ep = completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if ep >= config.num_episodes {
                    break;
                }

                let seed = 42 + worker_id as u64 * 100000 + local_episode as u64;
                let exploration_rate = (config.exploration_start
                    * config.exploration_decay.powf(ep as f64))
                .max(config.exploration_end);

                let (reward, length, transitions) = run_episode(
                    seed,
                    exploration_rate,
                    config.max_steps,
                    &network,
                    &replay_memory,
                    config.gamma,
                );

                let _ = tx.send((ep, reward, length, transitions));
                local_episode += 1;
            }
        }));
    }

    let mut total_episodes = 0;
    let mut total_reward = 0.0f32;
    let mut deaths = 0;
    let mut batch_rewards = Vec::new();
    let mut optimizer = {
        let vars = network.lock().unwrap().varmap.all_vars();
        SGD::new(vars, config.learning_rate as f64).unwrap()
    };

    for (ep, reward, _length, transitions) in episodes_rx {
        total_episodes += 1;
        total_reward += reward;
        if reward < -50.0 {
            deaths += 1;
        }

        replay_memory.lock().unwrap().memory.extend(transitions);
        batch_rewards.push(reward);

        if total_episodes % 10 == 0 && total_episodes >= 100 {
            let memory = replay_memory.lock().unwrap();
            let samples = memory.sample(64);
            if !samples.is_empty() {
                let batch_size = samples.len();
                let mut states = Vec::with_capacity(batch_size * STATE_SIZE);
                let mut actions = Vec::with_capacity(batch_size);
                let mut targets = Vec::with_capacity(batch_size);

                for t in samples.iter() {
                    states.extend_from_slice(&t.state);
                    actions.push(t.action as u32);
                    let next_q_values = network.lock().unwrap().predict(&t.next_state).unwrap();
                    let max_next_q = if t.done {
                        0.0
                    } else {
                        next_q_values.iter().fold(f32::MIN, |a, b| a.max(*b))
                    };
                    targets.push(t.reward + config.gamma as f32 * max_next_q);
                }

                drop(memory);

                let states_tensor =
                    Tensor::from_slice(&states, (batch_size, STATE_SIZE), &device).unwrap();
                let actions_tensor = Tensor::from_slice(&actions, (batch_size,), &device).unwrap();
                let targets_tensor = Tensor::from_slice(&targets, (batch_size,), &device).unwrap();

                let predictions = network.lock().unwrap().forward(&states_tensor).unwrap();
                let indices = actions_tensor.reshape((batch_size, 1)).unwrap();
                let selected_q = predictions.gather(&indices, 1).unwrap().squeeze(1).unwrap();
                let loss = selected_q
                    .sub(&targets_tensor)
                    .unwrap()
                    .sqr()
                    .unwrap()
                    .mean_all()
                    .unwrap();

                optimizer.backward_step(&loss).unwrap();
            }
        }

        if total_episodes % 50 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let eps_per_sec = total_episodes as f64 / elapsed;
            let avg_batch: f64 =
                batch_rewards.iter().map(|&r| r as f64).sum::<f64>() / batch_rewards.len() as f64;

            if total_episodes % 100 == 0 {
                let src_varmap = network.lock().unwrap().varmap.clone();
                let mut dst_net = network.lock().unwrap();
                dst_net.varmap = src_varmap;
            }

            if matches!(log_level, LogLevel::Info | LogLevel::Debug) {
                println!(
                    "Episode {:5}: {:.1} eps/s | Avg: {:7.2} | Deaths: {} | Eps: {:.3}",
                    total_episodes,
                    eps_per_sec,
                    avg_batch,
                    deaths,
                    (config.exploration_start
                        * config.exploration_decay.powf(total_episodes as f64))
                    .max(config.exploration_end)
                );
            }

            batch_rewards.clear();
        }

        if total_episodes % config.save_interval == 0 {
            let len = batch_rewards.len().max(1) as f64;
            let avg_batch: f64 = batch_rewards.iter().map(|&r| r as f64).sum::<f64>() / len;
            let path = format!("{}/model.safetensors", checkpoints_dir);
            network.lock().unwrap().save(&path);
            if matches!(log_level, LogLevel::Info | LogLevel::Debug) {
                println!(
                    "  -> Saved checkpoint at episode {} (avg reward: {:.2})",
                    total_episodes, avg_batch
                );
            }
        }

        if total_episodes >= config.num_episodes {
            break;
        }
    }

    for handle in handles {
        let _ = handle.join();
    }

    let total_time = start_time.elapsed().as_secs_f64();
    let avg_reward = total_reward as f64 / total_episodes as f64;

    if matches!(log_level, LogLevel::Info | LogLevel::Debug) {
        println!("\n=== Training Complete ===");
        println!("Total time: {:.1} seconds", total_time);
        println!("Total episodes: {}", total_episodes);
        println!("Episodes/second: {:.1}", total_episodes as f64 / total_time);
        println!("Average reward: {:.2}", avg_reward);
        println!(
            "Deaths: {} ({:.1}%)",
            deaths,
            deaths as f64 / total_episodes as f64 * 100.0
        );
        println!("\nCheckpoints saved to: {}/", checkpoints_dir);
    } else {
        println!(
            "{:.1} eps/sec, {:.2} avg reward, {} deaths",
            total_episodes as f64 / total_time,
            avg_reward,
            deaths
        );
    }
}

fn main() {
    let args = Args::parse();

    let config = ConfigArgs {
        num_episodes: args.episodes,
        max_steps: args.steps,
        num_workers: args.workers,
        save_interval: args.interval,
        exploration_start: args.exploration_start,
        exploration_end: args.exploration_end,
        exploration_decay: args.decay,
        gamma: args.gamma,
        learning_rate: args.lr,
    };

    train_multithreaded(config, &args);
}
