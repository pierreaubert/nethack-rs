//! Room structure comparison
//!
//! Compares room data structures and generation parameters between C and Rust.

use std::fs;
use std::path::Path;

use super::room_types::c_room_constants;
#[cfg(test)]
use super::room_types::CRoomType;

/// Room generation comparison result
#[derive(Debug, Clone)]
pub struct GenerationComparison {
    pub aspect: String,
    pub c_value: String,
    pub rust_value: String,
    pub matches: bool,
    pub notes: String,
}

/// Compare all room generation parameters between C and Rust
pub fn compare_room_generation() -> Vec<GenerationComparison> {
    let c = c_room_constants();
    let rust = rust_room_constants();

    vec![
        GenerationComparison {
            aspect: "Maximum rooms per level".to_string(),
            c_value: c.max_rooms.to_string(),
            rust_value: rust.max_rooms.to_string(),
            matches: c.max_rooms == rust.max_rooms,
            notes: if c.max_rooms != rust.max_rooms {
                format!("Rust generates fewer rooms ({}  vs C's {})", rust.max_rooms, c.max_rooms)
            } else {
                String::new()
            },
        },
        GenerationComparison {
            aspect: "Maximum doors per level".to_string(),
            c_value: c.max_doors.to_string(),
            rust_value: rust.max_doors.to_string(),
            matches: c.max_doors == rust.max_doors,
            notes: String::new(),
        },
        GenerationComparison {
            aspect: "Minimum room width".to_string(),
            c_value: c.min_room_width.to_string(),
            rust_value: rust.min_room_size.0.to_string(),
            matches: c.min_room_width == rust.min_room_size.0,
            notes: String::new(),
        },
        GenerationComparison {
            aspect: "Maximum room width".to_string(),
            c_value: c.max_room_width.to_string(),
            rust_value: rust.max_room_size.0.to_string(),
            matches: c.max_room_width == rust.max_room_size.0,
            notes: if c.max_room_width != rust.max_room_size.0 {
                "Rust uses smaller max width".to_string()
            } else {
                String::new()
            },
        },
        GenerationComparison {
            aspect: "Minimum room height".to_string(),
            c_value: c.min_room_height.to_string(),
            rust_value: rust.min_room_size.1.to_string(),
            matches: c.min_room_height == rust.min_room_size.1,
            notes: String::new(),
        },
        GenerationComparison {
            aspect: "Maximum room height".to_string(),
            c_value: c.max_room_height.to_string(),
            rust_value: rust.max_room_size.1.to_string(),
            matches: c.max_room_height == rust.max_room_size.1,
            notes: if c.max_room_height != rust.max_room_size.1 {
                "Rust uses smaller max height".to_string()
            } else {
                String::new()
            },
        },
        GenerationComparison {
            aspect: "Maximum room area".to_string(),
            c_value: c.max_room_area.to_string(),
            rust_value: rust.max_room_area.to_string(),
            matches: c.max_room_area == rust.max_room_area,
            notes: String::new(),
        },
    ]
}

/// Rust room constants from current implementation
#[derive(Debug, Clone)]
pub struct RustRoomConstants {
    pub max_rooms: usize,
    pub max_doors: usize,
    pub min_room_size: (usize, usize), // (width, height)
    pub max_room_size: (usize, usize),
    pub max_room_area: usize,
}

/// Get Rust room generation constants from current implementation
pub fn rust_room_constants() -> RustRoomConstants {
    // From generation.rs:
    // num_rooms = rng.rnd(4) + 5  -> 6-9 rooms
    // width = rng.rnd(7) + 2       -> 3-9
    // height = rng.rnd(5) + 2      -> 3-7
    RustRoomConstants {
        max_rooms: 9, // rnd(4) + 5 max
        max_doors: 120, // No explicit limit in Rust currently
        min_room_size: (3, 3),
        max_room_size: (9, 7),
        max_room_area: 63, // 9 * 7
    }
}

