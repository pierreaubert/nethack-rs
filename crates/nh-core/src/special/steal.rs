//! Monster stealing system (steal.c)
//!
//! Handles monsters stealing items/gold from the player,
//! and related inventory transfer mechanics.

use crate::object::Object;
use crate::rng::GameRng;

/// Calculate a proportional subset of gold to steal (somegold from steal.c:32).
///
/// Nymphs and leprechauns steal a portion, not all gold.
/// Amounts scale: below 50 = take all, above = random proportional.
pub fn somegold(gold: i32, rng: &mut GameRng) -> i32 {
    // rn1(x, y) = rn2(x) + y
    if gold < 50 {
        gold
    } else if gold < 100 {
        rng.rn2((gold - 25 + 1) as u32) as i32 + 25
    } else if gold < 500 {
        rng.rn2((gold - 50 + 1) as u32) as i32 + 50
    } else if gold < 1000 {
        rng.rn2((gold - 100 + 1) as u32) as i32 + 100
    } else if gold < 5000 {
        rng.rn2((gold - 500 + 1) as u32) as i32 + 500
    } else if gold < 10000 {
        rng.rn2((gold - 1000 + 1) as u32) as i32 + 1000
    } else {
        rng.rn2((gold - 5000 + 1) as u32) as i32 + 5000
    }
}

/// Result of a steal attempt
#[derive(Debug, Clone)]
pub enum StealResult {
    /// Stole gold
    StoleGold(i32),
    /// Stole an item (returns inventory letter of stolen item)
    StoleItem(char),
    /// Stole equipped armor (armor needs to be removed first)
    StoleArmor(char),
    /// Stole the Amulet of Yendor
    StoleAmulet,
    /// Nothing to steal
    Nothing,
    /// Failed to steal (player resisted)
    Failed,
}

/// Equipment slot names for stolen armor messages (equipname from steal.c:12).
pub fn equipname(class: &str) -> &'static str {
    match class {
        "shirt" => "shirt",
        "boots" => "boots",
        "shield" => "shield",
        "gloves" => "gloves",
        "cloak" => "cloak",
        "helmet" => "helmet",
        _ => "suit",
    }
}

/// Attempt to steal gold from the player (stealgold from steal.c:81).
///
/// Returns how much gold was stolen (0 if player has no gold).
pub fn stealgold(player_gold: i32, rng: &mut GameRng) -> i32 {
    if player_gold <= 0 {
        return 0;
    }
    somegold(player_gold, rng)
}

/// Check if a monster should try to steal the Amulet (stealamulet from steal.c:531).
///
/// Covetous monsters (Wizard, quest nemesis) target specific quest artifacts
/// and the Amulet of Yendor.
pub fn should_steal_amulet(inventory: &[Object]) -> Option<char> {
    // Check for Amulet of Yendor first
    for obj in inventory {
        if obj.is_artifact() {
            if let Some(ref name) = obj.name {
                if name.contains("Amulet of Yendor") {
                    return Some(obj.inv_letter);
                }
            }
        }
    }
    None
}

/// Pick a random stealable item from inventory (used by steal from steal.c:245).
///
/// Returns the inventory letter of the selected item, or None if nothing to steal.
pub fn pick_steal_target(inventory: &[Object], rng: &mut GameRng) -> Option<char> {
    if inventory.is_empty() {
        return None;
    }

    // Count stealable items (non-worn items are easier to steal)
    let stealable: Vec<_> = inventory.iter()
        .filter(|o| o.worn_mask == 0) // Not currently worn
        .collect();

    if stealable.is_empty() {
        // All items are worn — pick any item (armor theft)
        let idx = rng.rn2(inventory.len() as u32) as usize;
        Some(inventory[idx].inv_letter)
    } else {
        let idx = rng.rn2(stealable.len() as u32) as usize;
        Some(stealable[idx].inv_letter)
    }
}

/// Transfer an object to a monster's inventory (mpickobj from steal.c:482).
///
/// Returns messages to display.
pub fn mpickobj_message(monster_name: &str, item_name: &str) -> String {
    format!("{} picks up {}.", monster_name, item_name)
}

/// Drop all special objects from a dying monster (mdrop_special_objs from steal.c:702).
///
/// Returns list of object descriptions that were dropped.
pub fn mdrop_special_objs(monster_items: &[Object]) -> Vec<String> {
    let mut dropped = Vec::new();
    for obj in monster_items {
        if obj.is_artifact() || obj.is_container() {
            dropped.push(obj.display_name());
        }
    }
    dropped
}

/// Release all objects from a monster (relobj from steal.c:732).
///
/// Called when a monster dies — drops inventory at death location.
pub fn relobj_messages(monster_name: &str, items: &[Object], is_pet: bool) -> Vec<String> {
    let mut messages = Vec::new();
    if items.is_empty() {
        return messages;
    }

    if is_pet {
        for obj in items {
            messages.push(format!("{} drops {}.", monster_name, obj.display_name()));
        }
    } else {
        messages.push(format!("{} drops {} item(s).", monster_name, items.len()));
    }
    messages
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_somegold_small_amounts() {
        let mut rng = GameRng::new(42);
        // Below 50 → take all
        assert_eq!(somegold(10, &mut rng), 10);
        assert_eq!(somegold(49, &mut rng), 49);
    }

    #[test]
    fn test_somegold_medium_amounts() {
        let mut rng = GameRng::new(42);
        let stolen = somegold(100, &mut rng);
        assert!(stolen >= 50 && stolen <= 100, "Stolen {} not in 50-100", stolen);
    }

    #[test]
    fn test_somegold_large_amounts() {
        let mut rng = GameRng::new(42);
        let stolen = somegold(10000, &mut rng);
        assert!(stolen >= 1000 && stolen <= 10000, "Stolen {} not in 1000-10000", stolen);
    }

    #[test]
    fn test_somegold_ranges() {
        // Test all ranges by checking bounds
        for &(gold, min, max) in &[
            (50, 25, 50),
            (99, 25, 99),
            (100, 50, 100),
            (499, 50, 499),
            (500, 100, 500),
            (999, 100, 999),
            (1000, 500, 1000),
            (4999, 500, 4999),
            (5000, 1000, 5000),
            (9999, 1000, 9999),
            (10000, 5000, 10000),
        ] {
            let mut rng = GameRng::new(42);
            let stolen = somegold(gold, &mut rng);
            assert!(stolen >= min && stolen <= max,
                "somegold({}) = {} not in {}..{}", gold, stolen, min, max);
        }
    }

    #[test]
    fn test_stealgold_no_gold() {
        let mut rng = GameRng::new(42);
        assert_eq!(stealgold(0, &mut rng), 0);
    }

    #[test]
    fn test_stealgold_has_gold() {
        let mut rng = GameRng::new(42);
        let stolen = stealgold(500, &mut rng);
        assert!(stolen > 0 && stolen <= 500);
    }

    #[test]
    fn test_pick_steal_target_empty() {
        let mut rng = GameRng::new(42);
        assert!(pick_steal_target(&[], &mut rng).is_none());
    }

    #[test]
    fn test_equipname() {
        assert_eq!(equipname("boots"), "boots");
        assert_eq!(equipname("shield"), "shield");
        assert_eq!(equipname("unknown"), "suit");
    }

    #[test]
    fn test_mdrop_special_objs_empty() {
        let dropped = mdrop_special_objs(&[]);
        assert!(dropped.is_empty());
    }
}
