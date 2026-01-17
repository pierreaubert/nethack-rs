//! Special level definitions (sp_lev.c equivalent)
//!
//! Implements predefined special levels like Mines End, Sokoban, Oracle, etc.
//! Instead of parsing .des files, we define levels programmatically in Rust.

use crate::rng::GameRng;

use super::cell::CellType;
use super::level::{Level, Stairway, TrapType};
use super::room::RoomType;
use super::DLevel;

/// Map dimensions
pub const COLNO: usize = 80;
pub const ROWNO: usize = 21;

/// Special level identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecialLevelId {
    // Main dungeon
    Oracle,
    BigRoom,
    RogueLevel,
    Medusa,
    Castle,
    Valley,
    
    // Gnomish Mines
    MinesTown,
    MinesEnd1,  // Luckstone ending
    MinesEnd2,  // Alternate ending
    
    // Sokoban
    Sokoban1a,
    Sokoban1b,
    Sokoban2a,
    Sokoban2b,
    Sokoban3a,
    Sokoban3b,
    Sokoban4a,
    Sokoban4b,
    
    // Gehennom
    Juiblex,
    Baalzebub,
    Asmodeus,
    WizardTower1,
    WizardTower2,
    WizardTower3,
    Sanctum,
    VladsTower1,
    VladsTower2,
    VladsTower3,
    
    // Endgame
    AstralPlane,
    EarthPlane,
    AirPlane,
    FirePlane,
    WaterPlane,
}

impl SpecialLevelId {
    /// Get the dungeon and level for this special level
    pub fn location(&self) -> DLevel {
        match self {
            Self::Oracle => DLevel::new(0, 5),
            Self::BigRoom => DLevel::new(0, 10),
            Self::RogueLevel => DLevel::new(0, 15),
            Self::Medusa => DLevel::new(0, 20),
            Self::Castle => DLevel::new(0, 25),
            Self::Valley => DLevel::new(0, 26),
            
            Self::MinesTown => DLevel::new(2, 3),
            Self::MinesEnd1 | Self::MinesEnd2 => DLevel::new(2, 8),
            
            Self::Sokoban1a | Self::Sokoban1b => DLevel::new(3, 1),
            Self::Sokoban2a | Self::Sokoban2b => DLevel::new(3, 2),
            Self::Sokoban3a | Self::Sokoban3b => DLevel::new(3, 3),
            Self::Sokoban4a | Self::Sokoban4b => DLevel::new(3, 4),
            
            Self::Juiblex => DLevel::new(1, 5),
            Self::Baalzebub => DLevel::new(1, 10),
            Self::Asmodeus => DLevel::new(1, 15),
            Self::WizardTower1 => DLevel::new(1, 17),
            Self::WizardTower2 => DLevel::new(1, 18),
            Self::WizardTower3 => DLevel::new(1, 19),
            Self::Sanctum => DLevel::new(1, 20),
            Self::VladsTower1 => DLevel::new(6, 1),
            Self::VladsTower2 => DLevel::new(6, 2),
            Self::VladsTower3 => DLevel::new(6, 3),
            
            Self::AstralPlane => DLevel::new(7, 1),
            Self::EarthPlane => DLevel::new(7, 2),
            Self::AirPlane => DLevel::new(7, 3),
            Self::FirePlane => DLevel::new(7, 4),
            Self::WaterPlane => DLevel::new(7, 5),
        }
    }
    
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Oracle => "The Oracle",
            Self::BigRoom => "A Big Room",
            Self::RogueLevel => "A Primitive Area",
            Self::Medusa => "Medusa's Lair",
            Self::Castle => "The Castle",
            Self::Valley => "The Valley of the Dead",
            Self::MinesTown => "Minetown",
            Self::MinesEnd1 | Self::MinesEnd2 => "Mines End",
            Self::Sokoban1a | Self::Sokoban1b => "Sokoban Level 1",
            Self::Sokoban2a | Self::Sokoban2b => "Sokoban Level 2",
            Self::Sokoban3a | Self::Sokoban3b => "Sokoban Level 3",
            Self::Sokoban4a | Self::Sokoban4b => "Sokoban Level 4",
            Self::Juiblex => "Juiblex's Swamp",
            Self::Baalzebub => "Baalzebub's Lair",
            Self::Asmodeus => "Asmodeus's Lair",
            Self::WizardTower1 => "The Wizard's Tower Level 1",
            Self::WizardTower2 => "The Wizard's Tower Level 2",
            Self::WizardTower3 => "The Wizard's Tower Level 3",
            Self::Sanctum => "Moloch's Sanctum",
            Self::VladsTower1 => "Vlad's Tower Level 1",
            Self::VladsTower2 => "Vlad's Tower Level 2",
            Self::VladsTower3 => "Vlad's Tower Level 3",
            Self::AstralPlane => "The Astral Plane",
            Self::EarthPlane => "The Plane of Earth",
            Self::AirPlane => "The Plane of Air",
            Self::FirePlane => "The Plane of Fire",
            Self::WaterPlane => "The Plane of Water",
        }
    }
}

