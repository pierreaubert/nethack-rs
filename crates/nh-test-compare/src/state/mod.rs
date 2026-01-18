//! State representation and extraction for dual-game comparison.
//!
//! This module provides functionality to extract unified game states
//! from both the Rust and C implementations.

pub mod common;
pub use common::*;

pub mod rust_extractor;
pub mod c_extractor;
pub mod ffi_extractor;
