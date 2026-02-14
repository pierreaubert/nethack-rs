//! Step 7: Command coverage tests
//!
//! Verifies that every Command variant is handled and produces the expected
//! result type. Identifies stub vs real implementations.

use nh_core::action::{Command, Direction};
use nh_core::{GameLoop, GameLoopResult, GameState, GameRng};

// ============================================================================
// Helpers
// ============================================================================

fn test_gameloop() -> GameLoop {
    let rng = GameRng::new(42);
    let state = GameState::new(rng);
    GameLoop::new(state)
}

/// Execute a command and return the GameLoopResult.
fn exec(gl: &mut GameLoop, cmd: Command) -> GameLoopResult {
    gl.tick(cmd)
}

// ============================================================================
// 7.1: Movement commands
// ============================================================================

#[test]
fn test_command_move() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Move(Direction::East));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_run() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Run(Direction::South));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_move_until_interesting() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::MoveUntilInteresting(Direction::North));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_rest() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Rest);
    assert!(matches!(result, GameLoopResult::Continue));
    // Rest produces a message
    assert!(gl.state().messages.iter().any(|m| m.contains("wait")));
}

#[test]
fn test_command_go_up() {
    let mut gl = test_gameloop();
    // No stairs, so this should fail but not crash
    let result = exec(&mut gl, Command::GoUp);
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_go_down() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::GoDown);
    assert!(matches!(result, GameLoopResult::Continue));
}

// ============================================================================
// 7.2: Combat commands
// ============================================================================

#[test]
fn test_command_fight_empty() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Fight(Direction::East));
    assert!(matches!(result, GameLoopResult::Continue));
    // Fighting empty space should produce a message
    assert!(gl.state().messages.iter().any(|m| m.contains("empty space") || m.contains("strike")));
}

#[test]
fn test_command_fire_no_ammo() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Fire(Direction::East));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_throw_no_item() {
    let mut gl = test_gameloop();
    // Throw with a letter that doesn't exist in inventory
    let result = exec(&mut gl, Command::Throw('z', Direction::East));
    assert!(matches!(result, GameLoopResult::Continue));
}

// ============================================================================
// 7.3: Object manipulation commands
// ============================================================================

#[test]
fn test_command_pickup() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Pickup);
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_drop_no_item() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Drop('a'));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_eat_no_item() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Eat('a'));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_quaff_no_item() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Quaff('a'));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_read_no_item() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Read('a'));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_zap_no_item() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Zap('a', Direction::East));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_apply_no_item() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Apply('a'));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_wear_no_item() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Wear('a'));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_takeoff_no_item() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::TakeOff('a'));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_puton_no_item() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::PutOn('a'));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_remove_no_item() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Remove('a'));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_wield_none() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Wield(None));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_wield_no_item() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Wield(Some('z')));
    assert!(matches!(result, GameLoopResult::Continue));
}

// ============================================================================
// 7.4: Information commands (no time cost)
// ============================================================================

#[test]
fn test_command_inventory() {
    let mut gl = test_gameloop();
    let turns_before = gl.state().turns;
    let result = exec(&mut gl, Command::Inventory);
    assert!(matches!(result, GameLoopResult::Continue));
    // Inventory should not advance game time
    assert_eq!(gl.state().turns, turns_before);
}

#[test]
fn test_command_look() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Look);
    assert!(matches!(result, GameLoopResult::Continue));
    assert!(gl.state().messages.iter().any(|m| m.contains("look")));
}

#[test]
fn test_command_whats_here() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::WhatsHere);
    assert!(matches!(result, GameLoopResult::Continue));
    // Should report nothing or items
    assert!(gl.state().messages.iter().any(|m| m.contains("nothing") || m.contains("item")));
}

#[test]
fn test_command_help() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Help);
    assert!(matches!(result, GameLoopResult::Continue));
    // Help should produce multiple messages
    assert!(gl.state().messages.len() >= 3);
}

#[test]
fn test_command_discoveries() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Discoveries);
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_history() {
    let mut gl = test_gameloop();
    // First generate some messages
    exec(&mut gl, Command::Rest);
    let result = exec(&mut gl, Command::History);
    assert!(matches!(result, GameLoopResult::Continue));
}

// ============================================================================
// 7.5: Directional actions
// ============================================================================

#[test]
fn test_command_open() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Open(Direction::East));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_close() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Close(Direction::East));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_kick() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Kick(Direction::East));
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_search() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Search);
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_pray() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Pray);
    assert!(matches!(result, GameLoopResult::Continue));
}

#[test]
fn test_command_engrave() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Engrave("Elbereth".to_string()));
    assert!(matches!(result, GameLoopResult::Continue));
}

