//! Corridor generation (mklev.c: makecorridors, join, dig_corridor)
//!
//! Implements the NetHack 4-phase corridor algorithm:
//! 1. Connect adjacent rooms (room[i] to room[i+1])
//! 2. Connect rooms two steps apart if not already connected
//! 3. Ensure all rooms are reachable from room 0
//! 4. Add random extra corridors for variety

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::rng::GameRng;
use crate::{COLNO, ROWNO};

use super::room::Room;
use super::{CellType, Level};

/// Tracks room connectivity using equivalence classes (smeq[] in C)
#[derive(Debug, Clone)]
pub struct ConnectivityTracker {
    /// Each room's equivalence class (rooms in same class are connected)
    smeq: Vec<usize>,
}

impl ConnectivityTracker {
    /// Create a new tracker for the given number of rooms
    pub fn new(num_rooms: usize) -> Self {
        // Initially, each room is its own equivalence class
        Self {
            smeq: (0..num_rooms).collect(),
        }
    }

    /// Check if two rooms are connected (in same equivalence class)
    pub fn are_connected(&self, a: usize, b: usize) -> bool {
        if a >= self.smeq.len() || b >= self.smeq.len() {
            return false;
        }
        self.smeq[a] == self.smeq[b]
    }

    /// Merge equivalence classes when rooms are connected
    pub fn merge(&mut self, a: usize, b: usize) {
        if a >= self.smeq.len() || b >= self.smeq.len() {
            return;
        }

        let old_class = self.smeq[b];
        let new_class = self.smeq[a];

        // Update all rooms in b's class to a's class
        for eq in &mut self.smeq {
            if *eq == old_class {
                *eq = new_class;
            }
        }
    }

    /// Check if all rooms are connected
    pub fn all_connected(&self) -> bool {
        if self.smeq.is_empty() {
            return true;
        }
        let first_class = self.smeq[0];
        self.smeq.iter().all(|&c| c == first_class)
    }
}

/// Find a door position on a room wall facing the target
fn find_door_position(room: &Room, target: &Room, rng: &mut GameRng) -> (usize, usize) {
    let (rx, ry) = room.center();
    let (tx, ty) = target.center();

    // Determine which wall to use based on relative position
    if (tx as i32 - rx as i32).abs() > (ty as i32 - ry as i32).abs() {
        // Target is more horizontal - use left or right wall
        if tx > rx {
            // Right wall
            let y = room.y + rng.rn2(room.height as u32) as usize;
            (room.x + room.width, y)
        } else {
            // Left wall
            let y = room.y + rng.rn2(room.height as u32) as usize;
            (room.x.saturating_sub(1), y)
        }
    } else {
        // Target is more vertical - use top or bottom wall
        if ty > ry {
            // Bottom wall
            let x = room.x + rng.rn2(room.width as u32) as usize;
            (x, room.y + room.height)
        } else {
            // Top wall
            let x = room.x + rng.rn2(room.width as u32) as usize;
            (x, room.y.saturating_sub(1))
        }
    }
}

/// Dig a corridor between two points using an organic path
/// This mimics C's dig_corridor() from sp_lev.c
pub fn dig_corridor(
    level: &mut Level,
    start_x: usize,
    start_y: usize,
    end_x: usize,
    end_y: usize,
    rng: &mut GameRng,
    allow_secret: bool,
) {
    let mut x = start_x as i32;
    let mut y = start_y as i32;
    let tx = end_x as i32;
    let ty = end_y as i32;

    let mut steps = 0;
    const MAX_STEPS: i32 = 500;

    while (x != tx || y != ty) && steps < MAX_STEPS {
        steps += 1;

        // Calculate distances to target
        let dx = tx - x;
        let dy = ty - y;

        // Determine direction to move
        let (move_x, move_y) = if dx.abs() > dy.abs() {
            // Move horizontally with some randomness
            if rng.rn2((dx.abs() + 1) as u32) > 0 {
                (dx.signum(), 0)
            } else if dy != 0 {
                (0, dy.signum())
            } else {
                (dx.signum(), 0)
            }
        } else if dy.abs() > dx.abs() {
            // Move vertically with some randomness
            if rng.rn2((dy.abs() + 1) as u32) > 0 {
                (0, dy.signum())
            } else if dx != 0 {
                (dx.signum(), 0)
            } else {
                (0, dy.signum())
            }
        } else {
            // Equal distances - choose randomly
            if rng.one_in(2) {
                (dx.signum(), 0)
            } else {
                (0, dy.signum())
            }
        };

        x += move_x;
        y += move_y;

        // Bounds check
        if x < 0 || y < 0 || x >= COLNO as i32 || y >= ROWNO as i32 {
            break;
        }

        let ux = x as usize;
        let uy = y as usize;

        // Check what's at this position
        let cell_type = level.cells[ux][uy].typ;

        match cell_type {
            CellType::Stone => {
                // 1% chance of secret corridor if allowed
                if allow_secret && rng.rn2(100) == 0 {
                    level.cells[ux][uy].typ = CellType::SecretCorridor;
                } else {
                    level.cells[ux][uy].typ = CellType::Corridor;
                }
            }
            CellType::Room | CellType::Corridor | CellType::SecretCorridor => {
                // Already passable, continue
            }
            _ if cell_type.is_wall() => {
                // Hit a wall - this might become a door later
                // For now, convert to corridor to allow passage
                level.cells[ux][uy].typ = CellType::Corridor;
            }
            _ => {
                // Other terrain - stop
                break;
            }
        }
    }
}

