//! ISAAC64 random number generator - Rust port
//!
//! This is a faithful port of the ISAAC64 implementation from NetHack 3.6.7.
//! Matches NetHack 3.6.7 ISAAC64 implementation.

use serde::{Deserialize, Serialize};

/// Size of the ISAAC64 state arrays (2^8 = 256)
const ISAAC64_SZ_LOG: usize = 8;
const ISAAC64_SZ: usize = 1 << ISAAC64_SZ_LOG;

/// An RNG call trace entry for debugging divergences.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Clone, Serialize, Deserialize)]
pub struct Isaac64 {
    /// Results buffer - random values to be consumed
    r: Vec<u64>,
    /// Memory state
    m: Vec<u64>,
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
    #[serde(skip)]
    tracing: bool,
    /// Trace log (only populated when tracing is true)
    #[serde(skip)]
    trace: Vec<RngTraceEntry>,
}

impl core::fmt::Debug for Isaac64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Isaac64")
            .field("a", &self.a)
            .field("b", &self.b)
            .field("c", &self.c)
            .field("n", &self.n)
            .field("call_count", &self.call_count)
            .finish()
    }
}

impl Isaac64 {
    /// Create a new ISAAC64 instance seeded with a u64 value.
    pub fn new(seed: u64) -> Self {
        let mut seed_bytes = [0u8; 8];
        let mut s = seed;
        for byte in &mut seed_bytes {
            *byte = (s & 0xFF) as u8;
            s >>= 8;
        }

        let mut ctx = Self {
            r: vec![0; ISAAC64_SZ],
            m: vec![0; ISAAC64_SZ],
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
        for i in 0..ISAAC64_SZ { self.r[i] = 0; }
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
            for j in 0..8 {
                x[j] = x[j].wrapping_add(self.r[i + j]);
            }
            Self::mix(&mut x);
            for j in 0..8 {
                self.m[i + j] = x[j];
            }
        }

        // Second pass
        for i in (0..ISAAC64_SZ).step_by(8) {
            for j in 0..8 {
                x[j] = x[j].wrapping_add(self.m[i + j]);
            }
            Self::mix(&mut x);
            for j in 0..8 {
                self.m[i + j] = x[j];
            }
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
            let x = self.m[i];
            a = (!a ^ (a << 21)).wrapping_add(self.m[i + ISAAC64_SZ / 2]);
            let y = self.m[Self::lower_bits(x)].wrapping_add(a).wrapping_add(b);
            self.m[i] = y;
            b = self.m[Self::upper_bits(y)].wrapping_add(x);
            self.r[i] = b;

            let x = self.m[i + 1];
            a = (a ^ (a >> 5)).wrapping_add(self.m[i + 1 + ISAAC64_SZ / 2]);
            let y = self.m[Self::lower_bits(x)].wrapping_add(a).wrapping_add(b);
            self.m[i + 1] = y;
            b = self.m[Self::upper_bits(y)].wrapping_add(x);
            self.r[i + 1] = b;

            let x = self.m[i + 2];
            a = (a ^ (a << 12)).wrapping_add(self.m[i + 2 + ISAAC64_SZ / 2]);
            let y = self.m[Self::lower_bits(x)].wrapping_add(a).wrapping_add(b);
            self.m[i + 2] = y;
            b = self.m[Self::upper_bits(y)].wrapping_add(x);
            self.r[i + 2] = b;

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
        let val = self.r[self.n];
        self.call_count += 1;
        val
    }

    /// Returns a random value in [0, n)
    pub fn next_uint(&mut self, n: u64) -> u64 {
        if n == 0 { return 0; }
        let raw = self.next_u64();
        let res = raw % n;
        if self.tracing {
            self.trace.push(RngTraceEntry {
                seq: self.call_count - 1,
                func: "next_uint",
                arg: n,
                result: res,
                raw,
            });
        }
        res
    }

    /// Returns a random value in [0, x) - matches rn2(x)
    #[inline]
    pub fn rn2(&mut self, x: u32) -> u32 {
        if x == 0 { return 0; }
        let raw = self.next_u64();
        let res = (raw % x as u64) as u32;
        if self.tracing {
            self.trace.push(RngTraceEntry {
                seq: self.call_count - 1,
                func: "rn2",
                arg: x as u64,
                result: res as u64,
                raw,
            });
        }
        res
    }

    /// Returns a random value in [1, x] - matches rnd(x)
    #[inline]
    pub fn rnd(&mut self, x: u32) -> u32 {
        if x == 0 { return 0; }
        let raw = self.next_u64();
        let res = (raw % x as u64) as u32 + 1;
        if self.tracing {
            self.trace.push(RngTraceEntry {
                seq: self.call_count - 1,
                func: "rnd",
                arg: x as u64,
                result: res as u64,
                raw,
            });
        }
        res
    }

    /// Roll n dice of x sides - matches d(n, x)
    pub fn dice(&mut self, n: u32, x: u32) -> u32 {
        let mut result = n;
        for _ in 0..n {
            result += self.rn2(x);
        }
        result
    }

    /// Luck-adjusted random - matches rnl(x) from rnd.c
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
    pub fn rne(&mut self, x: u32, player_level: u32) -> u32 {
        let utmp = if player_level < 15 { 5 } else { player_level / 3 };
        let mut tmp = 1u32;
        while tmp < utmp && self.rn2(x) == 0 {
            tmp += 1;
        }
        tmp
    }

    /// "Everyone's favorite" - matches rnz(i) from rnd.c
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

    /// Enable RNG tracing
    pub fn enable_tracing(&mut self) {
        self.tracing = true;
        self.trace.clear();
    }

    /// Disable RNG tracing
    pub fn disable_tracing(&mut self) {
        self.tracing = false;
    }

    /// Enable RNG tracing (alias)
    pub fn start_tracing(&mut self) {
        self.enable_tracing();
    }

    /// Get current RNG trace
    pub fn get_trace(&self) -> Vec<RngTraceEntry> {
        self.trace.clone()
    }

    /// Get current RNG trace (alias)
    pub fn trace(&self) -> Vec<RngTraceEntry> {
        self.get_trace()
    }

    /// Total number of raw u64 calls
    pub fn call_count(&self) -> u64 {
        self.call_count
    }
}

impl Default for Isaac64 {
    fn default() -> Self {
        Self::new(0)
    }
}
