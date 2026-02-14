//! Shopkeeper system (shk.c)
//!
//! Handles shop mechanics, pricing, and shopkeeper interactions.
//!
//! Core systems:
//! - Shopkeeper creation and assignment
//! - Billing and debt tracking
//! - Movement and behavior AI
//! - Customer interaction and dialogue

use crate::action::ActionResult;
use crate::dungeon::Level;
use crate::gameloop::GameState;
use crate::monster::{Monster, MonsterId};
use crate::object::Object;
use crate::player::You;
use crate::rng::GameRng;

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

/// Individual bill entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BillEntry {
    /// Object ID on bill
    pub object_id: u32,
    /// Was the item used up?
    pub used_up: bool,
    /// Price per unit
    pub price: i32,
    /// Quantity on bill
    pub quantity: i32,
}

/// Shopkeeper extended data (equivalent to C struct eshk)
/// Stores all shopkeeper-specific state and billing information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShopkeeperExtension {
    /// Amount stolen - shopkeeper's demand
    pub robbed: i32,
    /// Available credit for customer
    pub credit: i32,
    /// Total debt for using unpaid items
    pub debit: i32,
    /// Shop-gold picked (part of debit)
    pub loan: i32,
    /// Shop type index
    pub shop_type: ShopType,
    /// Room index of the shop
    pub shop_room: u8,
    /// Is shopkeeper following unpaying customer?
    pub following: bool,
    /// Add surcharge when angry?
    pub surcharge: bool,
    /// Should dismiss Kops when pacified?
    pub dismiss_kops: bool,
    /// Usual shopkeeper position
    pub shop_pos: (i8, i8),
    /// Shop door position
    pub door_pos: (i8, i8),
    /// Shopkeeper's dungeon level
    pub shop_level: u8,
    /// Number of active bill entries
    pub bill_count: u32,
    /// Bill entries (up to 200 items)
    pub bills: Vec<BillEntry>,
    /// Number of visits by current customer
    pub visit_count: u32,
    /// Name of current customer being followed
    pub customer_name: String,
    /// Shopkeeper's name
    pub shop_name: String,
}

impl ShopkeeperExtension {
    /// Create new shopkeeper extension
    pub fn new(shop_type: ShopType, room: u8, pos: (i8, i8), door: (i8, i8)) -> Self {
        Self {
            robbed: 0,
            credit: 0,
            debit: 0,
            loan: 0,
            shop_type,
            shop_room: room,
            following: false,
            surcharge: false,
            dismiss_kops: false,
            shop_pos: pos,
            door_pos: door,
            shop_level: 1,
            bill_count: 0,
            bills: Vec::new(),
            visit_count: 0,
            customer_name: String::new(),
            shop_name: String::new(),
        }
    }

    /// Calculate total bill amount
    pub fn calculate_total_bill(&self) -> i32 {
        self.bills.iter().map(|b| b.price * b.quantity).sum()
    }

    /// Add item to bill
    pub fn bill_item(&mut self, obj_id: u32, price: i32, quantity: i32) {
        if let Some(entry) = self.bills.iter_mut().find(|b| b.object_id == obj_id) {
            entry.quantity += quantity;
        } else {
            self.bills.push(BillEntry {
                object_id: obj_id,
                used_up: false,
                price,
                quantity,
            });
            self.bill_count = self.bills.len() as u32;
        }
    }

    /// Mark item on bill as used up
    pub fn mark_used_up(&mut self, obj_id: u32) {
        if let Some(entry) = self.bills.iter_mut().find(|b| b.object_id == obj_id) {
            entry.used_up = true;
        }
    }

    /// Clear all paid items from bill
    pub fn clear_paid_items(&mut self) {
        self.bills.clear();
        self.bill_count = 0;
        self.debit = 0;
    }
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
        (x == x1 || x == x2 || y == y1 || y == y2) && self.contains(x, y)
    }
}

/// Check if a monster is a shopkeeper
pub fn is_shopkeeper(monster: &Monster) -> bool {
    monster.is_shopkeeper
}

/// Get shopkeeper extension if monster is shopkeeper
pub fn get_shopkeeper_ext(monster: &Monster) -> Option<&ShopkeeperExtension> {
    if monster.is_shopkeeper {
        monster.shopkeeper_extension.as_ref()
    } else {
        None
    }
}

/// Get mutable shopkeeper extension
pub fn get_shopkeeper_ext_mut(monster: &mut Monster) -> Option<&mut ShopkeeperExtension> {
    if monster.is_shopkeeper {
        monster.shopkeeper_extension.as_mut()
    } else {
        None
    }
}

/// Create shopkeeper extension (neweshk equivalent)
pub fn create_shopkeeper_extension(
    monster: &mut Monster,
    shop_type: ShopType,
    room: u8,
    pos: (i8, i8),
    door: (i8, i8),
) {
    monster.is_shopkeeper = true;
    monster.shopkeeper_extension = Some(ShopkeeperExtension::new(shop_type, room, pos, door));
}

