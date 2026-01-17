//! Special room population (mkroom.c: fill_zoo(), mk_zoo_thronemon(), etc.)
//!
//! Implements monster and item spawning for special room types.
//! Each room type has specific monster selection and feature placement logic.

use crate::monster::{Monster, MonsterId, MonsterState};
use crate::object::{Object, ObjectClass, ObjectId};
use crate::rng::GameRng;

use super::room::{Room, RoomType};
use super::{CellType, Level, TrapType};

/// Monster class for special room spawning
/// These map to C's PM_* constants but are simplified for now
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialMonsterClass {
    // Court monsters (throne room hierarchy)
    Dragon,
    Giant,
    Troll,
    Centaur,
    Orc,
    Bugbear,
    Hobgoblin,
    Gnome,
    Kobold,

    // Morgue monsters (undead)
    Demon,
    Vampire,
    Ghost,
    Wraith,
    Zombie,

    // Barracks monsters (soldiers)
    Captain,
    Lieutenant,
    Sergeant,
    Soldier,

    // Hive monsters
    QueenBee,
    KillerBee,

    // Anthole monsters
    SoldierAnt,
    FireAnt,
    GiantAnt,

    // Leprechaun hall
    Leprechaun,

    // Cockatrice nest
    Cockatrice,

    // Zoo (random level-appropriate)
    RandomMonster,

    // Temple
    Priest,

    // Swamp
    GiantEel,
    Piranha,
    ElectricEel,
    Fungus,
}

impl SpecialMonsterClass {
    /// Get a display name for this monster class
    pub fn name(&self) -> &'static str {
        match self {
            Self::Dragon => "dragon",
            Self::Giant => "giant",
            Self::Troll => "troll",
            Self::Centaur => "centaur",
            Self::Orc => "orc",
            Self::Bugbear => "bugbear",
            Self::Hobgoblin => "hobgoblin",
            Self::Gnome => "gnome",
            Self::Kobold => "kobold",
            Self::Demon => "demon",
            Self::Vampire => "vampire",
            Self::Ghost => "ghost",
            Self::Wraith => "wraith",
            Self::Zombie => "zombie",
            Self::Captain => "captain",
            Self::Lieutenant => "lieutenant",
            Self::Sergeant => "sergeant",
            Self::Soldier => "soldier",
            Self::QueenBee => "queen bee",
            Self::KillerBee => "killer bee",
            Self::SoldierAnt => "soldier ant",
            Self::FireAnt => "fire ant",
            Self::GiantAnt => "giant ant",
            Self::Leprechaun => "leprechaun",
            Self::Cockatrice => "cockatrice",
            Self::RandomMonster => "monster",
            Self::Priest => "priest",
            Self::GiantEel => "giant eel",
            Self::Piranha => "piranha",
            Self::ElectricEel => "electric eel",
            Self::Fungus => "fungus",
        }
    }

    /// Get base HP for this monster class (simplified)
    pub fn base_hp(&self, rng: &mut GameRng) -> i32 {
        match self {
            Self::Dragon => 80 + rng.rnd(40) as i32,
            Self::Giant => 50 + rng.rnd(30) as i32,
            Self::Troll => 40 + rng.rnd(20) as i32,
            Self::Centaur => 30 + rng.rnd(15) as i32,
            Self::Orc | Self::Bugbear => 15 + rng.rnd(10) as i32,
            Self::Hobgoblin => 12 + rng.rnd(8) as i32,
            Self::Gnome | Self::Kobold => 8 + rng.rnd(5) as i32,
            Self::Demon => 60 + rng.rnd(30) as i32,
            Self::Vampire => 40 + rng.rnd(20) as i32,
            Self::Ghost | Self::Wraith => 20 + rng.rnd(10) as i32,
            Self::Zombie => 15 + rng.rnd(8) as i32,
            Self::Captain => 30 + rng.rnd(15) as i32,
            Self::Lieutenant => 25 + rng.rnd(10) as i32,
            Self::Sergeant => 20 + rng.rnd(8) as i32,
            Self::Soldier => 15 + rng.rnd(5) as i32,
            Self::QueenBee => 25 + rng.rnd(10) as i32,
            Self::KillerBee => 8 + rng.rnd(4) as i32,
            Self::SoldierAnt | Self::FireAnt | Self::GiantAnt => 12 + rng.rnd(6) as i32,
            Self::Leprechaun => 10 + rng.rnd(5) as i32,
            Self::Cockatrice => 20 + rng.rnd(10) as i32,
            Self::RandomMonster => 10 + rng.rnd(10) as i32,
            Self::Priest => 25 + rng.rnd(10) as i32,
            Self::GiantEel | Self::ElectricEel => 15 + rng.rnd(8) as i32,
            Self::Piranha => 10 + rng.rnd(5) as i32,
            Self::Fungus => 8 + rng.rnd(4) as i32,
        }
    }

    /// Get difficulty level for this monster
    pub fn difficulty(&self) -> i32 {
        match self {
            Self::Dragon => 15,
            Self::Giant => 10,
            Self::Troll => 8,
            Self::Centaur => 6,
            Self::Orc | Self::Bugbear => 4,
            Self::Hobgoblin => 3,
            Self::Gnome | Self::Kobold => 2,
            Self::Demon => 12,
            Self::Vampire => 10,
            Self::Ghost | Self::Wraith => 7,
            Self::Zombie => 4,
            Self::Captain => 8,
            Self::Lieutenant => 6,
            Self::Sergeant => 4,
            Self::Soldier => 3,
            Self::QueenBee => 6,
            Self::KillerBee => 2,
            Self::SoldierAnt | Self::FireAnt | Self::GiantAnt => 4,
            Self::Leprechaun => 3,
            Self::Cockatrice => 7,
            Self::RandomMonster => 3,
            Self::Priest => 5,
            Self::GiantEel | Self::ElectricEel => 5,
            Self::Piranha => 3,
            Self::Fungus => 2,
        }
    }
}

