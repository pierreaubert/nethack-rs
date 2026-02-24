//! Particle and visual effects
//!
//! Provides:
//! - Water ripple effects
//! - Lava bubble effects
//! - Fountain spray effects
//! - Spell casting particles
//! - Environmental ambiance

use bevy::prelude::*;

use crate::components::MapPosition;
use crate::plugins::game::AppState;
use crate::resources::GameStateResource;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EffectsSettings>()
            .add_systems(
                Startup,
                spawn_environmental_effects.after(super::map::spawn_map),
            )
            .add_systems(
                Update,
                (
                    animate_water_ripples,
                    animate_lava_bubbles,
                    animate_fountain_spray,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// Settings for visual effects
#[derive(Resource)]
pub struct EffectsSettings {
    /// Enable water ripple effects
    pub water_ripples: bool,
    /// Enable lava bubble effects
    pub lava_bubbles: bool,
    /// Enable fountain spray effects
    pub fountain_spray: bool,
    /// Particle spawn rate (per second)
    pub particle_rate: f32,
}

impl Default for EffectsSettings {
    fn default() -> Self {
        Self {
            water_ripples: true,
            lava_bubbles: true,
            fountain_spray: true,
            particle_rate: 2.0,
        }
    }
}

/// Marker for water ripple effects
#[derive(Component)]
pub struct WaterRipple {
    pub timer: Timer,
    pub base_y: f32,
}

/// Marker for lava bubble effects
#[derive(Component)]
pub struct LavaBubble {
    pub timer: Timer,
    pub start_pos: Vec3,
    pub offset_x: f32,
    pub offset_z: f32,
}

/// Marker for fountain spray particles
#[derive(Component)]
pub struct FountainSpray {
    pub timer: Timer,
    pub start_pos: Vec3,
    pub velocity: Vec3,
}

/// Marker for effect spawner positions
#[derive(Component)]
pub struct EffectSpawner {
    pub effect_type: EffectType,
    pub spawn_timer: Timer,
    pub x: i8,
    pub y: i8,
}

#[derive(Clone, Copy)]
pub enum EffectType {
    WaterRipple,
    LavaBubble,
    FountainSpray,
}

fn spawn_environmental_effects(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    settings: Res<EffectsSettings>,
) {
    let level = &game_state.0.current_level;

    for y in 0..nh_core::ROWNO {
        for x in 0..nh_core::COLNO {
            let cell = level.cell(x, y);

            let effect_type = match cell.typ {
                nh_core::dungeon::CellType::Pool
                | nh_core::dungeon::CellType::Moat
                | nh_core::dungeon::CellType::Water
                    if settings.water_ripples =>
                {
                    Some(EffectType::WaterRipple)
                }
                nh_core::dungeon::CellType::Lava if settings.lava_bubbles => {
                    Some(EffectType::LavaBubble)
                }
                nh_core::dungeon::CellType::Fountain if settings.fountain_spray => {
                    Some(EffectType::FountainSpray)
                }
                _ => None,
            };

            if let Some(etype) = effect_type {
                commands.spawn(EffectSpawner {
                    effect_type: etype,
                    spawn_timer: Timer::from_seconds(
                        1.0 / settings.particle_rate + fastrand::f32() * 0.5,
                        TimerMode::Repeating,
                    ),
                    x: x as i8,
                    y: y as i8,
                });
            }
        }
    }
}

fn animate_water_ripples(
    mut commands: Commands,
    time: Res<Time>,
    mut spawners: Query<&mut EffectSpawner>,
    mut ripples: Query<(Entity, &mut Transform, &mut WaterRipple, &mut TextColor)>,
) {
    // Spawn new ripples
    for mut spawner in spawners.iter_mut() {
        if !matches!(spawner.effect_type, EffectType::WaterRipple) {
            continue;
        }

        spawner.spawn_timer.tick(time.delta());
        if spawner.spawn_timer.just_finished() {
            let pos = MapPosition {
                x: spawner.x,
                y: spawner.y,
            };
            let world_pos = pos.to_world();
            let offset_x = (fastrand::f32() - 0.5) * 0.8;
            let offset_z = (fastrand::f32() - 0.5) * 0.8;

            commands.spawn((
                WaterRipple {
                    timer: Timer::from_seconds(2.0, TimerMode::Once),
                    base_y: world_pos.y - 0.25,
                },
                Text2d::new("~"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgba(0.3, 0.5, 0.8, 0.6)),
                Transform::from_translation(world_pos + Vec3::new(offset_x, -0.25, offset_z))
                    .with_scale(Vec3::splat(0.01)),
            ));
        }
    }

    // Animate existing ripples
    for (entity, mut transform, mut ripple, mut color) in ripples.iter_mut() {
        ripple.timer.tick(time.delta());

        let t = ripple.timer.fraction();

        // Expand and fade
        let scale = 0.01 + t * 0.02;
        transform.scale = Vec3::splat(scale);

        // Fade out
        let alpha = 0.6 * (1.0 - t);
        color.0 = Color::srgba(0.3, 0.5, 0.8, alpha);

        if ripple.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn animate_lava_bubbles(
    mut commands: Commands,
    time: Res<Time>,
    mut spawners: Query<&mut EffectSpawner>,
    mut bubbles: Query<(Entity, &mut Transform, &mut LavaBubble, &mut TextColor)>,
) {
    // Spawn new bubbles
    for mut spawner in spawners.iter_mut() {
        if !matches!(spawner.effect_type, EffectType::LavaBubble) {
            continue;
        }

        spawner.spawn_timer.tick(time.delta());
        if spawner.spawn_timer.just_finished() {
            let pos = MapPosition {
                x: spawner.x,
                y: spawner.y,
            };
            let world_pos = pos.to_world();
            let offset_x = (fastrand::f32() - 0.5) * 0.6;
            let offset_z = (fastrand::f32() - 0.5) * 0.6;

            commands.spawn((
                LavaBubble {
                    timer: Timer::from_seconds(1.5, TimerMode::Once),
                    start_pos: world_pos + Vec3::new(offset_x, -0.15, offset_z),
                    offset_x,
                    offset_z,
                },
                Text2d::new("o"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 0.5, 0.1, 0.8)),
                Transform::from_translation(world_pos + Vec3::new(offset_x, -0.15, offset_z))
                    .with_scale(Vec3::splat(0.008)),
            ));
        }
    }

    // Animate existing bubbles
    for (entity, mut transform, mut bubble, mut color) in bubbles.iter_mut() {
        bubble.timer.tick(time.delta());

        let t = bubble.timer.fraction();

        // Rise up and expand then pop
        let rise = t * 0.4;
        transform.translation = bubble.start_pos + Vec3::Y * rise;

        // Expand then shrink at end
        let scale = if t < 0.8 {
            0.008 + t * 0.01
        } else {
            0.018 * (1.0 - (t - 0.8) * 5.0).max(0.0)
        };
        transform.scale = Vec3::splat(scale);

        // Color pulse
        let intensity = 0.8 + (t * 10.0).sin() * 0.2;
        color.0 = Color::srgba(1.0, 0.4 * intensity, 0.1, 0.8 * (1.0 - t * 0.5));

        if bubble.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn animate_fountain_spray(
    mut commands: Commands,
    time: Res<Time>,
    mut spawners: Query<&mut EffectSpawner>,
    mut sprays: Query<(Entity, &mut Transform, &mut FountainSpray, &mut TextColor)>,
) {
    // Spawn new spray particles
    for mut spawner in spawners.iter_mut() {
        if !matches!(spawner.effect_type, EffectType::FountainSpray) {
            continue;
        }

        spawner.spawn_timer.tick(time.delta());
        if spawner.spawn_timer.just_finished() {
            let pos = MapPosition {
                x: spawner.x,
                y: spawner.y,
            };
            let world_pos = pos.to_world();

            // Random upward velocity with slight spread
            let vx = (fastrand::f32() - 0.5) * 0.5;
            let vy = 1.5 + fastrand::f32() * 0.5;
            let vz = (fastrand::f32() - 0.5) * 0.5;

            commands.spawn((
                FountainSpray {
                    timer: Timer::from_seconds(1.0, TimerMode::Once),
                    start_pos: world_pos + Vec3::Y * 0.3,
                    velocity: Vec3::new(vx, vy, vz),
                },
                Text2d::new("."),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.8, 1.0, 0.8)),
                Transform::from_translation(world_pos + Vec3::Y * 0.3)
                    .with_scale(Vec3::splat(0.008)),
            ));
        }
    }

    // Animate existing spray particles
    for (entity, mut transform, mut spray, mut color) in sprays.iter_mut() {
        spray.timer.tick(time.delta());

        let t = spray.timer.fraction();

        // Arc motion (gravity)
        let gravity = -3.0;
        let pos = spray.start_pos + spray.velocity * t + Vec3::Y * 0.5 * gravity * t * t;
        transform.translation = pos;

        // Fade out
        let alpha = 0.8 * (1.0 - t);
        color.0 = Color::srgba(0.6, 0.8, 1.0, alpha);

        if spray.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
