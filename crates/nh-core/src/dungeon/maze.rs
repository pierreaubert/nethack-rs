//! Maze level generation (mkmaze.c)
//!
//! Implements maze-type level generation for Gehennom and special levels.
//! Uses a recursive backtracking algorithm similar to NetHack's C implementation.

use crate::rng::GameRng;

use super::cell::CellType;
use super::level::{Level, Stairway, TrapType};
use super::room::Room;
use super::DLevel;

/// Map dimensions (from global.h)
pub const COLNO: usize = 80;
pub const ROWNO: usize = 21;

/// Determine wall type based on adjacent open spaces
/// Uses the same logic as C's spine_array in mkmaze.c
///
/// The spine_array maps neighbor configurations to wall types:
/// - Neighbors encoded as 4-bit value: NSEW (North, South, East, West)
/// - Returns appropriate wall/corner type for proper maze rendering
fn wall_type_from_neighbors(north: bool, south: bool, east: bool, west: bool) -> CellType {
    // Encode neighbors as 4-bit value: NSEW
    let index = (north as usize) << 3 
              | (south as usize) << 2 
              | (east as usize) << 1 
              | (west as usize);
    
    // spine_array from mkmaze.c:
    // { VWALL, HWALL, HWALL, HWALL,
    //   VWALL, TRCORNER, TLCORNER, TDWALL,
    //   VWALL, BRCORNER, BLCORNER, TUWALL,
    //   VWALL, TLWALL, TRWALL, CROSSWALL }
    match index {
        0b0000 => CellType::VWall,      // No neighbors - default vertical
        0b0001 => CellType::HWall,      // West only
        0b0010 => CellType::HWall,      // East only
        0b0011 => CellType::HWall,      // East and West
        0b0100 => CellType::VWall,      // South only
        0b0101 => CellType::TRCorner,   // South and West
        0b0110 => CellType::TLCorner,   // South and East
        0b0111 => CellType::TDWall,     // South, East, West (T down)
        0b1000 => CellType::VWall,      // North only
        0b1001 => CellType::BRCorner,   // North and West
        0b1010 => CellType::BLCorner,   // North and East
        0b1011 => CellType::TUWall,     // North, East, West (T up)
        0b1100 => CellType::VWall,      // North and South
        0b1101 => CellType::TLWall,     // North, South, West (T left)
        0b1110 => CellType::TRWall,     // North, South, East (T right)
        0b1111 => CellType::CrossWall,  // All four neighbors
        _ => CellType::Stone,
    }
}

/// Generate a maze level
/// Matches C's makemaz() from mkmaze.c
pub fn generate_maze(level: &mut Level, rng: &mut GameRng) {
    let depth = level.dlevel.depth();

    // Fill with walls first (use Stone as the base wall type for mazes)
    for x in 0..COLNO {
        for y in 0..ROWNO {
            level.cells[x][y].typ = CellType::Stone;
            level.cells[x][y].lit = depth < 10; // Shallow mazes are lit
        }
    }

    // Create maze using recursive backtracking
    // Start from a random odd position (ensures walls between paths)
    let start_x = 2 + (rng.rn2(((COLNO - 4) / 2) as u32) as usize) * 2 + 1;
    let start_y = 2 + (rng.rn2(((ROWNO - 4) / 2) as u32) as usize) * 2 + 1;

    carve_maze(level, start_x, start_y, rng);

    // Add some random rooms in the maze (like C's mkmazewalls)
    let num_rooms = rng.rnd(3) as usize; // 1-3 rooms
    let mut rooms = Vec::new();
    for _ in 0..num_rooms {
        if let Some(room) = try_place_maze_room(level, rng) {
            rooms.push(room);
        }
    }

    // Fix up wall types based on neighbors (like C's wallification)
    fix_maze_walls(level);

    // Place stairs
    place_maze_stairs(level, rng);

    // Place traps (more in mazes)
    place_maze_traps(level, depth, rng);

    // Place some gold
    place_maze_gold(level, depth, rng);
}

