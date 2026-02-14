//! Object class definitions (objclass.h)

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Material types
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum Material {
    Liquid = 1,
    Wax = 2,
    Veggy = 3,
    Flesh = 4,
    Paper = 5,
    Cloth = 6,
    Leather = 7,
    Wood = 8,
    Bone = 9,
    DragonHide = 10,
    #[default]
    Iron = 11,
    Metal = 12,
    Copper = 13,
    Silver = 14,
    Gold = 15,
    Platinum = 16,
    Mithril = 17,
    Plastic = 18,
    Glass = 19,
    Gemstone = 20,
    Mineral = 21,
}

impl Material {
    /// Check if this material is metallic
    pub const fn is_metallic(&self) -> bool {
        matches!(
            self,
            Material::Iron
                | Material::Metal
                | Material::Copper
                | Material::Silver
                | Material::Gold
                | Material::Platinum
                | Material::Mithril
        )
    }

    /// Check if this material rusts
    pub const fn rusts(&self) -> bool {
        matches!(self, Material::Iron)
    }

    /// Check if this material corrodes
    pub const fn corrodes(&self) -> bool {
        matches!(self, Material::Copper | Material::Iron)
    }

    /// Check if this material burns
    pub const fn burns(&self) -> bool {
        matches!(
            self,
            Material::Wood | Material::Paper | Material::Cloth | Material::Leather
        )
    }

    /// Check if this material rots
    pub const fn rots(&self) -> bool {
        matches!(
            self,
            Material::Leather | Material::Wood | Material::Veggy | Material::Flesh
        )
    }
}

/// Object classes
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum ObjectClass {
    #[default]
    Random = 0,
    IllObj = 1,
    Weapon = 2,
    Armor = 3,
    Ring = 4,
    Amulet = 5,
    Tool = 6,
    Food = 7,
    Potion = 8,
    Scroll = 9,
    Spellbook = 10,
    Wand = 11,
    Coin = 12,
    Gem = 13,
    Rock = 14,
    Ball = 15,
    Chain = 16,
    Venom = 17,
}

impl ObjectClass {
    /// Get the inventory symbol for this class
    pub const fn symbol(&self) -> char {
        match self {
            ObjectClass::Random => '?',
            ObjectClass::IllObj => ']',
            ObjectClass::Weapon => ')',
            ObjectClass::Armor => '[',
            ObjectClass::Ring => '=',
            ObjectClass::Amulet => '"',
            ObjectClass::Tool => '(',
            ObjectClass::Food => '%',
            ObjectClass::Potion => '!',
            ObjectClass::Scroll => '?',
            ObjectClass::Spellbook => '+',
            ObjectClass::Wand => '/',
            ObjectClass::Coin => '$',
            ObjectClass::Gem => '*',
            ObjectClass::Rock => '`',
            ObjectClass::Ball => '0',
            ObjectClass::Chain => '_',
            ObjectClass::Venom => '.',
        }
    }

    /// Check if objects of this class can be enchanted
    pub const fn can_enchant(&self) -> bool {
        matches!(self, ObjectClass::Weapon | ObjectClass::Armor)
    }

    /// Check if objects of this class have charges
    pub const fn has_charges(&self) -> bool {
        matches!(self, ObjectClass::Wand | ObjectClass::Tool)
    }

    /// Check if objects of this class stack
    pub const fn stacks(&self) -> bool {
        matches!(
            self,
            ObjectClass::Coin
                | ObjectClass::Gem
                | ObjectClass::Rock
                | ObjectClass::Food
                | ObjectClass::Potion
                | ObjectClass::Scroll
                | ObjectClass::Weapon // some weapons
        )
    }

    /// Get object class from symbol (def_char_to_objclass equivalent)
    ///
    /// Returns the ObjectClass for a given inventory symbol, or None if not found.
    pub const fn from_symbol(ch: char) -> Option<Self> {
        match ch {
            ')' => Some(ObjectClass::Weapon),
            '[' => Some(ObjectClass::Armor),
            '=' => Some(ObjectClass::Ring),
            '"' => Some(ObjectClass::Amulet),
            '(' => Some(ObjectClass::Tool),
            '%' => Some(ObjectClass::Food),
            '!' => Some(ObjectClass::Potion),
            '?' => Some(ObjectClass::Scroll),
            '+' => Some(ObjectClass::Spellbook),
            '/' => Some(ObjectClass::Wand),
            '$' => Some(ObjectClass::Coin),
            '*' => Some(ObjectClass::Gem),
            '`' => Some(ObjectClass::Rock),
            '0' => Some(ObjectClass::Ball),
            '_' => Some(ObjectClass::Chain),
            '.' => Some(ObjectClass::Venom),
            ']' => Some(ObjectClass::IllObj),
            _ => None,
        }
    }

