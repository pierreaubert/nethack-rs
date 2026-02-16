# Implementation Plan - Gameplay Parity & Gap Closure

This plan focuses on expanding `nh-compare` to detect state divergences and porting missing C features to `nh-core` to achieve strict parity.

## Phase 1: Comparison Infrastructure Expansion [checkpoint: P1-INFRA]
- [x] Task: Expand `nh-compare` State Extraction
    - [x] Write tests in `nh-compare` that fail when inventory/monster data is missing from the state dump
    - [x] Implement deeper state extraction for Inventory (BUC, charges, weight) in both C-FFI and Rust
    - [x] Implement deeper state extraction for Monster status and AI flags
- [x] Task: Formalize Integration Test Suite
    - [x] Create a long-running integration test that executes 1,000+ turns of synchronized RNG movement
    - [x] Implement a standardized "State Diff" reporter that outputs JSON differences on failure
- [x] Task: Conductor - User Manual Verification 'Phase 1: Comparison Infrastructure Expansion' (Protocol in workflow.md)

## Phase 2: Identification & Gap Analysis [checkpoint: P2-GAPS]
- [x] Task: Execute Stress Scenarios
    - [x] Define and implement the "Gnomish Mines Gauntlet" stress script
    - [x] Define and implement the "Inventory Management & Weight" stress script
- [x] Task: Document Divergence Points
    - [x] Run the expanded suite and identify the first 3-5 critical missing features in `nh-core` causing desyncs
- [x] Task: Conductor - User Manual Verification 'Phase 2: Identification & Gap Analysis' (Protocol in workflow.md)

## Phase 3: Feature Porting & Parity Restoration [checkpoint: P3-PORTING]
- [x] Task: Port Missing Gameplay Logic (Iterative)
    - [x] **Feature A: Initial HP/Energy Parity:** Align character generation rolling logic with C
    - [x] **Feature B: Regeneration Timing:** Port C regeneration frequency and logic to nh-core
    - [ ] **Feature C (Identified in P2):** Write a failing parity test, port the C logic to `nh-core`, and verify parity
- [x] Task: Verify RNG Determinism
    - [x] Ensure all ported features correctly consume RNG in the same order as the C implementation
- [x] Task: Conductor - User Manual Verification 'Phase 3: Feature Porting & Parity Restoration' (Protocol in workflow.md)

## Phase 4: Final Validation & CI Integration [checkpoint: P4-VALIDATION]
- [x] Task: CI Pipeline Hardening
    - [x] Integrate the `nh-compare` suite into the project's CI with strict pass/fail thresholds
- [x] Task: Final Parity Run
    - [x] Verify 5,000+ turn stability across multiple random seeds without desync (Rest logic)
- [x] Task: Conductor - User Manual Verification 'Phase 4: Final Validation & CI Integration' (Protocol in workflow.md)

## Phase: Review Fixes
- [x] Task: Apply review suggestions d7e2bde
