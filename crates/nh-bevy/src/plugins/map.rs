//! Map rendering plugin - converts 80x21 grid to 3D geometry

use bevy::prelude::*;

use nh_core::data::tile::DungeonTile;
use crate::components::{DoorAnimation, DoorMarker, MapPosition, TileMarker, TileMaterialType};
use crate::plugins::game::AppState;
use crate::resources::GameStateResource;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapState>()
            .insert_resource(TextureVariants::new())
            .add_systems(Startup, (setup_tile_assets, spawn_map).chain())
            .add_systems(OnEnter(AppState::Playing), mark_map_for_respawn)
            .add_systems(
                Update,
                (
                    check_level_change,
                    sync_door_states,
                    animate_doors,
                    animate_liquids,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Resource, Default)]
struct MapState {
    current_dlevel: Option<nh_core::dungeon::DLevel>,
    room_change_count: usize,
    /// Set when entering Playing state to force a full map respawn
    needs_respawn: bool,
}

/// Tracks texture variants for each tile type
#[derive(Resource, Default)]
struct TextureVariants {
    /// Map from texture name to (current_index, max_count)
    variants: std::collections::HashMap<String, (usize, usize)>,
}

impl TextureVariants {
    /// Count how many variants exist for a texture (e.g., room-1.jpeg, room-2.jpeg, ...)
    fn count_variants(name: &str) -> usize {
        let mut count = 0;
        for i in 1.. {
            let path = format!("crates/nh-bevy/assets/textures/{}-{}.jpeg", name, i);
            if std::path::Path::new(&path).exists() {
                count = i;
            } else {
                break;
            }
        }
        count
    }

    /// Initialize variant counts for all texture types
    fn new() -> Self {
        let texture_names = [
            "floor", "corridor", "wall", "door", "stairs", "water", "lava", "stone", "tree",
            "fountain", "ice", "room",
        ];

        let mut variants = std::collections::HashMap::new();
        for name in texture_names {
            let count = Self::count_variants(name);
            if count > 0 {
                variants.insert(name.to_string(), (1, count)); // Start at variant 1
            }
        }

        Self { variants }
    }

    /// Get current texture path for a name, or None if no variants exist
    fn get_texture_path(&self, name: &str) -> Option<String> {
        self.variants
            .get(name)
            .map(|(idx, _)| format!("textures/{}-{}.jpeg", name, idx))
    }