/// Fix wall types based on adjacent passages
/// Converts Stone walls to proper HWALL/VWALL/corner types
fn fix_maze_walls(level: &mut Level) {
    // Collect wall positions and their proper types
    let mut wall_updates: Vec<(usize, usize, CellType)> = Vec::new();
    
    for x in 1..COLNO - 1 {
        for y in 1..ROWNO - 1 {
            if level.cells[x][y].typ == CellType::Stone {
                // Check if this wall is adjacent to any passage
                let north = is_passage(&level.cells[x][y.saturating_sub(1)].typ);
                let south = is_passage(&level.cells[x][y + 1].typ);
                let east = is_passage(&level.cells[x + 1][y].typ);
                let west = is_passage(&level.cells[x.saturating_sub(1)][y].typ);
                
                // Only convert to wall type if adjacent to at least one passage
                if north || south || east || west {
                    let wall_type = wall_type_from_neighbors(north, south, east, west);
                    wall_updates.push((x, y, wall_type));
                }
            }
        }
    }
    
    // Apply updates
    for (x, y, wall_type) in wall_updates {
        level.cells[x][y].typ = wall_type;
    }
}

/// Check if a cell type is a passage (corridor or room)
fn is_passage(typ: &CellType) -> bool {
    matches!(typ, CellType::Corridor | CellType::Room | CellType::Door)
}

/// Carve maze passages using recursive backtracking
fn carve_maze(level: &mut Level, x: usize, y: usize, rng: &mut GameRng) {
    level.cells[x][y].typ = CellType::Corridor;

    // Directions: N, S, E, W
    let mut directions = [(0i32, -2i32), (0, 2), (2, 0), (-2, 0)];

    // Shuffle directions
    for i in (1..4).rev() {
        let j = rng.rn2((i + 1) as u32) as usize;
        directions.swap(i, j);
    }

    for (dx, dy) in directions {
        let nx = (x as i32 + dx) as usize;
        let ny = (y as i32 + dy) as usize;

        // Check bounds
        if nx < 2 || nx >= COLNO - 2 || ny < 2 || ny >= ROWNO - 2 {
            continue;
        }

        // Check if unvisited (Stone = uncarved)
        if level.cells[nx][ny].typ == CellType::Stone {
            // Carve the wall between current and next
            let wx = (x as i32 + dx / 2) as usize;
            let wy = (y as i32 + dy / 2) as usize;
            level.cells[wx][wy].typ = CellType::Corridor;

            // Recurse
            carve_maze(level, nx, ny, rng);
        }
    }
}

/// Try to place a room in the maze
fn try_place_maze_room(level: &mut Level, rng: &mut GameRng) -> Option<Room> {
    // Room size: 3-6 x 3-4
    let width = 3 + rng.rn2(4) as usize;
    let height = 3 + rng.rn2(2) as usize;

    // Try to find a valid position
    for _ in 0..50 {
        let x = 3 + rng.rn2((COLNO - width - 6) as u32) as usize;
        let y = 2 + rng.rn2((ROWNO - height - 4) as u32) as usize;

        // Check if area is suitable (mostly walls)
        let mut wall_count = 0;
        let mut total = 0;
        for rx in x..x + width {
            for ry in y..y + height {
                total += 1;
                if level.cells[rx][ry].typ == CellType::Stone {
                    wall_count += 1;
                }
            }
        }

        // Need at least 60% walls to place room
        if wall_count * 100 / total >= 60 {
            // Carve the room
            for rx in x..x + width {
                for ry in y..y + height {
                    level.cells[rx][ry].typ = CellType::Room;
                    level.cells[rx][ry].lit = true;
                }
            }

            // Connect room to maze (find adjacent corridor)
            connect_room_to_maze(level, x, y, width, height);

            return Some(Room::new(x, y, width, height));
        }
    }

    None
}

