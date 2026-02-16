use nh_core::player::{Role, Race, Gender};
use nh_core::{GameState, GameRng};
use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use serial_test::serial;
use serde_json::Value;

#[test]
#[serial]
fn test_room_placement_parity_seed_42() {
    run_map_parity_test(42);
}

fn run_map_parity_test(seed: u64) {
    let role = Role::Valkyrie;
    let race = Race::Human;
    let gender = Gender::Female;

    // 1. Initialize C Engine
    let mut c_engine = CGameEngine::new();
    c_engine.init("Valkyrie", "Human", 0, 0).expect("C engine init failed");
    c_engine.generate_level().expect("C level generation failed");
    
    let c_map_str = c_engine.map_json();
    let c_map: Value = serde_json::from_str(&c_map_str).unwrap();
    let c_rooms = c_map["rooms"].as_array().expect("C rooms missing");

    println!("C generated {} rooms", c_rooms.len());

    // 2. Initialize Rust Engine
    // RE-SEED and RE-GENERATE to match C's mklev reseed(42) behavior
    let mut fresh_rng = GameRng::new(seed);
    let monster_vitals = nh_core::magic::genocide::MonsterVitals::new();
    let mut fresh_level = nh_core::dungeon::Level::new(nh_core::dungeon::DLevel::new(0, 1));
    nh_core::dungeon::generate_rooms_and_corridors(&mut fresh_level, &mut fresh_rng, &monster_vitals);
    
    let rs_rooms = &fresh_level.rooms;

    println!("C generated {} rooms", c_rooms.len());
    for (i, r) in c_rooms.iter().enumerate() {
        println!("  C Room {}: ({},{}) {}x{}", i, r["lx"], r["ly"], r["hx"].as_i64().unwrap() - r["lx"].as_i64().unwrap() + 1, r["hy"].as_i64().unwrap() - r["ly"].as_i64().unwrap() + 1);
    }
    
    println!("Rust generated {} rooms", rs_rooms.len());
    for (i, r) in rs_rooms.iter().enumerate() {
        println!("  Rust Room {}: ({},{}) {}x{}", i, r.x, r.y, r.width, r.height);
    }

    // 3. Compare Room Count and Layout
    // NOTE: This is EXPECTED TO FAIL initially until mklev.c is ported.
    assert_eq!(rs_rooms.len(), c_rooms.len(), "Room count mismatch for seed {}", seed);

    for (i, c_room) in c_rooms.iter().enumerate() {
        let rs_room = &rs_rooms[i];
        
        // Map Rust (x,y,w,h) to C (lx,hx,ly,hy)
        let rs_lx = rs_room.x as i64;
        let rs_hx = (rs_room.x + rs_room.width - 1) as i64;
        let rs_ly = rs_room.y as i64;
        let rs_hy = (rs_room.y + rs_room.height - 1) as i64;

        assert_eq!(rs_lx, c_room["lx"].as_i64().unwrap(), "Room {} lx mismatch", i);
        assert_eq!(rs_hx, c_room["hx"].as_i64().unwrap(), "Room {} hx mismatch", i);
        assert_eq!(rs_ly, c_room["ly"].as_i64().unwrap(), "Room {} ly mismatch", i);
        assert_eq!(rs_hy, c_room["hy"].as_i64().unwrap(), "Room {} hy mismatch", i);
    }
}