/// Join two rooms with a corridor
fn join_rooms(
    level: &mut Level,
    rooms: &[Room],
    room_a: usize,
    room_b: usize,
    tracker: &mut ConnectivityTracker,
    rng: &mut GameRng,
    nxcor: bool, // "new corridor" mode - allows more randomness
) {
    if room_a >= rooms.len() || room_b >= rooms.len() || room_a == room_b {
        return;
    }

    let a = &rooms[room_a];
    let b = &rooms[room_b];

    // Find door positions on each room
    let (ax, ay) = find_door_position(a, b, rng);
    let (bx, by) = find_door_position(b, a, rng);

    // Dig corridor between the door positions
    dig_corridor(level, ax, ay, bx, by, rng, nxcor);

    // Update connectivity
    tracker.merge(room_a, room_b);
}

/// Check if there's a door next to a position (4 cardinal directions)
/// Matches C's bydoor()
///
/// Checks if any of the 4 cardinal adjacent cells (N, S, E, W) contains a door.
pub fn bydoor(level: &Level, x: i32, y: i32) -> bool {
    let directions = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)];

    for (nx, ny) in &directions {
        if *nx >= 0 && *ny >= 0 && (*nx as usize) < COLNO && (*ny as usize) < ROWNO {
            let cell_type = level.cells[*nx as usize][*ny as usize].typ;
            if matches!(cell_type, CellType::Door | CellType::SecretDoor) {
                return true;
            }
        }
    }
    false
}

