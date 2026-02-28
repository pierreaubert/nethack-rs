//! nh-core: Core game logic for NetHack clone
//!
//! This crate contains all game logic with no I/O dependencies.
//! It is designed to be pure and testable.
//!
//! Supports `no_std` environments (e.g. PolkaVM smart contracts) by disabling
//! the default `std` feature. File I/O modules are gated behind `cfg(feature = "std")`.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

/// Re-exports of alloc types needed when building without std.
/// In std mode, these are provided by the std prelude.
#[cfg(not(feature = "std"))]
pub(crate) mod compat {
    pub use alloc::borrow::ToOwned;
    pub use alloc::boxed::Box;
    pub use alloc::format;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec;
    pub use alloc::vec::Vec;
}

pub mod action;
pub mod combat;
pub mod data;
pub mod dungeon;
pub mod magic;
pub mod monster;
pub mod object;
pub mod player;
#[cfg(feature = "std")]
pub mod save;
pub mod special;
pub mod world;

/// Trait for C game engine implementations used in comparison testing.
pub trait CGameEngineTrait {
    fn init(&mut self, role: &str, race: &str, gender: i32, alignment: i32) -> Result<(), String>;
    fn reset(&mut self, seed: u64) -> Result<(), String>;
    fn generate_and_place(&self) -> Result<(), String>;
    fn export_level(&self) -> String;
    fn exec_cmd(&self, cmd: char) -> Result<(), String>;
    fn exec_cmd_dir(&self, cmd: char, dx: i32, dy: i32) -> Result<(), String>;
    fn hp(&self) -> i32;
    fn max_hp(&self) -> i32;
    fn energy(&self) -> i32;
    fn max_energy(&self) -> i32;
    fn position(&self) -> (i32, i32);
    fn set_state(&self, hp: i32, hpmax: i32, x: i32, y: i32, ac: i32, moves: i64);
    fn armor_class(&self) -> i32;
    fn gold(&self) -> i32;
    fn experience_level(&self) -> i32;
    fn current_level(&self) -> i32;
    fn dungeon_depth(&self) -> i32;
    fn turn_count(&self) -> u64;
    fn is_dead(&self) -> bool;
    fn is_game_over(&self) -> bool;
    fn is_won(&self) -> bool;
    fn state_json(&self) -> String;
    fn last_message(&self) -> String;
    fn inventory_json(&self) -> String;
    fn monsters_json(&self) -> String;
    fn role(&self) -> String;
    fn race(&self) -> String;
    fn gender_string(&self) -> String;
    fn alignment_string(&self) -> String;
}

mod consts;
mod gameloop;
mod rng;

pub use consts::*;
pub use gameloop::{GameLoop, GameLoopResult, GameState};
pub use rng::GameRng;
