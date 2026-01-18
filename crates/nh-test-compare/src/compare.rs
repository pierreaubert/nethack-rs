//! State comparison engine for detecting differences between implementations.
//!
//! This module provides functionality to compare unified game states from
//! the Rust and C implementations and detect behavioral differences.

use serde_json::json;
use crate::state::common::*;

/// Compare two game states and report differences
pub fn compare_states(rust_state: &UnifiedGameState, c_state: &UnifiedGameState) -> Vec<StateDifference> {
    let mut differences = Vec::new();

    // Compare player position
    if rust_state.position != c_state.position {
        differences.push(StateDifference {
            category: DifferenceCategory::Position,
            field: "position".to_string(),
            rust_value: json!(rust_state.position),
            c_value: json!(c_state.position),
            severity: if rust_state.position != c_state.position {
                // Position difference could be critical if it affects gameplay
                if (rust_state.position.0 - c_state.position.0).abs() > 1 ||
                   (rust_state.position.1 - c_state.position.1).abs() > 1 {
                    DifferenceSeverity::Critical
                } else {
                    DifferenceSeverity::Major
                }
            } else {
                DifferenceSeverity::Minor
            },
            description: format!(
                "Position mismatch: Rust {:?} vs C {:?}",
                rust_state.position, c_state.position
            ),
        });
    }

    // Compare HP
    if rust_state.hp != c_state.hp {
        let hp_diff = (rust_state.hp - c_state.hp).abs();
        differences.push(StateDifference {
            category: DifferenceCategory::Health,
            field: "hp".to_string(),
            rust_value: json!(rust_state.hp),
            c_value: json!(c_state.hp),
            severity: if hp_diff > 5 {
                DifferenceSeverity::Critical
            } else if hp_diff > 2 {
                DifferenceSeverity::Major
            } else {
                DifferenceSeverity::Minor
            },
            description: format!(
                "HP mismatch: Rust {} vs C {} (diff: {})",
                rust_state.hp, c_state.hp, hp_diff
            ),
        });
    }

    // Compare max HP
    if rust_state.max_hp != c_state.max_hp {
        differences.push(StateDifference {
            category: DifferenceCategory::Health,
            field: "max_hp".to_string(),
            rust_value: json!(rust_state.max_hp),
            c_value: json!(c_state.max_hp),
            severity: DifferenceSeverity::Minor,
            description: format!(
                "Max HP mismatch: Rust {} vs C {}",
                rust_state.max_hp, c_state.max_hp
            ),
        });
    }

    // Compare energy
    if rust_state.energy != c_state.energy {
        differences.push(StateDifference {
            category: DifferenceCategory::Energy,
            field: "energy".to_string(),
            rust_value: json!(rust_state.energy),
            c_value: json!(c_state.energy),
            severity: DifferenceSeverity::Minor,
            description: format!(
                "Energy mismatch: Rust {} vs C {}",
                rust_state.energy, c_state.energy
            ),
        });
    }

    // Compare armor class
    if rust_state.armor_class != c_state.armor_class {
        differences.push(StateDifference {
            category: DifferenceCategory::Stats,
            field: "armor_class".to_string(),
            rust_value: json!(rust_state.armor_class),
            c_value: json!(c_state.armor_class),
            severity: DifferenceSeverity::Major,
            description: format!(
                "Armor class mismatch: Rust {} vs C {}",
                rust_state.armor_class, c_state.armor_class
            ),
        });
    }

    // Compare gold
    if rust_state.gold != c_state.gold {
        differences.push(StateDifference {
            category: DifferenceCategory::Stats,
            field: "gold".to_string(),
            rust_value: json!(rust_state.gold),
            c_value: json!(c_state.gold),
            severity: DifferenceSeverity::Minor,
            description: format!(
                "Gold mismatch: Rust {} vs C {}",
                rust_state.gold, c_state.gold
            ),
        });
    }

    // Compare experience level
    if rust_state.experience_level != c_state.experience_level {
        differences.push(StateDifference {
            category: DifferenceCategory::Stats,
            field: "experience_level".to_string(),
            rust_value: json!(rust_state.experience_level),
            c_value: json!(c_state.experience_level),
            severity: DifferenceSeverity::Major,
            description: format!(
                "Experience level mismatch: Rust {} vs C {}",
                rust_state.experience_level, c_state.experience_level
            ),
        });
    }

    // Compare dungeon depth
    if rust_state.dungeon_depth != c_state.dungeon_depth {
        differences.push(StateDifference {
            category: DifferenceCategory::Dungeon,
            field: "dungeon_depth".to_string(),
            rust_value: json!(rust_state.dungeon_depth),
            c_value: json!(c_state.dungeon_depth),
            severity: DifferenceSeverity::Critical,
            description: format!(
                "Dungeon depth mismatch: Rust level {} vs C level {}",
                rust_state.dungeon_depth, c_state.dungeon_depth
            ),
        });
    }

    // Compare attributes
    compare_attributes(rust_state, c_state, &mut differences);

    // Compare inventory
    compare_inventory(rust_state, c_state, &mut differences);

    // Compare monsters
    compare_monsters(rust_state, c_state, &mut differences);

    // Compare turn count (should match if same actions taken)
    if rust_state.turn != c_state.turn {
        differences.push(StateDifference {
            category: DifferenceCategory::Timing,
            field: "turn".to_string(),
            rust_value: json!(rust_state.turn),
            c_value: json!(c_state.turn),
            severity: DifferenceSeverity::Critical,
            description: format!(
                "Turn count mismatch: Rust turn {} vs C turn {}",
                rust_state.turn, c_state.turn
            ),
        });
    }

    // Compare death status
    if rust_state.is_dead != c_state.is_dead {
        differences.push(StateDifference {
            category: DifferenceCategory::Other,
            field: "is_dead".to_string(),
            rust_value: json!(rust_state.is_dead),
            c_value: json!(c_state.is_dead),
            severity: DifferenceSeverity::Critical,
            description: format!(
                "Death status mismatch: Rust {} vs C {}",
                rust_state.is_dead, c_state.is_dead
            ),
        });
    }

    differences
}

