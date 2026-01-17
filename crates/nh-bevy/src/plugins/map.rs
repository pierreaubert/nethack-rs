//! Map rendering plugin - converts 80x21 grid to 3D geometry

use bevy::prelude::*;

use crate::components::{DoorAnimation, DoorMarker, MapPosition, TileMarker};
use crate::resources::GameStateResource;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapState>()
            .add_systems(Startup, (setup_tile_assets, spawn_map).chain())
            .add_systems(
                Update,
                (
                    sync_door_states,
                    animate_doors,
                    check_level_change,
                    animate_liquids,
                ),
            );
    }
}

#[derive(Resource, Default)]
struct MapState {
    current_dlevel: Option<nh_core::dungeon::DLevel>,
}

/// Animate water and lava materials
fn animate_liquids(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    tile_materials: Res<TileMaterials>,
) {
    let t = time.elapsed_secs();

    // Animate Water
    if let Some(water) = materials.get_mut(&tile_materials.water) {
        let wave = (t * 1.5).sin() * 0.1;
        // Base alpha 0.7
        water.base_color.set_alpha((0.7 + wave).clamp(0.0, 1.0));
    }

    // Animate Lava
    if let Some(lava) = materials.get_mut(&tile_materials.lava) {
        let pulse = (t * 2.0).sin() * 0.5 + 0.5; // 0..1
        let intensity = 1.0 + pulse * 2.0; // 1..3
        lava.emissive = LinearRgba::new(1.0, 0.3 * intensity, 0.0, 1.0);
    }
}

/// Check if level changed and respawn map
fn check_level_change(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    mut map_state: ResMut<MapState>,
    tile_meshes: Res<TileMeshes>,
    tile_materials: Res<TileMaterials>,
    map_query: Query<Entity, With<TileMarker>>,
) {
    if !game_state.is_changed() {
        return;
    }

    let current_dlevel = game_state.0.current_level.dlevel;

    // Initialize on first run
    if map_state.current_dlevel.is_none() {
        map_state.current_dlevel = Some(current_dlevel);
        return;
    }

    if map_state.current_dlevel != Some(current_dlevel) {
        info!("Level changed from {:?} to {:?}", map_state.current_dlevel, current_dlevel);
        
        // Despawn old map
        for entity in map_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Spawn new map
        spawn_map_internal(&mut commands, &game_state.0.current_level, &tile_meshes, &tile_materials);
        
        // Update state
        map_state.current_dlevel = Some(current_dlevel);
    }
}

/// Door height constants
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
    asset_server: Res<AssetServer>,
) {
    // Create meshes
    let floor_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(0.5)));
    let wall_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    commands.insert_resource(TileMeshes {
        floor: floor_mesh,
        wall: wall_mesh,
    });

    // Helper to create material with optional texture
    let mut create_material = |path: &str, color: Color, roughness: f32, emissive: Option<LinearRgba>| -> Handle<StandardMaterial> {
        // Check if texture exists relative to assets/
        let texture_path = format!("textures/{}.png", path);
        // We can't synchronously check file existence for assets in Bevy generally, 
        // but we can try to load it. If it's missing, it will just show black/pink or fail silently depending on config.
        // To be safe and keep the colors as fallback, we can check file system (assuming local).
        
        let fs_path = std::path::Path::new("crates/nh-bevy/assets").join(&texture_path);
        let texture = if fs_path.exists() {
            Some(asset_server.load(texture_path))
        } else {
            None
        };

        materials.add(StandardMaterial {
            base_color: color,
            base_color_texture: texture,
            perceptual_roughness: roughness,
            emissive: emissive.unwrap_or(LinearRgba::BLACK),
            alpha_mode: if color.alpha() < 1.0 { AlphaMode::Blend } else { AlphaMode::Opaque },
            ..default()
        })
    };

    // Create materials with distinct colors/textures
    commands.insert_resource(TileMaterials {
        floor: create_material("floor", Color::srgb(0.6, 0.5, 0.4), 0.9, None),
        corridor: create_material("corridor", Color::srgb(0.4, 0.4, 0.4), 0.9, None),
        wall: create_material("wall", Color::srgb(0.5, 0.5, 0.5), 0.8, None),
        door: create_material("door", Color::srgb(0.5, 0.3, 0.15), 0.7, None),
        stairs: create_material("stairs", Color::srgb(0.7, 0.7, 0.7), 0.6, None),
        water: create_material("water", Color::srgba(0.2, 0.4, 0.8, 0.7), 0.1, None),
        lava: create_material("lava", Color::srgb(1.0, 0.4, 0.1), 0.3, Some(LinearRgba::new(1.0, 0.3, 0.0, 1.0))),
        stone: create_material("stone", Color::srgb(0.2, 0.2, 0.2), 1.0, None),
        tree: create_material("tree", Color::srgb(0.2, 0.5, 0.2), 0.9, None),
        fountain: create_material("fountain", Color::srgb(0.4, 0.6, 0.8), 0.3, None),
        ice: create_material("ice", Color::srgba(0.8, 0.9, 1.0, 0.8), 0.1, None),
    });
}

