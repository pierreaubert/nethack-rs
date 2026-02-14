//! Shopkeeper system (shk.c)
//!
//! Handles shop mechanics, pricing, and shopkeeper interactions.

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::object::Object;

use super::ShopType;

/// Damage record for shop property damage
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShopDamage {
    /// Position where damage occurred
    pub x: i8,
    pub y: i8,
    /// Cost of the damage
    pub cost: i32,
}

/// Shop data for a level
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Shop {
    /// Shop type
    pub shop_type: ShopType,
    /// Shopkeeper monster ID
    pub shopkeeper_id: Option<crate::monster::MonsterId>,
    /// Shop boundaries (x1, y1, x2, y2)
    pub bounds: (i8, i8, i8, i8),
    /// Whether the shop is open
    pub open: bool,
    /// Total debt owed by player (item purchases)
    pub debt: i32,
    /// Items the player has picked up but not paid for
    pub unpaid_items: Vec<crate::object::ObjectId>,
    /// Damage the player has done to the shop (broken doors, walls, etc.)
    pub damages: Vec<ShopDamage>,
    /// Player credit with this shop
    pub credit: i32,
}

impl Shop {
    /// Create a new shop
    pub fn new(shop_type: ShopType, bounds: (i8, i8, i8, i8)) -> Self {
        Self {
            shop_type,
            shopkeeper_id: None,
            bounds,
            open: true,
            debt: 0,
            unpaid_items: Vec::new(),
            damages: Vec::new(),
            credit: 0,
        }
    }

    /// Check if a position is inside the shop
    pub fn contains(&self, x: i8, y: i8) -> bool {
        x >= self.bounds.0 && x <= self.bounds.2 && y >= self.bounds.1 && y <= self.bounds.3
    }

    /// Record damage to the shop at a position (broken door, wall, etc.)
    pub fn add_damage(&mut self, x: i8, y: i8, cost: i32) {
        // Merge with existing damage at same position
        if let Some(d) = self.damages.iter_mut().find(|d| d.x == x && d.y == y) {
            d.cost += cost;
        } else {
            self.damages.push(ShopDamage { x, y, cost });
        }
    }

    /// Remove damage record at a position (when repaired or paid)
    pub fn remove_damage(&mut self, x: i8, y: i8) {
        self.damages.retain(|d| d.x != x || d.y != y);
    }

    /// Total damage cost
    pub fn damage_cost(&self) -> i32 {
        self.damages.iter().map(|d| d.cost).sum()
    }

    /// Total debt: unpaid items + property damage
    pub fn total_debt(&self) -> i32 {
        (self.debt + self.damage_cost() - self.credit).max(0)
    }

    /// Check if position is at the shop door
    pub fn is_door(&self, x: i8, y: i8) -> bool {
        // Door is typically at the edge of the shop
        let (x1, y1, x2, y2) = self.bounds;
        (x == x1 || x == x2 || y == y1 || y == y2)
            && self.contains(x, y)
    }
}

/// Calculate base price for an object
pub fn base_price(obj: &Object) -> i32 {
    // Use shop_price if set, otherwise estimate from object properties
    if obj.shop_price > 0 {
        return obj.shop_price;
    }

    // Base price by object class
    let class_price = match obj.class {
        crate::object::ObjectClass::Weapon => 10 + obj.damage_dice as i32 * 5,
        crate::object::ObjectClass::Armor => 10 + obj.base_ac as i32 * 10,
        crate::object::ObjectClass::Potion => 50,
        crate::object::ObjectClass::Scroll => 60,
        crate::object::ObjectClass::Wand => 100,
        crate::object::ObjectClass::Ring => 100,
        crate::object::ObjectClass::Amulet => 150,
        crate::object::ObjectClass::Tool => 30,
        crate::object::ObjectClass::Food => 5,
        crate::object::ObjectClass::Gem => 50,
        crate::object::ObjectClass::Rock => 1,
        crate::object::ObjectClass::Coin => 1,
        crate::object::ObjectClass::Spellbook => 100,
        _ => 10,
    };

    // Adjust for enchantment
    let enchant_bonus = obj.enchantment.max(0) as i32 * 10;

    class_price + enchant_bonus
}

/// Calculate selling price (what shopkeeper pays player)
pub fn selling_price(obj: &Object, charisma: i8) -> i32 {
    let base = base_price(obj);
    
    // Charisma affects selling price (higher charisma = better price)
    let cha_modifier = match charisma {
        0..=5 => 50,   // 50% of base
        6..=8 => 60,   // 60%
        9..=12 => 70,  // 70%
        13..=15 => 80, // 80%
        16..=18 => 90, // 90%
        _ => 100,      // 100% for very high charisma
    };

    (base * cha_modifier / 100).max(1)
}

/// Calculate buying price (what player pays shopkeeper)
pub fn buying_price(obj: &Object, charisma: i8) -> i32 {
    let base = base_price(obj);
    
    // Charisma affects buying price (higher charisma = lower price)
    let cha_modifier = match charisma {
        0..=5 => 200,  // 200% of base
        6..=8 => 175,  // 175%
        9..=12 => 150, // 150%
        13..=15 => 125, // 125%
        16..=18 => 110, // 110%
        _ => 100,      // 100% for very high charisma
    };

    (base * cha_modifier / 100).max(1)
}

/// Handle player picking up an item in a shop
pub fn pickup_in_shop(state: &mut GameState, obj: &Object, shop: &mut Shop) {
    let price = buying_price(obj, state.player.attr_current.get(crate::player::Attribute::Charisma));
    shop.debt += price * obj.quantity;
    shop.unpaid_items.push(obj.id);
    
    state.message(format!(
        "\"{}\" ({} zorkmids{})",
        obj.name.as_deref().unwrap_or("item"),
        price,
        if obj.quantity > 1 { " each" } else { "" }
    ));
}

