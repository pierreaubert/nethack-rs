//! Economy system (shk.c)
//!
//! Handles shop transactions, pricing, billing, and shopkeeper interactions.

use crate::object::{Object, ObjectClass, ObjectId};
use crate::rng::GameRng;

/// Bill entry for unpaid items
#[derive(Debug, Clone)]
pub struct BillEntry {
    /// Object ID of the item
    pub object_id: ObjectId,
    /// Original price when picked up
    pub price: i32,
    /// Quantity when picked up
    pub quantity: i32,
    /// Whether item was used (charges, etc.)
    pub useup: bool,
}

/// Shop bill tracking
#[derive(Debug, Clone, Default)]
pub struct ShopBill {
    /// Items on the bill
    pub entries: Vec<BillEntry>,
    /// Total debt owed
    pub debt: i32,
    /// Credit with shopkeeper
    pub credit: i32,
    /// Shopkeeper's room index
    pub shop_room: Option<usize>,
}

impl ShopBill {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an item to the bill
    pub fn add_item(&mut self, object_id: ObjectId, price: i32, quantity: i32) {
        self.entries.push(BillEntry {
            object_id,
            price,
            quantity,
            useup: false,
        });
        self.debt += price * quantity;
    }

    /// Remove an item from the bill (when paid or returned)
    pub fn remove_item(&mut self, object_id: ObjectId) -> Option<BillEntry> {
        if let Some(idx) = self.entries.iter().position(|e| e.object_id == object_id) {
            let entry = self.entries.remove(idx);
            self.debt -= entry.price * entry.quantity;
            Some(entry)
        } else {
            None
        }
    }

    /// Check if an item is on the bill
    pub fn is_on_bill(&self, object_id: ObjectId) -> bool {
        self.entries.iter().any(|e| e.object_id == object_id)
    }

    /// Get total debt
    pub fn total_debt(&self) -> i32 {
        self.debt.max(0)
    }

    /// Apply credit to reduce debt
    pub fn apply_credit(&mut self, amount: i32) -> i32 {
        let applied = amount.min(self.debt);
        self.debt -= applied;
        applied
    }

    /// Add credit
    pub fn add_credit(&mut self, amount: i32) {
        self.credit += amount;
    }

    /// Clear the bill (all paid)
    pub fn clear(&mut self) {
        self.entries.clear();
        self.debt = 0;
    }
}

/// Base prices for object classes (from objclass.h)
pub fn base_cost(class: ObjectClass) -> i32 {
    match class {
        ObjectClass::Amulet => 150,
        ObjectClass::Weapon => 10,
        ObjectClass::Armor => 10,
        ObjectClass::Ring => 100,
        ObjectClass::Tool => 10,
        ObjectClass::Food => 5,
        ObjectClass::Potion => 50,
        ObjectClass::Scroll => 50,
        ObjectClass::Spellbook => 100,
        ObjectClass::Wand => 100,
        ObjectClass::Gem => 1,
        ObjectClass::Rock => 0,
        ObjectClass::Ball => 10,
        ObjectClass::Chain => 10,
        ObjectClass::Venom | ObjectClass::Random | ObjectClass::IllObj | ObjectClass::Coin => 0,
    }
}

/// Calculate the price of an object
/// Based on get_cost() from shk.c
pub fn get_cost(obj: &Object, selling: bool) -> i32 {
    let base = if obj.shop_price > 0 {
        obj.shop_price
    } else {
        base_cost(obj.class)
    };

    let mut price = base;

    // Enchantment affects price
    if obj.enchantment != 0 {
        if obj.enchantment > 0 {
            price += obj.enchantment as i32 * 10;
        } else {
            // Negative enchantment reduces price
            price = (price + obj.enchantment as i32 * 5).max(1);
        }
    }

    // BUC status affects price
    if obj.is_cursed() {
        price = price * 3 / 4; // 75% for cursed
    } else if obj.is_blessed() {
        price = price * 5 / 4; // 125% for blessed
    }

    // Damaged items worth less
    if obj.erosion1 > 0 || obj.erosion2 > 0 {
        let erosion = (obj.erosion1 + obj.erosion2) as i32;
        price = price * (4 - erosion).max(1) / 4;
    }

    // Quantity multiplier
    price *= obj.quantity;

    // Selling price is typically 50% of buying price
    if selling {
        price /= 2;
    }

    price.max(1)
}

/// Calculate selling price (what shopkeeper pays you)
pub fn selling_price(obj: &Object) -> i32 {
    get_cost(obj, true)
}

/// Calculate buying price (what you pay shopkeeper)
pub fn buying_price(obj: &Object) -> i32 {
    get_cost(obj, false)
}

/// Charisma-based price adjustment
/// Higher charisma = better prices
pub fn charisma_price_adjustment(price: i32, charisma: i32) -> i32 {
    let adjustment = match charisma {
        0..=5 => 150,   // Very low charisma: 150% price
        6..=7 => 125,   // Low charisma: 125% price
        8..=10 => 110,  // Below average: 110% price
        11..=15 => 100, // Average: normal price
        16..=17 => 90,  // Good charisma: 90% price
        18..=20 => 80,  // High charisma: 80% price
        _ => 75,        // Very high charisma: 75% price
    };
    (price * adjustment) / 100
}

