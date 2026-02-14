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

    /// Get BUC sign value (bcsign equivalent)
    /// Returns +1 for blessed, 0 for uncursed, -1 for cursed
    pub const fn sign(&self) -> i8 {
        match self {
            BucStatus::Blessed => 1,
            BucStatus::Uncursed => 0,
            BucStatus::Cursed => -1,
        }
    }

    /// Check if blessed
    pub const fn is_blessed(&self) -> bool {
        matches!(self, BucStatus::Blessed)
    }

    /// Check if cursed
    pub const fn is_cursed(&self) -> bool {
        matches!(self, BucStatus::Cursed)
    }

    /// Check if uncursed
    pub const fn is_uncursed(&self) -> bool {
        matches!(self, BucStatus::Uncursed)
    }

    /// Opposite BUC status (blessed <-> cursed, uncursed stays)
    pub const fn opposite(&self) -> Self {
        match self {
            BucStatus::Blessed => BucStatus::Cursed,
            BucStatus::Cursed => BucStatus::Blessed,
            BucStatus::Uncursed => BucStatus::Uncursed,
        }
    }
}

/// Randomly bless or curse an object (blessorcurse equivalent)
/// Uses RNG to determine BUC state for newly generated objects
pub fn blessorcurse(rng: &mut crate::GameRng, chance: i32) -> BucStatus {
    if chance <= 0 {
        return BucStatus::Uncursed;
    }
    let chance_u32 = chance as u32;
    let roll = rng.rn2(chance_u32);
    if roll == 0 {
        BucStatus::Blessed
    } else if roll <= chance_u32 / 4 {
        BucStatus::Cursed
    } else {
        BucStatus::Uncursed
    }
}