    /// Get object class name as string
    pub const fn name(&self) -> &'static str {
        match self {
            ObjectClass::Random => "random",
            ObjectClass::IllObj => "illegal object",
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
            ObjectClass::Coin => "coin",
            ObjectClass::Gem => "gem",
            ObjectClass::Rock => "rock",
            ObjectClass::Ball => "ball",
            ObjectClass::Chain => "chain",
            ObjectClass::Venom => "venom",
        }
    }
}

/// Armor categories
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum ArmorCategory {
    #[default]
    Suit = 0,
    Shield = 1,
    Helm = 2,
    Gloves = 3,
    Boots = 4,
    Cloak = 5,
    Shirt = 6,
}

impl ArmorCategory {
    /// Get the worn mask bit for this armor slot
    pub const fn worn_mask(&self) -> u32 {
        1 << (*self as u32)
    }
}

/// Wand/spell direction types
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum DirectionType {
    #[default]
    None = 0,
    NonDirectional = 1,
    Immediate = 2,
    Ray = 3,
}

/// Object class definition (static data)
#[derive(Debug, Clone)]
pub struct ObjClassDef {
    /// Object name
    pub name: &'static str,

    /// Object description (unidentified appearance)
    pub description: &'static str,

    /// Object class
    pub class: ObjectClass,

    /// Material
    pub material: Material,

    /// Weight
    pub weight: u16,

    /// Base cost
    pub cost: i16,

    /// Generation probability
    pub probability: i16,

    /// Nutrition (food only)
    pub nutrition: u16,

    /// Weapon: small monster damage dice
    pub w_small_damage: u8,

    /// Weapon: large monster damage dice
    pub w_large_damage: u8,

    /// Weapon/armor: to-hit bonus or AC
    pub bonus: i8,

    /// Weapon skill type
    pub skill: i8,

    /// Use delay
    pub delay: i8,

    /// Color for display
    pub color: u8,

    /// Is magical
    pub magical: bool,

    /// Merges with similar objects
    pub merge: bool,

    /// Unique object
    pub unique: bool,

    /// Cannot be wished for
    pub no_wish: bool,

    /// Big (two-handed weapon / bulky armor)
    pub big: bool,

    /// Direction type (wands)
    pub direction: DirectionType,

    /// Armor category
    pub armor_category: Option<ArmorCategory>,

    /// Property conveyed when worn/wielded
    pub property: u8,
}

impl ObjClassDef {
    /// Check if this is a weapon
    pub const fn is_weapon(&self) -> bool {
        matches!(self.class, ObjectClass::Weapon)
    }

    /// Check if this is armor
    pub const fn is_armor(&self) -> bool {
        matches!(self.class, ObjectClass::Armor)
    }

    /// Check if this is a wand
    pub const fn is_wand(&self) -> bool {
        matches!(self.class, ObjectClass::Wand)
    }

    /// Check if this is food
    pub const fn is_food(&self) -> bool {
        matches!(self.class, ObjectClass::Food)
    }
}

// ============================================================================
// Free functions (C-style API equivalents)
// ============================================================================

/// Convert a character symbol to object class (def_char_to_objclass equivalent)
pub const fn def_char_to_objclass(ch: char) -> Option<ObjectClass> {
    ObjectClass::from_symbol(ch)
}

// ============================================================================
// Discovery System (objclass.c - discover_object equivalent)
// ============================================================================

/// Knowledge about a discovered object type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredType {
    /// Has player identified this object type
    pub known: bool,
    /// Turn when discovered
    pub discovered_turn: u64,
}

impl DiscoveredType {
    pub fn new(turn: u64) -> Self {
        Self {
            known: true,
            discovered_turn: turn,
        }
    }
}

/// Discovery state for all object types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryState {
    /// Discovered objects by type index
    /// Key: object_type, Value: DiscoveredType
    pub discovered: std::collections::HashMap<i16, DiscoveredType>,
    /// Count of discovered types
    pub disco_count: usize,
}

