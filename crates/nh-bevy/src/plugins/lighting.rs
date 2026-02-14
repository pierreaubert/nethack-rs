//! Dynamic lighting system
//!
//! Provides:
//! - Ambient lighting for lit rooms
//! - Player light source for dark areas
//! - Special lighting effects (lava glow, fountain shimmer)

use bevy::prelude::*;

use crate::components::{MapPosition, PlayerMarker};
use crate::plugins::game::AppState;
use crate::resources::GameStateResource;

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightingSettings>()
            .add_systems(Startup, setup_lighting)
            .add_systems(
                Update,
                (update_player_light, update_ambient_lighting).run_if(in_state(AppState::Playing)),
            );
    }
}

/// Lighting configuration
#[derive(Resource)]
pub struct LightingSettings {
    /// Player light radius in tiles
    pub player_light_radius: f32,
    /// Player light intensity in dark areas
    pub player_light_intensity: f32,
    /// Player light color
    pub player_light_color: Color,
    /// Ambient light in lit rooms
    pub lit_room_ambient: f32,
    /// Ambient light in dark corridors
    pub dark_ambient: f32,
    /// Lava glow intensity
    pub lava_glow_intensity: f32,
    /// Fountain shimmer intensity
    pub fountain_glow_intensity: f32,
}

impl Default for LightingSettings {
    fn default() -> Self {
        Self {
            player_light_radius: 5.0,
            player_light_intensity: 2000.0,
            player_light_color: Color::srgb(1.0, 0.9, 0.7), // Warm torch light
            lit_room_ambient: 800.0,
            dark_ambient: 200.0,
            lava_glow_intensity: 1500.0,
            fountain_glow_intensity: 500.0,
        }
    }
}

/// Marker for the player's carried light source
#[derive(Component)]
pub struct PlayerLight;

/// Marker for lava glow lights
#[derive(Component)]
pub struct LavaLight {
    pub x: i8,
    pub y: i8,
}

/// Marker for fountain lights
#[derive(Component)]
pub struct FountainLight {
    pub x: i8,
    pub y: i8,
}

fn setup_lighting(mut commands: Commands, settings: Res<LightingSettings>) {
    // Create a point light that follows the player
    commands.spawn((
        PlayerLight,
        PointLight {
            color: settings.player_light_color,
            intensity: settings.player_light_intensity,
            radius: settings.player_light_radius,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, 0.0),
    ));
}

/// Update player light position to follow player
fn update_player_light(
    player_query: Query<&Transform, With<PlayerMarker>>,
    mut light_query: Query<&mut Transform, (With<PlayerLight>, Without<PlayerMarker>)>,
    game_state: Res<GameStateResource>,
    settings: Res<LightingSettings>,
    mut ambient: ResMut<AmbientLight>,
) {
    // Update player light position
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok(mut light_transform) = light_query.get_single_mut() {
            light_transform.translation = player_transform.translation + Vec3::Y * 1.5; // Light above player
        }
    }

    // Adjust ambient based on whether player is in a lit room
    let state = &game_state.0;
    let player_x = state.player.pos.x as usize;
    let player_y = state.player.pos.y as usize;
    let cell = state.current_level.cell(player_x, player_y);

    let target_ambient = if cell.lit {
        settings.lit_room_ambient
    } else {
        settings.dark_ambient
    };

    // Smoothly interpolate ambient lighting
    let current = ambient.brightness;
    ambient.brightness = current + (target_ambient - current) * 0.1;
}

/// Update ambient lighting and spawn special light sources
fn update_ambient_lighting(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    settings: Res<LightingSettings>,
    lava_lights: Query<Entity, With<LavaLight>>,
    fountain_lights: Query<Entity, With<FountainLight>>,
    mut initialized: Local<bool>,
) {
    // Only initialize special lights once
    if *initialized {
        return;
    }

    let level = &game_state.0.current_level;

    // Spawn lights for lava and fountains
    for y in 0..nh_core::ROWNO {
        for x in 0..nh_core::COLNO {
            let cell = level.cell(x, y);
            let pos = MapPosition {
                x: x as i8,
                y: y as i8,
            };
            let world_pos = pos.to_world();

            match cell.typ {
                nh_core::dungeon::CellType::Lava => {
                    commands.spawn((
                        LavaLight {
                            x: x as i8,
                            y: y as i8,
                        },
                        PointLight {
                            color: Color::srgb(1.0, 0.4, 0.1),
                            intensity: settings.lava_glow_intensity,
                            radius: 3.0,
                            shadows_enabled: false,
                            ..default()
                        },
                        Transform::from_translation(world_pos + Vec3::Y * 0.5),
                    ));
                }
                nh_core::dungeon::CellType::Fountain => {
                    commands.spawn((
                        FountainLight {
                            x: x as i8,
                            y: y as i8,
                        },
                        PointLight {
                            color: Color::srgb(0.4, 0.6, 1.0),
                            intensity: settings.fountain_glow_intensity,
                            radius: 2.0,
                            shadows_enabled: false,
                            ..default()
                        },
                        Transform::from_translation(world_pos + Vec3::Y * 0.5),
                    ));
                }
                _ => {}
            }
        }
    }

    // Despawn old lights if level changes (simplified - in practice would track level changes)
    for entity in lava_lights.iter() {
        commands.entity(entity).despawn();
    }
    for entity in fountain_lights.iter() {
        commands.entity(entity).despawn();
    }

    *initialized = true;
}
