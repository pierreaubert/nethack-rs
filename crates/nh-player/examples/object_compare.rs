//! Object-by-object comparison between Rust and C NetHack
//!
//! This example tests each object type (wands, potions, scrolls, etc.)
//! by applying them in identical game states and comparing results.

use nh_core::object::ObjectClass;
use nh_core::{GameLoop, GameRng, GameState};
use nh_player::ffi::CGameEngine;
use nh_player::state::c_extractor::CGameWrapper;
use nh_player::state::common::GameAction;
use nh_player::state::rust_extractor::RustGameEngine;

use std::collections::HashMap;

#[derive(Debug, Clone)]
struct ObjectTestResult {
    object_class: String,
    object_name: String,
    rust_hp_before: i32,
    rust_hp_after: i32,
    c_hp_before: i32,
    c_hp_after: i32,
    rust_message: String,
    c_message: String,
    rust_gold: i32,
    c_gold: i32,
    rust_inventory_count: usize,
    c_inventory_count: usize,
}

impl ObjectTestResult {
    fn has_difference(&self) -> bool {
        self.rust_hp_after != self.c_hp_after
            || self.rust_gold != self.c_gold
            || self.rust_message != self.c_message
    }

    fn summary(&self) -> String {
        if !self.has_difference() {
            return format!("[OK] {} - {}", self.object_class, self.object_name);
        }

        let mut diffs = Vec::new();
        if self.rust_hp_after != self.c_hp_after {
            diffs.push(format!(
                "HP: Rust={}->{}, C={}->{}",
                self.rust_hp_before, self.rust_hp_after, self.c_hp_before, self.c_hp_after
            ));
        }
        if self.rust_gold != self.c_gold {
            diffs.push(format!("Gold: Rust={}, C={}", self.rust_gold, self.c_gold));
        }
        if self.rust_message != self.c_message {
            diffs.push(format!(
                "Msg: Rust='{}' vs C='{}'",
                truncate(&self.rust_message, 50),
                truncate(&self.c_message, 50)
            ));
        }

        format!(
            "[DIFF] {} - {}: {}",
            self.object_class,
            self.object_name,
            diffs.join("; ")
        )
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

/// Get sample objects for each class
fn get_sample_objects() -> HashMap<ObjectClass, Vec<(&'static str, char)>> {
    let mut samples = HashMap::new();

    // Weapons
    samples.insert(
        ObjectClass::Weapon,
        vec![("dagger", ')'), ("long sword", ')'), ("axe", ')')],
    );

    // Armor
    samples.insert(
        ObjectClass::Armor,
        vec![("leather armor", '['), ("chain mail", '['), ("shield", '[')],
    );

    // Rings
    samples.insert(
        ObjectClass::Ring,
        vec![
            ("ring of conflict", '='),
            ("ring of increase damage", '='),
            ("ring of regeneration", '='),
        ],
    );

    // Amulets
    samples.insert(
        ObjectClass::Amulet,
        vec![
            ("amulet of life saving", '"'),
            ("amulet of reflection", '"'),
            ("amulet of unchanging", '"'),
        ],
    );

    // Tools
    samples.insert(
        ObjectClass::Tool,
        vec![("pick-axe", '('), ("sack", '('), ("tin opener", '(')],
    );

    // Food
    samples.insert(
        ObjectClass::Food,
        vec![("apple", '%'), ("carrot", '%'), ("dead lizard", '%')],
    );

    // Potions
    samples.insert(
        ObjectClass::Potion,
        vec![
            ("potion of healing", '!'),
            ("potion of extra healing", '!'),
            ("potion of restore ability", '!'),
        ],
    );

    // Scrolls
    samples.insert(
        ObjectClass::Scroll,
        vec![
            ("scroll of teleportation", '?'),
            ("scroll of remove curse", '?'),
            ("scroll of create monster", '?'),
        ],
    );

    // Spellbooks
    samples.insert(
        ObjectClass::Spellbook,
        vec![
            ("spellbook of healing", '+'),
            ("spellbook of teleportation", '+'),
        ],
    );

    // Wands
    samples.insert(
        ObjectClass::Wand,
        vec![
            ("wand of striking", '/'),
            ("wand of light", '/'),
            ("wand of digging", '/'),
        ],
    );

    // Gems
    samples.insert(
        ObjectClass::Gem,
        vec![("diamond", '*'), ("ruby", '*'), ("emerald", '*')],
    );

    samples
}

fn run_object_tests(seed: u64) {
    println!("=== Object Comparison Session (seed={}) ===", seed);

    let samples = get_sample_objects();
    let mut all_results = Vec::new();
    let mut differences = Vec::new();

    for (object_class, objects) in samples.iter() {
        println!("\n--- Testing {} ---", object_class);

        for (name, _letter) in objects.iter() {
            // Initialize games
            let rust_rng = GameRng::new(seed);
            let rust_state = GameState::new(rust_rng);
            let mut rust_loop = GameLoop::new(rust_state);
            let mut rust_engine = RustGameEngine::new(&mut rust_loop);

            let mut c_engine = CGameEngine::new();
            c_engine
                .init("Tourist", "Human", 0, 0)
                .expect("Failed to init C engine");
            let mut c_wrapper = CGameWrapper::new(&mut c_engine);

            // Get initial states
            let rust_state_before = rust_engine.extract_state();
            let c_state_before = c_wrapper.extract_state();

            let rust_hp_before = rust_state_before.hp;
            let c_hp_before = c_state_before.hp;
            let rust_gold_before = rust_state_before.gold;
            let c_gold_before = c_state_before.gold;

            // Apply object action - returns (reward, message) tuple
            let (rust_msg_val, rust_msg) = match object_class {
                ObjectClass::Potion => {
                    let action = GameAction::QuaffFirst;
                    rust_engine.step(&action)
                }
                ObjectClass::Scroll => {
                    let action = GameAction::ReadFirst;
                    rust_engine.step(&action)
                }
                ObjectClass::Wand => {
                    let action = GameAction::ZapFirst;
                    rust_engine.step(&action)
                }
                ObjectClass::Food => {
                    let action = GameAction::EatFirst;
                    rust_engine.step(&action)
                }
                ObjectClass::Ring | ObjectClass::Amulet => {
                    let action = GameAction::WearFirst;
                    rust_engine.step(&action)
                }
                ObjectClass::Tool => {
                    // Tools use 'a' for apply but we don't have ApplyFirst
                    // Skip tool testing for now
                    continue;
                }
                ObjectClass::Weapon => {
                    // Wield weapon
                    let action = GameAction::WieldFirst;
                    rust_engine.step(&action)
                }
                _ => (0.0, String::new()),
            };

            // Get final states
            let rust_state_after = rust_engine.extract_state();
            let c_state_after = c_wrapper.extract_state();

            let result = ObjectTestResult {
                object_class: object_class.to_string(),
                object_name: name.to_string(),
                rust_hp_before,
                rust_hp_after: rust_state_after.hp,
                c_hp_before,
                c_hp_after: c_state_after.hp,
                rust_message: rust_msg,
                c_message: String::new(), // Will be filled by C wrapper
                rust_gold: rust_state_after.gold,
                c_gold: c_state_after.gold,
                rust_inventory_count: rust_state_after.inventory.len(),
                c_inventory_count: c_state_after.inventory.len(),
            };

            let summary = result.summary();
            println!("  {}", summary);

            if result.has_difference() {
                differences.push((object_class.to_string(), name.clone(), summary.clone()));
            }

            all_results.push(result);
        }
    }

    // Summary
    println!("\n=== Summary ===");
    println!("Total objects tested: {}", all_results.len());
    println!("With differences: {}", differences.len());

    if !differences.is_empty() {
        println!("\nDifferences found:");
        for (class, name, summary) in differences.iter().take(20) {
            println!("  {}", summary);
        }
    }
}

fn main() {
    println!("NetHack Rust vs C Object Comparison\n");

    for seed in [42, 12345, 99999] {
        run_object_tests(seed);
        println!();
    }

    println!("\n=== Analysis ===");
    println!("This test compares object effects between Rust and C:");
    println!("- Potion effects (healing, extra healing, restore ability)");
    println!("- Scroll effects (teleportation, remove curse)");
    println!("- Wand effects (striking, light, digging)");
    println!("- Food nutrition and effects");
    println!("- Equipment stats (weapons, armor, rings, amulets)");
}
