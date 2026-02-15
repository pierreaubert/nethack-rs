# Implementation Plan - Per-Item Icon System

## Phase 1: `nh-assets` Infrastructure & Core Logic [checkpoint: 386747c]
- [x] Task: Scaffold the `nh-assets` crate (08d53a8)
    - [ ] Create `crates/nh-assets` with basic `Cargo.toml` and `src/lib.rs`
    - [ ] Add `nh-assets` to the workspace `members` in the root `Cargo.toml`
- [x] Task: Define the Mapping Schema and Data Structures (842cf17)
    - [ ] Write unit tests for the mapping schema serialization/deserialization
    - [ ] Implement `ItemIconDefinition` and `AssetMapping` structs (supporting TUI symbols and Bevy paths)
- [x] Task: Implement the Lookup Registry (4b3a7e9)
    - [ ] Write tests for the lookup registry (base type, state, and material-based matching)
    - [ ] Implement the `AssetRegistry` with support for loading from a configuration file (YAML/JSON)
- [x] Task: Conductor - User Manual Verification 'Phase 1: nh-assets Infrastructure & Core Logic' (Protocol in workflow.md) (386747c)

## Phase 2: Asset Generation & Initial Mapping [checkpoint: e68f6ed]
- [x] Task: Setup Asset Generation Pipeline (e68f6ed)
    - [ ] Create a script or documentation for the "banana" icon generation workflow
    - [ ] Define the aesthetic parameters (palette, size, style) in a shared configuration
- [x] Task: Populate Initial Core Assets (e68f6ed)
    - [ ] Generate and map a subset of core items (e.g., Long Sword, Potion, Leather Armor)
    - [ ] Write tests to ensure these core assets are correctly resolved by the registry
- [x] Task: Conductor - User Manual Verification 'Phase 2: Asset Generation & Initial Mapping' (Protocol in workflow.md) (e68f6ed)

## Phase 3: `nh-tui` Integration [checkpoint: aa43056]
- [x] Task: Update `nh-tui` Map Rendering (aa43056)
    - [ ] Write tests for TUI icon-to-cell conversion
    - [ ] Update the map rendering loop in `nh-tui` to use `nh-assets` for item symbols and colors
- [x] Task: Integrate Icons into TUI Inventory (aa43056)
    - [ ] Write tests for inventory item rendering with icons
    - [ ] Update `nh-tui` inventory menus to display item-specific icons/symbols
- [x] Task: Conductor - User Manual Verification 'Phase 3: TUI Integration' (Protocol in workflow.md) (aa43056)

## Phase 4: Bevy & UI Integration [checkpoint: 07f41ea]
- [x] Task: Implement Bevy Asset Loading System (07f41ea)
    - [ ] Write tests for Bevy texture atlas/sprite loading from the mapping registry
    - [ ] Implement a Bevy system to load and manage item textures based on `nh-assets`
- [x] Task: Update Bevy Map and UI Rendering (07f41ea)
    - [ ] Update Bevy item entities to use the correct sprites in the dungeon view
    - [ ] Integrate item icons into the Bevy inventory and equipment UI panels
- [x] Task: Conductor - User Manual Verification 'Phase 4: Bevy & UI Integration' (Protocol in workflow.md) (07f41ea)

## Phase 5: Validation & Coverage
- [ ] Task: Implement Strict Coverage Validation
    - [ ] Write a validation tool/test that checks for missing mappings across all defined items
    - [ ] Implement build-time or startup checks that fail if critical items are unmapped
- [ ] Task: Conductor - User Manual Verification 'Phase 5: Validation & Coverage' (Protocol in workflow.md)