/// Compare player attributes
fn compare_attributes(
    rust_state: &UnifiedGameState,
    c_state: &UnifiedGameState,
    differences: &mut Vec<StateDifference>,
) {
    let attributes = [
        ("strength", rust_state.strength, c_state.strength),
        ("dexterity", rust_state.dexterity, c_state.dexterity),
        ("constitution", rust_state.constitution, c_state.constitution),
        ("intelligence", rust_state.intelligence, c_state.intelligence),
        ("wisdom", rust_state.wisdom, c_state.wisdom),
        ("charisma", rust_state.charisma, c_state.charisma),
    ];

    for (name, rust_val, c_val) in attributes {
        if rust_val != c_val {
            differences.push(StateDifference {
                category: DifferenceCategory::Attributes,
                field: name.to_string(),
                rust_value: json!(rust_val),
                c_value: json!(c_val),
                severity: DifferenceSeverity::Minor,
                description: format!(
                    "{} mismatch: Rust {} vs C {}",
                    name, rust_val, c_val
                ),
            });
        }
    }
}

/// Compare inventories
fn compare_inventory(
    rust_state: &UnifiedGameState,
    c_state: &UnifiedGameState,
    differences: &mut Vec<StateDifference>,
) {
    if rust_state.inventory.len() != c_state.inventory.len() {
        differences.push(StateDifference {
            category: DifferenceCategory::Inventory,
            field: "inventory_count".to_string(),
            rust_value: json!(rust_state.inventory.len()),
            c_value: json!(c_state.inventory.len()),
            severity: DifferenceSeverity::Major,
            description: format!(
                "Inventory size mismatch: Rust {} items vs C {} items",
                rust_state.inventory.len(), c_state.inventory.len()
            ),
        });
    }
}

/// Compare nearby monsters
fn compare_monsters(
    rust_state: &UnifiedGameState,
    c_state: &UnifiedGameState,
    differences: &mut Vec<StateDifference>,
) {
    if rust_state.nearby_monsters.len() != c_state.nearby_monsters.len() {
        differences.push(StateDifference {
            category: DifferenceCategory::Monsters,
            field: "monster_count".to_string(),
            rust_value: json!(rust_state.nearby_monsters.len()),
            c_value: json!(c_state.nearby_monsters.len()),
            severity: DifferenceSeverity::Major,
            description: format!(
                "Monster count mismatch: Rust {} vs C {}",
                rust_state.nearby_monsters.len(), c_state.nearby_monsters.len()
            ),
        });
    }
}

/// Filter differences by severity
pub fn filter_differences(
    differences: &[StateDifference],
    min_severity: DifferenceSeverity,
) -> Vec<StateDifference> {
    let severity_order = [
        DifferenceSeverity::Critical,
        DifferenceSeverity::Major,
        DifferenceSeverity::Minor,
        DifferenceSeverity::Info,
    ];

    let min_index = severity_order.iter()
        .position(|s| *s == min_severity)
        .unwrap_or(0);

    differences.iter()
        .filter(|d| {
            let d_index = severity_order.iter()
                .position(|s| s == &d.severity)
                .unwrap_or(3);
            d_index <= min_index
        })
        .cloned()
        .collect()
}