/// Check if a level should be a special level
pub fn get_special_level(dlevel: &DLevel) -> Option<SpecialLevelId> {
    match (dlevel.dungeon_num, dlevel.level_num) {
        // Main dungeon
        (0, 5) => Some(SpecialLevelId::Oracle),
        (0, 15) => Some(SpecialLevelId::RogueLevel),
        (0, 20) => Some(SpecialLevelId::Medusa),
        (0, 25) => Some(SpecialLevelId::Castle),
        (0, 26) => Some(SpecialLevelId::Valley),
        
        // Mines
        (2, 3) => Some(SpecialLevelId::MinesTown),
        (2, 8) => Some(SpecialLevelId::MinesEnd1),
        
        // Sokoban
        (3, 1) => Some(SpecialLevelId::Sokoban1a),
        (3, 2) => Some(SpecialLevelId::Sokoban2a),
        (3, 3) => Some(SpecialLevelId::Sokoban3a),
        (3, 4) => Some(SpecialLevelId::Sokoban4a),
        
        // Gehennom
        (1, 5) => Some(SpecialLevelId::Juiblex),
        (1, 10) => Some(SpecialLevelId::Baalzebub),
        (1, 15) => Some(SpecialLevelId::Asmodeus),
        (1, 17) => Some(SpecialLevelId::WizardTower1),
        (1, 18) => Some(SpecialLevelId::WizardTower2),
        (1, 19) => Some(SpecialLevelId::WizardTower3),
        (1, 20) => Some(SpecialLevelId::Sanctum),
        
        // Vlad's Tower
        (6, 1) => Some(SpecialLevelId::VladsTower1),
        (6, 2) => Some(SpecialLevelId::VladsTower2),
        (6, 3) => Some(SpecialLevelId::VladsTower3),
        
        // Endgame
        (7, 1) => Some(SpecialLevelId::AstralPlane),
        (7, 2) => Some(SpecialLevelId::EarthPlane),
        (7, 3) => Some(SpecialLevelId::AirPlane),
        (7, 4) => Some(SpecialLevelId::FirePlane),
        (7, 5) => Some(SpecialLevelId::WaterPlane),
        
        _ => None,
    }
}

/// Generate a special level
pub fn generate_special_level(level: &mut Level, level_id: SpecialLevelId, rng: &mut GameRng) {
    match level_id {
        SpecialLevelId::Oracle => generate_oracle(level, rng),
        SpecialLevelId::BigRoom => generate_big_room(level, rng),
        SpecialLevelId::RogueLevel => generate_rogue_level(level, rng),
        SpecialLevelId::Medusa => generate_medusa(level, rng),
        SpecialLevelId::Castle => generate_castle(level, rng),
        SpecialLevelId::Valley => generate_valley(level, rng),
        SpecialLevelId::MinesTown => generate_minetown(level, rng),
        SpecialLevelId::MinesEnd1 | SpecialLevelId::MinesEnd2 => generate_mines_end(level, rng),
        SpecialLevelId::Sokoban1a | SpecialLevelId::Sokoban1b => generate_sokoban_1(level, rng),
        SpecialLevelId::Sokoban2a | SpecialLevelId::Sokoban2b => generate_sokoban_2(level, rng),
        SpecialLevelId::Sokoban3a | SpecialLevelId::Sokoban3b => generate_sokoban_3(level, rng),
        SpecialLevelId::Sokoban4a | SpecialLevelId::Sokoban4b => generate_sokoban_4(level, rng),
        SpecialLevelId::WizardTower1 => generate_wizard_tower_1(level, rng),
        SpecialLevelId::WizardTower2 => generate_wizard_tower_2(level, rng),
        SpecialLevelId::WizardTower3 => generate_wizard_tower_3(level, rng),
        _ => generate_placeholder(level, rng),
    }
}

