//! Maze level generation (mkmaze.c)
//!
//! Implements maze-type level generation matching NetHack 3.6.7 precisely.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::rng::GameRng;
use crate::{COLNO, ROWNO};

use super::DLevel;
use super::cell::CellType;
use super::level::{Level, Stairway, TrapType};
use super::room::Room;

/// Coordinate structure for maze operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

/// Directions for maze movement (matching NetHack's internal order)
const MZ_DIRS: [(i32, i32); 4] = [
    (0, -1), // 0: North
    (1, 0),  // 1: East
    (0, 1),  // 2: South
    (-1, 0), // 3: West
];

/// Find a random starting point for maze generation
/// Matches C's maze0xy()
pub fn maze0xy(x_maze_max: usize, y_maze_max: usize, rng: &mut GameRng) -> Coord {
    let x = 3 + 2 * (rng.rn2(((x_maze_max >> 1) - 1) as u32) as usize);
    let y = 3 + 2 * (rng.rn2(((y_maze_max >> 1) - 1) as u32) as usize);
    Coord { x, y }
}

/// Find a random empty passage in the maze
/// Matches C's mazexy() in mkmaze.c
pub fn mazexy(level: &Level, rng: &mut GameRng) -> Coord {
    let x_maze_max = (COLNO - 1) & !1;
    let y_maze_max = (ROWNO - 1) & !1;
    let mut cpt = 0;
    
    loop {
        let x = 1 + rng.rn2(x_maze_max as u32) as usize;
        let y = 1 + rng.rn2(y_maze_max as u32) as usize;
        cpt += 1;
        
        let typ = &level.cells[x][y].typ;
        let is_pass = if level.flags.corridor_maze {
            *typ == CellType::Corridor
        } else {
            *typ == CellType::Room
        };
        
        if cpt >= 100 || is_pass {
            return Coord { x, y };
        }
    }
}

/// Place stairs on the level
/// Matches C's mkstairs() in mklev.c
pub fn mkstairs(level: &mut Level, x: usize, y: usize, up: bool) {
    if x == 0 { return; }
    
    // In NetHack, stairs are placed on the map. 
    level.cells[x][y].typ = CellType::Stairs;
    
    // In Rust Level, we also store them in the stairs vector
    level.stairs.push(super::level::Stairway {
        x: x as i8,
        y: y as i8,
        up,
        destination: DLevel::new(level.dlevel.dungeon_num, level.dlevel.level_num + if up { -1 } else { 1 }),
    });
}

/// Check if a coordinate is within maze bounds
/// Matches C's maze_inbounds() in mkmaze.c
pub fn maze_inbounds(x: usize, y: usize, x_maze_max: usize, y_maze_max: usize) -> bool {
    x >= 2 && y >= 2 && x < x_maze_max && y < y_maze_max
}

/// Check if it's okay to move into a cell during maze walk
/// Matches C's okay() in mkmaze.c
fn okay(level: &Level, x: i32, y: i32, dir: usize, x_maze_max: usize, y_maze_max: usize) -> bool {
    let mut nx = x + MZ_DIRS[dir].0;
    let mut ny = y + MZ_DIRS[dir].1;
    
    // mz_move(x, y, dir); // first step
    
    // mz_move(x, y, dir); // second step
    nx += MZ_DIRS[dir].0;
    ny += MZ_DIRS[dir].1;
    
    if nx < 3 || ny < 3 || nx > (x_maze_max as i32) || ny > (y_maze_max as i32) {
        return false;
    }
    
    // Must be stone to be carveable
    level.cells[nx as usize][ny as usize].typ == CellType::Stone
}

/// Precise port of walkfrom(x, y, typ) from mkmaze.c
pub fn walkfrom(
    level: &mut Level,
    mut x: usize,
    mut y: usize,
    typ: CellType,
    x_maze_max: usize,
    y_maze_max: usize,
    rng: &mut GameRng,
) {
    let mut stack = Vec::with_capacity(COLNO * ROWNO / 4);
    stack.push((x, y));

    // NetHack's typ logic: if !typ, use CORR if corrmaze, else ROOM
    let fill_typ = if typ == CellType::Stone {
        if level.flags.corridor_maze {
            CellType::Corridor
        } else {
            CellType::Room
        }
    } else {
        typ
    };

    while let Some(&(curr_x, curr_y)) = stack.last() {
        x = curr_x;
        y = curr_y;

        // Set type of current cell
        if level.cells[x][y].typ != CellType::Door {
            level.cells[x][y].typ = fill_typ;
        }

        let mut valid_dirs = [0usize; 4];
        let mut q = 0;

        for a in 0..4 {
            if okay(level, x as i32, y as i32, a, x_maze_max, y_maze_max) {
                valid_dirs[q] = a;
                q += 1;
            }
        }

        if q == 0 {
            stack.pop();
        } else {
            // Pick random direction: dir = dirs[rn2(q)]
            let dir_idx = rng.rn2(q as u32) as usize;
            let dir = valid_dirs[dir_idx];

            // Carve two steps
            let dx = MZ_DIRS[dir].0;
            let dy = MZ_DIRS[dir].1;

            let mid_x = (x as i32 + dx) as usize;
            let mid_y = (y as i32 + dy) as usize;
            level.cells[mid_x][mid_y].typ = fill_typ;

            let next_x = (x as i32 + 2 * dx) as usize;
            let next_y = (y as i32 + 2 * dy) as usize;
            level.cells[next_x][next_y].typ = fill_typ;

            stack.push((next_x, next_y));
        }
    }
}

