//! Spell mastery advancement system
//!
//! Tracks player progress through spell mastery levels, handles advancement
//! mechanics, unlocks special abilities, and manages mastery rewards.

use crate::magic::spell::{SpellMastery, SpellType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mastery advancement milestone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MasteryMilestone {
    ReachedNovice,   // First spell cast successfully
    ReachedAdept,    // 50 spells cast
    ReachedExpert,   // 200 spells cast
    ReachedMaster,   // 500 spells cast
    ReachedArchmage, // 1000 spells cast
}

impl MasteryMilestone {
    /// Get requirement for this milestone
    pub fn requirement(&self) -> i32 {
        match self {
            MasteryMilestone::ReachedNovice => 1,
            MasteryMilestone::ReachedAdept => 50,
            MasteryMilestone::ReachedExpert => 200,
            MasteryMilestone::ReachedMaster => 500,
            MasteryMilestone::ReachedArchmage => 1000,
        }
    }

    /// Get message for milestone achievement
    pub fn message(&self) -> &'static str {
        match self {
            MasteryMilestone::ReachedNovice => "You have reached Novice mastery!",
            MasteryMilestone::ReachedAdept => "You have reached Adept mastery!",
            MasteryMilestone::ReachedExpert => "You have reached Expert mastery!",
            MasteryMilestone::ReachedMaster => "You have reached Master mastery!",
            MasteryMilestone::ReachedArchmage => "You have reached Archmage mastery!",
        }
    }

    /// Get bonus at this level
    pub fn bonus(&self) -> i32 {
        match self {
            MasteryMilestone::ReachedNovice => 5,
            MasteryMilestone::ReachedAdept => 10,
            MasteryMilestone::ReachedExpert => 20,
            MasteryMilestone::ReachedMaster => 30,
            MasteryMilestone::ReachedArchmage => 50,
        }
    }
}

/// Track mastery advancement for a specific spell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellMasteryProgress {
    pub spell_type: SpellType,
    pub mastery: SpellMastery,
    pub times_cast: i32,
    pub times_succeeded: i32,
    pub times_failed: i32,
    pub critical_hits: i32, // Spell exceeded expected effect
    pub milestones_reached: Vec<MasteryMilestone>,
}

impl SpellMasteryProgress {
    pub fn new(spell_type: SpellType) -> Self {
        Self {
            spell_type,
            mastery: SpellMastery::Unknown,
            times_cast: 0,
            times_succeeded: 0,
            times_failed: 0,
            critical_hits: 0,
            milestones_reached: Vec::new(),
        }
    }

    /// Record a spell cast
    pub fn cast_spell(&mut self, succeeded: bool) {
        self.times_cast += 1;
        if succeeded {
            self.times_succeeded += 1;
        } else {
            self.times_failed += 1;
        }

        // Check for mastery advancement
        self.check_advancement();
    }

    /// Record a critical hit
    pub fn critical_hit(&mut self) {
        self.critical_hits += 1;
    }

    /// Get success rate (0-100)
    pub fn success_rate(&self) -> i32 {
        if self.times_cast == 0 {
            0
        } else {
            ((self.times_succeeded as f32 / self.times_cast as f32) * 100.0) as i32
        }
    }

    /// Check if should advance mastery
    fn check_advancement(&mut self) {
        // Advance when mastery requirements met
        let next_level = match self.mastery {
            SpellMastery::Unknown => {
                if self.times_cast >= 5 {
                    SpellMastery::Novice
                } else {
                    self.mastery
                }
            }
            SpellMastery::Novice => {
                if self.times_cast >= 50 {
                    SpellMastery::Adept
                } else {
                    self.mastery
                }
            }
            SpellMastery::Adept => {
                if self.times_cast >= 200 {
                    SpellMastery::Expert
                } else {
                    self.mastery
                }
            }
            SpellMastery::Expert => {
                if self.times_cast >= 500 {
                    SpellMastery::Master
                } else {
                    self.mastery
                }
            }
            SpellMastery::Master => self.mastery, // Already at max
        };

        if next_level != self.mastery {
            self.mastery = next_level;
        }
    }

    /// Get mastery description
    pub fn mastery_description(&self) -> String {
        format!(
            "{} ({}/{} casts, {:.0}% success rate)",
            self.mastery.name(),
            self.times_succeeded,
            self.times_cast,
            self.success_rate()
        )
    }
}

/// Track all spell mastery progress
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MasteryAdvancementTracker {
    pub spells: HashMap<SpellType, SpellMasteryProgress>,
    pub total_spells_cast: i32,
    pub total_successes: i32,
}

impl MasteryAdvancementTracker {
    pub fn new() -> Self {
        Self {
            spells: HashMap::new(),
            total_spells_cast: 0,
            total_successes: 0,
        }
    }

