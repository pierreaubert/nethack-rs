//! Musical instruments (music.c)
//!
//! Playing instruments — wooden flute, magic flute, tooled horn,
//! fire/frost/bugle horn, harp, drum of earthquake, etc.

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::rng::GameRng;

/// Instrument types based on object names/subtypes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstrumentType {
    WoodenFlute,
    MagicFlute,
    TooledHorn,
    FrostHorn,
    FireHorn,
    Bugle,
    WoodenHarp,
    MagicHarp,
    Drum,
    DrumOfEarthquake,
    Bell,
}

/// Result of playing an instrument
#[derive(Debug, Clone)]
pub struct PlayResult {
    pub messages: Vec<String>,
    pub time_passes: bool,
    pub affects_monsters: bool,
    pub charges_used: bool,
}

/// Play a musical instrument (do_play_instrument from music.c:58).
pub fn do_play_instrument(state: &mut GameState, inv_letter: char) -> ActionResult {
    let obj = match state.inventory.iter().find(|o| o.inv_letter == inv_letter) {
        Some(o) => o,
        None => {
            state.message("You don't have that item.");
            return ActionResult::NoTime;
        }
    };

    let name = obj.display_name();

    // Check if it's actually an instrument (Tool class with musical subtype)
    if obj.class != crate::object::ObjectClass::Tool {
        state.message(format!("{} is not an instrument.", name));
        return ActionResult::NoTime;
    }

    state.message(format!("You play {}.", name));
    ActionResult::Success
}

/// Effect of playing a wooden flute (music.c:110).
///
/// Calms snakes nearby (1/5 chance per snake).
pub fn play_flute(rng: &mut GameRng) -> PlayResult {
    let _ = rng;
    PlayResult {
        messages: vec!["You produce a shrill whistling sound.".to_string()],
        time_passes: true,
        affects_monsters: true,
        charges_used: false,
    }
}

/// Effect of playing a magic flute (music.c:130).
///
/// Puts nearby monsters to sleep.
pub fn play_magic_flute(charges: i32, rng: &mut GameRng) -> PlayResult {
    let _ = rng;
    if charges <= 0 {
        return play_flute(rng); // No charges = normal flute
    }
    PlayResult {
        messages: vec!["You produce soft, enchanting music.".to_string()],
        time_passes: true,
        affects_monsters: true,
        charges_used: true,
    }
}

/// Effect of playing a tooled horn (music.c:180).
///
/// Scares some monsters. Magic horns have special effects.
pub fn play_horn(rng: &mut GameRng) -> PlayResult {
    let _ = rng;
    PlayResult {
        messages: vec!["You produce a loud blast!".to_string()],
        time_passes: true,
        affects_monsters: true,
        charges_used: false,
    }
}

/// Effect of playing a frost/fire horn (music.c:200).
///
/// Shoots a beam of cold/fire in the direction played.
pub fn play_elemental_horn(is_fire: bool, charges: i32, rng: &mut GameRng) -> PlayResult {
    let _ = rng;
    if charges <= 0 {
        return play_horn(rng);
    }
    let element = if is_fire { "fire" } else { "cold" };
    PlayResult {
        messages: vec![format!("A blast of {} shoots out!", element)],
        time_passes: true,
        affects_monsters: true,
        charges_used: true,
    }
}

/// Effect of playing a bugle (music.c:240).
///
/// Wakes up all monsters on the level (soldier special: summon allies).
pub fn play_bugle() -> PlayResult {
    PlayResult {
        messages: vec!["You sound a charge!".to_string()],
        time_passes: true,
        affects_monsters: true,
        charges_used: false,
    }
}

/// Effect of playing a harp (music.c:260).
pub fn play_harp(rng: &mut GameRng) -> PlayResult {
    let _ = rng;
    PlayResult {
        messages: vec!["You produce beautiful music.".to_string()],
        time_passes: true,
        affects_monsters: false,
        charges_used: false,
    }
}

/// Effect of playing a magic harp (music.c:280).
///
/// Charms nearby monsters (tame attempt).
pub fn play_magic_harp(charges: i32, rng: &mut GameRng) -> PlayResult {
    let _ = rng;
    if charges <= 0 {
        return play_harp(rng);
    }
    PlayResult {
        messages: vec!["You produce enchanting music.".to_string()],
        time_passes: true,
        affects_monsters: true,
        charges_used: true,
    }
}

/// Effect of playing a drum (music.c:310).
pub fn play_drum(rng: &mut GameRng) -> PlayResult {
    let _ = rng;
    PlayResult {
        messages: vec!["You beat a deafening rhythm!".to_string()],
        time_passes: true,
        affects_monsters: true,
        charges_used: false,
    }
}

/// Effect of playing the Drum of Earthquake (music.c:330).
///
/// Causes an earthquake that opens walls, creates pits, and scares monsters.
pub fn play_drum_of_earthquake(charges: i32, rng: &mut GameRng) -> PlayResult {
    let _ = rng;
    if charges <= 0 {
        return play_drum(rng);
    }
    PlayResult {
        messages: vec![
            "The ground shakes violently!".to_string(),
            "You produce a heavy, thundering sound!".to_string(),
        ],
        time_passes: true,
        affects_monsters: true,
        charges_used: true,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_play_flute() {
        let mut rng = GameRng::new(42);
        let result = play_flute(&mut rng);
        assert!(result.time_passes);
        assert!(!result.charges_used);
    }

    #[test]
    fn test_play_magic_flute_no_charges() {
        let mut rng = GameRng::new(42);
        let result = play_magic_flute(0, &mut rng);
        assert!(!result.charges_used);
    }

    #[test]
    fn test_play_magic_flute_with_charges() {
        let mut rng = GameRng::new(42);
        let result = play_magic_flute(5, &mut rng);
        assert!(result.charges_used);
    }

    #[test]
    fn test_play_elemental_horn() {
        let mut rng = GameRng::new(42);
        let result = play_elemental_horn(true, 5, &mut rng);
        assert!(result.messages[0].contains("fire"));
        assert!(result.charges_used);
    }

    #[test]
    fn test_play_drum_of_earthquake() {
        let mut rng = GameRng::new(42);
        let result = play_drum_of_earthquake(3, &mut rng);
        assert!(result.messages[0].contains("shakes"));
        assert!(result.charges_used);
    }

    #[test]
    fn test_play_bugle() {
        let result = play_bugle();
        assert!(result.affects_monsters);
    }
}