/// Connect a room to the maze by finding/creating a door
fn connect_room_to_maze(level: &mut Level, x: usize, y: usize, width: usize, height: usize) {
    // Check each edge for adjacent corridor
    let edges = [
        // Top edge
        (x..x + width)
            .map(|rx| (rx, y.saturating_sub(1), rx, y))
            .collect::<Vec<_>>(),
        // Bottom edge
        (x..x + width)
            .map(|rx| (rx, y + height, rx, y + height - 1))
            .collect::<Vec<_>>(),
        // Left edge
        (y..y + height)
            .map(|ry| (x.saturating_sub(1), ry, x, ry))
            .collect::<Vec<_>>(),
        // Right edge
        (y..y + height)
            .map(|ry| (x + width, ry, x + width - 1, ry))
            .collect::<Vec<_>>(),
    ];

    for edge in edges {
        for (cx, cy, _rx, _ry) in edge {
            if cx < COLNO && cy < ROWNO && level.cells[cx][cy].typ == CellType::Corridor {
                // Found adjacent corridor - this is our connection point
                return;
            }
        }
    }

    // No adjacent corridor found - carve a path to nearest corridor
    // (simplified: just make the wall at one edge a corridor)
    if x > 2 {
        level.cells[x - 1][y + height / 2].typ = CellType::Corridor;
    }
}

/// Place stairs in the maze
fn place_maze_stairs(level: &mut Level, rng: &mut GameRng) {
    // Find valid positions for stairs (corridor cells)
    let mut up_placed = false;
    let mut down_placed = false;

    // Try to place stairs far apart
    for _ in 0..100 {
        if up_placed && down_placed {
            break;
        }

        let x = 2 + rng.rn2((COLNO - 4) as u32) as usize;
        let y = 2 + rng.rn2((ROWNO - 4) as u32) as usize;

        if level.cells[x][y].typ != CellType::Corridor
            && level.cells[x][y].typ != CellType::Room
        {
            continue;
        }

        if !up_placed {
            level.cells[x][y].typ = CellType::Stairs;
            level.stairs.push(Stairway {
                x: x as i8,
                y: y as i8,
                destination: DLevel::new(level.dlevel.dungeon_num, level.dlevel.level_num - 1),
                up: true,
            });
            up_placed = true;
        } else if !down_placed {
            // Ensure some distance from up stairs
            if let Some(up_stair) = level.stairs.first() {
                let dist = ((x as i32 - up_stair.x as i32).abs()
                    + (y as i32 - up_stair.y as i32).abs()) as usize;
                if dist < 20 {
                    continue;
                }
            }

            level.cells[x][y].typ = CellType::Stairs;
            level.stairs.push(Stairway {
                x: x as i8,
                y: y as i8,
                destination: DLevel::new(level.dlevel.dungeon_num, level.dlevel.level_num + 1),
                up: false,
            });
            down_placed = true;
        }
    }
}

/// Place traps in the maze (more than regular levels)
fn place_maze_traps(level: &mut Level, depth: i32, rng: &mut GameRng) {
    // Mazes have more traps: rnd(depth) + 2
    let num_traps = (rng.rnd(depth.max(1) as u32) + 2) as usize;
    let num_traps = num_traps.min(15);

    for _ in 0..num_traps {
        for _ in 0..20 {
            let x = 2 + rng.rn2((COLNO - 4) as u32) as usize;
            let y = 2 + rng.rn2((ROWNO - 4) as u32) as usize;

            if level.cells[x][y].typ == CellType::Corridor
                && level.cells[x][y].typ != CellType::Stairs
                && !level.traps.iter().any(|t| t.x == x as i8 && t.y == y as i8)
            {
                let trap_type = select_maze_trap(depth, rng);
                level.add_trap(x as i8, y as i8, trap_type);
                break;
            }
        }
    }
}

/// Select trap type for maze (includes teleport traps and holes)
fn select_maze_trap(depth: i32, rng: &mut GameRng) -> TrapType {
    let traps = if depth >= 15 {
        vec![
            TrapType::Teleport,
            TrapType::Teleport,
            TrapType::FireTrap,
            TrapType::Pit,
            TrapType::SpikedPit,
            TrapType::Hole,
            TrapType::TrapDoor,
            TrapType::LandMine,
            TrapType::MagicTrap,
            TrapType::AntiMagic,
        ]
    } else {
        vec![
            TrapType::Teleport,
            TrapType::Pit,
            TrapType::SpikedPit,
            TrapType::Arrow,
            TrapType::Dart,
            TrapType::BearTrap,
            TrapType::SleepingGas,
        ]
    };

    let idx = rng.rn2(traps.len() as u32) as usize;
    traps[idx]
}

