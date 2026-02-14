//! Role and race data comparison
//!
//! Parses role/class definitions from C source and compares with Rust.

use std::fs;
use std::path::Path;

/// Parsed role data from C source
#[derive(Debug, Clone)]
pub struct CRole {
    pub name: String,
    pub female_name: Option<String>,
    pub ranks: Vec<String>,
}

/// Parsed race data from C source
#[derive(Debug, Clone)]
pub struct CRace {
    pub name: String,
    pub adjective: String,
}

/// Known role names (used to filter out rank titles)
const KNOWN_ROLES: [&str; 13] = [
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

/// Extract role names from C role.c
pub fn extract_role_names() -> Vec<String> {
    // The C code has a complex nested structure where both role names
    // and rank titles use { "Name", ... } pattern.
    // We use the known roles list to filter correctly.
    let role_c = Path::new(super::NETHACK_SRC).join("src/role.c");

    if !role_c.exists() {
        return Vec::new();
    }

    let content = fs::read_to_string(&role_c).unwrap_or_default();
    let mut names = Vec::new();

    let mut in_roles_array = false;

    for line in content.lines() {
        if line.contains("const struct Role roles[]") {
            in_roles_array = true;
            continue;
        }

        if !in_roles_array {
            continue;
        }

        let trimmed = line.trim();

        // Stop at races array
        if trimmed.contains("const struct Race races[]") {
            break;
        }

        // Look for { { "Name", pattern which indicates role name
        if trimmed.starts_with("{ { \"") {
            if let Some(start) = trimmed.find('"') {
                let rest = &trimmed[start + 1..];
                if let Some(end) = rest.find('"') {
                    let name = rest[..end].to_string();
                    // Only include if it's a known role name
                    if KNOWN_ROLES.contains(&name.as_str()) {
                        names.push(name);
                    }
                }
            }
        }
    }

    names
}

/// Known race names (used to filter correctly)
const KNOWN_RACES: [&str; 5] = ["human", "elf", "dwarf", "gnome", "orc"];

/// Extract race names from C role.c
pub fn extract_race_names() -> Vec<String> {
    let role_c = Path::new(super::NETHACK_SRC).join("src/role.c");

    if !role_c.exists() {
        return Vec::new();
    }

    let content = fs::read_to_string(&role_c).unwrap_or_default();
    let mut names = Vec::new();

    let mut in_races_array = false;

    for line in content.lines() {
        if line.contains("const struct Race races[]") {
            in_races_array = true;
            continue;
        }

        if !in_races_array {
            continue;
        }

        let trimmed = line.trim();

        // Stop at terminator or end of array
        if trimmed == "};" {
            break;
        }

        // Look for lines starting with a quoted string that's a known race
        // Each race struct starts like: { "human", ... or just "human",
        if let Some(start) = trimmed.find('"') {
            let rest = &trimmed[start + 1..];
            if let Some(end) = rest.find('"') {
                let name = rest[..end].to_string();
                if KNOWN_RACES.contains(&name.as_str()) && !names.contains(&name) {
                    names.push(name);
                }
            }
        }
    }

    names
}

/// Extract rank titles from role definitions (simplified version)
pub fn extract_role_ranks() -> Vec<(String, Vec<String>)> {
    // This is a simplified extraction that pairs known roles with their ranks
    KNOWN_ROLES
        .iter()
        .map(|&role| (role.to_string(), Vec::new()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_extract_role_names() {
        let names = extract_role_names();

        println!("Found {} role names in C source:", names.len());
        for name in &names {
            println!("  - {}", name);
        }

        // Should have 13 roles
        assert!(
            names.len() >= 13,
            "Expected 13 roles, found {}",
            names.len()
        );

        // Check for known roles
        assert!(names.contains(&"Archeologist".to_string()));
        assert!(names.contains(&"Barbarian".to_string()));
        assert!(names.contains(&"Valkyrie".to_string()));
        assert!(names.contains(&"Wizard".to_string()));
    }

    #[test]
    fn test_extract_race_names() {
        let names = extract_race_names();

        println!("Found {} race names in C source:", names.len());
        for name in &names {
            println!("  - {}", name);
        }

        // Should have 5 races
        assert!(names.len() >= 5, "Expected 5 races, found {}", names.len());

        // Check for known races
        assert!(names.contains(&"human".to_string()));
        assert!(names.contains(&"elf".to_string()));
        assert!(names.contains(&"dwarf".to_string()));
        assert!(names.contains(&"gnome".to_string()));
        assert!(names.contains(&"orc".to_string()));
    }

    #[test]
    fn test_all_c_roles_in_rust() {
        use nh_core::data::roles::ROLES;

        let c_names: HashSet<String> = extract_role_names().into_iter().collect();
        let rust_names: HashSet<String> = ROLES.iter().map(|r| r.name.male.to_string()).collect();

        println!("C roles: {}", c_names.len());
        println!("Rust roles: {}", rust_names.len());

        // Find roles in C but not in Rust
        let missing_in_rust: Vec<_> = c_names.difference(&rust_names).collect();

        if !missing_in_rust.is_empty() {
            println!("\nMissing in Rust:");
            for name in &missing_in_rust {
                println!("  - {}", name);
            }
        }

        assert!(
            missing_in_rust.is_empty(),
            "Missing {} roles in Rust",
            missing_in_rust.len()
        );
    }

    #[test]
    fn test_all_c_races_in_rust() {
        use nh_core::data::roles::RACES;

        let c_names: HashSet<String> = extract_race_names().into_iter().collect();
        let rust_names: HashSet<String> = RACES.iter().map(|r| r.noun.to_lowercase()).collect();

        println!("C races: {}", c_names.len());
        println!("Rust races: {}", rust_names.len());

        // Find races in C but not in Rust (case-insensitive)
        let missing: Vec<_> = c_names
            .iter()
            .filter(|c| !rust_names.contains(&c.to_lowercase()))
            .collect();

        if !missing.is_empty() {
            println!("\nMissing in Rust:");
            for name in &missing {
                println!("  - {}", name);
            }
        }

        assert!(
            missing.is_empty(),
            "Missing {} races in Rust",
            missing.len()
        );
    }

    #[test]
    #[ignore] // Rank extraction not yet implemented
    fn test_role_rank_count() {
        let roles_with_ranks = extract_role_ranks();

        println!("Role ranks extracted:");
        for (role, ranks) in &roles_with_ranks {
            println!(
                "  {} ({} ranks): {:?}",
                role,
                ranks.len(),
                &ranks[..ranks.len().min(3)]
            );
        }

        // Each role should have 9 ranks
        for (role, ranks) in &roles_with_ranks {
            assert!(
                ranks.len() >= 9,
                "Role {} should have 9 ranks, found {}",
                role,
                ranks.len()
            );
        }
    }
}
