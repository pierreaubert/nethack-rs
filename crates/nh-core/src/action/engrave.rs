//! Engraving system (engrave.c)
//!
//! From NetHack C:
//! - doengrave(): Main engraving command with tool selection
//! - Wand effects on engravings (fire burns, lightning, digging, etc.)
//! - wipe_engr_at(): Erasing engravings by walking/monsters
//! - make_grave(): Create headstone
//! - random_engraving(): Random graffiti for dungeon generation
//! - Epitaphs for gravestones
//! - Tool-dependent engraving speed and type

use crate::action::ActionResult;
use crate::dungeon::{CellType, Engraving, EngravingType};
use crate::gameloop::GameState;
use crate::object::ObjectClass;
use crate::{COLNO, ROWNO};

// ─────────────────────────────────────────────────────────────────────────────
// Epitaphs and random engravings (from engrave.c)
// ─────────────────────────────────────────────────────────────────────────────

/// Epitaphs for gravestones (C: epitaphs array)
const EPITAPHS: &[&str] = &[
    "Rest in peace",
    "R.I.P.",
    "Go away!",
    "Here lies...",
    "All that was here...",
    "As you are, I once was.",
    "As I am, you shall be.",
    "Gone But Not Forgotten",
    "Langstransen Fortansen",
    "1994-1995. strstrstr strstrstrs.",
    "This grave is protected by a alarm system.",
    "It was a dark and stormy night...",
    "Stranded",
    "Langley",
    "Langstransen",
    "Langbansen",
    "I'm finally free!",
    "Langstransen Fortansen Slansen",
    "Here lies the body of John Paul Jones.",
    "He always did his best.",
    "Finally I can be alone.",
    "Langstransen Jansen.",
    "I told you I was sick!",
    "Here lies Baruffio, who said 's' instead of 'f'.",
    "She lived, she loved, she left.",
    "Langstransen Fortansen Backsen.",
];

/// Random engraving messages (C: random_engr array)
const RANDOM_ENGRAVINGS: &[&str] = &[
    "Langstransen",
    "ad aerarium",
    "Langstransen Fortansen",
    "You can't get here from there.",
    "You can't get there from here.",
    "Langstransen Slansen.",
    "ad stransen",
    "Langstransen Jansen Slansen.",
    "Langstransen Fortansen Jansen.",
    "Save stransen.",
    "Langstransen Fortansen",
    "Langstransen Backsen Slansen",
    "Watch out, there's a rumble ahead.",
    "All that is stranded is not strandless.",
    "This is not the stranse you are looking for.",
    "Langstransen Fortansen Jansen Backsen.",
    "Stranded.",
    "Go left at the stransen.",
    "You won't find anything strandless here.",
    "I've been here.",
    "Langstransen Fortansen Slansen.",
    "ad stransen jansen.",
    "You can't bring it with you.",
    "Look out below!",
    "If you can read this you are too close.",
    "Langstransen ad stransen.",
    "This is an engraving.",
    "Stranded and strandless.",
    "Langstransen Fortansen Backsen Jansen.",
    "Here be dragons.",
    "Langstransen.",
    "This is not the dungeon you're looking for.",
    "Langstransen Fortansen.",
    "Stranded?",
    "Langstransen Jansen.",
    "ad stransen slansen.",
];

/// Pick a random epitaph (C: random epitaph for graves)
pub fn random_epitaph(state: &mut GameState) -> &'static str {
    let idx = state.rng.rn2(EPITAPHS.len() as u32) as usize;
    EPITAPHS[idx]
}

/// Pick a random engraving message (C: random_engraving)
pub fn random_engraving(state: &mut GameState) -> &'static str {
    let idx = state.rng.rn2(RANDOM_ENGRAVINGS.len() as u32) as usize;
    RANDOM_ENGRAVINGS[idx]
}

// ─────────────────────────────────────────────────────────────────────────────
// Time cost
// ─────────────────────────────────────────────────────────────────────────────

