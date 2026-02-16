//! Room types and structures (mkroom.h)
//!
//! Defines room types matching NetHack C:
//! - 25 room types from OROOM (0) to CANDLESHOP (25)
//! - Room struct with type, dimensions, and properties

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::rng::GameRng;
use serde::{Deserialize, Serialize};

/// Room types matching C mkroom.h enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum RoomType {
    /// Ordinary room (OROOM = 0)
    #[default]
    Ordinary = 0,
    // Note: 1 is unused in C
    /// Throne room with king/queen (COURT = 2)
    Court = 2,
    /// Swamp with pools and eels (SWAMP = 3)
    Swamp = 3,
    /// Secret vault with gold (VAULT = 4)
    Vault = 4,
    /// Bee hive with queen bee (BEEHIVE = 5)
    Beehive = 5,
    /// Morgue with undead (MORGUE = 6)
    Morgue = 6,
    /// Soldier barracks (BARRACKS = 7)
    Barracks = 7,
    /// Zoo with caged monsters (ZOO = 8)
    Zoo = 8,
    /// Oracle's chamber (DELPHI = 9)
    Delphi = 9,
    /// Temple with altar and priest (TEMPLE = 10)
    Temple = 10,
    /// Leprechaun treasure hall (LEPREHALL = 11)
    LeprechaunHall = 11,
    /// Cockatrice nest with statues (COCKNEST = 12)
    CockatriceNest = 12,
    /// Ant colony (ANTHOLE = 13)
    Anthole = 13,
    /// General store (SHOPBASE = 14)
    GeneralShop = 14,
    /// Armor shop (ARMORSHOP = 15)
    ArmorShop = 15,
    /// Scroll shop (SCROLLSHOP = 16)
    ScrollShop = 16,
    /// Potion shop (POTIONSHOP = 17)
    PotionShop = 17,
    /// Weapon shop (WEAPONSHOP = 18)
    WeaponShop = 18,
    /// Food shop (FOODSHOP = 19)
    FoodShop = 19,
    /// Ring shop (RINGSHOP = 20)
    RingShop = 20,
    /// Wand shop (WANDSHOP = 21)
    WandShop = 21,
    /// Tool shop (TOOLSHOP = 22)
    ToolShop = 22,
    /// Bookstore (BOOKSHOP = 23)
    BookShop = 23,
    /// Health food store (FODDERSHOP = 24)
    HealthFoodShop = 24,
    /// Candle shop (CANDLESHOP = 25)
    CandleShop = 25,
}

impl RoomType {
    /// All room types for iteration
    pub const ALL: [RoomType; 25] = [
        RoomType::Ordinary,
        RoomType::Court,
        RoomType::Swamp,
        RoomType::Vault,
        RoomType::Beehive,
        RoomType::Morgue,
        RoomType::Barracks,
        RoomType::Zoo,
        RoomType::Delphi,
        RoomType::Temple,
        RoomType::LeprechaunHall,
        RoomType::CockatriceNest,
        RoomType::Anthole,
        RoomType::GeneralShop,
        RoomType::ArmorShop,
        RoomType::ScrollShop,
        RoomType::PotionShop,
        RoomType::WeaponShop,
        RoomType::FoodShop,
        RoomType::RingShop,
        RoomType::WandShop,
        RoomType::ToolShop,
        RoomType::BookShop,
        RoomType::HealthFoodShop,
        RoomType::CandleShop,
    ];

    /// All special (non-ordinary, non-shop) room types
    pub const SPECIAL_ROOMS: [RoomType; 12] = [
        RoomType::Court,
        RoomType::Swamp,
        RoomType::Vault,
        RoomType::Beehive,
        RoomType::Morgue,
        RoomType::Barracks,
        RoomType::Zoo,
        RoomType::Delphi,
        RoomType::Temple,
        RoomType::LeprechaunHall,
        RoomType::CockatriceNest,
        RoomType::Anthole,
    ];

