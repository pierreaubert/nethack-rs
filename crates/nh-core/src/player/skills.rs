//! Player weapon and spell skills

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Skill level
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
    Display,
    EnumIter,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumIter)]
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

    /// Remove practice points (clamped to 0)
    pub fn remove_practice(&mut self, points: u16) {
        self.practice = self.practice.saturating_sub(points);
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
        if let Some(next) = self.level.advance()
            && next <= self.max_level
        {
            self.practice = 0;
            self.level = next;
            return true;
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

// Utility functions (C equivalents)

/// Get the full name of a skill type (skill_name equivalent)
pub fn skill_name(skill: SkillType) -> &'static str {
    match skill {
        SkillType::Dagger => "dagger",
        SkillType::Knife => "knife",
        SkillType::Axe => "axe",
        SkillType::PickAxe => "pick-axe",
        SkillType::ShortSword => "short sword",
        SkillType::BroadSword => "broad sword",
        SkillType::LongSword => "long sword",
        SkillType::TwoHandedSword => "two-handed sword",
        SkillType::Scimitar => "scimitar",
        SkillType::Saber => "saber",
        SkillType::Club => "club",
        SkillType::Mace => "mace",
        SkillType::MorningStar => "morning star",
        SkillType::Flail => "flail",
        SkillType::Hammer => "hammer",
        SkillType::Quarterstaff => "quarterstaff",
        SkillType::Polearms => "polearms",
        SkillType::Spear => "spear",
        SkillType::Javelin => "javelin",
        SkillType::Trident => "trident",
        SkillType::Lance => "lance",
        SkillType::Bow => "bow",
        SkillType::Sling => "sling",
        SkillType::Crossbow => "crossbow",
        SkillType::Dart => "dart",
        SkillType::Shuriken => "shuriken",
        SkillType::Boomerang => "boomerang",
        SkillType::Whip => "whip",
        SkillType::UnicornHorn => "unicorn horn",
        SkillType::BareHanded => "bare handed combat",
        SkillType::TwoWeapon => "two weapon combat",
        SkillType::Riding => "riding",
        SkillType::AttackSpells => "attack spells",
        SkillType::HealingSpells => "healing spells",
        SkillType::DivinationSpells => "divination spells",
        SkillType::EnchantmentSpells => "enchantment spells",
        SkillType::ClericalSpells => "clerical spells",
        SkillType::EscapeSpells => "escape spells",
        SkillType::MatterSpells => "matter spells",
    }
}

/// Get the display name for a skill level (skill_level_name equivalent)
pub fn skill_level_name(level: SkillLevel) -> &'static str {
    match level {
        SkillLevel::Restricted => "Restricted",
        SkillLevel::Unskilled => "Unskilled",
        SkillLevel::Basic => "Basic",
        SkillLevel::Skilled => "Skilled",
        SkillLevel::Expert => "Expert",
        SkillLevel::Master => "Master",
        SkillLevel::GrandMaster => "Grand Master",
    }
}

/// Initialize player skills from role defaults (skill_init equivalent)
///
/// Sets up the skill set based on the player's role and race.
/// This would typically be called at character creation.
pub fn skill_init(skill_set: &mut SkillSet, role: &str) {
    // Role-specific skill initializations
    // This is a simplified version - the full implementation would be in tables
    match role.to_lowercase().as_str() {
        "barbarian" => {
            skill_set.set_max(SkillType::BareHanded, SkillLevel::Master);
            skill_set.set_max(SkillType::TwoWeapon, SkillLevel::Skilled);
            skill_set.set_max(SkillType::BroadSword, SkillLevel::Expert);
        }
        "warrior" => {
            skill_set.set_max(SkillType::BareHanded, SkillLevel::Expert);
            skill_set.set_max(SkillType::BroadSword, SkillLevel::Expert);
            skill_set.set_max(SkillType::Axe, SkillLevel::Skilled);
        }
        "monk" => {
            skill_set.set_max(SkillType::BareHanded, SkillLevel::GrandMaster);
            skill_set.set_max(SkillType::Quarterstaff, SkillLevel::Skilled);
        }
        "wizard" => {
            skill_set.set_max(SkillType::AttackSpells, SkillLevel::Expert);
            skill_set.set_max(SkillType::EnchantmentSpells, SkillLevel::Expert);
        }
        _ => {}
    }
}

