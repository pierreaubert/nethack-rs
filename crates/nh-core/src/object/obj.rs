//! Object instances (obj.h)

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

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
            age: 0,
            contents: Vec::new(),
            name: None,
            shop_price: 0,
            unpaid: false,
            base_ac: 0,
            damage_dice: 0,
            damage_sides: 0,
            weapon_tohit: 0,
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

    /// Check if can merge with another object
    pub fn can_merge(&self, other: &Object) -> bool {
        if !self.class.stacks() {
            return false;
        }

        self.object_type == other.object_type
            && self.buc == other.buc
            && self.enchantment == other.enchantment
            && self.erosion1 == other.erosion1
            && self.erosion2 == other.erosion2
            && self.poisoned == other.poisoned
            && self.name == other.name
    }

    /// Merge another object into this one
    pub fn merge(&mut self, other: Object) {
        self.quantity += other.quantity;
        self.weight += other.weight;
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

    /// Get erosion description prefix
    pub fn erosion_prefix(&self) -> String {
        let mut prefix = String::new();

        // First erosion type (rust/burn)
        if self.erosion1 > 0 {
            match self.erosion1 {
                2 => prefix.push_str("very "),
                3 => prefix.push_str("thoroughly "),
                _ => {}
            }
            // Assume metal = rusty, otherwise burnt
            if matches!(self.class, ObjectClass::Weapon | ObjectClass::Armor) {
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
            if matches!(self.class, ObjectClass::Weapon | ObjectClass::Armor) {
                prefix.push_str("corroded ");
            } else {
                prefix.push_str("rotted ");
            }
        }

        // Erosion-proof status (if known)
        if self.rust_known && self.erosion_proof {
            match self.class {
                ObjectClass::Weapon | ObjectClass::Armor => prefix.push_str("rustproof "),
                _ => prefix.push_str("fireproof "),
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
    pub fn enchantment_str(&self) -> String {
        if !self.known {
            return String::new();
        }
        match self.class {
            ObjectClass::Weapon | ObjectClass::Armor => {
                format!("{:+} ", self.enchantment)
            }
            ObjectClass::Ring | ObjectClass::Wand => {
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

        // BUC status
        let buc = self.buc_prefix();
        if !buc.is_empty() {
            parts.push(buc.trim().to_string());
        }

        // Enchantment
        let ench = self.enchantment_str();
        if !ench.is_empty() {
            parts.push(ench.trim().to_string());
        }

        // Greased
        if self.greased {
            parts.push("greased".to_string());
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

        // Base name
        parts.push(base_name.to_string());

        // Build the main string
        let mut result = parts.join(" ");

        // Add suffixes
        result.push_str(&self.charges_suffix());
        result.push_str(self.worn_suffix());

        // Container contents
        if self.is_container() && !self.contents.is_empty() {
            result.push_str(&format!(" (containing {} item{})",
                self.contents.len(),
                if self.contents.len() == 1 { "" } else { "s" }
            ));
        }

        // Lit status for light sources
        if self.lit {
            result.push_str(" (lit)");
        }

        result
    }

    /// Simple name without quantity or article (like NetHack's xname)
    pub fn xname(&self, base_name: &str) -> String {
        let mut parts = Vec::new();

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

        // Base name
        parts.push(base_name.to_string());

        parts.join(" ")
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
        assert!(name.starts_with("5 "));
        assert!(name.contains("arrow"));
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
}
