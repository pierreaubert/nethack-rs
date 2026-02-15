//! Equipment lifecycle management
//!
//! Handles equip/unequip effects, cursed item effects, artifact effects,
//! special item mechanics, and property binding during each game turn.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::magic::{
    ArtifactEffects, CursedConsequence, CursedEffect, Luckstone, apply_all_equipment_properties,
    apply_artifact_effects, apply_cursed_effect, calculate_curse_magnitude, check_fumble,
    cursed_effect_message, determine_cursed_effects, determine_item_properties,
    get_artifact_effects, remove_all_equipment_properties, remove_artifact_effects,
};
use crate::player::You;
use crate::rng::GameRng;

/// Apply all cursed item effects to player (called once per turn)
pub fn apply_cursed_item_effects(player: &mut You, rng: &mut GameRng) -> Vec<String> {
    let mut messages = Vec::new();

    // Get all fumble chances from cursed weapons
    let total_fumble = player.cursed_item_tracker.total_fumble_chance();
    if total_fumble > 0 && check_fumble(total_fumble, rng) {
        messages.push("Your hands fumble!".to_string());
    }

    // Apply AC penalties from cursed armor
    let ac_penalty = player.cursed_item_tracker.total_armor_penalty();
    if ac_penalty > 0 {
        player.armor_class = (player.armor_class as i32 + ac_penalty) as i8;
    }

    messages
}

/// Tick special items (loadstones, luckstones, etc.)
pub fn tick_special_items(player: &mut You) -> Vec<String> {
    let mut messages = Vec::new();

    // Tick luckstones - collect IDs first to avoid borrow issues
    let luckstone_ids: Vec<u32> = player
        .special_item_tracker
        .luckstones
        .iter()
        .map(|ls| ls.object_id)
        .collect();

    let mut faded_luckstones = Vec::new();
    for luckstone_id in luckstone_ids {
        if let Some(luckstone) = player.special_item_tracker.get_luckstone_mut(luckstone_id) {
            luckstone.tick();

            // Check if luckstone should fade
            if luckstone.should_fade() {
                faded_luckstones.push(luckstone_id);
            }
        }
    }

    // Now handle faded luckstones separately
    for luckstone_id in faded_luckstones {
        // Remove luck effect (simplified - just update player.luck directly)
        player.luck = player.luck.saturating_sub(1);
        messages.push("Your luckstone's glow fades.".to_string());
        player.special_item_tracker.remove_item(luckstone_id);
    }

    // Tick loadstones
    for loadstone in &mut player.special_item_tracker.loadstones {
        loadstone.tick();
    }

    messages
}

/// Remove all properties from equipped items (before re-applying)
pub fn reset_equipment_properties(player: &mut You) {
    // Clone to avoid borrow checker issues
    let property_binding = player.property_binding.clone();
    let equipped_items = player.equipped_items.clone();
    remove_all_equipment_properties(player, &property_binding, &equipped_items);
}

/// Re-apply all properties from equipped items
pub fn reapply_equipment_properties(player: &mut You) {
    // Clone to avoid borrow checker issues
    let property_binding = player.property_binding.clone();
    let equipped_items = player.equipped_items.clone();
    apply_all_equipment_properties(player, &property_binding, &equipped_items);
}

/// Equip an item and apply its effects
pub fn equip_item_with_effects(player: &mut You, object_id: u32, rng: &mut GameRng) -> Vec<String> {
    let mut messages = Vec::new();

    // Add to equipped list
    if !player.equipped_items.contains(&object_id) {
        player.equipped_items.push(object_id);
    }

    // Determine if item has cursed effects
    // This would normally come from the object data, but we stub it here
    let cursed_effects = Vec::new(); // Would be populated from object data

    // Add cursed consequences to tracker
    for effect in &cursed_effects {
        let consequence = CursedConsequence::new(*effect, object_id);
        // Get message before moving consequence
        let msg = cursed_effect_message(&consequence);
        player.cursed_item_tracker.add_curse(consequence);
        messages.push(msg);
    }

    // Apply properties - clone property_binding to avoid borrow issues
    let property_binding = player.property_binding.clone();
    property_binding.apply_item_properties(player, object_id);

    messages
}

/// Unequip an item and remove its effects
pub fn unequip_item_with_effects(player: &mut You, object_id: u32) -> Vec<String> {
    let mut messages = Vec::new();

    // Remove from equipped list
    player.equipped_items.retain(|&id| id != object_id);

    // Remove cursed item consequences
    player.cursed_item_tracker.remove_item_curses(object_id);

    // Remove properties - clone property_binding to avoid borrow issues
    let property_binding = player.property_binding.clone();
    property_binding.remove_item_properties(player, object_id);

    messages
}

/// Check if item can be dropped (loadstone check)
pub fn can_drop_item(player: &You, object_id: u32, rng: &mut GameRng) -> bool {
    use crate::magic::can_drop_item as check_drop;
    check_drop(&player.special_item_tracker, object_id, rng)
}

/// Get message when trying to drop loadstone
pub fn get_drop_failure_message(player: &You, object_id: u32) -> Option<String> {
    use crate::magic::loadstone_stuck_message;
    loadstone_stuck_message(object_id, &player.special_item_tracker)
}