/// Place gold in the maze
fn place_maze_gold(level: &mut Level, depth: i32, rng: &mut GameRng) {
    use crate::object::{Object, ObjectClass, ObjectId};

    let num_piles = 2 + rng.rnd(3) as usize; // 3-5 gold piles

    for _ in 0..num_piles {
        for _ in 0..20 {
            let x = 2 + rng.rn2((COLNO - 4) as u32) as usize;
            let y = 2 + rng.rn2((ROWNO - 4) as u32) as usize;

            if level.cells[x][y].typ == CellType::Corridor
                || level.cells[x][y].typ == CellType::Room
            {
                let amount = (rng.rnd((20 + depth * 3).max(1) as u32) + 10) as i32;
                let mut gold = Object::new(ObjectId(0), 0, ObjectClass::Coin);
                gold.quantity = amount;
                gold.name = Some("gold piece".to_string());
                level.add_object(gold, x as i8, y as i8);
                break;
            }
        }
    }
}

/// Check if a level should be a maze
pub fn is_maze_level(dlevel: &DLevel) -> bool {
    // Gehennom levels are mazes (dungeon_num 1)
    if dlevel.dungeon_num == 1 {
        return true;
    }

    // Deep main dungeon levels (25+) can be mazes
    if dlevel.dungeon_num == 0 && dlevel.level_num >= 25 {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maze_generation() {
        let mut rng = GameRng::new(42);
        let dlevel = DLevel {
            dungeon_num: 1, // Gehennom
            level_num: 5,
        };
        let mut level = Level::new(dlevel);

        generate_maze(&mut level, &mut rng);

        // Count corridor cells
        let corridor_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|cell| cell.typ == CellType::Corridor)
            .count();

        println!("Maze has {} corridor cells", corridor_count);
        assert!(corridor_count > 100, "Maze should have many corridors");

        // Should have stairs
        assert!(!level.stairs.is_empty(), "Maze should have stairs");

        // Should have traps
        assert!(!level.traps.is_empty(), "Maze should have traps");
    }

    #[test]
    fn test_maze_connectivity() {
        let mut rng = GameRng::new(123);
        let dlevel = DLevel {
            dungeon_num: 1,
            level_num: 10,
        };
        let mut level = Level::new(dlevel);

        generate_maze(&mut level, &mut rng);

        // Find up stairs
        let up_stair = level.stairs.iter().find(|s| s.up);
        let down_stair = level.stairs.iter().find(|s| !s.up);

        assert!(up_stair.is_some(), "Should have up stairs");
        assert!(down_stair.is_some(), "Should have down stairs");

        // Stairs should be reasonably far apart
        if let (Some(up), Some(down)) = (up_stair, down_stair) {
            let dist = (up.x as i32 - down.x as i32).abs() + (up.y as i32 - down.y as i32).abs();
            println!("Stair distance: {}", dist);
            assert!(dist >= 10, "Stairs should be at least 10 apart");
        }
    }

    #[test]
    fn test_is_maze_level() {
        // Gehennom is always maze
        assert!(is_maze_level(&DLevel {
            dungeon_num: 1,
            level_num: 1
        }));

        // Deep main dungeon is maze
        assert!(is_maze_level(&DLevel {
            dungeon_num: 0,
            level_num: 25
        }));

        // Shallow main dungeon is not maze
        assert!(!is_maze_level(&DLevel {
            dungeon_num: 0,
            level_num: 10
        }));
    }

    #[test]
    fn test_maze_rooms() {
        let mut rng = GameRng::new(999);
        let dlevel = DLevel {
            dungeon_num: 1,
            level_num: 5,
        };
        let mut level = Level::new(dlevel);

        generate_maze(&mut level, &mut rng);

        // Count room cells
        let room_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|cell| cell.typ == CellType::Room)
            .count();

        println!("Maze has {} room cells", room_count);
        // Mazes may or may not have rooms depending on RNG
    }
}