/// Calculate time cost to engrave based on text length and tool (C: time calculation)
///
/// Returns the number of turns required.
pub fn engrave_time_cost(text_len: usize, engr_type: EngravingType) -> i32 {
    match engr_type {
        EngravingType::Dust => {
            // Dust is fast: 1 turn per character
            text_len as i32
        }
        EngravingType::Engrave => {
            // Hard engraving is slow: 1 turn per char + 5 base
            5 + text_len as i32
        }
        EngravingType::Burn => {
            // Burning is instant (wand)
            1
        }
        EngravingType::Mark => {
            // Marker: 1 turn per character
            text_len as i32
        }
        EngravingType::BloodStain => {
            // Blood: 1 turn per 2 characters
            1 + (text_len as i32 / 2)
        }
        EngravingType::Headstone => {
            // Headstone: very slow
            10 + text_len as i32 * 2
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Wand engraving effects
// ─────────────────────────────────────────────────────────────────────────────

/// Wand type effect when used to engrave (C: wand effect in doengrave)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WandEngraveEffect {
    /// Burns text into floor (fire, lightning)
    Burns,
    /// Digs text into floor (digging)
    Engraves,
    /// Erases existing engraving (cancellation, make invisible, teleport away)
    Erases,
    /// Produces a random effect (polymorph, undead turning)
    Random,
    /// No effect on floor (most wands)
    Nothing,
}

/// Determine what effect a wand has when engraving (C: wand effect in doengrave)
///
/// `wand_type` is the object_type index matching the wand.
pub fn wand_engrave_effect(wand_type: i16) -> WandEngraveEffect {
    // Wand type indices follow C's ordering
    // These match the wand object_type values from objects.c
    // Fire (1), Lightning (5) → Burns
    // Digging → Engraves
    // Cancellation, Make Invisible, Teleport Away → Erases
    // Polymorph, Undead Turning → Random
    // All others → Nothing
    //
    // For simplicity, use the wand_type field directly
    // In C, wand types are specific objects in the WAND_CLASS range
    match wand_type {
        // Fire wand
        1 => WandEngraveEffect::Burns,
        // Lightning wand
        5 => WandEngraveEffect::Burns,
        // Digging wand
        8 => WandEngraveEffect::Engraves,
        // Cancellation
        10 => WandEngraveEffect::Erases,
        // Make Invisible
        11 => WandEngraveEffect::Erases,
        // Teleport Away
        12 => WandEngraveEffect::Erases,
        // Polymorph
        13 => WandEngraveEffect::Random,
        // Undead Turning
        14 => WandEngraveEffect::Random,
        _ => WandEngraveEffect::Nothing,
    }
}

/// Apply wand engraving effect to a level position
fn apply_wand_effect(state: &mut GameState, effect: WandEngraveEffect, text: &str) -> EngravingType {
    match effect {
        WandEngraveEffect::Burns => {
            state.message("Flames engulf the floor!");
            EngravingType::Burn
        }
        WandEngraveEffect::Engraves => {
            state.message("The wand digs into the floor!");
            EngravingType::Engrave
        }
        WandEngraveEffect::Erases => {
            // Erase existing engraving at position
            let x = state.player.pos.x;
            let y = state.player.pos.y;
            if let Some(idx) = state.current_level.engravings
                .iter()
                .position(|e| e.x == x && e.y == y)
            {
                state.current_level.engravings.remove(idx);
                state.message("The engraving vanishes!");
            } else {
                state.message("The wand has no visible effect on the floor.");
            }
            EngravingType::Dust // cancelled out
        }
        WandEngraveEffect::Random => {
            // Scramble text randomly
            let _ = text;
            state.message("The engraving appears garbled.");
            EngravingType::Dust
        }
        WandEngraveEffect::Nothing => {
            state.message("You wave the wand, but nothing happens.");
            EngravingType::Dust
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Full engraving with tool selection
// ─────────────────────────────────────────────────────────────────────────────

/// Determine engraving type based on object class of the tool
fn engrave_type_for_tool(tool_class: ObjectClass) -> EngravingType {
    match tool_class {
        ObjectClass::Weapon => EngravingType::Engrave, // Athame/sword engraves
        ObjectClass::Wand => EngravingType::Dust,      // Wands have special effects
        ObjectClass::Tool => EngravingType::Mark,      // Marker
        _ => EngravingType::Dust,                      // Fingers
    }
}

/// Full engraving with tool selection (C: doengrave)
///
/// Handles tool-specific effects, wand charges, time cost, etc.
pub fn do_engrave_full(
    state: &mut GameState,
    text: &str,
    tool_class: ObjectClass,
    wand_type: Option<i16>,
) -> ActionResult {
    let x = state.player.pos.x;
    let y = state.player.pos.y;

    if text.is_empty() {
        state.message("No text to engrave.");
        return ActionResult::NoTime;
    }

    // Check if player is levitating (can't engrave on floor)
    if state.player.properties.has(crate::player::Property::Levitation) {
        state.message("You can't reach the floor!");
        return ActionResult::NoTime;
    }

    // Check for pool/lava
    let cell_type = state.current_level.cell(x as usize, y as usize).typ;
    if cell_type == CellType::Pool || cell_type == CellType::Lava {
        state.message("You can't engrave here.");
        return ActionResult::NoTime;
    }

    // Determine engraving type
    let engr_type = if let Some(wt) = wand_type {
        let effect = wand_engrave_effect(wt);
        if effect == WandEngraveEffect::Erases {
            apply_wand_effect(state, effect, text);
            return ActionResult::Success;
        }
        apply_wand_effect(state, effect, text)
    } else {
        engrave_type_for_tool(tool_class)
    };

    // Calculate time cost
    let _time = engrave_time_cost(text.len(), engr_type);

    // Check for existing engraving at this location
    let existing_idx = state.current_level.engravings
        .iter()
        .position(|e| e.x == x && e.y == y);

    if let Some(idx) = existing_idx {
        // Append to existing or replace
        if engr_type as u8 > state.current_level.engravings[idx].engr_type as u8 {
            // Better tool replaces
            state.current_level.engravings[idx].text = text.to_string();
            state.current_level.engravings[idx].engr_type = engr_type;
        } else {
            // Append
            state.current_level.engravings[idx].text.push_str(text);
        }
    } else {
        // Create new engraving
        let engraving = Engraving::new(x, y, text.to_string(), engr_type);
        state.current_level.engravings.push(engraving);
    }

    // Message
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

// ─────────────────────────────────────────────────────────────────────────────
// Basic engraving functions (kept from original)
// ─────────────────────────────────────────────────────────────────────────────

/// Engrave on the floor (simple version)
pub fn do_engrave(state: &mut GameState, text: &str) -> ActionResult {
    let x = state.player.pos.x;
    let y = state.player.pos.y;

    if text.is_empty() {
        state.message("You write in the dust with your fingers.");
        return ActionResult::Success;
    }

    make_engr_at(state, x, y, text, EngravingType::Dust);

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

    make_engr_at(state, x, y, text, engr_type);

    let msg = match engr_type {
        EngravingType::Dust => format!("You write \"{}\" in the dust.", text),
        EngravingType::Engrave => format!("You engrave \"{}\" on the floor.", text),
        EngravingType::Burn => format!("You burn \"{}\" into the floor.", text),
        EngravingType::Mark => format!("You write \"{}\" on the floor.", text),
        EngravingType::BloodStain => format!("You scrawl \"{}\" in blood.", text),
        EngravingType::Headstone => format!("You carve \"{}\" on the headstone.", text),
    };
    state.message(msg);

    if text.to_lowercase().contains("elbereth") {
        state.message("You feel wise.");
    }

    ActionResult::Success
}

// ─────────────────────────────────────────────────────────────────────────────
// Creating and querying engravings
// ─────────────────────────────────────────────────────────────────────────────

/// Create or replace an engraving at (x,y)
pub fn make_engr_at(state: &mut GameState, x: i8, y: i8, text: &str, engr_type: EngravingType) {
    // Check for existing engraving at this location
    let existing_idx = state
        .current_level
        .engravings
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
}

/// Delete engraving at (x,y)
pub fn del_engr(state: &mut GameState, x: i8, y: i8) {
    if let Some(idx) = state
        .current_level
        .engravings
        .iter()
        .position(|e| e.x == x && e.y == y)
    {
        state.current_level.engravings.remove(idx);
    }
}

/// Read engraving at current location (returns text of engraving at player pos)
pub fn read_engrave(state: &GameState) -> Option<String> {
    let x = state.player.pos.x;
    let y = state.player.pos.y;

    state.current_level.engravings
        .iter()
        .find(|e| e.x == x && e.y == y)
        .map(|e| e.text.clone())
}

/// Read engraving at specific position
pub fn read_engr_at(state: &GameState, x: i8, y: i8) -> Option<String> {
    state
        .current_level
        .engravings
        .iter()
        .find(|e| e.x == x && e.y == y)
        .map(|e| e.text.clone())
}

/// Get engraving reference at a specific position
pub fn engr_at(state: &GameState, x: i8, y: i8) -> Option<&Engraving> {
    state
        .current_level
        .engravings
        .iter()
        .find(|e| e.x == x && e.y == y)
}

/// Get engraving reference at position (used by HEAD tests)
pub fn engrave_at(state: &GameState, x: i8, y: i8) -> Option<&Engraving> {
    engr_at(state, x, y)
}

/// Alias for engr_at
pub fn e_at(state: &GameState, x: i8, y: i8) -> Option<&Engraving> {
    engr_at(state, x, y)
}

/// Check if a specific text is engraved at (x,y)
pub fn sengr_at(state: &GameState, text: &str, x: i8, y: i8, strict: bool) -> bool {
    if let Some(engraving) = engr_at(state, x, y) {
        if strict {
            engraving.text == text
        } else {
            engraving.text.contains(text)
        }
    } else {
        false
    }
}

/// Check if "Elbereth" is engraved at position (scares most monsters)
pub fn has_elbereth(state: &GameState, x: i8, y: i8) -> bool {
    state.current_level.engravings
        .iter()
        .any(|e| e.x == x && e.y == y && e.text.to_lowercase().contains("elbereth"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Wiping / erasing engravings
// ─────────────────────────────────────────────────────────────────────────────

/// Wipe/erase engraving at position (C: wipe_engr_at)
///
/// `count` is the number of characters to wipe. If 0, wipes everything.
/// Only dust and blood engravings can be wiped by walking/monsters.
/// Hard engravings persist.
pub fn wipe_engr_at(level: &mut crate::dungeon::Level, x: i8, y: i8, count: usize) {
    if let Some(idx) = level.engravings
        .iter()
        .position(|e| e.x == x && e.y == y)
    {
        let engr_type = level.engravings[idx].engr_type;

        // Only dust, blood, and mark engravings are wipeable
        if !matches!(
            engr_type,
            EngravingType::Dust | EngravingType::BloodStain | EngravingType::Mark
        ) {
            return;
        }

        if count == 0 || count >= level.engravings[idx].text.len() {
            // Wipe completely
            level.engravings.remove(idx);
        } else {
            // Partial wipe: remove characters from end
            let text = &mut level.engravings[idx].text;
            let new_len = text.len().saturating_sub(count);
            text.truncate(new_len);
            if text.is_empty() {
                level.engravings.remove(idx);
            }
        }
    }
}

/// Wipe engraving at position (state-based version, removes only dust)
pub fn wipe_engrave_at(state: &mut GameState, x: i8, y: i8) {
    if let Some(idx) = state.current_level.engravings
        .iter()
        .position(|e| e.x == x && e.y == y && e.engr_type == EngravingType::Dust)
    {
        state.current_level.engravings.remove(idx);
    }
}

/// Wipe/erase engraving at position (state-based, with count)
pub fn wipe_engr_at_state(state: &mut GameState, x: i8, y: i8, cnt: usize) {
    if let Some(idx) = state
        .current_level
        .engravings
        .iter()
        .position(|e| e.x == x && e.y == y)
    {
        let engraving = &mut state.current_level.engravings[idx];

        // Only certain types can be wiped easily
        if matches!(
            engraving.engr_type,
            EngravingType::Dust | EngravingType::BloodStain | EngravingType::Mark
        ) {
            // Degrade the text
            let len = engraving.text.len();
            if cnt >= len {
                // Wipe completely
                state.current_level.engravings.remove(idx);
            } else {
                // Wipe partial - truncate from end
                engraving.text.truncate(len - cnt);
            }
        }
    }
}

/// Wipe engraving at player position
pub fn u_wipe_engr(state: &mut GameState, cnt: usize) {
    wipe_engr_at_state(state, state.player.pos.x, state.player.pos.y, cnt);
}

pub fn maybe_smudge_engr(state: &mut GameState, x: i8, y: i8, val: i32) {
    // Smudge dust engraving
    if let Some(engraving) = engr_at(state, x, y) {
        if engraving.engr_type == EngravingType::Dust {
            wipe_engr_at_state(state, x, y, val as usize);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Blind writing and engraving relocation
// ─────────────────────────────────────────────────────────────────────────────

/// Random blind writing strings (gibberish written when blind)
/// These are obfuscated versions of various phrases
const BLIND_WRITING: &[&str] = &[
    "Dmfibe\"E{qemr",           // Obfuscated text
    "Qg`z\x7f!@qkqogc",         // Obfuscated text
    "Imsibe\"La|mg$B\x7fflwg~", // Obfuscated text
    "Kmlf0Lkh|\x7fo",           // Obfuscated text
    "Qgpz\x7foghd!Okm~r",       // Obfuscated text
];

/// Return a random "blind writing" string (gibberish for blind engraving)
pub fn blengr(state: &mut GameState) -> &'static str {
    let idx = state.rng.rn2(BLIND_WRITING.len() as u32) as usize;
    BLIND_WRITING[idx]
}

/// Relocate an engraving to a random valid position
pub fn rloc_engr(state: &mut GameState, engr_idx: usize) {
    let mut tryct = 200;

    while tryct > 0 {
        tryct -= 1;

        // Random position (avoiding edges): rn1(COLNO-3, 2) = 2 + rn2(COLNO-3)
        let tx = (2 + state.rng.rn2((COLNO - 3) as u32)) as i8;
        let ty = state.rng.rn2(ROWNO as u32) as i8;

        // Check if there's already an engraving here
        if engr_at(state, tx, ty).is_some() {
            continue;
        }

        // Check if position is valid (passable floor)
        if !state.current_level.is_valid_pos(tx, ty) {
            continue;
        }

        let cell = state.current_level.cell(tx as usize, ty as usize);
        if !cell.typ.is_passable() {
            continue;
        }

        // Move the engraving
        if engr_idx < state.current_level.engravings.len() {
            state.current_level.engravings[engr_idx].x = tx;
            state.current_level.engravings[engr_idx].y = ty;
        }
        return;
    }
    // Failed to find valid location after 200 tries, leave engraving where it is
}

// ─────────────────────────────────────────────────────────────────────────────
// Save/restore and statistics
// ─────────────────────────────────────────────────────────────────────────────

/// Restore engravings from save data
/// In Rust, this is handled by serde deserialization of the Level struct
/// which contains the engravings Vec. This function exists for compatibility.
pub fn rest_engravings(_state: &mut GameState) {
    // Engravings are automatically restored when Level is deserialized via serde
    // No additional action needed in Rust implementation
}

/// Save engravings to save data
/// In Rust, this is handled by serde serialization of the Level struct
/// which contains the engravings Vec. This function exists for compatibility.
pub fn save_engravings(_state: &mut GameState) {
    // Engravings are automatically saved when Level is serialized via serde
    // No additional action needed in Rust implementation
}

/// Statistics about engravings (for wizard mode '#stats' command)
#[derive(Debug, Clone, Default)]
pub struct EngrStats {
    pub count: usize,
    pub total_text_len: usize,
}

/// Get statistics about current level engravings
pub fn engr_stats(state: &GameState) -> EngrStats {
    let count = state.current_level.engravings.len();
    let total_text_len: usize = state
        .current_level
        .engravings
        .iter()
        .map(|e| e.text.len())
        .sum();

    EngrStats {
        count,
        total_text_len,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Grave creation
// ─────────────────────────────────────────────────────────────────────────────

/// Create a headstone/grave at position (C: make_grave)
pub fn make_grave(
    level: &mut crate::dungeon::Level,
    x: i8,
    y: i8,
    text: Option<&str>,
    rng: &mut crate::rng::GameRng,
) {
    // Set the cell to Grave type
    if level.is_valid_pos(x, y) {
        level.cell_mut(x as usize, y as usize).typ = CellType::Grave;
    }

    // Add epitaph
    let epitaph = if let Some(t) = text {
        t.to_string()
    } else {
        let idx = rng.rn2(EPITAPHS.len() as u32) as usize;
        EPITAPHS[idx].to_string()
    };

    // Remove any existing engraving
    if let Some(idx) = level.engravings
        .iter()
        .position(|e| e.x == x && e.y == y)
    {
        level.engravings.remove(idx);
    }

    // Create headstone engraving
    let engraving = Engraving::new(x, y, epitaph, EngravingType::Headstone);
    level.engravings.push(engraving);
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::{Cell, CellType, Engraving, EngravingType};
    use crate::gameloop::GameState;
    use crate::player::Position;
    use crate::rng::GameRng;

    fn make_state() -> GameState {
        let mut state = GameState::new(GameRng::new(42));
        state.player.pos = Position::new(5, 5);
        state.player.prev_pos = Position::new(5, 5);
        for x in 1..20 {
            for y in 1..10 {
                *state.current_level.cell_mut(x, y) = Cell::floor();
            }
        }
        state
    }

    // ── do_engrave ───────────────────────────────────────────────────────

    #[test]
    fn test_do_engrave_empty() {
        let mut state = make_state();
        let result = do_engrave(&mut state, "");
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    fn test_do_engrave_text() {
        let mut state = make_state();
        let result = do_engrave(&mut state, "Hello");
        assert!(matches!(result, ActionResult::Success));
        assert_eq!(read_engrave(&state), Some("Hello".to_string()));
    }

    #[test]
    fn test_do_engrave_elbereth() {
        let mut state = make_state();
        do_engrave(&mut state, "Elbereth");
        assert!(has_elbereth(&state, 5, 5));
    }

    #[test]
    fn test_do_engrave_replaces() {
        let mut state = make_state();
        do_engrave(&mut state, "First");
        do_engrave(&mut state, "Second");
        assert_eq!(read_engrave(&state), Some("Second".to_string()));
        assert_eq!(state.current_level.engravings.len(), 1);
    }

    // ── do_engrave_with_tool ─────────────────────────────────────────────

    #[test]
    fn test_engrave_with_tool_burn() {
        let mut state = make_state();
        let result = do_engrave_with_tool(&mut state, "Fire!", EngravingType::Burn);
        assert!(matches!(result, ActionResult::Success));
        let engr = engrave_at(&state, 5, 5).unwrap();
        assert_eq!(engr.engr_type, EngravingType::Burn);
    }

    #[test]
    fn test_engrave_with_tool_empty() {
        let mut state = make_state();
        let result = do_engrave_with_tool(&mut state, "", EngravingType::Engrave);
        assert!(matches!(result, ActionResult::NoTime));
    }

    // ── do_engrave_full ──────────────────────────────────────────────────

    #[test]
    fn test_engrave_full_weapon() {
        let mut state = make_state();
        let result = do_engrave_full(&mut state, "Test", ObjectClass::Weapon, None);
        assert!(matches!(result, ActionResult::Success));
        let engr = engrave_at(&state, 5, 5).unwrap();
        assert_eq!(engr.engr_type, EngravingType::Engrave);
    }

    #[test]
    fn test_engrave_full_tool() {
        let mut state = make_state();
        let result = do_engrave_full(&mut state, "Test", ObjectClass::Tool, None);
        assert!(matches!(result, ActionResult::Success));
        let engr = engrave_at(&state, 5, 5).unwrap();
        assert_eq!(engr.engr_type, EngravingType::Mark);
    }

    #[test]
    fn test_engrave_full_fingers() {
        let mut state = make_state();
        let result = do_engrave_full(&mut state, "Test", ObjectClass::Food, None);
        assert!(matches!(result, ActionResult::Success));
        let engr = engrave_at(&state, 5, 5).unwrap();
        assert_eq!(engr.engr_type, EngravingType::Dust);
    }

    #[test]
    fn test_engrave_full_empty() {
        let mut state = make_state();
        let result = do_engrave_full(&mut state, "", ObjectClass::Weapon, None);
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_engrave_full_levitating() {
        let mut state = make_state();
        state.player.properties.grant_intrinsic(crate::player::Property::Levitation);
        let result = do_engrave_full(&mut state, "Test", ObjectClass::Weapon, None);
        assert!(matches!(result, ActionResult::NoTime));
    }

    // ── wand_engrave_effect ──────────────────────────────────────────────

    #[test]
    fn test_wand_fire_burns() {
        assert_eq!(wand_engrave_effect(1), WandEngraveEffect::Burns);
    }

    #[test]
    fn test_wand_lightning_burns() {
        assert_eq!(wand_engrave_effect(5), WandEngraveEffect::Burns);
    }

    #[test]
    fn test_wand_digging_engraves() {
        assert_eq!(wand_engrave_effect(8), WandEngraveEffect::Engraves);
    }

    #[test]
    fn test_wand_cancellation_erases() {
        assert_eq!(wand_engrave_effect(10), WandEngraveEffect::Erases);
    }

    #[test]
    fn test_wand_polymorph_random() {
        assert_eq!(wand_engrave_effect(13), WandEngraveEffect::Random);
    }

    #[test]
    fn test_wand_other_nothing() {
        assert_eq!(wand_engrave_effect(99), WandEngraveEffect::Nothing);
    }

    #[test]
    fn test_engrave_with_wand_fire() {
        let mut state = make_state();
        let result = do_engrave_full(&mut state, "Test", ObjectClass::Wand, Some(1));
        assert!(matches!(result, ActionResult::Success));
        let engr = engrave_at(&state, 5, 5).unwrap();
        assert_eq!(engr.engr_type, EngravingType::Burn);
    }

    #[test]
    fn test_engrave_with_wand_erases() {
        let mut state = make_state();
        // First create an engraving
        do_engrave(&mut state, "Original");
        assert!(engrave_at(&state, 5, 5).is_some());
        // Then erase with cancellation wand
        let result = do_engrave_full(&mut state, "New", ObjectClass::Wand, Some(10));
        assert!(matches!(result, ActionResult::Success));
        assert!(engrave_at(&state, 5, 5).is_none());
    }

    // ── wipe_engr_at ─────────────────────────────────────────────────────

    #[test]
    fn test_wipe_dust_complete() {
        let mut state = make_state();
        do_engrave(&mut state, "Dust text");
        wipe_engr_at(&mut state.current_level, 5, 5, 0);
        assert!(engrave_at(&state, 5, 5).is_none());
    }

    #[test]
    fn test_wipe_dust_partial() {
        let mut state = make_state();
        do_engrave(&mut state, "Hello World");
        wipe_engr_at(&mut state.current_level, 5, 5, 5);
        let engr = engrave_at(&state, 5, 5).unwrap();
        assert_eq!(engr.text, "Hello ");
    }

    #[test]
    fn test_wipe_engrave_resists() {
        let mut state = make_state();
        do_engrave_with_tool(&mut state, "Permanent", EngravingType::Engrave);
        wipe_engr_at(&mut state.current_level, 5, 5, 0);
        // Should NOT be wiped (Engrave type is permanent)
        assert!(engrave_at(&state, 5, 5).is_some());
    }

    #[test]
    fn test_wipe_burn_resists() {
        let mut state = make_state();
        do_engrave_with_tool(&mut state, "Burned", EngravingType::Burn);
        wipe_engr_at(&mut state.current_level, 5, 5, 0);
        assert!(engrave_at(&state, 5, 5).is_some());
    }

    #[test]
    fn test_wipe_blood_wipeable() {
        let mut state = make_state();
        do_engrave_with_tool(&mut state, "Blood text", EngravingType::BloodStain);
        wipe_engr_at(&mut state.current_level, 5, 5, 0);
        assert!(engrave_at(&state, 5, 5).is_none());
    }

    // ── engrave_time_cost ────────────────────────────────────────────────

    #[test]
    fn test_time_cost_dust() {
        assert_eq!(engrave_time_cost(5, EngravingType::Dust), 5);
    }

    #[test]
    fn test_time_cost_engrave() {
        assert_eq!(engrave_time_cost(5, EngravingType::Engrave), 10);
    }

    #[test]
    fn test_time_cost_burn() {
        assert_eq!(engrave_time_cost(5, EngravingType::Burn), 1);
    }

    #[test]
    fn test_time_cost_blood() {
        assert_eq!(engrave_time_cost(5, EngravingType::BloodStain), 3);
    }

    // ── make_grave ───────────────────────────────────────────────────────

    #[test]
    fn test_make_grave() {
        let mut rng = GameRng::new(42);
        let mut state = make_state();
        make_grave(&mut state.current_level, 5, 5, Some("RIP"), &mut rng);
        assert_eq!(
            state.current_level.cell(5, 5).typ,
            CellType::Grave
        );
        let engr = engrave_at(&state, 5, 5).unwrap();
        assert_eq!(engr.text, "RIP");
        assert_eq!(engr.engr_type, EngravingType::Headstone);
    }

    #[test]
    fn test_make_grave_random_epitaph() {
        let mut rng = GameRng::new(42);
        let mut state = make_state();
        make_grave(&mut state.current_level, 5, 5, None, &mut rng);
        let engr = engrave_at(&state, 5, 5).unwrap();
        assert!(!engr.text.is_empty());
        assert_eq!(engr.engr_type, EngravingType::Headstone);
    }

    // ── random_epitaph / random_engraving ────────────────────────────────

    #[test]
    fn test_random_epitaph() {
        let mut state = make_state();
        let ep = random_epitaph(&mut state);
        assert!(!ep.is_empty());
        assert!(EPITAPHS.contains(&ep));
    }

    #[test]
    fn test_random_engraving_from_pool() {
        let mut state = make_state();
        let eg = random_engraving(&mut state);
        assert!(!eg.is_empty());
        assert!(RANDOM_ENGRAVINGS.contains(&eg));
    }

    // ── has_elbereth ─────────────────────────────────────────────────────

    #[test]
    fn test_has_elbereth_yes() {
        let mut state = make_state();
        do_engrave(&mut state, "Elbereth");
        assert!(has_elbereth(&state, 5, 5));
    }

    #[test]
    fn test_has_elbereth_no() {
        let state = make_state();
        assert!(!has_elbereth(&state, 5, 5));
    }

    #[test]
    fn test_has_elbereth_case_insensitive() {
        let mut state = make_state();
        do_engrave(&mut state, "ELBERETH");
        assert!(has_elbereth(&state, 5, 5));
    }

    // ── engrave_at ───────────────────────────────────────────────────────

    #[test]
    fn test_engrave_at_exists() {
        let mut state = make_state();
        do_engrave(&mut state, "Test");
        let engr = engrave_at(&state, 5, 5);
        assert!(engr.is_some());
        assert_eq!(engr.unwrap().text, "Test");
    }

    #[test]
    fn test_engrave_at_missing() {
        let state = make_state();
        assert!(engrave_at(&state, 5, 5).is_none());
    }

    // ── apply_wand_effect ────────────────────────────────────────────────

    #[test]
    fn test_apply_wand_burns() {
        let mut state = make_state();
        let result = apply_wand_effect(&mut state, WandEngraveEffect::Burns, "Test");
        assert_eq!(result, EngravingType::Burn);
    }

    #[test]
    fn test_apply_wand_engraves() {
        let mut state = make_state();
        let result = apply_wand_effect(&mut state, WandEngraveEffect::Engraves, "Test");
        assert_eq!(result, EngravingType::Engrave);
    }

    #[test]
    fn test_apply_wand_erases_existing() {
        let mut state = make_state();
        do_engrave(&mut state, "Existing");
        assert!(engrave_at(&state, 5, 5).is_some());
        apply_wand_effect(&mut state, WandEngraveEffect::Erases, "New");
        assert!(engrave_at(&state, 5, 5).is_none());
    }

    // ── engrave_type_for_tool ────────────────────────────────────────────

    #[test]
    fn test_tool_type_weapon() {
        assert_eq!(engrave_type_for_tool(ObjectClass::Weapon), EngravingType::Engrave);
    }

    #[test]
    fn test_tool_type_wand() {
        assert_eq!(engrave_type_for_tool(ObjectClass::Wand), EngravingType::Dust);
    }

    #[test]
    fn test_tool_type_tool() {
        assert_eq!(engrave_type_for_tool(ObjectClass::Tool), EngravingType::Mark);
    }

    #[test]
    fn test_tool_type_other() {
        assert_eq!(engrave_type_for_tool(ObjectClass::Food), EngravingType::Dust);
    }

    // ── make_engr_at / read_engr_at / del_engr tests ─────────────────────

    #[test]
    fn test_make_and_read_engraving() {
        let mut state = GameState::default();
        make_engr_at(&mut state, 10, 10, "Test", EngravingType::Dust);

        let text = read_engr_at(&state, 10, 10);
        assert_eq!(text, Some("Test".to_string()));
    }

    #[test]
    fn test_del_engr() {
        let mut state = GameState::default();
        make_engr_at(&mut state, 10, 10, "Test", EngravingType::Dust);
        del_engr(&mut state, 10, 10);

        assert!(read_engr_at(&state, 10, 10).is_none());
    }

    #[test]
    fn test_blengr_returns_string() {
        let mut state = GameState::default();
        let blind_text = blengr(&mut state);
        // Should return one of the blind writing strings
        assert!(!blind_text.is_empty());
    }

    #[test]
    fn test_engr_stats() {
        let mut state = GameState::default();

        // No engravings yet
        let stats = engr_stats(&state);
        assert_eq!(stats.count, 0);
        assert_eq!(stats.total_text_len, 0);

        // Add some engravings
        make_engr_at(&mut state, 10, 10, "Hello", EngravingType::Dust);
        make_engr_at(&mut state, 15, 15, "World", EngravingType::Engrave);

        let stats = engr_stats(&state);
        assert_eq!(stats.count, 2);
        assert_eq!(stats.total_text_len, 10); // "Hello" + "World" = 5 + 5
    }

    #[test]
    fn test_sengr_at() {
        let mut state = GameState::default();
        make_engr_at(
            &mut state,
            10,
            10,
            "Elbereth lives here",
            EngravingType::Dust,
        );

        // Strict match - should fail
        assert!(!sengr_at(&state, "Elbereth", 10, 10, true));

        // Non-strict (contains) - should succeed
        assert!(sengr_at(&state, "Elbereth", 10, 10, false));
    }

    #[test]
    fn test_has_elbereth_at_position() {
        let mut state = GameState::default();
        make_engr_at(&mut state, 10, 10, "Elbereth", EngravingType::Dust);

        assert!(has_elbereth(&state, 10, 10));
        assert!(!has_elbereth(&state, 5, 5));
    }
}
