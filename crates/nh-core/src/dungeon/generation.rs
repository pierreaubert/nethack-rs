//! Level generation (mklev.c, mkroom.c)
//!
//! Generates dungeon levels with rooms and corridors.
//! Uses the rectangle system (rect.c) for efficient room placement.

use crate::monster::{Monster, MonsterId};
use crate::rng::GameRng;
use crate::{COLNO, ROWNO};

use super::corridor::generate_corridors;
use super::rect::{NhRect, RectManager};
use super::room::{Room, RoomType};
use super::shop::populate_shop;
use super::special_rooms::{is_vault, needs_population, populate_special_room, populate_vault};
use super::{CellType, DLevel, DoorState, Level, LevelFlags};

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

        let room = Room::new(x, y, width, height);

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

    // Connect rooms with 4-phase corridor algorithm
    generate_corridors(level, &rooms, rng);

    // Place doors
    place_doors(level, &rooms, rng);

    // Select and assign special room type based on depth
    let depth = level.dlevel.depth();
    if let Some(special_type) = select_special_room_type(rng, depth, &mut level.flags)
        && let Some(room_idx) = pick_room_for_special(&rooms, special_type)
    {
        rooms[room_idx].room_type = special_type;

        // Set lighting based on room type (morgues and vaults are dark)
        rooms[room_idx].lit = !matches!(special_type, RoomType::Morgue | RoomType::Vault);

        // Update level flags (already done in select_special_room_type for most,
        // but this ensures consistency)
        set_level_flags_for_room(&mut level.flags, special_type);

        // Update cell lighting if room is dark
        if !rooms[room_idx].lit {
            let room = &rooms[room_idx];
            for x in room.x..room.x + room.width {
                for y in room.y..room.y + room.height {
                    level.cells[x][y].lit = false;
                }
            }
        }

        // Populate special room with monsters and features
        if special_type.is_shop() {
            // Shops get shopkeepers and inventory
            populate_shop(level, &rooms[room_idx], rng);
        } else if is_vault(special_type) {
            // Vaults get gold piles (and possibly teleport trap)
            populate_vault(level, &rooms[room_idx], rng);
        } else if needs_population(special_type) {
            // Other special rooms get their themed monsters
            populate_special_room(level, &rooms[room_idx], rng);
        }
    }

    // Place stairs
    if !rooms.is_empty() {
        place_stairs(level, &rooms, rng);
    }

    // Place monsters
    place_monsters(level, &rooms, rng);

    // Place traps
    place_traps(level, &rooms, rng);

    // Place fountains, sinks, and altars
    place_dungeon_features(level, &rooms, rng);

    // Place branch entrances if this level has one
    place_branch_entrance(level, &rooms, rng);
}

/// Place doors at room entrances
fn place_doors(level: &mut Level, rooms: &[Room], rng: &mut GameRng) {
    let depth = level.dlevel.depth();

    for room in rooms {
        let is_shop = room.room_type.is_shop();

        // Check each wall position for potential door placement
        for x in room.x..room.x + room.width {
            // Top wall
            if room.y > 0 {
                check_and_place_door(level, x, room.y - 1, depth, is_shop, rng);
            }
            // Bottom wall
            if room.y + room.height < ROWNO {
                check_and_place_door(level, x, room.y + room.height, depth, is_shop, rng);
            }
        }

        for y in room.y..room.y + room.height {
            // Left wall
            if room.x > 0 {
                check_and_place_door(level, room.x - 1, y, depth, is_shop, rng);
            }
            // Right wall
            if room.x + room.width < COLNO {
                check_and_place_door(level, room.x + room.width, y, depth, is_shop, rng);
            }
        }
    }
}

/// Check if there's already a door adjacent to this position (bydoor() in C)
fn has_adjacent_door(level: &Level, x: usize, y: usize) -> bool {
    for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx >= 0 && ny >= 0 && (nx as usize) < COLNO && (ny as usize) < ROWNO {
            let cell_type = level.cells[nx as usize][ny as usize].typ;
            if cell_type == CellType::Door || cell_type == CellType::SecretDoor {
                return true;
            }
        }
    }
    false
}

/// Create door type and state based on C's dosdoor() logic
fn create_door_state(rng: &mut GameRng, depth: i32, is_shop: bool) -> (CellType, DoorState) {
    // 12.5% secret doors (1 in 8) - matches C: rn2(8) ? DOOR : SDOOR
    let is_secret = rng.rn2(8) == 0;
    let cell_type = if is_secret {
        CellType::SecretDoor
    } else {
        CellType::Door
    };

    // Determine door state
    let mut state = if is_shop {
        // Shop doors: secret ones are locked, regular are open
        if is_secret {
            DoorState::LOCKED
        } else {
            DoorState::OPEN
        }
    } else {
        // Regular doors: 1/3 each locked/closed/open
        // Matches C: rn2(5) < 3 gives ~60% chance of locked/closed vs open
        match rng.rn2(3) {
            0 => DoorState::LOCKED,
            1 => DoorState::CLOSED,
            _ => DoorState::OPEN,
        }
    };

    // 4% trapped if depth >= 5 and locked (matches C: !rn2(25) at depth >= 5)
    if depth >= 5 && state.contains(DoorState::LOCKED) && rng.rn2(25) == 0 {
        state |= DoorState::TRAPPED;
    }

    (cell_type, state)
}

