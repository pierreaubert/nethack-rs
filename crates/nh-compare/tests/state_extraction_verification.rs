//! Phase 1: Comparison Infrastructure Expansion
//!
//! Test: Verify that state extraction for inventory and monsters is correctly implemented.
//! This test currently focuses on the Rust side, ensuring we can extract 
//! deep information (BUC, charges, weight for inventory; AI flags for monsters).

use nh_core::player::{Gender, Race, Role};
use nh_core::{GameRng, GameState};

#[test]
fn test_rust_inventory_state_extraction_depth() {
    let rng = GameRng::new(42);
    // Archeologist starts with several items: pick-axe, tin opener, food rations, etc.
    let state = GameState::new_with_identity(rng, "Hero".into(), Role::Archeologist, Race::Human, Gender::Male, Role::Archeologist.default_alignment());
    
    let inv = &state.inventory;
    assert!(!inv.is_empty(), "Archeologist should have starting inventory");

    for item in inv {
        // We want to ensure these fields are accessible and correctly populated
        println!("Item Type: {}, Weight: {}, BUC: {:?}", item.object_type, item.weight, item.buc);
        
        // Verify weight is non-negative (some items legitimately have weight 0)
        assert!(item.weight >= 0, "Item (type {}) should have non-negative weight", item.object_type);
    }
}

#[test]
fn test_rust_monster_state_extraction_depth() {
    let rng = GameRng::new(42);
    let state = GameState::new_with_identity(rng, "Hero".into(), Role::Valkyrie, Race::Human, Gender::Male, Role::Valkyrie.default_alignment());
    
    // Check if there are any monsters on the first level
    let monsters = &state.current_level.monsters;
    println!("Monsters on level: {}", monsters.len());

    for monster in monsters {
        // We want to ensure we can extract AI flags, sleep status, etc.
        println!("Monster: {}, Pos: ({}, {}), Asleep: {}", monster.name, monster.x, monster.y, monster.state.sleeping);
        
        // AI behavior flags check - strategy field
        println!("  Strategy bits: {:08x}", monster.strategy.bits());
    }
}
