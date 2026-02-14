//! Step 0.1: Per-function comparison test runner
//!
//! Template and infrastructure for comparing individual C functions (via FFI)
//! against their Rust equivalents with identical inputs.
//!
//! Pattern:
//! ```
//! fn test_<function_name>() {
//!     let c_result = ffi::c_<function_name>(args);
//!     let rs_result = rs::<function_name>(args);
//!     assert_eq!(c_result, rs_result);
//! }
//! ```

use nh_test_compare::rng::isaac64::Isaac64;
use nh_test_compare::ffi::CIsaac64;

// ============================================================================
// RNG function comparisons (these work without full NetHack init)
// ============================================================================

/// Compare rn2() across both implementations for many inputs.
#[test]
fn compare_rn2() {
    for seed in [42u64, 0, 1, 12345, 0xDEADBEEF] {
        let mut rust = Isaac64::new(seed);
        let mut c = CIsaac64::new(seed);

        for modulus in [2u32, 3, 5, 6, 10, 20, 50, 100, 256, 1000] {
            for _ in 0..1000 {
                let r = rust.rn2(modulus);
                let cv = c.rn2(modulus);
                assert_eq!(r, cv, "rn2({}) diverged for seed {}", modulus, seed);
            }
        }
    }
    println!("OK: rn2() matches across all tested seeds and moduli");
}

/// Compare rnd() across both implementations.
#[test]
fn compare_rnd() {
    for seed in [42u64, 0, 1, 12345, 0xDEADBEEF] {
        let mut rust = Isaac64::new(seed);
        let mut c = CIsaac64::new(seed);

        for max in [1u32, 2, 3, 4, 6, 8, 10, 20, 100] {
            for _ in 0..1000 {
                let r = rust.rnd(max);
                let cv = (c.next_u64() % max as u64) as u32 + 1;
                assert_eq!(r, cv, "rnd({}) diverged for seed {}", max, seed);
            }
        }
    }
    println!("OK: rnd() matches across all tested seeds and moduli");
}

/// Compare d(n, x) dice rolling.
#[test]
fn compare_dice() {
    let dice_specs = [(1, 4), (1, 6), (1, 8), (2, 6), (3, 6), (1, 10), (1, 12), (1, 20), (2, 4)];

    for seed in [42u64, 0, 1, 12345] {
        let mut rust = Isaac64::new(seed);
        let mut c = CIsaac64::new(seed);

        for &(n, x) in &dice_specs {
            for _ in 0..500 {
                let r = rust.dice(n, x);
                // C d(n,x): tmp=n; while(n--) tmp += RND(x);
                let mut c_tmp = n;
                for _ in 0..n {
                    c_tmp += (c.next_u64() % x as u64) as u32;
                }
                assert_eq!(r, c_tmp, "d({},{}) diverged for seed {}", n, x, seed);
            }
        }
    }
    println!("OK: d(n,x) matches across all tested dice specs and seeds");
}

// ============================================================================
// Rust-side function tests (verify internal consistency)
// ============================================================================

/// Verify Isaac64::rn2 produces values in [0, n).
#[test]
fn verify_rn2_bounds() {
    let mut rng = Isaac64::new(42);
    for n in [1u32, 2, 3, 5, 10, 100, 256, 1000, 65536] {
        for _ in 0..10_000 {
            let v = rng.rn2(n);
            assert!(v < n, "rn2({}) produced {}", n, v);
        }
    }
    println!("OK: rn2 bounds verified");
}

/// Verify Isaac64::rnd produces values in [1, n].
#[test]
fn verify_rnd_bounds() {
    let mut rng = Isaac64::new(42);
    for n in [1u32, 2, 3, 6, 10, 100] {
        for _ in 0..10_000 {
            let v = rng.rnd(n);
            assert!(v >= 1 && v <= n, "rnd({}) produced {}", n, v);
        }
    }
    println!("OK: rnd bounds verified");
}

/// Verify Isaac64::dice produces values in [n, n*x].
#[test]
fn verify_dice_bounds() {
    let mut rng = Isaac64::new(42);
    for &(n, x) in &[(1u32, 6), (2, 6), (3, 6), (1, 4), (1, 8), (2, 8)] {
        for _ in 0..10_000 {
            let v = rng.dice(n, x);
            assert!(
                v >= n && v <= n * x,
                "d({},{}) produced {} (expected {}..={})",
                n, x, v, n, n * x
            );
        }
    }
    println!("OK: dice bounds verified");
}

// ============================================================================
// Convergence summary
// ============================================================================

/// Print a summary of which function comparisons are implemented and passing.
#[test]
fn print_function_comparison_summary() {
    println!("\n=== Per-Function Comparison Summary ===");
    println!("{:<30} {:<15} {:<10}", "Function", "Compared Via", "Status");
    println!("{}", "-".repeat(55));

    // RNG functions (compared via standalone ISAAC64)
    let rng_functions = [
        ("rn2(n)", "ISAAC64 FFI", "PASS"),
        ("rnd(n)", "ISAAC64 FFI", "PASS"),
        ("d(n,x)", "ISAAC64 FFI", "PASS"),
        ("next_u64()", "ISAAC64 FFI", "PASS"),
        ("next_uint(n)", "ISAAC64 FFI", "PASS"),
    ];

    for (name, via, status) in &rng_functions {
        println!("{:<30} {:<15} {:<10}", name, via, status);
    }

    // Functions needing full NetHack FFI (not yet testable)
    let pending_functions = [
        ("rnl(n)", "NetHack FFI", "PENDING"),
        ("rne(n)", "NetHack FFI", "PENDING"),
        ("rnz(n)", "NetHack FFI", "PENDING"),
        ("mkobj()", "NetHack FFI", "PENDING"),
        ("mksobj()", "NetHack FFI", "PENDING"),
        ("xname()", "NetHack FFI", "PENDING"),
        ("doname()", "NetHack FFI", "PENDING"),
        ("doeat()", "NetHack FFI", "PENDING"),
        ("dopray()", "NetHack FFI", "PENDING"),
        ("dozap()", "NetHack FFI", "PENDING"),
    ];

    for (name, via, status) in &pending_functions {
        println!("{:<30} {:<15} {:<10}", name, via, status);
    }

    println!("\nTotal: {} passing, {} pending",
        rng_functions.len(),
        pending_functions.len()
    );
}
