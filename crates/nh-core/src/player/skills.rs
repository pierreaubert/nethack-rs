//! Player weapon and spell skills

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Skill level
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum SkillLevel {
    /// Cannot use this skill
    #[default]
    Restricted = 0,
    /// Penalty to use
    Unskilled = 1,
    /// Basic competency
    Basic = 2,
    /// Good competency
    Skilled = 3,
    /// High competency
    Expert = 4,
    /// Master level (special skills only)
    Master = 5,
    /// Grand master (martial arts only)
    GrandMaster = 6,
}

impl SkillLevel {
    /// Get to-hit modifier for this skill level
    pub const fn to_hit_bonus(&self) -> i8 {
        match self {
            SkillLevel::Restricted => -4,
            SkillLevel::Unskilled => -2,
            SkillLevel::Basic => 0,
            SkillLevel::Skilled => 1,
            SkillLevel::Expert => 2,
            SkillLevel::Master => 3,
            SkillLevel::GrandMaster => 4,
        }
    }

    /// Get damage modifier for this skill level
    pub const fn damage_bonus(&self) -> i8 {
        match self {
            SkillLevel::Restricted => -2,
            SkillLevel::Unskilled => -1,
            SkillLevel::Basic => 0,
            SkillLevel::Skilled => 1,
            SkillLevel::Expert => 2,
            SkillLevel::Master => 3,
            SkillLevel::GrandMaster => 4,
        }
    }

    /// Experience needed to advance from this level
    pub const fn advance_threshold(&self) -> u16 {
        match self {
            SkillLevel::Restricted => 0, // cannot advance
            SkillLevel::Unskilled => 20,
            SkillLevel::Basic => 40,
            SkillLevel::Skilled => 80,
            SkillLevel::Expert => 160,
            SkillLevel::Master => 320,
            SkillLevel::GrandMaster => 0, // max level
        }
    }

    /// Advance to next level
    pub fn advance(&self) -> Option<Self> {
        match self {
            SkillLevel::Restricted => None,
            SkillLevel::Unskilled => Some(SkillLevel::Basic),
            SkillLevel::Basic => Some(SkillLevel::Skilled),
            SkillLevel::Skilled => Some(SkillLevel::Expert),
            SkillLevel::Expert => Some(SkillLevel::Master),
            SkillLevel::Master => Some(SkillLevel::GrandMaster),
            SkillLevel::GrandMaster => None,
        }
    }
}

/// Skill types (weapon and spell categories)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum SkillType {
    // Weapon skills
    Dagger = 0,
    Knife = 1,
    Axe = 2,
    PickAxe = 3,
    ShortSword = 4,
    BroadSword = 5,
    LongSword = 6,
    TwoHandedSword = 7,
    Scimitar = 8,
    Saber = 9,
    Club = 10,
    Mace = 11,
    MorningStar = 12,
    Flail = 13,
    Hammer = 14,
    Quarterstaff = 15,
    Polearms = 16,
    Spear = 17,
    Javelin = 18,
    Trident = 19,
    Lance = 20,
    Bow = 21,
    Sling = 22,
    Crossbow = 23,
    Dart = 24,
    Shuriken = 25,
    Boomerang = 26,
    Whip = 27,
    UnicornHorn = 28,

    // Special combat skills
    BareHanded = 29,
    TwoWeapon = 30,
    Riding = 31,

    // Spell skills
    AttackSpells = 32,
    HealingSpells = 33,
    DivinationSpells = 34,
    EnchantmentSpells = 35,
    ClericalSpells = 36,
    EscapeSpells = 37,
    MatterSpells = 38,
}

impl SkillType {
    pub const NUM_SKILLS: usize = 39;

    /// Check if this is a weapon skill
    pub const fn is_weapon(&self) -> bool {
        (*self as u8) <= 28
    }

    /// Check if this is a special combat skill
    pub const fn is_combat(&self) -> bool {
        matches!(
            self,
            SkillType::BareHanded | SkillType::TwoWeapon | SkillType::Riding
        )
    }

    /// Check if this is a spell skill
    pub const fn is_spell(&self) -> bool {
        (*self as u8) >= 32
    }
}

/// Individual skill tracking
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Skill {
    /// Current skill level
    pub level: SkillLevel,
    /// Maximum attainable level (role-dependent)
    pub max_level: SkillLevel,
    /// Practice points toward next level
    pub practice: u16,
}

impl Skill {
    /// Create a new skill with given level limits
    pub const fn new(level: SkillLevel, max_level: SkillLevel) -> Self {
        Self {
            level,
            max_level,
            practice: 0,
        }
    }

    /// Add practice points
    pub fn add_practice(&mut self, points: u16) {
        self.practice = self.practice.saturating_add(points);
    }

    /// Check if skill can be advanced
    pub fn can_advance(&self) -> bool {
        if self.level >= self.max_level {
            return false;
        }
        self.practice >= self.level.advance_threshold()
    }

    /// Advance to next level (returns true if successful)
    pub fn advance(&mut self) -> bool {
        if !self.can_advance() {
            return false;
        }
        if let Some(next) = self.level.advance() {
            if next <= self.max_level {
                self.practice = 0;
                self.level = next;
                return true;
            }
        }
        false
    }
}

/// Complete skill set for a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSet {
    skills: Vec<Skill>,
    /// Number of skill slots available for advancement
    pub slots: i32,
}

impl Default for SkillSet {
    fn default() -> Self {
        Self {
            skills: vec![Skill::default(); SkillType::NUM_SKILLS],
            slots: 0,
        }
    }
}

impl SkillSet {
    /// Get a skill
    pub fn get(&self, skill_type: SkillType) -> &Skill {
        &self.skills[skill_type as usize]
    }

    /// Get a mutable skill
    pub fn get_mut(&mut self, skill_type: SkillType) -> &mut Skill {
        &mut self.skills[skill_type as usize]
    }

    /// Set maximum level for a skill
    pub fn set_max(&mut self, skill_type: SkillType, max_level: SkillLevel) {
        self.skills[skill_type as usize].max_level = max_level;
    }

    /// Count skills that can be advanced
    pub fn advanceable_count(&self) -> usize {
        self.skills.iter().filter(|s| s.can_advance()).count()
    }
}
