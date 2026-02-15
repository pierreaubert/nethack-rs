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

mod consts;
mod gameloop;
mod rng;

pub use consts::*;
pub use gameloop::{GameLoop, GameLoopResult, GameState};
pub use rng::GameRng;
