//! School specialization system
//!
//! Allows players to specialize in specific spell schools, gaining bonuses
//! and unlocking special abilities. Specialization grows through practice.

use crate::magic::spell::SpellSchool;
use serde::{Deserialize, Serialize};
use hashbrown::HashMap;

/// Specialization level in a school
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SpecializationLevel {
    None,       // Not specialized
    Novice,     // 20% bonus
    Adept,      // 40% bonus
    Specialist, // 60% bonus
    Master,     // 80% bonus
    Archmage,   // 100% bonus + special ability
}

impl SpecializationLevel {
    /// Get damage/effect bonus multiplier
    pub fn bonus_multiplier(&self) -> f32 {
        match self {
            SpecializationLevel::None => 1.0,
            SpecializationLevel::Novice => 1.2,
            SpecializationLevel::Adept => 1.4,
            SpecializationLevel::Specialist => 1.6,
            SpecializationLevel::Master => 1.8,
            SpecializationLevel::Archmage => 2.0,
        }
    }

    /// Get mana cost reduction (percentage)
    pub fn mana_reduction(&self) -> i32 {
        match self {
            SpecializationLevel::None => 0,
            SpecializationLevel::Novice => 5,
            SpecializationLevel::Adept => 10,
            SpecializationLevel::Specialist => 15,
            SpecializationLevel::Master => 20,
            SpecializationLevel::Archmage => 25,
        }
    }

    /// Get failure chance reduction (percentage)
    pub fn failure_reduction(&self) -> i32 {
        match self {
            SpecializationLevel::None => 0,
            SpecializationLevel::Novice => 5,
            SpecializationLevel::Adept => 10,
            SpecializationLevel::Specialist => 15,
            SpecializationLevel::Master => 20,
            SpecializationLevel::Archmage => 30,
        }
    }

    /// Check if has special ability
    pub fn has_special_ability(&self) -> bool {
        matches!(self, SpecializationLevel::Archmage)
    }

    /// Advance to next level
    pub fn advance(&self) -> SpecializationLevel {
        match self {
            SpecializationLevel::None => SpecializationLevel::Novice,
            SpecializationLevel::Novice => SpecializationLevel::Adept,
            SpecializationLevel::Adept => SpecializationLevel::Specialist,
            SpecializationLevel::Specialist => SpecializationLevel::Master,
            SpecializationLevel::Master => SpecializationLevel::Archmage,
            SpecializationLevel::Archmage => SpecializationLevel::Archmage,
        }
    }

    /// Get name
    pub fn name(&self) -> &'static str {
        match self {
            SpecializationLevel::None => "unspecialized",
            SpecializationLevel::Novice => "novice",
            SpecializationLevel::Adept => "adept",
            SpecializationLevel::Specialist => "specialist",
            SpecializationLevel::Master => "master",
            SpecializationLevel::Archmage => "archmage",
        }
    }
}

/// School specialization including progress toward next level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchoolSpecialization {
    pub school: SpellSchool,
    pub level: SpecializationLevel,
    pub experience: i32,  // XP toward next level
    pub spells_cast: i32, // Total spells cast in this school
}

impl SchoolSpecialization {
    pub fn new(school: SpellSchool) -> Self {
        Self {
            school,
            level: SpecializationLevel::None,
            experience: 0,
            spells_cast: 0,
        }
    }

    /// Add experience (from casting spells)
    pub fn add_experience(&mut self, amount: i32) {
        self.experience += amount;

        // Each level requires progressively more XP
        let xp_needed = match self.level {
            SpecializationLevel::None => 100,
            SpecializationLevel::Novice => 200,
            SpecializationLevel::Adept => 400,
            SpecializationLevel::Specialist => 800,
            SpecializationLevel::Master => 1600,
            SpecializationLevel::Archmage => i32::MAX, // Can't advance further
        };

        if self.experience >= xp_needed {
            self.experience -= xp_needed;
            self.level = self.level.advance();
        }
    }

    /// Record spell cast
    pub fn spell_cast(&mut self) {
        self.spells_cast += 1;
        // Gain 10 XP per spell cast
        self.add_experience(10);
    }

    /// Get progress to next level (0-100)
    pub fn progress_to_next(&self) -> i32 {
        let xp_needed = match self.level {
            SpecializationLevel::None => 100,
            SpecializationLevel::Novice => 200,
            SpecializationLevel::Adept => 400,
            SpecializationLevel::Specialist => 800,
            SpecializationLevel::Master => 1600,
            SpecializationLevel::Archmage => i32::MAX,
        };

        if xp_needed == i32::MAX {
            100
        } else {
            ((self.experience as f32 / xp_needed as f32) * 100.0) as i32
        }
    }
}

