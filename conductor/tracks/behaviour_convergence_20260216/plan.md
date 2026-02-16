# Implementation Plan - Behaviour Convergence (C vs Rust)

This plan focuses on achieving functional parity between the Rust and C engines using the `synchronized_comparison` suite as the primary validation tool.

## Phase 1: Character Generation & Initial State Alignment [checkpoint: P1-INIT]
- [ ] Task: Alignment of Constitution-based HP bonuses
    - [ ] Write failing test in `nh-compare` for roles with high/low CON
    - [ ] Implement CON-based HP rolling in `nh-core::player::init`
    - [ ] Verify Turn 0 parity for all 13 roles
- [ ] Task: Initial Inventory count and property parity
    - [ ] Write failing test comparing starting inventory counts and BUC/enchantment
    - [ ] Refine `u_init` starting item generation to match C distribution
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Character Generation & Initial State Alignment' (Protocol in workflow.md)

## Phase 2: Map Generation Parity (Core Algorithms) [checkpoint: P2-MAP]
- [ ] Task: Port `mklev.c` core room placement logic
    - [ ] Write failing test using a fixed seed that produces divergent room layouts
    - [ ] Port room selection and placement algorithm to `nh-core::dungeon::generation`
- [ ] Task: Port `mkmaze.c` and special level logic
    - [ ] Implement parity for maze-style level generation
- [ ] Task: Port Door and Secret Passage placement
    - [ ] Fix desync where doors are missing or in incorrect positions
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Map Generation Parity (Core Algorithms)' (Protocol in workflow.md)

## Phase 3: Core Gameplay Loop & Mechanics [checkpoint: P3-MECHANICS]
- [ ] Task: Combat Formula Convergence (To-hit and Damage)
    - [ ] Write failing combat desync tests in `synchronized_comparison`
    - [ ] Port `mhitm.c` and `uhitm.c` formulas to `nh-core::combat`
- [ ] Task: Regeneration Timing Alignment (HP/Energy)
    - [ ] Fix Turn 0 regeneration desync and align frequency with C levels
- [ ] Task: Dungeon Interaction Parity (Traps/Stairs)
    - [ ] Port trap triggering and stair transition logic from C
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Core Gameplay Loop & Mechanics' (Protocol in workflow.md)

## Phase 4: Long-Turn Stability & Final Validation [checkpoint: P4-STABILITY]
- [ ] Task: 1,000 Turn Randomized Stress Test
    - [ ] Run `synchronized_comparison` for 1,000+ turns across 5+ seeds
    - [ ] Resolve any remaining edge-case desyncs identified in long runs
- [ ] Task: Stress Scenario Validation
    - [ ] Ensure "Gnomish Mines Gauntlet" and "Inventory Stress" pass with 100% parity
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Long-Turn Stability & Final Validation' (Protocol in workflow.md)