/// Tourist role gets worse prices
pub fn tourist_price_adjustment(price: i32, is_tourist: bool) -> i32 {
    if is_tourist {
        price * 4 / 3 // 133% price for tourists
    } else {
        price
    }
}

/// Payment result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentResult {
    /// Payment successful
    Paid,
    /// Partial payment made
    Partial(i32),
    /// Cannot afford
    CannotAfford,
    /// Nothing to pay
    NothingOwed,
}

/// Attempt to pay shop debt
pub fn pay_debt(bill: &mut ShopBill, gold_available: i32) -> (PaymentResult, i32) {
    if bill.debt <= 0 {
        return (PaymentResult::NothingOwed, 0);
    }

    // First apply any credit
    if bill.credit > 0 {
        let credit_used = bill.credit.min(bill.debt);
        bill.credit -= credit_used;
        bill.debt -= credit_used;
    }

    if bill.debt <= 0 {
        bill.clear();
        return (PaymentResult::Paid, 0);
    }

    if gold_available <= 0 {
        return (PaymentResult::CannotAfford, bill.debt);
    }

    if gold_available >= bill.debt {
        let paid = bill.debt;
        bill.clear();
        (PaymentResult::Paid, paid)
    } else {
        bill.debt -= gold_available;
        (PaymentResult::Partial(gold_available), bill.debt)
    }
}

/// Shopkeeper anger levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopkeeperMood {
    /// Normal, peaceful
    Peaceful,
    /// Slightly annoyed (player has debt)
    Annoyed,
    /// Angry (player stole or attacked)
    Angry,
    /// Furious (major theft or violence)
    Furious,
}

impl ShopkeeperMood {
    /// Get description text
    pub fn description(&self) -> &'static str {
        match self {
            ShopkeeperMood::Peaceful => "seems pleased to see you",
            ShopkeeperMood::Annoyed => "looks at you suspiciously",
            ShopkeeperMood::Angry => "is quite upset",
            ShopkeeperMood::Furious => "is furious",
        }
    }
}

/// Determine shopkeeper mood based on player actions
pub fn shopkeeper_mood(debt: i32, stolen: bool, attacked: bool) -> ShopkeeperMood {
    if attacked {
        ShopkeeperMood::Furious
    } else if stolen {
        ShopkeeperMood::Angry
    } else if debt > 500 {
        ShopkeeperMood::Annoyed
    } else {
        ShopkeeperMood::Peaceful
    }
}

/// Generate a price quote message
pub fn price_quote(obj: &Object, price: i32) -> String {
    let name = obj.name.as_deref().unwrap_or("item");
    if obj.quantity > 1 {
        format!("{} {} for {} zorkmids", obj.quantity, name, price)
    } else {
        format!("{} for {} zorkmids", name, price)
    }
}

/// Generate a selling offer message
pub fn sell_offer(obj: &Object, price: i32) -> String {
    let name = obj.name.as_deref().unwrap_or("item");
    format!("I'll give you {} zorkmids for {}.", price, name)
}

/// Check if player is in a shop
pub fn in_shop(x: i8, y: i8, shop_bounds: Option<(i8, i8, i8, i8)>) -> bool {
    if let Some((x1, y1, x2, y2)) = shop_bounds {
        x >= x1 && x <= x2 && y >= y1 && y <= y2
    } else {
        false
    }
}

/// Identify value of gems based on shopkeeper interaction
pub fn identify_gem_value(obj: &Object, rng: &mut GameRng) -> Option<i32> {
    if obj.class != ObjectClass::Gem {
        return None;
    }

    // Shopkeeper identifies gems when you try to sell
    // Real gems have value, glass is worthless
    if obj.shop_price > 0 {
        Some(obj.shop_price)
    } else {
        // Worthless glass
        Some(rng.rnd(10) as i32) // Pretend value for glass
    }
}

/// Calculate wand price based on remaining charges (from shk.c cost_per_charge).
///
/// Wands lose value as charges deplete. A fully charged wand is worth base price,
/// while an empty wand is worth a fraction.
pub fn cost_per_charge(base_price: i32, charges: i32, max_charges: i32) -> i32 {
    if max_charges <= 0 {
        return base_price;
    }
    let charge_fraction = if charges <= 0 {
        // Empty wand: worth 1/3 of base
        base_price / 3
    } else {
        // Scale linearly with remaining charges
        base_price * charges / max_charges
    };
    charge_fraction.max(1)
}

/// Calculate total value of items inside a container.
///
/// Mirrors contained_cost() from shk.c: sums get_cost() for all items
/// in the given list.
pub fn contained_cost(items: &[Object], selling: bool) -> i32 {
    items.iter().map(|obj| get_cost(obj, selling)).sum()
}