// ============================================================================
// 7.6: Meta commands
// ============================================================================

#[test]
fn test_command_save() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Save);
    assert!(matches!(result, GameLoopResult::SaveAndQuit));
}

#[test]
fn test_command_quit() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Quit);
    assert!(matches!(result, GameLoopResult::PlayerQuit));
}

// ============================================================================
// 7.7: Basic command handling (should not crash, produce appropriate messages)
// ============================================================================

#[test]
fn test_command_travel() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Travel);
    assert!(matches!(result, GameLoopResult::Continue));
    assert!(!gl.state().messages.is_empty(), "Travel should produce a message");
}

#[test]
fn test_command_offer() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Offer);
    assert!(matches!(result, GameLoopResult::Continue));
    assert!(
        gl.state().messages.iter().any(|m| m.contains("altar")),
        "Offer should mention altar requirement"
    );
}

#[test]
fn test_command_dip() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Dip);
    assert!(matches!(result, GameLoopResult::Continue));
    assert!(!gl.state().messages.is_empty(), "Dip should produce a message");
}

#[test]
fn test_command_pay() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Pay);
    assert!(matches!(result, GameLoopResult::Continue));
    assert!(
        gl.state().messages.iter().any(|m| m.contains("nobody")),
        "Pay outside shop should say nobody to pay"
    );
}

#[test]
fn test_command_chat() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Chat);
    assert!(matches!(result, GameLoopResult::Continue));
    assert!(
        gl.state().messages.iter().any(|m| m.contains("nobody")),
        "Chat should say nobody to chat with"
    );
}

#[test]
fn test_command_sit() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Sit);
    assert!(matches!(result, GameLoopResult::Continue));
    assert!(
        gl.state().messages.iter().any(|m| m.contains("sit")),
        "Sit should mention sitting"
    );
}

#[test]
fn test_command_options() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Options);
    assert!(matches!(result, GameLoopResult::Continue));
    assert!(!gl.state().messages.is_empty(), "Options should produce a message");
}

#[test]
fn test_command_extended() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::ExtendedCommand("test".to_string()));
    assert!(matches!(result, GameLoopResult::Continue));
    assert!(
        gl.state().messages.iter().any(|m| m.contains("Unknown")),
        "Unknown extended command should say so"
    );
}

#[test]
fn test_command_redraw() {
    let mut gl = test_gameloop();
    let result = exec(&mut gl, Command::Redraw);
    assert!(matches!(result, GameLoopResult::Continue));
    // Redraw produces no messages (UI-layer operation)
}

// ============================================================================
// 7.8: Coverage summary
// ============================================================================

#[test]
fn test_command_coverage_summary() {
    // Count command variants by category
    let implemented = [
        "Move", "Run", "MoveUntilInteresting", "Rest",
        "GoUp", "GoDown", "Search",
        "Fight", "Fire", "Throw",
        "Pickup", "Drop", "Eat", "Quaff", "Read", "Zap",
        "Apply", "Wear", "TakeOff", "PutOn", "Remove", "Wield",
        "Open", "Close", "Kick", "Pray", "Engrave",
        "Inventory", "Look", "WhatsHere", "Help", "Discoveries", "History",
        "Save", "Quit",
    ];

    let unimplemented = [
        "Travel", "Offer", "Dip", "Pay", "Chat", "Sit",
        "Options", "ExtendedCommand", "Redraw",
    ];

    let total = implemented.len() + unimplemented.len();
    let coverage = implemented.len() as f64 / total as f64 * 100.0;

    println!("\n=== Command Coverage Summary ===");
    println!("Total command variants: {}", total);
    println!("Implemented: {}", implemented.len());
    println!("Unimplemented: {}", unimplemented.len());
    println!("Coverage: {:.1}%", coverage);
    println!();
    println!("Unimplemented commands:");
    for cmd in &unimplemented {
        println!("  - {}", cmd);
    }
    println!();
    println!("=== C NetHack Commands Not in Rust Enum ===");
    println!("(from plan Step 7.1-7.3):");
    println!("  - Loot, Tip, Rub, Untrap, Force");
    println!("  - Wipe, Ride, TwoWeapon, SwapWeapon");
    println!("  - EnhanceSkill, SelectQuiver");
    println!("  - TurnUndead, MonsterAbility, Jump");
    println!("  - Invoke, NameLevel, NameItem");
    println!("  - ShowAttributes, ShowEquipment, ShowSpells");
    println!("  - ShowConduct, DungeonOverview, CountGold");
    println!("  - ClassDiscovery, TypeInventory, Organize");
    println!("  - Vanquished");
    println!("  - WizGenesis, WizIdentify, WizIntrinsic, etc.");
}
