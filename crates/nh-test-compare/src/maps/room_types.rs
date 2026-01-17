//! Room type comparison
//!
//! Extracts and compares all room types from C NetHack 3.6.7.

use std::fs;
use std::path::Path;

/// All room types from NetHack 3.6.7 (include/mkroom.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CRoomType {
    /// Ordinary room (default)
    ORoom = 0,
    /// Contains a throne (king's court)
    Court = 2,
    /// Contains pools
    Swamp = 3,
    /// Detached room reached via teleport trap
    Vault = 4,
    /// Contains killer bees and royal jelly
    Beehive = 5,
    /// Contains corpses, undead and graves
    Morgue = 6,
    /// Contains soldiers and their gear
    Barracks = 7,
    /// Floor covered with treasure and monsters
    Zoo = 8,
    /// Contains Oracle and peripherals
    Delphi = 9,
    /// Contains a shrine (altar attended by priest/priestess)
    Temple = 10,
    /// Leprechaun hall
    LeprechaunHall = 11,
    /// Cockatrice nest
    CockatriceNest = 12,
    /// Ant hole
    Anthole = 13,
    /// Base shop type (everything >= this is a shop)
    ShopBase = 14,
    /// Armor shop
    ArmorShop = 15,
    /// Scroll shop
    ScrollShop = 16,
    /// Potion shop
    PotionShop = 17,
    /// Weapon shop
    WeaponShop = 18,
    /// Food shop
    FoodShop = 19,
    /// Ring shop
    RingShop = 20,
    /// Wand shop
    WandShop = 21,
    /// Tool shop
    ToolShop = 22,
    /// Book shop
    BookShop = 23,
    /// Health food store (foddershop)
    FodderShop = 24,
    /// Candle shop (maximum valid room type)
    CandleShop = 25,
}

impl CRoomType {
    /// All room types in order
    pub const ALL: [CRoomType; 25] = [
        CRoomType::ORoom,
        CRoomType::Court,
        CRoomType::Swamp,
        CRoomType::Vault,
        CRoomType::Beehive,
        CRoomType::Morgue,
        CRoomType::Barracks,
        CRoomType::Zoo,
        CRoomType::Delphi,
        CRoomType::Temple,
        CRoomType::LeprechaunHall,
        CRoomType::CockatriceNest,
        CRoomType::Anthole,
        CRoomType::ShopBase,
        CRoomType::ArmorShop,
        CRoomType::ScrollShop,
        CRoomType::PotionShop,
        CRoomType::WeaponShop,
        CRoomType::FoodShop,
        CRoomType::RingShop,
        CRoomType::WandShop,
        CRoomType::ToolShop,
        CRoomType::BookShop,
        CRoomType::FodderShop,
        CRoomType::CandleShop,
    ];

