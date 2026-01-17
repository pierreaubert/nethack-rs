//! Quest system (quest.c, quest.h)
//!
//! Implements role-specific quests with nemesis monsters and quest artifacts.

use serde::{Deserialize, Serialize};

use crate::player::Role;
use crate::rng::GameRng;

use super::cell::CellType;
use super::level::{Level, Stairway, TrapType};
use super::DLevel;

/// Map dimensions
const COLNO: usize = 80;
const ROWNO: usize = 21;

/// Quest status tracking
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct QuestStatus {
    /// Has player been given the quest?
    pub got_quest: bool,
    /// Has player completed the quest?
    pub completed: bool,
    /// Has player obtained the quest artifact?
    pub got_artifact: bool,
    /// Has player killed the nemesis?
    pub killed_nemesis: bool,
    /// Quest leader alive?
    pub leader_alive: bool,
    /// Nemesis alive?
    pub nemesis_alive: bool,
    /// Number of times player entered quest
    pub times_entered: u8,
}

impl QuestStatus {
    pub fn new() -> Self {
        Self {
            leader_alive: true,
            nemesis_alive: true,
            ..Default::default()
        }
    }
}

/// Quest information for a role
#[derive(Debug, Clone)]
pub struct QuestInfo {
    /// Role this quest is for
    pub role: Role,
    /// Quest leader name
    pub leader_name: &'static str,
    /// Quest nemesis name
    pub nemesis_name: &'static str,
    /// Quest artifact name
    pub artifact_name: &'static str,
    /// Quest goal level name
    pub goal_name: &'static str,
    /// Home level name
    pub home_name: &'static str,
    /// Intermediate level name
    pub locate_name: &'static str,
}

impl QuestInfo {
    /// Get quest info for a role
    pub fn for_role(role: Role) -> Self {
        match role {
            Role::Archeologist => Self {
                role,
                leader_name: "Lord Carnarvon",
                nemesis_name: "the Minion of Huhetotl",
                artifact_name: "the Orb of Detection",
                goal_name: "the tomb of the Toltec kings",
                home_name: "the College of Archeology",
                locate_name: "the Tomb of the Toltec Kings",
            },
            Role::Barbarian => Self {
                role,
                leader_name: "Pelias",
                nemesis_name: "Thoth Amon",
                artifact_name: "the Heart of Ahriman",
                goal_name: "the Subterranean Temple",
                home_name: "the Camp of the Duali Tribe",
                locate_name: "the Subterranean Temple",
            },
            Role::Caveman => Self {
                role,
                leader_name: "Shaman Karnov",
                nemesis_name: "Chromatic Dragon",
                artifact_name: "the Sceptre of Might",
                goal_name: "the Dragon's Lair",
                home_name: "the Caves of the Ancestors",
                locate_name: "the Dragon's Lair",
            },
            Role::Healer => Self {
                role,
                leader_name: "Hippocrates",
                nemesis_name: "Cyclops",
                artifact_name: "the Staff of Aesculapius",
                goal_name: "the Temple of Epidaurus",
                home_name: "the Temple of Coeus",
                locate_name: "the Temple of Epidaurus",
            },
            Role::Knight => Self {
                role,
                leader_name: "King Arthur",
                nemesis_name: "Ixoth",
                artifact_name: "the Magic Mirror of Merlin",
                goal_name: "the Isle of Glass",
                home_name: "Camelot Castle",
                locate_name: "the Isle of Glass",
            },
            Role::Monk => Self {
                role,
                leader_name: "Grand Master",
                nemesis_name: "Master Kaen",
                artifact_name: "the Eyes of the Overworld",
                goal_name: "the Monastery of Chan-Sune",
                home_name: "the Monastery of Chan-Sune",
                locate_name: "the Monastery of Chan-Sune",
            },
            Role::Priest => Self {
                role,
                leader_name: "the High Priest",
                nemesis_name: "Nalzok",
                artifact_name: "the Mitre of Holiness",
                goal_name: "the Temple of Moloch",
                home_name: "the Great Temple",
                locate_name: "the Temple of Moloch",
            },
            Role::Ranger => Self {
                role,
                leader_name: "Orion",
                nemesis_name: "Scorpius",
                artifact_name: "the Longbow of Diana",
                goal_name: "the Cave of Scorpius",
                home_name: "Orion's camp",
                locate_name: "the Cave of Scorpius",
            },
            Role::Rogue => Self {
                role,
                leader_name: "Master of Thieves",
                nemesis_name: "Master Assassin",
                artifact_name: "the Master Key of Thievery",
                goal_name: "the Assassin's Guild",
                home_name: "the Thieves' Guild Hall",
                locate_name: "the Assassin's Guild",
            },
            Role::Samurai => Self {
                role,
                leader_name: "Lord Sato",
                nemesis_name: "Ashikaga Takauji",
                artifact_name: "the Tsurugi of Muramasa",
                goal_name: "the Shogun's Castle",
                home_name: "the Castle of the Taro Clan",
                locate_name: "the Shogun's Castle",
            },
            Role::Tourist => Self {
                role,
                leader_name: "Twoflower",
                nemesis_name: "the Master of Thieves",
                artifact_name: "the Platinum Yendorian Express Card",
                goal_name: "Thieves' Guild Hall",
                home_name: "Ankh-Morpork",
                locate_name: "Thieves' Guild Hall",
            },
            Role::Valkyrie => Self {
                role,
                leader_name: "Norn",
                nemesis_name: "Lord Surtur",
                artifact_name: "the Orb of Fate",
                goal_name: "the Caves of Muspelheim",
                home_name: "the Shrine of Destiny",
                locate_name: "the Caves of Muspelheim",
            },
            Role::Wizard => Self {
                role,
                leader_name: "Neferet the Green",
                nemesis_name: "the Dark One",
                artifact_name: "the Eye of the Aethiopica",
                goal_name: "the Tower of Darkness",
                home_name: "the Lonely Tower",
                locate_name: "the Tower of Darkness",
            },
        }
    }
}

