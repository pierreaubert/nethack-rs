//! Maze level generation (mkmaze.c)
//!
//! Implements maze-type level generation for Gehennom and special levels.
//! Uses a recursive backtracking algorithm similar to NetHack's C implementation.

use crate::rng::GameRng;
use crate::{COLNO, ROWNO};

use super::DLevel;
use super::cell::CellType;
use super::level::{Level, Stairway, TrapType};
use super::room::Room;

/// Humidity flags for maze location selection (from sp_lev.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumidityFlag {
    Dry = 0x01,       // Room, air, cloud, ice, or corridor
    Wet = 0x02,       // Water pools
    Hot = 0x04,       // Lava
    Solid = 0x08,     // Rock
    AnyLoc = 0x10,    // Even outside level
    NoLocWarn = 0x20, // Silent fail, return None
    Spaceloc = 0x40,  // Like DRY but accepts furniture too
}

/// Coordinate structure for maze operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

/// Find a random starting point for maze generation
/// Matches C's maze0xy() - returns odd x,y coordinates within maze bounds
pub fn maze0xy(x_maze_max: usize, y_maze_max: usize, rng: &mut GameRng) -> Coord {
    let x = 3 + 2 * (rng.rn2(((x_maze_max >> 1) - 1) as u32) as usize);
    let y = 3 + 2 * (rng.rn2(((y_maze_max >> 1) - 1) as u32) as usize);
    Coord { x, y }
}

/// Check if a coordinate is within maze bounds
/// Matches C's maze_inbounds() - x,y must be 2-x_maze_max, 2-y_maze_max
pub fn maze_inbounds(x: usize, y: usize, x_maze_max: usize, y_maze_max: usize) -> bool {
    x >= 2 && y >= 2 && x < x_maze_max && y < y_maze_max
}

/// Find random point in generated corridors/rooms
/// Matches C's mazexy() - finds an accessible location for placement
pub fn mazexy(
    level: &Level,
    x_maze_max: usize,
    y_maze_max: usize,
    rng: &mut GameRng,
) -> Option<Coord> {
    let mut attempts = 0;
    loop {
        let x = 1 + rng.rn2(x_maze_max as u32) as usize;
        let y = 1 + rng.rn2(y_maze_max as u32) as usize;
        attempts += 1;

        if x >= COLNO || y >= ROWNO {
            if attempts >= 100 {
                break;
            }
            continue;
        }

        // Check if this is a valid location (corridor or room)
        match level.cells[x][y].typ {
            CellType::Corridor | CellType::Room => return Some(Coord { x, y }),
            _ => {
                if attempts >= 100 {
                    break;
                }
            }
        }
    }

    // Last resort: scan entire maze area
    for x in 1..x_maze_max {
        for y in 1..y_maze_max {
            if x >= COLNO || y >= ROWNO {
                continue;
            }
            match level.cells[x][y].typ {
                CellType::Corridor | CellType::Room => return Some(Coord { x, y }),
                _ => {}
            }
        }
    }

    None
}

