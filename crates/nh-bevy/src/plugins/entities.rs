//! Entity rendering plugin - billboards for player, monsters, and objects
//!
//! Provides:
//! - Size-scaled monster billboards based on MonsterSize
//! - Health indicators (color tint based on HP percentage)
//! - Status effect indicators (icons for fleeing, stunned, etc.)
//! - Pet/peaceful indicators

use std::collections::HashSet;

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::components::{Billboard, CameraMode, MapPosition, MonsterMarker, PlayerMarker};
use crate::plugins::animation::AnimationEvent;
use crate::plugins::camera::MainCamera;
use crate::plugins::game::AppState;
use crate::plugins::model_assets::ModelAssets;
use crate::plugins::models::{BillboardSpawner, ModelBuilder};
use crate::plugins::sprites::SpriteAssets;
use crate::resources::{AssetRegistryResource, GameStateResource};

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityState>()
            .add_systems(Startup, spawn_entities.after(super::map::spawn_map))
            .add_systems(OnEnter(AppState::Playing), mark_entities_for_respawn)
            .add_systems(
                Update,
                (
                    check_level_change,
                    sync_monster_entities,
                    sync_entity_positions,
                    sync_floor_objects,
                    update_monster_indicators,
                    billboard_face_camera,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Resource, Default)]
struct EntityState {
    current_dlevel: Option<nh_core::dungeon::DLevel>,
    /// Set when entering Playing state to force a full entity respawn
    needs_respawn: bool,
}

/// Marker for floor object entities
#[derive(Component)]
pub struct FloorObjectMarker {
    pub object_id: nh_core::object::ObjectId,
}

/// Marker for pile indicator (multiple objects)
#[derive(Component)]
pub struct PileMarker {
    pub x: i8,
    pub y: i8,
}

/// Marker for health indicator bar
#[derive(Component)]
pub struct HealthIndicator {
    pub monster_id: nh_core::monster::MonsterId,
}

/// Marker for status effect indicator
#[derive(Component)]
pub struct StatusIndicator {
    pub monster_id: nh_core::monster::MonsterId,
}

/// Marker for pet/peaceful indicator
#[derive(Component)]
pub struct AllegianceIndicator {
    pub monster_id: nh_core::monster::MonsterId,
}

/// Mark entities for respawn when entering Playing state (e.g., after character creation)
fn mark_entities_for_respawn(mut entity_state: ResMut<EntityState>) {
    entity_state.needs_respawn = true;
}

/// Check if level changed (or respawn forced) and respawn entities
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn check_level_change(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    mut entity_state: ResMut<EntityState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sprite_assets: Option<Res<SpriteAssets>>,
    registry: Option<Res<AssetRegistryResource>>,
    asset_server: Res<AssetServer>,
    model_assets: Option<Res<ModelAssets>>,
    player_query: Query<Entity, With<PlayerMarker>>,
    monster_query: Query<Entity, With<MonsterMarker>>,
    object_query: Query<Entity, With<FloorObjectMarker>>,
    pile_query: Query<Entity, With<PileMarker>>,
    indicator_query: Query<
        Entity,
        Or<(
            With<HealthIndicator>,
            With<StatusIndicator>,
            With<AllegianceIndicator>,
        )>,
    >,
) {
    if !game_state.is_changed() && !entity_state.needs_respawn {
        return;
    }

    let current_dlevel = game_state.0.current_level.dlevel;
    let force_respawn = entity_state.needs_respawn;

    // Initialize on first run (but not if we're forcing a respawn)
    if entity_state.current_dlevel.is_none() && !force_respawn {
        entity_state.current_dlevel = Some(current_dlevel);
        return;
    }

    if entity_state.current_dlevel != Some(current_dlevel) || force_respawn {
        info!(
            "Respawning entities (level {:?} → {:?}, forced={})",
            entity_state.current_dlevel, current_dlevel, force_respawn
        );

        // Despawn all entities
        for entity in player_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in monster_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in object_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in pile_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in indicator_query.iter() {
            commands.entity(entity).despawn();
        }

        // Spawn new entities (3D model → billboard sprite → procedural fallback)
        spawn_entities_internal(
            &mut commands,
            &game_state.0,
            &mut meshes,
            &mut materials,
            sprite_assets.as_deref(),
            registry.as_deref(),
            &asset_server,
            model_assets.as_deref(),
        );

        // Update state
        entity_state.current_dlevel = Some(current_dlevel);
        entity_state.needs_respawn = false;
    }
}

