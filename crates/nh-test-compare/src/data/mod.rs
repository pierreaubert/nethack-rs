//! Static data comparison module
//!
//! Compares monster, object, artifact, and role definitions between
//! the C source and Rust implementations.

pub mod artifacts;
pub mod monsters;
pub mod objects;
pub mod roles;

/// Path to NetHack 3.6.7 source
pub const NETHACK_SRC: &str = "/Users/pierre/src/games/NetHack-3.6.7";
