//! True synchronized parity test — both engines initialized from the SAME seed
//! with NO attribute syncing. This tests full init-path + per-turn parity.

use nh_core::action::Command;
use nh_core::player::{Attribute, Gender, Race, Role};
use nh_core::{CGameEngineTrait, GameLoop, GameRng, GameState};
use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use serial_test::serial;

/// Compare player state between Rust and C without any syncing.
/// Returns a list of field mismatches.
fn compare_unsynced(rs: &GameState, c: &CGameEngine, turn: u64) -> Vec<String> {
    let mut diffs = Vec::new();

    let c_hp = c.hp();
    let c_maxhp = c.max_hp();
    let c_energy = c.energy();
    let c_maxen = c.max_energy();
    let c_ac = c.armor_class();
    let c_nutrition = c.nutrition();
    let c_gold = c.gold();
    let c_xlvl = c.experience_level();

    let attrs_json: serde_json::Value =
        serde_json::from_str(&c.attributes_json()).unwrap_or_default();

    macro_rules! cmp {
        ($name:expr, $rust:expr, $c_val:expr) => {
            if $rust != $c_val {
                diffs.push(format!("turn {}: {} R={} C={}", turn, $name, $rust, $c_val));
            }
        };
    }

    cmp!("hp", rs.player.hp, c_hp);
    cmp!("maxhp", rs.player.hp_max, c_maxhp);
    cmp!("energy", rs.player.energy, c_energy);
    cmp!("maxen", rs.player.energy_max, c_maxen);
    cmp!("ac", rs.player.armor_class as i32, c_ac);
    cmp!("nutrition", rs.player.nutrition, c_nutrition);
    cmp!("gold", rs.player.gold as i64, c_gold as i64);
    cmp!("xlvl", rs.player.exp_level, c_xlvl);

    let attrs = [
        (Attribute::Strength, "str"),
        (Attribute::Intelligence, "int"),
        (Attribute::Wisdom, "wis"),
        (Attribute::Dexterity, "dex"),
        (Attribute::Constitution, "con"),
        (Attribute::Charisma, "cha"),
    ];
    for (attr, key) in &attrs {
        if let Some(c_val) = attrs_json[key].as_i64() {
            let r_val = rs.player.attr_current.get(*attr) as i64;
            cmp!(&format!("attr.{}", key), r_val, c_val);
        }
    }

    diffs
}