impl Default for DiscoveryState {
    fn default() -> Self {
        Self {
            discovered: std::collections::HashMap::new(),
            disco_count: 0,
        }
    }
}

impl DiscoveryState {
    /// Mark an object type as discovered
    /// Returns true if this was newly discovered, false if already known
    pub fn discover_object(&mut self, object_type: i16, current_turn: u64) -> bool {
        if self.discovered.contains_key(&object_type) {
            false
        } else {
            self.discovered
                .insert(object_type, DiscoveredType::new(current_turn));
            self.disco_count += 1;
            true
        }
    }

    /// Check if an object type has been discovered
    pub fn is_discovered(&self, object_type: i16) -> bool {
        self.discovered.contains_key(&object_type)
    }

    /// Get count of discovered types
    pub fn count(&self) -> usize {
        self.disco_count
    }

    /// Clear all discoveries (for new game or testing)
    pub fn clear(&mut self) {
        self.discovered.clear();
        self.disco_count = 0;
    }

    /// Check if player knows a specific object class (knows_class equivalent)
    pub fn knows_class(&self, class: ObjectClass) -> bool {
        // Check if any object of this class has been discovered
        self.discovered.values().any(|_| true) // Simplified - would need class info
    }

    /// Check if player knows a specific object type (knows_object equivalent)
    pub fn knows_object(&self, object_type: i16) -> bool {
        self.is_discovered(object_type)
    }

    /// Undiscover an object type (undiscover_object equivalent)
    pub fn undiscover_object(&mut self, object_type: i16) -> bool {
        if self.discovered.remove(&object_type).is_some() {
            self.disco_count = self.disco_count.saturating_sub(1);
            true
        } else {
            false
        }
    }
}

// ============================================================================
// Object Class Helper Functions
// ============================================================================

/// Get the inventory letter for an object class (obj_to_let equivalent)
pub const fn obj_to_let(class: ObjectClass) -> char {
    class.symbol()
}

/// Get the name of an object class (oclass_to_name equivalent)
pub const fn oclass_to_name(class: ObjectClass) -> &'static str {
    class.name()
}

/// Convert inventory letter to class name (let_to_name equivalent)
pub fn let_to_name(ch: char) -> &'static str {
    match ObjectClass::from_symbol(ch) {
        Some(class) => class.name(),
        None => "strange object",
    }
}

/// Get a descriptive string for an object class (oc_to_str equivalent)
pub fn oc_to_str(class: ObjectClass) -> &'static str {
    match class {
        ObjectClass::Weapon => "weapons",
        ObjectClass::Armor => "armor",
        ObjectClass::Ring => "rings",
        ObjectClass::Amulet => "amulets",
        ObjectClass::Tool => "tools",
        ObjectClass::Food => "food",
        ObjectClass::Potion => "potions",
        ObjectClass::Scroll => "scrolls",
        ObjectClass::Spellbook => "spellbooks",
        ObjectClass::Wand => "wands",
        ObjectClass::Coin => "coins",
        ObjectClass::Gem => "gems",
        ObjectClass::Rock => "rocks",
        ObjectClass::Ball => "iron balls",
        ObjectClass::Chain => "chains",
        ObjectClass::Venom => "venom",
        ObjectClass::Random => "random objects",
        ObjectClass::IllObj => "illegal objects",
    }
}

/// Get the kind name for an object (kind_name equivalent)
/// Returns the base type name without enchantment or BUC status
pub fn kind_name(class: ObjectClass, object_type: i16) -> String {
    // Simplified - would need ObjClassDef lookup for full implementation
    let _ = object_type;
    class.name().to_string()
}

/// Check if an object type is interesting to discover (interesting_to_discover equivalent)
pub fn interesting_to_discover(class: ObjectClass) -> bool {
    matches!(
        class,
        ObjectClass::Ring
            | ObjectClass::Amulet
            | ObjectClass::Potion
            | ObjectClass::Scroll
            | ObjectClass::Spellbook
            | ObjectClass::Wand
            | ObjectClass::Gem
    )
}

