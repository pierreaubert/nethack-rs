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

/// An RNG call trace entry for debugging divergences.
#[derive(Debug, Clone)]
pub struct RngTraceEntry {
    /// Sequence number (0-indexed)
    pub seq: u64,
    /// Function name (e.g. "rn2", "rnd", "next_u64")
    pub func: &'static str,
    /// Argument (e.g. modulus for rn2)
    pub arg: u64,
    /// Result value
    pub result: u64,
    /// Raw u64 consumed from ISAAC64
    pub raw: u64,
}

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
    /// Total number of u64 values consumed (for tracing)
    call_count: u64,
    /// If true, record all calls into trace log
    tracing: bool,
    /// Trace log (only populated when tracing is true)
    trace: Vec<RngTraceEntry>,
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
            call_count: 0,
            tracing: false,
            trace: Vec::new(),
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

    // ================================================================
    // Tracing control
    // ================================================================

    /// Enable RNG call tracing. All subsequent calls will be logged.
    pub fn enable_tracing(&mut self) {
        self.tracing = true;
        self.trace.clear();
    }

    /// Disable RNG call tracing.
    pub fn disable_tracing(&mut self) {
        self.tracing = false;
    }

    /// Get the trace log.
    pub fn trace(&self) -> &[RngTraceEntry] {
        &self.trace
    }

    /// Clear the trace log.
    pub fn clear_trace(&mut self) {
        self.trace.clear();
    }

    /// Get total number of raw u64 values consumed.
    pub fn call_count(&self) -> u64 {
        self.call_count
    }

    // ================================================================
    // Core random generation
    // ================================================================

    /// Get the next random u64 (matches isaac64_next_uint64)
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        if self.n == 0 {
            self.update();
        }
        self.n -= 1;
        let val = self.r[self.n];
        self.call_count += 1;
        val
    }

    /// Internal: get next u64 and record trace entry.
    fn next_u64_traced(&mut self, func: &'static str, arg: u64) -> u64 {
        let raw = self.next_u64();
        if self.tracing {
            self.trace.push(RngTraceEntry {
                seq: self.call_count - 1,
                func,
                arg,
                result: raw,
                raw,
            });
        }
        raw
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

    // ================================================================
    // NetHack-compatible RNG functions
    // ================================================================

    /// Returns a random value in [0, x) - matches rn2(x)
    /// C: `int rn2(int x) { return RND(x); }`
    /// where `RND(x) = isaac64_next_uint64(&rng) % x`
    #[inline]
    pub fn rn2(&mut self, x: u32) -> u32 {
        let raw = self.next_u64();
        let result = (raw % x as u64) as u32;
        if self.tracing {
            self.trace.push(RngTraceEntry {
                seq: self.call_count - 1,
                func: "rn2",
                arg: x as u64,
                result: result as u64,
                raw,
            });
        }
        result
    }

    /// Returns a random value in [1, x] - matches rnd(x)
    /// C: `int rnd(int x) { return RND(x) + 1; }`
    #[inline]
    pub fn rnd(&mut self, x: u32) -> u32 {
        let raw = self.next_u64();
        let result = (raw % x as u64) as u32 + 1;
        if self.tracing {
            self.trace.push(RngTraceEntry {
                seq: self.call_count - 1,
                func: "rnd",
                arg: x as u64,
                result: result as u64,
                raw,
            });
        }
        result
    }

    /// Roll n dice of x sides - matches d(n, x)
    /// C: `int d(int n, int x) { int tmp = n; while (n--) tmp += RND(x); return tmp; }`
    pub fn dice(&mut self, n: u32, x: u32) -> u32 {
        let mut result = n;
        for _ in 0..n {
            result += self.rn2(x);
        }
        result
    }

    /// Luck-adjusted random - matches rnl(x) from rnd.c
    /// Note: This is a simplified version; full rnl requires game state (Luck).
    pub fn rnl(&mut self, x: u32, luck: i32) -> u32 {
        let mut i = self.rn2(x) as i32;
        let adjustment = if x <= 15 {
            (luck.abs() + 1) / 3 * luck.signum()
        } else {
            luck
        };
        if adjustment != 0 && self.rn2(37 + adjustment.unsigned_abs()) != 0 {
            i -= adjustment;
            if i < 0 {
                i = 0;
            } else if i >= x as i32 {
                i = x as i32 - 1;
            }
        }
        i as u32
    }

    /// Exponential distribution - matches rne(x) from rnd.c
    /// C: `int rne(int x) { int utmp = (u.ulevel < 15) ? 5 : u.ulevel/3;
    ///     int tmp = 1; while (tmp < utmp && !rn2(x)) tmp++; return tmp; }`
    pub fn rne(&mut self, x: u32, player_level: u32) -> u32 {
        let utmp = if player_level < 15 { 5 } else { player_level / 3 };
        let mut tmp = 1u32;
        while tmp < utmp && self.rn2(x) == 0 {
            tmp += 1;
        }
        tmp
    }

    /// "Everyone's favorite" - matches rnz(i) from rnd.c
    /// C implementation uses rn2(1000), rne(4), rn2(2) internally.
    pub fn rnz(&mut self, i: i32, player_level: u32) -> i32 {
        let mut x = i as i64;
        let mut tmp = 1000i64;
        tmp += self.rn2(1000) as i64;
        tmp *= self.rne(4, player_level) as i64;
        if self.rn2(2) != 0 {
            x = x * tmp / 1000;
        } else {
            x = x * 1000 / tmp;
        }
        x as i32
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