/// Track specialization in all schools
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecializationTracker {
    pub schools: HashMap<SpellSchool, SchoolSpecialization>,
}

impl SpecializationTracker {
    pub fn new() -> Self {
        Self {
            schools: HashMap::new(),
        }
    }

    /// Initialize all schools
    pub fn initialize_all_schools(&mut self) {
        for school in SpellSchool::all() {
            self.schools
                .insert(*school, SchoolSpecialization::new(*school));
        }
    }

    /// Get specialization for a school
    pub fn get(&self, school: SpellSchool) -> Option<&SchoolSpecialization> {
        self.schools.get(&school)
    }

    /// Get mutable specialization for a school
    pub fn get_mut(&mut self, school: SpellSchool) -> Option<&mut SchoolSpecialization> {
        self.schools.get_mut(&school)
    }

    /// Record spell cast in a school
    pub fn spell_cast_in_school(&mut self, school: SpellSchool) {
        self.schools
            .entry(school)
            .or_insert_with(|| SchoolSpecialization::new(school))
            .spell_cast();
    }

    /// Get total spells cast across all schools
    pub fn total_spells_cast(&self) -> i32 {
        self.schools.values().map(|s| s.spells_cast).sum()
    }

    /// Get highest specialization level
    pub fn highest_level(&self) -> SpecializationLevel {
        self.schools
            .values()
            .map(|s| s.level)
            .max()
            .unwrap_or(SpecializationLevel::None)
    }

    /// Get count of schools at specific level
    pub fn count_at_level(&self, level: SpecializationLevel) -> usize {
        self.schools.values().filter(|s| s.level == level).count()
    }

    /// Check if specialized (at least Novice)
    pub fn is_specialized(&self) -> bool {
        self.schools
            .values()
            .any(|s| s.level != SpecializationLevel::None)
    }

    /// Get all schools sorted by level
    pub fn schools_by_level(&self) -> Vec<(SpellSchool, SpecializationLevel)> {
        let mut schools: Vec<_> = self
            .schools
            .iter()
            .map(|(school, spec)| (*school, spec.level))
            .collect();

        schools.sort_by(|a, b| b.1.cmp(&a.1)); // Descending by level
        schools
    }
}

/// Get spell damage bonus from specialization
pub fn get_specialization_damage_bonus(
    tracker: &SpecializationTracker,
    school: SpellSchool,
) -> f32 {
    tracker
        .get(school)
        .map(|spec| spec.level.bonus_multiplier())
        .unwrap_or(1.0)
}

/// Get mana cost with specialization reduction
pub fn calculate_specialization_mana_cost(
    tracker: &SpecializationTracker,
    base_cost: i32,
    school: SpellSchool,
) -> i32 {
    let reduction_percent = tracker
        .get(school)
        .map(|spec| spec.level.mana_reduction())
        .unwrap_or(0);

    let reduction = (reduction_percent as f32 / 100.0) * base_cost as f32;
    (base_cost as f32 - reduction).max(base_cost as f32 / 2.0) as i32
}

/// Get failure chance reduction from specialization
pub fn get_specialization_failure_reduction(
    tracker: &SpecializationTracker,
    school: SpellSchool,
) -> i32 {
    tracker
        .get(school)
        .map(|spec| spec.level.failure_reduction())
        .unwrap_or(0)
}