/// Generate the Oracle level
fn generate_oracle(level: &mut Level, rng: &mut GameRng) {
    // Fill with stone
    fill_level(level, CellType::Stone);
    
    // Create central oracle chamber (delphi)
    let cx = COLNO / 2;
    let cy = ROWNO / 2;
    let room_w = 11;
    let room_h = 7;
    
    // Carve the oracle room
    for x in (cx - room_w/2)..(cx + room_w/2) {
        for y in (cy - room_h/2)..(cy + room_h/2) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }
    
    // Add fountains in the oracle room (4 corners)
    level.cells[cx - 3][cy - 2].typ = CellType::Fountain;
    level.cells[cx + 3][cy - 2].typ = CellType::Fountain;
    level.cells[cx - 3][cy + 2].typ = CellType::Fountain;
    level.cells[cx + 3][cy + 2].typ = CellType::Fountain;
    
    // Surrounding rooms
    create_surrounding_rooms(level, cx, cy, room_w, room_h, rng);
    
    // Place stairs
    place_special_stairs(level, rng);
}

/// Generate a big room level
fn generate_big_room(level: &mut Level, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);
    
    // One large room taking most of the level
    let margin = 3;
    for x in margin..(COLNO - margin) {
        for y in margin..(ROWNO - margin) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }
    
    // Random pillars
    let num_pillars = 5 + rng.rn2(10) as usize;
    for _ in 0..num_pillars {
        let x = margin + 2 + rng.rn2((COLNO - margin * 2 - 4) as u32) as usize;
        let y = margin + 2 + rng.rn2((ROWNO - margin * 2 - 4) as u32) as usize;
        level.cells[x][y].typ = CellType::Stone;
    }
    
    place_special_stairs(level, rng);
}

/// Generate the Castle level
fn generate_castle(level: &mut Level, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);
    
    // Castle is a large fortified structure with moat
    let castle_x = 10;
    let castle_y = 3;
    let castle_w = 60;
    let castle_h = 15;
    
    // Outer moat
    for x in (castle_x - 2)..(castle_x + castle_w + 2) {
        for y in (castle_y - 2)..(castle_y + castle_h + 2) {
            if x < castle_x || x >= castle_x + castle_w || y < castle_y || y >= castle_y + castle_h {
                level.cells[x][y].typ = CellType::Moat;
            }
        }
    }
    
    // Castle walls
    for x in castle_x..(castle_x + castle_w) {
        level.cells[x][castle_y].typ = CellType::HWall;
        level.cells[x][castle_y + castle_h - 1].typ = CellType::HWall;
    }
    for y in castle_y..(castle_y + castle_h) {
        level.cells[castle_x][y].typ = CellType::VWall;
        level.cells[castle_x + castle_w - 1][y].typ = CellType::VWall;
    }
    
    // Castle interior
    for x in (castle_x + 1)..(castle_x + castle_w - 1) {
        for y in (castle_y + 1)..(castle_y + castle_h - 1) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }
    
    // Drawbridge
    let bridge_x = castle_x + castle_w / 2;
    level.cells[bridge_x][castle_y - 1].typ = CellType::DrawbridgeDown;
    level.cells[bridge_x][castle_y - 2].typ = CellType::DrawbridgeDown;
    level.cells[bridge_x][castle_y].typ = CellType::Door;
    
    // Throne room in center
    let throne_x = castle_x + castle_w / 2;
    let throne_y = castle_y + castle_h / 2;
    level.cells[throne_x][throne_y].typ = CellType::Throne;
    
    place_special_stairs(level, rng);
}

