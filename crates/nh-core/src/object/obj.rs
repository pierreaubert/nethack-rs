//! Object instances (obj.h)

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use super::objname::makeplural;
use super::ObjectClass;

/// Unique identifier for object instances
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub u32);

impl ObjectId {
    pub const NONE: ObjectId = ObjectId(0);

    pub fn next(self) -> Self {
        ObjectId(self.0 + 1)
    }
}

/// Where the object is located
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum ObjectLocation {
    /// Not in game
    #[default]
    Free = 0,
    /// On the floor
    Floor = 1,
    /// Inside a container
    Contained = 2,
    /// In player inventory
    PlayerInventory = 3,
    /// In monster inventory
    MonsterInventory = 4,
    /// Moving between levels
    Migrating = 5,
    /// Buried in ground
    Buried = 6,
    /// On shopkeeper bill
    OnBill = 7,
}

/// BUC (blessed/uncursed/cursed) status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum BucStatus {
    Blessed,
    #[default]
    Uncursed,
    Cursed,
}

impl BucStatus {
    /// Get status string for display
    pub const fn as_str(&self) -> &'static str {
        match self {
            BucStatus::Blessed => "blessed",
            BucStatus::Uncursed => "uncursed",
            BucStatus::Cursed => "cursed",
        }
    }
}

/// Object instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    /// Unique identifier
    pub id: ObjectId,

    /// Object type index (into ObjClassDef array)
    pub object_type: i16,

    /// Object class (cached from type)
    pub class: ObjectClass,

    /// Position (when on floor)
    pub x: i8,
    pub y: i8,

    /// Weight (can differ from base for containers)
    pub weight: u32,

    /// Quantity (for stackable items)
    pub quantity: i32,

    /// Enchantment/charges
    pub enchantment: i8,

    /// Inventory letter
    pub inv_letter: char,

    /// Artifact index (0 = not artifact)
    pub artifact: u8,

    /// Current location
    pub location: ObjectLocation,

    /// BUC status
    pub buc: BucStatus,

    /// Known to player
    pub known: bool,

    /// Description/appearance known
    pub desc_known: bool,

    /// BUC status known
    pub buc_known: bool,

    /// Rustproof status known
    pub rust_known: bool,

    /// Erosion level (rust/burn) 0-3
    pub erosion1: u8,

    /// Erosion level (corrode/rot) 0-3
    pub erosion2: u8,

    /// Erosion-proof
    pub erosion_proof: bool,

    /// Locked (containers)
    pub locked: bool,

    /// Broken (lock)
    pub broken: bool,

    /// Trapped (containers)
    pub trapped: bool,

    /// Recharged count
    pub recharged: u8,

    /// Lit (light sources)
    pub lit: bool,

    /// Greased
    pub greased: bool,

    /// Poisoned (weapons)
    pub poisoned: bool,

    /// Thrown by player (for pickup)
    pub thrown: bool,

    /// Currently in use
    pub in_use: bool,

    /// Worn mask (body slots)
    pub worn_mask: u32,

    /// Corpse monster type (for corpses)
    pub corpse_type: i16,

    /// Corpse won't revive (trolls, riders)
    pub norevive: bool,

    /// Age (creation time)
    pub age: i64,

    /// Contents (for containers)
    pub contents: Vec<Object>,

    /// Custom name
    pub name: Option<String>,

    /// Shop price (when unpaid)
    pub shop_price: i32,

    /// Unpaid flag
    pub unpaid: bool,

    /// Base AC value (for armor, set from ObjClassDef.bonus)
    /// In NetHack, lower AC is better. Base is 10, this is subtracted.
    pub base_ac: i8,

    /// Weapon damage dice (number of dice, from ObjClassDef.w_small_damage or w_large_damage)
    pub damage_dice: u8,

    /// Weapon damage sides (sides per die)
    pub damage_sides: u8,

    /// Weapon to-hit bonus (from ObjClassDef.bonus for weapons)
    pub weapon_tohit: i8,

    /// Nutrition value (from ObjClassDef.nutrition for food)
    pub nutrition: u16,
}

