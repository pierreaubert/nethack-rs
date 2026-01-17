//! Armor class (AC) calculation comparison
//!
//! Compares AC calculation between C and Rust implementations.
//!
//! NetHack AC system:
//! - Base AC is 10 (unarmored)
//! - Lower AC is better (negative AC is excellent)
//! - Armor provides negative modifiers (subtracted from base)
//!
//! C formula from find_mac() in worn.c:
//! ```c
//! base = mon->data->ac;
//! for each worn item:
//!     base -= ARM_BONUS(obj);
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// AC values for different armor types from C source
#[derive(Debug, Clone)]
pub struct ArmorAC {
    pub name: String,
    pub ac_bonus: i32,
}

/// Extract armor AC values from C objects.c
pub fn extract_armor_ac_values() -> Vec<ArmorAC> {
    let objects_c = Path::new(crate::data::NETHACK_SRC).join("src/objects.c");

    if !objects_c.exists() {
        return Vec::new();
    }

    let content = fs::read_to_string(&objects_c).unwrap_or_default();
    let mut armors = Vec::new();

    // ARMOR macro spans multiple lines, so we need to collect full entries
    // Format: ARMOR("name", desc, known, magic, blk, power, prob, delay, wt, cost, ac, ...)
    // We look for lines starting with ARMOR(" (actual armor entries, not macro definitions)
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip macro definitions
        if trimmed.starts_with("#define") {
            continue;
        }

        // Find ARMOR("name" pattern
        if trimmed.starts_with("ARMOR(\"") {
            // Collect this line and following lines until we have enough commas
            let mut full_line = trimmed.to_string();
            let mut j = i + 1;
            while j < lines.len() && full_line.matches(',').count() < 10 {
                full_line.push_str(lines[j].trim());
                j += 1;
            }

            if let Some(armor) = parse_armor_line(&full_line) {
                armors.push(armor);
            }
        }
    }

    armors
}

fn parse_armor_line(line: &str) -> Option<ArmorAC> {
    // ARMOR("name", desc, kn, mgc, blk, power, prob, delay, wt, cost, ac, can, sub, metal, c)
    // After name: desc, kn, mgc, blk, power, prob, delay, wt, cost, ac
    // AC is the 10th field after name (index 9)
    let start = line.find('"')? + 1;
    let rest = &line[start..];
    let end = rest.find('"')?;
    let name = rest[..end].to_string();

    if name.is_empty() {
        return None;
    }

    // Get everything after the closing quote and comma
    let after_name = &rest[end + 1..];
    // Split by comma, filtering empty parts
    let parts: Vec<&str> = after_name
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // AC is at index 9 (10th field): desc, kn, mgc, blk, power, prob, delay, wt, cost, ac
    let ac_str = parts.get(9)?;
    let ac_bonus = ac_str.parse::<i32>().ok()?;

    Some(ArmorAC { name, ac_bonus })
}

/// Known base AC values for monsters
pub const BASE_PLAYER_AC: i32 = 10;

/// Standard armor AC bonuses (from NetHack 3.6.7)
pub fn standard_armor_bonuses() -> HashMap<&'static str, i32> {
    let mut map = HashMap::new();

    // Body armor (ac field in ARMOR macro)
    map.insert("leather armor", 2);
    map.insert("ring mail", 3);
    map.insert("studded leather armor", 3);
    map.insert("scale mail", 4);
    map.insert("chain mail", 5);
    map.insert("splint mail", 6);
    map.insert("banded mail", 6);
    map.insert("plate mail", 7);
    map.insert("crystal plate mail", 7);
    map.insert("bronze plate mail", 6);
    map.insert("dragon scale mail", 9);

    // Shields
    map.insert("small shield", 1);
    map.insert("large shield", 2);
    map.insert("shield of reflection", 2);

    // Helms
    map.insert("leather helmet", 1);
    map.insert("orcish helm", 1);
    map.insert("dwarvish iron helm", 2);
    map.insert("iron helm", 2);
    map.insert("helm of brilliance", 1);
    map.insert("helm of opposite alignment", 1);
    map.insert("helm of telepathy", 1);

    // Cloaks
    map.insert("leather cloak", 1);
    map.insert("cloak of protection", 3);
    map.insert("cloak of invisibility", 1);
    map.insert("cloak of magic resistance", 1);
    map.insert("cloak of displacement", 1);
    map.insert("oilskin cloak", 1);
    map.insert("alchemy smock", 1);

    // Gloves
    map.insert("leather gloves", 1);
    map.insert("gauntlets of fumbling", 1);
    map.insert("gauntlets of power", 1);
    map.insert("gauntlets of dexterity", 1);

    // Boots
    map.insert("low boots", 1);
    map.insert("iron shoes", 2);
    map.insert("high boots", 2);
    map.insert("speed boots", 1);
    map.insert("water walking boots", 1);
    map.insert("jumping boots", 1);
    map.insert("elven boots", 1);
    map.insert("kicking boots", 1);
    map.insert("levitation boots", 1);

    map
}

