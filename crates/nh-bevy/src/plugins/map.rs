//! Map rendering plugin - converts 80x21 grid to 3D geometry

use bevy::prelude::*;

use crate::components::{DoorAnimation, DoorMarker, MapPosition, TileMarker};
use crate::resources::GameStateResource;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_tile_assets, spawn_map).chain())
            .add_systems(Update, (sync_door_states, animate_doors));
    }
}

/// Door height constants
const DOOR_OPEN_HEIGHT: f32 = 0.2;
const DOOR_CLOSED_HEIGHT: f32 = 0.8;
const DOOR_ANIMATION_DURATION: f32 = 0.25;

/// Tile mesh handles
#[derive(Resource)]
pub struct TileMeshes {
    pub floor: Handle<Mesh>,
    pub wall: Handle<Mesh>,
}

/// Tile material handles
#[derive(Resource)]
pub struct TileMaterials {
    pub floor: Handle<StandardMaterial>,
    pub corridor: Handle<StandardMaterial>,
    pub wall: Handle<StandardMaterial>,
    pub door: Handle<StandardMaterial>,
    pub stairs: Handle<StandardMaterial>,
    pub water: Handle<StandardMaterial>,
    pub lava: Handle<StandardMaterial>,
    pub stone: Handle<StandardMaterial>,
    pub tree: Handle<StandardMaterial>,
    pub fountain: Handle<StandardMaterial>,
    pub ice: Handle<StandardMaterial>,
}

fn setup_tile_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create meshes
    let floor_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(0.5)));
    let wall_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    commands.insert_resource(TileMeshes {
        floor: floor_mesh,
        wall: wall_mesh,
    });

    // Create materials with distinct colors
    commands.insert_resource(TileMaterials {
        floor: materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.5, 0.4), // Tan floor
            perceptual_roughness: 0.9,
            ..default()
        }),
        corridor: materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.4, 0.4), // Gray corridor
            perceptual_roughness: 0.9,
            ..default()
        }),
        wall: materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5), // Gray walls
            perceptual_roughness: 0.8,
            ..default()
        }),
        door: materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.3, 0.15), // Brown door
            perceptual_roughness: 0.7,
            ..default()
        }),
        stairs: materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.7, 0.7), // Light gray stairs
            perceptual_roughness: 0.6,
            ..default()
        }),
        water: materials.add(StandardMaterial {
            base_color: Color::srgba(0.2, 0.4, 0.8, 0.7), // Blue water
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.1,
            ..default()
        }),
        lava: materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.4, 0.1), // Orange lava
            emissive: LinearRgba::new(1.0, 0.3, 0.0, 1.0),
            perceptual_roughness: 0.3,
            ..default()
        }),
        stone: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.2), // Dark stone
            perceptual_roughness: 1.0,
            ..default()
        }),
        tree: materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.5, 0.2), // Green tree
            perceptual_roughness: 0.9,
            ..default()
        }),
        fountain: materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.6, 0.8), // Light blue fountain
            perceptual_roughness: 0.3,
            ..default()
        }),
        ice: materials.add(StandardMaterial {
            base_color: Color::srgba(0.8, 0.9, 1.0, 0.8), // White-blue ice
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.1,
            ..default()
        }),
    });
}

pub fn spawn_map(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    tile_meshes: Res<TileMeshes>,
    tile_materials: Res<TileMaterials>,
) {
    let level = &game_state.0.current_level;

    // Spawn tiles for entire map
    for y in 0..nh_core::ROWNO {
        for x in 0..nh_core::COLNO {
            let cell = level.cell(x, y);
            let map_pos = MapPosition {
                x: x as i8,
                y: y as i8,
            };

            spawn_tile(&mut commands, cell, map_pos, &tile_meshes, &tile_materials);
        }
    }

    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
    });

    // Add directional light for shadows
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(40.0, 50.0, 10.0).looking_at(Vec3::new(40.0, 0.0, 10.0), Vec3::Y),
    ));
}