/// Get BUC sign from status (bcsign equivalent)
pub const fn bcsign(buc: BucStatus) -> i8 {
    buc.sign()
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

    /// Wand usage count (for degradation tracking)
    pub wand_use_count: i32,
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
            wand_use_count: 0,
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

    /// Check if this is a boulder
    /// Boulder object type is typically Rock class with specific type
    pub fn is_boulder(&self) -> bool {
        // Boulder object type in NetHack is typically index ~1 in the Rock class
        // For now, check class and weight (boulders are very heavy, ~6000)
        self.class == ObjectClass::Rock && self.weight >= 1000
    }

    /// Check if this is a statue
    pub fn is_statue(&self) -> bool {
        // Statues are typically in the Rock class
        self.class == ObjectClass::Rock && self.corpse_type >= 0
    }

    /// Check if this is a corpse
    pub fn is_corpse(&self) -> bool {
        self.class == ObjectClass::Food && self.corpse_type >= 0
    }

    /// Check if this is an egg
    pub fn is_egg(&self) -> bool {
        // Eggs are food items with monster type set
        self.class == ObjectClass::Food && self.corpse_type >= 0
    }

    /// Check if this is a figurine
    pub fn is_figurine(&self) -> bool {
        // Figurines are tools with monster type
        self.class == ObjectClass::Tool && self.corpse_type >= 0
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
            // Import get_wand_status would require circular import, so inline basic status here
            let wear_status = if self.wand_use_count > 0 {
                let wear = (self.wand_use_count as f32) * 0.05;
                let wear = wear.min(1.0);
                if wear > 0.8 {
                    " [heavily worn]"
                } else if wear > 0.5 {
                    " [worn]"
                } else if wear > 0.2 {
                    " [slightly worn]"
                } else {
                    ""
                }
            } else {
                ""
            };
            format!(" ({}:{}){}", self.recharged, self.enchantment, wear_status)
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

    // ========================================================================
    // BUC state modification (from mkobj.c / do.c)
    // ========================================================================

    /// Bless this object (set to blessed status)
    pub fn bless(&mut self) {
        self.buc = BucStatus::Blessed;
    }

    /// Curse this object (set to cursed status)
    pub fn curse(&mut self) {
        self.buc = BucStatus::Cursed;
    }

    /// Uncurse this object (set to uncursed status)
    pub fn uncurse(&mut self) {
        self.buc = BucStatus::Uncursed;
    }

    /// Set BUC status and reveal it
    pub fn set_buc(&mut self, buc: BucStatus, reveal: bool) {
        self.buc = buc;
        if reveal {
            self.buc_known = true;
        }
    }

    /// Check if can be blessed (already blessed = no change)
    pub fn can_bless(&self) -> bool {
        !matches!(self.buc, BucStatus::Blessed)
    }

    /// Check if can be cursed (already cursed = no change)
    pub fn can_curse(&self) -> bool {
        !matches!(self.buc, BucStatus::Cursed)
    }

    // ========================================================================
    // Object type checks (from objnam.c and mkobj.c)
    // ========================================================================

    /// Check if this is a weapon
    pub const fn is_weapon(&self) -> bool {
        matches!(self.class, ObjectClass::Weapon)
    }

    /// Check if this is a ring
    pub const fn is_ring(&self) -> bool {
        matches!(self.class, ObjectClass::Ring)
    }

    /// Check if this is an amulet
    pub const fn is_amulet(&self) -> bool {
        matches!(self.class, ObjectClass::Amulet)
    }

    /// Check if this is a tool
    pub const fn is_tool(&self) -> bool {
        matches!(self.class, ObjectClass::Tool)
    }

    /// Check if this is food
    pub const fn is_food(&self) -> bool {
        matches!(self.class, ObjectClass::Food)
    }

    /// Check if this is a potion
    pub const fn is_potion(&self) -> bool {
        matches!(self.class, ObjectClass::Potion)
    }

    /// Check if this is a scroll
    pub const fn is_scroll(&self) -> bool {
        matches!(self.class, ObjectClass::Scroll)
    }

    /// Check if this is a spellbook
    pub const fn is_spellbook(&self) -> bool {
        matches!(self.class, ObjectClass::Spellbook)
    }

    /// Check if this is a wand
    pub const fn is_wand(&self) -> bool {
        matches!(self.class, ObjectClass::Wand)
    }

    /// Check if this is gold/coins
    pub const fn is_gold(&self) -> bool {
        matches!(self.class, ObjectClass::Coin)
    }

    /// Check if this is a gem
    pub const fn is_gem(&self) -> bool {
        matches!(self.class, ObjectClass::Gem)
    }

    /// Check if this is a rock class item
    pub const fn is_rock(&self) -> bool {
        matches!(self.class, ObjectClass::Rock)
    }

    /// Check if this is a ball (punishment)
    pub const fn is_ball(&self) -> bool {
        matches!(self.class, ObjectClass::Ball)
    }

    /// Check if this is a chain (punishment)
    pub const fn is_chain(&self) -> bool {
        matches!(self.class, ObjectClass::Chain)
    }

    /// Check if this is venom
    pub const fn is_venom(&self) -> bool {
        matches!(self.class, ObjectClass::Venom)
    }

    /// Check if this object is unpaid
    pub const fn is_unpaid(&self) -> bool {
        self.unpaid
    }

    /// Check if this object is lit (light source)
    pub const fn is_lit(&self) -> bool {
        self.lit
    }

    /// Check if this object is greased
    pub const fn is_greased(&self) -> bool {
        self.greased
    }

    /// Check if this object is poisoned
    pub const fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    /// Check if this object is an artifact
    pub const fn is_artifact(&self) -> bool {
        self.artifact != 0
    }

    /// Check if this is erosion-proof
    pub const fn is_erodeproof(&self) -> bool {
        self.erosion_proof
    }

    /// Check if this is a light source (could be lit)
    pub fn is_light_source(&self) -> bool {
        // Tools that can be lit: candles, lamps, lanterns
        // This would check specific object types in a real implementation
        matches!(self.class, ObjectClass::Tool)
    }

    // ========================================================================
    // Luck and special effects
    // ========================================================================

    /// Check if this object confers luck (confers_luck equivalent)
    /// Luckstones prevent luck from timing out
    pub fn confers_luck(&self) -> bool {
        // Luckstone is a gem type
        self.class == ObjectClass::Gem && self.object_type == crate::LUCKSTONE
    }

    /// Check if this is a luckstone
    pub fn is_luckstone(&self) -> bool {
        self.class == ObjectClass::Gem && self.object_type == crate::LUCKSTONE
    }

    /// Check if this is a loadstone (cursed stone that can't be dropped)
    pub fn is_loadstone(&self) -> bool {
        self.class == ObjectClass::Gem && self.object_type == crate::LOADSTONE
    }

    /// Check if this is a touchstone (used for gem identification)
    pub fn is_touchstone(&self) -> bool {
        self.class == ObjectClass::Gem && self.object_type == crate::TOUCHSTONE
    }

    /// Check if this object cancels luck timeout
    /// When carried, prevents luck from decaying toward 0
    pub fn cancels_luck_timeout(&self) -> bool {
        self.confers_luck()
    }

    /// Check if this is a gray stone (special stone type)
    pub fn is_graystone(&self) -> bool {
        self.is_luckstone() || self.is_loadstone() || self.is_touchstone() || self.is_flint()
    }

    /// Check if this is flint (for fire starting)
    pub fn is_flint(&self) -> bool {
        self.class == ObjectClass::Gem && self.object_type == crate::FLINT
    }

    /// Check if this is a rock
    pub fn is_rock_obj(&self) -> bool {
        self.class == ObjectClass::Gem && self.object_type == crate::ROCK
    }

    /// Check if object is chargeable (is_chargeable equivalent)
    pub const fn is_chargeable(&self) -> bool {
        matches!(self.class, ObjectClass::Wand | ObjectClass::Tool)
    }

    /// Check if object can rot (is_rottable equivalent)
    ///
    /// Food items and corpses can rot.
    pub const fn is_rottable(&self) -> bool {
        matches!(self.class, ObjectClass::Food)
    }

    /// Check if object is flammable (is_flammable equivalent)
    ///
    /// Scrolls, spellbooks, and potions can burn/be destroyed by fire.
    /// Note: Full material check would require object type lookup.
    pub const fn is_flammable(&self) -> bool {
        matches!(self.class, ObjectClass::Scroll | ObjectClass::Spellbook)
    }

    /// Check if object is edible (is_edible equivalent)
    pub const fn is_edible(&self) -> bool {
        matches!(self.class, ObjectClass::Food)
    }

    // ========================================================================
    // Container functions (invent.c, mkobj.c)
    // ========================================================================

    /// Add object to this container (add_to_container equivalent)
    /// Returns true if object was added successfully.
    pub fn add_to_container(&mut self, mut obj: Object) -> bool {
        if !self.is_container() {
            return false;
        }

        // Can't put container inside itself
        if obj.id == self.id {
            return false;
        }

        // Update the object's location
        obj.location = ObjectLocation::Contained;

        // Try to merge with existing contents
        for existing in &mut self.contents {
            if existing.can_merge(&obj) {
                existing.merge(obj);
                return true;
            }
        }

        // Add as new item
        self.contents.push(obj);
        true
    }

    /// Remove object from this container (out_container equivalent concept)
    /// Returns the removed object if found.
    pub fn remove_from_container(&mut self, object_id: ObjectId) -> Option<Object> {
        if let Some(idx) = self.contents.iter().position(|o| o.id == object_id) {
            let mut obj = self.contents.remove(idx);
            obj.location = ObjectLocation::Free;
            Some(obj)
        } else {
            None
        }
    }

    /// Check if this container contains a specific object type
    pub fn container_contains(&self, object_type: i16) -> bool {
        self.contents.iter().any(|o| o.object_type == object_type)
    }

    /// Get total weight of container contents (container_weight equivalent)
    pub fn container_weight(&self) -> u32 {
        self.contents
            .iter()
            .map(|o| o.weight * o.quantity as u32)
            .sum()
    }

    /// Get count of items in container
    pub fn container_count(&self) -> usize {
        self.contents.len()
    }

    /// Check if container is empty
    pub fn container_empty(&self) -> bool {
        self.contents.is_empty()
    }

    // ========================================================================
    // Object stack/split functions (invent.c)
    // ========================================================================

    /// Split this object stack, removing `count` items (splitobj equivalent)
    /// Returns a new object with the split items, or None if can't split.
    pub fn splitobj(&mut self, count: i32) -> Option<Object> {
        if count <= 0 || count >= self.quantity {
            return None;
        }

        // Create a copy for the split portion
        let mut split = self.clone();
        split.quantity = count;
        split.weight = (self.weight / self.quantity as u32) * count as u32;

        // Reduce this object's quantity
        self.quantity -= count;
        self.weight -= split.weight;

        // The split object needs a new ID (caller should assign)
        split.id = ObjectId::NONE;

        Some(split)
    }

    /// Check if this object can merge with another (full 15-condition check)
    ///
    /// # Merging Rules (all must be true):
    /// 1. Same object_type
    /// 2. Same BUC status
    /// 3. Same enchantment
    /// 4. Same erosion1 (rust/burn)
    /// 5. Same erosion2 (corrode/rot)
    /// 6. Same poisoned status
    /// 7. Same greased status
    /// 8. Same broken status
    /// 9. Neither worn/wielded
    /// 10. Same name (or both unnamed)
    /// 11. Not artifacts (never merge)
    /// 12. Not containers with contents
    /// 13. Not unique objects
    /// 14. For corpses: same age (within tolerance)
    /// 15. Object class must support stacking
    pub fn mergable(&self, other: &Object) -> bool {
        // Rule 15: Check if class stacks
        if !self.class.stacks() {
            return false;
        }

        // Rule 1: Same object type
        if self.object_type != other.object_type {
            return false;
        }

        // Rule 2: Same BUC status
        if self.buc != other.buc {
            return false;
        }

        // Rule 3: Same enchantment
        if self.enchantment != other.enchantment {
            return false;
        }

        // Rule 4: Same erosion1
        if self.erosion1 != other.erosion1 {
            return false;
        }

        // Rule 5: Same erosion2
        if self.erosion2 != other.erosion2 {
            return false;
        }

        // Rule 6: Same poisoned status
        if self.poisoned != other.poisoned {
            return false;
        }

        // Rule 7: Same greased status
        if self.greased != other.greased {
            return false;
        }

        // Rule 8: Same broken status
        if self.broken != other.broken {
            return false;
        }

        // Rule 9: Neither worn/wielded
        if self.is_worn() || other.is_worn() {
            return false;
        }

        // Rule 10: Same name (or both unnamed)
        if self.name != other.name {
            return false;
        }

        // Rule 11: Not artifacts
        if self.artifact != 0 || other.artifact != 0 {
            return false;
        }

        // Rule 12: Not containers with contents
        if (self.is_container() && !self.contents.is_empty())
            || (other.is_container() && !other.contents.is_empty())
        {
            return false;
        }

        // Rule 13: Not unique objects (would need access to ObjClassDef)
        // This is a simplified check - full implementation needs OBJECTS array
        // For now, assume non-artifacts aren't unique

        // Rule 14: For corpses, check age within tolerance (50 turns)
        if self.is_corpse() && other.is_corpse() {
            let age_diff = (self.age - other.age).abs();
            if age_diff > 50 {
                return false;
            }
        }

        true
    }

    /// Merge with another object, consuming it (merged equivalent concept)
    /// Returns true if merge was successful.
    pub fn merged(&mut self, other: &Object) -> bool {
        if !self.can_merge(other) {
            return false;
        }

        self.quantity += other.quantity;
        self.weight += other.weight;
        true
    }

    /// Check if this object is at the top of a pile (obj_is_piletop equivalent)
    /// In Rust/this system, pile order is determined by position in Vec
    /// Top of pile means this is the first object at a location
    pub const fn is_piletop(&self) -> bool {
        // This is a status flag that should be set when object is the top of pile
        // For now, we check if we're not marked as being under something
        // This would require a "piletop" or "underlying" flag in Object struct
        // Placeholder: always true (would need additional state to track pile order)
        true
    }

    /// Get base weight without contents (get_obj_weight equivalent)
    /// Returns weight as stored, not including container contents
    pub const fn get_obj_weight(&self) -> u32 {
        self.weight
    }

    /// Check if object is buried
    pub const fn is_buried_in_ground(&self) -> bool {
        matches!(self.location, ObjectLocation::Buried)
    }

    /// Mark object as bypassed from autopickup
    pub fn set_bypass(&mut self) {
        self.thrown = true; // Reuse thrown flag for bypass marking
    }

    /// Check if object is bypassed from autopickup
    pub const fn is_bypassed(&self) -> bool {
        self.thrown
    }

    /// Clear bypass flag
    pub fn clear_bypass(&mut self) {
        self.thrown = false;
    }

    // ========================================================================
    // Object use/consume functions (invent.c)
    // ========================================================================

    /// Use up (consume) one of this object stack (useup equivalent concept)
    /// Returns true if the entire stack was consumed.
    pub fn useup_one(&mut self) -> bool {
        if self.quantity <= 1 {
            // Entire stack is used up
            true
        } else {
            self.quantity -= 1;
            // Reduce weight proportionally
            let unit_weight = self.weight / (self.quantity + 1) as u32;
            self.weight = self.weight.saturating_sub(unit_weight);
            false
        }
    }

    /// Use up all of this object stack (useupall equivalent concept)
    /// Returns the count that was consumed.
    pub fn useup_all(&mut self) -> i32 {
        let count = self.quantity;
        self.quantity = 0;
        self.weight = 0;
        count
    }

    /// Consume a charge from this object (for wands, tools)
    /// Returns true if a charge was consumed.
    pub fn consume_charge(&mut self) -> bool {
        if self.enchantment <= 0 {
            return false;
        }
        self.enchantment -= 1;
        true
    }

    /// Check if this object has charges remaining
    pub fn has_charges(&self) -> bool {
        self.enchantment > 0
    }

    /// Get magic cancellation level (a_can equivalent)
    ///
    /// Returns the magic cancellation level for armor pieces.
    /// Magic cancellation (mc) reduces incoming spell damage.
    ///
    /// # Returns
    /// Magic cancellation level (0-3), where higher is better
    pub fn magic_cancellation(&self) -> i8 {
        // Only armor pieces grant magic cancellation
        if !self.is_armor() {
            return 0;
        }

        // Base magic cancellation depends on enchantment level
        // For simplicity: +1 per point of enchantment, capped at 3
        if self.enchantment <= 0 {
            0
        } else if self.enchantment > 3 {
            3
        } else {
            self.enchantment
        }
    }

    // ========================================================================
    // Object lifecycle functions (obj_extract_self, obfree, etc.)
    // ========================================================================

    /// Check if object is in free state (not in game)
    pub const fn is_free(&self) -> bool {
        matches!(self.location, ObjectLocation::Free)
    }

    /// Check if object is on the floor
    pub const fn is_on_floor(&self) -> bool {
        matches!(self.location, ObjectLocation::Floor)
    }

    /// Check if object is in a container
    pub const fn is_contained(&self) -> bool {
        matches!(self.location, ObjectLocation::Contained)
    }

    /// Check if object is in player inventory
    pub const fn is_in_player_inventory(&self) -> bool {
        matches!(self.location, ObjectLocation::PlayerInventory)
    }

    /// Check if object is in monster inventory
    pub const fn is_in_monster_inventory(&self) -> bool {
        matches!(self.location, ObjectLocation::MonsterInventory)
    }

    /// Check if object is being migrated between levels
    pub const fn is_migrating(&self) -> bool {
        matches!(self.location, ObjectLocation::Migrating)
    }

    /// Check if object is buried underground
    pub const fn is_buried(&self) -> bool {
        matches!(self.location, ObjectLocation::Buried)
    }

    /// Check if object is on a shop bill
    pub const fn is_on_bill(&self) -> bool {
        matches!(self.location, ObjectLocation::OnBill)
    }

    /// Mark this object as in the free state (effectively "deallocated" but in Rust we just mark location)
    /// This is equivalent to the start of obfree() in NetHack
    pub fn set_free(&mut self) {
        self.location = ObjectLocation::Free;
        self.x = 0;
        self.y = 0;
    }

    /// Mark this object as destroyed/deallocated (freed from all chains)
    /// This should only be called when the object is being removed from the game entirely
    pub fn deallocate(&mut self) {
        self.set_free();
        self.contents.clear();
        self.quantity = 0;
        self.weight = 0;
    }
}

// ============================================================================
// Object extraction and discovery functions
// ============================================================================

/// Extract an object from any location (obj_extract_self equivalent)
///
/// This removes an object from whatever container/inventory/location it's in.
/// The object will be moved to the Free state but will retain its data.
///
/// # Arguments
/// * `obj` - The object to extract
pub fn obj_extract_self(obj: &mut Object) {
    obj.set_free();
}

/// Discover an object type and add it to the discovery list (discover_object equivalent)
///
/// # Arguments
/// * `discovery_state` - Mutable reference to the discovery state
/// * `object_type` - The object type to discover
/// * `current_turn` - The current game turn
///
/// # Returns
/// true if this was a new discovery, false if already known
pub fn discover_object(
    discovery_state: &mut crate::object::DiscoveryState,
    object_type: i16,
    current_turn: u64,
) -> bool {
    discovery_state.discover_object(object_type, current_turn)
}

/// Mark an object as "makeknown" - known to the player
///
/// Sets the object's known flag and discovers its type if not already discovered.
///
/// # Arguments
/// * `obj` - The object to make known
/// * `discovery_state` - Mutable reference to the discovery state
/// * `current_turn` - The current game turn
pub fn makeknown(
    obj: &mut Object,
    discovery_state: &mut crate::object::DiscoveryState,
    current_turn: u64,
) {
    obj.known = true;
    discover_object(discovery_state, obj.object_type, current_turn);
}

// ============================================================================
// Weight calculation functions
// ============================================================================

/// Calculate recursive weight of an object, including container contents
/// Equivalent to weight() in NetHack obj.c
///
/// Handles Bag of Holding modifications:
/// - Blessed: contents_weight / 4
/// - Uncursed: contents_weight / 2
/// - Cursed: contents_weight * 2
pub fn weight(obj: &Object) -> u32 {
    if obj.is_container() {
        let base = obj.weight;

        // Calculate total weight of contents
        let contents_weight: u32 = obj.contents.iter().map(weight).sum();

        // Apply Bag of Holding modifications
        let modified_weight = if obj.object_type == 365 {
            // BAG_OF_HOLDING
            match obj.buc {
                BucStatus::Blessed => contents_weight / 4,
                BucStatus::Uncursed => contents_weight / 2,
                BucStatus::Cursed => contents_weight.saturating_mul(2),
            }
        } else {
            contents_weight
        };

        base.saturating_add(modified_weight)
    } else {
        obj.weight.saturating_mul(obj.quantity as u32)
    }
}

/// Get base weight without applying stack quantity (for single item)
pub const fn get_obj_weight_single(obj: &Object) -> u32 {
    obj.weight
}

/// Extract object from a list of objects, by index
/// Returns the extracted object if found, or None if out of bounds
pub fn extract_nobj(objects: &mut Vec<Object>, index: usize) -> Option<Object> {
    if index < objects.len() {
        Some(objects.remove(index))
    } else {
        None
    }
}

/// Find and extract next object in list at same location
/// Searches from given index forward for next object at same (x,y)
pub fn extract_nexthere(
    objects: &mut Vec<Object>,
    x: i8,
    y: i8,
    start_index: usize,
) -> Option<Object> {
    for (i, obj) in objects.iter().enumerate() {
        if i > start_index && obj.x == x && obj.y == y {
            return Some(objects.remove(i));
        }
    }
    None
}

/// Find next object in list at location
/// Returns index of next object at (x,y) after start_index
pub fn nexthere(objects: &[Object], x: i8, y: i8, start_index: usize) -> Option<usize> {
    for (i, obj) in objects.iter().enumerate() {
        if i > start_index && obj.x == x && obj.y == y {
            return Some(i);
        }
    }
    None
}

/// Count total quantity of objects at a location
/// Sums quantity field for all objects at (x,y)
pub fn curr_cnt(objects: &[Object], x: i8, y: i8) -> i32 {
    objects
        .iter()
        .filter(|obj| obj.x == x && obj.y == y)
        .map(|obj| obj.quantity)
        .sum()
}

// ============================================================================
// Object movement functions
// ============================================================================

/// Lift object from floor to inventory (lift_object equivalent)
///
/// Moves object from floor location to player inventory location
pub fn lift_object(obj: &mut Object) {
    obj.location = ObjectLocation::PlayerInventory;
    // Clear position since now in inventory
    obj.x = 0;
    obj.y = 0;
}

/// Add object to container (hold_another_object equivalent)
///
/// Moves object into container's contents
pub fn hold_another_object(container: &mut Object, mut obj: Object) -> bool {
    if !container.is_container() {
        return false;
    }

    // Move object to contained location
    obj.location = ObjectLocation::Contained;
    obj.x = 0;
    obj.y = 0;

    // Try to merge with existing stack first
    for contained in &mut container.contents {
        if contained.mergable(&obj) {
            contained.merged(&obj);
            return true;
        }
    }

    // No merge, add as new item
    container.contents.push(obj);
    true
}

// ============================================================================
// Autopickup bypass functions
// ============================================================================

/// Mark an object as bypassed (skipped by autopickup)
pub fn bypass_obj(obj: &mut Object) {
    obj.set_bypass();
}

/// Mark all objects in a list as bypassed
pub fn bypass_objlist(objects: &mut [Object]) {
    for obj in objects {
        obj.set_bypass();
    }
}

/// Clear all bypass flags from objects at a location
pub fn clear_bypasses(objects: &mut [Object]) {
    for obj in objects {
        obj.clear_bypass();
    }
}

// ============================================================================
// Object placement functions (Phase 4)
// ============================================================================

/// Place an object at a specific location (place_object equivalent)
///
/// Sets object location to Floor and positions it at (x, y).
pub fn place_object(obj: &mut Object, x: i8, y: i8) {
    obj.location = ObjectLocation::Floor;
    obj.x = x;
    obj.y = y;
}

/// Remove an object from a location (remove_object equivalent)
///
/// Sets object to Free state and clears position.
pub fn remove_object(obj: &mut Object) {
    obj.location = ObjectLocation::Free;
    obj.x = 0;
    obj.y = 0;
}

/// Scatter objects in a radius from a center point (obj_scatter equivalent)
///
/// Distributes objects randomly around a center location.
///
/// # Arguments
/// * `objects` - Vec of objects to scatter
/// * `center_x` - Center X coordinate
/// * `center_y` - Center Y coordinate
/// * `radius` - Scatter radius
/// * `rng` - Random number generator
///
/// # Returns
/// Vec of scattered objects with updated positions
pub fn obj_scatter(
    objects: Vec<Object>,
    center_x: i8,
    center_y: i8,
    radius: i8,
    rng: &mut crate::rng::GameRng,
) -> Vec<Object> {
    objects
        .into_iter()
        .map(|mut obj| {
            // Random offset within radius
            let dx = (rng.rn2(radius as u32 * 2 + 1) as i8) - radius;
            let dy = (rng.rn2(radius as u32 * 2 + 1) as i8) - radius;

            let new_x = (center_x + dx).max(0).min(79); // Map bounds
            let new_y = (center_y + dy).max(0).min(20); // Map bounds

            place_object(&mut obj, new_x, new_y);
            obj
        })
        .collect()
}

/// Check if object should have floor effects applied (flooreffects equivalent)
///
/// Returns true if object can be affected by current floor type.
pub fn flooreffects(obj: &Object, floor_type: u8) -> bool {
    match floor_type {
        1 => {
            // Fire floor - affects scrolls, books, food
            obj.class == ObjectClass::Scroll
                || obj.class == ObjectClass::Spellbook
                || obj.class == ObjectClass::Food
        }
        2 => {
            // Acid floor - affects metal/armor
            obj.class == ObjectClass::Armor || obj.class == ObjectClass::Weapon
        }
        3 => true, // Water - most objects affected
        _ => false,
    }
}

/// Check if two locations are adjacent
pub fn is_adjacent(x1: i8, y1: i8, x2: i8, y2: i8) -> bool {
    (x1 - x2).abs() <= 1 && (y1 - y2).abs() <= 1 && (x1, y1) != (x2, y2)
}

/// Check if object is visible from a location
pub fn is_visible_from(obj_x: i8, obj_y: i8, viewer_x: i8, viewer_y: i8, sight_range: i8) -> bool {
    (obj_x - viewer_x).abs() <= sight_range && (obj_y - viewer_y).abs() <= sight_range
}

// ============================================================================
// Artifact functions
// ============================================================================

use crate::combat::DamageType;

/// Check if object is an artifact with the specified attack damage type.
///
/// This checks if the artifact has special attack properties matching the given damage type.
///
/// # Arguments
/// * `obj` - The object to check
/// * `damage_type` - The damage type to look for
///
/// # Returns
/// true if the object is an artifact with this damage type, false otherwise
pub fn attacks(obj: &Object, damage_type: DamageType) -> bool {
    if !obj.is_artifact() {
        return false;
    }

    // Would need access to artifact definitions from nh-data crate
    // For now, return false as placeholder - this needs nh-data integration
    let _damage_type = damage_type;
    false
}

/// Check if object confers magical Protection.
///
/// Checks both base object properties and artifact flags.
///
/// # Arguments
/// * `_obj` - The object to check
/// * `_being_worn` - Whether the object is currently being worn
///
/// # Returns
/// true if object grants magical Protection, false otherwise
pub fn protects(_obj: &Object, _being_worn: bool) -> bool {
    // Worn artifacts with PROTECT flag would grant protection
    // This is a simplified implementation that returns false as placeholder
    false
}

// ============================================================================
// Grease protection
// ============================================================================

/// Handle grease protection effect.
///
/// When a greased object is hit, it may protect from the attack and/or dissolve.
///
/// # Arguments
/// * `obj` - The object to check for grease
/// * `rng` - Random number generator for dissolution chance
///
/// # Returns
/// true if grease dissolved, false if still greased
pub fn grease_protect(obj: &mut Object, rng: &mut crate::rng::GameRng) -> bool {
    if !obj.greased {
        return false;
    }

    // 50% chance grease dissolves
    if rng.one_in(2) {
        obj.greased = false;
        return true;
    }

    false
}

// ============================================================================
// Shop Integration (Phase 5)
// ============================================================================

/// Calculate the value/price of an object.
///
/// Determines the shop price or intrinsic value of an object based on its
/// type, enchantment, erosion, and other properties.
///
/// # Arguments
/// * `obj` - The object to valuate
///
/// # Returns
/// The value in gold pieces
pub fn obj_value(obj: &Object) -> i32 {
    if obj.shop_price > 0 {
        return obj.shop_price;
    }

    // Base value depends on object class
    let mut value = match obj.class {
        ObjectClass::Weapon => 10 + (obj.damage_dice as i32 * obj.damage_sides as i32),
        ObjectClass::Armor => 15,
        ObjectClass::Ring => 100,
        ObjectClass::Amulet => 150,
        ObjectClass::Wand => 50,
        ObjectClass::Scroll => 20,
        ObjectClass::Potion => 20,
        ObjectClass::Spellbook => 100,
        ObjectClass::Food => 5,
        ObjectClass::Tool => 10,
        ObjectClass::Coin => 1,
        ObjectClass::Gem => 50,
        _ => 10,
    };

    // Quantity multiplier for stackable items
    if obj.quantity > 1 {
        value *= obj.quantity;
    }

    // Enchantment bonus (positive enchantments increase value)
    if obj.enchantment > 0 {
        value += (obj.enchantment as i32) * 50;
    }

    // Erosion penalty (reduce value by 25% per erosion level)
    let total_erosion = (obj.erosion1.max(obj.erosion2)) as i32;
    for _ in 0..total_erosion {
        value = value * 3 / 4; // 25% reduction each level
    }

    // Apply blessing/curse modifiers
    match obj.buc {
        BucStatus::Blessed => value = (value * 125) / 100, // 25% premium
        BucStatus::Cursed => value = (value * 75) / 100,   // 25% discount
        BucStatus::Uncursed => {}
    }

    value.max(1) // Minimum value of 1
}

/// Check if an object can be on a shop bill.
///
/// Items that can be billed to a character.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if the object can be billed, false otherwise
pub fn can_be_billed(obj: &Object) -> bool {
    // Coins, identified items, and stackable goods can be billed
    // Some items like artifacts cannot be billed easily
    !obj.is_artifact() && !matches!(obj.class, ObjectClass::Coin)
}

/// Mark an object as on a shop bill.
///
/// # Arguments
/// * `obj` - The object to bill
/// * `price` - The price per unit
pub fn addtobill(obj: &mut Object, price: i32) {
    obj.location = ObjectLocation::OnBill;
    obj.shop_price = price;
    obj.unpaid = true;
}

/// Remove an object from a shop bill.
///
/// # Arguments
/// * `obj` - The object to remove from bill
pub fn subfrombill(obj: &mut Object) {
    obj.unpaid = false;
    if obj.location == ObjectLocation::OnBill {
        obj.location = ObjectLocation::Free;
    }
}

/// Check if a position is in a shop (costly spot).
///
/// Used to determine if a location is inside a shop room where items
/// are automatically billed to the player.
///
/// # Arguments
/// * `x` - X coordinate
/// * `y` - Y coordinate
/// * `level` - The current dungeon level
///
/// # Returns
/// true if the position is in a shop, false otherwise
pub fn costly_spot(x: i8, y: i8, level: &crate::dungeon::Level) -> bool {
    if !level.is_valid_pos(x, y) {
        return false;
    }

    let cell = &level.cells[x as usize][y as usize];

    // A costly spot is a room cell (CellType::Room) with a valid room number.
    // In NetHack, shops are rooms and customers are billed when walking around.
    // The actual shop-specific logic would check room types against the dungeon
    // room array, but for now we identify shops by checking if it's a room cell.
    // This is a simplified check that would be enhanced with full room tracking.
    matches!(cell.typ, crate::dungeon::CellType::Room) && cell.room_number > 0
}

/// Handle payment of a shop bill.
///
/// Processes payment for unpaid items from a shop. Deducts the amount from
/// the player's gold and marks items as paid.
///
/// # Arguments
/// * `objects` - Objects to check for unpaid items
/// * `amount_paid` - Amount of gold paid
/// * `player_gold` - Mutable reference to player's gold amount
///
/// # Returns
/// The amount paid, or 0 if insufficient funds
pub fn pay_bill(objects: &mut [Object], amount_paid: i32, player_gold: &mut i32) -> i32 {
    if amount_paid <= 0 || *player_gold < amount_paid {
        return 0; // Can't pay
    }

    let mut total_due = 0;

    // Calculate total amount due
    for obj in objects.iter() {
        if obj.unpaid && obj.location == ObjectLocation::OnBill {
            total_due += obj.shop_price * obj.quantity;
        }
    }

    if total_due <= 0 {
        return 0; // Nothing to pay
    }

    let actual_payment = amount_paid.min(total_due);

    // Deduct payment from player's gold
    *player_gold -= actual_payment;

    // Mark items as paid (simplified - in real game would keep track of partial payments)
    let mut remaining = actual_payment;
    for obj in objects.iter_mut() {
        if obj.unpaid && obj.location == ObjectLocation::OnBill && remaining > 0 {
            let obj_cost = obj.shop_price * obj.quantity;
            if remaining >= obj_cost {
                subfrombill(obj);
                remaining -= obj_cost;
            }
        }
    }

    actual_payment
}

/// Track shopkeeper movement after transaction.
///
/// Updates shopkeeper position based on interaction. In NetHack, shopkeepers
/// may follow the player or move away based on transaction type.
///
/// # Arguments
/// * `shopkeeper_id` - The shopkeeper monster ID
/// * `transaction_type` - Type of transaction (0=purchase, 1=theft, 2=payment)
/// * `amount` - Amount involved in transaction
///
/// # Returns
/// The shopkeeper's mood change (-10 to +10, negative = angrier)
pub fn shk_move(_shopkeeper_id: i32, transaction_type: i32, amount: i32) -> i32 {
    // Mood change based on transaction
    match transaction_type {
        0 => {
            // Purchase - happy if big sale
            if amount > 100 {
                5
            } else if amount > 10 {
                2
            } else {
                0
            }
        }
        1 => {
            // Theft - very angry
            -10
        }
        2 => {
            // Payment - happy
            if amount > 50 {
                5
            } else if amount > 10 {
                3
            } else {
                1
            }
        }
        _ => 0,
    }
}

// ============================================================================
// Light Source Management (Phase 5)
// ============================================================================

/// Begin burning a light source.
///
/// Marks an object as lit and activates its light source effect.
///
/// # Arguments
/// * `obj` - The light source object
pub fn begin_burn(obj: &mut Object) {
    if matches!(
        obj.class,
        ObjectClass::Tool | ObjectClass::Potion | ObjectClass::Wand
    ) {
        obj.lit = true;
        obj.age = 0; // Reset age for burn tracking
    }
}

/// End burning of a light source.
///
/// Extinguishes a light source and removes its lighting effects.
///
/// # Arguments
/// * `obj` - The light source object
pub fn end_burn(obj: &mut Object) {
    obj.lit = false;
}

/// Check if an object emits light.
///
/// Determines if the object provides ambient light when lit.
/// Different from `obj.lit` which tracks if it IS lit.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if object can emit light, false otherwise
pub fn obj_sheds_light(obj: &Object) -> bool {
    if !obj.lit {
        return false;
    }

    match obj.class {
        ObjectClass::Tool => {
            // Tools that emit light: lamps, candles, lanterns
            // Check object type or special flags
            true // Simplified: all lit tools emit light
        }
        ObjectClass::Potion => {
            // Luminous potions (potion of enlightenment, etc.)
            // Would check object_type
            obj.enchantment > 0 // Magic potions emit light
        }
        ObjectClass::Wand => {
            // Some wands emit light
            obj.enchantment > 0
        }
        _ => false,
    }
}

/// Get the light radius for a lit object.
///
/// # Arguments
/// * `obj` - The lit object
///
/// # Returns
/// The radius of light in cells (0 if not a light source)
pub fn light_radius(obj: &Object) -> i32 {
    if !obj_sheds_light(obj) {
        return 0;
    }

    match obj.class {
        ObjectClass::Tool => {
            // Different light radii based on enchantment
            if obj.enchantment >= 2 {
                5 // Magic lantern
            } else if obj.enchantment == 1 {
                3 // Enchanted lamp
            } else {
                2 // Regular candle/lamp
            }
        }
        ObjectClass::Potion => 2, // Luminous potions
        ObjectClass::Wand => {
            if obj.enchantment > 0 {
                3
            } else {
                0
            }
        }
        _ => 0,
    }
}

/// Check if a torch-like object is shedding light.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if torch is lit and functional
pub fn torch_sheds_light(obj: &Object) -> bool {
    if obj.class != ObjectClass::Tool {
        return false;
    }

    obj.lit && obj.erosion1 == 0 // Torch burns out with erosion
}

/// Check if a candle-like object is shedding light.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if candle is lit and functional
pub fn candle_sheds_light(obj: &Object) -> bool {
    if obj.class != ObjectClass::Tool {
        return false;
    }

    // Candles need to be lit and not fully burnt (eroded)
    obj.lit && obj.erosion1 < 3
}

/// Get all object types that can emit light.
///
/// # Returns
/// Vector of object type indices that can emit light
pub fn light_emitting_objs() -> Vec<i16> {
    vec![
        // Tool types (lamps, candles, lanterns)
        // These would be actual object type indices from the OBJECTS array
        // For now, using placeholder values
        45,  // LAMP
        46,  // CANDLE
        47,  // LANTERN
        100, // POTION_OF_ENLIGHTENMENT
        200, // MAGIC_LAMP
    ]
}

/// Snuff all burning light sources in an area.
///
/// Extinguishes all lit objects in the given radius.
///
/// # Arguments
/// * `objects` - List of objects to check and possibly extinguish
/// * `x` - X coordinate of effect center
/// * `y` - Y coordinate of effect center
/// * `radius` - Radius of effect
pub fn snuff_candles(objects: &mut [Object], x: i8, y: i8, radius: i8) {
    for obj in objects {
        // Only snuff objects within radius
        if obj.location == ObjectLocation::Floor {
            let dx = (obj.x - x).abs();
            let dy = (obj.y - y).abs();

            if dx <= radius && dy <= radius {
                if obj.lit && matches!(obj.class, ObjectClass::Tool) {
                    end_burn(obj);
                }
            }
        }
    }
}

/// Check if an object sheds light anywhere (on floor, in inventory, etc).
///
/// Similar to obj_sheds_light but doesn't require lit flag necessarily.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if the object can shed light in its current state
pub fn obj_sheds_light_anywhere(obj: &Object) -> bool {
    if !obj.lit {
        return false;
    }

    matches!(
        obj.class,
        ObjectClass::Tool | ObjectClass::Potion | ObjectClass::Wand
    ) && light_radius(obj) > 0
}

// ============================================================================
// Materials & Erosion (Phase 6)
// ============================================================================

/// Check if an object's material is flammable.
///
/// Objects made of flammable materials can burn when exposed to fire.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if the object's material can burn, false otherwise
pub fn is_flammable(obj: &Object) -> bool {
    // Would need access to object type material mapping from OBJECTS array
    // For now, check by object class
    matches!(
        obj.class,
        ObjectClass::Scroll | ObjectClass::Spellbook | ObjectClass::Food | ObjectClass::Tool
    )
}

/// Check if an object's material can rot.
///
/// Objects made of rottable materials decay over time when not protected.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if the object's material can rot, false otherwise
pub fn is_rottable(obj: &Object) -> bool {
    // Organic materials can rot
    matches!(
        obj.class,
        ObjectClass::Food | ObjectClass::Armor | ObjectClass::Tool
    ) && obj.erosion_proof == false
}

/// Check if an object's material is rust-prone.
///
/// Objects made of iron can rust when exposed to rust or water.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if the object's material can rust, false otherwise
pub fn is_rustprone(obj: &Object) -> bool {
    // Iron and some weapons rust
    matches!(
        obj.class,
        ObjectClass::Weapon | ObjectClass::Armor | ObjectClass::Tool
    ) && obj.erosion_proof == false
}

/// Check if an object's material can corrode.
///
/// Objects made of copper or iron can corrode when exposed to acid.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if the object's material can corrode, false otherwise
pub fn is_corrodeable(obj: &Object) -> bool {
    // Metal objects can corrode
    matches!(
        obj.class,
        ObjectClass::Weapon | ObjectClass::Armor | ObjectClass::Tool
    ) && obj.erosion_proof == false
}

/// Check if an object can take any erosion damage.
///
/// Generally true for most objects unless they're protected or indestructible.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if the object can be damaged by erosion, false otherwise
pub fn is_damageable(obj: &Object) -> bool {
    !obj.is_artifact() && !obj.erosion_proof
}

/// Check if two objects are made of the same material.
///
/// # Arguments
/// * `obj1` - First object
/// * `obj2` - Second object
///
/// # Returns
/// true if both objects have the same material type, false otherwise
pub fn objects_are_same_material(obj1: &Object, obj2: &Object) -> bool {
    // Simplified: same object class indicates same material family
    // In real NetHack, would check OBJECTS array material field
    obj1.class == obj2.class
}

/// Check if an object's material is flammable (by material type).
///
/// This is a material-level check, independent of object class.
///
/// # Arguments
/// * `material` - The material to check
///
/// # Returns
/// true if the material is flammable, false otherwise
pub fn obj_material_is_flammable(material: ObjectClass) -> bool {
    matches!(
        material,
        ObjectClass::Scroll | ObjectClass::Spellbook | ObjectClass::Food | ObjectClass::Tool
    )
}

/// Check if an object's material is rust-prone (by material type).
///
/// This is a material-level check, independent of object class.
///
/// # Arguments
/// * `material` - The material to check
///
/// # Returns
/// true if the material rusts, false otherwise
pub fn obj_material_is_rustprone(material: ObjectClass) -> bool {
    matches!(
        material,
        ObjectClass::Weapon | ObjectClass::Armor | ObjectClass::Tool
    )
}

/// Check if an object resists a particular type of damage.
///
/// Objects may resist erosion based on blessing, material, or special properties.
///
/// # Arguments
/// * `obj` - The object to check
/// * `damage_type` - Type of damage (1=fire/rust, 2=acid/corrode, 3=water/rot)
///
/// # Returns
/// Resistance percentage (0-100, where 100 = immune)
pub fn obj_resists(obj: &Object, damage_type: i32) -> i32 {
    // Base resistance from blessing
    let mut resistance = match obj.buc {
        BucStatus::Blessed => 75,  // 75% chance to resist
        BucStatus::Uncursed => 25, // 25% chance to resist
        BucStatus::Cursed => 0,    // 0% resistance, actually takes more damage
    };

    // Greased objects have some protection
    if obj.greased {
        resistance += 25;
    }

    // Artifacts always resist
    if obj.is_artifact() {
        resistance = 100;
    }

    // Erosion-proof items are immune
    if obj.erosion_proof {
        resistance = 100;
    }

    // Cap at 100%
    resistance.min(100)
}

/// Apply erosion damage to an object.
///
/// Increases erosion level based on damage type and object resistance.
///
/// # Arguments
/// * `obj` - The object to erode
/// * `damage_type` - Type of damage (1=fire/rust, 2=acid/corrode, 3=water/rot)
/// * `rng` - Random number generator for resistance roll
pub fn erode_obj(obj: &mut Object, damage_type: i32, rng: &mut crate::rng::GameRng) {
    if !is_damageable(obj) {
        return;
    }

    // Check if object resists this damage
    let resistance = obj_resists(obj, damage_type);
    if rng.rn2(100) < resistance as u32 {
        return; // Resisted!
    }

    // Apply erosion
    match damage_type {
        1 => {
            // Fire/rust damage → erosion1
            if is_rustprone(obj) || is_flammable(obj) {
                if obj.erosion1 < 3 {
                    obj.erosion1 += 1;
                }
            }
        }
        2 => {
            // Acid/corrode → erosion2
            if is_corrodeable(obj) {
                if obj.erosion2 < 3 {
                    obj.erosion2 += 1;
                }
            }
        }
        3 => {
            // Water/rot → erosion1
            if is_rottable(obj) {
                if obj.erosion1 < 3 {
                    obj.erosion1 += 1;
                }
            }
        }
        _ => {} // Unknown damage type
    }
}

/// Check if an object is completely destroyed by erosion.
///
/// Objects at maximum erosion (3) are destroyed and unusable.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if the object is fully eroded, false otherwise
pub fn obj_is_destroyed(obj: &Object) -> bool {
    obj.erosion1 >= 3 || obj.erosion2 >= 3
}

/// Get the material name for an object as a string.
///
/// # Arguments
/// * `material` - The material/class to get name for
///
/// # Returns
/// A string describing the material
pub fn obj_name_from_material(material: ObjectClass) -> &'static str {
    match material {
        ObjectClass::Weapon => "iron",
        ObjectClass::Armor => "leather",
        ObjectClass::Tool => "metal",
        ObjectClass::Food => "organic",
        ObjectClass::Scroll => "paper",
        ObjectClass::Spellbook => "leather",
        ObjectClass::Potion => "glass",
        ObjectClass::Wand => "wood",
        ObjectClass::Ring => "silver",
        ObjectClass::Amulet => "metal",
        ObjectClass::Gem => "mineral",
        ObjectClass::Rock => "stone",
        _ => "material",
    }
}

