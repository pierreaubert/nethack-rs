//! NetHack virtual player system
//!
//! This crate provides a virtual player that can play both implementations
//! in parallel to detect behavioral differences:
//!
//! - `state::common`: Unified state representation for comparison
//! - `state::rust_extractor`: Extract unified state from Rust implementation
//! - `state::c_extractor`: Extract unified state from C implementation
//! - `compare`: Compare states and detect differences
//! - `agent`: RL-based virtual player for automated testing
//! - `orchestrator`: Run parallel comparison sessions

pub mod agent;
pub mod compare;
pub mod orchestrator;
pub mod state;

pub use state::common::*;
