//! Random number generation for NetHack
//!
//! Uses a seeded ChaCha RNG for reproducibility (save/restore).

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Game random number generator
///
/// Wraps ChaCha8Rng for reproducible random number generation.
/// Note: RNG state is not serialized - games restore with a new seed derived from the original.
#[derive(Debug, Clone)]
pub struct GameRng {
    rng: ChaCha8Rng,
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
            rng: ChaCha8Rng::seed_from_u64(seed),
            seed,
        }
    }

    /// Create a new RNG with a random seed
    pub fn from_entropy() -> Self {
        let seed = rand::random();
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
        self.rng.gen_range(0..n)
    }

    /// Equivalent to NetHack's rnd(n) - returns 1..n
    ///
    /// Returns 0 if n is 0.
    pub fn rnd(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        self.rng.gen_range(1..=n)
    }

    /// Equivalent to NetHack's d(n, m) - roll n dice with m sides
    ///
    /// Returns sum of n rolls of 1..m
    pub fn dice(&mut self, n: u32, m: u32) -> u32 {
        (0..n).map(|_| self.rnd(m)).sum()
    }

    /// Equivalent to NetHack's rnl(n) - luck-adjusted random
    ///
    /// Returns 0..n-1, adjusted by luck (positive luck favors lower values)
    pub fn rnl(&mut self, n: u32, luck: i8) -> u32 {
        if n == 0 {
            return 0;
        }
        let mut result = self.rn2(n) as i32;
        result -= luck as i32;
        result.clamp(0, n as i32 - 1) as u32
    }

    /// Returns true with probability 1/n
    pub fn one_in(&mut self, n: u32) -> bool {
        self.rn2(n) == 0
    }

    /// Returns true with probability percent/100
    pub fn percent(&mut self, percent: u32) -> bool {
        self.rn2(100) < percent
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
        (
            self.rn2(max_x as u32) as u8,
            self.rn2(max_y as u32) as u8,
        )
    }
}

impl Default for GameRng {
    fn default() -> Self {
        Self::from_entropy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rn2_bounds() {
        let mut rng = GameRng::new(42);
        for _ in 0..1000 {
            let n = rng.rn2(10);
            assert!(n < 10);
        }
    }

    #[test]
    fn test_rnd_bounds() {
        let mut rng = GameRng::new(42);
        for _ in 0..1000 {
            let n = rng.rnd(6);
            assert!(n >= 1 && n <= 6);
        }
    }

    #[test]
    fn test_dice() {
        let mut rng = GameRng::new(42);
        for _ in 0..1000 {
            let n = rng.dice(2, 6); // 2d6
            assert!(n >= 2 && n <= 12);
        }
    }

    #[test]
    fn test_reproducibility() {
        let mut rng1 = GameRng::new(42);
        let mut rng2 = GameRng::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.rn2(100), rng2.rn2(100));
        }
    }

    #[test]
    fn test_zero_inputs() {
        let mut rng = GameRng::new(42);
        assert_eq!(rng.rn2(0), 0);
        assert_eq!(rng.rnd(0), 0);
        assert_eq!(rng.dice(0, 6), 0);
        assert_eq!(rng.dice(2, 0), 0);
    }
}