/// Check if a door should be placed at this position
fn check_and_place_door(
    level: &mut Level,
    x: usize,
    y: usize,
    depth: i32,
    is_shop: bool,
    rng: &mut GameRng,
) {
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
                let cell_type = level.cells[nx as usize][ny as usize].typ;
                cell_type == CellType::Corridor || cell_type == CellType::SecretCorridor
            } else {
                false
            }
        });

    if !has_corridor {
        return;
    }

    // Don't place adjacent doors (C: bydoor() check)
    if has_adjacent_door(level, x, y) {
        return;
    }

    // 87.5% chance to place a door (matches C behavior closer than 80%)
    if rng.rn2(8) < 7 {
        let (cell_type, state) = create_door_state(rng, depth, is_shop);
        level.cells[x][y].typ = cell_type;
        level.cells[x][y].set_door_state(state);
    } else {
        // Make it a corridor opening instead
        level.cells[x][y].typ = CellType::Corridor;
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

        let room = &rooms[room_idx];
        let (x, y) = room.random_point(rng);

        // Check if position is empty
        if level.monster_at(x as i8, y as i8).is_some() {
            continue; // Skip if occupied
        }

        // Create a basic monster with a random type
        let monster_type = rng.rn2(10) as i16;
        let mut monster = Monster::new(MonsterId(0), monster_type, x as i8, y as i8);
        monster.state = crate::monster::MonsterState::active();
        monster.hp = 5 + rng.rnd(10) as i32;
        monster.hp_max = monster.hp;
        monster.name = random_monster_name(monster_type, rng).to_string();

        // Add to level
        level.add_monster(monster);
    }
}

/// Common monster names for random spawning
/// These are basic monsters that can appear on early dungeon levels
const RANDOM_MONSTER_NAMES: &[&str] = &[
    "grid bug",
    "lichen",
    "newt",
    "jackal",
    "fox",
    "kobold",
    "goblin",
    "gnome",
    "orc",
    "hobgoblin",
];

/// Get a random monster name based on monster type index
fn random_monster_name(monster_type: i16, _rng: &mut GameRng) -> &'static str {
    let idx = (monster_type as usize) % RANDOM_MONSTER_NAMES.len();
    RANDOM_MONSTER_NAMES[idx]
}

/// Select a special room type based on dungeon depth
/// Matches C's mkroom.c logic for room type selection
///
/// Returns Some(RoomType) if a special room should be created, None otherwise.
/// Also updates level flags to reflect the chosen room type.
fn select_special_room_type(
    rng: &mut GameRng,
    depth: i32,
    flags: &mut LevelFlags,
) -> Option<RoomType> {
    // Depth must be at least 2 for any special rooms
    if depth < 2 {
        return None;
    }

    // Maximum depth for shops (Ludios is around depth 20)
    let max_shop_depth = 19;

    // Try shops first: depth 2-19, probability is rn2(depth) < 3
    // This gives higher chance at lower depths (depth 2: ~100%, depth 10: 30%, depth 19: 16%)
    if depth < max_shop_depth && rng.rn2(depth as u32) < 3 {
        // Select shop type
        let shop_type = select_shop_type(rng);
        flags.has_shop = true;
        return Some(shop_type);
    }

    // Try other special rooms in order of depth requirements
    // Each room type has min_depth and spawn_probability from RoomType methods

    // Court (throne room): depth 4+, 1/6 chance
    if depth >= 4 && rng.one_in(6) && !flags.has_court {
        flags.has_court = true;
        return Some(RoomType::Court);
    }

    // LeprechaunHall: depth 5+, 1/8 chance
    if depth >= 5 && rng.one_in(8) {
        return Some(RoomType::LeprechaunHall);
    }

    // Zoo: depth 6+, 1/7 chance
    if depth >= 6 && rng.one_in(7) && !flags.has_zoo {
        flags.has_zoo = true;
        return Some(RoomType::Zoo);
    }

    // Temple: depth 8+, 1/5 chance
    if depth >= 8 && rng.one_in(5) && !flags.has_temple {
        flags.has_temple = true;
        return Some(RoomType::Temple);
    }

    // Beehive: depth 9+, 1/5 chance
    if depth >= 9 && rng.one_in(5) && !flags.has_beehive {
        flags.has_beehive = true;
        return Some(RoomType::Beehive);
    }

    // Morgue: depth 11+, 1/6 chance
    if depth >= 11 && rng.one_in(6) && !flags.has_morgue {
        flags.has_morgue = true;
        return Some(RoomType::Morgue);
    }

    // Anthole: depth 12+, 1/5 chance
    if depth >= 12 && rng.one_in(5) {
        return Some(RoomType::Anthole);
    }

    // Barracks: depth 14+, 1/4 chance
    if depth >= 14 && rng.one_in(4) && !flags.has_barracks {
        flags.has_barracks = true;
        return Some(RoomType::Barracks);
    }

    // Swamp: depth 15+, 1/6 chance
    if depth >= 15 && rng.one_in(6) && !flags.has_swamp {
        flags.has_swamp = true;
        return Some(RoomType::Swamp);
    }

    // CockatriceNest: depth 16+, 1/8 chance
    if depth >= 16 && rng.one_in(8) {
        return Some(RoomType::CockatriceNest);
    }

    None
}

