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

pub mod c_source_parser;
pub mod calc;
pub mod ffi;
pub mod maps;
pub mod rng;

// Re-export commonly used items
pub use rng::isaac64::Isaac64;
