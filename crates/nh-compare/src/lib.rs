//! Convergence comparison framework for nethack-rs vs NetHack C 3.6.7.
//!
//! Provides structured snapshot capture, diffing with severity classification,
//! RNG trace comparison, and convergence reporting.

pub mod diff;
pub mod report;
pub mod snapshot;