/// Generate Minetown
fn generate_minetown(level: &mut Level, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);
    
    // Minetown is a collection of shops and a temple
    let town_x = 15;
    let town_y = 3;
    let town_w = 50;
    let town_h = 15;
    
    // Main street (corridor)
    for x in town_x..(town_x + town_w) {
        level.cells[x][town_y + town_h / 2].typ = CellType::Corridor;
    }
    
    // Shops along the street
    let shop_positions = [
        (town_x + 5, town_y + 2, 6, 4, RoomType::GeneralShop),
        (town_x + 15, town_y + 2, 5, 4, RoomType::FoodShop),
        (town_x + 25, town_y + 2, 6, 4, RoomType::ToolShop),
        (town_x + 35, town_y + 2, 5, 4, RoomType::ArmorShop),
        (town_x + 5, town_y + 10, 6, 4, RoomType::WeaponShop),
        (town_x + 15, town_y + 10, 8, 4, RoomType::Temple),
    ];
    
    for (x, y, w, h, room_type) in shop_positions {
        create_room(level, x, y, w, h);
        // Connect to street
        level.cells[x + w / 2][town_y + town_h / 2].typ = CellType::Door;
        
        // Mark room type (for later population)
        if room_type == RoomType::Temple {
            level.cells[x + w / 2][y + h / 2].typ = CellType::Altar;
        }
    }
    
    // Fountain in town square
    level.cells[town_x + town_w / 2][town_y + town_h / 2 + 1].typ = CellType::Fountain;
    level.cells[town_x + town_w / 2][town_y + town_h / 2 - 1].typ = CellType::Fountain;
    
    place_special_stairs(level, rng);
}

/// Generate Mines End (luckstone level)
fn generate_mines_end(level: &mut Level, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);
    
    // Mines end has a central chamber with the luckstone
    let cx = COLNO / 2;
    let cy = ROWNO / 2;
    
    // Create irregular cavern-like rooms
    for _ in 0..8 {
        let rx = 10 + rng.rn2(50) as usize;
        let ry = 3 + rng.rn2(12) as usize;
        let rw = 4 + rng.rn2(6) as usize;
        let rh = 3 + rng.rn2(4) as usize;
        
        for x in rx..(rx + rw).min(COLNO - 2) {
            for y in ry..(ry + rh).min(ROWNO - 2) {
                level.cells[x][y].typ = CellType::Room;
            }
        }
    }
    
    // Central treasure room
    for x in (cx - 4)..(cx + 4) {
        for y in (cy - 2)..(cy + 2) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }
    
    // Connect rooms with corridors
    connect_rooms_randomly(level, rng);
    
    place_special_stairs(level, rng);
}

/// Generate Sokoban level 1
fn generate_sokoban_1(level: &mut Level, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);
    
    // Sokoban is a puzzle level - simplified version
    // Real implementation would load exact puzzle layout
    let start_x = 20;
    let start_y = 5;
    
    // Create puzzle area
    for x in start_x..(start_x + 30) {
        for y in start_y..(start_y + 10) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }
    
    // Add some walls for puzzle structure
    for i in 0..5 {
        let wx = start_x + 5 + i * 5;
        for y in (start_y + 2)..(start_y + 7) {
            if rng.rn2(3) != 0 {
                level.cells[wx][y].typ = CellType::Stone;
            }
        }
    }
    
    // Pit traps (holes in sokoban)
    for _ in 0..4 {
        let px = start_x + 3 + rng.rn2(24) as usize;
        let py = start_y + 2 + rng.rn2(6) as usize;
        if level.cells[px][py].typ == CellType::Room {
            level.add_trap(px as i8, py as i8, TrapType::Hole);
        }
    }
    
    // Up stairs only (Sokoban is one-way)
    level.cells[start_x + 28][start_y + 5].typ = CellType::Stairs;
    level.stairs.push(Stairway {
        x: (start_x + 28) as i8,
        y: (start_y + 5) as i8,
        destination: DLevel::new(3, 2),
        up: false,
    });
    
    // Entry from below
    level.cells[start_x + 2][start_y + 5].typ = CellType::Stairs;
    level.stairs.push(Stairway {
        x: (start_x + 2) as i8,
        y: (start_y + 5) as i8,
        destination: DLevel::new(0, 6), // Back to main dungeon
        up: true,
    });
}