/// Calculate rust damage to an object.
///
/// Determines how much rust reduces an object's effectiveness.
///
/// # Arguments
/// * `erosion_level` - The erosion level (0-3)
///
/// # Returns
/// Damage percentage (0-100, where 100 = completely destroyed)
pub fn rust_dmg(erosion_level: u8) -> i32 {
    match erosion_level {
        0 => 0,   // No damage
        1 => 25,  // 25% damage
        2 => 50,  // 50% damage
        3 => 100, // Completely destroyed
        _ => 100, // Cap at destroyed
    }
}

// ============================================================================
// Polish & Edge Cases (Phase 7)
// ============================================================================

/// Handle object destruction when no longer held.
///
/// Called when an object is destroyed while being held by player or monster.
/// May drop it on the ground or remove it entirely.
///
/// # Arguments
/// * `obj` - The object being destroyed
///
/// # Returns
/// true if object was successfully handled, false if still present
pub fn obj_no_longer_held(obj: &mut Object) -> bool {
    if obj_is_destroyed(obj) {
        // Completely destroyed objects are removed
        obj.location = ObjectLocation::Free;
        return true;
    }

    // Otherwise, object may drop on ground
    false
}

/// Drop an object at a specific location on the floor.
///
/// Places an object on the floor at given coordinates with proper location tracking.
///
/// # Arguments
/// * `obj` - The object to drop
/// * `x` - X coordinate
/// * `y` - Y coordinate
pub fn dropx(obj: &mut Object, x: i8, y: i8) {
    place_object(obj, x, y);
}

