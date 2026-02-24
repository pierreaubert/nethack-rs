//! Animation system for smooth movement and visual feedback
//!
//! Provides:
//! - Entity movement interpolation
//! - Combat hit flashes and floating damage numbers
//! - Death fade-out animations
//! - Environmental animations (torches, tiles, doors)

use bevy::prelude::*;

use crate::components::{MapPosition, MonsterMarker, PlayerMarker, TileMarker};
use crate::plugins::game::AppState;
use crate::resources::{CombatTracker, GameStateResource};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimationSettings>()
            .init_resource::<CombatTracker>()
            .add_message::<AnimationEvent>()
            .add_systems(Startup, setup_environmental_animations)
            .add_systems(
                Update,
                (
                    track_combat,
                    handle_animation_events,
                    animate_movement,
                    animate_combat_flash,
                    animate_floating_text,
                    animate_death,
                    cleanup_finished_animations,
                    // Environmental animations
                    animate_torches,
                    animate_ambient_tiles,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// Animation configuration
#[derive(Resource)]
pub struct AnimationSettings {
    pub movement_duration: f32,
    pub flash_duration: f32,
    pub floating_text_duration: f32,
    pub death_duration: f32,
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            movement_duration: 0.1,      // 100ms for movement
            flash_duration: 0.15,        // 150ms for hit flash
            floating_text_duration: 1.0, // 1 second for floating text
            death_duration: 0.3,         // 300ms for death fade
        }
    }
}

/// Animation events that can be triggered
#[derive(Message)]
pub enum AnimationEvent {
    /// Entity moved from one position to another
    EntityMoved {
        entity: Entity,
        from: Vec3,
        to: Vec3,
    },
    /// Entity was hit in combat
    CombatHit {
        entity: Entity,
        damage: i32,
        position: Vec3,
    },
    /// Entity missed an attack
    CombatMiss { position: Vec3 },
    /// Entity died
    EntityDied { entity: Entity },
    /// Item was picked up
    ItemPickedUp { from: Vec3, to: Vec3 },
}

/// Component for entities currently animating movement
#[derive(Component)]
pub struct MovementAnimation {
    pub start_pos: Vec3,
    pub end_pos: Vec3,
    pub timer: Timer,
}

/// Component for entities with hit flash effect
#[derive(Component)]
pub struct CombatFlash {
    pub timer: Timer,
    pub original_color: Color,
}

/// Component for floating damage text
#[derive(Component)]
pub struct FloatingText {
    pub timer: Timer,
    pub start_y: f32,
}

/// Component for death animation
#[derive(Component)]
pub struct DeathAnimation {
    pub timer: Timer,
}

/// Component for bump animation (when movement blocked)
#[derive(Component)]
pub struct BumpAnimation {
    pub timer: Timer,
    pub direction: Vec3,
    pub origin: Vec3,
}

fn handle_animation_events(
    mut commands: Commands,
    mut events: MessageReader<AnimationEvent>,
    settings: Res<AnimationSettings>,
    materials: Res<Assets<StandardMaterial>>,
    mesh_query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    for event in events.read() {
        match event {
            AnimationEvent::EntityMoved { entity, from, to } => {
                if let Ok(mut entity_commands) = commands.get_entity(*entity) {
                    entity_commands.insert(MovementAnimation {
                        start_pos: *from,
                        end_pos: *to,
                        timer: Timer::from_seconds(settings.movement_duration, TimerMode::Once),
                    });
                }
            }
            AnimationEvent::CombatHit {
                entity,
                damage,
                position,
            } => {
                // Add flash to hit entity (3D mesh)
                if let Ok(mut entity_commands) = commands.get_entity(*entity) {
                    let original_color = mesh_query
                        .get(*entity)
                        .ok()
                        .and_then(|mat_handle| materials.get(&mat_handle.0))
                        .map(|mat| mat.base_color)
                        .unwrap_or(Color::WHITE);
                    entity_commands.insert(CombatFlash {
                        timer: Timer::from_seconds(settings.flash_duration, TimerMode::Once),
                        original_color,
                    });
                }

                // Spawn floating damage number
                spawn_floating_damage(&mut commands, *position, *damage, &settings);
            }
            AnimationEvent::CombatMiss { position } => {
                // Spawn "miss" text
                spawn_floating_miss(&mut commands, *position, &settings);
            }
            AnimationEvent::EntityDied { entity } => {
                if let Ok(mut entity_commands) = commands.get_entity(*entity) {
                    entity_commands.insert(DeathAnimation {
                        timer: Timer::from_seconds(settings.death_duration, TimerMode::Once),
                    });
                }
            }
            AnimationEvent::ItemPickedUp { from, to } => {
                // Could spawn a particle trail here
                spawn_pickup_particle(&mut commands, *from, *to, &settings);
            }
        }
    }
}

