//! Entity rendering plugin - billboards for player, monsters, and objects
//!
//! Provides:
//! - Size-scaled monster billboards based on MonsterSize
//! - Health indicators (color tint based on HP percentage)
//! - Status effect indicators (icons for fleeing, stunned, etc.)
//! - Pet/peaceful indicators

use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::components::{Billboard, MapPosition, MonsterMarker, PlayerMarker};
use crate::plugins::animation::AnimationEvent;
use crate::plugins::camera::MainCamera;
use crate::plugins::game::AppState;
use crate::resources::GameStateResource;

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_entities.after(super::map::spawn_map))
            .add_systems(
                Update,
                (
                    billboard_face_camera,
                    sync_entity_positions,
                    sync_floor_objects,
                    update_monster_indicators,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
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

fn spawn_entities(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    _asset_server: Res<AssetServer>,
) {
    let state = &game_state.0;

    // Spawn player
    let player_pos = MapPosition {
        x: state.player.pos.x,
        y: state.player.pos.y,
    };
    let world_pos = player_pos.to_world() + Vec3::Y * 0.5;

    commands.spawn((
        PlayerMarker,
        Billboard,
        player_pos,
        Text2d::new("@"),
        TextFont {
            font_size: 64.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_translation(world_pos).with_scale(Vec3::splat(0.02)),
        Anchor::Center,
    ));

    // Spawn monsters
    spawn_monsters(&mut commands, state);

    // Spawn floor objects
    spawn_floor_objects(&mut commands, state);
}

fn spawn_monsters(commands: &mut Commands, state: &nh_core::GameState) {
    let monsters = nh_data::monsters::MONSTERS;
    for monster in &state.current_level.monsters {
        let map_pos = MapPosition {
            x: monster.x,
            y: monster.y,
        };
        let world_pos = map_pos.to_world() + Vec3::Y * 0.5;

        // Get monster symbol and color from permonst data
        let permonst = &monsters[monster.monster_type as usize];
        let symbol = permonst.symbol;
        let base_color = nethack_color_to_bevy(permonst.color);

        // Apply health-based color tint
        let hp_percent = if monster.hp_max > 0 {
            (monster.hp as f32 / monster.hp_max as f32).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let color = health_tinted_color(base_color, hp_percent);

        // Calculate size based on monster size
        let base_scale = 0.02;
        let size_scale = monster_size_scale(permonst.size);
        let scale = base_scale * size_scale;

        // Spawn monster billboard
        commands.spawn((
            MonsterMarker {
                monster_id: monster.id,
            },
            Billboard,
            map_pos,
            Text2d::new(symbol.to_string()),
            TextFont {
                font_size: 64.0,
                ..default()
            },
            TextColor(color),
            Transform::from_translation(world_pos).with_scale(Vec3::splat(scale)),
            Anchor::Center,
        ));

        // Spawn health indicator for damaged monsters
        if hp_percent < 1.0 && hp_percent > 0.0 {
            spawn_health_indicator(commands, monster, world_pos, hp_percent);
        }

        // Spawn status indicator if any status effects
        if has_visible_status(monster) {
            spawn_status_indicator(commands, monster, world_pos);
        }

        // Spawn allegiance indicator for pets/peaceful
        if monster.state.tame || monster.state.peaceful {
            spawn_allegiance_indicator(commands, monster, world_pos);
        }
    }
}

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
        Anchor::Center,
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
        Anchor::Center,
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
        Anchor::Center,
    ));
}

fn spawn_floor_objects(commands: &mut Commands, state: &nh_core::GameState) {
    use std::collections::HashMap;

    let level = &state.current_level;

    // Group objects by position to detect piles
    let mut objects_by_pos: HashMap<(i8, i8), Vec<&nh_core::object::Object>> = HashMap::new();

    for obj in &level.objects {
        if obj.location == nh_core::object::ObjectLocation::Floor {
            objects_by_pos
                .entry((obj.x, obj.y))
                .or_default()
                .push(obj);
        }
    }

    for ((x, y), objects) in objects_by_pos {
        let map_pos = MapPosition { x, y };
        // Objects render slightly lower than monsters/player
        let world_pos = map_pos.to_world() + Vec3::Y * 0.25;

        if objects.len() == 1 {
            // Single object - show its symbol
            let obj = objects[0];
            let (symbol, color) = object_symbol_and_color(obj);

            commands.spawn((
                FloorObjectMarker { object_id: obj.id },
                Billboard,
                map_pos,
                Text2d::new(symbol.to_string()),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(color),
                Transform::from_translation(world_pos).with_scale(Vec3::splat(0.015)),
                Anchor::Center,
            ));
        } else {
            // Multiple objects - show pile indicator
            commands.spawn((
                PileMarker { x, y },
                Billboard,
                map_pos,
                Text2d::new(format!("({})", objects.len())),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.8, 0.2)), // Gold color for piles
                Transform::from_translation(world_pos).with_scale(Vec3::splat(0.015)),
                Anchor::Center,
            ));
        }
    }
}

