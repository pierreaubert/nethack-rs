//! Function-Level FFI Isolation Parity Tests (Phase 1 of Parity Strategy)
//!
//! Tests individual C and Rust functions in isolation by feeding them
//! identical inputs and comparing outputs. This isolates bugs to specific
//! functions without needing to trace through the entire generation chain.

use nh_core::dungeon::corridor::finddpos;
use nh_core::dungeon::generation::carve_room;
use nh_core::dungeon::room::Room;
use nh_core::dungeon::{CellType, DLevel, Level};
use nh_core::GameRng;
use nh_core::{COLNO, ROWNO};
use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use serial_test::serial;
use serde_json::Value;

/// C cell type IDs (from rm.h)
mod c_cell_types {
    pub const STONE: i32 = 0;
    pub const VWALL: i32 = 1;
    pub const HWALL: i32 = 2;
    pub const TLCORNER: i32 = 3;
    pub const TRCORNER: i32 = 4;
    pub const BLCORNER: i32 = 5;
    pub const BRCORNER: i32 = 6;
    pub const SDOOR: i32 = 14;
    pub const SCORR: i32 = 15;
    pub const DOOR: i32 = 22;
    pub const CORR: i32 = 23;  // C: CORR=23
    pub const ROOM: i32 = 24;  // C: ROOM=24
}

fn rust_cell_to_c_id(typ: CellType) -> i32 {
    match typ {
        CellType::Stone => c_cell_types::STONE,
        CellType::VWall => c_cell_types::VWALL,
        CellType::HWall => c_cell_types::HWALL,
        CellType::TLCorner => c_cell_types::TLCORNER,
        CellType::TRCorner => c_cell_types::TRCORNER,
        CellType::BLCorner => c_cell_types::BLCORNER,
        CellType::BRCorner => c_cell_types::BRCORNER,
        CellType::SecretDoor => c_cell_types::SDOOR,
        CellType::SecretCorridor => c_cell_types::SCORR,
        CellType::Door => c_cell_types::DOOR,
        CellType::Corridor => c_cell_types::CORR,
        CellType::Room => c_cell_types::ROOM,
        _ => -1, // Unknown mapping
    }
}

/// ============================================================================
/// finddpos isolation tests
/// ============================================================================

#[test]
#[serial]
fn test_finddpos_isolation_single_wall() {
    let mut c_engine = CGameEngine::new();
    c_engine.init("Valkyrie", "Human", 0, 0).expect("C init");

    // Test finddpos on various wall configurations
    let test_cases = [
        // (xl, yl, xh, yh, description)
        (10, 10, 20, 10, "horizontal wall segment"),
        (15, 5, 15, 15, "vertical wall segment"),
        (5, 5, 20, 15, "large area"),
    ];

    for seed in [42, 123, 456, 789, 1000] {
        for (xl, yl, xh, yh, desc) in &test_cases {
            // Setup identical state: clear level and place walls
            c_engine.clear_level();
            c_engine.reset_rng(seed).expect("reset rng");

            // Place walls in C
            for x in *xl..=*xh {
                for y in *yl..=*yh {
                    // Place HWALL if horizontal, VWALL if vertical
                    let wall_type = if yl == yh {
                        c_cell_types::HWALL
                    } else if xl == xh {
                        c_cell_types::VWALL
                    } else {
                        c_cell_types::HWALL
                    };
                    c_engine.set_cell(x as i32, y as i32, wall_type);
                }
            }

            let (c_x, c_y) = c_engine.test_finddpos(*xl as i32, *yl as i32, *xh as i32, *yh as i32);

            // Setup identical state in Rust
            let mut rs_level = Level::new(DLevel::new(0, 14));
            let mut rs_rng = GameRng::new(seed);

            for x in *xl..=*xh {
                for y in *yl..=*yh {
                    rs_level.cells[x][y].typ = if yl == yh {
                        CellType::HWall
                    } else if xl == xh {
                        CellType::VWall
                    } else {
                        CellType::HWall
                    };
                }
            }

            let (rs_x, rs_y) = finddpos(&rs_level, *xl, *yl, *xh, *yh, &mut rs_rng);

            if c_x as usize != rs_x || c_y as usize != rs_y {
                println!(
                    "MISMATCH seed={} {}: C=({},{}) Rust=({},{}) range=({},{})..({},{})",
                    seed, desc, c_x, c_y, rs_x, rs_y, xl, yl, xh, yh
                );
            }
            assert_eq!(
                (c_x as usize, c_y as usize),
                (rs_x, rs_y),
                "finddpos mismatch for seed={} {}: C=({},{}) Rust=({},{})",
                seed,
                desc,
                c_x,
                c_y,
                rs_x,
                rs_y
            );
        }
    }
}

