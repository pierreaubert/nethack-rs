//! nh-bevy: Bevy-based 2.5D frontend for NetHack-rs
//!
//! This crate provides a 3D rendering layer with 2D billboard sprites
//! for the nethack-rs roguelike game.

pub mod components;
pub mod plugins;
pub mod resources;

pub use plugins::game::GamePlugin;
