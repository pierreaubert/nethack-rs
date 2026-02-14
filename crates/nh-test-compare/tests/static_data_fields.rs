//! Step 2 Gap: Field-level static data comparison
//!
//! The original static_data.rs only compared NAMES. This file compares
//! actual field values between C and Rust monster/object definitions.

use std::collections::HashMap;
use std::process::Command;

const C_SRC: &str = "/Users/pierre/src/games/NetHack-3.6.7/src/monst.c";

/// Parsed monster fields from C source
#[derive(Debug)]
struct CMonsterFields {
    name: String,
    level: i32,
    speed: i32,
    ac: i32,
    mr: i32,
    alignment: i32,
    weight: i32,
    nutrition: i32,
}

/// Resolve C weight constants to numeric values
fn resolve_weight(s: &str) -> i32 {
    match s {
        "WT_HUMAN" => 1450,
        "WT_ELF" => 800,
        "WT_DRAGON" => 4500,
        _ => s.parse().unwrap_or(0),
    }
}

/// Extract monster fields from C monst.c using perl regex
fn extract_c_monster_fields() -> Vec<CMonsterFields> {
    // Use \w+ for weight field to capture WT_HUMAN, WT_ELF, WT_DRAGON
    let output = Command::new("perl")
        .args([
            "-0777", "-ne",
            r#"while (/MON\("([^"]+)",\s*\w+,\s*LVL\((\d+),\s*(\d+),\s*(-?\d+),\s*(\d+),\s*(-?\d+)\).*?SIZ\((\w+),\s*(\d+),/gs) { print "$1|$2|$3|$4|$5|$6|$7|$8\n" }"#,
            C_SRC,
        ])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut monsters = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() == 8 {
            monsters.push(CMonsterFields {
                name: parts[0].to_string(),
                level: parts[1].parse().unwrap_or(0),
                speed: parts[2].parse().unwrap_or(0),
                ac: parts[3].parse().unwrap_or(0),
                mr: parts[4].parse().unwrap_or(0),
                alignment: parts[5].parse().unwrap_or(0),
                weight: resolve_weight(parts[6]),
                nutrition: parts[7].parse().unwrap_or(0),
            });
        }
    }

    monsters
}

// ============================================================================
// Monster field comparison tests
// ============================================================================

#[test]
fn test_monster_fields_extraction() {
    let c_monsters = extract_c_monster_fields();
    assert!(
        c_monsters.len() > 390,
        "Should extract 390+ monster field records from C, got {}",
        c_monsters.len()
    );

    // Spot-check first monster
    let giant_ant = c_monsters.iter().find(|m| m.name == "giant ant");
    assert!(giant_ant.is_some(), "Should find 'giant ant' in C data");
    let ga = giant_ant.unwrap();
    assert_eq!(ga.level, 2);
    assert_eq!(ga.speed, 18);
    assert_eq!(ga.ac, 3);
    assert_eq!(ga.mr, 0);
}

#[test]
fn test_monster_level_comparison() {
    let c_monsters = extract_c_monster_fields();
    let c_map: HashMap<&str, &CMonsterFields> = c_monsters.iter()
        .map(|m| (m.name.as_str(), m))
        .collect();

    let rust_monsters = nh_data::monsters::MONSTERS;

    let mut matched = 0;
    let mut mismatched = Vec::new();
    let mut not_found = 0;

    for rm in rust_monsters.iter() {
        if let Some(cm) = c_map.get(rm.name) {
            if rm.level as i32 == cm.level {
                matched += 1;
            } else {
                mismatched.push((rm.name, rm.level as i32, cm.level));
            }
        } else {
            not_found += 1;
        }
    }

    println!("\n=== Monster Level Comparison ===");
    println!("Matched: {}", matched);
    println!("Mismatched: {}", mismatched.len());
    println!("Not in C: {}", not_found);

    if !mismatched.is_empty() {
        println!("\nFirst 20 level mismatches:");
        for (name, rust_val, c_val) in mismatched.iter().take(20) {
            println!("  {}: Rust={} C={}", name, rust_val, c_val);
        }
    }

    // At least 90% should match
    let total_compared = matched + mismatched.len();
    if total_compared > 0 {
        let pct = matched * 100 / total_compared;
        assert!(pct >= 90, "Only {}% of monster levels match (need 90%+)", pct);
    }
}

