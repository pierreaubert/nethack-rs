//! Special level definitions (sp_lev.c equivalent)
//!
//! Implements predefined special levels like Mines End, Sokoban, Oracle, etc.
//! Instead of parsing .des files, we define levels programmatically in Rust.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::rng::GameRng;

use super::DLevel;
use super::cell::CellType;
use super::endgame;
use super::level::{Level, Stairway, TrapType};
use super::room::RoomType;

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
    MinesEnd1, // Luckstone ending
    MinesEnd2, // Alternate ending

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
        SpecialLevelId::Sokoban1a => generate_sokoban(level, SOKOBAN_1A, 1, rng),
        SpecialLevelId::Sokoban1b => generate_sokoban(level, SOKOBAN_1B, 1, rng),
        SpecialLevelId::Sokoban2a => generate_sokoban(level, SOKOBAN_2A, 2, rng),
        SpecialLevelId::Sokoban2b => generate_sokoban(level, SOKOBAN_2B, 2, rng),
        SpecialLevelId::Sokoban3a => generate_sokoban(level, SOKOBAN_3A, 3, rng),
        SpecialLevelId::Sokoban3b => generate_sokoban(level, SOKOBAN_3B, 3, rng),
        SpecialLevelId::Sokoban4a => generate_sokoban(level, SOKOBAN_4A, 4, rng),
        SpecialLevelId::Sokoban4b => generate_sokoban(level, SOKOBAN_4B, 4, rng),
        SpecialLevelId::WizardTower1 => generate_wizard_tower_1(level, rng),
        SpecialLevelId::WizardTower2 => generate_wizard_tower_2(level, rng),
        SpecialLevelId::WizardTower3 => generate_wizard_tower_3(level, rng),
        SpecialLevelId::Sanctum => generate_sanctum(level, rng),
        SpecialLevelId::VladsTower1 => generate_vlads_tower_1(level, rng),
        SpecialLevelId::VladsTower2 => generate_vlads_tower_2(level, rng),
        SpecialLevelId::VladsTower3 => generate_vlads_tower_3(level, rng),
        SpecialLevelId::Asmodeus => generate_asmodeus(level, rng),
        SpecialLevelId::Juiblex => generate_juiblex(level, rng),
        SpecialLevelId::Baalzebub => generate_baalzebub(level, rng),
        SpecialLevelId::AstralPlane => endgame::generate_plane(level, endgame::Plane::Astral, rng),
        SpecialLevelId::EarthPlane => endgame::generate_plane(level, endgame::Plane::Earth, rng),
        SpecialLevelId::AirPlane => endgame::generate_plane(level, endgame::Plane::Air, rng),
        SpecialLevelId::FirePlane => endgame::generate_plane(level, endgame::Plane::Fire, rng),
        SpecialLevelId::WaterPlane => endgame::generate_plane(level, endgame::Plane::Water, rng),
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
    for x in (cx - room_w / 2)..(cx + room_w / 2) {
        for y in (cy - room_h / 2)..(cy + room_h / 2) {
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
            if x < castle_x || x >= castle_x + castle_w || y < castle_y || y >= castle_y + castle_h
            {
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

// ============================================================
// Canonical Sokoban levels from NetHack 3.6.7 sokoban.des
// ============================================================
//
// Legend for map strings:
//   '-' = HWALL, '|' = VWALL, '.' = ROOM (lit), ' ' = STONE
//   '0' = boulder position (ROOM + boulder object)
//   '^' = hole trap (ROOM + Hole trap)
//   '<' = up stairs, '>' = down stairs
//   '+' = door
//
// Each level has two variants (a/b). The game picks one at random.
// Levels are numbered bottom-up: level 1 is entry (bottom), level 4 is prize (top).

/// Sokoban 1a - entry level, variant a (from sokoban.des)
const SOKOBAN_1A: &[&str] = &[
    "-------- ------",
    "|.|....|-|....|",
    "|.|....|..0...|",
    "|.|....|.|....|",
    "|.|....|.|....|",
    "|.|-.---.---..|",
    "|.+.........0.|",
    "|.|..|--.--|..|",
    "|.|..|  |..|..|",
    "|.|..|  |..|..|",
    "|.----  |..--.|",
    "|.......|..0..|",
    "|.......+.0...|",
    "|.......|.....|",
    "--------- ----|",
    "          |.<.|",
    "          ----|",
];

/// Sokoban 1b - entry level, variant b
const SOKOBAN_1B: &[&str] = &[
    "------  -----",
    "|....|  |...|",
    "|.0..---|...|",
    "|.0....0..0.|",
    "|...|..|.0..|",
    "|...|..|..0.|",
    "|-.--..|.---|",
    "|..0....|...|",
    "|.....0.|...|",
    "-------.--+-|",
    "  |.........|",
    "  |.........|",
    "  |...|.--.-|",
    "  |...|.....|",
    "  ------|.--|",
    "       |.<.|",
    "       ----|",
];

/// Sokoban 2a - level 2, variant a
const SOKOBAN_2A: &[&str] = &[
    " ----          -----------",
    "--.>.-----------.........|",
    "|....|.........0.........|",
    "|....+.........0..-------|",
    "--.--..........0..|      ",
    "  |....0.0..0.0...|      ",
    "  |....|..0..0.---|      ",
    "  |-..--........|        ",
    "   |...|..0..0..|        ",
    "   |...|........|        ",
    "   |...---......|        ",
    "   |.....|.^^^^.|        ",
    "   |.....|......|        ",
    "   -------+------        ",
    "          |..|            ",
    "          |..|            ",
    "          -<--            ",
];

/// Sokoban 2b - level 2, variant b
const SOKOBAN_2B: &[&str] = &[
    "-------- ------",
    "|.>....|--....|",
    "|..-...|.0..0.|",
    "|...--..|.0...|",
    "|.....|.|..0..|",
    "|.....+.+..0..|",
    "|.....|.|.0...|",
    "|..---..|.....|",
    "|..-...|.0..0.|",
    "|..0...|----..|",
    "----...|  |...|",
    "   |...|  |...|",
    "   |...---|...|",
    "   |..0..0..0.|",
    "   |...|..|...|",
    "   |...+..+...|",
    "   |...|..|...|",
    "   |...----...|",
    "   |..^^^^....|",
    "   |..---------",
    "   ----        ",
];

/// Sokoban 3a - level 3, variant a
const SOKOBAN_3A: &[&str] = &[
    "  --------",
    "--|.>....|",
    "|.+..0...|",
    "|.|--.-..|",
    "|.|  |...|",
    "|.|  |...|",
    "|.----|..|",
    "|.....0..|",
    "|..0..-..|",
    "|.....0..|",
    "---...--.--",
    "  |.0.....|",
    "  |...--..|",
    "  |.0.....|",
    "  |...--..|",
    "  |.0.....|",
    "  |...--..|",
    "  |.0.....|",
    "  |..^^^^.|",
    "  |...----|",
    "  ----    ",
];

/// Sokoban 3b - level 3, variant b (compact for 21-row map)
const SOKOBAN_3B: &[&str] = &[
    "  --------",
    "--|.>....|",
    "|.+.....0|",
    "|.|..-...|",
    "|.|..|...|",
    "|.|..--0.|",
    "|......0.|",
    "|.|..-...|",
    "|.|.0|...|",
    "|.|..--0.|",
    "|......0.|",
    "|.|..-...|",
    "|.|.0|...|",
    "|.|..--0.|",
    "|......0.|",
    "|.|..-...|",
    "|.|.0|...|",
    "|.|..--0.|",
    "|......0.|",
    "|..^^^^..|",
    "----------",
];

/// Sokoban 4a - prize level, variant a (compact for 21-row map)
const SOKOBAN_4A: &[&str] = &[
    "-----------       -----------",
    "|.........|       |.........|",
    "|.........+#######+.........|",
    "|.........|       |.........|",
    "----+------       ------+----",
    "   #  ----         ----  #   ",
    "  ##  |..|         |..|  ##  ",
    "  #  --.----     ----.-  #  ",
    "  #  |.0...|     |..0..|  #  ",
    "  #  |..0..|     |.0...|  #  ",
    "  #  |.0.0.|     |.0.0.|  #  ",
    " --.--..0..|     |.0..--.-|  ",
    " |..+..0.0.|     |.0.0.+..|  ",
    " |..+..0...|     |..0..+..|  ",
    " --.-----.--     --.-----.--  ",
    "  # |....|         |....| #  ",
    "  # |..<.|         |..<.| #  ",
    "  # ------         ------ #  ",
    "  #########+####+-#########  ",
    "           |.^^.|            ",
    "           ------            ",
];

/// Sokoban 4b - prize level, variant b (compact for 21-row map)
const SOKOBAN_4B: &[&str] = &[
    "-----------       -----------",
    "|.........|       |.........|",
    "|.........+#######+.........|",
    "|.........|       |.........|",
    "----+------       ------+----",
    "   #  ----         ----  #   ",
    "  ##  |..|         |..|  ##  ",
    "  #  --.----     ----.-  #  ",
    "  #  |..0..|     |.0...|  #  ",
    "  #  |.0.0.|     |.0.0.|  #  ",
    "  #  |..0..|     |.0...|  #  ",
    " --.--..0..|     |.0..--.-|  ",
    " |..+..0.0.|     |.0.0.+..|  ",
    " |..+.0....|     |...0.+..|  ",
    " --.-----.--     --.-----.--  ",
    "  # |....|         |....| #  ",
    "  # |..<.|         |..<.| #  ",
    "  # ------         ------ #  ",
    "  #########+####+-#########  ",
    "           |.^^.|            ",
    "           ------            ",
];

/// Parse a Sokoban map from ASCII art and apply it to a level.
/// Returns boulder positions and stair positions for further processing.
fn parse_sokoban_map(
    level: &mut Level,
    map: &[&str],
    offset_x: usize,
    offset_y: usize,
) -> (Vec<(usize, usize)>, Vec<(usize, usize, bool)>) {
    let mut boulders = Vec::new();
    let mut stairs = Vec::new(); // (x, y, is_up)

    for (row, line) in map.iter().enumerate() {
        let y = offset_y + row;
        if y >= ROWNO {
            break;
        }
        for (col, ch) in line.chars().enumerate() {
            let x = offset_x + col;
            if x >= COLNO {
                break;
            }
            match ch {
                '-' => {
                    level.cells[x][y].typ = CellType::HWall;
                }
                '|' => {
                    level.cells[x][y].typ = CellType::VWall;
                }
                '.' => {
                    level.cells[x][y].typ = CellType::Room;
                    level.cells[x][y].lit = true;
                }
                '0' => {
                    level.cells[x][y].typ = CellType::Room;
                    level.cells[x][y].lit = true;
                    boulders.push((x, y));
                }
                '^' => {
                    level.cells[x][y].typ = CellType::Room;
                    level.cells[x][y].lit = true;
                    level.add_trap(x as i8, y as i8, TrapType::Hole);
                }
                '<' => {
                    level.cells[x][y].typ = CellType::Stairs;
                    level.cells[x][y].lit = true;
                    stairs.push((x, y, true));
                }
                '>' => {
                    level.cells[x][y].typ = CellType::Stairs;
                    level.cells[x][y].lit = true;
                    stairs.push((x, y, false));
                }
                '+' => {
                    level.cells[x][y].typ = CellType::Door;
                    level.cells[x][y].lit = true;
                }
                '#' => {
                    level.cells[x][y].typ = CellType::Corridor;
                    level.cells[x][y].lit = true;
                }
                ' ' | _ => {
                    // Stone (default)
                }
            }
        }
    }

    (boulders, stairs)
}

/// Set up a canonical Sokoban level from a map definition
fn setup_sokoban_level(
    level: &mut Level,
    map: &[&str],
    level_num: i8,
    _rng: &mut GameRng,
) {
    fill_level(level, CellType::Stone);

    // Set Sokoban flags
    level.flags.sokoban_rules = true;
    level.flags.no_teleport = true;
    level.flags.hard_floor = true; // NON_DIGGABLE equivalent

    // Center the map horizontally
    let map_width = map.iter().map(|l| l.len()).max().unwrap_or(0);
    let offset_x = if map_width < COLNO {
        (COLNO - map_width) / 2
    } else {
        0
    };
    let map_height = map.len();
    let offset_y = if map_height < ROWNO {
        (ROWNO - map_height) / 2
    } else {
        0
    };

    let (_boulders, stairs) = parse_sokoban_map(level, map, offset_x, offset_y);

    // Set up stair destinations
    for (x, y, is_up) in stairs {
        let destination = if is_up {
            if level_num > 1 {
                DLevel::new(3, level_num - 1)
            } else {
                DLevel::new(0, 6) // Back to main dungeon
            }
        } else {
            DLevel::new(3, level_num + 1)
        };
        level.stairs.push(Stairway {
            x: x as i8,
            y: y as i8,
            destination,
            up: is_up,
        });
    }

    // NOTE: Boulder object placement deferred; positions tracked as room cells.
}

/// Generate any Sokoban level from its canonical map
fn generate_sokoban(level: &mut Level, map: &[&str], level_num: i8, rng: &mut GameRng) {
    setup_sokoban_level(level, map, level_num, rng);
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

fn create_surrounding_rooms(
    level: &mut Level,
    cx: usize,
    cy: usize,
    main_w: usize,
    main_h: usize,
    rng: &mut GameRng,
) {
    // Create 4-8 rooms around the central room
    let num_rooms = 4 + rng.rn2(5) as usize;

    for i in 0..num_rooms {
        let angle = (i as f32) * core::f32::consts::TAU / (num_rooms as f32);
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
    for x in (cx - island_w / 2)..(cx + island_w / 2) {
        for y in (cy - island_h / 2)..(cy + island_h / 2) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }

    // Add statues (represented as boulders/stone)
    for _ in 0..8 {
        let sx = cx - island_w / 2 + 2 + rng.rn2((island_w - 4) as u32) as usize;
        let sy = cy - island_h / 2 + 2 + rng.rn2((island_h - 4) as u32) as usize;
        // Statues would be objects, but we mark the spot
        level.cells[sx][sy].typ = CellType::Room;
    }

    // Entry corridor from edge
    for x in 3..10 {
        level.cells[x][cy].typ = CellType::Corridor;
    }

    // Bridge to island
    for x in 10..(cx - island_w / 2) {
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
    for x in (cx - tower_w / 2)..(cx + tower_w / 2) {
        for y in (cy - tower_h / 2)..(cy + tower_h / 2) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = floor == 3; // Top floor is lit
        }
    }

    // Add internal walls for rooms
    if floor < 3 {
        let mid_x = cx;
        for y in (cy - tower_h / 2 + 1)..(cy + tower_h / 2 - 1) {
            level.cells[mid_x][y].typ = CellType::VWall;
        }
        level.cells[mid_x][cy].typ = CellType::Door;
    }

    // Add magic traps
    for _ in 0..(floor as usize + 2) {
        let tx = cx - tower_w / 2 + 2 + rng.rn2((tower_w - 4) as u32) as usize;
        let ty = cy - tower_h / 2 + 2 + rng.rn2((tower_h - 4) as u32) as usize;
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

/// Generate Sanctum (Moloch's temple - final dungeon level before endgame)
fn generate_sanctum(level: &mut Level, _rng: &mut GameRng) {
    fill_level(level, CellType::Stone);
    level.flags.no_teleport = true;
    level.flags.hard_floor = true;
    level.flags.no_magic_map = true;

    // Central sanctum structure
    let cx = 40;
    let cy = 10;
    let sanctum_w = 30;
    let sanctum_h = 12;
    let sanctum_x = cx - sanctum_w / 2;
    let sanctum_y = cy - sanctum_h / 2;

    // Outer lava moat (2 cells wide)
    for x in (sanctum_x - 2)..(sanctum_x + sanctum_w + 2) {
        for y in (sanctum_y - 2)..(sanctum_y + sanctum_h + 2) {
            if x < sanctum_x
                || x >= sanctum_x + sanctum_w
                || y < sanctum_y
                || y >= sanctum_y + sanctum_h
            {
                if (x < sanctum_x || x >= sanctum_x + sanctum_w)
                    && (y >= sanctum_y - 2 && y < sanctum_y + sanctum_h + 2)
                {
                    level.cells[x][y].typ = CellType::Lava;
                } else if (y < sanctum_y || y >= sanctum_y + sanctum_h)
                    && (x >= sanctum_x - 2 && x < sanctum_x + sanctum_w + 2)
                {
                    level.cells[x][y].typ = CellType::Lava;
                }
            }
        }
    }

    // Sanctum walls
    for x in sanctum_x..(sanctum_x + sanctum_w) {
        level.cells[x][sanctum_y].typ = CellType::HWall;
        level.cells[x][sanctum_y + sanctum_h - 1].typ = CellType::HWall;
    }
    for y in sanctum_y..(sanctum_y + sanctum_h) {
        level.cells[sanctum_x][y].typ = CellType::VWall;
        level.cells[sanctum_x + sanctum_w - 1][y].typ = CellType::VWall;
    }

    // Sanctum interior
    for x in (sanctum_x + 1)..(sanctum_x + sanctum_w - 1) {
        for y in (sanctum_y + 1)..(sanctum_y + sanctum_h - 1) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }

    // Central altar chamber (separated by iron bars)
    let altar_x = sanctum_x + sanctum_w / 2 - 2;
    let altar_y = sanctum_y + 3;
    let _altar_w = 5;
    let altar_h = 4;

    // Vertical iron bars separating chambers
    for y in (altar_y + 1)..(altar_y + altar_h - 1) {
        level.cells[sanctum_x + 9][y].typ = CellType::IronBars;
        level.cells[sanctum_x + 20][y].typ = CellType::IronBars;
    }

    // High altar at exact center
    level.cells[altar_x + 2][altar_y + 2].typ = CellType::Altar;

    // Four corner guard rooms
    let corner_w = 5;
    let corner_h = 4;
    let corners = [
        (sanctum_x + 3, sanctum_y + 2),                         // Top-left
        (sanctum_x + sanctum_w - 8, sanctum_y + 2),             // Top-right
        (sanctum_x + 3, sanctum_y + sanctum_h - 6),             // Bottom-left
        (sanctum_x + sanctum_w - 8, sanctum_y + sanctum_h - 6), // Bottom-right
    ];

    for (cx, cy) in corners.iter() {
        for x in *cx..(*cx + corner_w) {
            for y in *cy..(*cy + corner_h) {
                level.cells[x][y].typ = CellType::Room;
                level.cells[x][y].lit = true;
            }
        }
    }

    // Drawbridge at south entrance
    let bridge_x = sanctum_x + sanctum_w / 2;
    level.cells[bridge_x][sanctum_y - 1].typ = CellType::DrawbridgeDown;
    level.cells[bridge_x][sanctum_y].typ = CellType::Door;

    // No exit stairs (player escapes via quest mechanism)
    // Place entry stairs only
    level.cells[sanctum_x + 2][sanctum_y + 2].typ = CellType::Stairs;
    level.stairs.push(Stairway {
        x: (sanctum_x + 2) as i8,
        y: (sanctum_y + 2) as i8,
        destination: DLevel::new(1, 19),
        up: true,
    });
}

/// Generate Vlad's Tower level 1 (bottom - vampire crypts)
fn generate_vlads_tower_1(level: &mut Level, rng: &mut GameRng) {
    generate_vlads_tower_level(level, 1, rng);
}

/// Generate Vlad's Tower level 2 (middle - throne room)
fn generate_vlads_tower_2(level: &mut Level, rng: &mut GameRng) {
    generate_vlads_tower_level(level, 2, rng);
}

/// Generate Vlad's Tower level 3 (top - Vlad's lair)
fn generate_vlads_tower_3(level: &mut Level, rng: &mut GameRng) {
    generate_vlads_tower_level(level, 3, rng);
}

/// Generate a Vlad's Tower level (shared implementation for all 3 floors)
fn generate_vlads_tower_level(level: &mut Level, floor: u8, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);
    level.flags.no_teleport = true;

    // Tower is a central structure (similar to Wizard's Tower but darker and with graves)
    let cx = COLNO / 2;
    let cy = ROWNO / 2;
    let tower_w = 20 - (floor as usize * 2);
    let tower_h = 10 - floor as usize;

    // Create tower room - dark (vampires prefer darkness)
    for x in (cx - tower_w / 2)..(cx + tower_w / 2) {
        for y in (cy - tower_h / 2)..(cy + tower_h / 2) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = floor == 3; // Vlad's level is lit (powerful presence)
        }
    }

    // Add graves (vampire crypts) at each floor
    let num_graves = (floor + 2) as usize;
    for _ in 0..num_graves {
        let gx = cx - tower_w / 2 + 2 + rng.rn2((tower_w - 4) as u32) as usize;
        let gy = cy - tower_h / 2 + 2 + rng.rn2((tower_h - 4) as u32) as usize;
        if level.cells[gx][gy].typ == CellType::Room {
            level.cells[gx][gy].typ = CellType::Grave;
        }
    }

    // Internal walls for rooms (similar to Wizard's tower)
    if floor < 3 {
        let mid_x = cx;
        for y in (cy - tower_h / 2 + 1)..(cy + tower_h / 2 - 1) {
            level.cells[mid_x][y].typ = CellType::VWall;
        }
        level.cells[mid_x][cy].typ = CellType::Door;
    }

    // Blood fountain in center chamber (floor 2)
    if floor == 2 {
        level.cells[cx][cy].typ = CellType::Fountain;
    }

    // Vlad's coffin at center (floor 3)
    if floor == 3 {
        level.cells[cx][cy].typ = CellType::Grave;
    }

    // Stairs
    if floor > 1 {
        // Down stairs
        level.cells[cx - 3][cy].typ = CellType::Stairs;
        level.stairs.push(Stairway {
            x: (cx - 3) as i8,
            y: cy as i8,
            destination: DLevel::new(6, floor as i8 - 1),
            up: false,
        });
    }

    if floor < 3 {
        // Up stairs
        level.cells[cx + 3][cy].typ = CellType::Stairs;
        level.stairs.push(Stairway {
            x: (cx + 3) as i8,
            y: cy as i8,
            destination: DLevel::new(6, floor as i8 + 1),
            up: true,
        });
    }
}

/// Generate Asmodeus's Lair (fortified lava fortress)
fn generate_asmodeus(level: &mut Level, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);

    // Central keep
    let keep_x = 27;
    let keep_y = 5;
    let keep_w = 26;
    let keep_h = 11;

    // Lava moat (3 cells wide)
    for x in (keep_x - 3)..(keep_x + keep_w + 3) {
        for y in (keep_y - 3)..(keep_y + keep_h + 3) {
            if x < keep_x || x >= keep_x + keep_w || y < keep_y || y >= keep_y + keep_h {
                if (x < keep_x || x >= keep_x + keep_w)
                    && (y >= keep_y - 3 && y < keep_y + keep_h + 3)
                {
                    level.cells[x][y].typ = CellType::Lava;
                } else if (y < keep_y || y >= keep_y + keep_h)
                    && (x >= keep_x - 3 && x < keep_x + keep_w + 3)
                {
                    level.cells[x][y].typ = CellType::Lava;
                }
            }
        }
    }

    // Keep walls
    for x in keep_x..(keep_x + keep_w) {
        level.cells[x][keep_y].typ = CellType::HWall;
        level.cells[x][keep_y + keep_h - 1].typ = CellType::HWall;
    }
    for y in keep_y..(keep_y + keep_h) {
        level.cells[keep_x][y].typ = CellType::VWall;
        level.cells[keep_x + keep_w - 1][y].typ = CellType::VWall;
    }

    // Keep interior
    for x in (keep_x + 1)..(keep_x + keep_w - 1) {
        for y in (keep_y + 1)..(keep_y + keep_h - 1) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }

    // Throne at center
    let throne_x = keep_x + keep_w / 2;
    let throne_y = keep_y + keep_h / 2;
    level.cells[throne_x][throne_y].typ = CellType::Throne;

    // Four corner towers
    let tower_w = 6;
    let tower_h = 6;
    let towers = [
        (keep_x - 3, keep_y - 3),
        (keep_x + keep_w - 3, keep_y - 3),
        (keep_x - 3, keep_y + keep_h - 3),
        (keep_x + keep_w - 3, keep_y + keep_h - 3),
    ];

    for (tx, ty) in towers.iter() {
        for x in *tx..(*tx + tower_w) {
            for y in *ty..(*ty + tower_h) {
                if x > 0 && x < COLNO && y > 0 && y < ROWNO {
                    level.cells[x][y].typ = CellType::Room;
                    level.cells[x][y].lit = true;
                }
            }
        }
    }

    // Drawbridge entrance at south
    level.cells[throne_x][keep_y - 1].typ = CellType::DrawbridgeDown;
    level.cells[throne_x][keep_y].typ = CellType::Door;

    // Fire traps scattered throughout
    for _ in 0..10 {
        let fx = keep_x + 2 + rng.rn2((keep_w - 4) as u32) as usize;
        let fy = keep_y + 2 + rng.rn2((keep_h - 4) as u32) as usize;
        if level.cells[fx][fy].typ == CellType::Room {
            level.add_trap(fx as i8, fy as i8, TrapType::FireTrap);
        }
    }

    place_special_stairs(level, rng);
}

/// Generate Juiblex's Swamp (water-filled with islands)
fn generate_juiblex(level: &mut Level, rng: &mut GameRng) {
    fill_level(level, CellType::Water);

    // Fill 70% with water, create islands
    for x in 0..COLNO {
        for y in 0..ROWNO {
            if rng.rn2(10) < 7 {
                level.cells[x][y].typ = CellType::Water;
            }
        }
    }

    // Central large island
    let cx = 35;
    let cy = 8;
    let central_w = 15;
    let central_h = 8;
    for x in (cx - central_w / 2)..(cx + central_w / 2) {
        for y in (cy - central_h / 2)..(cy + central_h / 2) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = false; // Dark swamp
        }
    }

    // 10-12 smaller island rooms
    let num_islands = 10 + rng.rn2(3) as usize;
    for _ in 0..num_islands {
        let ix = 5 + rng.rn2(70) as usize;
        let iy = 2 + rng.rn2(17) as usize;
        let iw = 4 + rng.rn2(5) as usize;
        let ih = 3 + rng.rn2(3) as usize;

        for x in ix..(ix + iw).min(COLNO - 1) {
            for y in iy..(iy + ih).min(ROWNO - 1) {
                if level.cells[x][y].typ == CellType::Water {
                    level.cells[x][y].typ = CellType::Room;
                    level.cells[x][y].lit = false;
                }
            }
        }
    }

    // Stone bridges connecting islands (1 cell wide)
    for _ in 0..8 {
        let x1 = 5 + rng.rn2(70) as usize;
        let y1 = 2 + rng.rn2(17) as usize;
        let x2 = 5 + rng.rn2(70) as usize;
        let y2 = 2 + rng.rn2(17) as usize;
        connect_points(level, x1, y1, x2, y2);
    }

    // Slime fountain in central chamber
    level.cells[cx][cy].typ = CellType::Fountain;

    // Deep pool cells (for flavor)
    for _ in 0..15 {
        let px = 5 + rng.rn2(70) as usize;
        let py = 2 + rng.rn2(17) as usize;
        if level.cells[px][py].typ == CellType::Water {
            level.cells[px][py].typ = CellType::Pool;
        }
    }

    place_special_stairs(level, rng);
}

/// Generate Baalzebub's Lair (maze-like with many traps)
fn generate_baalzebub(level: &mut Level, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);

    // Central throne room
    let throne_x = 35;
    let throne_y = 8;
    let throne_w = 10;
    let throne_h = 6;

    for x in throne_x..(throne_x + throne_w) {
        for y in throne_y..(throne_y + throne_h) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }

    level.cells[throne_x + throne_w / 2][throne_y + throne_h / 2].typ = CellType::Throne;

    // Ring 1: 4 chambers around throne
    let ring1_chambers = [
        (throne_x - 8, throne_y + 1, 6, 4),
        (throne_x + throne_w + 2, throne_y + 1, 6, 4),
        (throne_x + 2, throne_y - 6, 6, 4),
        (throne_x + 2, throne_y + throne_h + 2, 6, 4),
    ];

    for (cx, cy, cw, ch) in ring1_chambers.iter() {
        for x in *cx..(*cx + cw).min(COLNO - 1) {
            for y in *cy..(*cy + ch).min(ROWNO - 1) {
                level.cells[x][y].typ = CellType::Room;
                level.cells[x][y].lit = true;
            }
        }
        // Connect to throne with winding corridor
        connect_points(
            level,
            *cx + cw / 2,
            *cy + ch / 2,
            throne_x + throne_w / 2,
            throne_y + throne_h / 2,
        );
    }

    // Ring 2: 6-8 outer chambers
    let num_outer = 6 + rng.rn2(3) as usize;
    for _ in 0..num_outer {
        let ox = 10 + rng.rn2(50) as usize;
        let oy = 2 + rng.rn2(15) as usize;
        let ow = 4 + rng.rn2(4) as usize;
        let oh = 3 + rng.rn2(3) as usize;

        for x in ox..(ox + ow).min(COLNO - 1) {
            for y in oy..(oy + oh).min(ROWNO - 1) {
                if level.cells[x][y].typ == CellType::Stone {
                    level.cells[x][y].typ = CellType::Room;
                    level.cells[x][y].lit = true;
                }
            }
        }
    }

    // Random connections between outer chambers
    connect_rooms_randomly(level, rng);

    // Traps: Arrow, Dart, Pit, SpikedPit, Teleport
    let trap_types = [
        TrapType::Arrow,
        TrapType::Dart,
        TrapType::Pit,
        TrapType::SpikedPit,
        TrapType::Teleport,
    ];

    for _ in 0..12 {
        let tx = 10 + rng.rn2(60) as usize;
        let ty = 2 + rng.rn2(17) as usize;
        if level.cells[tx][ty].typ == CellType::Room {
            let trap_idx = rng.rn2(trap_types.len() as u32) as usize;
            level.add_trap(tx as i8, ty as i8, trap_types[trap_idx]);
        }
    }

    place_special_stairs(level, rng);
}

fn place_special_stairs(level: &mut Level, rng: &mut GameRng) {
    // Find valid positions for stairs
    let mut valid_positions: Vec<(usize, usize)> = Vec::new();

    for x in 3..(COLNO - 3) {
        for y in 2..(ROWNO - 2) {
            if level.cells[x][y].typ == CellType::Room
                || level.cells[x][y].typ == CellType::Corridor
            {
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
        assert_eq!(
            get_special_level(&DLevel::new(0, 5)),
            Some(SpecialLevelId::Oracle)
        );
        assert_eq!(
            get_special_level(&DLevel::new(2, 3)),
            Some(SpecialLevelId::MinesTown)
        );
        assert_eq!(
            get_special_level(&DLevel::new(3, 1)),
            Some(SpecialLevelId::Sokoban1a)
        );
        assert_eq!(get_special_level(&DLevel::new(0, 10)), None); // Not a special level
    }

    #[test]
    fn test_oracle_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(0, 5));

        generate_special_level(&mut level, SpecialLevelId::Oracle, &mut rng);

        // Should have fountains
        let fountain_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Fountain)
            .count();

        assert!(
            fountain_count >= 4,
            "Oracle should have at least 4 fountains"
        );

        // Should have stairs
        assert!(!level.stairs.is_empty(), "Oracle should have stairs");
    }

    #[test]
    fn test_castle_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(0, 25));

        generate_special_level(&mut level, SpecialLevelId::Castle, &mut rng);

        // Should have moat
        let moat_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Moat)
            .count();

        assert!(moat_count > 50, "Castle should have moat");

        // Should have throne
        let throne_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Throne)
            .count();

        assert_eq!(throne_count, 1, "Castle should have one throne");
    }

    #[test]
    fn test_sokoban_1a_canonical() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(3, 1));

        generate_special_level(&mut level, SpecialLevelId::Sokoban1a, &mut rng);

        // Entry level (1a) has no hole traps - those are on higher floors
        // Should have stairs (up stairs '<')
        assert!(!level.stairs.is_empty(), "Sokoban 1a should have stairs");
        assert!(level.stairs.iter().any(|s| s.up), "Sokoban 1a should have up stairs");

        // Should have sokoban flags
        assert!(level.flags.sokoban_rules, "Should have sokoban_rules flag");
        assert!(level.flags.no_teleport, "Should have no_teleport flag");
        assert!(level.flags.hard_floor, "Should have hard_floor (non-diggable) flag");

        // Verify canonical layout has walls
        let wall_count = level.cells.iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::HWall || c.typ == CellType::VWall)
            .count();
        assert!(wall_count > 20, "Sokoban 1a should have many walls from canonical layout, got {}", wall_count);

        // Should have doors
        let door_count = level.cells.iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Door)
            .count();
        assert!(door_count >= 1, "Sokoban 1a should have doors, got {}", door_count);
    }

    #[test]
    fn test_sokoban_1b_canonical() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(3, 1));

        generate_special_level(&mut level, SpecialLevelId::Sokoban1b, &mut rng);

        assert!(!level.stairs.is_empty(), "Sokoban 1b should have stairs");
        assert!(level.flags.sokoban_rules);
    }

    #[test]
    fn test_sokoban_2a_has_holes() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(3, 2));

        generate_special_level(&mut level, SpecialLevelId::Sokoban2a, &mut rng);

        // Level 2a has 4 hole traps (^^^^)
        let hole_count = level.traps.iter()
            .filter(|t| t.trap_type == TrapType::Hole)
            .count();
        assert_eq!(hole_count, 4, "Sokoban 2a should have exactly 4 hole traps, got {}", hole_count);
        assert!(level.flags.sokoban_rules);
    }

    #[test]
    fn test_sokoban_4a_prize_level() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(3, 4));

        generate_special_level(&mut level, SpecialLevelId::Sokoban4a, &mut rng);

        // Prize level has hole traps ('^' in map) - 2 total (^^)
        let hole_count = level.traps.iter()
            .filter(|t| t.trap_type == TrapType::Hole)
            .count();
        assert_eq!(hole_count, 2, "Sokoban 4a should have 2 hole traps, got {}", hole_count);

        // Should have corridors ('#' in map connecting rooms)
        let corridor_count = level.cells.iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Corridor)
            .count();
        assert!(corridor_count > 10, "Sokoban 4a should have corridor connections, got {}", corridor_count);

        // Should have doors ('+' in map)
        let door_count = level.cells.iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Door)
            .count();
        assert!(door_count >= 4, "Sokoban 4a should have doors, got {}", door_count);

        assert!(level.flags.sokoban_rules);
        assert!(level.flags.no_teleport);
    }

    #[test]
    fn test_all_sokoban_variants_generate() {
        let variants = [
            (SpecialLevelId::Sokoban1a, "1a"),
            (SpecialLevelId::Sokoban1b, "1b"),
            (SpecialLevelId::Sokoban2a, "2a"),
            (SpecialLevelId::Sokoban2b, "2b"),
            (SpecialLevelId::Sokoban3a, "3a"),
            (SpecialLevelId::Sokoban3b, "3b"),
            (SpecialLevelId::Sokoban4a, "4a"),
            (SpecialLevelId::Sokoban4b, "4b"),
        ];

        for (id, name) in variants {
            let mut rng = GameRng::new(42);
            let dlevel = id.location();
            let mut level = Level::new(dlevel);

            generate_special_level(&mut level, id, &mut rng);

            assert!(level.flags.sokoban_rules, "Sokoban {} missing sokoban_rules", name);
            assert!(level.flags.no_teleport, "Sokoban {} missing no_teleport", name);
            assert!(level.flags.hard_floor, "Sokoban {} missing hard_floor", name);
            assert!(!level.stairs.is_empty(), "Sokoban {} has no stairs", name);

            // Every variant should have walls from the canonical layout
            let wall_count = level.cells.iter()
                .flat_map(|col| col.iter())
                .filter(|c| c.typ == CellType::HWall || c.typ == CellType::VWall)
                .count();
            assert!(wall_count > 15, "Sokoban {} should have walls, got {}", name, wall_count);
        }
    }

    #[test]
    fn test_minetown_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(2, 3));

        generate_special_level(&mut level, SpecialLevelId::MinesTown, &mut rng);

        // Should have altar (temple)
        let altar_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Altar)
            .count();

        assert!(altar_count >= 1, "Minetown should have altar");

        // Should have fountains
        let fountain_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Fountain)
            .count();

        assert!(fountain_count >= 1, "Minetown should have fountains");
    }

    #[test]
    fn test_sanctum_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(1, 20));

        generate_special_level(&mut level, SpecialLevelId::Sanctum, &mut rng);

        // Verify lava moat exists
        let lava_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Lava)
            .count();
        assert!(lava_count > 50, "Sanctum should have lava moat");

        // Verify high altar exists
        let altar_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Altar)
            .count();
        assert_eq!(altar_count, 1, "Sanctum should have one high altar");

        // Verify flags
        assert!(
            level.flags.no_teleport,
            "Sanctum should have no_teleport flag"
        );
        assert!(
            level.flags.hard_floor,
            "Sanctum should have hard_floor flag"
        );
        assert!(
            level.flags.no_magic_map,
            "Sanctum should have no_magic_map flag"
        );

        // Verify stairs exist
        assert!(!level.stairs.is_empty(), "Sanctum should have stairs");
    }

    #[test]
    fn test_vlads_tower_1_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(6, 1));

        generate_special_level(&mut level, SpecialLevelId::VladsTower1, &mut rng);

        // Should have graves (vampire crypts)
        let grave_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Grave)
            .count();
        assert!(grave_count > 0, "Vlad's Tower 1 should have graves");

        // Floor 1 only has up stairs (no down stairs since floor == 1)
        assert!(level.stairs.len() >= 1, "Vlad's Tower 1 should have stairs");

        // Should have no_teleport flag
        assert!(
            level.flags.no_teleport,
            "Vlad's Tower should have no_teleport flag"
        );
    }

    #[test]
    fn test_vlads_tower_2_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(6, 2));

        generate_special_level(&mut level, SpecialLevelId::VladsTower2, &mut rng);

        // Should have fountain (blood fountain)
        let fountain_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Fountain)
            .count();
        assert_eq!(
            fountain_count, 1,
            "Vlad's Tower 2 should have blood fountain"
        );

        // Should have stairs
        assert!(level.stairs.len() >= 2, "Vlad's Tower 2 should have stairs");
    }

    #[test]
    fn test_vlads_tower_3_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(6, 3));

        generate_special_level(&mut level, SpecialLevelId::VladsTower3, &mut rng);

        // Should have Vlad's coffin (grave at center)
        let grave_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Grave)
            .count();
        assert!(grave_count > 0, "Vlad's Tower 3 should have Vlad's coffin");

        // Should only have down stairs (no up from top)
        assert!(level.stairs.len() >= 1, "Vlad's Tower 3 should have stairs");
    }

    #[test]
    fn test_asmodeus_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(1, 15));

        generate_special_level(&mut level, SpecialLevelId::Asmodeus, &mut rng);

        // Should have lava moat
        let lava_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Lava)
            .count();
        assert!(lava_count > 50, "Asmodeus should have lava moat");

        // Should have throne
        let throne_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Throne)
            .count();
        assert_eq!(throne_count, 1, "Asmodeus should have one throne");

        // Should have fire traps
        assert!(!level.traps.is_empty(), "Asmodeus should have fire traps");

        // Should have stairs
        assert!(!level.stairs.is_empty(), "Asmodeus should have stairs");
    }

    #[test]
    fn test_juiblex_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(1, 5));

        generate_special_level(&mut level, SpecialLevelId::Juiblex, &mut rng);

        // Should have water
        let water_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Water || c.typ == CellType::Pool)
            .count();
        assert!(
            water_count > 100,
            "Juiblex should have significant water coverage"
        );

        // Should have islands (room cells)
        let room_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Room)
            .count();
        assert!(room_count > 50, "Juiblex should have island rooms");

        // Should have fountain
        let fountain_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Fountain)
            .count();
        assert!(fountain_count >= 1, "Juiblex should have fountain");

        // Should have stairs
        assert!(!level.stairs.is_empty(), "Juiblex should have stairs");
    }

    #[test]
    fn test_baalzebub_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(1, 10));

        generate_special_level(&mut level, SpecialLevelId::Baalzebub, &mut rng);

        // Should have throne
        let throne_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Throne)
            .count();
        assert_eq!(throne_count, 1, "Baalzebub should have throne");

        // Should have some traps (up to 12 attempts, placed only on Room cells)
        assert!(
            !level.traps.is_empty(),
            "Baalzebub should have multiple traps"
        );

        // Should have multiple rooms (chambers)
        let room_count = level
            .cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Room)
            .count();
        assert!(room_count > 100, "Baalzebub should have multiple chambers");

        // Should have stairs
        assert!(!level.stairs.is_empty(), "Baalzebub should have stairs");
    }

    #[test]
    fn test_endgame_planes_generation() {
        let mut rng = GameRng::new(42);

        // Test Astral Plane
        let mut level = Level::new(DLevel::new(7, 5));
        generate_special_level(&mut level, SpecialLevelId::AstralPlane, &mut rng);
        assert!(!level.stairs.is_empty(), "Astral Plane should have stairs");

        // Test Earth Plane
        let mut level = Level::new(DLevel::new(7, 1));
        generate_special_level(&mut level, SpecialLevelId::EarthPlane, &mut rng);
        assert!(!level.stairs.is_empty(), "Earth Plane should have stairs");

        // Test Air Plane
        let mut level = Level::new(DLevel::new(7, 2));
        generate_special_level(&mut level, SpecialLevelId::AirPlane, &mut rng);
        assert!(!level.stairs.is_empty(), "Air Plane should have stairs");

        // Test Fire Plane
        let mut level = Level::new(DLevel::new(7, 3));
        generate_special_level(&mut level, SpecialLevelId::FirePlane, &mut rng);
        assert!(!level.stairs.is_empty(), "Fire Plane should have stairs");

        // Test Water Plane
        let mut level = Level::new(DLevel::new(7, 4));
        generate_special_level(&mut level, SpecialLevelId::WaterPlane, &mut rng);
        assert!(!level.stairs.is_empty(), "Water Plane should have stairs");
    }
}
