//! ISAAC64 random number generator - Rust port
//!
//! This is a faithful port of the ISAAC64 implementation from NetHack 3.6.7.
//! Original by Timothy B. Terriberry, based on Bob Jenkins' ISAAC.
//!
//! CC0 (Public domain) - see http://creativecommons.org/publicdomain/zero/1.0/

/// Size of the ISAAC64 state arrays (2^8 = 256)
const ISAAC64_SZ_LOG: usize = 8;
const ISAAC64_SZ: usize = 1 << ISAAC64_SZ_LOG;
const ISAAC64_MASK: u64 = u64::MAX;

/// ISAAC64 random number generator context
#[derive(Clone)]
pub struct Isaac64 {
    /// Results buffer - random values to be consumed
    r: [u64; ISAAC64_SZ],
    /// Memory state
    m: [u64; ISAAC64_SZ],
    /// Accumulator
    a: u64,
    /// Previous result
    b: u64,
    /// Counter
    c: u64,
    /// Number of results remaining (counts down from 256)
    n: usize,
}

impl Isaac64 {
    /// Create a new ISAAC64 instance seeded with a u64 value.
    ///
    /// This matches NetHack's `init_isaac64(seed, fn)` initialization.
    pub fn new(seed: u64) -> Self {
        let mut seed_bytes = [0u8; 8];
        let mut s = seed;
        for byte in &mut seed_bytes {
            *byte = (s & 0xFF) as u8;
            s >>= 8;
        }

        let mut ctx = Self {
            r: [0; ISAAC64_SZ],
            m: [0; ISAAC64_SZ],
            a: 0,
            b: 0,
            c: 0,
            n: 0,
        };

        ctx.init(&seed_bytes);
        ctx
    }

    /// Initialize with seed bytes (matches isaac64_init)
    fn init(&mut self, seed: &[u8]) {
        self.a = 0;
        self.b = 0;
        self.c = 0;
        self.r = [0; ISAAC64_SZ];
        self.reseed(seed);
    }

    /// Mix seed bytes into state (matches isaac64_reseed)
    fn reseed(&mut self, seed: &[u8]) {
        let nseed = seed.len().min(ISAAC64_SZ * 8);

        // XOR seed bytes into r[] as little-endian u64s
        let full_words = nseed / 8;
        for i in 0..full_words {
            let base = i * 8;
            let val = (seed[base + 7] as u64) << 56
                | (seed[base + 6] as u64) << 48
                | (seed[base + 5] as u64) << 40
                | (seed[base + 4] as u64) << 32
                | (seed[base + 3] as u64) << 24
                | (seed[base + 2] as u64) << 16
                | (seed[base + 1] as u64) << 8
                | (seed[base] as u64);
            self.r[i] ^= val;
        }

        // Handle remaining bytes
        let remaining = nseed - (full_words * 8);
        if remaining > 0 {
            let base = full_words * 8;
            let mut val = seed[base] as u64;
            for j in 1..remaining {
                val |= (seed[base + j] as u64) << (j * 8);
            }
            self.r[full_words] ^= val;
        }

        // Initialize with the golden ratio
        let mut x = [0x9E3779B97F4A7C13u64; 8];

        // Mix 4 times
        for _ in 0..4 {
            Self::mix(&mut x);
        }

        // Fill m[] using mixed values
        for i in (0..ISAAC64_SZ).step_by(8) {
            for (xj, rj) in x.iter_mut().zip(self.r[i..i + 8].iter()) {
                *xj = xj.wrapping_add(*rj);
            }
            Self::mix(&mut x);
            self.m[i..i + 8].copy_from_slice(&x);
        }

        // Second pass
        for i in (0..ISAAC64_SZ).step_by(8) {
            for (xj, mj) in x.iter_mut().zip(self.m[i..i + 8].iter()) {
                *xj = xj.wrapping_add(*mj);
            }
            Self::mix(&mut x);
            self.m[i..i + 8].copy_from_slice(&x);
        }

        // Generate initial results
        self.update();
    }

