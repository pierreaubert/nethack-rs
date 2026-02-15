# Specification - Gameplay Parity Enhancement & Gap Closure

## Overview
This track focuses on achieving 100% functional parity between the legacy C engine (NetHack 3.6.7) and the Rust `nh-core`. By expanding the `nh-compare` infrastructure, we will identify missing logic and features in the Rust implementation and port them from the C source to restore and maintain strict gameplay parity.

## Functional Requirements
- **Deep State Comparison (nh-compare Expansion):**
    - Expand comparison logic to include **Inventory** (types, charges, BUC status, weight).
    - Expand comparison logic to include **Monster State** (active monsters, HP, positions, AI flags).
    - Include full **Player Attributes**, intrinsic/extrinsic properties, and hidden state variables.
- **Gap Closure & Feature Porting:**
    - Identify missing gameplay features in `nh-core` that cause desyncs during comparison.
    - Port missing logic (e.g., specific item effects, monster behaviors, dungeon events) from the C source to `nh-core`.
- **Regression Suite Formalization:**
    - Create a suite of long-running parity tests that execute as standard Rust integration tests.
    - Ensure results are formatted for CI failure reporting with detailed diffs.
- **Scenario Stress Testing:**
    - Implement specific test "scripts" for complex behaviors: combat loops, inventory management, spellcasting, and level transitions.
    - Use RNG synchronization to ensure these scenarios remain deterministic across both engines.
- **Diagnostic Tooling:**
    - Enhance the "Immediate Halt" mechanism to produce readable, side-by-side state diffs for rapid debugging of feature gaps.

## Non-Functional Requirements
- **Deterministic Execution:** All tests must produce identical results on every run for a given seed.
- **FFI Stability:** Ensure the C-FFI layer in `nh-compare` is robust enough for long-running simulations.

## Acceptance Criteria
- [ ] `nh-compare` successfully validates Pack/Inventory parity across long-turn sequences.
- [ ] `nh-compare` successfully validates Monster spawn and AI behavior parity.
- [ ] **All features identified as missing during comparison are ported to Rust, resulting in restored parity.**
- [ ] CI pipeline fails and provides a clear diff if a regression or missing feature is detected.
- [ ] Successful execution of 3+ "Stress Scenarios" (e.g., "The Gnomish Mines Gauntlet").

## Out of Scope
- Performance optimization (accuracy and parity take priority).
- Frontend visual comparison.