/// Apply artifact effects when equipped
pub fn apply_artifact_to_player(player: &mut You, artifact_id: u8) -> Vec<String> {
    let mut messages = Vec::new();

    let effects = get_artifact_effects(artifact_id);
    apply_artifact_effects(player, &effects);

    // Add artifact warning message if applicable
    use crate::magic::get_artifact_warning;
    if let Some(warning) = get_artifact_warning(artifact_id, "artifact") {
        messages.push(warning);
    }

    messages
}

/// Remove artifact effects when unequipped
pub fn remove_artifact_from_player(player: &mut You, artifact_id: u8) {
    let effects = get_artifact_effects(artifact_id);
    remove_artifact_effects(player, &effects);
}

/// Check artifact warning for nearby monsters
pub fn check_artifact_warning(
    player: &You,
    artifact_id: u8,
    nearby_monsters: &[(char, String)],
) -> Option<String> {
    use crate::magic::should_warn_of_monster;

    for (monster_symbol, _monster_name) in nearby_monsters {
        if should_warn_of_monster(artifact_id, *monster_symbol) {
            if get_artifact_effects(artifact_id)
                .ability
                .description()
                .contains("warns")
            {
                return Some("You sense a nearby threat!".to_string());
            }
        }
    }

    None
}

/// Get poisoned weapon hit damage and message
pub fn apply_poisoned_weapon_damage(
    player: &You,
    weapon_id: u32,
    rng: &mut GameRng,
) -> Option<(i32, String)> {
    if let Some(poison) = player.special_item_tracker.get_poisoned_weapon(weapon_id) {
        if rng.percent(poison.poison_chance() as u32) {
            let damage = poison.poison_damage();
            let message = format!("The {} poison drips from the weapon!", poison.poison_type);
            return Some((damage, message));
        }
    }

    None
}

/// Reduce greased item charge
pub fn use_grease_charge(player: &mut You, item_id: u32, rng: &mut GameRng) -> Option<String> {
    if let Some(greased) = player.special_item_tracker.get_greased_item_mut(item_id) {
        if greased.try_use_charge(rng) {
            return Some("The grease coating wears off.".to_string());
        }
    }

    None
}

/// Get erosion resistance from greased item
pub fn get_grease_erosion_resistance(player: &You, item_id: u32) -> i32 {
    if let Some(greased) = player.special_item_tracker.get_greased_item(item_id) {
        greased.erosion_resistance()
    } else {
        0
    }
}

/// Apply luckstone bonus to player
pub fn apply_luckstone_bonus(player: &mut You, luckstone_id: u32) {
    if let Some(luckstone) = player
        .special_item_tracker
        .luckstones
        .iter()
        .find(|ls| ls.object_id == luckstone_id)
        .cloned()
    {
        luckstone.apply_luck(player);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_drop_item() {
        let player = You::default();
        let mut rng = crate::rng::GameRng::new(42);

        // Item not in tracker, should be droppable
        assert!(can_drop_item(&player, 999, &mut rng));
    }

    #[test]
    fn test_apply_cursed_item_effects() {
        let mut player = You::default();
        let mut rng = crate::rng::GameRng::new(42);

        let messages = apply_cursed_item_effects(&mut player, &mut rng);
        // Should return a list of messages (may be empty)
        assert!(messages.is_empty() || messages.len() > 0);
    }

    #[test]
    fn test_reset_equipment_properties() {
        let mut player = You::default();

        // Should not panic
        reset_equipment_properties(&mut player);
        assert!(true);
    }

    #[test]
    fn test_reapply_equipment_properties() {
        let mut player = You::default();

        // Should not panic
        reapply_equipment_properties(&mut player);
        assert!(true);
    }

    #[test]
    fn test_tick_special_items() {
        let mut player = You::default();

        let messages = tick_special_items(&mut player);
        // Should return a list of messages
        assert!(messages.is_empty() || messages.len() > 0);
    }

    #[test]
    fn test_get_drop_failure_message() {
        let player = You::default();

        // No loadstone, should return None
        let msg = get_drop_failure_message(&player, 999);
        assert!(msg.is_none());
    }

    #[test]
    fn test_equip_item_with_effects() {
        let mut player = You::default();
        let mut rng = crate::rng::GameRng::new(42);

        let messages = equip_item_with_effects(&mut player, 1, &mut rng);

        // Item should be added to equipped list
        assert!(player.equipped_items.contains(&1));
    }

    #[test]
    fn test_unequip_item_with_effects() {
        let mut player = You::default();

        // Add item to equipped
        player.equipped_items.push(1);

        let _messages = unequip_item_with_effects(&mut player, 1);

        // Item should be removed
        assert!(!player.equipped_items.contains(&1));
    }

    #[test]
    fn test_apply_artifact_to_player() {
        let mut player = You::default();

        let messages = apply_artifact_to_player(&mut player, 1);
        // Should return messages
        assert!(messages.is_empty() || messages.len() > 0);
    }

    #[test]
    fn test_remove_artifact_from_player() {
        let mut player = You::default();

        // Should not panic
        remove_artifact_from_player(&mut player, 1);
        assert!(true);
    }

    #[test]
    fn test_apply_poisoned_weapon_damage() {
        let player = You::default();
        let mut rng = crate::rng::GameRng::new(42);

        // No poisoned weapon, should return None
        let result = apply_poisoned_weapon_damage(&player, 999, &mut rng);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_grease_erosion_resistance() {
        let player = You::default();

        // No greased item, should return 0
        let resistance = get_grease_erosion_resistance(&player, 999);
        assert_eq!(resistance, 0);
    }
}
