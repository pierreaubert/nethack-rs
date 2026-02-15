//! Special item mechanics
//!
//! Implements unique item behaviors like loadstones that can't be dropped,
//! luckstones that affect luck, poisoned weapons, and greased items.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::object::Object;
use crate::player::You;
use crate::rng::GameRng;
use serde::{Deserialize, Serialize};

/// Type of special item effect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialItemType {
    Loadstone, // Cursed stone that can't be dropped
    Luckstone, // Lucky gem that affects luck
    Poisoned,  // Weapon coated in poison
    Greased,   // Item protected by grease
    Enchanted, // Magically enhanced item
    Blessed,   // Holy item with special properties
}

impl SpecialItemType {
    /// Get description of special item type
    pub fn description(&self) -> &'static str {
        match self {
            SpecialItemType::Loadstone => "cursed stone (cannot be dropped)",
            SpecialItemType::Luckstone => "lucky stone (affects fortune)",
            SpecialItemType::Poisoned => "poisoned (applies poison on hit)",
            SpecialItemType::Greased => "slippery (resists erosion)",
            SpecialItemType::Enchanted => "magically enhanced",
            SpecialItemType::Blessed => "holy item",
        }
    }
}

/// Loadstone - cursed stone that prevents dropping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loadstone {
    pub object_id: u32,
    pub held_turns: i32,     // How long held
    pub curse_strength: i32, // How hard to drop (1-5)
}

impl Loadstone {
    pub fn new(object_id: u32) -> Self {
        Self {
            object_id,
            held_turns: 0,
            curse_strength: 3,
        }
    }

    /// Chance to drop loadstone (based on strength and turns held)
    pub fn drop_chance(&self) -> i32 {
        let base_chance = (100 / (self.curse_strength * 2)).max(5);
        let turn_bonus = (self.held_turns / 50).min(30);
        base_chance + turn_bonus
    }

    /// Tick time held
    pub fn tick(&mut self) {
        self.held_turns += 1;
    }

    /// Try to drop the loadstone
    pub fn try_drop(&self, rng: &mut GameRng) -> bool {
        rng.percent(self.drop_chance() as u32)
    }
}

/// Luckstone - gem that grants luck bonus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Luckstone {
    pub object_id: u32,
    pub luck_bonus: i32, // +1 to +3
    pub active_turns: i32,
}

impl Luckstone {
    pub fn new(object_id: u32) -> Self {
        Self {
            object_id,
            luck_bonus: 1,
            active_turns: 0,
        }
    }

    pub fn with_bonus(mut self, bonus: i32) -> Self {
        self.luck_bonus = bonus.clamp(1, 3);
        self
    }

    /// Apply luck bonus to player
    pub fn apply_luck(&self, player: &mut You) {
        player.luck = (player.luck as i32 + self.luck_bonus).clamp(-13, 13) as i8;
    }

    /// Remove luck bonus from player
    pub fn remove_luck(&self, player: &mut You) {
        player.luck = (player.luck as i32 - self.luck_bonus).clamp(-13, 13) as i8;
    }

    /// Tick active turns
    pub fn tick(&mut self) {
        self.active_turns += 1;
    }

    /// Check if luckstone should fade
    pub fn should_fade(&self) -> bool {
        // Luckstone effect fades after 500 turns of use
        self.active_turns > 500
    }
}

/// Poisoned weapon effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoisonedWeapon {
    pub object_id: u32,
    pub poison_type: String,   // Type of poison
    pub poison_strength: i32,  // 1-5, how potent
    pub charge_remaining: i32, // Poison uses remaining
}

impl PoisonedWeapon {
    pub fn new(object_id: u32, poison_type: String, strength: i32) -> Self {
        Self {
            object_id,
            poison_type,
            poison_strength: strength.clamp(1, 5),
            charge_remaining: 10,
        }
    }

    /// Damage from poison hit
    pub fn poison_damage(&self) -> i32 {
        match self.poison_strength {
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 5,
            5 => 8,
            _ => 2,
        }
    }

    /// Chance to poison target on hit
    pub fn poison_chance(&self) -> i32 {
        match self.poison_strength {
            1 => 25,
            2 => 40,
            3 => 55,
            4 => 70,
            5 => 85,
            _ => 50,
        }
    }

    /// Use poison charge
    pub fn use_charge(&mut self) {
        if self.charge_remaining > 0 {
            self.charge_remaining -= 1;
        }
    }

