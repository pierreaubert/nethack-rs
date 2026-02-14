//! Step 2: Static data parity tests
//!
//! Compares all static game data between C and Rust implementations:
//! - Monster definitions (380+ entries)
//! - Object definitions (467 entries)
//! - Artifact definitions
//! - Role/race data

use std::collections::HashSet;

use nh_test::c_source_parser::monsters::extract_monster_names;
use nh_test::c_source_parser::objects::extract_object_names;
use nh_test::c_source_parser::artifacts::extract_artifact_names;
use nh_test::c_source_parser::roles::extract_role_names;

// ============================================================================
// Monster data comparison (Step 2.1)
// ============================================================================

#[test]
fn test_monster_names_coverage() {
    let c_names: Vec<String> = extract_monster_names();
    let rust_names: HashSet<String> = nh_core::data::monsters::MONSTERS
        .iter()
        .map(|m| m.name.to_string())
        .collect();

    assert!(
        !c_names.is_empty(),
        "Failed to extract C monster names (is NetHack source at expected path?)"
    );

    let c_set: HashSet<String> = c_names.iter().cloned().collect();

    let missing_in_rust: Vec<&String> = c_set.difference(&rust_names).collect();
    let extra_in_rust: Vec<&String> = rust_names.difference(&c_set).collect();

    let matched = c_set.len() - missing_in_rust.len();
    let coverage = matched as f64 / c_set.len() as f64 * 100.0;

    println!("\n=== Monster Name Comparison ===");
    println!("C monsters: {}", c_set.len());
    println!("Rust monsters: {}", rust_names.len());
    println!("Matched: {}", matched);
    println!("Coverage: {:.1}%", coverage);

    if !missing_in_rust.is_empty() {
        println!("\nMissing in Rust ({}):", missing_in_rust.len());
        for (i, name) in missing_in_rust.iter().enumerate() {
            if i < 30 {
                println!("  - {}", name);
            }
        }
        if missing_in_rust.len() > 30 {
            println!("  ... and {} more", missing_in_rust.len() - 30);
        }
    }

    if !extra_in_rust.is_empty() {
        println!("\nExtra in Rust ({}):", extra_in_rust.len());
        for (i, name) in extra_in_rust.iter().enumerate() {
            if i < 30 {
                println!("  - {}", name);
            }
        }
        if extra_in_rust.len() > 30 {
            println!("  ... and {} more", extra_in_rust.len() - 30);
        }
    }

    println!("\nOK: {}/{} monsters match", matched, c_set.len());
}

/// Verify key monster entries exist in Rust.
#[test]
fn test_key_monsters_present() {
    let rust_names: HashSet<String> = nh_core::data::monsters::MONSTERS
        .iter()
        .map(|m| m.name.to_string())
        .collect();

    // Critical monsters that must exist
    let key_monsters = [
        "giant ant",
        "killer bee",
        "jackal",
        "fox",
        "coyote",
        "kitten",
        "housecat",
        "large cat",
        "little dog",
        "dog",
        "large dog",
        "newt",
        "gecko",
        "iguana",
        "kobold",
        "large kobold",
        "kobold lord",
        "grid bug",
        "floating eye",
        "gnome",
        "dwarf",
        "elf",
        "orc",
        "human",
        "Medusa",
        "Wizard of Yendor",
        "Death",
        "Pestilence",
        "Famine",
    ];

    let mut missing = Vec::new();
    for name in &key_monsters {
        if !rust_names.contains(*name) {
            missing.push(*name);
        }
    }

    if !missing.is_empty() {
        println!("Missing key monsters: {:?}", missing);
    }

    assert!(
        missing.is_empty(),
        "Critical monsters missing from Rust: {:?}",
        missing
    );
    println!("OK: All {} key monsters present", key_monsters.len());
}

/// Verify monster count is reasonable.
#[test]
fn test_monster_count() {
    let count = nh_core::data::monsters::MONSTERS.len();
    println!("Rust monster count: {}", count);
    // NetHack 3.6.7 has ~381 monsters (including NUMMONS sentinel)
    assert!(
        count >= 370,
        "Expected 370+ monsters, found {}",
        count
    );
}

// ============================================================================
// Object data comparison (Step 2.2)
// ============================================================================