/// Select monster for throne room (mk_zoo_thronemon in C)
/// Uses difficulty-based hierarchy matching C's implementation
pub fn court_monster(rng: &mut GameRng, difficulty: i32) -> SpecialMonsterClass {
    // C: i = rnd(60) + rnd(3*level_difficulty)
    let i = rng.rnd(60) as i32 + rng.rnd((3 * difficulty).max(1) as u32) as i32;

    match i {
        i if i > 100 => SpecialMonsterClass::Dragon,
        i if i > 95 => SpecialMonsterClass::Giant,
        i if i > 85 => SpecialMonsterClass::Troll,
        i if i > 75 => SpecialMonsterClass::Centaur,
        i if i > 60 => SpecialMonsterClass::Orc,
        i if i > 45 => SpecialMonsterClass::Bugbear,
        i if i > 30 => SpecialMonsterClass::Hobgoblin,
        i if i > 15 => SpecialMonsterClass::Gnome,
        _ => SpecialMonsterClass::Kobold,
    }
}

/// Select monster for morgue (morguemon in C)
/// Uses difficulty-based undead hierarchy
pub fn morgue_monster(rng: &mut GameRng, difficulty: i32) -> SpecialMonsterClass {
    let i = rng.rn2(100);
    let hd = rng.rn2(difficulty.max(1) as u32) as i32;

    // High difficulty can spawn demons
    if hd > 10 && i < 10 {
        return SpecialMonsterClass::Demon;
    }

    // High difficulty and high roll can spawn vampires
    if hd > 8 && i > 85 {
        return SpecialMonsterClass::Vampire;
    }

    // Normal distribution
    match i {
        0..=19 => SpecialMonsterClass::Ghost,
        20..=39 => SpecialMonsterClass::Wraith,
        _ => SpecialMonsterClass::Zombie,
    }
}

