//! ISAAC64 random number generator - Rust port
//!
//! This module re-exports ISAAC64 from the core nh-rng crate to avoid
//! circular dependencies while maintaining the existing test infrastructure.

pub use nh_rng::{Isaac64, RngTraceEntry};
