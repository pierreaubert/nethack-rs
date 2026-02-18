//! Phase 31: Save/Restore System Tests
//!
//! Behavioral tests for the save/restore system, covering:
//! - Player field round-trip preservation
//! - Level data serialization
//! - Inventory persistence
//! - Monster serialization
//! - Full GameState round-trip
//! - Bones file structure
//! - Multi-level persistence
//! - Save format stability

use nh_core::dungeon::{BonesFile, BonesHeader, DLevel, Level, Trap, TrapType};
use nh_core::monster::{Monster, MonsterId};
use nh_core::object::{BucStatus, Object, ObjectClass};
use nh_core::player::{Gender, HungerState, Race, Role};
use nh_core::save::{
    load_game, save_game, save_game_compact, SaveHeader, SAVE_VERSION,
};
use nh_core::{GameRng, GameState};

// ============================================================================
// Helpers
// ============================================================================

/// Create a GameState with a known seed and customized player fields
fn make_test_state(seed: u64) -> GameState {
    let rng = GameRng::new(seed);
    GameState::new_with_identity(rng, "TestHero".into(), Role::Valkyrie, Race::Human, Gender::Female, Role::Valkyrie.default_alignment())
}

/// Unique temp file path (avoids collisions across parallel tests)
fn temp_save_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("nhrs_phase31_{}.json", name))
}

/// Round-trip a GameState through save/load, returning the loaded state.
/// Cleans up the temp file afterward.
fn round_trip(state: &GameState, label: &str) -> GameState {
    let path = temp_save_path(label);
    save_game(state, &path).expect("save_game should succeed");
    let loaded = load_game(&path).expect("load_game should succeed");
    std::fs::remove_file(&path).ok();
    loaded
}

// ============================================================================
// Test 1: Player fields survive save/load round-trip
// ============================================================================

#[test]
fn test_player_save_restore() {
    let mut state = make_test_state(1001);

    // Customize player fields so we can detect round-trip fidelity
    state.player.name = "Brunhilde".to_string();
    state.player.hp = 42;
    state.player.hp_max = 60;
    state.player.exp_level = 7;
    state.player.energy = 33;
    state.player.energy_max = 50;
    state.player.nutrition = 800;
    state.player.hunger_state = HungerState::NotHungry;
    state.player.luck = 3;
    state.player.armor_class = -2;
    state.player.pos.x = 20;
    state.player.pos.y = 10;

    let loaded = round_trip(&state, "player");

    assert_eq!(loaded.player.name, "Brunhilde");
    assert_eq!(loaded.player.role, Role::Valkyrie);
    assert_eq!(loaded.player.race, Race::Human);
    assert_eq!(loaded.player.gender, Gender::Female);
    assert_eq!(loaded.player.hp, 42);
    assert_eq!(loaded.player.hp_max, 60);
    assert_eq!(loaded.player.exp_level, 7);
    assert_eq!(loaded.player.energy, 33);
    assert_eq!(loaded.player.energy_max, 50);
    assert_eq!(loaded.player.nutrition, 800);
    assert_eq!(loaded.player.hunger_state, HungerState::NotHungry);
    assert_eq!(loaded.player.luck, 3);
    assert_eq!(loaded.player.armor_class, -2);
    assert_eq!(loaded.player.pos.x, 20);
    assert_eq!(loaded.player.pos.y, 10);
}

// ============================================================================
// Test 2: Level data survives save/load
// ============================================================================

#[test]
fn test_level_save_restore() {
    let mut state = make_test_state(2002);

    // Verify the current level dlevel is preserved
    let original_dlevel = state.current_level.dlevel;

    // Add a trap to the level
    state.current_level.traps.push(Trap {
        x: 15,
        y: 8,
        trap_type: TrapType::Pit,
        activated: false,
        seen: true,
        once: false,
        madeby_u: false,
        launch_oid: None,
    });

    // Mark a flag
    state.current_level.flags.has_shop = true;

    let loaded = round_trip(&state, "level");

    assert_eq!(loaded.current_level.dlevel, original_dlevel);
    assert!(!loaded.current_level.traps.is_empty(), "traps should be preserved");

    let trap = &loaded.current_level.traps.last().unwrap();
    assert_eq!(trap.x, 15);
    assert_eq!(trap.y, 8);
    assert!(trap.seen);
    assert!(loaded.current_level.flags.has_shop);
}