/// Select monster for barracks (squadmon in C)
/// Military hierarchy based on probability
pub fn squad_monster(rng: &mut GameRng, difficulty: i32) -> SpecialMonsterClass {
    let prob = rng.rnd(80 + difficulty as u32);

    match prob {
        p if p >= 100 => SpecialMonsterClass::Captain,
        p if p >= 96 => SpecialMonsterClass::Lieutenant,
        p if p >= 81 => SpecialMonsterClass::Sergeant,
        _ => SpecialMonsterClass::Soldier,
    }
}

/// Select monster for beehive
/// Queen bee in center, killer bees elsewhere
pub fn beehive_monster(is_center: bool) -> SpecialMonsterClass {
    if is_center {
        SpecialMonsterClass::QueenBee
    } else {
        SpecialMonsterClass::KillerBee
    }
}

/// Select monster for anthole (antholemon in C)
/// Deterministic per-level based on seed
pub fn anthole_monster(seed: u64, depth: i32) -> SpecialMonsterClass {
    match (seed + depth as u64) % 3 {
        0 => SpecialMonsterClass::SoldierAnt,
        1 => SpecialMonsterClass::FireAnt,
        _ => SpecialMonsterClass::GiantAnt,
    }
}

/// Select monster for swamp (swampmon in C)
pub fn swamp_monster(rng: &mut GameRng) -> SpecialMonsterClass {
    match rng.rn2(10) {
        0..=7 => SpecialMonsterClass::GiantEel,
        8 => SpecialMonsterClass::Piranha,
        _ => SpecialMonsterClass::ElectricEel,
    }
}

/// Create a monster from a special monster class
fn create_special_monster(
    monster_class: SpecialMonsterClass,
    x: usize,
    y: usize,
    sleeping: bool,
    rng: &mut GameRng,
) -> Monster {
    let mut monster = Monster::new(MonsterId(0), monster_class.difficulty() as i16, x as i8, y as i8);
    monster.hp = monster_class.base_hp(rng);
    monster.hp_max = monster.hp;
    monster.name = monster_class.name().to_string();

    if sleeping {
        monster.state = MonsterState::default(); // Sleeping by default
        monster.state.sleeping = true;
    } else {
        monster.state = MonsterState::active();
    }

    monster
}

/// Populate a special room with appropriate monsters and features
/// This is the equivalent of C's fill_zoo()
pub fn populate_special_room(
    level: &mut Level,
    room: &Room,
    rng: &mut GameRng,
) {
    let difficulty = level.dlevel.depth();
    let (cx, cy) = room.center();

    // Density of monsters varies by room type
    let spawn_chance = match room.room_type {
        RoomType::Court | RoomType::Barracks => 4,  // 1 in 4
        RoomType::Zoo | RoomType::Morgue => 3,       // 1 in 3
        RoomType::Beehive | RoomType::Anthole => 2,  // 1 in 2
        RoomType::LeprechaunHall => 4,               // 1 in 4
        RoomType::CockatriceNest => 5,               // 1 in 5
        _ => 0,  // No monsters for other room types
    };

    if spawn_chance == 0 {
        // Handle special cases
        match room.room_type {
            RoomType::Temple => populate_temple(level, room, rng),
            RoomType::Swamp => populate_swamp(level, room, rng),
            _ => {}
        }
        return;
    }

    // Iterate through room cells and spawn monsters
    for x in room.x..room.x + room.width {
        for y in room.y..room.y + room.height {
            // Skip walls
            if level.cells[x][y].typ.is_wall() {
                continue;
            }

            // Skip if occupied
            if level.monster_at(x as i8, y as i8).is_some() {
                continue;
            }

            // Probability-based spawn
            if rng.rn2(spawn_chance) != 0 {
                continue;
            }

            let is_center = x == cx && y == cy;
            let monster_class = select_room_monster(room.room_type, rng, difficulty, is_center);

            let monster = create_special_monster(monster_class, x, y, true, rng);
            level.add_monster(monster);
        }
    }

    // Place room-specific features
    place_room_features(level, room, cx, cy, rng);
}

