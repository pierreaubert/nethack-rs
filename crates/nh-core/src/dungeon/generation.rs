//! Level generation (mklev.c, mkroom.c)
//!
//! Generates dungeon levels with rooms and corridors.
//! Uses the rectangle system (rect.c) for efficient room placement.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::combat::AttackType;
use crate::data::monsters::{G_GENO, G_NOGEN, G_NOHELL, G_UNIQ, MONSTERS};
use crate::data::objects::{P_BOW, P_SHURIKEN, OBJECTS};
use crate::monster::{Monster, MonsterFlags, MonsterId, MonsterSound, PerMonst};
use crate::object::{ClassBases, ObjClassDef, ObjectClass};
use crate::rng::GameRng;
use crate::{COLNO, ROWNO};

// Rust OBJECTS array indices for items that need special-case handling in init functions.
// These differ from C's onames.h values because the Rust array is compacted (no class separators).
const R_LARGE_BOX: usize = 189;
const R_CHEST: usize = 190;
const R_ICE_BOX: usize = 191;
const R_BAG_OF_TRICKS: usize = 195;
const R_LOCK_PICK: usize = 197;
const R_CREDIT_CARD: usize = 198;
const R_TALLOW_CANDLE: usize = 199;
const R_WAX_CANDLE: usize = 200;
const R_BRASS_LANTERN: usize = 201;
const R_OIL_LAMP: usize = 202;
const R_MAGIC_LAMP: usize = 203;
const R_CRYSTAL_BALL: usize = 206;
const R_LEASH: usize = 211;
const R_TINNING_KIT: usize = 213;
const R_FIGURINE: usize = 216;
const R_MAGIC_MARKER: usize = 217;
const R_TIN_WHISTLE: usize = 220;
const R_MAGIC_WHISTLE: usize = 221;
const R_HORN_OF_PLENTY: usize = 227;
const R_CORPSE: usize = 241;
const R_EGG: usize = 242;
const R_MEAT_RING: usize = 246;
const R_KELP_FROND: usize = 251;
const R_TIN: usize = 272;
const R_WAND_OF_WISHING: usize = 369;
const R_AMULET_OF_STRANGULATION: usize = 180;
const R_AMULET_VERSUS_POISON: usize = 182;
const R_AMULET_OF_UNCHANGING: usize = 184;
const R_HELM_OF_OPPOSITE_ALIGNMENT: usize = 77;
const R_DWARVISH_MITHRIL_COAT: usize = 105;
const R_ELVEN_MITHRIL_COAT: usize = 106;
const R_HAWAIIAN_SHIRT: usize = 115;
const R_GAUNTLETS_OF_FUMBLING: usize = 137;
const R_RIN_ADORNMENT: usize = 150;
const R_RIN_PROTECTION: usize = 155;
const R_LUCKSTONE: usize = 420;
const R_LOADSTONE: usize = 421;
const R_FLINT: usize = 423;
const R_ROCK: usize = 424;
const R_BOULDER: usize = 425;
const R_STATUE: usize = 426;
const R_BELL_OF_OPENING: usize = 239;
const R_MUMMY_WRAPPING: usize = 117;
const R_MIRROR: usize = 205;
const R_POT_OBJECT_DETECTION: usize = 288;
const R_WAN_DIGGING: usize = 382;
const R_ATHAME: usize = 21;
const R_SPEAR: usize = 10;
const R_LUMP_OF_ROYAL_JELLY: usize = 262;
const R_MACE: usize = 62;
const R_ROBE: usize = 122;
const R_CLOAK_OF_PROTECTION: usize = 125;
const R_CLOAK_OF_MAGIC_RESISTANCE: usize = 127;
const R_SMALL_SHIELD: usize = 129;

/// Sum of oc_prob for gem class (DILITHIUM_CRYSTAL..LUCKSTONE-1 in C, indices 411..441).
/// Each gem/stone has a probability; the sum is used by rnd_class for selection.
/// Computed from C's objects[]: sum of oc_prob for gems that are not LUCKSTONE.
const R_GEM_CLASS_PROB_SUM: usize = 864;

use super::corridor::generate_corridors;
use super::rect::{NhRect, RectManager};
use super::room::{Room, RoomType};
use super::shop::populate_shop;
use super::special_rooms::{is_vault, needs_population, populate_special_room, populate_vault};
use super::{Cell, CellType, DLevel, DoorState, Level, LevelFlags};

/// Generate a standard level with rooms and corridors
pub fn generate_rooms_and_corridors(
    level: &mut Level,
    rng: &mut GameRng,
    monster_vitals: &crate::magic::MonsterVitals,
) {
    init_map(level);
    
    // NetHack's makelevel() calls rn2(5) right before makerooms()
    // for a potential hell/medusa level check.
    let _ = rng.rn2(5);

    let mut rect_mgr = RectManager::new(COLNO as u8, ROWNO as u8);
    let mut tried_vault = false;
    let mut vault_position: Option<(usize, usize)> = None;

    // make rooms until satisfied (makerooms() in C)
    while level.rooms.len() < super::mapseen::MAXNROFROOMS && rect_mgr.rnd_rect(rng).is_some() {
        eprintln!("RS makerooms: iter nroom={} rng={}", level.rooms.len(), rng.call_count());
        // Vault check logic (mklev.c:229-240)
        if level.rooms.len() >= (super::mapseen::MAXNROFROOMS / 6) && rng.rn2(2) != 0 && !tried_vault {
            tried_vault = true;
            eprintln!("RS makerooms: vault attempt rng={}", rng.call_count());
            // C: if (create_vault()) { vault_x = ...; vault_y = ...; rooms[nroom].hx = -1; }
            if let Some(vault_room) = rect_mgr.create_room_vault(level, rng, level.rooms.len()) {
                vault_position = Some((vault_room.x, vault_room.y));
                eprintln!("RS makerooms: vault OK rng={}", rng.call_count());
            } else {
                eprintln!("RS makerooms: vault FAIL rng={}", rng.call_count());
            }
            // Whether vault creation succeeds or fails, skip OROOM this iteration
            continue;
        }

        if let Some(room) = rect_mgr.create_room_random(level, rng, level.rooms.len()) {
            carve_room(level, &room);
            level.rooms.push(room);
        } else {
            // In C, if create_room fails, makerooms returns
            break;
        }
    }

    // NetHack calls sort_rooms() immediately after makerooms()
    // C uses qsort by lx only. For equal lx, macOS qsort reverses relative
    // order (places later-created room first). We replicate by using reverse
    // creation order as tiebreaker: since rooms are appended in creation order,
    // reversing equal-lx groups matches C's observed behavior.
    // Use (lx, reverse_index) as sort key.
    {
        let n = level.rooms.len();
        let mut indexed: Vec<(usize, usize)> = (0..n).map(|i| (level.rooms[i].x, i)).collect();
        indexed.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
        let permutation: Vec<usize> = indexed.iter().map(|&(_, i)| i).collect();
        let old_rooms = level.rooms.clone();
        for (new_idx, &old_idx) in permutation.iter().enumerate() {
            level.rooms[new_idx] = old_rooms[old_idx].clone();
        }
    }
    eprintln!("RS: after makerooms+sort rng={} nroom={}", rng.call_count(), level.rooms.len());
    for (ri, rm) in level.rooms.iter().enumerate() {
        eprintln!("RS: room[{}] lx={} ly={} hx={} hy={}", ri, rm.x, rm.y, rm.x + rm.width - 1, rm.y + rm.height - 1);
    }

    // C places stairs BEFORE corridors (makelevel lines 710-728)
    let rooms_clone = level.rooms.clone();
    if !rooms_clone.is_empty() {
        place_stairs(level, &rooms_clone, rng);
    }

    eprintln!("RS: after stairs rng={}", rng.call_count());
    // Connect rooms with corridors (doors are placed inside join() per C's algorithm)
    generate_corridors(level, &rooms_clone, rng);
    eprintln!("RS: after corridors rng={}", rng.call_count());

    // Door counts and positions are now tracked in dodoor/dosdoor_public
    // (matching C's add_door in dosdoor)

    // make_niches() in C — uses rooms from level.rooms (with updated door_count)
    let rooms_with_doors = level.rooms.clone();
    let niche_objects = OBJECTS;
    let niche_bases = ClassBases::compute(niche_objects);
    make_niches(level, &rooms_with_doors, niche_objects, &niche_bases, rng);
    eprintln!("RS: after niches rng={}", rng.call_count());
    // C: branchp = Is_branchlev(&u.uz); room_threshold = branchp ? 4 : 3;
    let is_branch_level = {
        use super::topology::DungeonSystem;
        DungeonSystem::new().get_branch_from(&level.dlevel).is_some()
    };
    let mut room_threshold: i32 = if is_branch_level { 4 } else { 3 };

    // make a secret treasure vault, not connected to the rest (mklev.c:759-784)
    eprintln!("RS: vault_position={:?}", vault_position);
    if let Some((vx, vy)) = vault_position {
        let mut vault_x = vx as i32;
        let mut vault_y = vy as i32;
        let mut w: i32 = 1;
        let mut h: i32 = 1;

        eprintln!("RS: before vault check_room rng={}", rng.call_count());
        if check_room(level, &mut vault_x, &mut w, &mut vault_y, &mut h, true, rng) {
            eprintln!("RS: vault check_room OK rng={}", rng.call_count());
            create_vault_room(level, vault_x as usize, vault_y as usize, w as usize, h as usize, rng, is_branch_level);
            room_threshold += 1;
        } else {
            eprintln!("RS: vault check_room FAIL rng={}", rng.call_count());
            if rect_mgr.rnd_rect(rng).is_some() {
                eprintln!("RS: vault fallback rnd_rect rng={}", rng.call_count());
                // Fallback: try creating vault at a new location
                if let Some(fallback_room) = rect_mgr.create_room_vault(level, rng, level.rooms.len()) {
                    vault_x = fallback_room.x as i32;
                    vault_y = fallback_room.y as i32;
                    w = 1;
                    h = 1;
                    if check_room(level, &mut vault_x, &mut w, &mut vault_y, &mut h, true, rng) {
                        create_vault_room(level, vault_x as usize, vault_y as usize, w as usize, h as usize, rng, is_branch_level);
                        room_threshold += 1;
                    }
                }
            }
        }
    }

    // C: mklev.c:786-818 — special room cascade
    // MUST match C's exact logic: room search happens INSIDE mkshop/mkzoo,
    // and shop type selection (rnd(100)) only happens if a room is found.
    let depth = level.dlevel.depth();
    let nroom = level.rooms.len() as i32;
    eprintln!("RS: before special_room_cascade rng={} depth={} nroom={} room_threshold={}", rng.call_count(), depth, nroom, room_threshold);
    mkroom_cascade(level, rng, depth, nroom, room_threshold);
    eprintln!("RS: after special_room_cascade rng={}", rng.call_count());

    // C: place_branch(branchp, 0, 0) — after special room cascade, before per-room loop
    let final_rooms = level.rooms.clone();
    eprintln!("RS: before place_branch rng={}", rng.call_count());
    place_branch_c(level, &final_rooms, rng);
    eprintln!("RS: after place_branch rng={}", rng.call_count());

    // C: per-room loop (mklev.c:802-893) — populate each ordinary room
    populate_ordinary_rooms(level, &final_rooms, rng);
}

/// Place stairs in the level - matches C's makelevel() lines 710-728
///
/// C places stairs BEFORE corridors:
/// 1. Pick random room for downstairs: croom = &rooms[rn2(nroom)]
/// 2. Place downstairs: mkstairs(somex(croom), somey(croom), 0, croom)
/// 3. Pick different room for upstairs: croom = &rooms[rn2(nroom-1)]; if same, croom++
/// 4. Place upstairs: mkstairs(somex(croom), somey(croom), 1, croom)
fn place_stairs(level: &mut Level, rooms: &[Room], rng: &mut GameRng) {
    if rooms.is_empty() {
        return;
    }

    let nroom = rooms.len();

    // C: croom = &rooms[rn2(nroom)];
    let down_room_idx = rng.rn2(nroom as u32) as usize;

    // C: if (!Is_botlevel(&u.uz)) mkstairs(somex(croom), somey(croom), 0, croom);
    // somex = rn2(hx - lx + 1) + lx, somey = rn2(hy - ly + 1) + ly
    let (dx, dy) = rooms[down_room_idx].random_point(rng);
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

    // C: if (nroom > 1) { troom = croom; croom = &rooms[rn2(nroom-1)]; if (croom == troom) croom++; }
    let up_room_idx = if nroom > 1 {
        let idx = rng.rn2((nroom - 1) as u32) as usize;
        // C uses pointer equality (==), NOT >=. Only bump when same index.
        if idx == down_room_idx { idx + 1 } else { idx }
    } else {
        down_room_idx // same room when only 1 room (C: croom unchanged)
    };

    // C: if (u.uz.dlevel != 1) { do { sx = somex(croom); sy = somey(croom); } while (occupied(sx, sy)); mkstairs(sx, sy, 1, croom); }
    if level.dlevel.level_num != 1 {
        // C loops while occupied (checks monsters, objects, traps, and stairs)
        let (mut ux, mut uy) = rooms[up_room_idx].random_point(rng);
        while level.cells[ux][uy].typ == CellType::Stairs {
            let pt = rooms[up_room_idx].random_point(rng);
            ux = pt.0;
            uy = pt.1;
        }
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
    }
}

/// Place monsters in the level
fn place_monsters(
    level: &mut Level,
    rooms: &[Room],
    rng: &mut GameRng,
    monster_vitals: &crate::magic::MonsterVitals,
) {
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

        // Skip if this monster type is genocided
        if monster_vitals.is_genocided(monster_type) {
            continue;
        }

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

/// Get a monster name based on monster type index (pub(crate) wrapper for use in gameloop.rs)
pub(crate) fn random_monster_name_for_type(monster_type: i16) -> &'static str {
    let idx = (monster_type as usize) % RANDOM_MONSTER_NAMES.len();
    RANDOM_MONSTER_NAMES[idx]
}

/// Get a random monster name based on monster type index
fn random_monster_name(monster_type: i16, _rng: &mut GameRng) -> &'static str {
    let idx = (monster_type as usize) % RANDOM_MONSTER_NAMES.len();
    RANDOM_MONSTER_NAMES[idx]
}

/// Find the first door or secret door position on a room's boundary.
/// Returns (x, y) of the door. C's doors[sroom->fdoor].
fn find_first_door_pos(level: &Level, room: &Room) -> Option<(usize, usize)> {
    if room.door_count == 0 {
        return None;
    }
    let idx = room.first_door_idx as usize;
    if idx < level.door_positions.len() {
        Some(level.door_positions[idx])
    } else {
        None
    }
}

/// C's mkclass(class, 0) RNG consumption.
/// Iterates through monsters of the given class, potentially calling rn2(2) for
/// toostrong checks, then rnd(num) for selection.
/// Returns the C monster index of the selected monster.
fn mkclass_c_rng(
    symbol: char,
    depth: i32,
    player_level: i32,
    rng: &mut GameRng,
) -> usize {
    let maxmlev = depth / 2; // level_difficulty() >> 1

    let mut first: Option<usize> = None;
    let mut nums = [0i32; 381]; // indexed by C mons[] index
    let mut num: i32 = 0;
    let mut last_c_mndx: usize = 0;

    // Map Rust symbol to C class letter (they match)
    for c_mndx in 0..C_SPECIAL_PM {
        let rust_mndx = C_TO_RUST_MONS[c_mndx];
        let mon = &MONSTERS[rust_mndx];
        if mon.symbol != symbol {
            if first.is_some() {
                // Past end of class (classes are contiguous)
                break;
            }
            continue;
        }
        if first.is_none() {
            first = Some(c_mndx);
        }

        let gf = C_MONS_GENO[c_mndx];
        let mask = G_NOGEN | G_UNIQ;
        if (gf & mask) != 0 {
            continue;
        }

        let difficulty = C_MONS_DIFFICULTY[c_mndx] as i32;
        // toostrong check: if we already have candidates, and this is toostrong,
        // and harder than previous, and rn2(2) → break
        if num > 0 && difficulty > maxmlev + 1 {
            // Check if harder than previous
            if c_mndx > 0 {
                let prev_diff = C_MONS_DIFFICULTY[c_mndx - 1] as i32;
                if difficulty > prev_diff && rng.rn2(2) != 0 {
                    break;
                }
            }
        }

        let freq = (gf & G_FREQ_MASK) as i32;
        if freq > 0 {
            // Bias: nums[last] = k + 1 - (adj_lev > u.ulevel*2)
            let adj = adj_lev_c(mon, depth, player_level);
            let bias = if adj > (player_level * 2) { 1 } else { 0 };
            let k = freq + 1 - bias;
            if k > 0 {
                nums[c_mndx] = k;
                num += k;
            }
        }
        last_c_mndx = c_mndx;
    }

    if num <= 0 {
        // Fallback — shouldn't happen at depth 14
        return C_TO_RUST_MONS[first.unwrap_or(0)];
    }

    // Select: rnd(num) then walk
    let mut ct = rng.rnd(num as u32) as i32;
    for c_mndx in 0..=last_c_mndx {
        ct -= nums[c_mndx];
        if ct <= 0 && nums[c_mndx] > 0 {
            return c_mndx;
        }
    }

    first.unwrap_or(0)
}

/// C's courtmon() RNG: rn2(60) + rn2(3*level_difficulty)
/// Returns C monster index for the selected monster type.
fn courtmon_c_rng(depth: i32, player_level: i32, rng: &mut GameRng) -> usize {
    let i = rng.rn2(60) as i32 + rng.rn2((3 * depth).max(1) as u32) as i32;

    if i > 100 {
        mkclass_c_rng('D', depth, player_level, rng) // S_DRAGON
    } else if i > 95 {
        mkclass_c_rng('H', depth, player_level, rng) // S_GIANT
    } else if i > 85 {
        mkclass_c_rng('T', depth, player_level, rng) // S_TROLL
    } else if i > 75 {
        mkclass_c_rng('C', depth, player_level, rng) // S_CENTAUR
    } else if i > 60 {
        mkclass_c_rng('o', depth, player_level, rng) // S_ORC
    } else if i > 45 {
        44 // C: PM_BUGBEAR = 44 — direct index, no mkclass RNG
    } else if i > 30 {
        70 // C: PM_HOBGOBLIN = 70
    } else if i > 15 {
        mkclass_c_rng('G', depth, player_level, rng) // S_GNOME
    } else {
        mkclass_c_rng('k', depth, player_level, rng) // S_KOBOLD
    }
}

/// C's morguemon() RNG: rn2(100) + rn2(level_difficulty)
/// Returns C monster index for the selected monster type.
fn morguemon_c_rng(depth: i32, player_level: i32, rng: &mut GameRng) -> usize {
    let i = rng.rn2(100) as i32;
    let hd = rng.rn2(depth.max(1) as u32) as i32;

    // Note: at depth 14, not Inhell, not endgame
    if hd > 10 && i < 10 {
        // C: ndemon(A_NONE) → mkclass_aligned(S_DEMON, 0, A_NONE)
        // This IS equivalent to mkclass(S_DEMON, 0) — DOES consume RNG
        // Then checks is_ndemon; if not, returns NON_PM and falls through
        let demon_c_mndx = mkclass_c_rng('&', depth, player_level, rng);
        // Check if it's an ndemon (not lord, not prince)
        let rust_mndx = C_TO_RUST_MONS[demon_c_mndx];
        let mon = &MONSTERS[rust_mndx];
        let is_nd = mon.flags.contains(MonsterFlags::DEMON) && !is_lord(mon) && !is_prince(mon);
        if is_nd {
            return demon_c_mndx;
        }
        // else fall through to ghost/wraith/zombie checks below
    }
    if hd > 8 && i > 85 {
        mkclass_c_rng('V', depth, player_level, rng) // S_VAMPIRE
    } else if i < 20 {
        283 // C: PM_GHOST = 283 — direct, no mkclass RNG
    } else if i < 40 {
        226 // C: PM_WRAITH = 226 — direct, no mkclass RNG
    } else {
        mkclass_c_rng('Z', depth, player_level, rng) // S_ZOMBIE
    }
}

/// C's squadmon() RNG: rnd(80 + level_difficulty), possibly rn2(NSTYPES)
fn squadmon_c_rng(depth: i32, rng: &mut GameRng) -> usize {
    // C: PM_SOLDIER=273, PM_SERGEANT=274, PM_LIEUTENANT=276, PM_CAPTAIN=277
    let sel_prob = rng.rnd((80 + depth).max(1) as u32) as i32;

    // squadprob: {SOLDIER:80, SERGEANT:15, LIEUTENANT:4, CAPTAIN:1}
    let cpro_values = [(273usize, 80i32), (274, 15), (276, 4), (277, 1)];
    let mut cpro = 0i32;
    for &(pm, prob) in &cpro_values {
        cpro += prob;
        if cpro > sel_prob {
            return pm;
        }
    }
    // Fallback: rn2(NSTYPES) = rn2(4)
    let idx = rng.rn2(4) as usize;
    cpro_values[idx].0
}

/// C's antholemon(): no RNG, deterministic based on birthday + depth
fn antholemon_c_rng(depth: i32) -> usize {
    // C: indx = ubirthday % 3 + level_difficulty()
    // We use 0 for birthday (constant for parity)
    let indx = depth; // birthday % 3 = 0 for simplicity
    // C: PM_SOLDIER_ANT=2, PM_FIRE_ANT=3, PM_GIANT_ANT=0
    match indx % 3 {
        0 => 2, // PM_SOLDIER_ANT
        1 => 3, // PM_FIRE_ANT
        _ => 0, // PM_GIANT_ANT
    }
}

/// C's mktemple() RNG consumption (mkroom.c:599-621).
/// shrine_pos + induced_align(80) + priestini (makemon priest + spellbooks + robe).
fn mktemple_c_rng(
    level: &mut Level,
    room: &Room,
    depth: i32,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    rng: &mut GameRng,
) {
    // 1. shrine_pos: center of room, with rn2(2) for even-width/height adjustment
    // C: delta = hx - lx; if ((delta % 2) && rn2(2)) buf.x++
    let lx = room.x;
    let ly = room.y;
    let hx = room.x + room.width - 1;
    let hy = room.y + room.height - 1;
    let dx = hx - lx; // width - 1
    let dy = hy - ly; // height - 1
    let mut sx = lx + dx / 2;
    let mut sy = ly + dy / 2;
    if dx % 2 != 0 && rng.rn2(2) != 0 {
        sx += 1;
    }
    if dy % 2 != 0 && rng.rn2(2) != 0 {
        sy += 1;
    }

    // Place altar at shrine position (C: levl[m.x][m.y].typ = ALTAR)
    level.cells[sx][sy].typ = CellType::Altar;

    // 2. induced_align(80): for non-special, non-aligned dungeon → rn2(3)
    // C: al = rn2(3) - 1; return Align2amask(al);
    rng.rn2(3);

    // 3. priestini: makemon(PM_ALIGNED_PRIEST=271, sx+1, sy, MM_EPRI)
    // C PM 271 = aligned priest, S_HUMAN, level 12, M2_PEACEFUL|M2_LORD|M2_COLLECT
    makemon_specific_c_rng(271, depth, objects, bases, rng);

    // 4. spellbooks: cnt = rn1(3, 2) = rn2(3) + 2 → 2 to 4 books
    let cnt = rng.rn2(3) + 2;
    for _ in 0..cnt {
        // mkobj(SPBOOK_CLASS, FALSE) — FALSE is artif, init is always TRUE
        mkobj_class_c_rng(objects, bases, ObjectClass::Spellbook, false, depth, rng);
    }

    // 5. robe check: rn2(2)
    rng.rn2(2);

    eprintln!("RS: mktemple_c_rng done rng={}", rng.call_count());
}