// ============================================================================
// Test 3: Inventory items survive save/load
// ============================================================================

#[test]
fn test_inventory_save_restore() {
    let mut state = make_test_state(3003);

    // Create some inventory items
    let mut sword = Object::default();
    sword.class = ObjectClass::Weapon;
    sword.enchantment = 3;
    sword.buc = BucStatus::Blessed;
    sword.known = true;
    sword.quantity = 1;
    sword.name = Some("Excalibur".to_string());

    let mut arrows = Object::default();
    arrows.class = ObjectClass::Weapon;
    arrows.quantity = 15;
    arrows.poisoned = true;

    let mut potion = Object::default();
    potion.class = ObjectClass::Potion;
    potion.buc = BucStatus::Cursed;
    potion.buc_known = true;

    let base_count = state.inventory.len();
    state.inventory.push(sword);
    state.inventory.push(arrows);
    state.inventory.push(potion);

    let loaded = round_trip(&state, "inventory");

    assert_eq!(loaded.inventory.len(), base_count + 3);

    // Our added items are at the end
    let loaded_sword = &loaded.inventory[base_count];
    assert_eq!(loaded_sword.class, ObjectClass::Weapon);
    assert_eq!(loaded_sword.enchantment, 3);
    assert_eq!(loaded_sword.buc, BucStatus::Blessed);
    assert!(loaded_sword.known);
    assert_eq!(loaded_sword.name.as_deref(), Some("Excalibur"));

    let loaded_arrows = &loaded.inventory[base_count + 1];
    assert_eq!(loaded_arrows.quantity, 15);
    assert!(loaded_arrows.poisoned);

    let loaded_potion = &loaded.inventory[base_count + 2];
    assert_eq!(loaded_potion.class, ObjectClass::Potion);
    assert_eq!(loaded_potion.buc, BucStatus::Cursed);
    assert!(loaded_potion.buc_known);
}

// ============================================================================
// Test 4: Monsters on the level survive save/load
// ============================================================================

#[test]
fn test_monster_save_restore() {
    let mut state = make_test_state(4004);

    // Add monsters at known positions
    let mut goblin = Monster::new(MonsterId::NONE, 5, 20, 12);
    goblin.name = "goblin".to_string();
    goblin.hp = 8;
    goblin.hp_max = 8;
    goblin.level = 1;
    goblin.state.peaceful = false;
    goblin.state.sleeping = true;

    let mut npc = Monster::new(MonsterId::NONE, 10, 30, 15);
    npc.name = "shopkeeper".to_string();
    npc.hp = 50;
    npc.hp_max = 50;
    npc.level = 12;
    npc.state.peaceful = true;

    let _goblin_id = state.current_level.add_monster(goblin);
    let _npc_id = state.current_level.add_monster(npc);

    let loaded = round_trip(&state, "monsters");

    // Grids are rebuilt by load_game, so monster_at should work
    let loaded_goblin = loaded.current_level.monster_at(20, 12);
    assert!(loaded_goblin.is_some(), "goblin should be at (20,12) after load");
    let loaded_goblin = loaded_goblin.unwrap();
    assert_eq!(loaded_goblin.name, "goblin");
    assert_eq!(loaded_goblin.hp, 8);
    assert!(loaded_goblin.state.sleeping);
    assert!(!loaded_goblin.state.peaceful);

    let loaded_npc = loaded.current_level.monster_at(30, 15);
    assert!(loaded_npc.is_some(), "shopkeeper should be at (30,15) after load");
    let loaded_npc = loaded_npc.unwrap();
    assert_eq!(loaded_npc.name, "shopkeeper");
    assert!(loaded_npc.state.peaceful);
    assert_eq!(loaded_npc.level, 12);
}

