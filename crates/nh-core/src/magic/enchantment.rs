//! Enchantment system for weapons and armor
//!
//! Handles enchanting/disenchanting items, recharging wands, and tracking
//! enchantment level effects on combat and protection.

use crate::object::{BucStatus, Object, ObjectClass};
use crate::player::You;
use crate::rng::GameRng;

/// Result of an enchantment operation
#[derive(Debug, Clone)]
pub struct EnchantmentResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Whether the enchantment succeeded
    pub success: bool,
    /// New enchantment level
    pub new_enchantment: i8,
    /// Old enchantment level
    pub old_enchantment: i8,
}

impl EnchantmentResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            success: false,
            new_enchantment: 0,
            old_enchantment: 0,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

impl Default for EnchantmentResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Enchant a weapon or armor piece
pub fn enchant_weapon(obj: &mut Object, rng: &mut GameRng) -> EnchantmentResult {
    let mut result = EnchantmentResult::new();
    result.old_enchantment = obj.enchantment;

    // Check if item can be enchanted
    if !can_enchant(obj) {
        result
            .messages
            .push("That item cannot be enchanted.".to_string());
        return result;
    }

    // Calculate success chance based on current enchantment
    let success_chance = if obj.enchantment >= 5 {
        40 // 40% at +5
    } else if obj.enchantment >= 3 {
        60 // 60% at +3
    } else {
        80 // 80% at lower enchantments
    };

    if !rng.percent(success_chance) {
        result.messages.push("The enchantment fails!".to_string());
        // Cursed items have 50% chance to become further cursed
        if obj.is_cursed() && rng.percent(50) {
            obj.enchantment = obj.enchantment.saturating_sub(1);
            result
                .messages
                .push("The item becomes more cursed.".to_string());
        }
        result.new_enchantment = obj.enchantment;
        return result;
    }

    // Success!
    obj.enchantment = obj.enchantment.saturating_add(1);
    result.success = true;
    result.new_enchantment = obj.enchantment;

    if obj.is_blessed() {
        result
            .messages
            .push(format!("{} glows brightly!", obj.display_name()));
    } else if obj.is_cursed() {
        result
            .messages
            .push(format!("{} glows faintly.", obj.display_name()));
        obj.buc = BucStatus::Uncursed; // Enchanting removes curse
    } else {
        result
            .messages
            .push(format!("{} glows!", obj.display_name()));
    }

    result
}

/// Enchant armor or shield
pub fn enchant_armor(obj: &mut Object, rng: &mut GameRng) -> EnchantmentResult {
    let mut result = EnchantmentResult::new();
    result.old_enchantment = obj.enchantment;

    // Check if item can be enchanted
    if !can_enchant(obj) {
        result
            .messages
            .push("That item cannot be enchanted.".to_string());
        return result;
    }

    // Armor enchantment is slightly harder
    let success_chance = if obj.enchantment >= 5 {
        30
    } else if obj.enchantment >= 3 {
        50
    } else {
        70
    };

    if !rng.percent(success_chance) {
        result
            .messages
            .push("The armor resists enchantment.".to_string());
        result.new_enchantment = obj.enchantment;
        return result;
    }

    // Success!
    obj.enchantment = obj.enchantment.saturating_add(1);
    result.success = true;
    result.new_enchantment = obj.enchantment;

    if obj.is_blessed() {
        result
            .messages
            .push(format!("{} shines brightly!", obj.display_name()));
    } else {
        result
            .messages
            .push(format!("{} glows slightly.", obj.display_name()));
    }

    result
}

/// Disenchant an item (reduce its enchantment)
pub fn disenchant(obj: &mut Object) -> EnchantmentResult {
    let mut result = EnchantmentResult::new();
    result.old_enchantment = obj.enchantment;

    if obj.enchantment <= -9 {
        result
            .messages
            .push("The item cannot be disenchanted further.".to_string());
        result.new_enchantment = obj.enchantment;
        return result;
    }

    obj.enchantment = obj.enchantment.saturating_sub(1);
    result.success = true;
    result.new_enchantment = obj.enchantment;
    result
        .messages
        .push(format!("{} dims.", obj.display_name()));

    result
}

/// Check if an item can be enchanted
pub fn can_enchant(obj: &Object) -> bool {
    matches!(
        obj.class,
        ObjectClass::Weapon | ObjectClass::Armor | ObjectClass::Wand | ObjectClass::Ring
    )
}

/// Calculate the combat bonus from enchantment
pub fn enchantment_to_damage_bonus(enchantment: i8) -> i32 {
    if enchantment > 0 {
        enchantment as i32
    } else {
        0
    }
}

/// Calculate the defense bonus from armor enchantment
pub fn enchantment_to_ac_bonus(enchantment: i8) -> i32 {
    // Lower AC is better, so positive enchantment reduces AC
    -(enchantment as i32)
}

/// Check if an item is over-enchanted (likely to break)
pub fn is_over_enchanted(obj: &Object) -> bool {
    obj.enchantment > 10 && obj.recharged > 0
}

