//! Fog of War system
//!
//! Syncs visibility from nh-core's authoritative state (Level::is_visible/is_explored)
//! and applies fog of war rendering:
//! - Explored but not visible cells shown dimmed
//! - Unexplored cells hidden
//! - Monsters/objects only shown when currently visible

use bevy::prelude::*;

use crate::components::{MapPosition, MonsterMarker, PlayerMarker, TileMarker, TileMaterialType};
use crate::plugins::entities::{FloorObjectMarker, PileMarker};
use crate::plugins::game::AppState;
use crate::plugins::map::TileMaterials;
use crate::resources::GameStateResource;

pub struct FogOfWarPlugin;

impl Plugin for FogOfWarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VisibilityMap>()
            .init_resource::<FogSettings>()
            .add_systems(
                Update,
                (
                    sync_visibility_from_core,
                    apply_fog_to_tiles,
                    apply_fog_to_entities,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// Settings for fog of war
#[derive(Resource)]
pub struct FogSettings {
    /// Whether fog of war is enabled
    pub enabled: bool,
}

impl Default for FogSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Tracks visibility state for each cell, synced from nh-core
#[derive(Resource, Default)]
pub struct VisibilityMap {
    /// Currently visible cells (synced from Level each frame)
    pub visible: Vec<Vec<bool>>,
    /// Explored cells (synced from Level each frame)
    pub explored: Vec<Vec<bool>>,
    /// Whether the map has been initialized
    pub initialized: bool,
}

impl VisibilityMap {
    /// Check if a cell is currently visible
    pub fn is_visible(&self, x: usize, y: usize) -> bool {
        if !self.initialized {
            return true; // Show everything if not initialized
        }
        self.visible
            .get(y)
            .and_then(|row| row.get(x))
            .copied()
            .unwrap_or(false)
    }

    /// Check if a cell has been explored (ever visible)
    pub fn is_explored(&self, x: usize, y: usize) -> bool {
        if !self.initialized {
            return false;
        }
        self.explored
            .get(y)
            .and_then(|row| row.get(x))
            .copied()
            .unwrap_or(false)
    }

    /// Initialize the visibility map
    pub fn init(&mut self, width: usize, height: usize) {
        self.visible = vec![vec![false; width]; height];
        self.explored = vec![vec![false; width]; height];
        self.initialized = true;
    }
}

/// Sync visibility directly from nh-core's authoritative Level state.
/// This matches the TUI behavior exactly — both read from Level::is_visible/is_explored.
fn sync_visibility_from_core(
    mut visibility: ResMut<VisibilityMap>,
    game_state: Res<GameStateResource>,
    settings: Res<FogSettings>,
) {
    if !settings.enabled {
        return;
    }

    let level = &game_state.0.current_level;

    // Initialize visibility map if needed
    if !visibility.initialized {
        visibility.init(nh_core::COLNO, nh_core::ROWNO);
    }

    // Read visibility and explored state directly from nh-core
    // This is the same data the TUI reads via level.is_visible()/is_explored()
    for y in 0..nh_core::ROWNO {
        for x in 0..nh_core::COLNO {
            visibility.visible[y][x] = level.is_visible(x as i8, y as i8);
            visibility.explored[y][x] = level.is_explored(x as i8, y as i8);
        }
    }
}

/// Apply fog of war effect to tile entities
/// Swaps between normal and unexplored (semi-transparent) materials based on explored state
fn apply_fog_to_tiles(
    visibility: Res<VisibilityMap>,
    settings: Res<FogSettings>,
    tile_materials: Res<TileMaterials>,
    mut tile_query: Query<
        (
            &MapPosition,
            &TileMaterialType,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        With<TileMarker>,
    >,
) {
    if !settings.enabled || !visibility.initialized {
        return;
    }

    for (pos, mat_type, mut material) in tile_query.iter_mut() {
        let x = pos.x as usize;
        let y = pos.y as usize;

        // Get the appropriate material based on explored state
        let (normal, unexplored) = match mat_type {
            TileMaterialType::Floor => (&tile_materials.floor, &tile_materials.floor_unexplored),
            TileMaterialType::Corridor => (
                &tile_materials.corridor,
                &tile_materials.corridor_unexplored,
            ),
            TileMaterialType::Wall => (&tile_materials.wall, &tile_materials.wall_unexplored),
            TileMaterialType::Door => (&tile_materials.door, &tile_materials.door_unexplored),
            TileMaterialType::Stairs => (&tile_materials.stairs, &tile_materials.stairs_unexplored),
            TileMaterialType::Water => (&tile_materials.water, &tile_materials.water_unexplored),
            TileMaterialType::Lava => (&tile_materials.lava, &tile_materials.lava_unexplored),
            TileMaterialType::Stone => (&tile_materials.stone, &tile_materials.stone_unexplored),
            TileMaterialType::Tree => (&tile_materials.tree, &tile_materials.tree_unexplored),
            TileMaterialType::Fountain => (
                &tile_materials.fountain,
                &tile_materials.fountain_unexplored,
            ),
            TileMaterialType::Ice => (&tile_materials.ice, &tile_materials.ice_unexplored),
        };

        if visibility.is_explored(x, y) {
            // Explored - use normal material
            if material.0 != *normal {
                material.0 = normal.clone();
            }
        } else {
            // Not explored - use semi-transparent unexplored material
            if material.0 != *unexplored {
                material.0 = unexplored.clone();
            }
        }
    }
}

/// Apply fog of war to entities (monsters, items)
fn apply_fog_to_entities(
    visibility: Res<VisibilityMap>,
    settings: Res<FogSettings>,
    mut monster_query: Query<
        (&MapPosition, &mut Visibility),
        (With<MonsterMarker>, Without<PlayerMarker>),
    >,
    mut object_query: Query<
        (&MapPosition, &mut Visibility),
        (With<FloorObjectMarker>, Without<MonsterMarker>),
    >,
    mut pile_query: Query<
        (&MapPosition, &mut Visibility),
        (
            With<PileMarker>,
            Without<FloorObjectMarker>,
            Without<MonsterMarker>,
        ),
    >,
) {
    if !settings.enabled || !visibility.initialized {
        return;
    }

    // Monsters only visible when in line of sight
    for (pos, mut vis) in monster_query.iter_mut() {
        let x = pos.x as usize;
        let y = pos.y as usize;
        if visibility.is_visible(x, y) {
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    // Floor objects visible if currently visible
    for (pos, mut vis) in object_query.iter_mut() {
        let x = pos.x as usize;
        let y = pos.y as usize;
        if visibility.is_visible(x, y) {
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    // Piles visible if currently visible
    for (pos, mut vis) in pile_query.iter_mut() {
        let x = pos.x as usize;
        let y = pos.y as usize;
        if visibility.is_visible(x, y) {
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}
