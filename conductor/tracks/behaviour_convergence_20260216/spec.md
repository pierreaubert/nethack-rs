# Specification - Behaviour Convergence (C vs Rust)

## Overview
This track aims to achieve strict behavioral parity between `nh-core` (Rust) and NetHack 3.6.7 (C). The primary driver for convergence will be the existing `nh-compare` integration suite, specifically the `synchronized_comparison` test. We will iteratively detect desyncs, analyze the underlying logic differences, and port or refine Rust implementations until 1:1 functional compatibility is achieved across core gameplay systems.

## Functional Requirements
- **Core Subsystem Convergence:**
    - **Combat Mechanics:** Align to-hit/damage formulas, weapon skills, and basic monster AI.
    - **Inventory & Items:** Implement parity for container behavior, BUC status effects, and weight accumulation.
    - **Dungeon Interaction:** Sync logic for secret doors, traps, stairs, and level transitions.
    - **Magic & Spells:** Align casting success rates, spell effects, and energy regeneration.
- **Map Generation Parity:**
    - Port core level generation algorithms from C (`mklev.c`, `mkmaze.c`, `mkroom.c`) to Rust.
    - Ensure identical room layouts, door placements, and accessibility for any given seed.
- **Initial State Alignment:**
    - Close the remaining gaps in character generation (e.g., Constitution-based HP bonuses) to ensure matching Turn 0 states for all roles.

## Non-Functional Requirements
- **Deterministic Execution:** The Rust engine must produce identical state changes to the C engine for every turn in a synchronized sequence.
- **Test-Driven Refinement:** Every fix must be validated by running `cargo test -p nh-compare --test synchronized_comparison`.

## Acceptance Criteria
- [ ] `synchronized_comparison` successfully completes 1,000+ turns across 5+ random seeds without a single desync.
- [ ] Side-by-side map comparison confirms 100% identical level layouts (walls, floors, doors).
- [ ] All "Stress Scenarios" (Gnomish Mines Gauntlet, Inventory Stress) pass with zero divergences.
- [ ] Initial HP, Energy, and starting inventory counts match the C engine exactly for all 13 roles.

## Out of Scope
- Visual parity in TUI or Bevy frontends.
- Porting of non-gameplay C subsystems (e.g., UI code, OS-specific terminal handling).