    /// Check if poison is depleted
    pub fn is_depleted(&self) -> bool {
        self.charge_remaining <= 0
    }

    /// Get poison description
    pub fn description(&self) -> String {
        format!(
            "{} poison ({} strength)",
            self.poison_type, self.poison_strength
        )
    }
}

/// Greased item - protected from erosion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreasedItem {
    pub object_id: u32,
    pub grease_thickness: i32, // 0-3, how well covered
    pub uses_remaining: i32,   // How many hits before grease wears off
}

impl GreasedItem {
    pub fn new(object_id: u32) -> Self {
        Self {
            object_id,
            grease_thickness: 3,
            uses_remaining: 10,
        }
    }

    /// Erosion resistance from grease (percentage)
    pub fn erosion_resistance(&self) -> i32 {
        match self.grease_thickness {
            0 => 0,
            1 => 25,
            2 => 50,
            3 => 75,
            _ => 75,
        }
    }

    /// Try to use up grease
    pub fn try_use_charge(&mut self, rng: &mut GameRng) -> bool {
        if rng.percent(30) {
            self.uses_remaining -= 1;
            if self.uses_remaining <= 0 {
                self.grease_thickness = 0;
                return true; // Grease worn off
            }
        }
        false
    }

    /// Check if grease is still active
    pub fn is_active(&self) -> bool {
        self.grease_thickness > 0 && self.uses_remaining > 0
    }

    /// Get grease status description
    pub fn status(&self) -> &'static str {
        match self.grease_thickness {
            0 => "no longer greased",
            1 => "lightly greased",
            2 => "well greased",
            3 => "thoroughly greased",
            _ => "thoroughly greased",
        }
    }
}

/// Tracker for all special items in inventory
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecialItemTracker {
    pub loadstones: Vec<Loadstone>,
    pub luckstones: Vec<Luckstone>,
    pub poisoned_weapons: Vec<PoisonedWeapon>,
    pub greased_items: Vec<GreasedItem>,
}

impl SpecialItemTracker {
    pub fn new() -> Self {
        Self {
            loadstones: Vec::new(),
            luckstones: Vec::new(),
            poisoned_weapons: Vec::new(),
            greased_items: Vec::new(),
        }
    }

    /// Add loadstone
    pub fn add_loadstone(&mut self, loadstone: Loadstone) {
        self.loadstones.push(loadstone);
    }

    /// Add luckstone
    pub fn add_luckstone(&mut self, luckstone: Luckstone) {
        self.luckstones.push(luckstone);
    }

    /// Add poisoned weapon
    pub fn add_poisoned_weapon(&mut self, weapon: PoisonedWeapon) {
        self.poisoned_weapons.push(weapon);
    }

    /// Add greased item
    pub fn add_greased_item(&mut self, item: GreasedItem) {
        self.greased_items.push(item);
    }

    /// Remove item by object_id
    pub fn remove_item(&mut self, object_id: u32) {
        self.loadstones.retain(|l| l.object_id != object_id);
        self.luckstones.retain(|l| l.object_id != object_id);
        self.poisoned_weapons.retain(|w| w.object_id != object_id);
        self.greased_items.retain(|g| g.object_id != object_id);
    }

    /// Check if item is a loadstone
    pub fn is_loadstone(&self, object_id: u32) -> bool {
        self.loadstones.iter().any(|l| l.object_id == object_id)
    }

    /// Check if item is a luckstone
    pub fn is_luckstone(&self, object_id: u32) -> bool {
        self.luckstones.iter().any(|l| l.object_id == object_id)
    }

    /// Check if item is poisoned
    pub fn is_poisoned(&self, object_id: u32) -> bool {
        self.poisoned_weapons
            .iter()
            .any(|w| w.object_id == object_id)
    }

    /// Check if item is greased
    pub fn is_greased(&self, object_id: u32) -> bool {
        self.greased_items.iter().any(|g| g.object_id == object_id)
    }

    /// Get luckstone by object_id
    pub fn get_luckstone(&self, object_id: u32) -> Option<&Luckstone> {
        self.luckstones.iter().find(|l| l.object_id == object_id)
    }

    /// Get luckstone mutable by object_id
    pub fn get_luckstone_mut(&mut self, object_id: u32) -> Option<&mut Luckstone> {
        self.luckstones
            .iter_mut()
            .find(|l| l.object_id == object_id)
    }

