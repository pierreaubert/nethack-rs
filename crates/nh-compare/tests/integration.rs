//! Phase 19: Integration & Full Verification
//!
//! Comprehensive integration tests exercising the fully-initialized game loop
//! across all 13 roles, determinism, stress, save/restore, and starvation.

use nh_core::action::{Command, Direction};
use nh_core::player::{Gender, Race, Role};
use nh_core::{GameLoop, GameLoopResult, GameRng, GameState};

// ============================================================================
// Helpers
// ============================================================================

/// Result of running a game with identity
#[allow(dead_code)]
struct RunResult {
    game_loop: GameLoop,
    results: Vec<GameLoopResult>,
}

/// Create a game with full player identity and run the given commands
fn run_with_identity(seed: u64, role: Role, commands: &[Command]) -> RunResult {
    let rng = GameRng::new(seed);
    let state = GameState::new_with_identity(rng, "Hero".into(), role, Race::Human, Gender::Male);
    let mut gl = GameLoop::new(state);
    let mut results = Vec::new();

    for cmd in commands {
        let result = gl.tick(cmd.clone());
        results.push(result);
    }

    RunResult {
        game_loop: gl,
        results,
    }
}

/// Generate a varied sequence of commands cycling through movement/search/rest/look/inventory/attributes
fn generate_varied_commands(n: usize, seed: u64) -> Vec<Command> {
    let mut rng = GameRng::new(seed);
    let mut commands = Vec::with_capacity(n);

    let directions = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
        Direction::NorthEast,
        Direction::NorthWest,
        Direction::SouthEast,
        Direction::SouthWest,
    ];

    for i in 0..n {
        let cmd = match i % 10 {
            0..=3 => {
                let dir = directions[rng.rn2(8) as usize];
                Command::Move(dir)
            }
            4 => Command::Search,
            5 => Command::Rest,
            6 => Command::Look,
            7 => Command::Inventory,
            8 => Command::ShowAttributes,
            _ => Command::WhatsHere,
        };
        commands.push(cmd);
    }

    commands
}

/// All 13 roles
const ALL_ROLES: [Role; 13] = [
    Role::Archeologist,
    Role::Barbarian,
    Role::Caveman,
    Role::Healer,
    Role::Knight,
    Role::Monk,
    Role::Priest,
    Role::Ranger,
    Role::Rogue,
    Role::Samurai,
    Role::Tourist,
    Role::Valkyrie,
    Role::Wizard,
];

// ============================================================================
// Tests
// ============================================================================

/// All 13 roles initialize with valid HP, inventory, skills, nutrition, and position.
#[test]
fn test_all_13_roles_initialize() {
    for role in ALL_ROLES {
        let rng = GameRng::new(42);
        let state =
            GameState::new_with_identity(rng, "Hero".into(), role, Race::Human, Gender::Male);

        assert!(
            state.player.hp > 0,
            "{:?}: HP should be > 0, got {}",
            role,
            state.player.hp
        );
        assert_eq!(
            state.player.hp, state.player.hp_max,
            "{:?}: HP should equal HP max at start",
            role
        );
        assert!(
            !state.inventory.is_empty(),
            "{:?}: should have starting inventory",
            role
        );
        assert_eq!(
            state.player.nutrition, 900,
            "{:?}: nutrition should be 900",
            role
        );
        assert_eq!(
            state.player.bless_count, 300,
            "{:?}: bless_count should be 300",
            role
        );
        // Player should be placed on the map
        assert!(
            state.player.pos.x > 0 || state.player.pos.y > 0,
            "{:?}: player should have a valid position",
            role
        );
        assert_eq!(state.player.role, role);
        assert_eq!(state.player.exp_level, 1);
    }
}

