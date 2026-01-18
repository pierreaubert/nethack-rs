//! Multi-Threaded RL Bot Training with CLI Arguments
//!
//! Usage:
//!   cargo run --example rl_bot_parallel -p nh-test-compare -- [OPTIONS]
//!
//! Options:
//!   -e, --episodes <N>     Number of episodes to run (default: 2000)
//!   -w, --workers <N>      Number of worker threads (default: 8)
//!   -s, --steps <N>        Max steps per episode (default: 5000)
//!   -b, --batch <N>        Batch size for replay memory (default: 64)
//!   -i, --interval <N>     Checkpoint save interval (default: 500)
//!   --help                 Show this help message

use nh_test_compare::state::common::{GameAction, UnifiedGameState};
use nh_test_compare::state::rust_extractor::RustGameEngine;
use nh_core::{GameLoop, GameState, GameRng};

use std::sync::{Arc, Mutex, atomic::AtomicUsize};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;
use std::fs::{self, File};
use std::io::Write;
use std::collections::VecDeque;
use rand::Rng;
use clap::{Parser, ValueEnum};

// ============================================================================
// CLI Arguments
// ============================================================================

#[derive(Debug, Clone, Copy, ValueEnum)]
enum LogLevel {
    Info,
    Debug,
    Quiet,
}

#[derive(Debug, Clone, Copy, Parser)]
#[command(name = "rl_bot_parallel")]
#[command(author = "NetHack RL")]
#[command(version = "1.0")]
#[command(about = "Multi-threaded RL training for NetHack bot", long_about = None)]
struct Args {
    /// Number of episodes to run
    #[arg(short, long, default_value = "2000")]
    episodes: usize,

    /// Number of worker threads
    #[arg(short, long, default_value = "8")]
    workers: usize,

    /// Max steps per episode
    #[arg(short, long, default_value = "5000")]
    steps: usize,

    /// Batch size for replay memory
    #[arg(short, long, default_value = "64")]
    batch: usize,

    /// Checkpoint save interval
    #[arg(short, long, default_value = "500")]
    interval: usize,

    /// Replay memory capacity
    #[arg(long, default_value = "100000")]
    memory: usize,

    /// Exploration decay rate
    #[arg(long, default_value = "0.995")]
    decay: f64,

    /// Starting exploration rate
    #[arg(long, default_value = "1.0")]
    exploration_start: f64,

    /// Ending exploration rate
    #[arg(long, default_value = "0.01")]
    exploration_end: f64,

    /// Discount factor (gamma)
    #[arg(long, default_value = "0.99")]
    gamma: f64,

    /// Log level
    #[arg(long, value_enum, default_value = "info")]
    log_level: LogLevel,
}

// ============================================================================
// Configuration
// ============================================================================

#[derive(Clone)]
struct Config {
    num_episodes: usize,
    max_steps: usize,
    memory_size: usize,
    batch_size: usize,
    num_workers: usize,
    save_interval: usize,
    exploration_start: f64,
    exploration_end: f64,
    exploration_decay: f64,
    gamma: f64,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Self {
            num_episodes: args.episodes,
            max_steps: args.steps,
            memory_size: args.memory,
            batch_size: args.batch,
            num_workers: args.workers,
            save_interval: args.interval,
            exploration_start: args.exploration_start,
            exploration_end: args.exploration_end,
            exploration_decay: args.decay,
            gamma: args.gamma,
        }
    }
}

// ============================================================================
// Message Types
// ============================================================================

#[derive(Clone)]
struct WorkerMessage {
    worker_id: usize,
    episode: usize,
    reward: f64,
    length: u64,
    died: bool,
    won: bool,
    transitions: Vec<TransitionData>,
    avg_q: f64,
}

#[derive(Clone)]
struct TransitionData {
    state: Vec<f64>,
    action: usize,
    reward: f64,
    next_state: Vec<f64>,
    done: bool,
}

const STATE_SIZE: usize = 512;
const ACTION_SIZE: usize = 16;

// ============================================================================
// Replay Memory
// ============================================================================

struct ReplayMemory {
    memory: VecDeque<TransitionData>,
    capacity: usize,
}

