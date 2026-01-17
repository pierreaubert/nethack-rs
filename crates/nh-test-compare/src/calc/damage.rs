//! Damage dice comparison
//!
//! Verifies that damage dice rolling (d(n,x)) produces identical results
//! between C and Rust implementations when using the same RNG seed.

use crate::ffi::CIsaac64;
use crate::Isaac64;

/// Compare dice rolling between Rust and C implementations
///
/// The d(n,x) function rolls n dice of x sides.
/// Formula: result = n; for i in 0..n { result += rn2(x); }
/// Range: [n, n*x]
pub fn compare_dice(seed: u64, n: u32, x: u32) -> (u32, u32) {
    let mut rust_rng = Isaac64::new(seed);
    let mut c_rng = CIsaac64::new(seed);

    let rust_result = rust_rng.dice(n, x);
    let c_result = c_dice(&mut c_rng, n, x);

    (rust_result, c_result)
}

/// C-style d(n, x) using our C FFI RNG
fn c_dice(rng: &mut CIsaac64, n: u32, x: u32) -> u32 {
    let mut result = n;
    for _ in 0..n {
        result += rng.rn2(x);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dice_single_die() {
        // Test d(1, 6) - single d6
        for seed in [42u64, 12345, 99999, 1] {
            let (rust, c) = compare_dice(seed, 1, 6);
            assert_eq!(
                rust, c,
                "d(1,6) mismatch with seed {}: rust={}, c={}",
                seed, rust, c
            );
            // Range check: d(1,6) should be 1-6
            assert!(rust >= 1 && rust <= 6, "d(1,6) out of range: {}", rust);
        }
    }

    #[test]
    fn test_dice_multiple_dice() {
        // Test d(2, 6) - 2d6
        for seed in [42u64, 12345, 99999, 1] {
            let (rust, c) = compare_dice(seed, 2, 6);
            assert_eq!(
                rust, c,
                "d(2,6) mismatch with seed {}: rust={}, c={}",
                seed, rust, c
            );
            // Range check: d(2,6) should be 2-12
            assert!(rust >= 2 && rust <= 12, "d(2,6) out of range: {}", rust);
        }
    }

    #[test]
    fn test_dice_common_weapon_damages() {
        // Test common weapon damage dice
        let weapon_dice = [
            (1, 4, "dagger"),       // d4
            (1, 6, "short sword"),  // d6
            (1, 8, "long sword"),   // d8
            (2, 4, "broadsword"),   // 2d4
            (1, 10, "halberd"),     // d10
            (2, 6, "two-handed"),   // 2d6
            (1, 12, "great axe"),   // d12
        ];

        for seed in [42u64, 12345, 99999] {
            for (n, x, name) in &weapon_dice {
                let (rust, c) = compare_dice(seed, *n, *x);
                assert_eq!(
                    rust, c,
                    "{} d({},{}) mismatch with seed {}: rust={}, c={}",
                    name, n, x, seed, rust, c
                );
            }
        }
    }

    #[test]
    fn test_dice_sequence_matches() {
        // Roll many dice in sequence, verify all match
        let seed = 42u64;
        let mut rust_rng = Isaac64::new(seed);
        let mut c_rng = CIsaac64::new(seed);

        for i in 0..1000 {
            let n = (i % 5) as u32 + 1; // 1-5 dice
            let x = (i % 12) as u32 + 1; // 1-12 sides

            let rust_result = rust_rng.dice(n, x);
            let c_result = c_dice(&mut c_rng, n, x);

            assert_eq!(
                rust_result, c_result,
                "dice sequence mismatch at iteration {}: d({},{}) rust={}, c={}",
                i, n, x, rust_result, c_result
            );
        }
    }

    #[test]
    fn test_rn2_matches() {
        // Verify rn2 (the building block of dice) matches
        let seed = 42u64;
        let mut rust_rng = Isaac64::new(seed);
        let mut c_rng = CIsaac64::new(seed);

        for i in 0..10000 {
            let max = (i % 100) as u32 + 1;
            let rust_val = rust_rng.rn2(max);
            let c_val = c_rng.rn2(max);

            assert_eq!(
                rust_val, c_val,
                "rn2({}) mismatch at iteration {}: rust={}, c={}",
                max, i, rust_val, c_val
            );
        }
    }

    #[test]
    fn test_rnd_matches() {
        // Verify rnd (1 to x inclusive) matches
        let seed = 42u64;
        let mut rust_rng = Isaac64::new(seed);
        let mut c_rng = CIsaac64::new(seed);

        for i in 0..10000 {
            let max = (i % 20) as u32 + 1;
            let rust_val = rust_rng.rnd(max);
            // C rnd is rn2(x) + 1
            let c_val = c_rng.rn2(max) + 1;

            assert_eq!(
                rust_val, c_val,
                "rnd({}) mismatch at iteration {}: rust={}, c={}",
                max, i, rust_val, c_val
            );
        }
    }
}