/// Handle player paying for items by shop index
pub fn pay_bill_at(state: &mut GameState, shop_idx: usize) -> ActionResult {
    if shop_idx >= state.current_level.shops.len() {
        state.message("There is nobody here to pay.");
        return ActionResult::NoTime;
    }

    let total = state.current_level.shops[shop_idx].total_debt();
    if total == 0 {
        state.message("You don't owe anything.");
        return ActionResult::NoTime;
    }

    if state.player.gold >= total {
        state.player.gold -= total;
        state.message(format!("You pay {} zorkmids.", total));
        let shop = &mut state.current_level.shops[shop_idx];
        shop.debt = 0;
        shop.credit = 0;
        shop.damages.clear();
        shop.unpaid_items.clear();

        // Mark items as paid
        for obj in &mut state.inventory {
            obj.unpaid = false;
        }
    } else {
        // Partial payment: apply what we have
        let paid = state.player.gold;
        state.player.gold = 0;
        state.current_level.shops[shop_idx].credit += paid;
        let remaining = state.current_level.shops[shop_idx].total_debt();
        state.message(format!(
            "You pay {} zorkmids. You still owe {} zorkmids.",
            paid, remaining
        ));
    }
    ActionResult::Success
}

/// Handle player selling an item to shopkeeper
pub fn sell_item(state: &mut GameState, obj_letter: char) -> bool {
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o.clone(),
        None => {
            state.message("You don't have that item.");
            return false;
        }
    };

    if obj.unpaid {
        state.message("You haven't paid for that yet!");
        return false;
    }

    let price = selling_price(&obj, state.player.attr_current.get(crate::player::Attribute::Charisma));
    
    state.player.gold += price;
    state.remove_from_inventory(obj_letter);
    
    state.message(format!(
        "You sell the {} for {} zorkmids.",
        obj.name.as_deref().unwrap_or("item"),
        price
    ));
    
    true
}

/// Check if player is trying to leave shop with unpaid items
pub fn check_leaving_shop(state: &mut GameState, shop: &Shop) -> bool {
    let total = shop.total_debt();
    if total > 0 {
        state.message(format!(
            "\"Hey! You owe me {} zorkmids!\"",
            total
        ));
        return false;
    }
    true
}

/// Shopkeeper greeting when player enters shop
pub fn shopkeeper_greeting(state: &mut GameState, shop: &Shop) {
    let greeting = match shop.shop_type {
        ShopType::General => "Welcome to my general store!",
        ShopType::Armor => "Welcome! Looking for some protection?",
        ShopType::Weapon => "Welcome! Need something sharp?",
        ShopType::Food => "Welcome! Hungry?",
        ShopType::Scroll => "Welcome to my scroll emporium!",
        ShopType::Potion => "Welcome! Need a potion?",
        ShopType::Wand => "Welcome! Looking for magical implements?",
        ShopType::Tool => "Welcome! Need some tools?",
        ShopType::Book => "Welcome to my bookstore!",
        ShopType::Ring => "Welcome! Looking for jewelry?",
        ShopType::Candle => "Welcome! Need some light?",
        ShopType::Tin => "Welcome to my tin shop!",
    };
    state.message(format!("\"{}\"", greeting));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{Object, ObjectClass, ObjectId};

    #[test]
    fn test_base_price() {
        let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        obj.damage_dice = 2;
        
        let price = base_price(&obj);
        assert!(price > 0);
    }

    #[test]
    fn test_charisma_affects_price() {
        let obj = Object::new(ObjectId(1), 0, ObjectClass::Potion);
        
        let low_cha_buy = buying_price(&obj, 5);
        let high_cha_buy = buying_price(&obj, 18);
        
        // Higher charisma should mean lower buying price
        assert!(high_cha_buy < low_cha_buy);
        
        let low_cha_sell = selling_price(&obj, 5);
        let high_cha_sell = selling_price(&obj, 18);
        
        // Higher charisma should mean higher selling price
        assert!(high_cha_sell > low_cha_sell);
    }

    #[test]
    fn test_shop_contains() {
        let shop = Shop::new(ShopType::General, (10, 10, 20, 15));

        assert!(shop.contains(15, 12));
        assert!(shop.contains(10, 10));
        assert!(shop.contains(20, 15));
        assert!(!shop.contains(5, 5));
        assert!(!shop.contains(25, 12));
    }

    #[test]
    fn test_shop_damage_tracking() {
        let mut shop = Shop::new(ShopType::General, (10, 10, 20, 15));

        shop.add_damage(12, 12, 50);
        shop.add_damage(15, 12, 100);
        assert_eq!(shop.damage_cost(), 150);

        // Merge damage at same position
        shop.add_damage(12, 12, 30);
        assert_eq!(shop.damage_cost(), 180);
        assert_eq!(shop.damages.len(), 2);

        shop.remove_damage(12, 12);
        assert_eq!(shop.damage_cost(), 100);
        assert_eq!(shop.damages.len(), 1);
    }

    #[test]
    fn test_shop_total_debt() {
        let mut shop = Shop::new(ShopType::General, (10, 10, 20, 15));
        shop.debt = 200;
        shop.add_damage(12, 12, 50);

        assert_eq!(shop.total_debt(), 250);

        // Credit reduces total debt
        shop.credit = 100;
        assert_eq!(shop.total_debt(), 150);

        // Credit can't make debt negative
        shop.credit = 500;
        assert_eq!(shop.total_debt(), 0);
    }
}
