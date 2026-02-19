use nh_core::player::{Role, Race, Gender};
use nh_core::{GameState, GameRng};
use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use serial_test::serial;
use serde_json::Value;
use nh_core::magic::genocide::MonsterVitals;

#[test]
#[serial]
fn test_room_placement_parity_seeds() {
    let seeds = [42, 123, 999, 1337, 2026];
    for seed in seeds {
        println!("--- Testing Seed {} ---", seed);
        run_room_parity_test(seed);
    }
}

fn run_room_parity_test(seed: u64) {
    let mut c_engine = CGameEngine::new();
    let monster_vitals = MonsterVitals::default();

    // 1. Initialize C Engine
    c_engine.init("Valkyrie", "Human", 0, 0).expect("C engine init failed");
    
    // Set state to Main Dungeon 14 (dnum=0, dlevel=14)
    c_engine.set_dlevel(0, 14);
    
    // Reset RNG state RIGHT before generation to match Rust's fresh RNG
    c_engine.reset_rng(seed).expect("C RNG reset failed");
    c_engine.generate_level().expect("C level generation failed");
    
    let c_map_str = c_engine.map_json();
    let c_map: Value = serde_json::from_str(&c_map_str).unwrap();
    println!("C nroom = {}", c_map["nroom"]);
    let c_rooms = c_map["rooms"].as_array().expect("C rooms missing");
    
    println!("C generated {} rooms", c_rooms.len());
    for (i, r) in c_rooms.iter().enumerate() {
        println!("  C Room {}: ({},{}) {}x{} (type={})", i, r["lx"], r["ly"], r["hx"].as_i64().unwrap() - r["lx"].as_i64().unwrap() + 1, r["hy"].as_i64().unwrap() - r["ly"].as_i64().unwrap() + 1, r["type"]);
    }

    // 2. Initialize Rust Engine
    let mut fresh_rng = GameRng::new(seed);
    let mut fresh_level = nh_core::dungeon::Level::new(nh_core::dungeon::DLevel::new(0, 14));
    fresh_rng.enable_tracing();
    nh_core::dungeon::generate_rooms_and_corridors(&mut fresh_level, &mut fresh_rng, &monster_vitals);
    println!("Rust: total RNG calls = {}", fresh_rng.call_count());

    // Dump Rust RNG trace
    let trace = fresh_rng.get_trace();
    let trace_path = format!("/Users/pierre/src/games/tmp/rs_trace_seed{}.txt", seed);
    let mut trace_file = std::fs::File::create(&trace_path).unwrap();
    use std::io::Write;
    for entry in &trace {
        writeln!(trace_file, "RS: {}({}) = {} (raw={})", entry.func, entry.arg, entry.result, entry.raw).unwrap();
    }
    println!("Rust: wrote {} trace entries to {}", trace.len(), trace_path);
    
    // Filter out vault rooms from both sides (type=4 in C is VAULT)
    let rs_rooms: Vec<_> = fresh_level.rooms.iter()
        .filter(|r| r.room_type != nh_core::dungeon::RoomType::Vault)
        .collect();
    let c_rooms: Vec<_> = c_rooms.iter()
        .filter(|r| r["type"].as_i64().unwrap_or(0) != 4)
        .collect();

    println!("Rust generated {} rooms ({} total incl. vault)", rs_rooms.len(), fresh_level.rooms.len());
    for (i, r) in rs_rooms.iter().enumerate() {
        println!("  Rust Room {}: ({},{}) {}x{}", i, r.x, r.y, r.width, r.height);
    }

    // 3. Compare Room Count and Layout
    assert_eq!(rs_rooms.len(), c_rooms.len(), "Room count mismatch for seed {}", seed);

    for (i, c_room) in c_rooms.iter().enumerate() {
        let rs_room = &rs_rooms[i];
        
        // Map Rust (x,y,w,h) to C (lx,hx,ly,hy)
        let rs_lx = rs_room.x as i64;
        let rs_hx = (rs_room.x + rs_room.width - 1) as i64;
        let rs_ly = rs_room.y as i64;
        let rs_hy = (rs_room.y + rs_room.height - 1) as i64;

        assert_eq!(rs_lx, c_room["lx"].as_i64().unwrap(), "Room {} lx mismatch for seed {}", i, seed);
        assert_eq!(rs_hx, c_room["hx"].as_i64().unwrap(), "Room {} hx mismatch for seed {}", i, seed);
        assert_eq!(rs_ly, c_room["ly"].as_i64().unwrap(), "Room {} ly mismatch for seed {}", i, seed);
        assert_eq!(rs_hy, c_room["hy"].as_i64().unwrap(), "Room {} hy mismatch for seed {}", i, seed);
    }

    // 4. Compare full map cells
    let c_cells = &c_map["cells"];
    let mut mismatches = Vec::new();
    // Normalize C type names to match Rust's Debug output
    let normalize_c_type = |s: &str| -> String {
        match s {
            "SDoor" => "SecretDoor".to_string(),
            "SCorr" => "SecretCorridor".to_string(),
            other => other.to_string(),
        }
    };
    for x in 0..nh_core::COLNO {
        for y in 0..nh_core::ROWNO {
            let c_cell = &c_cells[x][y];
            let c_typ = normalize_c_type(c_cell["type"].as_str().unwrap());
            let rs_cell = &fresh_level.cells[x][y];
            let rs_typ = format!("{:?}", rs_cell.typ);

            if rs_typ != c_typ {
                mismatches.push((x, y, rs_typ, c_typ));
            }
        }
    }

    if !mismatches.is_empty() {
        println!("Cell mismatches for seed {}: {} total", seed, mismatches.len());
        // Group by type of mismatch
        let mut type_counts: std::collections::HashMap<(String, String), usize> = std::collections::HashMap::new();
        for (_, _, rs, c) in &mismatches {
            *type_counts.entry((rs.clone(), c.clone())).or_insert(0) += 1;
        }
        for ((rs, c), count) in type_counts.iter() {
            println!("  Rust={} vs C={}: {} cells", rs, c, count);
        }
        // Show first 20
        for (x, y, rs, c) in mismatches.iter().take(20) {
            println!("  ({},{}) Rust={} C={}", x, y, rs, c);
        }
        panic!("Cell mismatches found for seed {} (see above)", seed);
    }
}