/// Check if there's a door next to a position (8 directions including diagonals)
/// Matches C's nexttodoor()
///
/// Checks all 8 adjacent cells (including diagonals) for doors.
pub fn nexttodoor(level: &Level, x: i32, y: i32) -> bool {
    for dx in -1..=1 {
        for dy in -1..=1 {
            if dx == 0 && dy == 0 {
                continue; // Skip center
            }
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < COLNO && (ny as usize) < ROWNO {
                let cell_type = level.cells[nx as usize][ny as usize].typ;
                if matches!(cell_type, CellType::Door | CellType::SecretDoor) {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if a position is valid for placing a door
/// Matches C's okdoor()
///
/// A position is valid for a door if:
/// 1. It's on a wall (HWall or VWall)
/// 2. There's no door already next to it (bydoor check)
pub fn okdoor(level: &Level, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= COLNO as i32 || y >= ROWNO as i32 {
        return false;
    }

    let cell_type = level.cells[x as usize][y as usize].typ;

    // Must be on a wall
    if !matches!(cell_type, CellType::HWall | CellType::VWall) {
        return false;
    }

    // Must not be near another door
    !bydoor(level, x, y)
}

/// Place a corridor or secret corridor at a position
/// Matches C's corr()
///
/// Randomly chooses between a regular corridor (98%) or secret corridor (2%).
pub fn corr(level: &mut Level, x: usize, y: usize, rng: &mut GameRng) {
    if x >= COLNO || y >= ROWNO {
        return;
    }

    // 2% chance of secret corridor (matches C's rn2(50) != 0)
    if rng.rn2(50) == 0 {
        level.cells[x][y].typ = CellType::SecretCorridor;
    } else {
        level.cells[x][y].typ = CellType::Corridor;
    }
}

/// Find a door position in a wall region
/// Matches C's finddpos() - finds a random door position in a wall
///
/// Tries to find a valid door position (via okdoor) in the given area,
/// with multiple fallback strategies.
pub fn finddpos(
    level: &Level,
    xl: usize,
    yl: usize,
    xh: usize,
    yh: usize,
    rng: &mut GameRng,
) -> Option<(usize, usize)> {
    // Try random position first
    let x = xl + rng.rn2((xh - xl + 1) as u32) as usize;
    let y = yl + rng.rn2((yh - yl + 1) as u32) as usize;

    if okdoor(level, x as i32, y as i32) {
        return Some((x, y));
    }

    // Scan the area linearly
    for x in xl..=xh {
        for y in yl..=yh {
            if okdoor(level, x as i32, y as i32) {
                return Some((x, y));
            }
        }
    }

    // If no okdoor found, look for any door or diagonal to door
    for x in xl..=xh {
        for y in yl..=yh {
            if nexttodoor(level, x as i32, y as i32) {
                return Some((x, y));
            }
        }
    }

    // Last resort: return corner
    Some((xl, yh))
}

/// Generate corridors using the 4-phase algorithm
/// Matches C's makecorridors()
pub fn generate_corridors(level: &mut Level, rooms: &[Room], rng: &mut GameRng) {
    if rooms.len() < 2 {
        return;
    }

    let mut tracker = ConnectivityTracker::new(rooms.len());

    // Phase 1: Connect adjacent rooms (room[i] to room[i+1])
    // With 2% chance of early stop (matches C: !rn2(50))
    for i in 0..rooms.len() - 1 {
        join_rooms(level, rooms, i, i + 1, &mut tracker, rng, false);
        if rng.rn2(50) == 0 {
            break;
        }
    }

    // Phase 2: Connect rooms two steps apart if not connected
    for i in 0..rooms.len().saturating_sub(2) {
        if !tracker.are_connected(i, i + 2) {
            join_rooms(level, rooms, i, i + 2, &mut tracker, rng, false);
        }
    }

    // Phase 3: Ensure all rooms reachable from room 0
    // Keep connecting until all rooms are in the same equivalence class
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 100; // Prevent infinite loops

    while !tracker.all_connected() && iterations < MAX_ITERATIONS {
        iterations += 1;
        let mut made_connection = false;

        for a in 0..rooms.len() {
            for b in 0..rooms.len() {
                if !tracker.are_connected(a, b) {
                    join_rooms(level, rooms, a, b, &mut tracker, rng, false);
                    made_connection = true;
                    break;
                }
            }
            if made_connection {
                break;
            }
        }

        if !made_connection {
            break;
        }
    }

    // Phase 4: Add random extra corridors (4-7 additional)
    // This creates more interesting level topology
    if rooms.len() > 2 {
        let extra = rng.rn2(rooms.len() as u32) as usize + 4;
        for _ in 0..extra.min(10) {
            let a = rng.rn2(rooms.len() as u32) as usize;
            let mut b = rng.rn2((rooms.len() - 2) as u32) as usize;
            if b >= a {
                b += 2;
            }
            if b < rooms.len() {
                join_rooms(level, rooms, a, b, &mut tracker, rng, true);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::DLevel;

    #[test]
    fn test_connectivity_tracker() {
        let mut tracker = ConnectivityTracker::new(5);

        // Initially, no rooms are connected
        assert!(!tracker.are_connected(0, 1));
        assert!(!tracker.are_connected(1, 2));

        // Connect 0 and 1
        tracker.merge(0, 1);
        assert!(tracker.are_connected(0, 1));
        assert!(!tracker.are_connected(0, 2));

        // Connect 1 and 2 (should also connect 0 and 2)
        tracker.merge(1, 2);
        assert!(tracker.are_connected(0, 2));
        assert!(tracker.are_connected(1, 2));

        // Not all connected yet
        assert!(!tracker.all_connected());

        // Connect remaining rooms
        tracker.merge(2, 3);
        tracker.merge(3, 4);
        assert!(tracker.all_connected());
    }

    #[test]
    fn test_generate_corridors() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut rng = GameRng::new(42);

        // Create some test rooms
        let rooms = vec![
            Room::new(5, 5, 5, 4),
            Room::new(20, 5, 5, 4),
            Room::new(35, 5, 5, 4),
            Room::new(50, 5, 5, 4),
        ];

        // Carve rooms first
        for room in &rooms {
            for x in room.x..room.x + room.width {
                for y in room.y..room.y + room.height {
                    level.cells[x][y].typ = CellType::Room;
                }
            }
        }

        // Generate corridors
        generate_corridors(&mut level, &rooms, &mut rng);

        // Count corridor cells
        let corridor_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|cell| cell.typ == CellType::Corridor)
            .count();

        println!("Generated {} corridor cells", corridor_count);
        assert!(corridor_count > 0, "Should have generated corridors");

        // Verify connectivity by flood fill
        let (start_x, start_y) = rooms[0].center();
        let reachable = flood_fill_count(&level, start_x, start_y);
        println!("Reachable cells from room 0: {}", reachable);

        // Should be able to reach cells in other rooms
        let total_room_cells: usize = rooms.iter().map(|r| r.width * r.height).sum();
        assert!(
            reachable >= total_room_cells,
            "Should be able to reach all room cells"
        );
    }

    fn flood_fill_count(level: &Level, start_x: usize, start_y: usize) -> usize {
        let mut visited = vec![vec![false; ROWNO]; COLNO];
        let mut stack = vec![(start_x, start_y)];
        let mut count = 0;

        while let Some((x, y)) = stack.pop() {
            if x >= COLNO || y >= ROWNO || visited[x][y] {
                continue;
            }
            visited[x][y] = true;

            let cell_type = level.cells[x][y].typ;
            if cell_type == CellType::Stone || cell_type.is_wall() {
                continue;
            }

            count += 1;

            // Add neighbors
            if x > 0 {
                stack.push((x - 1, y));
            }
            if x + 1 < COLNO {
                stack.push((x + 1, y));
            }
            if y > 0 {
                stack.push((x, y - 1));
            }
            if y + 1 < ROWNO {
                stack.push((x, y + 1));
            }
        }

        count
    }

    #[test]
    fn test_dig_corridor() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut rng = GameRng::new(42);

        // Dig a corridor from (10, 10) to (30, 10)
        dig_corridor(&mut level, 10, 10, 30, 10, &mut rng, false);

        // Should have corridor cells along the path
        let corridor_count = (10..=30)
            .filter(|&x| level.cells[x][10].typ == CellType::Corridor)
            .count();

        assert!(corridor_count >= 10, "Should have corridor cells");
    }

    #[test]
    fn test_bydoor() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut rng = GameRng::new(42);

        // Place a door at (20, 20)
        level.cells[20][20].typ = CellType::Door;

        // Should detect door next to position (adjacent cells)
        assert!(bydoor(&level, 20, 19)); // Door to the south
        assert!(bydoor(&level, 20, 21)); // Door to the north
        assert!(bydoor(&level, 19, 20)); // Door to the west
        assert!(bydoor(&level, 21, 20)); // Door to the east

        // Should not detect door at diagonal
        assert!(!bydoor(&level, 21, 19));
        assert!(!bydoor(&level, 19, 19));

        // Should not detect door far away
        assert!(!bydoor(&level, 10, 10));
    }

    #[test]
    fn test_nexttodoor() {
        let mut level = Level::new(DLevel::main_dungeon_start());

        // Place a door at (20, 20)
        level.cells[20][20].typ = CellType::SecretDoor;

        // Should detect door in all 8 directions (including diagonals)
        assert!(nexttodoor(&level, 20, 19)); // North
        assert!(nexttodoor(&level, 20, 21)); // South
        assert!(nexttodoor(&level, 19, 20)); // West
        assert!(nexttodoor(&level, 21, 20)); // East
        assert!(nexttodoor(&level, 19, 19)); // NW diagonal
        assert!(nexttodoor(&level, 21, 19)); // NE diagonal
        assert!(nexttodoor(&level, 19, 21)); // SW diagonal
        assert!(nexttodoor(&level, 21, 21)); // SE diagonal

        // Should not detect door far away
        assert!(!nexttodoor(&level, 10, 10));
    }

    #[test]
    fn test_okdoor() {
        let mut level = Level::new(DLevel::main_dungeon_start());

        // Set up a wall
        level.cells[20][20].typ = CellType::HWall;

        // Should be valid on wall with no adjacent door
        assert!(okdoor(&level, 20, 20));

        // Place a door nearby
        level.cells[20][19].typ = CellType::Door;

        // Should now be invalid (has adjacent door)
        assert!(!okdoor(&level, 20, 20));

        // Position not on wall should be invalid
        level.cells[15][15].typ = CellType::Stone;
        assert!(!okdoor(&level, 15, 15));

        // Out of bounds should be invalid
        assert!(!okdoor(&level, -1, 10));
        assert!(!okdoor(&level, 100, 100));
    }

    #[test]
    fn test_corr() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut rng = GameRng::new(42);

        // Place corridor at location (within COLNO x ROWNO bounds)
        level.cells[25][15].typ = CellType::Stone;
        corr(&mut level, 25, 15, &mut rng);

        // Should be either corridor or secret corridor
        match level.cells[25][15].typ {
            CellType::Corridor | CellType::SecretCorridor => (),
            _ => panic!("Expected corridor or secret corridor"),
        }
    }

    #[test]
    fn test_finddpos() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut rng = GameRng::new(42);

        // Set up wall area (HWall at top wall)
        for x in 10..=20 {
            level.cells[x][10].typ = CellType::HWall;
        }

        // Find door position in wall area
        let pos = finddpos(&level, 10, 10, 20, 10, &mut rng);
        assert!(pos.is_some(), "Should find a valid position");

        if let Some((x, y)) = pos {
            // Should be within bounds
            assert!(x >= 10 && x <= 20);
            assert!(y >= 10 && y <= 10);
        }
    }

    #[test]
    fn test_finddpos_empty_area() {
        let level = Level::new(DLevel::main_dungeon_start());
        let mut rng = GameRng::new(42);

        // No walls in empty area - should return last resort corner
        let pos = finddpos(&level, 30, 30, 35, 35, &mut rng);
        assert_eq!(pos, Some((30, 35)), "Should return corner as last resort");
    }
}
