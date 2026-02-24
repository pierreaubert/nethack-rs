//! Mouse click navigation with A* pathfinding
//!
//! Allows the player to click on a tile to automatically navigate there.
//! Uses A* algorithm for optimal pathfinding around obstacles.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::plugins::camera::MainCamera;
use crate::plugins::game::AppState;
use crate::plugins::input::GameCommand;
use crate::plugins::ui::InventoryState;
use crate::resources::GameStateResource;

pub struct NavigationPlugin;

impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NavigationState>().add_systems(
            Update,
            (handle_mouse_click, process_navigation_queue)
                .chain()
                .run_if(in_state(AppState::Playing)),
        );
    }
}

/// Navigation state tracking the current path
#[derive(Resource, Default)]
pub struct NavigationState {
    /// Queue of positions to move through
    pub path: Vec<(i8, i8)>,
    /// Whether we're currently auto-navigating
    pub active: bool,
    /// Target position for visual feedback
    pub target: Option<(i8, i8)>,
}

impl NavigationState {
    pub fn clear(&mut self) {
        self.path.clear();
        self.active = false;
        self.target = None;
    }
}

/// A* node for the priority queue
#[derive(Clone, Eq, PartialEq)]
struct AStarNode {
    x: i8,
    y: i8,
    g_cost: i32, // Cost from start
    f_cost: i32, // g_cost + heuristic
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other
            .f_cost
            .cmp(&self.f_cost)
            .then_with(|| other.g_cost.cmp(&self.g_cost))
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Calculate Manhattan distance heuristic
fn heuristic(x1: i8, y1: i8, x2: i8, y2: i8) -> i32 {
    (x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs()
}

/// Reconstruct path from came_from map
fn reconstruct_path(
    came_from: &[[Option<(i8, i8)>; nh_core::ROWNO]; nh_core::COLNO],
    end: (i8, i8),
) -> Vec<(i8, i8)> {
    let mut path = Vec::new();
    let mut current = end;

    while let Some(prev) = came_from[current.0 as usize][current.1 as usize] {
        path.push(current);
        current = prev;
    }

    path.reverse();
    path
}

/// Find path using A* algorithm
pub fn find_path(
    level: &nh_core::dungeon::Level,
    start: (i8, i8),
    end: (i8, i8),
) -> Option<Vec<(i8, i8)>> {
    // Quick check if destination is valid
    if !level.is_valid_pos(end.0, end.1) {
        return None;
    }

    // Allow clicking on non-walkable tiles (will path to adjacent)
    // But if the tile is completely inaccessible, return None
    let target_walkable = level.is_walkable(end.0, end.1);

    if start == end {
        return Some(vec![]);
    }

    let mut open_set = BinaryHeap::new();
    let mut g_scores = [[i32::MAX; nh_core::ROWNO]; nh_core::COLNO];
    let mut came_from: [[Option<(i8, i8)>; nh_core::ROWNO]; nh_core::COLNO] =
        [[None; nh_core::ROWNO]; nh_core::COLNO];
    let mut closed = [[false; nh_core::ROWNO]; nh_core::COLNO];

    g_scores[start.0 as usize][start.1 as usize] = 0;
    open_set.push(AStarNode {
        x: start.0,
        y: start.1,
        g_cost: 0,
        f_cost: heuristic(start.0, start.1, end.0, end.1),
    });

    // 8-directional movement
    let directions: [(i8, i8); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    while let Some(current) = open_set.pop() {
        let (cx, cy) = (current.x, current.y);

        // Check if we reached the destination (or adjacent if destination is unwalkable)
        if (cx, cy) == end {
            return Some(reconstruct_path(&came_from, end));
        }

        // If target isn't walkable, check if we're adjacent to it
        if !target_walkable {
            let dx = (cx as i32 - end.0 as i32).abs();
            let dy = (cy as i32 - end.1 as i32).abs();
            if dx <= 1 && dy <= 1 {
                return Some(reconstruct_path(&came_from, (cx, cy)));
            }
        }

        if closed[cx as usize][cy as usize] {
            continue;
        }
        closed[cx as usize][cy as usize] = true;

        for (dx, dy) in directions {
            let nx = cx + dx;
            let ny = cy + dy;

            if !level.is_valid_pos(nx, ny) {
                continue;
            }

            // Allow moving to target even if not walkable (to get adjacent)
            if !level.is_walkable(nx, ny) && (nx, ny) != end {
                continue;
            }

            // Movement cost (diagonal costs more)
            let move_cost = if dx != 0 && dy != 0 { 14 } else { 10 }; // 14 ~= 10 * sqrt(2)
            let new_g = current.g_cost + move_cost;

            if new_g < g_scores[nx as usize][ny as usize] {
                g_scores[nx as usize][ny as usize] = new_g;
                came_from[nx as usize][ny as usize] = Some((cx, cy));

                let f_cost = new_g + heuristic(nx, ny, end.0, end.1);
                open_set.push(AStarNode {
                    x: nx,
                    y: ny,
                    g_cost: new_g,
                    f_cost,
                });
            }
        }
    }

    None // No path found
}

/// Convert direction from current pos to next pos into a Command::Move direction
fn pos_to_direction(from: (i8, i8), to: (i8, i8)) -> Option<nh_core::action::Direction> {
    use nh_core::action::Direction;

    let dx = to.0 - from.0;
    let dy = to.1 - from.1;

    match (dx, dy) {
        (-1, -1) => Some(Direction::NorthWest),
        (0, -1) => Some(Direction::North),
        (1, -1) => Some(Direction::NorthEast),
        (-1, 0) => Some(Direction::West),
        (1, 0) => Some(Direction::East),
        (-1, 1) => Some(Direction::SouthWest),
        (0, 1) => Some(Direction::South),
        (1, 1) => Some(Direction::SouthEast),
        _ => None,
    }
}

/// Handle left mouse clicks to set navigation target
fn handle_mouse_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    game_state: Res<GameStateResource>,
    mut nav_state: ResMut<NavigationState>,
    inv_state: Res<InventoryState>,
) {
    // Don't navigate when inventory is open
    if inv_state.open {
        return;
    }

    // Only handle left click
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Convert screen position to world position
    // For top-down view, we intersect with the y=0 plane
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };

    // Find intersection with y=0 plane (ground level)
    // Ray equation: point = origin + t * direction
    // For y=0: origin.y + t * direction.y = 0
    // t = -origin.y / direction.y
    if ray.direction.y.abs() < 0.0001 {
        return; // Ray is parallel to ground
    }

    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        return; // Intersection is behind camera
    }

