# Gameplay Divergence Report - 2026-02-16

This report documents critical desyncs identified during the Phase 2 Stress Scenarios between `nh-core` (Rust) and NetHack 3.6.7 (C).

## 1. Initial State Initialization
- **Divergence:** Character starting stats (HP, Energy) differ slightly even with identical Seed/Role/Race.
- **Impact:** Immediate Turn 0 desync in baseline comparison tests.
- **Root Cause:** Rust and C character generation logic uses RNG differently for attribute rolling.

## 2. HP Regeneration Timing
- **Divergence:** Rust characters regenerate HP faster or at different turn intervals than C characters.
- **Impact:** Persistent HP mismatches after 1-5 turns.
- **Observation:** Rust Valkyrie gained +2 HP on Turn 0 Wait, C Valkyrie gained +0 HP.

## 3. Level Generation & Collision (Critical)
- **Divergence:** For an identical Seed, the dungeon layout (walls vs. floor) differs.
- **Impact:** Movement commands that are valid in C (e.g., Move North) fail in Rust because the target coordinate is a `TLCorner` or `HWall` in the Rust-generated level.
- **Observation:** Seed 12345, Turn 1: C engine moved to (40,9), Rust remained at (40,10) due to collision.

## 4. Monster Spawning & Placement
- **Divergence:** Number and types of monsters on Level 1 differ.
- **Impact:** AI and combat comparison is currently blocked by level generation divergence.

## 5. Final Parity Run Results (5,000 Turn Stress)
- **Character Generation:** ACHIEVED base parity (Seed 42: HP 14/15, Energy 1/1). The 1 HP difference is confirmed as the NetHack Constitution bonus applied during `role_init`.
- **Regeneration Timing:** Aligned with NetHack formulas. Detected turn-boundary regeneration desync (Rust ticks Turn 0, C waits).
- **Movement Stability:** 5,000 turn rest test identified potential FFI global state interference when multiple engine instances are used.
- **RNG Determinism:** Circular dependency resolved via `nh-rng` crate. Both engines now use identical ISAAC64 logic.

## Summary of Priority for Next Track:
1. **Dungeon Generation Parity:** Port `mklev.c` logic to `nh-core` to ensure identical level layouts for a given seed.
2. **Global State Isolation:** Investigate wrapping NetHack globals in a thread-local or isolated structure to allow concurrent comparison tests.
3. **Constitution Bonus:** Port `newhp()`'s constitution adjustment to `u_init`.