// ============================================================================
// Test 5: Full GameState round-trip
// ============================================================================

#[test]
fn test_game_state_save_restore() {
    let mut state = make_test_state(5005);

    // Advance the turn counter
    state.turns = 1234;
    state.monster_turns = 1200;

    // Set some flags
    state.flags.verbose = true;
    state.flags.autopickup = true;

    // Add an item to inventory
    let base_inv_count = state.inventory.len();
    let mut ring = Object::default();
    ring.class = ObjectClass::Ring;
    ring.enchantment = 1;
    state.inventory.push(ring);

    // Add a monster
    let base_mon_count = state.current_level.monsters.len();
    let zombie = Monster::new(MonsterId::NONE, 20, 25, 10);
    state.current_level.add_monster(zombie);

    let loaded = round_trip(&state, "gamestate");

    assert_eq!(loaded.turns, 1234);
    assert_eq!(loaded.monster_turns, 1200);
    assert_eq!(loaded.player.name, state.player.name);
    assert_eq!(loaded.player.role, state.player.role);
    assert_eq!(loaded.inventory.len(), base_inv_count + 1);
    assert_eq!(loaded.inventory[base_inv_count].class, ObjectClass::Ring);
    assert!(loaded.current_level.monsters.len() > base_mon_count);
    assert!(loaded.flags.verbose);
    assert!(loaded.flags.autopickup);
}

// ============================================================================
// Test 6: BonesFile/BonesHeader structure exists and can be created
// ============================================================================

#[test]
fn test_bones_file_structure() {
    let header = BonesHeader::new(
        "FallenHero".to_string(),
        "Wizard".to_string(),
        "Elf".to_string(),
        DLevel::new(0, 5),
        "killed by a cockatrice corpse".to_string(),
        2500,
        12,
        1200,
        80,
    );

    assert_eq!(header.player_name, "FallenHero");
    assert_eq!(header.role, "Wizard");
    assert_eq!(header.race, "Elf");
    assert_eq!(header.dlevel, DLevel::new(0, 5));
    assert_eq!(header.death_reason, "killed by a cockatrice corpse");
    assert_eq!(header.turn_count, 2500);
    assert_eq!(header.exp_level, 12);
    assert_eq!(header.gold, 1200);
    assert_eq!(header.max_hp, 80);

    // Create a BonesFile and verify it round-trips through serde
    let level = Level::new(DLevel::new(0, 5));
    let bones = BonesFile::new(header, level);
    assert!(bones.is_compatible());

    let json = serde_json::to_string(&bones).expect("BonesFile should serialize");
    let restored: BonesFile = serde_json::from_str(&json).expect("BonesFile should deserialize");

    assert_eq!(restored.header.player_name, "FallenHero");
    assert_eq!(restored.header.death_reason, "killed by a cockatrice corpse");
    assert_eq!(restored.header.dlevel, DLevel::new(0, 5));
    assert!(restored.is_compatible());
}

// ============================================================================
// Test 7: Multiple levels can be saved/restored
// ============================================================================