/// Generate quest home level (level 1)
pub fn generate_quest_home(level: &mut Level, role: Role, rng: &mut GameRng) {
    let quest_info = QuestInfo::for_role(role);
    
    fill_level(level, CellType::Stone);
    
    // Create home base - a large room with the quest leader
    let home_x = 20;
    let home_y = 5;
    let home_w = 40;
    let home_h = 11;
    
    // Carve the home area
    for x in home_x..(home_x + home_w) {
        for y in home_y..(home_y + home_h) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = true;
        }
    }
    
    // Add some internal structure based on role
    match role {
        Role::Knight => {
            // Castle-like structure
            add_castle_features(level, home_x, home_y, home_w, home_h, rng);
        }
        Role::Monk => {
            // Monastery with meditation areas
            add_monastery_features(level, home_x, home_y, home_w, home_h, rng);
        }
        Role::Priest => {
            // Temple with altar
            level.cells[home_x + home_w / 2][home_y + home_h / 2].typ = CellType::Altar;
        }
        Role::Valkyrie => {
            // Shrine with throne
            level.cells[home_x + home_w / 2][home_y + 2].typ = CellType::Throne;
        }
        _ => {
            // Generic home with fountains
            level.cells[home_x + 5][home_y + home_h / 2].typ = CellType::Fountain;
            level.cells[home_x + home_w - 5][home_y + home_h / 2].typ = CellType::Fountain;
        }
    }
    
    // Stairs down to quest levels
    level.cells[home_x + home_w - 3][home_y + home_h / 2].typ = CellType::Stairs;
    level.stairs.push(Stairway {
        x: (home_x + home_w - 3) as i8,
        y: (home_y + home_h / 2) as i8,
        destination: DLevel::new(4, 2), // Quest level 2
        up: false,
    });
    
    // Stairs up back to main dungeon
    level.cells[home_x + 2][home_y + home_h / 2].typ = CellType::Stairs;
    level.stairs.push(Stairway {
        x: (home_x + 2) as i8,
        y: (home_y + home_h / 2) as i8,
        destination: DLevel::new(0, 14), // Back to main dungeon
        up: true,
    });
    
    // Set level name
    level.flags.sokoban_rules = false; // Quest doesn't have sokoban rules
    
    // Mark as quest level
    let _ = quest_info; // Use quest_info for future enhancements
}

