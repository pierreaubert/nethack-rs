//! FFI bindings to NetHack C code
//!
//! These bindings allow calling into the original C implementation
//! for comparison testing.

use libc::{c_int, c_uchar, c_uint};

/// ISAAC64 context size
pub const ISAAC64_SZ: usize = 256;

/// ISAAC64 context - matches struct isaac64_ctx from isaac64.h
#[repr(C)]
pub struct Isaac64Ctx {
    pub n: c_uint,
    pub r: [u64; ISAAC64_SZ],
    pub m: [u64; ISAAC64_SZ],
    pub a: u64,
    pub b: u64,
    pub c: u64,
}

unsafe extern "C" {
    /// Initialize ISAAC64 context with seed bytes
    pub fn isaac64_init(ctx: *mut Isaac64Ctx, seed: *const c_uchar, nseed: c_int);

    /// Reseed an existing context
    pub fn isaac64_reseed(ctx: *mut Isaac64Ctx, seed: *const c_uchar, nseed: c_int);

    /// Get next random u64
    pub fn isaac64_next_uint64(ctx: *mut Isaac64Ctx) -> u64;

    /// Get next random value in [0, n)
    pub fn isaac64_next_uint(ctx: *mut Isaac64Ctx, n: u64) -> u64;
}

/// Safe wrapper around the C ISAAC64 implementation
pub struct CIsaac64 {
    ctx: Isaac64Ctx,
}

impl CIsaac64 {
    /// Create a new C ISAAC64 instance with a u64 seed
    pub fn new(seed: u64) -> Self {
        let mut seed_bytes = [0u8; 8];
        let mut s = seed;
        for byte in &mut seed_bytes {
            *byte = (s & 0xFF) as u8;
            s >>= 8;
        }

        let mut ctx = Isaac64Ctx {
            n: 0,
            r: [0; ISAAC64_SZ],
            m: [0; ISAAC64_SZ],
            a: 0,
            b: 0,
            c: 0,
        };

        unsafe {
            isaac64_init(&mut ctx, seed_bytes.as_ptr(), 8);
        }

        Self { ctx }
    }

    /// Get the next random u64
    pub fn next_u64(&mut self) -> u64 {
        unsafe { isaac64_next_uint64(&mut self.ctx) }
    }

    /// Get a random value in [0, n)
    pub fn next_uint(&mut self, n: u64) -> u64 {
        unsafe { isaac64_next_uint(&mut self.ctx, n) }
    }

    /// Returns a random value in [0, x) - matches rn2(x)
    pub fn rn2(&mut self, x: u32) -> u32 {
        (self.next_u64() % x as u64) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::isaac64::Isaac64;

    #[test]
    fn test_rust_matches_c_isaac64() {
        let mut rust_rng = Isaac64::new(12345);
        let mut c_rng = CIsaac64::new(12345);

        // Compare 10,000 values
        for i in 0..10000 {
            let rust_val = rust_rng.next_u64();
            let c_val = c_rng.next_u64();
            assert_eq!(
                rust_val, c_val,
                "Mismatch at iteration {}: Rust={}, C={}",
                i, rust_val, c_val
            );
        }
    }

    #[test]
    fn test_rn2_matches_c() {
        let mut rust_rng = Isaac64::new(42);
        let mut c_rng = CIsaac64::new(42);

        for i in 0..1000 {
            let rust_val = rust_rng.rn2(100);
            let c_val = c_rng.rn2(100);
            assert_eq!(
                rust_val, c_val,
                "rn2 mismatch at iteration {}: Rust={}, C={}",
                i, rust_val, c_val
            );
        }
    }

    #[test]
    fn test_various_seeds() {
        for seed in [0u64, 1, 42, 12345, 0xDEADBEEF, u64::MAX] {
            let mut rust_rng = Isaac64::new(seed);
            let mut c_rng = CIsaac64::new(seed);

            for _ in 0..100 {
                assert_eq!(rust_rng.next_u64(), c_rng.next_u64());
            }
        }
    }
}