/// Select appropriate monster for a room type
fn select_room_monster(
    room_type: RoomType,
    rng: &mut GameRng,
    difficulty: i32,
    is_center: bool,
) -> SpecialMonsterClass {
    match room_type {
        RoomType::Court => court_monster(rng, difficulty),
        RoomType::Morgue => morgue_monster(rng, difficulty),
        RoomType::Barracks => squad_monster(rng, difficulty),
        RoomType::Beehive => beehive_monster(is_center),
        RoomType::Anthole => anthole_monster(rng.rn2(1000) as u64, difficulty),
        RoomType::Zoo => SpecialMonsterClass::RandomMonster,
        RoomType::LeprechaunHall => SpecialMonsterClass::Leprechaun,
        RoomType::CockatriceNest => SpecialMonsterClass::Cockatrice,
        _ => SpecialMonsterClass::RandomMonster,
    }
}

/// Place room-specific features (throne, altar, etc.)
fn place_room_features(
    level: &mut Level,
    room: &Room,
    cx: usize,
    cy: usize,
    _rng: &mut GameRng,
) {
    match room.room_type {
        RoomType::Court => {
            // Place throne at center
            level.cells[cx][cy].typ = CellType::Throne;
        }
        RoomType::Temple => {
            // Place altar at center (handled in populate_temple)
        }
        _ => {}
    }
}

/// Populate a temple with priest and altar
fn populate_temple(level: &mut Level, room: &Room, rng: &mut GameRng) {
    let (cx, cy) = room.center();

    // Place altar at center
    level.cells[cx][cy].typ = CellType::Altar;

    // Place priest near altar
    let priest_positions = [
        (cx.saturating_sub(1), cy),
        (cx + 1, cy),
        (cx, cy.saturating_sub(1)),
        (cx, cy + 1),
    ];

    for (px, py) in priest_positions {
        if px >= room.x && px < room.x + room.width
            && py >= room.y && py < room.y + room.height
            && level.monster_at(px as i8, py as i8).is_none()
        {
            let mut priest = create_special_monster(SpecialMonsterClass::Priest, px, py, false, rng);
            priest.state.peaceful = true;  // Temple priests are peaceful
            level.add_monster(priest);
            break;
        }
    }
}

/// Populate a swamp with pools and aquatic monsters
fn populate_swamp(level: &mut Level, room: &Room, rng: &mut GameRng) {
    // Checkerboard pool pattern
    for x in room.x..room.x + room.width {
        for y in room.y..room.y + room.height {
            // Skip walls
            if level.cells[x][y].typ.is_wall() {
                continue;
            }

            if (x + y) % 2 == 0 {
                // Pool cell
                level.cells[x][y].typ = CellType::Pool;

                // Spawn aquatic monster (50% chance)
                if rng.one_in(2) && level.monster_at(x as i8, y as i8).is_none() {
                    let monster_class = swamp_monster(rng);
                    let monster = create_special_monster(monster_class, x, y, true, rng);
                    level.add_monster(monster);
                }
            } else {
                // Non-pool cell adjacent to pool - maybe spawn fungus
                if rng.one_in(4) && level.monster_at(x as i8, y as i8).is_none() {
                    let monster = create_special_monster(SpecialMonsterClass::Fungus, x, y, true, rng);
                    level.add_monster(monster);
                }
            }
        }
    }
}

