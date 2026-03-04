//! Player attributes (STR, INT, WIS, DEX, CON, CHA)

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::consts::NUM_ATTRS;
use crate::rng::GameRng;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Attribute type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumIter)]
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
    /// Full name of the attribute (alias for full_name)
    pub const fn name(&self) -> &'static str {
        self.full_name()
    }

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

    /// Full name of the attribute (attr2attrname equivalent)
    pub const fn full_name(&self) -> &'static str {
        match self {
            Attribute::Strength => "strength",
            Attribute::Intelligence => "intelligence",
            Attribute::Wisdom => "wisdom",
            Attribute::Dexterity => "dexterity",
            Attribute::Constitution => "constitution",
            Attribute::Charisma => "charisma",
        }
    }

    /// Create from index (0-5)
    pub const fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Attribute::Strength),
            1 => Some(Attribute::Intelligence),
            2 => Some(Attribute::Wisdom),
            3 => Some(Attribute::Dexterity),
            4 => Some(Attribute::Constitution),
            5 => Some(Attribute::Charisma),
            _ => None,
        }
    }

    /// Get index (0-5)
    pub const fn index(&self) -> usize {
        *self as usize
    }

    /// All attributes in order
    pub const ALL: [Attribute; 6] = [
        Attribute::Strength,
        Attribute::Intelligence,
        Attribute::Wisdom,
        Attribute::Dexterity,
        Attribute::Constitution,
        Attribute::Charisma,
    ];
}

/// Convert attribute index to name (attr2attrname equivalent)
pub fn attr2attrname(idx: usize) -> Option<&'static str> {
    Attribute::from_index(idx).map(|a| a.full_name())
}

/// Get attribute value description (attrval equivalent)
/// Returns a description like "very weak", "average", "strong", etc.
pub fn attrval(attr: Attribute, value: i8) -> &'static str {
    // Strength has special handling for 18/xx values
    if attr == Attribute::Strength && value > 18 {
        return if value < 50 {
            "very strong"
        } else if value < 90 {
            "extremely strong"
        } else {
            "supernaturally strong"
        };
    }

    // General attribute descriptions
    match value {
        ..=3 => "very weak",
        4..=5 => "weak",
        6..=7 => "below average",
        8..=10 => "average",
        11..=13 => "above average",
        14..=15 => "good",
        16..=17 => "very good",
        18 => "excellent",
        19..=21 => "superb",
        22..=24 => "extraordinary",
        _ => "supernatural",
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

    /// Base carry capacity from strength and constitution (C: weight_cap)
    ///
    /// Formula: `25 * (STR + CON) + 50`, matching C NetHack's weight_cap().
    /// STR values above 18 use the 18/xx encoding (e.g., 19 = 18/01, 118 = 18/**).
    pub fn base_carry_capacity(&self) -> i32 {
        let str_val = self.get(Attribute::Strength) as i32;
        let con_val = self.get(Attribute::Constitution) as i32;
        let cap = 25 * (str_val + con_val) + 50;
        cap.min(crate::MAX_CARR_CAP)
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
            18 => 3,
            _ => 4,
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

// Utility functions

/// Format strength value for display (get_strength_str equivalent)
pub fn format_strength(strength: i8) -> String {
    if strength <= 18 {
        format!("{}", strength)
    } else if strength <= 121 {
        // 18/01 to 18/99 format
        format!("18/{:02}", strength - 18)
    } else {
        // 18/100 or higher
        "18/**".to_string()
    }
}

/// Record attribute exercise (C: exercise() from attrib.c:413)
///
/// Free function to avoid borrow conflicts in gameloop.
/// Consumes rn2(19) for gains (harder at higher attribute values),
/// rn2(2) for losses (50% chance). Capped at |AVAL|=50.
pub fn record_exercise(
    exercise: &mut [i8; 6],
    attr_current: &Attributes,
    is_polymorphed: bool,
    attr: Attribute,
    gaining: bool,
    rng: &mut GameRng,
) {
    const AVAL: i8 = 50;

    // C: if (i == A_INT || i == A_CHA) return;
    if matches!(attr, Attribute::Intelligence | Attribute::Charisma) {
        return;
    }

    // C: if (Upolyd && i != A_WIS) return;
    if is_polymorphed && attr != Attribute::Wisdom {
        return;
    }

    let idx = attr.index();
    // C: if (abs(AEXE(i)) < AVAL)
    if exercise[idx].abs() < AVAL {
        // C: AEXE(i) += (inc_or_dec) ? (rn2(19) > ACURR(i)) : -rn2(2);
        if gaining {
            let roll = rng.rn2(19) as i8;
            let cur = attr_current.get(attr);
            if roll > cur {
                exercise[idx] = exercise[idx].saturating_add(1);
            }
        } else {
            let roll = rng.rn2(2) as i8;
            exercise[idx] = exercise[idx].saturating_sub(roll);
        }
    }
}

/// Exercise/abuse flavor text (C: exertext from attrib.c:517-524)
///
/// Returns the message shown when an attribute changes due to exercise/abuse.
pub fn exercise_message(attr_idx: usize, gained: bool) -> Option<String> {
    let texts: [(Option<&str>, Option<&str>); 6] = [
        (Some("exercising diligently"), Some("exercising properly")), // Str
        (None, None),                                                 // Int
        (Some("very observant"), Some("paying attention")),           // Wis
        (
            Some("working on your reflexes"),
            Some("working on reflexes lately"),
        ), // Dex
        (
            Some("leading a healthy life-style"),
            Some("watching your health"),
        ), // Con
        (None, None),                                                 // Cha
    ];

    if attr_idx >= 6 {
        return None;
    }

    let (gain_text, loss_text) = texts[attr_idx];
    if gained {
        gain_text.map(|t| format!("You must have been {}.", t))
    } else {
        loss_text.map(|t| format!("You haven't been {}.", t))
    }
}