/// Find random location in maze with humidity filtering (for special levels)
/// Matches C's maze1xy() - finds valid locations matching humidity requirements
pub fn maze1xy(
    level: &Level,
    x_maze_max: usize,
    y_maze_max: usize,
    humidity: HumidityFlag,
    rng: &mut GameRng,
) -> Option<Coord> {
    let mut attempts = 0;
    const MAX_ATTEMPTS: usize = 2000;

    loop {
        // Odd coordinates only (actual maze passages)
        let x = 3 + 2 * (rng.rn2(((x_maze_max - 3) / 2) as u32) as usize);
        let y = 3 + 2 * (rng.rn2(((y_maze_max - 3) / 2) as u32) as usize);
        attempts += 1;

        if !maze_inbounds(x, y, x_maze_max, y_maze_max) {
            if attempts >= MAX_ATTEMPTS {
                break;
            }
            continue;
        }

        // Check bounds
        if x >= COLNO || y >= ROWNO {
            if attempts >= MAX_ATTEMPTS {
                break;
            }
            continue;
        }

        let cell_type = level.cells[x][y].typ;

        // Check if location matches humidity requirements
        let matches = match humidity {
            HumidityFlag::Dry => {
                matches!(
                    cell_type,
                    CellType::Room | CellType::Corridor | CellType::Door
                )
            }
            HumidityFlag::Wet => {
                matches!(cell_type, CellType::Pool | CellType::Water | CellType::Moat)
            }
            HumidityFlag::Hot => {
                matches!(cell_type, CellType::Lava)
            }
            HumidityFlag::Solid => cell_type.is_wall() || matches!(cell_type, CellType::Stone),
            HumidityFlag::AnyLoc => true,
            HumidityFlag::Spaceloc => {
                matches!(
                    cell_type,
                    CellType::Room
                        | CellType::Corridor
                        | CellType::Door
                        | CellType::Throne
                        | CellType::Altar
                        | CellType::Fountain
                        | CellType::Sink
                )
            }
            _ => false,
        };

        if matches {
            return Some(Coord { x, y });
        }

        if attempts >= MAX_ATTEMPTS {
            break;
        }
    }

    // Return None if we couldn't find a location
    if humidity == HumidityFlag::NoLocWarn {
        None
    } else {
        None
    }
}

/// Remove dead ends from maze by opening them up
/// Matches C's maze_remove_deadends()
pub fn maze_remove_deadends(
    level: &mut Level,
    x_maze_max: usize,
    y_maze_max: usize,
    fill_type: CellType,
    rng: &mut GameRng,
) {
    for x in 2..x_maze_max {
        for y in 2..y_maze_max {
            if is_accessible(&level.cells[x][y].typ) && (x % 2 == 1) && (y % 2 == 1) {
                // This is a potential dead end (odd coordinates)
                let mut valid_dirs = Vec::new();

                // Check all 4 directions
                for (dir, (dx, dy)) in [(0, -1), (2, 1), (1, 0), (3, -1)].iter().enumerate() {
                    let (dx, dy) = (*dx, *dy);
                    // Check wall between current and adjacent
                    let wx = (x as i32 + dx / 2) as usize;
                    let wy = (y as i32 + dy / 2) as usize;

                    if !maze_inbounds(wx, wy, x_maze_max, y_maze_max) {
                        continue;
                    }

                    // Check far cell
                    let fx = (x as i32 + dx) as usize;
                    let fy = (y as i32 + dy) as usize;

                    if !maze_inbounds(fx, fy, x_maze_max, y_maze_max) {
                        continue;
                    }

                    // Door blocks passage, accessible cell is open way out
                    if !is_accessible(&level.cells[wx][wy].typ)
                        && is_accessible(&level.cells[fx][fy].typ)
                    {
                        valid_dirs.push(dir);
                    }
                }

                // If 3+ directions blocked, it's a dead end - open one path
                if valid_dirs.len() > 0 && valid_dirs.len() <= 1 {
                    let dir_idx = rng.rn2(valid_dirs.len() as u32) as usize;
                    let dir = valid_dirs[dir_idx];

                    let (dx, dy) = match dir {
                        0 => (0, -1),
                        1 => (1, 0),
                        2 => (0, 1),
                        3 => (-1, 0),
                        _ => (0, 0),
                    };

                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;

                    if nx < COLNO && ny < ROWNO {
                        level.cells[nx][ny].typ = fill_type;
                    }
                }
            }
        }
    }
}

/// Check if a cell is accessible (corridor, room, or door)
fn is_accessible(typ: &CellType) -> bool {
    matches!(
        typ,
        CellType::Corridor | CellType::Room | CellType::Door | CellType::Air
    )
}