/// Create a maze with specified corridor width and wall thickness
/// Matches C's create_maze() in mkmaze.c
fn create_maze(
    level: &mut Level,
    mut corrwid: i32,
    mut wallthick: i32,
    rng: &mut GameRng,
) {
    if wallthick < 1 { wallthick = 1; }
    else if wallthick > 5 { wallthick = 5; }
    
    if corrwid < 1 { corrwid = 1; }
    else if corrwid > 5 { corrwid = 5; }
    
    let scale = (corrwid + wallthick) as usize;
    let x_maze_max = (COLNO - 1) & !1;
    let y_maze_max = (ROWNO - 1) & !1;
    
    let rdx = x_maze_max / scale;
    let rdy = y_maze_max / scale;
    
    let sub_xmax = rdx * 2;
    let sub_ymax = rdy * 2;
    
    // 1. Initial sub-maze initialization
    if level.flags.corridor_maze {
        for x in 2..sub_xmax {
            for y in 2..sub_ymax {
                level.cells[x][y].typ = CellType::Stone;
            }
        }
    } else {
        for x in 2..=sub_xmax {
            for y in 2..=sub_ymax {
                level.cells[x][y].typ = if (x % 2 != 0) && (y % 2 != 0) {
                    CellType::Stone
                } else {
                    CellType::HWall
                };
            }
        }
    }
    
    // 2. Walkfrom on sub-maze
    let mm = maze0xy(sub_xmax, sub_ymax, rng);
    walkfrom(level, mm.x, mm.y, CellType::Stone, sub_xmax, sub_ymax, rng);
    
    // Dead ends
    if rng.rn2(5) == 0 {
        let typ = if level.flags.corridor_maze { CellType::Corridor } else { CellType::Room };
        maze_remove_deadends(level, sub_xmax, sub_ymax, typ, rng);
    }
    
    // 3. Scaling up
    if scale > 2 {
        let mut tmpmap = [[CellType::Stone; ROWNO]; COLNO];
        for x in 2..=sub_xmax {
            for y in 2..=sub_ymax {
                tmpmap[x][y] = level.cells[x][y].typ;
            }
        }
        
        // Clear level first
        for x in 2..=x_maze_max {
            for y in 2..=y_maze_max {
                level.cells[x][y].typ = CellType::Stone;
            }
        }
        
        let mut rx = 2;
        let mut x = 2;
        while rx < x_maze_max {
            let mx = if x % 2 != 0 {
                corrwid as usize
            } else {
                if x == 2 || x == sub_xmax { 1 } else { wallthick as usize }
            };
            
            let mut ry = 2;
            let mut y = 2;
            while ry < y_maze_max {
                let my = if y % 2 != 0 {
                    corrwid as usize
                } else {
                    if y == 2 || y == sub_ymax { 1 } else { wallthick as usize }
                };
                
                for dx in 0..mx {
                    for dy in 0..my {
                        if rx + dx >= x_maze_max || ry + dy >= y_maze_max {
                            break;
                        }
                        level.cells[rx + dx][ry + dy].typ = tmpmap[x][y];
                    }
                }
                ry += my;
                y += 1;
            }
            rx += mx;
            x += 1;
        }
    }
}

