//! Gap verification: Missing command enum variants (Plan Step 7)
//!
//! Checks which C NetHack commands are missing from the Rust Command enum
//! and which implemented commands are still stubs.

use nh_core::action::{Command, Direction};
use nh_core::{GameLoop, GameLoopResult, GameState, GameRng};

fn test_gameloop() -> GameLoop {
    let rng = GameRng::new(42);
    let state = GameState::new(rng);
    GameLoop::new(state)
}

fn exec(gl: &mut GameLoop, cmd: Command) -> GameLoopResult {
    gl.tick(cmd)
}

// ============================================================================
// 7.1: Commands that exist but are stubs ("not yet implemented")
// ============================================================================

/// Returns true if the command is a stub (produces "not yet implemented" message)
fn is_stub_command(cmd: Command) -> bool {
    let mut gl = test_gameloop();
    gl.state_mut().messages.clear();
    let _result = exec(&mut gl, cmd);
    gl.state().messages.iter().any(|m| m.contains("not yet implemented"))
}

#[test]
fn test_stub_commands_audit() {
    let stub_commands: Vec<(&str, Command)> = vec![
        ("Travel", Command::Travel),
        ("Offer", Command::Offer),
        ("Dip", Command::Dip),
        ("Pay", Command::Pay),
        ("Chat", Command::Chat),
        ("Sit", Command::Sit),
        ("Options", Command::Options),
        ("ExtendedCommand", Command::ExtendedCommand("test".to_string())),
        ("Redraw", Command::Redraw),
    ];

    let mut still_stubbed = Vec::new();
    let mut now_implemented = Vec::new();

    for (name, cmd) in stub_commands {
        if is_stub_command(cmd) {
            still_stubbed.push(name);
        } else {
            now_implemented.push(name);
        }
    }

    println!("\n=== Stub Command Audit ===");
    println!("Still stubbed: {}", still_stubbed.len());
    for name in &still_stubbed {
        println!("  STUB: {}", name);
    }
    if !now_implemented.is_empty() {
        println!("Now implemented: {}", now_implemented.len());
        for name in &now_implemented {
            println!("  DONE: {}", name);
        }
    }
}

// ============================================================================
// 7.2: Commands missing from the Rust enum entirely (Plan Step 7.1-7.3)
// ============================================================================

const ACTION_MOD_PATH: &str =
    "/Users/pierre/src/games/nethack-rs/crates/nh-core/src/action/mod.rs";

/// Check if a string appears as a variant name in the Command enum source
fn command_variant_exists_in_source(variant: &str) -> bool {
    let source = std::fs::read_to_string(ACTION_MOD_PATH)
        .expect("Could not read action/mod.rs");
    source.contains(variant)
}

#[test]
fn test_missing_action_commands() {
    // Plan Step 7.1: Missing action commands from C
    let required_action_commands = [
        ("Loot", "doloot() - loot containers"),
        ("Tip", "dotip() - tip over containers"),
        ("Rub", "dorub() - rub lamp/tstone"),
        ("Untrap", "dountrap() - disarm traps"),
        ("Force", "doforce() - force locks"),
        ("Wipe", "dowipe() - wipe face"),
        ("Ride", "doride() - mount steeds"),
        ("TwoWeapon", "dotwoweapon() - dual wield"),
        ("SwapWeapon", "doswapweapon() - swap main/offhand"),
        ("EnhanceSkill", "enhance_weapon_skill()"),
        ("SelectQuiver", "dowieldquiver() - ready projectiles"),
        ("TurnUndead", "doturn() - cleric ability"),
        ("MonsterAbility", "domonability() - #monster"),
        ("Jump", "dojump() - #jump"),
        ("Invoke", "doinvoke() - invoke artifact"),
        ("NameLevel", "donamelevel() - name dungeon level"),
        ("NameItem", "docallcmd() - name item/type"),
    ];

    let mut present = Vec::new();
    let mut missing = Vec::new();

    for (name, desc) in &required_action_commands {
        if command_variant_exists_in_source(name) {
            present.push(*name);
        } else {
            missing.push((*name, *desc));
        }
    }

    println!("\n=== Missing Action Commands (Plan 7.1) ===");
    println!("Present in enum: {}/{}", present.len(), required_action_commands.len());
    for name in &present {
        println!("  OK: {}", name);
    }
    println!("Missing from enum: {}", missing.len());
    for (name, desc) in &missing {
        println!("  MISSING: {} - {}", name, desc);
    }
}