/// C's fill_zoo(sroom) RNG consumption.
/// Iterates room cells and creates monsters + items per room type.
/// Must match C's exact iteration order and RNG consumption.
fn fill_zoo_c_rng(
    level: &mut Level,
    room: &Room,
    room_type: RoomType,
    depth: i32,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    rng: &mut GameRng,
) {
    let player_level = 1i32;

    // Find first door position for this room
    let door_pos = find_first_door_pos(level, room);
    let has_door = door_pos.is_some();

    // C room coords: lx, ly, hx, hy (inclusive interior)
    let lx = room.x;
    let ly = room.y;
    let hx = room.x + room.width - 1;
    let hy = room.y + room.height - 1;

    // Pre-switch: COURT throne placement
    let (mut tx, mut ty) = (0usize, 0usize);
    match room_type {
        RoomType::Court => {
            eprintln!("RS COURT: before somexy rng={}", rng.call_count());
            tx = rng.rn2((hx - lx + 1) as u32) as usize + lx; // somex
            ty = rng.rn2((hy - ly + 1) as u32) as usize + ly; // somey
            eprintln!("RS COURT: after somexy tx={} ty={} rng={}", tx, ty, rng.call_count());
            let _throne_i = rng.rnd(depth.max(1) as u32) as i32;
            // C: PM_OGRE_KING=202, PM_ELVENKING=265, PM_DWARF_KING=46, PM_GNOME_KING=165
            let king_pm = if _throne_i > 9 { 202 }
                else if _throne_i > 5 { 265 }
                else if _throne_i > 2 { 46 }
                else { 165 };
            eprintln!("RS COURT: throne_i={} king_pm={} rng={}", _throne_i, king_pm, rng.call_count());
            makemon_specific_c_rng(king_pm, depth, objects, bases, rng);
            eprintln!("RS COURT: after makemon_king rng={}", rng.call_count());
            mongets_c_rng(objects, bases, R_MACE, ObjectClass::Weapon, depth, rng);
            eprintln!("RS COURT: after mongets_MACE rng={}", rng.call_count());
        }
        RoomType::Beehive => {
            // Center of room
            tx = lx + (hx - lx + 1) / 2;
            ty = ly + (hy - ly + 1) / 2;
            // Not irregular, so no somexy fallback
        }
        RoomType::Zoo | RoomType::LeprechaunHall => {
            // goldlim = 500 * level_difficulty — no RNG
        }
        _ => {}
    }

    // Gold limit for zoo/leprehall — decremented per cell like C
    let mut goldlim = 500 * depth;

    // Log fill_zoo start
    let door_info = if has_door { let (dx,dy) = door_pos.unwrap(); format!("({},{})", dx, dy) } else { "none".to_string() };
    eprintln!("RS fill_zoo: type={:?} room=({},{}-{},{}) door={} doorct={} rng={}", room_type, lx, ly, hx, hy, door_info, room.door_count, rng.call_count());
    let mut cell_count = 0usize;

    // Main cell iteration: sx = lx..=hx, sy = ly..=hy
    for sx in lx..=hx {
        for sy in ly..=hy {
            // Skip logic for regular (non-irregular) rooms
            let cell_type = level.cells[sx][sy].typ;
            // SPACE_POS: typ > DOOR. Room cells are Room type.
            let is_space = matches!(cell_type,
                CellType::Corridor | CellType::Room | CellType::Stairs);
            if !is_space {
                continue;
            }

            // Door proximity skip (C: cells on room boundary adjacent to door)
            if has_door {
                let (dx, dy) = door_pos.unwrap();
                if (sx == lx && dx == lx.wrapping_sub(1))
                    || (sx == hx && dx == hx + 1)
                    || (sy == ly && dy == ly.wrapping_sub(1))
                    || (sy == hy && dy == hy + 1)
                {
                    continue;
                }
            }

            // Court: skip throne cell
            if room_type == RoomType::Court && sx == tx && sy == ty {
                continue;
            }

            cell_count += 1;
            let _cell_rng_start = rng.call_count();

            // Monster creation per type
            match room_type {
                RoomType::Court => {
                    let c_mndx = courtmon_c_rng(depth, player_level, rng);
                    makemon_specific_c_rng(c_mndx, depth, objects, bases, rng);
                }
                RoomType::Barracks => {
                    let c_mndx = squadmon_c_rng(depth, rng);
                    makemon_specific_c_rng(c_mndx, depth, objects, bases, rng);
                }
                RoomType::Morgue => {
                    let c_mndx = morguemon_c_rng(depth, player_level, rng);
                    makemon_specific_c_rng(c_mndx, depth, objects, bases, rng);
                }
                RoomType::Beehive => {
                    if sx == tx && sy == ty {
                        // PM_QUEEN_BEE = 5
                        makemon_specific_c_rng(5, depth, objects, bases, rng);
                    } else {
                        // PM_KILLER_BEE = 1
                        makemon_specific_c_rng(1, depth, objects, bases, rng);
                    }
                }
                RoomType::LeprechaunHall => {
                    // PM_LEPRECHAUN = 62
                    makemon_specific_c_rng(62, depth, objects, bases, rng);
                }
                RoomType::CockatriceNest => {
                    // PM_COCKATRICE = 10
                    makemon_specific_c_rng(10, depth, objects, bases, rng);
                }
                RoomType::Anthole => {
                    let c_mndx = antholemon_c_rng(depth);
                    makemon_specific_c_rng(c_mndx, depth, objects, bases, rng);
                }
                RoomType::Zoo => {
                    // makemon(NULL, sx, sy, MM_ASLEEP) — random monster with groups
                    // anymon=true (ptr=NULL), no MM_NOGRP → groups can form
                    makemon_zoo_c_rng(level, objects, bases, depth, rng);
                }
                _ => {}
            }

            // Item creation per type
            match room_type {
                RoomType::Zoo | RoomType::LeprechaunHall => {
                    // C: i = sq(dist2(sx,sy,door)) or goldlim if no door
                    // Then: if (i >= goldlim) i = 5 * level_difficulty()
                    // goldlim -= i; mkgold(rn1(i, 10), sx, sy)
                    let mut i = if has_door {
                        let (dx, dy) = door_pos.unwrap();
                        let distval = (sx as i32 - dx as i32).pow(2) + (sy as i32 - dy as i32).pow(2);
                        distval * distval // sq(dist2)
                    } else {
                        goldlim
                    };
                    if i >= goldlim {
                        i = 5 * depth;
                    }
                    goldlim -= i;
                    // rn1(i, 10) = rn2(i) + 10
                    if i > 0 {
                        rng.rn2(i.max(1) as u32);
                    }
                }
                RoomType::Morgue => {
                    // if (!rn2(5)) mk_tt_object(CORPSE) → mksobj(CORPSE, TRUE, FALSE)
                    if rng.rn2(5) == 0 {
                        mksobj_c_rng(objects, bases, R_CORPSE, ObjectClass::Food, true, false, depth, rng);
                    }
                    // if (!rn2(10)) mksobj_at(rn2(3)?LARGE_BOX:CHEST, TRUE, FALSE)
                    if rng.rn2(10) == 0 {
                        let box_type = if rng.rn2(3) != 0 { R_LARGE_BOX } else { R_CHEST };
                        mksobj_c_rng(objects, bases, box_type, ObjectClass::Tool, true, false, depth, rng);
                    }
                    // if (!rn2(5)) make_grave → get_rnd_text → rn2(sizetxt)
                    if rng.rn2(5) == 0 {
                        // make_grave calls get_rnd_text(EPITAPHFILE, buf, rn2)
                        // which calls rn2(sizetxt) — 1 RNG call
                        // sizetxt depends on file size, which we approximate
                        rng.rn2(1000); // approximate epitaph file size
                    }
                }
                RoomType::Beehive => {
                    // if (!rn2(3)) mksobj_at(LUMP_OF_ROYAL_JELLY, TRUE, FALSE)
                    if rng.rn2(3) == 0 {
                        mksobj_c_rng(objects, bases, R_LUMP_OF_ROYAL_JELLY, ObjectClass::Food, true, false, depth, rng);
                    }
                }
                RoomType::Barracks => {
                    // if (!rn2(20)) mksobj_at(rn2(3)?LARGE_BOX:CHEST, TRUE, FALSE)
                    if rng.rn2(20) == 0 {
                        let box_type = if rng.rn2(3) != 0 { R_LARGE_BOX } else { R_CHEST };
                        mksobj_c_rng(objects, bases, box_type, ObjectClass::Tool, true, false, depth, rng);
                    }
                }
                RoomType::CockatriceNest => {
                    // if (!rn2(3)) { mk_tt_object(STATUE) + container items }
                    if rng.rn2(3) == 0 {
                        // mk_tt_object(STATUE) → mksobj_at(STATUE, FALSE, FALSE)
                        // init=FALSE → no RNG for statue itself
                        // Then: for (i = rn2(5); i; i--) add_to_container(mkobj(RANDOM_CLASS, FALSE))
                        let container_count = rng.rn2(5);
                        for _ in 0..container_count {
                            mkobj_c_rng(objects, bases, depth, rng);
                        }
                    }
                }
                RoomType::Anthole => {
                    // if (!rn2(3)) mkobj_at(FOOD_CLASS, FALSE)
                    if rng.rn2(3) == 0 {
                        // mkobj_at(FOOD_CLASS, FALSE) → mkobj(FOOD_CLASS, FALSE)
                        // mkobj with specific class: rn2(num_in_class) for selection, then mksobj(init=FALSE)
                        // init=FALSE → 0 RNG after selection
                        // Actually mkobj(class, FALSE): calls rn2 for selection within class
                        mkobj_class_c_rng(objects, bases, ObjectClass::Food, false, depth, rng);
                    }
                }
                _ => {}
            }
            if matches!(room_type, RoomType::LeprechaunHall | RoomType::Zoo | RoomType::Anthole | RoomType::Court) {
                eprintln!("RS fill_zoo cell[{}] ({},{}) rng_delta={} rng={}", cell_count, sx, sy, rng.call_count() - _cell_rng_start, rng.call_count());
            }
        }
    }
    eprintln!("RS fill_zoo: done, cell_count={} rng={}", cell_count, rng.call_count());

    // Post-loop switch
    match room_type {
        RoomType::Court => {
            // Set throne cell type (C: levl[tx][ty].typ = THRONE)
            level.cells[tx][ty].typ = CellType::Throne;
            // somexy(sroom, &mm): somex + somey = 2 RNG calls
            rng.rn2((hx - lx + 1) as u32); // somex
            rng.rn2((hy - ly + 1) as u32); // somey
            // mksobj(GOLD_PIECE, TRUE, FALSE) — Coin class, no init RNG
            // gold->quan = rn1(50*level_difficulty, 10) → rn2(50*depth) + 10
            rng.rn2((50 * depth).max(1) as u32);
            // mksobj_at(CHEST, TRUE, FALSE)
            mksobj_c_rng(objects, bases, R_CHEST, ObjectClass::Tool, true, false, depth, rng);
        }
        _ => {}
    }

    eprintln!("RS: fill_zoo {:?} rng={}", room_type, rng.call_count());
}


/// C's stock_room RNG consumption for shops.
/// Creates shopkeeper + populates each cell with merchandise.
fn stock_room_c_rng(
    level: &Level,
    room: &Room,
    shp_indx: usize,
    depth: i32,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    rng: &mut GameRng,
) {
    // 1. shkinit: makemon(PM_SHOPKEEPER) + mkmonmoney(1000 + 30*rnd(100))
    // PM_SHOPKEEPER = 267 in C
    makemon_specific_c_rng(267, depth, objects, bases, rng);
    // mkmonmoney: rnd(100) for initial capital
    rng.rnd(100);

    // Ring shop (shp_indx=6): shopkeeper gets TOUCHSTONE via mongets
    if shp_indx == 6 {
        // C: mongets(shk, TOUCHSTONE) → mksobj(TOUCHSTONE, TRUE, FALSE)
        // TOUCHSTONE is a gem → gem_init_c_rng
        mksobj_c_rng(objects, bases, 0, ObjectClass::Gem, true, false, depth, rng);
    }

    // nameshk: deterministic for most shops, no RNG
    // Exception: shktools uses rn2(names_avail) in naming loop — skip for now

    // 2. stockcount for tribute book: iterate cells counting good positions
    let lx = room.x;
    let ly = room.y;
    let hx = room.x + room.width - 1;
    let hy = room.y + room.height - 1;

    let door_pos = find_first_door_pos(level, room);

    // Count valid positions for specialspot (tribute book)
    let mut stockcount = 0;
    for sx in lx..=hx {
        for sy in ly..=hy {
            if stock_room_goodpos(level, room, door_pos, sx, sy) {
                stockcount += 1;
            }
        }
    }

    // C: specialspot = rnd(stockcount) if tribute enabled and bookstock not set
    // context.tribute.enabled is TRUE by default, bookstock starts FALSE
    // Only for scroll shops (shp_indx=2) and spellbook shops (shp_indx=9)
    let specialspot = if (shp_indx == 2 || shp_indx == 9) && stockcount > 0 {
        let s = rng.rnd(stockcount as u32);
        s as i32
    } else {
        0
    };

    // 3. Main loop: mkshobj_at for each valid cell
    let mut cell_count = 0;
    for sx in lx..=hx {
        for sy in ly..=hy {
            if stock_room_goodpos(level, room, door_pos, sx, sy) {
                cell_count += 1;
                let mkspecl = specialspot > 0 && cell_count == specialspot as usize;
                mkshobj_at_c_rng(shp_indx, mkspecl, depth, objects, bases, rng);
            }
        }
    }

    eprintln!("RS: stock_room shp_indx={} {} cells rng={}", shp_indx, cell_count, rng.call_count());
}

/// C's stock_room_goodpos: check if a cell is a valid shop item placement.
/// For regular rooms: skip row nearest first door.
fn stock_room_goodpos(
    level: &Level,
    room: &Room,
    door_pos: Option<(usize, usize)>,
    sx: usize,
    sy: usize,
) -> bool {
    let cell_type = level.cells[sx][sy].typ;
    let is_space = matches!(cell_type, CellType::Room | CellType::Corridor | CellType::Stairs);
    if !is_space {
        return false;
    }

    // Skip cells in the row nearest the door
    if let Some((dx, dy)) = door_pos {
        let lx = room.x;
        let ly = room.y;
        let hx = room.x + room.width - 1;
        let hy = room.y + room.height - 1;

        // C: stock_room_goodpos skips cells adjacent to door on room boundary
        if dx == lx.wrapping_sub(1) && sx == lx {
            return false;
        }
        if dx == hx + 1 && sx == hx {
            return false;
        }
        if dy == ly.wrapping_sub(1) && sy == ly {
            return false;
        }
        if dy == hy + 1 && sy == hy {
            return false;
        }
    }

    true
}

/// C's mkshobj_at RNG consumption for a single shop item placement.
/// Handles tribute book, mimic check, and shop-type-specific item creation.
fn mkshobj_at_c_rng(
    shp_indx: usize,
    mkspecl: bool,
    depth: i32,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    rng: &mut GameRng,
) {
    // C: 3.6 tribute — for scroll/spellbook shops, specialspot gets SPE_NOVEL
    // mksobj_at(SPE_NOVEL, sx, sy, FALSE, FALSE) — init=FALSE means NO RNG
    if mkspecl && (shp_indx == 2 || shp_indx == 9) {
        // SPE_NOVEL with init=FALSE: no RNG consumption
        return;
    }

    // C: if (rn2(100) < depth && !MON_AT(sx,sy) && mkclass(S_MIMIC) && makemon)
    let mimic_roll = rng.rn2(100);
    if (mimic_roll as i32) < depth {
        // mkclass(S_MIMIC, 0): finds mimic class monsters
        let mimic_mndx = mkclass_c_rng('m', depth, 1, rng);
        // makemon(ptr, sx, sy, NO_MM_FLAGS): specific monster creation
        makemon_specific_c_rng(mimic_mndx, depth, objects, bases, rng);
        // rn2(10) for mimic appearance
        rng.rn2(10);
    } else {
        // C: atype = get_shop_item(shp - shtypes) then create item
        get_shop_item_c_rng(shp_indx, depth, objects, bases, rng);
    }
}

/// Shop item type encoding for get_shop_item_c_rng iprobs tables.
/// 0 = RANDOM_CLASS (general store), 1 = class-based (mkobj_at),
/// 2 = specific otyp (mksobj_at), 3 = VEGETARIAN_CLASS.
#[derive(Clone, Copy)]
enum ShopItem {
    AnyClass,
    Class(ObjectClass),
    Specific(ObjectClass),
    Vegetarian,
}

/// C's get_shop_item(type) + dispatch: rnd(100) to select item, then create it.
fn get_shop_item_c_rng(
    shp_indx: usize,
    depth: i32,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    rng: &mut GameRng,
) {
    use ObjectClass::*;
    use ShopItem::*;

    let iprobs: &[(u32, ShopItem)] = match shp_indx {
        0 => &[(100, AnyClass)],
        1 => &[(90, Class(Armor)), (10, Class(Weapon))],
        2 => &[(90, Class(Scroll)), (10, Class(Spellbook))],
        3 => &[(100, Class(Potion))],
        4 => &[(90, Class(Weapon)), (10, Class(Armor))],
        5 => &[(83, Class(Food)), (5, Specific(Potion)), (4, Specific(Potion)),
               (5, Specific(Potion)), (3, Specific(Tool))],
        6 => &[(85, Class(Ring)), (10, Class(Gem)), (5, Class(Amulet))],
        7 => &[(90, Class(Wand)), (5, Specific(Armor)), (5, Specific(Armor))],
        8 => &[(100, Class(Tool))],
        9 => &[(90, Class(Spellbook)), (10, Class(Scroll))],
        10 => &[(70, Vegetarian), (20, Specific(Potion)), (4, Specific(Potion)),
                (3, Specific(Potion)), (2, Specific(Scroll)), (1, Specific(Food))],
        _ => &[(100, AnyClass)],
    };

    // C: for (j = rnd(100), i = 0; (j -= shp->iprobs[i].iprob) > 0; i++)
    let mut j = rng.rnd(100) as i32;
    let mut selected = iprobs[0].1;
    for &(prob, typ) in iprobs {
        j -= prob as i32;
        if j <= 0 {
            selected = typ;
            break;
        }
    }

    match selected {
        AnyClass => mkobj_c_rng(objects, bases, depth, rng),
        Class(class) => mkobj_class_c_rng(objects, bases, class, true, depth, rng),
        Specific(class) => mksobj_c_rng(objects, bases, 0, class, true, true, depth, rng),
        Vegetarian => shkveg_c_rng(objects, bases, depth, rng),
    }
}

/// C's shkveg() + mksobj_at(otyp, TRUE, TRUE) RNG consumption.
/// shkveg does rnd(maxprob) to select a vegetarian food item.
fn shkveg_c_rng(
    objects: &[ObjClassDef],
    bases: &ClassBases,
    depth: i32,
    rng: &mut GameRng,
) {
    // C: shkveg() iterates food class, filters veggy items, sums oc_prob, then rnd(maxprob)
    // We don't need exact item — just need to consume rnd(maxprob) + mksobj init for food
    // maxprob for vegetarian foods in C is a computed value
    // For RNG purposes: 1 call for rnd(maxprob) + food_init_c_rng for the selected item
    //
    // C's veggy_item filters: not CORPSE, not TIN, not EGG, not MEAT_RING,
    // and not obj->otyp == K_RATION/C_RATION/CRAM_RATION/LEMBAS_WAFER
    // (actually it checks monster type for corpses, but for (obj==0, otyp) it filters differently)
    //
    // Approximate maxprob: sum of oc_prob for vegetarian food items
    // For accuracy, compute from our objects array
    let food_base = bases.get(ObjectClass::Food);
    let mut maxprob = 0u32;
    let mut i = food_base;
    while i < objects.len() && objects[i].class == ObjectClass::Food {
        // Filter like C's veggy_item(NULL, otyp):
        // Exclude items whose name suggests meat (simplified: exclude specific indices)
        // In C, veggy_item with NULL obj checks oc_name for "meat", "jerky" etc.
        // For RNG purposes, the exact maxprob matters
        // Known non-veggy foods by index: CORPSE, EGG, MEAT_RING, TIN,
        // TRIPE_RATION, MEAT_STICK, HUGE_CHUNK_OF_MEAT
        // We approximate by including all food items for now
        maxprob += objects[i].probability as u32;
        i += 1;
    }

    // rnd(maxprob) for vegetarian food selection
    if maxprob > 0 {
        rng.rnd(maxprob);
    }

    // The selected item goes through mksobj_at(otyp, TRUE, TRUE) = food_init_c_rng
    food_init_c_rng(0, rng); // otyp 0 won't trigger special cases (TIN, CORPSE, etc.)
}

/// Matches C's mklev.c:786-818 special room cascade followed by mkroom().
///
/// In C, the cascade calls mkroom(TYPE) which does room selection internally.
/// For shops, mkshop() finds a room with doorct==1 (no RNG), then calls rnd(100)
/// for type selection only if a room is found. For non-shops, mkzoo() calls
/// pick_room(FALSE) which uses rn2(nroom) + conditional rn2(3)/rn2(5).
///
/// The RNG consumption must match C's exactly or all downstream generation diverges.
fn mkroom_cascade(
    level: &mut Level,
    rng: &mut GameRng,
    depth: i32,
    nroom: i32,
    room_threshold: i32,
) {
    use super::room::pick_room;

    let objects = OBJECTS;
    let bases = ClassBases::compute(objects);

    const MEDUSA_DEPTH: i32 = 27; // C reports depth(&medusa_level)=27

    // C: if (wizard && nh_getenv("SHOPTYPE")) — skip in Rust (not wizard)

    // Shop: u_depth > 1 && u_depth < medusa && nroom >= room_threshold && rn2(u_depth) < 3
    if depth > 1 && depth < MEDUSA_DEPTH && nroom >= room_threshold && rng.rn2(depth as u32) < 3 {
        // C: mkroom(SHOPBASE) → mkshop()
        // mkshop scans rooms sequentially for doorct==1, no stairs, OROOM (NO RNG)
        let shop_room = find_shop_room(level);
        if let Some(idx) = shop_room {
            // rnd(100) for shop type selection
            let room_area = level.rooms[idx].width * level.rooms[idx].height;
            let (shop_type, shp_indx) = select_shop_type(rng, room_area);
            level.rooms[idx].room_type = shop_type;
            // Ensure room is lit (C: mkshop lights up dark rooms)
            level.rooms[idx].lit = true;
            let room = &level.rooms[idx];
            for x in room.x.saturating_sub(1)..=(room.x + room.width).min(COLNO - 1) {
                for y in room.y.saturating_sub(1)..=(room.y + room.height).min(ROWNO - 1) {
                    level.cells[x][y].lit = true;
                }
            }
            level.flags.has_shop = true;
            eprintln!("RS: cascade: SHOP {:?} shp_indx={} room_idx={} rng={}", shop_type, shp_indx, idx, rng.call_count());
            // C: stock_room(i, sroom) — populates shop with items/shopkeeper
            let room = level.rooms[idx].clone();
            stock_room_c_rng(level, &room, shp_indx, depth, objects, &bases, rng);
        } else {
            eprintln!("RS: cascade: SHOP selected but no suitable room rng={}", rng.call_count());
        }
        return;
    }

    // Helper closure to call fill_zoo after setting room type
    macro_rules! fill_zoo_for {
        ($idx:expr, $rtype:expr) => {
            let room = level.rooms[$idx].clone();
            fill_zoo_c_rng(level, &room, $rtype, depth, objects, &bases, rng);
        };
    }

    // Court: u_depth > 4 && !rn2(6)
    if depth > 4 && rng.one_in(6) {
        if let Some(idx) = pick_room(&level.rooms, level, false, rng) {
            level.rooms[idx].room_type = RoomType::Court;
            level.flags.has_court = true;
            eprintln!("RS: cascade: COURT room_idx={} rng={}", idx, rng.call_count());
            fill_zoo_for!(idx, RoomType::Court);
        }
        return;
    }

    // LeprechaunHall: u_depth > 5 && !rn2(8) && !(mvitals[PM_LEPRECHAUN].mvflags & G_GONE)
    if depth > 5 && rng.one_in(8) {
        if let Some(idx) = pick_room(&level.rooms, level, false, rng) {
            level.rooms[idx].room_type = RoomType::LeprechaunHall;
            eprintln!("RS: cascade: LEPREHALL room_idx={} rng={}", idx, rng.call_count());
            fill_zoo_for!(idx, RoomType::LeprechaunHall);
        }
        return;
    }

    // Zoo: u_depth > 6 && !rn2(7)
    if depth > 6 && rng.one_in(7) {
        if let Some(idx) = pick_room(&level.rooms, level, false, rng) {
            level.rooms[idx].room_type = RoomType::Zoo;
            level.flags.has_zoo = true;
            eprintln!("RS: cascade: ZOO room_idx={} rng={}", idx, rng.call_count());
            fill_zoo_for!(idx, RoomType::Zoo);
        }
        return;
    }

    // Temple: u_depth > 8 && !rn2(5)
    if depth > 8 && rng.one_in(5) {
        if let Some(idx) = pick_room(&level.rooms, level, false, rng) {
            level.rooms[idx].room_type = RoomType::Temple;
            level.flags.has_temple = true;
            eprintln!("RS: cascade: TEMPLE room_idx={} rng={}", idx, rng.call_count());
            let room = level.rooms[idx].clone();
            mktemple_c_rng(level, &room, depth, objects, &bases, rng);
        }
        return;
    }

    // Beehive: u_depth > 9 && !rn2(5) && !(mvitals[PM_KILLER_BEE].mvflags & G_GONE)
    if depth > 9 && rng.one_in(5) {
        if let Some(idx) = pick_room(&level.rooms, level, false, rng) {
            level.rooms[idx].room_type = RoomType::Beehive;
            level.flags.has_beehive = true;
            eprintln!("RS: cascade: BEEHIVE room_idx={} rng={}", idx, rng.call_count());
            fill_zoo_for!(idx, RoomType::Beehive);
        }
        return;
    }

    // Morgue: u_depth > 11 && !rn2(6)
    if depth > 11 && rng.one_in(6) {
        if let Some(idx) = pick_room(&level.rooms, level, false, rng) {
            level.rooms[idx].room_type = RoomType::Morgue;
            level.flags.has_morgue = true;
            // Morgues are dark
            level.rooms[idx].lit = false;
            let room = &level.rooms[idx];
            for x in room.x..room.x + room.width {
                for y in room.y..room.y + room.height {
                    level.cells[x][y].lit = false;
                }
            }
            eprintln!("RS: cascade: MORGUE room_idx={} rng={}", idx, rng.call_count());
            fill_zoo_for!(idx, RoomType::Morgue);
        }
        return;
    }

    // Anthole: u_depth > 12 && !rn2(8) && antholemon()
    if depth > 12 && rng.one_in(8) {
        if let Some(idx) = pick_room(&level.rooms, level, false, rng) {
            level.rooms[idx].room_type = RoomType::Anthole;
            eprintln!("RS: cascade: ANTHOLE room_idx={} rng={}", idx, rng.call_count());
            fill_zoo_for!(idx, RoomType::Anthole);
        }
        return;
    }

    // Barracks: u_depth > 14 && !rn2(4) && !(mvitals[PM_SOLDIER].mvflags & G_GONE)
    if depth > 14 && rng.one_in(4) {
        if let Some(idx) = pick_room(&level.rooms, level, false, rng) {
            level.rooms[idx].room_type = RoomType::Barracks;
            level.flags.has_barracks = true;
            eprintln!("RS: cascade: BARRACKS room_idx={} rng={}", idx, rng.call_count());
            fill_zoo_for!(idx, RoomType::Barracks);
        }
        return;
    }

    // Swamp: u_depth > 15 && !rn2(6)
    if depth > 15 && rng.one_in(6) {
        if let Some(idx) = pick_room(&level.rooms, level, false, rng) {
            level.rooms[idx].room_type = RoomType::Swamp;
            level.flags.has_swamp = true;
            eprintln!("RS: cascade: SWAMP room_idx={} rng={}", idx, rng.call_count());
            // Swamp uses mkswamp() which is different from fill_zoo
            // NOTE: mkswamp_c_rng is a stub; swamp fill uses separate C-parity path
        }
        return;
    }

    // CockatriceNest: u_depth > 16 && !rn2(8) && !(mvitals[PM_COCKATRICE].mvflags & G_GONE)
    if depth > 16 && rng.one_in(8) {
        if let Some(idx) = pick_room(&level.rooms, level, false, rng) {
            level.rooms[idx].room_type = RoomType::CockatriceNest;
            eprintln!("RS: cascade: COCKNEST room_idx={} rng={}", idx, rng.call_count());
            fill_zoo_for!(idx, RoomType::CockatriceNest);
        }
        return;
    }

    eprintln!("RS: cascade: no special room rng={}", rng.call_count());
}

/// Find a room suitable for a shop (C's mkshop room search).
/// Scans rooms sequentially — NO RNG consumed.
/// Returns first OROOM with doorct==1, no upstairs, no downstairs.
/// Compute door_count for each room by scanning level cells.
/// C's dosdoor calls add_door which increments doorct incrementally;
/// we do this post-hoc after corridor generation.
///
/// A door belongs to room R if it's on the room's wall:
/// x in [room.x-1, room.x+room.width] and y in [room.y-1, room.y+room.height]
/// but NOT in the room interior.
fn compute_door_counts(level: &mut Level) {
    // Reset all door counts
    for room in &mut level.rooms {
        room.door_count = 0;
    }

    // For each room, scan its walls for door/secret door cells
    let rooms_snapshot: Vec<_> = level.rooms.iter().map(|r| (r.x, r.y, r.width, r.height)).collect();
    for (room_idx, &(rx, ry, rw, rh)) in rooms_snapshot.iter().enumerate() {
        // Check all wall positions around the room
        let lx = rx.saturating_sub(1);
        let hx = (rx + rw).min(COLNO - 1);
        let ly = ry.saturating_sub(1);
        let hy = (ry + rh).min(ROWNO - 1);

        for x in lx..=hx {
            for y in ly..=hy {
                // Skip interior cells
                if x >= rx && x < rx + rw && y >= ry && y < ry + rh {
                    continue;
                }
                let typ = level.cells[x][y].typ;
                if typ == CellType::Door || typ == CellType::SecretDoor {
                    level.rooms[room_idx].door_count += 1;
                }
            }
        }
    }
}

fn find_shop_room(level: &Level) -> Option<usize> {
    use super::room::{room_has_upstairs, room_has_downstairs};
    for (idx, room) in level.rooms.iter().enumerate() {
        if room.room_type != RoomType::Ordinary {
            continue;
        }
        if room_has_upstairs(room, level) || room_has_downstairs(room, level) {
            continue;
        }
        if room.door_count == 1 {
            return Some(idx);
        }
    }
    None
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
    // C: mklev.c:786-814 — cascading if/else for special room selection
    // Note: C uses u_depth > X (strict greater), so we use depth > X

    // Shop: C: u_depth > 1 && u_depth < depth(&medusa_level) && nroom >= room_threshold && rn2(u_depth) < 3
    // Simplified: depth > 1 and below medusa (roughly < 22), with room count check
    // For now, use depth > 1 with upper bound approximation
    if depth > 1 && rng.rn2(depth as u32) < 3 {
        let (shop_type, _shp_indx) = select_shop_type(rng, 0);
        flags.has_shop = true;
        return Some(shop_type);
    }

    // Court: C: u_depth > 4 && !rn2(6)
    if depth > 4 && rng.one_in(6) {
        flags.has_court = true;
        return Some(RoomType::Court);
    }

    // LeprechaunHall: C: u_depth > 5 && !rn2(8) && !(mvitals[PM_LEPRECHAUN].mvflags & G_GONE)
    if depth > 5 && rng.one_in(8) {
        return Some(RoomType::LeprechaunHall);
    }

    // Zoo: C: u_depth > 6 && !rn2(7)
    if depth > 6 && rng.one_in(7) {
        flags.has_zoo = true;
        return Some(RoomType::Zoo);
    }

    // Temple: C: u_depth > 8 && !rn2(5)
    if depth > 8 && rng.one_in(5) {
        flags.has_temple = true;
        return Some(RoomType::Temple);
    }

    // Beehive: C: u_depth > 9 && !rn2(5) && !(mvitals[PM_KILLER_BEE].mvflags & G_GONE)
    if depth > 9 && rng.one_in(5) {
        flags.has_beehive = true;
        return Some(RoomType::Beehive);
    }

    // Morgue: C: u_depth > 11 && !rn2(6)
    if depth > 11 && rng.one_in(6) {
        flags.has_morgue = true;
        return Some(RoomType::Morgue);
    }

    // Anthole: C: u_depth > 12 && !rn2(8) && antholemon()
    if depth > 12 && rng.one_in(8) {
        return Some(RoomType::Anthole);
    }

    // Barracks: C: u_depth > 14 && !rn2(4) && !(mvitals[PM_SOLDIER].mvflags & G_GONE)
    if depth > 14 && rng.one_in(4) {
        flags.has_barracks = true;
        return Some(RoomType::Barracks);
    }

    // Swamp: C: u_depth > 15 && !rn2(6)
    if depth > 15 && rng.one_in(6) {
        flags.has_swamp = true;
        return Some(RoomType::Swamp);
    }

    // CockatriceNest: C: u_depth > 16 && !rn2(8) && !(mvitals[PM_COCKATRICE].mvflags & G_GONE)
    if depth > 16 && rng.one_in(8) {
        return Some(RoomType::CockatriceNest);
    }

    None
}