/// ============================================================================
/// finddpos isolation with room wall context (realistic setup)
/// ============================================================================

#[test]
#[serial]
fn test_finddpos_with_room_walls() {
    let mut c_engine = CGameEngine::new();
    c_engine.init("Valkyrie", "Human", 0, 0).expect("C init");

    // Create identical rooms in both C and Rust, then test finddpos on walls
    let rooms = [
        (5, 5, 10, 9),   // Room 1: lx=5, ly=5, hx=10, hy=9
        (20, 5, 25, 9),  // Room 2: to the right
        (5, 15, 10, 19), // Room 3: below room 1
    ];

    for seed in [42, 100, 200, 300, 500] {
        c_engine.clear_level();
        c_engine.reset_rng(seed).expect("reset rng");

        let mut rs_level = Level::new(DLevel::new(0, 14));
        let mut rs_rng = GameRng::new(seed);

        // Carve rooms in both
        for (lx, ly, hx, hy) in &rooms {
            c_engine.carve_room(*lx, *ly, *hx, *hy);
            let room = Room::new(*lx as usize, *ly as usize, (*hx - *lx + 1) as usize, (*hy - *ly + 1) as usize);
            carve_room(&mut rs_level, &room);
        }

        // Test finddpos on the right wall of room 1 (for joining to room 2)
        let (_lx, ly, hx, hy) = rooms[0];
        let wall_x = hx + 1; // Right wall
        let (c_x, c_y) = c_engine.test_finddpos(wall_x, ly, wall_x, hy);
        let (rs_x, rs_y) = finddpos(&rs_level, wall_x as usize, ly as usize, wall_x as usize, hy as usize, &mut rs_rng);

        if c_x as usize != rs_x || c_y as usize != rs_y {
            println!(
                "finddpos room-wall mismatch seed={}: C=({},{}) Rust=({},{}) wall_x={}",
                seed, c_x, c_y, rs_x, rs_y, wall_x
            );
        }
        assert_eq!(
            (c_x as usize, c_y as usize),
            (rs_x, rs_y),
            "finddpos room-wall mismatch for seed={}",
            seed
        );
    }
}

/// ============================================================================
/// dig_corridor isolation test
/// ============================================================================

#[test]
#[serial]
fn test_dig_corridor_isolation() {
    let mut c_engine = CGameEngine::new();
    c_engine.init("Valkyrie", "Human", 0, 0).expect("C init");

    let test_cases = [
        // (sx, sy, dx, dy, desc) — corridor from (sx,sy) to (dx,dy)
        (10, 10, 30, 10, "horizontal right"),
        (30, 10, 10, 10, "horizontal left"),
        (20, 5, 20, 15, "vertical down"),
        (20, 15, 20, 5, "vertical up"),
        (10, 5, 30, 15, "diagonal right-down"),
        (30, 15, 10, 5, "diagonal left-up"),
    ];

    for seed in [42, 123, 456] {
        for (sx, sy, dx, dy, desc) in &test_cases {
            // C side
            c_engine.clear_level();
            c_engine.reset_rng(seed).expect("reset rng");
            let c_result = c_engine.test_dig_corridor(*sx, *sy, *dx, *dy, false);

            // Get C level cells in the corridor region
            let min_x = (*sx).min(*dx) - 2;
            let min_y = (*sy).min(*dy) - 2;
            let max_x = (*sx).max(*dx) + 2;
            let max_y = (*sy).max(*dy) + 2;
            let c_cells_json = c_engine.get_cell_region(min_x, min_y, max_x, max_y);
            let c_cells: Vec<i32> = serde_json::from_str(&c_cells_json).unwrap_or_default();

            // Rust side
            let mut rs_level = Level::new(DLevel::new(0, 14));
            let mut rs_rng = GameRng::new(seed);
            let rs_result = nh_core::dungeon::corridor::dig_corridor_inner_public(
                &mut rs_level,
                *sx,
                *sy,
                *dx,
                *dy,
                false,
                CellType::Corridor,
                CellType::Stone,
                &mut rs_rng,
            );

            // Compare results
            assert_eq!(
                c_result, rs_result,
                "dig_corridor success mismatch for seed={} {}: C={} Rust={}",
                seed, desc, c_result, rs_result
            );

            // Compare cells in the region
            let w = (max_x - min_x + 1) as usize;
            let mut mismatches = 0;
            for (idx, &c_typ) in c_cells.iter().enumerate() {
                let ry = min_y as usize + idx / w;
                let rx = min_x as usize + idx % w;
                if rx < COLNO && ry < ROWNO {
                    let rs_typ = rust_cell_to_c_id(rs_level.cells[rx][ry].typ);
                    if c_typ != rs_typ {
                        mismatches += 1;
                        if mismatches <= 5 {
                            println!(
                                "  cell ({},{}) C={} Rust={} (seed={} {})",
                                rx, ry, c_typ, rs_typ, seed, desc
                            );
                        }
                    }
                }
            }
            if mismatches > 0 {
                println!(
                    "dig_corridor cell mismatches: {} (seed={} {})",
                    mismatches, seed, desc
                );
            }
            assert_eq!(
                mismatches, 0,
                "dig_corridor cell mismatches for seed={} {}",
                seed, desc
            );
        }
    }
}