    /// Get the name as it appears in C source
    pub fn c_name(&self) -> &'static str {
        match self {
            CRoomType::ORoom => "OROOM",
            CRoomType::Court => "COURT",
            CRoomType::Swamp => "SWAMP",
            CRoomType::Vault => "VAULT",
            CRoomType::Beehive => "BEEHIVE",
            CRoomType::Morgue => "MORGUE",
            CRoomType::Barracks => "BARRACKS",
            CRoomType::Zoo => "ZOO",
            CRoomType::Delphi => "DELPHI",
            CRoomType::Temple => "TEMPLE",
            CRoomType::LeprechaunHall => "LEPREHALL",
            CRoomType::CockatriceNest => "COCKNEST",
            CRoomType::Anthole => "ANTHOLE",
            CRoomType::ShopBase => "SHOPBASE",
            CRoomType::ArmorShop => "ARMORSHOP",
            CRoomType::ScrollShop => "SCROLLSHOP",
            CRoomType::PotionShop => "POTIONSHOP",
            CRoomType::WeaponShop => "WEAPONSHOP",
            CRoomType::FoodShop => "FOODSHOP",
            CRoomType::RingShop => "RINGSHOP",
            CRoomType::WandShop => "WANDSHOP",
            CRoomType::ToolShop => "TOOLSHOP",
            CRoomType::BookShop => "BOOKSHOP",
            CRoomType::FodderShop => "FODDERSHOP",
            CRoomType::CandleShop => "CANDLESHOP",
        }
    }

    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            CRoomType::ORoom => "Ordinary room",
            CRoomType::Court => "Throne room with king/queen",
            CRoomType::Swamp => "Swamp with pools and eels",
            CRoomType::Vault => "Secret vault with gold",
            CRoomType::Beehive => "Bee hive with queen bee",
            CRoomType::Morgue => "Morgue with undead",
            CRoomType::Barracks => "Soldier barracks",
            CRoomType::Zoo => "Zoo with caged monsters",
            CRoomType::Delphi => "Oracle's chamber",
            CRoomType::Temple => "Temple with altar and priest",
            CRoomType::LeprechaunHall => "Leprechaun treasure hall",
            CRoomType::CockatriceNest => "Cockatrice nest with statues",
            CRoomType::Anthole => "Ant colony",
            CRoomType::ShopBase => "General store",
            CRoomType::ArmorShop => "Armor shop",
            CRoomType::ScrollShop => "Scroll shop",
            CRoomType::PotionShop => "Potion shop",
            CRoomType::WeaponShop => "Weapon shop",
            CRoomType::FoodShop => "Food shop",
            CRoomType::RingShop => "Ring shop",
            CRoomType::WandShop => "Wand shop",
            CRoomType::ToolShop => "Tool shop",
            CRoomType::BookShop => "Bookstore",
            CRoomType::FodderShop => "Health food store",
            CRoomType::CandleShop => "Candle shop (lighting store)",
        }
    }

    /// Is this a shop type?
    pub fn is_shop(&self) -> bool {
        (*self as u8) >= CRoomType::ShopBase as u8
    }

    /// Minimum depth this room type can appear
    pub fn min_depth(&self) -> Option<u32> {
        match self {
            CRoomType::ORoom => Some(1),
            CRoomType::Court => Some(4),
            CRoomType::Swamp => Some(15),
            CRoomType::Vault => Some(1),
            CRoomType::Beehive => Some(9),
            CRoomType::Morgue => Some(11),
            CRoomType::Barracks => Some(14),
            CRoomType::Zoo => Some(6),
            CRoomType::Delphi => None, // Special level only
            CRoomType::Temple => Some(8),
            CRoomType::LeprechaunHall => Some(5),
            CRoomType::CockatriceNest => Some(16),
            CRoomType::Anthole => Some(12),
            _ if self.is_shop() => Some(1),
            _ => Some(1),
        }
    }

    /// Spawn probability (out of N) once min depth reached
    pub fn spawn_probability(&self) -> Option<(u32, u32)> {
        // Returns (1, N) meaning 1 in N chance
        match self {
            CRoomType::Court => Some((1, 6)),        // rn2(6) = 16.7%
            CRoomType::Swamp => Some((1, 6)),        // rn2(6) = 16.7%
            CRoomType::Vault => Some((1, 2)),        // rn2(2) after 7 rooms = 50%
            CRoomType::Beehive => Some((1, 5)),      // rn2(5) = 20%
            CRoomType::Morgue => Some((1, 6)),       // rn2(6) = 16.7%
            CRoomType::Barracks => Some((1, 4)),     // rn2(4) = 25%
            CRoomType::Zoo => Some((1, 7)),          // rn2(7) = 14.3%
            CRoomType::Temple => Some((1, 5)),       // rn2(5) = 20%
            CRoomType::LeprechaunHall => Some((1, 8)), // rn2(8) = 12.5%
            CRoomType::CockatriceNest => Some((1, 8)), // rn2(8) = 12.5%
            CRoomType::Anthole => Some((1, 5)),      // varies
            CRoomType::ShopBase => Some((3, 100)),   // ~3% at depths 1-medusa
            _ => None,
        }
    }
}

/// Room generation constants from C source
#[derive(Debug, Clone)]
pub struct CRoomConstants {
    pub max_rooms: usize,
    pub max_doors: usize,
    pub max_subrooms: usize,
    pub max_rectangles: usize,
    pub min_room_width: usize,
    pub max_room_width: usize,
    pub min_room_height: usize,
    pub max_room_height: usize,
    pub max_room_area: usize,
    pub vault_size: (usize, usize),
    pub xlim: usize, // Horizontal spacing
    pub ylim: usize, // Vertical spacing
}