/// Summarize differences
pub fn summarize_differences(differences: &[StateDifference]) -> String {
    let critical = differences.iter()
        .filter(|d| d.severity == DifferenceSeverity::Critical)
        .count();
    let major = differences.iter()
        .filter(|d| d.severity == DifferenceSeverity::Major)
        .count();
    let minor = differences.iter()
        .filter(|d| d.severity == DifferenceSeverity::Minor)
        .count();
    let info = differences.iter()
        .filter(|d| d.severity == DifferenceSeverity::Info)
        .count();

    format!(
        "Differences found: {} critical, {} major, {} minor, {} info",
        critical, major, minor, info
    )
}

/// Check if states are functionally equivalent
pub fn states_are_equivalent(rust_state: &UnifiedGameState, c_state: &UnifiedGameState) -> bool {
    let diffs = compare_states(rust_state, c_state);
    
    // Filter out minor differences and info
    let significant = filter_differences(&diffs, DifferenceSeverity::Major);
    
    significant.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_states() {
        let state = UnifiedGameState::default_start("Tourist", "Human");
        let diffs = compare_states(&state, &state);
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_position_difference() {
        let mut rust = UnifiedGameState::default_start("Wizard", "Elf");
        let mut c = UnifiedGameState::default_start("Wizard", "Elf");
        
        rust.position = (40, 10);
        c.position = (50, 10);
        
        let diffs = compare_states(&rust, &c);
        assert!(!diffs.is_empty());
        assert!(diffs.iter().any(|d| d.category == DifferenceCategory::Position));
    }

    #[test]
    fn test_hp_difference_critical() {
        let mut rust = UnifiedGameState::default_start("Rogue", "Gnome");
        let mut c = UnifiedGameState::default_start("Rogue", "Gnome");
        
        rust.hp = 10;
        c.hp = 2;
        
        let diffs = compare_states(&rust, &c);
        let hp_diff = diffs.iter()
            .find(|d| d.field == "hp")
            .expect("Should have HP difference");
        assert_eq!(hp_diff.severity, DifferenceSeverity::Critical);
    }

    #[test]
    fn test_filter_differences() {
        let _state = UnifiedGameState::default_start("Priest", "Dwarf");
        
        let diffs = vec![
            StateDifference {
                category: DifferenceCategory::Position,
                field: "pos".to_string(),
                rust_value: json!(1),
                c_value: json!(2),
                severity: DifferenceSeverity::Critical,
                description: "test".to_string(),
            },
            StateDifference {
                category: DifferenceCategory::Stats,
                field: "gold".to_string(),
                rust_value: json!(10),
                c_value: json!(15),
                severity: DifferenceSeverity::Minor,
                description: "test".to_string(),
            },
        ];
        
        let critical_only = filter_differences(&diffs, DifferenceSeverity::Critical);
        assert_eq!(critical_only.len(), 1);
        
        let major_or_worse = filter_differences(&diffs, DifferenceSeverity::Major);
        assert_eq!(major_or_worse.len(), 1);
    }

    #[test]
    fn test_summarize_differences() {
        let diffs = vec![
            StateDifference {
                category: DifferenceCategory::Position,
                field: "pos".to_string(),
                rust_value: json!(1),
                c_value: json!(2),
                severity: DifferenceSeverity::Critical,
                description: "test".to_string(),
            },
            StateDifference {
                category: DifferenceCategory::Stats,
                field: "gold".to_string(),
                rust_value: json!(10),
                c_value: json!(15),
                severity: DifferenceSeverity::Minor,
                description: "test".to_string(),
            },
            StateDifference {
                category: DifferenceCategory::Health,
                field: "hp".to_string(),
                rust_value: json!(10),
                c_value: json!(10),
                severity: DifferenceSeverity::Info,
                description: "test".to_string(),
            },
        ];
        
        let summary = summarize_differences(&diffs);
        assert!(summary.contains("1 critical"));
        assert!(summary.contains("0 major"));
        assert!(summary.contains("1 minor"));
        assert!(summary.contains("1 info"));
    }

    #[test]
    fn test_states_equivalent() {
        let state = UnifiedGameState::default_start("Samurai", "Human");
        assert!(states_are_equivalent(&state, &state));
        
        let mut different = UnifiedGameState::default_start("Samurai", "Human");
        different.hp = 5;
        different.armor_class = 5;
        
        assert!(!states_are_equivalent(&state, &different));
    }
}