fn spawn_floating_damage(
    commands: &mut Commands,
    position: Vec3,
    damage: i32,
    settings: &AnimationSettings,
) {
    let color = if damage > 10 {
        Color::srgb(1.0, 0.0, 0.0) // Red for big damage
    } else if damage > 5 {
        Color::srgb(1.0, 0.5, 0.0) // Orange for medium
    } else {
        Color::srgb(1.0, 1.0, 0.0) // Yellow for small
    };

    commands.spawn((
        FloatingText {
            timer: Timer::from_seconds(settings.floating_text_duration, TimerMode::Once),
            start_y: position.y + 0.8,
        },
        Text2d::new(format!("-{}", damage)),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(color),
        Transform::from_translation(position + Vec3::Y * 0.8).with_scale(Vec3::splat(0.015)),
    ));
}

fn spawn_floating_miss(commands: &mut Commands, position: Vec3, settings: &AnimationSettings) {
    commands.spawn((
        FloatingText {
            timer: Timer::from_seconds(settings.floating_text_duration * 0.7, TimerMode::Once),
            start_y: position.y + 0.8,
        },
        Text2d::new("miss"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
        Transform::from_translation(position + Vec3::Y * 0.8).with_scale(Vec3::splat(0.012)),
    ));
}

fn spawn_pickup_particle(
    commands: &mut Commands,
    from: Vec3,
    _to: Vec3,
    settings: &AnimationSettings,
) {
    // Simple sparkle effect at pickup location
    commands.spawn((
        FloatingText {
            timer: Timer::from_seconds(settings.floating_text_duration * 0.5, TimerMode::Once),
            start_y: from.y,
        },
        Text2d::new("*"),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.84, 0.0)), // Gold
        Transform::from_translation(from + Vec3::Y * 0.3).with_scale(Vec3::splat(0.02)),
    ));
}

fn animate_movement(time: Res<Time>, mut query: Query<(&mut Transform, &mut MovementAnimation)>) {
    for (mut transform, mut anim) in query.iter_mut() {
        anim.timer.tick(time.delta());

        let t = anim.timer.fraction();
        // Smooth ease-out interpolation
        let t = 1.0 - (1.0 - t).powi(2);

        transform.translation = anim.start_pos.lerp(anim.end_pos, t);
    }
}

fn animate_combat_flash(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&MeshMaterial3d<StandardMaterial>, &mut CombatFlash)>,
) {
    for (mat_handle, mut flash) in query.iter_mut() {
        flash.timer.tick(time.delta());

        let t = flash.timer.fraction();
        // Flash white then back to original
        let flash_intensity = (1.0 - t).powi(2);

        let original = flash.original_color.to_srgba();
        let new_color = Color::srgb(
            original.red + (1.0 - original.red) * flash_intensity,
            original.green + (1.0 - original.green) * flash_intensity * 0.3,
            original.blue + (1.0 - original.blue) * flash_intensity * 0.3,
        );
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            material.base_color = new_color;
        }
    }
}

fn animate_floating_text(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut FloatingText, &mut TextColor)>,
) {
    for (mut transform, mut floating, mut text_color) in query.iter_mut() {
        floating.timer.tick(time.delta());

        let t = floating.timer.fraction();

        // Float upward
        transform.translation.y = floating.start_y + t * 1.5;

        // Fade out
        let alpha = 1.0 - t;
        let current = text_color.0.to_srgba();
        text_color.0 = Color::srgba(current.red, current.green, current.blue, alpha);

        // Scale down slightly
        let scale = 0.015 * (1.0 - t * 0.3);
        transform.scale = Vec3::splat(scale);
    }
}

fn animate_death(
    time: Res<Time>,
    mut text_query: Query<
        (&mut Transform, &mut TextColor, &mut DeathAnimation),
        Without<Mesh3d>,
    >,
    mut mesh_query: Query<
        (&mut Transform, &mut DeathAnimation),
        (With<Mesh3d>, Without<TextColor>),
    >,
) {
    // Billboard/text entities: fade alpha + shrink
    for (mut transform, mut text_color, mut death) in text_query.iter_mut() {
        death.timer.tick(time.delta());

        let t = death.timer.fraction();

        let alpha = 1.0 - t;
        let current = text_color.0.to_srgba();
        text_color.0 = Color::srgba(current.red, current.green, current.blue, alpha);

        transform.translation.y -= time.delta_secs() * 2.0;
        let shrink = (1.0 - time.delta_secs() * 2.0).max(0.01);
        transform.scale *= shrink;
    }

    // 3D mesh entities: fall down + shrink
    for (mut transform, mut death) in mesh_query.iter_mut() {
        death.timer.tick(time.delta());

        transform.translation.y -= time.delta_secs() * 2.0;
        let shrink = 1.0 - time.delta_secs() * 3.0;
        transform.scale *= shrink.max(0.01);
    }
}