    /// All shop types
    pub const SHOP_TYPES: [RoomType; 12] = [
        RoomType::GeneralShop,
        RoomType::ArmorShop,
        RoomType::ScrollShop,
        RoomType::PotionShop,
        RoomType::WeaponShop,
        RoomType::FoodShop,
        RoomType::RingShop,
        RoomType::WandShop,
        RoomType::ToolShop,
        RoomType::BookShop,
        RoomType::HealthFoodShop,
        RoomType::CandleShop,
    ];

    /// Check if this is a shop type
    pub fn is_shop(self) -> bool {
        matches!(
            self,
            RoomType::GeneralShop
                | RoomType::ArmorShop
                | RoomType::ScrollShop
                | RoomType::PotionShop
                | RoomType::WeaponShop
                | RoomType::FoodShop
                | RoomType::RingShop
                | RoomType::WandShop
                | RoomType::ToolShop
                | RoomType::BookShop
                | RoomType::HealthFoodShop
                | RoomType::CandleShop
        )
    }

    /// Check if this is a special room (non-ordinary, non-shop)
    pub fn is_special(self) -> bool {
        !matches!(self, RoomType::Ordinary) && !self.is_shop()
    }

    /// Check if this room type has monsters that should be asleep
    pub fn monsters_sleep(self) -> bool {
        matches!(
            self,
            RoomType::Court
                | RoomType::Beehive
                | RoomType::Morgue
                | RoomType::Barracks
                | RoomType::Zoo
                | RoomType::LeprechaunHall
                | RoomType::CockatriceNest
                | RoomType::Anthole
        )
    }

    /// Minimum dungeon depth for this room type to spawn
    /// Returns None if the room can spawn at any depth or is not randomly generated
    pub fn min_depth(self) -> Option<u8> {
        match self {
            RoomType::Ordinary => Some(1),
            RoomType::Court => Some(4),
            RoomType::Swamp => Some(15),
            RoomType::Vault => Some(1),
            RoomType::Beehive => Some(9),
            RoomType::Morgue => Some(11),
            RoomType::Barracks => Some(14),
            RoomType::Zoo => Some(6),
            RoomType::Delphi => None, // Special level only
            RoomType::Temple => Some(8),
            RoomType::LeprechaunHall => Some(5),
            RoomType::CockatriceNest => Some(16),
            RoomType::Anthole => Some(12),
            // Shops can spawn from depth 2
            _ if self.is_shop() => Some(2),
            _ => None,
        }
    }

    /// Spawn probability as (numerator, denominator)
    /// E.g., (1, 6) means 1-in-6 chance
    pub fn spawn_probability(self) -> Option<(u8, u8)> {
        match self {
            RoomType::Court => Some((1, 6)),
            RoomType::Swamp => Some((1, 6)),
            RoomType::Vault => Some((1, 2)),
            RoomType::Beehive => Some((1, 5)),
            RoomType::Morgue => Some((1, 6)),
            RoomType::Barracks => Some((1, 4)),
            RoomType::Zoo => Some((1, 7)),
            RoomType::Temple => Some((1, 5)),
            RoomType::LeprechaunHall => Some((1, 8)),
            RoomType::CockatriceNest => Some((1, 8)),
            RoomType::Anthole => Some((1, 5)),
            // Shops have special probability logic
            _ if self.is_shop() => Some((3, 100)),
            _ => None,
        }
    }