impl ReplayMemory {
    fn new(capacity: usize) -> Self {
        Self {
            memory: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    
    fn push(&mut self, transitions: &[TransitionData]) {
        for t in transitions {
            if self.memory.len() >= self.capacity {
                self.memory.pop_front();
            }
            self.memory.push_back(t.clone());
        }
    }
}

// ============================================================================
// Multi-Threaded Training
// ============================================================================

fn train_multithreaded(config: Config, args: &Args) {
    let log_level = args.log_level;
    
    if matches!(log_level, LogLevel::Info | LogLevel::Debug) {
        println!("=== Multi-Threaded RL Training ===");
        println!("Episodes: {}, Workers: {}, Steps: {}", config.num_episodes, config.num_workers, config.max_steps);
        println!("Memory: {}, Batch: {}, Interval: {}", config.memory_size, config.batch_size, config.save_interval);
        println!("Exploration: {:.3} -> {:.3} (decay: {:.4})", config.exploration_start, config.exploration_end, config.exploration_decay);
        println!();
    }
    
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let checkpoints_dir = format!("{}/checkpoints", target_dir);
    fs::create_dir_all(&checkpoints_dir).ok();
    
    let replay_memory = Arc::new(Mutex::new(ReplayMemory::new(config.memory_size)));
    let active_workers = Arc::new(AtomicUsize::new(0));
    let completed_episodes = Arc::new(AtomicUsize::new(0));
    let start_time = Instant::now();
    
    let (episodes_tx, episodes_rx) = mpsc::channel();
    
    let mut handles = Vec::new();
    for worker_id in 0..config.num_workers {
        let config = config.clone();
        let tx = episodes_tx.clone();
        let active_workers = active_workers.clone();
        let completed_episodes = completed_episodes.clone();
        
        handles.push(thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut local_episode = 0;
            
            loop {
                if completed_episodes.load(std::sync::atomic::Ordering::Relaxed) >= config.num_episodes {
                    break;
                }
                
                active_workers.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                
                let seed = 42 + worker_id as u64 * 100000 + local_episode as u64;
                let global_ep = completed_episodes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                
                if global_ep >= config.num_episodes {
                    active_workers.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                    break;
                }
                
                let exploration_rate = (config.exploration_start * config.exploration_decay.powf(global_ep as f64))
                    .max(config.exploration_end);
                
                let (reward, length, died, won, transitions, avg_q) = run_episode_threaded(
                    seed, exploration_rate, config.max_steps,
                );
                
                let msg = WorkerMessage {
                    worker_id,
                    episode: global_ep,
                    reward,
                    length,
                    died,
                    won,
                    transitions,
                    avg_q,
                };
                
                let _ = tx.send(msg);
                
                local_episode += 1;
                active_workers.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            }
        }));
    }
    
    let mut total_episodes = 0;
    let mut total_reward = 0.0;
    let mut deaths = 0;
    let mut wins = 0;
    let mut batch_rewards = Vec::new();
    
    for msg in episodes_rx {
        total_episodes += 1;
        total_reward += msg.reward;
        if msg.died { deaths += 1; }
        if msg.won { wins += 1; }
        
        replay_memory.lock().unwrap().push(&msg.transitions);
        
        batch_rewards.push(msg.reward);
        
        if total_episodes % 50 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let eps_per_sec = total_episodes as f64 / elapsed;
            let active = active_workers.load(std::sync::atomic::Ordering::Relaxed);
            let avg_batch: f64 = batch_rewards.iter().sum::<f64>() / batch_rewards.len() as f64;
            
            if matches!(log_level, LogLevel::Info | LogLevel::Debug) {
                println!("Episode {:5}: {:.1} eps/sec | Active: {} | Avg Reward: {:7.2} | Deaths: {} | Wins: {}",
                    total_episodes, eps_per_sec, active, avg_batch, deaths, wins);
            }
            
            batch_rewards.clear();
        }
        