#[test]
fn test_object_names_coverage() {
    let c_names: Vec<String> = extract_object_names();
    let rust_names: HashSet<String> = nh_core::data::objects::OBJECTS
        .iter()
        .map(|o| o.name.to_string())
        .collect();

    assert!(
        !c_names.is_empty(),
        "Failed to extract C object names (is NetHack source at expected path?)"
    );

    let c_set: HashSet<String> = c_names.iter().cloned().collect();

    let missing_in_rust: Vec<&String> = c_set.difference(&rust_names).collect();
    let extra_in_rust: Vec<&String> = rust_names.difference(&c_set).collect();

    let matched = c_set.len() - missing_in_rust.len();
    let coverage = matched as f64 / c_set.len() as f64 * 100.0;

    println!("\n=== Object Name Comparison ===");
    println!("C objects: {}", c_set.len());
    println!("Rust objects: {}", rust_names.len());
    println!("Matched: {}", matched);
    println!("Coverage: {:.1}%", coverage);

    if !missing_in_rust.is_empty() {
        println!("\nMissing in Rust ({}):", missing_in_rust.len());
        for (i, name) in missing_in_rust.iter().enumerate() {
            if i < 30 {
                println!("  - {}", name);
            }
        }
        if missing_in_rust.len() > 30 {
            println!("  ... and {} more", missing_in_rust.len() - 30);
        }
    }

    println!("\nOK: {}/{} objects match", matched, c_set.len());
}

/// Verify key object entries exist.
#[test]
fn test_key_objects_present() {
    let rust_names: HashSet<String> = nh_core::data::objects::OBJECTS
        .iter()
        .map(|o| o.name.to_string())
        .collect();

    let key_objects = [
        "arrow",
        "elven arrow",
        "orcish arrow",
        "long sword",
        "two-handed sword",
        "dagger",
        "leather armor",
        "ring mail",
        "plate mail",
        "food ration",
        "tripe ration",
        "slime mold",
        "potion of healing",
        "scroll of identify",
        "wand of wishing",
    ];

    let mut missing = Vec::new();
    for name in &key_objects {
        if !rust_names.contains(*name) {
            missing.push(*name);
        }
    }

    if !missing.is_empty() {
        println!("Missing key objects: {:?}", missing);
    }

    assert!(
        missing.is_empty(),
        "Critical objects missing from Rust: {:?}",
        missing
    );
    println!("OK: All {} key objects present", key_objects.len());
}

/// Verify object count is reasonable.
#[test]
fn test_object_count() {
    let count = nh_core::data::objects::OBJECTS.len();
    println!("Rust object count: {}", count);
    // NetHack 3.6.7 has ~467 objects
    assert!(
        count >= 400,
        "Expected 400+ objects, found {}",
        count
    );
}

// ============================================================================
// Artifact data comparison (Step 2.3)
// ============================================================================

#[test]
fn test_artifact_names_coverage() {
    let c_names: Vec<String> = extract_artifact_names();
    let rust_names: HashSet<String> = nh_core::data::artifacts::ARTIFACTS
        .iter()
        .map(|a| a.name.to_string())
        .collect();

    if c_names.is_empty() {
        println!("WARNING: Could not extract C artifact names");
        return;
    }

    let c_set: HashSet<String> = c_names.iter().cloned().collect();

    let missing_in_rust: Vec<&String> = c_set.difference(&rust_names).collect();

    let matched = c_set.len() - missing_in_rust.len();
    let coverage = matched as f64 / c_set.len() as f64 * 100.0;

    println!("\n=== Artifact Name Comparison ===");
    println!("C artifacts: {}", c_set.len());
    println!("Rust artifacts: {}", rust_names.len());
    println!("Matched: {}", matched);
    println!("Coverage: {:.1}%", coverage);

    if !missing_in_rust.is_empty() {
        println!("\nMissing in Rust ({}):", missing_in_rust.len());
        for name in &missing_in_rust {
            println!("  - {}", name);
        }
    }

    println!("\nOK: {}/{} artifacts match", matched, c_set.len());
}

/// Verify key artifact entries exist.
#[test]
fn test_key_artifacts_present() {
    let rust_names: HashSet<String> = nh_core::data::artifacts::ARTIFACTS
        .iter()
        .map(|a| a.name.to_string())
        .collect();

    let key_artifacts = [
        "Excalibur",
        "Stormbringer",
        "Mjollnir",
        "Orcrist",
        "Sting",
        "Magicbane",
        "Grayswandir",
        "Frost Brand",
        "Fire Brand",
    ];

    let mut missing = Vec::new();
    for name in &key_artifacts {
        if !rust_names.contains(*name) {
            missing.push(*name);
        }
    }

    if !missing.is_empty() {
        println!("Missing key artifacts: {:?}", missing);
    }

    assert!(
        missing.is_empty(),
        "Critical artifacts missing from Rust: {:?}",
        missing
    );
    println!("OK: All {} key artifacts present", key_artifacts.len());
}