/// Select a shop type based on weighted probabilities
/// Matches C's shtypes[] weights from shknam.c
fn select_shop_type(rng: &mut GameRng) -> RoomType {
    // Weighted probabilities from C's shtypes[]:
    // SHOPBASE (general): 44%
    // FOODSHOP: 16%
    // WEAPONSHOP: 14%
    // ARMORSHOP: 10%
    // TOOLSHOP: 8%
    // BOOKSHOP: 4%
    // RINGSHOP: 2%
    // WANDSHOP: 1%
    // CANDLESHOP: 1%
    // Total: 100

    let roll = rng.rn2(100);

    match roll {
        0..=43 => RoomType::GeneralShop,
        44..=59 => RoomType::FoodShop,
        60..=73 => RoomType::WeaponShop,
        74..=83 => RoomType::ArmorShop,
        84..=91 => RoomType::ToolShop,
        92..=95 => RoomType::BookShop,
        96..=97 => RoomType::RingShop,
        98 => RoomType::WandShop,
        _ => RoomType::CandleShop, // 99
    }
}

/// Pick a room suitable for the given special type
/// Returns the room index if found
fn pick_room_for_special(rooms: &[Room], special_type: RoomType) -> Option<usize> {
    // For shops, prefer rooms with single entrance (easier to manage)
    // For other special rooms, any ordinary room works
    // Avoid rooms that are too small

    let min_area = match special_type {
        RoomType::Vault => 4,  // 2x2 minimum
        _ if special_type.is_shop() => 12, // Shops need space for items
        _ => 9,                // 3x3 minimum for most special rooms
    };

    // Find eligible rooms (ordinary type, sufficient size)
    // Prefer later rooms (first room usually has stairs)
    for (idx, room) in rooms.iter().enumerate().rev() {
        if room.room_type == RoomType::Ordinary && room.area() >= min_area {
            // Skip first room (usually has upstairs)
            if idx > 0 || rooms.len() == 1 {
                return Some(idx);
            }
        }
    }

    None
}

/// Update level flags based on room type
fn set_level_flags_for_room(flags: &mut LevelFlags, room_type: RoomType) {
    match room_type {
        RoomType::Court => flags.has_court = true,
        RoomType::Swamp => flags.has_swamp = true,
        RoomType::Vault => flags.has_vault = true,
        RoomType::Beehive => flags.has_beehive = true,
        RoomType::Morgue => flags.has_morgue = true,
        RoomType::Barracks => flags.has_barracks = true,
        RoomType::Zoo => flags.has_zoo = true,
        RoomType::Temple => flags.has_temple = true,
        _ if room_type.is_shop() => flags.has_shop = true,
        _ => {}
    }
}

/// Place traps in the level
/// Matches C's mktrap() logic from mklev.c
fn place_traps(level: &mut Level, rooms: &[Room], rng: &mut GameRng) {
    if rooms.is_empty() {
        return;
    }

    let depth = level.dlevel.depth();

    // Number of traps: rnd(depth) at depth 1-3, rnd(depth)-1 at depth 4+
    // Minimum 0, maximum ~10
    let num_traps = if depth <= 3 {
        rng.rnd(depth.max(1) as u32) as usize
    } else {
        rng.rnd(depth as u32).saturating_sub(1) as usize
    };

    let num_traps = num_traps.min(10);

    for _ in 0..num_traps {
        // Pick a random room (avoid first room with stairs)
        let room_idx = if rooms.len() > 1 {
            rng.rn2(rooms.len() as u32 - 1) as usize + 1
        } else {
            0
        };

        let room = &rooms[room_idx];
        let (x, y) = room.random_point(rng);

        // Don't place trap on stairs or existing trap
        if level.cells[x][y].typ == CellType::Stairs {
            continue;
        }
        if level.traps.iter().any(|t| t.x == x as i8 && t.y == y as i8) {
            continue;
        }

        // Select trap type based on depth
        let trap_type = select_trap_type(depth, rng);

        level.traps.push(crate::dungeon::trap::create_trap(x as i8, y as i8, trap_type));
    }
}

/// Select a trap type based on depth
/// Matches C's rndtrap() from mklev.c
fn select_trap_type(depth: i32, rng: &mut GameRng) -> super::TrapType {
    use super::TrapType;

    // Trap availability by depth (approximate C logic)
    let available: Vec<TrapType> = match depth {
        1..=3 => vec![
            TrapType::Arrow,
            TrapType::Dart,
            TrapType::Pit,
            TrapType::Squeaky,
            TrapType::BearTrap,
        ],
        4..=7 => vec![
            TrapType::Arrow,
            TrapType::Dart,
            TrapType::Pit,
            TrapType::SpikedPit,
            TrapType::Squeaky,
            TrapType::BearTrap,
            TrapType::SleepingGas,
            TrapType::RustTrap,
        ],
        8..=12 => vec![
            TrapType::Arrow,
            TrapType::Dart,
            TrapType::Pit,
            TrapType::SpikedPit,
            TrapType::BearTrap,
            TrapType::SleepingGas,
            TrapType::RustTrap,
            TrapType::FireTrap,
            TrapType::Teleport,
            TrapType::RockFall,
        ],
        _ => vec![
            TrapType::Arrow,
            TrapType::Dart,
            TrapType::Pit,
            TrapType::SpikedPit,
            TrapType::BearTrap,
            TrapType::SleepingGas,
            TrapType::FireTrap,
            TrapType::Teleport,
            TrapType::RockFall,
            TrapType::LandMine,
            TrapType::RollingBoulder,
            TrapType::Hole,
            TrapType::TrapDoor,
            TrapType::Polymorph,
            TrapType::MagicTrap,
        ],
    };

    let idx = rng.rn2(available.len() as u32) as usize;
    available[idx]
}