    let world_pos = ray.origin + ray.direction * t;

    // Convert world position to map coordinates
    // In our system: world X = map X, world Z = map Y
    let map_x = world_pos.x.round() as i8;
    let map_y = world_pos.z.round() as i8;

    // Validate position
    if !game_state.0.current_level.is_valid_pos(map_x, map_y) {
        return;
    }

    // Get player position
    let player_x = game_state.0.player.pos.x;
    let player_y = game_state.0.player.pos.y;

    // Don't navigate to current position
    if map_x == player_x && map_y == player_y {
        nav_state.clear();
        return;
    }

    // Find path
    if let Some(path) = find_path(
        &game_state.0.current_level,
        (player_x, player_y),
        (map_x, map_y),
    ) {
        if !path.is_empty() {
            nav_state.path = path;
            nav_state.active = true;
            nav_state.target = Some((map_x, map_y));
        }
    } else {
        // No path found - clear navigation
        nav_state.clear();
    }
}

/// Process the navigation queue, moving one step at a time
fn process_navigation_queue(
    mut nav_state: ResMut<NavigationState>,
    game_state: Res<GameStateResource>,
    mut commands: MessageWriter<GameCommand>,
) {
    if !nav_state.active || nav_state.path.is_empty() {
        nav_state.active = false;
        return;
    }

    let player_x = game_state.0.player.pos.x;
    let player_y = game_state.0.player.pos.y;

    // Get next position in path
    let next_pos = nav_state.path[0];

    // Check if we're already there (might have been moved by something else)
    if next_pos.0 == player_x && next_pos.1 == player_y {
        nav_state.path.remove(0);
        if nav_state.path.is_empty() {
            nav_state.clear();
        }
        return;
    }

    // Verify the path is still valid (tile might have become blocked)
    if !game_state
        .0
        .current_level
        .is_walkable(next_pos.0, next_pos.1)
    {
        // Path blocked - recalculate or stop
        if let Some(target) = nav_state.target {
            if let Some(new_path) =
                find_path(&game_state.0.current_level, (player_x, player_y), target)
            {
                if !new_path.is_empty() {
                    nav_state.path = new_path;
                    return; // Try again next frame with new path
                }
            }
        }
        nav_state.clear();
        return;
    }

    // Check if there's a monster in the way
    if game_state
        .0
        .current_level
        .monster_at(next_pos.0, next_pos.1)
        .is_some()
    {
        // Stop navigation when encountering a monster
        nav_state.clear();
        return;
    }

    // Convert position to direction and send move command
    if let Some(direction) = pos_to_direction((player_x, player_y), next_pos) {
        commands.write(GameCommand(nh_core::action::Command::Move(direction)));
        nav_state.path.remove(0);

        if nav_state.path.is_empty() {
            nav_state.clear();
        }
    } else {
        // Invalid direction - clear navigation
        nav_state.clear();
    }
}
