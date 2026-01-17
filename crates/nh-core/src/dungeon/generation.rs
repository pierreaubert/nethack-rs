//! Level generation (mklev.c, mkroom.c)
//!
//! Generates dungeon levels with rooms and corridors.

use crate::monster::{Monster, MonsterId};
use crate::rng::GameRng;
use crate::{COLNO, ROWNO};

use super::{CellType, DLevel, DoorState, Level};

/// Rectangle representing a room
#[derive(Debug, Clone, Copy)]
pub struct Room {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Room {
    /// Check if this room overlaps with another (with buffer)
    fn overlaps(&self, other: &Room, buffer: usize) -> bool {
        let x1 = self.x.saturating_sub(buffer);
        let y1 = self.y.saturating_sub(buffer);
        let x2 = self.x + self.width + buffer;
        let y2 = self.y + self.height + buffer;

        let ox1 = other.x.saturating_sub(buffer);
        let oy1 = other.y.saturating_sub(buffer);
        let ox2 = other.x + other.width + buffer;
        let oy2 = other.y + other.height + buffer;

        !(x2 <= ox1 || x1 >= ox2 || y2 <= oy1 || y1 >= oy2)
    }

    /// Get center point of room
    pub fn center(&self) -> (usize, usize) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    /// Check if point is inside room
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Get a random point inside the room
    pub fn random_point(&self, rng: &mut GameRng) -> (usize, usize) {
        let x = self.x + rng.rn2(self.width as u32) as usize;
        let y = self.y + rng.rn2(self.height as u32) as usize;
        (x, y)
    }
}

/// Generate a standard level with rooms and corridors
pub fn generate_rooms_and_corridors(level: &mut Level, rng: &mut GameRng) {
    // Try to place 6-9 rooms (rnd returns 1..=n, so rnd(4) + 5 = 6-9)
    let num_rooms = (rng.rnd(4) + 5) as usize;
    let mut rooms = Vec::new();

    // Try to place rooms
    for _ in 0..num_rooms * 3 {
        // Try multiple times
        let width = (rng.rnd(7) + 2) as usize; // 3-9
        let height = (rng.rnd(5) + 2) as usize; // 3-7

        // Leave room for walls
        let max_x = COLNO.saturating_sub(width + 2);
        let max_y = ROWNO.saturating_sub(height + 2);

        if max_x < 2 || max_y < 2 {
            continue;
        }

        let x = (rng.rn2(max_x.saturating_sub(1) as u32) + 1) as usize;
        let y = (rng.rn2(max_y.saturating_sub(1) as u32) + 1) as usize;

        let room = Room {
            x,
            y,
            width,
            height,
        };

        // Check if room overlaps with existing rooms
        if rooms.iter().any(|r: &Room| room.overlaps(r, 1)) {
            continue;
        }

        rooms.push(room);

        if rooms.len() >= num_rooms {
            break;
        }
    }

    // Carve out rooms
    for room in &rooms {
        // Create walls around the room
        for x in room.x.saturating_sub(1)..=(room.x + room.width).min(COLNO - 1) {
            for y in room.y.saturating_sub(1)..=(room.y + room.height).min(ROWNO - 1) {
                let is_vertical_edge = x == room.x.saturating_sub(1) || x == room.x + room.width;
                let is_horizontal_edge = y == room.y.saturating_sub(1) || y == room.y + room.height;

                if is_vertical_edge && !is_horizontal_edge {
                    level.cells[x][y].typ = CellType::VWall;
                } else if is_horizontal_edge && !is_vertical_edge {
                    level.cells[x][y].typ = CellType::HWall;
                } else if is_vertical_edge && is_horizontal_edge {
                    // Corner - use HWall for simplicity
                    level.cells[x][y].typ = CellType::HWall;
                } else {
                    level.cells[x][y].typ = CellType::Room;
                    level.cells[x][y].lit = true;
                }
            }
        }
    }

    // Connect rooms with corridors
    connect_rooms(level, &rooms, rng);

    // Place doors
    place_doors(level, &rooms, rng);

    // Place stairs
    if !rooms.is_empty() {
        place_stairs(level, &rooms, rng);
    }

    // Place monsters
    place_monsters(level, &rooms, rng);
}

/// Connect rooms with corridors using a simple approach
fn connect_rooms(level: &mut Level, rooms: &[Room], rng: &mut GameRng) {
    if rooms.is_empty() {
        return;
    }

    // Connect each room to the next one
    for i in 0..rooms.len() {
        let next_i = (i + 1) % rooms.len();
        let (x1, y1) = rooms[i].center();
        let (x2, y2) = rooms[next_i].center();

        // Create L-shaped corridor
        if rng.one_in(2) {
            // Horizontal then vertical
            dig_horizontal_corridor(level, x1, x2, y1);
            dig_vertical_corridor(level, y1, y2, x2);
        } else {
            // Vertical then horizontal
            dig_vertical_corridor(level, y1, y2, x1);
            dig_horizontal_corridor(level, x1, x2, y2);
        }
    }
}

/// Dig a horizontal corridor
fn dig_horizontal_corridor(level: &mut Level, x1: usize, x2: usize, y: usize) {
    let (start, end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };

    for x in start..=end {
        if x >= COLNO || y >= ROWNO {
            continue;
        }

        let cell = &mut level.cells[x][y];
        if cell.typ == CellType::Stone {
            cell.typ = CellType::Corridor;
        }
    }
}

/// Dig a vertical corridor
fn dig_vertical_corridor(level: &mut Level, y1: usize, y2: usize, x: usize) {
    let (start, end) = if y1 < y2 { (y1, y2) } else { (y2, y1) };

    for y in start..=end {
        if x >= COLNO || y >= ROWNO {
            continue;
        }

        let cell = &mut level.cells[x][y];
        if cell.typ == CellType::Stone {
            cell.typ = CellType::Corridor;
        }
    }
}

/// Place doors at room entrances
fn place_doors(level: &mut Level, rooms: &[Room], rng: &mut GameRng) {
    for room in rooms {
        // Check each wall position for potential door placement
        for x in room.x..room.x + room.width {
            // Top wall
            if room.y > 0 {
                check_and_place_door(level, x, room.y - 1, rng);
            }
            // Bottom wall
            if room.y + room.height < ROWNO {
                check_and_place_door(level, x, room.y + room.height, rng);
            }
        }

        for y in room.y..room.y + room.height {
            // Left wall
            if room.x > 0 {
                check_and_place_door(level, room.x - 1, y, rng);
            }
            // Right wall
            if room.x + room.width < COLNO {
                check_and_place_door(level, room.x + room.width, y, rng);
            }
        }
    }
}

/// Check if a door should be placed at this position
fn check_and_place_door(level: &mut Level, x: usize, y: usize, rng: &mut GameRng) {
    if x >= COLNO || y >= ROWNO {
        return;
    }

    let cell = &level.cells[x][y];

    // Only place doors on walls
    if !cell.typ.is_wall() {
        return;
    }

    // Check if there's a corridor adjacent
    let has_corridor = [(-1, 0), (1, 0), (0, -1), (0, 1)]
        .iter()
        .any(|(dx, dy)| {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && ny >= 0 && (nx as usize) < COLNO && (ny as usize) < ROWNO {
                level.cells[nx as usize][ny as usize].typ == CellType::Corridor
            } else {
                false
            }
        });

    if has_corridor {
        // 80% chance to place a door
        if rng.percent(80) {
            level.cells[x][y].typ = CellType::Door;
            // 90% closed, 10% open
            level.cells[x][y].set_door_state(if rng.one_in(10) {
                DoorState::OPEN
            } else {
                DoorState::CLOSED
            });
        } else {
            // Make it a corridor opening instead
            level.cells[x][y].typ = CellType::Corridor;
        }
    }
}

/// Place stairs in the level
fn place_stairs(level: &mut Level, rooms: &[Room], rng: &mut GameRng) {
    if rooms.is_empty() {
        return;
    }

    // Place upstairs in first room
    let (ux, uy) = rooms[0].random_point(rng);
    level.cells[ux][uy].typ = CellType::Stairs;
    level.stairs.push(super::Stairway {
        x: ux as i8,
        y: uy as i8,
        destination: DLevel {
            dungeon_num: level.dlevel.dungeon_num,
            level_num: level.dlevel.level_num - 1,
        },
        up: true,
    });

    // Place downstairs in last room (avoid same room if possible)
    let down_room_idx = if rooms.len() > 1 {
        rooms.len() - 1
    } else {
        0
    };

    let (dx, dy) = rooms[down_room_idx].random_point(rng);
    // Make sure we don't place on same cell as upstairs
    if dx != ux || dy != uy {
        level.cells[dx][dy].typ = CellType::Stairs;
        level.stairs.push(super::Stairway {
            x: dx as i8,
            y: dy as i8,
            destination: DLevel {
                dungeon_num: level.dlevel.dungeon_num,
                level_num: level.dlevel.level_num + 1,
            },
            up: false,
        });
    }
}

/// Place monsters in the level
fn place_monsters(level: &mut Level, rooms: &[Room], rng: &mut GameRng) {
    if rooms.is_empty() {
        return;
    }

    // Spawn 3-8 monsters
    let num_monsters = (rng.rnd(6) + 2) as usize; // 3-8

    for _ in 0..num_monsters {
        // Pick a random room (not the first room where stairs are)
        let room_idx = if rooms.len() > 1 {
            rng.rn2(rooms.len() as u32 - 1) as usize + 1
        } else {
            0
        };

        let room = rooms[room_idx];
        let (x, y) = room.random_point(rng);

        // Check if position is empty
        if level.monster_at(x as i8, y as i8).is_some() {
            continue; // Skip if occupied
        }

        // Create a basic monster (data will be populated by nethack binary)
        let mut monster = Monster::new(MonsterId(0), rng.rn2(10) as i16, x as i8, y as i8);
        monster.state = crate::monster::MonsterState::active();
        monster.hp = 5 + rng.rnd(10) as i32;
        monster.hp_max = monster.hp;
        monster.name = format!("Monster {}", rng.rn2(100));

        // Add to level
        level.add_monster(monster);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_overlap() {
        let room1 = Room {
            x: 5,
            y: 5,
            width: 5,
            height: 5,
        };

        let room2 = Room {
            x: 8,
            y: 8,
            width: 5,
            height: 5,
        };

        let room3 = Room {
            x: 15,
            y: 15,
            width: 5,
            height: 5,
        };

        assert!(room1.overlaps(&room2, 0));
        assert!(!room1.overlaps(&room3, 0));
        assert!(room1.overlaps(&room3, 15));
    }

    #[test]
    fn test_generation() {
        let mut rng = GameRng::new(12345);
        let mut level = Level::new(DLevel::main_dungeon_start());

        generate_rooms_and_corridors(&mut level, &mut rng);

        // Check that we have some room cells
        let room_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|cell| cell.typ == CellType::Room)
            .count();

        assert!(room_count > 0, "Should have generated some room cells");

        // Check that we have stairs
        assert!(!level.stairs.is_empty(), "Should have generated stairs");
    }
}