    /// Get C name for this room type
    pub fn c_name(self) -> &'static str {
        match self {
            RoomType::Ordinary => "OROOM",
            RoomType::Court => "COURT",
            RoomType::Swamp => "SWAMP",
            RoomType::Vault => "VAULT",
            RoomType::Beehive => "BEEHIVE",
            RoomType::Morgue => "MORGUE",
            RoomType::Barracks => "BARRACKS",
            RoomType::Zoo => "ZOO",
            RoomType::Delphi => "DELPHI",
            RoomType::Temple => "TEMPLE",
            RoomType::LeprechaunHall => "LEPREHALL",
            RoomType::CockatriceNest => "COCKNEST",
            RoomType::Anthole => "ANTHOLE",
            RoomType::GeneralShop => "SHOPBASE",
            RoomType::ArmorShop => "ARMORSHOP",
            RoomType::ScrollShop => "SCROLLSHOP",
            RoomType::PotionShop => "POTIONSHOP",
            RoomType::WeaponShop => "WEAPONSHOP",
            RoomType::FoodShop => "FOODSHOP",
            RoomType::RingShop => "RINGSHOP",
            RoomType::WandShop => "WANDSHOP",
            RoomType::ToolShop => "TOOLSHOP",
            RoomType::BookShop => "BOOKSHOP",
            RoomType::HealthFoodShop => "FODDERSHOP",
            RoomType::CandleShop => "CANDLESHOP",
        }
    }

    /// Get description for this room type
    pub fn description(self) -> &'static str {
        match self {
            RoomType::Ordinary => "Ordinary room",
            RoomType::Court => "Throne room with king/queen",
            RoomType::Swamp => "Swamp with pools and eels",
            RoomType::Vault => "Secret vault with gold",
            RoomType::Beehive => "Bee hive with queen bee",
            RoomType::Morgue => "Morgue with undead",
            RoomType::Barracks => "Soldier barracks",
            RoomType::Zoo => "Zoo with caged monsters",
            RoomType::Delphi => "Oracle's chamber",
            RoomType::Temple => "Temple with altar and priest",
            RoomType::LeprechaunHall => "Leprechaun treasure hall",
            RoomType::CockatriceNest => "Cockatrice nest with statues",
            RoomType::Anthole => "Ant colony",
            RoomType::GeneralShop => "General store",
            RoomType::ArmorShop => "Armor shop",
            RoomType::ScrollShop => "Scroll shop",
            RoomType::PotionShop => "Potion shop",
            RoomType::WeaponShop => "Weapon shop",
            RoomType::FoodShop => "Food shop",
            RoomType::RingShop => "Ring shop",
            RoomType::WandShop => "Wand shop",
            RoomType::ToolShop => "Tool shop",
            RoomType::BookShop => "Bookstore",
            RoomType::HealthFoodShop => "Health food store",
            RoomType::CandleShop => "Candle shop",
        }
    }
}

/// Maximum number of subrooms per room (from C mkroom.h)
pub const MAX_SUBROOMS: usize = 24;

/// Rectangle representing a room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    /// X coordinate of room interior (left edge)
    pub x: usize,
    /// Y coordinate of room interior (top edge)
    pub y: usize,
    /// Width of room interior
    pub width: usize,
    /// Height of room interior
    pub height: usize,
    /// Type of room
    pub room_type: RoomType,
    /// Whether the room is lit
    pub lit: bool,
    /// Number of doors in this room
    pub door_count: u8,
    /// Index of first door in level's door array
    pub first_door_idx: u8,
    /// Whether this room has irregular shape
    pub irregular: bool,
    /// Parent room index (if this is a subroom)
    pub parent: Option<usize>,
    /// Subroom indices
    pub subrooms: Vec<usize>,
}

impl Default for Room {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            room_type: RoomType::Ordinary,
            lit: true,
            door_count: 0,
            first_door_idx: 0,
            irregular: false,
            parent: None,
            subrooms: Vec::new(),
        }
    }
}