/// Spawn new monster entities that appeared in GameState but don't have Bevy entities yet.
/// Handles monsters spawning mid-game (e.g., MonsterSpawn timed events).
#[allow(clippy::too_many_arguments)]
fn sync_monster_entities(
    game_state: Res<GameStateResource>,
    monster_query: Query<&MonsterMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sprite_assets: Option<Res<SpriteAssets>>,
    registry: Option<Res<AssetRegistryResource>>,
    asset_server: Res<AssetServer>,
    model_assets: Option<Res<ModelAssets>>,
) {
    if !game_state.is_changed() {
        return;
    }

    let level = &game_state.0.current_level;
    let monsters_data = nh_core::data::monsters::MONSTERS;

    // Collect existing monster entity IDs
    let existing_ids: HashSet<nh_core::monster::MonsterId> =
        monster_query.iter().map(|m| m.monster_id).collect();

    for monster in &level.monsters {
        if !existing_ids.contains(&monster.id) {
            let map_pos = MapPosition {
                x: monster.x,
                y: monster.y,
            };
            let world_pos = map_pos.to_world();
            let monster_def = &monsters_data[monster.monster_type as usize];

            // Try 3D model → billboard sprite → procedural fallback
            let transform = Transform::from_translation(world_pos);

            let entity = if let Some(ref sprites) = sprite_assets {
                let mut spawner = BillboardSpawner::new(
                    sprites,
                    &mut materials,
                    registry.as_deref(),
                    &asset_server,
                )
                .with_model_assets(model_assets.as_deref());

                spawner
                    .spawn_3d_monster(&mut commands, monster, monster_def, transform)
                    .or_else(|| {
                        spawner.spawn_monster_billboard(
                            &mut commands,
                            monster,
                            monster_def,
                            transform,
                        )
                    })
            } else {
                None
            };

            let entity = entity.unwrap_or_else(|| {
                let mut model_builder = ModelBuilder::new(&mut meshes, &mut materials);
                model_builder.spawn_monster(
                    &mut commands,
                    monster,
                    monster_def,
                    transform,
                )
            });

            commands.entity(entity).insert(map_pos);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_entities(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sprite_assets: Option<Res<SpriteAssets>>,
    registry: Option<Res<AssetRegistryResource>>,
    asset_server: Res<AssetServer>,
    model_assets: Option<Res<ModelAssets>>,
) {
    let state = &game_state.0;
    spawn_entities_internal(
        &mut commands,
        state,
        &mut meshes,
        &mut materials,
        sprite_assets.as_deref(),
        registry.as_deref(),
        &asset_server,
        model_assets.as_deref(),
    );
}

#[allow(clippy::too_many_arguments)]
fn spawn_entities_internal(
    commands: &mut Commands,
    state: &nh_core::GameState,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    sprite_assets: Option<&SpriteAssets>,
    registry: Option<&AssetRegistryResource>,
    asset_server: &AssetServer,
    model_assets: Option<&ModelAssets>,
) {
    // Spawn player — try billboard, fall back to 3D model
    let player_pos = MapPosition {
        x: state.player.pos.x,
        y: state.player.pos.y,
    };
    let world_pos = player_pos.to_world();

    let player_spawned = if let Some(sprites) = sprite_assets {
        let mut spawner = BillboardSpawner::new(sprites, materials, registry, asset_server)
            .with_model_assets(model_assets);
        spawner.spawn_player_billboard(
            commands,
            &state.player,
            Transform::from_translation(world_pos),
        )
    } else {
        None
    };

    if player_spawned.is_none() {
        let mut model_builder = ModelBuilder::new(meshes, materials);
        model_builder.spawn_player(
            commands,
            &state.player,
            Transform::from_translation(world_pos),
        );
    }

    // Spawn monsters (3D model → billboard → procedural fallback)
    let monsters_data = nh_core::data::monsters::MONSTERS;
    for monster in &state.current_level.monsters {
        let map_pos = MapPosition {
            x: monster.x,
            y: monster.y,
        };
        let world_pos = map_pos.to_world();
        let monster_def = &monsters_data[monster.monster_type as usize];
        let transform = Transform::from_translation(world_pos);

        let entity = if let Some(sprites) = sprite_assets {
            let mut spawner = BillboardSpawner::new(sprites, materials, registry, asset_server)
                .with_model_assets(model_assets);
            spawner
                .spawn_3d_monster(commands, monster, monster_def, transform)
                .or_else(|| {
                    spawner.spawn_monster_billboard(commands, monster, monster_def, transform)
                })
        } else {
            None
        };

        let entity = entity.unwrap_or_else(|| {
            let mut model_builder = ModelBuilder::new(meshes, materials);
            model_builder.spawn_monster(commands, monster, monster_def, transform)
        });

        commands.entity(entity).insert(map_pos);

        // Spawn indicators above sprites
        let hp_percent = if monster.hp_max > 0 {
            (monster.hp as f32 / monster.hp_max as f32).clamp(0.0, 1.0)
        } else {
            1.0
        };

        if hp_percent < 1.0 && hp_percent > 0.0 {
            spawn_health_indicator(commands, monster, world_pos + Vec3::Y * 0.5, hp_percent);
        }

        if has_visible_status(monster) {
            spawn_status_indicator(commands, monster, world_pos + Vec3::Y * 0.5);
        }

        if monster.state.tame || monster.state.peaceful {
            spawn_allegiance_indicator(commands, monster, world_pos + Vec3::Y * 0.5);
        }
    }

    // Spawn floor objects (3D model → billboard sprite → procedural fallback)
    spawn_floor_objects(
        commands,
        state,
        meshes,
        materials,
        sprite_assets,
        registry,
        asset_server,
        model_assets,
    );
}

// spawn_monsters function is removed as it is integrated into spawn_entities_internal

/*
/// Get size scale multiplier based on MonsterSize
fn monster_size_scale(size: nh_core::monster::MonsterSize) -> f32 {
    use nh_core::monster::MonsterSize;
    match size {
        MonsterSize::Tiny => 0.6,
        MonsterSize::Small => 0.8,
        MonsterSize::Medium => 1.0,
        MonsterSize::Large => 1.3,
        MonsterSize::Huge => 1.6,
        MonsterSize::Gigantic => 2.0,
    }
}
*/

/// Apply health-based color tinting (red tint for low HP)
fn health_tinted_color(base_color: Color, hp_percent: f32) -> Color {
    if hp_percent > 0.75 {
        base_color
    } else {
        let rgba = base_color.to_srgba();
        if hp_percent > 0.5 {
            // Slight yellow tint
            Color::srgba(
                rgba.red * 0.9 + 0.1,
                rgba.green * 0.9 + 0.1,
                rgba.blue * 0.5,
                rgba.alpha,
            )
        } else if hp_percent > 0.25 {
            // Orange tint
            Color::srgba(
                rgba.red * 0.6 + 0.4,
                rgba.green * 0.5,
                rgba.blue * 0.3,
                rgba.alpha,
            )
        } else {
            // Red tint for critical
            Color::srgba(
                rgba.red * 0.3 + 0.7,
                rgba.green * 0.3,
                rgba.blue * 0.3,
                rgba.alpha,
            )
        }
    }
}

/// Check if monster has any visible status effects
fn has_visible_status(monster: &nh_core::monster::Monster) -> bool {
    monster.state.fleeing
        || monster.state.confused
        || monster.state.stunned
        || monster.state.sleeping
        || monster.state.blinded
        || monster.state.paralyzed
}

/// Spawn a health bar indicator below the monster
fn spawn_health_indicator(
    commands: &mut Commands,
    monster: &nh_core::monster::Monster,
    world_pos: Vec3,
    hp_percent: f32,
) {
    // Health bar character based on percentage
    let bar_char = if hp_percent > 0.75 {
        "▓▓▓▓"
    } else if hp_percent > 0.5 {
        "▓▓▓░"
    } else if hp_percent > 0.25 {
        "▓▓░░"
    } else {
        "▓░░░"
    };

    let bar_color = if hp_percent > 0.5 {
        Color::srgb(0.2, 0.8, 0.2) // Green
    } else if hp_percent > 0.25 {
        Color::srgb(0.9, 0.7, 0.1) // Yellow
    } else {
        Color::srgb(0.9, 0.2, 0.2) // Red
    };

    commands.spawn((
        HealthIndicator {
            monster_id: monster.id,
        },
        Billboard,
        Text2d::new(bar_char),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(bar_color),
        Transform::from_translation(world_pos + Vec3::new(0.0, -0.35, 0.0))
            .with_scale(Vec3::splat(0.01)),
        Anchor::CENTER,
    ));
}

/// Spawn status effect indicator above the monster
fn spawn_status_indicator(
    commands: &mut Commands,
    monster: &nh_core::monster::Monster,
    world_pos: Vec3,
) {
    // Build status string from effects
    let mut status = String::new();
    if monster.state.sleeping {
        status.push('Z'); // Zzz for sleeping
    }
    if monster.state.confused {
        status.push('?');
    }
    if monster.state.stunned {
        status.push('*');
    }
    if monster.state.fleeing {
        status.push('!');
    }
    if monster.state.blinded {
        status.push('☼');
    }
    if monster.state.paralyzed {
        status.push('▬');
    }

    if status.is_empty() {
        return;
    }

    commands.spawn((
        StatusIndicator {
            monster_id: monster.id,
        },
        Billboard,
        Text2d::new(status),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.8, 0.2)), // Yellow for status
        Transform::from_translation(world_pos + Vec3::new(0.0, 0.5, 0.0))
            .with_scale(Vec3::splat(0.012)),
        Anchor::CENTER,
    ));
}

/// Spawn allegiance indicator (heart for pet, olive branch for peaceful)
fn spawn_allegiance_indicator(
    commands: &mut Commands,
    monster: &nh_core::monster::Monster,
    world_pos: Vec3,
) {
    let (symbol, color) = if monster.state.tame {
        ("♥", Color::srgb(1.0, 0.3, 0.5)) // Pink heart for pet
    } else {
        ("☮", Color::srgb(0.3, 0.8, 0.3)) // Green peace for peaceful
    };

    commands.spawn((
        AllegianceIndicator {
            monster_id: monster.id,
        },
        Billboard,
        Text2d::new(symbol),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(color),
        Transform::from_translation(world_pos + Vec3::new(0.25, 0.4, 0.0))
            .with_scale(Vec3::splat(0.012)),
        Anchor::CENTER,
    ));
}

#[allow(clippy::too_many_arguments)]
fn spawn_floor_objects(
    commands: &mut Commands,
    state: &nh_core::GameState,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    sprite_assets: Option<&SpriteAssets>,
    registry: Option<&AssetRegistryResource>,
    asset_server: &AssetServer,
    model_assets: Option<&ModelAssets>,
) {
    use std::collections::HashMap;

    let level = &state.current_level;

    // Group objects by position to detect piles
    let mut objects_by_pos: HashMap<(i8, i8), Vec<&nh_core::object::Object>> = HashMap::new();

    for obj in &level.objects {
        if obj.location == nh_core::object::ObjectLocation::Floor {
            objects_by_pos.entry((obj.x, obj.y)).or_default().push(obj);
        }
    }

    for ((x, y), objects) in objects_by_pos {
        let map_pos = MapPosition { x, y };
        let world_pos = map_pos.to_world();

        if objects.len() == 1 {
            let obj = objects[0];
            let transform = Transform::from_translation(world_pos);

            // Try 3D model → billboard sprite → procedural fallback
            let entity = if let Some(sprites) = sprite_assets {
                let mut spawner =
                    BillboardSpawner::new(sprites, materials, registry, asset_server)
                        .with_model_assets(model_assets);
                spawner
                    .spawn_3d_object(commands, obj, transform)
                    .or_else(|| spawner.spawn_object_billboard(commands, obj, transform))
            } else {
                None
            };

            let entity = entity.unwrap_or_else(|| {
                let mut model_builder = ModelBuilder::new(meshes, materials);
                model_builder.spawn_object(commands, obj, transform)
            });

            commands
                .entity(entity)
                .insert((FloorObjectMarker { object_id: obj.id }, map_pos));
        } else {
            // Multiple objects - spawn pile indicator (keep as 3D)
            let mut model_builder = ModelBuilder::new(meshes, materials);
            let entity = model_builder.spawn_pile(
                commands,
                objects.len(),
                Transform::from_translation(world_pos),
            );

            commands
                .entity(entity)
                .insert((PileMarker { x, y }, map_pos));
        }
    }
}

fn billboard_face_camera(
    camera_query: Query<&Transform, With<MainCamera>>,
    camera_mode: Res<State<CameraMode>>,
    mut billboards: Query<&mut Transform, (With<Billboard>, Without<MainCamera>)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    match camera_mode.get() {
        CameraMode::TopDown => {
            // Camera looks straight down — lay billboards flat on the ground plane
            // facing upward so they're visible from above.
            // The quad's default normal is +Z; rotating it to face +Y means
            // we look along -Y with forward = -Z (screen up).
            for mut transform in billboards.iter_mut() {
                let pos = transform.translation;
                let scale = transform.scale;
                *transform = Transform::from_translation(pos)
                    .with_scale(scale)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z);
            }
        }
        CameraMode::Isometric => {
            // Camera is at a fixed 45-degree angle — face the camera direction
            let cam_forward = camera_transform.forward().as_vec3();
            for mut transform in billboards.iter_mut() {
                let pos = transform.translation;
                let scale = transform.scale;
                *transform = Transform::from_translation(pos)
                    .with_scale(scale)
                    .looking_to(-cam_forward, Vec3::Y);
            }
        }
        CameraMode::ThirdPerson | CameraMode::FirstPerson => {
            // Upright billboards that rotate around Y to face camera
            for mut transform in billboards.iter_mut() {
                let to_camera = camera_transform.translation - transform.translation;
                let horizontal = Vec3::new(to_camera.x, 0.0, to_camera.z);
                if horizontal.length_squared() > 0.001 {
                    transform.look_to(-horizontal.normalize(), Vec3::Y);
                }
            }
        }
    }
}

