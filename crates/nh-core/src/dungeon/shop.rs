//! Shop system (shknam.c, shk.c)
//!
//! Implements shop room creation with shopkeepers and inventory.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::monster::{Monster, MonsterId, MonsterState};
use crate::object::{Object, ObjectClass, ObjectId};
use crate::rng::GameRng;

use super::room::{Room, RoomType};
use super::{CellType, Level};

/// Shop type weights from C's shtypes[] array
/// Total weight is 100
const SHOP_WEIGHTS: [(RoomType, u32); 9] = [
    (RoomType::GeneralShop, 44),
    (RoomType::FoodShop, 16),
    (RoomType::WeaponShop, 14),
    (RoomType::ArmorShop, 10),
    (RoomType::ToolShop, 8),
    (RoomType::BookShop, 4),
    (RoomType::RingShop, 2),
    (RoomType::WandShop, 1),
    (RoomType::CandleShop, 1),
];

/// Select a shop type based on room size
/// Big rooms (>20 cells) become general shops
pub fn select_shop_type(rng: &mut GameRng, room_size: usize) -> RoomType {
    // Big rooms can't be specialized shops
    if room_size > 20 {
        return RoomType::GeneralShop;
    }

    // Weighted random selection
    let roll = rng.rn2(100);
    let mut cumulative = 0;

    for (shop_type, weight) in SHOP_WEIGHTS {
        cumulative += weight;
        if roll < cumulative {
            return shop_type;
        }
    }

    // Fallback (shouldn't happen)
    RoomType::GeneralShop
}

/// Get the object classes that can appear in a shop type
pub fn shop_object_classes(shop_type: RoomType) -> Vec<ObjectClass> {
    match shop_type {
        RoomType::ArmorShop => vec![ObjectClass::Armor],
        RoomType::WeaponShop => vec![ObjectClass::Weapon],
        RoomType::ScrollShop => vec![ObjectClass::Scroll],
        RoomType::PotionShop => vec![ObjectClass::Potion],
        RoomType::RingShop => vec![ObjectClass::Ring],
        RoomType::WandShop => vec![ObjectClass::Wand],
        RoomType::FoodShop => vec![ObjectClass::Food],
        RoomType::BookShop => vec![ObjectClass::Spellbook],
        RoomType::ToolShop => vec![ObjectClass::Tool],
        RoomType::CandleShop => vec![ObjectClass::Tool], // Candles are tools
        RoomType::HealthFoodShop => vec![ObjectClass::Food],
        RoomType::GeneralShop => vec![
            ObjectClass::Weapon,
            ObjectClass::Armor,
            ObjectClass::Ring,
            ObjectClass::Amulet,
            ObjectClass::Tool,
            ObjectClass::Food,
            ObjectClass::Potion,
            ObjectClass::Scroll,
            ObjectClass::Wand,
        ],
        _ => vec![],
    }
}

/// Find the door position for a room
fn find_room_door(level: &Level, room: &Room) -> Option<(usize, usize)> {
    // Check each wall position for doors
    for x in room.x.saturating_sub(1)..=room.x + room.width {
        for y in room.y.saturating_sub(1)..=room.y + room.height {
            // Skip interior cells
            if x >= room.x && x < room.x + room.width && y >= room.y && y < room.y + room.height {
                continue;
            }

            let cell_type = level.cells[x][y].typ;
            if cell_type == CellType::Door || cell_type == CellType::SecretDoor {
                return Some((x, y));
            }
        }
    }
    None
}

/// Find a position near the door for the shopkeeper
fn position_near_door(room: &Room, door_pos: (usize, usize)) -> (usize, usize) {
    let (dx, dy) = door_pos;

    // Shopkeeper stands inside the room, adjacent to the door
    if dx < room.x {
        // Door is on left wall
        (room.x, dy.max(room.y).min(room.y + room.height - 1))
    } else if dx >= room.x + room.width {
        // Door is on right wall
        (
            room.x + room.width - 1,
            dy.max(room.y).min(room.y + room.height - 1),
        )
    } else if dy < room.y {
        // Door is on top wall
        (dx.max(room.x).min(room.x + room.width - 1), room.y)
    } else {
        // Door is on bottom wall
        (
            dx.max(room.x).min(room.x + room.width - 1),
            room.y + room.height - 1,
        )
    }
}

/// Create a shopkeeper monster
fn create_shopkeeper(x: usize, y: usize, shop_type: RoomType, rng: &mut GameRng) -> Monster {
    let mut keeper = Monster::new(MonsterId(0), 10, x as i8, y as i8);
    keeper.state = MonsterState::active();
    keeper.state.peaceful = true;
    keeper.hp = 30 + rng.rnd(20) as i32;
    keeper.hp_max = keeper.hp;

    // Generate a shopkeeper name
    keeper.name = generate_shopkeeper_name(shop_type, rng);

    keeper
}