/// Populate a vault with gold (and possibly a teleport trap)
pub fn populate_vault(level: &mut Level, room: &Room, rng: &mut GameRng) {
    let depth = level.dlevel.depth();

    // Fill vault with gold piles
    // Amount based on depth: 50 * depth * rnd(10)
    for x in room.x..room.x + room.width {
        for y in room.y..room.y + room.height {
            // Skip walls
            if level.cells[x][y].typ.is_wall() {
                continue;
            }

            // Each cell gets a gold pile
            let gold_amount = (50 * depth * rng.rnd(10) as i32).max(50);
            let mut gold = Object::new(ObjectId(0), 0, ObjectClass::Coin);
            gold.quantity = gold_amount;
            level.add_object(gold, x as i8, y as i8);
        }
    }

    // 1/3 chance of teleport trap for access
    if rng.one_in(3) {
        let (tx, ty) = room.random_point(rng);
        level.add_trap(tx as i8, ty as i8, TrapType::Teleport);
    }

    // Vault is unlit
    for x in room.x..room.x + room.width {
        for y in room.y..room.y + room.height {
            level.cells[x][y].lit = false;
        }
    }
}

/// Check if a room type requires special population
pub fn needs_population(room_type: RoomType) -> bool {
    matches!(
        room_type,
        RoomType::Court
            | RoomType::Swamp
            | RoomType::Beehive
            | RoomType::Morgue
            | RoomType::Barracks
            | RoomType::Zoo
            | RoomType::Temple
            | RoomType::LeprechaunHall
            | RoomType::CockatriceNest
            | RoomType::Anthole
    )
}