fn sync_entity_positions(
    game_state: Res<GameStateResource>,
    mut player_query: Query<(Entity, &Transform, &mut MapPosition), With<PlayerMarker>>,
    mut monster_query: Query<
        (Entity, &MonsterMarker, &Transform, &mut MapPosition),
        Without<PlayerMarker>,
    >,
    mut animation_events: MessageWriter<AnimationEvent>,
) {
    // Only sync when game state changes
    if !game_state.is_changed() {
        return;
    }

    let state = &game_state.0;

    // Update player position
    if let Ok((entity, transform, mut map_pos)) = player_query.single_mut() {
        let new_x = state.player.pos.x;
        let new_y = state.player.pos.y;

        if map_pos.x != new_x || map_pos.y != new_y {
            let old_world_pos = transform.translation;
            let new_map_pos = MapPosition { x: new_x, y: new_y };
            let new_world_pos = new_map_pos.to_world() + Vec3::Y * 0.5;

            // Send animation event
            animation_events.write(AnimationEvent::EntityMoved {
                entity,
                from: old_world_pos,
                to: new_world_pos,
            });

            map_pos.x = new_x;
            map_pos.y = new_y;
            // Don't update transform here - animation will handle it
        }
    }

    // Update monster positions, despawn dead monsters
    let level = &state.current_level;
    for (entity, marker, transform, mut map_pos) in monster_query.iter_mut() {
        if let Some(monster) = level.monster(marker.monster_id) {
            if map_pos.x != monster.x || map_pos.y != monster.y {
                let old_world_pos = transform.translation;
                let new_map_pos = MapPosition {
                    x: monster.x,
                    y: monster.y,
                };
                let new_world_pos = new_map_pos.to_world() + Vec3::Y * 0.5;

                // Send animation event
                animation_events.write(AnimationEvent::EntityMoved {
                    entity,
                    from: old_world_pos,
                    to: new_world_pos,
                });

                map_pos.x = monster.x;
                map_pos.y = monster.y;
                // Don't update transform here - animation will handle it
            }
        } else {
            // Monster no longer exists - send death animation
            animation_events.write(AnimationEvent::EntityDied { entity });
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn sync_floor_objects(
    game_state: Res<GameStateResource>,
    existing_objects: Query<Entity, With<FloorObjectMarker>>,
    existing_piles: Query<Entity, With<PileMarker>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sprite_assets: Option<Res<SpriteAssets>>,
    registry: Option<Res<AssetRegistryResource>>,
    asset_server: Res<AssetServer>,
    model_assets: Option<Res<ModelAssets>>,
) {
    if !game_state.is_changed() {
        return;
    }

    for entity in existing_objects.iter() {
        commands.entity(entity).despawn();
    }
    for entity in existing_piles.iter() {
        commands.entity(entity).despawn();
    }

    spawn_floor_objects(
        &mut commands,
        &game_state.0,
        &mut meshes,
        &mut materials,
        sprite_assets.as_deref(),
        registry.as_deref(),
        &asset_server,
        model_assets.as_deref(),
    );
}

/// Update monster indicators (health bars, status effects) when game state changes
fn update_monster_indicators(
    game_state: Res<GameStateResource>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    monster_query: Query<(Entity, &MonsterMarker, &MapPosition, &MeshMaterial3d<StandardMaterial>)>,
    health_indicators: Query<Entity, With<HealthIndicator>>,
    status_indicators: Query<Entity, With<StatusIndicator>>,
    allegiance_indicators: Query<Entity, With<AllegianceIndicator>>,
) {
    // Only update when game state changes
    if !game_state.is_changed() {
        return;
    }

    let monsters = nh_core::data::monsters::MONSTERS;
    let level = &game_state.0.current_level;

    // Clear old indicators
    for entity in health_indicators.iter() {
        commands.entity(entity).despawn();
    }
    for entity in status_indicators.iter() {
        commands.entity(entity).despawn();
    }
    for entity in allegiance_indicators.iter() {
        commands.entity(entity).despawn();
    }

    // Update monster visuals and respawn indicators
    for (_entity, marker, map_pos, mat_handle) in monster_query.iter() {
        if let Some(monster) = level.monster(marker.monster_id) {
            let permonst = &monsters[monster.monster_type as usize];
            let base_color = nethack_color_to_bevy(permonst.color);

            // Update material color based on health
            let hp_percent = if monster.hp_max > 0 {
                (monster.hp as f32 / monster.hp_max as f32).clamp(0.0, 1.0)
            } else {
                1.0
            };
            if let Some(material) = materials.get_mut(&mat_handle.0) {
                material.base_color = health_tinted_color(base_color, hp_percent);
            }

            let world_pos = map_pos.to_world() + Vec3::Y * 0.5;

            // Respawn health indicator for damaged monsters
            if hp_percent < 1.0 && hp_percent > 0.0 {
                spawn_health_indicator(&mut commands, monster, world_pos, hp_percent);
            }

            // Respawn status indicator if any status effects
            if has_visible_status(monster) {
                spawn_status_indicator(&mut commands, monster, world_pos);
            }

            // Respawn allegiance indicator for pets/peaceful
            if monster.state.tame || monster.state.peaceful {
                spawn_allegiance_indicator(&mut commands, monster, world_pos);
            }
        }
    }
}

use super::models::nethack_color_to_bevy;