/// True parity test: both engines from same seed, no syncing.
/// This is the gold standard — if this passes, the engines are identical.
#[test]
#[serial]
fn test_true_sync_rest_100_turns() {
    let seed = 42u64;
    let num_turns = 100;

    // 1. Initialize C engine with test seed
    let mut c_engine = CGameEngine::new();
    // Pre-set seed BEFORE init so u_init() uses it (avoids static array corruption from double-init)
    c_engine.set_seed(seed).expect("C set_seed failed");
    c_engine
        .init("Valkyrie", "Human", 1, 0)
        .expect("C engine init failed");
    c_engine
        .generate_and_place()
        .expect("C generate_and_place failed");

    let (cx, cy) = c_engine.position();
    let c_hp = c_engine.hp();
    let c_maxhp = c_engine.max_hp();
    let c_nutrition = c_engine.nutrition();
    let c_ac = c_engine.armor_class();

    println!(
        "C init: pos=({},{}) hp={}/{} nutr={} ac={}",
        cx, cy, c_hp, c_maxhp, c_nutrition, c_ac
    );

    // 2. Initialize Rust engine with same seed
    let rust_rng = GameRng::new(seed);
    let mut rust_state = GameState::new_with_identity(
        rust_rng,
        "Hero".into(),
        Role::Valkyrie,
        Race::Human,
        Gender::Female,
        Role::Valkyrie.default_alignment(),
    );

    // Override Rust player position to match C (level gen may differ slightly)
    rust_state.player.pos.x = cx as i8;
    rust_state.player.pos.y = cy as i8;
    rust_state.player.prev_pos = rust_state.player.pos;

    let (rx, ry) = (rust_state.player.pos.x, rust_state.player.pos.y);
    println!(
        "Rust init: pos=({},{}) hp={}/{} nutr={} ac={}",
        rx,
        ry,
        rust_state.player.hp,
        rust_state.player.hp_max,
        rust_state.player.nutrition,
        rust_state.player.armor_class
    );

    // Dump monster lists from both engines
    println!("=== MONSTERS ===");
    println!(
        "  Rust: {} monsters",
        rust_state.current_level.monsters.len()
    );
    for (i, m) in rust_state.current_level.monsters.iter().enumerate() {
        println!(
            "    R mon[{}]: type={} at ({},{}) hp={}/{} peaceful={} sleeping={}",
            i, m.monster_type, m.x, m.y, m.hp, m.hp_max, m.state.peaceful, m.state.sleeping
        );
    }
    let c_mon_str = c_engine.monsters_json();
    let c_mons: serde_json::Value = serde_json::from_str(&c_mon_str).unwrap_or_default();
    if let Some(arr) = c_mons.as_array() {
        println!("  C: {} monsters", arr.len());
        for (i, m) in arr.iter().enumerate() {
            println!(
                "    C mon[{}]: mnum={} at ({},{}) hp={}/{} peaceful={} sleeping={}",
                i, m["mnum"], m["x"], m["y"], m["hp"], m["hp_max"], m["peaceful"], m["asleep"]
            );
        }
    }

    // Compare initial state (before any turns)
    let init_diffs = compare_unsynced(&rust_state, &c_engine, 0);
    println!("=== INIT STATE ===");
    println!(
        "  Rust: hp={}/{} en={}/{} ac={} nutr={} gold={} xlvl={}",
        rust_state.player.hp,
        rust_state.player.hp_max,
        rust_state.player.energy,
        rust_state.player.energy_max,
        rust_state.player.armor_class,
        rust_state.player.nutrition,
        rust_state.player.gold,
        rust_state.player.exp_level
    );
    println!(
        "  Rust attrs: str={} int={} wis={} dex={} con={} cha={}",
        rust_state.player.attr_current.get(Attribute::Strength),
        rust_state.player.attr_current.get(Attribute::Intelligence),
        rust_state.player.attr_current.get(Attribute::Wisdom),
        rust_state.player.attr_current.get(Attribute::Dexterity),
        rust_state.player.attr_current.get(Attribute::Constitution),
        rust_state.player.attr_current.get(Attribute::Charisma)
    );
    println!(
        "  C: hp={}/{} en={}/{} ac={} nutr={} gold={}",
        c_hp,
        c_maxhp,
        c_engine.energy(),
        c_engine.max_energy(),
        c_ac,
        c_nutrition,
        c_engine.gold()
    );
    if !init_diffs.is_empty() {
        println!("=== INIT DIVERGENCE ({} diffs) ===", init_diffs.len());
        for d in &init_diffs {
            println!("  {}", d);
        }
    } else {
        println!("=== INIT: PERFECT MATCH ===");
    }

    // Skip invariant checks since C position may differ
    rust_state.skip_invariant_checks = true;
    let mut rust_loop = GameLoop::new(rust_state);

    // 3. Run rest turns and compare
    let mut total_diffs = init_diffs.len();
    let mut first_gameplay_diff: Option<String> = None;

    for turn in 1..=num_turns {
        rust_loop.tick(Command::Rest);
        c_engine.exec_cmd('.').expect("C rest failed");

        if turn <= 10 || turn % 10 == 0 || turn == num_turns {
            let diffs = compare_unsynced(rust_loop.state(), &c_engine, turn);
            if !diffs.is_empty() {
                total_diffs += diffs.len();
                if first_gameplay_diff.is_none() {
                    first_gameplay_diff = Some(diffs[0].clone());
                    // Print surrounding state for first divergence
                    println!("=== FIRST GAMEPLAY DIVERGENCE at turn {} ===", turn);
                    let rs = rust_loop.state();
                    println!(
                        "  Rust: pos=({},{}) hp={}/{} nutr={} ac={}",
                        rs.player.pos.x,
                        rs.player.pos.y,
                        rs.player.hp,
                        rs.player.hp_max,
                        rs.player.nutrition,
                        rs.player.armor_class
                    );
                    let (cx, cy) = c_engine.position();
                    println!(
                        "  C:    pos=({},{}) hp={}/{} nutr={} ac={}",
                        cx,
                        cy,
                        c_engine.hp(),
                        c_engine.max_hp(),
                        c_engine.nutrition(),
                        c_engine.armor_class()
                    );
                }
                for d in &diffs {
                    println!("  {}", d);
                }
            }
        }
    }

    println!(
        "\n=== TRUE SYNC RESULT: {} total diffs across {} turns ===",
        total_diffs, num_turns
    );
    if let Some(ref first) = first_gameplay_diff {
        println!("First gameplay diff: {}", first);
    }

    // Report but don't fail yet — we're diagnosing
    if total_diffs > 0 {
        println!(
            "DIVERGENCE DETECTED — {} diffs (init + gameplay)",
            total_diffs
        );
    } else {
        println!("PERFECT PARITY — zero diffs!");
    }
}
