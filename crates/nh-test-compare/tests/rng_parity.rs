//! Step 1.1: Exhaustive RNG comparison test
//!
//! Verifies that the Rust ISAAC64 implementation produces the exact same
//! sequence as the C ISAAC64 implementation for many seeds and many values.
//! Also tests the NetHack RNG wrapper functions (rn2, rnd, d).

use nh_test_compare::rng::isaac64::Isaac64;
use nh_test_compare::ffi::CIsaac64;

/// Test 100 different seeds, 10,000 values each, bitwise equality.
#[test]
fn test_100_seeds_10k_values() {
    let seeds: Vec<u64> = (0..100).map(|i| i * 97 + 1).collect();

    for seed in seeds {
        let mut rust = Isaac64::new(seed);
        let mut c = CIsaac64::new(seed);

        for i in 0..10_000 {
            let r = rust.next_u64();
            let c_val = c.next_u64();
            assert_eq!(
                r, c_val,
                "Seed {}: mismatch at position {}: Rust={:#018x}, C={:#018x}",
                seed, i, r, c_val
            );
        }
    }
}

/// Test edge-case seeds: 0, 1, MAX, powers of 2, etc.
#[test]
fn test_edge_case_seeds() {
    let seeds = [
        0u64,
        1,
        u64::MAX,
        u64::MAX - 1,
        0x8000_0000_0000_0000,
        0xFFFF_FFFF,
        0xDEAD_BEEF,
        0xCAFE_BABE,
        0x0123_4567_89AB_CDEF,
        42,
    ];

    for &seed in &seeds {
        let mut rust = Isaac64::new(seed);
        let mut c = CIsaac64::new(seed);

        for i in 0..10_000 {
            let r = rust.next_u64();
            let c_val = c.next_u64();
            assert_eq!(
                r, c_val,
                "Seed {:#x}: mismatch at position {}: Rust={:#018x}, C={:#018x}",
                seed, i, r, c_val
            );
        }
    }
}

/// Test rn2(n) wrapper for various moduli.
#[test]
fn test_rn2_parity() {
    let moduli = [2, 3, 5, 6, 7, 10, 20, 50, 100, 256, 1000, 10000];

    for &n in &moduli {
        let mut rust = Isaac64::new(42);
        let mut c = CIsaac64::new(42);

        for i in 0..5_000 {
            let r = rust.rn2(n);
            let c_val = c.rn2(n);
            assert_eq!(
                r, c_val,
                "rn2({}): mismatch at position {}: Rust={}, C={}",
                n, i, r, c_val
            );
        }
    }
}

/// Test rnd(n) wrapper: returns [1, n].
#[test]
fn test_rnd_parity() {
    let moduli = [1, 2, 3, 4, 6, 8, 10, 20, 100];

    for &n in &moduli {
        let mut rust = Isaac64::new(7777);
        let mut c = CIsaac64::new(7777);

        for i in 0..5_000 {
            let r = rust.rnd(n);
            // C rnd uses: RND(x) + 1 where RND(x) = isaac64_next_uint64() % x
            let c_raw = (c.next_u64() % n as u64) as u32 + 1;
            assert_eq!(
                r, c_raw,
                "rnd({}): mismatch at position {}: Rust={}, C={}",
                n, i, r, c_raw
            );
        }
    }
}

/// Test d(n, x) == NdX dice roll wrapper.
/// C implementation: tmp = n; while (n--) tmp += RND(x); return tmp;
#[test]
fn test_dice_parity() {
    let dice_rolls = [(1, 6), (2, 6), (3, 6), (1, 4), (1, 8), (2, 8), (1, 10), (1, 12), (1, 20)];

    for &(n, x) in &dice_rolls {
        let mut rust = Isaac64::new(12345);
        let mut c = CIsaac64::new(12345);

        for i in 0..2_000 {
            let r = rust.dice(n, x);
            // C d(n,x): tmp=n; while(n--) tmp += RND(x);
            let mut c_tmp = n;
            for _ in 0..n {
                c_tmp += (c.next_u64() % x as u64) as u32;
            }
            assert_eq!(
                r, c_tmp,
                "d({},{}): mismatch at position {}: Rust={}, C={}",
                n, x, i, r, c_tmp
            );
        }
    }
}

/// Test next_uint(n) uniform distribution wrapper.
#[test]
fn test_next_uint_parity() {
    let ranges = [2u64, 3, 5, 10, 100, 256, 1000, 65536, 1_000_000];

    for &n in &ranges {
        let mut rust = Isaac64::new(99);
        let mut c = CIsaac64::new(99);

        for i in 0..1_000 {
            let r = rust.next_uint(n);
            let c_val = c.next_uint(n);
            assert_eq!(
                r, c_val,
                "next_uint({}): mismatch at position {}: Rust={}, C={}",
                n, i, r, c_val
            );
        }
    }
}

/// Stress test: generate 100K values for a single seed.
#[test]
fn test_long_sequence_100k() {
    let mut rust = Isaac64::new(314159);
    let mut c = CIsaac64::new(314159);

    for i in 0..100_000 {
        let r = rust.next_u64();
        let c_val = c.next_u64();
        assert_eq!(
            r, c_val,
            "Long sequence: mismatch at position {}: Rust={:#018x}, C={:#018x}",
            i, r, c_val
        );
    }
}

/// Test that multiple reseed/create cycles produce consistent results.
/// This is important for save/load scenarios.
#[test]
fn test_reseed_consistency() {
    for seed in 0..50u64 {
        // Create, generate some, then create fresh with same seed
        let mut rng1 = Isaac64::new(seed);
        let mut c1 = CIsaac64::new(seed);

        // Skip first 500
        for _ in 0..500 {
            rng1.next_u64();
            c1.next_u64();
        }

        // Create fresh instances
        let mut rng2 = Isaac64::new(seed);
        let mut c2 = CIsaac64::new(seed);

        // Skip same 500
        for _ in 0..500 {
            rng2.next_u64();
            c2.next_u64();
        }

        // Next 100 should match
        for i in 0..100 {
            let r1 = rng1.next_u64();
            let r2 = rng2.next_u64();
            let cv1 = c1.next_u64();
            let cv2 = c2.next_u64();
            assert_eq!(r1, r2, "Rust reseed inconsistency at seed={}, pos={}", seed, i);
            assert_eq!(cv1, cv2, "C reseed inconsistency at seed={}, pos={}", seed, i);
            assert_eq!(r1, cv1, "Cross-impl mismatch at seed={}, pos={}", seed, i);
        }
    }
}
