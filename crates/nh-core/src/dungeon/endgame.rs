//! Endgame levels (Elemental Planes and Astral Plane)
//!
//! Implements the final levels of the game after escaping Gehennom.

use crate::rng::GameRng;

use super::DLevel;
use super::cell::CellType;
use super::level::{Level, Stairway, TrapType};

/// Map dimensions
const COLNO: usize = 80;
const ROWNO: usize = 21;

/// Endgame plane types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plane {
    Earth,
    Air,
    Fire,
    Water,
    Astral,
}

impl Plane {
    /// Get the DLevel for this plane
    pub fn dlevel(&self) -> DLevel {
        match self {
            Plane::Earth => DLevel::new(7, 1),
            Plane::Air => DLevel::new(7, 2),
            Plane::Fire => DLevel::new(7, 3),
            Plane::Water => DLevel::new(7, 4),
            Plane::Astral => DLevel::new(7, 5),
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Plane::Earth => "The Plane of Earth",
            Plane::Air => "The Plane of Air",
            Plane::Fire => "The Plane of Fire",
            Plane::Water => "The Plane of Water",
            Plane::Astral => "The Astral Plane",
        }
    }
}

/// Generate an endgame plane level
pub fn generate_plane(level: &mut Level, plane: Plane, rng: &mut GameRng) {
    match plane {
        Plane::Earth => generate_earth_plane(level, rng),
        Plane::Air => generate_air_plane(level, rng),
        Plane::Fire => generate_fire_plane(level, rng),
        Plane::Water => generate_water_plane(level, rng),
        Plane::Astral => generate_astral_plane(level, rng),
    }
}

/// Generate the Plane of Earth
fn generate_earth_plane(level: &mut Level, rng: &mut GameRng) {
    // Earth plane is a maze of caverns with earth elementals
    fill_level(level, CellType::Stone);

    // Create winding caverns
    for _ in 0..15 {
        let cx = 5 + rng.rn2(70) as usize;
        let cy = 2 + rng.rn2(17) as usize;
        let radius = 2 + rng.rn2(4) as usize;

        for dx in 0..radius * 2 {
            for dy in 0..radius * 2 {
                let x = cx.saturating_sub(radius) + dx;
                let y = cy.saturating_sub(radius) + dy;
                if x < COLNO && y < ROWNO {
                    let dist = ((dx as i32 - radius as i32).pow(2)
                        + (dy as i32 - radius as i32).pow(2))
                        as usize;
                    if dist <= radius * radius {
                        level.cells[x][y].typ = CellType::Room;
                        level.cells[x][y].lit = false;
                    }
                }
            }
        }
    }

    // Connect caverns
    connect_caverns(level, rng);

    // Add rock traps (falling rocks)
    for _ in 0..10 {
        if let Some((x, y)) = find_empty_spot(level, rng) {
            level.add_trap(x as i8, y as i8, TrapType::RockFall);
        }
    }

    // Portal to next plane
    place_magic_portal(level, Plane::Air, rng);

    // Entry from Sanctum
    place_entry_portal(level, rng);
}

/// Generate the Plane of Air
fn generate_air_plane(level: &mut Level, rng: &mut GameRng) {
    // Air plane is mostly open with clouds and air pockets
    fill_level(level, CellType::Air);

    // Create cloud platforms
    for _ in 0..20 {
        let cx = 5 + rng.rn2(70) as usize;
        let cy = 2 + rng.rn2(17) as usize;
        let w = 3 + rng.rn2(6) as usize;
        let h = 2 + rng.rn2(4) as usize;

        for x in cx..(cx + w).min(COLNO - 1) {
            for y in cy..(cy + h).min(ROWNO - 1) {
                level.cells[x][y].typ = CellType::Cloud;
                level.cells[x][y].lit = true;
            }
        }
    }

    // Central platform (more solid)
    let cx = COLNO / 2;
    let cy = ROWNO / 2;
    for x in (cx - 5)..(cx + 5) {
        for y in (cy - 3)..(cy + 3) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }

    // Portal to next plane
    place_magic_portal(level, Plane::Fire, rng);

    // Entry from Earth
    place_entry_portal(level, rng);
}

