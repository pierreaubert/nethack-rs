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
use super::{CellType, DoorState, Level};

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
    /// C: if (smeq[a] < smeq[b]) smeq[b] = smeq[a]; else smeq[a] = smeq[b];
    /// Note: C only updates ONE entry, not all entries in the class.
    pub fn merge(&mut self, a: usize, b: usize) {
        if a >= self.smeq.len() || b >= self.smeq.len() {
            return;
        }

        if self.smeq[a] < self.smeq[b] {
            self.smeq[b] = self.smeq[a];
        } else {
            self.smeq[a] = self.smeq[b];
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

/// Check if there's a door next to a position (4 cardinal directions)
/// Matches C's bydoor()
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
#[allow(dead_code)]
pub fn nexttodoor(level: &Level, x: i32, y: i32) -> bool {
    for dx in -1..=1 {
        for dy in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
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
) -> (usize, usize) {
    // Try random position first (2 RNG calls)
    let x = xl + rng.rn2((xh.saturating_sub(xl) + 1).max(1) as u32) as usize;
    let y = yl + rng.rn2((yh.saturating_sub(yl) + 1).max(1) as u32) as usize;

    if okdoor(level, x as i32, y as i32) {
        return (x, y);
    }

    // Scan the area linearly
    for sx in xl..=xh {
        for sy in yl..=yh {
            if okdoor(level, sx as i32, sy as i32) {
                return (sx, sy);
            }
        }
    }

    // C: if (IS_DOOR(levl[x][y].typ) || levl[x][y].typ == SDOOR)
    for sx in xl..=xh {
        for sy in yl..=yh {
            let typ = level.cells[sx][sy].typ;
            if typ == CellType::Door || typ == CellType::SecretDoor {
                return (sx, sy);
            }
        }
    }

    // Last resort: return corner
    (xl, yh)
}

/// C's dodoor() - place a door with random type (mklev.c:1248-1258)
///
/// Decides whether door is regular or secret (rn2(8) ? DOOR : SDOOR),
/// then delegates to dosdoor().
fn dodoor(level: &mut Level, x: usize, y: usize, _room_idx: usize, rng: &mut GameRng) {
    let door_type = if rng.rn2(8) != 0 {
        CellType::Door
    } else {
        CellType::SecretDoor
    };
    dosdoor(level, x, y, door_type, rng);
}

/// C's dosdoor() - place door with specific type (mklev.c:385-449)
///
/// Sets door type and state based on C's logic including shop awareness.
/// Public wrapper for dosdoor — used by niche generation
pub fn dosdoor_public(level: &mut Level, x: usize, y: usize, door_type: CellType, rng: &mut GameRng) {
    dosdoor(level, x, y, door_type, rng);
}

fn dosdoor(level: &mut Level, x: usize, y: usize, mut door_type: CellType, rng: &mut GameRng) {
    if x >= COLNO || y >= ROWNO {
        return;
    }

    // shdoor = in_rooms(x,y,SHOPBASE) - check if in a shop
    // For now, simplified: check if adjacent room is a shop
    let shdoor = false; // TODO: full in_rooms check when room numbering is active

    // If not on a wall, force regular DOOR (avoid SDOOR on existing openings)
    if !level.cells[x][y].typ.is_wall() {
        door_type = CellType::Door;
    }

    let depth = level.dlevel.depth();

    level.cells[x][y].typ = door_type;

    if door_type == CellType::Door {
        // Regular door
        if rng.rn2(3) != 0 {
            // 67% chance: shop door is OPEN, otherwise NODOOR (empty doorway)
            if shdoor {
                level.cells[x][y].set_door_state(DoorState::OPEN);
            } else {
                level.cells[x][y].set_door_state(DoorState::NO_DOOR);
            }
        } else {
            // 33% chance: detailed state
            let state = if rng.rn2(5) == 0 {
                DoorState::OPEN
            } else if rng.rn2(6) == 0 {
                DoorState::LOCKED
            } else {
                DoorState::CLOSED
            };
            // Trap check: not open, not shop, depth >= 5, 4% chance
            if !state.contains(DoorState::OPEN) && !shdoor && depth >= 5 && rng.rn2(25) == 0 {
                level.cells[x][y].set_door_state(state | DoorState::TRAPPED);
            } else {
                level.cells[x][y].set_door_state(state);
            }
        }

        // C mklev.c:461-474 — D_TRAPPED mimic check
        if level.cells[x][y].door_state().contains(DoorState::TRAPPED) {
            if depth >= 9 && rng.rn2(5) == 0 {
                // TODO: actually create mimic monster and consume makemon RNG
                // C: makemon(mkclass(S_MIMIC, 0), x, y, NO_MM_FLAGS)
                // mkclass consumes RNG: rn2(num_monsters_in_class)
                // TODO: actually create mimic monster
                // For RNG parity, consume mkclass + makemon RNG
                level.cells[x][y].set_door_state(DoorState::NO_DOOR);
            }
        }

        // C: Rogue level check — skip for now (not rogue level at depth 14)
    } else {
        // Secret door
        let state = if shdoor || rng.rn2(5) == 0 {
            DoorState::LOCKED
        } else {
            DoorState::CLOSED
        };
        if !shdoor && depth >= 4 && rng.rn2(20) == 0 {
            level.cells[x][y].set_door_state(state | DoorState::TRAPPED);
        } else {
            level.cells[x][y].set_door_state(state);
        }
    }
}

/// Dig a corridor between two points.
/// 1:1 port of sp_lev.c dig_corridor() (lines 2218-2325)
///
/// Uses cell-type-aware lookahead: checks whether adjacent cells are
/// btyp (background, usually Stone), ftyp (foreground, usually Corridor),
/// or SCORR before changing direction.
fn dig_corridor_inner(
    level: &mut Level,
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    nxcor: bool,
    ftyp: CellType,
    btyp: CellType,
    rng: &mut GameRng,
) -> bool {
    let tx = end_x;
    let ty = end_y;
    let mut xx = start_x;
    let mut yy = start_y;

    // Bounds check (C: xx <= 0 || yy <= 0 || tx <= 0 || ty <= 0
    //   || xx > COLNO-1 || tx > COLNO-1 || yy > ROWNO-1 || ty > ROWNO-1)
    if xx <= 0
        || yy <= 0
        || tx <= 0
        || ty <= 0
        || xx > COLNO as i32 - 1
        || tx > COLNO as i32 - 1
        || yy > ROWNO as i32 - 1
        || ty > ROWNO as i32 - 1
    {
        return false;
    }

    // Determine initial direction
    let mut dx: i32 = 0;
    let mut dy: i32 = 0;
    if tx > xx {
        dx = 1;
    } else if ty > yy {
        dy = 1;
    } else if tx < xx {
        dx = -1;
    } else {
        dy = -1;
    }

    // Step back so first iteration steps forward
    xx -= dx;
    yy -= dy;
    let mut cct = 0;

    while xx != tx || yy != ty {
        // C: if (cct++ > 500 || (nxcor && !rn2(35))) return FALSE;
        cct += 1;
        if cct > 500 || (nxcor && rng.rn2(35) == 0) {
            return false;
        }

        xx += dx;
        yy += dy;

        // C: if (xx >= COLNO-1 || xx <= 0 || yy <= 0 || yy >= ROWNO-1)
        if xx >= COLNO as i32 - 1 || xx <= 0 || yy <= 0 || yy >= ROWNO as i32 - 1 {
            return false;
        }

        let ux = xx as usize;
        let uy = yy as usize;
        let crm_typ = level.cells[ux][uy].typ;

        if crm_typ == btyp {
            // C: if (ftyp != CORR || rn2(100)) crm->typ = ftyp; else crm->typ = SCORR;
            if ftyp != CellType::Corridor || rng.rn2(100) != 0 {
                level.cells[ux][uy].typ = ftyp;
                if nxcor && rng.rn2(50) == 0 {
                    // C: mksobj_at(BOULDER, xx, yy, TRUE, FALSE)
                    // Boulder mksobj consumes 0 RNG (ROCK_CLASS, no init)
                    // TODO: place boulder object
                }
            } else {
                level.cells[ux][uy].typ = CellType::SecretCorridor;
            }
        } else if crm_typ != ftyp && crm_typ != CellType::SecretCorridor {
            // C: return FALSE; (strange terrain)
            return false;
        }

        // Find next corridor position
        let mut dix = (xx - tx).unsigned_abs() as i32;
        let mut diy = (yy - ty).unsigned_abs() as i32;

        // C: if ((dix > diy) && diy && !rn2(dix-diy+1)) dix = 0;
        //    else if ((diy > dix) && dix && !rn2(diy-dix+1)) diy = 0;
        if dix > diy && diy != 0 && rng.rn2((dix - diy + 1) as u32) == 0 {
            dix = 0;
        } else if diy > dix && dix != 0 && rng.rn2((diy - dix + 1) as u32) == 0 {
            diy = 0;
        }

        // Do we have to change direction?
        if dy != 0 && dix > diy {
            // Currently moving vertically, but more horizontal distance remains
            let ddx: i32 = if xx > tx { -1 } else { 1 };
            let next_typ = level.cells[(xx + ddx) as usize][yy as usize].typ;
            if next_typ == btyp || next_typ == ftyp || next_typ == CellType::SecretCorridor {
                dx = ddx;
                dy = 0;
                continue;
            }
        } else if dx != 0 && diy > dix {
            // Currently moving horizontally, but more vertical distance remains
            let ddy: i32 = if yy > ty { -1 } else { 1 };
            let next_typ = level.cells[xx as usize][(yy + ddy) as usize].typ;
            if next_typ == btyp || next_typ == ftyp || next_typ == CellType::SecretCorridor {
                dy = ddy;
                dx = 0;
                continue;
            }
        }

        // Continue straight on?
        let straight_typ = level.cells[(xx + dx) as usize][(yy + dy) as usize].typ;
        if straight_typ == btyp || straight_typ == ftyp || straight_typ == CellType::SecretCorridor
        {
            continue;
        }

        // No, switch to orthogonal direction
        if dx != 0 {
            dx = 0;
            dy = if ty < yy { -1 } else { 1 };
        } else {
            dy = 0;
            dx = if tx < xx { -1 } else { 1 };
        }
        let ortho_typ = level.cells[(xx + dx) as usize][(yy + dy) as usize].typ;
        if ortho_typ == btyp || ortho_typ == ftyp || ortho_typ == CellType::SecretCorridor {
            continue;
        }

        // Last resort: reverse direction
        dy = -dy;
        dx = -dx;
    }
    true
}

/// 1:1 port of C's join() from mklev.c:245-317
///
/// Determines wall ranges from relative room positions, finds door positions
/// using finddpos(), places doors via dodoor(), and digs corridor between them.
fn join_rooms(
    level: &mut Level,
    rooms: &[Room],
    room_a: usize,
    room_b: usize,
    tracker: &mut ConnectivityTracker,
    rng: &mut GameRng,
    nxcor: bool,
) {
    if room_a >= rooms.len() || room_b >= rooms.len() || room_a == room_b {
        return;
    }

    let croom = &rooms[room_a];
    let troom = &rooms[room_b];

    // Room bounds (C's lx, ly, hx, hy)
    let c_lx = croom.x;
    let c_ly = croom.y;
    let c_hx = croom.x + croom.width - 1;
    let c_hy = croom.y + croom.height - 1;

    let t_lx = troom.x;
    let t_ly = troom.y;
    let t_hx = troom.x + troom.width - 1;
    let t_hy = troom.y + troom.height - 1;

    // C's 4-case structure from mklev.c join() lines 262-290
    let dx: i32;
    let dy: i32;
    let cc: (usize, usize);
    let tt: (usize, usize);

    if t_lx > c_hx {
        // troom is to the RIGHT of croom
        dx = 1;
        dy = 0;
        let xx = c_hx + 1;
        let tx = t_lx - 1;
        cc = finddpos(level, xx, c_ly, xx, c_hy, rng);
        tt = finddpos(level, tx, t_ly, tx, t_hy, rng);
    } else if t_hy < c_ly {
        // troom is ABOVE croom
        dy = -1;
        dx = 0;
        let yy = c_ly - 1;
        let ty = t_hy + 1;
        cc = finddpos(level, c_lx, yy, c_hx, yy, rng);
        tt = finddpos(level, t_lx, ty, t_hx, ty, rng);
    } else if t_hx < c_lx {
        // troom is to the LEFT of croom
        dx = -1;
        dy = 0;
        let xx = c_lx - 1;
        let tx = t_hx + 1;
        cc = finddpos(level, xx, c_ly, xx, c_hy, rng);
        tt = finddpos(level, tx, t_ly, tx, t_hy, rng);
    } else {
        // troom is BELOW croom (or overlapping)
        dy = 1;
        dx = 0;
        let yy = c_hy + 1;
        let ty = t_ly - 1;
        cc = finddpos(level, c_lx, yy, c_hx, yy, rng);
        tt = finddpos(level, t_lx, ty, t_hx, ty, rng);
    }

    let xx = cc.0 as i32;
    let yy = cc.1 as i32;
    let tx = tt.0 as i32 - dx;
    let ty = tt.1 as i32 - dy;

    // Early exit check for nxcor: if cell beyond door already has terrain
    if nxcor {
        let check_x = xx + dx;
        let check_y = yy + dy;
        if check_x > 0
            && check_y > 0
            && (check_x as usize) < COLNO
            && (check_y as usize) < ROWNO
            && level.cells[check_x as usize][check_y as usize].typ != CellType::Stone
        {
            return;
        }
    }

    // Place door on croom wall
    if okdoor(level, xx, yy) || !nxcor {
        dodoor(level, cc.0, cc.1, room_a, rng);
    }

    // C: dig_corridor(&org, &dest, nxcor, level.flags.arboreal ? ROOM : CORR, STONE)
    // For standard levels, ftyp=CORR, btyp=STONE
    if !dig_corridor_inner(
        level,
        xx + dx,
        yy + dy,
        tx,
        ty,
        nxcor,
        CellType::Corridor,
        CellType::Stone,
        rng,
    ) {
        return;
    }

    // Place door on troom wall
    if okdoor(level, tt.0 as i32, tt.1 as i32) || !nxcor {
        dodoor(level, tt.0, tt.1, room_b, rng);
    }

    // C: if (smeq[a] < smeq[b]) smeq[b] = smeq[a]; else smeq[a] = smeq[b];
    tracker.merge(room_a, room_b);
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

    // Phase 3: Ensure all rooms are connected
    // C: for (a=0; any && a<nroom; a++) { any=FALSE; for (b=0;b<nroom;b++) if(smeq[a]!=smeq[b]) { join(a,b,FALSE); any=TRUE; } }
    let mut any = true;
    let mut a = 0;
    while any && a < rooms.len() {
        any = false;
        for b in 0..rooms.len() {
            if !tracker.are_connected(a, b) {
                join_rooms(level, rooms, a, b, &mut tracker, rng, false);
                any = true;
            }
        }
        a += 1;
    }

    // Phase 4: Add random extra corridors (mklev.c:341-348)
    // C: for (i = rn2(nroom) + 4; i; i--) { a = rn2(nroom); b = rn2(nroom-2); if (b>=a) b+=2; join(a,b,TRUE); }
    if rooms.len() > 2 {
        let extra = rng.rn2(rooms.len() as u32) as usize + 4;
        for _ in 0..extra {
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
    use crate::dungeon::generation::carve_room;

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

        // Create some test rooms and carve walls properly
        let rooms = vec![
            Room::new(5, 5, 5, 4),
            Room::new(20, 5, 5, 4),
            Room::new(35, 5, 5, 4),
            Room::new(50, 5, 5, 4),
        ];

        // Carve rooms with walls (needed for okdoor/finddpos to work)
        for room in &rooms {
            carve_room(&mut level, room);
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

        // Count door cells (doors are now placed inside join)
        let door_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|cell| matches!(cell.typ, CellType::Door | CellType::SecretDoor))
            .count();

        println!("Generated {} door cells", door_count);
        assert!(door_count > 0, "Should have generated doors");

        // Verify connectivity by flood fill
        let (start_x, start_y) = rooms[0].center();
        let reachable = flood_fill_count(&level, start_x, start_y);
        println!("Reachable cells from room 0: {}", reachable);

        // Should be able to reach cells in other rooms
        let total_room_cells: usize = rooms.iter().map(|r| r.width * r.height).sum();
        assert!(
            reachable >= total_room_cells,
            "Should be able to reach all room cells (reachable={}, total={})",
            reachable,
            total_room_cells,
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
        dig_corridor_inner(
            &mut level,
            10,
            10,
            30,
            10,
            false,
            CellType::Corridor,
            CellType::Stone,
            &mut rng,
        );

        // Should have corridor cells along the path
        let corridor_count = (10..=30)
            .filter(|&x| {
                matches!(
                    level.cells[x][10].typ,
                    CellType::Corridor | CellType::SecretCorridor
                )
            })
            .count();

        assert!(corridor_count >= 10, "Should have corridor cells");
    }

    #[test]
    fn test_bydoor() {
        let mut level = Level::new(DLevel::main_dungeon_start());

        // Place a door at (20, 20)
        level.cells[20][20].typ = CellType::Door;

        // Should detect door next to position (adjacent cells)
        assert!(bydoor(&level, 20, 19));
        assert!(bydoor(&level, 20, 21));
        assert!(bydoor(&level, 19, 20));
        assert!(bydoor(&level, 21, 20));

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
        assert!(nexttodoor(&level, 20, 19));
        assert!(nexttodoor(&level, 20, 21));
        assert!(nexttodoor(&level, 19, 20));
        assert!(nexttodoor(&level, 21, 20));
        assert!(nexttodoor(&level, 19, 19));
        assert!(nexttodoor(&level, 21, 19));
        assert!(nexttodoor(&level, 19, 21));
        assert!(nexttodoor(&level, 21, 21));

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

        level.cells[25][15].typ = CellType::Stone;
        corr(&mut level, 25, 15, &mut rng);

        match level.cells[25][15].typ {
            CellType::Corridor | CellType::SecretCorridor => (),
            _ => panic!("Expected corridor or secret corridor"),
        }
    }

    #[test]
    fn test_finddpos() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut rng = GameRng::new(42);

        // Set up wall area
        for x in 10..=20 {
            level.cells[x][10].typ = CellType::HWall;
        }

        let (x, y) = finddpos(&level, 10, 10, 20, 10, &mut rng);
        assert!(x >= 10 && x <= 20);
        assert_eq!(y, 10);
    }

    #[test]
    fn test_finddpos_empty_area() {
        let level = Level::new(DLevel::main_dungeon_start());
        let mut rng = GameRng::new(42);

        // No walls in empty area - should return last resort corner (xl, yh)
        let pos = finddpos(&level, 30, 10, 35, 15, &mut rng);
        assert_eq!(pos, (30, 15), "Should return corner as last resort");
    }

    #[test]
    fn test_dosdoor_regular() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut rng = GameRng::new(42);

        // Place a wall first
        level.cells[10][10].typ = CellType::HWall;

        dosdoor(&mut level, 10, 10, CellType::Door, &mut rng);

        // Should now be a door
        assert_eq!(level.cells[10][10].typ, CellType::Door);
    }

    #[test]
    fn test_dosdoor_secret() {
        let mut level = Level::new(DLevel::main_dungeon_start());
        let mut rng = GameRng::new(42);

        level.cells[10][10].typ = CellType::VWall;

        dosdoor(&mut level, 10, 10, CellType::SecretDoor, &mut rng);

        assert_eq!(level.cells[10][10].typ, CellType::SecretDoor);
    }
}