/// Generate Sokoban level 2
fn generate_sokoban_2(level: &mut Level, rng: &mut GameRng) {
    generate_sokoban_generic(level, 2, rng);
}

/// Generate Sokoban level 3
fn generate_sokoban_3(level: &mut Level, rng: &mut GameRng) {
    generate_sokoban_generic(level, 3, rng);
}

/// Generate Sokoban level 4 (prize level)
fn generate_sokoban_4(level: &mut Level, rng: &mut GameRng) {
    generate_sokoban_generic(level, 4, rng);
    
    // Add prize room at the end
    let prize_x = 55;
    let prize_y = 8;
    for x in prize_x..(prize_x + 8) {
        for y in prize_y..(prize_y + 5) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }
}

/// Generic Sokoban level generator
fn generate_sokoban_generic(level: &mut Level, level_num: i8, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);
    
    let start_x = 15 + rng.rn2(10) as usize;
    let start_y = 4;
    let width = 35 + rng.rn2(10) as usize;
    let height = 12;
    
    // Create puzzle area
    for x in start_x..(start_x + width).min(COLNO - 2) {
        for y in start_y..(start_y + height).min(ROWNO - 2) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }
    
    // Add internal walls
    let num_walls = 3 + level_num as usize;
    for _ in 0..num_walls {
        let wx = start_x + 3 + rng.rn2((width - 6) as u32) as usize;
        let wall_len = 3 + rng.rn2(5) as usize;
        for dy in 0..wall_len {
            let wy = start_y + 2 + dy;
            if wy < start_y + height - 2 {
                level.cells[wx][wy].typ = CellType::Stone;
            }
        }
    }
    
    // Holes
    let num_holes = 2 + level_num as usize;
    for _ in 0..num_holes {
        let hx = start_x + 2 + rng.rn2((width - 4) as u32) as usize;
        let hy = start_y + 2 + rng.rn2((height - 4) as u32) as usize;
        if level.cells[hx][hy].typ == CellType::Room {
            level.add_trap(hx as i8, hy as i8, TrapType::Hole);
        }
    }
    
    // Stairs
    let next_level = if level_num < 4 { level_num + 1 } else { level_num };
    level.cells[start_x + width - 3][start_y + height / 2].typ = CellType::Stairs;
    level.stairs.push(Stairway {
        x: (start_x + width - 3) as i8,
        y: (start_y + height / 2) as i8,
        destination: DLevel::new(3, next_level),
        up: false,
    });
    
    level.cells[start_x + 2][start_y + height / 2].typ = CellType::Stairs;
    level.stairs.push(Stairway {
        x: (start_x + 2) as i8,
        y: (start_y + height / 2) as i8,
        destination: DLevel::new(3, level_num - 1),
        up: true,
    });
}

/// Generate a placeholder level for unimplemented special levels
fn generate_placeholder(level: &mut Level, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);
    
    // Simple room
    let cx = COLNO / 2;
    let cy = ROWNO / 2;
    for x in (cx - 10)..(cx + 10) {
        for y in (cy - 5)..(cy + 5) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }
    
    place_special_stairs(level, rng);
}

// Helper functions

fn fill_level(level: &mut Level, cell_type: CellType) {
    for x in 0..COLNO {
        for y in 0..ROWNO {
            level.cells[x][y].typ = cell_type;
            level.cells[x][y].lit = false;
        }
    }
}

fn create_room(level: &mut Level, x: usize, y: usize, w: usize, h: usize) {
    for rx in x..(x + w).min(COLNO - 1) {
        for ry in y..(y + h).min(ROWNO - 1) {
            level.cells[rx][ry].typ = CellType::Room;
            level.cells[rx][ry].lit = true;
        }
    }
}