/// Generate the Plane of Fire
fn generate_fire_plane(level: &mut Level, rng: &mut GameRng) {
    // Fire plane is lava with islands
    fill_level(level, CellType::Lava);

    // Create stone islands
    for _ in 0..12 {
        let cx = 5 + rng.rn2(70) as usize;
        let cy = 2 + rng.rn2(17) as usize;
        let radius = 2 + rng.rn2(5) as usize;

        for dx in 0..radius * 2 {
            for dy in 0..radius * 2 {
                let x = cx.saturating_sub(radius) + dx;
                let y = cy.saturating_sub(radius) + dy;
                if x < COLNO && y < ROWNO {
                    let dist = ((dx as i32 - radius as i32).pow(2)
                        + (dy as i32 - radius as i32).pow(2))
                        as usize;
                    if dist <= radius * radius {
                        level.cells[x][y].typ = CellType::Room;
                        level.cells[x][y].lit = true;
                    }
                }
            }
        }
    }

    // Central tower
    let cx = COLNO / 2;
    let cy = ROWNO / 2;
    for x in (cx - 6)..(cx + 6) {
        for y in (cy - 4)..(cy + 4) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }

    // Fire traps
    for _ in 0..8 {
        if let Some((x, y)) = find_empty_spot(level, rng) {
            level.add_trap(x as i8, y as i8, TrapType::FireTrap);
        }
    }

    // Portal to next plane
    place_magic_portal(level, Plane::Water, rng);

    // Entry from Air
    place_entry_portal(level, rng);
}

/// Water level bubble state for dynamic movement
#[derive(Debug, Clone)]
pub struct WaterBubble {
    pub x: usize,
    pub y: usize,
    pub radius: usize,
    pub dx: i8,
    pub dy: i8,
}

impl WaterBubble {
    pub fn new(x: usize, y: usize, radius: usize, rng: &mut GameRng) -> Self {
        // Random direction
        let dx = (rng.rn2(3) as i8) - 1;
        let dy = (rng.rn2(3) as i8) - 1;
        Self {
            x,
            y,
            radius,
            dx,
            dy,
        }
    }

    /// Move the bubble one step
    pub fn move_step(&mut self, rng: &mut GameRng) {
        // Occasionally change direction
        if rng.one_in(10) {
            self.dx = (rng.rn2(3) as i8) - 1;
            self.dy = (rng.rn2(3) as i8) - 1;
        }

        // Move
        let new_x = (self.x as i32 + self.dx as i32)
            .clamp(self.radius as i32 + 2, (COLNO - self.radius - 2) as i32)
            as usize;
        let new_y = (self.y as i32 + self.dy as i32)
            .clamp(self.radius as i32 + 2, (ROWNO - self.radius - 2) as i32)
            as usize;

        self.x = new_x;
        self.y = new_y;

        // Bounce off edges
        if new_x <= self.radius + 2 || new_x >= COLNO - self.radius - 2 {
            self.dx = -self.dx;
        }
        if new_y <= self.radius + 2 || new_y >= ROWNO - self.radius - 2 {
            self.dy = -self.dy;
        }
    }
}

/// Update water level with moving bubbles
pub fn update_water_level(level: &mut Level, bubbles: &[WaterBubble]) {
    // First, fill everything with water
    for x in 0..COLNO {
        for y in 0..ROWNO {
            if level.cells[x][y].typ == CellType::Room {
                level.cells[x][y].typ = CellType::Water;
            }
        }
    }

    // Then carve out bubbles
    for bubble in bubbles {
        for dx in 0..bubble.radius * 2 {
            for dy in 0..bubble.radius * 2 {
                let x = bubble.x.saturating_sub(bubble.radius) + dx;
                let y = bubble.y.saturating_sub(bubble.radius) + dy;
                if x < COLNO && y < ROWNO {
                    let dist = ((dx as i32 - bubble.radius as i32).pow(2)
                        + (dy as i32 - bubble.radius as i32).pow(2))
                        as usize;
                    if dist <= bubble.radius * bubble.radius {
                        level.cells[x][y].typ = CellType::Room;
                        level.cells[x][y].lit = true;
                    }
                }
            }
        }
    }
}

