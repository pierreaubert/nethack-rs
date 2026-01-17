//! Bevy components for NetHack-rs

use bevy::prelude::*;

/// Position in map coordinates (0-79, 0-20)
#[derive(Component, Clone, Copy, Debug)]
pub struct MapPosition {
    pub x: i8,
    pub y: i8,
}

impl MapPosition {
    /// Convert map position to world position
    /// X maps to X, Y maps to Z (Y is up in Bevy)
    pub fn to_world(&self) -> Vec3 {
        Vec3::new(self.x as f32, 0.0, self.y as f32)
    }
}

/// Marker for tile entities
#[derive(Component)]
pub struct TileMarker;

/// Marker for the player entity
#[derive(Component)]
pub struct PlayerMarker;

/// Marker for monster entities
#[derive(Component)]
pub struct MonsterMarker {
    pub monster_id: nh_core::monster::MonsterId,
}

/// Marker for billboard sprites that should face camera
#[derive(Component)]
pub struct Billboard;

/// Current camera mode
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum CameraMode {
    #[default]
    TopDown,
    Isometric,
    ThirdPerson,
    FirstPerson,
}

/// Marker for door entities with their position
#[derive(Component)]
pub struct DoorMarker {
    pub x: i8,
    pub y: i8,
    pub is_open: bool,
}

/// Animation component for doors
#[derive(Component)]
pub struct DoorAnimation {
    pub timer: Timer,
    pub start_height: f32,
    pub target_height: f32,
}
