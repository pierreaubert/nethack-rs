//! Bevy resources for NetHack-rs

use bevy::prelude::*;
use nh_core::monster::MonsterId;
use nh_core::player::{AlignmentType, Gender, Race, Role};
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

/// Character creation wizard step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharacterCreationStep {
    #[default]
    EnterName,
    AskRandom,
    SelectRole,
    SelectRace,
    SelectGender,
    SelectAlignment,
    Done,
}

/// Resource tracking character creation state
#[derive(Resource)]
pub struct CharacterCreationState {
    pub step: CharacterCreationStep,
    pub name: String,
    pub role: Option<Role>,
    pub race: Option<Race>,
    pub gender: Option<Gender>,
    pub alignment: Option<AlignmentType>,
    pub cursor: usize,
}

impl Default for CharacterCreationState {
    fn default() -> Self {
        Self {
            step: CharacterCreationStep::EnterName,
            name: String::new(),
            role: None,
            race: None,
            gender: None,
            alignment: None,
            cursor: 0,
        }
    }
}

impl CharacterCreationState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Info about how the game ended (death or victory)
#[derive(Resource, Default)]
pub struct GameOverInfo {
    pub cause_of_death: Option<String>,
    pub is_victory: bool,
}
