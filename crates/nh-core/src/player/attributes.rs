//! Player attributes (STR, INT, WIS, DEX, CON, CHA)

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::consts::NUM_ATTRS;
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

/// Extended attribute tracking with modifiers
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AttributeModifiers {
    /// Base attribute values
    pub base: [i8; NUM_ATTRS],
    /// Maximum attainable values
    pub max: [i8; NUM_ATTRS],
    /// Temporary modifiers (equipment, effects)
    pub temp: [i8; NUM_ATTRS],
    /// Equipment bonuses
    pub bonus: [i8; NUM_ATTRS],
    /// Exercise values (for attribute growth)
    pub exercise: [i8; NUM_ATTRS],
}

impl AttributeModifiers {
    /// Create new modifiers with given base values (init_attr equivalent)
    ///
    /// Initializes attributes based on role base values and distribution points.
    /// The points parameter represents ability points to distribute.
    pub fn init(role_base: [i8; NUM_ATTRS], dist_points: i32) -> Self {
        let mut result = Self {
            base: role_base,
            max: role_base,
            temp: [0; NUM_ATTRS],
            bonus: [0; NUM_ATTRS],
            exercise: [0; NUM_ATTRS],
        };

        // Distribute bonus points according to role distribution weights
        let mut remaining = dist_points;
        let mut try_count = 0;

        while remaining > 0 && try_count < 100 {
            // Pick random attribute biased toward role strengths
            let idx = (try_count % NUM_ATTRS) as usize;

            // Can't go above max
            if result.base[idx] >= 25 {
                try_count += 1;
                continue;
            }

            try_count = 0;
            result.base[idx] = result.base[idx].saturating_add(1);
            result.max[idx] = result.max[idx].saturating_add(1);
            remaining -= 1;
        }

        result
    }

    /// Get current attribute value with all modifiers applied (acurr equivalent)
    pub fn current(&self, attr: Attribute) -> i8 {
        let idx = attr.index();
        let sum = self.base[idx]
            .saturating_add(self.temp[idx])
            .saturating_add(self.bonus[idx]);

        // Clamp to valid range
        sum.clamp(3, 25)
    }

    /// Get current strength with special handling (acurrstr equivalent)
    ///
    /// Condenses strength values > 18 into formula-friendly range.
    /// Maps strength >= 18 to condensed values:
    /// - 18 -> 18
    /// - 18/01..18/50 -> 19
    /// - 18/51..18/99 -> 20
    /// - 18/100 -> 21
    /// - 19..24 -> 22..25
    pub fn current_strength_condensed(&self) -> i8 {
        let str_val = self.current(Attribute::Strength);

        if str_val <= 18 {
            str_val
        } else if str_val <= 121 {
            19 + (str_val - 18) / 50
        } else {
            (str_val - 100).min(25)
        }
    }

    /// Check if attribute is at extreme (min or max) (extremeattr equivalent)
    pub fn is_extreme(&self, attr: Attribute) -> bool {
        let current = self.current(attr);
        current <= 3 || current >= 25
    }

    /// Adjust attribute value (adjattrib equivalent - simplified)
    ///
    /// Returns true if the attribute changed.
    /// This is a simplified version without message generation.
    pub fn adjust(&mut self, attr: Attribute, delta: i8) -> bool {
        if delta == 0 {
            return false;
        }

        let idx = attr.index();
        let old_current = self.current(attr);

        if delta > 0 {
            // Increasing attribute
            self.base[idx] = self.base[idx].saturating_add(delta);
            if self.base[idx] > self.max[idx] {
                self.max[idx] = self.base[idx];
                if self.max[idx] > 25 {
                    self.base[idx] = 25;
                    self.max[idx] = 25;
                }
            }
        } else {
            // Decreasing attribute
            self.base[idx] = self.base[idx].saturating_sub(delta.abs() as i8);
            if self.base[idx] < 3 {
                // If base drops below minimum, reduce max instead (permanent loss)
                let loss = 3 - self.base[idx];
                self.base[idx] = 3;
                self.max[idx] = (self.max[idx] - loss).max(3);
            }
        }

        self.current(attr) != old_current
    }

    /// Record attribute exercise (exercise equivalent - simplified)
    ///
    /// Tracks practice toward attribute improvement based on activity.
    pub fn record_exercise(&mut self, attr: Attribute, gaining: bool) {
        let idx = attr.index();

        // Can't exercise Int or Cha; physical attrs only when not polymorphed
        if matches!(attr, Attribute::Intelligence | Attribute::Charisma) {
            return;
        }

        // Law of diminishing returns:
        // - Gaining is harder at higher values (0% at 18)
        // - Loss is even at all levels (50%)
        let val = self.current(attr);
        if gaining {
            // Higher attributes make exercise less likely
            // Probability inversely related to current value
            if (val as i32 + (idx as i32 * 10)) % 19 > val as i32 {
                self.exercise[idx] = self.exercise[idx].saturating_add(1);
            }
        } else {
            self.exercise[idx] = self.exercise[idx].saturating_sub(1);
        }
    }

    /// Redistribute attributes (redist_attr equivalent - simplified)
    ///
    /// Called when polymorphing to adjust physical attributes.
    /// Int and Wis are not changed.
    pub fn redistribute(&mut self) {
        for attr in [
            Attribute::Strength,
            Attribute::Dexterity,
            Attribute::Constitution,
        ] {
            let idx = attr.index();
            let old_max = self.max[idx];

            // Adjust max by -2 to +2
            let delta = ((idx as i8) % 5) - 2; // Pseudo-random
            self.max[idx] = (self.max[idx].saturating_add(delta)).clamp(3, 25);

            // Adjust base proportionally
            if old_max > 0 {
                self.base[idx] =
                    (self.base[idx] as i32 * self.max[idx] as i32 / old_max as i32) as i8;
            }
            self.base[idx] = self.base[idx].clamp(3, 25);
        }
    }