fn spawn_tile(
    commands: &mut Commands,
    cell: &nh_core::dungeon::Cell,
    map_pos: MapPosition,
    meshes: &TileMeshes,
    materials: &TileMaterials,
) {
    use nh_core::dungeon::CellType;

    let world_pos = map_pos.to_world();

    match cell.typ {
        // Floor types - flat plane at y=0
        CellType::Room => {
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(materials.floor.clone()),
                Transform::from_translation(world_pos),
            ));
        }
        CellType::Corridor => {
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(materials.corridor.clone()),
                Transform::from_translation(world_pos),
            ));
        }

        // Wall types - cube at y=0.5
        CellType::VWall
        | CellType::HWall
        | CellType::TLCorner
        | CellType::TRCorner
        | CellType::BLCorner
        | CellType::BRCorner
        | CellType::CrossWall
        | CellType::TUWall
        | CellType::TDWall
        | CellType::TLWall
        | CellType::TRWall
        | CellType::DBWall => {
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.wall.clone()),
                MeshMaterial3d(materials.wall.clone()),
                Transform::from_translation(world_pos + Vec3::Y * 0.5),
            ));
        }

        // Door - variable height based on state
        CellType::Door => {
            let door_state = cell.door_state();
            let is_open = door_state.contains(nh_core::dungeon::DoorState::OPEN);
            let height = if is_open {
                DOOR_OPEN_HEIGHT
            } else {
                DOOR_CLOSED_HEIGHT
            };

            // Floor under door
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(materials.corridor.clone()),
                Transform::from_translation(world_pos),
            ));

            // Door itself
            commands.spawn((
                TileMarker,
                DoorMarker {
                    x: map_pos.x,
                    y: map_pos.y,
                    is_open,
                },
                map_pos,
                Mesh3d(meshes.wall.clone()),
                MeshMaterial3d(materials.door.clone()),
                Transform::from_translation(world_pos + Vec3::Y * height * 0.5)
                    .with_scale(Vec3::new(1.0, height, 1.0)),
            ));
        }

        // Stairs
        CellType::Stairs | CellType::Ladder => {
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(materials.stairs.clone()),
                Transform::from_translation(world_pos),
            ));
        }

        // Liquids - plane below floor level
        CellType::Pool | CellType::Moat | CellType::Water => {
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(materials.water.clone()),
                Transform::from_translation(world_pos - Vec3::Y * 0.3),
            ));
        }

        CellType::Lava => {
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(materials.lava.clone()),
                Transform::from_translation(world_pos - Vec3::Y * 0.2),
            ));
        }

        // Special terrain
        CellType::Fountain => {
            // Floor
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(materials.floor.clone()),
                Transform::from_translation(world_pos),
            ));
            // Fountain pedestal
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.wall.clone()),
                MeshMaterial3d(materials.fountain.clone()),
                Transform::from_translation(world_pos + Vec3::Y * 0.15)
                    .with_scale(Vec3::new(0.4, 0.3, 0.4)),
            ));
        }

        CellType::Throne | CellType::Altar | CellType::Grave | CellType::Sink => {
            // Floor with special feature (simplified as smaller cube)
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(materials.floor.clone()),
                Transform::from_translation(world_pos),
            ));
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.wall.clone()),
                MeshMaterial3d(materials.stone.clone()),
                Transform::from_translation(world_pos + Vec3::Y * 0.15)
                    .with_scale(Vec3::new(0.5, 0.3, 0.5)),
            ));
        }

        CellType::Tree => {
            // Floor
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(materials.floor.clone()),
                Transform::from_translation(world_pos),
            ));
            // Tree as tall cube
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.wall.clone()),
                MeshMaterial3d(materials.tree.clone()),
                Transform::from_translation(world_pos + Vec3::Y * 0.75)
                    .with_scale(Vec3::new(0.6, 1.5, 0.6)),
            ));
        }

        CellType::Ice => {
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(materials.ice.clone()),
                Transform::from_translation(world_pos),
            ));
        }

        CellType::IronBars => {
            // Floor with bars (simplified as grid-like cube)
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(materials.corridor.clone()),
                Transform::from_translation(world_pos),
            ));
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.wall.clone()),
                MeshMaterial3d(materials.stone.clone()),
                Transform::from_translation(world_pos + Vec3::Y * 0.5)
                    .with_scale(Vec3::new(0.1, 1.0, 1.0)),
            ));
        }

        CellType::DrawbridgeDown => {
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(materials.door.clone()),
                Transform::from_translation(world_pos),
            ));
        }

        // Stone/unexplored - dark cube
        CellType::Stone => {
            // Only render stone if explored, otherwise leave empty for fog of war
            if cell.explored {
                commands.spawn((
                    TileMarker,
                    map_pos,
                    Mesh3d(meshes.wall.clone()),
                    MeshMaterial3d(materials.stone.clone()),
                    Transform::from_translation(world_pos + Vec3::Y * 0.5),
                ));
            }
        }

        // Secret door/corridor - looks like wall
        CellType::SecretDoor | CellType::SecretCorridor => {
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.wall.clone()),
                MeshMaterial3d(materials.wall.clone()),
                Transform::from_translation(world_pos + Vec3::Y * 0.5),
            ));
        }

        // Air/Cloud/DrawbridgeUp - no geometry
        CellType::Air | CellType::Cloud | CellType::DrawbridgeUp => {
            // Empty space - no geometry
        }
    }
}

/// Sync door states with game state and trigger animations
fn sync_door_states(
    game_state: Res<GameStateResource>,
    mut commands: Commands,
    mut door_query: Query<(Entity, &mut DoorMarker, &Transform), Without<DoorAnimation>>,
) {
    if !game_state.is_changed() {
        return;
    }

    let level = &game_state.0.current_level;

    for (entity, mut door, transform) in door_query.iter_mut() {
        let cell = level.cell(door.x as usize, door.y as usize);
        let door_state = cell.door_state();
        let is_now_open = door_state.contains(nh_core::dungeon::DoorState::OPEN);

        if is_now_open != door.is_open {
            // Door state changed - trigger animation
            let current_height = transform.scale.y;
            let target_height = if is_now_open {
                DOOR_OPEN_HEIGHT
            } else {
                DOOR_CLOSED_HEIGHT
            };

            commands.entity(entity).insert(DoorAnimation {
                timer: Timer::from_seconds(DOOR_ANIMATION_DURATION, TimerMode::Once),
                start_height: current_height,
                target_height,
            });

            door.is_open = is_now_open;
        }
    }
}

/// Animate doors opening/closing
fn animate_doors(
    time: Res<Time>,
    mut commands: Commands,
    mut door_query: Query<(Entity, &mut Transform, &MapPosition, &mut DoorAnimation)>,
) {
    for (entity, mut transform, map_pos, mut anim) in door_query.iter_mut() {
        anim.timer.tick(time.delta());

        let t = anim.timer.fraction();
        // Smooth ease-out interpolation
        let t = 1.0 - (1.0 - t).powi(2);

        let height = anim.start_height + (anim.target_height - anim.start_height) * t;
        transform.scale.y = height;

        // Adjust Y position to keep door bottom on ground
        let world_pos = map_pos.to_world();
        transform.translation.y = world_pos.y + height * 0.5;

        if anim.timer.finished() {
            commands.entity(entity).remove::<DoorAnimation>();
        }
    }
}
