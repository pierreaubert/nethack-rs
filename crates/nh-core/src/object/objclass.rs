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