    /// Get to-hit/damage bonus from ability scores (abon equivalent - simplified)
    ///
    /// Calculates contribution to attack roll from Strength and Dexterity.
    /// Returns modifier that should be added to roll.
    pub fn ability_bonus(&self, player_level: i32) -> i8 {
        let str_val = self.current(Attribute::Strength);
        let dex_val = self.current(Attribute::Dexterity);

        let str_bonus = match str_val {
            ..=5 => -2,
            6..=7 => -1,
            8..=16 => 0,
            17..=18 => 1,
            19..=100 => 2,
            _ => 3,
        };

        let dex_bonus = match dex_val {
            ..=3 => -3,
            4 => -2,
            5 => -1,
            6..=14 => 0,
            15 => 1,
            16 => 2,
            _ => 3,
        };

        let low_level_bonus = if player_level < 3 { 1 } else { 0 };

        (str_bonus + dex_bonus + low_level_bonus).clamp(-3, 3)
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
        format!("18/**")
    }
}

/// Adjust attribute from worn item (adj_abon equivalent - stub)
///
/// Handles specific items that modify attributes (e.g., Gauntlets of Dexterity).
/// This is a stub - full implementation requires object types.
pub fn adjust_from_item(modifiers: &mut AttributeModifiers, item_type: &str, equipping: bool) {
    let delta = if equipping { 1 } else { -1 };

    match item_type {
        "gauntlets_of_dexterity" => {
            modifiers.bonus[Attribute::Dexterity.index()] =
                modifiers.bonus[Attribute::Dexterity.index()].saturating_add(delta);
        }
        "helm_of_brilliance" => {
            modifiers.bonus[Attribute::Intelligence.index()] =
                modifiers.bonus[Attribute::Intelligence.index()].saturating_add(delta);
            modifiers.bonus[Attribute::Wisdom.index()] =
                modifiers.bonus[Attribute::Wisdom.index()].saturating_add(delta);
        }
        _ => {}
    }
}

/// Check if an innate ability source matches requirements (innately equivalent - stub)
///
/// Determines if an ability is innate (from role, race, or form).
/// Full implementation requires tracking ability sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InnatSource {
    /// Not innate
    None = 0,
    /// From character role
    Role = 1,
    /// From character race
    Race = 2,
    /// From form/polymorph
    Form = 3,
    /// External source (cursed item, etc.)
    External = 4,
}

pub fn check_innate_ability(ability_id: u32, player_level: u32) -> InnatSource {
    // Check if an ability is innate to role/race at a given level
    // Ability ID would map to specific abilities in full implementation

    match ability_id {
        1 => {
            // Example: Some role-specific ability at level 1
            if player_level >= 1 {
                InnatSource::Role
            } else {
                InnatSource::None
            }
        }
        2 => {
            // Example: Some race-specific ability at level 1
            if player_level >= 1 {
                InnatSource::Race
            } else {
                InnatSource::None
            }
        }
        3 => {
            // Example: Some ability gained at higher level
            if player_level >= 5 {
                InnatSource::Role
            } else {
                InnatSource::None
            }
        }
        _ => InnatSource::None,
    }
}

/// Apply random intrinsic curse effect (attrcurse equivalent)
///
/// Removes a random intrinsic property (curse effect).
/// Returns the name of the property removed, if any.
pub fn apply_attribute_curse(rng: &mut crate::GameRng) -> Option<&'static str> {
    // Randomly remove an intrinsic property (curse effect)
    // Examples: Fire resistance, Teleportation, Poison resistance, etc.
    let curses = [
        "fire resistance",
        "teleportation",
        "poison resistance",
        "telepathy",
        "cold resistance",
        "invisibility",
        "see invisible",
        "speed",
        "regeneration",
        "magical resistance",
        "a special power",
    ];

    // Pick a random curse from the list
    let idx = (rng.rn2(curses.len() as u32) as usize).min(curses.len() - 1);
    Some(curses[idx])
}

/// Perform periodic attribute exercise (exerper equivalent - stub)
///
/// Called every 10 turns to apply exercise based on hunger and encumbrance.
/// Full implementation requires player state and hunger tracking.
pub fn periodic_exercise(
    modifiers: &mut AttributeModifiers,
    hunger_state: &str,
    encumbrance_level: i32,
) {
    // Simplified version - real one checks hunger state and encumbrance
    match hunger_state {
        "satiated" => {
            modifiers.record_exercise(Attribute::Dexterity, false);
        }
        "normal" => {
            modifiers.record_exercise(Attribute::Constitution, true);
        }
        "weak" => {
            modifiers.record_exercise(Attribute::Strength, false);
        }
        _ => {}
    }

    // Encumbrance effects
    match encumbrance_level {
        1 => {
            // Moderately encumbered
            modifiers.record_exercise(Attribute::Strength, true);
        }
        2 => {
            // Heavily encumbered
            modifiers.record_exercise(Attribute::Strength, true);
            modifiers.record_exercise(Attribute::Dexterity, false);
        }
        3 => {
            // Extremely encumbered
            modifiers.record_exercise(Attribute::Dexterity, false);
            modifiers.record_exercise(Attribute::Constitution, false);
        }
        _ => {}
    }
}
