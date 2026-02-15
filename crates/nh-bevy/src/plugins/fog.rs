//! Fog of War system
//!
//! Implements visibility calculations and fog of war rendering:
//! - Line-of-sight from player position
//! - Explored vs visible cell tracking
//! - Dimming of explored but not visible cells
//! - Hiding of unexplored cells

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
                    calculate_visibility,
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
    /// Base visibility radius in dark areas
    pub dark_vision_radius: i32,
    /// Visibility radius in lit areas
    pub lit_vision_radius: i32,
    /// Whether fog of war is enabled
    pub enabled: bool,
    /// Brightness multiplier for visible tiles (1.0 = normal)
    pub visible_brightness: f32,
    /// Brightness multiplier for explored but not visible tiles
    pub explored_brightness: f32,
}

impl Default for FogSettings {
    fn default() -> Self {
        Self {
            dark_vision_radius: 1,
            lit_vision_radius: 15,
            enabled: true,
            visible_brightness: 1.0,
            explored_brightness: 0.3,
        }
    }
}

/// Tracks visibility state for each cell
#[derive(Resource, Default)]
pub struct VisibilityMap {
    /// Currently visible cells (calculated each frame)
    pub visible: Vec<Vec<bool>>,
    /// Explored cells (monotonically increasing, synced from GameState + fog calculations)
    pub explored: Vec<Vec<bool>>,
    /// Player position for visibility calculations
    pub player_pos: (i8, i8),
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

    /// Clear all visibility
    pub fn clear(&mut self) {
        for row in &mut self.visible {
            for cell in row {
                *cell = false;
            }
        }
    }

    /// Mark a cell as visible (also marks as explored)
    pub fn set_visible(&mut self, x: usize, y: usize) {
        if let Some(cell) = self.visible.get_mut(y).and_then(|row| row.get_mut(x)) {
            *cell = true;
        }
        if let Some(cell) = self.explored.get_mut(y).and_then(|row| row.get_mut(x)) {
            *cell = true;
        }
    }
}

/// Calculate visibility from player position
fn calculate_visibility(
    mut visibility: ResMut<VisibilityMap>,
    game_state: Res<GameStateResource>,
    settings: Res<FogSettings>,
) {
    if !settings.enabled {
        return;
    }

    let state = &game_state.0;
    let level = &state.current_level;
    let player_x = state.player.pos.x as usize;
    let player_y = state.player.pos.y as usize;

    // Initialize visibility map if needed
    if !visibility.initialized {
        visibility.init(nh_core::COLNO, nh_core::ROWNO);
    }

    // Sync explored state from GameState when it changes (handles level transitions too)
    if game_state.is_changed() {
        for y in 0..nh_core::ROWNO {
            for x in 0..nh_core::COLNO {
                visibility.explored[y][x] = level.cells[x][y].explored;
            }
        }
    }

    // Clear previous visibility (explored is persistent, visible is per-frame)
    visibility.clear();
    visibility.player_pos = (state.player.pos.x, state.player.pos.y);

    // Player's current cell is always visible
    visibility.set_visible(player_x, player_y);

    // Check if player is in a lit room
    let player_cell = level.cell(player_x, player_y);
    let in_lit_room = player_cell.lit && player_cell.room_number > 0;

    // If in a lit room, the entire room is visible
    if in_lit_room {
        let room_num = player_cell.room_number;
        for y in 0..nh_core::ROWNO {
            for x in 0..nh_core::COLNO {
                let cell = level.cell(x, y);
                if cell.room_number == room_num || is_room_adjacent(level, x, y, room_num) {
                    visibility.set_visible(x, y);
                }
            }
        }
    }

    // Calculate line-of-sight visibility
    let max_radius = if in_lit_room {
        settings.lit_vision_radius
    } else {
        settings.dark_vision_radius
    };

    // Cast rays in all directions
    for angle in 0..360 {
        let rad = (angle as f32).to_radians();
        let dx = rad.cos();
        let dy = rad.sin();

        cast_ray(
            &mut visibility,
            level,
            player_x as f32,
            player_y as f32,
            dx,
            dy,
            max_radius as f32,
        );
    }

    // Also check immediate adjacent cells (always visible)
    for dy in -1..=1 {
        for dx in -1..=1 {
            let nx = player_x as i32 + dx;
            let ny = player_y as i32 + dy;
            if nx >= 0 && nx < nh_core::COLNO as i32 && ny >= 0 && ny < nh_core::ROWNO as i32 {
                visibility.set_visible(nx as usize, ny as usize);
            }
        }
    }
}

/// Check if a cell is adjacent to a specific room
fn is_room_adjacent(level: &nh_core::dungeon::Level, x: usize, y: usize, room_num: u8) -> bool {
    for dy in -1..=1i32 {
        for dx in -1..=1i32 {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0
                && nx < nh_core::COLNO as i32
                && ny >= 0
                && ny < nh_core::ROWNO as i32
                && level.cell(nx as usize, ny as usize).room_number == room_num
            {
                return true;
            }
        }
    }
    false
}

/// Cast a ray for visibility calculation
fn cast_ray(
    visibility: &mut VisibilityMap,
    level: &nh_core::dungeon::Level,
    start_x: f32,
    start_y: f32,
    dx: f32,
    dy: f32,
    max_distance: f32,
) {
    let mut x = start_x;
    let mut y = start_y;
    let mut distance = 0.0;

    while distance < max_distance {
        x += dx * 0.5;
        y += dy * 0.5;
        distance += 0.5;

        let ix = x.round() as i32;
        let iy = y.round() as i32;

        if ix < 0 || ix >= nh_core::COLNO as i32 || iy < 0 || iy >= nh_core::ROWNO as i32 {
            break;
        }

        let ux = ix as usize;
        let uy = iy as usize;

        visibility.set_visible(ux, uy);

        // Stop at walls, closed doors, and other vision blockers
        let cell = level.cell(ux, uy);
        if blocks_vision(cell) {
            break;
        }
    }
}

/// Check if a cell blocks line of sight
fn blocks_vision(cell: &nh_core::dungeon::Cell) -> bool {
    use nh_core::dungeon::CellType;

    match cell.typ {
        // Walls always block vision
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
        | CellType::DBWall => true,

        // Stone blocks vision
        CellType::Stone => true,

        // Trees block vision
        CellType::Tree => true,

        // Closed doors block vision
        CellType::Door => {
            let door_state = cell.door_state();
            !door_state.contains(nh_core::dungeon::DoorState::OPEN)
        }

        // Secret doors look like walls, block vision
        CellType::SecretDoor | CellType::SecretCorridor => true,

        // Iron bars partially block (let's say they don't fully block)
        CellType::IronBars => false,

        // Everything else is transparent
        _ => false,
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

    // Floor objects visible if explored
    for (pos, mut vis) in object_query.iter_mut() {
        let x = pos.x as usize;
        let y = pos.y as usize;
        if visibility.is_visible(x, y) {
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    // Piles visible if explored
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