    /// Get loadstone by object_id
    pub fn get_loadstone(&self, object_id: u32) -> Option<&Loadstone> {
        self.loadstones.iter().find(|l| l.object_id == object_id)
    }

    /// Get loadstone mutable by object_id
    pub fn get_loadstone_mut(&mut self, object_id: u32) -> Option<&mut Loadstone> {
        self.loadstones
            .iter_mut()
            .find(|l| l.object_id == object_id)
    }

    /// Get poisoned weapon by object_id
    pub fn get_poisoned_weapon(&self, object_id: u32) -> Option<&PoisonedWeapon> {
        self.poisoned_weapons
            .iter()
            .find(|w| w.object_id == object_id)
    }

    /// Get poisoned weapon mutable by object_id
    pub fn get_poisoned_weapon_mut(&mut self, object_id: u32) -> Option<&mut PoisonedWeapon> {
        self.poisoned_weapons
            .iter_mut()
            .find(|w| w.object_id == object_id)
    }

    /// Get greased item by object_id
    pub fn get_greased_item(&self, object_id: u32) -> Option<&GreasedItem> {
        self.greased_items.iter().find(|g| g.object_id == object_id)
    }

    /// Get greased item mutable by object_id
    pub fn get_greased_item_mut(&mut self, object_id: u32) -> Option<&mut GreasedItem> {
        self.greased_items
            .iter_mut()
            .find(|g| g.object_id == object_id)
    }

    /// Tick all time-based effects
    pub fn tick_effects(&mut self) {
        for loadstone in &mut self.loadstones {
            loadstone.tick();
        }
        for luckstone in &mut self.luckstones {
            luckstone.tick();
        }
    }

    /// Count total special items
    pub fn count_special_items(&self) -> usize {
        self.loadstones.len()
            + self.luckstones.len()
            + self.poisoned_weapons.len()
            + self.greased_items.len()
    }
}

/// Check if item is a special item
pub fn is_special_item(obj: &Object) -> bool {
    obj.poisoned || obj.greased
}

/// Detect luckstone (gem type)
pub fn detect_luckstone_type(obj: &Object) -> bool {
    // Check if this could be a luckstone based on object type
    // In NetHack, luckstones are specific gem types
    // For now, check if it's a gem
    matches!(obj.class, crate::object::ObjectClass::Gem)
}

/// Detect loadstone (cursed stone)
pub fn detect_loadstone_type(obj: &Object) -> bool {
    // Check if cursed stone
    obj.buc.is_cursed() && matches!(obj.class, crate::object::ObjectClass::Rock)
}

/// Try to drop an item (accounting for loadstone)
pub fn can_drop_item(tracker: &SpecialItemTracker, object_id: u32, rng: &mut GameRng) -> bool {
    if let Some(loadstone) = tracker.get_loadstone(object_id) {
        !loadstone.try_drop(rng)
    } else {
        true
    }
}