/// Generate a shopkeeper name
/// In real NetHack this uses shknam.c's names, here we use simplified names
fn generate_shopkeeper_name(shop_type: RoomType, rng: &mut GameRng) -> String {
    let base_names = match shop_type {
        RoomType::ArmorShop => ["Narfi", "Hjuki", "Skuld", "Delling", "Dagr"],
        RoomType::WeaponShop => ["Sigurd", "Brynhild", "Gunnar", "Hogni", "Atli"],
        RoomType::FoodShop => ["Njord", "Frey", "Freya", "Skirnir", "Byggvir"],
        RoomType::ScrollShop => ["Odin", "Mimir", "Saga", "Snotra", "Vör"],
        RoomType::PotionShop => ["Idunn", "Bragi", "Gefjon", "Hlin", "Sjöfn"],
        RoomType::RingShop => [
            "Draupnir",
            "Andvaranaut",
            "Brisingamen",
            "Gullinbursti",
            "Megingjörð",
        ],
        RoomType::WandShop => ["Gandalf", "Merlin", "Circe", "Morgana", "Prospero"],
        RoomType::BookShop => [
            "Snorri",
            "Völuspá",
            "Hávamál",
            "Gylfaginning",
            "Skáldskaparmál",
        ],
        RoomType::ToolShop => ["Völund", "Dvalin", "Alfrik", "Berling", "Grer"],
        RoomType::CandleShop => ["Nótt", "Hati", "Skoll", "Mani", "Sol"],
        _ => [
            "Izchak",
            "Asidonhopo",
            "Adjama",
            "Akhastatoth",
            "Annodharma",
        ],
    };

    let idx = rng.rn2(base_names.len() as u32) as usize;
    base_names[idx].to_string()
}

/// Create a shop item
fn create_shop_item(classes: &[ObjectClass], rng: &mut GameRng) -> Object {
    let class = if classes.is_empty() {
        ObjectClass::Coin
    } else {
        classes[rng.rn2(classes.len() as u32) as usize]
    };

    let mut obj = Object::new(ObjectId(0), 0, class);
    obj.unpaid = true; // Mark as shop inventory

    // Set quantity for stackable items
    obj.quantity = match class {
        ObjectClass::Food | ObjectClass::Potion | ObjectClass::Scroll => (rng.rnd(3) + 1) as i32,
        ObjectClass::Coin => rng.rn2(100) as i32 + 10,
        _ => 1,
    };

    // Set shop price (simplified)
    obj.shop_price = match class {
        ObjectClass::Weapon => 10 + rng.rnd(50) as i32,
        ObjectClass::Armor => 20 + rng.rnd(80) as i32,
        ObjectClass::Ring => 100 + rng.rnd(200) as i32,
        ObjectClass::Amulet => 150 + rng.rnd(250) as i32,
        ObjectClass::Wand => 50 + rng.rnd(100) as i32,
        ObjectClass::Scroll => 20 + rng.rnd(30) as i32,
        ObjectClass::Potion => 20 + rng.rnd(50) as i32,
        ObjectClass::Spellbook => 100 + rng.rnd(400) as i32,
        ObjectClass::Food => 5 + rng.rnd(15) as i32,
        ObjectClass::Tool => 10 + rng.rnd(40) as i32,
        _ => 10,
    };

    obj
}

/// Stock a shop with items
fn stock_shop(level: &mut Level, room: &Room, shop_type: RoomType, rng: &mut GameRng) {
    let classes = shop_object_classes(shop_type);

    if classes.is_empty() {
        return;
    }

    // Fill floor with items (not every cell, ~50% coverage)
    for x in room.x..room.x + room.width {
        for y in room.y..room.y + room.height {
            let cell_type = level.cells[x][y].typ;

            // Skip walls and non-room cells
            if cell_type != CellType::Room {
                continue;
            }

            // Skip if monster is there
            if level.monster_at(x as i8, y as i8).is_some() {
                continue;
            }

            // 50% chance to place item
            if rng.one_in(2) {
                let obj = create_shop_item(&classes, rng);
                level.add_object(obj, x as i8, y as i8);
            }
        }
    }
}