impl Default for Object {
    fn default() -> Self {
        Self {
            id: ObjectId::NONE,
            object_type: 0,
            class: ObjectClass::default(),
            x: 0,
            y: 0,
            weight: 0,
            quantity: 1,
            enchantment: 0,
            inv_letter: '\0',
            artifact: 0,
            location: ObjectLocation::Free,
            buc: BucStatus::Uncursed,
            known: false,
            desc_known: false,
            buc_known: false,
            rust_known: false,
            erosion1: 0,
            erosion2: 0,
            erosion_proof: false,
            locked: false,
            broken: false,
            trapped: false,
            recharged: 0,
            lit: false,
            greased: false,
            poisoned: false,
            thrown: false,
            in_use: false,
            worn_mask: 0,
            corpse_type: -1,
            norevive: false,
            age: 0,
            contents: Vec::new(),
            name: None,
            shop_price: 0,
            unpaid: false,
            base_ac: 0,
            damage_dice: 0,
            damage_sides: 0,
            weapon_tohit: 0,
            nutrition: 0,
        }
    }
}

impl Object {
    /// Create a new object of the given type
    pub fn new(id: ObjectId, object_type: i16, class: ObjectClass) -> Self {
        Self {
            id,
            object_type,
            class,
            ..Default::default()
        }
    }

    /// Check if object is blessed
    pub const fn is_blessed(&self) -> bool {
        matches!(self.buc, BucStatus::Blessed)
    }

    /// Check if object is cursed
    pub const fn is_cursed(&self) -> bool {
        matches!(self.buc, BucStatus::Cursed)
    }

    /// Check if object is a container
    ///
    /// Container object types (matching nh-data/objects.rs):
    /// - LargeBox = 360
    /// - Chest = 361
    /// - IceBox = 362
    /// - Sack = 363
    /// - OilskinSack = 364
    /// - BagOfHolding = 365
    /// - BagOfTricks = 366
    pub fn is_container(&self) -> bool {
        matches!(self.object_type, 360..=366)
    }

    /// Check if object is worn
    pub const fn is_worn(&self) -> bool {
        self.worn_mask != 0
    }

    /// Check if object is wielded
    pub const fn is_wielded(&self) -> bool {
        self.worn_mask & 0x8000 != 0 // W_WEP flag
    }

    /// Get total erosion level
    pub const fn erosion(&self) -> u8 {
        self.erosion1.saturating_add(self.erosion2)
    }

    /// Check if maximally eroded
    pub const fn is_destroyed(&self) -> bool {
        self.erosion1 >= 3 || self.erosion2 >= 3
    }

    /// Apply erosion (returns true if destroyed)
    pub fn erode(&mut self, erosion_type: u8) -> bool {
        if self.erosion_proof || self.greased {
            return false;
        }

        let erosion = if erosion_type == 0 {
            &mut self.erosion1
        } else {
            &mut self.erosion2
        };

        if *erosion < 3 {
            *erosion += 1;
        }

        self.is_destroyed()
    }

    /// Get effective enchantment (accounting for erosion)
    pub fn effective_enchantment(&self) -> i8 {
        self.enchantment - self.erosion() as i8
    }

    /// Check if this is armor
    pub const fn is_armor(&self) -> bool {
        matches!(self.class, ObjectClass::Armor)
    }

    /// Get effective AC contribution for armor
    /// Returns the AC bonus this armor provides (higher = more protection)
    /// Accounts for base AC, enchantment, and erosion
    pub fn effective_ac(&self) -> i8 {
        if !self.is_armor() {
            return 0;
        }
        // base_ac is the base protection
        // enchantment improves it (positive = better)
        // erosion degrades it (each point of erosion reduces AC by 1)
        self.base_ac + self.enchantment - self.erosion() as i8
    }