/// Generate quest locate level (intermediate levels 2-4)
pub fn generate_quest_locate(level: &mut Level, role: Role, level_num: i8, rng: &mut GameRng) {
    fill_level(level, CellType::Stone);
    
    // Create a maze-like or cavern level
    let is_maze = matches!(role, Role::Wizard | Role::Monk | Role::Rogue);
    
    if is_maze {
        generate_quest_maze(level, rng);
    } else {
        generate_quest_cavern(level, rng);
    }
    
    // Add role-specific monsters and features
    add_quest_monsters(level, role, level_num, rng);
    
    // Stairs
    place_quest_stairs(level, level_num, rng);
}

/// Generate quest goal level (level 5 - nemesis lair)
pub fn generate_quest_goal(level: &mut Level, role: Role, rng: &mut GameRng) {
    let quest_info = QuestInfo::for_role(role);
    
    fill_level(level, CellType::Stone);
    
    // Create the nemesis lair
    let lair_x = 25;
    let lair_y = 5;
    let lair_w = 30;
    let lair_h = 11;
    
    // Carve the lair
    for x in lair_x..(lair_x + lair_w) {
        for y in lair_y..(lair_y + lair_h) {
            level.cells[x][y].typ = CellType::Room;
            level.cells[x][y].lit = false; // Dark lair
        }
    }
    
    // Add role-specific lair features
    match role {
        Role::Valkyrie => {
            // Fire giant lair with lava
            for x in (lair_x + 2)..(lair_x + lair_w - 2) {
                if rng.one_in(4) {
                    level.cells[x][lair_y + lair_h / 2].typ = CellType::Lava;
                }
            }
        }
        Role::Ranger => {
            // Scorpion cave with pools
            for _ in 0..5 {
                let px = lair_x + 2 + rng.rn2((lair_w - 4) as u32) as usize;
                let py = lair_y + 2 + rng.rn2((lair_h - 4) as u32) as usize;
                level.cells[px][py].typ = CellType::Pool;
            }
        }
        Role::Wizard => {
            // Dark tower with magic traps
            for _ in 0..5 {
                let tx = lair_x + 2 + rng.rn2((lair_w - 4) as u32) as usize;
                let ty = lair_y + 2 + rng.rn2((lair_h - 4) as u32) as usize;
                level.add_trap(tx as i8, ty as i8, TrapType::MagicTrap);
            }
        }
        Role::Caveman => {
            // Dragon lair with treasure
            level.cells[lair_x + lair_w / 2][lair_y + lair_h / 2].typ = CellType::Throne;
        }
        _ => {
            // Generic lair with altar
            level.cells[lair_x + lair_w / 2][lair_y + lair_h / 2].typ = CellType::Altar;
        }
    }
    
    // Corridor to entrance
    for x in 5..lair_x {
        level.cells[x][lair_y + lair_h / 2].typ = CellType::Corridor;
    }
    
    // Stairs up only (no escape except back)
    level.cells[5][lair_y + lair_h / 2].typ = CellType::Stairs;
    level.stairs.push(Stairway {
        x: 5,
        y: (lair_y + lair_h / 2) as i8,
        destination: DLevel::new(4, 4),
        up: true,
    });
    
    let _ = quest_info;
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

fn add_castle_features(level: &mut Level, x: usize, y: usize, w: usize, h: usize, _rng: &mut GameRng) {
    // Add throne
    level.cells[x + w / 2][y + 2].typ = CellType::Throne;
    
    // Add internal walls for rooms
    for ry in (y + 3)..(y + h - 3) {
        level.cells[x + w / 3][ry].typ = CellType::VWall;
        level.cells[x + 2 * w / 3][ry].typ = CellType::VWall;
    }
    
    // Doors in internal walls
    level.cells[x + w / 3][y + h / 2].typ = CellType::Door;
    level.cells[x + 2 * w / 3][y + h / 2].typ = CellType::Door;
}

fn add_monastery_features(level: &mut Level, x: usize, y: usize, w: usize, h: usize, _rng: &mut GameRng) {
    // Central meditation area
    let cx = x + w / 2;
    let cy = y + h / 2;
    
    // Fountain in center
    level.cells[cx][cy].typ = CellType::Fountain;
    
    // Surrounding pillars
    for dx in [-2i32, 2] {
        for dy in [-2i32, 2] {
            let px = (cx as i32 + dx) as usize;
            let py = (cy as i32 + dy) as usize;
            level.cells[px][py].typ = CellType::Stone;
        }
    }
}

fn generate_quest_maze(level: &mut Level, rng: &mut GameRng) {
    // Simple maze generation
    let start_x = 5;
    let start_y = 3;
    let end_x = COLNO - 5;
    let end_y = ROWNO - 3;
    
    // Carve corridors
    let mut x = start_x;
    let mut y = start_y;
    
    while x < end_x || y < end_y {
        level.cells[x][y].typ = CellType::Corridor;
        level.cells[x][y].lit = true;
        
        if rng.one_in(2) && x < end_x {
            x += 1;
        } else if y < end_y {
            y += 1;
        } else if x < end_x {
            x += 1;
        }
        
        // Add some branches
        if rng.one_in(5) {
            let branch_len = 3 + rng.rn2(5) as usize;
            let dir = rng.rn2(4);
            for _ in 0..branch_len {
                let (bx, by) = match dir {
                    0 if x > start_x => (x - 1, y),
                    1 if x < end_x => (x + 1, y),
                    2 if y > start_y => (x, y - 1),
                    _ if y < end_y => (x, y + 1),
                    _ => (x, y),
                };
                if bx >= start_x && bx < end_x && by >= start_y && by < end_y {
                    level.cells[bx][by].typ = CellType::Corridor;
                }
            }
        }
    }
}

fn generate_quest_cavern(level: &mut Level, rng: &mut GameRng) {
    // Create irregular cavern rooms
    for _ in 0..8 {
        let rx = 5 + rng.rn2(60) as usize;
        let ry = 3 + rng.rn2(12) as usize;
        let rw = 4 + rng.rn2(8) as usize;
        let rh = 3 + rng.rn2(5) as usize;
        
        for x in rx..(rx + rw).min(COLNO - 2) {
            for y in ry..(ry + rh).min(ROWNO - 2) {
                level.cells[x][y].typ = CellType::Room;
                level.cells[x][y].lit = rng.one_in(3);
            }
        }
    }
    
    // Connect rooms with corridors
    connect_quest_rooms(level, rng);
}

fn connect_quest_rooms(level: &mut Level, rng: &mut GameRng) {
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
    for _ in 0..10 {
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
        if x < x2 { x += 1; } else { x -= 1; }
    }
    while y != y2 {
        if level.cells[x][y].typ == CellType::Stone {
            level.cells[x][y].typ = CellType::Corridor;
        }
        if y < y2 { y += 1; } else { y -= 1; }
    }
}

fn add_quest_monsters(level: &mut Level, role: Role, level_num: i8, rng: &mut GameRng) {
    // Add traps based on role and level
    let num_traps = (level_num as usize) + rng.rn2(3) as usize;
    
    let trap_types = match role {
        Role::Wizard => vec![TrapType::MagicTrap, TrapType::Teleport, TrapType::AntiMagic],
        Role::Rogue => vec![TrapType::Dart, TrapType::Arrow, TrapType::SleepingGas],
        Role::Valkyrie => vec![TrapType::FireTrap, TrapType::Pit, TrapType::SpikedPit],
        Role::Ranger => vec![TrapType::Arrow, TrapType::BearTrap, TrapType::Pit],
        _ => vec![TrapType::Pit, TrapType::Arrow, TrapType::Dart],
    };
    
    for _ in 0..num_traps {
        for _ in 0..20 {
            let x = 5 + rng.rn2(70) as usize;
            let y = 2 + rng.rn2(17) as usize;
            
            if level.cells[x][y].typ == CellType::Room || level.cells[x][y].typ == CellType::Corridor {
                let trap_idx = rng.rn2(trap_types.len() as u32) as usize;
                level.add_trap(x as i8, y as i8, trap_types[trap_idx]);
                break;
            }
        }
    }
}

fn place_quest_stairs(level: &mut Level, level_num: i8, rng: &mut GameRng) {
    // Find valid positions
    let mut valid: Vec<(usize, usize)> = Vec::new();
    
    for x in 5..(COLNO - 5) {
        for y in 3..(ROWNO - 3) {
            if level.cells[x][y].typ == CellType::Room || level.cells[x][y].typ == CellType::Corridor {
                valid.push((x, y));
            }
        }
    }
    
    if valid.is_empty() {
        return;
    }
    
    // Up stairs
    let up_idx = rng.rn2(valid.len() as u32) as usize;
    let (ux, uy) = valid[up_idx];
    level.cells[ux][uy].typ = CellType::Stairs;
    level.stairs.push(Stairway {
        x: ux as i8,
        y: uy as i8,
        destination: DLevel::new(4, level_num - 1),
        up: true,
    });
    
    // Down stairs (if not goal level)
    if level_num < 5 {
        let mut best_idx = 0;
        let mut best_dist = 0;
        for (i, (x, y)) in valid.iter().enumerate() {
            let dist = ((*x as i32 - ux as i32).abs() + (*y as i32 - uy as i32).abs()) as usize;
            if dist > best_dist {
                best_dist = dist;
                best_idx = i;
            }
        }
        
        let (dx, dy) = valid[best_idx];
        level.cells[dx][dy].typ = CellType::Stairs;
        level.stairs.push(Stairway {
            x: dx as i8,
            y: dy as i8,
            destination: DLevel::new(4, level_num + 1),
            up: false,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_info_all_roles() {
        for role in [
            Role::Archeologist, Role::Barbarian, Role::Caveman, Role::Healer,
            Role::Knight, Role::Monk, Role::Priest, Role::Ranger,
            Role::Rogue, Role::Samurai, Role::Tourist, Role::Valkyrie, Role::Wizard,
        ] {
            let info = QuestInfo::for_role(role);
            assert!(!info.leader_name.is_empty());
            assert!(!info.nemesis_name.is_empty());
            assert!(!info.artifact_name.is_empty());
        }
    }

    #[test]
    fn test_quest_home_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(4, 1));
        
        generate_quest_home(&mut level, Role::Valkyrie, &mut rng);
        
        // Should have room cells
        let room_count = level.cells.iter()
            .flat_map(|col| col.iter())
            .filter(|c| c.typ == CellType::Room)
            .count();
        
        assert!(room_count > 100, "Quest home should have rooms");
        
        // Should have stairs
        assert!(level.stairs.len() >= 2, "Quest home should have up and down stairs");
    }

    #[test]
    fn test_quest_goal_generation() {
        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::new(4, 5));
        
        generate_quest_goal(&mut level, Role::Wizard, &mut rng);
        
        // Should have magic traps
        assert!(!level.traps.is_empty(), "Wizard quest goal should have traps");
        
        // Should have stairs
        assert!(!level.stairs.is_empty(), "Quest goal should have stairs");
    }

    #[test]
    fn test_quest_status() {
        let status = QuestStatus::new();
        
        assert!(status.leader_alive);
        assert!(status.nemesis_alive);
        assert!(!status.got_quest);
        assert!(!status.completed);
    }
}