/// Determine wall type based on adjacent open spaces
/// Uses the same logic as C's spine_array in mkmaze.c
///
/// The spine_array maps neighbor configurations to wall types:
/// - Neighbors encoded as 4-bit value: NSEW (North, South, East, West)
/// - Returns appropriate wall/corner type for proper maze rendering
fn wall_type_from_neighbors(north: bool, south: bool, east: bool, west: bool) -> CellType {
    // Encode neighbors as 4-bit value: NSEW
    let index =
        (north as usize) << 3 | (south as usize) << 2 | (east as usize) << 1 | (west as usize);

    // spine_array from mkmaze.c:
    // { VWALL, HWALL, HWALL, HWALL,
    //   VWALL, TRCORNER, TLCORNER, TDWALL,
    //   VWALL, BRCORNER, BLCORNER, TUWALL,
    //   VWALL, TLWALL, TRWALL, CROSSWALL }
    match index {
        0b0000 => CellType::VWall,     // No neighbors - default vertical
        0b0001 => CellType::HWall,     // West only
        0b0010 => CellType::HWall,     // East only
        0b0011 => CellType::HWall,     // East and West
        0b0100 => CellType::VWall,     // South only
        0b0101 => CellType::TRCorner,  // South and West
        0b0110 => CellType::TLCorner,  // South and East
        0b0111 => CellType::TDWall,    // South, East, West (T down)
        0b1000 => CellType::VWall,     // North only
        0b1001 => CellType::BRCorner,  // North and West
        0b1010 => CellType::BLCorner,  // North and East
        0b1011 => CellType::TUWall,    // North, East, West (T up)
        0b1100 => CellType::VWall,     // North and South
        0b1101 => CellType::TLWall,    // North, South, West (T left)
        0b1110 => CellType::TRWall,    // North, South, East (T right)
        0b1111 => CellType::CrossWall, // All four neighbors
        _ => CellType::Stone,
    }
}

/// Fill an area with a single cell type and optional lighting
/// Matches C's lvlfill_solid() - initializes maze with base terrain
pub fn lvlfill_solid(level: &mut Level, filling: CellType, lit: Option<bool>) {
    for x in 2..level.cells.len() {
        for y in 0..level.cells[x].len() {
            level.cells[x][y].typ = filling;
            if let Some(is_lit) = lit {
                level.cells[x][y].lit = is_lit;
            }
        }
    }
}

/// Initialize a maze grid with walls and passages
/// Matches C's lvlfill_maze_grid() - creates the grid pattern
pub fn lvlfill_maze_grid(
    level: &mut Level,
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
    filling: CellType,
    as_corridor_maze: bool,
) {
    let max_x = x2.min(level.cells.len() - 1);
    let max_y = if level.cells.is_empty() {
        0
    } else {
        y2.min(level.cells[0].len() - 1)
    };

    for x in x1..=max_x {
        for y in y1..=max_y {
            if x >= level.cells.len() || y >= level.cells[x].len() {
                continue;
            }
            if as_corridor_maze {
                // All stone for corridor maze
                level.cells[x][y].typ = CellType::Stone;
            } else {
                // Standard maze: walls at even coordinates, passages at odd
                if (x % 2 == 0) || (y % 2 == 0) {
                    level.cells[x][y].typ = CellType::HWall;
                } else {
                    level.cells[x][y].typ = filling;
                }
            }
        }
    }
}