#[test]
fn test_multi_level_persistence() {
    let mut state = make_test_state(7007);

    // Create additional visited levels and store them in the levels HashMap
    let mines_dlevel = DLevel::new(2, 3);
    let mut mines_level = Level::new(mines_dlevel);
    mines_level.flags.has_shop = true;
    let mines_monster = Monster::new(MonsterId::NONE, 8, 10, 10);
    mines_level.add_monster(mines_monster);
    state.levels.insert(mines_dlevel, mines_level);

    let deep_dlevel = DLevel::new(0, 10);
    let mut deep_level = Level::new(deep_dlevel);
    deep_level.flags.has_temple = true;
    let deep_obj = Object::default();
    deep_level.add_object(deep_obj, 5, 5);
    state.levels.insert(deep_dlevel, deep_level);

    let loaded = round_trip(&state, "multilevel");

    // Verify both extra levels are present
    assert!(
        loaded.levels.contains_key(&mines_dlevel),
        "Mines level should be preserved"
    );
    assert!(
        loaded.levels.contains_key(&deep_dlevel),
        "Deep level should be preserved"
    );

    let loaded_mines = &loaded.levels[&mines_dlevel];
    assert!(loaded_mines.flags.has_shop);
    assert!(!loaded_mines.monsters.is_empty(), "Mines monster should be preserved");

    let loaded_deep = &loaded.levels[&deep_dlevel];
    assert!(loaded_deep.flags.has_temple);
    assert!(!loaded_deep.objects.is_empty(), "Deep level object should be preserved");
}

// ============================================================================
// Test 8: Save format stability — saved data loads back consistently
// ============================================================================

#[test]
fn test_save_format_stability() {
    let mut state = make_test_state(8008);
    state.player.name = "StabilityTest".to_string();
    state.player.hp = 55;
    state.turns = 999;

    // Add a named item to inventory for richer comparison
    let base_inv_count = state.inventory.len();
    let mut wand = Object::default();
    wand.class = ObjectClass::Wand;
    wand.enchantment = -1;
    wand.buc = BucStatus::Cursed;
    wand.name = Some("wand of death".to_string());
    state.inventory.push(wand);

    // Save twice — once pretty, once compact — and verify both load identically
    let path_pretty = temp_save_path("stability_pretty");
    let path_compact = temp_save_path("stability_compact");

    save_game(&state, &path_pretty).expect("pretty save should succeed");
    save_game_compact(&state, &path_compact).expect("compact save should succeed");

    let loaded_pretty = load_game(&path_pretty).expect("pretty load should succeed");
    let loaded_compact = load_game(&path_compact).expect("compact load should succeed");

    // Core fields must match across both formats
    assert_eq!(loaded_pretty.player.name, "StabilityTest");
    assert_eq!(loaded_compact.player.name, "StabilityTest");
    assert_eq!(loaded_pretty.player.hp, loaded_compact.player.hp);
    assert_eq!(loaded_pretty.turns, loaded_compact.turns);
    assert_eq!(loaded_pretty.inventory.len(), loaded_compact.inventory.len());

    let wand_p = &loaded_pretty.inventory[base_inv_count];
    let wand_c = &loaded_compact.inventory[base_inv_count];
    assert_eq!(wand_p.class, wand_c.class);
    assert_eq!(wand_p.enchantment, wand_c.enchantment);
    assert_eq!(wand_p.buc, wand_c.buc);
    assert_eq!(wand_p.name, wand_c.name);

    // Re-save the loaded-pretty state and load again — third generation must match
    let path_resave = temp_save_path("stability_resave");
    save_game(&loaded_pretty, &path_resave).expect("re-save should succeed");
    let loaded_resave = load_game(&path_resave).expect("re-load should succeed");

    assert_eq!(loaded_resave.player.name, "StabilityTest");
    assert_eq!(loaded_resave.player.hp, 55);
    assert_eq!(loaded_resave.turns, 999);
    assert_eq!(loaded_resave.inventory.len(), base_inv_count + 1);
    assert_eq!(loaded_resave.inventory[base_inv_count].name.as_deref(), Some("wand of death"));

    // Verify header fields are correct
    let header = SaveHeader::new(&state);
    assert_eq!(header.player_name, "StabilityTest");
    assert_eq!(header.version, SAVE_VERSION);
    assert_eq!(header.turns, 999);
    assert!(header.validate().is_ok());

    // Cleanup
    std::fs::remove_file(&path_pretty).ok();
    std::fs::remove_file(&path_compact).ok();
    std::fs::remove_file(&path_resave).ok();
}
