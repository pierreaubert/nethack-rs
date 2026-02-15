# Implementation Plan - Gameplay Parity & Gap Closure

This plan focuses on expanding `nh-compare` to detect state divergences and porting missing C features to `nh-core` to achieve strict parity.

## Phase 1: Comparison Infrastructure Expansion [checkpoint: P1-INFRA]
- [ ] Task: Expand `nh-compare` State Extraction
    - [ ] Write tests in `nh-compare` that fail when inventory/monster data is missing from the state dump
    - [ ] Implement deeper state extraction for Inventory (BUC, charges, weight) in both C-FFI and Rust
    - [ ] Implement deeper state extraction for Monster status and AI flags
- [ ] Task: Formalize Integration Test Suite
    - [ ] Create a long-running integration test that executes 1,000+ turns of synchronized RNG movement
    - [ ] Implement a standardized "State Diff" reporter that outputs JSON differences on failure
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Comparison Infrastructure Expansion' (Protocol in workflow.md)

## Phase 2: Identification & Gap Analysis [checkpoint: P2-GAPS]
- [ ] Task: Execute Stress Scenarios
    - [ ] Define and implement the "Gnomish Mines Gauntlet" stress script
    - [ ] Define and implement the "Inventory Management & Weight" stress script
- [ ] Task: Document Divergence Points
    - [ ] Run the expanded suite and identify the first 3-5 critical missing features in `nh-core` causing desyncs
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Identification & Gap Analysis' (Protocol in workflow.md)

## Phase 3: Feature Porting & Parity Restoration [checkpoint: P3-PORTING]
- [ ] Task: Port Missing Gameplay Logic (Iterative)
    - [ ] **Feature A (Identified in P2):** Write a failing parity test, port the C logic to `nh-core`, and verify parity
    - [ ] **Feature B (Identified in P2):** Write a failing parity test, port the C logic to `nh-core`, and verify parity
    - [ ] **Feature C (Identified in P2):** Write a failing parity test, port the C logic to `nh-core`, and verify parity
- [ ] Task: Verify RNG Determinism
    - [ ] Ensure all ported features correctly consume RNG in the same order as the C implementation
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Feature Porting & Parity Restoration' (Protocol in workflow.md)

## Phase 4: Final Validation & CI Integration [checkpoint: P4-VALIDATION]
- [ ] Task: CI Pipeline Hardening
    - [ ] Integrate the `nh-compare` suite into the project's CI with strict pass/fail thresholds
- [ ] Task: Final Parity Run
    - [ ] Verify 5,000+ turn stability across multiple random seeds without desync
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Final Validation & CI Integration' (Protocol in workflow.md)