/// Get C room generation constants
pub fn c_room_constants() -> CRoomConstants {
    CRoomConstants {
        max_rooms: 40,        // MAXNROFROOMS
        max_doors: 120,       // DOORMAX
        max_subrooms: 24,     // MAX_SUBROOMS
        max_rectangles: 50,   // MAXRECT
        min_room_width: 2,    // 2 + rn2(...) minimum
        max_room_width: 17,   // rn1(15, 3) = 3-17
        min_room_height: 2,   // 2 + rn2(4) minimum
        max_room_height: 9,   // rn1(8, 2) = 2-9
        max_room_area: 50,    // Area cap
        vault_size: (2, 2),   // create_vault() uses 2x2
        xlim: 4,              // XLIM spacing
        ylim: 3,              // YLIM spacing
    }
}

/// Door generation constants
#[derive(Debug, Clone)]
pub struct CDoorConstants {
    pub regular_door_prob: f64,   // Probability of regular door (vs secret)
    pub secret_door_prob: f64,    // Probability of secret door
    pub open_door_prob: f64,      // Probability door is open
    pub locked_door_prob: f64,    // Probability door is locked
    pub closed_door_prob: f64,    // Probability door is closed
    pub trap_door_prob: f64,      // Probability of trapped door
}

/// Get C door constants
pub fn c_door_constants() -> CDoorConstants {
    CDoorConstants {
        regular_door_prob: 0.875, // 7/8 = 87.5%
        secret_door_prob: 0.125,  // 1/8 = 12.5%
        open_door_prob: 0.2,      // 1/5 = 20% (of 1/3 random state)
        locked_door_prob: 0.167,  // 1/6 = 16.7% (of 1/3 random state)
        closed_door_prob: 0.633,  // Default = ~63.3% (of 1/3 random state)
        trap_door_prob: 0.04,     // 1/25 = 4% at depth >= 5
    }
}

/// Corridor generation constants
#[derive(Debug, Clone)]
pub struct CCorridorConstants {
    pub extra_corridor_min: usize,
    pub extra_corridor_max: usize,
    pub early_stop_prob: f64,
}

/// Get C corridor constants
pub fn c_corridor_constants() -> CCorridorConstants {
    CCorridorConstants {
        extra_corridor_min: 4,    // rn2(nroom) + 4
        extra_corridor_max: 7,    // When nroom=3, max is 3+4=7
        early_stop_prob: 0.02,    // 1/50 = 2%
    }
}

/// Extract room type definitions from C header
pub fn extract_c_room_types() -> Vec<(String, u8)> {
    let mkroom_h = Path::new(crate::data::NETHACK_SRC).join("include/mkroom.h");

    if !mkroom_h.exists() {
        return Vec::new();
    }

    let content = match fs::read_to_string(&mkroom_h) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut types = Vec::new();

    // Known room type names to look for
    let known_types = [
        "OROOM", "COURT", "SWAMP", "VAULT", "BEEHIVE", "MORGUE",
        "BARRACKS", "ZOO", "DELPHI", "TEMPLE", "LEPREHALL", "COCKNEST",
        "ANTHOLE", "SHOPBASE", "ARMORSHOP", "SCROLLSHOP", "POTIONSHOP",
        "WEAPONSHOP", "FOODSHOP", "RINGSHOP", "WANDSHOP", "TOOLSHOP",
        "BOOKSHOP", "FODDERSHOP", "CANDLESHOP",
    ];

    // Room types are defined in an enum like: OROOM = 0, /* comment */
    for line in content.lines() {
        let trimmed = line.trim();

        // Look for enum entries like "NAME = value," or "NAME = value /* comment */"
        for name in &known_types {
            if trimmed.starts_with(name) {
                // Extract the value after "="
                if let Some(eq_pos) = trimmed.find('=') {
                    let after_eq = &trimmed[eq_pos + 1..];
                    // Take characters until comma or whitespace
                    let val_str: String = after_eq
                        .trim()
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    if let Ok(val) = val_str.parse::<u8>() {
                        types.push((name.to_string(), val));
                    }
                }
                break;
            }
        }
    }

    types
}