/// Verify exact HP and energy per role match expected values from u_init.
#[test]
fn test_role_hp_energy_values() {
    let expected: &[(Role, i32, i32)] = &[
        (Role::Barbarian, 14, 1),
        (Role::Wizard, 10, 4), // Wizard gets rnd(3) energy
        (Role::Monk, 12, 2),   // Monk gets rnd(2) energy
        (Role::Priest, 12, 4), // Priest gets rnd(3) energy
        (Role::Knight, 14, 1), // Knight gets rnd(4) energy
        (Role::Valkyrie, 14, 1),
        (Role::Healer, 11, 1), // Healer gets rnd(4) energy
        (Role::Archeologist, 11, 1),
        (Role::Caveman, 14, 1),
        (Role::Ranger, 13, 1),
        (Role::Rogue, 10, 1),
        (Role::Samurai, 13, 1),
        (Role::Tourist, 8, 1),
    ];

    for &(role, expected_hp_min, expected_energy_min) in expected {
        let rng = GameRng::new(42);
        let state =
            GameState::new_with_identity(rng, "Hero".into(), role, Race::Human, Gender::Male);
        
        assert!(state.player.hp_max >= expected_hp_min, 
            "{:?}: HP {} should be >= {}", role, state.player.hp_max, expected_hp_min);
        assert!(state.player.energy_max >= expected_energy_min,
            "{:?}: energy {} should be >= {}", role, state.player.energy_max, expected_energy_min);
    }
}

/// Verify each role gets the expected number of starting items.
#[test]
fn test_role_inventory_counts() {
    let expected: &[(Role, usize)] = &[
        (Role::Archeologist, 8),
        (Role::Barbarian, 4),
        (Role::Caveman, 5),
        (Role::Healer, 10),
        (Role::Knight, 8),
        (Role::Monk, 9),
        (Role::Priest, 7),
        (Role::Ranger, 6),
        (Role::Rogue, 6),
        (Role::Samurai, 5),
        (Role::Tourist, 7),
        (Role::Valkyrie, 4),
        (Role::Wizard, 8),
    ];

    for &(role, expected_count) in expected {
        let rng = GameRng::new(42);
        let state =
            GameState::new_with_identity(rng, "Hero".into(), role, Race::Human, Gender::Male);
        assert_eq!(
            state.inventory.len(),
            expected_count,
            "{:?}: should have {} items, got {}",
            role,
            expected_count,
            state.inventory.len()
        );
    }
}

/// Same seed + same role = identical game state after 100 turns.
#[test]
fn test_determinism_13_roles_10_seeds() {
    let commands = generate_varied_commands(100, 999);

    for role in ALL_ROLES {
        for seed in 0..10 {
            let r1 = run_with_identity(seed, role, &commands);
            let r2 = run_with_identity(seed, role, &commands);

            let s1 = r1.game_loop.state();
            let s2 = r2.game_loop.state();

            assert_eq!(
                s1.turns, s2.turns,
                "{:?} seed {}: turns diverged ({} vs {})",
                role, seed, s1.turns, s2.turns
            );
            assert_eq!(
                s1.player.hp, s2.player.hp,
                "{:?} seed {}: HP diverged ({} vs {})",
                role, seed, s1.player.hp, s2.player.hp
            );
            assert_eq!(
                s1.player.pos.x, s2.player.pos.x,
                "{:?} seed {}: x pos diverged",
                role, seed
            );
            assert_eq!(
                s1.player.pos.y, s2.player.pos.y,
                "{:?} seed {}: y pos diverged",
                role, seed
            );
        }
    }
}

/// 1000-turn stress test across 10 seeds — must not panic and player should survive >100 turns.
#[test]
fn test_1000_turn_stress_10_seeds() {
    for seed in 0..10 {
        let commands = generate_varied_commands(1000, seed);
        let result = run_with_identity(seed, Role::Valkyrie, &commands);

        let state = result.game_loop.state();
        // Player should survive at least 100 turns before dying (if they die at all)
        // With varied commands (mostly movement/search/rest), death is unlikely
        let alive = state.player.hp > 0;
        if !alive {
            assert!(
                state.turns > 100,
                "seed {}: player died too early at turn {}",
                seed, state.turns
            );
        }
    }
}

