//! Engraving system (engrave.c)
//!
//! Engraving types from NetHack:
//! - DUST (0): Written in dust, easily erased by walking
//! - ENGRAVE (1): Engraved with hard tool, permanent
//! - BURN (2): Burned with wand of fire, permanent
//! - MARK (3): Written with marker
//! - BLOOD (4): Written in blood
//! - HEADSTONE (5): Grave inscription

use crate::action::ActionResult;
use crate::dungeon::{Engraving, EngravingType};
use crate::gameloop::GameState;

/// Engrave on the floor
pub fn do_engrave(state: &mut GameState, text: &str) -> ActionResult {
    let x = state.player.pos.x;
    let y = state.player.pos.y;

    if text.is_empty() {
        state.message("You write in the dust with your fingers.");
        return ActionResult::Success;
    }

    // Check for existing engraving at this location
    let existing_idx = state.current_level.engravings
        .iter()
        .position(|e| e.x == x && e.y == y);

    if let Some(idx) = existing_idx {
        // Replace existing engraving
        state.current_level.engravings[idx].text = text.to_string();
        state.current_level.engravings[idx].engr_type = EngravingType::Dust;
    } else {
        // Create new engraving
        let engraving = Engraving::new(x, y, text.to_string(), EngravingType::Dust);
        state.current_level.engravings.push(engraving);
    }

    // Special case: Elbereth grants wisdom (from C: exercise(A_WIS, TRUE))
    if text.to_lowercase().contains("elbereth") {
        state.message("You feel wise.");
    }

    state.message(format!("You write \"{}\" in the dust.", text));
    ActionResult::Success
}

/// Engrave with a specific tool (athame, wand, etc.)
pub fn do_engrave_with_tool(
    state: &mut GameState,
    text: &str,
    engr_type: EngravingType,
) -> ActionResult {
    let x = state.player.pos.x;
    let y = state.player.pos.y;

    if text.is_empty() {
        return ActionResult::NoTime;
    }

    // Check for existing engraving at this location
    let existing_idx = state.current_level.engravings
        .iter()
        .position(|e| e.x == x && e.y == y);

    if let Some(idx) = existing_idx {
        // Replace existing engraving
        state.current_level.engravings[idx].text = text.to_string();
        state.current_level.engravings[idx].engr_type = engr_type;
    } else {
        // Create new engraving
        let engraving = Engraving::new(x, y, text.to_string(), engr_type);
        state.current_level.engravings.push(engraving);
    }

    let msg = match engr_type {
        EngravingType::Dust => format!("You write \"{}\" in the dust.", text),
        EngravingType::Engrave => format!("You engrave \"{}\" on the floor.", text),
        EngravingType::Burn => format!("You burn \"{}\" into the floor.", text),
        EngravingType::Mark => format!("You write \"{}\" on the floor.", text),
        EngravingType::BloodStain => format!("You scrawl \"{}\" in blood.", text),
        EngravingType::Headstone => format!("You carve \"{}\" on the headstone.", text),
    };
    state.message(msg);

    // Special case: Elbereth grants wisdom
    if text.to_lowercase().contains("elbereth") {
        state.message("You feel wise.");
    }

    ActionResult::Success
}

/// Read engraving at current location
pub fn read_engrave(state: &GameState) -> Option<String> {
    let x = state.player.pos.x;
    let y = state.player.pos.y;

    state.current_level.engravings
        .iter()
        .find(|e| e.x == x && e.y == y)
        .map(|e| e.text.clone())
}

/// Get engraving at a specific position
pub fn engrave_at(state: &GameState, x: i8, y: i8) -> Option<&Engraving> {
    state.current_level.engravings
        .iter()
        .find(|e| e.x == x && e.y == y)
}

/// Wipe/erase engraving at position (e.g., when walking over dust)
pub fn wipe_engrave_at(state: &mut GameState, x: i8, y: i8) {
    // Only dust engravings can be wiped
    if let Some(idx) = state.current_level.engravings
        .iter()
        .position(|e| e.x == x && e.y == y && e.engr_type == EngravingType::Dust)
    {
        state.current_level.engravings.remove(idx);
    }
}

/// Check if "Elbereth" is engraved at position (scares most monsters)
pub fn has_elbereth(state: &GameState, x: i8, y: i8) -> bool {
    state.current_level.engravings
        .iter()
        .any(|e| e.x == x && e.y == y && e.text.to_lowercase().contains("elbereth"))
}