/// Check if an object type can be called/named by the player (objtyp_is_callable equivalent)
pub fn objtyp_is_callable(class: ObjectClass) -> bool {
    matches!(
        class,
        ObjectClass::Ring
            | ObjectClass::Amulet
            | ObjectClass::Potion
            | ObjectClass::Scroll
            | ObjectClass::Spellbook
            | ObjectClass::Wand
            | ObjectClass::Gem
            | ObjectClass::Tool
    )
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_class_symbol() {
        assert_eq!(ObjectClass::Weapon.symbol(), ')');
        assert_eq!(ObjectClass::Armor.symbol(), '[');
        assert_eq!(ObjectClass::Potion.symbol(), '!');
        assert_eq!(ObjectClass::Scroll.symbol(), '?');
    }

    #[test]
    fn test_object_class_from_symbol() {
        assert_eq!(ObjectClass::from_symbol(')'), Some(ObjectClass::Weapon));
        assert_eq!(ObjectClass::from_symbol('['), Some(ObjectClass::Armor));
        assert_eq!(ObjectClass::from_symbol('!'), Some(ObjectClass::Potion));
        assert_eq!(ObjectClass::from_symbol('x'), None);
    }

    #[test]
    fn test_object_class_name() {
        assert_eq!(ObjectClass::Weapon.name(), "weapon");
        assert_eq!(ObjectClass::Armor.name(), "armor");
        assert_eq!(ObjectClass::Potion.name(), "potion");
    }

    #[test]
    fn test_def_char_to_objclass() {
        assert_eq!(def_char_to_objclass(')'), Some(ObjectClass::Weapon));
        assert_eq!(def_char_to_objclass('x'), None);
    }

    #[test]
    fn test_obj_to_let() {
        assert_eq!(obj_to_let(ObjectClass::Weapon), ')');
        assert_eq!(obj_to_let(ObjectClass::Coin), '$');
    }

    #[test]
    fn test_oclass_to_name() {
        assert_eq!(oclass_to_name(ObjectClass::Weapon), "weapon");
        assert_eq!(oclass_to_name(ObjectClass::Potion), "potion");
    }

    #[test]
    fn test_let_to_name() {
        assert_eq!(let_to_name(')'), "weapon");
        assert_eq!(let_to_name('!'), "potion");
        assert_eq!(let_to_name('x'), "strange object");
    }

    #[test]
    fn test_oc_to_str() {
        assert_eq!(oc_to_str(ObjectClass::Weapon), "weapons");
        assert_eq!(oc_to_str(ObjectClass::Potion), "potions");
    }

    #[test]
    fn test_interesting_to_discover() {
        assert!(interesting_to_discover(ObjectClass::Ring));
        assert!(interesting_to_discover(ObjectClass::Potion));
        assert!(!interesting_to_discover(ObjectClass::Weapon));
        assert!(!interesting_to_discover(ObjectClass::Food));
    }

    #[test]
    fn test_objtyp_is_callable() {
        assert!(objtyp_is_callable(ObjectClass::Ring));
        assert!(objtyp_is_callable(ObjectClass::Wand));
        assert!(!objtyp_is_callable(ObjectClass::Weapon));
        assert!(!objtyp_is_callable(ObjectClass::Armor));
    }

    #[test]
    fn test_discovery_state() {
        let mut state = DiscoveryState::default();

        assert!(!state.is_discovered(100));
        assert_eq!(state.count(), 0);

        assert!(state.discover_object(100, 1));
        assert!(state.is_discovered(100));
        assert_eq!(state.count(), 1);

        // Discovering again returns false
        assert!(!state.discover_object(100, 2));
        assert_eq!(state.count(), 1);
    }

    #[test]
    fn test_discovery_state_undiscover() {
        let mut state = DiscoveryState::default();
        state.discover_object(100, 1);

        assert!(state.undiscover_object(100));
        assert!(!state.is_discovered(100));
        assert_eq!(state.count(), 0);

        // Undiscovering again returns false
        assert!(!state.undiscover_object(100));
    }

    #[test]
    fn test_discovery_state_knows_object() {
        let mut state = DiscoveryState::default();

        assert!(!state.knows_object(100));
        state.discover_object(100, 1);
        assert!(state.knows_object(100));
    }

    #[test]
    fn test_material_properties() {
        assert!(Material::Iron.is_metallic());
        assert!(Material::Gold.is_metallic());
        assert!(!Material::Wood.is_metallic());

        assert!(Material::Iron.rusts());
        assert!(!Material::Gold.rusts());

        assert!(Material::Wood.burns());
        assert!(!Material::Iron.burns());
    }
}
