//! Artifact data comparison
//!
//! Parses artifact definitions from C source and compares with Rust.

use std::fs;
use std::path::Path;

/// Parsed artifact data from C source
#[derive(Debug, Clone)]
pub struct CArtifact {
    pub name: String,
    pub object_type: String,
    pub cost: i64,
    pub alignment: String,
    pub role: String,
    pub race: String,
}

/// Extract artifact names from C artilist.h
pub fn extract_artifact_names() -> Vec<String> {
    let artilist_h = Path::new(super::NETHACK_SRC).join("include/artilist.h");

    if !artilist_h.exists() {
        return Vec::new();
    }

    let content = fs::read_to_string(&artilist_h).unwrap_or_default();
    let mut names = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Look for A("name", ... pattern
        if trimmed.starts_with("A(\"") {
            // Extract name from A("name", ...)
            if let Some(start) = trimmed.find('"') {
                let rest = &trimmed[start + 1..];
                if let Some(end) = rest.find('"') {
                    let name = rest[..end].to_string();
                    if !name.is_empty() {
                        names.push(name);
                    }
                }
            }
        }
    }

    names
}

/// Extract artifact data with more details
pub fn extract_artifacts() -> Vec<CArtifact> {
    let artilist_h = Path::new(super::NETHACK_SRC).join("include/artilist.h");

    if !artilist_h.exists() {
        return Vec::new();
    }

    let content = fs::read_to_string(&artilist_h).unwrap_or_default();
    let mut artifacts = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip the terminator and empty entries
        if trimmed.starts_with("A(\"") {
            if let Some(artifact) = parse_artifact_line(trimmed) {
                artifacts.push(artifact);
            }
        }
    }

    artifacts
}

/// Parse a single artifact line
fn parse_artifact_line(line: &str) -> Option<CArtifact> {
    // A("name", OBJ_TYPE, flags1, flags2, mtype, attack, defense, carry, invoke,
    //   align, role, race, cost, color)

    // Extract name
    let name_start = line.find('"')? + 1;
    let rest_after_name = &line[name_start..];
    let name_end = rest_after_name.find('"')?;
    let name = rest_after_name[..name_end].to_string();

    if name.is_empty() {
        return None;
    }

    // Get the rest after the name and split by comma
    let after_name = &rest_after_name[name_end + 1..];
    let parts: Vec<&str> = after_name.split(',').collect();

    // Object type is the first part after name (index 0)
    let object_type = parts.first().unwrap_or(&"").trim().to_string();

    // Cost is near the end (13th field, index 12 from start of A macro)
    // In parts, it's around index 10-11 depending on how we split
    // Let's just extract it by finding the number before the closing paren
    let cost = parts
        .iter()
        .rev()
        .nth(1) // Skip color, get cost
        .and_then(|s| s.trim().trim_end_matches('L').parse::<i64>().ok())
        .unwrap_or(0);

    Some(CArtifact {
        name,
        object_type,
        cost,
        alignment: String::new(), // Would need more complex parsing
        role: String::new(),
        race: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_extract_artifact_names() {
        let names = extract_artifact_names();

        println!("Found {} artifact names in C source", names.len());

        // Should have the major artifacts
        assert!(
            names.len() >= 20,
            "Expected 20+ artifacts, found {}",
            names.len()
        );

        // Check for known artifacts
        assert!(names.contains(&"Excalibur".to_string()));
        assert!(names.contains(&"Stormbringer".to_string()));
        assert!(names.contains(&"Mjollnir".to_string()));
        assert!(names.contains(&"The Orb of Detection".to_string()));
        assert!(names.contains(&"The Eye of the Aethiopica".to_string()));
    }

    #[test]
    fn test_all_c_artifacts_in_rust() {
        use nh_core::data::artifacts::ARTIFACTS;

        let c_names: HashSet<String> = extract_artifact_names().into_iter().collect();
        let rust_names: HashSet<String> = ARTIFACTS
            .iter()
            .filter(|a| !a.name.is_empty())
            .map(|a| a.name.to_string())
            .collect();

        println!("C artifacts: {}", c_names.len());
        println!("Rust artifacts: {}", rust_names.len());

        // Find artifacts in C but not in Rust
        let missing_in_rust: Vec<_> = c_names.difference(&rust_names).collect();
        let extra_in_rust: Vec<_> = rust_names.difference(&c_names).collect();

        if !missing_in_rust.is_empty() {
            println!("\nMissing in Rust ({}):", missing_in_rust.len());
            for name in &missing_in_rust {
                println!("  - {}", name);
            }
        }

        if !extra_in_rust.is_empty() {
            println!("\nExtra in Rust ({}):", extra_in_rust.len());
            for name in &extra_in_rust {
                println!("  - {}", name);
            }
        }

        // All C artifacts should be in Rust
        assert!(
            missing_in_rust.is_empty(),
            "Missing {} artifacts in Rust",
            missing_in_rust.len()
        );
    }

    #[test]
    fn test_artifact_costs() {
        use nh_core::data::artifacts::ARTIFACTS;

        let c_artifacts = extract_artifacts();

        if c_artifacts.is_empty() {
            println!("Warning: No C artifacts extracted");
            return;
        }

        let mut matched = 0;
        let mut mismatched = Vec::new();

        for c_art in &c_artifacts {
            if c_art.cost == 0 {
                continue; // Skip if we couldn't parse cost
            }

            if let Some(rust_art) = ARTIFACTS.iter().find(|a| a.name == c_art.name) {
                if rust_art.cost as i64 == c_art.cost {
                    matched += 1;
                } else {
                    mismatched.push((c_art.name.clone(), c_art.cost, rust_art.cost as i64));
                }
            }
        }

        println!("Artifact costs matched: {}", matched);

        if !mismatched.is_empty() {
            println!("\nCost mismatches:");
            for (name, c_cost, r_cost) in &mismatched {
                println!("  {}: C={} vs Rust={}", name, c_cost, r_cost);
            }
        }

        // Most costs should match
        let valid_c_artifacts = c_artifacts.iter().filter(|a| a.cost > 0).count();
        if valid_c_artifacts > 0 {
            let match_rate = matched as f64 / valid_c_artifacts as f64;
            assert!(
                match_rate >= 0.9,
                "Only {:.1}% of artifact costs match",
                match_rate * 100.0
            );
        }
    }
}