#[test]
fn test_monster_speed_comparison() {
    let c_monsters = extract_c_monster_fields();
    let c_map: HashMap<&str, &CMonsterFields> = c_monsters.iter()
        .map(|m| (m.name.as_str(), m))
        .collect();

    let rust_monsters = nh_data::monsters::MONSTERS;

    let mut matched = 0;
    let mut mismatched = Vec::new();

    for rm in rust_monsters.iter() {
        if let Some(cm) = c_map.get(rm.name) {
            if rm.move_speed as i32 == cm.speed {
                matched += 1;
            } else {
                mismatched.push((rm.name, rm.move_speed as i32, cm.speed));
            }
        }
    }

    println!("\n=== Monster Speed Comparison ===");
    println!("Matched: {}", matched);
    println!("Mismatched: {}", mismatched.len());

    if !mismatched.is_empty() {
        println!("\nFirst 20 speed mismatches:");
        for (name, rust_val, c_val) in mismatched.iter().take(20) {
            println!("  {}: Rust={} C={}", name, rust_val, c_val);
        }
    }

    let total_compared = matched + mismatched.len();
    if total_compared > 0 {
        let pct = matched * 100 / total_compared;
        assert!(pct >= 90, "Only {}% of monster speeds match (need 90%+)", pct);
    }
}

#[test]
fn test_monster_ac_comparison() {
    let c_monsters = extract_c_monster_fields();
    let c_map: HashMap<&str, &CMonsterFields> = c_monsters.iter()
        .map(|m| (m.name.as_str(), m))
        .collect();

    let rust_monsters = nh_data::monsters::MONSTERS;

    let mut matched = 0;
    let mut mismatched = Vec::new();

    for rm in rust_monsters.iter() {
        if let Some(cm) = c_map.get(rm.name) {
            if rm.armor_class as i32 == cm.ac {
                matched += 1;
            } else {
                mismatched.push((rm.name, rm.armor_class as i32, cm.ac));
            }
        }
    }

    println!("\n=== Monster AC Comparison ===");
    println!("Matched: {}", matched);
    println!("Mismatched: {}", mismatched.len());

    if !mismatched.is_empty() {
        println!("\nFirst 20 AC mismatches:");
        for (name, rust_val, c_val) in mismatched.iter().take(20) {
            println!("  {}: Rust={} C={}", name, rust_val, c_val);
        }
    }

    let total_compared = matched + mismatched.len();
    if total_compared > 0 {
        let pct = matched * 100 / total_compared;
        assert!(pct >= 90, "Only {}% of monster ACs match (need 90%+)", pct);
    }
}

#[test]
fn test_monster_mr_comparison() {
    let c_monsters = extract_c_monster_fields();
    let c_map: HashMap<&str, &CMonsterFields> = c_monsters.iter()
        .map(|m| (m.name.as_str(), m))
        .collect();

    let rust_monsters = nh_data::monsters::MONSTERS;

    let mut matched = 0;
    let mut mismatched = Vec::new();

    for rm in rust_monsters.iter() {
        if let Some(cm) = c_map.get(rm.name) {
            if rm.magic_resistance as i32 == cm.mr {
                matched += 1;
            } else {
                mismatched.push((rm.name, rm.magic_resistance as i32, cm.mr));
            }
        }
    }

    println!("\n=== Monster MR Comparison ===");
    println!("Matched: {}", matched);
    println!("Mismatched: {}", mismatched.len());

    if !mismatched.is_empty() {
        println!("\nFirst 20 MR mismatches:");
        for (name, rust_val, c_val) in mismatched.iter().take(20) {
            println!("  {}: Rust={} C={}", name, rust_val, c_val);
        }
    }

    let total_compared = matched + mismatched.len();
    if total_compared > 0 {
        let pct = matched * 100 / total_compared;
        assert!(pct >= 90, "Only {}% of monster MRs match (need 90%+)", pct);
    }
}

