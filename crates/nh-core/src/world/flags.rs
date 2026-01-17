//! Game flags

use serde::{Deserialize, Serialize};

/// Global game flags
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Flags {
    // Game mode
    pub wizard: bool,
    pub explore: bool,
    pub debug: bool,

    // Game state
    pub started: bool,
    pub panic: bool,

    // Display options
    pub show_room: bool,
    pub show_corridor: bool,
    pub show_objects: bool,
    pub autopickup: bool,
    pub verbose: bool,
    pub silent: bool,

    // Gameplay options
    pub safe_pet: bool,
    pub safe_peaceful: bool,
    pub confirm: bool,
    pub pickup_thrown: bool,
    pub pushweapon: bool,

    // Number pad
    pub num_pad: bool,

    // Run modes
    pub run: i8, // 0 = not running, >0 = running mode

    // Sound
    pub soundlib: bool,

    // Travel
    pub travel_debug: bool,

    // End game
    pub ascended: bool,
    pub made_amulet: bool,
    pub invoked: bool,
}