fn create_surrounding_rooms(level: &mut Level, cx: usize, cy: usize, main_w: usize, main_h: usize, rng: &mut GameRng) {
    // Create 4-8 rooms around the central room
    let num_rooms = 4 + rng.rn2(5) as usize;
    
    for i in 0..num_rooms {
        let angle = (i as f32) * std::f32::consts::TAU / (num_rooms as f32);
        let dist = (main_w.max(main_h) as f32) * 1.5;
        
        let rx = (cx as f32 + angle.cos() * dist) as usize;
        let ry = (cy as f32 + angle.sin() * dist * 0.5) as usize; // Compress vertically
        
        if rx > 3 && rx < COLNO - 8 && ry > 2 && ry < ROWNO - 6 {
            let rw = 4 + rng.rn2(4) as usize;
            let rh = 3 + rng.rn2(3) as usize;
            create_room(level, rx, ry, rw, rh);
            
            // Connect to center with corridor
            connect_points(level, rx + rw / 2, ry + rh / 2, cx, cy);
        }
    }
}

fn connect_points(level: &mut Level, x1: usize, y1: usize, x2: usize, y2: usize) {
    let mut x = x1;
    let mut y = y1;
    
    // L-shaped corridor
    while x != x2 {
        if level.cells[x][y].typ == CellType::Stone {
            level.cells[x][y].typ = CellType::Corridor;
        }
        if x < x2 { x += 1; } else { x -= 1; }
    }
    while y != y2 {
        if level.cells[x][y].typ == CellType::Stone {
            level.cells[x][y].typ = CellType::Corridor;
        }
        if y < y2 { y += 1; } else { y -= 1; }
    }
}

fn connect_rooms_randomly(level: &mut Level, rng: &mut GameRng) {
    // Find room cells and connect some randomly
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
    for _ in 0..5 {
        let i1 = rng.rn2(room_cells.len() as u32) as usize;
        let i2 = rng.rn2(room_cells.len() as u32) as usize;
        if i1 != i2 {
            let (x1, y1) = room_cells[i1];
            let (x2, y2) = room_cells[i2];
            connect_points(level, x1, y1, x2, y2);
        }
    }
}

/// Generate the Rogue level (old-fashioned ASCII presentation)
fn generate_rogue_level(level: &mut Level, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);
    level.flags.corridor_maze = true; // Rogue-like display
    
    // Create a grid of small rooms connected by corridors
    let room_w = 8;
    let room_h = 4;
    let grid_x = 3;
    let grid_y = 3;
    
    for gx in 0..grid_x {
        for gy in 0..grid_y {
            let rx = 5 + gx * (COLNO - 10) / grid_x;
            let ry = 2 + gy * (ROWNO - 4) / grid_y;
            
            // Randomly skip some rooms
            if rng.one_in(4) {
                continue;
            }
            
            let w = room_w - (rng.rn2(3) as usize).min(room_w - 3);
            let h = room_h - (rng.rn2(2) as usize).min(room_h - 2);
            
            create_room(level, rx, ry, w, h);
        }
    }
    
    connect_rooms_randomly(level, rng);
    place_special_stairs(level, rng);
}

/// Generate Medusa's lair
fn generate_medusa(level: &mut Level, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);
    
    // Central island surrounded by water
    let cx = COLNO / 2;
    let cy = ROWNO / 2;
    
    // Fill with water first
    for x in 10..(COLNO - 10) {
        for y in 3..(ROWNO - 3) {
            level.cells[x][y].typ = CellType::Water;
        }
    }
    
    // Create central island
    let island_w = 20;
    let island_h = 10;
    for x in (cx - island_w/2)..(cx + island_w/2) {
        for y in (cy - island_h/2)..(cy + island_h/2) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }
    
    // Add statues (represented as boulders/stone)
    for _ in 0..8 {
        let sx = cx - island_w/2 + 2 + rng.rn2((island_w - 4) as u32) as usize;
        let sy = cy - island_h/2 + 2 + rng.rn2((island_h - 4) as u32) as usize;
        // Statues would be objects, but we mark the spot
        level.cells[sx][sy].typ = CellType::Room;
    }
    
    // Entry corridor from edge
    for x in 3..10 {
        level.cells[x][cy].typ = CellType::Corridor;
    }
    
    // Bridge to island
    for x in 10..(cx - island_w/2) {
        level.cells[x][cy].typ = CellType::Room;
    }
    
    place_special_stairs(level, rng);
}