#[test]
fn test_missing_info_commands() {
    // Plan Step 7.2: Missing info commands
    let required_info_commands = [
        ("ShowAttributes", "doattributes() - show base stats"),
        ("ShowEquipment", "doprinuse() - show worn items"),
        ("ShowSpells", "dovspell() - spell list"),
        ("ShowConduct", "doconduct() - behavior tracking"),
        ("DungeonOverview", "dooverview() - dungeon map"),
        ("CountGold", "doprgold() - count gold"),
        ("ClassDiscovery", "doclassdisco() - discoveries by class"),
        ("TypeInventory", "dotypeinv() - inventory by type"),
        ("Organize", "doorganize() - sort inventory"),
        ("Vanquished", "dovanquished() - kill list"),
    ];

    let mut present = Vec::new();
    let mut missing = Vec::new();

    for (name, desc) in &required_info_commands {
        if command_variant_exists_in_source(name) {
            present.push(*name);
        } else {
            missing.push((*name, *desc));
        }
    }

    println!("\n=== Missing Info Commands (Plan 7.2) ===");
    println!("Present in enum: {}/{}", present.len(), required_info_commands.len());
    for name in &present {
        println!("  OK: {}", name);
    }
    println!("Missing from enum: {}", missing.len());
    for (name, desc) in &missing {
        println!("  MISSING: {} - {}", name, desc);
    }
}

#[test]
fn test_missing_wizard_commands() {
    // Plan Step 7.3: Missing wizard mode commands
    let required_wiz_commands = [
        ("WizGenesis", "create monsters"),
        ("WizIdentify", "identify all items"),
        ("WizIntrinsic", "set intrinsics"),
        ("WizLevelTele", "level teleport"),
        ("WizMap", "reveal map"),
        ("WizWish", "wish for item"),
        ("WizDetect", "detect monsters"),
    ];

    let mut present = Vec::new();
    let mut missing = Vec::new();

    for (name, desc) in &required_wiz_commands {
        if command_variant_exists_in_source(name) {
            present.push(*name);
        } else {
            missing.push((*name, *desc));
        }
    }

    println!("\n=== Missing Wizard Commands (Plan 7.3) ===");
    println!("Present in enum: {}/{}", present.len(), required_wiz_commands.len());
    for name in &present {
        println!("  OK: {}", name);
    }
    println!("Missing from enum: {}", missing.len());
    for (name, desc) in &missing {
        println!("  MISSING: {} - {}", name, desc);
    }
}

// ============================================================================
// 7.4: Implemented command behavior verification
// ============================================================================