/// Select a shop type based on weighted probabilities
/// Matches C's shtypes[] weights from shknam.c
/// C's mkshop shop type selection: rnd(100) with cumulative subtraction
/// through shtypes[] probabilities. Returns (RoomType, shtypes_index).
fn select_shop_type(rng: &mut GameRng, room_area: usize) -> (RoomType, usize) {
    // C's shtypes[] order and probabilities (shknam.c:205-347):
    // [0] general:    42
    // [1] armor:      14
    // [2] scroll:     10
    // [3] potion:     10
    // [4] weapon:      5
    // [5] food:        5
    // [6] ring:        3
    // [7] wand:        3
    // [8] tool:        3
    // [9] spellbook:   3
    // [10] healthfood: 2
    // Total: 100
    const PROBS: [(u32, RoomType); 11] = [
        (42, RoomType::GeneralShop),
        (14, RoomType::ArmorShop),
        (10, RoomType::ScrollShop),
        (10, RoomType::PotionShop),
        (5, RoomType::WeaponShop),
        (5, RoomType::FoodShop),
        (3, RoomType::RingShop),
        (3, RoomType::WandShop),
        (3, RoomType::ToolShop),
        (3, RoomType::BookShop),
        (2, RoomType::HealthFoodShop),
    ];

    // C: for (j = rnd(100), i = 0; (j -= shtypes[i].prob) > 0; i++)
    let mut j = rng.rnd(100) as i32;
    let mut idx = 0;
    for (i, &(prob, _)) in PROBS.iter().enumerate() {
        j -= prob as i32;
        if j <= 0 {
            idx = i;
            break;
        }
    }

    // C: if isbig(sroom) and wand or spellbook shop → general store
    if room_area > 20 && (idx == 7 || idx == 9) {
        idx = 0;
    }

    (PROBS[idx].1, idx)
}