/// Missing features in Rust dungeon generation
#[derive(Debug, Clone)]
pub struct MissingFeature {
    pub feature: String,
    pub c_location: String,
    pub priority: FeaturePriority,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeaturePriority {
    Critical,  // Core gameplay
    High,      // Important for authenticity
    Medium,    // Nice to have
    Low,       // Minor detail
}

/// Get list of features missing in Rust vs C
pub fn missing_features() -> Vec<MissingFeature> {
    vec![
        // Room types
        MissingFeature {
            feature: "Special room types".to_string(),
            c_location: "mkroom.c".to_string(),
            priority: FeaturePriority::Critical,
            description: "Court, Swamp, Vault, Beehive, Morgue, Barracks, Zoo, Temple, etc.".to_string(),
        },
        MissingFeature {
            feature: "Shop rooms".to_string(),
            c_location: "mkroom.c: mkshop()".to_string(),
            priority: FeaturePriority::Critical,
            description: "12 shop types with shopkeepers and inventory".to_string(),
        },
        MissingFeature {
            feature: "Room type enum".to_string(),
            c_location: "mkroom.h".to_string(),
            priority: FeaturePriority::High,
            description: "25 room types (OROOM to CANDLESHOP)".to_string(),
        },

        // Room generation
        MissingFeature {
            feature: "Rectangle system for room placement".to_string(),
            c_location: "rect.c".to_string(),
            priority: FeaturePriority::High,
            description: "Efficient space tracking with MAXRECT=50 rectangles".to_string(),
        },
        MissingFeature {
            feature: "Room count based on available space".to_string(),
            c_location: "mklev.c: makerooms()".to_string(),
            priority: FeaturePriority::High,
            description: "C creates rooms until MAXNROFROOMS or no rectangles left".to_string(),
        },
        MissingFeature {
            feature: "Subrooms".to_string(),
            c_location: "mkroom.h".to_string(),
            priority: FeaturePriority::Medium,
            description: "MAX_SUBROOMS=24 subrooms per parent room".to_string(),
        },
        MissingFeature {
            feature: "Irregular (non-rectangular) rooms".to_string(),
            c_location: "mkroom.irregular".to_string(),
            priority: FeaturePriority::Low,
            description: "Support for non-rectangular room shapes".to_string(),
        },

        // Corridors
        MissingFeature {
            feature: "4-phase corridor connection".to_string(),
            c_location: "mklev.c: makecorridors()".to_string(),
            priority: FeaturePriority::High,
            description: "Sequential, skip, connectivity, random extra phases".to_string(),
        },
        MissingFeature {
            feature: "Connectivity tracking (smeq[])".to_string(),
            c_location: "mklev.c".to_string(),
            priority: FeaturePriority::High,
            description: "Track connected components to ensure all rooms reachable".to_string(),
        },
        MissingFeature {
            feature: "Extra random corridors".to_string(),
            c_location: "mklev.c: makecorridors()".to_string(),
            priority: FeaturePriority::Medium,
            description: "4-7 additional random corridors after base connectivity".to_string(),
        },

        // Doors
        MissingFeature {
            feature: "Secret doors (SDOOR)".to_string(),
            c_location: "mklev.c: dodoor()".to_string(),
            priority: FeaturePriority::High,
            description: "12.5% of doors are secret doors".to_string(),
        },
        MissingFeature {
            feature: "Door states (open/closed/locked)".to_string(),
            c_location: "mklev.c: dosdoor()".to_string(),
            priority: FeaturePriority::High,
            description: "Doors can be open, closed, locked, or trapped".to_string(),
        },
        MissingFeature {
            feature: "Trapped doors".to_string(),
            c_location: "mklev.c: dosdoor()".to_string(),
            priority: FeaturePriority::Medium,
            description: "4% chance at depth >= 5".to_string(),
        },
        MissingFeature {
            feature: "No adjacent doors rule".to_string(),
            c_location: "mklev.c: bydoor()".to_string(),
            priority: FeaturePriority::Low,
            description: "Doors cannot be placed adjacent to existing doors".to_string(),
        },

        // Special levels
        MissingFeature {
            feature: "Maze levels".to_string(),
            c_location: "mkmaze.c".to_string(),
            priority: FeaturePriority::High,
            description: "Maze-type levels with different generation".to_string(),
        },
        MissingFeature {
            feature: "Special level loading (.des files)".to_string(),
            c_location: "sp_lev.c".to_string(),
            priority: FeaturePriority::High,
            description: "Load predefined levels from .des proto files".to_string(),
        },
        MissingFeature {
            feature: "Vault generation".to_string(),
            c_location: "mklev.c: create_vault()".to_string(),
            priority: FeaturePriority::Medium,
            description: "2x2 secret vaults with gold and teleport access".to_string(),
        },

        // Level flags
        MissingFeature {
            feature: "Level flags".to_string(),
            c_location: "you.h: level.flags".to_string(),
            priority: FeaturePriority::Medium,
            description: "has_shop, has_vault, has_zoo, noteleport, hardfloor, etc.".to_string(),
        },

        // Monsters
        MissingFeature {
            feature: "Room-appropriate monster spawning".to_string(),
            c_location: "mkroom.c: fill_zoo()".to_string(),
            priority: FeaturePriority::Critical,
            description: "Different monsters for each room type".to_string(),
        },
        MissingFeature {
            feature: "Monster difficulty scaling".to_string(),
            c_location: "mkroom.c: courtmon(), morguemon(), etc.".to_string(),
            priority: FeaturePriority::High,
            description: "Monsters chosen based on level difficulty".to_string(),
        },
    ]
}

/// Room constants from C source (extract actual values)
pub fn extract_c_room_constants() -> Option<super::room_types::CRoomConstants> {
    let global_h = Path::new(crate::data::NETHACK_SRC).join("include/global.h");

    if !global_h.exists() {
        return None;
    }

    let content = fs::read_to_string(&global_h).ok()?;

    let mut max_rooms = 40;
    for line in content.lines() {
        if line.contains("#define MAXNROFROOMS") {
            if let Some(num_str) = line.split_whitespace().nth(2) {
                if let Ok(num) = num_str.parse::<usize>() {
                    max_rooms = num;
                }
            }
        }
    }

    Some(super::room_types::CRoomConstants {
        max_rooms,
        max_doors: 120,
        max_subrooms: 24,
        max_rectangles: 50,
        min_room_width: 2,
        max_room_width: 17,
        min_room_height: 2,
        max_room_height: 9,
        max_room_area: 50,
        vault_size: (2, 2),
        xlim: 4,
        ylim: 3,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_room_constants() {
        let rust = rust_room_constants();

        println!("Rust room constants:");
        println!("  Max rooms: {}", rust.max_rooms);
        println!(
            "  Room size: {}x{} to {}x{}",
            rust.min_room_size.0, rust.min_room_size.1,
            rust.max_room_size.0, rust.max_room_size.1
        );
        println!("  Max area: {}", rust.max_room_area);

        assert!(rust.max_rooms > 0);
        assert!(rust.min_room_size.0 >= 2);
        assert!(rust.max_room_size.0 >= rust.min_room_size.0);
    }

    #[test]
    fn test_c_room_constants() {
        let c = extract_c_room_constants();

        if let Some(c) = c {
            println!("C room constants:");
            println!("  MAXNROFROOMS: {}", c.max_rooms);
            println!("  DOORMAX: {}", c.max_doors);
            println!("  MAX_SUBROOMS: {}", c.max_subrooms);
            println!("  Room size: {}x{} to {}x{}",
                c.min_room_width, c.min_room_height,
                c.max_room_width, c.max_room_height);
            println!("  Max area: {}", c.max_room_area);
            println!("  XLIM/YLIM: {}/{}", c.xlim, c.ylim);

            assert_eq!(c.max_rooms, 40);
            assert_eq!(c.max_doors, 120);
        }
    }

    #[test]
    fn test_generation_comparison() {
        let comparisons = compare_room_generation();

        println!("\n=== Room Generation Comparison ===\n");
        println!("{:<30} {:>10} {:>10} {:>8}", "Aspect", "C", "Rust", "Match");
        println!("{}", "-".repeat(60));

        let mut matches = 0;
        let mut total = 0;

        for comp in &comparisons {
            let match_str = if comp.matches { "✓" } else { "✗" };
            println!("{:<30} {:>10} {:>10} {:>8}",
                comp.aspect, comp.c_value, comp.rust_value, match_str);
            if !comp.notes.is_empty() {
                println!("    Note: {}", comp.notes);
            }
            if comp.matches {
                matches += 1;
            }
            total += 1;
        }

        println!("\nMatched: {}/{} ({:.1}%)",
            matches, total, matches as f64 / total as f64 * 100.0);
    }

    #[test]
    fn test_missing_features() {
        let missing = missing_features();

        println!("\n=== Missing Features in Rust ===\n");

        let critical: Vec<_> = missing.iter().filter(|f| f.priority == FeaturePriority::Critical).collect();
        let high: Vec<_> = missing.iter().filter(|f| f.priority == FeaturePriority::High).collect();
        let medium: Vec<_> = missing.iter().filter(|f| f.priority == FeaturePriority::Medium).collect();
        let low: Vec<_> = missing.iter().filter(|f| f.priority == FeaturePriority::Low).collect();

        println!("CRITICAL ({}):", critical.len());
        for f in &critical {
            println!("  - {} ({})", f.feature, f.c_location);
            println!("    {}", f.description);
        }

        println!("\nHIGH ({}):", high.len());
        for f in &high {
            println!("  - {} ({})", f.feature, f.c_location);
        }

        println!("\nMEDIUM ({}):", medium.len());
        for f in &medium {
            println!("  - {}", f.feature);
        }

        println!("\nLOW ({}):", low.len());
        for f in &low {
            println!("  - {}", f.feature);
        }

        println!("\nTotal missing features: {}", missing.len());
    }

    #[test]
    fn test_room_type_implementation_status() {
        println!("\n=== Room Type Implementation Status ===\n");

        for room_type in CRoomType::ALL.iter() {
            let implemented = matches!(room_type, CRoomType::ORoom);
            let status = if implemented { "✓" } else { "✗" };

            println!("{} {:?} ({}): {}",
                status,
                room_type,
                room_type.c_name(),
                room_type.description()
            );
        }

        let implemented_count = CRoomType::ALL.iter()
            .filter(|t| matches!(t, CRoomType::ORoom))
            .count();

        println!("\nImplemented: {}/{} ({:.1}%)",
            implemented_count, CRoomType::ALL.len(),
            implemented_count as f64 / CRoomType::ALL.len() as f64 * 100.0);
    }

    #[test]
    fn test_special_room_spawn_requirements() {
        println!("\n=== Special Room Spawn Requirements ===\n");
        println!("{:<20} {:>10} {:>15}", "Room Type", "Min Depth", "Probability");
        println!("{}", "-".repeat(50));

        for room_type in CRoomType::ALL.iter() {
            if room_type.is_shop() && *room_type != CRoomType::ShopBase {
                continue; // Skip individual shop types
            }

            let depth = room_type.min_depth()
                .map(|d| d.to_string())
                .unwrap_or_else(|| "N/A".to_string());

            let prob = room_type.spawn_probability()
                .map(|(n, d)| format!("{:.1}% (1/{})", n as f64 / d as f64 * 100.0, d))
                .unwrap_or_else(|| "N/A".to_string());

            println!("{:<20} {:>10} {:>15}", room_type.c_name(), depth, prob);
        }
    }

    #[test]
    fn test_corridor_algorithm_comparison() {
        println!("\n=== Corridor Algorithm Comparison ===\n");

        println!("C Implementation (4 phases):");
        println!("  1. Sequential: Connect room N to N+1 (2% early stop)");
        println!("  2. Skip: Connect room N to N+2 if not connected");
        println!("  3. Complete: Ensure all rooms reachable from room 0");
        println!("  4. Extra: Add 4-7 random corridors (can cross)");

        println!("\nRust Implementation:");
        println!("  - Single pass: Connect each room to next in circular fashion");
        println!("  - L-shaped corridors (random H-V or V-H)");

        println!("\nDifferences:");
        println!("  - C has more connectivity guarantees");
        println!("  - C adds extra random corridors for variety");
        println!("  - C tracks connected components (smeq[] array)");
    }

    #[test]
    fn test_door_algorithm_comparison() {
        println!("\n=== Door Algorithm Comparison ===\n");

        println!("C Implementation:");
        println!("  - 87.5% regular doors, 12.5% secret doors");
        println!("  - Random state: 20% open, 16.7% locked, 63.3% closed");
        println!("  - Shop doors: always open (regular) or locked (secret)");
        println!("  - 4% trap chance at depth >= 5");
        println!("  - No adjacent doors allowed (bydoor() check)");

        println!("\nRust Implementation:");
        println!("  - 80% door placement chance on eligible walls");
        println!("  - 90% closed, 10% open");
        println!("  - No secret doors");
        println!("  - No trapped doors");
        println!("  - No adjacent door check");
    }
}
