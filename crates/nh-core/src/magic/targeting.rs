//! Advanced targeting utilities
//!
//! Helper functions for smart targeting of spells and wands,
//! including nearest monster detection, line-of-fire checks, and distance calculations.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::dungeon::Level;
use crate::monster::{Monster, MonsterId};
use crate::player::You;

/// Information about a valid target
#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub monster_id: MonsterId,
    pub name: String,
    pub x: i8,
    pub y: i8,
    pub distance: i32,
    pub hp: i32,
    pub hp_max: i32,
    pub is_hostile: bool,
}

impl TargetInfo {
    /// Get the direction vector toward this target from a position
    pub fn direction_from(&self, from_x: i8, from_y: i8) -> (i8, i8) {
        direction_to(from_x, from_y, self.x, self.y)
    }

    /// Get direction name (e.g., "northeast", "up", "self")
    pub fn direction_name(&self, from_x: i8, from_y: i8) -> &'static str {
        let (dx, dy) = self.direction_from(from_x, from_y);
        direction_name(dx, dy)
    }

    /// Get health percentage
    pub fn health_percent(&self) -> i32 {
        if self.hp_max > 0 {
            (self.hp * 100) / self.hp_max
        } else {
            0
        }
    }
}

/// Find the nearest visible monster to the player
pub fn find_nearest_monster(player: &You, level: &Level) -> Option<TargetInfo> {
    let mut nearest: Option<TargetInfo> = None;
    let mut min_distance = i32::MAX;

    for monster in level.visible_monsters(player.pos.x, player.pos.y) {
        let distance = calculate_distance(player.pos.x, player.pos.y, monster.x, monster.y);
        if distance < min_distance && distance > 0 {
            min_distance = distance;
            nearest = Some(TargetInfo {
                monster_id: monster.id,
                name: monster.name.clone(),
                x: monster.x,
                y: monster.y,
                distance,
                hp: monster.hp,
                hp_max: monster.hp_max,
                is_hostile: monster.is_hostile(),
            });
        }
    }

    nearest
}

/// Get all visible monsters sorted by distance
pub fn find_monsters_in_range(player: &You, level: &Level, max_range: i32) -> Vec<TargetInfo> {
    let mut monsters: Vec<TargetInfo> = level
        .visible_monsters(player.pos.x, player.pos.y)
        .iter()
        .filter_map(|monster| {
            let distance = calculate_distance(player.pos.x, player.pos.y, monster.x, monster.y);
            if distance > 0 && distance <= max_range {
                Some(TargetInfo {
                    monster_id: monster.id,
                    name: monster.name.clone(),
                    x: monster.x,
                    y: monster.y,
                    distance,
                    hp: monster.hp,
                    hp_max: monster.hp_max,
                    is_hostile: monster.is_hostile(),
                })
            } else {
                None
            }
        })
        .collect();

    // Sort by distance (nearest first)
    monsters.sort_by_key(|m| m.distance);
    monsters
}

/// Calculate direction from one position to another
fn direction_to(from_x: i8, from_y: i8, to_x: i8, to_y: i8) -> (i8, i8) {
    let dx = (to_x - from_x).signum() as i8;
    let dy = (to_y - from_y).signum() as i8;
    (dx, dy)
}

/// Get human-readable direction name
fn direction_name(dx: i8, dy: i8) -> &'static str {
    match (dx, dy) {
        (0, 0) => "self",
        (0, -1) => "north",
        (0, 1) => "south",
        (-1, 0) => "west",
        (1, 0) => "east",
        (-1, -1) => "northwest",
        (1, -1) => "northeast",
        (-1, 1) => "southwest",
        (1, 1) => "southeast",
        _ => "unknown",
    }
}

/// Calculate Chebyshev distance (max of absolute differences)
/// This matches the 8-direction distance used in NetHack
pub fn calculate_distance(x1: i8, y1: i8, x2: i8, y2: i8) -> i32 {
    let dx = (x1 - x2).abs() as i32;
    let dy = (y1 - y2).abs() as i32;
    dx.max(dy)
}