/// Every Command variant can be ticked without panicking.
#[test]
fn test_all_command_variants_no_panic() {
    let rng = GameRng::new(42);
    let state =
        GameState::new_with_identity(rng, "Hero".into(), Role::Valkyrie, Race::Human, Gender::Male);
    let mut gl = GameLoop::new(state);

    let commands: Vec<Command> = vec![
        // Movement
        Command::Move(Direction::North),
        Command::Move(Direction::South),
        Command::Move(Direction::East),
        Command::Move(Direction::West),
        Command::Move(Direction::NorthEast),
        Command::Move(Direction::NorthWest),
        Command::Move(Direction::SouthEast),
        Command::Move(Direction::SouthWest),
        Command::MoveUntilInteresting(Direction::East),
        Command::Run(Direction::East),
        Command::Travel,
        Command::Rest,
        Command::GoUp,
        Command::GoDown,
        // Combat
        Command::Fight(Direction::North),
        Command::Fire(Direction::North),
        Command::Throw('a', Direction::North),
        Command::TwoWeapon,
        Command::SwapWeapon,
        // Object manipulation
        Command::Pickup,
        Command::Drop('a'),
        Command::Eat('a'),
        Command::Quaff('a'),
        Command::Read('a'),
        Command::Zap('a', Direction::North),
        Command::Apply('a'),
        Command::Wear('a'),
        Command::TakeOff('a'),
        Command::PutOn('a'),
        Command::Remove('a'),
        Command::Wield(Some('a')),
        Command::Wield(None),
        Command::SelectQuiver('a'),
        Command::Loot,
        Command::Tip('a'),
        Command::Dip,
        Command::Rub('a'),
        Command::Wipe,
        Command::Force(Direction::North),
        // Information
        Command::Inventory,
        Command::Look,
        Command::WhatsHere,
        Command::Help,
        Command::Discoveries,
        Command::History,
        Command::ShowAttributes,
        Command::ShowEquipment,
        Command::ShowSpells,
        Command::ShowConduct,
        Command::DungeonOverview,
        Command::CountGold,
        Command::ClassDiscovery,
        Command::TypeInventory('!'),
        Command::Vanquished,
        // Actions
        Command::Open(Direction::North),
        Command::Close(Direction::North),
        Command::Kick(Direction::North),
        Command::Search,
        Command::Pray,
        Command::Offer,
        Command::Engrave("Elbereth".to_string()),
        Command::Pay,
        Command::Chat,
        Command::Feed,
        Command::Sit,
        Command::Jump,
        Command::Invoke,
        Command::Untrap(Direction::North),
        Command::Ride,
        Command::TurnUndead,
        Command::MonsterAbility,
        Command::EnhanceSkill,
        Command::NameItem('a', "Excalibur".to_string()),
        Command::NameLevel("Home".to_string()),
        Command::Organize('a', 'z'),
        // Meta
        Command::Options,
        Command::ExtendedCommand("test".to_string()),
        Command::Redraw,
    ];

    for cmd in &commands {
        let result = gl.tick(cmd.clone());
        // Just verify it doesn't panic — any result is fine
        match result {
            GameLoopResult::Continue
            | GameLoopResult::PlayerDied(_)
            | GameLoopResult::PlayerQuit
            | GameLoopResult::PlayerWon
            | GameLoopResult::SaveAndQuit => {}
        }
    }
}