    /// Advance to next variant for all textures (wraps around)
    fn advance_all(&mut self) {
        for (_, (idx, max)) in self.variants.iter_mut() {
            *idx = (*idx % *max) + 1;
        }
    }
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

/// Mark map for respawn when entering Playing state (e.g., after character creation)
fn mark_map_for_respawn(mut map_state: ResMut<MapState>) {
    map_state.needs_respawn = true;
}

/// Check if level changed (or respawn forced) and respawn map
fn check_level_change(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    mut map_state: ResMut<MapState>,
    tile_meshes: Res<TileMeshes>,
    tile_materials: Res<TileMaterials>,
    map_query: Query<Entity, With<TileMarker>>,
    mut texture_variants: ResMut<TextureVariants>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    if !game_state.is_changed() && !map_state.needs_respawn {
        return;
    }

    let current_dlevel = game_state.0.current_level.dlevel;
    let force_respawn = map_state.needs_respawn;

    // Initialize on first run (but not if we're forcing a respawn)
    if map_state.current_dlevel.is_none() && !force_respawn {
        map_state.current_dlevel = Some(current_dlevel);
        return;
    }

    if map_state.current_dlevel != Some(current_dlevel) || force_respawn {
        info!(
            "Respawning map (level {:?} → {:?}, forced={})",
            map_state.current_dlevel, current_dlevel, force_respawn
        );

        // Advance texture variants for visual variety
        texture_variants.advance_all();
        map_state.room_change_count += 1;
        info!(
            "Texture variants advanced (room change #{})",
            map_state.room_change_count
        );

        // Update material textures with new variants
        update_material_textures(
            &tile_materials,
            &mut materials,
            &texture_variants,
            &asset_server,
        );

        // Despawn old map
        for entity in map_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Spawn new map
        spawn_map_internal(
            &mut commands,
            &game_state.0.current_level,
            &tile_meshes,
            &tile_materials,
        );

        // Update state
        map_state.current_dlevel = Some(current_dlevel);
        map_state.needs_respawn = false;
    }
}

/// Update material textures to use current variant
fn update_material_textures(
    tile_materials: &TileMaterials,
    materials: &mut Assets<StandardMaterial>,
    texture_variants: &TextureVariants,
    asset_server: &AssetServer,
) {
    let material_mappings: [(&Handle<StandardMaterial>, &str); 11] = [
        (&tile_materials.floor, "room"),
        (&tile_materials.corridor, "corridor"),
        (&tile_materials.wall, "wall"),
        (&tile_materials.door, "door"),
        (&tile_materials.stairs, "stairs"),
        (&tile_materials.water, "water"),
        (&tile_materials.lava, "lava"),
        (&tile_materials.stone, "stone"),
        (&tile_materials.tree, "tree"),
        (&tile_materials.fountain, "fountain"),
        (&tile_materials.ice, "ice"),
    ];

    for (handle, name) in material_mappings {
        if let Some(material) = materials.get_mut(handle) {
            material.base_color_texture = texture_variants
                .get_texture_path(name)
                .map(|path| asset_server.load(path));
        }
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
    // Unexplored variants (alpha 0.3 for semi-transparent fog of war)
    pub floor_unexplored: Handle<StandardMaterial>,
    pub corridor_unexplored: Handle<StandardMaterial>,
    pub wall_unexplored: Handle<StandardMaterial>,
    pub door_unexplored: Handle<StandardMaterial>,
    pub stairs_unexplored: Handle<StandardMaterial>,
    pub water_unexplored: Handle<StandardMaterial>,
    pub lava_unexplored: Handle<StandardMaterial>,
    pub stone_unexplored: Handle<StandardMaterial>,
    pub tree_unexplored: Handle<StandardMaterial>,
    pub fountain_unexplored: Handle<StandardMaterial>,
    pub ice_unexplored: Handle<StandardMaterial>,
}

fn setup_tile_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    texture_variants: Res<TextureVariants>,
) {
    // Create meshes
    let floor_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(0.5)));
    let wall_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    commands.insert_resource(TileMeshes {
        floor: floor_mesh,
        wall: wall_mesh,
    });

    // Helper to create material with optional texture using variant system
    let create_material = |materials: &mut Assets<StandardMaterial>,
                           name: &str,
                           color: Color,
                           roughness: f32,
                           emissive: Option<LinearRgba>|
     -> Handle<StandardMaterial> {
        // Try to get texture path from variants (e.g., "wall" -> "textures/wall-1.jpeg")
        let texture = texture_variants
            .get_texture_path(name)
            .map(|path| asset_server.load(path));

        materials.add(StandardMaterial {
            base_color: color,
            base_color_texture: texture,
            perceptual_roughness: roughness,
            emissive: emissive.unwrap_or(LinearRgba::BLACK),
            alpha_mode: if color.alpha() < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            ..default()
        })
    };

    // Helper to create unexplored material variant (alpha 0.3)
    let create_unexplored = |materials: &mut Assets<StandardMaterial>,
                             name: &str,
                             color: Color,
                             roughness: f32,
                             emissive: Option<LinearRgba>|
     -> Handle<StandardMaterial> {
        let texture = texture_variants
            .get_texture_path(name)
            .map(|path| asset_server.load(path));

        // Apply alpha 0.3 to the color
        let unexplored_color = color.with_alpha(0.3);

        materials.add(StandardMaterial {
            base_color: unexplored_color,
            base_color_texture: texture,
            perceptual_roughness: roughness,
            emissive: emissive.unwrap_or(LinearRgba::BLACK),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })
    };

    // Create materials with distinct colors/textures
    commands.insert_resource(TileMaterials {
        // Normal materials
        floor: create_material(
            &mut materials,
            "room",
            Color::srgb(0.6, 0.5, 0.4),
            0.9,
            None,
        ),
        corridor: create_material(
            &mut materials,
            "corridor",
            Color::srgb(0.4, 0.4, 0.4),
            0.9,
            None,
        ),
        wall: create_material(
            &mut materials,
            "wall",
            Color::srgb(0.5, 0.5, 0.5),
            0.8,
            None,
        ),
        door: create_material(
            &mut materials,
            "door",
            Color::srgb(0.5, 0.3, 0.15),
            0.7,
            None,
        ),
        stairs: create_material(
            &mut materials,
            "stairs",
            Color::srgb(0.7, 0.7, 0.7),
            0.6,
            None,
        ),
        water: create_material(
            &mut materials,
            "water",
            Color::srgba(0.2, 0.4, 0.8, 0.7),
            0.1,
            None,
        ),
        lava: create_material(
            &mut materials,
            "lava",
            Color::srgb(1.0, 0.4, 0.1),
            0.3,
            Some(LinearRgba::new(1.0, 0.3, 0.0, 1.0)),
        ),
        stone: create_material(
            &mut materials,
            "stone",
            Color::srgb(0.2, 0.2, 0.2),
            1.0,
            None,
        ),
        tree: create_material(
            &mut materials,
            "tree",
            Color::srgb(0.2, 0.5, 0.2),
            0.9,
            None,
        ),
        fountain: create_material(
            &mut materials,
            "fountain",
            Color::srgb(0.4, 0.6, 0.8),
            0.3,
            None,
        ),
        ice: create_material(
            &mut materials,
            "ice",
            Color::srgba(0.8, 0.9, 1.0, 0.8),
            0.1,
            None,
        ),
        // Unexplored variants (alpha 0.3)
        floor_unexplored: create_unexplored(
            &mut materials,
            "room",
            Color::srgb(0.6, 0.5, 0.4),
            0.9,
            None,
        ),
        corridor_unexplored: create_unexplored(
            &mut materials,
            "corridor",
            Color::srgb(0.4, 0.4, 0.4),
            0.9,
            None,
        ),
        wall_unexplored: create_unexplored(
            &mut materials,
            "wall",
            Color::srgb(0.5, 0.5, 0.5),
            0.8,
            None,
        ),
        door_unexplored: create_unexplored(
            &mut materials,
            "door",
            Color::srgb(0.5, 0.3, 0.15),
            0.7,
            None,
        ),
        stairs_unexplored: create_unexplored(
            &mut materials,
            "stairs",
            Color::srgb(0.7, 0.7, 0.7),
            0.6,
            None,
        ),
        water_unexplored: create_unexplored(
            &mut materials,
            "water",
            Color::srgba(0.2, 0.4, 0.8, 0.7),
            0.1,
            None,
        ),
        lava_unexplored: create_unexplored(
            &mut materials,
            "lava",
            Color::srgb(1.0, 0.4, 0.1),
            0.3,
            Some(LinearRgba::new(1.0, 0.3, 0.0, 1.0)),
        ),
        stone_unexplored: create_unexplored(
            &mut materials,
            "stone",
            Color::srgb(0.2, 0.2, 0.2),
            1.0,
            None,
        ),
        tree_unexplored: create_unexplored(
            &mut materials,
            "tree",
            Color::srgb(0.2, 0.5, 0.2),
            0.9,
            None,
        ),
        fountain_unexplored: create_unexplored(
            &mut materials,
            "fountain",
            Color::srgb(0.4, 0.6, 0.8),
            0.3,
            None,
        ),
        ice_unexplored: create_unexplored(
            &mut materials,
            "ice",
            Color::srgba(0.8, 0.9, 1.0, 0.8),
            0.1,
            None,
        ),
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

            spawn_tile(commands, cell, map_pos, tile_meshes, tile_materials, level);
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

impl From<DungeonTile> for TileMaterialType {
    fn from(dt: DungeonTile) -> Self {
        match dt {
            DungeonTile::Floor => TileMaterialType::Floor,
            DungeonTile::VerticalWall | DungeonTile::HorizontalWall | DungeonTile::Corner => {
                TileMaterialType::Wall
            }
            DungeonTile::DoorClosed | DungeonTile::DoorOpen => TileMaterialType::Door,
            DungeonTile::StairsUp | DungeonTile::StairsDown => TileMaterialType::Stairs,
            DungeonTile::Pool => TileMaterialType::Water,
            DungeonTile::Lava => TileMaterialType::Lava,
            DungeonTile::Stone => TileMaterialType::Stone,
        }
    }
}

fn spawn_tile(
    commands: &mut Commands,
    cell: &nh_core::dungeon::Cell,
    map_pos: MapPosition,
    meshes: &TileMeshes,
    materials: &TileMaterials,
    level: &nh_core::dungeon::Level,
) {
    let world_pos = map_pos.to_world();
    let explored = cell.explored;

    // Helper to pick normal or unexplored material
    let mat = |normal: &Handle<StandardMaterial>, unexplored: &Handle<StandardMaterial>| {
        if explored {
            normal.clone()
        } else {
            unexplored.clone()
        }
    };

    let dungeon_tile = DungeonTile::from(cell.typ);
    let mat_type = TileMaterialType::from(dungeon_tile);

    match dungeon_tile {
        DungeonTile::Floor => {
            commands.spawn((
                TileMarker,
                mat_type,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(mat(&materials.floor, &materials.floor_unexplored)),
                Transform::from_translation(world_pos),
            ));
        }
        DungeonTile::VerticalWall | DungeonTile::HorizontalWall | DungeonTile::Corner => {
            commands.spawn((
                TileMarker,
                mat_type,
                map_pos,
                Mesh3d(meshes.wall.clone()),
                MeshMaterial3d(mat(&materials.wall, &materials.wall_unexplored)),
                Transform::from_translation(world_pos + Vec3::Y * 0.5),
            ));
        }
        DungeonTile::DoorClosed | DungeonTile::DoorOpen => {
            let door_state = cell.door_state();
            let is_open = door_state.contains(nh_core::dungeon::DoorState::OPEN);

            // Determine door orientation from adjacent cells.
            // The door slab base shape is (0.2, 1.0, 1.0): thin in X, full in Z.
            //
            // Door in vertical wall (|+|): passage goes E-W (world X).
            //   Door must block X → thin in X, fill Z → IDENTITY
            //   Neighbors: N/S are walls, E/W are walkable.
            //
            // Door in horizontal wall (---+---): passage goes N-S (world Z).
            //   Door must block Z → thin in Z, fill X → PI/2
            //   Neighbors: E/W are walls, N/S are walkable.
            let x = map_pos.x as usize;
            let y = map_pos.y as usize;
            let ew_walkable = (x > 0 && !level.cell(x - 1, y).blocks_sight())
                || (x + 1 < nh_core::COLNO && !level.cell(x + 1, y).blocks_sight());
            let ns_walkable = (y > 0 && !level.cell(x, y - 1).blocks_sight())
                || (y + 1 < nh_core::ROWNO && !level.cell(x, y + 1).blocks_sight());

            let closed_rotation = if ew_walkable && !ns_walkable {
                // Vertical wall door: passage goes E-W, thin in X
                Quat::IDENTITY
            } else if ns_walkable && !ew_walkable {
                // Horizontal wall door: passage goes N-S, thin in Z
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
            } else {
                // Ambiguous — default IDENTITY
                Quat::IDENTITY
            };

            let open_offset = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
            let rotation = if is_open {
                closed_rotation * open_offset
            } else {
                closed_rotation
            };

            // Floor under door
            commands.spawn((
                TileMarker,
                TileMaterialType::Corridor,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(mat(&materials.corridor, &materials.corridor_unexplored)),
                Transform::from_translation(world_pos),
            ));

            // Door itself - thin slab
            commands.spawn((
                TileMarker,
                mat_type,
                DoorMarker {
                    x: map_pos.x,
                    y: map_pos.y,
                    is_open,
                    closed_rotation,
                },
                map_pos,
                Mesh3d(meshes.wall.clone()),
                MeshMaterial3d(mat(&materials.door, &materials.door_unexplored)),
                Transform::from_translation(world_pos + Vec3::Y * 0.5)
                    .with_rotation(rotation)
                    .with_scale(Vec3::new(0.2, 1.0, 1.0)),
            ));
        }
        DungeonTile::StairsUp | DungeonTile::StairsDown => {
            commands.spawn((
                TileMarker,
                mat_type,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(mat(&materials.stairs, &materials.stairs_unexplored)),
                Transform::from_translation(world_pos),
            ));
        }
        DungeonTile::Pool => {
            commands.spawn((
                TileMarker,
                mat_type,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(mat(&materials.water, &materials.water_unexplored)),
                Transform::from_translation(world_pos - Vec3::Y * 0.3),
            ));
        }
        DungeonTile::Lava => {
            commands.spawn((
                TileMarker,
                mat_type,
                map_pos,
                Mesh3d(meshes.floor.clone()),
                MeshMaterial3d(mat(&materials.lava, &materials.lava_unexplored)),
                Transform::from_translation(world_pos - Vec3::Y * 0.2),
            ));
        }
        DungeonTile::Stone => {
            commands.spawn((
                TileMarker,
                mat_type,
                map_pos,
                Mesh3d(meshes.wall.clone()),
                MeshMaterial3d(mat(&materials.stone, &materials.stone_unexplored)),
                Transform::from_translation(world_pos + Vec3::Y * 0.5),
            ));
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
            // Use the door's stored closed_rotation as base, apply PI/2 offset for open
            let current_rotation = transform.rotation;
            let open_offset = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
            let target_rotation = if is_now_open {
                door.closed_rotation * open_offset
            } else {
                door.closed_rotation
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