/// Find shopkeeper in a level for a given room (findshk/shop_keeper equivalent)
pub fn find_room_shopkeeper(level: &Level, room_num: u8) -> Option<MonsterId> {
    for monster in &level.monsters {
        if monster.is_shopkeeper {
            if let Some(ext) = get_shopkeeper_ext(monster) {
                if ext.shop_room == room_num {
                    return Some(monster.id);
                }
            }
        }
    }
    None
}

/// Check if shopkeeper is angry (ANGRY macro equivalent)
pub fn is_angry_shopkeeper(monster: &Monster) -> bool {
    if let Some(ext) = get_shopkeeper_ext(monster) {
        ext.surcharge
    } else {
        false
    }
}

/// Make shopkeeper angry and apply surcharge (rile_shk equivalent)
pub fn anger_shopkeeper(monster: &mut Monster) {
    if let Some(ext) = get_shopkeeper_ext_mut(monster) {
        ext.surcharge = true;
        monster.state.peaceful = false;
    }
}

/// Pacify shopkeeper and clear surcharge (pacify_shk equivalent)
pub fn pacify_shopkeeper(monster: &mut Monster) {
    if let Some(ext) = get_shopkeeper_ext_mut(monster) {
        ext.surcharge = false;
        ext.dismiss_kops = true; // Mark to dismiss summoned Kops
        monster.state.peaceful = true;
    }
}

/// Move shopkeeper toward shop (shk_move equivalent)
pub fn move_shopkeeper_to_shop(shopkeeper: &mut Monster, level: &Level, player: &You) -> bool {
    if let Some(ext) = get_shopkeeper_ext_mut(shopkeeper) {
        // Calculate distance to shop
        let shop_x = ext.shop_pos.0;
        let shop_y = ext.shop_pos.1;
        let dx = (shop_x - shopkeeper.x).signum();
        let dy = (shop_y - shopkeeper.y).signum();

        let new_x = shopkeeper.x + dx;
        let new_y = shopkeeper.y + dy;

        // Check if position is walkable
        if level.is_valid_pos(new_x, new_y) {
            shopkeeper.x = new_x;
            shopkeeper.y = new_y;
            return true;
        }
    }
    false
}

/// Handle shopkeeper post-move cleanup (after_shk_move equivalent)
pub fn handle_shopkeeper_move(shopkeeper: &mut Monster, _level: &Level) {
    // Read position first to avoid borrow issues
    let (sk_x, sk_y) = (shopkeeper.x, shopkeeper.y);

    if let Some(ext) = get_shopkeeper_ext_mut(shopkeeper) {
        // If shopkeeper re-entered shop and billing pointer needs reset
        if sk_x == ext.shop_pos.0 && sk_y == ext.shop_pos.1 {
            // Billing pointer reset would happen here
        }
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
        0..=5 => 200,   // 200% of base
        6..=8 => 175,   // 175%
        9..=12 => 150,  // 150%
        13..=15 => 125, // 125%
        16..=18 => 110, // 110%
        _ => 100,       // 100% for very high charisma
    };

    (base * cha_modifier / 100).max(1)
}