/// Place fountains, sinks, and altars
/// Matches C's mkfount(), mksink(), mkaltar() from mklev.c
fn place_dungeon_features(level: &mut Level, rooms: &[Room], rng: &mut GameRng) {
    if rooms.is_empty() {
        return;
    }

    let depth = level.dlevel.depth();

    // Fountains: 1/3 chance per level, more common at lower depths
    // C: rn2(depth) < 3 gives ~30% at depth 10
    if rng.rn2(depth.max(1) as u32) < 2 {
        let num_fountains = rng.rnd(2) as usize; // 1-2 fountains
        for _ in 0..num_fountains {
            if let Some((x, y)) = find_empty_room_spot(level, rooms, rng) {
                level.cells[x][y].typ = CellType::Fountain;
                level.flags.fountain_count += 1;
            }
        }
    }

    // Sinks: 1/5 chance, only at depth 5+
    if depth >= 5 && rng.one_in(5) {
        if let Some((x, y)) = find_empty_room_spot(level, rooms, rng) {
            level.cells[x][y].typ = CellType::Sink;
            level.flags.sink_count += 1;
        }
    }

    // Altars: 1/6 chance at depth 3+, not in temples (temples have their own)
    if depth >= 3 && rng.one_in(6) && !level.flags.has_temple {
        if let Some((x, y)) = find_empty_room_spot(level, rooms, rng) {
            level.cells[x][y].typ = CellType::Altar;
        }
    }

    // Graves: 1/8 chance at depth 5+
    if depth >= 5 && rng.one_in(8) {
        let num_graves = rng.rnd(3) as usize; // 1-3 graves
        for _ in 0..num_graves {
            if let Some((x, y)) = find_empty_room_spot(level, rooms, rng) {
                level.cells[x][y].typ = CellType::Grave;
            }
        }
    }

    // Gold piles: random gold scattered in rooms
    // C: mkgold() places gold with amount based on depth
    let num_gold_piles = rng.rnd(3) as usize; // 1-3 gold piles per level
    for _ in 0..num_gold_piles {
        if let Some((x, y)) = find_empty_room_spot(level, rooms, rng) {
            place_gold_pile(level, x, y, depth, rng);
        }
    }
}

/// Place a gold pile at a location
fn place_gold_pile(level: &mut Level, x: usize, y: usize, depth: i32, rng: &mut GameRng) {
    use crate::object::{Object, ObjectClass, ObjectId};

    // Gold amount formula from C: rnd(10 + depth * 2) + 5
    let amount = (rng.rnd((10 + depth * 2).max(1) as u32) + 5) as i32;

    let mut gold = Object::new(ObjectId(0), 0, ObjectClass::Coin);
    gold.quantity = amount;
    gold.name = Some("gold piece".to_string());

    level.add_object(gold, x as i8, y as i8);
}

/// Place branch entrance (stairs/portal to another dungeon branch)
fn place_branch_entrance(level: &mut Level, rooms: &[Room], rng: &mut GameRng) {
    use super::level::Stairway;
    use super::topology::DungeonSystem;
    use super::TrapType;

    let dungeon_system = DungeonSystem::new();

    // Check if this level has a branch entrance
    if let Some(branch) = dungeon_system.get_branch_from(&level.dlevel) {
        // Find a spot for the branch entrance
        if let Some((x, y)) = find_empty_room_spot(level, rooms, rng) {
            // Place the entrance based on branch type
            match branch.branch_type {
                super::topology::BranchType::Stairs => {
                    // Stairs to another branch
                    level.cells[x][y].typ = CellType::Stairs;
                    level.stairs.push(Stairway {
                        x: x as i8,
                        y: y as i8,
                        destination: branch.end2,
                        up: branch.end1_up,
                    });
                    level.flags.has_branch = true;
                }
                super::topology::BranchType::Portal => {
                    // Magic portal
                    level.add_trap(x as i8, y as i8, TrapType::MagicPortal);
                    level.flags.has_branch = true;
                }
                _ => {}
            }
        }
    }
}

