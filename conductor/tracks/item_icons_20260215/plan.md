# Implementation Plan - Per-Item Icon System

## Phase 1: `nh-assets` Infrastructure & Core Logic
- [ ] Task: Scaffold the `nh-assets` crate
    - [ ] Create `crates/nh-assets` with basic `Cargo.toml` and `src/lib.rs`
    - [ ] Add `nh-assets` to the workspace `members` in the root `Cargo.toml`
- [ ] Task: Define the Mapping Schema and Data Structures
    - [ ] Write unit tests for the mapping schema serialization/deserialization
    - [ ] Implement `ItemIconDefinition` and `AssetMapping` structs (supporting TUI symbols and Bevy paths)
- [ ] Task: Implement the Lookup Registry
    - [ ] Write tests for the lookup registry (base type, state, and material-based matching)
    - [ ] Implement the `AssetRegistry` with support for loading from a configuration file (YAML/JSON)
- [ ] Task: Conductor - User Manual Verification 'Phase 1: nh-assets Infrastructure & Core Logic' (Protocol in workflow.md)

## Phase 2: Asset Generation & Initial Mapping
- [ ] Task: Setup Asset Generation Pipeline
    - [ ] Create a script or documentation for the "banana" icon generation workflow
    - [ ] Define the aesthetic parameters (palette, size, style) in a shared configuration
- [ ] Task: Populate Initial Core Assets
    - [ ] Generate and map a subset of core items (e.g., Long Sword, Potion, Leather Armor)
    - [ ] Write tests to ensure these core assets are correctly resolved by the registry
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Asset Generation & Initial Mapping' (Protocol in workflow.md)

## Phase 3: TUI Integration
- [ ] Task: Update `nh-tui` Map Rendering
    - [ ] Write tests for TUI icon-to-cell conversion
    - [ ] Update the map rendering loop in `nh-tui` to use `nh-assets` for item symbols and colors
- [ ] Task: Integrate Icons into TUI Inventory
    - [ ] Write tests for inventory item rendering with icons
    - [ ] Update `nh-tui` inventory menus to display item-specific icons/symbols
- [ ] Task: Conductor - User Manual Verification 'Phase 3: TUI Integration' (Protocol in workflow.md)

## Phase 4: Bevy & UI Integration
- [ ] Task: Implement Bevy Asset Loading System
    - [ ] Write tests for Bevy texture atlas/sprite loading from the mapping registry
    - [ ] Implement a Bevy system to load and manage item textures based on `nh-assets`
- [ ] Task: Update Bevy Map and UI Rendering
    - [ ] Update Bevy item entities to use the correct sprites in the dungeon view
    - [ ] Integrate item icons into the Bevy inventory and equipment UI panels
- [ ] Task: Conductor - User Manual Verification 'Phase 4: Bevy & UI Integration' (Protocol in workflow.md)

## Phase 5: Validation & Coverage
- [ ] Task: Implement Strict Coverage Validation
    - [ ] Write a validation tool/test that checks for missing mappings across all defined items
    - [ ] Implement build-time or startup checks that fail if critical items are unmapped
- [ ] Task: Conductor - User Manual Verification 'Phase 5: Validation & Coverage' (Protocol in workflow.md)