/// Create initial bubbles for water level
pub fn create_water_bubbles(rng: &mut GameRng) -> Vec<WaterBubble> {
    let mut bubbles = Vec::new();

    // Create 5-8 bubbles
    let num_bubbles = 5 + rng.rn2(4) as usize;
    for _ in 0..num_bubbles {
        let x = 10 + rng.rn2(60) as usize;
        let y = 4 + rng.rn2(13) as usize;
        let radius = 2 + rng.rn2(3) as usize;
        bubbles.push(WaterBubble::new(x, y, radius, rng));
    }

    bubbles
}

/// Generate the Plane of Water
fn generate_water_plane(level: &mut Level, rng: &mut GameRng) {
    // Water plane is water with bubbles of air
    fill_level(level, CellType::Water);

    // Create air bubbles (rooms)
    for _ in 0..15 {
        let cx = 5 + rng.rn2(70) as usize;
        let cy = 2 + rng.rn2(17) as usize;
        let radius = 2 + rng.rn2(4) as usize;

        for dx in 0..radius * 2 {
            for dy in 0..radius * 2 {
                let x = cx.saturating_sub(radius) + dx;
                let y = cy.saturating_sub(radius) + dy;
                if x < COLNO && y < ROWNO {
                    let dist = ((dx as i32 - radius as i32).pow(2)
                        + (dy as i32 - radius as i32).pow(2))
                        as usize;
                    if dist <= radius * radius {
                        level.cells[x][y].typ = CellType::Room;
                        level.cells[x][y].lit = true;
                    }
                }
            }
        }
    }

    // Central bubble
    let cx = COLNO / 2;
    let cy = ROWNO / 2;
    for x in (cx - 8)..(cx + 8) {
        for y in (cy - 5)..(cy + 5) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }

    // Portal to Astral
    place_magic_portal(level, Plane::Astral, rng);

    // Entry from Fire
    place_entry_portal(level, rng);
}

/// Generate the Astral Plane
fn generate_astral_plane(level: &mut Level, _rng: &mut GameRng) {
    // Astral plane has three temples (Lawful, Neutral, Chaotic)
    fill_level(level, CellType::Cloud);

    // Main corridor
    for x in 5..(COLNO - 5) {
        level.cells[x][ROWNO / 2].typ = CellType::Room;
        level.cells[x][ROWNO / 2].lit = true;
    }

    // Three temples
    let temple_positions = [
        (15, 5), // Lawful (left)
        (40, 5), // Neutral (center)
        (65, 5), // Chaotic (right)
    ];

    for (tx, ty) in temple_positions {
        // Temple room
        for x in (tx - 5)..(tx + 5) {
            for y in ty..(ty + 8) {
                if x < COLNO && y < ROWNO {
                    level.cells[x][y].typ = CellType::Room;
                    level.cells[x][y].lit = true;
                }
            }
        }

        // Altar in center
        level.cells[tx][ty + 4].typ = CellType::Altar;

        // Connect to main corridor
        for y in (ty + 8)..(ROWNO / 2) {
            level.cells[tx][y].typ = CellType::Corridor;
        }
    }

    // High altar in center temple (the correct one depends on player alignment)
    // For now, mark all three as potential victory points

    // Entry from Water
    level.cells[5][ROWNO / 2].typ = CellType::Stairs;
    level.stairs.push(Stairway {
        x: 5,
        y: (ROWNO / 2) as i8,
        destination: Plane::Water.dlevel(),
        up: true,
    });

    // No exit - victory is achieved by offering the Amulet at the correct altar
    level.flags.no_teleport = true;
}

// Helper functions

fn fill_level(level: &mut Level, cell_type: CellType) {
    for x in 0..COLNO {
        for y in 0..ROWNO {
            level.cells[x][y].typ = cell_type;
            level.cells[x][y].lit = false;
        }
    }
    level.flags.no_teleport = true; // No teleporting on planes
}

fn connect_caverns(level: &mut Level, rng: &mut GameRng) {
    // Find room cells and connect them
    let mut room_cells: Vec<(usize, usize)> = Vec::new();

    for x in 2..(COLNO - 2) {
        for y in 2..(ROWNO - 2) {
            if level.cells[x][y].typ == CellType::Room {
                room_cells.push((x, y));
            }
        }
    }

    if room_cells.len() < 2 {
        return;
    }

    // Connect random pairs
    for _ in 0..15 {
        let i1 = rng.rn2(room_cells.len() as u32) as usize;
        let i2 = rng.rn2(room_cells.len() as u32) as usize;
        if i1 != i2 {
            let (x1, y1) = room_cells[i1];
            let (x2, y2) = room_cells[i2];
            connect_points(level, x1, y1, x2, y2);
        }
    }
}