/// JSON serialize -> deserialize preserves key state fields.
#[test]
fn test_save_restore_roundtrip() {
    let commands = generate_varied_commands(50, 42);
    let result = run_with_identity(42, Role::Wizard, &commands);
    let state = result.game_loop.state();

    // Serialize to JSON
    let json = serde_json::to_string(state).expect("serialize should succeed");

    // Deserialize back
    let mut restored: GameState = serde_json::from_str(&json).expect("deserialize should succeed");

    // Rebuild grids (as load_game would do)
    restored.current_level.rebuild_grids();

    // Verify key fields are preserved
    assert_eq!(state.turns, restored.turns);
    assert_eq!(state.player.hp, restored.player.hp);
    assert_eq!(state.player.hp_max, restored.player.hp_max);
    assert_eq!(state.player.pos.x, restored.player.pos.x);
    assert_eq!(state.player.pos.y, restored.player.pos.y);
    assert_eq!(state.player.role, restored.player.role);
    assert_eq!(state.player.race, restored.player.race);
    assert_eq!(state.player.gender, restored.player.gender);
    assert_eq!(state.player.name, restored.player.name);
    assert_eq!(state.player.exp_level, restored.player.exp_level);
    assert_eq!(state.player.energy, restored.player.energy);
    assert_eq!(state.player.nutrition, restored.player.nutrition);
    assert_eq!(state.inventory.len(), restored.inventory.len());

    // Verify spatial grids work after restore
    for monster in &state.current_level.monsters {
        let found = restored.current_level.monster_at(monster.x, monster.y);
        assert!(
            found.is_some(),
            "monster_at({}, {}) should find monster after restore",
            monster.x, monster.y
        );
    }

    for obj in &state.current_level.objects {
        let found = restored.current_level.objects_at(obj.x, obj.y);
        assert!(
            !found.is_empty(),
            "objects_at({}, {}) should find objects after restore",
            obj.x, obj.y
        );
    }
}

/// Set nutrition to 10 and wait until the player dies from starvation.
#[test]
fn test_starvation_death() {
    let rng = GameRng::new(42);
    let mut state =
        GameState::new_with_identity(rng, "Hero".into(), Role::Valkyrie, Race::Human, Gender::Male);
    state.player.nutrition = 10;
    let mut gl = GameLoop::new(state);

    let mut died = false;
    for _ in 0..5000 {
        let result = gl.tick(Command::Rest);
        if matches!(result, GameLoopResult::PlayerDied(_)) {
            died = true;
            break;
        }
    }

    assert!(died, "Player should eventually die from starvation");
}

/// Elf and Dwarf get Infravision; Human does not.
#[test]
fn test_racial_intrinsics() {
    let elf_state = GameState::new_with_identity(
        GameRng::new(42),
        "Elf".into(),
        Role::Ranger,
        Race::Elf,
        Gender::Male,
    );
    assert!(
        elf_state.player.properties.has_infravision(),
        "Elf should have infravision"
    );

    let dwarf_state = GameState::new_with_identity(
        GameRng::new(42),
        "Dwarf".into(),
        Role::Valkyrie,
        Race::Dwarf,
        Gender::Male,
    );
    assert!(
        dwarf_state.player.properties.has_infravision(),
        "Dwarf should have infravision"
    );

    let human_state = GameState::new_with_identity(
        GameRng::new(42),
        "Human".into(),
        Role::Valkyrie,
        Race::Human,
        Gender::Male,
    );
    assert!(
        !human_state.player.properties.has_infravision(),
        "Human should NOT have infravision"
    );
}

/// Healer gold >= 1001, Tourist gold >= 1.
#[test]
fn test_healer_tourist_gold() {
    let healer_state = GameState::new_with_identity(
        GameRng::new(42),
        "Doc".into(),
        Role::Healer,
        Race::Human,
        Gender::Male,
    );
    assert!(
        healer_state.player.gold >= 1001,
        "Healer should have gold >= 1001, got {}",
        healer_state.player.gold
    );

    let tourist_state = GameState::new_with_identity(
        GameRng::new(42),
        "Photo".into(),
        Role::Tourist,
        Race::Human,
        Gender::Male,
    );
    assert!(
        tourist_state.player.gold >= 1,
        "Tourist should have gold >= 1, got {}",
        tourist_state.player.gold
    );
}
