//! NetHack C vs Rust comparison test system
//!
//! This crate provides tools to verify that the Rust nethack-rs implementation
//! matches the behavior of the original NetHack 3.6.7 C code.
//!
//! ## Tiers of Testing
//!
//! 1. **Static Data Comparison**: Verify monster/object/artifact definitions match
//! 2. **RNG Equivalence**: Port ISAAC64 to Rust for identical random sequences
//! 3. **Calculation Comparison**: Verify combat/AC/movement calculations via FFI
//! 4. **Map Comparison**: Verify dungeon generation algorithms produce equivalent results
//! 5. **Behavioral Comparison**: Full record-replay with state snapshots
//!
//! ## Virtual Player System
//!
//! This crate includes a virtual player system that can play both implementations
//! in parallel to detect behavioral differences:
//!
//! - `state::common`: Unified state representation for comparison
//! - `state::rust_extractor`: Extract unified state from Rust implementation
//! - `state::c_extractor`: Extract unified state from C implementation
//! - `compare`: Compare states and detect differences
//! - `agent`: RL-based virtual player for automated testing

pub mod calc;
pub mod data;
pub mod ffi;
pub mod maps;
pub mod rng;
pub mod state;
pub mod compare;
pub mod agent;
pub mod c_interface;
pub mod c_interface_ffi;
pub mod orchestrator;

// Re-export commonly used items
pub use rng::isaac64::Isaac64;
pub use state::common::*;