/// Drop an object at the player's current location.
///
/// Shortcut for dropping an object where the player stands.
///
/// # Arguments
/// * `obj` - The object to drop
/// * `player_x` - Player's X coordinate
/// * `player_y` - Player's Y coordinate
pub fn dropy(obj: &mut Object, player_x: i8, player_y: i8) {
    place_object(obj, player_x, player_y);
}

/// Handle objects falling off hero due to polymorph or transformation.
///
/// When player polymorphs into a form that can't hold equipment, items fall.
///
/// # Arguments
/// * `obj` - The object falling
pub fn obj_falls_off_hero(obj: &mut Object) {
    // Object becomes unworn when it falls off
    obj.worn_mask = 0;
}

/// Adjust light radius for a lit object.
///
/// Updates the light radius when enchantment changes or object state changes.
///
/// # Arguments
/// * `obj` - The object emitting light
///
/// # Returns
/// The new light radius
pub fn obj_adjust_light_radius(obj: &Object) -> i32 {
    light_radius(obj)
}

/// Get the current light radius for an object.
///
/// Alias for light_radius to query current light emission range.
///
/// # Arguments
/// * `obj` - The lit object
///
/// # Returns
/// Light radius in cells (0 if not emitting light)
pub fn obj_sheds_light_radius(obj: &Object) -> i32 {
    light_radius(obj)
}

/// Check if an object reflects damage or effects.
///
/// Certain objects (mirrors, shiny armor) can reflect spells or beams.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if object can reflect, false otherwise
pub fn obj_reflects(obj: &Object) -> bool {
    // Mirrors and reflective armor reflect
    match obj.class {
        ObjectClass::Tool => {
            // Mirror (object type would be checked normally)
            false // Placeholder
        }
        ObjectClass::Armor => {
            // Shiny armor (plate mail, mithril) reflects
            obj.enchantment > 0
        }
        _ => false,
    }
}

/// Check if an object is currently in use.
///
/// An object is in use if it's being wielded, worn, or actively used.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if object is actively in use, false otherwise
pub fn obj_is_in_use(obj: &Object) -> bool {
    obj.in_use || obj.worn_mask > 0
}

/// Check if an object is at the top of a pile at a location.
///
/// For objects on the floor, checks if this is the topmost object.
///
/// # Arguments
/// * `obj` - The object to check
/// * `x` - X coordinate
/// * `y` - Y coordinate
/// * `all_objects` - All objects to search
///
/// # Returns
/// true if object is the topmost at that location
pub fn obj_is_piletop_at(obj: &Object, x: i8, y: i8, all_objects: &[Object]) -> bool {
    if obj.location != ObjectLocation::Floor || obj.x != x || obj.y != y {
        return false;
    }

    // Check if any other object at same location comes after this one
    let mut found_self = false;
    for other in all_objects {
        if other.location == ObjectLocation::Floor && other.x == x && other.y == y {
            if other.id == obj.id {
                found_self = true;
            } else if found_self {
                // Found another object after self, so self is not on top
                return false;
            }
        }
    }

    found_self
}

/// Remap an object's material due to magical transformation.
///
/// Changes an object's material type (e.g., stone to flesh by polymorph).
///
/// # Arguments
/// * `obj` - The object to transform
/// * `new_material` - The new material/class to assign
pub fn remap_obj_material(obj: &mut Object, new_material: ObjectClass) {
    obj.class = new_material;
}

/// Handle touch-sensitive object effects.
///
/// Called when an object is touched or picked up. May trigger cursed item effects.
///
/// # Arguments
/// * `obj` - The object being touched
///
/// # Returns
/// true if touch had an effect, false otherwise
pub fn retouch_object(obj: &mut Object) -> bool {
    // Cursed items may trigger effects when touched
    if obj.buc == BucStatus::Cursed && !obj.buc_known {
        // First touch of unknown cursed item: reveal it
        obj.buc_known = true;
        return true;
    }

    false
}

/// Handle touch effects for all worn equipment.
///
/// Applies touch detection to currently worn items.
///
/// # Arguments
/// * `worn_objects` - List of objects being worn
///
/// # Returns
/// Number of items that had touch effects triggered
pub fn retouch_equipment(worn_objects: &mut [Object]) -> usize {
    let mut count = 0;

    for obj in worn_objects {
        if obj.worn_mask > 0 && retouch_object(obj) {
            count += 1;
        }
    }

    count
}

/// Deselect all objects in a list.
///
/// Clears selection marks from all objects.
///
/// # Arguments
/// * `objects` - Objects to deselect
pub fn select_off(objects: &mut [Object]) {
    for obj in objects {
        obj.in_use = false;
    }
}

/// Select a weapon for active use.
///
/// Mark a weapon as the active wielded weapon.
///
/// # Arguments
/// * `weapon` - The weapon to select
pub fn select_hwep(weapon: &mut Object) {
    if weapon.class == ObjectClass::Weapon {
        weapon.in_use = true;
    }
}

// ============================================================================
// Object Property Functions (from mkobj.c / objnam.c)
// ============================================================================

/// Check if an object has a proper name (obj_is_pname equivalent).
///
/// A proper name is a unique name given to the object (like an artifact name
/// or a user-assigned name that should be treated as a proper noun).
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if the object has a proper name
pub fn obj_is_pname(obj: &Object) -> bool {
    // Artifacts always have proper names
    if obj.artifact > 0 {
        return true;
    }
    // Objects with custom names that start with uppercase are proper names
    if let Some(ref name) = obj.name {
        if let Some(first_char) = name.chars().next() {
            return first_char.is_uppercase();
        }
    }
    false
}

/// Check if an object is cursed (cursed equivalent).
///
/// Simple helper to check BUC status.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if the object is cursed
pub fn cursed(obj: &Object) -> bool {
    obj.buc == BucStatus::Cursed
}

/// Get curse description text (cursetxt equivalent).
///
/// Returns appropriate text describing the curse state for messages.
///
/// # Arguments
/// * `obj` - The object to describe
///
/// # Returns
/// Description string for the curse state
pub fn cursetxt(obj: &Object) -> &'static str {
    match obj.buc {
        BucStatus::Cursed => "cursed",
        BucStatus::Blessed => "blessed",
        BucStatus::Uncursed => "uncursed",
    }
}

/// Randomly curse an object (rndcurse equivalent).
///
/// Has a chance to curse the object. Used for traps and negative effects.
///
/// # Arguments
/// * `obj` - The object to potentially curse
/// * `rng` - Random number generator
///
/// # Returns
/// true if the object was cursed, false otherwise
pub fn rndcurse(obj: &mut Object, rng: &mut crate::rng::GameRng) -> bool {
    // Don't curse coins
    if obj.class == ObjectClass::Coin {
        return false;
    }

    // Already cursed
    if obj.buc == BucStatus::Cursed {
        return false;
    }

    // 50% chance to curse
    if rng.rn2(2) == 0 {
        obj.buc = BucStatus::Cursed;
        true
    } else {
        false
    }
}

/// Set the BUC known flag (set_bknown equivalent).
///
/// Marks whether the player knows the BUC status of this object.
///
/// # Arguments
/// * `obj` - The object to update
/// * `known` - Whether BUC status is known
pub fn set_bknown(obj: &mut Object, known: bool) {
    obj.buc_known = known;
}

/// Get object material type (o_material equivalent).
///
/// Returns the material of the object based on its class.
/// In full implementation, this would look up the ObjClassDef.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// Material type for the object
pub fn o_material(obj: &Object) -> crate::object::Material {
    // Simplified material lookup based on class
    // Full implementation would use ObjClassDef lookup
    match obj.class {
        ObjectClass::Weapon => crate::object::Material::Iron,
        ObjectClass::Armor => crate::object::Material::Iron,
        ObjectClass::Ring => crate::object::Material::Gold,
        ObjectClass::Amulet => crate::object::Material::Iron,
        ObjectClass::Tool => crate::object::Material::Iron,
        ObjectClass::Food => crate::object::Material::Flesh,
        ObjectClass::Potion => crate::object::Material::Glass,
        ObjectClass::Scroll => crate::object::Material::Paper,
        ObjectClass::Spellbook => crate::object::Material::Paper,
        ObjectClass::Wand => crate::object::Material::Wood,
        ObjectClass::Coin => crate::object::Material::Gold,
        ObjectClass::Gem => crate::object::Material::Gemstone,
        ObjectClass::Rock => crate::object::Material::Mineral,
        ObjectClass::Ball => crate::object::Material::Iron,
        ObjectClass::Chain => crate::object::Material::Iron,
        ObjectClass::Venom => crate::object::Material::Liquid,
        _ => crate::object::Material::Iron,
    }
}

