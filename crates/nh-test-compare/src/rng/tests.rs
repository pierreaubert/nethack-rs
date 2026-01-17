//! RNG comparison tests

use super::isaac64::Isaac64;
use crate::ffi::CIsaac64;

#[test]
fn test_isaac64_sequence_matches_c() {
    // Test with NetHack's typical seed range
    let seeds = [0u64, 1, 42, 12345, 9999, 0xCAFEBABE];

    for seed in seeds {
        let mut rust = Isaac64::new(seed);
        let mut c = CIsaac64::new(seed);

        // Generate and compare many values
        for i in 0..10000 {
            let r = rust.next_u64();
            let c_val = c.next_u64();
            assert_eq!(
                r, c_val,
                "Seed {}: mismatch at position {}: Rust={:#x}, C={:#x}",
                seed, i, r, c_val
            );
        }
    }
}

#[test]
fn test_rn2_distribution() {
    let mut rng = Isaac64::new(42);
    let mut counts = [0u32; 10];

    // Should be roughly uniform
    for _ in 0..100000 {
        let val = rng.rn2(10);
        counts[val as usize] += 1;
    }

    // Each bucket should be roughly 10000 +/- 500
    for (i, &count) in counts.iter().enumerate() {
        assert!(
            count > 9000 && count < 11000,
            "Bucket {} has count {}, expected ~10000",
            i,
            count
        );
    }
}