fn object_symbol_and_color(obj: &nh_core::object::Object) -> (char, Color) {
    use nh_core::object::ObjectClass;

    let symbol = match obj.class {
        ObjectClass::Weapon => ')',
        ObjectClass::Armor => '[',
        ObjectClass::Ring => '=',
        ObjectClass::Amulet => '"',
        ObjectClass::Tool => '(',
        ObjectClass::Food => '%',
        ObjectClass::Potion => '!',
        ObjectClass::Scroll => '?',
        ObjectClass::Spellbook => '+',
        ObjectClass::Wand => '/',
        ObjectClass::Coin => '$',
        ObjectClass::Gem => '*',
        ObjectClass::Rock => '*',
        ObjectClass::Ball => '0',
        ObjectClass::Chain => '_',
        ObjectClass::Venom => '.',
        ObjectClass::Random | ObjectClass::IllObj => '?',
    };

    let color = match obj.class {
        ObjectClass::Weapon => Color::srgb(0.7, 0.7, 0.8),
        ObjectClass::Armor => Color::srgb(0.6, 0.6, 0.8),
        ObjectClass::Ring => Color::srgb(1.0, 0.84, 0.0),
        ObjectClass::Amulet => Color::srgb(1.0, 0.65, 0.0),
        ObjectClass::Tool => Color::srgb(0.55, 0.35, 0.17),
        ObjectClass::Food => Color::srgb(0.55, 0.27, 0.07),
        ObjectClass::Potion => Color::srgb(1.0, 0.4, 0.7),
        ObjectClass::Scroll => Color::srgb(0.96, 0.96, 0.86),
        ObjectClass::Spellbook => Color::srgb(0.54, 0.17, 0.89),
        ObjectClass::Wand => Color::srgb(0.0, 0.75, 1.0),
        ObjectClass::Coin => Color::srgb(1.0, 0.84, 0.0),
        ObjectClass::Gem => Color::srgb(0.0, 1.0, 1.0),
        ObjectClass::Rock => Color::srgb(0.5, 0.5, 0.5),
        ObjectClass::Ball => Color::srgb(0.4, 0.4, 0.4),
        ObjectClass::Chain => Color::srgb(0.75, 0.75, 0.75),
        ObjectClass::Venom => Color::srgb(0.0, 0.5, 0.0),
        ObjectClass::Random | ObjectClass::IllObj => Color::srgb(1.0, 0.0, 1.0),
    };

    (symbol, color)
}