    /// Record spell cast
    pub fn record_spell_cast(&mut self, spell_type: SpellType, succeeded: bool) {
        self.spells
            .entry(spell_type)
            .or_insert_with(|| SpellMasteryProgress::new(spell_type))
            .cast_spell(succeeded);

        self.total_spells_cast += 1;
        if succeeded {
            self.total_successes += 1;
        }
    }

    /// Record critical hit
    pub fn record_critical_hit(&mut self, spell_type: SpellType) {
        if let Some(progress) = self.spells.get_mut(&spell_type) {
            progress.critical_hit();
        }
    }

    /// Get mastery for spell
    pub fn get_mastery(&self, spell_type: SpellType) -> SpellMastery {
        self.spells
            .get(&spell_type)
            .map(|p| p.mastery)
            .unwrap_or(SpellMastery::Unknown)
    }

    /// Get overall success rate
    pub fn overall_success_rate(&self) -> i32 {
        if self.total_spells_cast == 0 {
            0
        } else {
            ((self.total_successes as f32 / self.total_spells_cast as f32) * 100.0) as i32
        }
    }

    /// Get highest mastery level across all spells
    pub fn highest_mastery(&self) -> SpellMastery {
        self.spells
            .values()
            .map(|p| p.mastery)
            .max()
            .unwrap_or(SpellMastery::Unknown)
    }

    /// Get count of spells at specific mastery
    pub fn spells_at_mastery(&self, mastery: SpellMastery) -> usize {
        self.spells
            .values()
            .filter(|p| p.mastery == mastery)
            .count()
    }

    /// Get spells sorted by times cast
    pub fn spells_by_usage(&self) -> Vec<(SpellType, i32)> {
        let mut spells: Vec<_> = self
            .spells
            .iter()
            .map(|(spell_type, progress)| (*spell_type, progress.times_cast))
            .collect();

        spells.sort_by(|a, b| b.1.cmp(&a.1)); // Descending by usage
        spells
    }

    /// Get milestone message if achieved
    pub fn check_milestone(&mut self, spell_type: SpellType) -> Option<(MasteryMilestone, String)> {
        if let Some(progress) = self.spells.get_mut(&spell_type) {
            let next_milestone = match progress.times_cast {
                1 => Some(MasteryMilestone::ReachedNovice),
                50 => Some(MasteryMilestone::ReachedAdept),
                200 => Some(MasteryMilestone::ReachedExpert),
                500 => Some(MasteryMilestone::ReachedMaster),
                1000 => Some(MasteryMilestone::ReachedArchmage),
                _ => None,
            };

            if let Some(milestone) = next_milestone {
                if !progress.milestones_reached.contains(&milestone) {
                    progress.milestones_reached.push(milestone);
                    return Some((milestone, milestone.message().to_string()));
                }
            }
        }

        None
    }
}

/// Get spell damage bonus from mastery
pub fn get_mastery_damage_bonus(mastery: SpellMastery) -> f32 {
    match mastery {
        SpellMastery::Unknown => 0.5, // Can't cast reliably
        SpellMastery::Novice => 1.0,
        SpellMastery::Adept => 1.2,
        SpellMastery::Expert => 1.4,
        SpellMastery::Master => 1.6,
    }
}

/// Get mana efficiency bonus from mastery
pub fn get_mastery_mana_efficiency(mastery: SpellMastery) -> f32 {
    match mastery {
        SpellMastery::Unknown => 2.0, // Double cost
        SpellMastery::Novice => 1.5,
        SpellMastery::Adept => 1.0,
        SpellMastery::Expert => 0.8,
        SpellMastery::Master => 0.6,
    }
}

