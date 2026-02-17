//! Random number generation for NetHack
//!
//! Uses a seeded ISAAC64 RNG for reproducibility and parity with C NetHack 3.6.7.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use nh_rng::Isaac64;
use serde::{Deserialize, Serialize};

/// Game random number generator
///
/// Wraps Isaac64 for Reproducible random number generation matching C engine.
#[derive(Debug, Clone)]
pub struct GameRng {
    rng: Isaac64,
    seed: u64,
}

// Custom serialization - only serialize seed, recreate RNG on deserialize
impl Serialize for GameRng {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.seed.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GameRng {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let seed = u64::deserialize(deserializer)?;
        Ok(GameRng::new(seed))
    }
}

impl GameRng {
    /// Create a new RNG with the given seed
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Isaac64::new(seed),
            seed,
        }
    }

    /// Create a new RNG with a random seed (requires std feature for entropy source)
    #[cfg(feature = "std")]
    pub fn from_entropy() -> Self {
        use rand::RngCore;
        let seed = rand::thread_rng().next_u64();
        Self::new(seed)
    }

    /// Get the seed used to create this RNG
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Equivalent to NetHack's rn2(n) - returns 0..n-1
    ///
    /// Returns 0 if n is 0.
    pub fn rn2(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        let r = self.rng.rn2(n);
        eprintln!("RS: rn2({}) = {}", n, r);
        r
    }

    pub fn rnd(&mut self, n: u32) -> u32 {
        let r = self.rng.rnd(n);
        eprintln!("RS: rnd({}) = {}", n, r);
        r
    }

    /// Equivalent to NetHack's d(n, m) - roll n dice with m sides
    ///
    /// Returns sum of n rolls of 1..m
    pub fn dice(&mut self, n: u32, m: u32) -> u32 {
        self.rng.dice(n, m)
    }

    /// Equivalent to NetHack's rnl(n) - luck-adjusted random
    ///
    /// Returns 0..n-1, adjusted by luck (positive luck favors lower values)
    pub fn rnl(&mut self, n: u32, luck: i8) -> u32 {
        self.rng.rnl(n, luck as i32)
    }

    /// Returns true with probability 1/n
    pub fn one_in(&mut self, n: u32) -> bool {
        self.rn2(n) == 0
    }

    /// Returns true with probability percent/100
    pub fn percent(&mut self, percent: u32) -> bool {
        self.rn2(100) < percent
    }

    /// Equivalent to NetHack's rnz(i) - returns 1..i biased toward lower values
    pub fn rnz(&mut self, i: u32) -> u32 {
        // C implementation level dependent - use level 1 for generation
        self.rng.rnz(i as i32, 1) as u32
    }

    /// Equivalent to NetHack's rne(x) - exponential distribution 1..x
    pub fn rne(&mut self, x: u32) -> u32 {
        self.rng.rne(x, 1)
    }

    /// Choose a random element from a slice
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            None
        } else {
            Some(&items[self.rn2(items.len() as u32) as usize])
        }
    }

    /// Shuffle a slice in place
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.rn2(i as u32 + 1) as usize;
            items.swap(i, j);
        }
    }

    /// Generate a random coordinate within bounds
    pub fn random_coord(&mut self, max_x: u8, max_y: u8) -> (u8, u8) {
        (self.rn2(max_x as u32) as u8, self.rn2(max_y as u32) as u8)
    }

    /// Enable RNG tracing
    pub fn start_tracing(&mut self) {
        self.rng.start_tracing();
    }

    /// Get current RNG trace
    pub fn get_trace(&self) -> Vec<nh_rng::RngTraceEntry> {
        self.rng.get_trace()
    }
}

#[cfg(feature = "std")]
impl Default for GameRng {
    fn default() -> Self {
        Self::from_entropy()
    }
}