/// Check if school ability is unlocked (requires Archmage)
pub fn can_use_school_ability(tracker: &SpecializationTracker, school: SpellSchool) -> bool {
    tracker
        .get(school)
        .map(|spec| spec.level.has_special_ability())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specialization_level_bonus_multiplier() {
        assert_eq!(SpecializationLevel::None.bonus_multiplier(), 1.0);
        assert_eq!(SpecializationLevel::Novice.bonus_multiplier(), 1.2);
        assert_eq!(SpecializationLevel::Archmage.bonus_multiplier(), 2.0);
    }

    #[test]
    fn test_specialization_level_mana_reduction() {
        assert_eq!(SpecializationLevel::None.mana_reduction(), 0);
        assert_eq!(SpecializationLevel::Novice.mana_reduction(), 5);
        assert_eq!(SpecializationLevel::Archmage.mana_reduction(), 25);
    }

    #[test]
    fn test_specialization_level_failure_reduction() {
        assert_eq!(SpecializationLevel::None.failure_reduction(), 0);
        assert_eq!(SpecializationLevel::Master.failure_reduction(), 20);
    }

    #[test]
    fn test_specialization_level_advance() {
        let level = SpecializationLevel::Novice;
        assert_eq!(level.advance(), SpecializationLevel::Adept);
    }

    #[test]
    fn test_school_specialization_new() {
        let spec = SchoolSpecialization::new(SpellSchool::Attack);
        assert_eq!(spec.school, SpellSchool::Attack);
        assert_eq!(spec.level, SpecializationLevel::None);
    }

    #[test]
    fn test_school_specialization_spell_cast() {
        let mut spec = SchoolSpecialization::new(SpellSchool::Attack);
        spec.spell_cast();
        assert_eq!(spec.spells_cast, 1);
        assert!(spec.experience > 0);
    }

    #[test]
    fn test_school_specialization_add_experience() {
        let mut spec = SchoolSpecialization::new(SpellSchool::Attack);
        spec.add_experience(50);
        assert_eq!(spec.experience, 50);
    }

    #[test]
    fn test_school_specialization_level_up() {
        let mut spec = SchoolSpecialization::new(SpellSchool::Attack);
        spec.add_experience(100); // Should level up to Novice
        assert_eq!(spec.level, SpecializationLevel::Novice);
    }

    #[test]
    fn test_school_specialization_progress_to_next() {
        let mut spec = SchoolSpecialization::new(SpellSchool::Attack);
        spec.add_experience(50);
        let progress = spec.progress_to_next();
        assert!(progress > 0);
        assert!(progress < 100);
    }

    #[test]
    fn test_specialization_tracker_new() {
        let tracker = SpecializationTracker::new();
        assert!(tracker.schools.is_empty());
    }

    #[test]
    fn test_specialization_tracker_initialize() {
        let mut tracker = SpecializationTracker::new();
        tracker.initialize_all_schools();
        assert_eq!(tracker.schools.len(), SpellSchool::all().len());
    }

    #[test]
    fn test_specialization_tracker_spell_cast() {
        let mut tracker = SpecializationTracker::new();
        tracker.spell_cast_in_school(SpellSchool::Attack);
        assert!(tracker.get(SpellSchool::Attack).is_some());
    }

    #[test]
    fn test_specialization_tracker_total_spells() {
        let mut tracker = SpecializationTracker::new();
        tracker.spell_cast_in_school(SpellSchool::Attack);
        tracker.spell_cast_in_school(SpellSchool::Attack);
        tracker.spell_cast_in_school(SpellSchool::Healing);
        assert_eq!(tracker.total_spells_cast(), 3);
    }

    #[test]
    fn test_specialization_tracker_highest_level() {
        let mut tracker = SpecializationTracker::new();
        tracker.initialize_all_schools();
        assert_eq!(tracker.highest_level(), SpecializationLevel::None);
    }

    #[test]
    fn test_get_specialization_damage_bonus() {
        let mut tracker = SpecializationTracker::new();
        tracker.spell_cast_in_school(SpellSchool::Attack);

        // Add enough experience to level up
        if let Some(spec) = tracker.get_mut(SpellSchool::Attack) {
            spec.add_experience(100);
        }

        let bonus = get_specialization_damage_bonus(&tracker, SpellSchool::Attack);
        assert!(bonus >= 1.2); // At least Novice level
    }

    #[test]
    fn test_calculate_specialization_mana_cost() {
        let mut tracker = SpecializationTracker::new();
        tracker.spell_cast_in_school(SpellSchool::Attack);
        if let Some(spec) = tracker.get_mut(SpellSchool::Attack) {
            spec.add_experience(100);
        }

        let cost = calculate_specialization_mana_cost(&tracker, 100, SpellSchool::Attack);
        assert!(cost < 100);
    }

    #[test]
    fn test_can_use_school_ability() {
        let mut tracker = SpecializationTracker::new();
        tracker.spell_cast_in_school(SpellSchool::Attack);

        // At Novice level, should not be able to use ability
        assert!(!can_use_school_ability(&tracker, SpellSchool::Attack));
    }

    #[test]
    fn test_specialization_tracker_is_specialized() {
        let mut tracker = SpecializationTracker::new();
        assert!(!tracker.is_specialized());

        tracker.spell_cast_in_school(SpellSchool::Attack);
        if let Some(spec) = tracker.get_mut(SpellSchool::Attack) {
            spec.add_experience(100); // Level to Novice
        }

        assert!(tracker.is_specialized());
    }

    #[test]
    fn test_specialization_level_name() {
        assert_eq!(SpecializationLevel::None.name(), "unspecialized");
        assert_eq!(SpecializationLevel::Archmage.name(), "archmage");
    }
}