#[test]
fn test_implemented_commands_dont_crash() {
    let mut gl = test_gameloop();

    let commands: Vec<(&str, Command)> = vec![
        ("Move E", Command::Move(Direction::East)),
        ("Move W", Command::Move(Direction::West)),
        ("Move N", Command::Move(Direction::North)),
        ("Move S", Command::Move(Direction::South)),
        ("Run E", Command::Run(Direction::East)),
        ("MoveUntil E", Command::MoveUntilInteresting(Direction::East)),
        ("Rest", Command::Rest),
        ("GoUp", Command::GoUp),
        ("GoDown", Command::GoDown),
        ("Search", Command::Search),
        ("Fight E", Command::Fight(Direction::East)),
        ("Fire E", Command::Fire(Direction::East)),
        ("Throw z E", Command::Throw('z', Direction::East)),
        ("Pickup", Command::Pickup),
        ("Drop a", Command::Drop('a')),
        ("Eat a", Command::Eat('a')),
        ("Quaff a", Command::Quaff('a')),
        ("Read a", Command::Read('a')),
        ("Zap a E", Command::Zap('a', Direction::East)),
        ("Apply a", Command::Apply('a')),
        ("Wear a", Command::Wear('a')),
        ("TakeOff a", Command::TakeOff('a')),
        ("PutOn a", Command::PutOn('a')),
        ("Remove a", Command::Remove('a')),
        ("Wield None", Command::Wield(None)),
        ("Open E", Command::Open(Direction::East)),
        ("Close E", Command::Close(Direction::East)),
        ("Kick E", Command::Kick(Direction::East)),
        ("Pray", Command::Pray),
        ("Engrave", Command::Engrave("Elbereth".to_string())),
        ("Inventory", Command::Inventory),
        ("Look", Command::Look),
        ("WhatsHere", Command::WhatsHere),
        ("Help", Command::Help),
        ("Discoveries", Command::Discoveries),
        ("History", Command::History),
    ];

    for (name, cmd) in commands {
        let result = exec(&mut gl, cmd);
        assert!(
            matches!(result, GameLoopResult::Continue),
            "Command {} should Continue, got {:?}",
            name, result
        );
    }
}

#[test]
fn test_meta_commands_terminate() {
    let mut gl1 = test_gameloop();
    let result = exec(&mut gl1, Command::Save);
    assert!(matches!(result, GameLoopResult::SaveAndQuit));

    let mut gl2 = test_gameloop();
    let result = exec(&mut gl2, Command::Quit);
    assert!(matches!(result, GameLoopResult::PlayerQuit));
}

// ============================================================================
// Summary
// ============================================================================

#[test]
fn test_command_gap_summary() {
    let action_commands = [
        "Loot", "Tip", "Rub", "Untrap", "Force", "Wipe", "Ride",
        "TwoWeapon", "SwapWeapon", "EnhanceSkill", "SelectQuiver",
        "TurnUndead", "MonsterAbility", "Jump", "Invoke", "NameLevel", "NameItem",
    ];
    let info_commands = [
        "ShowAttributes", "ShowEquipment", "ShowSpells", "ShowConduct",
        "DungeonOverview", "CountGold", "ClassDiscovery", "TypeInventory",
        "Organize", "Vanquished",
    ];
    let wiz_commands = [
        "WizGenesis", "WizIdentify", "WizIntrinsic", "WizLevelTele",
        "WizMap", "WizWish", "WizDetect",
    ];

    let mut total_missing = 0;
    let mut total_present = 0;

    for name in action_commands.iter().chain(info_commands.iter()).chain(wiz_commands.iter()) {
        if command_variant_exists_in_source(name) {
            total_present += 1;
        } else {
            total_missing += 1;
        }
    }

    let stub_count = [
        Command::Travel,
        Command::Offer,
        Command::Dip,
        Command::Pay,
        Command::Chat,
        Command::Sit,
        Command::Options,
        Command::ExtendedCommand("x".to_string()),
        Command::Redraw,
    ]
    .iter()
    .filter(|cmd| is_stub_command((*cmd).clone()))
    .count();

    let total_required = action_commands.len() + info_commands.len() + wiz_commands.len();

    println!("\n=== Command Gap Summary (Plan Step 7) ===");
    println!("C commands missing from enum: {}/{}", total_missing, total_required);
    println!("C commands present in enum:   {}/{}", total_present, total_required);
    println!("Existing commands still stubbed: {}/9", stub_count);
    println!();
    println!("To reach Plan Step 7 completion:");
    println!("  1. Add {} command variants to Command enum", total_missing);
    println!("  2. Implement {} stub commands", stub_count);
    println!(
        "  3. Total implementation work: {} new commands",
        total_missing + stub_count
    );
}
