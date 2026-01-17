//! Game context (current action state)

use serde::{Deserialize, Serialize};

/// Current game context/state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Context {
    // Movement
    pub move_made: bool,
    pub mv: bool,         // running
    pub run: i8,          // run mode
    pub travel: bool,
    pub travel_1: bool,   // first travel step
    pub forcefight: bool, // attacking a position

    // Action state
    pub nopick: bool,     // don't autopickup
    pub botl: bool,       // update status line
    pub botlx: bool,      // update extended status
    pub door_opened: bool,

    // Monster state
    pub monsters_moving: bool,

    // Menu state
    pub menu_active: bool,
    pub using_gui_prompts: bool,

    // Vision
    pub vision_full_recalc: bool,

    // Save/restore
    pub restoring: bool,
    pub saving: bool,
}