/// Check if spell is ready for major advancement
pub fn is_ready_for_advancement(progress: &SpellMasteryProgress) -> bool {
    match progress.mastery {
        SpellMastery::Unknown => progress.times_cast >= 5 && progress.success_rate() >= 50,
        SpellMastery::Novice => progress.times_cast >= 50 && progress.success_rate() >= 60,
        SpellMastery::Adept => progress.times_cast >= 200 && progress.success_rate() >= 70,
        SpellMastery::Expert => progress.times_cast >= 500 && progress.success_rate() >= 80,
        SpellMastery::Master => false, // Can't advance further
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mastery_milestone_requirement() {
        assert_eq!(MasteryMilestone::ReachedNovice.requirement(), 1);
        assert_eq!(MasteryMilestone::ReachedAdept.requirement(), 50);
        assert_eq!(MasteryMilestone::ReachedArchmage.requirement(), 1000);
    }

    #[test]
    fn test_mastery_milestone_message() {
        assert!(!MasteryMilestone::ReachedNovice.message().is_empty());
        assert!(!MasteryMilestone::ReachedMaster.message().is_empty());
    }

    #[test]
    fn test_mastery_milestone_bonus() {
        assert!(MasteryMilestone::ReachedNovice.bonus() < MasteryMilestone::ReachedMaster.bonus());
    }

    #[test]
    fn test_spell_mastery_progress_new() {
        let progress = SpellMasteryProgress::new(SpellType::ForceBolt);
        assert_eq!(progress.spell_type, SpellType::ForceBolt);
        assert_eq!(progress.mastery, SpellMastery::Unknown);
    }

    #[test]
    fn test_spell_mastery_progress_cast_spell() {
        let mut progress = SpellMasteryProgress::new(SpellType::ForceBolt);
        progress.cast_spell(true);
        assert_eq!(progress.times_cast, 1);
        assert_eq!(progress.times_succeeded, 1);
    }

    #[test]
    fn test_spell_mastery_progress_success_rate() {
        let mut progress = SpellMasteryProgress::new(SpellType::ForceBolt);
        progress.cast_spell(true);
        progress.cast_spell(true);
        progress.cast_spell(false);
        assert_eq!(progress.success_rate(), 66);
    }

    #[test]
    fn test_spell_mastery_progress_advancement() {
        let mut progress = SpellMasteryProgress::new(SpellType::ForceBolt);
        for _ in 0..5 {
            progress.cast_spell(true);
        }
        assert_eq!(progress.mastery, SpellMastery::Novice);
    }

    #[test]
    fn test_spell_mastery_progress_critical_hit() {
        let mut progress = SpellMasteryProgress::new(SpellType::ForceBolt);
        progress.critical_hit();
        assert_eq!(progress.critical_hits, 1);
    }

    #[test]
    fn test_mastery_advancement_tracker_new() {
        let tracker = MasteryAdvancementTracker::new();
        assert_eq!(tracker.total_spells_cast, 0);
    }

    #[test]
    fn test_mastery_advancement_tracker_record_spell() {
        let mut tracker = MasteryAdvancementTracker::new();
        tracker.record_spell_cast(SpellType::ForceBolt, true);
        assert_eq!(tracker.total_spells_cast, 1);
        assert_eq!(tracker.total_successes, 1);
    }

    #[test]
    fn test_mastery_advancement_tracker_overall_success() {
        let mut tracker = MasteryAdvancementTracker::new();
        tracker.record_spell_cast(SpellType::ForceBolt, true);
        tracker.record_spell_cast(SpellType::ForceBolt, false);
        assert_eq!(tracker.overall_success_rate(), 50);
    }

    #[test]
    fn test_get_mastery_damage_bonus() {
        assert_eq!(get_mastery_damage_bonus(SpellMastery::Unknown), 0.5);
        assert_eq!(get_mastery_damage_bonus(SpellMastery::Novice), 1.0);
        assert_eq!(get_mastery_damage_bonus(SpellMastery::Master), 1.6);
    }

    #[test]
    fn test_get_mastery_mana_efficiency() {
        assert_eq!(get_mastery_mana_efficiency(SpellMastery::Unknown), 2.0);
        assert_eq!(get_mastery_mana_efficiency(SpellMastery::Adept), 1.0);
        assert_eq!(get_mastery_mana_efficiency(SpellMastery::Master), 0.6);
    }

    #[test]
    fn test_is_ready_for_advancement() {
        // Construct a progress manually to test is_ready_for_advancement
        // without auto-advancement interfering.
        let mut progress = SpellMasteryProgress::new(SpellType::ForceBolt);
        // Manually set to Novice with 50 casts, 100% success rate
        progress.mastery = SpellMastery::Novice;
        progress.times_cast = 50;
        progress.times_succeeded = 50;

        // Novice with times_cast >= 50 and success_rate >= 60 => ready
        assert!(is_ready_for_advancement(&progress));

        // Also test that Unknown with not enough casts is NOT ready
        let fresh = SpellMasteryProgress::new(SpellType::ForceBolt);
        assert!(!is_ready_for_advancement(&fresh));
    }

    #[test]
    fn test_mastery_advancement_tracker_highest_mastery() {
        let mut tracker = MasteryAdvancementTracker::new();
        tracker.record_spell_cast(SpellType::ForceBolt, true);
        for _ in 0..4 {
            tracker.record_spell_cast(SpellType::ForceBolt, true);
        }

        assert_eq!(tracker.highest_mastery(), SpellMastery::Novice);
    }

    #[test]
    fn test_mastery_advancement_tracker_check_milestone() {
        let mut tracker = MasteryAdvancementTracker::new();
        // Milestones trigger at exactly 1, 50, 200, 500, 1000 casts.
        // Cast once to reach the ReachedNovice milestone (times_cast == 1).
        tracker.record_spell_cast(SpellType::ForceBolt, true);

        let milestone = tracker.check_milestone(SpellType::ForceBolt);
        assert!(milestone.is_some());
        assert_eq!(milestone.unwrap().0, MasteryMilestone::ReachedNovice);
    }

    #[test]
    fn test_spell_mastery_progress_mastery_description() {
        let progress = SpellMasteryProgress::new(SpellType::ForceBolt);
        let desc = progress.mastery_description();
        assert!(desc.contains("Unknown"));
    }
}