pub fn spawn_map(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    tile_meshes: Res<TileMeshes>,
    tile_materials: Res<TileMaterials>,
) {
    let level = &game_state.0.current_level;
    spawn_map_internal(&mut commands, level, &tile_meshes, &tile_materials);
}

fn spawn_map_internal(
    commands: &mut Commands,
    level: &nh_core::dungeon::Level,
    tile_meshes: &TileMeshes,
    tile_materials: &TileMaterials,
) {
    // Spawn tiles for entire map
    for y in 0..nh_core::ROWNO {
        for x in 0..nh_core::COLNO {
            let cell = level.cell(x, y);
            let map_pos = MapPosition {
                x: x as i8,
                y: y as i8,
            };

            spawn_tile(commands, cell, map_pos, tile_meshes, tile_materials);
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

        // Door - rotates based on state
        CellType::Door => {
            let door_state = cell.door_state();
            let is_open = door_state.contains(nh_core::dungeon::DoorState::OPEN);
            
            // Determine orientation: check if horizontal walls are adjacent
            // Note: We don't have easy access to neighbors here without passing level ref
            // For now, assume East-West if x is odd (checkerboard) as a hack, 
            // or we could check neighbors if we passed 'level' to spawn_tile.
            // Let's assume North-South (Z-axis aligned) by default, rotated 90 deg if East-West.
            
            // Better: defaulting to Z-axis aligned (North-South). 
            // If it's open, it rotates 90 degrees relative to its frame.
            
            let base_rotation = Quat::IDENTITY; // Aligned with Z axis (North-South)
            let open_rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
            
            let rotation = if is_open { open_rotation } else { base_rotation };

            // Floor under door
            commands.spawn((
                TileMarker,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(materials.corridor.clone()),
                Transform::from_translation(world_pos),
            ));

            // Door itself - thin slab
            commands.spawn((
                TileMarker,
                DoorMarker {
                    x: map_pos.x,
                    y: map_pos.y,
                    is_open,
                },
                map_pos,
                Mesh3d(meshes.wall.clone()), // Reusing wall mesh (cube)
                MeshMaterial3d(materials.door.clone()),
                Transform::from_translation(world_pos + Vec3::Y * 0.5)
                    .with_rotation(rotation)
                    .with_scale(Vec3::new(0.2, 1.0, 1.0)), // Thin door
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
            let current_rotation = transform.rotation;
            let target_rotation = if is_now_open {
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
            } else {
                Quat::IDENTITY
            };

            commands.entity(entity).insert(DoorAnimation {
                timer: Timer::from_seconds(DOOR_ANIMATION_DURATION, TimerMode::Once),
                start_rotation: current_rotation,
                target_rotation,
            });

            door.is_open = is_now_open;
        }
    }
}

/// Animate doors opening/closing
fn animate_doors(
    time: Res<Time>,
    mut commands: Commands,
    mut door_query: Query<(Entity, &mut Transform, &mut DoorAnimation)>,
) {
    for (entity, mut transform, mut anim) in door_query.iter_mut() {
        anim.timer.tick(time.delta());

        let t = anim.timer.fraction();
        // Smooth ease-out interpolation
        let t = 1.0 - (1.0 - t).powi(2);

        transform.rotation = anim.start_rotation.slerp(anim.target_rotation, t);

        if anim.timer.finished() {
            commands.entity(entity).remove::<DoorAnimation>();
        }
    }
}
