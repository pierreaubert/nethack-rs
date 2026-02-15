# Specification - Shared Tile Representation System

## Overview
Implement a unified system for representing game tiles (monsters, objects, dungeon features) that can be consumed by both the TUI (`nh-tui`) and Bevy (`nh-bevy`) frontends. This ensures visual consistency and fulfills the "Adaptive & Versatile" design guideline.

## Functional Requirements
- Define a central `Tile` type in a shared location (e.g., `nh-core` or a new `nh-graphics` crate).
- Map every NetHack game object to a unique `Tile` identifier.
- Support ASCII/Unicode mapping for terminal-based rendering.
- Support asset-path/index mapping for sprite-based rendering in Bevy.
- Ensure the representation is `no_std` compatible for PolkaVM targets.

## Non-Functional Requirements
- **Performance:** Mapping should be O(1) or O(log N) to avoid slowing down rendering loops.
- **Extensibility:** Modders should be able to easily add new tile mappings.

## Acceptance Criteria
- [ ] A shared `Tile` data structure exists.
- [ ] TUI can render a simple room using the new tile system.
- [ ] Bevy can display a sprite based on the same tile identifier.
- [ ] The system compiles for the `wasm32-unknown-unknown` target.