    /// Mix function (matches isaac64_mix)
    fn mix(x: &mut [u64; 8]) {
        const SHIFT: [u32; 8] = [9, 9, 23, 15, 14, 20, 17, 14];

        for i in (0..8).step_by(2) {
            x[i] = x[i].wrapping_sub(x[(i + 4) & 7]);
            x[(i + 5) & 7] ^= x[(i + 7) & 7] >> SHIFT[i];
            x[(i + 7) & 7] = x[(i + 7) & 7].wrapping_add(x[i]);

            let i = i + 1;
            x[i] = x[i].wrapping_sub(x[(i + 4) & 7]);
            x[(i + 5) & 7] ^= x[(i + 7) & 7] << SHIFT[i];
            x[(i + 7) & 7] = x[(i + 7) & 7].wrapping_add(x[i]);
        }
    }

    /// Extract lower bits for indexing (bits 3..3+ISAAC64_SZ_LOG)
    #[inline]
    fn lower_bits(x: u64) -> usize {
        ((x & (((ISAAC64_SZ - 1) as u64) << 3)) >> 3) as usize
    }

    /// Extract upper bits for indexing (bits ISAAC64_SZ_LOG+3..2*ISAAC64_SZ_LOG+3)
    #[inline]
    fn upper_bits(y: u64) -> usize {
        ((y >> (ISAAC64_SZ_LOG + 3)) & ((ISAAC64_SZ - 1) as u64)) as usize
    }

    /// Generate 256 new random values (matches isaac64_update)
    fn update(&mut self) {
        let mut a = self.a;
        self.c = self.c.wrapping_add(1);
        let mut b = self.b.wrapping_add(self.c);

        // First half
        for i in (0..ISAAC64_SZ / 2).step_by(4) {
            // Pattern 1: ~(a ^ (a << 21))
            let x = self.m[i];
            a = (!a ^ (a << 21)).wrapping_add(self.m[i + ISAAC64_SZ / 2]);
            let y = self.m[Self::lower_bits(x)].wrapping_add(a).wrapping_add(b);
            self.m[i] = y;
            b = self.m[Self::upper_bits(y)].wrapping_add(x);
            self.r[i] = b;

            // Pattern 2: a ^ (a >> 5)
            let x = self.m[i + 1];
            a = (a ^ (a >> 5)).wrapping_add(self.m[i + 1 + ISAAC64_SZ / 2]);
            let y = self.m[Self::lower_bits(x)].wrapping_add(a).wrapping_add(b);
            self.m[i + 1] = y;
            b = self.m[Self::upper_bits(y)].wrapping_add(x);
            self.r[i + 1] = b;

            // Pattern 3: a ^ (a << 12)
            let x = self.m[i + 2];
            a = (a ^ (a << 12)).wrapping_add(self.m[i + 2 + ISAAC64_SZ / 2]);
            let y = self.m[Self::lower_bits(x)].wrapping_add(a).wrapping_add(b);
            self.m[i + 2] = y;
            b = self.m[Self::upper_bits(y)].wrapping_add(x);
            self.r[i + 2] = b;

            // Pattern 4: a ^ (a >> 33)
            let x = self.m[i + 3];
            a = (a ^ (a >> 33)).wrapping_add(self.m[i + 3 + ISAAC64_SZ / 2]);
            let y = self.m[Self::lower_bits(x)].wrapping_add(a).wrapping_add(b);
            self.m[i + 3] = y;
            b = self.m[Self::upper_bits(y)].wrapping_add(x);
            self.r[i + 3] = b;
        }

        // Second half
        for i in (ISAAC64_SZ / 2..ISAAC64_SZ).step_by(4) {
            let x = self.m[i];
            a = (!a ^ (a << 21)).wrapping_add(self.m[i - ISAAC64_SZ / 2]);
            let y = self.m[Self::lower_bits(x)].wrapping_add(a).wrapping_add(b);
            self.m[i] = y;
            b = self.m[Self::upper_bits(y)].wrapping_add(x);
            self.r[i] = b;

            let x = self.m[i + 1];
            a = (a ^ (a >> 5)).wrapping_add(self.m[i + 1 - ISAAC64_SZ / 2]);
            let y = self.m[Self::lower_bits(x)].wrapping_add(a).wrapping_add(b);
            self.m[i + 1] = y;
            b = self.m[Self::upper_bits(y)].wrapping_add(x);
            self.r[i + 1] = b;

            let x = self.m[i + 2];
            a = (a ^ (a << 12)).wrapping_add(self.m[i + 2 - ISAAC64_SZ / 2]);
            let y = self.m[Self::lower_bits(x)].wrapping_add(a).wrapping_add(b);
            self.m[i + 2] = y;
            b = self.m[Self::upper_bits(y)].wrapping_add(x);
            self.r[i + 2] = b;

            let x = self.m[i + 3];
            a = (a ^ (a >> 33)).wrapping_add(self.m[i + 3 - ISAAC64_SZ / 2]);
            let y = self.m[Self::lower_bits(x)].wrapping_add(a).wrapping_add(b);
            self.m[i + 3] = y;
            b = self.m[Self::upper_bits(y)].wrapping_add(x);
            self.r[i + 3] = b;
        }

        self.b = b;
        self.a = a;
        self.n = ISAAC64_SZ;
    }