/// Get object nutrition value (obj_nutrition equivalent).
///
/// Returns the nutrition value for food items.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// Nutrition value (0 for non-food items)
pub fn obj_nutrition(obj: &Object) -> u16 {
    if obj.class != ObjectClass::Food {
        return 0;
    }

    // Base nutrition values by food type
    // In full implementation, would look up from ObjClassDef
    // For corpses, nutrition depends on monster size
    if obj.corpse_type >= 0 {
        // Corpse nutrition based on monster (simplified)
        return 100; // Default corpse nutrition
    }

    // Default food nutrition
    50
}

/// Check if an object can be split (splittable equivalent).
///
/// Objects can be split if they are stackable and have quantity > 1.
///
/// # Arguments
/// * `obj` - The object to check
///
/// # Returns
/// true if the object can be split
pub fn splittable(obj: &Object) -> bool {
    obj.quantity > 1 && obj.contents.is_empty() && obj.class.stacks()
}

/// Check if two objects should merge (merge_choice equivalent).
///
/// Determines if two objects are compatible for stacking.
///
/// # Arguments
/// * `obj1` - First object
/// * `obj2` - Second object
///
/// # Returns
/// true if objects can merge
pub fn merge_choice(obj1: &Object, obj2: &Object) -> bool {
    // Must be same type
    if obj1.object_type != obj2.object_type {
        return false;
    }

    // Must be stackable class
    if !obj1.class.stacks() {
        return false;
    }

    // Containers with contents can't merge
    if !obj1.contents.is_empty() || !obj2.contents.is_empty() {
        return false;
    }

    // Must have same BUC status
    if obj1.buc != obj2.buc {
        return false;
    }

    // Must have same enchantment
    if obj1.enchantment != obj2.enchantment {
        return false;
    }

    // Must have same erosion
    if obj1.erosion1 != obj2.erosion1 || obj1.erosion2 != obj2.erosion2 {
        return false;
    }

    // Must have same name
    if obj1.name != obj2.name {
        return false;
    }

    // Must have same poisoned status (for weapons)
    if obj1.poisoned != obj2.poisoned {
        return false;
    }

    true
}

/// Stack an object with similar objects in a list (stackobj equivalent).
///
/// Attempts to merge the object with an existing stack in the list.
///
/// # Arguments
/// * `obj` - The object to stack
/// * `objects` - List of objects to check for stacking
///
/// # Returns
/// Some(index) if merged with object at index, None if not merged
pub fn stackobj(obj: &Object, objects: &mut Vec<Object>) -> Option<usize> {
    for (idx, existing) in objects.iter_mut().enumerate() {
        if merge_choice(obj, existing) {
            existing.quantity += obj.quantity;
            existing.weight += obj.weight;
            return Some(idx);
        }
    }
    None
}

/// Find an object by ID in a list (find_oid equivalent).
///
/// # Arguments
/// * `objects` - List of objects to search
/// * `id` - Object ID to find
///
/// # Returns
/// Index of the object if found
pub fn find_oid(objects: &[Object], id: ObjectId) -> Option<usize> {
    objects.iter().position(|obj| obj.id == id)
}

/// Find an object at a specific location (sobj_at equivalent).
///
/// # Arguments
/// * `objects` - List of objects to search
/// * `object_type` - Type of object to find
/// * `x` - X coordinate
/// * `y` - Y coordinate
///
/// # Returns
/// Index of the first matching object
pub fn sobj_at(objects: &[Object], object_type: i16, x: i8, y: i8) -> Option<usize> {
    objects.iter().position(|obj| {
        obj.object_type == object_type
            && obj.x == x
            && obj.y == y
            && obj.location == ObjectLocation::Floor
    })
}

/// Find gold at a specific location (g_at equivalent).
///
/// # Arguments
/// * `objects` - List of objects to search
/// * `x` - X coordinate
/// * `y` - Y coordinate
///
/// # Returns
/// Index of gold object if found
pub fn g_at(objects: &[Object], x: i8, y: i8) -> Option<usize> {
    objects.iter().position(|obj| {
        obj.class == ObjectClass::Coin
            && obj.x == x
            && obj.y == y
            && obj.location == ObjectLocation::Floor
    })
}

/// Check if there's an object at a location (obj_here equivalent).
///
/// # Arguments
/// * `objects` - List of objects to search
/// * `object_type` - Type of object to find
/// * `x` - X coordinate
/// * `y` - Y coordinate
///
/// # Returns
/// true if matching object exists at location
pub fn obj_here(objects: &[Object], object_type: i16, x: i8, y: i8) -> bool {
    sobj_at(objects, object_type, x, y).is_some()
}

/// Get next object in chain at same location (nxtobj equivalent).
///
/// # Arguments
/// * `objects` - List of objects
/// * `start_idx` - Index to start searching from
/// * `object_type` - Type of object to find
///
/// # Returns
/// Index of next matching object after start_idx
pub fn nxtobj(objects: &[Object], start_idx: usize, object_type: i16) -> Option<usize> {
    for (idx, obj) in objects.iter().enumerate() {
        if idx > start_idx && obj.object_type == object_type {
            return Some(idx);
        }
    }
    None
}

/// Check if object resists a damage type (obj_resists from mkobj.c).
///
/// Determines if an object can resist destruction from various damage types.
///
/// # Arguments
/// * `obj` - The object to check
/// * `damage_type` - Type of damage (0=fire, 1=cold, 2=shock, 3=acid)
///
/// # Returns
/// true if object resists the damage
pub fn obj_resists_damage(obj: &Object, damage_type: i32) -> bool {
    // Artifacts always resist
    if obj.artifact > 0 {
        return true;
    }

    // Blessed items have better resistance
    let base_resist = match obj.buc {
        BucStatus::Blessed => 75,
        BucStatus::Uncursed => 50,
        BucStatus::Cursed => 25,
    };

    // Material-based resistance
    let material = o_material(obj);
    let material_resist = match (damage_type, material) {
        (0, crate::object::Material::Iron) => 90,  // Fire vs metal
        (0, crate::object::Material::Gold) => 100, // Fire vs gold
        (1, _) => 50,                              // Cold
        (2, crate::object::Material::Iron) => 25,  // Shock vs metal
        (3, crate::object::Material::Glass) => 0,  // Acid vs glass
        _ => 50,
    };

    // Combined resistance check
    (base_resist + material_resist) / 2 > 50
}