/// Get message when trying to drop loadstone
pub fn loadstone_stuck_message(object_id: u32, tracker: &SpecialItemTracker) -> Option<String> {
    if let Some(loadstone) = tracker.get_loadstone(object_id) {
        let msg = match loadstone.curse_strength {
            1..=2 => "You feel a slight resistance dropping this stone.",
            3..=4 => "The stone seems reluctant to leave your possession!",
            5 => "The stone clings firmly to your hand!",
            _ => "You cannot drop this cursed stone!",
        };
        Some(msg.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loadstone_creation() {
        let loadstone = Loadstone::new(1);
        assert_eq!(loadstone.object_id, 1);
        assert_eq!(loadstone.curse_strength, 3);
        assert_eq!(loadstone.held_turns, 0);
    }

    #[test]
    fn test_loadstone_drop_chance() {
        let loadstone = Loadstone::new(1);
        let base = loadstone.drop_chance();
        assert!(base > 0);
        assert!(base < 50);
    }

    #[test]
    fn test_loadstone_tick() {
        let mut loadstone = Loadstone::new(1);
        loadstone.tick();
        assert_eq!(loadstone.held_turns, 1);
    }

    #[test]
    fn test_luckstone_creation() {
        let luckstone = Luckstone::new(1);
        assert_eq!(luckstone.object_id, 1);
        assert_eq!(luckstone.luck_bonus, 1);
    }

    #[test]
    fn test_luckstone_with_bonus() {
        let luckstone = Luckstone::new(1).with_bonus(3);
        assert_eq!(luckstone.luck_bonus, 3);
    }

    #[test]
    fn test_poisoned_weapon_creation() {
        let weapon = PoisonedWeapon::new(1, "serpent".to_string(), 3);
        assert_eq!(weapon.object_id, 1);
        assert_eq!(weapon.poison_strength, 3);
        assert!(!weapon.is_depleted());
    }

    #[test]
    fn test_poisoned_weapon_damage() {
        let weapon = PoisonedWeapon::new(1, "serpent".to_string(), 3);
        assert_eq!(weapon.poison_damage(), 3);
    }

    #[test]
    fn test_poisoned_weapon_chance() {
        let weapon = PoisonedWeapon::new(1, "serpent".to_string(), 3);
        assert_eq!(weapon.poison_chance(), 55);
    }

    #[test]
    fn test_poisoned_weapon_depletion() {
        let mut weapon = PoisonedWeapon::new(1, "serpent".to_string(), 1);
        for _ in 0..10 {
            weapon.use_charge();
        }
        assert!(weapon.is_depleted());
    }

    #[test]
    fn test_greased_item_creation() {
        let item = GreasedItem::new(1);
        assert_eq!(item.object_id, 1);
        assert_eq!(item.grease_thickness, 3);
        assert!(item.is_active());
    }

    #[test]
    fn test_greased_item_resistance() {
        let item = GreasedItem::new(1);
        assert_eq!(item.erosion_resistance(), 75);
    }

    #[test]
    fn test_greased_item_status() {
        let item = GreasedItem::new(1);
        assert!(item.status().contains("greased"));
    }

    #[test]
    fn test_special_item_tracker_add() {
        let mut tracker = SpecialItemTracker::new();
        tracker.add_loadstone(Loadstone::new(1));
        assert_eq!(tracker.loadstones.len(), 1);
    }

    #[test]
    fn test_special_item_tracker_remove() {
        let mut tracker = SpecialItemTracker::new();
        tracker.add_loadstone(Loadstone::new(1));
        tracker.add_luckstone(Luckstone::new(2));
        tracker.remove_item(1);
        assert_eq!(tracker.loadstones.len(), 0);
        assert_eq!(tracker.luckstones.len(), 1);
    }

    #[test]
    fn test_special_item_tracker_is_loadstone() {
        let mut tracker = SpecialItemTracker::new();
        tracker.add_loadstone(Loadstone::new(1));
        assert!(tracker.is_loadstone(1));
        assert!(!tracker.is_loadstone(2));
    }

    #[test]
    fn test_special_item_tracker_is_poisoned() {
        let mut tracker = SpecialItemTracker::new();
        tracker.add_poisoned_weapon(PoisonedWeapon::new(1, "poison".to_string(), 2));
        assert!(tracker.is_poisoned(1));
        assert!(!tracker.is_poisoned(2));
    }

    #[test]
    fn test_special_item_tracker_get_loadstone() {
        let mut tracker = SpecialItemTracker::new();
        tracker.add_loadstone(Loadstone::new(1));
        let loadstone = tracker.get_loadstone(1);
        assert!(loadstone.is_some());
    }

    #[test]
    fn test_special_item_tracker_count() {
        let mut tracker = SpecialItemTracker::new();
        tracker.add_loadstone(Loadstone::new(1));
        tracker.add_luckstone(Luckstone::new(2));
        tracker.add_poisoned_weapon(PoisonedWeapon::new(3, "poison".to_string(), 1));
        assert_eq!(tracker.count_special_items(), 3);
    }

    #[test]
    fn test_can_drop_item_no_loadstone() {
        let tracker = SpecialItemTracker::new();
        let mut rng = crate::rng::GameRng::new(42);
        assert!(can_drop_item(&tracker, 1, &mut rng));
    }

    #[test]
    fn test_loadstone_stuck_message() {
        let mut tracker = SpecialItemTracker::new();
        tracker.add_loadstone(Loadstone::new(1));
        let msg = loadstone_stuck_message(1, &tracker);
        assert!(msg.is_some());
    }

    #[test]
    fn test_special_item_type_descriptions() {
        assert!(!SpecialItemType::Loadstone.description().is_empty());
        assert!(!SpecialItemType::Luckstone.description().is_empty());
        assert!(!SpecialItemType::Poisoned.description().is_empty());
    }
}