fn cleanup_finished_animations(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    movement_query: Query<(Entity, &MovementAnimation)>,
    flash_query: Query<(Entity, &CombatFlash, Option<&MeshMaterial3d<StandardMaterial>>)>,
    floating_query: Query<(Entity, &FloatingText)>,
    death_query: Query<(Entity, &DeathAnimation)>,
) {
    // Remove finished movement animations
    for (entity, anim) in movement_query.iter() {
        if anim.timer.is_finished() {
            commands.entity(entity).remove::<MovementAnimation>();
        }
    }

    // Remove finished flash effects and restore material color
    for (entity, flash, mat_handle) in flash_query.iter() {
        if flash.timer.is_finished() {
            // Restore original material color for 3D mesh entities
            if let Some(mat_handle) = mat_handle
                && let Some(material) = materials.get_mut(&mat_handle.0)
            {
                material.base_color = flash.original_color;
            }
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.remove::<CombatFlash>();
            }
        }
    }

    // Despawn finished floating text
    for (entity, floating) in floating_query.iter() {
        if floating.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }

    // Despawn dead entities after death animation
    for (entity, death) in death_query.iter() {
        if death.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Track HP changes and emit combat animation events
fn track_combat(
    game_state: Res<GameStateResource>,
    mut tracker: ResMut<CombatTracker>,
    mut events: MessageWriter<AnimationEvent>,
    player_query: Query<(Entity, &Transform), With<PlayerMarker>>,
    monster_query: Query<(Entity, &MonsterMarker, &Transform)>,
) {
    if !game_state.is_changed() {
        return;
    }

    let state = &game_state.0;

    // Initialize tracker on first run
    if !tracker.initialized {
        tracker.prev_player_hp = state.player.hp;
        tracker.prev_inventory_count = state.inventory.len();
        for monster in &state.current_level.monsters {
            tracker.prev_monster_hp.insert(monster.id, monster.hp);
        }
        tracker.initialized = true;
        return;
    }

    // Check player damage
    let player_hp = state.player.hp;
    if player_hp < tracker.prev_player_hp {
        let damage = tracker.prev_player_hp - player_hp;
        if let Ok((entity, transform)) = player_query.single() {
            events.write(AnimationEvent::CombatHit {
                entity,
                damage,
                position: transform.translation,
            });
        }
    }
    tracker.prev_player_hp = player_hp;

    // Check item pickup
    let inventory_count = state.inventory.len();
    if inventory_count > tracker.prev_inventory_count {
        // Items were picked up - show sparkle at player position
        if let Ok((_, transform)) = player_query.single() {
            events.write(AnimationEvent::ItemPickedUp {
                from: transform.translation - Vec3::Y * 0.3,
                to: transform.translation,
            });
        }
    }
    tracker.prev_inventory_count = inventory_count;

    // Check monster damage
    let mut current_monsters = std::collections::HashSet::<nh_core::monster::MonsterId>::new();
    for monster in &state.current_level.monsters {
        current_monsters.insert(monster.id);

        let prev_hp = tracker.prev_monster_hp.get(&monster.id).copied();
        if let Some(prev) = prev_hp {
            if monster.hp < prev {
                let damage = prev - monster.hp;
                // Find the monster entity
                for (entity, marker, transform) in monster_query.iter() {
                    if marker.monster_id == monster.id {
                        events.write(AnimationEvent::CombatHit {
                            entity,
                            damage,
                            position: transform.translation,
                        });
                        break;
                    }
                }
            }
        }
        tracker.prev_monster_hp.insert(monster.id, monster.hp);
    }

    // Clean up dead monsters from tracker
    tracker
        .prev_monster_hp
        .retain(|id, _| current_monsters.contains(id));
}

// =============================================================================
// Environmental Animations
// =============================================================================

/// Component for animated torch tiles (flickering light effect)
#[derive(Component)]
pub struct TorchAnimation {
    pub phase: f32,
    pub flicker_speed: f32,
    pub base_color: Color,
}

/// Component for ambient tile animations (grass swaying, water ripple, etc.)
#[derive(Component)]
pub struct AmbientTileAnimation {
    pub phase: f32,
    pub animation_type: AmbientAnimationType,
}

#[derive(Clone, Copy)]
pub enum AmbientAnimationType {
    /// Grass/foliage gentle sway
    Grass,
    /// Tree leaves rustling
    Tree,
    /// Altar glow pulsing
    Altar,
    /// Grave eerie pulse
    Grave,
}

/// Setup environmental animations on map tiles
fn setup_environmental_animations(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    tiles: Query<(Entity, &MapPosition), With<TileMarker>>,
) {
    let level = &game_state.0.current_level;

    for (entity, pos) in tiles.iter() {
        let cell = level.cell(pos.x as usize, pos.y as usize);

        // Add torch animation to lit corridors and rooms
        if cell.lit && matches!(cell.typ, nh_core::dungeon::CellType::Corridor) {
            commands.entity(entity).insert(TorchAnimation {
                phase: fastrand::f32() * std::f32::consts::TAU,
                flicker_speed: 8.0 + fastrand::f32() * 4.0,
                base_color: Color::srgb(1.0, 0.8, 0.4),
            });
        }

        // Add ambient animations to specific tile types
        let ambient_type = match cell.typ {
            nh_core::dungeon::CellType::Tree => Some(AmbientAnimationType::Tree),
            nh_core::dungeon::CellType::Altar => Some(AmbientAnimationType::Altar),
            nh_core::dungeon::CellType::Grave => Some(AmbientAnimationType::Grave),
            _ => None,
        };

        if let Some(anim_type) = ambient_type {
            commands.entity(entity).insert(AmbientTileAnimation {
                phase: fastrand::f32() * std::f32::consts::TAU,
                animation_type: anim_type,
            });
        }
    }
}

/// Animate torch flickering effect via emissive glow on 3D tiles
fn animate_torches(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&MeshMaterial3d<StandardMaterial>, &mut TorchAnimation)>,
) {
    for (mat_handle, mut torch) in query.iter_mut() {
        torch.phase += time.delta_secs() * torch.flicker_speed;
        if torch.phase > std::f32::consts::TAU {
            torch.phase -= std::f32::consts::TAU;
        }

        // Multi-frequency flicker for realistic torch effect
        let flicker1 = (torch.phase).sin() * 0.1;
        let flicker2 = (torch.phase * 2.3).sin() * 0.05;
        let flicker3 = (torch.phase * 5.7).sin() * 0.03;
        let flicker = 1.0 + flicker1 + flicker2 + flicker3;

        let base = torch.base_color.to_srgba();
        if let Some(material) = materials.get_mut(&mat_handle.0) {
            material.emissive = LinearRgba::new(
                (base.red * flicker).clamp(0.0, 1.0),
                (base.green * flicker * 0.9).clamp(0.0, 1.0),
                (base.blue * flicker * 0.7).clamp(0.0, 1.0),
                base.alpha,
            );
        }
    }
}

