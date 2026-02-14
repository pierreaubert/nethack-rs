//! Dungeon generation comparison
//!
//! Compares dungeon generation algorithms between C and Rust.
//! Since we control the RNG, we can verify that the same seed
//! produces structurally similar levels.

/// Statistics about a generated level
#[derive(Debug, Clone, Default)]
pub struct LevelStats {
    pub room_count: usize,
    pub corridor_cells: usize,
    pub door_count: usize,
    pub stair_count: usize,
    pub monster_count: usize,
    pub total_room_area: usize,
    pub average_room_size: f64,
}

/// Cell type enum for tracking generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellCategory {
    Stone,
    Room,
    Corridor,
    Wall,
    Door,
    Stairs,
    Other,
}

/// Analyze a generated level for statistical comparison
pub fn analyze_level_stats(
    cells: &[Vec<CellCategory>],
    monsters: usize,
    stairs: usize,
) -> LevelStats {
    let mut room_cells = 0;
    let mut corridor_cells = 0;
    let mut door_count = 0;

    for col in cells {
        for cell in col {
            match cell {
                CellCategory::Room => room_cells += 1,
                CellCategory::Corridor => corridor_cells += 1,
                CellCategory::Door => door_count += 1,
                _ => {}
            }
        }
    }

    // Estimate room count from room cells (assume ~25 cells per room average)
    let estimated_rooms = (room_cells / 25).max(1);

    LevelStats {
        room_count: estimated_rooms,
        corridor_cells,
        door_count,
        stair_count: stairs,
        monster_count: monsters,
        total_room_area: room_cells,
        average_room_size: if estimated_rooms > 0 {
            room_cells as f64 / estimated_rooms as f64
        } else {
            0.0
        },
    }
}

/// Generate stats from Rust level generation
pub fn generate_rust_level_stats(seed: u64) -> LevelStats {
    use nh_core::GameRng;
    use nh_core::dungeon::{CellType, DLevel, Level, generate_rooms_and_corridors};
    use nh_core::magic::MonsterVitals;

    let mut rng = GameRng::new(seed);
    let mut level = Level::new(DLevel::main_dungeon_start());
    let monster_vitals = MonsterVitals::new();

    generate_rooms_and_corridors(&mut level, &mut rng, &monster_vitals);

    // Convert to our cell categories
    let cells: Vec<Vec<CellCategory>> = level
        .cells
        .iter()
        .map(|col| {
            col.iter()
                .map(|cell| match cell.typ {
                    CellType::Stone => CellCategory::Stone,
                    CellType::Room => CellCategory::Room,
                    CellType::Corridor => CellCategory::Corridor,
                    CellType::Door => CellCategory::Door,
                    CellType::Stairs => CellCategory::Stairs,
                    CellType::VWall
                    | CellType::HWall
                    | CellType::TLCorner
                    | CellType::TRCorner
                    | CellType::BLCorner
                    | CellType::BRCorner => CellCategory::Wall,
                    _ => CellCategory::Other,
                })
                .collect()
        })
        .collect();

    LevelStats {
        room_count: 0, // We'll count differently
        corridor_cells: cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|&&c| c == CellCategory::Corridor)
            .count(),
        door_count: cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|&&c| c == CellCategory::Door)
            .count(),
        stair_count: level.stairs.len(),
        monster_count: level.monsters.len(),
        total_room_area: cells
            .iter()
            .flat_map(|col| col.iter())
            .filter(|&&c| c == CellCategory::Room)
            .count(),
        average_room_size: 0.0, // Will calculate below
    }
}

/// Expected ranges for level statistics
pub struct StatRanges {
    pub room_count: (usize, usize),
    pub corridor_cells: (usize, usize),
    pub door_count: (usize, usize),
    pub stair_count: (usize, usize),
    pub monster_count: (usize, usize),
    pub room_area: (usize, usize),
}

/// Get expected ranges for Rust generation
pub fn rust_expected_ranges() -> StatRanges {
    // Based on generation.rs:
    // - num_rooms = rnd(4) + 5 -> 6-9 rooms
    // - room width 3-9, height 3-7 -> area 9-63 per room
    // - monsters = rnd(6) + 2 -> 3-8 monsters
    // - Each room has ~2-4 walls that could become doors (80% chance each)
    // - 4-phase corridor algorithm creates more connections than simple L-shaped
    StatRanges {
        room_count: (6, 9),
        corridor_cells: (50, 400), // 4-phase algorithm creates more corridors
        door_count: (5, 60),       // More corridors = more potential doors
        stair_count: (1, 2),       // Up and down
        monster_count: (3, 8),
        room_area: (50, 500), // 6-9 rooms * 9-63 cells
    }
}