/// Calculate robbery penalty (Kops summoning threshold)
pub fn robbery_penalty(stolen_value: i32) -> i32 {
    // Higher value = more Kops
    match stolen_value {
        0..=100 => 0,
        101..=500 => 1,
        501..=1000 => 2,
        1001..=5000 => 3,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::BucStatus;

    fn make_test_object(class: ObjectClass, price: i32) -> Object {
        let mut obj = Object::new(ObjectId(1), 0, class);
        obj.shop_price = price;
        obj.quantity = 1;
        obj
    }

    #[test]
    fn test_base_cost() {
        assert!(base_cost(ObjectClass::Amulet) > base_cost(ObjectClass::Food));
        assert!(base_cost(ObjectClass::Wand) > base_cost(ObjectClass::Weapon));
    }

    #[test]
    fn test_get_cost_basic() {
        let obj = make_test_object(ObjectClass::Weapon, 50);
        let buy_price = buying_price(&obj);
        let sell_price = selling_price(&obj);

        assert_eq!(buy_price, 50);
        assert_eq!(sell_price, 25); // 50% of buying price
    }

    #[test]
    fn test_get_cost_enchantment() {
        let mut obj = make_test_object(ObjectClass::Weapon, 50);
        obj.enchantment = 3;

        let price = buying_price(&obj);
        assert!(price > 50, "Enchanted items should cost more");
    }

    #[test]
    fn test_get_cost_cursed() {
        let mut obj = make_test_object(ObjectClass::Weapon, 100);
        obj.buc = BucStatus::Cursed;

        let price = buying_price(&obj);
        assert!(price < 100, "Cursed items should cost less");
    }

    #[test]
    fn test_get_cost_quantity() {
        let mut obj = make_test_object(ObjectClass::Potion, 50);
        obj.quantity = 3;

        let price = buying_price(&obj);
        assert_eq!(price, 150); // 50 * 3
    }

    #[test]
    fn test_charisma_adjustment() {
        let base_price = 100;

        let low_cha = charisma_price_adjustment(base_price, 5);
        let avg_cha = charisma_price_adjustment(base_price, 12);
        let high_cha = charisma_price_adjustment(base_price, 18);

        assert!(low_cha > avg_cha, "Low charisma should mean higher prices");
        assert!(avg_cha > high_cha, "High charisma should mean lower prices");
    }

    #[test]
    fn test_shop_bill() {
        let mut bill = ShopBill::new();

        bill.add_item(ObjectId(1), 50, 1);
        bill.add_item(ObjectId(2), 30, 2);

        assert_eq!(bill.total_debt(), 110); // 50 + 30*2
        assert!(bill.is_on_bill(ObjectId(1)));
        assert!(bill.is_on_bill(ObjectId(2)));

        bill.remove_item(ObjectId(1));
        assert_eq!(bill.total_debt(), 60);
        assert!(!bill.is_on_bill(ObjectId(1)));
    }

    #[test]
    fn test_pay_debt() {
        let mut bill = ShopBill::new();
        bill.add_item(ObjectId(1), 100, 1);

        // Partial payment
        let (result, remaining) = pay_debt(&mut bill, 40);
        assert_eq!(result, PaymentResult::Partial(40));
        assert_eq!(remaining, 60);

        // Full payment
        let (result, _) = pay_debt(&mut bill, 100);
        assert_eq!(result, PaymentResult::Paid);
        assert_eq!(bill.total_debt(), 0);
    }

    #[test]
    fn test_pay_debt_with_credit() {
        let mut bill = ShopBill::new();
        bill.add_item(ObjectId(1), 100, 1);
        bill.add_credit(30);

        let (result, _) = pay_debt(&mut bill, 70);
        assert_eq!(result, PaymentResult::Paid);
    }

    #[test]
    fn test_shopkeeper_mood() {
        assert_eq!(shopkeeper_mood(0, false, false), ShopkeeperMood::Peaceful);
        assert_eq!(shopkeeper_mood(1000, false, false), ShopkeeperMood::Annoyed);
        assert_eq!(shopkeeper_mood(0, true, false), ShopkeeperMood::Angry);
        assert_eq!(shopkeeper_mood(0, false, true), ShopkeeperMood::Furious);
    }

    #[test]
    fn test_robbery_penalty() {
        assert_eq!(robbery_penalty(50), 0);
        assert_eq!(robbery_penalty(200), 1);
        assert_eq!(robbery_penalty(800), 2);
        assert_eq!(robbery_penalty(10000), 4);
    }

    #[test]
    fn test_cost_per_charge() {
        // Full wand
        assert_eq!(cost_per_charge(100, 10, 10), 100);
        // Half charged
        assert_eq!(cost_per_charge(100, 5, 10), 50);
        // Empty wand
        assert_eq!(cost_per_charge(100, 0, 10), 33); // 100/3
        // No max charges
        assert_eq!(cost_per_charge(100, 0, 0), 100);
    }

    #[test]
    fn test_contained_cost() {
        let items = vec![
            make_test_object(ObjectClass::Potion, 50),
            make_test_object(ObjectClass::Scroll, 60),
        ];
        let total = contained_cost(&items, false);
        assert_eq!(total, 110);

        let sell_total = contained_cost(&items, true);
        assert_eq!(sell_total, 55); // 50% of buying
    }
}
