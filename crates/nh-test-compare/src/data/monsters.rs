//! Monster data comparison
//!
//! Parses monster definitions from C source and compares with Rust.

use std::fs;
use std::path::Path;

/// Parsed monster data from C source
#[derive(Debug, Clone)]
pub struct CMonster {
    pub name: String,
    pub symbol: char,
    pub level: i32,
    pub move_speed: i32,
    pub ac: i32,
    pub mr: i32,
    pub alignment: i32,
    pub gen_flags: u32,
    pub attacks: Vec<CAttack>,
    pub weight: i32,
    pub nutrition: i32,
    pub sound: String,
    pub size: String,
    pub resistances: u32,
    pub resistances_conveyed: u32,
    pub flags1: u64,
    pub flags2: u64,
    pub flags3: u64,
    pub difficulty: i32,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct CAttack {
    pub attack_type: String,
    pub damage_type: String,
    pub dice_num: i32,
    pub dice_sides: i32,
}

/// Parse a single MON() macro call from the C source
fn parse_mon_macro(line: &str) -> Option<CMonster> {
    // MON("monster name", S_CHAR, LVL(lvl, mov, ac, mr, align), ...)
    if !line.trim().starts_with("MON(") {
        return None;
    }

    // Extract the name (first quoted string)
    let name_start = line.find('"')? + 1;
    let name_end = line[name_start..].find('"')? + name_start;
    let name = line[name_start..name_end].to_string();

    // This is a simplified parser - for full comparison we'd need
    // to handle all the macro expansions properly
    Some(CMonster {
        name,
        symbol: ' ',
        level: 0,
        move_speed: 0,
        ac: 0,
        mr: 0,
        alignment: 0,
        gen_flags: 0,
        attacks: Vec::new(),
        weight: 0,
        nutrition: 0,
        sound: String::new(),
        size: String::new(),
        resistances: 0,
        resistances_conveyed: 0,
        flags1: 0,
        flags2: 0,
        flags3: 0,
        difficulty: 0,
        color: String::new(),
    })
}

/// Load and parse all monster definitions from C source
pub fn load_c_monsters() -> Vec<CMonster> {
    let monst_c = Path::new(super::NETHACK_SRC).join("src/monst.c");

    if !monst_c.exists() {
        return Vec::new();
    }

    let content = fs::read_to_string(&monst_c).unwrap_or_default();
    let mut monsters = Vec::new();
    let mut in_mons_array = false;

    for line in content.lines() {
        if line.contains("struct permonst mons[]") {
            in_mons_array = true;
            continue;
        }

        if in_mons_array {
            if let Some(monster) = parse_mon_macro(line) {
                monsters.push(monster);
            }
        }
    }

    monsters
}

/// Extract just monster names from C source for basic comparison
pub fn extract_monster_names() -> Vec<String> {
    let monst_c = Path::new(super::NETHACK_SRC).join("src/monst.c");

    if !monst_c.exists() {
        return Vec::new();
    }

    let content = fs::read_to_string(&monst_c).unwrap_or_default();
    let mut names = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("MON(\"") || trimmed.starts_with("MON3(\"") {
            // Extract name from MON("name", ...) or MON3("name", ...)
            if let Some(start) = trimmed.find('"') {
                let rest = &trimmed[start + 1..];
                if let Some(end) = rest.find('"') {
                    names.push(rest[..end].to_string());
                }
            }
        }
    }

    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_extract_monster_names() {
        let names = extract_monster_names();

        // Should have many monsters
        assert!(
            names.len() > 300,
            "Expected 300+ monsters, found {}",
            names.len()
        );

        // Check for some known monsters
        assert!(names.contains(&"giant ant".to_string()));
        assert!(names.contains(&"killer bee".to_string()));
        assert!(names.contains(&"Medusa".to_string()));
        assert!(names.contains(&"Wizard of Yendor".to_string()));
    }

    #[test]
    fn test_all_c_monsters_in_rust() {
        use nh_data::monsters::MONSTERS;

        let c_names: HashSet<String> = extract_monster_names().into_iter().collect();
        let rust_names: HashSet<String> = MONSTERS.iter().map(|m| m.name.to_string()).collect();

        // Find monsters in C but not in Rust
        let missing_in_rust: Vec<_> = c_names.difference(&rust_names).collect();

        // Find monsters in Rust but not in C
        let extra_in_rust: Vec<_> = rust_names.difference(&c_names).collect();

        println!("C monsters: {}", c_names.len());
        println!("Rust monsters: {}", rust_names.len());

        if !missing_in_rust.is_empty() {
            println!("\nMissing in Rust ({}):", missing_in_rust.len());
            for name in missing_in_rust.iter().take(20) {
                println!("  - {}", name);
            }
        }

        if !extra_in_rust.is_empty() {
            println!("\nExtra in Rust ({}):", extra_in_rust.len());
            for name in extra_in_rust.iter().take(20) {
                println!("  - {}", name);
            }
        }

        // Check coverage - most C monsters should be in Rust
        let coverage = (c_names.len() - missing_in_rust.len()) as f64 / c_names.len() as f64;
        assert!(
            coverage > 0.95,
            "Only {:.1}% of C monsters are in Rust",
            coverage * 100.0
        );
    }
}
