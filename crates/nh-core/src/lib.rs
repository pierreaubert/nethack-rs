//! nh-core: Core game logic for NetHack clone
//!
//! This crate contains all game logic with no I/O dependencies.
//! It is designed to be pure and testable.

pub mod action;
pub mod combat;
pub mod data;
pub mod dungeon;
pub mod magic;
pub mod monster;
pub mod object;
pub mod player;
pub mod save;
pub mod special;
pub mod world;

mod consts;
mod gameloop;
mod rng;

pub use consts::*;
pub use gameloop::{GameLoop, GameLoopResult, GameState};
pub use rng::GameRng;
