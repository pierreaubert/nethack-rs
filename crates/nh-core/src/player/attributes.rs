//! Player attributes (STR, INT, WIS, DEX, CON, CHA)

use crate::consts::NUM_ATTRS;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Attribute type
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum Attribute {
    Strength = 0,
    Intelligence = 1,
    Wisdom = 2,
    Dexterity = 3,
    Constitution = 4,
    Charisma = 5,
}

impl Attribute {
    /// Short name for display
    pub const fn short_name(&self) -> &'static str {
        match self {
            Attribute::Strength => "St",
            Attribute::Intelligence => "In",
            Attribute::Wisdom => "Wi",
            Attribute::Dexterity => "Dx",
            Attribute::Constitution => "Co",
            Attribute::Charisma => "Ch",
        }
    }
}

/// Player attributes set
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Attributes {
    /// Current attribute values
    values: [i8; NUM_ATTRS],
}

impl Attributes {
    /// Create new attributes with given values
    pub const fn new(values: [i8; NUM_ATTRS]) -> Self {
        Self { values }
    }

    /// Get an attribute value
    pub const fn get(&self, attr: Attribute) -> i8 {
        self.values[attr as usize]
    }

    /// Set an attribute value
    pub fn set(&mut self, attr: Attribute, value: i8) {
        self.values[attr as usize] = value.clamp(3, 25);
    }

    /// Modify an attribute by delta
    pub fn modify(&mut self, attr: Attribute, delta: i8) {
        let new_value = self.values[attr as usize].saturating_add(delta);
        self.set(attr, new_value);
    }

    /// Get strength as display string (handles 18/** notation)
    pub fn strength_string(&self) -> String {
        let str = self.get(Attribute::Strength);
        if str <= 18 {
            str.to_string()
        } else if str < 118 {
            format!("18/{:02}", str - 18)
        } else {
            "18/**".to_string()
        }
    }

    /// Get to-hit bonus from strength
    pub fn strength_to_hit_bonus(&self) -> i8 {
        let str = self.get(Attribute::Strength);
        match str {
            ..=5 => -2,
            6..=7 => -1,
            8..=16 => 0,
            17 => 1,
            18..=118 => (str - 18) / 25 + 1,
            _ => 3,
        }
    }

    /// Get damage bonus from strength
    pub fn strength_damage_bonus(&self) -> i8 {
        let str = self.get(Attribute::Strength);
        match str {
            ..=5 => -1,
            6..=15 => 0,
            16 => 1,
            17 => 2,
            18..=40 => 2,
            41..=68 => 3,
            69..=92 => 4,
            93..=117 => 5,
            _ => 6,
        }
    }

    /// Get carry capacity from strength
    pub fn carry_capacity(&self) -> i32 {
        let str = self.get(Attribute::Strength);
        let base = match str {
            ..=2 => 120,
            3 => 250,
            4 => 400,
            5 => 450,
            6 => 500,
            7 => 550,
            8 => 600,
            9 => 650,
            10 => 700,
            11 => 750,
            12 => 800,
            13 => 850,
            14 => 900,
            15 => 950,
            16 => 1000,
            17 => 1050,
            18 => 1100,
            _ => 1100 + ((str - 18) as i32) * 25,
        };
        base.min(2500)
    }

    /// Get AC bonus from dexterity
    pub fn dexterity_ac_bonus(&self) -> i8 {
        let dex = self.get(Attribute::Dexterity);
        match dex {
            ..=3 => 3,
            4 => 2,
            5 => 1,
            6..=14 => 0,
            15 => -1,
            16 => -2,
            17 => -3,
            _ => -4,
        }
    }

    /// Get to-hit bonus from dexterity
    pub fn dexterity_to_hit_bonus(&self) -> i8 {
        let dex = self.get(Attribute::Dexterity);
        match dex {
            ..=3 => -3,
            4 => -2,
            5 => -1,
            6..=14 => 0,
            15 => 1,
            16 => 2,
            _ => 3,
        }
    }

    /// Get HP bonus from constitution
    pub fn constitution_hp_bonus(&self) -> i8 {
        let con = self.get(Attribute::Constitution);
        match con {
            ..=3 => -2,
            4..=6 => -1,
            7..=14 => 0,
            15..=16 => 1,
            17 => 2,
            _ => 3,
        }
    }

    /// Get charisma-based price modifier (0-100+ percent)
    pub fn charisma_price_modifier(&self) -> i32 {
        let cha = self.get(Attribute::Charisma);
        match cha {
            ..=5 => 150,
            6..=7 => 140,
            8..=10 => 120,
            11..=15 => 100,
            16..=17 => 90,
            18..=24 => 80,
            _ => 70,
        }
    }
}