fn billboard_face_camera(
    camera_query: Query<&Transform, With<MainCamera>>,
    mut billboards: Query<&mut Transform, (With<Billboard>, Without<MainCamera>)>,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    for mut transform in billboards.iter_mut() {
        // Get direction from billboard to camera, ignoring Y for upright billboards
        let to_camera = camera_transform.translation - transform.translation;
        let horizontal = Vec3::new(to_camera.x, 0.0, to_camera.z);

        if horizontal.length_squared() > 0.001 {
            // Face camera (billboard technique)
            transform.look_to(-horizontal.normalize(), Vec3::Y);
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
    mut animation_events: EventWriter<AnimationEvent>,
) {
    // Only sync when game state changes
    if !game_state.is_changed() {
        return;
    }

    let state = &game_state.0;

    // Update player position
    if let Ok((entity, transform, mut map_pos)) = player_query.get_single_mut() {
        let new_x = state.player.pos.x;
        let new_y = state.player.pos.y;

        if map_pos.x != new_x || map_pos.y != new_y {
            let old_world_pos = transform.translation;
            let new_map_pos = MapPosition { x: new_x, y: new_y };
            let new_world_pos = new_map_pos.to_world() + Vec3::Y * 0.5;

            // Send animation event
            animation_events.send(AnimationEvent::EntityMoved {
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
                animation_events.send(AnimationEvent::EntityMoved {
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
            animation_events.send(AnimationEvent::EntityDied { entity });
        }
    }
}

fn sync_floor_objects(
    game_state: Res<GameStateResource>,
    existing_objects: Query<Entity, With<FloorObjectMarker>>,
    existing_piles: Query<Entity, With<PileMarker>>,
    mut commands: Commands,
) {
    // Only sync when game state changes
    if !game_state.is_changed() {
        return;
    }

    // Despawn all existing floor object entities and respawn
    // (Simple approach - could optimize with change detection)
    for entity in existing_objects.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in existing_piles.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Respawn floor objects
    spawn_floor_objects(&mut commands, &game_state.0);
}

/// Update monster indicators (health bars, status effects) when game state changes
fn update_monster_indicators(
    game_state: Res<GameStateResource>,
    mut commands: Commands,
    mut monster_query: Query<(Entity, &MonsterMarker, &MapPosition, &mut TextColor)>,
    health_indicators: Query<Entity, With<HealthIndicator>>,
    status_indicators: Query<Entity, With<StatusIndicator>>,
    allegiance_indicators: Query<Entity, With<AllegianceIndicator>>,
) {
    // Only update when game state changes
    if !game_state.is_changed() {
        return;
    }

    let monsters = nh_data::monsters::MONSTERS;
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
    for (_entity, marker, map_pos, mut text_color) in monster_query.iter_mut() {
        if let Some(monster) = level.monster(marker.monster_id) {
            let permonst = &monsters[monster.monster_type as usize];
            let base_color = nethack_color_to_bevy(permonst.color);

            // Update color based on health
            let hp_percent = if monster.hp_max > 0 {
                (monster.hp as f32 / monster.hp_max as f32).clamp(0.0, 1.0)
            } else {
                1.0
            };
            text_color.0 = health_tinted_color(base_color, hp_percent);

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

/// Convert NetHack color index to Bevy Color
fn nethack_color_to_bevy(color: u8) -> Color {
    match color {
        0 => Color::BLACK,                   // CLR_BLACK
        1 => Color::srgb(0.8, 0.0, 0.0),     // CLR_RED
        2 => Color::srgb(0.0, 0.6, 0.0),     // CLR_GREEN
        3 => Color::srgb(0.6, 0.4, 0.2),     // CLR_BROWN
        4 => Color::srgb(0.0, 0.0, 0.8),     // CLR_BLUE
        5 => Color::srgb(0.8, 0.0, 0.8),     // CLR_MAGENTA
        6 => Color::srgb(0.0, 0.8, 0.8),     // CLR_CYAN
        7 => Color::srgb(0.6, 0.6, 0.6),     // CLR_GRAY
        8 => Color::srgb(0.3, 0.3, 0.3),     // CLR_NO_COLOR (dark gray)
        9 => Color::srgb(1.0, 0.5, 0.0),     // CLR_ORANGE
        10 => Color::srgb(0.0, 1.0, 0.0),    // CLR_BRIGHT_GREEN
        11 => Color::srgb(1.0, 1.0, 0.0),    // CLR_YELLOW
        12 => Color::srgb(0.3, 0.3, 1.0),    // CLR_BRIGHT_BLUE
        13 => Color::srgb(1.0, 0.3, 1.0),    // CLR_BRIGHT_MAGENTA
        14 => Color::srgb(0.3, 1.0, 1.0),    // CLR_BRIGHT_CYAN
        15 => Color::WHITE,                  // CLR_WHITE
        _ => Color::WHITE,
    }
}
