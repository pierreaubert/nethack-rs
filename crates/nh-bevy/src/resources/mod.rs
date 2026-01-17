//! Bevy resources for NetHack-rs

use bevy::prelude::*;
use nh_core::monster::MonsterId;
use std::collections::HashMap;

/// Wrapper around nh_core::GameState for Bevy
#[derive(Resource)]
pub struct GameStateResource(pub nh_core::GameState);

/// Tracks previous HP values for combat detection
#[derive(Resource, Default)]
pub struct CombatTracker {
    /// Player HP from last frame
    pub prev_player_hp: i32,
    /// Monster HP from last frame (keyed by monster id)
    pub prev_monster_hp: HashMap<MonsterId, i32>,
    /// Previous inventory item count (for pickup detection)
    pub prev_inventory_count: usize,
    /// Whether this is initialized
    pub initialized: bool,
}
