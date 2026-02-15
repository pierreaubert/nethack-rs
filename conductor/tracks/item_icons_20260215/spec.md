# Specification - Per-Item Icon System

## Overview
Implement a comprehensive icon mapping and generation system for NetHack items. This includes providing specific visual representations for the Bevy (graphical) and TUI (terminal) frontends, as well as the UI/Inventory menus. The project will involve setting up a dedicated `nh-assets` crate and an asset generation pipeline (potentially utilizing external generation tools like 'banana' or internal scripts).

## Functional Requirements

### 1. `nh-assets` Crate
- Create a new workspace crate `nh-assets` to serve as the single source of truth for asset mapping.
- Define a unified schema (YAML/JSON) that maps NetHack item identifiers (type, state, material) to:
    - **TUI:** Unicode characters and color attributes.
    - **Bevy/UI:** Sprite paths, texture atlas coordinates, or generation prompts.

### 2. Asset Generation Pipeline
- Establish a workflow for generating icon graphics.
- Define parameters or prompts for "banana" (or equivalent generation tool) to ensure a consistent visual style across all items.
- Automate the ingestion of generated assets into the `nh-assets` library.

### 3. Granular Mapping Logic
- The mapping system must support:
    - **Base Type:** Mapping by item class.
    - **State:** Different icons for identified vs. unidentified items.
    - **Material/Dynamic:** Material-specific variations (e.g., Gold vs. Silver).
    - **Artifacts:** Unique icons for specific named artifacts.

### 4. Frontend & UI Integration
- **nh-tui:** Update the rendering loop for the map and menus (inventory) to use the new mapping.
- **nh-bevy:** Implement an asset loading system for map entities.
- **UI/Menus:** Integrate item icons into inventory lists, equipment panels, and loot windows in both Bevy and TUI (where possible).

### 5. Strict Validation
- If an item lacks a valid mapping or asset, the system must trigger a logged error or build failure to ensure 100% coverage.

## Non-Functional Requirements
- **Visual Consistency:** Generated assets must adhere to a defined aesthetic (e.g., pixel art, consistent palette).
- **Performance:** Efficient lookup for real-time rendering and menu navigation.

## Acceptance Criteria
- [ ] `nh-assets` crate is present and functional.
- [ ] Asset generation pipeline is documented or implemented.
- [ ] `nh-tui` renders items in the dungeon and inventory using the new system.
- [ ] `nh-bevy` displays correct sprites on the map and in UI menus.
- [ ] Missing mappings trigger a clear validation error.
- [ ] Unit tests verify lookup logic and schema validity.

## Out of Scope
- Procedural generation of *gameplay logic* or item stats (only visual/symbolic representation is covered).