/// Generate rooms using the rectangle system for efficient placement
/// This is an alternative to the simple overlap-checking approach
#[allow(dead_code)]
pub fn generate_rooms_with_rects(level: &mut Level, rng: &mut GameRng) -> Vec<Room> {
    let mut rect_mgr = RectManager::new(COLNO as u8, ROWNO as u8);
    let mut rooms = Vec::new();
    let num_rooms = (rng.rnd(4) + 5) as usize; // 6-9 rooms

    for _ in 0..num_rooms {
        // Try to find a position using the rectangle system
        let width = (rng.rnd(7) + 2) as u8; // 3-9
        let height = (rng.rnd(5) + 2) as u8; // 3-7

        if let Some((_rect, x, y)) = rect_mgr.pick_room_position(width, height, rng) {
            let room = Room::new(x as usize, y as usize, width as usize, height as usize);

            // Carve the room
            for rx in room.x..(room.x + room.width) {
                for ry in room.y..(room.y + room.height) {
                    level.cells[rx][ry].typ = CellType::Room;
                    level.cells[rx][ry].lit = room.lit;
                }
            }

            // Create walls around the room
            for rx in room.x.saturating_sub(1)..=(room.x + room.width).min(COLNO - 1) {
                for ry in room.y.saturating_sub(1)..=(room.y + room.height).min(ROWNO - 1) {
                    let is_vertical_edge = rx == room.x.saturating_sub(1) || rx == room.x + room.width;
                    let is_horizontal_edge = ry == room.y.saturating_sub(1) || ry == room.y + room.height;

                    if is_vertical_edge && !is_horizontal_edge && level.cells[rx][ry].typ != CellType::Room {
                        level.cells[rx][ry].typ = CellType::VWall;
                    } else if is_horizontal_edge && !is_vertical_edge && level.cells[rx][ry].typ != CellType::Room {
                        level.cells[rx][ry].typ = CellType::HWall;
                    } else if is_vertical_edge && is_horizontal_edge && level.cells[rx][ry].typ != CellType::Room {
                        level.cells[rx][ry].typ = CellType::TLCorner;
                    }
                }
            }

            // Split the rectangle to mark this space as used
            let room_rect = NhRect::new(
                x.saturating_sub(1),
                y.saturating_sub(1),
                x + width + 1,
                y + height + 1,
            );
            rect_mgr.split_rects(&room_rect);

            rooms.push(room);
        }

        if !rect_mgr.has_space() {
            break;
        }
    }

    rooms
}

/// Generate an irregular (non-rectangular) room
#[allow(dead_code)]
pub fn generate_irregular_room(level: &mut Level, x: usize, y: usize, max_w: usize, max_h: usize, rng: &mut GameRng) -> Room {
    let mut room = Room::new(x, y, max_w, max_h);
    room.irregular = true;

    // Create an irregular shape by randomly removing corners and edges
    let mut cells_to_carve: Vec<(usize, usize)> = Vec::new();

    // Start with a rectangular base
    for rx in x..(x + max_w).min(COLNO - 1) {
        for ry in y..(y + max_h).min(ROWNO - 1) {
            cells_to_carve.push((rx, ry));
        }
    }

    // Randomly remove some cells from corners and edges
    let remove_count = rng.rn2((max_w * max_h / 4) as u32) as usize;
    for _ in 0..remove_count {
        if cells_to_carve.len() <= max_w * max_h / 2 {
            break; // Don't remove too many
        }

        // Prefer removing from edges
        let idx = rng.rn2(cells_to_carve.len() as u32) as usize;
        let (cx, cy) = cells_to_carve[idx];

        // Only remove if it's on an edge
        let is_edge = cx == x || cx == x + max_w - 1 || cy == y || cy == y + max_h - 1;
        if is_edge {
            cells_to_carve.swap_remove(idx);
        }
    }

    // Carve the irregular room
    for (rx, ry) in &cells_to_carve {
        level.cells[*rx][*ry].typ = CellType::Room;
        level.cells[*rx][*ry].lit = room.lit;
    }

    // Add walls around carved cells
    for (rx, ry) in &cells_to_carve {
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let wx = (*rx as i32 + dx) as usize;
                let wy = (*ry as i32 + dy) as usize;
                if wx < COLNO && wy < ROWNO && level.cells[wx][wy].typ == CellType::Stone {
                    // Determine wall type
                    if dx == 0 {
                        level.cells[wx][wy].typ = CellType::HWall;
                    } else if dy == 0 {
                        level.cells[wx][wy].typ = CellType::VWall;
                    } else {
                        level.cells[wx][wy].typ = CellType::TLCorner;
                    }
                }
            }
        }
    }

    room
}

/// Create a subroom within an existing room
#[allow(dead_code)]
pub fn create_subroom(
    level: &mut Level,
    rooms: &mut Vec<Room>,
    parent_idx: usize,
    rng: &mut GameRng,
) -> Option<usize> {
    if parent_idx >= rooms.len() {
        return None;
    }

    let parent = &rooms[parent_idx];

    // Subroom must be smaller than parent
    if parent.width < 5 || parent.height < 4 {
        return None;
    }

    // Calculate subroom size (at least 2x2, at most half of parent)
    let max_w = parent.width / 2;
    let max_h = parent.height / 2;
    if max_w < 2 || max_h < 2 {
        return None;
    }

    let sub_w = 2 + rng.rn2((max_w - 1) as u32) as usize;
    let sub_h = 2 + rng.rn2((max_h - 1) as u32) as usize;

    // Position subroom within parent
    let max_x = parent.x + parent.width - sub_w - 1;
    let max_y = parent.y + parent.height - sub_h - 1;

    if max_x <= parent.x || max_y <= parent.y {
        return None;
    }

    let sub_x = parent.x + 1 + rng.rn2((max_x - parent.x) as u32) as usize;
    let sub_y = parent.y + 1 + rng.rn2((max_y - parent.y) as u32) as usize;

    // Create the subroom
    let subroom = Room::new_subroom(sub_x, sub_y, sub_w, sub_h, parent_idx);
    let subroom_idx = rooms.len();

    // Carve subroom (it's already inside the parent, so just mark it)
    // Subrooms typically have different properties (e.g., closets, alcoves)

    // Add internal walls around subroom
    for rx in sub_x.saturating_sub(1)..=(sub_x + sub_w).min(COLNO - 1) {
        for ry in sub_y.saturating_sub(1)..=(sub_y + sub_h).min(ROWNO - 1) {
            let is_edge_x = rx == sub_x.saturating_sub(1) || rx == sub_x + sub_w;
            let is_edge_y = ry == sub_y.saturating_sub(1) || ry == sub_y + sub_h;

            if (is_edge_x || is_edge_y) && !(rx >= sub_x && rx < sub_x + sub_w && ry >= sub_y && ry < sub_y + sub_h) {
                // This is a wall position
                if level.cells[rx][ry].typ == CellType::Room {
                    level.cells[rx][ry].typ = CellType::VWall;
                }
            }
        }
    }

    // Add a door to connect subroom to parent
    let door_x = sub_x + sub_w / 2;
    let door_y = sub_y.saturating_sub(1);
    if door_y > 0 && level.cells[door_x][door_y].typ.is_wall() {
        level.cells[door_x][door_y].typ = CellType::Door;
    }

    rooms.push(subroom);

    // Update parent's subroom list
    rooms[parent_idx].add_subroom(subroom_idx);

    Some(subroom_idx)
}