/// Generate the Valley of the Dead
fn generate_valley(level: &mut Level, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);
    level.flags.graveyard = true;
    
    // Valley is a long corridor with graves
    let valley_y = ROWNO / 2;
    
    // Main valley path
    for x in 5..(COLNO - 5) {
        for dy in -2i32..=2 {
            let y = (valley_y as i32 + dy) as usize;
            if y < ROWNO {
                level.cells[x][y].typ = CellType::Room;
                level.cells[x][y].lit = false; // Dark
            }
        }
    }
    
    // Add graves along the sides
    for x in (8..(COLNO - 8)).step_by(4) {
        if rng.one_in(2) {
            level.cells[x][valley_y - 2].typ = CellType::Grave;
        }
        if rng.one_in(2) {
            level.cells[x][valley_y + 2].typ = CellType::Grave;
        }
    }
    
    // Temple at the end (entrance to Gehennom)
    let temple_x = COLNO - 15;
    for x in temple_x..(temple_x + 10) {
        for y in (valley_y - 3)..(valley_y + 4) {
            level.cells[x][y].typ = CellType::Room;
        }
    }
    level.cells[temple_x + 5][valley_y].typ = CellType::Altar;
    
    place_special_stairs(level, rng);
}

/// Generate Wizard's Tower level 1 (bottom)
fn generate_wizard_tower_1(level: &mut Level, rng: &mut GameRng) {
    generate_tower_level(level, 1, rng);
}

/// Generate Wizard's Tower level 2 (middle)
fn generate_wizard_tower_2(level: &mut Level, rng: &mut GameRng) {
    generate_tower_level(level, 2, rng);
}

/// Generate Wizard's Tower level 3 (top - Wizard's lair)
fn generate_wizard_tower_3(level: &mut Level, rng: &mut GameRng) {
    generate_tower_level(level, 3, rng);
}

/// Generate a tower level (shared by Wizard's Tower and Vlad's Tower)
fn generate_tower_level(level: &mut Level, floor: u8, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);
    level.flags.no_teleport = true;
    
    // Tower is a central structure
    let cx = COLNO / 2;
    let cy = ROWNO / 2;
    let tower_w = 16 - (floor as usize * 2); // Gets smaller as you go up
    let tower_h = 10 - floor as usize;
    
    // Create tower room
    for x in (cx - tower_w/2)..(cx + tower_w/2) {
        for y in (cy - tower_h/2)..(cy + tower_h/2) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = floor == 3; // Top floor is lit
        }
    }
    
    // Add internal walls for rooms
    if floor < 3 {
        let mid_x = cx;
        for y in (cy - tower_h/2 + 1)..(cy + tower_h/2 - 1) {
            level.cells[mid_x][y].typ = CellType::VWall;
        }
        level.cells[mid_x][cy].typ = CellType::Door;
    }
    
    // Add magic traps
    for _ in 0..(floor as usize + 2) {
        let tx = cx - tower_w/2 + 2 + rng.rn2((tower_w - 4) as u32) as usize;
        let ty = cy - tower_h/2 + 2 + rng.rn2((tower_h - 4) as u32) as usize;
        level.add_trap(tx as i8, ty as i8, TrapType::MagicTrap);
    }
    
    // Stairs
    if floor > 1 {
        // Down stairs
        level.cells[cx - 3][cy].typ = CellType::Stairs;
        level.stairs.push(Stairway {
            x: (cx - 3) as i8,
            y: cy as i8,
            destination: DLevel::new(1, 16 + floor as i8),
            up: false,
        });
    }
    
    if floor < 3 {
        // Up stairs
        level.cells[cx + 3][cy].typ = CellType::Stairs;
        level.stairs.push(Stairway {
            x: (cx + 3) as i8,
            y: cy as i8,
            destination: DLevel::new(1, 18 + floor as i8),
            up: true,
        });
    }
}