/// Check if cursed object is at location (cursed_object_at equivalent).
///
/// # Arguments
/// * `objects` - List of objects
/// * `x` - X coordinate
/// * `y` - Y coordinate
///
/// # Returns
/// true if there's a cursed object at the location
pub fn cursed_object_at(objects: &[Object], x: i8, y: i8) -> bool {
    objects.iter().any(|obj| {
        obj.x == x
            && obj.y == y
            && obj.location == ObjectLocation::Floor
            && obj.buc == BucStatus::Cursed
    })
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

    // ========================================================================
    // Phase 1 Tests: Object Lifecycle & Discovery
    // ========================================================================

    #[test]
    fn test_object_location_queries() {
        let mut obj = Object::default();

        // Initially free
        assert!(obj.is_free());
        assert!(!obj.is_on_floor());
        assert!(!obj.is_contained());
        assert!(!obj.is_in_player_inventory());

        // Move to floor
        obj.location = ObjectLocation::Floor;
        assert!(!obj.is_free());
        assert!(obj.is_on_floor());
        assert!(!obj.is_contained());

        // Move to inventory
        obj.location = ObjectLocation::PlayerInventory;
        assert!(!obj.is_free());
        assert!(obj.is_in_player_inventory());
        assert!(!obj.is_on_floor());

        // Move to container
        obj.location = ObjectLocation::Contained;
        assert!(obj.is_contained());
        assert!(!obj.is_in_player_inventory());
    }

    #[test]
    fn test_obj_extract_self() {
        let mut obj = Object::default();
        obj.object_type = 42;
        obj.location = ObjectLocation::PlayerInventory;
        obj.x = 10;
        obj.y = 20;
        obj.quantity = 5;

        // Extract the object
        super::obj_extract_self(&mut obj);

        // Should now be free
        assert!(obj.is_free());
        assert_eq!(obj.x, 0);
        assert_eq!(obj.y, 0);
        // Data should be preserved
        assert_eq!(obj.object_type, 42);
        assert_eq!(obj.quantity, 5);
    }

    #[test]
    fn test_discover_object() {
        use crate::object::DiscoveryState;

        let mut discovery_state = DiscoveryState::default();

        // First discovery should return true
        assert!(super::discover_object(&mut discovery_state, 42, 100));
        assert_eq!(discovery_state.count(), 1);
        assert!(discovery_state.is_discovered(42));

        // Second discovery of same type should return false
        assert!(!super::discover_object(&mut discovery_state, 42, 200));
        assert_eq!(discovery_state.count(), 1);

        // Different type should return true
        assert!(super::discover_object(&mut discovery_state, 43, 200));
        assert_eq!(discovery_state.count(), 2);
    }

    #[test]
    fn test_makeknown() {
        use crate::object::DiscoveryState;

        let mut obj = Object::default();
        obj.object_type = 42;
        obj.known = false;

        let mut discovery_state = DiscoveryState::default();

        // Make known
        super::makeknown(&mut obj, &mut discovery_state, 100);

        // Object should be known
        assert!(obj.known);

        // Type should be discovered
        assert!(discovery_state.is_discovered(42));
    }

    #[test]
    fn test_discovery_state_persistence() {
        use crate::object::{DiscoveredType, DiscoveryState};

        let mut discovery_state = DiscoveryState::default();

        // Discover some objects
        discovery_state.discover_object(1, 10);
        discovery_state.discover_object(5, 20);
        discovery_state.discover_object(10, 30);

        assert_eq!(discovery_state.count(), 3);

        // Verify they're all discovered
        assert!(discovery_state.is_discovered(1));
        assert!(discovery_state.is_discovered(5));
        assert!(discovery_state.is_discovered(10));

        // Verify they're in the map
        assert!(discovery_state.discovered.contains_key(&1));
        assert!(discovery_state.discovered.contains_key(&5));
        assert!(discovery_state.discovered.contains_key(&10));

        // Check turn discovery
        let dt1 = discovery_state.discovered.get(&1).unwrap();
        assert_eq!(dt1.discovered_turn, 10);
    }

    #[test]
    fn test_object_deallocate() {
        let mut obj = Object::default();
        obj.object_type = 42;
        obj.location = ObjectLocation::PlayerInventory;
        obj.quantity = 5;
        obj.weight = 100;
        obj.contents.push(Object::default());

        // Deallocate
        obj.deallocate();

        // Should be free with reset values
        assert!(obj.is_free());
        assert_eq!(obj.quantity, 0);
        assert_eq!(obj.weight, 0);
        assert!(obj.contents.is_empty());
        // Original data lost
        assert_eq!(obj.object_type, 42); // But type preserved
    }

    // ========================================================================
    // Phase 2 Tests: Stack Operations & Merging
    // ========================================================================

    #[test]
    fn test_mergable_same_object() {
        let obj1 = Object::new(ObjectId(1), 10, ObjectClass::Coin);
        let obj2 = Object::new(ObjectId(2), 10, ObjectClass::Coin);

        // Same type, no special properties, no stacking restrictions
        assert!(obj1.mergable(&obj2));
    }

    #[test]
    fn test_mergable_different_type() {
        let obj1 = Object::new(ObjectId(1), 10, ObjectClass::Coin);
        let obj2 = Object::new(ObjectId(2), 20, ObjectClass::Coin);

        // Different object type
        assert!(!obj1.mergable(&obj2));
    }

    #[test]
    fn test_mergable_different_buc() {
        let mut obj1 = Object::new(ObjectId(1), 10, ObjectClass::Coin);
        let mut obj2 = Object::new(ObjectId(2), 10, ObjectClass::Coin);

        obj1.buc = BucStatus::Blessed;
        obj2.buc = BucStatus::Cursed;

        assert!(!obj1.mergable(&obj2));
    }

    #[test]
    fn test_mergable_different_enchantment() {
        let mut obj1 = Object::new(ObjectId(1), 10, ObjectClass::Weapon);
        let mut obj2 = Object::new(ObjectId(2), 10, ObjectClass::Weapon);

        obj1.enchantment = 2;
        obj2.enchantment = 1;

        assert!(!obj1.mergable(&obj2));
    }

    #[test]
    fn test_mergable_different_erosion() {
        let mut obj1 = Object::new(ObjectId(1), 10, ObjectClass::Weapon);
        let mut obj2 = Object::new(ObjectId(2), 10, ObjectClass::Weapon);

        obj1.erosion1 = 1;
        obj2.erosion1 = 0;

        assert!(!obj1.mergable(&obj2));
    }

    #[test]
    fn test_mergable_worn_items_dont_merge() {
        let mut obj1 = Object::new(ObjectId(1), 10, ObjectClass::Weapon);
        let mut obj2 = Object::new(ObjectId(2), 10, ObjectClass::Weapon);

        obj1.worn_mask = 1; // Worn

        assert!(!obj1.mergable(&obj2));
    }

    #[test]
    fn test_mergable_artifacts_dont_merge() {
        let mut obj1 = Object::new(ObjectId(1), 10, ObjectClass::Weapon);
        let mut obj2 = Object::new(ObjectId(2), 10, ObjectClass::Weapon);

        obj1.artifact = 5; // Is artifact

        assert!(!obj1.mergable(&obj2));
    }

    #[test]
    fn test_mergable_non_stacking_class() {
        let obj1 = Object::new(ObjectId(1), 10, ObjectClass::Armor);
        let obj2 = Object::new(ObjectId(2), 10, ObjectClass::Armor);

        // Armor doesn't stack
        assert!(!obj1.mergable(&obj2));
    }

    #[test]
    fn test_merged() {
        let mut obj1 = Object {
            id: ObjectId(1),
            object_type: 10,
            class: ObjectClass::Coin,
            quantity: 50,
            weight: 50,
            ..Default::default()
        };

        let obj2 = Object {
            id: ObjectId(2),
            object_type: 10,
            class: ObjectClass::Coin,
            quantity: 30,
            weight: 30,
            ..Default::default()
        };

        assert!(obj1.merged(&obj2));
        assert_eq!(obj1.quantity, 80);
        assert_eq!(obj1.weight, 80);
    }

    #[test]
    fn test_splitobj() {
        let mut obj = Object {
            id: ObjectId(1),
            object_type: 10,
            class: ObjectClass::Coin,
            quantity: 100,
            weight: 100,
            ..Default::default()
        };

        let split = obj.splitobj(30).unwrap();

        // Original should have 70 left
        assert_eq!(obj.quantity, 70);
        assert_eq!(obj.weight, 70);

        // Split should have 30
        assert_eq!(split.quantity, 30);
        assert_eq!(split.weight, 30);
        assert_eq!(split.id, ObjectId::NONE);
    }

    #[test]
    fn test_splitobj_invalid() {
        let mut obj = Object {
            id: ObjectId(1),
            object_type: 10,
            class: ObjectClass::Coin,
            quantity: 50,
            weight: 50,
            ..Default::default()
        };

        // Can't split with 0 count
        assert!(obj.splitobj(0).is_none());

        // Can't split with count >= quantity
        assert!(obj.splitobj(50).is_none());
        assert!(obj.splitobj(100).is_none());
    }

    #[test]
    fn test_weight_simple_object() {
        let obj = Object {
            id: ObjectId(1),
            object_type: 10,
            class: ObjectClass::Coin,
            quantity: 10,
            weight: 100,
            ..Default::default()
        };

        assert_eq!(super::weight(&obj), 1000); // 100 * 10
    }

    #[test]
    fn test_weight_container_with_contents() {
        let mut container = Object {
            id: ObjectId(1),
            object_type: 365,          // BAG_OF_HOLDING
            class: ObjectClass::Armor, // Treat as container for test
            quantity: 1,
            weight: 15,
            buc: BucStatus::Uncursed,
            ..Default::default()
        };

        let mut item = Object {
            id: ObjectId(2),
            object_type: 10,
            class: ObjectClass::Coin,
            quantity: 1,
            weight: 100,
            ..Default::default()
        };

        container.contents.push(item);

        // Uncursed Bag of Holding: contents_weight / 2
        // 15 (base) + 100/2 (contents) = 65
        assert_eq!(super::weight(&container), 65);
    }

    #[test]
    fn test_weight_blessed_bag_of_holding() {
        let mut container = Object {
            id: ObjectId(1),
            object_type: 365,
            class: ObjectClass::Armor,
            quantity: 1,
            weight: 15,
            buc: BucStatus::Blessed,
            ..Default::default()
        };

        let item = Object {
            id: ObjectId(2),
            object_type: 10,
            class: ObjectClass::Coin,
            quantity: 1,
            weight: 400,
            ..Default::default()
        };

        container.contents.push(item);

        // Blessed Bag: 15 + 400/4 = 115
        assert_eq!(super::weight(&container), 115);
    }

    #[test]
    fn test_weight_cursed_bag_of_holding() {
        let mut container = Object {
            id: ObjectId(1),
            object_type: 365,
            class: ObjectClass::Armor,
            quantity: 1,
            weight: 15,
            buc: BucStatus::Cursed,
            ..Default::default()
        };

        let item = Object {
            id: ObjectId(2),
            object_type: 10,
            class: ObjectClass::Coin,
            quantity: 1,
            weight: 100,
            ..Default::default()
        };

        container.contents.push(item);

        // Cursed Bag: 15 + 100*2 = 215
        assert_eq!(super::weight(&container), 215);
    }

    #[test]
    fn test_extract_nobj() {
        let mut objects = vec![
            Object::new(ObjectId(1), 10, ObjectClass::Coin),
            Object::new(ObjectId(2), 20, ObjectClass::Coin),
            Object::new(ObjectId(3), 30, ObjectClass::Coin),
        ];

        let extracted = super::extract_nobj(&mut objects, 1);
        assert!(extracted.is_some());
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[1].object_type, 30); // Third became second
    }

    #[test]
    fn test_curr_cnt() {
        let objects = vec![
            Object {
                id: ObjectId(1),
                object_type: 10,
                class: ObjectClass::Coin,
                x: 5,
                y: 5,
                quantity: 50,
                ..Default::default()
            },
            Object {
                id: ObjectId(2),
                object_type: 10,
                class: ObjectClass::Coin,
                x: 5,
                y: 5,
                quantity: 30,
                ..Default::default()
            },
            Object {
                id: ObjectId(3),
                object_type: 10,
                class: ObjectClass::Coin,
                x: 6,
                y: 5,
                quantity: 20,
                ..Default::default()
            },
        ];

        // Count at (5, 5)
        assert_eq!(super::curr_cnt(&objects, 5, 5), 80);

        // Count at (6, 5)
        assert_eq!(super::curr_cnt(&objects, 6, 5), 20);

        // Count at (0, 0) - none
        assert_eq!(super::curr_cnt(&objects, 0, 0), 0);
    }

    #[test]
    fn test_nexthere() {
        let objects = vec![
            Object {
                id: ObjectId(1),
                x: 5,
                y: 5,
                ..Default::default()
            },
            Object {
                id: ObjectId(2),
                x: 5,
                y: 5,
                ..Default::default()
            },
            Object {
                id: ObjectId(3),
                x: 6,
                y: 5,
                ..Default::default()
            },
        ];

        // Find next at (5,5) after index 0
        assert_eq!(super::nexthere(&objects, 5, 5, 0), Some(1));

        // Find next at (5,5) after index 1 - none
        assert_eq!(super::nexthere(&objects, 5, 5, 1), None);

        // Find next at (6,5) after index 0
        assert_eq!(super::nexthere(&objects, 6, 5, 0), Some(2));
    }

    #[test]
    fn test_lift_object() {
        let mut obj = Object {
            id: ObjectId(1),
            location: ObjectLocation::Floor,
            x: 10,
            y: 20,
            ..Default::default()
        };

        super::lift_object(&mut obj);

        assert_eq!(obj.location, ObjectLocation::PlayerInventory);
        assert_eq!(obj.x, 0);
        assert_eq!(obj.y, 0);
    }

    #[test]
    fn test_hold_another_object() {
        let mut container = Object::new(ObjectId(1), 360, ObjectClass::Armor);
        container.contents = Vec::new();

        let item = Object {
            id: ObjectId(2),
            object_type: 10,
            location: ObjectLocation::Floor,
            x: 5,
            y: 5,
            ..Default::default()
        };

        assert!(super::hold_another_object(&mut container, item));
        assert_eq!(container.contents.len(), 1);
        assert_eq!(container.contents[0].location, ObjectLocation::Contained);
        assert_eq!(container.contents[0].x, 0);
        assert_eq!(container.contents[0].y, 0);
    }

    #[test]
    fn test_bypass_functions() {
        let mut obj = Object::default();

        assert!(!obj.is_bypassed());

        super::bypass_obj(&mut obj);
        assert!(obj.is_bypassed());

        obj.clear_bypass();
        assert!(!obj.is_bypassed());
    }

    #[test]
    fn test_bypass_objlist() {
        let mut objects = vec![Object::default(), Object::default(), Object::default()];

        super::bypass_objlist(&mut objects);

        for obj in &objects {
            assert!(obj.is_bypassed());
        }

        super::clear_bypasses(&mut objects);

        for obj in &objects {
            assert!(!obj.is_bypassed());
        }
    }

    // ========================================================================
    // Phase 4 Tests: Placement and Floor Effects
    // ========================================================================

    #[test]
    fn test_place_object() {
        let mut obj = Object::default();
        obj.location = ObjectLocation::Free;

        super::place_object(&mut obj, 10, 20);

        assert_eq!(obj.location, ObjectLocation::Floor);
        assert_eq!(obj.x, 10);
        assert_eq!(obj.y, 20);
    }

    #[test]
    fn test_remove_object() {
        let mut obj = Object::default();
        obj.location = ObjectLocation::Floor;
        obj.x = 10;
        obj.y = 20;

        super::remove_object(&mut obj);

        assert_eq!(obj.location, ObjectLocation::Free);
        assert_eq!(obj.x, 0);
        assert_eq!(obj.y, 0);
    }

    #[test]
    fn test_obj_scatter() {
        let objects = vec![
            Object::new(ObjectId(1), 10, ObjectClass::Coin),
            Object::new(ObjectId(2), 10, ObjectClass::Coin),
            Object::new(ObjectId(3), 10, ObjectClass::Coin),
        ];

        let mut rng = crate::rng::GameRng::from_entropy();
        let scattered = super::obj_scatter(objects, 40, 10, 5, &mut rng);

        // All objects should be on floor
        for obj in &scattered {
            assert_eq!(obj.location, ObjectLocation::Floor);
            // Should be within bounds
            assert!(obj.x >= 0 && obj.x < 80);
            assert!(obj.y >= 0 && obj.y < 21);
        }
    }

    #[test]
    fn test_flooreffects_fire() {
        let scroll = Object::new(ObjectId(1), 1, ObjectClass::Scroll);
        let weapon = Object::new(ObjectId(2), 1, ObjectClass::Weapon);

        // Fire affects scrolls
        assert!(super::flooreffects(&scroll, 1));

        // Fire doesn't affect weapons much
        assert!(!super::flooreffects(&weapon, 1));
    }

    #[test]
    fn test_flooreffects_acid() {
        let armor = Object::new(ObjectId(1), 1, ObjectClass::Armor);
        let food = Object::new(ObjectId(2), 1, ObjectClass::Food);

        // Acid affects armor
        assert!(super::flooreffects(&armor, 2));

        // Acid doesn't affect food
        assert!(!super::flooreffects(&food, 2));
    }

    #[test]
    fn test_flooreffects_water() {
        let obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);

        // Water affects most objects
        assert!(super::flooreffects(&obj, 3));
    }

    #[test]
    fn test_is_adjacent() {
        // Adjacent cases
        assert!(super::is_adjacent(10, 10, 10, 11));
        assert!(super::is_adjacent(10, 10, 11, 10));
        assert!(super::is_adjacent(10, 10, 11, 11));
        assert!(super::is_adjacent(10, 10, 9, 9));

        // Not adjacent
        assert!(!super::is_adjacent(10, 10, 10, 12));
        assert!(!super::is_adjacent(10, 10, 12, 10));
        assert!(!super::is_adjacent(10, 10, 10, 10)); // Same location
    }

    #[test]
    fn test_is_visible_from() {
        // Within sight range
        assert!(super::is_visible_from(10, 10, 5, 5, 10));

        // At edge of sight range
        assert!(super::is_visible_from(20, 20, 10, 10, 10));

        // Beyond sight range
        assert!(!super::is_visible_from(30, 30, 10, 10, 10));

        // Same location
        assert!(super::is_visible_from(10, 10, 10, 10, 10));
    }

    // ========================================================================
    // Phase 5: Shop Integration & Light Sources Tests
    // ========================================================================

    #[test]
    fn test_obj_value_base() {
        let weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        let value = super::obj_value(&weapon);
        assert!(value > 0);

        let armor = Object::new(ObjectId(2), 1, ObjectClass::Armor);
        let armor_value = super::obj_value(&armor);
        assert!(armor_value > 0);
    }

    #[test]
    fn test_obj_value_with_quantity() {
        let mut food = Object::new(ObjectId(1), 1, ObjectClass::Food);
        food.quantity = 5;

        let single_value = super::obj_value(&food);

        food.quantity = 1;
        let base_value = super::obj_value(&food);

        // Multi-quantity should be more valuable
        assert!(single_value > base_value);
    }

    #[test]
    fn test_obj_value_with_enchantment() {
        let mut weapon1 = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        let base_value = super::obj_value(&weapon1);

        weapon1.enchantment = 2;
        let enchanted_value = super::obj_value(&weapon1);

        // Enchanted items are more valuable
        assert!(enchanted_value > base_value);
    }

    #[test]
    fn test_obj_value_with_erosion() {
        let mut weapon1 = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        let base_value = super::obj_value(&weapon1);

        weapon1.erosion1 = 2;
        let eroded_value = super::obj_value(&weapon1);

        // Eroded items are less valuable
        assert!(eroded_value < base_value);
    }

    #[test]
    fn test_obj_value_with_blessing() {
        let mut weapon_uncursed = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        weapon_uncursed.buc = BucStatus::Uncursed;
        let base_value = super::obj_value(&weapon_uncursed);

        let mut weapon_blessed = Object::new(ObjectId(2), 1, ObjectClass::Weapon);
        weapon_blessed.buc = BucStatus::Blessed;
        let blessed_value = super::obj_value(&weapon_blessed);

        // Blessed items are more valuable
        assert!(blessed_value > base_value);
    }

    #[test]
    fn test_can_be_billed() {
        let coin = Object::new(ObjectId(1), 1, ObjectClass::Coin);
        assert!(!super::can_be_billed(&coin)); // Coins can't be billed

        let weapon = Object::new(ObjectId(2), 1, ObjectClass::Weapon);
        assert!(super::can_be_billed(&weapon)); // Weapons can be billed
    }

    #[test]
    fn test_addtobill() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        super::addtobill(&mut obj, 50);

        assert_eq!(obj.location, ObjectLocation::OnBill);
        assert_eq!(obj.shop_price, 50);
        assert!(obj.unpaid);
    }

    #[test]
    fn test_subfrombill() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.location = ObjectLocation::OnBill;
        obj.unpaid = true;

        super::subfrombill(&mut obj);

        assert!(!obj.unpaid);
        assert_eq!(obj.location, ObjectLocation::Free);
    }

    #[test]
    fn test_begin_burn() {
        let mut lamp = Object::new(ObjectId(1), 1, ObjectClass::Tool);
        assert!(!lamp.lit);

        super::begin_burn(&mut lamp);
        assert!(lamp.lit);
    }

    #[test]
    fn test_end_burn() {
        let mut lamp = Object::new(ObjectId(1), 1, ObjectClass::Tool);
        lamp.lit = true;

        super::end_burn(&mut lamp);
        assert!(!lamp.lit);
    }

    #[test]
    fn test_obj_sheds_light_unlit() {
        let lamp = Object::new(ObjectId(1), 1, ObjectClass::Tool);
        assert!(!super::obj_sheds_light(&lamp)); // Not lit
    }

    #[test]
    fn test_obj_sheds_light_lit_tool() {
        let mut lamp = Object::new(ObjectId(1), 1, ObjectClass::Tool);
        lamp.lit = true;

        assert!(super::obj_sheds_light(&lamp));
    }

    #[test]
    fn test_obj_sheds_light_lit_potion() {
        let mut potion = Object::new(ObjectId(1), 1, ObjectClass::Potion);
        potion.lit = true;
        potion.enchantment = 1; // Magic potion

        assert!(super::obj_sheds_light(&potion));
    }

    #[test]
    fn test_light_radius_tool() {
        let mut lamp = Object::new(ObjectId(1), 1, ObjectClass::Tool);
        lamp.lit = true;
        lamp.enchantment = 0;

        let radius = super::light_radius(&lamp);
        assert_eq!(radius, 2); // Regular lamp
    }

    #[test]
    fn test_light_radius_enchanted_tool() {
        let mut lamp = Object::new(ObjectId(1), 1, ObjectClass::Tool);
        lamp.lit = true;
        lamp.enchantment = 2;

        let radius = super::light_radius(&lamp);
        assert_eq!(radius, 5); // Magic lantern
    }

    #[test]
    fn test_light_radius_unlit() {
        let lamp = Object::new(ObjectId(1), 1, ObjectClass::Tool);

        let radius = super::light_radius(&lamp);
        assert_eq!(radius, 0); // No light if not lit
    }

    #[test]
    fn test_torch_sheds_light() {
        let mut torch = Object::new(ObjectId(1), 1, ObjectClass::Tool);
        torch.lit = true;
        torch.erosion1 = 0;

        assert!(super::torch_sheds_light(&torch));
    }

    #[test]
    fn test_torch_sheds_light_burnt() {
        let mut torch = Object::new(ObjectId(1), 1, ObjectClass::Tool);
        torch.lit = true;
        torch.erosion1 = 3; // Burnt out

        assert!(!super::torch_sheds_light(&torch));
    }

    #[test]
    fn test_candle_sheds_light() {
        let mut candle = Object::new(ObjectId(1), 1, ObjectClass::Tool);
        candle.lit = true;
        candle.erosion1 = 1;

        assert!(super::candle_sheds_light(&candle));
    }

    #[test]
    fn test_candle_sheds_light_burnt() {
        let mut candle = Object::new(ObjectId(1), 1, ObjectClass::Tool);
        candle.lit = true;
        candle.erosion1 = 3; // Fully burnt

        assert!(!super::candle_sheds_light(&candle));
    }

    #[test]
    fn test_light_emitting_objs() {
        let types = super::light_emitting_objs();
        assert!(!types.is_empty());
        assert!(types.len() >= 5); // Should have at least 5 light-emitting types
    }

    #[test]
    fn test_snuff_candles() {
        let mut objects = vec![
            {
                let mut lamp = Object::new(ObjectId(1), 1, ObjectClass::Tool);
                lamp.location = ObjectLocation::Floor;
                lamp.lit = true;
                lamp.x = 10;
                lamp.y = 10;
                lamp
            },
            {
                let mut candle = Object::new(ObjectId(2), 1, ObjectClass::Tool);
                candle.location = ObjectLocation::Floor;
                candle.lit = true;
                candle.x = 15;
                candle.y = 15;
                candle
            },
        ];

        // Snuff within radius 3
        super::snuff_candles(&mut objects, 10, 10, 3);

        // First candle should be snuffed
        assert!(!objects[0].lit);
        // Second candle is too far away
        assert!(objects[1].lit);
    }

    #[test]
    fn test_obj_sheds_light_anywhere_lit() {
        let mut lamp = Object::new(ObjectId(1), 1, ObjectClass::Tool);
        lamp.lit = true;

        assert!(super::obj_sheds_light_anywhere(&lamp));
    }

    #[test]
    fn test_obj_sheds_light_anywhere_unlit() {
        let lamp = Object::new(ObjectId(1), 1, ObjectClass::Tool);

        assert!(!super::obj_sheds_light_anywhere(&lamp));
    }

    #[test]
    fn test_obj_sheds_light_anywhere_non_light() {
        let mut weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        weapon.lit = true;

        assert!(!super::obj_sheds_light_anywhere(&weapon)); // Weapons can't emit light
    }

    #[test]
    fn test_pay_bill_success() {
        let mut objects = vec![
            {
                let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
                obj.location = ObjectLocation::OnBill;
                obj.shop_price = 50;
                obj.quantity = 1;
                obj.unpaid = true;
                obj
            },
            {
                let mut obj = Object::new(ObjectId(2), 1, ObjectClass::Armor);
                obj.location = ObjectLocation::OnBill;
                obj.shop_price = 30;
                obj.quantity = 1;
                obj.unpaid = true;
                obj
            },
        ];

        let mut gold = 100;
        let paid = super::pay_bill(&mut objects, 50, &mut gold);

        assert_eq!(paid, 50);
        assert_eq!(gold, 50); // 100 - 50
    }

    #[test]
    fn test_pay_bill_insufficient_funds() {
        let mut objects = vec![Object::new(ObjectId(1), 1, ObjectClass::Weapon)];

        let mut gold = 20;
        let paid = super::pay_bill(&mut objects, 50, &mut gold);

        assert_eq!(paid, 0);
        assert_eq!(gold, 20); // No change
    }

    #[test]
    fn test_pay_bill_nothing_due() {
        let mut objects = vec![{
            let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
            obj.unpaid = false; // Already paid
            obj
        }];

        let mut gold = 100;
        let paid = super::pay_bill(&mut objects, 50, &mut gold);

        assert_eq!(paid, 0);
        assert_eq!(gold, 100); // No change
    }

    #[test]
    fn test_shk_move_purchase() {
        let mood = super::shk_move(1, 0, 150);
        assert!(mood > 0); // Should be happy with big purchase

        let mood = super::shk_move(1, 0, 5);
        assert_eq!(mood, 0); // Indifferent to small purchase
    }

    #[test]
    fn test_shk_move_theft() {
        let mood = super::shk_move(1, 1, 0);
        assert_eq!(mood, -10); // Very angry at theft
    }

    #[test]
    fn test_shk_move_payment() {
        let mood = super::shk_move(1, 2, 60);
        assert!(mood > 0); // Should be happy with payment
    }

    // ========================================================================
    // Phase 6: Materials & Erosion Tests
    // ========================================================================

    #[test]
    fn test_is_flammable_scroll() {
        let scroll = Object::new(ObjectId(1), 1, ObjectClass::Scroll);
        assert!(super::is_flammable(&scroll));
    }

    #[test]
    fn test_is_flammable_weapon() {
        let weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        assert!(!super::is_flammable(&weapon));
    }

    #[test]
    fn test_is_rottable_food() {
        let mut food = Object::new(ObjectId(1), 1, ObjectClass::Food);
        food.erosion_proof = false;
        assert!(super::is_rottable(&food));
    }

    #[test]
    fn test_is_rottable_erosion_proof() {
        let mut food = Object::new(ObjectId(1), 1, ObjectClass::Food);
        food.erosion_proof = true;
        assert!(!super::is_rottable(&food));
    }

    #[test]
    fn test_is_rustprone_weapon() {
        let mut weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        weapon.erosion_proof = false;
        assert!(super::is_rustprone(&weapon));
    }

    #[test]
    fn test_is_rustprone_scroll() {
        let mut scroll = Object::new(ObjectId(1), 1, ObjectClass::Scroll);
        scroll.erosion_proof = false;
        assert!(!super::is_rustprone(&scroll));
    }

    #[test]
    fn test_is_corrodeable_armor() {
        let mut armor = Object::new(ObjectId(1), 1, ObjectClass::Armor);
        armor.erosion_proof = false;
        assert!(super::is_corrodeable(&armor));
    }

    #[test]
    fn test_is_corrodeable_artifact() {
        let mut weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        weapon.artifact = 1; // Mark as artifact
        weapon.erosion_proof = false;
        assert!(super::is_corrodeable(&weapon)); // Still can corrode, erosion_proof is separate
    }

    #[test]
    fn test_is_damageable() {
        let mut weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        weapon.erosion_proof = false;
        assert!(super::is_damageable(&weapon));
    }

    #[test]
    fn test_is_damageable_artifact() {
        let mut weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        weapon.artifact = 1;
        assert!(!super::is_damageable(&weapon)); // Artifacts can't be damaged
    }

    #[test]
    fn test_objects_are_same_material() {
        let weapon1 = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        let weapon2 = Object::new(ObjectId(2), 1, ObjectClass::Weapon);
        assert!(super::objects_are_same_material(&weapon1, &weapon2));
    }

    #[test]
    fn test_objects_are_different_material() {
        let weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        let scroll = Object::new(ObjectId(2), 1, ObjectClass::Scroll);
        assert!(!super::objects_are_same_material(&weapon, &scroll));
    }

    #[test]
    fn test_obj_material_is_flammable() {
        assert!(super::obj_material_is_flammable(ObjectClass::Scroll));
        assert!(super::obj_material_is_flammable(ObjectClass::Spellbook));
        assert!(!super::obj_material_is_flammable(ObjectClass::Weapon));
    }

    #[test]
    fn test_obj_material_is_rustprone() {
        assert!(super::obj_material_is_rustprone(ObjectClass::Weapon));
        assert!(super::obj_material_is_rustprone(ObjectClass::Armor));
        assert!(!super::obj_material_is_rustprone(ObjectClass::Scroll));
    }

    #[test]
    fn test_obj_resists_blessed() {
        let mut weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        weapon.buc = BucStatus::Blessed;
        let resistance = super::obj_resists(&weapon, 1);
        assert_eq!(resistance, 75);
    }

    #[test]
    fn test_obj_resists_cursed() {
        let mut weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        weapon.buc = BucStatus::Cursed;
        let resistance = super::obj_resists(&weapon, 1);
        assert_eq!(resistance, 0);
    }

    #[test]
    fn test_obj_resists_greased() {
        let mut weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        weapon.buc = BucStatus::Uncursed;
        weapon.greased = true;
        let resistance = super::obj_resists(&weapon, 1);
        assert_eq!(resistance, 50); // 25 + 25
    }

    #[test]
    fn test_obj_resists_artifact() {
        let mut weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        weapon.artifact = 1;
        let resistance = super::obj_resists(&weapon, 1);
        assert_eq!(resistance, 100); // Artifacts are immune
    }

    #[test]
    fn test_erode_obj_fire() {
        let mut scroll = Object::new(ObjectId(1), 1, ObjectClass::Scroll);
        scroll.erosion1 = 0;

        let mut rng = crate::rng::GameRng::from_entropy();
        super::erode_obj(&mut scroll, 1, &mut rng);

        // Scroll might erode (depends on resistance roll)
        assert!(scroll.erosion1 <= 1);
    }

    #[test]
    fn test_erode_obj_acid() {
        let mut weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        weapon.erosion2 = 0;

        let mut rng = crate::rng::GameRng::from_entropy();
        super::erode_obj(&mut weapon, 2, &mut rng);

        // Weapon might erode from acid
        assert!(weapon.erosion2 <= 1);
    }

    #[test]
    fn test_erode_obj_protected() {
        let mut weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        weapon.erosion1 = 0;
        weapon.buc = BucStatus::Blessed; // High resistance

        let mut rng = crate::rng::GameRng::new(42); // Fixed seed for deterministic test
        super::erode_obj(&mut weapon, 1, &mut rng);

        // With high resistance, likely won't erode
        // (exact behavior depends on RNG)
    }

    #[test]
    fn test_erode_obj_artifact() {
        let mut weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        weapon.artifact = 1;
        weapon.erosion1 = 0;

        let mut rng = crate::rng::GameRng::from_entropy();
        super::erode_obj(&mut weapon, 1, &mut rng);

        // Artifacts never erode
        assert_eq!(weapon.erosion1, 0);
    }

    #[test]
    fn test_obj_is_destroyed_not_eroded() {
        let obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        assert!(!super::obj_is_destroyed(&obj));
    }

    #[test]
    fn test_obj_is_destroyed_level_1() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.erosion1 = 1;
        assert!(!super::obj_is_destroyed(&obj));
    }

    #[test]
    fn test_obj_is_destroyed_level_3() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.erosion1 = 3;
        assert!(super::obj_is_destroyed(&obj));
    }

    #[test]
    fn test_obj_name_from_material_weapon() {
        let name = super::obj_name_from_material(ObjectClass::Weapon);
        assert_eq!(name, "iron");
    }

    #[test]
    fn test_obj_name_from_material_armor() {
        let name = super::obj_name_from_material(ObjectClass::Armor);
        assert_eq!(name, "leather");
    }

    #[test]
    fn test_obj_name_from_material_scroll() {
        let name = super::obj_name_from_material(ObjectClass::Scroll);
        assert_eq!(name, "paper");
    }

    #[test]
    fn test_rust_dmg_none() {
        let dmg = super::rust_dmg(0);
        assert_eq!(dmg, 0);
    }

    #[test]
    fn test_rust_dmg_light() {
        let dmg = super::rust_dmg(1);
        assert_eq!(dmg, 25);
    }

    #[test]
    fn test_rust_dmg_moderate() {
        let dmg = super::rust_dmg(2);
        assert_eq!(dmg, 50);
    }

    #[test]
    fn test_rust_dmg_destroyed() {
        let dmg = super::rust_dmg(3);
        assert_eq!(dmg, 100);
    }

    // ========================================================================
    // Phase 7: Polish & Edge Cases Tests
    // ========================================================================

    #[test]
    fn test_obj_no_longer_held_destroyed() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.erosion1 = 3; // Destroyed

        let handled = super::obj_no_longer_held(&mut obj);

        assert!(handled);
        assert_eq!(obj.location, ObjectLocation::Free);
    }

    #[test]
    fn test_obj_no_longer_held_intact() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.erosion1 = 1; // Not destroyed

        let handled = super::obj_no_longer_held(&mut obj);

        assert!(!handled); // Still present
    }

    #[test]
    fn test_dropx() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);

        super::dropx(&mut obj, 15, 20);

        assert_eq!(obj.location, ObjectLocation::Floor);
        assert_eq!(obj.x, 15);
        assert_eq!(obj.y, 20);
    }

    #[test]
    fn test_dropy() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);

        super::dropy(&mut obj, 10, 10);

        assert_eq!(obj.location, ObjectLocation::Floor);
        assert_eq!(obj.x, 10);
        assert_eq!(obj.y, 10);
    }

    #[test]
    fn test_obj_falls_off_hero() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Armor);
        obj.worn_mask = 0xFF; // Fully worn

        super::obj_falls_off_hero(&mut obj);

        assert_eq!(obj.worn_mask, 0); // No longer worn
    }

    #[test]
    fn test_obj_adjust_light_radius() {
        let mut lamp = Object::new(ObjectId(1), 1, ObjectClass::Tool);
        lamp.lit = true;
        lamp.enchantment = 0;

        let radius = super::obj_adjust_light_radius(&lamp);
        assert_eq!(radius, 2);
    }

    #[test]
    fn test_obj_sheds_light_radius() {
        let mut lamp = Object::new(ObjectId(1), 1, ObjectClass::Tool);
        lamp.lit = true;
        lamp.enchantment = 2;

        let radius = super::obj_sheds_light_radius(&lamp);
        assert_eq!(radius, 5);
    }

    #[test]
    fn test_obj_reflects_non_reflective() {
        let weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        assert!(!super::obj_reflects(&weapon));
    }

    #[test]
    fn test_obj_reflects_enchanted_armor() {
        let mut armor = Object::new(ObjectId(1), 1, ObjectClass::Armor);
        armor.enchantment = 2;

        assert!(super::obj_reflects(&armor));
    }

    #[test]
    fn test_obj_is_in_use_not_in_use() {
        let obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        assert!(!super::obj_is_in_use(&obj));
    }

    #[test]
    fn test_obj_is_in_use_worn() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Armor);
        obj.worn_mask = 1; // Worn somewhere

        assert!(super::obj_is_in_use(&obj));
    }

    #[test]
    fn test_obj_is_in_use_active() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.in_use = true;

        assert!(super::obj_is_in_use(&obj));
    }

    #[test]
    fn test_obj_is_piletop_at_single() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.location = ObjectLocation::Floor;
        obj.x = 10;
        obj.y = 15;

        let objects = vec![obj.clone()];

        assert!(super::obj_is_piletop_at(&obj, 10, 15, &objects));
    }

    #[test]
    fn test_obj_is_piletop_at_multiple() {
        let mut obj1 = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj1.location = ObjectLocation::Floor;
        obj1.x = 10;
        obj1.y = 15;

        let mut obj2 = Object::new(ObjectId(2), 1, ObjectClass::Armor);
        obj2.location = ObjectLocation::Floor;
        obj2.x = 10;
        obj2.y = 15;

        let objects = vec![obj1.clone(), obj2.clone()];

        // obj1 is not on top (obj2 is after it)
        assert!(!super::obj_is_piletop_at(&obj1, 10, 15, &objects));
        // obj2 is on top (nothing after it)
        assert!(super::obj_is_piletop_at(&obj2, 10, 15, &objects));
    }

    #[test]
    fn test_obj_is_piletop_at_wrong_location() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.location = ObjectLocation::Floor;
        obj.x = 10;
        obj.y = 15;

        let objects = vec![obj.clone()];

        assert!(!super::obj_is_piletop_at(&obj, 20, 20, &objects));
    }

    #[test]
    fn test_remap_obj_material() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        assert_eq!(obj.class, ObjectClass::Weapon);

        super::remap_obj_material(&mut obj, ObjectClass::Rock);

        assert_eq!(obj.class, ObjectClass::Rock);
    }

    #[test]
    fn test_retouch_object_unknown_cursed() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.buc = BucStatus::Cursed;
        obj.buc_known = false;

        let had_effect = super::retouch_object(&mut obj);

        assert!(had_effect);
        assert!(obj.buc_known); // Cursed status revealed
    }

    #[test]
    fn test_retouch_object_known_cursed() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.buc = BucStatus::Cursed;
        obj.buc_known = true;

        let had_effect = super::retouch_object(&mut obj);

        assert!(!had_effect); // Already known, no effect
    }

    #[test]
    fn test_retouch_object_blessed() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.buc = BucStatus::Blessed;
        obj.buc_known = false;

        let had_effect = super::retouch_object(&mut obj);

        assert!(!had_effect); // Blessed items don't trigger touch effects
    }

    #[test]
    fn test_retouch_equipment() {
        let mut objects = vec![
            {
                let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Armor);
                obj.worn_mask = 1;
                obj.buc = BucStatus::Cursed;
                obj.buc_known = false;
                obj
            },
            {
                let mut obj = Object::new(ObjectId(2), 1, ObjectClass::Weapon);
                obj.worn_mask = 0; // Not worn
                obj.buc = BucStatus::Cursed;
                obj.buc_known = false;
                obj
            },
        ];

        let count = super::retouch_equipment(&mut objects);

        assert_eq!(count, 1); // Only first object had effect
    }

    #[test]
    fn test_select_off() {
        let mut objects = vec![
            {
                let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
                obj.in_use = true;
                obj
            },
            {
                let mut obj = Object::new(ObjectId(2), 1, ObjectClass::Armor);
                obj.in_use = true;
                obj
            },
        ];

        super::select_off(&mut objects);

        for obj in &objects {
            assert!(!obj.in_use);
        }
    }

    #[test]
    fn test_select_hwep() {
        let mut weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        assert!(!weapon.in_use);

        super::select_hwep(&mut weapon);

        assert!(weapon.in_use);
    }

    #[test]
    fn test_select_hwep_non_weapon() {
        let mut armor = Object::new(ObjectId(1), 1, ObjectClass::Armor);
        armor.in_use = true;

        super::select_hwep(&mut armor);

        assert!(armor.in_use); // Should not change non-weapons
    }

    // ========================================================================
    // Object Property Functions Tests
    // ========================================================================

    #[test]
    fn test_obj_is_pname_artifact() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.artifact = 1; // Excalibur
        assert!(super::obj_is_pname(&obj));
    }

    #[test]
    fn test_obj_is_pname_named_uppercase() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.name = Some("Sting".to_string());
        assert!(super::obj_is_pname(&obj));
    }

    #[test]
    fn test_obj_is_pname_named_lowercase() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.name = Some("my sword".to_string());
        assert!(!super::obj_is_pname(&obj));
    }

    #[test]
    fn test_obj_is_pname_no_name() {
        let obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        assert!(!super::obj_is_pname(&obj));
    }

    #[test]
    fn test_cursed() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.buc = BucStatus::Cursed;
        assert!(super::cursed(&obj));

        obj.buc = BucStatus::Blessed;
        assert!(!super::cursed(&obj));

        obj.buc = BucStatus::Uncursed;
        assert!(!super::cursed(&obj));
    }

    #[test]
    fn test_cursetxt() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);

        obj.buc = BucStatus::Cursed;
        assert_eq!(super::cursetxt(&obj), "cursed");

        obj.buc = BucStatus::Blessed;
        assert_eq!(super::cursetxt(&obj), "blessed");

        obj.buc = BucStatus::Uncursed;
        assert_eq!(super::cursetxt(&obj), "uncursed");
    }

    #[test]
    fn test_set_bknown() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        assert!(!obj.buc_known);

        super::set_bknown(&mut obj, true);
        assert!(obj.buc_known);

        super::set_bknown(&mut obj, false);
        assert!(!obj.buc_known);
    }

    #[test]
    fn test_o_material() {
        let weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        assert_eq!(super::o_material(&weapon), crate::object::Material::Iron);

        let scroll = Object::new(ObjectId(2), 1, ObjectClass::Scroll);
        assert_eq!(super::o_material(&scroll), crate::object::Material::Paper);

        let potion = Object::new(ObjectId(3), 1, ObjectClass::Potion);
        assert_eq!(super::o_material(&potion), crate::object::Material::Glass);
    }

    #[test]
    fn test_obj_nutrition_food() {
        let mut food = Object::new(ObjectId(1), 1, ObjectClass::Food);
        food.corpse_type = -1; // Not a corpse
        assert_eq!(super::obj_nutrition(&food), 50);
    }

    #[test]
    fn test_obj_nutrition_corpse() {
        let mut corpse = Object::new(ObjectId(1), 1, ObjectClass::Food);
        corpse.corpse_type = 10; // Some monster type
        assert_eq!(super::obj_nutrition(&corpse), 100);
    }

    #[test]
    fn test_obj_nutrition_non_food() {
        let weapon = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        assert_eq!(super::obj_nutrition(&weapon), 0);
    }

    #[test]
    fn test_splittable() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Coin);
        obj.quantity = 5;
        assert!(super::splittable(&obj));

        obj.quantity = 1;
        assert!(!super::splittable(&obj));
    }

    #[test]
    fn test_splittable_non_stackable() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Armor);
        obj.quantity = 5;
        assert!(!super::splittable(&obj)); // Armor doesn't stack
    }

    #[test]
    fn test_merge_choice_same_objects() {
        let obj1 = Object::new(ObjectId(1), 100, ObjectClass::Coin);
        let obj2 = Object::new(ObjectId(2), 100, ObjectClass::Coin);
        assert!(super::merge_choice(&obj1, &obj2));
    }

    #[test]
    fn test_merge_choice_different_types() {
        let obj1 = Object::new(ObjectId(1), 100, ObjectClass::Coin);
        let obj2 = Object::new(ObjectId(2), 101, ObjectClass::Coin);
        assert!(!super::merge_choice(&obj1, &obj2));
    }

    #[test]
    fn test_merge_choice_different_buc() {
        let mut obj1 = Object::new(ObjectId(1), 100, ObjectClass::Potion);
        let mut obj2 = Object::new(ObjectId(2), 100, ObjectClass::Potion);
        obj1.buc = BucStatus::Blessed;
        obj2.buc = BucStatus::Cursed;
        assert!(!super::merge_choice(&obj1, &obj2));
    }

    #[test]
    fn test_stackobj() {
        let obj = Object::new(ObjectId(1), 100, ObjectClass::Coin);
        let mut objects = vec![Object::new(ObjectId(2), 100, ObjectClass::Coin)];
        objects[0].quantity = 10;

        let result = super::stackobj(&obj, &mut objects);
        assert!(result.is_some());
        assert_eq!(objects[0].quantity, 11); // 10 + 1
    }

    #[test]
    fn test_stackobj_no_match() {
        let obj = Object::new(ObjectId(1), 100, ObjectClass::Coin);
        let mut objects = vec![Object::new(ObjectId(2), 101, ObjectClass::Coin)];

        let result = super::stackobj(&obj, &mut objects);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_oid() {
        let objects = vec![
            Object::new(ObjectId(1), 1, ObjectClass::Weapon),
            Object::new(ObjectId(2), 2, ObjectClass::Armor),
            Object::new(ObjectId(3), 3, ObjectClass::Potion),
        ];

        assert_eq!(super::find_oid(&objects, ObjectId(2)), Some(1));
        assert_eq!(super::find_oid(&objects, ObjectId(99)), None);
    }

    #[test]
    fn test_sobj_at() {
        let mut obj1 = Object::new(ObjectId(1), 100, ObjectClass::Weapon);
        obj1.x = 5;
        obj1.y = 10;
        obj1.location = ObjectLocation::Floor;

        let mut obj2 = Object::new(ObjectId(2), 100, ObjectClass::Weapon);
        obj2.x = 5;
        obj2.y = 10;
        obj2.location = ObjectLocation::PlayerInventory; // Not on floor

        let objects = vec![obj1, obj2];

        assert_eq!(super::sobj_at(&objects, 100, 5, 10), Some(0));
        assert_eq!(super::sobj_at(&objects, 100, 1, 1), None);
    }

    #[test]
    fn test_g_at() {
        let mut gold = Object::new(ObjectId(1), 0, ObjectClass::Coin);
        gold.x = 5;
        gold.y = 10;
        gold.location = ObjectLocation::Floor;

        let objects = vec![gold];

        assert_eq!(super::g_at(&objects, 5, 10), Some(0));
        assert_eq!(super::g_at(&objects, 1, 1), None);
    }

    #[test]
    fn test_obj_here() {
        let mut obj = Object::new(ObjectId(1), 100, ObjectClass::Weapon);
        obj.x = 5;
        obj.y = 10;
        obj.location = ObjectLocation::Floor;

        let objects = vec![obj];

        assert!(super::obj_here(&objects, 100, 5, 10));
        assert!(!super::obj_here(&objects, 100, 1, 1));
        assert!(!super::obj_here(&objects, 999, 5, 10));
    }

    #[test]
    fn test_nxtobj() {
        let objects = vec![
            Object::new(ObjectId(1), 100, ObjectClass::Weapon),
            Object::new(ObjectId(2), 200, ObjectClass::Armor),
            Object::new(ObjectId(3), 100, ObjectClass::Weapon),
        ];

        assert_eq!(super::nxtobj(&objects, 0, 100), Some(2));
        assert_eq!(super::nxtobj(&objects, 2, 100), None);
    }

    #[test]
    fn test_cursed_object_at() {
        let mut cursed_obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        cursed_obj.x = 5;
        cursed_obj.y = 10;
        cursed_obj.location = ObjectLocation::Floor;
        cursed_obj.buc = BucStatus::Cursed;

        let mut blessed_obj = Object::new(ObjectId(2), 1, ObjectClass::Armor);
        blessed_obj.x = 5;
        blessed_obj.y = 10;
        blessed_obj.location = ObjectLocation::Floor;
        blessed_obj.buc = BucStatus::Blessed;

        let objects = vec![cursed_obj, blessed_obj];

        assert!(super::cursed_object_at(&objects, 5, 10));
        assert!(!super::cursed_object_at(&objects, 1, 1));
    }

    #[test]
    fn test_obj_resists_damage_artifact() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.artifact = 1;
        assert!(super::obj_resists_damage(&obj, 0)); // Fire
        assert!(super::obj_resists_damage(&obj, 1)); // Cold
    }

    #[test]
    fn test_obj_resists_damage_blessed() {
        let mut obj = Object::new(ObjectId(1), 1, ObjectClass::Weapon);
        obj.buc = BucStatus::Blessed;
        // Blessed iron weapon vs fire should resist well
        assert!(super::obj_resists_damage(&obj, 0));
    }
}
