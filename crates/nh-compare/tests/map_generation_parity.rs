use nh_core::player::{Role, Race, Gender};
use nh_core::{GameState, GameRng};
use nh_test::ffi::CGameEngine;
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
    
    let c_map_str = c_engine.map_json();
    let c_map: Value = serde_json::from_str(&c_map_str).unwrap();
    let c_rooms = c_map["rooms"].as_array().expect("C rooms missing");

    println!("C generated {} rooms", c_rooms.len());

    // 2. Initialize Rust Engine
    let rust_rng = GameRng::new(seed);
    let rust_state = GameState::new_with_identity(rust_rng, "Hero".into(), role, race, gender);
    let rs_rooms = &rust_state.levels[&nh_core::dungeon::DLevel::new(1, 1)].rooms;

    println!("Rust generated {} rooms", rs_rooms.len());

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
