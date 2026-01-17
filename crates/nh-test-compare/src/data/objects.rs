//! Object data comparison
//!
//! Parses object definitions from C source and compares with Rust.

use std::fs;
use std::path::Path;

/// Parsed object data from C source
#[derive(Debug, Clone)]
pub struct CObject {
    pub name: String,
    pub description: String,
    pub class: char,
    pub weight: i32,
    pub cost: i32,
    pub material: String,
}

/// Extract object names from C source
pub fn extract_object_names() -> Vec<String> {
    let objects_c = Path::new(super::NETHACK_SRC).join("src/objects.c");

    if !objects_c.exists() {
        return Vec::new();
    }

    let content = fs::read_to_string(&objects_c).unwrap_or_default();
    let mut names = Vec::new();

    // Object macros: WEAPON("name", ...), ARMOR("name", ...), etc.
    let macros = [
        "PROJECTILE(",
        "BOW(",
        "WEAPON(",
        "ARMOR(",
        "HELM(",
        "CLOAK(",
        "SHIELD(",
        "GLOVES(",
        "BOOTS(",
        "RING(",
        "AMULET(",
        "TOOL(",
        "FOOD(",
        "POTION(",
        "SCROLL(",
        "SPBOOK(",
        "WAND(",
        "COIN(",
        "GEM(",
        "ROCK(",
        "BALL(",
        "CHAIN(",
        "VENOM(",
    ];

    for line in content.lines() {
        let trimmed = line.trim();
        for macro_name in &macros {
            if trimmed.starts_with(macro_name) {
                // Extract name from MACRO("name", ...)
                if let Some(start) = trimmed.find('"') {
                    let rest = &trimmed[start + 1..];
                    if let Some(end) = rest.find('"') {
                        let name = rest[..end].to_string();
                        if !name.is_empty() {
                            names.push(name);
                        }
                    }
                }
                break;
            }
        }
    }

    names
}

/// Extract object weights from C source
pub fn extract_object_weights() -> Vec<(String, i32)> {
    let objects_c = Path::new(super::NETHACK_SRC).join("src/objects.c");

    if !objects_c.exists() {
        return Vec::new();
    }

    let content = fs::read_to_string(&objects_c).unwrap_or_default();
    let mut results = Vec::new();

    // This is a simplified parser - real parsing would need to handle
    // the macro parameters properly
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("WEAPON(\"") {
            // WEAPON("name", desc, known, merge, big, prob, wt, cost, sdam, ldam, hitbon, typ, skill, mat, c)
            if let Some(start) = trimmed.find('"') {
                let rest = &trimmed[start + 1..];
                if let Some(end) = rest.find('"') {
                    let name = rest[..end].to_string();
                    // Try to extract weight (7th field after commas)
                    let after_name = &rest[end + 1..];
                    let parts: Vec<&str> = after_name.split(',').collect();
                    if parts.len() > 6 {
                        if let Ok(wt) = parts[6].trim().parse::<i32>() {
                            results.push((name, wt));
                        }
                    }
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_object_names() {
        let names = extract_object_names();

        // Should have many objects
        println!("Extracted {} object names from C source", names.len());

        // Check count - we expect around 450+ objects
        assert!(
            names.len() > 300,
            "Expected 300+ objects, found {}",
            names.len()
        );

        // Check for some known objects
        assert!(names.contains(&"dagger".to_string()));
        assert!(names.contains(&"plate mail".to_string()));
    }

    #[test]
    fn test_all_c_objects_in_rust() {
        use nh_data::objects::OBJECTS;

        let c_names: Vec<String> = extract_object_names();
        let rust_names: Vec<String> = OBJECTS
            .iter()
            .filter(|o| !o.name.is_empty() && o.name != "strange object")
            .map(|o| o.name.to_string())
            .collect();

        // For each C name, check if it exists in Rust (with fuzzy matching)
        // C macros use base names, Rust may have prefixes like "potion of", "scroll of"
        let prefixes = [
            "potion of ",
            "scroll of ",
            "spellbook of ",
            "wand of ",
            "ring of ",
        ];

        let mut matched = 0;
        let mut missing_in_rust = Vec::new();

        for c_name in &c_names {
            let found = rust_names.iter().any(|r_name| {
                // Exact match
                if r_name == c_name {
                    return true;
                }
                // Check if Rust name is "prefix + c_name"
                for prefix in &prefixes {
                    let full_name = format!("{}{}", prefix, c_name);
                    if r_name == &full_name {
                        return true;
                    }
                }
                // Check if c_name ends the Rust name
                if r_name.ends_with(c_name) {
                    return true;
                }
                false
            });

            if found {
                matched += 1;
            } else {
                missing_in_rust.push(c_name.clone());
            }
        }

        println!("C objects: {}", c_names.len());
        println!("Rust objects: {}", rust_names.len());
        println!("Matched (with fuzzy): {}", matched);

        if !missing_in_rust.is_empty() {
            println!("\nMissing in Rust ({}):", missing_in_rust.len());
            for name in missing_in_rust.iter().take(20) {
                println!("  - '{}'", name);
            }
        }

        // Check coverage
        let coverage = matched as f64 / c_names.len() as f64;
        assert!(
            coverage > 0.90,
            "Only {:.1}% of C objects found in Rust",
            coverage * 100.0
        );
    }

    #[test]
    fn test_weapon_weights_match() {
        use nh_data::objects::OBJECTS;

        let c_weapons = extract_object_weights();

        if c_weapons.is_empty() {
            println!("Warning: No C weapon weights extracted");
            return;
        }

        let mut matched = 0;
        let mut mismatched = Vec::new();

        for (name, c_weight) in &c_weapons {
            // Find in Rust objects
            if let Some(rust_obj) = OBJECTS.iter().find(|o| o.name == name) {
                if rust_obj.weight as i32 == *c_weight {
                    matched += 1;
                } else {
                    mismatched.push((
                        name.clone(),
                        *c_weight,
                        rust_obj.weight as i32,
                    ));
                }
            }
        }

        println!("Weapon weights matched: {}/{}", matched, c_weapons.len());
        if !mismatched.is_empty() {
            println!("First 10 weight mismatches:");
            for (name, c, r) in mismatched.iter().take(10) {
                println!("  '{}': C={} vs Rust={}", name, c, r);
            }
        }

        // All weights should match
        assert!(
            mismatched.is_empty(),
            "Found {} weight mismatches",
            mismatched.len()
        );
    }
}