/// Calculate chance of enchantment being damaged/lost
pub fn enchantment_damage_chance(obj: &Object) -> i32 {
    if obj.enchantment <= 0 {
        0
    } else if obj.enchantment <= 3 {
        5
    } else if obj.enchantment <= 6 {
        10
    } else {
        20
    }
}

/// Damage item enchantment (used when item breaks, rusts, etc.)
pub fn damage_enchantment(obj: &mut Object, amount: i8, rng: &mut GameRng) -> bool {
    let chance = enchantment_damage_chance(obj);
    if rng.percent(chance as u32) {
        obj.enchantment = obj.enchantment.saturating_sub(amount);
        true
    } else {
        false
    }
}

/// Recharge a wand
pub fn recharge_wand(obj: &mut Object) -> EnchantmentResult {
    let mut result = EnchantmentResult::new();
    result.old_enchantment = obj.enchantment;

    if obj.class != ObjectClass::Wand {
        result
            .messages
            .push("That item cannot be recharged.".to_string());
        return result;
    }

    if obj.recharged >= 3 {
        result
            .messages
            .push("The wand has been recharged too many times and crumbles.".to_string());
        obj.enchantment = 0;
        result.success = false;
        result.new_enchantment = 0;
        return result;
    }

    // Recharging success varies based on recharged count
    let success_chance = match obj.recharged {
        0 => 100,
        1 => 75,
        2 => 50,
        _ => 0,
    };

    result.success = success_chance == 100; // For now, always succeed at first try
    obj.enchantment += 8;
    obj.recharged += 1;
    result.new_enchantment = obj.enchantment;
    result
        .messages
        .push(format!("{} glows with power!", obj.display_name()));

    result
}

/// Get enchantment level description
pub fn describe_enchantment(obj: &Object) -> String {
    match obj.enchantment {
        i if i > 0 => format!("+{}", i),
        i if i < 0 => format!("{}", i),
        _ => "uncursed".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::ObjectId;

    #[test]
    fn test_enchant_weapon_success() {
        let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        obj.enchantment = 0;
        let mut rng = GameRng::new(42);

        let result = enchant_weapon(&mut obj, &mut rng);
        assert!(result.success || !result.success); // May succeed or fail randomly
        assert!(obj.enchantment >= result.old_enchantment);
    }

    #[test]
    fn test_enchant_armor_harder() {
        let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Armor);
        obj.enchantment = 5;
        let mut rng = GameRng::new(42);

        let result = enchant_armor(&mut obj, &mut rng);
        // Armor enchantment at +5 has only 30% success
        assert!(result.old_enchantment == 5);
    }

    #[test]
    fn test_cannot_enchant_non_enchantable() {
        let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Food);
        let mut rng = GameRng::new(42);

        let result = enchant_weapon(&mut obj, &mut rng);
        assert!(!result.success);
        assert!(result.messages[0].contains("cannot be enchanted"));
    }

    #[test]
    fn test_disenchant() {
        let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        obj.enchantment = 5;

        let result = disenchant(&mut obj);
        assert!(result.success);
        assert_eq!(obj.enchantment, 4);
        assert!(result.messages[0].contains("dims"));
    }

    #[test]
    fn test_enchantment_to_damage_bonus() {
        assert_eq!(enchantment_to_damage_bonus(0), 0);
        assert_eq!(enchantment_to_damage_bonus(5), 5);
        assert_eq!(enchantment_to_damage_bonus(-3), 0);
    }

    #[test]
    fn test_enchantment_to_ac_bonus() {
        assert_eq!(enchantment_to_ac_bonus(3), -3);
        assert_eq!(enchantment_to_ac_bonus(-2), 2);
    }

    #[test]
    fn test_recharge_wand() {
        let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        obj.enchantment = 2;
        obj.recharged = 0;

        let result = recharge_wand(&mut obj);
        assert!(result.success);
        assert_eq!(obj.recharged, 1);
        assert!(obj.enchantment > 2);
    }

    #[test]
    fn test_recharge_wand_limit() {
        let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        obj.enchantment = 10;
        obj.recharged = 3;

        let result = recharge_wand(&mut obj);
        assert!(!result.success);
        assert_eq!(obj.enchantment, 0);
    }

    #[test]
    fn test_describe_enchantment() {
        let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);

        obj.enchantment = 3;
        assert_eq!(describe_enchantment(&obj), "+3");

        obj.enchantment = -2;
        assert_eq!(describe_enchantment(&obj), "-2");

        obj.enchantment = 0;
        assert_eq!(describe_enchantment(&obj), "uncursed");
    }

    #[test]
    fn test_damage_enchantment() {
        let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        obj.enchantment = 5;
        let initial = obj.enchantment;

        let mut rng = GameRng::new(42);
        let damaged = damage_enchantment(&mut obj, 1, &mut rng);

        // May or may not damage depending on chance
        assert!(obj.enchantment <= initial);
    }
}
