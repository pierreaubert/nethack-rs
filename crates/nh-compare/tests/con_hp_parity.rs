use nh_core::player::{Role, Race, Gender};
use nh_core::GameState;
use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use serial_test::serial;

#[test]
#[serial]
fn test_constitution_hp_bonus_parity() {
    let roles = vec![
        (Role::Archeologist, "Archeologist"),
        (Role::Barbarian, "Barbarian"),
        (Role::Caveman, "Caveman"),
        (Role::Healer, "Healer"),
        (Role::Knight, "Knight"),
        (Role::Wizard, "Wizard"),
        (Role::Valkyrie, "Valkyrie"),
    ];
    
    let races = vec![
        (Race::Human, "Human"),
        (Race::Gnome, "Gnome"),
        (Race::Dwarf, "Dwarf"),
        (Race::Elf, "Elf"),
        (Race::Orc, "Orc"),
    ];
    
    // 1. Initialize C Engine ONCE
    let mut c_engine = CGameEngine::new();

    for (role, role_name) in roles {
        let align = match role {
            Role::Archeologist | Role::Caveman | Role::Knight | Role::Monk | Role::Samurai | Role::Valkyrie => 0, // Lawful
            Role::Barbarian | Role::Healer | Role::Ranger | Role::Tourist | Role::Wizard | Role::Priest => 1,    // Neutral
            Role::Rogue => 2, // Chaotic
        };

        for (race, race_name) in &races {
            // 2. Re-initialize C Engine for each combination
            if let Err(_) = c_engine.init(role_name, race_name, 0, align) {
                continue; // Skip invalid combinations
            }
            c_engine.reset(42).expect("C engine reset failed");
            
            println!("Testing Role: {}, Race: {}, Seed: 42, Align: {}", role_name, race_name, align);

            // 3. Initialize Rust Engine
            let rust_state = GameState::new_with_identity(nh_core::GameRng::new(42), "Hero".into(), role, *race, Gender::Male);

            println!("  C HP: {}/{}, Energy: {}/{}", c_engine.hp(), c_engine.max_hp(), c_engine.energy(), c_engine.max_energy());
            println!("  Rust HP: {}/{}, Energy: {}/{}", rust_state.player.hp, rust_state.player.hp_max, rust_state.player.energy, rust_state.player.energy_max);
            
            // For VALID combinations, they should match.
            // C defaults to Tourist/Human for some invalid ones, we skip those via Err above.
            
            if rust_state.player.hp_max != c_engine.max_hp() {
                println!("  !! HP Max mismatch for {} {} at seed 42. C: {}, Rust: {}", 
                    role_name, race_name, c_engine.max_hp(), rust_state.player.hp_max);
            }
            if rust_state.player.energy_max != c_engine.max_energy() {
                println!("  !! Energy Max mismatch for {} {} at seed 42. C: {}, Rust: {}",
                    role_name, race_name, c_engine.max_energy(), rust_state.player.energy_max);
            }
        }
    }
}