/// Handle player picking up an item in a shop
pub fn pickup_in_shop(state: &mut GameState, obj: &Object, shop: &mut Shop) {
    let price = buying_price(
        obj,
        state
            .player
            .attr_current
            .get(crate::player::Attribute::Charisma),
    );
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

    let price = selling_price(
        &obj,
        state
            .player
            .attr_current
            .get(crate::player::Attribute::Charisma),
    );

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

/// Handle shopkeeper conversation (shk_chat equivalent)
pub fn shopkeeper_chat(shopkeeper: &Monster, level: &Level) -> String {
    let mut response = String::new();

    if let Some(ext) = get_shopkeeper_ext(shopkeeper) {
        // Check shopkeeper status
        if is_angry_shopkeeper(shopkeeper) {
            response.push_str(&format!(
                "\"You owe me {} zorkmids! Pay up or get out!\"",
                ext.debit + ext.robbed
            ));
        } else if ext.following {
            response.push_str(&format!(
                "\"{}! I'm still waiting for my money!\"",
                ext.customer_name
            ));
        } else if ext.bill_count > 0 {
            response.push_str(&format!(
                "\"Your current bill is {} zorkmids.\"",
                ext.calculate_total_bill()
            ));
        } else if ext.robbed > 0 {
            response.push_str(&format!(
                "\"I know you stole {} zorkmids from me!\"",
                ext.robbed
            ));
        } else {
            response.push_str("\"How can I help you today?\"");
        }
    }

    response
}

/// Player trying to pay shopkeeper debt (dopay equivalent)
pub fn pay_shopkeeper(shopkeeper: &mut Monster, level: &Level, player: &mut You) -> bool {
    if let Some(ext) = get_shopkeeper_ext_mut(shopkeeper) {
        let total_debt = ext.debit + ext.robbed;

        if total_debt == 0 {
            return false;
        }

        if player.gold >= total_debt {
            player.gold -= total_debt;
            ext.debit = 0;
            ext.robbed = 0;
            ext.surcharge = false;
            shopkeeper.state.peaceful = true;
            return true;
        } else if player.gold > 0 {
            // Partial payment
            let payment = player.gold;
            player.gold = 0;

            let remaining = total_debt - payment;
            if remaining < 0 {
                ext.debit = 0;
                ext.robbed = 0;
            } else if ext.robbed > 0 {
                ext.robbed -= payment.min(ext.robbed);
                if ext.robbed == 0 {
                    ext.debit -= remaining;
                }
            } else {
                ext.debit -= payment;
            }

            return true;
        }
    }

    false
}

/// Get shopkeeper's display name with proper formatting (shkname equivalent)
pub fn get_shopkeeper_name(shopkeeper: &Monster) -> String {
    if let Some(ext) = get_shopkeeper_ext(shopkeeper) {
        if !ext.shop_name.is_empty() {
            ext.shop_name.clone()
        } else {
            shopkeeper.name.clone()
        }
    } else {
        shopkeeper.name.clone()
    }
}

/// Make shopkeeper happy with payment (make_happy_shopkeeper equivalent)
pub fn make_happy_shopkeeper(shopkeeper: &mut Monster) {
    if let Some(ext) = get_shopkeeper_ext_mut(shopkeeper) {
        ext.surcharge = false;
        ext.following = false;
        ext.robbed = 0;
    }
    shopkeeper.state.peaceful = true;
}

/// Check if player is in shop (inhishop equivalent)
pub fn is_in_shop(shopkeeper: &Monster, level: &Level, player_x: i8, player_y: i8) -> bool {
    if let Some(ext) = get_shopkeeper_ext(shopkeeper) {
        // Simple check: within bounds of shop
        let (x1, y1, x2, y2) = (
            ext.shop_pos.0 - 5,
            ext.shop_pos.1 - 5,
            ext.shop_pos.0 + 5,
            ext.shop_pos.1 + 5,
        );
        player_x >= x1 && player_x <= x2 && player_y >= y1 && player_y <= y2
    } else {
        false
    }
}

/// Check if shop is tended (has a living shopkeeper) - tended_shop equivalent
pub fn tended_shop(level: &Level, shop: &Shop) -> bool {
    if let Some(keeper_id) = shop.shopkeeper_id {
        level.monsters.iter().any(|m| m.id == keeper_id && m.hp > 0)
    } else {
        false
    }
}

/// Check if shop is deserted (no shopkeeper) - deserted_shop equivalent
pub fn deserted_shop(level: &Level, shop: &Shop) -> bool {
    !tended_shop(level, shop)
}

/// Check if a position is inside a given shop - inside_shop equivalent
pub fn inside_shop(shop: &Shop, x: i8, y: i8) -> bool {
    shop.contains(x, y)
}

/// Find which shop (if any) contains a position from a list of shops
pub fn find_shop_at<'a>(shops: &'a [Shop], x: i8, y: i8) -> Option<&'a Shop> {
    shops.iter().find(|shop| shop.contains(x, y))
}

/// Check if shopkeeper is Izchak (special shopkeeper in Minetown)
pub fn is_izchak(shopkeeper: &Monster) -> bool {
    if let Some(ext) = get_shopkeeper_ext(shopkeeper) {
        ext.shop_name == "Izchak" || shopkeeper.name == "Izchak"
    } else {
        false
    }
}

/// Check if object is on shopkeeper's bill - onbill equivalent
pub fn on_bill(shopkeeper: &Monster, obj_id: u32) -> bool {
    if let Some(ext) = get_shopkeeper_ext(shopkeeper) {
        ext.bills.iter().any(|b| b.object_id == obj_id)
    } else {
        false
    }
}

/// Calculate total bill amount - addupbill equivalent
pub fn add_up_bill(shopkeeper: &Monster) -> i32 {
    if let Some(ext) = get_shopkeeper_ext(shopkeeper) {
        ext.calculate_total_bill()
    } else {
        0
    }
}