impl Room {
    /// Create a new ordinary room
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
            room_type: RoomType::Ordinary,
            lit: true,
            door_count: 0,
            first_door_idx: 0,
            irregular: false,
            parent: None,
            subrooms: Vec::new(),
        }
    }

    /// Create a room with a specific type
    pub fn with_type(x: usize, y: usize, width: usize, height: usize, room_type: RoomType) -> Self {
        Self {
            x,
            y,
            width,
            height,
            room_type,
            lit: !matches!(room_type, RoomType::Morgue | RoomType::Vault),
            door_count: 0,
            first_door_idx: 0,
            irregular: false,
            parent: None,
            subrooms: Vec::new(),
        }
    }

    /// Create a subroom within a parent room
    pub fn new_subroom(x: usize, y: usize, width: usize, height: usize, parent_idx: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
            room_type: RoomType::Ordinary,
            lit: true,
            door_count: 0,
            first_door_idx: 0,
            irregular: false,
            parent: Some(parent_idx),
            subrooms: Vec::new(),
        }
    }

    /// Check if this is a subroom
    pub fn is_subroom(&self) -> bool {
        self.parent.is_some()
    }

    /// Check if this room has subrooms
    pub fn has_subrooms(&self) -> bool {
        !self.subrooms.is_empty()
    }

    /// Add a subroom index
    pub fn add_subroom(&mut self, subroom_idx: usize) {
        if self.subrooms.len() < MAX_SUBROOMS {
            self.subrooms.push(subroom_idx);
        }
    }

    /// Check if this room overlaps with another (with buffer)
    pub fn overlaps(&self, other: &Room, buffer: usize) -> bool {
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

    /// Get room area (interior cells)
    pub fn area(&self) -> usize {
        self.width * self.height
    }

    /// Check if room is a shop
    pub fn is_shop(&self) -> bool {
        self.room_type.is_shop()
    }

    /// Check if room is a special room
    pub fn is_special(&self) -> bool {
        self.room_type.is_special()
    }

    /// Get bounds as (left, top, right, bottom)
    pub fn bounds(&self) -> (usize, usize, usize, usize) {
        (
            self.x,
            self.y,
            self.x + self.width - 1,
            self.y + self.height - 1,
        )
    }

    /// Get wall bounds (including walls) as (left, top, right, bottom)
    pub fn wall_bounds(&self) -> (usize, usize, usize, usize) {
        (
            self.x.saturating_sub(1),
            self.y.saturating_sub(1),
            self.x + self.width,
            self.y + self.height,
        )
    }
}

// ============================================================================
// Room query and utility functions (from C's mkroom.c)
// ============================================================================

/// Get a random X coordinate within a room (somex equivalent)
///
/// Returns a random X coordinate within the room's interior bounds.
pub fn somex(room: &Room, rng: &mut GameRng) -> usize {
    let lx = room.x;
    let hx = room.x + room.width - 1;
    lx + rng.rn2((hx - lx + 1) as u32) as usize
}

/// Get a random Y coordinate within a room (somey equivalent)
///
/// Returns a random Y coordinate within the room's interior bounds.
pub fn somey(room: &Room, rng: &mut GameRng) -> usize {
    let ly = room.y;
    let hy = room.y + room.height - 1;
    ly + rng.rn2((hy - ly + 1) as u32) as usize
}

/// Check if coordinates are inside a room including its walls (inside_room equivalent)
///
/// Returns true if (x, y) is within the room's walls (1 cell buffer around interior).
pub fn inside_room(room: &Room, x: usize, y: usize) -> bool {
    let lx = room.x.saturating_sub(1);
    let ly = room.y.saturating_sub(1);
    let hx = room.x + room.width; // +1 for wall
    let hy = room.y + room.height; // +1 for wall

    x >= lx && x <= hx && y >= ly && y <= hy
}