#[test]
fn test_monster_weight_comparison() {
    let c_monsters = extract_c_monster_fields();
    let c_map: HashMap<&str, &CMonsterFields> = c_monsters.iter()
        .map(|m| (m.name.as_str(), m))
        .collect();

    let rust_monsters = nh_data::monsters::MONSTERS;

    let mut matched = 0;
    let mut mismatched = Vec::new();

    for rm in rust_monsters.iter() {
        if let Some(cm) = c_map.get(rm.name) {
            if rm.corpse_weight as i32 == cm.weight {
                matched += 1;
            } else {
                mismatched.push((rm.name, rm.corpse_weight as i32, cm.weight));
            }
        }
    }

    println!("\n=== Monster Weight Comparison ===");
    println!("Matched: {}", matched);
    println!("Mismatched: {}", mismatched.len());

    if !mismatched.is_empty() {
        println!("\nFirst 20 weight mismatches:");
        for (name, rust_val, c_val) in mismatched.iter().take(20) {
            println!("  {}: Rust={} C={}", name, rust_val, c_val);
        }
    }

    let total_compared = matched + mismatched.len();
    if total_compared > 0 {
        let pct = matched * 100 / total_compared;
        assert!(pct >= 90, "Only {}% of monster weights match (need 90%+)", pct);
    }
}

#[test]
fn test_monster_nutrition_comparison() {
    let c_monsters = extract_c_monster_fields();
    let c_map: HashMap<&str, &CMonsterFields> = c_monsters.iter()
        .map(|m| (m.name.as_str(), m))
        .collect();

    let rust_monsters = nh_data::monsters::MONSTERS;

    let mut matched = 0;
    let mut mismatched = Vec::new();

    for rm in rust_monsters.iter() {
        if let Some(cm) = c_map.get(rm.name) {
            if rm.corpse_nutrition as i32 == cm.nutrition {
                matched += 1;
            } else {
                mismatched.push((rm.name, rm.corpse_nutrition as i32, cm.nutrition));
            }
        }
    }

    println!("\n=== Monster Nutrition Comparison ===");
    println!("Matched: {}", matched);
    println!("Mismatched: {}", mismatched.len());

    if !mismatched.is_empty() {
        println!("\nFirst 20 nutrition mismatches:");
        for (name, rust_val, c_val) in mismatched.iter().take(20) {
            println!("  {}: Rust={} C={}", name, rust_val, c_val);
        }
    }

    let total_compared = matched + mismatched.len();
    if total_compared > 0 {
        let pct = matched * 100 / total_compared;
        assert!(pct >= 90, "Only {}% of monster nutritions match (need 90%+)", pct);
    }
}

// ============================================================================
// Comprehensive field summary
// ============================================================================

#[test]
fn test_monster_field_summary() {
    let c_monsters = extract_c_monster_fields();
    let c_map: HashMap<&str, &CMonsterFields> = c_monsters.iter()
        .map(|m| (m.name.as_str(), m))
        .collect();

    let rust_monsters = nh_data::monsters::MONSTERS;

    let fields = ["level", "speed", "ac", "mr", "alignment", "weight", "nutrition"];
    let mut results: HashMap<&str, (usize, usize)> = HashMap::new(); // (matched, total)

    for rm in rust_monsters.iter() {
        if let Some(cm) = c_map.get(rm.name) {
            let checks = [
                ("level", rm.level as i32 == cm.level),
                ("speed", rm.move_speed as i32 == cm.speed),
                ("ac", rm.armor_class as i32 == cm.ac),
                ("mr", rm.magic_resistance as i32 == cm.mr),
                ("alignment", rm.alignment as i32 == cm.alignment),
                ("weight", rm.corpse_weight as i32 == cm.weight),
                ("nutrition", rm.corpse_nutrition as i32 == cm.nutrition),
            ];

            for (field, ok) in &checks {
                let entry = results.entry(field).or_insert((0, 0));
                entry.1 += 1;
                if *ok {
                    entry.0 += 1;
                }
            }
        }
    }

    println!("\n=== Monster Field-Level Summary ===");
    println!("{:<15} {:>8} {:>8} {:>6}", "Field", "Match", "Total", "%");
    println!("{}", "-".repeat(40));

    for field in &fields {
        if let Some((matched, total)) = results.get(field) {
            let pct = if *total > 0 { matched * 100 / total } else { 0 };
            println!("{:<15} {:>8} {:>8} {:>5}%", field, matched, total, pct);
        }
    }

    println!("\nC monsters parsed: {}", c_monsters.len());
    println!("Rust monsters: {}", rust_monsters.len());
}
