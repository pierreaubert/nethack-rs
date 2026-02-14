//! Step 1.2: RNG call-site tracing tests
//!
//! Tests the RNG tracing mode that logs every call for diffing
//! between C and Rust implementations.

use nh_test::rng::isaac64::Isaac64;

/// Test that tracing mode records calls correctly.
#[test]
fn test_tracing_basic() {
    let mut rng = Isaac64::new(42);
    rng.enable_tracing();

    // Make some calls
    let v1 = rng.rn2(10);
    let v2 = rng.rnd(6);
    let v3 = rng.rn2(100);

    let trace = rng.trace();
    assert_eq!(trace.len(), 3);

    assert_eq!(trace[0].func, "rn2");
    assert_eq!(trace[0].arg, 10);
    assert_eq!(trace[0].result, v1 as u64);

    assert_eq!(trace[1].func, "rnd");
    assert_eq!(trace[1].arg, 6);
    assert_eq!(trace[1].result, v2 as u64);

    assert_eq!(trace[2].func, "rn2");
    assert_eq!(trace[2].arg, 100);
    assert_eq!(trace[2].result, v3 as u64);
}

/// Test that disabling tracing stops recording.
#[test]
fn test_tracing_disable() {
    let mut rng = Isaac64::new(42);
    rng.enable_tracing();

    rng.rn2(10);
    rng.rn2(10);
    assert_eq!(rng.trace().len(), 2);

    rng.disable_tracing();
    rng.rn2(10);
    rng.rn2(10);
    assert_eq!(rng.trace().len(), 2); // no new entries
}

/// Test call_count tracks total raw u64 consumption.
#[test]
fn test_call_count() {
    let mut rng = Isaac64::new(42);
    assert_eq!(rng.call_count(), 0);

    rng.rn2(10); // 1 raw u64
    assert_eq!(rng.call_count(), 1);

    rng.rnd(6); // 1 raw u64
    assert_eq!(rng.call_count(), 2);

    rng.dice(3, 6); // 3 raw u64s (3 calls to rn2)
    assert_eq!(rng.call_count(), 5);
}

/// Compare trace between two identically-seeded RNGs.
/// This is the template for comparing C and Rust RNG call sequences.
#[test]
fn test_trace_comparison() {
    let mut rng1 = Isaac64::new(42);
    let mut rng2 = Isaac64::new(42);

    rng1.enable_tracing();
    rng2.enable_tracing();

    // Simulate a game sequence: move, combat check, damage roll
    for _ in 0..10 {
        let _movement_check = rng1.rn2(8); // direction
        let _combat_check = rng1.rn2(20); // to-hit
        let _damage = rng1.dice(1, 6); // damage

        let _movement_check = rng2.rn2(8);
        let _combat_check = rng2.rn2(20);
        let _damage = rng2.dice(1, 6);
    }

    let trace1 = rng1.trace();
    let trace2 = rng2.trace();

    assert_eq!(trace1.len(), trace2.len());
    for (i, (t1, t2)) in trace1.iter().zip(trace2.iter()).enumerate() {
        assert_eq!(
            t1.raw, t2.raw,
            "Trace mismatch at entry {}: raw {:#x} vs {:#x}",
            i, t1.raw, t2.raw
        );
        assert_eq!(t1.func, t2.func);
        assert_eq!(t1.arg, t2.arg);
        assert_eq!(t1.result, t2.result);
    }
}

/// Test rne and rnz functions produce reasonable values.
#[test]
fn test_rne_rnz_basics() {
    let mut rng = Isaac64::new(42);

    // rne(4) at level 1: should return 1..5 with 1 most common
    let mut counts = [0u32; 6];
    for _ in 0..10_000 {
        let v = rng.rne(4, 1);
        assert!(v >= 1 && v <= 5, "rne(4) produced {}", v);
        counts[v as usize] += 1;
    }
    // 1 should be most common (75% chance per iteration)
    assert!(counts[1] > counts[2], "rne: 1 should be more common than 2");
    assert!(counts[2] > counts[3], "rne: 2 should be more common than 3");

    // rnz(100) at level 1
    for _ in 0..1_000 {
        let v = rng.rnz(100, 1);
        // rnz can produce wide range, just check it doesn't panic
        let _ = v; // exercises the code path
    }
}

/// Print a sample trace for visual inspection.
#[test]
fn test_print_sample_trace() {
    let mut rng = Isaac64::new(42);
    rng.enable_tracing();

    // Simulate a few turns of a game
    for turn in 0..5 {
        let _dir = rng.rn2(8);
        let _hit = rng.rn2(20);
        if rng.rn2(3) == 0 {
            let _dmg = rng.dice(1, 6);
        }
        let _ = turn;
    }

    println!("\n=== Sample RNG Trace (seed=42, 5 turns) ===");
    println!("{:<6} {:<10} {:<6} {:<8} {:<20}", "Seq", "Function", "Arg", "Result", "Raw");
    println!("{}", "-".repeat(55));
    for entry in rng.trace() {
        println!(
            "{:<6} {:<10} {:<6} {:<8} {:#018x}",
            entry.seq, entry.func, entry.arg, entry.result, entry.raw
        );
    }
    println!("Total raw u64 calls: {}", rng.call_count());
}
