//! Diagnostic: compare Rust ISAAC64 fresh-init state against C's after
//! `init_isaac64(seed)`. The synchronized_comparison test imports C state
//! into Rust; this one compares the post-init state directly to find where
//! they diverge.

use nh_core::CGameEngineTrait;
use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use serde_json::Value;
use serial_test::serial;

#[test]
#[serial]
fn rng_init_diverge() {
    let seed = 1u64;

    // C: init engine + reset to seed → calls init_isaac64(seed) and zeros counter.
    let mut c = CGameEngine::new();
    c.init("Wizard", "Human", 1, 0).expect("C init");
    c.reset(seed).expect("C reset");

    // Export C state immediately, before any draws.
    let rng_json = c.export_rng_state();
    let v: Value = serde_json::from_str(&rng_json).expect("parse");
    let c_a = v["a"].as_str().unwrap().parse::<u64>().unwrap();
    let c_b = v["b"].as_str().unwrap().parse::<u64>().unwrap();
    let c_c = v["c"].as_str().unwrap().parse::<u64>().unwrap();
    let c_n = v["n"].as_u64().unwrap() as usize;
    let c_r: Vec<u64> = v["r"].as_array().unwrap().iter()
        .map(|x| x.as_str().unwrap().parse::<u64>().unwrap()).collect();
    let c_m: Vec<u64> = v["m"].as_array().unwrap().iter()
        .map(|x| x.as_str().unwrap().parse::<u64>().unwrap()).collect();

    // Rust fresh init.
    let rust_rng = nh_rng::Isaac64::new(seed);
    let (rs_n, rs_r, rs_m, rs_a, rs_b, rs_c, _cc) = rust_rng.export_c_fields();

    println!("=== C state ===");
    println!("a={:016x} b={:016x} c={:016x} n={}", c_a, c_b, c_c, c_n);
    println!("r[0..4] = {:016x} {:016x} {:016x} {:016x}",
        c_r[0], c_r[1], c_r[2], c_r[3]);
    println!("r[252..256] = {:016x} {:016x} {:016x} {:016x}",
        c_r[252], c_r[253], c_r[254], c_r[255]);
    println!("m[0..4] = {:016x} {:016x} {:016x} {:016x}",
        c_m[0], c_m[1], c_m[2], c_m[3]);

    println!("\n=== Rust state ===");
    println!("a={:016x} b={:016x} c={:016x} n={}", rs_a, rs_b, rs_c, rs_n);
    println!("r[0..4] = {:016x} {:016x} {:016x} {:016x}",
        rs_r[0], rs_r[1], rs_r[2], rs_r[3]);
    println!("r[252..256] = {:016x} {:016x} {:016x} {:016x}",
        rs_r[252], rs_r[253], rs_r[254], rs_r[255]);
    println!("m[0..4] = {:016x} {:016x} {:016x} {:016x}",
        rs_m[0], rs_m[1], rs_m[2], rs_m[3]);

    let mut mismatches = 0;
    if c_a != rs_a { println!("a MISMATCH: C={:x} Rust={:x}", c_a, rs_a); mismatches += 1; }
    if c_b != rs_b { println!("b MISMATCH: C={:x} Rust={:x}", c_b, rs_b); mismatches += 1; }
    if c_c != rs_c { println!("c MISMATCH: C={:x} Rust={:x}", c_c, rs_c); mismatches += 1; }
    if c_n != rs_n { println!("n MISMATCH: C={} Rust={}", c_n, rs_n); mismatches += 1; }
    let r_diffs: Vec<usize> = (0..256).filter(|&i| c_r[i] != rs_r[i]).collect();
    let m_diffs: Vec<usize> = (0..256).filter(|&i| c_m[i] != rs_m[i]).collect();
    println!("\nr[] mismatches: {} entries", r_diffs.len());
    println!("m[] mismatches: {} entries", m_diffs.len());
    if !r_diffs.is_empty() {
        for &i in r_diffs.iter().take(5) {
            println!("  r[{}] C={:x} Rust={:x}", i, c_r[i], rs_r[i]);
        }
    }
    if !m_diffs.is_empty() {
        for &i in m_diffs.iter().take(5) {
            println!("  m[{}] C={:x} Rust={:x}", i, c_m[i], rs_m[i]);
        }
    }
    let total_diffs = mismatches + r_diffs.len() + m_diffs.len();
    println!("\nTOTAL field mismatches: {}", total_diffs);
}