/// Remove dead ends from maze
pub fn maze_remove_deadends(
    level: &mut Level,
    x_maze_max: usize,
    y_maze_max: usize,
    typ: CellType,
    rng: &mut GameRng,
) {
    let mut dirok = [0usize; 4];
    for x in 2..x_maze_max {
        for y in 2..y_maze_max {
            if is_accessible(&level.cells[x][y].typ) && (x % 2 != 0) && (y % 2 != 0) {
                let mut idx = 0;
                let mut idx2 = 0;
                for dir in 0..4 {
                    let mut dx = x as i32;
                    let mut dy = y as i32;
                    let mut dx2 = x as i32;
                    let mut dy2 = y as i32;
                    
                    // mz_move(dx, dy, dir)
                    dx += MZ_DIRS[dir].0;
                    dy += MZ_DIRS[dir].1;
                    
                    if !maze_inbounds(dx as usize, dy as usize, x_maze_max, y_maze_max) {
                        idx2 += 1;
                        continue;
                    }
                    
                    // mz_move(dx2, dy2, dir) * 2
                    dx2 += 2 * MZ_DIRS[dir].0;
                    dy2 += 2 * MZ_DIRS[dir].1;
                    
                    if !maze_inbounds(dx2 as usize, dy2 as usize, x_maze_max, y_maze_max) {
                        idx2 += 1;
                        continue;
                    }
                    
                    if !is_accessible(&level.cells[dx as usize][dy as usize].typ)
                        && is_accessible(&level.cells[dx2 as usize][dy2 as usize].typ) {
                        dirok[idx] = dir;
                        idx += 1;
                        idx2 += 1;
                    }
                }
                
                if idx2 >= 3 && idx > 0 {
                    let dir = dirok[rng.rn2(idx as u32) as usize];
                    let nx = (x as i32 + MZ_DIRS[dir].0) as usize;
                    let ny = (y as i32 + MZ_DIRS[dir].1) as usize;
                    level.cells[nx][ny].typ = typ;
                }
            }
        }
    }
}

/// Generate a maze level
pub fn generate_maze(level: &mut Level, is_invocation: bool, rng: &mut GameRng) {
    let depth = level.dlevel.depth();

    level.flags.is_maze = true;
    level.flags.corridor_maze = rng.rn2(3) == 0;

    // Fill with Stone
    for x in 0..COLNO {
        for y in 0..ROWNO {
            level.cells[x][y].typ = CellType::Stone;
            level.cells[x][y].lit = depth < 10;
        }
    }
    
    if !is_invocation && rng.rn2(2) != 0 {
        let corrwid = rng.rnd(4) as i32;
        let wallthick = rng.rnd(4) as i32 - corrwid;
        create_maze(level, corrwid, wallthick, rng);
    } else {
        create_maze(level, 1, 1, rng);
    }

    if !level.flags.corridor_maze {
        fix_maze_walls(level);
    }
    
    // Stairs (makemaz additions)
    let up_stair = mazexy(level, rng);
    mkstairs(level, up_stair.x, up_stair.y, true);
    
    if !is_invocation {
        let down_stair = mazexy(level, rng);
        mkstairs(level, down_stair.x, down_stair.y, false);
    }
}

/// Fix wall types based on adjacent passages
fn fix_maze_walls(level: &mut Level) {
    let mut wall_updates: Vec<(usize, usize, CellType)> = Vec::new();

    for x in 1..COLNO - 1 {
        for y in 1..ROWNO - 1 {
            if level.cells[x][y].typ.is_wall() || level.cells[x][y].typ == CellType::Stone {
                let north = is_passage(&level.cells[x][y - 1].typ);
                let south = is_passage(&level.cells[x][y + 1].typ);
                let east = is_passage(&level.cells[x + 1][y].typ);
                let west = is_passage(&level.cells[x - 1][y].typ);

                if north || south || east || west {
                    let wall_type = wall_type_from_neighbors(north, south, east, west);
                    wall_updates.push((x, y, wall_type));
                }
            }
        }
    }

    for (x, y, wall_type) in wall_updates {
        level.cells[x][y].typ = wall_type;
    }
}

fn is_passage(typ: &CellType) -> bool {
    matches!(typ, CellType::Corridor | CellType::Room | CellType::Door)
}

fn is_accessible(typ: &CellType) -> bool {
    matches!(
        typ,
        CellType::Corridor | CellType::Room | CellType::Door | CellType::Air
    )
}

fn wall_type_from_neighbors(north: bool, south: bool, east: bool, west: bool) -> CellType {
    let index = (north as usize) << 3 | (south as usize) << 2 | (east as usize) << 1 | (west as usize);
    match index {
        0b0000 => CellType::VWall,
        0b0001 => CellType::HWall,
        0b0010 => CellType::HWall,
        0b0011 => CellType::HWall,
        0b0100 => CellType::VWall,
        0b0101 => CellType::TRCorner,
        0b0110 => CellType::TLCorner,
        0b0111 => CellType::TDWall,
        0b1000 => CellType::VWall,
        0b1001 => CellType::BRCorner,
        0b1010 => CellType::BLCorner,
        0b1011 => CellType::TUWall,
        0b1100 => CellType::VWall,
        0b1101 => CellType::TLWall,
        0b1110 => CellType::TRWall,
        0b1111 => CellType::CrossWall,
        _ => CellType::Stone,
    }
}

/// Check if a level should be a maze
pub fn is_maze_level(dlevel: &DLevel) -> bool {
    if dlevel.dungeon_num == 1 { return true; }
    if dlevel.dungeon_num == 0 && dlevel.level_num >= 25 { return true; }
    false
}