    /// Check if can merge with another object (C: mergable)
    ///
    /// Two objects can merge only if they match on all visible and mechanical
    /// properties. This follows C NetHack's 20+ condition merge check.
    pub fn can_merge(&self, other: &Object) -> bool {
        // Must not be same object
        if self.id == other.id {
            return false;
        }

        // Must be same type and type must allow merging
        if self.object_type != other.object_type || !self.class.stacks() {
            return false;
        }

        // Coins always merge
        if self.class == ObjectClass::Coin {
            return true;
        }

        // BUC must match
        if self.buc != other.buc {
            return false;
        }

        // Enchantment/charges must match
        if self.enchantment != other.enchantment {
            return false;
        }

        // Unpaid/shop status must match
        if self.unpaid != other.unpaid {
            return false;
        }

        // Broken/trapped/lit/greased must match
        if self.broken != other.broken
            || self.trapped != other.trapped
            || self.lit != other.lit
            || self.greased != other.greased
        {
            return false;
        }

        // Erosion must match
        if self.erosion1 != other.erosion1 || self.erosion2 != other.erosion2 {
            return false;
        }

        // For weapons/armor: erosion-proof and rust_known must match
        if matches!(self.class, ObjectClass::Weapon | ObjectClass::Armor)
            && (self.erosion_proof != other.erosion_proof
                || self.rust_known != other.rust_known)
        {
            return false;
        }

        // Poisoned must match
        if self.poisoned != other.poisoned {
            return false;
        }

        // Knowledge state must match
        if self.desc_known != other.desc_known || self.buc_known != other.buc_known {
            return false;
        }

        // Corpse/egg/tin: monster type must match
        if self.corpse_type != other.corpse_type {
            return false;
        }

        // Revivable corpses don't merge
        if self.norevive || other.norevive {
            return false;
        }

        // Lit items (oil lamps, candles) don't merge while burning
        if self.lit {
            return false;
        }

        // Artifact items never merge (artifacts are unique)
        if self.artifact != other.artifact {
            return false;
        }

        // Known status must match
        if self.known != other.known {
            return false;
        }

        // Names must match
        if self.name != other.name {
            return false;
        }

        // Unpaid items must have same price
        if self.unpaid && self.shop_price != other.shop_price {
            return false;
        }

        true
    }

    /// Merge another object into this one (C: merged)
    ///
    /// Adds the other object's quantity and weight.
    /// For coins, weight is recalculated instead of added.
    pub fn merge(&mut self, other: Object) {
        // Average age weighted by quantity
        if !self.lit {
            self.age = (self.age * self.quantity as i64 + other.age * other.quantity as i64)
                / (self.quantity as i64 + other.quantity as i64);
        }

        self.quantity += other.quantity;

        // Coins: recalculate weight (100 coins = 1 weight unit)
        if self.class == ObjectClass::Coin {
            self.weight = ((self.quantity as u32 + 50) / 100).max(1);
        } else {
            self.weight += other.weight;
        }

        // If we have no name but other does, take it
        if self.name.is_none() && other.name.is_some() {
            self.name = other.name;
        }
    }

    /// Split a quantity from this stack, returning the new object (C: splitobj)
    ///
    /// The current object keeps `self.quantity - count`, the returned object
    /// gets `count`. Panics if count <= 0, count >= self.quantity, or object
    /// is a container.
    pub fn split(&mut self, count: i32, new_id: ObjectId) -> Object {
        assert!(count > 0, "split: count must be positive");
        assert!(
            count < self.quantity,
            "split: count must be less than total quantity"
        );
        assert!(
            self.contents.is_empty(),
            "split: cannot split containers"
        );

        // Calculate per-unit weight before modifying quantities
        let original_qty = self.quantity;
        let per_unit = if self.class == ObjectClass::Coin {
            0 // coins handled separately
        } else if original_qty > 0 {
            self.weight / original_qty as u32
        } else {
            1
        };

        let mut new_obj = self.clone();
        new_obj.id = new_id;
        new_obj.worn_mask = 0; // new stack is not worn

        // Adjust quantities
        self.quantity -= count;
        new_obj.quantity = count;

        // Recalculate weights
        if self.class == ObjectClass::Coin {
            self.weight = ((self.quantity as u32 + 50) / 100).max(1);
            new_obj.weight = ((count as u32 + 50) / 100).max(1);
        } else {
            self.weight = per_unit * self.quantity as u32;
            new_obj.weight = per_unit * count as u32;
        }

        new_obj
    }

    /// Get display name for the object (simple version, use xname/doname for full)
    pub fn display_name(&self) -> String {
        if let Some(ref name) = self.name {
            if self.quantity > 1 {
                format!("{} {}", self.quantity, name)
            } else {
                name.clone()
            }
        } else {
            // Fallback to class name
            let class_name = self.class_name();
            if self.quantity > 1 {
                format!("{} {}s", self.quantity, class_name)
            } else {
                format!("a {}", class_name)
            }
        }
    }