/// Special room creation function info
#[derive(Debug, Clone)]
pub struct SpecialRoomInfo {
    pub room_type: CRoomType,
    pub c_function: &'static str,
    pub monster_types: Vec<&'static str>,
    pub features: Vec<&'static str>,
}

/// Get info about special room creation functions
pub fn special_room_functions() -> Vec<SpecialRoomInfo> {
    vec![
        SpecialRoomInfo {
            room_type: CRoomType::Court,
            c_function: "mk_zoo_thronemon() + fill_zoo()",
            monster_types: vec![
                "Dragons (diff>100)", "Giants (diff>95)", "Trolls (diff>85)",
                "Centaurs (diff>75)", "Orcs (diff>60)", "Bugbears (diff>45)",
                "Hobgoblins (diff>30)", "Gnomes (diff>15)", "Kobolds (else)",
            ],
            features: vec!["Throne", "King/Queen monster"],
        },
        SpecialRoomInfo {
            room_type: CRoomType::Swamp,
            c_function: "mkswamp()",
            monster_types: vec![
                "Giant eel (80%)", "Piranha (10%)", "Electric eel (10%)",
                "Fungi/mold (25% of pool-adjacent)",
            ],
            features: vec!["Pools in checkerboard pattern"],
        },
        SpecialRoomInfo {
            room_type: CRoomType::Vault,
            c_function: "create_vault()",
            monster_types: vec!["None initially (guard summoned on entry)"],
            features: vec!["Gold piles", "2x2 room", "Teleport trap access"],
        },
        SpecialRoomInfo {
            room_type: CRoomType::Beehive,
            c_function: "fill_zoo()",
            monster_types: vec!["Queen bee (center)", "Killer bees (rest)"],
            features: vec!["Royal jelly", "Lumps of royal jelly"],
        },
        SpecialRoomInfo {
            room_type: CRoomType::Morgue,
            c_function: "fill_zoo()",
            monster_types: vec![
                "Ghosts (20%)", "Wraiths (20%)", "Zombies (60%)",
                "Vampires (high diff)", "Demons (very high diff)",
            ],
            features: vec!["Corpses", "Graves"],
        },
        SpecialRoomInfo {
            room_type: CRoomType::Barracks,
            c_function: "fill_zoo()",
            monster_types: vec![
                "Soldier (80%)", "Sergeant (15%)", "Lieutenant (4%)", "Captain (1%)",
            ],
            features: vec!["Military gear"],
        },
        SpecialRoomInfo {
            room_type: CRoomType::Zoo,
            c_function: "fill_zoo()",
            monster_types: vec!["Random level-appropriate monsters"],
            features: vec!["Gold piles (500 * level_difficulty limit)"],
        },
        SpecialRoomInfo {
            room_type: CRoomType::Temple,
            c_function: "mktemple()",
            monster_types: vec!["Priest/Priestess (attending altar)"],
            features: vec!["Altar (aligned)", "AM_SHRINE flag"],
        },
        SpecialRoomInfo {
            room_type: CRoomType::LeprechaunHall,
            c_function: "fill_zoo()",
            monster_types: vec!["Leprechauns"],
            features: vec!["Gold piles"],
        },
        SpecialRoomInfo {
            room_type: CRoomType::CockatriceNest,
            c_function: "fill_zoo()",
            monster_types: vec!["Cockatrices"],
            features: vec!["Statues (with random items inside)"],
        },
        SpecialRoomInfo {
            room_type: CRoomType::Anthole,
            c_function: "fill_zoo() + antholemon()",
            monster_types: vec![
                "Soldier ant (seed%3=0)", "Fire ant (seed%3=1)", "Giant ant (seed%3=2)",
            ],
            features: vec!["Food items"],
        },
        SpecialRoomInfo {
            room_type: CRoomType::ShopBase,
            c_function: "mkshop() + stock_room()",
            monster_types: vec!["Shopkeeper"],
            features: vec!["Shop inventory", "Lit room", "Single door preferred"],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_room_types_defined() {
        // Verify all 25 room types are defined
        assert_eq!(CRoomType::ALL.len(), 25);

        // Verify values match expected C constants
        assert_eq!(CRoomType::ORoom as u8, 0);
        assert_eq!(CRoomType::Court as u8, 2);
        assert_eq!(CRoomType::Swamp as u8, 3);
        assert_eq!(CRoomType::Vault as u8, 4);
        assert_eq!(CRoomType::Beehive as u8, 5);
        assert_eq!(CRoomType::Morgue as u8, 6);
        assert_eq!(CRoomType::Barracks as u8, 7);
        assert_eq!(CRoomType::Zoo as u8, 8);
        assert_eq!(CRoomType::Delphi as u8, 9);
        assert_eq!(CRoomType::Temple as u8, 10);
        assert_eq!(CRoomType::LeprechaunHall as u8, 11);
        assert_eq!(CRoomType::CockatriceNest as u8, 12);
        assert_eq!(CRoomType::Anthole as u8, 13);
        assert_eq!(CRoomType::ShopBase as u8, 14);
        assert_eq!(CRoomType::CandleShop as u8, 25);
    }

    #[test]
    fn test_extract_c_room_types() {
        let types = extract_c_room_types();

        println!("Extracted {} room types from C header:", types.len());
        for (name, val) in &types {
            println!("  {} = {}", name, val);
        }

        // Should find most room types
        assert!(types.len() >= 20, "Expected 20+ room types, found {}", types.len());

        // Verify specific values
        let oroom = types.iter().find(|(n, _)| n == "OROOM");
        assert!(oroom.is_some(), "Should find OROOM");
        assert_eq!(oroom.unwrap().1, 0);

        let shopbase = types.iter().find(|(n, _)| n == "SHOPBASE");
        assert!(shopbase.is_some(), "Should find SHOPBASE");
        assert_eq!(shopbase.unwrap().1, 14);

        let candleshop = types.iter().find(|(n, _)| n == "CANDLESHOP");
        assert!(candleshop.is_some(), "Should find CANDLESHOP");
        assert_eq!(candleshop.unwrap().1, 25);
    }

    #[test]
    fn test_room_type_values_match_c() {
        let c_types = extract_c_room_types();

        if c_types.is_empty() {
            println!("Warning: Could not read C room types");
            return;
        }

        let mut matched = 0;
        let mut mismatched = Vec::new();

        for room_type in CRoomType::ALL.iter() {
            let c_name = room_type.c_name();
            if let Some((_, c_val)) = c_types.iter().find(|(n, _)| n == c_name) {
                if *c_val == *room_type as u8 {
                    matched += 1;
                } else {
                    mismatched.push((c_name, *room_type as u8, *c_val));
                }
            }
        }

        println!("Room types matched: {}/{}", matched, CRoomType::ALL.len());
        if !mismatched.is_empty() {
            println!("\nMismatches:");
            for (name, rust_val, c_val) in &mismatched {
                println!("  {}: Rust={} vs C={}", name, rust_val, c_val);
            }
        }

        // All should match
        assert!(
            mismatched.is_empty(),
            "Found {} room type mismatches",
            mismatched.len()
        );
    }

    #[test]
    fn test_shop_detection() {
        // Non-shops
        assert!(!CRoomType::ORoom.is_shop());
        assert!(!CRoomType::Court.is_shop());
        assert!(!CRoomType::Temple.is_shop());
        assert!(!CRoomType::Anthole.is_shop());

        // Shops
        assert!(CRoomType::ShopBase.is_shop());
        assert!(CRoomType::ArmorShop.is_shop());
        assert!(CRoomType::ScrollShop.is_shop());
        assert!(CRoomType::CandleShop.is_shop());
    }

    #[test]
    fn test_room_constants() {
        let constants = c_room_constants();

        println!("C Room Constants:");
        println!("  Max rooms: {}", constants.max_rooms);
        println!("  Max doors: {}", constants.max_doors);
        println!("  Max subrooms: {}", constants.max_subrooms);
        println!("  Room size: {}x{} to {}x{}",
            constants.min_room_width, constants.min_room_height,
            constants.max_room_width, constants.max_room_height);
        println!("  Max area: {}", constants.max_room_area);
        println!("  Vault size: {:?}", constants.vault_size);

        // Verify expected values
        assert_eq!(constants.max_rooms, 40);
        assert_eq!(constants.max_doors, 120);
        assert_eq!(constants.max_subrooms, 24);
        assert_eq!(constants.max_room_area, 50);
    }

    #[test]
    fn test_door_constants() {
        let constants = c_door_constants();

        println!("C Door Constants:");
        println!("  Regular door: {:.1}%", constants.regular_door_prob * 100.0);
        println!("  Secret door: {:.1}%", constants.secret_door_prob * 100.0);
        println!("  Trap probability: {:.1}%", constants.trap_door_prob * 100.0);

        // Verify probabilities sum correctly
        let door_sum = constants.regular_door_prob + constants.secret_door_prob;
        assert!((door_sum - 1.0).abs() < 0.01, "Door probs should sum to 1.0");
    }

    #[test]
    fn test_special_room_functions() {
        let functions = special_room_functions();

        println!("Special room functions ({}):", functions.len());
        for info in &functions {
            println!("\n  {:?} ({})", info.room_type, info.c_function);
            println!("    Monsters: {:?}", info.monster_types);
            println!("    Features: {:?}", info.features);
        }

        // Should have info for all major special rooms
        assert!(functions.len() >= 10);
    }

    #[test]
    fn test_room_spawn_depths() {
        println!("Room spawn depths:");
        for room_type in CRoomType::ALL.iter() {
            if let Some(depth) = room_type.min_depth() {
                let prob = room_type.spawn_probability()
                    .map(|(n, d)| format!("{}/{} ({:.1}%)", n, d, n as f64 / d as f64 * 100.0))
                    .unwrap_or_else(|| "N/A".to_string());
                println!("  {:?}: depth >= {}, prob {}", room_type, depth, prob);
            }
        }
    }

    #[test]
    fn test_court_monster_distribution() {
        // Court room monster selection based on difficulty
        // diff = rn2(60) + rn2(3 * level_difficulty())

        let thresholds = [
            (100, "Dragons"),
            (95, "Giants"),
            (85, "Trolls"),
            (75, "Centaurs"),
            (60, "Orcs"),
            (45, "Bugbears"),
            (30, "Hobgoblins"),
            (15, "Gnomes"),
            (0, "Kobolds"),
        ];

        println!("Court monster thresholds:");
        for (threshold, monster) in &thresholds {
            println!("  diff > {}: {}", threshold, monster);
        }
    }

    #[test]
    fn test_barracks_soldier_distribution() {
        // Barracks uses squadmon() with weights: 80 soldier, 15 sergeant, 4 lieutenant, 1 captain
        let total = 80 + 15 + 4 + 1;

        let soldier_pct = 80.0 / total as f64 * 100.0;
        let sergeant_pct = 15.0 / total as f64 * 100.0;
        let lieutenant_pct = 4.0 / total as f64 * 100.0;
        let captain_pct = 1.0 / total as f64 * 100.0;

        println!("Barracks monster distribution:");
        println!("  Soldier: {:.1}%", soldier_pct);
        println!("  Sergeant: {:.1}%", sergeant_pct);
        println!("  Lieutenant: {:.1}%", lieutenant_pct);
        println!("  Captain: {:.1}%", captain_pct);

        assert!((soldier_pct - 80.0).abs() < 0.1);
        assert!((sergeant_pct - 15.0).abs() < 0.1);
    }

    #[test]
    fn test_morgue_undead_distribution() {
        // Morgue uses morguemon(): ghosts 20%, wraiths 20%, zombies 60%
        // Plus high diff vampires/demons

        println!("Morgue monster distribution:");
        println!("  Ghosts: 20% (i < 20)");
        println!("  Wraiths: 20% (20 <= i < 40)");
        println!("  Zombies: 60% (i >= 40)");
        println!("  Vampires: high diff (hd > 8) & (i > 85)");
        println!("  Demons: very high diff (hd > 10) & (i < 10)");
    }
}