/// Get random coordinates within a room, avoiding subrooms and walls (somexy equivalent)
///
/// For irregular rooms or rooms with subrooms, this searches for a valid interior cell.
/// Returns Some((x, y)) if found, None if no valid cell exists.
///
/// # Arguments
/// * `room` - The room to search within
/// * `room_index` - The index of this room in the rooms array
/// * `all_rooms` - All rooms (needed for subroom checking)
/// * `level` - Level data (needed for cell type checking)
/// * `rng` - Random number generator
pub fn somexy(
    room: &Room,
    room_index: usize,
    all_rooms: &[Room],
    level: &super::Level,
    rng: &mut GameRng,
) -> Option<(usize, usize)> {
    use super::CellType;
    use super::generation::ROOMOFFSET;

    let roomno = (room_index + ROOMOFFSET as usize) as u8;

    // For irregular rooms, need to check roomno
    if room.irregular {
        // Try random positions first
        for _ in 0..100 {
            let x = somex(room, rng);
            let y = somey(room, rng);

            if !level.cells[x][y].edge && level.cells[x][y].room_number == roomno {
                return Some((x, y));
            }
        }

        // Exhaustive search fallback
        for x in room.x..(room.x + room.width) {
            for y in room.y..(room.y + room.height) {
                if !level.cells[x][y].edge && level.cells[x][y].room_number == roomno {
                    return Some((x, y));
                }
            }
        }

        return None;
    }

    // Simple case: no subrooms
    if room.subrooms.is_empty() {
        let x = somex(room, rng);
        let y = somey(room, rng);
        return Some((x, y));
    }

    // Room has subrooms - need to avoid them
    for _ in 0..100 {
        let x = somex(room, rng);
        let y = somey(room, rng);

        // Check if it's a wall
        if level.cells[x][y].typ.is_wall() {
            continue;
        }

        // Check if it's inside any subroom
        let in_subroom = room.subrooms.iter().any(|&subroom_idx| {
            if subroom_idx < all_rooms.len() {
                inside_room(&all_rooms[subroom_idx], x, y)
            } else {
                false
            }
        });

        if !in_subroom {
            return Some((x, y));
        }
    }

    None
}

/// Pick an unused ordinary room (pick_room equivalent)
///
/// Selects a random room that is ordinary (not special), preferably with only one door.
///
/// # Arguments
/// * `rooms` - Array of rooms to search
/// * `level` - Level to check for stairs
/// * `strict` - If true, reject rooms with any stairs; if false, only reject upstairs
/// * `rng` - Random number generator
///
/// # Returns
/// Index of a suitable room, or None if none found
pub fn pick_room(
    rooms: &[Room],
    level: &super::Level,
    strict: bool,
    rng: &mut GameRng,
) -> Option<usize> {
    if rooms.is_empty() {
        return None;
    }

    let start = rng.rn2(rooms.len() as u32) as usize;

    for i in 0..rooms.len() {
        let idx = (start + i) % rooms.len();
        let room = &rooms[idx];

        // Skip non-ordinary rooms
        if room.room_type != RoomType::Ordinary {
            continue;
        }

        // Check for stairs
        let has_up = room_has_upstairs(room, level);
        let has_down = room_has_downstairs(room, level);

        if strict {
            if has_up || has_down {
                continue;
            }
        } else {
            // Skip if has upstairs, or 2/3 chance to skip if has downstairs
            if has_up || (has_down && !rng.one_in(3)) {
                continue;
            }
        }

        // Prefer rooms with only one door, or 1-in-5 chance for others
        if room.door_count == 1 || rng.one_in(5) {
            return Some(idx);
        }
    }

    None
}

/// Check if room has upstairs
fn room_has_upstairs(room: &Room, level: &super::Level) -> bool {
    for stair in &level.stairs {
        if stair.up && room.contains(stair.x as usize, stair.y as usize) {
            return true;
        }
    }
    false
}

/// Check if room has downstairs
fn room_has_downstairs(room: &Room, level: &super::Level) -> bool {
    for stair in &level.stairs {
        if !stair.up && room.contains(stair.x as usize, stair.y as usize) {
            return true;
        }
    }
    false
}