fn place_special_stairs(level: &mut Level, rng: &mut GameRng) {
    // Find valid positions for stairs
    let mut valid_positions: Vec<(usize, usize)> = Vec::new();
    
    for x in 3..(COLNO - 3) {
        for y in 2..(ROWNO - 2) {
            if level.cells[x][y].typ == CellType::Room || level.cells[x][y].typ == CellType::Corridor {
                valid_positions.push((x, y));
            }
        }
    }
    
    if valid_positions.is_empty() {
        return;
    }
    
    // Place up stairs
    let up_idx = rng.rn2(valid_positions.len() as u32) as usize;
    let (ux, uy) = valid_positions[up_idx];
    level.cells[ux][uy].typ = CellType::Stairs;
    level.stairs.push(Stairway {
        x: ux as i8,
        y: uy as i8,
        destination: DLevel::new(level.dlevel.dungeon_num, level.dlevel.level_num - 1),
        up: true,
    });
    
    // Place down stairs (far from up stairs)
    let mut best_idx = 0;
    let mut best_dist = 0;
    for (i, (x, y)) in valid_positions.iter().enumerate() {
        let dist = ((*x as i32 - ux as i32).abs() + (*y as i32 - uy as i32).abs()) as usize;
        if dist > best_dist {
            best_dist = dist;
            best_idx = i;
        }
    }
    
    let (dx, dy) = valid_positions[best_idx];
    level.cells[dx][dy].typ = CellType::Stairs;
    level.stairs.push(Stairway {
        x: dx as i8,
        y: dy as i8,
        destination: DLevel::new(level.dlevel.dungeon_num, level.dlevel.level_num + 1),
        up: false,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_level_lookup() {
        assert_eq!(get_special_level(&DLevel::new(0, 5)), Some(SpecialLevelId::Oracle));
        assert_eq!(get_special_level(&DLevel::new(2, 3)), Some(SpecialLevelId::MinesTown));
        assert_eq!(get_special_level(&DLevel::new(3, 1)), Some(SpecialLevelId::Sokoban1a));
        assert_eq!(get_special_level(&DLevel::new(0, 10)), None); // Not a special level
    }

    #[test]
    fn test_oracle_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(0, 5));
        
        generate_special_level(&mut level, SpecialLevelId::Oracle, &mut rng);
        
        // Should have fountains
        let fountain_count = level.cells.iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Fountain)
            .count();
        
        assert!(fountain_count >= 4, "Oracle should have at least 4 fountains");
        
        // Should have stairs
        assert!(!level.stairs.is_empty(), "Oracle should have stairs");
    }

    #[test]
    fn test_castle_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(0, 25));
        
        generate_special_level(&mut level, SpecialLevelId::Castle, &mut rng);
        
        // Should have moat
        let moat_count = level.cells.iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Moat)
            .count();
        
        assert!(moat_count > 50, "Castle should have moat");
        
        // Should have throne
        let throne_count = level.cells.iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Throne)
            .count();
        
        assert_eq!(throne_count, 1, "Castle should have one throne");
    }

    #[test]
    fn test_sokoban_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(3, 1));
        
        generate_special_level(&mut level, SpecialLevelId::Sokoban1a, &mut rng);
        
        // Should have hole traps
        assert!(!level.traps.is_empty(), "Sokoban should have hole traps");
        
        // Should have stairs
        assert!(level.stairs.len() >= 2, "Sokoban should have entry and exit stairs");
    }

    #[test]
    fn test_minetown_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(2, 3));
        
        generate_special_level(&mut level, SpecialLevelId::MinesTown, &mut rng);
        
        // Should have altar (temple)
        let altar_count = level.cells.iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Altar)
            .count();
        
        assert!(altar_count >= 1, "Minetown should have altar");
        
        // Should have fountains
        let fountain_count = level.cells.iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Fountain)
            .count();
        
        assert!(fountain_count >= 1, "Minetown should have fountains");
    }
}