fn connect_points(level: &mut Level, x1: usize, y1: usize, x2: usize, y2: usize) {
    let mut x = x1;
    let mut y = y1;

    while x != x2 {
        if level.cells[x][y].typ == CellType::Stone {
            level.cells[x][y].typ = CellType::Corridor;
        }
        if x < x2 {
            x += 1;
        } else {
            x -= 1;
        }
    }
    while y != y2 {
        if level.cells[x][y].typ == CellType::Stone {
            level.cells[x][y].typ = CellType::Corridor;
        }
        if y < y2 {
            y += 1;
        } else {
            y -= 1;
        }
    }
}

fn find_empty_spot(level: &Level, rng: &mut GameRng) -> Option<(usize, usize)> {
    for _ in 0..50 {
        let x = 5 + rng.rn2(70) as usize;
        let y = 2 + rng.rn2(17) as usize;

        if level.cells[x][y].typ == CellType::Room || level.cells[x][y].typ == CellType::Corridor {
            return Some((x, y));
        }
    }
    None
}

fn place_magic_portal(level: &mut Level, to_plane: Plane, rng: &mut GameRng) {
    // Find a spot for the portal to the next plane
    for _ in 0..100 {
        let x = 5 + rng.rn2(70) as usize;
        let y = 2 + rng.rn2(17) as usize;

        if level.cells[x][y].typ == CellType::Room {
            level.add_trap(x as i8, y as i8, TrapType::MagicPortal);
            // Store destination in level data (simplified - real impl would track portal destinations)
            level.stairs.push(Stairway {
                x: x as i8,
                y: y as i8,
                destination: to_plane.dlevel(),
                up: false,
            });
            return;
        }
    }
}

fn place_entry_portal(level: &mut Level, rng: &mut GameRng) {
    // Entry point from previous plane
    for _ in 0..100 {
        let x = 5 + rng.rn2(70) as usize;
        let y = 2 + rng.rn2(17) as usize;

        if level.cells[x][y].typ == CellType::Room {
            level.cells[x][y].typ = CellType::Stairs;
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_dlevels() {
        assert_eq!(Plane::Earth.dlevel(), DLevel::new(7, 1));
        assert_eq!(Plane::Air.dlevel(), DLevel::new(7, 2));
        assert_eq!(Plane::Fire.dlevel(), DLevel::new(7, 3));
        assert_eq!(Plane::Water.dlevel(), DLevel::new(7, 4));
        assert_eq!(Plane::Astral.dlevel(), DLevel::new(7, 5));
    }

    #[test]
    fn test_earth_plane_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(Plane::Earth.dlevel());

        generate_plane(&mut level, Plane::Earth, &mut rng);

        // Should have room cells (caverns)
        let room_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Room)
            .count();

        assert!(room_count > 50, "Earth plane should have caverns");

        // Should have rock traps
        assert!(!level.traps.is_empty(), "Earth plane should have traps");
    }

    #[test]
    fn test_fire_plane_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(Plane::Fire.dlevel());

        generate_plane(&mut level, Plane::Fire, &mut rng);

        // Should have lava
        let lava_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Lava)
            .count();

        assert!(lava_count > 100, "Fire plane should have lava");

        // Should have fire traps
        let fire_traps = level
            .traps
            .iter()
            .filter(|t| t.trap_type == TrapType::FireTrap)
            .count();

        assert!(fire_traps > 0, "Fire plane should have fire traps");
    }

    #[test]
    fn test_astral_plane_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(Plane::Astral.dlevel());

        generate_plane(&mut level, Plane::Astral, &mut rng);

        // Should have altars (3 temples)
        let altar_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Altar)
            .count();

        assert_eq!(altar_count, 3, "Astral plane should have 3 altars");

        // Should have no teleport flag
        assert!(
            level.flags.no_teleport,
            "Astral plane should block teleport"
        );
    }

    #[test]
    fn test_water_plane_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(Plane::Water.dlevel());

        generate_plane(&mut level, Plane::Water, &mut rng);

        // Should have water
        let water_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Water)
            .count();

        assert!(water_count > 100, "Water plane should have water");
    }
}