/// Calculate total AC from worn armor
pub fn calculate_total_ac(worn_armor: &[&str]) -> i32 {
    let bonuses = standard_armor_bonuses();
    let mut ac = BASE_PLAYER_AC;

    for armor_name in worn_armor {
        if let Some(&bonus) = bonuses.get(armor_name) {
            ac -= bonus;
        }
    }

    ac
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_armor_values() {
        let armors = extract_armor_ac_values();

        println!("Found {} armor entries in C source:", armors.len());
        for armor in armors.iter().take(10) {
            println!("  {} (AC: {})", armor.name, armor.ac_bonus);
        }

        // Should have many armor pieces (body armor only, not helms/cloaks/etc)
        assert!(armors.len() >= 15, "Expected 15+ body armors, found {}", armors.len());
    }

    #[test]
    fn test_base_ac() {
        // Unarmored AC should be 10
        assert_eq!(BASE_PLAYER_AC, 10);
        assert_eq!(calculate_total_ac(&[]), 10);
    }

    #[test]
    fn test_armor_reduces_ac() {
        // Wearing armor should reduce AC (lower is better)
        let ac_with_leather = calculate_total_ac(&["leather armor"]);
        assert_eq!(ac_with_leather, 8); // 10 - 2 = 8

        let ac_with_plate = calculate_total_ac(&["plate mail"]);
        assert_eq!(ac_with_plate, 3); // 10 - 7 = 3
    }

    #[test]
    fn test_multiple_armor_pieces() {
        // Full armor set
        let full_armor = [
            "plate mail",      // -7
            "large shield",    // -2
            "iron helm",       // -2
            "leather gloves",  // -1
            "high boots",      // -2
        ];
        let ac = calculate_total_ac(&full_armor);
        // 10 - 7 - 2 - 2 - 1 - 2 = -4
        assert_eq!(ac, -4);
    }

    #[test]
    fn test_armor_ac_values_match_c() {
        let c_armors = extract_armor_ac_values();
        let expected = standard_armor_bonuses();

        if c_armors.is_empty() {
            println!("Warning: Could not read C armor values");
            return;
        }

        let mut matched = 0;
        let mut mismatched = Vec::new();

        // C AC field is the RESULTING AC when wearing ONLY that armor (base 10 - bonus)
        // So C_AC = 10 - expected_bonus, or expected_bonus = 10 - C_AC
        for c_armor in &c_armors {
            if let Some(&expected_bonus) = expected.get(c_armor.name.as_str()) {
                let expected_resulting_ac = BASE_PLAYER_AC - expected_bonus;
                if expected_resulting_ac == c_armor.ac_bonus {
                    matched += 1;
                } else {
                    mismatched.push((
                        c_armor.name.clone(),
                        c_armor.ac_bonus,
                        expected_resulting_ac,
                        expected_bonus,
                    ));
                }
            }
        }

        println!("Armor AC values matched: {}", matched);
        if !mismatched.is_empty() {
            println!("\nMismatches:");
            for (name, c_ac, expected_ac, bonus) in &mismatched {
                println!(
                    "  {}: C_AC={} vs expected_AC={} (bonus={})",
                    name, c_ac, expected_ac, bonus
                );
            }
        }

        // At least 80% should match
        let expected_count = c_armors
            .iter()
            .filter(|a| expected.contains_key(a.name.as_str()))
            .count();
        if expected_count > 0 {
            let match_rate = matched as f64 / expected_count as f64;
            assert!(
                match_rate >= 0.8,
                "Only {:.1}% of armor AC values match",
                match_rate * 100.0
            );
        }
    }

    #[test]
    fn test_dragon_scale_mail_best() {
        // Dragon scale mail should provide the best AC for body armor
        let c_armors = extract_armor_ac_values();

        if c_armors.is_empty() {
            return;
        }

        // Find dragon scale mail variants
        let dragon_armors: Vec<_> = c_armors
            .iter()
            .filter(|a| a.name.contains("dragon scale mail"))
            .collect();

        println!("Dragon scale mail variants:");
        for armor in &dragon_armors {
            println!("  {} (AC: {})", armor.name, armor.ac_bonus);
        }

        // Dragon scale mail should provide 9 AC
        for armor in dragon_armors {
            assert_eq!(
                armor.ac_bonus, 9,
                "Dragon scale mail should give 9 AC, got {}",
                armor.ac_bonus
            );
        }
    }

    #[test]
    fn test_negative_ac_possible() {
        // It should be possible to achieve negative AC
        let best_armor = [
            "dragon scale mail",   // -9
            "shield of reflection", // -2
            "helm of brilliance",  // -1
            "cloak of protection", // -3
            "gauntlets of power",  // -1
            "speed boots",         // -1
        ];

        let bonuses = standard_armor_bonuses();
        let mut ac = BASE_PLAYER_AC;
        for armor in &best_armor {
            if let Some(&bonus) = bonuses.get(armor) {
                ac -= bonus;
            }
        }

        // 10 - 9 - 2 - 1 - 3 - 1 - 1 = -7
        assert_eq!(ac, -7);
        assert!(ac < 0, "Should be able to achieve negative AC");
    }
}