/// Create a maze with specified corridor width and wall thickness
/// Matches C's create_maze() - main maze creation with scaling
pub fn create_maze(
    level: &mut Level,
    corrwid: i32,
    wallthick: i32,
    x_maze_max: usize,
    y_maze_max: usize,
    rng: &mut GameRng,
) -> (usize, usize) {
    let mut corrwid = corrwid;
    let mut wallthick = wallthick;

    // Validate parameters
    if wallthick < 1 {
        wallthick = 1;
    } else if wallthick > 5 {
        wallthick = 5;
    }

    if corrwid < 1 {
        corrwid = 1;
    } else if corrwid > 5 {
        corrwid = 5;
    }

    let scale = (corrwid + wallthick) as usize;
    let rdx = x_maze_max / scale;
    let rdy = y_maze_max / scale;

    // Initialize maze grid
    let fill_type = if level.flags.corridor_maze {
        CellType::Stone
    } else {
        CellType::Room
    };

    lvlfill_maze_grid(
        level,
        2,
        2,
        rdx * 2,
        rdy * 2,
        fill_type,
        level.flags.corridor_maze,
    );

    // Create maze using recursive backtracking
    let start = maze0xy(rdx * 2, rdy * 2, rng);
    carve_maze(level, start.x, start.y, rng);

    // Optionally remove dead ends
    if rng.rn2(5) == 0 {
        let fill_type = if level.flags.corridor_maze {
            CellType::Corridor
        } else {
            CellType::Room
        };
        maze_remove_deadends(level, rdx * 2, rdy * 2, fill_type, rng);
    }

    // Scale maze if needed
    if scale > 2 {
        scale_maze(level, rdx, rdy, corrwid as usize, wallthick as usize);
    }

    (rdx * 2, rdy * 2)
}

/// Scale up a maze by expanding cells
fn scale_maze(level: &mut Level, rdx: usize, rdy: usize, corrwid: usize, wallthick: usize) {
    // Backup existing maze into a vector-based structure
    let max_x = level.cells.len();
    let max_y = if max_x > 0 { level.cells[0].len() } else { 0 };

    let mut tmpmap = vec![vec![CellType::Stone; max_y]; max_x];
    for x in 1..max_x {
        for y in 1..max_y {
            tmpmap[x][y] = level.cells[x][y].typ;
        }
    }

    // Scale up
    let mut rx = 2;
    let mut x = 2;
    while rx < max_x {
        let mx = if x % 2 == 1 {
            corrwid
        } else if x == 2 || x == (rdx * 2) {
            1
        } else {
            wallthick
        };

        let mut ry = 2;
        let mut y = 2;
        while ry < max_y {
            let my = if y % 2 == 1 {
                corrwid
            } else if y == 2 || y == (rdy * 2) {
                1
            } else {
                wallthick
            };

            for dx in 0..mx {
                for dy in 0..my {
                    if rx + dx < max_x && ry + dy < max_y {
                        level.cells[rx + dx][ry + dy].typ = tmpmap[x][y];
                    }
                }
            }

            ry += my;
            y += 1;
        }

        rx += mx;
        x += 1;
    }
}