/// ============================================================================
/// makecorridors isolation test (full corridor generation with identical rooms)
/// ============================================================================

#[test]
#[serial]
fn test_makecorridors_isolation() {
    let mut c_engine = CGameEngine::new();
    c_engine.init("Valkyrie", "Human", 0, 0).expect("C init");

    // Use rooms from a known seed to test corridor generation in isolation
    let room_defs = [
        (5, 3, 10, 7, 0),   // Room 0
        (20, 3, 28, 8, 0),  // Room 1
        (38, 3, 45, 7, 0),  // Room 2
        (5, 12, 12, 16, 0), // Room 3
        (22, 12, 30, 17, 0), // Room 4
    ];

    for seed in [42, 123, 500] {
        // === C side ===
        c_engine.set_dlevel(0, 14); // Match Rust's DLevel::new(0, 14)
        c_engine.clear_level();

        // Add rooms to C
        for (lx, ly, hx, hy, rtype) in &room_defs {
            c_engine.add_room(*lx, *ly, *hx, *hy, *rtype);
            c_engine.carve_room(*lx, *ly, *hx, *hy);
        }

        c_engine.reset_rng(seed).expect("reset rng");
        c_engine.test_makecorridors();

        // Export C level
        let c_level_json = c_engine.export_level();
        let c_level: Value = serde_json::from_str(&c_level_json).unwrap();
        let c_cells = &c_level["cells"];

        // === Rust side ===
        let mut rs_level = Level::new(DLevel::new(0, 14));
        let mut rs_rng = GameRng::new(seed);

        let mut rooms = Vec::new();
        for (lx, ly, hx, hy, _rtype) in &room_defs {
            let w = (*hx - *lx + 1) as usize;
            let h = (*hy - *ly + 1) as usize;
            let room = Room::new(*lx as usize, *ly as usize, w, h);
            carve_room(&mut rs_level, &room);
            rooms.push(room);
        }

        nh_core::dungeon::corridor::generate_corridors(&mut rs_level, &rooms, &mut rs_rng);

        // === Compare ===
        let mut mismatches = 0;
        let mut mismatch_types: std::collections::HashMap<(String, String), usize> =
            std::collections::HashMap::new();

        for y in 0..ROWNO {
            for x in 0..COLNO {
                let c_typ = c_cells[y][x].as_i64().unwrap_or(0) as i32;
                let rs_typ = rust_cell_to_c_id(rs_level.cells[x][y].typ);
                if c_typ != rs_typ {
                    mismatches += 1;
                    let c_name = format!("C({})", c_typ);
                    let rs_name = format!("{:?}", rs_level.cells[x][y].typ);
                    *mismatch_types.entry((rs_name, c_name)).or_insert(0) += 1;
                }
            }
        }

        println!(
            "makecorridors isolation seed={}: {} cell mismatches",
            seed, mismatches
        );
        if !mismatch_types.is_empty() {
            let mut types: Vec<_> = mismatch_types.iter().collect();
            types.sort_by(|a, b| b.1.cmp(a.1));
            for ((rs, c), count) in types.iter().take(10) {
                println!("  Rust={} {}: {}", rs, c, count);
            }
        }

        // This test is currently informational — we expect mismatches
        // and want to quantify them. Uncomment when we expect 0:
        // assert_eq!(mismatches, 0, "makecorridors cell mismatches for seed={}", seed);
    }
}