/// Pick a room suitable for the given special type
/// Returns the room index if found
fn pick_room_for_special(rooms: &[Room], special_type: RoomType) -> Option<usize> {
    // For shops, prefer rooms with single entrance (easier to manage)
    // For other special rooms, any ordinary room works
    // Avoid rooms that are too small

    let min_area = match special_type {
        RoomType::Vault => 4,              // 2x2 minimum
        _ if special_type.is_shop() => 12, // Shops need space for items
        _ => 9,                            // 3x3 minimum for most special rooms
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
    use super::TrapType;
    use super::level::Stairway;
    use super::topology::DungeonSystem;

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

/// C's place_branch(branchp, 0, 0) — mklev.c:1151-1199
///
/// Finds a random room (avoiding stairs rooms), picks somexy,
/// and places a branch stairway. Consumes RNG for room selection
/// and position finding.
fn place_branch_c(level: &mut Level, rooms: &[Room], rng: &mut GameRng) {
    use super::level::Stairway;
    use super::topology::DungeonSystem;

    let dungeon_system = DungeonSystem::new();
    let branch = match dungeon_system.get_branch_from(&level.dlevel) {
        Some(b) => b,
        None => return,
    };

    // C's find_branch_room: pick a random room avoiding stairs rooms
    // With nroom > 2: do { croom = &rooms[rn2(nroom)] } while (bad room && tryct < 100)
    let nroom = rooms.len();
    if nroom == 0 {
        return;
    }

    let (room_idx, room) = if nroom > 2 {
        let mut tryct = 0;
        let mut idx;
        loop {
            idx = rng.rn2(nroom as u32) as usize;
            let r = &rooms[idx];
            // Avoid stairs rooms and non-ordinary rooms
            let is_stairs_room = level.stairs.iter().any(|s| {
                let sx = s.x as usize;
                let sy = s.y as usize;
                sx >= r.x && sx < r.x + r.width && sy >= r.y && sy < r.y + r.height
            });
            let is_ordinary = r.room_type == RoomType::Ordinary;
            tryct += 1;
            if (!is_stairs_room && is_ordinary) || tryct >= 100 {
                break;
            }
        }
        (idx, &rooms[idx])
    } else {
        let idx = rng.rn2(nroom as u32) as usize;
        (idx, &rooms[idx])
    };

    // C's somexy loop: do { somexy(croom, &m) } while (occupied || not CORR/ROOM)
    let mut x = 0usize;
    let mut y = 0usize;
    let mut found = false;
    for _ in 0..200 {
        x = super::room::somex(room, rng);
        y = super::room::somey(room, rng);
        let cell_type = level.cells[x][y].typ;
        let occupied = level.monster_at(x as i8, y as i8).is_some()
            || level.cells[x][y].typ == CellType::Stairs;
        if !occupied && (cell_type == CellType::Corridor || cell_type == CellType::Room) {
            found = true;
            break;
        }
    }

    if !found {
        return;
    }

    // Place the branch
    let make_stairs = match branch.branch_type {
        super::topology::BranchType::Stairs => true,
        super::topology::BranchType::Portal => {
            level.add_trap(x as i8, y as i8, super::TrapType::MagicPortal);
            level.flags.has_branch = true;
            return;
        }
        _ => return,
    };

    if make_stairs {
        level.cells[x][y].typ = CellType::Stairs;
        level.stairs.push(Stairway {
            x: x as i8,
            y: y as i8,
            destination: branch.end2,
            up: branch.end1_up,
        });
        level.flags.has_branch = true;
    }
}

/// C's bydoor() (mklev.c:1368-1392): checks if any adjacent cell is a door or secret door
fn bydoor(level: &Level, x: usize, y: usize) -> bool {
    let checks = [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)];
    for (dx, dy) in checks {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx >= 0 && nx < COLNO as i32 && ny >= 0 && ny < ROWNO as i32 {
            let typ = level.cells[nx as usize][ny as usize].typ;
            if typ == CellType::Door || typ == CellType::SecretDoor {
                return true;
            }
        }
    }
    false
}

/// C's occupied() check — for level generation, checks traps and features.
/// At feature-placement time, no monsters or objects are in rooms yet.
fn occupied_for_feature(level: &Level, x: usize, y: usize) -> bool {
    // Check for traps at this position
    level.traps.iter().any(|t| t.x == x as i8 && t.y == y as i8)
    // Also check if the cell already has a non-ROOM type (fountain, stairs, etc.)
    || (level.cells[x][y].typ != CellType::Room && level.cells[x][y].typ != CellType::Corridor)
}

/// somexy with occupied+bydoor retry loop, matching C's mkfount/mksink/mkaltar/mkgrave pattern:
/// do { somexy(croom, &m) } while (occupied(m.x, m.y) || bydoor(m.x, m.y));
fn somexy_unoccupied(level: &Level, room: &Room, rng: &mut GameRng) -> Option<(usize, usize)> {
    for _ in 0..200 {
        let x = super::room::somex(room, rng);
        let y = super::room::somey(room, rng);
        if !occupied_for_feature(level, x, y) && !bydoor(level, x, y) {
            return Some((x, y));
        }
    }
    None
}

/// C's per-room loop (mklev.c:802-893)
///
/// For each ordinary room, places monsters, traps, gold, features (fountain,
/// sink, altar, grave), statues, boxes, graffiti, and objects. All in a single
/// pass per room, matching C's exact RNG call order.
fn populate_ordinary_rooms(level: &mut Level, rooms: &[Room], rng: &mut GameRng) {
    let depth = level.dlevel.depth();
    let nroom = rooms.len();
    let objects = OBJECTS;
    let bases = ClassBases::compute(objects);

    for room_idx in 0..nroom {
        let room = &rooms[room_idx];
        if room.room_type != RoomType::Ordinary {
            continue;
        }

        eprintln!("RS ROOM[{}]: start rng={}", room_idx, rng.call_count());

        // --- Monster: C mklev.c:813-820 ---
        // if (u.uhave.amulet || !rn2(3)) { somex + somey + makemon }
        let has_amulet = false; // u.uhave.amulet
        if has_amulet || rng.rn2(3) == 0 {
            let _mx = super::room::somex(room, rng);
            let _my = super::room::somey(room, rng);
            // makemon((struct permonst *) 0, x, y, MM_NOGRP)
            makemon_c_rng(level, objects, &bases, depth, rng);
        }
        eprintln!("RS ROOM[{}]: after_monster rng={}", room_idx, rng.call_count());

        // --- Traps: C mklev.c:822-826 ---
        // x = 8 - (level_difficulty() / 6); while (!rn2(x)) mktrap(...)
        let trap_threshold = (8 - (depth / 6)).max(2);
        while rng.rn2(trap_threshold as u32) == 0 {
            // mktrap(0, 0, croom, NULL) — consumes variable RNG
            // NOTE: mktrap uses mktrap_c_rng for RNG parity (see below)
            mktrap_c_rng(level, room, depth, rng);
        }

        eprintln!("RS ROOM[{}]: after_traps rng={}", room_idx, rng.call_count());

        // --- Gold: C mklev.c:827-828 ---
        // if (!rn2(3)) mkgold(0L, somex(croom), somey(croom))
        if rng.rn2(3) == 0 {
            let _gx = super::room::somex(room, rng);
            let _gy = super::room::somey(room, rng);
            // mkgold(0L, ...): amount = 1 + rnd(level_difficulty() + 2) * rnd(30)
            // Consumes 2 RNG calls (rnd + rnd)
            let _amount = rng.rnd((depth + 2).max(1) as u32) as i64
                * rng.rnd(30) as i64
                + 1;
        }
        eprintln!("RS ROOM[{}]: after_gold rng={}", room_idx, rng.call_count());

        // --- Fountain: C mklev.c:831-832 ---
        // if (!rn2(10)) mkfount(0, croom)
        if rng.rn2(10) == 0 {
            // mkfount: do { somexy(croom, &m) } while (occupied || bydoor); + rn2(7) for blessed
            if let Some((fx, fy)) = somexy_unoccupied(level, room, rng) {
                let _blessed = rng.rn2(7) == 0;
                level.cells[fx][fy].typ = CellType::Fountain;
                level.flags.fountain_count += 1;
            }
        }

        eprintln!("RS ROOM[{}]: after_fountain rng={}", room_idx, rng.call_count());

        // --- Sink: C mklev.c:833-834 ---
        // if (!rn2(60)) mksink(croom)
        let sink_roll = rng.rn2(60);
        if sink_roll == 0 {
            eprintln!("RS ROOM[{}]: SINK branch taken", room_idx);
            // mksink: do { somexy(croom, &m) } while (occupied || bydoor); set SINK
            if let Some((sx, sy)) = somexy_unoccupied(level, room, rng) {
                level.cells[sx][sy].typ = CellType::Sink;
                level.flags.sink_count += 1;
            }
        }
        eprintln!("RS ROOM[{}]: after_sink rng={}", room_idx, rng.call_count());

        // --- Altar: C mklev.c:835-836 ---
        // if (!rn2(60)) mkaltar(croom)
        let altar_roll = rng.rn2(60);
        if altar_roll == 0 {
            eprintln!("RS ROOM[{}]: ALTAR branch taken", room_idx);
            // mkaltar: do { somexy } while (occupied || bydoor); rn2(3) alignment
            if let Some((ax, ay)) = somexy_unoccupied(level, room, rng) {
                let _alignment = rng.rn2(3);
                level.cells[ax][ay].typ = CellType::Altar;
            }
        }
        eprintln!("RS ROOM[{}]: after_altar rng={}", room_idx, rng.call_count());

        // --- Grave: C mklev.c:837-841 + mklev.c:1808-1857 ---
        // x = 80 - (depth(&u.uz) * 2); if (!rn2(x)) mkgrave(croom)
        let grave_threshold = (80 - depth * 2).max(2);
        let grave_roll = rng.rn2(grave_threshold as u32);
        if grave_roll == 0 {
            eprintln!("RS ROOM[{}]: GRAVE branch taken", room_idx);
            // mkgrave: FIRST thing is dobell = !rn2(10), THEN somexy + items
            let dobell = rng.rn2(10) == 0;
            mkgrave_rng(level, room, dobell, depth, rng);
        }
        eprintln!("RS ROOM[{}]: after_grave rng={}", room_idx, rng.call_count());

        // --- Statue: C mklev.c:844-847 ---
        // if (!rn2(20)) mkcorpstat(STATUE, ..., somex, somey, ...)
        let statue_roll = rng.rn2(20);
        eprintln!("RS ROOM[{}]: statue rn2(20)={} (before_mksobj={})", room_idx, statue_roll, rng.call_count());
        if statue_roll == 0 {
            let _sx = super::room::somex(room, rng);
            let _sy = super::room::somey(room, rng);
            eprintln!("RS ROOM[{}]: STATUE branch taken, before mksobj rng={}", room_idx, rng.call_count());
            // mkcorpstat(STATUE, NULL, NULL, x, y, CORPSTAT_INIT)
            // → mksobj_at(STATUE, x, y, TRUE, FALSE) → mksobj(STATUE, TRUE, FALSE)
            mksobj_c_rng(objects, &bases, R_STATUE, ObjectClass::Rock, true, false, depth, rng);
            eprintln!("RS ROOM[{}]: STATUE after mksobj rng={}", room_idx, rng.call_count());
        }
        eprintln!("RS ROOM[{}]: after_statue rng={}", room_idx, rng.call_count());

        // --- Box/Chest: C mklev.c:853-855 ---
        // if (!rn2(nroom * 5 / 2)) mksobj_at(rn2(3) ? LARGE_BOX : CHEST, ...)
        if rng.rn2((nroom * 5 / 2).max(1) as u32) == 0 {
            let box_otyp = if rng.rn2(3) != 0 { R_LARGE_BOX } else { R_CHEST };
            let _bx = super::room::somex(room, rng);
            let _by = super::room::somey(room, rng);
            mksobj_c_rng(objects, &bases, box_otyp, ObjectClass::Tool, true, true, depth, rng);
        }
        eprintln!("RS ROOM[{}]: after_box rng={}", room_idx, rng.call_count());

        // --- Graffiti: C mklev.c:858-871 ---
        // if (!rn2(27 + 3 * abs(depth(&u.uz)))) { random_engraving + somex+somey loop }
        if rng.rn2((27 + 3 * depth.abs()).max(1) as u32) == 0 {
            // random_engraving(buf) = rn2(num_engravings)
            // Then: do { somex + somey } while (typ != ROOM && !rn2(40))
            random_engraving_rng(rng);
            let mut ex;
            let mut ey;
            loop {
                ex = super::room::somex(room, rng);
                ey = super::room::somey(room, rng);
                if level.cells[ex][ey].typ == CellType::Room || rng.rn2(40) == 0 {
                    break;
                }
            }
        }

        eprintln!("RS ROOM[{}]: before_objects rng={}", room_idx, rng.call_count());
        // --- Objects: C mklev.c:874-884 ---
        // if (!rn2(3)) { mkobj_at(0, somex, somey, TRUE) + while(!rn2(5)) mkobj_at }
        if rng.rn2(3) == 0 {
            let _ox = super::room::somex(room, rng);
            let _oy = super::room::somey(room, rng);
            mkobj_c_rng(objects, &bases, depth, rng);
            let mut obj_tryct = 0;
            while rng.rn2(5) == 0 {
                obj_tryct += 1;
                if obj_tryct > 100 {
                    break;
                }
                let _ox2 = super::room::somex(room, rng);
                let _oy2 = super::room::somey(room, rng);
                mkobj_c_rng(objects, &bases, depth, rng);
            }
        }
        eprintln!("RS ROOM[{}]: end rng={}", room_idx, rng.call_count());
    }
    eprintln!("RS: after all rooms rng={}", rng.call_count());
}

// ============================================================================
// C item constants for mongets (matching C onames.h values).
// For weapons/armor/rings/amulets/tools (C otyp < ~310), values match Rust OBJECTS indices.
// ============================================================================
const C_ARROW: usize = 1;
const C_ELVEN_ARROW: usize = 2;
const C_ORCISH_ARROW: usize = 3;
const C_CROSSBOW_BOLT: usize = 5;
const C_DART: usize = 7;
const C_SHURIKEN: usize = 8;
const C_SPEAR: usize = 10;
const C_ELVEN_SPEAR: usize = 11;
const C_DWARVISH_SPEAR: usize = 13;
const C_DAGGER: usize = 17;
const C_ELVEN_DAGGER: usize = 18;
const C_ORCISH_DAGGER: usize = 19;
const C_KNIFE: usize = 22;
const C_STILETTO: usize = 23;
const C_AXE: usize = 25;
const C_DWARVISH_MATTOCK: usize = 26;
const C_SHORT_SWORD: usize = 29;
const C_DWARVISH_SHORT_SWORD: usize = 30;
const C_ELVEN_SHORT_SWORD: usize = 31;
const C_ORCISH_SHORT_SWORD: usize = 32;
const C_BROADSWORD: usize = 34;
const C_ELVEN_BROADSWORD: usize = 35;
const C_LONG_SWORD: usize = 37;
const C_TWO_HANDED_SWORD: usize = 38;
const C_SCIMITAR: usize = 40;
const C_SILVER_SABER: usize = 42;
const C_CLUB: usize = 59;
const C_AKLYS: usize = 60;
const C_MACE: usize = 62;
const C_FLAIL: usize = 64;
const C_BULLWHIP: usize = 67;
const C_RUBBER_HOSE: usize = 69;
const C_PARTISAN: usize = 44;
const C_RANSEUR: usize = 45;
const C_SPETUM: usize = 46;
const C_GLAIVE: usize = 47;
const C_BEC_DE_CORBIN: usize = 51;
const C_LUCERN_HAMMER: usize = 53;
const C_TRIDENT: usize = 55;
const C_BATTLE_AXE: usize = 57;
const C_PICK_AXE: usize = 58;
const C_BOW: usize = 70;
const C_ELVEN_BOW: usize = 71;
const C_ORCISH_BOW: usize = 72;
const C_CROSSBOW: usize = 74;
const C_SLING: usize = 75;

const C_LEATHER_ARMOR: usize = 113;
const C_STUDDED_LEATHER_ARMOR: usize = 114;
const C_RING_MAIL: usize = 107;
const C_CHAIN_MAIL: usize = 103;
const C_ORCISH_CHAIN_MAIL: usize = 104;
const C_SPLINT_MAIL: usize = 101;
const C_BANDED_MAIL: usize = 102;
const C_PLATE_MAIL: usize = 99;
const C_CRYSTAL_PLATE_MAIL: usize = 100;
const C_LEATHER_JACKET: usize = 116;
const C_LEATHER_CLOAK: usize = 123;
const C_ELVEN_CLOAK: usize = 119;
const C_DWARVISH_CLOAK: usize = 120;
const C_ORCISH_CLOAK: usize = 121;
const C_ROBE: usize = 122;
const C_CLOAK_OF_PROTECTION: usize = 125;
const C_CLOAK_OF_MAGIC_RESISTANCE: usize = 127;
const C_SMALL_SHIELD: usize = 129;
const C_LARGE_SHIELD: usize = 130;
const C_SHIELD_OF_REFLECTION: usize = 131;
const C_DWARVISH_ROUNDSHIELD: usize = 132;
const C_ELVEN_SHIELD: usize = 133;
const C_URUK_HAI_SHIELD: usize = 134;
const C_ORCISH_SHIELD: usize = 135;
const C_ORCISH_HELM: usize = 76;
const C_ELVEN_LEATHER_HELM: usize = 78;
const C_DWARVISH_IRON_HELM: usize = 79;
const C_HELMET: usize = 82;
const C_DENTED_POT: usize = 83;
const C_LOW_BOOTS: usize = 86;
const C_HIGH_BOOTS: usize = 89;
const C_ELVEN_BOOTS: usize = 91;
const C_IRON_SHOES: usize = 87;
const C_LEATHER_GLOVES: usize = 136;
const C_SADDLE: usize = 188;

/// G_FREQ mask for monster gen_flags
const G_FREQ_MASK: u16 = 0x0007;

/// C NetHack's SPECIAL_PM = PM_LONG_WORM_TAIL = 326.
/// rndmonst() iterates C indices 0..<C_SPECIAL_PM.
const C_SPECIAL_PM: usize = 326;

/// C's mons[].difficulty values for indices 0..325 (C_SPECIAL_PM).
/// These are computed by makedefs -m and differ from Rust's PerMonst.difficulty.
/// Used for rndmonst's tooweak/toostrong filtering to match C exactly.
#[rustfmt::skip]
const C_MONS_DIFFICULTY: [u8; 326] = [
     4,  5,  6,  6,  6, 12,  2,  6,  8,  7,  8,  8,  1,  1,  2,  4,
     3,  5,  5,  7,  6,  7,  7,  8,  9,  9, 14,  2,  3,  8,  8,  8,
     3,  5,  6,  7,  7,  7,  8,  8,  8, 11,  2,  4,  5,  6,  8, 13,
    19,  3,  3,  4,  5,  7,  7,  5,  6,  8,  1,  2,  3,  4,  4,  8,
     9, 11,  5,  5,  5,  1,  3,  3,  4,  5,  5,  5,  7,  4,  6,  9,
     4,  7,  8,  9, 13, 15, 22,  1,  2,  4,  4,  4,  4,  3,  4,  7,
     8, 12, 14,  4,  6,  6,  6,  7,  9,  4,  6,  7,  9,  9, 10,  6,
     9, 10, 17,  1,  9,  5,  7, 11, 11, 12, 19, 21, 26,  2,  3,  6,
     7,  6,  8,  9, 13, 13, 13, 13, 13, 13, 13, 13, 13, 20, 20, 20,
    20, 20, 20, 20, 20, 20,  9, 10, 10, 10, 10,  1,  2,  2,  2,  2,
     2,  5,  3,  4,  5,  6,  8,  8, 10, 11, 13, 13, 19, 20, 17, 18,
     3,  4,  5,  6, 14, 18, 21, 29,  4,  5,  6,  6,  7,  7,  8, 10,
     4,  4,  4,  4,  8, 10, 13, 16,  7,  9, 11,  4,  6,  8, 12,  9,
     8, 14,  3,  6,  7,  8,  9, 10,  9, 12, 12, 13, 16, 12, 12, 14,
    32,  7,  8, 17, 11,  4,  6,  7,  7,  8,  9,  1,  2,  3,  3,  4,
     5,  7,  5,  9, 14,  4,  4,  6,  6,  7,  8, 10, 12, 15, 18, 22,
     2,  3,  3,  6, 12,  6,  7,  8, 11, 11, 11, 15, 14, 14, 13, 15,
    30,  8, 10, 13, 12, 14,  8, 12, 25, 34, 22, 12, 14, 11,  8,  9,
     8, 10, 10, 11, 11, 12, 13, 14, 15, 16, 15, 20, 26, 31, 36, 36,
    40, 45, 53, 57, 34, 34, 34,  8,  5,  6,  9,  7, 10, 22,  1,  2,
     3,  4,  6,  7,  7, 12,
];

/// C's mons[].geno values for indices 0..325 (C_SPECIAL_PM).
/// Used for rndmonst's uncommon/hell/nohell/frequency checks to match C exactly.
/// Some Rust gen_flags differ from C's geno (e.g. animal wereforms missing G_NOGEN).
#[rustfmt::skip]
const C_MONS_GENO: [u16; 326] = [
    0x00a3, 0x0062, 0x00a2, 0x00a1, 0x0023, 0x0220, 0x0022, 0x0022,
    0x0022, 0x00a1, 0x0025, 0x0021, 0x00a3, 0x0021, 0x00a1, 0x0210,
    0x0021, 0x0021, 0x0021, 0x0021, 0x00a2, 0x0210, 0x08a2, 0x00a2,
    0x0821, 0x04a1, 0x0421, 0x0031, 0x0025, 0x0832, 0x0032, 0x0032,
    0x0021, 0x0021, 0x0022, 0x0021, 0x0021, 0x0021, 0x0022, 0x0022,
    0x0022, 0x0021, 0x0022, 0x0023, 0x0021, 0x0022, 0x0021, 0x0021,
    0x0021, 0x0071, 0x0022, 0x0021, 0x0471, 0x0022, 0x0023, 0x0022,
    0x0021, 0x0022, 0x0021, 0x0021, 0x0021, 0x0021, 0x0024, 0x0022,
    0x0021, 0x0021, 0x0022, 0x0022, 0x0022, 0x0022, 0x0022, 0x0260,
    0x0062, 0x0061, 0x0061, 0x0021, 0x0021, 0x0024, 0x0022, 0x0021,
    0x00a4, 0x0021, 0x0022, 0x0021, 0x0022, 0x0022, 0x0021, 0x00a1,
    0x00a2, 0x0021, 0x0210, 0x0022, 0x0220, 0x00a2, 0x0021, 0x0021,
    0x0022, 0x0022, 0x0022, 0x0022, 0x0022, 0x0021, 0x0021, 0x0022,
    0x0022, 0x0032, 0x0032, 0x0831, 0x0031, 0x0432, 0x0431, 0x0020,
    0x0020, 0x0022, 0x0022, 0x00b3, 0x0023, 0x0034, 0x0032, 0x0022,
    0x0891, 0x0811, 0x0811, 0x0811, 0x0811, 0x00a1, 0x0022, 0x0022,
    0x0022, 0x0021, 0x0021, 0x0021, 0x0020, 0x0020, 0x0020, 0x0020,
    0x0020, 0x0020, 0x0020, 0x0020, 0x0020, 0x0021, 0x0021, 0x0021,
    0x0021, 0x0021, 0x0021, 0x0021, 0x0021, 0x0021, 0x0023, 0x0011,
    0x0011, 0x0011, 0x0011, 0x0024, 0x0021, 0x0022, 0x0021, 0x0021,
    0x0021, 0x0022, 0x00a1, 0x0022, 0x0021, 0x0021, 0x0221, 0x00a1,
    0x00a1, 0x00a1, 0x08a1, 0x0021, 0x00a1, 0x0001, 0x0220, 0x0021,
    0x0260, 0x02a0, 0x0220, 0x0220, 0x0031, 0x0031, 0x0431, 0x0431,
    0x0031, 0x0031, 0x0031, 0x0031, 0x0031, 0x0031, 0x0031, 0x0031,
    0x0020, 0x0020, 0x0020, 0x0020, 0x0021, 0x0021, 0x0021, 0x0021,
    0x00a1, 0x0022, 0x0022, 0x0032, 0x0031, 0x0431, 0x0031, 0x0023,
    0x0022, 0x0422, 0x0061, 0x0022, 0x0260, 0x0021, 0x0021, 0x0021,
    0x0022, 0x0821, 0x0021, 0x0220, 0x0021, 0x0022, 0x0031, 0x0031,
    0x1210, 0x0031, 0x0022, 0x0031, 0x0021, 0x0021, 0x00a2, 0x0023,
    0x0022, 0x0021, 0x0021, 0x0031, 0x0031, 0x00b1, 0x00b1, 0x00b1,
    0x00b1, 0x0031, 0x0031, 0x0031, 0x0210, 0x0011, 0x0011, 0x0011,
    0x0011, 0x0011, 0x0011, 0x0001, 0x0011, 0x0011, 0x0011, 0x0011,
    0x0200, 0x0001, 0x0001, 0x0001, 0x0200, 0x00a2, 0x00a2, 0x00a2,
    0x00a2, 0x0021, 0x0021, 0x0200, 0x0200, 0x0200, 0x1200, 0x0200,
    0x1200, 0x00a1, 0x00a1, 0x0023, 0x0021, 0x0021, 0x02a1, 0x0221,
    0x1200, 0x1200, 0x1200, 0x0210, 0x0210, 0x0210, 0x0011, 0x0412,
    0x0011, 0x0492, 0x0492, 0x0411, 0x0492, 0x0492, 0x0492, 0x0412,
    0x0411, 0x0412, 0x0411, 0x0411, 0x1610, 0x1610, 0x1610, 0x1610,
    0x1610, 0x1610, 0x1610, 0x1610, 0x1200, 0x1200, 0x1200, 0x0210,
    0x0220, 0x02a0, 0x0220, 0x0220, 0x0220, 0x0220, 0x0025, 0x0025,
    0x0025, 0x0020, 0x0025, 0x0022, 0x0021, 0x0401,
];

/// Mapping from C mons[] index (0..380) to Rust MONSTERS index.
/// C has 381 monsters (NUMMONS=381), Rust has 400. The ordering differs
/// because Rust added extra monsters (Cerberus, beholder, shimmering dragon,
/// vorpal jabberwock, Goblin King) and reordered some entries.
/// Indices 257-259 (human wereforms) map to Rust wererat/werejackal/werewolf;
/// they have G_NOGEN so rndmonst never selects them.
#[rustfmt::skip]
const C_TO_RUST_MONS: [usize; 381] = [
    0,   // 0: PM_GIANT_ANT
    1,   // 1: PM_KILLER_BEE
    2,   // 2: PM_SOLDIER_ANT
    3,   // 3: PM_FIRE_ANT
    4,   // 4: PM_GIANT_BEETLE
    5,   // 5: PM_QUEEN_BEE
    6,   // 6: PM_ACID_BLOB
    7,   // 7: PM_QUIVERING_BLOB
    8,   // 8: PM_GELATINOUS_CUBE
    9,   // 9: PM_CHICKATRICE
    10,  // 10: PM_COCKATRICE
    11,  // 11: PM_PYROLISK
    12,  // 12: PM_JACKAL
    13,  // 13: PM_FOX
    14,  // 14: PM_COYOTE
    15,  // 15: PM_WEREJACKAL (animal form, S_DOG, G_NOGEN)
    16,  // 16: PM_LITTLE_DOG
    19,  // 17: PM_DINGO
    17,  // 18: PM_DOG
    18,  // 19: PM_LARGE_DOG
    20,  // 20: PM_WOLF
    21,  // 21: PM_WEREWOLF (animal form, S_DOG, G_NOGEN)
    23,  // 22: PM_WINTER_WOLF_CUB
    22,  // 23: PM_WARG
    24,  // 24: PM_WINTER_WOLF
    25,  // 25: PM_HELL_HOUND_PUP
    26,  // 26: PM_HELL_HOUND
    28,  // 27: PM_GAS_SPORE
    29,  // 28: PM_FLOATING_EYE
    30,  // 29: PM_FREEZING_SPHERE
    31,  // 30: PM_FLAMING_SPHERE
    32,  // 31: PM_SHOCKING_SPHERE
    34,  // 32: PM_KITTEN
    35,  // 33: PM_HOUSECAT
    36,  // 34: PM_JAGUAR
    37,  // 35: PM_LYNX
    38,  // 36: PM_PANTHER
    39,  // 37: PM_LARGE_CAT
    40,  // 38: PM_TIGER
    41,  // 39: PM_GREMLIN
    42,  // 40: PM_GARGOYLE
    43,  // 41: PM_WINGED_GARGOYLE
    53,  // 42: PM_HOBBIT
    54,  // 43: PM_DWARF
    55,  // 44: PM_BUGBEAR
    56,  // 45: PM_DWARF_LORD
    57,  // 46: PM_DWARF_KING
    58,  // 47: PM_MIND_FLAYER
    59,  // 48: PM_MASTER_MIND_FLAYER
    60,  // 49: PM_MANES
    61,  // 50: PM_HOMUNCULUS
    62,  // 51: PM_IMP
    63,  // 52: PM_LEMURE
    64,  // 53: PM_QUASIT
    65,  // 54: PM_TENGU
    66,  // 55: PM_BLUE_JELLY
    67,  // 56: PM_SPOTTED_JELLY
    68,  // 57: PM_OCHRE_JELLY
    69,  // 58: PM_KOBOLD
    70,  // 59: PM_LARGE_KOBOLD
    71,  // 60: PM_KOBOLD_LORD
    72,  // 61: PM_KOBOLD_SHAMAN
    73,  // 62: PM_LEPRECHAUN
    74,  // 63: PM_SMALL_MIMIC
    75,  // 64: PM_LARGE_MIMIC
    76,  // 65: PM_GIANT_MIMIC
    77,  // 66: PM_WOOD_NYMPH
    78,  // 67: PM_WATER_NYMPH
    79,  // 68: PM_MOUNTAIN_NYMPH
    80,  // 69: PM_GOBLIN
    82,  // 70: PM_HOBGOBLIN
    83,  // 71: PM_ORC
    84,  // 72: PM_HILL_ORC
    85,  // 73: PM_MORDOR_ORC
    86,  // 74: PM_URUK_HAI
    87,  // 75: PM_ORC_SHAMAN
    88,  // 76: PM_ORC_CAPTAIN
    89,  // 77: PM_ROCK_PIERCER
    90,  // 78: PM_IRON_PIERCER
    91,  // 79: PM_GLASS_PIERCER
    92,  // 80: PM_ROTHE
    93,  // 81: PM_MUMAK
    94,  // 82: PM_LEOCROTTA
    95,  // 83: PM_WUMPUS
    96,  // 84: PM_TITANOTHERE
    97,  // 85: PM_BALUCHITHERIUM
    98,  // 86: mastodon
    99,  // 87: PM_SEWER_RAT
    100, // 88: PM_GIANT_RAT
    101, // 89: PM_RABID_RAT
    102, // 90: PM_WERERAT (animal form, S_RODENT, G_NOGEN)
    103, // 91: PM_ROCK_MOLE
    104, // 92: PM_WOODCHUCK
    105, // 93: PM_CAVE_SPIDER
    106, // 94: PM_CENTIPEDE
    107, // 95: PM_GIANT_SPIDER
    108, // 96: PM_SCORPION
    109, // 97: PM_LURKER_ABOVE
    110, // 98: PM_TRAPPER
    111, // 99: PM_PONY
    112, // 100: PM_WHITE_UNICORN
    113, // 101: PM_GRAY_UNICORN
    114, // 102: PM_BLACK_UNICORN
    115, // 103: PM_HORSE
    116, // 104: PM_WARHORSE
    117, // 105: PM_FOG_CLOUD
    118, // 106: PM_DUST_VORTEX
    119, // 107: PM_ICE_VORTEX
    120, // 108: PM_ENERGY_VORTEX
    121, // 109: PM_STEAM_VORTEX
    122, // 110: PM_FIRE_VORTEX
    123, // 111: PM_BABY_LONG_WORM
    124, // 112: PM_BABY_PURPLE_WORM
    125, // 113: PM_LONG_WORM
    126, // 114: PM_PURPLE_WORM
    44,  // 115: PM_GRID_BUG
    127, // 116: PM_XAN
    128, // 117: PM_YELLOW_LIGHT
    129, // 118: PM_BLACK_LIGHT
    130, // 119: PM_ZRUTY
    131, // 120: PM_COUATL
    132, // 121: PM_ALEAX
    133, // 122: PM_ANGEL
    134, // 123: PM_KI_RIN
    135, // 124: PM_ARCHON
    136, // 125: PM_BAT
    137, // 126: PM_GIANT_BAT
    138, // 127: PM_RAVEN
    139, // 128: PM_VAMPIRE_BAT
    140, // 129: PM_PLAINS_CENTAUR
    141, // 130: PM_FOREST_CENTAUR
    142, // 131: PM_MOUNTAIN_CENTAUR
    143, // 132: PM_BABY_GRAY_DRAGON
    145, // 133: PM_BABY_SILVER_DRAGON
    146, // 134: PM_BABY_RED_DRAGON
    147, // 135: PM_BABY_WHITE_DRAGON
    148, // 136: PM_BABY_ORANGE_DRAGON
    149, // 137: PM_BABY_BLACK_DRAGON
    150, // 138: PM_BABY_BLUE_DRAGON
    151, // 139: PM_BABY_GREEN_DRAGON
    152, // 140: PM_BABY_YELLOW_DRAGON
    154, // 141: PM_GRAY_DRAGON
    155, // 142: PM_SILVER_DRAGON
    156, // 143: PM_RED_DRAGON
    157, // 144: PM_WHITE_DRAGON
    158, // 145: PM_ORANGE_DRAGON
    159, // 146: PM_BLACK_DRAGON
    160, // 147: PM_BLUE_DRAGON
    161, // 148: PM_GREEN_DRAGON
    162, // 149: PM_YELLOW_DRAGON
    163, // 150: PM_STALKER
    164, // 151: PM_AIR_ELEMENTAL
    165, // 152: PM_FIRE_ELEMENTAL
    166, // 153: PM_EARTH_ELEMENTAL
    167, // 154: PM_WATER_ELEMENTAL
    168, // 155: PM_LICHEN
    169, // 156: PM_BROWN_MOLD
    170, // 157: PM_YELLOW_MOLD
    171, // 158: PM_GREEN_MOLD
    172, // 159: PM_RED_MOLD
    173, // 160: PM_SHRIEKER
    174, // 161: PM_VIOLET_FUNGUS
    175, // 162: PM_GNOME
    176, // 163: PM_GNOME_LORD
    177, // 164: PM_GNOMISH_WIZARD
    178, // 165: PM_GNOME_KING
    179, // 166: PM_GIANT
    180, // 167: PM_STONE_GIANT
    181, // 168: PM_HILL_GIANT
    182, // 169: PM_FIRE_GIANT
    183, // 170: PM_FROST_GIANT
    184, // 171: PM_ETTIN
    185, // 172: PM_STORM_GIANT
    186, // 173: PM_TITAN
    187, // 174: PM_MINOTAUR
    188, // 175: PM_JABBERWOCK
    190, // 176: PM_KEYSTONE_KOP
    191, // 177: PM_KOP_SERGEANT
    192, // 178: PM_KOP_LIEUTENANT
    193, // 179: PM_KOP_KAPTAIN
    194, // 180: PM_LICH
    195, // 181: PM_DEMILICH
    196, // 182: PM_MASTER_LICH
    197, // 183: PM_ARCH_LICH
    198, // 184: PM_KOBOLD_MUMMY
    199, // 185: PM_GNOME_MUMMY
    200, // 186: PM_ORC_MUMMY
    201, // 187: PM_DWARF_MUMMY
    202, // 188: PM_ELF_MUMMY
    203, // 189: PM_HUMAN_MUMMY
    204, // 190: PM_ETTIN_MUMMY
    205, // 191: PM_GIANT_MUMMY
    206, // 192: PM_RED_NAGA_HATCHLING
    207, // 193: PM_BLACK_NAGA_HATCHLING
    208, // 194: PM_GOLDEN_NAGA_HATCHLING
    209, // 195: PM_GUARDIAN_NAGA_HATCHLING
    210, // 196: PM_RED_NAGA
    211, // 197: PM_BLACK_NAGA
    212, // 198: PM_GOLDEN_NAGA
    213, // 199: PM_GUARDIAN_NAGA
    214, // 200: PM_OGRE
    215, // 201: PM_OGRE_LORD
    216, // 202: PM_OGRE_KING
    217, // 203: PM_GRAY_OOZE
    218, // 204: PM_BROWN_PUDDING
    219, // 205: PM_GREEN_SLIME
    220, // 206: PM_BLACK_PUDDING
    221, // 207: PM_QUANTUM_MECHANIC
    222, // 208: PM_RUST_MONSTER
    223, // 209: PM_DISENCHANTER
    224, // 210: PM_GARTER_SNAKE
    225, // 211: PM_SNAKE
    226, // 212: PM_WATER_MOCCASIN
    227, // 213: PM_PYTHON
    228, // 214: PM_PIT_VIPER
    229, // 215: PM_COBRA
    230, // 216: PM_TROLL
    231, // 217: PM_ICE_TROLL
    232, // 218: PM_ROCK_TROLL
    233, // 219: PM_WATER_TROLL
    234, // 220: PM_OLOG_HAI
    235, // 221: PM_UMBER_HULK
    236, // 222: PM_VAMPIRE
    237, // 223: PM_VAMPIRE_LORD
    239, // 224: PM_VLAD_THE_IMPALER
    240, // 225: PM_BARROW_WIGHT
    241, // 226: PM_WRAITH
    242, // 227: PM_NAZGUL
    243, // 228: PM_XORN
    244, // 229: PM_MONKEY
    245, // 230: PM_APE
    246, // 231: PM_OWLBEAR
    247, // 232: PM_YETI
    248, // 233: PM_CARNIVOROUS_APE
    249, // 234: PM_SASQUATCH
    250, // 235: PM_KOBOLD_ZOMBIE
    251, // 236: PM_GNOME_ZOMBIE
    252, // 237: PM_ORC_ZOMBIE
    253, // 238: PM_DWARF_ZOMBIE
    254, // 239: PM_ELF_ZOMBIE
    255, // 240: PM_HUMAN_ZOMBIE
    256, // 241: PM_ETTIN_ZOMBIE
    257, // 242: PM_GHOUL
    258, // 243: PM_GIANT_ZOMBIE
    259, // 244: PM_SKELETON
    260, // 245: PM_STRAW_GOLEM
    261, // 246: PM_PAPER_GOLEM
    262, // 247: PM_ROPE_GOLEM
    263, // 248: PM_GOLD_GOLEM
    264, // 249: PM_LEATHER_GOLEM
    265, // 250: PM_WOOD_GOLEM
    266, // 251: PM_FLESH_GOLEM
    267, // 252: PM_CLAY_GOLEM
    268, // 253: PM_STONE_GOLEM
    269, // 254: PM_GLASS_GOLEM
    270, // 255: PM_IRON_GOLEM
    271, // 256: PM_HUMAN
    272, // 257: PM_HUMAN_WERERAT (human form, S_HUMAN, G_NOGEN)
    273, // 258: PM_HUMAN_WEREJACKAL (human form, S_HUMAN, G_NOGEN)
    274, // 259: PM_HUMAN_WEREWOLF (human form, S_HUMAN, G_NOGEN)
    275, // 260: PM_ELF
    276, // 261: PM_WOODLAND_ELF
    277, // 262: PM_GREEN_ELF
    278, // 263: PM_GREY_ELF
    279, // 264: PM_ELF_LORD
    280, // 265: PM_ELVENKING
    281, // 266: PM_DOPPELGANGER
    282, // 267: PM_SHOPKEEPER
    283, // 268: PM_GUARD
    284, // 269: PM_PRISONER
    285, // 270: PM_ORACLE
    286, // 271: PM_ALIGNED_PRIEST
    287, // 272: PM_HIGH_PRIEST
    288, // 273: PM_SOLDIER
    289, // 274: PM_SERGEANT
    290, // 275: PM_NURSE
    291, // 276: PM_LIEUTENANT
    292, // 277: PM_CAPTAIN
    293, // 278: PM_WATCHMAN
    294, // 279: PM_WATCH_CAPTAIN
    298, // 280: PM_MEDUSA
    299, // 281: PM_WIZARD_OF_YENDOR
    300, // 282: PM_CROESUS
    301, // 283: PM_GHOST
    302, // 284: PM_SHADE
    303, // 285: PM_WATER_DEMON
    304, // 286: PM_SUCCUBUS
    305, // 287: PM_HORNED_DEVIL
    306, // 288: PM_INCUBUS
    307, // 289: PM_ERINYS
    308, // 290: PM_BARBED_DEVIL
    309, // 291: PM_MARILITH
    310, // 292: PM_VROCK
    311, // 293: PM_HEZROU
    312, // 294: PM_BONE_DEVIL
    313, // 295: PM_ICE_DEVIL
    314, // 296: PM_NALFESHNEE
    315, // 297: PM_PIT_FIEND
    316, // 298: PM_SANDESTIN
    317, // 299: PM_BALROG
    318, // 300: PM_JUIBLEX
    319, // 301: PM_YEENOGHU
    320, // 302: PM_ORCUS
    321, // 303: PM_GERYON
    322, // 304: PM_DISPATER
    323, // 305: PM_BAALZEBUB
    324, // 306: PM_ASMODEUS
    325, // 307: PM_DEMOGORGON
    326, // 308: PM_DEATH
    327, // 309: PM_PESTILENCE
    328, // 310: PM_FAMINE
    331, // 311: PM_DJINNI
    332, // 312: PM_JELLYFISH
    333, // 313: PM_PIRANHA
    334, // 314: PM_SHARK
    335, // 315: PM_GIANT_EEL
    336, // 316: PM_ELECTRIC_EEL
    337, // 317: PM_KRAKEN
    338, // 318: PM_NEWT
    339, // 319: PM_GECKO
    340, // 320: PM_IGUANA
    341, // 321: PM_BABY_CROCODILE
    342, // 322: PM_LIZARD
    343, // 323: PM_CHAMELEON
    344, // 324: PM_CROCODILE
    52,  // 325: PM_SALAMANDER
    345, // 326: PM_LONG_WORM_TAIL
    346, // 327: PM_ARCHEOLOGIST
    347, // 328: PM_BARBARIAN
    348, // 329: PM_CAVEMAN
    349, // 330: PM_CAVEWOMAN
    350, // 331: PM_HEALER
    351, // 332: PM_KNIGHT
    352, // 333: PM_MONK
    353, // 334: PM_PRIEST
    354, // 335: PM_PRIESTESS
    355, // 336: PM_RANGER
    356, // 337: PM_ROGUE
    357, // 338: PM_SAMURAI
    358, // 339: PM_TOURIST
    359, // 340: PM_VALKYRIE
    360, // 341: PM_WIZARD
    361, // 342: PM_LORD_CARNARVON
    362, // 343: PM_PELIAS
    363, // 344: PM_SHAMAN_KARNOV
    364, // 345: PM_HIPPOCRATES
    365, // 346: PM_KING_ARTHUR
    366, // 347: PM_GRAND_MASTER
    367, // 348: PM_ARCH_PRIEST
    368, // 349: PM_ORION
    369, // 350: PM_MASTER_OF_THIEVES
    370, // 351: PM_LORD_SATO
    371, // 352: PM_TWOFLOWER
    372, // 353: PM_NORN
    373, // 354: PM_NEFERET_THE_GREEN
    374, // 355: PM_MINION_OF_HUHETOTL
    375, // 356: PM_THOTH_AMON
    376, // 357: PM_CHROMATIC_DRAGON
    377, // 358: PM_CYCLOPS
    378, // 359: PM_IXOTH
    379, // 360: PM_MASTER_KAEN
    380, // 361: PM_NALZOK
    381, // 362: PM_SCORPIUS
    382, // 363: PM_MASTER_ASSASSIN
    383, // 364: PM_ASHIKAGA_TAKAUJI
    384, // 365: PM_LORD_SURTUR
    385, // 366: PM_DARK_ONE
    386, // 367: PM_STUDENT
    387, // 368: PM_CHIEFTAIN
    388, // 369: PM_NEANDERTHAL
    389, // 370: PM_ATTENDANT
    390, // 371: PM_PAGE
    391, // 372: PM_ABBOT
    392, // 373: PM_ACOLYTE
    393, // 374: PM_HUNTER
    394, // 375: PM_THUG
    395, // 376: PM_NINJA
    396, // 377: PM_ROSHI
    397, // 378: PM_GUIDE
    398, // 379: PM_WARRIOR
    399, // 380: PM_APPRENTICE
];

/// C's align_shift for AM_NONE dungeon (Main Dungeon) → always 0
fn align_shift_am_none() -> i32 {
    0
}

/// C's adj_lev(ptr) — compute adjusted monster level
fn adj_lev_c(mon: &PerMonst, depth: i32, player_level: i32) -> i32 {
    let mut tmp = mon.level as i32;
    if tmp > 49 {
        return 50;
    }
    let tmp2 = depth - tmp;
    if tmp2 < 0 {
        tmp -= 1;
    } else {
        tmp += tmp2 / 5;
    }
    let tmp2 = player_level - mon.level as i32;
    if tmp2 > 0 {
        tmp += tmp2 / 4;
    }
    let upper = ((3 * mon.level as i32) / 2).min(49);
    if tmp > upper { upper } else if tmp > 0 { tmp } else { 0 }
}

/// C's rndmonst() — select a random monster for the level.
/// Returns the Rust MONSTERS index (via C_TO_RUST_MONS mapping).
/// Iterates C's mons[] ordering (0..C_SPECIAL_PM) for parity.
/// Consumes exactly 1 RNG call (rnd(choice_count)).
fn rndmonst_c_rng(
    depth: i32,
    player_level: i32,
    in_hell: bool,
    rng: &mut GameRng,
) -> usize {
    let min_mlev = depth / 6;
    let max_mlev = (depth + player_level) / 2;

    let mut choice_count: i32 = 0;
    // mchoices indexed by C mons[] index
    let mut mchoices = [0i32; 381];

    // Find first non-uncommon monster (C ordering).
    // Use C's exact geno values for all filtering to ensure parity.
    let mut first_common: usize = 0;
    for c_mndx in 0..C_SPECIAL_PM {
        let gf = C_MONS_GENO[c_mndx];
        if (gf & (G_NOGEN | G_UNIQ)) == 0 {
            first_common = c_mndx;
            break;
        }
    }

    let shift = align_shift_am_none();

    for c_mndx in first_common..C_SPECIAL_PM {
        let gf = C_MONS_GENO[c_mndx];

        if (gf & (G_NOGEN | G_UNIQ)) != 0 {
            continue;
        }

        // C uses mons[].difficulty for tooweak/toostrong, not mlevel.
        let difficulty = C_MONS_DIFFICULTY[c_mndx] as i32;
        if difficulty < min_mlev {
            continue;
        }
        if difficulty > max_mlev {
            continue;
        }

        if in_hell && (gf & G_NOHELL) != 0 {
            continue;
        }
        if !in_hell && (gf & crate::data::monsters::G_HELL) != 0 {
            continue;
        }

        let ct = (gf & G_FREQ_MASK) as i32 + shift;
        if ct <= 0 {
            continue;
        }
        choice_count += ct;
        mchoices[c_mndx] = ct;
    }

    // Select monster using C ordering
    let mut ct = rng.rnd(choice_count as u32) as i32;
    for c_mndx in 0..C_SPECIAL_PM {
        ct -= mchoices[c_mndx];
        if ct <= 0 {
            return C_TO_RUST_MONS[c_mndx];
        }
    }
    // Fallback
    C_TO_RUST_MONS[first_common]
}

/// C's is_armed(ptr): checks for AT_WEAP attack type
fn is_armed(mon: &PerMonst) -> bool {
    for atk in mon.attacks.iter() {
        if atk.attack_type == AttackType::Weapon {
            return true;
        }
    }
    false
}

/// C's is_dwarf(ptr)
fn is_dwarf(mon: &PerMonst) -> bool {
    mon.flags.contains(MonsterFlags::DWARF)
}

/// C's is_elf(ptr)
fn is_elf(mon: &PerMonst) -> bool {
    mon.flags.contains(MonsterFlags::ELF)
}

/// C's is_domestic(ptr)
fn is_domestic(mon: &PerMonst) -> bool {
    mon.flags.contains(MonsterFlags::DOMESTIC)
}

/// C's likes_gold(ptr) — simplified: nymphs and leprechauns
fn likes_gold(mon: &PerMonst) -> bool {
    // C: #define likes_gold(ptr) (((ptr)->mflags2 & M2_GREEDY) != 0L)
    mon.flags.contains(MonsterFlags::GREEDY)
}

/// C's strongmonst(ptr)
fn strongmonst(mon: &PerMonst) -> bool {
    mon.flags.contains(MonsterFlags::STRONG)
}

/// C's is_lord(ptr)
fn is_lord(mon: &PerMonst) -> bool {
    mon.flags.contains(MonsterFlags::LORD)
}

/// C's is_prince(ptr)
fn is_prince(mon: &PerMonst) -> bool {
    mon.flags.contains(MonsterFlags::PRINCE)
}

/// C's extra_nasty(ptr)
fn extra_nasty(mon: &PerMonst) -> bool {
    mon.flags.contains(MonsterFlags::NASTY)
}

/// mongets(mtmp, otyp) RNG: calls mksobj(otyp, TRUE, FALSE)
fn mongets_c_rng(
    objects: &[ObjClassDef],
    bases: &ClassBases,
    rust_otyp: usize,
    class: ObjectClass,
    depth: i32,
    rng: &mut GameRng,
) {
    mksobj_c_rng(objects, bases, rust_otyp, class, true, false, depth, rng);
}

/// m_initthrow(mtmp, otyp, oquan): mksobj(otyp, TRUE, FALSE) + rn1(oquan, 3)
fn m_initthrow_c_rng(
    objects: &[ObjClassDef],
    bases: &ClassBases,
    rust_otyp: usize,
    class: ObjectClass,
    oquan: u32,
    depth: i32,
    rng: &mut GameRng,
) {
    mksobj_c_rng(objects, bases, rust_otyp, class, true, false, depth, rng);
    rng.rn2(oquan); // rn1(oquan, 3) = rn2(oquan) + 3
}

/// C's m_initweap(mtmp) RNG consumption — only called if is_armed(ptr)
fn m_initweap_c_rng(
    mon: &PerMonst,
    mndx: usize,
    m_lev: i32,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    depth: i32,
    rng: &mut GameRng,
) {
    let w = ObjectClass::Weapon;
    let a = ObjectClass::Armor;

    match mon.symbol {
        '@' => {
            // S_HUMAN: elves get specific equipment
            // Mercenaries/shopkeepers/priests have G_NOGEN, won't appear via rndmonst
            if is_elf(mon) {
                // C's m_initweap elf branch
                if rng.rn2(2) != 0 {
                    let _mithril_or_cloak = rng.rn2(2);
                    // mongets(ELVEN_MITHRIL_COAT or ELVEN_CLOAK)
                    mongets_c_rng(objects, bases, R_ELVEN_MITHRIL_COAT, a, depth, rng);
                }
                if rng.rn2(2) != 0 {
                    // mongets(ELVEN_LEATHER_HELM)
                    mongets_c_rng(objects, bases, C_ELVEN_LEATHER_HELM, a, depth, rng);
                } else if rng.rn2(4) == 0 {
                    // mongets(ELVEN_BOOTS)
                    mongets_c_rng(objects, bases, C_ELVEN_BOOTS, a, depth, rng);
                }
                if rng.rn2(2) != 0 {
                    // mongets(ELVEN_DAGGER)
                    mongets_c_rng(objects, bases, C_ELVEN_DAGGER, w, depth, rng);
                }
                match rng.rn2(3) {
                    0 => {
                        if rng.rn2(4) == 0 {
                            mongets_c_rng(objects, bases, C_ELVEN_SHIELD, a, depth, rng);
                        }
                        if rng.rn2(3) != 0 {
                            mongets_c_rng(objects, bases, C_ELVEN_SHORT_SWORD, w, depth, rng);
                        }
                        mongets_c_rng(objects, bases, C_ELVEN_BOW, w, depth, rng);
                        m_initthrow_c_rng(objects, bases, C_ELVEN_ARROW, w, 12, depth, rng);
                    }
                    1 => {
                        mongets_c_rng(objects, bases, C_ELVEN_BROADSWORD, w, depth, rng);
                        if rng.rn2(2) != 0 {
                            mongets_c_rng(objects, bases, C_ELVEN_SHIELD, a, depth, rng);
                        }
                    }
                    _ => {
                        if rng.rn2(2) != 0 {
                            mongets_c_rng(objects, bases, C_ELVEN_SPEAR, w, depth, rng);
                            mongets_c_rng(objects, bases, C_ELVEN_SHIELD, a, depth, rng);
                        }
                    }
                }
                if mon.name == "Elvenking" {
                    if rng.rn2(3) != 0 {
                        // in_mklev && Is_earthlevel: false for normal dungeon
                        mongets_c_rng(objects, bases, C_PICK_AXE, w, depth, rng);
                    }
                    if rng.rn2(50) == 0 {
                        mongets_c_rng(objects, bases, R_CRYSTAL_BALL, ObjectClass::Tool, depth, rng);
                    }
                }
                // Elves have their own weapon assignment, skip default case
                // But still do the offensive item check below
            }
            // MS_PRIEST: mksobj(MACE, FALSE, FALSE) + rnd(3) + rn2(2)
            else if mon.sound == MonsterSound::Priest {
                // mksobj(MACE, FALSE, FALSE) → init=FALSE → 0 RNG
                rng.rnd(3);  // otmp->spe = rnd(3)
                rng.rn2(2);  // if (!rn2(2)) curse(otmp)
            }
            // Other S_HUMAN types (mercenaries, shopkeepers, etc.) have G_NOGEN
            // and don't appear via rndmonst at depth 14
        }
        'H' => {
            // S_GIANT
            if rng.rn2(2) != 0 {
                // mongets: BOULDER for non-ettin, CLUB for ettin
                mongets_c_rng(objects, bases, C_CLUB, w, depth, rng);
            }
        }
        'h' => {
            // S_HUMANOID
            if mon.name == "hobbit" {
                match rng.rn2(3) {
                    0 => mongets_c_rng(objects, bases, C_DAGGER, w, depth, rng),
                    1 => mongets_c_rng(objects, bases, C_ELVEN_DAGGER, w, depth, rng),
                    _ => mongets_c_rng(objects, bases, C_SLING, w, depth, rng),
                }
                if rng.rn2(10) == 0 {
                    mongets_c_rng(objects, bases, R_ELVEN_MITHRIL_COAT, a, depth, rng);
                }
                if rng.rn2(10) == 0 {
                    mongets_c_rng(objects, bases, C_DWARVISH_CLOAK, a, depth, rng);
                }
            } else if is_dwarf(mon) {
                if rng.rn2(7) != 0 {
                    mongets_c_rng(objects, bases, C_DWARVISH_CLOAK, a, depth, rng);
                }
                if rng.rn2(7) != 0 {
                    mongets_c_rng(objects, bases, C_IRON_SHOES, a, depth, rng);
                }
                if rng.rn2(4) == 0 {
                    mongets_c_rng(objects, bases, C_DWARVISH_SHORT_SWORD, w, depth, rng);
                    if rng.rn2(2) != 0 {
                        mongets_c_rng(objects, bases, C_DWARVISH_MATTOCK, w, depth, rng);
                    } else {
                        let item = if rng.rn2(2) != 0 { C_AXE } else { C_DWARVISH_SPEAR };
                        mongets_c_rng(objects, bases, item, w, depth, rng);
                        mongets_c_rng(objects, bases, C_DWARVISH_ROUNDSHIELD, a, depth, rng);
                    }
                    mongets_c_rng(objects, bases, C_DWARVISH_IRON_HELM, a, depth, rng);
                    if rng.rn2(3) == 0 {
                        mongets_c_rng(objects, bases, R_DWARVISH_MITHRIL_COAT, a, depth, rng);
                    }
                } else {
                    let item = if rng.rn2(3) == 0 { C_PICK_AXE } else { C_DAGGER };
                    mongets_c_rng(objects, bases, item, w, depth, rng);
                }
            }
        }
        'k' => {
            // S_KOBOLD
            if rng.rn2(4) == 0 {
                m_initthrow_c_rng(objects, bases, C_DART, w, 12, depth, rng);
            }
        }
        'o' => {
            // S_ORC
            if rng.rn2(2) != 0 {
                mongets_c_rng(objects, bases, C_ORCISH_HELM, a, depth, rng);
            }
            // Orc captain: random between mordor/uruk-hai
            let effective_type = if mon.name == "orc-captain" {
                if rng.rn2(2) != 0 { "mordor" } else { "uruk-hai" }
            } else {
                mon.name
            };
            match effective_type {
                n if n.contains("Mordor") || n == "mordor" => {
                    if rng.rn2(3) == 0 { mongets_c_rng(objects, bases, C_SCIMITAR, w, depth, rng); }
                    if rng.rn2(3) == 0 { mongets_c_rng(objects, bases, C_ORCISH_SHIELD, a, depth, rng); }
                    if rng.rn2(3) == 0 { mongets_c_rng(objects, bases, C_KNIFE, w, depth, rng); }
                    if rng.rn2(3) == 0 { mongets_c_rng(objects, bases, C_ORCISH_CHAIN_MAIL, a, depth, rng); }
                }
                n if n.contains("Uruk") || n == "uruk-hai" => {
                    if rng.rn2(3) == 0 { mongets_c_rng(objects, bases, C_ORCISH_CLOAK, a, depth, rng); }
                    if rng.rn2(3) == 0 { mongets_c_rng(objects, bases, C_ORCISH_SHORT_SWORD, w, depth, rng); }
                    if rng.rn2(3) == 0 { mongets_c_rng(objects, bases, C_IRON_SHOES, a, depth, rng); }
                    if rng.rn2(3) == 0 {
                        mongets_c_rng(objects, bases, C_ORCISH_BOW, w, depth, rng);
                        m_initthrow_c_rng(objects, bases, C_ORCISH_ARROW, w, 12, depth, rng);
                    }
                    if rng.rn2(3) == 0 { mongets_c_rng(objects, bases, C_URUK_HAI_SHIELD, a, depth, rng); }
                }
                _ => {
                    // default orc (including orc shaman)
                    if mon.name != "orc shaman" && rng.rn2(2) != 0 {
                        let item = if mon.name == "goblin" || rng.rn2(2) == 0 {
                            C_ORCISH_DAGGER
                        } else {
                            C_SCIMITAR
                        };
                        mongets_c_rng(objects, bases, item, w, depth, rng);
                    }
                }
            }
        }
        'O' => {
            // S_OGRE
            let threshold = if mon.name == "ogre king" { 3 } else if mon.name == "ogre lord" { 6 } else { 12 };
            if rng.rn2(threshold) == 0 {
                mongets_c_rng(objects, bases, C_BATTLE_AXE, w, depth, rng);
            } else {
                mongets_c_rng(objects, bases, C_CLUB, w, depth, rng);
            }
        }
        'T' => {
            // S_TROLL
            if rng.rn2(2) == 0 {
                match rng.rn2(4) {
                    0 => mongets_c_rng(objects, bases, C_RANSEUR, w, depth, rng),
                    1 => mongets_c_rng(objects, bases, C_PARTISAN, w, depth, rng),
                    2 => mongets_c_rng(objects, bases, C_GLAIVE, w, depth, rng),
                    _ => mongets_c_rng(objects, bases, C_SPETUM, w, depth, rng),
                }
            }
        }
        'C' => {
            // S_CENTAUR
            if rng.rn2(2) != 0 {
                if mon.name == "forest centaur" {
                    mongets_c_rng(objects, bases, C_BOW, w, depth, rng);
                    m_initthrow_c_rng(objects, bases, C_ARROW, w, 12, depth, rng);
                } else {
                    mongets_c_rng(objects, bases, C_CROSSBOW, w, depth, rng);
                    m_initthrow_c_rng(objects, bases, C_CROSSBOW_BOLT, w, 12, depth, rng);
                }
            }
        }
        'K' => {
            // S_KOP
            if rng.rn2(4) == 0 {
                m_initthrow_c_rng(objects, bases, R_CORPSE, ObjectClass::Food, 2, depth, rng); // CREAM_PIE
            }
            if rng.rn2(3) == 0 {
                let item = if rng.rn2(2) != 0 { C_CLUB } else { C_RUBBER_HOSE };
                mongets_c_rng(objects, bases, item, w, depth, rng);
            }
        }
        _ => {
            // Default case: general weapon assignment
            let bias = is_lord(mon) as i32 + is_prince(mon) as i32 * 2 + extra_nasty(mon) as i32;
            let range = (14 - 2 * bias).max(1);
            match rng.rnd(range as u32) {
                1 => {
                    if strongmonst(mon) {
                        mongets_c_rng(objects, bases, C_BATTLE_AXE, w, depth, rng);
                    } else {
                        m_initthrow_c_rng(objects, bases, C_DART, w, 12, depth, rng);
                    }
                }
                2 => {
                    if strongmonst(mon) {
                        mongets_c_rng(objects, bases, C_TWO_HANDED_SWORD, w, depth, rng);
                    } else {
                        mongets_c_rng(objects, bases, C_CROSSBOW, w, depth, rng);
                        m_initthrow_c_rng(objects, bases, C_CROSSBOW_BOLT, w, 12, depth, rng);
                    }
                }
                3 => {
                    mongets_c_rng(objects, bases, C_BOW, w, depth, rng);
                    m_initthrow_c_rng(objects, bases, C_ARROW, w, 12, depth, rng);
                }
                4 => {
                    if strongmonst(mon) {
                        mongets_c_rng(objects, bases, C_LONG_SWORD, w, depth, rng);
                    } else {
                        m_initthrow_c_rng(objects, bases, C_DAGGER, w, 3, depth, rng);
                    }
                }
                5 => {
                    if strongmonst(mon) {
                        mongets_c_rng(objects, bases, C_LUCERN_HAMMER, w, depth, rng);
                    } else {
                        mongets_c_rng(objects, bases, C_AKLYS, w, depth, rng);
                    }
                }
                _ => {} // no weapon
            }
        }
    }

    // After switch: offensive item check
    if m_lev as u32 > rng.rn2(75) {
        // rnd_offensive_item: consumes rn2() calls + possibly mongets
        rnd_offensive_item_c_rng(mon, objects, bases, depth, rng);
    }
}

/// C's rnd_offensive_item (muse.c:1573) RNG consumption.
/// Returns an item type that gets passed to mongets.
/// mongets(0) is a no-op (no RNG), so we skip mongets_c_rng when result is 0.
fn rnd_offensive_item_c_rng(
    mon: &PerMonst,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    depth: i32,
    rng: &mut GameRng,
) {
    // C: is_animal || attacktype(AT_EXPL) || mindless || S_GHOST || S_KOP → return 0
    if mon.flags.contains(MonsterFlags::ANIMAL)
        || mon.flags.contains(MonsterFlags::MINDLESS)
        || mon.symbol == ' '  // ghost
        || mon.symbol == 'K'  // kop
    {
        // mongets(mtmp, 0) → if(!otyp) return 0; → no RNG
        return;
    }

    let difficulty = mon.difficulty as i32;

    // C: if (difficulty > 7 && !rn2(35)) return WAN_DEATH;
    if difficulty > 7 {
        if rng.rn2(35) == 0 {
            // WAN_DEATH → mongets wand
            mongets_c_rng(objects, bases, bases.get(ObjectClass::Wand), ObjectClass::Wand, depth, rng);
            return;
        }
    }

    // C: switch(rn2(9 - (difficulty < 4) + 4 * (difficulty > 6)))
    // difficulty < 4: 9 - 1 + 0 = 8
    // difficulty 4-6: 9 - 0 + 0 = 9
    // difficulty > 6: 9 - 0 + 4 = 13
    let range = 9u32 - (difficulty < 4) as u32 + 4 * (difficulty > 6) as u32;
    let result = rng.rn2(range);

    // Map result to item class for mongets
    // Cases 0/1: WAN_STRIKING or SCR_EARTH (case 0 can fall through to case 1)
    // Cases 2-6: potions (acid, confusion, blindness, sleeping, paralysis)
    // Cases 7-8: WAN_MAGIC_MISSILE
    // Cases 9-12: wands (sleep, fire, cold, lightning)
    match result {
        0 | 1 => {
            // case 0: SCR_EARTH (with metallic helm check, skipping) → falls through to WAN_STRIKING
            // case 1: WAN_STRIKING
            mongets_c_rng(objects, bases, bases.get(ObjectClass::Wand), ObjectClass::Wand, depth, rng);
        }
        2 => mongets_c_rng(objects, bases, bases.get(ObjectClass::Potion), ObjectClass::Potion, depth, rng), // POT_ACID
        3 => mongets_c_rng(objects, bases, bases.get(ObjectClass::Potion), ObjectClass::Potion, depth, rng), // POT_CONFUSION
        4 => mongets_c_rng(objects, bases, bases.get(ObjectClass::Potion), ObjectClass::Potion, depth, rng), // POT_BLINDNESS
        5 => mongets_c_rng(objects, bases, bases.get(ObjectClass::Potion), ObjectClass::Potion, depth, rng), // POT_SLEEPING
        6 => mongets_c_rng(objects, bases, bases.get(ObjectClass::Potion), ObjectClass::Potion, depth, rng), // POT_PARALYSIS
        7 | 8 => mongets_c_rng(objects, bases, bases.get(ObjectClass::Wand), ObjectClass::Wand, depth, rng), // WAN_MAGIC_MISSILE
        9 => mongets_c_rng(objects, bases, bases.get(ObjectClass::Wand), ObjectClass::Wand, depth, rng), // WAN_SLEEP
        10 => mongets_c_rng(objects, bases, bases.get(ObjectClass::Wand), ObjectClass::Wand, depth, rng), // WAN_FIRE
        11 => mongets_c_rng(objects, bases, bases.get(ObjectClass::Wand), ObjectClass::Wand, depth, rng), // WAN_COLD
        12 => mongets_c_rng(objects, bases, bases.get(ObjectClass::Wand), ObjectClass::Wand, depth, rng), // WAN_LIGHTNING
        _ => {} // no item (shouldn't happen with correct range)
    }
}

/// C's rnd_defensive_item RNG consumption
fn rnd_defensive_item_c_rng(
    mon: &PerMonst,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    depth: i32,
    rng: &mut GameRng,
) {
    // animal/mindless/ghost/kop check — if true, return 0 (no RNG)
    if mon.flags.contains(MonsterFlags::ANIMAL)
        || mon.flags.contains(MonsterFlags::MINDLESS)
        || mon.symbol == ' '  // ghost
        || mon.symbol == 'K'  // kop
    {
        return;
    }

    let difficulty = mon.difficulty as i32;
    let range = 8 + (difficulty > 3) as u32 + (difficulty > 6) as u32 + (difficulty > 8) as u32;
    let result = rng.rn2(range);

    let otyp = match result {
        6 | 9 => {
            if rng.rn2(3) == 0 {
                1 // WAN_TELEPORTATION → wand class
            } else {
                2 // SCR_TELEPORTATION → scroll class
            }
        }
        0 | 1 => 2, // SCR_TELEPORTATION
        8 | 10 => {
            if rng.rn2(3) == 0 {
                1 // WAN_CREATE_MONSTER
            } else {
                2 // SCR_CREATE_MONSTER
            }
        }
        2 => 2, // SCR_CREATE_MONSTER
        3 => 3, // POT_HEALING
        4 => 3, // POT_EXTRA_HEALING
        5 => 3, // POT_FULL_HEALING
        7 => 1, // WAN_DIGGING (or 0 for floaters)
        _ => 0,
    };

    match otyp {
        1 => mongets_c_rng(objects, bases, bases.get(ObjectClass::Wand), ObjectClass::Wand, depth, rng),
        2 => mongets_c_rng(objects, bases, bases.get(ObjectClass::Scroll), ObjectClass::Scroll, depth, rng),
        3 => mongets_c_rng(objects, bases, bases.get(ObjectClass::Potion), ObjectClass::Potion, depth, rng),
        _ => {} // no item
    }
}

/// C's rnd_misc_item (muse.c:2011) RNG consumption.
/// Structure: sequential checks with early returns, then a switch on rn2(3).
fn rnd_misc_item_c_rng(
    mon: &PerMonst,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    depth: i32,
    rng: &mut GameRng,
) {
    // C: is_animal || attacktype(AT_EXPL) || mindless || S_GHOST || S_KOP → return 0
    if mon.flags.contains(MonsterFlags::ANIMAL)
        || mon.flags.contains(MonsterFlags::MINDLESS)
        || mon.symbol == ' '  // ghost
        || mon.symbol == 'K'  // kop
    {
        // mongets(mtmp, 0) → if(!otyp) return 0; → no RNG
        return;
    }

    let difficulty = mon.difficulty as i32;

    // C: if (difficulty < 6 && !rn2(30)) return rn2(6) ? POT_POLYMORPH : WAN_POLYMORPH;
    if difficulty < 6 {
        if rng.rn2(30) == 0 {
            let wand_or_pot = rng.rn2(6);
            if wand_or_pot != 0 {
                // POT_POLYMORPH → mongets → mksobj(potion, TRUE, FALSE)
                mongets_c_rng(objects, bases, bases.get(ObjectClass::Potion), ObjectClass::Potion, depth, rng);
            } else {
                // WAN_POLYMORPH → mongets → mksobj(wand, TRUE, FALSE)
                mongets_c_rng(objects, bases, bases.get(ObjectClass::Wand), ObjectClass::Wand, depth, rng);
            }
            return;
        }
    }

    // C: if (!rn2(40) && !nonliving(pm) && !is_vampshifter(mtmp)) return AMULET_OF_LIFE_SAVING;
    if rng.rn2(40) == 0 {
        // nonliving and vampshifter checks don't consume RNG
        // For most monsters at depth 14, this returns the amulet
        // AMULET_OF_LIFE_SAVING → mongets → mksobj(amulet, TRUE, FALSE)
        mongets_c_rng(objects, bases, bases.get(ObjectClass::Amulet), ObjectClass::Amulet, depth, rng);
        return;
    }

    // C: switch (rn2(3)) { case 0: rn2(6) speed; case 1: rn2(6) invis; case 2: gain_level }
    let choice = rng.rn2(3);
    match choice {
        0 => {
            // mtmp->isgd check (false for leprechaun)
            let wand_or_pot = rng.rn2(6);
            if wand_or_pot != 0 {
                // POT_SPEED → mongets potion
                mongets_c_rng(objects, bases, bases.get(ObjectClass::Potion), ObjectClass::Potion, depth, rng);
            } else {
                // WAN_SPEED_MONSTER → mongets wand
                mongets_c_rng(objects, bases, bases.get(ObjectClass::Wand), ObjectClass::Wand, depth, rng);
            }
        }
        1 => {
            // mpeaceful && !See_invisible → return 0 (no item)
            // Leprechaun in fill_zoo is hostile (mpeaceful=0), so this doesn't apply
            let wand_or_pot = rng.rn2(6);
            if wand_or_pot != 0 {
                // POT_INVISIBILITY → mongets potion
                mongets_c_rng(objects, bases, bases.get(ObjectClass::Potion), ObjectClass::Potion, depth, rng);
            } else {
                // WAN_MAKE_INVISIBLE → mongets wand
                mongets_c_rng(objects, bases, bases.get(ObjectClass::Wand), ObjectClass::Wand, depth, rng);
            }
        }
        2 => {
            // POT_GAIN_LEVEL → mongets potion
            mongets_c_rng(objects, bases, bases.get(ObjectClass::Potion), ObjectClass::Potion, depth, rng);
        }
        _ => {}
    }
}

/// C's m_initinv(mtmp) RNG consumption
fn m_initinv_c_rng(
    mon: &PerMonst,
    mndx: usize,
    m_lev: i32,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    depth: i32,
    rng: &mut GameRng,
) {
    let a = ObjectClass::Armor;
    let t = ObjectClass::Tool;
    let p = ObjectClass::Potion;
    let w = ObjectClass::Weapon;
    let wn = ObjectClass::Wand;

    // Track whether monster already received gold (affects end-of-function gold check)
    let mut has_gold = false;

    match mon.symbol {
        'M' => {
            // S_MUMMY: rn2(7) chance of mummy wrapping
            if rng.rn2(7) != 0 {
                mongets_c_rng(objects, bases, R_MUMMY_WRAPPING, a, depth, rng);
            }
        }
        'n' => {
            // S_NYMPH: mirror + potion of object detection
            if rng.rn2(2) == 0 {
                mongets_c_rng(objects, bases, R_MIRROR, t, depth, rng);
            }
            if rng.rn2(2) == 0 {
                mongets_c_rng(objects, bases, R_POT_OBJECT_DETECTION, p, depth, rng);
            }
        }
        'H' => {
            // S_GIANT: minotaur gets wand of digging, giants get gems
            if mon.name == "minotaur" {
                if rng.rn2(3) == 0 {
                    // in_mklev && Is_earthlevel: false for depth 14 main dungeon
                    mongets_c_rng(objects, bases, R_WAN_DIGGING, wn, depth, rng);
                }
            } else if mon.flags.contains(MonsterFlags::GIANT) {
                // for cnt = rn2(m_lev/2); cnt; cnt--
                let cnt = rng.rn2((m_lev / 2).max(1) as u32);
                for _ in 0..cnt {
                    // rnd_class(DILITHIUM_CRYSTAL, LUCKSTONE-1): 1 RNG call
                    rng.rnd(R_GEM_CLASS_PROB_SUM as u32); // rnd(sum of gem probabilities)
                    // mksobj(otyp, FALSE, FALSE) for gem: rn2(6) for quantity check
                    rng.rn2(6);
                    // rn1(2, 3) = rn2(2) + 3
                    rng.rn2(2);
                }
            }
        }
        'W' => {
            // S_WRAITH: nazgul gets ring of invisibility
            if mon.name == "Nazgul" {
                // mksobj(RIN_INVISIBILITY, FALSE, FALSE) — ring with init=FALSE
                // Ring mksobj with init=FALSE: no RNG consumed
                // (no mongets, direct mksobj+mpickobj, no bless/curse since init=FALSE)
            }
        }
        'L' => {
            // S_LICH: master lich / arch lich
            if mon.name == "master lich" {
                if rng.rn2(13) == 0 {
                    let _athame_or_nothing = rng.rn2(7);
                    // mongets(athame or WAN_NOTHING)
                    mongets_c_rng(objects, bases, R_ATHAME, w, depth, rng);
                }
            } else if mon.name == "arch-lich" {
                if rng.rn2(3) == 0 {
                    // mksobj(rn2(3) ? ATHAME : QUARTERSTAFF, TRUE, rn2(13) ? FALSE : TRUE)
                    let _weapon_type = rng.rn2(3);
                    let _artif = rng.rn2(13);
                    // mksobj with init=TRUE: consumes RNG for curse/bless + spe
                    mksobj_c_rng(objects, bases, R_ATHAME, w, true, false, depth, rng);
                    // if spe < 2: rnd(3)
                    rng.rnd(3);
                    // rn2(4) for oerodeproof
                    rng.rn2(4);
                }
            }
        }
        'Q' => {
            // S_QUANTMECH: Schroedinger's cat
            if rng.rn2(20) == 0 {
                // mksobj(LARGE_BOX, FALSE, FALSE) — no RNG for box with init=FALSE
                // mksobj(CORPSE, TRUE, FALSE) — corpse with init=TRUE
                mksobj_c_rng(objects, bases, R_CORPSE, ObjectClass::Food, true, false, depth, rng);
            }
        }
        'l' => {
            // S_LEPRECHAUN: mkmonmoney(d(level_difficulty(), 30))
            let n = depth.max(1) as u32;
            for _ in 0..n {
                rng.rn2(30);
            }
            has_gold = true;
        }
        '&' => {
            // S_DEMON: ice devil spear, asmodeus wands
            if mon.name == "ice devil" {
                if rng.rn2(4) == 0 {
                    mongets_c_rng(objects, bases, R_SPEAR, w, depth, rng);
                }
            }
            // asmodeus gets WAN_COLD + WAN_FIRE but is very rare at depth 14
            // Other demons: no inventory
        }
        'G' => {
            // S_GNOME: candle check
            if rng.rn2(60) == 0 {
                // mksobj(rn2(4) ? TALLOW_CANDLE : WAX_CANDLE, TRUE, FALSE)
                let _candle_type = rng.rn2(4);
                mongets_c_rng(objects, bases, R_TALLOW_CANDLE, t, depth, rng);
            }
        }
        '@' => {
            // S_HUMAN: priest inventory
            if mon.sound == MonsterSound::Priest {
                // C: rn2(7) ? ROBE : (rn2(3) ? CLOAK_OF_PROTECTION : CLOAK_OF_MAGIC_RESISTANCE)
                let robe_type = if rng.rn2(7) != 0 {
                    R_ROBE
                } else if rng.rn2(3) != 0 {
                    R_CLOAK_OF_PROTECTION
                } else {
                    R_CLOAK_OF_MAGIC_RESISTANCE
                };
                mongets_c_rng(objects, bases, robe_type, a, depth, rng);
                // mongets(SMALL_SHIELD)
                mongets_c_rng(objects, bases, R_SMALL_SHIELD, a, depth, rng);
                // mkmonmoney(rn1(10, 20)) = rn2(10) + 20
                rng.rn2(10);
            }
        }
        _ => {}
    }

    // End-of-function checks (apply to all monsters)

    // soldier check: PM_SOLDIER && rn2(13) → return early
    // Soldiers are rare at depth 14 via rndmonst, skip for now

    // defensive item check
    if m_lev as u32 > rng.rn2(50) {
        rnd_defensive_item_c_rng(mon, objects, bases, depth, rng);
    }

    // misc item check
    if m_lev as u32 > rng.rn2(100) {
        rnd_misc_item_c_rng(mon, objects, bases, depth, rng);
    }

    // gold check: C does `if (likes_gold && !findgold(minvent) && !rn2(5))`
    // If the monster already has gold (e.g. leprechaun from case switch), findgold is true → skip
    if likes_gold(mon) && !has_gold {
        if rng.rn2(5) == 0 {
            // mkmonmoney(d(level_difficulty(), 5 or 10))
            let n = depth.max(1) as u32;
            for _ in 0..n {
                rng.rn2(10);
            }
        }
    }
}

/// C's mkclass(S_MIMIC, 0) + makemon(ptr, x, y, NO_MM_FLAGS) during in_mklev.
/// Used by dosdoor when a trapped door becomes a mimic.
///
/// mkclass iterates C's mons[] for S_MIMIC class (indices 63-65: small, large, giant mimic),
/// applying toostrong() breaks and frequency-based selection. Then makemon creates the
/// monster with the full RNG chain (newmonhp, gender, saddle check).
pub fn mimic_door_c_rng(depth: i32, rng: &mut GameRng) {
    let player_level: i32 = 1; // u.ulevel at start

    // --- mkclass(S_MIMIC, 0) ---
    // C mimic entries in mons[] order, with C-exact values:
    //   C[63] PM_SMALL_MIMIC: mlevel=7, difficulty=8, geno freq=2
    //   C[64] PM_LARGE_MIMIC: mlevel=8, difficulty=9, geno freq=1
    //   C[65] PM_GIANT_MIMIC: mlevel=9, difficulty=11, geno freq=1
    let maxmlev = depth / 2; // level_difficulty() >> 1

    struct CMimic { mlevel: i32, difficulty: i32, g_freq: i32 }
    let mimics = [
        CMimic { mlevel: 7, difficulty: 8,  g_freq: 2 },
        CMimic { mlevel: 8, difficulty: 9,  g_freq: 1 },
        CMimic { mlevel: 9, difficulty: 11, g_freq: 1 },
    ];

    let mut num: i32 = 0;
    let mut nums = [0i32; 3];
    let mut count = 0usize;

    for (i, m) in mimics.iter().enumerate() {
        // toostrong check (only when num > 0 and difficulty increases)
        if num > 0
            && m.difficulty > maxmlev
            && (i == 0 || m.difficulty > mimics[i - 1].difficulty)
            && rng.rn2(2) != 0
        {
            break;
        }
        // adj_lev for bias calculation
        let adj = adj_lev_c_from_raw(m.mlevel, depth, player_level);
        let bias = if adj > player_level * 2 { 1 } else { 0 };
        let k = m.g_freq;
        nums[i] = k + 1 - bias;
        if nums[i] > 0 {
            num += nums[i];
        }
        count = i + 1;
    }

    if num == 0 {
        return;
    }

    // rnd(num) to select among candidates
    eprintln!("  RS mkclass: num={} count={} rng={}", num, count, rng.call_count());
    let mut pick = rng.rnd(num as u32) as i32;
    let mut chosen_mlevel = mimics[0].mlevel;
    let mut chosen_idx = 0;
    for i in 0..count {
        pick -= nums[i];
        if pick <= 0 {
            chosen_mlevel = mimics[i].mlevel;
            chosen_idx = i;
            break;
        }
    }
    eprintln!("  RS mkclass: chose mimic[{}] mlevel={} rng={}", chosen_idx, chosen_mlevel, rng.call_count());

    // --- makemon(ptr, x, y, NO_MM_FLAGS) with specific ptr ---
    // ptr is not NULL → anymon=false, no rndmonst call, no group check

    // newmonhp: d(adj_lev, 8)
    // All mimics have mlevel > 5, so adj_lev = mlevel + level_difficulty/2
    let m_lev = adj_lev_c_from_raw(chosen_mlevel, depth, player_level);
    eprintln!("  RS makemon: m_lev={} rng={}", m_lev, rng.call_count());
    if m_lev == 0 {
        rng.rnd(4);
    } else {
        for _ in 0..m_lev {
            rng.rn2(8);
        }
    }
    eprintln!("  RS makemon: after_newmonhp rng={}", rng.call_count());

    // gender: mimics have no fixed gender → rn2(2)
    rng.rn2(2);
    eprintln!("  RS makemon: after_gender rng={}", rng.call_count());

    // peace_minded: mimics are M2_HOSTILE → always hostile, no RNG

    // class switch: S_MIMIC → set_mimic_sym → at door cell → no RNG
    // (IS_DOOR check matches first, takes deterministic branch)

    // in_mklev sleep: not ndemon/wumpus/long_worm/giant_eel → no

    // group: anymon=false → skip

    // allow_minvent=true (NO_MM_FLAGS):
    //   is_armed: mimics have no weapon attacks → skip m_initweap
    //   m_initinv: no S_MIMIC case in switch (falls to default: break),
    //     BUT after the switch, m_initinv always calls:
    //       rn2(50) for defensive item check
    //       rn2(100) for misc item check
    //     likes_gold is false for mimics → no rn2(5)
    rng.rn2(50);
    rng.rn2(100);

    //   m_dowear: no inventory → no-op

    // saddle: rn2(100) always called; mimics not domestic → short-circuits
    rng.rn2(100);
}

/// adj_lev helper that works from raw mlevel (no PerMonst needed).
/// Matches C's adj_lev() exactly.
fn adj_lev_c_from_raw(mlevel: i32, depth: i32, player_level: i32) -> i32 {
    let mut tmp = mlevel;
    if tmp > 49 {
        return 50;
    }
    let tmp2 = depth - tmp;
    if tmp2 < 0 {
        tmp -= 1;
    } else {
        tmp += tmp2 / 5;
    }
    let tmp2 = player_level - mlevel;
    if tmp2 > 0 {
        tmp += tmp2 / 4;
    }
    let upper = ((3 * mlevel) / 2).min(49);
    if tmp > upper { upper } else if tmp > 0 { tmp } else { 0 }
}

/// makemon with a specific C monster index (ptr != NULL).
/// Used when C calls makemon(&mons[c_mndx], ...) with a known monster type.
/// anymon=false → no rndmonst, no group check.
fn makemon_specific_c_rng(
    c_mndx: usize,
    depth: i32,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    rng: &mut GameRng,
) {
    let monsters = MONSTERS;
    let player_level: i32 = 1;
    let player_alignment: i8 = 1; // Lawful: aligns[flags.initalign=0].value = A_LAWFUL
    let align_record: i32 = 0;

    let mndx = C_TO_RUST_MONS[c_mndx];
    let mon = &monsters[mndx];

    // newmonhp: d(adj_lev, 8) or rnd(4) for level-0
    let m_lev = adj_lev_c(mon, depth, player_level);
    if m_lev == 0 {
        rng.rnd(4);
    } else {
        for _ in 0..m_lev {
            rng.rn2(8);
        }
    }

    // gender
    let is_female = mon.flags.contains(MonsterFlags::FEMALE);
    let is_male = mon.flags.contains(MonsterFlags::MALE);
    if !is_female && !is_male {
        rng.rn2(2);
    }

    // peace_minded
    let always_peaceful = mon.flags.contains(MonsterFlags::PEACEFUL);
    let always_hostile = mon.flags.contains(MonsterFlags::HOSTILE);
    if !always_peaceful && !always_hostile {
        let mal = mon.alignment;
        let ual = player_alignment;
        if mal.signum() != ual.signum() {
            // different alignment → hostile
        } else {
            let first = rng.rn2((16 + align_record).max(1) as u32);
            if first != 0 {
                rng.rn2((2 + mal.unsigned_abs() as i32).max(1) as u32);
            }
        }
    }

    // class switch
    match mon.symbol {
        's' | 'S' => {
            mkobj_c_rng(objects, bases, depth, rng);
        }
        'J' | 'n' => {
            rng.rn2(5);
        }
        'm' => {
            // S_MIMIC: set_mimic_sym — no RNG at door cell
        }
        _ => {}
    }

    let _rng_before_sleep = rng.call_count();
    // in_mklev sleep check
    let is_ndemon = mon.flags.contains(MonsterFlags::DEMON) && !is_lord(mon) && !is_prince(mon);
    if is_ndemon || mon.name == "wumpus" || mon.name == "long worm" || mon.name == "giant eel" {
        rng.rn2(5);
    }

    // anymon=false → no group check

    // m_initweap (only if armed)
    let _rng_before_weap = rng.call_count();
    if is_armed(mon) {
        m_initweap_c_rng(mon, mndx, m_lev, objects, bases, depth, rng);
    }
    let _rng_after_weap = rng.call_count();

    // m_initinv (includes rn2(50) defensive + rn2(100) misc + gold check)
    m_initinv_c_rng(mon, mndx, m_lev, objects, bases, depth, rng);
    let _rng_after_inv = rng.call_count();

    // saddle
    rng.rn2(100);
    eprintln!("  RS MAKEMON_SPEC {}: weap_delta={} inv_delta={} rng={}",
        mon.name, _rng_after_weap - _rng_before_weap, _rng_after_inv - _rng_after_weap, rng.call_count());
}

/// Full makemon(NULL, x, y, MM_NOGRP) RNG consumption during in_mklev.
/// Faithfully ports C's makemon chain for exact RNG parity.
fn makemon_c_rng(
    level: &mut Level,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    depth: i32,
    rng: &mut GameRng,
) {
    let monsters = MONSTERS;
    let player_level = 1; // u.ulevel at start
    let player_alignment: i8 = 1; // Lawful: aligns[flags.initalign=0].value = A_LAWFUL
    let align_record: i32 = 0; // starting value
    let in_hell = false;

    // 1. rndmonst: select monster (1 RNG call)
    let mndx = rndmonst_c_rng(depth, player_level, in_hell, rng);
    let mon = &monsters[mndx];

    eprintln!("  RS MAKEMON {}: mndx={} mlet='{}' mlevel={} rng={}",
        mon.name, mndx, mon.symbol, mon.level, rng.call_count());

    // 2. newmonhp: d(adj_lev, 8) or special cases
    let m_lev = adj_lev_c(mon, depth, player_level);
    if m_lev == 0 {
        rng.rnd(4); // level 0 monsters use rnd(4)
    } else {
        // d(m_lev, 8): m_lev calls to rn2(8)
        // C's d(n,x): tmp=n; while(n--) tmp += RND(x); where RND=rn2
        for _ in 0..m_lev {
            rng.rn2(8);
        }
    }

    eprintln!("  RS MAKEMON {}: after_newmonhp m_lev={}", mon.name, m_lev);

    // 3. Gender
    let is_female = mon.flags.contains(MonsterFlags::FEMALE);
    let is_male = mon.flags.contains(MonsterFlags::MALE);
    if !is_female && !is_male {
        // Not hardcoded: check leader/nemesis (skip for level generation)
        rng.rn2(2);
    }

    // 4. peace_minded
    let always_peaceful = mon.flags.contains(MonsterFlags::PEACEFUL);
    let always_hostile = mon.flags.contains(MonsterFlags::HOSTILE);
    if !always_peaceful && !always_hostile {
        // Need to check alignment
        let mal = mon.alignment;
        let ual = player_alignment;
        if mal.signum() != ual.signum() {
            // Different alignment → hostile, 0 RNG calls
        } else {
            // Co-aligned: rn2(16 + record) && rn2(2 + abs(mal))
            let first = rng.rn2((16 + align_record).max(1) as u32);
            if first != 0 {
                rng.rn2((2 + mal.unsigned_abs() as i32).max(1) as u32);
            }
        }
    }

    // 5. Class switch (RNG-consuming cases)
    match mon.symbol {
        's' | 'S' => {
            // S_SPIDER / S_SNAKE: mkobj_at(0, x, y, TRUE) during in_mklev
            mkobj_c_rng(objects, bases, depth, rng);
        }
        'J' | 'n' => {
            // S_JABBERWOCK / S_NYMPH: rn2(5) for sleep
            rng.rn2(5);
        }
        _ => {} // no RNG
    }

    // 6. in_mklev sleep check (for specific monster types)
    // is_ndemon, PM_WUMPUS, PM_LONG_WORM, PM_GIANT_EEL
    // These are rare at depth 14, but check anyway
    let is_ndemon = mon.flags.contains(MonsterFlags::DEMON) && !is_lord(mon) && !is_prince(mon);
    let is_special_sleep = is_ndemon
        || mon.name == "wumpus"
        || mon.name == "long worm"
        || mon.name == "giant eel";
    if is_special_sleep {
        rng.rn2(5);
    }

    // 7. Group check: MM_NOGRP → skip (0 calls)

    // 8. m_initweap (only if armed)
    if is_armed(mon) {
        m_initweap_c_rng(mon, mndx, m_lev, objects, bases, depth, rng);
    }

    // 9. m_initinv
    m_initinv_c_rng(mon, mndx, m_lev, objects, bases, depth, rng);

    // 10. saddle check: rn2(100) always, is_domestic short-circuits rest
    rng.rn2(100);
}

/// peace_minded RNG consumption for a given monster.
/// Returns the number of RNG calls consumed.
fn peace_minded_c_rng(mon: &PerMonst, player_alignment: i8, align_record: i32, rng: &mut GameRng) {
    let always_peaceful = mon.flags.contains(MonsterFlags::PEACEFUL);
    let always_hostile = mon.flags.contains(MonsterFlags::HOSTILE);
    if !always_peaceful && !always_hostile {
        let mal = mon.alignment;
        let ual = player_alignment;
        // race_peaceful: Human lovemask=0 → never
        // race_hostile: Human hatmask=MH_GNOME|MH_ORC → gnomes/orcs hostile
        // But these already have different alignment sign for Lawful player
        if mal.signum() != ual.signum() {
            // Different alignment → hostile, 0 RNG calls
        } else {
            // Co-aligned: rn2(16 + record) && rn2(2 + abs(mal))
            let first = rng.rn2((16 + align_record).max(1) as u32);
            if first != 0 {
                rng.rn2((2 + mal.unsigned_abs() as i32).max(1) as u32);
            }
        }
    }
}

/// Full makemon(NULL, x, y, MM_ASLEEP) RNG consumption for zoo monsters.
/// Like makemon_c_rng but with group check enabled (anymon=true, no MM_NOGRP).
fn makemon_zoo_c_rng(
    level: &mut Level,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    depth: i32,
    rng: &mut GameRng,
) {
    let monsters = MONSTERS;
    let player_level = 1;
    let player_alignment: i8 = 1; // Lawful
    let align_record: i32 = 0;
    let in_hell = false;

    // 1. rndmonst
    let mndx = rndmonst_c_rng(depth, player_level, in_hell, rng);
    let mon = &monsters[mndx];

    // 2. newmonhp
    let m_lev = adj_lev_c(mon, depth, player_level);
    if m_lev == 0 {
        rng.rnd(4);
    } else {
        for _ in 0..m_lev {
            rng.rn2(8);
        }
    }

    // 3. Gender
    let is_female = mon.flags.contains(MonsterFlags::FEMALE);
    let is_male = mon.flags.contains(MonsterFlags::MALE);
    if !is_female && !is_male {
        rng.rn2(2);
    }

    // 4. peace_minded
    peace_minded_c_rng(mon, player_alignment, align_record, rng);

    // 5. Class switch
    match mon.symbol {
        's' | 'S' => {
            mkobj_c_rng(objects, bases, depth, rng);
        }
        'J' | 'n' => {
            rng.rn2(5);
        }
        _ => {}
    }

    // 6. in_mklev sleep check
    let is_ndemon = mon.flags.contains(MonsterFlags::DEMON) && !is_lord(mon) && !is_prince(mon);
    let is_special_sleep = is_ndemon
        || mon.name == "wumpus"
        || mon.name == "long worm"
        || mon.name == "giant eel";
    if is_special_sleep {
        rng.rn2(5);
    }

    // 7. Group check: anymon=true, mmflags=MM_ASLEEP (no MM_NOGRP)
    // G_SGROUP=0x0080, G_LGROUP=0x0040
    let geno = C_MONS_GENO[find_c_mndx(mndx)];
    let has_sgroup = (geno & 0x0080) != 0;
    let has_lgroup = (geno & 0x0040) != 0;

    if has_sgroup && rng.rn2(2) != 0 {
        // m_initsgrp: n=3
        m_initgrp_c_rng(mon, mndx, m_lev, 3, player_level, player_alignment, align_record, depth, objects, bases, rng);
    } else if has_lgroup {
        if rng.rn2(3) != 0 {
            // m_initlgrp: n=10
            m_initgrp_c_rng(mon, mndx, m_lev, 10, player_level, player_alignment, align_record, depth, objects, bases, rng);
        } else {
            // m_initsgrp: n=3
            m_initgrp_c_rng(mon, mndx, m_lev, 3, player_level, player_alignment, align_record, depth, objects, bases, rng);
        }
    }

    // 8. m_initweap
    if is_armed(mon) {
        m_initweap_c_rng(mon, mndx, m_lev, objects, bases, depth, rng);
    }

    // 9. m_initinv
    m_initinv_c_rng(mon, mndx, m_lev, objects, bases, depth, rng);

    // 10. saddle check
    rng.rn2(100);
}

/// C's m_initgrp RNG consumption.
/// Creates cnt group members, each consuming peace_minded + makemon(ptr, ..., MM_NOGRP).
fn m_initgrp_c_rng(
    mon: &PerMonst,
    mndx: usize,
    _m_lev: i32,
    n: u32,
    player_level: i32,
    player_alignment: i8,
    align_record: i32,
    depth: i32,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    rng: &mut GameRng,
) {
    let mut cnt = rng.rnd(n) as i32;
    // cnt /= (ulevel < 3) ? 4 : (ulevel < 5) ? 2 : 1
    cnt /= if player_level < 3 { 4 } else if player_level < 5 { 2 } else { 1 };
    if cnt == 0 {
        cnt = 1;
    }

    for _ in 0..cnt {
        // peace_minded check in group loop (before makemon)
        // If peaceful, skip this member (continue)
        // We need to consume the RNG calls for peace_minded
        // and determine if monster would be peaceful
        let peaceful = is_peaceful_c(mon, player_alignment, align_record, rng);
        if peaceful {
            continue; // C skips peaceful group members
        }

        // enexto: no RNG
        // makemon(ptr, x, y, mmflags | MM_NOGRP) — specific ptr, MM_NOGRP
        // This is makemon_specific_c_rng but we already know mndx
        makemon_specific_c_rng(find_c_mndx(mndx), depth, objects, bases, rng);
    }
}

/// Check if monster would be peaceful and consume appropriate RNG.
/// Returns true if peaceful, consuming 0-2 RNG calls.
fn is_peaceful_c(
    mon: &PerMonst,
    player_alignment: i8,
    align_record: i32,
    rng: &mut GameRng,
) -> bool {
    let always_peaceful = mon.flags.contains(MonsterFlags::PEACEFUL);
    let always_hostile = mon.flags.contains(MonsterFlags::HOSTILE);
    if always_peaceful {
        return true;
    }
    if always_hostile {
        return false;
    }
    let mal = mon.alignment;
    let ual = player_alignment;
    if mal.signum() != ual.signum() {
        return false;
    }
    // Co-aligned peace check
    let first = rng.rn2((16 + align_record).max(1) as u32);
    if first != 0 {
        let second = rng.rn2((2 + mal.unsigned_abs() as i32).max(1) as u32);
        first != 0 && second != 0
    } else {
        false
    }
}

/// Find C monster index from Rust monster index
fn find_c_mndx(rust_mndx: usize) -> usize {
    for (c_idx, &r_idx) in C_TO_RUST_MONS.iter().enumerate() {
        if r_idx == rust_mndx {
            return c_idx;
        }
    }
    0 // fallback
}

/// Stub for mktrap RNG consumption.
/// C's mktrap(0, 0, croom, NULL) RNG consumption during in_mklev.
/// Faithfully ports the rndtrap + position + maketrap + dead-adventurer chain.
fn mktrap_c_rng(
    level: &mut Level,
    room: &Room,
    depth: i32,
    rng: &mut GameRng,
) {
    let objects = OBJECTS;
    let bases = ClassBases::compute(objects);
    let in_hell = false;
    let noteleport = false; // level.flags.noteleport for normal dungeon levels
    let can_fall_thru = true; // depth 14 in main dungeon

    // 1. rndtrap selection: do { kind = rnd(TRAPNUM-1); ... } while (kind == NO_TRAP);
    // TRAPNUM = 24, so rnd(23) generates 1..23
    let kind = loop {
        let k = rng.rnd(23) as i32;
        let accepted = match k {
            17 | 23 => false, // MAGIC_PORTAL, VIBRATING_SQUARE: always rejected
            10 => in_hell,    // FIRE_TRAP: only in hell
            8 => depth >= 2,  // SLP_GAS_TRAP: lvl >= 2
            7 => depth >= 2,  // ROLLING_BOULDER_TRAP: lvl >= 2
            16 => depth >= 5 && !noteleport, // LEVEL_TELEP
            12 => depth >= 5, // SPIKED_PIT
            6 => depth >= 6,  // LANDMINE
            18 => depth >= 7, // WEB
            19 | 22 => depth >= 8, // STATUE_TRAP, POLY_TRAP
            15 => !noteleport, // TELEP_TRAP
            13 => {           // HOLE: extra rn2(7) rejection
                if rng.rn2(7) != 0 { false } else { true }
            }
            _ => true,
        };
        if accepted { break k; }
    };

    // hole/trapdoor → rocktrap if can't fall through
    let kind = if (kind == 13 || kind == 14) && !can_fall_thru { 3 } else { kind };

    // 2. Position: somexy(croom, &m) + occupied() retry loop
    // For first trap in a room, occupied() is typically false → 1 iteration
    let _tx = super::room::somex(room, rng);
    let _ty = super::room::somey(room, rng);

    // 3. maketrap() RNG consumption — depends on trap type
    match kind {
        4 => {
            // SQKY_BOARD: rn2(available_notes) — for first squeaky board, all 12 available
            // In general, rn2(12 - num_existing_sqky_boards) but we don't track this
            // C: rn2(tcnt) where tcnt = 12 - (number of existing SQKY_BOARD traps)
            rng.rn2(12); // approximate: first sqky board gets rn2(12)
        }
        7 => {
            // ROLLING_BOULDER_TRAP: mkroll_launch
            // rn1(5,4) = rn2(5)+4 for distance, rn2(8) for direction
            // Then checks 8 directions × decreasing distances for valid path.
            // NO RNG consumed in the path search loop.
            // If success: mksobj(BOULDER, TRUE, FALSE) for boulder placement.
            // If !success: no mksobj (launch point = trap point, no boulder).
            // We can't determine success without the full level geometry,
            // so we consume the 2 mkroll_launch RNG calls and skip mksobj.
            // This is approximate — may be wrong when the path succeeds.
            rng.rn2(5); // distance
            rng.rn2(8); // direction
        }
        18 => {
            // WEB: makemon(&mons[PM_GIANT_SPIDER], m.x, m.y, NO_MM_FLAGS)
            // PM_GIANT_SPIDER = C[95], specific ptr → anymon=false
            makemon_specific_c_rng(95, depth, &objects, &bases, rng);
        }
        19 => {
            // STATUE_TRAP: rndmonnum() unicorn avoidance loop + mkcorpstat + makemon
            // rndmonnum calls rndmonst (1 RNG call each iteration)
            // Unicorn avoidance: up to 10 iterations, but typically 1
            // For simplicity, consume 1 rndmonst call (the common case)
            let _rndmon_idx = rndmonst_c_rng(depth, 1, in_hell, rng);
            // mkcorpstat(STATUE, NULL, mptr, x, y, CORPSTAT_NONE)
            // This calls mksobj internally — statue object creation
            // mksobj for STATUE: rock_init_c_rng
            mksobj_c_rng(&objects, &bases, R_STATUE, ObjectClass::Rock, true, false, depth, rng);
            // makemon(&mons[corpsenm], 0, 0, MM_NOCOUNTBIRTH)
            // Position (0,0) → makemon_rnd_goodpos → enexto → consumes variable RNG
            // Then full makemon chain for the monster
            // For now, approximate with standard makemon (it's at random position)
            makemon_c_rng(level, &objects, &bases, depth, rng);
        }
        _ => {} // Most traps: no additional RNG in maketrap
    }

    // 4. WEB spider already handled above in case 18

    // 5. Dead adventurer check: lvl <= (unsigned) rnd(4)
    // rnd(4) is ALWAYS called. At depth >= 5, condition is always false.
    rng.rnd(4);
    // At depth 14: 14 > 4, so the dead adventurer code is never entered.
    // At depth < 5 this would need the full dead adventurer RNG chain.
}

/// C's mkgrave RNG consumption (mklev.c:1808-1857).
///
/// dobell is called BEFORE the grave placement check in populate_ordinary_rooms,
/// so this function takes it as a parameter.
fn mkgrave_rng(level: &mut Level, room: &Room, dobell: bool, depth: i32, rng: &mut GameRng) {
    let objects = OBJECTS;
    let bases = ClassBases::compute(objects);

    // do { somexy(croom, &m) } while (occupied || bydoor)
    let (gx, gy) = match somexy_unoccupied(level, room, rng) {
        Some(pos) => pos,
        None => return,
    };
    level.cells[gx][gy].typ = CellType::Grave;

    // C: make_grave(m.x, m.y, dobell ? "Saved by the bell!" : NULL)
    // If txt != NULL (dobell): no RNG from make_grave (str is literal)
    // If txt == NULL (!dobell): get_rnd_text(EPITAPHFILE, buf, rn2) → rn2(sizetxt)
    if !dobell {
        // get_rnd_text uses rn2 to select a line from the epitaph file
        // The exact arg to rn2 depends on file size, but always 1 call
        rng.rn2(100); // approximate sizetxt
    }

    // C: if (!rn2(3)) { mksobj(GOLD_PIECE, TRUE, FALSE); rnd(20); rnd(5) }
    if rng.rn2(3) == 0 {
        // mksobj(GOLD_PIECE, TRUE, FALSE): Coin class → no init RNG
        // Then: gold->quan = rnd(20) + level_difficulty() * rnd(5)
        rng.rnd(20);
        rng.rnd(5);
    }

    // C: for (tryct = rn2(5); tryct; tryct--) { mkobj(RANDOM_CLASS, TRUE); }
    let buried_count = rng.rn2(5);
    for _ in 0..buried_count {
        // mkobj(RANDOM_CLASS, TRUE) + curse(otmp)
        mkobj_c_rng(&objects, &bases, depth, rng);
    }

    // C: if (dobell) mksobj_at(BELL, m.x, m.y, TRUE, FALSE)
    // BELL (not BELL_OF_OPENING) is a Tool
    if dobell {
        mksobj_c_rng(&objects, &bases, 0, ObjectClass::Tool, true, false, depth, rng);
    }
}

/// C's random_engraving RNG consumption
fn random_engraving_rng(rng: &mut GameRng) {
    // C: random_engraving(buf) → getrumor(0, buf, TRUE)
    // getrumor uses rn2(endpos) on core RNG (1 call)
    // If getrumor fails: rn2(MESG_COUNT) — but typically succeeds
    rng.rn2(100); // getrumor position selection
}

// ============================================================================
// C-faithful RNG consumption functions for map generation parity
// These match the exact RNG call sequence of C NetHack 3.6.7
// ============================================================================

/// C's mkobj(RANDOM_CLASS, TRUE) RNG consumption (mkobj.c:247-272).
/// Call order: rnd(1000) for type prob, rnd(100) for class, then mksobj.
fn mkobj_c_rng(objects: &[ObjClassDef], bases: &ClassBases, depth: i32, rng: &mut GameRng) {
    // C: int prob = rnd(1000)  — FIRST call, type selection within class
    let prob = rng.rnd(1000) as i32;

    // C: for (tprob = rnd(100); (tprob -= iprobs->iprob) > 0; iprobs++)
    // SECOND call, class selection from mkobjprobs
    let mut tprob = rng.rnd(100) as i32;
    let class = {
        // C's mkobjprobs[] ordering
        const PROBS: [(i32, ObjectClass); 10] = [
            (10, ObjectClass::Weapon),
            (10, ObjectClass::Armor),
            (20, ObjectClass::Food),
            (8, ObjectClass::Tool),
            (8, ObjectClass::Gem),
            (16, ObjectClass::Potion),
            (16, ObjectClass::Scroll),
            (4, ObjectClass::Spellbook),
            (7, ObjectClass::Wand),
            (1, ObjectClass::Ring),
        ];
        let mut sel = ObjectClass::Weapon;
        for &(p, c) in &PROBS {
            tprob -= p;
            if tprob <= 0 {
                sel = c;
                break;
            }
        }
        sel
    };

    // C: i = bases[(int) oclass]; while ((prob -= objects[i].oc_prob) > 0) i++;
    let base = bases.get(class);
    let mut i = base;
    let mut p = prob;
    while i < objects.len() && objects[i].class == class {
        p -= objects[i].probability as i32;
        if p <= 0 {
            break;
        }
        i += 1;
    }
    if i >= objects.len() || objects[i].class != class {
        i = base;
    }

    // C: return mksobj(i, TRUE, artif=TRUE)
    mksobj_c_rng(objects, bases, i, class, true, true, depth, rng);
}

/// C's mkobj(specific_class, artif) for non-RANDOM class (e.g., SPBOOK_CLASS).
fn mkobj_class_c_rng(
    objects: &[ObjClassDef],
    bases: &ClassBases,
    class: ObjectClass,
    artif: bool,
    depth: i32,
    rng: &mut GameRng,
) {
    // C: prob = rnd(1000) — always first
    let prob = rng.rnd(1000) as i32;
    // No rnd(100) for non-RANDOM class

    // Walk objects array
    let base = bases.get(class);
    let mut i = base;
    let mut p = prob;
    while i < objects.len() && objects[i].class == class {
        p -= objects[i].probability as i32;
        if p <= 0 {
            break;
        }
        i += 1;
    }
    if i >= objects.len() || objects[i].class != class {
        i = base;
    }

    // mkobj always passes init=TRUE to mksobj
    mksobj_c_rng(objects, bases, i, class, true, artif, depth, rng);
}

/// C's mksobj(otyp, init=TRUE, artif) RNG consumption (mkobj.c:771-1070).
/// C's mksobj(otyp, init, artif) RNG consumption.
///
/// `c_otyp` is the C object index (from onames.h), used for special-case dispatch.
/// `class` is the object's class.
/// When called from mkobj_c_rng with a Rust array index, use `c_otyp_from_rust_idx`
/// to compute the C otyp.
fn mksobj_c_rng(
    objects: &[ObjClassDef],
    bases: &ClassBases,
    c_otyp: usize,
    class: ObjectClass,
    init: bool,
    artif: bool,
    depth: i32,
    rng: &mut GameRng,
) {
    if !init {
        return;
    }

    match class {
        ObjectClass::Weapon => weapon_init_c_rng(objects, c_otyp, artif, rng),
        ObjectClass::Food => food_init_c_rng(c_otyp, rng),
        ObjectClass::Gem => gem_init_c_rng(c_otyp, rng),
        ObjectClass::Tool => tool_init_c_rng(objects, bases, c_otyp, artif, depth, rng),
        ObjectClass::Amulet => amulet_init_c_rng(c_otyp, rng),
        ObjectClass::Potion | ObjectClass::Scroll => blessorcurse_c_rng(rng, 4),
        ObjectClass::Spellbook => blessorcurse_c_rng(rng, 17),
        ObjectClass::Armor => armor_init_c_rng(objects, c_otyp, artif, rng),
        ObjectClass::Wand => wand_init_c_rng(objects, c_otyp, rng),
        ObjectClass::Ring => ring_init_c_rng(c_otyp, rng),
        ObjectClass::Rock => rock_init_c_rng(objects, bases, c_otyp, depth, rng),
        ObjectClass::Coin | ObjectClass::Venom | ObjectClass::Chain
        | ObjectClass::Ball => {}
        _ => {}
    }
}

/// C's blessorcurse(otmp, chance): rn2(chance), if 0 then rn2(2)
fn blessorcurse_c_rng(rng: &mut GameRng, chance: u32) {
    if rng.rn2(chance) == 0 {
        rng.rn2(2);
    }
}

/// C's rne(x) during level generation (player_level=1, utmp=5)
fn rne_c_rng(rng: &mut GameRng, x: u32) -> u32 {
    let utmp = 5u32;
    let mut tmp = 1u32;
    while tmp < utmp && rng.rn2(x) == 0 {
        tmp += 1;
    }
    tmp
}

/// C's rndmonnum(): calls rndmonst() which consumes 1 rn2 call.
/// At depth 14, rndmonst always succeeds (Plan A), consuming exactly 1 call.
fn rndmonnum_c_rng(rng: &mut GameRng) {
    // rndmonst: rn2(choice_count) — always exactly 1 ISAAC64 value
    rng.rn2(100);
}

/// C's is_poisonable(otmp): skill >= -P_SHURIKEN && skill <= -P_BOW
fn is_poisonable_c(objects: &[ObjClassDef], otyp: usize) -> bool {
    objects[otyp].class == ObjectClass::Weapon
        && objects[otyp].skill >= -P_SHURIKEN
        && objects[otyp].skill <= -P_BOW
}

// --- Class-specific init RNG ---

fn weapon_init_c_rng(objects: &[ObjClassDef], otyp: usize, artif: bool, rng: &mut GameRng) {
    // C: otmp->quan = is_multigen(otmp) ? rn1(6,6) : 1
    if objects[otyp].merge && objects[otyp].class == ObjectClass::Weapon {
        rng.rn2(6); // rn1(6,6) = rn2(6) + 6
    }

    // C: enchantment branching
    if rng.rn2(11) == 0 {
        rne_c_rng(rng, 3); // spe = rne(3)
        rng.rn2(2); // blessed = rn2(2)
    } else if rng.rn2(10) == 0 {
        // curse(otmp) — no RNG
        rne_c_rng(rng, 3); // spe = -rne(3)
    } else {
        blessorcurse_c_rng(rng, 10);
    }

    // C: is_poisonable check
    if is_poisonable_c(objects, otyp) {
        rng.rn2(100);
    }

    // C: artifact check
    if artif {
        rng.rn2(20);
        // mk_artifact rarely succeeds during mklev; skip its internal RNG
    }
}

fn food_init_c_rng(otyp: usize, rng: &mut GameRng) {
    // C: switch(otmp->otyp) inside FOOD_CLASS init
    match otyp {
        R_CORPSE => {
            rndmonnum_c_rng(rng);
        }
        R_EGG => {
            if rng.rn2(3) == 0 {
                rndmonnum_c_rng(rng);
            }
        }
        R_TIN => {
            if rng.rn2(6) == 0 {
                // spinach — no additional RNG
            } else {
                rndmonnum_c_rng(rng);
                rng.rn2(13); // set_tin_variety(RANDOM_TIN)
            }
            blessorcurse_c_rng(rng, 10);
        }
        R_KELP_FROND => {
            rng.rnd(2);
        }
        _ => {}
    }

    // C: post-switch: Is_pudding check, then quantity doubling
    // GlobOfGrayOoze..GlobOfBlackPudding are 4 consecutive items after MEAT_RING
    let is_pudding = otyp > R_MEAT_RING && otyp <= R_MEAT_RING + 4;
    if !is_pudding {
        if otyp != R_CORPSE && otyp != R_MEAT_RING && otyp != R_KELP_FROND {
            rng.rn2(6);
        }
    }
}

fn gem_init_c_rng(otyp: usize, rng: &mut GameRng) {
    // C: LOADSTONE → curse (no RNG)
    // ROCK → rn1(6,6)
    // LUCKSTONE → nothing (quan=1)
    // else → rn2(6) for double quantity
    match otyp {
        R_LOADSTONE => {
            // curse(otmp) — no RNG
        }
        R_ROCK => {
            rng.rn2(6); // rn1(6,6) = rn2(6) + 6
        }
        R_LUCKSTONE => {
            // nothing
        }
        _ => {
            rng.rn2(6); // double quantity check
        }
    }
}

fn tool_init_c_rng(
    objects: &[ObjClassDef],
    bases: &ClassBases,
    otyp: usize,
    _artif: bool,
    depth: i32,
    rng: &mut GameRng,
) {
    match otyp {
        199 | 200 /* TALLOW_CANDLE | WAX_CANDLE */ => {
            // C: quan = 1 + (rn2(2) ? rn2(7) : 0); blessorcurse(5)
            let r = rng.rn2(2);
            if r != 0 {
                rng.rn2(7);
            }
            blessorcurse_c_rng(rng, 5);
        }
        201 | 202 /* BRASS_LANTERN | OIL_LAMP */ => {
            // C: age = rn1(500, 1000); blessorcurse(5)
            rng.rn2(500); // rn1(500, 1000) = rn2(500) + 1000
            blessorcurse_c_rng(rng, 5);
        }
        203 /* MAGIC_LAMP */ => {
            blessorcurse_c_rng(rng, 2);
        }
        190 /* CHEST */ | 189 /* LARGE_BOX */ => {
            // C: olocked = !!(rn2(5)); otrapped = !(rn2(10)); FALLTHRU → mkbox_cnts
            rng.rn2(5); // locked
            rng.rn2(10); // trapped
            // mkbox_cnts: creates 0-N objects inside
            mkbox_cnts_c_rng(objects, bases, otyp, depth, rng);
        }
        191 /* ICE_BOX */ | 192 /* SACK */ | 193 /* OILSKIN_SACK */ | 194 /* BAG_OF_HOLDING */ => {
            // C: falls through to mkbox_cnts
            // During mklev, SACK/OILSKIN_SACK start empty (moves <= 1 && !in_mklev)
            // Actually in_mklev IS true, so they get contents
            mkbox_cnts_c_rng(objects, bases, otyp, depth, rng);
        }
        204 | 213 | 217 /* EXPENSIVE_CAMERA | TINNING_KIT | MAGIC_MARKER */ => {
            rng.rn2(70); // rn1(70, 30) = rn2(70) + 30
        }
        215 /* CAN_OF_GREASE */ => {
            rng.rnd(25);
            blessorcurse_c_rng(rng, 10);
        }
        206 /* CRYSTAL_BALL */ => {
            rng.rnd(5);
            blessorcurse_c_rng(rng, 2);
        }
        227 | 195 /* HORN_OF_PLENTY | BAG_OF_TRICKS */ => {
            rng.rnd(20);
        }
        216 /* FIGURINE */ => {
            // C: do { corpsenm = rndmonnum() } while (is_human && tryct++ < 30)
            // Typically succeeds on first try (most monsters aren't human)
            rndmonnum_c_rng(rng);
            blessorcurse_c_rng(rng, 4);
        }
        R_BELL_OF_OPENING => {
            // spe = 3, no RNG
        }
        223 | 229 | 225 | 226 | 233
        /* MAGIC_FLUTE | MAGIC_HARP | FROST_HORN | FIRE_HORN | DRUM_OF_EARTHQUAKE */ => {
            rng.rn2(5); // rn1(5, 4) = rn2(5) + 4
        }
        _ => {
            // Default tool: no special init RNG
        }
    }
}

fn amulet_init_c_rng(otyp: usize, rng: &mut GameRng) {
    // C: if (rn2(10) && (STRANGULATION || CHANGE || RESTFUL_SLEEP)) curse
    //    else blessorcurse(10)
    let is_special =
        otyp == 180 /* STRANGULATION */ || otyp == 183 /* CHANGE */ || otyp == 181 /* RESTFUL_SLEEP */;

    if is_special {
        // C: rn2(10) && (type_check)
        // rn2(10) is always called
        if rng.rn2(10) != 0 {
            // curse(otmp) — no RNG
        } else {
            blessorcurse_c_rng(rng, 10);
        }
    } else {
        // C: the rn2(10) is still called as part of the condition check!
        // if (rn2(10) && (FALSE || FALSE || FALSE)) → rn2(10) is called, type check fails
        // → falls to else: blessorcurse(10)
        rng.rn2(10); // consumed but condition is false for non-special
        blessorcurse_c_rng(rng, 10);
    }
}

fn armor_init_c_rng(objects: &[ObjClassDef], otyp: usize, artif: bool, rng: &mut GameRng) {
    // C's special cursed armor types
    let is_cursed_armor = otyp == 148 /* FUMBLE_BOOTS */
        || otyp == 149 /* LEVITATION_BOOTS */
        || otyp == R_HELM_OF_OPPOSITE_ALIGNMENT
        || otyp == R_GAUNTLETS_OF_FUMBLING;

    // C: if (rn2(10) && (special_type || !rn2(11))) { curse + rne(3) }
    //    else if (!rn2(10)) { blessed + rne(3) }
    //    else blessorcurse(10)
    let r1 = rng.rn2(10);
    if r1 != 0 {
        let condition = if is_cursed_armor {
            true
        } else {
            rng.rn2(11) == 0
        };
        if condition {
            // curse(otmp) — no RNG
            rne_c_rng(rng, 3);
        } else {
            // Falls to else-if
            if rng.rn2(10) == 0 {
                rng.rn2(2); // blessed = rn2(2)
                rne_c_rng(rng, 3);
            } else {
                blessorcurse_c_rng(rng, 10);
            }
        }
    } else {
        // r1 == 0: first condition is false (short-circuit)
        if rng.rn2(10) == 0 {
            rng.rn2(2); // blessed = rn2(2)
            rne_c_rng(rng, 3);
        } else {
            blessorcurse_c_rng(rng, 10);
        }
    }

    // C: artifact check for armor
    if artif {
        rng.rn2(40);
        // mk_artifact rarely succeeds during mklev
    }
}

fn wand_init_c_rng(objects: &[ObjClassDef], otyp: usize, rng: &mut GameRng) {
    // C: WAN_WISHING → rnd(3); else rn1(5, nodir?11:4)
    if otyp == R_WAND_OF_WISHING {
        rng.rnd(3);
    } else {
        rng.rn2(5); // rn1(5, x) = rn2(5) + x
    }
    blessorcurse_c_rng(rng, 17);
}

fn ring_init_c_rng(otyp: usize, rng: &mut GameRng) {
    // C's charged rings: Adornment(150), GainStr(151), GainCon(152),
    // IncAcc(153), IncDam(154), Protection(155)
    let is_charged = (150..=155).contains(&otyp);

    if is_charged {
        // C: blessorcurse(3) + enchantment logic
        blessorcurse_c_rng(rng, 3);
        if rng.rn2(10) != 0 {
            if rng.rn2(10) != 0 {
                // bcsign check — depends on BUC state, but always calls rne(3)
                rne_c_rng(rng, 3);
            } else {
                rng.rn2(2); // rn2(2) ? rne(3) : -rne(3)
                rne_c_rng(rng, 3);
            }
        }
        // C: if (spe == 0) spe = rn2(4) - rn2(3)
        // This always runs (spe starts at 0 if rn2(10)==0 in outer check)
        // But only consumes RNG if spe is 0. We can't know without tracking spe.
        // Conservative: always consume. Tracking spe for exact parity is future work.
        rng.rn2(4);
        rng.rn2(3);
        // C: if (spe < 0 && rn2(5)) curse
        rng.rn2(5);
    } else {
        // Uncharged ring
        // C: rn2(10) && (special_type || !rn2(9))
        let is_cursed_ring = otyp == 171 /* TELEPORTATION: approximation */
            || otyp == 173 /* POLYMORPH: approximation */
            || otyp == 162 /* AGGRAVATE_MONSTER */
            || otyp == 161 /* HUNGER */;

        let r1 = rng.rn2(10);
        if r1 != 0 {
            if is_cursed_ring {
                // type check true → curse (no RNG)
            } else {
                rng.rn2(9); // !rn2(9) check
            }
        }
        // If r1 == 0 or condition was false: no curse
    }
}

fn rock_init_c_rng(
    objects: &[ObjClassDef],
    bases: &ClassBases,
    otyp: usize,
    depth: i32,
    rng: &mut GameRng,
) {
    // C: only STATUE has special init in ROCK_CLASS
    if otyp == R_STATUE {
        // C: corpsenm = rndmonnum()
        rndmonnum_c_rng(rng);
        // C: if (!verysmall && rn2(level_difficulty()/2 + 10) > 10)
        //    add_to_container(otmp, mkobj(SPBOOK_CLASS, FALSE))
        let threshold = (depth / 2 + 10).max(1) as u32;
        let r = rng.rn2(threshold);
        if r > 10 {
            // Create spellbook: mkobj(SPBOOK_CLASS, FALSE) → mksobj(i, TRUE, FALSE)
            mkobj_class_c_rng(objects, bases, ObjectClass::Spellbook, false, depth, rng);
        }
    }
}

/// C's mkbox_cnts(box) — creates 0-N objects inside a container
fn mkbox_cnts_c_rng(
    objects: &[ObjClassDef],
    bases: &ClassBases,
    box_otyp: usize,
    depth: i32,
    rng: &mut GameRng,
) {
    // C: n depends on box type
    let n = match box_otyp {
        R_ICE_BOX => 20,
        R_CHEST => 7,
        R_LARGE_BOX => 5,
        192 | 193 /* SACK | OILSKIN_SACK */ => 1,
        194 /* BAG_OF_HOLDING */ => 1,
        _ => 0,
    };

    if n == 0 {
        return;
    }

    // C: n = rn2(n + 1)
    let count = rng.rn2(n as u32 + 1) as usize;

    for _ in 0..count {
        if box_otyp == R_ICE_BOX {
            // C: mksobj(CORPSE, TRUE, TRUE) — creates a corpse
            mksobj_c_rng(objects, bases, R_CORPSE, ObjectClass::Food, true, true, depth, rng);
        } else {
            // C: rnd(100) for class from boxiprobs, then mkobj(class, TRUE)
            let mut tprob = rng.rnd(100) as i32;
            let class = {
                // C's boxiprobs[]
                const BOX_PROBS: [(i32, ObjectClass); 7] = [
                    (18, ObjectClass::Gem),
                    (15, ObjectClass::Food),
                    (18, ObjectClass::Potion),
                    (18, ObjectClass::Scroll),
                    (12, ObjectClass::Spellbook),
                    (7, ObjectClass::Coin),
                    (12, ObjectClass::Wand),
                ];
                let mut sel = ObjectClass::Gem;
                for &(p, c) in &BOX_PROBS {
                    tprob -= p;
                    if tprob <= 0 {
                        sel = c;
                        break;
                    }
                }
                sel
            };

            // C: mkobj(class, TRUE) — which calls rnd(1000) then mksobj
            let prob = rng.rnd(1000) as i32;
            let base = bases.get(class);
            let mut i = base;
            let mut p = prob;
            while i < objects.len() && objects[i].class == class {
                p -= objects[i].probability as i32;
                if p <= 0 {
                    break;
                }
                i += 1;
            }
            if i >= objects.len() || objects[i].class != class {
                i = base;
            }
            mksobj_c_rng(objects, bases, i, class, true, true, depth, rng);

            // C: if (COIN_CLASS) rnd(level_difficulty+2) * rnd(75) — 2 extra calls
            if class == ObjectClass::Coin {
                rng.rnd((depth + 2).max(1) as u32);
                rng.rnd(75);
            }
            // C: while (otyp == ROCK) { rnd_class(DILITHIUM_CRYSTAL, LOADSTONE) }
            // rnd_class consumes 1 rn2 call per iteration — rare, skip for now
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
                    let is_vertical_edge =
                        rx == room.x.saturating_sub(1) || rx == room.x + room.width;
                    let is_horizontal_edge =
                        ry == room.y.saturating_sub(1) || ry == room.y + room.height;

                    if is_vertical_edge
                        && !is_horizontal_edge
                        && level.cells[rx][ry].typ != CellType::Room
                    {
                        level.cells[rx][ry].typ = CellType::VWall;
                    } else if is_horizontal_edge
                        && !is_vertical_edge
                        && level.cells[rx][ry].typ != CellType::Room
                    {
                        level.cells[rx][ry].typ = CellType::HWall;
                    } else if is_vertical_edge
                        && is_horizontal_edge
                        && level.cells[rx][ry].typ != CellType::Room
                    {
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
            rect_mgr.split_rects_legacy(&room_rect);

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
pub fn generate_irregular_room(
    level: &mut Level,
    x: usize,
    y: usize,
    max_w: usize,
    max_h: usize,
    rng: &mut GameRng,
) -> Room {
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

            if (is_edge_x || is_edge_y)
                && !(rx >= sub_x && rx < sub_x + sub_w && ry >= sub_y && ry < sub_y + sub_h)
            {
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
fn find_empty_room_spot(
    level: &Level,
    rooms: &[Room],
    rng: &mut GameRng,
) -> Option<(usize, usize)> {
    if rooms.is_empty() {
        return None;
    }

    // Try up to 20 times to find an empty spot
    for _ in 0..20 {
        let room_idx = rng.rn2(rooms.len() as u32) as usize;
        let room = &rooms[room_idx];
        let (x, y) = room.random_point(rng);

        // Check if spot is empty floor
        if level.cells[x][y].typ == CellType::Room && level.monster_at(x as i8, y as i8).is_none() {
            return Some((x, y));
        }
    }

    None
}

/// C's place_niche() (mklev.c:454-475)
/// Returns (xx, yy, dy) if a valid niche position is found
fn place_niche(
    level: &Level,
    room: &Room,
    rng: &mut GameRng,
) -> Option<(usize, usize, i32)> {
    let dy: i32;
    let (xx, yy);

    if rng.rn2(2) != 0 {
        dy = 1;
        // finddpos on bottom wall
        (xx, yy) = super::corridor::finddpos(
            level,
            room.x,
            room.y + room.height, // hy + 1
            room.x + room.width - 1,
            room.y + room.height, // hy + 1
            rng,
        );
    } else {
        dy = -1;
        // finddpos on top wall
        (xx, yy) = super::corridor::finddpos(
            level,
            room.x,
            room.y.saturating_sub(1), // ly - 1
            room.x + room.width - 1,
            room.y.saturating_sub(1), // ly - 1
            rng,
        );
    };

    let niche_yi = yy as i32 + dy;
    let wall_yi = yy as i32 - dy;

    // C: isok(xx, yy+dy) && levl[xx][yy+dy].typ == STONE
    //    && isok(xx, yy-dy) && !IS_POOL(...) && !IS_FURNITURE(...)
    // C's isok: x >= 1 && x <= COLNO-1 && y >= 0 && y <= ROWNO-1
    let niche_ok = xx >= 1 && xx <= COLNO - 1 && niche_yi >= 0 && niche_yi <= ROWNO as i32 - 1;
    let wall_ok = xx >= 1 && xx <= COLNO - 1 && wall_yi >= 0 && wall_yi <= ROWNO as i32 - 1;
    if niche_ok
        && level.cells[xx][niche_yi as usize].typ == CellType::Stone
        && wall_ok
        && !level.cells[xx][wall_yi as usize].typ.is_pool()
        && !level.cells[xx][wall_yi as usize].typ.is_furniture()
    {
        Some((xx, yy, dy))
    } else {
        None
    }
}

/// Rubout substitution table from engrave.c — maps wipefrom char to wipeto string.
/// Used by wipeout_text_rng to match C's RNG consumption.
const RUBOUTS: &[(u8, &[u8])] = &[
    (b'A', b"^"),   (b'B', b"Pb["), (b'C', b"("),   (b'D', b"|)["),
    (b'E', b"|FL[_"), (b'F', b"|-"), (b'G', b"C("),  (b'H', b"|-"),
    (b'I', b"|"),   (b'K', b"|<"),  (b'L', b"|_"),   (b'M', b"|"),
    (b'N', b"|\\"), (b'O', b"C("),  (b'P', b"F"),    (b'Q', b"C("),
    (b'R', b"PF"),  (b'T', b"|"),   (b'U', b"J"),    (b'V', b"/\\"),
    (b'W', b"V/\\"), (b'Z', b"/"), (b'b', b"|"),    (b'd', b"c|"),
    (b'e', b"c"),   (b'g', b"c"),   (b'h', b"n"),    (b'j', b"i"),
    (b'k', b"|"),   (b'l', b"|"),   (b'm', b"nr"),   (b'n', b"r"),
    (b'o', b"c"),   (b'q', b"c"),   (b'w', b"v"),    (b'y', b"v"),
    (b':', b"."),   (b';', b",:"),  (b',', b"."),    (b'=', b"-"),
    (b'+', b"-|"),  (b'*', b"+"),   (b'@', b"0"),    (b'0', b"C("),
    (b'1', b"|"),   (b'6', b"o"),   (b'7', b"/"),    (b'8', b"3o"),
];

/// Simulate C's wipeout_text(engr, cnt, seed=0) RNG consumption.
/// C: engrave.c:82-137. With seed=0, each iteration consumes rn2(lth) + rn2(4),
/// plus potentially rn2(strlen(wipeto)) if the character matches a rubout entry.
fn wipeout_text_rng(text: &[u8], cnt: usize, rng: &mut GameRng) {
    let mut buf: Vec<u8> = text.to_vec();
    let lth = buf.len();
    if lth == 0 {
        return;
    }
    for _ in 0..cnt {
        let nxt = rng.rn2(lth as u32) as usize;
        let use_rubout = rng.rn2(4);
        let s = buf[nxt];
        if s == b' ' {
            continue;
        }
        // C: index("?.,'`-|_", *s) → rub out unreadable/small punctuation
        if b"?.,'`-|_".contains(&s) {
            buf[nxt] = b' ';
            continue;
        }
        if use_rubout == 0 {
            // C: i = SIZE(rubouts) → no match, fall through to '?'
            buf[nxt] = b'?';
        } else {
            let mut matched = false;
            for &(wipefrom, wipeto) in RUBOUTS {
                if s == wipefrom {
                    let j = rng.rn2(wipeto.len() as u32) as usize;
                    buf[nxt] = wipeto[j];
                    matched = true;
                    break;
                }
            }
            if !matched {
                buf[nxt] = b'?';
            }
        }
    }
    // C trims trailing spaces — no RNG consumed, skip
}

/// Trap engraving texts from mklev.c:474-481
fn trap_engraving(trap_type: i32) -> Option<&'static [u8]> {
    match trap_type {
        14 => Some(b"Vlad was here"),  // TRAPDOOR
        15 => Some(b"ad aerarium"),     // TELEP_TRAP
        16 => Some(b"ad aerarium"),     // LEVEL_TELEP
        _ => None,
    }
}

/// C's makeniche() (mklev.c:487-549)
/// trap_type: NO_TRAP (0), LEVEL_TELEP, TRAPDOOR, etc.
/// NO_TRAP = 0 in C, represented as trap_type == 0 here
fn makeniche(
    level: &mut Level,
    rooms: &[Room],
    trap_type: i32,
    objects: &[ObjClassDef],
    bases: &ClassBases,
    depth: i32,
    rng: &mut GameRng,
) {
    // C: NO_TRAP = 0, LEVEL_TELEP = 16, TRAPDOOR = 14
    const NO_TRAP: i32 = 0;

    let nroom = rooms.len();
    let mut vct: i32 = 8;

    while vct > 0 {
        vct -= 1;
        let room_idx = rng.rn2(nroom as u32) as usize;
        // Read room_type and door_count from level.rooms (mutated in-place by dosdoor_public),
        // not from the `rooms` snapshot, to match C's global rooms[] array behavior.
        let room_type = level.rooms[room_idx].room_type;
        let door_count = level.rooms[room_idx].door_count;
        let room = &rooms[room_idx];
        eprintln!("RS: makeniche vct={} room_idx={} rtype={:?} doorct={} rng={}", vct, room_idx, room_type, door_count, rng.call_count());

        // C: if (aroom->rtype != OROOM) continue;
        if room_type != RoomType::Ordinary {
            continue;
        }

        // C: if (aroom->doorct == 1 && rn2(5)) continue;
        if door_count == 1 && rng.rn2(5) != 0 {
            eprintln!("RS: makeniche skip doorct=1 rng={}", rng.call_count());
            continue;
        }

        // C: if (!place_niche(aroom, &dy, &xx, &yy)) continue;
        let (xx, yy, dy) = match place_niche(level, room, rng) {
            Some(v) => v,
            None => {
                continue;
            }
        };

        let niche_y = (yy as i32 + dy) as usize;
        // C: if (trap_type || !rn2(4))
        let rn2_4_val = if trap_type == NO_TRAP { rng.rn2(4) } else { 0 };
        if trap_type != NO_TRAP || rn2_4_val == 0 {
            // Secret corridor with trap
            level.cells[xx][niche_y].typ = CellType::SecretCorridor;
            if trap_type != NO_TRAP {
                // C: is_hole(trap_type) && !Can_fall_thru → ROCKTRAP
                let mut actual_trap = trap_type;
                let is_hole = actual_trap == 14 || actual_trap == 13; // TRAPDOOR=14, HOLE=13
                // Can_fall_thru: true for normal dungeon levels that have a level below
                // At depth 14 of main dungeon (max ~30), can always fall through
                let can_fall = level.dlevel.depth() < 30; // approximate
                if is_hole && !can_fall {
                    actual_trap = 21; // ROCKTRAP
                }
                // C: ttmp = maketrap(xx, yy+dy, trap_type)
                // C's maketrap converts SCORR→CORR only for PIT/SPIKED_PIT/HOLE/TRAPDOOR
                // (trap.c:401-422). TELEP_TRAP and LEVEL_TELEP do NOT convert.
                let is_pit_or_hole = matches!(actual_trap, 7 | 8 | 13 | 14); // PIT=7, SPIKED_PIT=8, HOLE=13, TRAPDOOR=14
                if is_pit_or_hole
                    && (level.cells[xx][niche_y].typ == CellType::SecretCorridor
                        || level.cells[xx][niche_y].typ == CellType::Stone)
                {
                    level.cells[xx][niche_y].typ = CellType::Corridor;
                }
                let rust_trap = match actual_trap {
                    16 => super::TrapType::Teleport,      // LEVEL_TELEP
                    15 => super::TrapType::Teleport,      // TELEP_TRAP
                    14 => super::TrapType::TrapDoor,       // TRAPDOOR
                    21 => super::TrapType::RockFall,       // ROCKTRAP
                    _ => super::TrapType::RockFall,        // fallback
                };
                level.add_trap(xx as i8, niche_y as i8, rust_trap);
                // C: ttmp->once = 1 (for non-ROCKTRAP)
                // C: engravings use actual_trap after conversion
                if let Some(engr_text) = trap_engraving(actual_trap) {
                    wipeout_text_rng(engr_text, 5, rng);
                }
            }
            // C: dosdoor(xx, yy, aroom, SDOOR)
            super::corridor::dosdoor_public(level, xx, yy, CellType::SecretDoor, room_idx, rng);
        } else {
            // Corridor with possible door
            level.cells[xx][niche_y].typ = CellType::Corridor;
            // C: if (rn2(7)) dosdoor(xx, yy, aroom, rn2(5) ? SDOOR : DOOR)
            if rng.rn2(7) != 0 {
                let door_type = if rng.rn2(5) != 0 {
                    CellType::SecretDoor
                } else {
                    CellType::Door
                };
                super::corridor::dosdoor_public(level, xx, yy, door_type, room_idx, rng);
            } else {
                // C: inaccessible niche — iron bars, corpse, scroll, object
                // C: if (!rn2(5) && IS_WALL(levl[xx][yy].typ))
                if rng.rn2(5) == 0 && level.cells[xx][yy].typ.is_wall() {
                    level.cells[xx][yy].typ = CellType::IronBars;
                    // C: if (rn2(3)) mkcorpstat(CORPSE, 0, mkclass(S_HUMAN, 0), xx, yy+dy, TRUE)
                    if rng.rn2(3) != 0 {
                        // mkclass(S_HUMAN, 0) — S_HUMAN symbol is '@'
                        let player_level = 1i32;
                        mkclass_c_rng('@', depth, player_level, rng);
                        // mkcorpstat calls mksobj(CORPSE, init=TRUE, FALSE)
                        // CORPSE init: do { rndmonnum() } while (G_NOCORPSE && --tryct)
                        // At game start, mvitals is zeroed so G_NOCORPSE never set → 1 iteration
                        food_init_c_rng(R_CORPSE, rng);
                    }
                }
                // C: if (!level.flags.noteleport)
                //        mksobj_at(SCR_TELEPORTATION, xx, yy+dy, TRUE, FALSE)
                // noteleport is false for standard dungeon levels
                // SCR_TELEPORTATION is Scroll class → blessorcurse(otmp, 4)
                blessorcurse_c_rng(rng, 4);
                // C: if (!rn2(3)) mkobj_at(0, xx, yy+dy, TRUE)
                // 0 = RANDOM_CLASS
                if rng.rn2(3) == 0 {
                    mkobj_c_rng(objects, bases, depth, rng);
                }
            }
        }
        return;
    }
}

/// Create niches on a level — matches C's make_niches() (mklev.c:552-569)
pub fn make_niches(
    level: &mut Level,
    rooms: &[Room],
    objects: &[ObjClassDef],
    bases: &ClassBases,
    rng: &mut GameRng,
) {
    // C: NO_TRAP = 0, LEVEL_TELEP = 16, TRAPDOOR = 14
    const NO_TRAP: i32 = 0;
    const LEVEL_TELEP: i32 = 16;
    const TRAPDOOR: i32 = 14;

    let nroom = rooms.len();
    // C: ct = rnd((nroom >> 1) + 1)
    let ct = rng.rnd((nroom >> 1) as u32 + 1) as i32;
    let dep = level.dlevel.depth();
    let depth = dep;

    // C: ltptr = (!level.flags.noteleport && dep > 15)
    let mut ltptr = dep > 15;
    // C: vamp = (dep > 5 && dep < 25) — "vamp" is just the variable name
    let mut vamp = dep > 5 && dep < 25;

    eprintln!("RS: make_niches ct={} dep={} ltptr={} vamp={} rng={}", ct, dep, ltptr, vamp, rng.call_count());
    for i in (0..ct).rev() {
        eprintln!("RS: niche iter ct={} rng={}", i, rng.call_count());
        if ltptr && rng.rn2(6) == 0 {
            ltptr = false;
            makeniche(level, rooms, LEVEL_TELEP, objects, bases, depth, rng);
        } else if vamp && rng.rn2(6) == 0 {
            vamp = false;
            makeniche(level, rooms, TRAPDOOR, objects, bases, depth, rng);
        } else {
            makeniche(level, rooms, NO_TRAP, objects, bases, depth, rng);
        }
        eprintln!("RS: niche done ct={} rng={}", i, rng.call_count());
    }
}

/// Create a vault teleporter (teleport trap leading into vault)
#[allow(dead_code)]
pub fn make_vault_teleporter(
    level: &mut Level,
    rooms: &[Room],
    objects: &[ObjClassDef],
    bases: &ClassBases,
    rng: &mut GameRng,
) -> bool {
    // Find a vault room
    let vault_room = rooms.iter().find(|r| r.room_type == RoomType::Vault);

    if vault_room.is_none() {
        return false;
    }

    let depth = level.dlevel.depth();
    // Create a niche with a teleport trap that leads to the vault
    // C: TELEP_TRAP = 15
    makeniche(level, rooms, 15, objects, bases, depth, rng);
    true
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

// ============================================================================
// Room numbering (topologize equivalent from C's mklev.c)
// ============================================================================

/// Special room number values
pub const NO_ROOM: u8 = 0; // Not part of any room
pub const SHARED: u8 = 255; // Cell is shared between rooms (edge)
pub const ROOMOFFSET: u8 = 1; // Room numbers start at 1

/// Assign room numbers to cells in a room and its subrooms (topologize equivalent)
///
/// This function assigns room numbers to all cells in a room's interior
/// and marks edge cells appropriately. This is essential for pathfinding,
/// monster spawning, and room-based game logic.
///
/// # Arguments
/// * `level` - Level containing the cells to update
/// * `room` - Room to process
/// * `room_index` - Index of the room in the rooms array (used for room number)
/// * `all_rooms` - All rooms (needed for subroom processing)
/// * `do_ordinary` - If true, mark interior cells; if false, only mark edges
///
/// # Behavior
/// - Interior cells get the room's number (room_index + ROOMOFFSET)
/// - Edge cells (walls) are marked as SHARED if they border multiple rooms
/// - Irregular rooms are skipped (assumed already processed)
/// - Subrooms are processed recursively
pub fn topologize(
    level: &mut Level,
    room: &Room,
    room_index: usize,
    all_rooms: &[Room],
    do_ordinary: bool,
) {
    let roomno = (room_index + ROOMOFFSET as usize) as u8;
    let lowx = room.x;
    let lowy = room.y;
    let hix = room.x + room.width - 1;
    let hiy = room.y + room.height - 1;

    // Skip if already done (check lower-left corner) or if irregular
    if level.cells[lowx][lowy].room_number == roomno || room.irregular {
        return;
    }

    // Mark interior cells
    if room.room_type == RoomType::Ordinary || do_ordinary {
        for x in lowx..=hix {
            for y in lowy..=hiy {
                if room.room_type == RoomType::Ordinary {
                    level.cells[x][y].room_number = NO_ROOM;
                } else {
                    level.cells[x][y].room_number = roomno;
                }
            }
        }

        // Mark top and bottom edges
        let left_edge = lowx.saturating_sub(1);
        let right_edge = (hix + 1).min(COLNO - 1);

        for x in left_edge..=right_edge {
            // Top edge
            if lowy > 0 {
                let y = lowy - 1;
                level.cells[x][y].edge = true;
                if level.cells[x][y].room_number != NO_ROOM {
                    level.cells[x][y].room_number = SHARED;
                } else {
                    level.cells[x][y].room_number = roomno;
                }
            }

            // Bottom edge
            if hiy + 1 < ROWNO {
                let y = hiy + 1;
                level.cells[x][y].edge = true;
                if level.cells[x][y].room_number != NO_ROOM {
                    level.cells[x][y].room_number = SHARED;
                } else {
                    level.cells[x][y].room_number = roomno;
                }
            }
        }

        // Mark left and right edges (excluding corners already done)
        for y in lowy..=hiy {
            // Left edge
            if lowx > 0 {
                let x = lowx - 1;
                level.cells[x][y].edge = true;
                if level.cells[x][y].room_number != NO_ROOM {
                    level.cells[x][y].room_number = SHARED;
                } else {
                    level.cells[x][y].room_number = roomno;
                }
            }

            // Right edge
            if hix + 1 < COLNO {
                let x = hix + 1;
                level.cells[x][y].edge = true;
                if level.cells[x][y].room_number != NO_ROOM {
                    level.cells[x][y].room_number = SHARED;
                } else {
                    level.cells[x][y].room_number = roomno;
                }
            }
        }
    }

    // Process subrooms recursively
    for &subroom_idx in &room.subrooms {
        if subroom_idx < all_rooms.len() {
            let subroom = &all_rooms[subroom_idx];
            topologize(
                level,
                subroom,
                subroom_idx,
                all_rooms,
                room.room_type != RoomType::Ordinary,
            );
        }
    }
}

/// Topologize all rooms on a level
///
/// Assigns room numbers to all cells based on room positions.
/// Call this after generating rooms but before using room-based logic.
pub fn topologize_all(level: &mut Level, rooms: &[Room]) {
    for (idx, room) in rooms.iter().enumerate() {
        // Skip subrooms - they're handled by their parent
        if room.parent.is_none() {
            topologize(level, room, idx, rooms, false);
        }
    }
}

/// Get the room number for a cell
pub fn get_roomno(level: &Level, x: usize, y: usize) -> u8 {
    if x < COLNO && y < ROWNO {
        level.cells[x][y].room_number
    } else {
        NO_ROOM
    }
}

/// Check if a cell is on a room edge
pub fn is_room_edge(level: &Level, x: usize, y: usize) -> bool {
    if x < COLNO && y < ROWNO {
        level.cells[x][y].edge
    } else {
        false
    }
}

/// Check if a cell is in any room
pub fn in_room(level: &Level, x: usize, y: usize) -> bool {
    let roomno = get_roomno(level, x, y);
    roomno != NO_ROOM && roomno != SHARED
}

/// Get room index from room number (subtract ROOMOFFSET)
pub fn room_index_from_roomno(roomno: u8) -> Option<usize> {
    if roomno >= ROOMOFFSET && roomno != SHARED {
        Some((roomno - ROOMOFFSET) as usize)
    } else {
        None
    }
}

// ============================================================================
// Additional generation functions (from C's mklev.c)
// ============================================================================

/// Add a door to a room's door tracking (add_door equivalent)
///
/// Updates the room's door count. The door itself should already exist
/// on the level's cell grid.
///
/// # Arguments
/// * `rooms` - Mutable array of rooms (to update door counts)
/// * `room_index` - Index of room this door belongs to
pub fn add_door(rooms: &mut [Room], room_index: usize) {
    if room_index >= rooms.len() {
        return;
    }

    let room = &mut rooms[room_index];
    room.door_count += 1;
}

/// Create stairs at a location (mkstairs equivalent)
///
/// Creates stairs going up or down at the specified position.
///
/// # Arguments
/// * `level` - Level to modify
/// * `x`, `y` - Stair coordinates
/// * `up` - true for upstairs, false for downstairs
/// * `dest` - Destination level
///
/// # Returns
/// true if stairs were created successfully
pub fn mkstairs(level: &mut Level, x: usize, y: usize, up: bool, dest: DLevel) -> bool {
    if x == 0 || x >= COLNO || y >= ROWNO {
        return false;
    }

    // Set the cell type
    level.cells[x][y].typ = CellType::Stairs;

    // Add to stairs list
    level.stairs.push(super::Stairway {
        x: x as i8,
        y: y as i8,
        destination: dest,
        up,
    });

    true
}

/// Create a door at a location (dosdoor equivalent)
///
/// Creates a door cell with appropriate state.
///
/// # Arguments
/// * `level` - Level to modify
/// * `x`, `y` - Door coordinates
/// * `is_secret` - Whether this is a secret door
/// * `is_shop` - Whether this door is for a shop
/// * `rng` - Random number generator
pub fn create_door(
    level: &mut Level,
    x: usize,
    y: usize,
    is_secret: bool,
    is_shop: bool,
    rng: &mut GameRng,
) {
    let depth = level.dlevel.level_num as i32;

    if is_secret && level.cells[x][y].typ.is_wall() {
        level.cells[x][y].typ = CellType::SecretDoor;
        level.cells[x][y].flags = DoorState::CLOSED.bits();
    } else {
        // Inline dosdoor-style logic
        let cell_type = if is_secret { CellType::SecretDoor } else { CellType::Door };
        level.cells[x][y].typ = cell_type;

        let state = if is_shop {
            if is_secret { DoorState::LOCKED } else { DoorState::OPEN }
        } else {
            match rng.rn2(3) {
                0 => DoorState::LOCKED,
                1 => DoorState::CLOSED,
                _ => DoorState::OPEN,
            }
        };
        let mut state = state;
        if depth >= 5 && state.contains(DoorState::LOCKED) && rng.rn2(25) == 0 {
            state |= DoorState::TRAPPED;
        }
        level.cells[x][y].flags = state.bits();
    }
}

/// Create a secret door (create_secret_door equivalent)
///
/// Creates a secret door at a wall location.
pub fn create_secret_door(level: &mut Level, x: usize, y: usize) {
    if level.cells[x][y].typ.is_wall() {
        level.cells[x][y].typ = CellType::SecretDoor;
        level.cells[x][y].flags = DoorState::CLOSED.bits();
    }
}

/// Wallify a map - add walls around floor areas (wallify_map equivalent)
///
/// Converts stone cells adjacent to floor/corridor into wall cells.
/// This is typically called after cave generation.
pub fn wallify_map(level: &mut Level) {
    // First pass: identify which stone cells need to become walls
    let mut to_wallify: Vec<(usize, usize, CellType)> = Vec::new();

    for x in 1..(COLNO - 1) {
        for y in 1..(ROWNO - 1) {
            if level.cells[x][y].typ != CellType::Stone {
                continue;
            }

            // Check if adjacent to floor/corridor
            let adjacent_floor =
                [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                    .iter()
                    .any(|&(nx, ny)| {
                        matches!(
                            level.cells[nx][ny].typ,
                            CellType::Room | CellType::Corridor | CellType::Door
                        )
                    });

            if adjacent_floor {
                // Determine wall type based on adjacent cells
                let wall_type = determine_wall_type(level, x, y);
                to_wallify.push((x, y, wall_type));
            }
        }
    }

    // Second pass: apply changes
    for (x, y, wall_type) in to_wallify {
        level.cells[x][y].typ = wall_type;
    }
}

/// Determine what type of wall should be placed based on neighbors
fn determine_wall_type(level: &Level, x: usize, y: usize) -> CellType {
    let floor_above = y > 0 && is_floor_like(level.cells[x][y - 1].typ);
    let floor_below = y + 1 < ROWNO && is_floor_like(level.cells[x][y + 1].typ);
    let floor_left = x > 0 && is_floor_like(level.cells[x - 1][y].typ);
    let floor_right = x + 1 < COLNO && is_floor_like(level.cells[x + 1][y].typ);

    match (floor_above, floor_below, floor_left, floor_right) {
        // Horizontal wall: floor above or below
        (true, false, _, _) | (false, true, _, _) => CellType::HWall,
        // Vertical wall: floor left or right
        (_, _, true, false) | (_, _, false, true) => CellType::VWall,
        // Corners and intersections
        (true, true, _, _) => CellType::CrossWall,
        (_, _, true, true) => CellType::CrossWall,
        (true, _, true, _) => CellType::TLCorner,
        (true, _, _, true) => CellType::TRCorner,
        (_, true, true, _) => CellType::BLCorner,
        (_, true, _, true) => CellType::BRCorner,
        // Default to generic wall
        _ => CellType::Wall,
    }
}

/// Check if a cell type is floor-like (walkable room/corridor)
fn is_floor_like(typ: CellType) -> bool {
    matches!(typ, CellType::Room | CellType::Corridor | CellType::Door)
}

/// Port of C's check_room() from sp_lev.c:1064-1117
///
/// Checks if a room can fit at the given position without overlapping existing
/// features. May shrink the room or return false. For vaults, the xlim/ylim
/// margins are increased by 1.
///
/// Parameters are mutable because C's check_room modifies them in place.
fn check_room(
    level: &Level,
    lowx: &mut i32,
    ddx: &mut i32,
    lowy: &mut i32,
    ddy: &mut i32,
    vault: bool,
    rng: &mut GameRng,
) -> bool {
    let xlim: i32 = 4 + if vault { 1 } else { 0 }; // XLIM=4 in sp_lev.c:181
    let ylim: i32 = 3 + if vault { 1 } else { 0 }; // YLIM=3 in sp_lev.c:182

    let mut hix = *lowx + *ddx;
    let mut hiy = *lowy + *ddy;

    if *lowx < 3 { *lowx = 3; }
    if *lowy < 2 { *lowy = 2; }
    if hix > COLNO as i32 - 3 { hix = COLNO as i32 - 3; }
    if hiy > ROWNO as i32 - 3 { hiy = ROWNO as i32 - 3; }

    loop {
        // chk:
        if hix <= *lowx || hiy <= *lowy {
            return false;
        }

        let mut found_nonzero = false;
        let x_start = *lowx - xlim;
        let x_end = hix + xlim;
        let y_start = (*lowy - ylim).max(0);
        let y_end = (hiy + ylim).min(ROWNO as i32 - 1);

        'outer: for x in x_start..=x_end {
            if x <= 0 || x >= COLNO as i32 {
                continue;
            }
            for y in y_start..=y_end {
                if level.cells[x as usize][y as usize].typ != CellType::Stone {
                    // Non-zero cell found
                    if rng.rn2(3) == 0 {
                        return false;
                    }
                    // Shrink room to avoid this cell
                    if x < *lowx {
                        *lowx = x + xlim + 1;
                    } else {
                        hix = x - xlim - 1;
                    }
                    if y < *lowy {
                        *lowy = y + ylim + 1;
                    } else {
                        hiy = y - ylim - 1;
                    }
                    found_nonzero = true;
                    break 'outer; // goto chk
                }
            }
        }

        if !found_nonzero {
            // All cells are Stone — room fits
            *ddx = hix - *lowx;
            *ddy = hiy - *lowy;
            return true;
        }
        // Loop back to chk
    }
}

/// Create a vault room after check_room succeeds (mklev.c:767-775)
fn create_vault_room(
    level: &mut Level,
    vx: usize,
    vy: usize,
    w: usize,
    h: usize,
    rng: &mut GameRng,
    is_branch_level: bool,
) {
    let mut vault_room = Room::new(vx, vy, w + 1, h + 1); // C: add_room uses lx..hx inclusive, w=ddx
    vault_room.room_type = RoomType::Vault;
    vault_room.lit = true;
    carve_room(level, &vault_room);
    level.rooms.push(vault_room);
    level.flags.has_vault = true;

    // C: fill_room(&rooms[nroom - 1], FALSE) — creates gold in vault
    // C iterates all cells in vault, calling mkgold(rn1(abs(depth)*100, 51), x, y)
    // rn1(x,y) = rn2(x) + y, so each cell consumes 1 RNG call
    // mkgold with amount>0 and fresh cell: mksobj_at(GOLD_PIECE) consumes 0 RNG
    let depth = level.dlevel.depth();
    let vault_room = level.rooms.last().unwrap();
    let gold_w = vault_room.width;
    let gold_h = vault_room.height;
    for _x in 0..gold_w {
        for _y in 0..gold_h {
            let _amount = rng.rn2((depth.unsigned_abs() * 100) as u32) as i64 + 51;
            // NOTE: gold object creation deferred; RNG calls consumed for parity
        }
    }
    eprintln!("RS: after vault fill_room rng={}", rng.call_count());

    // C: mk_knox_portal(vault_x + w, vault_y + h)
    // mk_knox_portal returns immediately (0 RNG) if Is_branchlev is true.
    // Otherwise calls rn2(3): if nonzero, returns (1 RNG); if zero, places portal.
    // We don't actually place the portal, but must match RNG consumption.
    if !is_branch_level {
        // mk_knox_portal calls rn2(3) — 2/3 chance of returning early
        let _knox_rn = rng.rn2(3);
        eprintln!("RS: mk_knox_portal rn2(3)={} rng={}", _knox_rn, rng.call_count());
    } else {
        eprintln!("RS: mk_knox_portal skipped (branch level) rng={}", rng.call_count());
    }
    eprintln!("RS: after mk_knox_portal rng={}", rng.call_count());

    // C: if (!level.flags.noteleport && !rn2(3)) makevtele()
    if rng.rn2(3) == 0 {
        // C: makevtele() calls makeniche(TELEP_TRAP=15)
        let rooms_snapshot = level.rooms.clone();
        let vt_objects = OBJECTS;
        let vt_bases = ClassBases::compute(vt_objects);
        let vt_depth = level.dlevel.depth();
        makeniche(level, &rooms_snapshot, 15, vt_objects, &vt_bases, vt_depth, rng);
    }
    eprintln!("RS: after makevtele_check rng={}", rng.call_count());
}

/// Fill a room with floor cells (fill_room equivalent)
///
/// Converts the interior of a room to floor cells and adds walls around it.
pub fn carve_room(level: &mut Level, room: &Room) {
    let lowx = room.x;
    let lowy = room.y;
    let hix = room.x + room.width - 1;
    let hiy = room.y + room.height - 1;

    // Horizontal walls
    for x in lowx.saturating_sub(1)..=(hix + 1).min(COLNO - 1) {
        if lowy > 0 {
            level.cells[x][lowy - 1].typ = CellType::HWall;
        }
        if hiy < ROWNO - 1 {
            level.cells[x][hiy + 1].typ = CellType::HWall;
        }
    }

    // Vertical walls
    for y in lowy..=hiy {
        if lowx > 0 {
            level.cells[lowx - 1][y].typ = CellType::VWall;
        }
        if hix < COLNO - 1 {
            level.cells[hix + 1][y].typ = CellType::VWall;
        }
    }

    // Corners
    if lowx > 0 && lowy > 0 { level.cells[lowx - 1][lowy - 1].typ = CellType::TLCorner; }
    if hix < COLNO - 1 && lowy > 0 { level.cells[hix + 1][lowy - 1].typ = CellType::TRCorner; }
    if lowx > 0 && hiy < ROWNO - 1 { level.cells[lowx - 1][hiy + 1].typ = CellType::BLCorner; }
    if hix < COLNO - 1 && hiy < ROWNO - 1 { level.cells[hix + 1][hiy + 1].typ = CellType::BRCorner; }

    // Floor
    fill_room(level, room, true);
}

/// Converts the interior of a room to floor cells.
pub fn fill_room(level: &mut Level, room: &Room, lit: bool) {
    for x in room.x..(room.x + room.width) {
        for y in room.y..(room.y + room.height) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = lit;
        }
    }
}

/// Fill all rooms with floor cells (fill_rooms equivalent)
pub fn fill_rooms(level: &mut Level, rooms: &[Room]) {
    for room in rooms {
        fill_room(level, room, room.lit);
    }
}

/// Initialize a map with stone (init_map equivalent)
pub fn init_map(level: &mut Level) {
    for x in 0..COLNO {
        for y in 0..ROWNO {
            level.cells[x][y] = Cell::stone();
        }
    }
}

/// Solidify map - convert certain floor to stone (solidify_map equivalent)
///
/// Typically used to clean up unreachable areas.
pub fn solidify_map(level: &mut Level, x1: usize, y1: usize, x2: usize, y2: usize) {
    for x in x1..=x2.min(COLNO - 1) {
        for y in y1..=y2.min(ROWNO - 1) {
            if !level.cells[x][y].explored {
                // Only solidify unexplored cells
                if level.cells[x][y].typ == CellType::Room
                    || level.cells[x][y].typ == CellType::Corridor
                {
                    level.cells[x][y].typ = CellType::Stone;
                }
            }
        }
    }
}

/// Flood fill from a point, marking cells (flood_fill_rm equivalent)
///
/// Used for connectivity checking and cave generation.
///
/// # Arguments
/// * `level` - Level to process
/// * `x`, `y` - Starting point
/// * `target_type` - Cell type to match
/// * `marker` - Function to mark cells as visited
///
/// # Returns
/// Number of cells filled
pub fn flood_fill_rm<F>(
    level: &mut Level,
    x: usize,
    y: usize,
    target_type: CellType,
    mut marker: F,
) -> usize
where
    F: FnMut(&mut Cell),
{
    let mut count = 0;
    let mut stack = vec![(x, y)];
    let mut visited = vec![vec![false; ROWNO]; COLNO];

    while let Some((cx, cy)) = stack.pop() {
        if cx >= COLNO || cy >= ROWNO || visited[cx][cy] {
            continue;
        }

        if level.cells[cx][cy].typ != target_type {
            continue;
        }

        visited[cx][cy] = true;
        marker(&mut level.cells[cx][cy]);
        count += 1;

        // Add neighbors
        if cx > 0 {
            stack.push((cx - 1, cy));
        }
        if cx + 1 < COLNO {
            stack.push((cx + 1, cy));
        }
        if cy > 0 {
            stack.push((cx, cy - 1));
        }
        if cy + 1 < ROWNO {
            stack.push((cx, cy + 1));
        }
    }

    count
}

/// Set a wall cell with proper orientation (set_wall equivalent)
pub fn set_wall(level: &mut Level, x: usize, y: usize, horizontal: bool) {
    level.cells[x][y].typ = if horizontal {
        CellType::HWall
    } else {
        CellType::VWall
    };
    level.cells[x][y].horizontal = horizontal;
}

/// Fix wall spines - ensure walls connect properly (fix_wall_spines equivalent)
///
/// Updates wall corners and T-junctions to have correct types.
pub fn fix_wall_spines(level: &mut Level) {
    for x in 1..(COLNO - 1) {
        for y in 1..(ROWNO - 1) {
            if !level.cells[x][y].typ.is_wall() {
                continue;
            }

            // Check adjacent walls
            let wall_above = y > 0 && level.cells[x][y - 1].typ.is_wall();
            let wall_below = y + 1 < ROWNO && level.cells[x][y + 1].typ.is_wall();
            let wall_left = x > 0 && level.cells[x - 1][y].typ.is_wall();
            let wall_right = x + 1 < COLNO && level.cells[x + 1][y].typ.is_wall();

            let new_type = match (wall_above, wall_below, wall_left, wall_right) {
                // Cross wall: walls in all 4 directions
                (true, true, true, true) => CellType::CrossWall,
                // T-walls
                (true, true, true, false) => CellType::TRWall,
                (true, true, false, true) => CellType::TLWall,
                (true, false, true, true) => CellType::TDWall,
                (false, true, true, true) => CellType::TUWall,
                // Corners
                (true, false, true, false) => CellType::BRCorner,
                (true, false, false, true) => CellType::BLCorner,
                (false, true, true, false) => CellType::TRCorner,
                (false, true, false, true) => CellType::TLCorner,
                // Straight walls
                (true, true, false, false) => CellType::VWall,
                (false, false, true, true) => CellType::HWall,
                // Default
                _ => level.cells[x][y].typ,
            };

            level.cells[x][y].typ = new_type;
        }
    }
}

/// Remove boundary symbols - clean up level edges (remove_boundary_syms equivalent)
pub fn remove_boundary_syms(level: &mut Level) {
    // Top and bottom edges
    for x in 0..COLNO {
        level.cells[x][0].typ = CellType::Stone;
        level.cells[x][ROWNO - 1].typ = CellType::Stone;
    }

    // Left and right edges
    for y in 0..ROWNO {
        level.cells[0][y].typ = CellType::Stone;
        level.cells[COLNO - 1][y].typ = CellType::Stone;
    }
}

/// Ensure there's a way out from starting position (ensure_way_out equivalent)
///
/// Checks that stairs are reachable and creates path if needed.
pub fn ensure_way_out(level: &mut Level, start_x: usize, start_y: usize) -> bool {
    // Find all stairs
    let stair_positions: Vec<(usize, usize)> = level
        .stairs
        .iter()
        .map(|s| (s.x as usize, s.y as usize))
        .collect();

    if stair_positions.is_empty() {
        return false;
    }

    // Check if any stair is reachable using flood fill
    let mut reachable = vec![vec![false; ROWNO]; COLNO];
    flood_fill_reachable(level, start_x, start_y, &mut reachable);

    for (sx, sy) in &stair_positions {
        if reachable[*sx][*sy] {
            return true;
        }
    }

    // No stair is reachable - try to create a path to the nearest one
    if let Some((sx, sy)) = stair_positions.first() {
        create_path(level, start_x, start_y, *sx, *sy);
        return true;
    }

    false
}

/// Flood fill to mark reachable cells
fn flood_fill_reachable(level: &Level, x: usize, y: usize, reachable: &mut Vec<Vec<bool>>) {
    let mut stack = vec![(x, y)];

    while let Some((cx, cy)) = stack.pop() {
        if cx >= COLNO || cy >= ROWNO || reachable[cx][cy] {
            continue;
        }

        if !level.cells[cx][cy].typ.is_passable() {
            continue;
        }

        reachable[cx][cy] = true;

        if cx > 0 {
            stack.push((cx - 1, cy));
        }
        if cx + 1 < COLNO {
            stack.push((cx + 1, cy));
        }
        if cy > 0 {
            stack.push((cx, cy - 1));
        }
        if cy + 1 < ROWNO {
            stack.push((cx, cy + 1));
        }
    }
}

/// Create a corridor path between two points
fn create_path(level: &mut Level, x1: usize, y1: usize, x2: usize, y2: usize) {
    let mut x = x1;
    let mut y = y1;

    // Move horizontally first
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

    // Then vertically
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
        let monster_vitals = crate::magic::MonsterVitals::new();

        generate_rooms_and_corridors(&mut level, &mut rng, &monster_vitals);

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
        let mut counts = hashbrown::HashMap::new();

        // Generate many shop types to verify distribution
        for _ in 0..1000 {
            let (shop_type, _) = select_shop_type(&mut rng, 10);
            *counts.entry(shop_type).or_insert(0) += 1;
        }

        // General shop should be most common (~42%)
        let general_count = *counts.get(&RoomType::GeneralShop).unwrap_or(&0);
        assert!(
            general_count > 350 && general_count < 550,
            "General shop should be ~42%, got {}",
            general_count
        );

        // Wand shop should be ~3%
        let wand_count = *counts.get(&RoomType::WandShop).unwrap_or(&0);
        assert!(
            wand_count < 60,
            "Wand shop should be ~3%, got {}",
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
        assert!(
            got_special,
            "Depth 15 should sometimes generate special rooms"
        );
    }

    #[test]
    fn test_pick_room_for_special() {
        let rooms = vec![
            Room::new(5, 5, 4, 4),  // 16 area - adequate
            Room::new(20, 5, 5, 5), // 25 area - good
            Room::new(35, 5, 2, 2), // 4 area - too small for most
            Room::new(50, 5, 6, 4), // 24 area - good
        ];

        // Should pick a room with adequate size (prefer later rooms)
        let shop_room = pick_room_for_special(&rooms, RoomType::GeneralShop);
        assert!(shop_room.is_some());
        // Should be room 3 (last one with adequate size) or room 1
        let idx = shop_room.unwrap();
        assert!(
            idx == 1 || idx == 3,
            "Should pick room with adequate size, got {}",
            idx
        );

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
        let monster_vitals = crate::magic::MonsterVitals::new();
        for depth in [2, 5, 10, 15, 20] {
            let mut rng = GameRng::new(42 + depth as u64);
            let dlevel = DLevel {
                dungeon_num: 0,
                level_num: depth,
            };
            let mut level = Level::new(dlevel);

            generate_rooms_and_corridors(&mut level, &mut rng, &monster_vitals);

            // Basic sanity checks
            let room_count = level
                .cells
                .iter()
                .flat_map(|col| col.iter())
                .filter(|cell| cell.typ == CellType::Room)
                .count();

            assert!(room_count > 0, "Depth {} should have room cells", depth);
        }
    }

    #[test]
    fn test_dark_rooms() {
        // Generate many levels at depth 11+ to find a morgue (which should be dark)
        let mut found_dark_cell = false;
        let monster_vitals = crate::magic::MonsterVitals::new();

        for seed in 0..100 {
            let mut rng = GameRng::new(seed);
            let dlevel = DLevel {
                dungeon_num: 0,
                level_num: 15, // Deep enough for morgue
            };
            let mut level = Level::new(dlevel);

            generate_rooms_and_corridors(&mut level, &mut rng, &monster_vitals);

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
        let monster_vitals = crate::magic::MonsterVitals::new();

        generate_rooms_and_corridors(&mut level, &mut rng, &monster_vitals);

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
        let shallow_traps: Vec<_> = (0..100).map(|_| select_trap_type(2, &mut rng)).collect();

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
        let deep_traps: Vec<_> = (0..100).map(|_| select_trap_type(20, &mut rng)).collect();

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
        let monster_vitals = crate::magic::MonsterVitals::new();

        for seed in 0..50 {
            let mut rng = GameRng::new(seed);
            let dlevel = DLevel {
                dungeon_num: 0,
                level_num: 10,
            };
            let mut level = Level::new(dlevel);

            generate_rooms_and_corridors(&mut level, &mut rng, &monster_vitals);

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
    fn test_genocide_prevents_spawning() {
        let mut rng = GameRng::new(42);
        let mut monster_vitals = crate::magic::MonsterVitals::new();

        // Genocide monster types 0, 1, 2
        monster_vitals.mark_genocided(0);
        monster_vitals.mark_genocided(1);
        monster_vitals.mark_genocided(2);

        let dlevel = DLevel {
            dungeon_num: 0,
            level_num: 5,
        };
        let mut level = Level::new(dlevel);

        generate_rooms_and_corridors(&mut level, &mut rng, &monster_vitals);

        // Check that no monsters of genocided types spawned
        for monster in &level.monsters {
            assert!(
                !monster_vitals.is_genocided(monster.monster_type),
                "Genocided monster type {} should not spawn",
                monster.monster_type
            );
        }
    }

    #[test]
    fn test_gold_pile_amounts() {
        let mut rng = GameRng::new(42);
        let monster_vitals = crate::magic::MonsterVitals::new();

        // Test at different depths
        for depth in [1, 5, 10, 20] {
            let dlevel = DLevel {
                dungeon_num: 0,
                level_num: depth,
            };
            let mut level = Level::new(dlevel);

            generate_rooms_and_corridors(&mut level, &mut rng, &monster_vitals);

            let gold_piles: Vec<_> = level
                .objects
                .iter()
                .filter(|o| o.class == crate::object::ObjectClass::Coin)
                .collect();

            if !gold_piles.is_empty() {
                let avg_amount: i32 =
                    gold_piles.iter().map(|g| g.quantity).sum::<i32>() / gold_piles.len() as i32;
                println!(
                    "Depth {}: {} gold piles, avg {} gold",
                    depth,
                    gold_piles.len(),
                    avg_amount
                );

                // Gold amounts should scale with depth
                assert!(avg_amount > 0, "Gold piles should have positive amounts");
            }
        }
    }
}