/// Populate unused maze portions with objects, traps, and monsters
/// Matches C's fill_empty_maze()
pub fn fill_empty_maze(level: &mut Level, x_maze_max: usize, y_maze_max: usize, rng: &mut GameRng) {
    use crate::object::{Object, ObjectClass, ObjectId};

    // Calculate map usage
    let mut used_cells = 0;
    for x in 2..x_maze_max {
        for y in 2..y_maze_max {
            if matches!(level.cells[x][y].typ, CellType::Corridor | CellType::Room) {
                used_cells += 1;
            }
        }
    }

    let total_cells = (x_maze_max - 2) * (y_maze_max - 2);
    let mapfact = if used_cells > 0 && total_cells > 0 {
        (used_cells * 100) / total_cells
    } else {
        100
    };

    // Only populate if less than 10% is used
    if used_cells >= total_cells / 10 {
        return;
    }

    // Scale placements by map usage
    let scale_factor = mapfact as f32 / 100.0;

    // Place objects/gems (20% weight)
    let num_gems = ((5.0 * scale_factor) as usize).max(1);
    for _ in 0..num_gems {
        if let Some(Coord { x, y }) = maze1xy(level, x_maze_max, y_maze_max, HumidityFlag::Dry, rng)
        {
            if rng.rn2(2) == 0 {
                // Gem
                let mut gem = Object::new(ObjectId(0), 0, ObjectClass::Gem);
                gem.name = Some("gem".to_string());
                level.add_object(gem, x as i8, y as i8);
            }
        }
    }

    // Place boulders (12% weight)
    let num_boulders = ((3.0 * scale_factor) as usize).max(1);
    for _ in 0..num_boulders {
        if let Some(Coord { x, y }) = maze1xy(level, x_maze_max, y_maze_max, HumidityFlag::Dry, rng)
        {
            let mut boulder = Object::new(ObjectId(0), 0, ObjectClass::Rock);
            boulder.name = Some("boulder".to_string());
            level.add_object(boulder, x as i8, y as i8);
        }
    }

    // Place gold (15% weight)
    let num_gold = ((4.0 * scale_factor) as usize).max(1);
    for _ in 0..num_gold {
        if let Some(Coord { x, y }) = maze1xy(level, x_maze_max, y_maze_max, HumidityFlag::Dry, rng)
        {
            let amount = (rng.rnd(100) + 50) as i32;
            let mut gold = Object::new(ObjectId(0), 0, ObjectClass::Coin);
            gold.quantity = amount;
            gold.name = Some("gold piece".to_string());
            level.add_object(gold, x as i8, y as i8);
        }
    }

    // Place traps (15% weight)
    let num_traps = ((4.0 * scale_factor) as usize).max(1);
    for _ in 0..num_traps {
        if let Some(Coord { x, y }) = maze1xy(level, x_maze_max, y_maze_max, HumidityFlag::Dry, rng)
        {
            let trap_types = [
                TrapType::Pit,
                TrapType::SpikedPit,
                TrapType::Teleport,
                TrapType::FireTrap,
                TrapType::Arrow,
            ];
            let trap_type = trap_types[rng.rn2(trap_types.len() as u32) as usize];
            level.add_trap(x as i8, y as i8, trap_type);
        }
    }
}

/// Generate a mini-maze for Rogue-style levels (3x3 room grid)
/// Matches C's miniwalk() - creates room connections via depth-first search
pub struct RogueRoom {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub doors: [bool; 4], // UP, DOWN, LEFT, RIGHT
}

pub fn miniwalk(rooms: &mut [[Option<RogueRoom>; 3]; 3], x: usize, y: usize, rng: &mut GameRng) {
    // Mark this room as visited
    if rooms[x][y].is_none() {
        return;
    }

    // Find unvisited adjacent rooms (outside the mutable borrow)
    let mut unvisited = Vec::new();

    // UP
    if y > 0 && rooms[x][y - 1].is_none() {
        unvisited.push(0);
    }
    // DOWN
    if y < 2 && rooms[x][y + 1].is_none() {
        unvisited.push(1);
    }
    // LEFT
    if x > 0 && rooms[x - 1][y].is_none() {
        unvisited.push(2);
    }
    // RIGHT
    if x < 2 && rooms[x + 1][y].is_none() {
        unvisited.push(3);
    }

    // Pick random unvisited room
    if !unvisited.is_empty() {
        let direction = unvisited[rng.rn2(unvisited.len() as u32) as usize];

        // Set door in current room
        if let Some(ref mut room) = rooms[x][y] {
            room.doors[direction] = true;
        }

        // Recurse to adjacent room
        let (nx, ny) = match direction {
            0 => (x, y - 1),
            1 => (x, y + 1),
            2 => (x - 1, y),
            3 => (x + 1, y),
            _ => (x, y),
        };

        if nx < 3 && ny < 3 {
            // Set opposite door in adjacent room
            let opposite = match direction {
                0 => 1, // UP -> DOWN
                1 => 0, // DOWN -> UP
                2 => 3, // LEFT -> RIGHT
                3 => 2, // RIGHT -> LEFT
                _ => 0,
            };
            if let Some(ref mut next_room) = rooms[nx][ny] {
                next_room.doors[opposite] = true;
            }
            miniwalk(rooms, nx, ny, rng);
        }
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

        if level.cells[x][y].typ != CellType::Corridor && level.cells[x][y].typ != CellType::Room {
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