/// Search for a room of a specific type (search_special equivalent)
///
/// # Arguments
/// * `rooms` - Array of rooms to search
/// * `room_type` - Type to search for, or None for any non-ordinary room
/// * `any_shop` - If true, match any shop type
///
/// # Returns
/// Index of the first matching room, or None if not found
pub fn search_special(
    rooms: &[Room],
    room_type: Option<RoomType>,
    any_shop: bool,
) -> Option<usize> {
    for (idx, room) in rooms.iter().enumerate() {
        // Skip subrooms initially - could add subroom search if needed
        if room.parent.is_some() {
            continue;
        }

        match room_type {
            None => {
                // ANY_TYPE: any non-ordinary room
                if room.room_type != RoomType::Ordinary {
                    return Some(idx);
                }
            }
            Some(wanted) => {
                if any_shop && room.room_type.is_shop() {
                    return Some(idx);
                }
                if room.room_type == wanted {
                    return Some(idx);
                }
            }
        }
    }
    None
}

/// Find the room containing a point (pos_to_room equivalent)
///
/// # Arguments
/// * `rooms` - Array of rooms to search
/// * `x`, `y` - Coordinates to check
///
/// # Returns
/// Index of the room containing the point, or None if not in any room
pub fn pos_to_room(rooms: &[Room], x: usize, y: usize) -> Option<usize> {
    for (idx, room) in rooms.iter().enumerate() {
        if inside_room(room, x, y) {
            return Some(idx);
        }
    }
    None
}

/// Get list of room numbers at a coordinate (in_rooms equivalent)
///
/// For cells that are SHARED between rooms (like doorways), this returns
/// all room numbers that border the cell.
///
/// # Arguments
/// * `level` - Level data
/// * `rooms` - Array of rooms
/// * `x`, `y` - Coordinates to check
/// * `type_wanted` - Optional room type filter
///
/// # Returns
/// Vector of room indices that contain or border this cell
pub fn in_rooms(
    level: &super::Level,
    rooms: &[Room],
    x: usize,
    y: usize,
    type_wanted: Option<RoomType>,
) -> Vec<usize> {
    use super::generation::{NO_ROOM, ROOMOFFSET, SHARED};
    use crate::{COLNO, ROWNO};

    let mut result = Vec::new();

    if x >= COLNO || y >= ROWNO {
        return result;
    }

    let roomno = level.cells[x][y].room_number;

    match roomno {
        n if n == NO_ROOM => {
            // Not in any room
        }
        n if n == SHARED => {
            // SHARED cell - check neighboring cells
            let min_x = if x > 0 { x - 1 } else { x };
            let max_x = if x + 1 < COLNO { x + 1 } else { x };
            let min_y = if y > 0 { y - 1 } else { y };
            let max_y = if y + 1 < ROWNO { y + 1 } else { y };

            for check_x in min_x..=max_x {
                for check_y in min_y..=max_y {
                    let neighbor_roomno = level.cells[check_x][check_y].room_number;
                    if neighbor_roomno >= ROOMOFFSET && neighbor_roomno != SHARED {
                        let room_idx = (neighbor_roomno - ROOMOFFSET) as usize;
                        if room_idx < rooms.len() {
                            let room = &rooms[room_idx];
                            let type_matches = match type_wanted {
                                None => true,
                                Some(wanted) if wanted.is_shop() => room.room_type.is_shop(),
                                Some(wanted) => room.room_type == wanted,
                            };

                            if type_matches && !result.contains(&room_idx) {
                                result.push(room_idx);
                            }
                        }
                    }
                }
            }
        }
        n if n >= ROOMOFFSET => {
            // Regular room number
            let room_idx = (n - ROOMOFFSET) as usize;
            if room_idx < rooms.len() {
                let room = &rooms[room_idx];
                let type_matches = match type_wanted {
                    None => true,
                    Some(wanted) if wanted.is_shop() => room.room_type.is_shop(),
                    Some(wanted) => room.room_type == wanted,
                };

                if type_matches {
                    result.push(room_idx);
                }
            }
        }
        _ => {}
    }

    result
}