/// Check if item can be sold to shopkeeper - saleable equivalent
pub fn saleable(shopkeeper: &Monster, obj: &Object) -> bool {
    // Can't sell gold
    if obj.class == crate::object::ObjectClass::Coin {
        return false;
    }

    // Can't sell artifacts
    if obj.artifact > 0 {
        return false;
    }

    // Can't sell unpaid items (they belong to the shop)
    if obj.unpaid {
        return false;
    }

    // Check if shop accepts this object class
    if let Some(ext) = get_shopkeeper_ext(shopkeeper) {
        match ext.shop_type {
            ShopType::General => true, // Accepts everything
            ShopType::Armor => matches!(obj.class, crate::object::ObjectClass::Armor),
            ShopType::Weapon => matches!(obj.class, crate::object::ObjectClass::Weapon),
            ShopType::Food => matches!(obj.class, crate::object::ObjectClass::Food),
            ShopType::Scroll => matches!(obj.class, crate::object::ObjectClass::Scroll),
            ShopType::Potion => matches!(obj.class, crate::object::ObjectClass::Potion),
            ShopType::Wand => matches!(obj.class, crate::object::ObjectClass::Wand),
            ShopType::Tool => matches!(obj.class, crate::object::ObjectClass::Tool),
            ShopType::Book => matches!(obj.class, crate::object::ObjectClass::Spellbook),
            ShopType::Ring => matches!(
                obj.class,
                crate::object::ObjectClass::Ring | crate::object::ObjectClass::Amulet
            ),
            ShopType::Candle => matches!(obj.class, crate::object::ObjectClass::Tool), // Candles are tools
            ShopType::Tin => matches!(obj.class, crate::object::ObjectClass::Food),
        }
    } else {
        false
    }
}

/// Calculate value of stolen items - stolen_value equivalent
pub fn stolen_value(obj: &Object, shopkeeper: &Monster) -> i32 {
    let base = base_price(obj);

    // Stolen items valued at 150% of base price (shopkeeper's loss)
    let theft_multiplier = 150;

    (base * theft_multiplier / 100) * obj.quantity
}

/// Handle player digging in shop - shopdig equivalent
pub fn shop_dig(shopkeeper: &mut Monster, _level: &Level, x: i8, y: i8) {
    // Check if digging location is in shop bounds
    let in_bounds = if let Some(ext) = get_shopkeeper_ext(shopkeeper) {
        x >= ext.shop_pos.0 - 5
            && x <= ext.shop_pos.0 + 5
            && y >= ext.shop_pos.1 - 5
            && y <= ext.shop_pos.1 + 5
    } else {
        false
    };

    if in_bounds {
        // Shopkeeper gets angry about property damage
        anger_shopkeeper(shopkeeper);
    }
}

/// Wake up shopkeeper - rouse_shk equivalent
pub fn rouse_shopkeeper(shopkeeper: &mut Monster) {
    shopkeeper.state.sleeping = false;
    shopkeeper.state.paralyzed = false;
}

/// Put shopkeeper in hot pursuit mode - hot_pursuit equivalent
pub fn hot_pursuit(shopkeeper: &mut Monster, customer_name: &str) {
    if let Some(ext) = get_shopkeeper_ext_mut(shopkeeper) {
        ext.following = true;
        ext.customer_name = customer_name.to_string();
    }
}

/// Remove object from bill - subfrombill equivalent
pub fn sub_from_bill(shopkeeper: &mut Monster, obj_id: u32) -> bool {
    if let Some(ext) = get_shopkeeper_ext_mut(shopkeeper) {
        if let Some(idx) = ext.bills.iter().position(|b| b.object_id == obj_id) {
            ext.bills.remove(idx);
            ext.bill_count = ext.bills.len() as u32;
            return true;
        }
    }
    false
}

/// Split bill entry for partial quantity - splitbill equivalent
pub fn split_bill(shopkeeper: &mut Monster, obj_id: u32, new_quantity: i32) -> bool {
    if let Some(ext) = get_shopkeeper_ext_mut(shopkeeper) {
        if let Some(entry) = ext.bills.iter_mut().find(|b| b.object_id == obj_id) {
            if entry.quantity > new_quantity {
                entry.quantity = new_quantity;
                return true;
            }
        }
    }
    false
}

/// Clear unpaid flag from object - clear_unpaid_obj equivalent
pub fn clear_unpaid_obj(obj: &mut Object) {
    obj.unpaid = false;
}

/// Count unpaid objects in inventory - count_unpaid equivalent
pub fn count_unpaid(inventory: &[Object]) -> i32 {
    inventory.iter().filter(|obj| obj.unpaid).count() as i32
}

/// Find first unpaid object - find_unpaid equivalent
pub fn find_unpaid(inventory: &[Object]) -> Option<&Object> {
    inventory.iter().find(|obj| obj.unpaid)
}

/// Check if two items have same shop price - same_price equivalent
pub fn same_price(obj1: &Object, obj2: &Object) -> bool {
    base_price(obj1) == base_price(obj2)
}

/// Get the cheapest item in a list - cheapest_item equivalent
pub fn cheapest_item<'a>(items: &'a [Object]) -> Option<&'a Object> {
    items.iter().min_by_key(|obj| base_price(obj))
}

/// Generate a price quote string - price_quote equivalent
pub fn price_quote_str(obj: &Object, charisma: i8) -> String {
    let price = buying_price(obj, charisma);
    let name = obj.name.as_deref().unwrap_or("item");

    if obj.quantity > 1 {
        format!("{} {} for {} zorkmids", obj.quantity, name, price)
    } else {
        format!("{} for {} zorkmids", name, price)
    }
}