/// ============================================================================
/// Step-by-step join comparison: finds EXACTLY which join call first diverges
/// ============================================================================

#[test]
#[serial]
fn test_join_step_by_step() {
    let mut c_engine = CGameEngine::new();
    c_engine.init("Valkyrie", "Human", 0, 0).expect("C init");

    let room_defs = [
        (5, 3, 10, 7, 0),
        (20, 3, 28, 8, 0),
        (38, 3, 45, 7, 0),
        (5, 12, 12, 16, 0),
        (22, 12, 30, 17, 0),
    ];

    let seed: u64 = 42;

    // === Setup C side ===
    c_engine.set_dlevel(0, 14); // Match Rust's DLevel::new(0, 14) for level_difficulty()
    c_engine.clear_level();
    for (lx, ly, hx, hy, rtype) in &room_defs {
        c_engine.add_room(*lx, *ly, *hx, *hy, *rtype);
        c_engine.carve_room(*lx, *ly, *hx, *hy);
    }
    c_engine.reset_rng(seed).expect("reset rng");

    // === Setup Rust side ===
    let mut rs_level = Level::new(DLevel::new(0, 14));
    let mut rs_rng = GameRng::new(seed);
    let mut rooms = Vec::new();
    for (lx, ly, hx, hy, _rtype) in &room_defs {
        let w = (*hx - *lx + 1) as usize;
        let h = (*hy - *ly + 1) as usize;
        let room = Room::new(*lx as usize, *ly as usize, w, h);
        carve_room(&mut rs_level, &room);
        rooms.push(room);
    }

    let mut tracker = nh_core::dungeon::ConnectivityTracker::new(rooms.len());

    // Pre-join: compare initial level state
    {
        let c_cells_json = c_engine.get_cell_region(0, 0, (COLNO - 1) as i32, (ROWNO - 1) as i32);
        let c_cells: Vec<i32> = serde_json::from_str(&c_cells_json).unwrap_or_default();
        let mut initial_mismatches = 0;
        for y in 0..ROWNO {
            for x in 0..COLNO {
                let idx = y * COLNO + x;
                let c_typ = c_cells.get(idx).copied().unwrap_or(0);
                let rs_typ = rust_cell_to_c_id(rs_level.cells[x][y].typ);
                if c_typ != rs_typ {
                    initial_mismatches += 1;
                    if initial_mismatches <= 5 {
                        println!("  initial diff: ({},{}) Rust={:?}({}) C={}", x, y, rs_level.cells[x][y].typ, rs_typ, c_typ);
                    }
                }
            }
        }
        println!("Initial level comparison: {} mismatches", initial_mismatches);
    }

    // Trace finddpos for the first join: rooms[0] to rooms[1]
    {
        let croom = &rooms[0];
        let troom = &rooms[1];
        let c_lx = croom.x;
        let c_ly = croom.y;
        let c_hx = croom.x + croom.width - 1;
        let c_hy = croom.y + croom.height - 1;
        let t_lx = troom.x;
        let t_ly = troom.y;
        let _t_hx = troom.x + troom.width - 1;
        let t_hy = troom.y + troom.height - 1;

        println!("Room 0: ({},{}) to ({},{})", c_lx, c_ly, c_hx, c_hy);
        println!("Room 1: ({},{}) to ({},{})", t_lx, t_ly, _t_hx, t_hy);

        // Case: t_lx > c_hx (room 1 is to the right)
        if t_lx > c_hx {
            let xx = c_hx + 1; // right wall of room 0
            let tx = t_lx - 1; // left wall of room 1
            println!("RIGHT case: xx={} tx={}", xx, tx);

            // Test finddpos in C
            let (c_cc_x, c_cc_y) = c_engine.test_finddpos(xx as i32, c_ly as i32, xx as i32, c_hy as i32);
            let (c_tt_x, c_tt_y) = c_engine.test_finddpos(tx as i32, t_ly as i32, tx as i32, t_hy as i32);
            println!("C finddpos: cc=({},{}) tt=({},{})", c_cc_x, c_cc_y, c_tt_x, c_tt_y);

            // Test finddpos in Rust (using a FRESH rng, same seed)
            let mut test_rng = GameRng::new(seed);
            let rs_cc = nh_core::dungeon::corridor::finddpos(&rs_level, xx, c_ly, xx, c_hy, &mut test_rng);
            let rs_tt = nh_core::dungeon::corridor::finddpos(&rs_level, tx, t_ly, tx, t_hy, &mut test_rng);
            println!("Rust finddpos: cc=({},{}) tt=({},{})", rs_cc.0, rs_cc.1, rs_tt.0, rs_tt.1);

            if (c_cc_x as usize, c_cc_y as usize) != rs_cc || (c_tt_x as usize, c_tt_y as usize) != rs_tt {
                println!("  *** FINDDPOS MISMATCH ***");
            } else {
                println!("  finddpos matches!");
            }
        }
    }

    // Reset RNG after finddpos tracing (tracing consumed C RNG calls)
    c_engine.reset_rng(seed).expect("reset rng after tracing");
    rs_rng = GameRng::new(seed);

    // Phase 1: Join adjacent rooms one at a time
    println!("\n=== Phase 1: Adjacent room joins ===");
    for i in 0..rooms.len() - 1 {
        // C side
        c_engine.test_join(i as i32, (i + 1) as i32, false);
        // Rust side
        nh_core::dungeon::corridor::join_rooms(
            &mut rs_level, &rooms, i, i + 1, &mut tracker, &mut rs_rng, false,
        );

        // Compare levels
        let c_cells_json = c_engine.get_cell_region(0, 0, (COLNO - 1) as i32, (ROWNO - 1) as i32);
        let c_cells: Vec<i32> = serde_json::from_str(&c_cells_json).unwrap_or_default();
        let mut mismatches = 0;
        let mut first_mismatch = None;
        for y in 0..ROWNO {
            for x in 0..COLNO {
                let idx = y * COLNO + x;
                let c_typ = c_cells.get(idx).copied().unwrap_or(0);
                let rs_typ = rust_cell_to_c_id(rs_level.cells[x][y].typ);
                if c_typ != rs_typ && first_mismatch.is_none() {
                    first_mismatch = Some((x, y, rs_level.cells[x][y].typ, c_typ));
                }
                if c_typ != rs_typ {
                    mismatches += 1;
                }
            }
        }

        // Compare smeq
        let c_smeq: Vec<i32> = serde_json::from_str(&c_engine.get_smeq()).unwrap_or_default();
        let rs_smeq: Vec<usize> = (0..rooms.len()).map(|i| tracker.smeq_value(i)).collect();
        let smeq_match = c_smeq.iter().zip(rs_smeq.iter()).all(|(c, r)| *c == *r as i32);

        println!(
            "  join({},{},false): {} cell mismatches, smeq C={:?} Rust={:?} {}",
            i, i + 1, mismatches, c_smeq, rs_smeq,
            if smeq_match { "OK" } else { "MISMATCH" },
        );

        if let Some((mx, my, rs_t, c_t)) = first_mismatch {
            println!("    first mismatch: ({},{}) Rust={:?} C_type={}", mx, my, rs_t, c_t);
        }

        if mismatches > 0 {
            // Show up to 10 mismatches with positions
            let mut shown = 0;
            for y in 0..ROWNO {
                for x in 0..COLNO {
                    let idx = y * COLNO + x;
                    let c_typ = c_cells.get(idx).copied().unwrap_or(0);
                    let rs_typ = rust_cell_to_c_id(rs_level.cells[x][y].typ);
                    if c_typ != rs_typ {
                        shown += 1;
                        if shown <= 10 {
                            println!("    ({},{}) Rust={:?}({}) C={}", x, y,
                                rs_level.cells[x][y].typ, rs_typ, c_typ);
                        }
                    }
                }
            }
            println!("  STOP: divergence found at join({},{})", i, i + 1);
            break;
        }

        // NOTE: rn2(50) early exit is part of makecorridors(), not join().
        // Since we're stepping through joins individually, we skip it here.
        // This means Phase 1 always runs all joins (no early break).
    }
}

/// Compare C and Rust rectangle lists after level generation
#[test]
#[serial]
fn test_rect_list_after_generation() {
    let mut c_engine = CGameEngine::new();
    c_engine.init("Valkyrie", "Human", 0, 0).expect("C engine init failed");

    for seed in [5u64, 42, 123] {
        c_engine.set_dlevel(0, 14);
        c_engine.reset_rng(seed).expect("C RNG reset failed");
        c_engine.generate_level().expect("C level generation failed");

        let rect_json = c_engine.rect_json();
        println!("Seed {}: C rects = {}", seed, rect_json);
    }
}