/// Get a free location within a room (get_free_room_loc equivalent)
///
/// Finds a random location in the room that doesn't have a monster or object.
pub fn get_free_room_loc(
    room: &Room,
    room_index: usize,
    all_rooms: &[Room],
    level: &super::Level,
    rng: &mut GameRng,
) -> Option<(usize, usize)> {
    for _ in 0..100 {
        if let Some((x, y)) = somexy(room, room_index, all_rooms, level, rng) {
            // Check if there's no monster here
            let has_monster = level
                .monsters
                .iter()
                .any(|m| m.x as usize == x && m.y as usize == y);
            // Check if there's no blocking object
            let has_blocking_obj = level.objects.iter().any(|o| {
                o.x == x as i8 && o.y == y as i8
                // Could add size/blocking check here
            });

            if !has_monster && !has_blocking_obj {
                return Some((x, y));
            }
        }
    }
    None
}

/// Get random location in a room (get_room_loc equivalent)
///
/// Similar to somexy but without avoiding objects/monsters.
pub fn get_room_loc(
    room: &Room,
    room_index: usize,
    all_rooms: &[Room],
    level: &super::Level,
    rng: &mut GameRng,
) -> Option<(usize, usize)> {
    somexy(room, room_index, all_rooms, level, rng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_type_values() {
        assert_eq!(RoomType::Ordinary as u8, 0);
        assert_eq!(RoomType::Court as u8, 2);
        assert_eq!(RoomType::GeneralShop as u8, 14);
        assert_eq!(RoomType::CandleShop as u8, 25);
    }

    #[test]
    fn test_is_shop() {
        assert!(!RoomType::Ordinary.is_shop());
        assert!(!RoomType::Court.is_shop());
        assert!(RoomType::GeneralShop.is_shop());
        assert!(RoomType::ArmorShop.is_shop());
        assert!(RoomType::CandleShop.is_shop());
    }

    #[test]
    fn test_is_special() {
        assert!(!RoomType::Ordinary.is_special());
        assert!(RoomType::Court.is_special());
        assert!(RoomType::Morgue.is_special());
        assert!(!RoomType::GeneralShop.is_special());
    }

    #[test]
    fn test_min_depth() {
        assert_eq!(RoomType::Ordinary.min_depth(), Some(1));
        assert_eq!(RoomType::Court.min_depth(), Some(4));
        assert_eq!(RoomType::Swamp.min_depth(), Some(15));
        assert_eq!(RoomType::GeneralShop.min_depth(), Some(2));
        assert_eq!(RoomType::Delphi.min_depth(), None); // Special level only
    }

    #[test]
    fn test_room_overlap() {
        let room1 = Room::new(5, 5, 5, 5);
        let room2 = Room::new(8, 8, 5, 5);
        let room3 = Room::new(15, 15, 5, 5);

        assert!(room1.overlaps(&room2, 0));
        assert!(!room1.overlaps(&room3, 0));
        assert!(room1.overlaps(&room3, 10));
    }

    #[test]
    fn test_room_center() {
        let room = Room::new(10, 10, 5, 5);
        assert_eq!(room.center(), (12, 12));
    }

    #[test]
    fn test_room_area() {
        let room = Room::new(0, 0, 5, 4);
        assert_eq!(room.area(), 20);
    }

    #[test]
    fn test_room_bounds() {
        let room = Room::new(10, 20, 5, 4);
        assert_eq!(room.bounds(), (10, 20, 14, 23));
        assert_eq!(room.wall_bounds(), (9, 19, 15, 24));
    }

    #[test]
    fn test_all_room_types_count() {
        assert_eq!(RoomType::ALL.len(), 25);
        assert_eq!(RoomType::SPECIAL_ROOMS.len(), 12);
        assert_eq!(RoomType::SHOP_TYPES.len(), 12);
    }
}