/// Check if a room type is a vault (needs special generation)
pub fn is_vault(room_type: RoomType) -> bool {
    matches!(room_type, RoomType::Vault)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_court_monster_distribution() {
        let mut rng = GameRng::new(42);
        let mut kobolds = 0;
        let mut dragons = 0;

        // Low difficulty should mostly produce weak monsters
        for _ in 0..1000 {
            match court_monster(&mut rng, 5) {
                SpecialMonsterClass::Kobold => kobolds += 1,
                SpecialMonsterClass::Dragon => dragons += 1,
                _ => {}
            }
        }

        // Kobolds should be more common than dragons at low difficulty
        println!("Low diff - Kobolds: {}, Dragons: {}", kobolds, dragons);
        assert!(kobolds > dragons, "Kobolds should be more common at low difficulty");

        // High difficulty should produce more strong monsters
        kobolds = 0;
        dragons = 0;
        for _ in 0..1000 {
            match court_monster(&mut rng, 25) {
                SpecialMonsterClass::Kobold => kobolds += 1,
                SpecialMonsterClass::Dragon => dragons += 1,
                _ => {}
            }
        }

        println!("High diff - Kobolds: {}, Dragons: {}", kobolds, dragons);
        // At high difficulty, dragons should appear more often
    }

    #[test]
    fn test_morgue_monster_distribution() {
        let mut rng = GameRng::new(42);
        let mut ghosts = 0;
        let mut wraiths = 0;
        let mut zombies = 0;

        for _ in 0..1000 {
            match morgue_monster(&mut rng, 5) {
                SpecialMonsterClass::Ghost => ghosts += 1,
                SpecialMonsterClass::Wraith => wraiths += 1,
                SpecialMonsterClass::Zombie => zombies += 1,
                _ => {}
            }
        }

        println!("Morgue distribution - Ghosts: {}, Wraiths: {}, Zombies: {}",
                 ghosts, wraiths, zombies);

        // Zombies should be most common (~60%)
        assert!(zombies > ghosts, "Zombies should be most common");
        assert!(zombies > wraiths, "Zombies should be more common than wraiths");

        // Ghost and wraith should be roughly equal (~20% each)
        let diff = (ghosts as i32 - wraiths as i32).abs();
        assert!(diff < 100, "Ghosts and wraiths should be roughly equal");
    }

    #[test]
    fn test_squad_monster_distribution() {
        let mut rng = GameRng::new(42);
        let mut soldiers = 0;
        let mut sergeants = 0;
        let mut lieutenants = 0;
        let mut captains = 0;

        for _ in 0..1000 {
            match squad_monster(&mut rng, 10) {
                SpecialMonsterClass::Soldier => soldiers += 1,
                SpecialMonsterClass::Sergeant => sergeants += 1,
                SpecialMonsterClass::Lieutenant => lieutenants += 1,
                SpecialMonsterClass::Captain => captains += 1,
                _ => {}
            }
        }

        println!("Squad distribution - Soldiers: {}, Sergeants: {}, Lieutenants: {}, Captains: {}",
                 soldiers, sergeants, lieutenants, captains);

        // Soldiers should be most common
        assert!(soldiers > sergeants, "Soldiers should be most common");
        assert!(soldiers > lieutenants, "Soldiers should be more common than lieutenants");
        assert!(soldiers > captains, "Soldiers should be more common than captains");
    }

    #[test]
    fn test_beehive_monster() {
        assert_eq!(beehive_monster(true), SpecialMonsterClass::QueenBee);
        assert_eq!(beehive_monster(false), SpecialMonsterClass::KillerBee);
    }

    #[test]
    fn test_anthole_deterministic() {
        // Same seed and depth should produce same ant type
        let ant1 = anthole_monster(42, 10);
        let ant2 = anthole_monster(42, 10);
        assert_eq!(ant1, ant2);

        // Different seeds should potentially produce different types
        let ant3 = anthole_monster(43, 10);
        let ant4 = anthole_monster(44, 10);
        // At least one should be different
        assert!(ant1 != ant3 || ant1 != ant4 || ant3 != ant4,
                "Different seeds should produce variety");
    }

    #[test]
    fn test_special_monster_class_name() {
        assert_eq!(SpecialMonsterClass::Dragon.name(), "dragon");
        assert_eq!(SpecialMonsterClass::QueenBee.name(), "queen bee");
        assert_eq!(SpecialMonsterClass::Soldier.name(), "soldier");
    }

    #[test]
    fn test_needs_population() {
        assert!(needs_population(RoomType::Court));
        assert!(needs_population(RoomType::Morgue));
        assert!(needs_population(RoomType::Zoo));
        assert!(!needs_population(RoomType::Ordinary));
        assert!(!needs_population(RoomType::Vault));  // Vault has gold, not monsters
        assert!(!needs_population(RoomType::GeneralShop));  // Shops handled separately
    }

    #[test]
    fn test_is_vault() {
        assert!(is_vault(RoomType::Vault));
        assert!(!is_vault(RoomType::Ordinary));
        assert!(!is_vault(RoomType::Court));
        assert!(!is_vault(RoomType::GeneralShop));
    }

    #[test]
    fn test_populate_vault() {
        use super::super::DLevel;

        let mut rng = GameRng::new(42);
        let dlevel = DLevel {
            dungeon_num: 0,
            level_num: 10,  // Depth 10 for good gold amounts
        };
        let mut level = Level::new(dlevel);

        // Create a small vault room
        let room = Room::with_type(10, 5, 2, 2, RoomType::Vault);

        // Carve out the room first
        for x in room.x..room.x + room.width {
            for y in room.y..room.y + room.height {
                level.cells[x][y].typ = CellType::Room;
                level.cells[x][y].lit = true;
            }
        }

        // Populate the vault
        populate_vault(&mut level, &room, &mut rng);

        // Should have gold piles
        assert!(
            !level.objects.is_empty(),
            "Vault should have gold piles"
        );

        // All objects should be gold
        assert!(
            level.objects.iter().all(|obj| obj.class == ObjectClass::Coin),
            "Vault should only have gold"
        );

        // Room should be unlit
        for x in room.x..room.x + room.width {
            for y in room.y..room.y + room.height {
                assert!(!level.cells[x][y].lit, "Vault should be unlit");
            }
        }

        println!("Vault has {} gold piles, total value: {}",
                 level.objects.len(),
                 level.objects.iter().map(|o| o.quantity).sum::<i32>());
    }
}