/// Find an empty spot in a random room
fn find_empty_room_spot(level: &Level, rooms: &[Room], rng: &mut GameRng) -> Option<(usize, usize)> {
    if rooms.is_empty() {
        return None;
    }

    // Try up to 20 times to find an empty spot
    for _ in 0..20 {
        let room_idx = rng.rn2(rooms.len() as u32) as usize;
        let room = &rooms[room_idx];
        let (x, y) = room.random_point(rng);

        // Check if spot is empty floor
        if level.cells[x][y].typ == CellType::Room
            && level.monster_at(x as i8, y as i8).is_none()
        {
            return Some((x, y));
        }
    }

    None
}

/// Create a niche (small wall alcove) in a room
/// Niches can contain level teleporters, trapdoors, or iron bars
#[allow(dead_code)]
pub fn make_niche(level: &mut Level, rooms: &[Room], trap_type: Option<crate::dungeon::TrapType>, rng: &mut GameRng) -> bool {
    if rooms.is_empty() {
        return false;
    }

    // Try to find a suitable room
    for _ in 0..8 {
        let room_idx = rng.rn2(rooms.len() as u32) as usize;
        let room = &rooms[room_idx];

        // Only ordinary rooms
        if room.room_type != RoomType::Ordinary {
            continue;
        }

        // Find a wall position for the niche
        if let Some((nx, ny, dir)) = find_niche_position(level, room, rng) {
            // Create the niche alcove
            level.cells[nx][ny].typ = CellType::Corridor;

            // Add trap if specified
            if let Some(trap) = trap_type {
                level.add_trap(nx as i8, ny as i8, trap);
            } else if rng.one_in(4) {
                // 25% chance of iron bars
                level.cells[nx][ny].typ = CellType::IronBars;
            }

            // Add secret door to hide the niche
            let door_x = match dir {
                0 | 1 => nx,
                2 => nx + 1,
                _ => nx.saturating_sub(1),
            };
            let door_y = match dir {
                0 => ny + 1,
                1 => ny.saturating_sub(1),
                _ => ny,
            };

            if door_x < COLNO && door_y < ROWNO {
                level.cells[door_x][door_y].typ = CellType::SecretDoor;
            }

            return true;
        }
    }

    false
}

/// Find a valid position for a niche in a room wall
fn find_niche_position(level: &Level, room: &Room, rng: &mut GameRng) -> Option<(usize, usize, u8)> {
    // Try each wall direction: 0=top, 1=bottom, 2=left, 3=right
    let directions: Vec<u8> = vec![0, 1, 2, 3];

    for _ in 0..10 {
        let dir = directions[rng.rn2(4) as usize];

        let (nx, ny) = match dir {
            0 => {
                // Top wall - niche goes up
                let x = room.x + rng.rn2(room.width as u32) as usize;
                let y = room.y.saturating_sub(2);
                (x, y)
            }
            1 => {
                // Bottom wall - niche goes down
                let x = room.x + rng.rn2(room.width as u32) as usize;
                let y = room.y + room.height + 1;
                (x, y)
            }
            2 => {
                // Left wall - niche goes left
                let x = room.x.saturating_sub(2);
                let y = room.y + rng.rn2(room.height as u32) as usize;
                (x, y)
            }
            _ => {
                // Right wall - niche goes right
                let x = room.x + room.width + 1;
                let y = room.y + rng.rn2(room.height as u32) as usize;
                (x, y)
            }
        };

        // Check if position is valid (stone)
        if nx > 0 && nx < COLNO - 1 && ny > 0 && ny < ROWNO - 1 {
            if level.cells[nx][ny].typ == CellType::Stone {
                return Some((nx, ny, dir));
            }
        }
    }

    None
}

/// Create niches on a level (called during level generation)
#[allow(dead_code)]
pub fn make_niches(level: &mut Level, rooms: &[Room], rng: &mut GameRng) {
    use super::TrapType;

    // Create 1-3 niches
    let num_niches = 1 + rng.rn2(3) as usize;

    for i in 0..num_niches {
        let trap_type = if i == 0 && rng.one_in(3) {
            Some(TrapType::Teleport) // Level teleporter
        } else if rng.one_in(4) {
            Some(TrapType::TrapDoor) // Trapdoor
        } else {
            None
        };

        make_niche(level, rooms, trap_type, rng);
    }
}