/// Check if shopkeeper owns object - shk_owns equivalent
pub fn shopkeeper_owns(shopkeeper: &Monster, obj: &Object, shop: &Shop) -> bool {
    // Object is in shop bounds and marked as shop inventory
    obj.unpaid && shop.contains(obj.x, obj.y)
}

/// Handle player entering shop - u_entered_shop equivalent
pub fn player_entered_shop(shopkeeper: &mut Monster, player_name: &str) {
    if let Some(ext) = get_shopkeeper_ext_mut(shopkeeper) {
        ext.visit_count += 1;
        ext.customer_name = player_name.to_string();
    }
}

/// Handle player leaving shop - u_left_shop equivalent
pub fn player_left_shop(shopkeeper: &mut Monster, shop: &mut Shop) -> bool {
    if shop.debt > 0 {
        // Player leaving with unpaid items - shopkeeper pursues
        hot_pursuit(shopkeeper, "thief");
        return false;
    }
    true
}

/// Get shop debt amount - shop_debt equivalent
pub fn get_shop_debt(shopkeeper: &Monster) -> i32 {
    if let Some(ext) = get_shopkeeper_ext(shopkeeper) {
        ext.debit + ext.robbed
    } else {
        0
    }
}

/// Add to robbery amount - rob_shop equivalent
pub fn rob_shop(shopkeeper: &mut Monster, amount: i32) {
    if let Some(ext) = get_shopkeeper_ext_mut(shopkeeper) {
        ext.robbed += amount;
        ext.surcharge = true;
    }
    shopkeeper.state.peaceful = false;
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

    // ========== EXPANDED TEST COVERAGE ==========

    #[test]
    fn test_bill_entry_creation() {
        let entry = BillEntry {
            object_id: 1,
            used_up: false,
            price: 100,
            quantity: 5,
        };

        assert_eq!(entry.object_id, 1);
        assert!(!entry.used_up);
        assert_eq!(entry.price, 100);
        assert_eq!(entry.quantity, 5);
    }

    #[test]
    fn test_bill_entry_used_up() {
        let mut entry = BillEntry {
            object_id: 1,
            used_up: false,
            price: 100,
            quantity: 5,
        };

        assert!(!entry.used_up);
        entry.used_up = true;
        assert!(entry.used_up);
    }

    #[test]
    fn test_shopkeeper_extension_new() {
        let ext = ShopkeeperExtension {
            robbed: 0,
            credit: 0,
            debit: 0,
            loan: 0,
            shop_type: ShopType::General,
            shop_room: 1,
            following: false,
            surcharge: false,
            dismiss_kops: false,
            shop_pos: (10, 10),
            door_pos: (15, 15),
            shop_level: 0,
            bill_count: 0,
            bills: Vec::new(),
            visit_count: 0,
            customer_name: String::new(),
            shop_name: String::new(),
        };

        assert_eq!(ext.robbed, 0);
        assert_eq!(ext.credit, 0);
        assert_eq!(ext.debit, 0);
        assert!(!ext.following);
        assert_eq!(ext.bills.len(), 0);
    }

    #[test]
    fn test_shopkeeper_extension_add_bill() {
        let mut ext = ShopkeeperExtension {
            robbed: 0,
            credit: 0,
            debit: 0,
            loan: 0,
            shop_type: ShopType::General,
            shop_room: 1,
            following: false,
            surcharge: false,
            dismiss_kops: false,
            shop_pos: (10, 10),
            door_pos: (15, 15),
            shop_level: 0,
            bill_count: 0,
            bills: Vec::new(),
            visit_count: 0,
            customer_name: String::new(),
            shop_name: String::new(),
        };

        ext.bills.push(BillEntry {
            object_id: 1,
            used_up: false,
            price: 100,
            quantity: 1,
        });

        assert_eq!(ext.bills.len(), 1);
        assert_eq!(ext.bills[0].object_id, 1);
    }

    #[test]
    fn test_base_price_positive() {
        let obj = Object::new(ObjectId(1), 0, ObjectClass::Potion);
        let price = base_price(&obj);
        assert!(price > 0);
    }

    #[test]
    fn test_buying_price_less_than_base() {
        let obj = Object::new(ObjectId(1), 0, ObjectClass::Potion);
        let base = base_price(&obj);
        let buy = buying_price(&obj, 10);
        // Buying price includes charisma markup (150% at cha 10)
        assert!(buy >= base);
    }

    #[test]
    fn test_selling_price_less_than_base() {
        let obj = Object::new(ObjectId(1), 0, ObjectClass::Potion);
        let base = base_price(&obj);
        let sell = selling_price(&obj, 10);
        assert!(sell < base); // Selling gets less than base value
    }

    #[test]
    fn test_high_charisma_buys_cheaper() {
        let obj = Object::new(ObjectId(1), 0, ObjectClass::Armor);

        let low_cha = buying_price(&obj, 3);
        let high_cha = buying_price(&obj, 18);

        // Higher charisma should result in lower buying price
        assert!(high_cha <= low_cha);
    }

    #[test]
    fn test_high_charisma_sells_for_more() {
        let obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);

        let low_cha = selling_price(&obj, 3);
        let high_cha = selling_price(&obj, 18);

        // Higher charisma should result in higher selling price
        assert!(high_cha >= low_cha);
    }

    #[test]
    fn test_shop_types() {
        let _general = ShopType::General;
        let _armor = ShopType::Armor;
        let _weapon = ShopType::Weapon;
        let _food = ShopType::Food;
        let _scroll = ShopType::Scroll;
        let _potion = ShopType::Potion;
        let _wand = ShopType::Wand;
        let _tool = ShopType::Tool;
        let _book = ShopType::Book;
        let _ring = ShopType::Ring;
        let _candle = ShopType::Candle;
        let _tin = ShopType::Tin;
    }

    #[test]
    fn test_shop_bounds() {
        let shop = Shop::new(ShopType::Weapon, (5, 5, 15, 15));

        assert!(shop.contains(10, 10)); // Center
        assert!(shop.contains(5, 5)); // Corner
        assert!(shop.contains(15, 15)); // Opposite corner
        assert!(!shop.contains(4, 10)); // Just outside
        assert!(!shop.contains(16, 10)); // Just outside
    }

    #[test]
    fn test_shop_boundaries_edge_cases() {
        let shop = Shop::new(ShopType::Food, (10, 10, 20, 20));

        // On edges
        assert!(shop.contains(10, 10));
        assert!(shop.contains(10, 20));
        assert!(shop.contains(20, 10));
        assert!(shop.contains(20, 20));

        // Just outside
        assert!(!shop.contains(9, 15));
        assert!(!shop.contains(21, 15));
        assert!(!shop.contains(15, 9));
        assert!(!shop.contains(15, 21));
    }

    #[test]
    fn test_shopkeeper_extension_location() {
        let ext = ShopkeeperExtension {
            robbed: 0,
            credit: 0,
            debit: 0,
            loan: 0,
            shop_type: ShopType::Scroll,
            shop_room: 5,
            following: false,
            surcharge: false,
            dismiss_kops: false,
            shop_pos: (20, 20),
            door_pos: (25, 25),
            shop_level: 3,
            bill_count: 0,
            bills: Vec::new(),
            visit_count: 1,
            customer_name: "Player".to_string(),
            shop_name: "Wand Shop".to_string(),
        };

        assert_eq!(ext.shop_pos, (20, 20));
        assert_eq!(ext.door_pos, (25, 25));
    }

    #[test]
    fn test_shopkeeper_debt_tracking() {
        let mut ext = ShopkeeperExtension {
            robbed: 0,
            credit: 0,
            debit: 100,
            loan: 50,
            shop_type: ShopType::Armor,
            shop_room: 2,
            following: false,
            surcharge: false,
            dismiss_kops: false,
            shop_pos: (10, 10),
            door_pos: (15, 15),
            shop_level: 1,
            bill_count: 0,
            bills: Vec::new(),
            visit_count: 0,
            customer_name: String::new(),
            shop_name: String::new(),
        };

        // Debt increases
        assert_eq!(ext.debit, 100);
        assert_eq!(ext.loan, 50);

        // Pay some debt
        ext.debit -= 30;
        assert_eq!(ext.debit, 70);
    }

    #[test]
    fn test_multiple_bills() {
        let mut ext = ShopkeeperExtension {
            robbed: 0,
            credit: 0,
            debit: 0,
            loan: 0,
            shop_type: ShopType::General,
            shop_room: 1,
            following: false,
            surcharge: false,
            dismiss_kops: false,
            shop_pos: (10, 10),
            door_pos: (15, 15),
            shop_level: 0,
            bill_count: 0,
            bills: Vec::new(),
            visit_count: 0,
            customer_name: String::new(),
            shop_name: String::new(),
        };

        // Add multiple bills
        for i in 1..=5 {
            ext.bills.push(BillEntry {
                object_id: i,
                used_up: false,
                price: 100 * i as i32,
                quantity: 1,
            });
        }

        assert_eq!(ext.bills.len(), 5);
        assert_eq!(ext.bills[0].price, 100);
        assert_eq!(ext.bills[4].price, 500);
    }

    #[test]
    fn test_bill_total_calculation() {
        let bills = vec![
            BillEntry {
                object_id: 1,
                used_up: false,
                price: 100,
                quantity: 1,
            },
            BillEntry {
                object_id: 2,
                used_up: false,
                price: 200,
                quantity: 2,
            },
            BillEntry {
                object_id: 3,
                used_up: true,
                price: 50,
                quantity: 1,
            },
        ];

        let total: i32 = bills.iter().map(|b| b.price * b.quantity).sum();
        assert_eq!(total, 100 + 400 + 50);
    }

    #[test]
    fn test_price_scaling_with_charisma() {
        let obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);

        let cha3_buy = buying_price(&obj, 3);
        let cha9_buy = buying_price(&obj, 9);
        let cha15_buy = buying_price(&obj, 15);
        let cha18_buy = buying_price(&obj, 18);

        // Each increase in charisma should improve buying price (lower)
        assert!(cha18_buy <= cha15_buy);
        assert!(cha15_buy <= cha9_buy);
        assert!(cha9_buy <= cha3_buy);
    }

    #[test]
    fn test_shopkeeper_following_state() {
        let mut ext = ShopkeeperExtension {
            robbed: 0,
            credit: 0,
            debit: 0,
            loan: 0,
            shop_type: ShopType::General,
            shop_room: 1,
            following: false,
            surcharge: false,
            dismiss_kops: false,
            shop_pos: (10, 10),
            door_pos: (15, 15),
            shop_level: 0,
            bill_count: 0,
            bills: Vec::new(),
            visit_count: 0,
            customer_name: String::new(),
            shop_name: String::new(),
        };

        assert!(!ext.following);
        ext.following = true;
        assert!(ext.following);
    }

    #[test]
    fn test_shopkeeper_visit_count() {
        let mut ext = ShopkeeperExtension {
            robbed: 0,
            credit: 0,
            debit: 0,
            loan: 0,
            shop_type: ShopType::General,
            shop_room: 1,
            following: false,
            surcharge: false,
            dismiss_kops: false,
            shop_pos: (10, 10),
            door_pos: (15, 15),
            shop_level: 0,
            bill_count: 0,
            bills: Vec::new(),
            visit_count: 0,
            customer_name: String::new(),
            shop_name: String::new(),
        };

        ext.visit_count += 1;
        assert_eq!(ext.visit_count, 1);

        ext.visit_count += 1;
        assert_eq!(ext.visit_count, 2);
    }

    // ========== Tests for new shop functions ==========

    #[test]
    fn test_inside_shop() {
        let shop = Shop::new(ShopType::General, (10, 10, 20, 20));

        assert!(inside_shop(&shop, 15, 15));
        assert!(inside_shop(&shop, 10, 10));
        assert!(!inside_shop(&shop, 5, 5));
    }

    #[test]
    fn test_find_shop_at() {
        let shops = vec![
            Shop::new(ShopType::Armor, (5, 5, 10, 10)),
            Shop::new(ShopType::Weapon, (20, 20, 30, 30)),
        ];

        let found = find_shop_at(&shops, 8, 8);
        assert!(found.is_some());
        assert_eq!(found.unwrap().shop_type, ShopType::Armor);

        let found2 = find_shop_at(&shops, 25, 25);
        assert!(found2.is_some());
        assert_eq!(found2.unwrap().shop_type, ShopType::Weapon);

        let not_found = find_shop_at(&shops, 15, 15);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_is_izchak() {
        let mut monster = Monster::new(MonsterId(1), 5, 10, 10);
        monster.is_shopkeeper = true;
        monster.shopkeeper_extension = Some(ShopkeeperExtension::new(
            ShopType::General,
            1,
            (10, 10),
            (15, 15),
        ));

        // Not Izchak by default
        assert!(!is_izchak(&monster));

        // Set name to Izchak
        if let Some(ext) = monster.shopkeeper_extension.as_mut() {
            ext.shop_name = "Izchak".to_string();
        }
        assert!(is_izchak(&monster));
    }

    #[test]
    fn test_saleable() {
        let mut monster = Monster::new(MonsterId(1), 5, 10, 10);
        monster.is_shopkeeper = true;
        monster.shopkeeper_extension = Some(ShopkeeperExtension::new(
            ShopType::Armor,
            1,
            (10, 10),
            (15, 15),
        ));

        // Armor shop accepts armor
        let armor = Object::new(ObjectId(1), 0, ObjectClass::Armor);
        assert!(saleable(&monster, &armor));

        // Armor shop doesn't accept weapons
        let weapon = Object::new(ObjectId(2), 0, ObjectClass::Weapon);
        assert!(!saleable(&monster, &weapon));

        // Can't sell unpaid items
        let mut unpaid = Object::new(ObjectId(3), 0, ObjectClass::Armor);
        unpaid.unpaid = true;
        assert!(!saleable(&monster, &unpaid));
    }

    #[test]
    fn test_on_bill() {
        let mut monster = Monster::new(MonsterId(1), 5, 10, 10);
        monster.is_shopkeeper = true;
        monster.shopkeeper_extension = Some(ShopkeeperExtension::new(
            ShopType::General,
            1,
            (10, 10),
            (15, 15),
        ));

        // Add an item to the bill
        if let Some(ext) = monster.shopkeeper_extension.as_mut() {
            ext.bill_item(123, 100, 1);
        }

        assert!(on_bill(&monster, 123));
        assert!(!on_bill(&monster, 456));
    }

    #[test]
    fn test_add_up_bill() {
        let mut monster = Monster::new(MonsterId(1), 5, 10, 10);
        monster.is_shopkeeper = true;
        monster.shopkeeper_extension = Some(ShopkeeperExtension::new(
            ShopType::General,
            1,
            (10, 10),
            (15, 15),
        ));

        if let Some(ext) = monster.shopkeeper_extension.as_mut() {
            ext.bill_item(1, 100, 1);
            ext.bill_item(2, 50, 2);
        }

        assert_eq!(add_up_bill(&monster), 200); // 100 + 50*2
    }

    #[test]
    fn test_sub_from_bill() {
        let mut monster = Monster::new(MonsterId(1), 5, 10, 10);
        monster.is_shopkeeper = true;
        monster.shopkeeper_extension = Some(ShopkeeperExtension::new(
            ShopType::General,
            1,
            (10, 10),
            (15, 15),
        ));

        if let Some(ext) = monster.shopkeeper_extension.as_mut() {
            ext.bill_item(123, 100, 1);
        }

        assert!(on_bill(&monster, 123));
        assert!(sub_from_bill(&mut monster, 123));
        assert!(!on_bill(&monster, 123));
    }

    #[test]
    fn test_count_unpaid() {
        let mut obj1 = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        obj1.unpaid = true;
        let mut obj2 = Object::new(ObjectId(2), 0, ObjectClass::Armor);
        obj2.unpaid = true;
        let obj3 = Object::new(ObjectId(3), 0, ObjectClass::Food);

        let inventory = vec![obj1, obj2, obj3];
        assert_eq!(count_unpaid(&inventory), 2);
    }

    #[test]
    fn test_find_unpaid() {
        let obj1 = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        let mut obj2 = Object::new(ObjectId(2), 0, ObjectClass::Armor);
        obj2.unpaid = true;

        let inventory = vec![obj1, obj2];
        let found = find_unpaid(&inventory);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, ObjectId(2));
    }

    #[test]
    fn test_same_price() {
        let mut obj1 = Object::new(ObjectId(1), 0, ObjectClass::Potion);
        obj1.shop_price = 100;
        let mut obj2 = Object::new(ObjectId(2), 0, ObjectClass::Potion);
        obj2.shop_price = 100;
        let mut obj3 = Object::new(ObjectId(3), 0, ObjectClass::Potion);
        obj3.shop_price = 50;

        assert!(same_price(&obj1, &obj2));
        assert!(!same_price(&obj1, &obj3));
    }

    #[test]
    fn test_cheapest_item() {
        let mut obj1 = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        obj1.shop_price = 100;
        let mut obj2 = Object::new(ObjectId(2), 0, ObjectClass::Armor);
        obj2.shop_price = 50;
        let mut obj3 = Object::new(ObjectId(3), 0, ObjectClass::Ring);
        obj3.shop_price = 200;

        let items = vec![obj1, obj2, obj3];
        let cheapest = cheapest_item(&items);
        assert!(cheapest.is_some());
        assert_eq!(cheapest.unwrap().id, ObjectId(2));
    }

    #[test]
    fn test_price_quote_str() {
        let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        obj.shop_price = 100;
        obj.name = Some("sword".to_string());

        let quote = price_quote_str(&obj, 10);
        assert!(quote.contains("sword"));
        assert!(quote.contains("zorkmids"));
    }

    #[test]
    fn test_hot_pursuit() {
        let mut monster = Monster::new(MonsterId(1), 5, 10, 10);
        monster.is_shopkeeper = true;
        monster.shopkeeper_extension = Some(ShopkeeperExtension::new(
            ShopType::General,
            1,
            (10, 10),
            (15, 15),
        ));

        hot_pursuit(&mut monster, "PlayerName");

        if let Some(ext) = get_shopkeeper_ext(&monster) {
            assert!(ext.following);
            assert_eq!(ext.customer_name, "PlayerName");
        }
    }

    #[test]
    fn test_get_shop_debt() {
        let mut monster = Monster::new(MonsterId(1), 5, 10, 10);
        monster.is_shopkeeper = true;
        monster.shopkeeper_extension = Some(ShopkeeperExtension::new(
            ShopType::General,
            1,
            (10, 10),
            (15, 15),
        ));

        if let Some(ext) = monster.shopkeeper_extension.as_mut() {
            ext.debit = 100;
            ext.robbed = 50;
        }

        assert_eq!(get_shop_debt(&monster), 150);
    }

    #[test]
    fn test_rob_shop() {
        let mut monster = Monster::new(MonsterId(1), 5, 10, 10);
        monster.is_shopkeeper = true;
        monster.state.peaceful = true;
        monster.shopkeeper_extension = Some(ShopkeeperExtension::new(
            ShopType::General,
            1,
            (10, 10),
            (15, 15),
        ));

        rob_shop(&mut monster, 500);

        assert!(!monster.state.peaceful);
        if let Some(ext) = get_shopkeeper_ext(&monster) {
            assert_eq!(ext.robbed, 500);
            assert!(ext.surcharge);
        }
    }
}
