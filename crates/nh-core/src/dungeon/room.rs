//! Room types and structures (mkroom.h)
//!
//! Defines room types matching NetHack C:
//! - 25 room types from OROOM (0) to CANDLESHOP (25)
//! - Room struct with type, dimensions, and properties

use crate::rng::GameRng;

/// Room types matching C mkroom.h enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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
#[derive(Debug, Clone)]
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
        (self.x, self.y, self.x + self.width - 1, self.y + self.height - 1)
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