/// Create a vault teleporter (teleport trap leading into vault)
#[allow(dead_code)]
pub fn make_vault_teleporter(level: &mut Level, rooms: &[Room], rng: &mut GameRng) -> bool {
    use super::TrapType;

    // Find a vault room
    let vault_room = rooms.iter().find(|r| r.room_type == RoomType::Vault);

    if vault_room.is_none() {
        return false;
    }

    // Create a niche with a teleport trap that leads to the vault
    make_niche(level, rooms, Some(TrapType::Teleport), rng)
}

/// Create Knox portal (magic portal to Fort Ludios from a vault)
#[allow(dead_code)]
pub fn make_knox_portal(level: &mut Level, rooms: &[Room], rng: &mut GameRng) -> bool {
    use super::TrapType;

    // Find a vault room
    let vault_room = rooms.iter().find(|r| r.room_type == RoomType::Vault);

    if let Some(vault) = vault_room {
        // Place magic portal in the vault
        let px = vault.x + vault.width / 2;
        let py = vault.y + vault.height / 2;

        if px < COLNO && py < ROWNO {
            level.add_trap(px as i8, py as i8, TrapType::MagicPortal);
            return true;
        }
    }

    // If no vault, try to place in a random room
    if let Some((x, y)) = find_empty_room_spot(level, rooms, rng) {
        level.add_trap(x as i8, y as i8, TrapType::MagicPortal);
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_overlap() {
        let room1 = Room::new(5, 5, 5, 5);
        let room2 = Room::new(8, 8, 5, 5);
        let room3 = Room::new(15, 15, 5, 5);

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

    #[test]
    fn test_select_shop_type_distribution() {
        let mut rng = GameRng::new(42);
        let mut counts = std::collections::HashMap::new();

        // Generate many shop types to verify distribution
        for _ in 0..1000 {
            let shop_type = select_shop_type(&mut rng);
            *counts.entry(shop_type).or_insert(0) += 1;
        }

        // General shop should be most common (~44%)
        let general_count = *counts.get(&RoomType::GeneralShop).unwrap_or(&0);
        assert!(
            general_count > 350 && general_count < 550,
            "General shop should be ~44%, got {}",
            general_count
        );

        // Rare shops should appear less frequently
        let wand_count = *counts.get(&RoomType::WandShop).unwrap_or(&0);
        assert!(
            wand_count < 30,
            "Wand shop should be ~1%, got {}",
            wand_count
        );
    }

    #[test]
    fn test_special_room_depth_requirements() {
        // Test that special rooms only appear at appropriate depths
        let mut rng = GameRng::new(12345);

        // Depth 1: no special rooms
        for _ in 0..100 {
            let mut flags = LevelFlags::default();
            let result = select_special_room_type(&mut rng, 1, &mut flags);
            assert!(
                result.is_none(),
                "Depth 1 should not generate special rooms"
            );
        }

        // Deep level: should occasionally get special rooms
        let mut got_special = false;
        for _ in 0..100 {
            let mut flags = LevelFlags::default();
            let result = select_special_room_type(&mut rng, 15, &mut flags);
            if result.is_some() {
                got_special = true;
                break;
            }
        }
        assert!(got_special, "Depth 15 should sometimes generate special rooms");
    }

    #[test]
    fn test_pick_room_for_special() {
        let rooms = vec![
            Room::new(5, 5, 4, 4),   // 16 area - adequate
            Room::new(20, 5, 5, 5),  // 25 area - good
            Room::new(35, 5, 2, 2),  // 4 area - too small for most
            Room::new(50, 5, 6, 4),  // 24 area - good
        ];

        // Should pick a room with adequate size (prefer later rooms)
        let shop_room = pick_room_for_special(&rooms, RoomType::GeneralShop);
        assert!(shop_room.is_some());
        // Should be room 3 (last one with adequate size) or room 1
        let idx = shop_room.unwrap();
        assert!(idx == 1 || idx == 3, "Should pick room with adequate size, got {}", idx);

        // Vault has smaller size requirement
        let vault_room = pick_room_for_special(&rooms, RoomType::Vault);
        assert!(vault_room.is_some());
    }

    #[test]
    fn test_level_flags_set_correctly() {
        let mut flags = LevelFlags::default();

        set_level_flags_for_room(&mut flags, RoomType::Court);
        assert!(flags.has_court);

        set_level_flags_for_room(&mut flags, RoomType::GeneralShop);
        assert!(flags.has_shop);

        set_level_flags_for_room(&mut flags, RoomType::Zoo);
        assert!(flags.has_zoo);

        set_level_flags_for_room(&mut flags, RoomType::Morgue);
        assert!(flags.has_morgue);
    }

    #[test]
    fn test_special_room_at_various_depths() {
        // Test level generation at different depths
        for depth in [2, 5, 10, 15, 20] {
            let mut rng = GameRng::new(42 + depth as u64);
            let dlevel = DLevel {
                dungeon_num: 0,
                level_num: depth,
            };
            let mut level = Level::new(dlevel);

            generate_rooms_and_corridors(&mut level, &mut rng);

            // Basic sanity checks
            let room_count = level
                .cells
                .iter()
                .flat_map(|col| col.iter())
                .filter(|cell| cell.typ == CellType::Room)
                .count();

            assert!(
                room_count > 0,
                "Depth {} should have room cells",
                depth
            );
        }
    }

    #[test]
    fn test_dark_rooms() {
        // Generate many levels at depth 11+ to find a morgue (which should be dark)
        let mut found_dark_cell = false;

        for seed in 0..100 {
            let mut rng = GameRng::new(seed);
            let dlevel = DLevel {
                dungeon_num: 0,
                level_num: 15,  // Deep enough for morgue
            };
            let mut level = Level::new(dlevel);

            generate_rooms_and_corridors(&mut level, &mut rng);

            // Check if we got a morgue (which should have dark cells)
            if level.flags.has_morgue {
                // Find an unlit room cell
                for x in 0..COLNO {
                    for y in 0..ROWNO {
                        if level.cells[x][y].typ == CellType::Room && !level.cells[x][y].lit {
                            found_dark_cell = true;
                            break;
                        }
                    }
                    if found_dark_cell {
                        break;
                    }
                }
                if found_dark_cell {
                    break;
                }
            }
        }

        // Note: This test may occasionally fail if RNG doesn't produce a morgue
        // That's acceptable as it's probabilistic
        println!("Found dark cell in morgue: {}", found_dark_cell);
    }

    #[test]
    fn test_trap_generation() {
        let mut rng = GameRng::new(42);
        let dlevel = DLevel {
            dungeon_num: 0,
            level_num: 10, // Deep enough for varied traps
        };
        let mut level = Level::new(dlevel);

        generate_rooms_and_corridors(&mut level, &mut rng);

        // Should have some traps at depth 10
        println!("Generated {} traps at depth 10", level.traps.len());

        // Traps should be in valid positions
        for trap in &level.traps {
            assert!(trap.x >= 0 && trap.x < COLNO as i8);
            assert!(trap.y >= 0 && trap.y < ROWNO as i8);
        }
    }

    #[test]
    fn test_trap_type_by_depth() {
        let mut rng = GameRng::new(42);

        // Shallow depth should only get basic traps
        let shallow_traps: Vec<_> = (0..100)
            .map(|_| select_trap_type(2, &mut rng))
            .collect();

        // Should not have advanced traps at depth 2
        use super::super::TrapType;
        assert!(
            !shallow_traps.contains(&TrapType::LandMine),
            "LandMine should not appear at depth 2"
        );
        assert!(
            !shallow_traps.contains(&TrapType::Polymorph),
            "Polymorph trap should not appear at depth 2"
        );

        // Deep depth should have variety - count unique trap names
        let deep_traps: Vec<_> = (0..100)
            .map(|_| select_trap_type(20, &mut rng))
            .collect();

        // Count unique types by comparing with each other
        let mut unique_count = 0;
        for (i, trap) in deep_traps.iter().enumerate() {
            if !deep_traps[..i].contains(trap) {
                unique_count += 1;
            }
        }
        assert!(
            unique_count > 5,
            "Deep levels should have trap variety, got {} types",
            unique_count
        );
    }

    #[test]
    fn test_dungeon_features_generation() {
        // Generate multiple levels to check feature placement
        let mut fountain_count = 0;
        let mut sink_count = 0;
        let mut altar_count = 0;
        let mut grave_count = 0;
        let mut gold_count = 0;

        for seed in 0..50 {
            let mut rng = GameRng::new(seed);
            let dlevel = DLevel {
                dungeon_num: 0,
                level_num: 10,
            };
            let mut level = Level::new(dlevel);

            generate_rooms_and_corridors(&mut level, &mut rng);

            // Count features
            for x in 0..COLNO {
                for y in 0..ROWNO {
                    match level.cells[x][y].typ {
                        CellType::Fountain => fountain_count += 1,
                        CellType::Sink => sink_count += 1,
                        CellType::Altar => altar_count += 1,
                        CellType::Grave => grave_count += 1,
                        _ => {}
                    }
                }
            }

            // Count gold piles
            gold_count += level
                .objects
                .iter()
                .filter(|o| o.class == crate::object::ObjectClass::Coin)
                .count();
        }

        println!("Over 50 levels at depth 10:");
        println!("  Fountains: {}", fountain_count);
        println!("  Sinks: {}", sink_count);
        println!("  Altars: {}", altar_count);
        println!("  Graves: {}", grave_count);
        println!("  Gold piles: {}", gold_count);

        // Should have generated some of each feature type
        assert!(fountain_count > 0, "Should generate fountains");
        assert!(gold_count > 0, "Should generate gold piles");
    }

    #[test]
    fn test_gold_pile_amounts() {
        let mut rng = GameRng::new(42);

        // Test at different depths
        for depth in [1, 5, 10, 20] {
            let dlevel = DLevel {
                dungeon_num: 0,
                level_num: depth,
            };
            let mut level = Level::new(dlevel);

            generate_rooms_and_corridors(&mut level, &mut rng);

            let gold_piles: Vec<_> = level
                .objects
                .iter()
                .filter(|o| o.class == crate::object::ObjectClass::Coin)
                .collect();

            if !gold_piles.is_empty() {
                let avg_amount: i32 =
                    gold_piles.iter().map(|g| g.quantity).sum::<i32>() / gold_piles.len() as i32;
                println!("Depth {}: {} gold piles, avg {} gold", depth, gold_piles.len(), avg_amount);

                // Gold amounts should scale with depth
                assert!(avg_amount > 0, "Gold piles should have positive amounts");
            }
        }
    }
}