/// Add a weapon skill to the list of available skills (add_weapon_skill equivalent)
///
/// Increases the number of skills available for advancement.
pub fn add_weapon_skill(skill_set: &mut SkillSet, count: i32) {
    skill_set.slots = skill_set.slots.saturating_add(count);
}

/// Remove a weapon skill from available advancement (lose_weapon_skill equivalent)
pub fn lose_weapon_skill(skill_set: &mut SkillSet, count: i32) {
    skill_set.slots = skill_set.slots.saturating_sub(count);
}

/// Unrestrict a weapon skill (unrestrict_weapon_skill equivalent)
///
/// Marks a previously restricted skill as available for use.
pub fn unrestrict_weapon_skill(skill_set: &mut SkillSet, skill_type: SkillType) {
    let skill = skill_set.get_mut(skill_type);
    if skill.level == SkillLevel::Restricted {
        skill.level = SkillLevel::Unskilled;
    }
}

/// Use a skill and record practice (use_skill equivalent)
///
/// Adds practice points to a skill based on the degree of success.
pub fn use_skill(skill_set: &mut SkillSet, skill_type: SkillType, degree: i32) {
    let skill = skill_set.get_mut(skill_type);
    let practice_points = (degree.abs() as u16).max(1);
    skill.add_practice(practice_points);
}

/// Check if a skill has peaked (peaked_skill equivalent)
///
/// Returns true if the skill is at max level and has enough practice
/// to have advanced further if possible.
pub fn peaked_skill(skill_set: &SkillSet, skill_type: SkillType) -> bool {
    let skill = skill_set.get(skill_type);
    if skill.level == SkillLevel::Restricted {
        return false;
    }
    skill.level >= skill.max_level && skill.practice >= skill.level.advance_threshold()
}

/// Enhance weapon skill interactively (enhance_weapon_skill equivalent - stub)
///
/// In the original game, this presents a menu for the player to choose
/// which skill to advance. This is a simplified version that advances
/// the first available skill.
pub fn enhance_weapon_skill(skill_set: &mut SkillSet) -> bool {
    // Find the first skill that can be advanced
    for skill_type in [
        SkillType::BareHanded,
        SkillType::Dagger,
        SkillType::Knife,
        SkillType::Axe,
        SkillType::ShortSword,
        SkillType::BroadSword,
        SkillType::LongSword,
    ] {
        let skill = skill_set.get_mut(skill_type);
        if skill.can_advance() {
            return skill.advance();
        }
    }
    false
}

/// Get spell skill type from spellbook object type (spell_skilltype equivalent)
///
/// Maps a spellbook object type to its corresponding skill type.
pub fn spell_skilltype(booktype: u16) -> Option<SkillType> {
    // This would be based on object table lookups in the real implementation
    match booktype {
        1..=10 => Some(SkillType::AttackSpells),
        11..=20 => Some(SkillType::HealingSpells),
        21..=30 => Some(SkillType::DivinationSpells),
        _ => None,
    }
}

/// Get weapon description (weapon_descr equivalent)
///
/// Returns a descriptive name for a weapon based on its type.
/// Used in messages and UI.
pub fn weapon_descr(weapon_type: SkillType) -> &'static str {
    match weapon_type {
        SkillType::BareHanded => "fists",
        SkillType::Dagger => "dagger",
        SkillType::Knife => "knife",
        SkillType::Axe => "axe",
        SkillType::PickAxe => "pick-axe",
        SkillType::ShortSword => "short sword",
        SkillType::BroadSword => "broad sword",
        SkillType::LongSword => "long sword",
        SkillType::TwoHandedSword => "two-handed sword",
        SkillType::Scimitar => "scimitar",
        SkillType::Saber => "saber",
        _ => skill_name(weapon_type),
    }
}