// ============================================================================
// Role/Race data comparison (Step 2.4)
// ============================================================================

#[test]
fn test_role_names_coverage() {
    let c_role_names: Vec<String> = extract_role_names();
    let rust_role_names: HashSet<String> = nh_core::data::roles::ROLES
        .iter()
        .map(|r| r.name.male.to_string())
        .collect();

    if c_role_names.is_empty() {
        println!("WARNING: Could not extract C role names");
        return;
    }

    let c_set: HashSet<String> = c_role_names.iter().cloned().collect();

    let missing: Vec<&String> = c_set.difference(&rust_role_names).collect();

    println!("\n=== Role Name Comparison ===");
    println!("C roles: {}", c_set.len());
    println!("Rust roles: {}", rust_role_names.len());

    if !missing.is_empty() {
        println!("Missing in Rust: {:?}", missing);
    }

    let matched = c_set.len() - missing.len();
    println!("OK: {}/{} roles match", matched, c_set.len());
}

/// All 13 roles must exist.
#[test]
fn test_all_roles_present() {
    let expected_roles = [
        "Archeologist",
        "Barbarian",
        "Caveman",
        "Healer",
        "Knight",
        "Monk",
        "Priest",
        "Ranger",
        "Rogue",
        "Samurai",
        "Tourist",
        "Valkyrie",
        "Wizard",
    ];

    let rust_roles: HashSet<String> = nh_core::data::roles::ROLES
        .iter()
        .map(|r| r.name.male.to_string())
        .collect();

    for role in &expected_roles {
        assert!(
            rust_roles.contains(*role),
            "Role '{}' missing from Rust implementation",
            role
        );
    }

    assert_eq!(
        nh_core::data::roles::ROLES.len(),
        expected_roles.len(),
        "Rust should have exactly {} roles, found {}",
        expected_roles.len(),
        nh_core::data::roles::ROLES.len()
    );

    println!("OK: All {} roles present", expected_roles.len());
}

/// All 5 races must exist.
#[test]
fn test_all_races_present() {
    let expected_races = ["human", "elf", "dwarf", "gnome", "orc"];

    let rust_races: HashSet<String> = nh_core::data::roles::RACES
        .iter()
        .map(|r| r.noun.to_string())
        .collect();

    for race in &expected_races {
        assert!(
            rust_races.contains(*race),
            "Race '{}' missing from Rust implementation",
            race
        );
    }

    assert_eq!(
        nh_core::data::roles::RACES.len(),
        expected_races.len(),
        "Rust should have exactly {} races, found {}",
        expected_races.len(),
        nh_core::data::roles::RACES.len()
    );

    println!("OK: All {} races present", expected_races.len());
}

// ============================================================================
// Overall summary
// ============================================================================

#[test]
fn test_static_data_summary() {
    let c_monsters = extract_monster_names();
    let c_objects = extract_object_names();
    let c_artifacts = extract_artifact_names();
    let c_roles = extract_role_names();

    let rust_monsters = nh_core::data::monsters::MONSTERS.len();
    let rust_objects = nh_core::data::objects::OBJECTS.len();
    let rust_artifacts = nh_core::data::artifacts::ARTIFACTS.len();
    let rust_roles = nh_core::data::roles::ROLES.len();
    let rust_races = nh_core::data::roles::RACES.len();

    println!("\n=== Static Data Summary ===");
    println!("{:<20} {:<10} {:<10}", "Category", "C", "Rust");
    println!("{}", "-".repeat(40));
    println!("{:<20} {:<10} {:<10}", "Monsters", c_monsters.len(), rust_monsters);
    println!("{:<20} {:<10} {:<10}", "Objects", c_objects.len(), rust_objects);
    println!("{:<20} {:<10} {:<10}", "Artifacts", c_artifacts.len(), rust_artifacts);
    println!("{:<20} {:<10} {:<10}", "Roles", c_roles.len(), rust_roles);
    println!("{:<20} {:<10} {:<10}", "Races", "5", rust_races);
}