/// Populate a shop with shopkeeper and inventory
pub fn populate_shop(level: &mut Level, room: &Room, rng: &mut GameRng) {
    let shop_type = room.room_type;

    // Find door position for shopkeeper placement
    let door_pos = find_room_door(level, room);

    // Create shopkeeper near door
    if let Some(door) = door_pos {
        let (kx, ky) = position_near_door(room, door);

        // Only place shopkeeper if position is valid and unoccupied
        if kx >= room.x
            && kx < room.x + room.width
            && ky >= room.y
            && ky < room.y + room.height
            && level.monster_at(kx as i8, ky as i8).is_none()
        {
            let keeper = create_shopkeeper(kx, ky, shop_type, rng);
            level.add_monster(keeper);
        }
    } else {
        // No door found, place shopkeeper in center
        let (cx, cy) = room.center();
        if level.monster_at(cx as i8, cy as i8).is_none() {
            let keeper = create_shopkeeper(cx, cy, shop_type, rng);
            level.add_monster(keeper);
        }
    }

    // Stock shop with items
    stock_shop(level, room, shop_type, rng);
}

/// Check if a room type is a shop
pub fn is_shop_room(room_type: RoomType) -> bool {
    room_type.is_shop()
}

#[cfg(test)]
mod tests {
    use super::super::DLevel;
    use super::*;

    #[test]
    fn test_select_shop_type() {
        let mut rng = GameRng::new(42);
        let mut general_count = 0;
        let mut other_count = 0;

        // Test shop selection for small rooms
        for _ in 0..1000 {
            let shop_type = select_shop_type(&mut rng, 15);
            if shop_type == RoomType::GeneralShop {
                general_count += 1;
            } else {
                other_count += 1;
            }
        }

        println!(
            "Small room - General: {}, Other: {}",
            general_count, other_count
        );

        // General shop should be ~44%
        assert!(
            general_count > 350 && general_count < 550,
            "General shop should be ~44%, got {}",
            general_count
        );

        // Big rooms always become general shops
        for _ in 0..100 {
            let shop_type = select_shop_type(&mut rng, 25);
            assert_eq!(
                shop_type,
                RoomType::GeneralShop,
                "Big rooms should be general shops"
            );
        }
    }

    #[test]
    fn test_shop_object_classes() {
        let armor_classes = shop_object_classes(RoomType::ArmorShop);
        assert!(armor_classes.contains(&ObjectClass::Armor));
        assert!(!armor_classes.contains(&ObjectClass::Weapon));

        let general_classes = shop_object_classes(RoomType::GeneralShop);
        assert!(
            general_classes.len() > 5,
            "General shop should have many classes"
        );
        assert!(general_classes.contains(&ObjectClass::Weapon));
        assert!(general_classes.contains(&ObjectClass::Armor));
    }

    #[test]
    fn test_shopkeeper_name() {
        let mut rng = GameRng::new(42);

        let name1 = generate_shopkeeper_name(RoomType::ArmorShop, &mut rng);
        assert!(!name1.is_empty());

        let name2 = generate_shopkeeper_name(RoomType::WandShop, &mut rng);
        assert!(!name2.is_empty());
    }

    #[test]
    fn test_populate_shop() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::main_dungeon_start());

        // Create a shop room
        let room = Room::with_type(10, 5, 6, 4, RoomType::GeneralShop);

        // Carve out the room first
        for x in room.x..room.x + room.width {
            for y in room.y..room.y + room.height {
                level.cells[x][y].typ = CellType::Room;
                level.cells[x][y].lit = true;
            }
        }

        // Add a door
        level.cells[room.x][room.y - 1] = super::super::Cell {
            typ: CellType::Door,
            ..Default::default()
        };

        // Populate the shop
        populate_shop(&mut level, &room, &mut rng);

        // Should have a shopkeeper
        assert!(!level.monsters.is_empty(), "Shop should have a shopkeeper");

        // Shopkeeper should be peaceful
        let keeper = &level.monsters[0];
        assert!(keeper.state.peaceful, "Shopkeeper should be peaceful");

        // Should have items
        assert!(!level.objects.is_empty(), "Shop should have items");

        // Items should be unpaid
        assert!(
            level.objects.iter().all(|obj| obj.unpaid),
            "Shop items should be marked as unpaid"
        );

        println!(
            "Shop has {} monsters and {} items",
            level.monsters.len(),
            level.objects.len()
        );
    }

    #[test]
    fn test_position_near_door() {
        let room = Room::new(10, 5, 6, 4);

        // Door on left wall
        let pos = position_near_door(&room, (9, 7));
        assert_eq!(pos.0, 10, "Should be at room left edge");
        assert!(pos.1 >= room.y && pos.1 < room.y + room.height);

        // Door on top wall
        let pos = position_near_door(&room, (12, 4));
        assert_eq!(pos.1, 5, "Should be at room top edge");
        assert!(pos.0 >= room.x && pos.0 < room.x + room.width);
    }
}