/// Animate ambient environmental tiles via 3D material properties
fn animate_ambient_tiles(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
        &mut AmbientTileAnimation,
    )>,
) {
    for (mut transform, mat_handle, mut anim) in query.iter_mut() {
        anim.phase += time.delta_secs();
        if anim.phase > std::f32::consts::TAU * 10.0 {
            anim.phase -= std::f32::consts::TAU * 10.0;
        }

        match anim.animation_type {
            AmbientAnimationType::Grass => {
                // Gentle swaying motion (Transform-only, no color change)
                let sway = (anim.phase * 1.5).sin() * 0.02;
                transform.rotation = Quat::from_rotation_z(sway);
            }
            AmbientAnimationType::Tree => {
                // Rustling leaves effect - subtle color variation
                if let Some(material) = materials.get_mut(&mat_handle.0) {
                    let rustle = (anim.phase * 2.0).sin() * 0.1;
                    let base_green = 0.5 + rustle * 0.1;
                    material.base_color = Color::srgb(0.1, base_green, 0.1);
                }

                // Very slight sway
                let sway = (anim.phase * 0.8).sin() * 0.01;
                transform.rotation = Quat::from_rotation_z(sway);
            }
            AmbientAnimationType::Altar => {
                // Mystical pulsing glow
                if let Some(material) = materials.get_mut(&mat_handle.0) {
                    let pulse = (anim.phase * 1.2).sin() * 0.5 + 0.5;
                    let intensity = 0.6 + pulse * 0.4;
                    material.base_color =
                        Color::srgba(intensity, intensity * 0.8, intensity, 1.0);
                }
            }
            AmbientAnimationType::Grave => {
                // Eerie subtle pulse
                if let Some(material) = materials.get_mut(&mat_handle.0) {
                    let pulse = (anim.phase * 0.5).sin() * 0.3 + 0.7;
                    material.base_color =
                        Color::srgba(0.5 * pulse, 0.5 * pulse, 0.6 * pulse, 1.0);
                }
            }
        }
    }
}