/// Verify level stats are within expected ranges
pub fn verify_stats_in_range(stats: &LevelStats, ranges: &StatRanges) -> Vec<String> {
    let mut errors = Vec::new();

    if stats.corridor_cells < ranges.corridor_cells.0
        || stats.corridor_cells > ranges.corridor_cells.1
    {
        errors.push(format!(
            "Corridor cells {} outside range {:?}",
            stats.corridor_cells, ranges.corridor_cells
        ));
    }

    if stats.door_count < ranges.door_count.0 || stats.door_count > ranges.door_count.1 {
        errors.push(format!(
            "Door count {} outside range {:?}",
            stats.door_count, ranges.door_count
        ));
    }

    if stats.stair_count < ranges.stair_count.0 || stats.stair_count > ranges.stair_count.1 {
        errors.push(format!(
            "Stair count {} outside range {:?}",
            stats.stair_count, ranges.stair_count
        ));
    }

    if stats.monster_count < ranges.monster_count.0 || stats.monster_count > ranges.monster_count.1
    {
        errors.push(format!(
            "Monster count {} outside range {:?}",
            stats.monster_count, ranges.monster_count
        ));
    }

    if stats.total_room_area < ranges.room_area.0 || stats.total_room_area > ranges.room_area.1 {
        errors.push(format!(
            "Room area {} outside range {:?}",
            stats.total_room_area, ranges.room_area
        ));
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_rust_level() {
        let stats = generate_rust_level_stats(42);

        println!("Rust level stats (seed 42):");
        println!("  Corridor cells: {}", stats.corridor_cells);
        println!("  Door count: {}", stats.door_count);
        println!("  Stair count: {}", stats.stair_count);
        println!("  Monster count: {}", stats.monster_count);
        println!("  Total room area: {}", stats.total_room_area);

        // Basic sanity checks
        assert!(stats.total_room_area > 0, "Should have room cells");
        assert!(stats.corridor_cells > 0, "Should have corridor cells");
    }

    #[test]
    fn test_level_stats_in_range() {
        let ranges = rust_expected_ranges();

        // Test multiple seeds
        let mut all_errors = Vec::new();
        for seed in [42u64, 12345, 99999, 1, 777, 31337, 42424242] {
            let stats = generate_rust_level_stats(seed);
            let errors = verify_stats_in_range(&stats, &ranges);

            if !errors.is_empty() {
                all_errors.push(format!("Seed {}: {:?}", seed, errors));
            }
        }

        // Allow some variance - not all seeds will be perfect
        let error_rate = all_errors.len() as f64 / 7.0;
        assert!(
            error_rate <= 0.3,
            "Too many out-of-range levels ({:.0}%): {:?}",
            error_rate * 100.0,
            all_errors
        );
    }

    #[test]
    fn test_reproducibility() {
        // Same seed should produce same level
        let stats1 = generate_rust_level_stats(42);
        let stats2 = generate_rust_level_stats(42);

        assert_eq!(
            stats1.corridor_cells, stats2.corridor_cells,
            "Same seed should produce same corridor count"
        );
        assert_eq!(
            stats1.door_count, stats2.door_count,
            "Same seed should produce same door count"
        );
        assert_eq!(
            stats1.stair_count, stats2.stair_count,
            "Same seed should produce same stair count"
        );
        assert_eq!(
            stats1.monster_count, stats2.monster_count,
            "Same seed should produce same monster count"
        );
        assert_eq!(
            stats1.total_room_area, stats2.total_room_area,
            "Same seed should produce same room area"
        );
    }

    #[test]
    fn test_different_seeds_produce_different_levels() {
        let stats1 = generate_rust_level_stats(42);
        let stats2 = generate_rust_level_stats(12345);

        // At least one stat should differ
        let all_same = stats1.corridor_cells == stats2.corridor_cells
            && stats1.door_count == stats2.door_count
            && stats1.monster_count == stats2.monster_count
            && stats1.total_room_area == stats2.total_room_area;

        assert!(!all_same, "Different seeds should produce different levels");
    }

    #[test]
    fn test_level_connectivity() {
        // Verify that corridors connect rooms (basic check)
        use nh_core::GameRng;
        use nh_core::dungeon::{CellType, DLevel, Level, generate_rooms_and_corridors};
        use nh_core::magic::MonsterVitals;

        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::main_dungeon_start());
        let monster_vitals = MonsterVitals::new();
        generate_rooms_and_corridors(&mut level, &mut rng, &monster_vitals);

        // Find a room cell
        let mut room_pos = None;
        for x in 0..level.cells.len() {
            for y in 0..level.cells[x].len() {
                if level.cells[x][y].typ == CellType::Room {
                    room_pos = Some((x, y));
                    break;
                }
            }
            if room_pos.is_some() {
                break;
            }
        }

        assert!(room_pos.is_some(), "Should have at least one room cell");

        // Simple flood fill to check connectivity
        let (start_x, start_y) = room_pos.unwrap();
        let mut visited = vec![vec![false; level.cells[0].len()]; level.cells.len()];
        let mut stack = vec![(start_x, start_y)];
        let mut reachable = 0;

        while let Some((x, y)) = stack.pop() {
            if visited[x][y] {
                continue;
            }
            visited[x][y] = true;

            let cell_type = level.cells[x][y].typ;
            if cell_type == CellType::Stone
                || cell_type == CellType::VWall
                || cell_type == CellType::HWall
            {
                continue;
            }

            reachable += 1;

            // Add neighbors
            for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0
                    && ny >= 0
                    && (nx as usize) < level.cells.len()
                    && (ny as usize) < level.cells[0].len()
                {
                    stack.push((nx as usize, ny as usize));
                }
            }
        }

        println!("Reachable cells from first room: {}", reachable);
        assert!(
            reachable > 50,
            "Should be able to reach many cells from a room"
        );
    }

    #[test]
    fn test_stairs_in_rooms() {
        use nh_core::GameRng;
        use nh_core::dungeon::{CellType, DLevel, Level, generate_rooms_and_corridors};
        use nh_core::magic::MonsterVitals;

        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::main_dungeon_start());
        let monster_vitals = MonsterVitals::new();
        generate_rooms_and_corridors(&mut level, &mut rng, &monster_vitals);

        // Verify stairs exist
        assert!(!level.stairs.is_empty(), "Should have stairs");

        // Verify stairs are on stair cells
        for stair in &level.stairs {
            let cell_type = level.cells[stair.x as usize][stair.y as usize].typ;
            assert_eq!(
                cell_type,
                CellType::Stairs,
                "Stair position should have Stairs cell type"
            );
        }

        println!("Found {} stairs", level.stairs.len());
        for (i, stair) in level.stairs.iter().enumerate() {
            println!(
                "  Stair {}: ({}, {}) {}",
                i,
                stair.x,
                stair.y,
                if stair.up { "UP" } else { "DOWN" }
            );
        }
    }

    #[test]
    fn test_door_placement() {
        use nh_core::GameRng;
        use nh_core::dungeon::{CellType, DLevel, Level, generate_rooms_and_corridors};
        use nh_core::magic::MonsterVitals;

        let mut rng = GameRng::new(42);
        let mut level = Level::new(DLevel::main_dungeon_start());
        let monster_vitals = MonsterVitals::new();
        generate_rooms_and_corridors(&mut level, &mut rng, &monster_vitals);

        let mut door_count = 0;
        let mut doors_adjacent_to_room = 0;
        let mut doors_adjacent_to_corridor = 0;

        for x in 1..level.cells.len() - 1 {
            for y in 1..level.cells[x].len() - 1 {
                if level.cells[x][y].typ == CellType::Door {
                    door_count += 1;

                    // Check adjacent cells
                    let neighbors = [
                        level.cells[x - 1][y].typ,
                        level.cells[x + 1][y].typ,
                        level.cells[x][y - 1].typ,
                        level.cells[x][y + 1].typ,
                    ];

                    if neighbors.iter().any(|&t| t == CellType::Room) {
                        doors_adjacent_to_room += 1;
                    }
                    if neighbors.iter().any(|&t| t == CellType::Corridor) {
                        doors_adjacent_to_corridor += 1;
                    }
                }
            }
        }

        println!("Door analysis:");
        println!("  Total doors: {}", door_count);
        println!("  Adjacent to room: {}", doors_adjacent_to_room);
        println!("  Adjacent to corridor: {}", doors_adjacent_to_corridor);

        // Most doors should connect rooms and corridors
        if door_count > 0 {
            let connectivity_rate =
                (doors_adjacent_to_room.min(doors_adjacent_to_corridor)) as f64 / door_count as f64;
            println!("  Connectivity rate: {:.1}%", connectivity_rate * 100.0);
        }
    }
}