    /// Get the generic class name for this object
    pub const fn class_name(&self) -> &'static str {
        match self.class {
            ObjectClass::Weapon => "weapon",
            ObjectClass::Armor => "armor",
            ObjectClass::Ring => "ring",
            ObjectClass::Amulet => "amulet",
            ObjectClass::Tool => "tool",
            ObjectClass::Food => "food",
            ObjectClass::Potion => "potion",
            ObjectClass::Scroll => "scroll",
            ObjectClass::Spellbook => "spellbook",
            ObjectClass::Wand => "wand",
            ObjectClass::Coin => "gold piece",
            ObjectClass::Gem => "gem",
            ObjectClass::Rock => "rock",
            ObjectClass::Ball => "ball",
            ObjectClass::Chain => "chain",
            ObjectClass::Venom => "venom",
            ObjectClass::Random => "strange object",
            ObjectClass::IllObj => "strange object",
        }
    }

    /// Get erosion description prefix (C: erode_obj display logic)
    ///
    /// Uses class-based heuristic for erosion words:
    /// - Weapons/armor/ball/chain: "rusty"/"corroded", proof = "rustproof"/"corrodeproof"
    /// - Other (scrolls, spellbooks, etc.): "burnt"/"rotted", proof = "fireproof"
    pub fn erosion_prefix(&self) -> String {
        let mut prefix = String::new();
        let is_metallic = matches!(
            self.class,
            ObjectClass::Weapon | ObjectClass::Armor | ObjectClass::Ball | ObjectClass::Chain
        );

        // First erosion type (rust/burn)
        if self.erosion1 > 0 {
            match self.erosion1 {
                2 => prefix.push_str("very "),
                3 => prefix.push_str("thoroughly "),
                _ => {}
            }
            if is_metallic {
                prefix.push_str("rusty ");
            } else {
                prefix.push_str("burnt ");
            }
        }

        // Second erosion type (corrode/rot)
        if self.erosion2 > 0 {
            match self.erosion2 {
                2 => prefix.push_str("very "),
                3 => prefix.push_str("thoroughly "),
                _ => {}
            }
            if is_metallic {
                prefix.push_str("corroded ");
            } else {
                prefix.push_str("rotted ");
            }
        }

        // Erosion-proof status (if known)
        if self.rust_known && self.erosion_proof {
            if is_metallic {
                // C: rustproof for rust, corrodeproof for corrosion, fixed for both
                if self.erosion1 == 0 && self.erosion2 == 0 {
                    prefix.push_str("rustproof ");
                } else {
                    prefix.push_str("fixed ");
                }
            } else {
                prefix.push_str("fireproof ");
            }
        }

        prefix
    }

    /// Get BUC status prefix (if known)
    pub fn buc_prefix(&self) -> &'static str {
        if !self.buc_known {
            return "";
        }
        match self.buc {
            BucStatus::Blessed => "blessed ",
            BucStatus::Cursed => "cursed ",
            BucStatus::Uncursed => {
                // Only show "uncursed" for items where it matters
                if matches!(
                    self.class,
                    ObjectClass::Weapon
                        | ObjectClass::Armor
                        | ObjectClass::Ring
                        | ObjectClass::Amulet
                        | ObjectClass::Wand
                        | ObjectClass::Tool
                        | ObjectClass::Potion
                        | ObjectClass::Scroll
                ) {
                    "uncursed "
                } else {
                    ""
                }
            }
        }
    }

    /// Get enchantment string (e.g., "+2" or "-1")
    ///
    /// Wands do NOT show enchantment prefix — they use `charges_suffix()` instead.
    /// Rings show enchantment only when non-zero.
    pub fn enchantment_str(&self) -> String {
        if !self.known {
            return String::new();
        }
        match self.class {
            ObjectClass::Weapon | ObjectClass::Armor => {
                format!("{:+} ", self.enchantment)
            }
            ObjectClass::Ring => {
                if self.enchantment != 0 {
                    format!("{:+} ", self.enchantment)
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        }
    }

    /// Get worn/wielded suffix
    pub fn worn_suffix(&self) -> &'static str {
        if self.worn_mask == 0 {
            return "";
        }
        match self.class {
            ObjectClass::Weapon => " (weapon in hand)",
            ObjectClass::Armor => " (being worn)",
            ObjectClass::Amulet => " (being worn)",
            ObjectClass::Ring => " (on finger)",
            ObjectClass::Tool => " (being worn)",
            _ => " (in use)",
        }
    }

    /// Get charges suffix for wands (if known)
    pub fn charges_suffix(&self) -> String {
        if self.known && self.class == ObjectClass::Wand {
            format!(" ({}:{})", self.recharged, self.enchantment)
        } else {
            String::new()
        }
    }

    /// Full object name for display (like NetHack's doname)
    /// This builds a complete description including quantity, BUC, enchantment,
    /// erosion, the base name, and any suffixes.
    ///
    /// # Arguments
    /// * `base_name` - The base name of the object (from OBJECTS array or custom name)
    pub fn doname(&self, base_name: &str) -> String {
        let mut parts = Vec::new();

        // Quantity
        if self.quantity > 1 {
            parts.push(format!("{}", self.quantity));
        }

        // Empty container (C: cknown && !Has_contents)
        if self.is_container() && self.known && self.contents.is_empty() {
            parts.push("empty".to_string());
        }

        // BUC status
        let buc = self.buc_prefix();
        if !buc.is_empty() {
            parts.push(buc.trim().to_string());
        }

        // Lock status for containers (C: lknown)
        if self.is_container() && self.known {
            if self.broken {
                parts.push("broken".to_string());
            } else if self.locked {
                parts.push("locked".to_string());
            }
        }

        // Greased
        if self.greased {
            parts.push("greased".to_string());
        }

        // Enchantment
        let ench = self.enchantment_str();
        if !ench.is_empty() {
            parts.push(ench.trim().to_string());
        }

        // Erosion-proof and erosion
        let erosion = self.erosion_prefix();
        if !erosion.is_empty() {
            parts.push(erosion.trim().to_string());
        }

        // Poisoned (for weapons)
        if self.poisoned {
            parts.push("poisoned".to_string());
        }

        // Base name (pluralized if quantity > 1)
        if self.quantity > 1 {
            parts.push(makeplural(base_name));
        } else {
            parts.push(base_name.to_string());
        }

        // Build the main string
        let mut result = parts.join(" ");

        // Add suffixes
        result.push_str(&self.charges_suffix());
        result.push_str(self.worn_suffix());

        // Container contents
        if self.is_container() && !self.contents.is_empty() {
            let n = self.contents.len();
            result.push_str(&format!(
                " (containing {} item{})",
                n,
                if n == 1 { "" } else { "s" }
            ));
        }

        // Lit status for light sources
        if self.lit {
            result.push_str(" (lit)");
        }

        // Named items (C: "named X" suffix, separate from class/type name)
        if let Some(ref name) = self.name.as_ref().filter(|_| self.desc_known) {
            result.push_str(&format!(" named {}", name));
        }

        // Unpaid items
        if self.unpaid && self.shop_price > 0 {
            result.push_str(&format!(
                " (unpaid, {} zorkmid{})",
                self.shop_price,
                if self.shop_price == 1 { "" } else { "s" }
            ));
        }

        result
    }

    /// Simple name without BUC or enchantment (like NetHack's xname)
    ///
    /// Includes: quantity, pluralization, poisoned, greased, erosion, base name,
    /// and "named X" suffix. Does NOT include BUC, enchantment, worn status, or
    /// container/pricing info — those are added by `doname()`.
    pub fn xname(&self, base_name: &str) -> String {
        let mut parts = Vec::new();

        // Quantity (C: xname adds quantity and pluralizes)
        if self.quantity > 1 {
            parts.push(format!("{}", self.quantity));
        }

        // Poisoned (for weapons)
        if self.poisoned {
            parts.push("poisoned".to_string());
        }

        // Greased
        if self.greased {
            parts.push("greased".to_string());
        }

        // Erosion
        let erosion = self.erosion_prefix();
        if !erosion.is_empty() {
            parts.push(erosion.trim().to_string());
        }

        // Base name (pluralized if quantity > 1)
        if self.quantity > 1 {
            parts.push(makeplural(base_name));
        } else {
            parts.push(base_name.to_string());
        }

        let mut result = parts.join(" ");

        // Named items (C: "named X" suffix)
        if let Some(ref name) = self.name.as_ref().filter(|_| self.desc_known) {
            result.push_str(&format!(" named {}", name));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buc_prefix() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;

        // Unknown BUC
        obj.buc_known = false;
        assert_eq!(obj.buc_prefix(), "");

        // Known blessed
        obj.buc_known = true;
        obj.buc = BucStatus::Blessed;
        assert_eq!(obj.buc_prefix(), "blessed ");

        // Known cursed
        obj.buc = BucStatus::Cursed;
        assert_eq!(obj.buc_prefix(), "cursed ");

        // Known uncursed (weapon shows it)
        obj.buc = BucStatus::Uncursed;
        assert_eq!(obj.buc_prefix(), "uncursed ");

        // Known uncursed food (doesn't show it)
        obj.class = ObjectClass::Food;
        assert_eq!(obj.buc_prefix(), "");
    }

    #[test]
    fn test_enchantment_str() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.enchantment = 2;

        // Unknown
        obj.known = false;
        assert_eq!(obj.enchantment_str(), "");

        // Known weapon
        obj.known = true;
        assert_eq!(obj.enchantment_str(), "+2 ");

        // Negative
        obj.enchantment = -1;
        assert_eq!(obj.enchantment_str(), "-1 ");

        // Food doesn't show enchantment
        obj.class = ObjectClass::Food;
        assert_eq!(obj.enchantment_str(), "");
    }

    #[test]
    fn test_erosion_prefix() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;

        // No erosion
        assert_eq!(obj.erosion_prefix(), "");

        // Rusty
        obj.erosion1 = 1;
        assert_eq!(obj.erosion_prefix(), "rusty ");

        // Very rusty
        obj.erosion1 = 2;
        assert_eq!(obj.erosion_prefix(), "very rusty ");

        // Thoroughly rusty and corroded
        obj.erosion1 = 3;
        obj.erosion2 = 1;
        assert_eq!(obj.erosion_prefix(), "thoroughly rusty corroded ");

        // Rustproof
        obj.erosion1 = 0;
        obj.erosion2 = 0;
        obj.rust_known = true;
        obj.erosion_proof = true;
        assert_eq!(obj.erosion_prefix(), "rustproof ");
    }

    #[test]
    fn test_doname() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.quantity = 1;
        obj.buc_known = true;
        obj.buc = BucStatus::Uncursed;
        obj.known = true;
        obj.enchantment = 2;

        let name = obj.doname("long sword");
        assert!(name.contains("uncursed"));
        assert!(name.contains("+2"));
        assert!(name.contains("long sword"));
    }

    #[test]
    fn test_doname_with_quantity() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.quantity = 5;

        let name = obj.doname("arrow");
        assert_eq!(name, "5 arrows"); // pluralized
    }

    #[test]
    fn test_doname_snapshot_blessed_weapon() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.quantity = 1;
        obj.buc_known = true;
        obj.buc = BucStatus::Blessed;
        obj.known = true;
        obj.enchantment = 3;
        assert_eq!(obj.doname("long sword"), "blessed +3 long sword");
    }

    #[test]
    fn test_doname_snapshot_cursed_rusty_weapon() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.quantity = 1;
        obj.buc_known = true;
        obj.buc = BucStatus::Cursed;
        obj.known = true;
        obj.enchantment = -1;
        obj.erosion1 = 2; // very rusty
        assert_eq!(
            obj.doname("long sword"),
            "cursed -1 very rusty long sword"
        );
    }

    #[test]
    fn test_doname_snapshot_greased_armor() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Armor;
        obj.quantity = 1;
        obj.buc_known = true;
        obj.buc = BucStatus::Uncursed;
        obj.known = true;
        obj.enchantment = 0;
        obj.greased = true;
        assert_eq!(
            obj.doname("leather armor"),
            "uncursed greased +0 leather armor"
        );
    }

    #[test]
    fn test_doname_snapshot_stack_poisoned() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.quantity = 12;
        obj.buc_known = true;
        obj.buc = BucStatus::Uncursed;
        obj.known = true;
        obj.enchantment = 0;
        obj.poisoned = true;
        assert_eq!(
            obj.doname("dart"),
            "12 uncursed +0 poisoned darts"
        );
    }

    #[test]
    fn test_doname_snapshot_named_item() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.quantity = 1;
        obj.buc_known = true;
        obj.buc = BucStatus::Uncursed;
        obj.known = true;
        obj.enchantment = 5;
        obj.desc_known = true;
        obj.name = Some("Sting".to_string());
        assert_eq!(
            obj.doname("elven short sword"),
            "uncursed +5 elven short sword named Sting"
        );
    }

    #[test]
    fn test_doname_snapshot_wand_with_charges() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Wand;
        obj.quantity = 1;
        obj.buc_known = true;
        obj.buc = BucStatus::Uncursed;
        obj.known = true;
        obj.enchantment = 4;
        obj.recharged = 0;
        assert_eq!(
            obj.doname("wand of fire"),
            "uncursed wand of fire (0:4)"
        );
    }

    #[test]
    fn test_doname_snapshot_empty_container() {
        let mut obj = Object::default();
        obj.object_type = 363; // sack
        obj.class = ObjectClass::Tool;
        obj.quantity = 1;
        obj.known = true;
        assert_eq!(obj.doname("sack"), "empty sack");
    }

    #[test]
    fn test_doname_snapshot_locked_container() {
        let mut obj = Object::default();
        obj.object_type = 361; // chest
        obj.class = ObjectClass::Tool;
        obj.quantity = 1;
        obj.known = true;
        obj.locked = true;
        assert_eq!(obj.doname("chest"), "empty locked chest");
    }

    #[test]
    fn test_doname_snapshot_container_with_contents() {
        let mut obj = Object::default();
        obj.object_type = 363; // sack
        obj.class = ObjectClass::Tool;
        obj.quantity = 1;
        obj.known = true;
        obj.contents.push(Object::default());
        obj.contents.push(Object::default());
        assert_eq!(
            obj.doname("sack"),
            "sack (containing 2 items)"
        );
    }

    #[test]
    fn test_doname_snapshot_unpaid() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Potion;
        obj.quantity = 1;
        obj.unpaid = true;
        obj.shop_price = 100;
        assert_eq!(
            obj.doname("potion of healing"),
            "potion of healing (unpaid, 100 zorkmids)"
        );
    }

    #[test]
    fn test_doname_snapshot_lit_lamp() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Tool;
        obj.quantity = 1;
        obj.lit = true;
        assert_eq!(obj.doname("oil lamp"), "oil lamp (lit)");
    }

    #[test]
    fn test_doname_snapshot_erosion_proof_weapon() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.quantity = 1;
        obj.buc_known = true;
        obj.buc = BucStatus::Uncursed;
        obj.known = true;
        obj.enchantment = 2;
        obj.rust_known = true;
        obj.erosion_proof = true;
        assert_eq!(
            obj.doname("long sword"),
            "uncursed +2 rustproof long sword"
        );
    }

    #[test]
    fn test_xname_basic() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.quantity = 1;
        assert_eq!(obj.xname("long sword"), "long sword");
    }

    #[test]
    fn test_xname_with_quantity() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.quantity = 5;
        assert_eq!(obj.xname("arrow"), "5 arrows");
    }

    #[test]
    fn test_xname_with_erosion() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.quantity = 1;
        obj.erosion1 = 1;
        assert_eq!(obj.xname("long sword"), "rusty long sword");
    }

    #[test]
    fn test_xname_named() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.quantity = 1;
        obj.desc_known = true;
        obj.name = Some("Orcrist".to_string());
        assert_eq!(
            obj.xname("elven broadsword"),
            "elven broadsword named Orcrist"
        );
    }

    #[test]
    fn test_worn_suffix() {
        let mut obj = Object::default();

        // Not worn
        assert_eq!(obj.worn_suffix(), "");

        // Weapon wielded
        obj.class = ObjectClass::Weapon;
        obj.worn_mask = 1;
        assert_eq!(obj.worn_suffix(), " (weapon in hand)");

        // Armor worn
        obj.class = ObjectClass::Armor;
        assert_eq!(obj.worn_suffix(), " (being worn)");
    }

    #[test]
    fn test_charges_suffix() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Wand;
        obj.enchantment = 5;
        obj.recharged = 1;

        // Unknown
        obj.known = false;
        assert_eq!(obj.charges_suffix(), "");

        // Known
        obj.known = true;
        assert_eq!(obj.charges_suffix(), " (1:5)");
    }

    fn make_arrow(id: u32, qty: i32, buc: BucStatus) -> Object {
        let mut obj = Object::default();
        obj.id = ObjectId(id);
        obj.object_type = 10; // arrow type
        obj.class = ObjectClass::Weapon;
        obj.quantity = qty;
        obj.weight = qty as u32; // 1 per unit
        obj.buc = buc;
        obj
    }

    #[test]
    fn test_can_merge_same_type_same_buc() {
        let a = make_arrow(1, 5, BucStatus::Uncursed);
        let b = make_arrow(2, 3, BucStatus::Uncursed);
        assert!(a.can_merge(&b));
    }

    #[test]
    fn test_cannot_merge_different_buc() {
        let a = make_arrow(1, 5, BucStatus::Uncursed);
        let b = make_arrow(2, 3, BucStatus::Cursed);
        assert!(!a.can_merge(&b));
    }

    #[test]
    fn test_cannot_merge_same_id() {
        let a = make_arrow(1, 5, BucStatus::Uncursed);
        assert!(!a.can_merge(&a));
    }

    #[test]
    fn test_cannot_merge_different_enchantment() {
        let mut a = make_arrow(1, 5, BucStatus::Uncursed);
        let mut b = make_arrow(2, 3, BucStatus::Uncursed);
        a.enchantment = 1;
        b.enchantment = 2;
        assert!(!a.can_merge(&b));
    }

    #[test]
    fn test_cannot_merge_different_erosion() {
        let mut a = make_arrow(1, 5, BucStatus::Uncursed);
        let b = make_arrow(2, 3, BucStatus::Uncursed);
        a.erosion1 = 1;
        assert!(!a.can_merge(&b));
    }

    #[test]
    fn test_coins_always_merge() {
        let mut a = Object::default();
        a.id = ObjectId(1);
        a.object_type = 100;
        a.class = ObjectClass::Coin;
        a.quantity = 50;

        let mut b = a.clone();
        b.id = ObjectId(2);
        b.buc = BucStatus::Cursed; // coins ignore BUC for merge
        assert!(a.can_merge(&b));
    }

    #[test]
    fn test_cannot_merge_lit_items() {
        let mut a = make_arrow(1, 5, BucStatus::Uncursed);
        let mut b = make_arrow(2, 3, BucStatus::Uncursed);
        a.lit = true;
        b.lit = true;
        assert!(!a.can_merge(&b));
    }

    #[test]
    fn test_merge_adds_quantities() {
        let mut a = make_arrow(1, 5, BucStatus::Uncursed);
        let b = make_arrow(2, 3, BucStatus::Uncursed);
        a.merge(b);
        assert_eq!(a.quantity, 8);
        assert_eq!(a.weight, 8);
    }

    #[test]
    fn test_split_stack() {
        let mut stack = make_arrow(1, 20, BucStatus::Uncursed);
        stack.weight = 20; // 1 per arrow
        let new_stack = stack.split(5, ObjectId(100));

        assert_eq!(stack.quantity, 15);
        assert_eq!(stack.weight, 15);
        assert_eq!(new_stack.quantity, 5);
        assert_eq!(new_stack.weight, 5);
        assert_eq!(new_stack.id, ObjectId(100));
        assert_eq!(new_stack.worn_mask, 0); // not worn
    }

    #[test]
    #[should_panic(expected = "count must be less than")]
    fn test_split_entire_stack_panics() {
        let mut stack = make_arrow(1, 5, BucStatus::Uncursed);
        stack.split(5, ObjectId(100));
    }

    #[test]
    #[should_panic(expected = "count must be positive")]
    fn test_split_zero_panics() {
        let mut stack = make_arrow(1, 5, BucStatus::Uncursed);
        stack.split(0, ObjectId(100));
    }
}