        if total_episodes % config.save_interval == 0 {
            let path = format!("{}/model_episode_{}.txt", checkpoints_dir, total_episodes);
            let mut file = File::create(&path).unwrap();
            let avg_reward_batch: f64 = batch_rewards.iter().sum::<f64>() / batch_rewards.len() as f64;
            
            writeln!(file, "# NetHack RL Bot Checkpoint").unwrap();
            writeln!(file, "episode: {}", total_episodes).unwrap();
            writeln!(file, "workers: {}", config.num_workers).unwrap();
            writeln!(file, "avg_reward: {:.4}", avg_reward_batch).unwrap();
            writeln!(file, "total_episodes: {}", total_episodes).unwrap();
            writeln!(file, "eps_per_sec: {:.1}", total_episodes as f64 / start_time.elapsed().as_secs_f64()).unwrap();
            
            if matches!(log_level, LogLevel::Info | LogLevel::Debug) {
                println!("  -> Saved checkpoint at episode {}", total_episodes);
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
    let avg_reward = total_reward / total_episodes as f64;
    
    if matches!(log_level, LogLevel::Info | LogLevel::Debug) {
        println!("\n=== Training Complete ===");
        println!("Total time: {:.1} seconds ({:.2} minutes)", total_time, total_time / 60.0);
        println!("Total episodes: {}", total_episodes);
        println!("Episodes/second: {:.1}", total_episodes as f64 / total_time);
        println!("Average reward: {:.2}", avg_reward);
        println!("Total deaths: {} ({:.1}%)", deaths, deaths as f64 / total_episodes as f64 * 100.0);
        println!("Total wins: {} ({:.1}%)", wins, wins as f64 / total_episodes as f64 * 100.0);
        println!("\nCheckpoints saved to: {}/", checkpoints_dir);
    } else {
        println!("{:.1} eps/sec, {:.2} avg reward, {} deaths, {} wins",
            total_episodes as f64 / total_time, avg_reward, deaths, wins);
    }
}

fn run_episode_threaded(
    seed: u64,
    exploration_rate: f64,
    max_steps: usize,
) -> (f64, u64, bool, bool, Vec<TransitionData>, f64) {
    let rust_rng = GameRng::new(seed);
    let rust_state = GameState::new(rust_rng);
    let mut rust_loop = GameLoop::new(rust_state);
    let mut rust_engine = RustGameEngine::new(&mut rust_loop);
    
    let mut total_reward = 0.0;
    let mut transitions = Vec::new();
    let mut avg_q = 0.0;
    let mut q_samples = 0;
    let mut rng = rand::thread_rng();
    
    let mut current_state = state_to_features(&rust_engine.extract_state());
    let mut step = 0;
    
    while step < max_steps {
        let state = rust_engine.extract_state();
        let valid_actions = get_valid_actions(&state);
        
        let action_idx = if rng.r#gen::<f64>() < exploration_rate {
            rng.gen_range(0..valid_actions.len().min(ACTION_SIZE))
        } else {
            rng.gen_range(0..valid_actions.len().min(ACTION_SIZE))
        };
        
        let action = valid_actions[action_idx].clone();
        let (_reward, msg) = rust_engine.step(&action);
        let new_state = rust_engine.extract_state();
        let next_state_features = state_to_features(&new_state);
        
        let r = calculate_reward(&rust_engine.extract_state(), &new_state, &action, &msg);
        total_reward += r;
        avg_q += r;
        q_samples += 1;
        
        let done = new_state.is_dead || new_state.is_won;
        
        transitions.push(TransitionData {
            state: current_state.clone(),
            action: action_idx,
            reward: r,
            next_state: next_state_features.clone(),
            done,
        });
        
        current_state = next_state_features;
        step += 1;
        
        let _ = rust_engine.step(&GameAction::Wait);
        
        if done { break; }
    }
    
    (
        total_reward,
        step as u64,
        rust_engine.extract_state().is_dead,
        rust_engine.extract_state().is_won,
        transitions,
        avg_q / q_samples.max(1) as f64,
    )
}

// ============================================================================
// Game State Features
// ============================================================================

fn state_to_features(state: &UnifiedGameState) -> Vec<f64> {
    let mut features = Vec::with_capacity(STATE_SIZE);
    
    features.push((state.hp as f64 / state.max_hp as f64).clamp(0.0, 1.0));
    features.push((state.dungeon_depth as f64 / 30.0).clamp(0.0, 1.0));
    features.push((state.position.0 as f64 / 80.0).clamp(0.0, 1.0));
    features.push((state.position.1 as f64 / 25.0).clamp(0.0, 1.0));
    features.push((state.gold as f64 / 1000.0).clamp(0.0, 1.0));
    features.push((state.experience_level as f64 / 30.0).clamp(0.0, 1.0));
    features.push(((10 - state.armor_class) as f64 / 20.0).clamp(0.0, 1.0));
    features.push((state.energy as f64 / state.max_energy as f64).clamp(0.0, 1.0));
    features.push((state.status_effects.len() as f64 / 10.0).clamp(0.0, 1.0));
    
    let nearest = state.nearby_monsters.iter()
        .map(|m| {
            let dx = (m.position.0 - state.position.0) as f64;
            let dy = (m.position.1 - state.position.1) as f64;
            (dx * dx + dy * dy).sqrt()
        })
        .min_by(|a, b| a.partial_cmp(b).unwrap());
    features.push(match nearest {
        Some(d) => (1.0 / (d + 1.0)).clamp(0.0, 1.0),
        None => 0.0,
    });
    
    features.push((state.inventory.len() as f64 / 20.0).clamp(0.0, 1.0));
    features.push((state.nearby_monsters.len() as f64 / 10.0).clamp(0.0, 1.0));
    features.push((state.turn as f64 / 10000.0).clamp(0.0, 1.0));
    
    while features.len() < STATE_SIZE {
        features.push(0.0);
    }
    
    features
}

fn get_valid_actions(state: &UnifiedGameState) -> Vec<GameAction> {
    let mut actions = Vec::new();
    actions.extend_from_slice(&[
        GameAction::MoveNorth, GameAction::MoveSouth,
        GameAction::MoveEast, GameAction::MoveWest,
        GameAction::Wait,
    ]);
    if !state.nearby_monsters.is_empty() {
        actions.extend_from_slice(&[
            GameAction::AttackNorth, GameAction::AttackSouth,
            GameAction::AttackEast, GameAction::AttackWest,
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

fn calculate_reward(old_state: &UnifiedGameState, new_state: &UnifiedGameState, _action: &GameAction, message: &str) -> f64 {
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
    if msg.contains("die") { reward -= 5.0; }
    
    reward
}

fn main() {
    let args = Args::parse();
    let config = Config::from(args.clone());
    
    train_multithreaded(config, &args);
}