/// Check if a target is in line of fire (roughly - simplified LOS)
/// Returns true if target is generally in the direction and no walls block immediate path
pub fn is_in_line_of_fire(
    start_x: i8,
    start_y: i8,
    target_x: i8,
    target_y: i8,
    level: &Level,
) -> bool {
    // Basic check: no walls directly between start and target on cardinal direction
    let (dx, dy) = direction_to(start_x, start_y, target_x, target_y);

    // Check straight line in the primary direction
    if dx == 0 {
        // Vertical line
        let step = dy.signum();
        let mut y = start_y + step;
        while y != target_y {
            if !level.is_walkable(start_x, y) {
                return false;
            }
            y += step;
        }
    } else if dy == 0 {
        // Horizontal line
        let step = dx.signum();
        let mut x = start_x + step;
        while x != target_x {
            if !level.is_walkable(x, start_y) {
                return false;
            }
            x += step;
        }
    } else {
        // Diagonal: check both directions
        let dx_step = dx.signum();
        let dy_step = dy.signum();
        let mut x = start_x + dx_step;
        let mut y = start_y + dy_step;
        while x != target_x || y != target_y {
            if !level.is_walkable(x, y) {
                return false;
            }
            if x != target_x {
                x += dx_step;
            }
            if y != target_y {
                y += dy_step;
            }
        }
    }

    true
}

/// Get monsters visible in a specific direction from player
pub fn monsters_in_direction(
    player: &You,
    direction: (i8, i8),
    level: &Level,
    range: i32,
) -> Vec<TargetInfo> {
    let mut targets = Vec::new();

    for monster in level.visible_monsters(player.pos.x, player.pos.y) {
        let (dx, dy) = direction_to(player.pos.x, player.pos.y, monster.x, monster.y);
        let distance = calculate_distance(player.pos.x, player.pos.y, monster.x, monster.y);

        // Check if monster is in the specified direction
        if dx == direction.0 && dy == direction.1 && distance <= range {
            targets.push(TargetInfo {
                monster_id: monster.id,
                name: monster.name.clone(),
                x: monster.x,
                y: monster.y,
                distance,
                hp: monster.hp,
                hp_max: monster.hp_max,
                is_hostile: monster.is_hostile(),
            });
        }
    }

    // Sort by distance
    targets.sort_by_key(|m| m.distance);
    targets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_to_adjacent() {
        assert_eq!(direction_to(5, 5, 6, 5), (1, 0)); // East
        assert_eq!(direction_to(5, 5, 4, 5), (-1, 0)); // West
        assert_eq!(direction_to(5, 5, 5, 4), (0, -1)); // North
        assert_eq!(direction_to(5, 5, 5, 6), (0, 1)); // South
    }

    #[test]
    fn test_direction_to_diagonal() {
        assert_eq!(direction_to(5, 5, 6, 6), (1, 1)); // Southeast
        assert_eq!(direction_to(5, 5, 4, 4), (-1, -1)); // Northwest
        assert_eq!(direction_to(5, 5, 6, 4), (1, -1)); // Northeast
        assert_eq!(direction_to(5, 5, 4, 6), (-1, 1)); // Southwest
    }

    #[test]
    fn test_direction_to_far() {
        assert_eq!(direction_to(5, 5, 20, 30), (1, 1)); // Normalized to (1, 1)
        assert_eq!(direction_to(10, 10, 5, 5), (-1, -1)); // Normalized to (-1, -1)
    }

    #[test]
    fn test_direction_name() {
        assert_eq!(direction_name(0, 0), "self");
        assert_eq!(direction_name(0, -1), "north");
        assert_eq!(direction_name(1, 0), "east");
        assert_eq!(direction_name(1, 1), "southeast");
    }

    #[test]
    fn test_calculate_distance_adjacent() {
        assert_eq!(calculate_distance(5, 5, 6, 5), 1);
        assert_eq!(calculate_distance(5, 5, 5, 4), 1);
        assert_eq!(calculate_distance(5, 5, 6, 6), 1);
    }

    #[test]
    fn test_calculate_distance_far() {
        assert_eq!(calculate_distance(0, 0, 5, 5), 5);
        assert_eq!(calculate_distance(10, 10, 0, 0), 10);
        assert_eq!(calculate_distance(5, 5, 5, 15), 10);
    }

    #[test]
    fn test_calculate_distance_same_position() {
        assert_eq!(calculate_distance(5, 5, 5, 5), 0);
    }

    #[test]
    fn test_target_info_health_percent() {
        let target = TargetInfo {
            monster_id: crate::monster::MonsterId(1),
            name: "goblin".to_string(),
            x: 5,
            y: 5,
            distance: 1,
            hp: 50,
            hp_max: 100,
            is_hostile: true,
        };
        assert_eq!(target.health_percent(), 50);
    }

    #[test]
    fn test_target_info_health_percent_low() {
        let target = TargetInfo {
            monster_id: crate::monster::MonsterId(1),
            name: "goblin".to_string(),
            x: 5,
            y: 5,
            distance: 1,
            hp: 10,
            hp_max: 100,
            is_hostile: true,
        };
        assert_eq!(target.health_percent(), 10);
    }
}