    /// Get the next random u64 (matches isaac64_next_uint64)
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        if self.n == 0 {
            self.update();
        }
        self.n -= 1;
        self.r[self.n]
    }

    /// Get a random value in range [0, n) with uniform distribution
    /// (matches isaac64_next_uint)
    pub fn next_uint(&mut self, n: u64) -> u64 {
        loop {
            let r = self.next_u64();
            let v = r % n;
            let d = r - v;
            // Reject if we're in the incomplete final bucket
            if (d.wrapping_add(n).wrapping_sub(1) & ISAAC64_MASK) >= d {
                return v;
            }
        }
    }

    // NetHack-compatible RNG functions

    /// Returns a random value in [0, x) - matches rn2(x)
    #[inline]
    pub fn rn2(&mut self, x: u32) -> u32 {
        (self.next_u64() % x as u64) as u32
    }

    /// Returns a random value in [1, x] - matches rnd(x)
    #[inline]
    pub fn rnd(&mut self, x: u32) -> u32 {
        (self.next_u64() % x as u64) as u32 + 1
    }

    /// Roll n dice of x sides - matches d(n, x)
    pub fn dice(&mut self, n: u32, x: u32) -> u32 {
        let mut result = n;
        for _ in 0..n {
            result += self.rn2(x);
        }
        result
    }
}

impl Default for Isaac64 {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reproducibility() {
        let mut rng1 = Isaac64::new(12345);
        let mut rng2 = Isaac64::new(12345);

        for _ in 0..1000 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_different_seeds() {
        let mut rng1 = Isaac64::new(12345);
        let mut rng2 = Isaac64::new(54321);

        // Different seeds should produce different sequences
        let mut all_same = true;
        for _ in 0..100 {
            if rng1.next_u64() != rng2.next_u64() {
                all_same = false;
                break;
            }
        }
        assert!(!all_same);
    }

    #[test]
    fn test_rn2_bounds() {
        let mut rng = Isaac64::new(42);
        for _ in 0..10000 {
            let val = rng.rn2(100);
            assert!(val < 100);
        }
    }

    #[test]
    fn test_rnd_bounds() {
        let mut rng = Isaac64::new(42);
        for _ in 0..10000 {
            let val = rng.rnd(6);
            assert!(val >= 1 && val <= 6);
        }
    }

    #[test]
    fn test_dice_bounds() {
        let mut rng = Isaac64::new(42);
        for _ in 0..10000 {
            let val = rng.dice(2, 6); // 2d6
            assert!(val >= 2 && val <= 12);
        }
    }
}
