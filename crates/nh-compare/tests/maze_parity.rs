use nh_core::{GameState, GameRng, COLNO, ROWNO};
use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use serial_test::serial;
use nh_core::player::{Role, Race, Gender};
use serde_json::Value;

#[test]
#[serial]
fn test_maze_parity_seeds() {
    for seed in [42, 123, 999, 1337, 2026] {
        println!("--- Testing Seed {} ---", seed);
        // 1. Initialize C Engine
        let mut c_engine = CGameEngine::new();
        c_engine.init("Valkyrie", "Human", 0, 0).expect("C engine init failed");
        c_engine.reset(seed).expect("C engine reset failed");

        // Set state to Main Dungeon 14 (dnum=0, dlevel=14)
        c_engine.set_dlevel(0, 14);
        
        // Reset RNG state RIGHT before generation to match Rust's fresh RNG
        c_engine.reset_rng(seed).expect("C RNG reset failed");
        c_engine.generate_maze().expect("C maze generation failed");
        
        let c_map_str = c_engine.map_json();
        let c_map: Value = serde_json::from_str(&c_map_str).unwrap();
        let c_cells = &c_map["cells"];
        
        // Count C passages
        let mut c_passages = 0;
        for col in c_cells.as_array().unwrap() {
            for cell in col.as_array().unwrap() {
                let t = cell["type"].as_str().unwrap();
                if t == "Corridor" || t == "Room" {
                    c_passages += 1;
                }
            }
        }
        println!("C generated {} maze passages", c_passages);

        // 2. Initialize Rust Engine
        let mut fresh_rng = GameRng::new(seed);
        // Note: FFI helper now consumes rn2(3) for corrmaze and potentially rn2(2)/rnd(4) for scale
        
        let mut fresh_level = nh_core::dungeon::Level::new(nh_core::dungeon::DLevel::new(0, 14));
        
        nh_core::dungeon::generate_maze(&mut fresh_level, &mut fresh_rng);
        
        let rs_passages = fresh_level.cells.iter()
            .flat_map(|col| col.iter())
            .filter(|cell| cell.typ == nh_core::dungeon::CellType::Corridor || cell.typ == nh_core::dungeon::CellType::Room)
            .count();
        
        println!("Rust generated {} maze passages", rs_passages);
        
        if rs_passages != c_passages {
            println!("C Map:");
            for y in 0..ROWNO {
                for x in 0..COLNO {
                    let cell = &c_cells[x][y];
                    let t = cell["type"].as_str().unwrap();
                    let ch = match t {
                        "Stone" => ' ',
                        "Room" | "Corridor" => '.',
                        _ => '#',
                    };
                    print!("{}", ch);
                }
                println!();
            }

            println!("Rust Map:");
            for y in 0..ROWNO {
                for x in 0..COLNO {
                    let typ = &fresh_level.cells[x][y].typ;
                    let ch = match typ {
                        nh_core::dungeon::CellType::Stone => ' ',
                        nh_core::dungeon::CellType::Room | nh_core::dungeon::CellType::Corridor => '.',
                        _ => '#',
                    };
                    print!("{}", ch);
                }
                println!();
            }
        }
        
        assert!(c_passages > 100, "C level should be a maze");
        assert_eq!(rs_passages, c_passages, "Maze passage count mismatch for seed {}", seed);
    }
}
