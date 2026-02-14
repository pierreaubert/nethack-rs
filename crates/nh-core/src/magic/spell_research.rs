//! Spell research system - Discover and mutate new spell variants
//!
//! Players can research spells to unlock mutations that modify spell behavior
//! in interesting ways, creating customized versions of spells.

use crate::magic::spell::SpellType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of spell mutations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpellMutation {
    /// Damage increased by 50%
    IncreasedDamage,
    /// Range increased by 100%
    IncreasedRange,
    /// Split beam - hits two targets
    SplitBeam,
    /// Projectile automatically targets enemies
    HomingProjectile,
    /// Spell can bounce off walls
    Ricochet,
    /// Effect radius doubled
    ExpandedArea,
    /// Duration doubled
    ExtendedDuration,
    /// Spell costs 50% less mana
    ManaEfficiency,
    /// Spell works even if silenced
    SilentCast,
    /// Spell works even if restrained
    StillCast,
}

impl SpellMutation {
    /// Get name of the mutation
    pub const fn name(&self) -> &'static str {
        match self {
            SpellMutation::IncreasedDamage => "Increased Damage",
            SpellMutation::IncreasedRange => "Increased Range",
            SpellMutation::SplitBeam => "Split Beam",
            SpellMutation::HomingProjectile => "Homing Projectile",
            SpellMutation::Ricochet => "Ricochet",
            SpellMutation::ExpandedArea => "Expanded Area",
            SpellMutation::ExtendedDuration => "Extended Duration",
            SpellMutation::ManaEfficiency => "Mana Efficiency",
            SpellMutation::SilentCast => "Silent Cast",
            SpellMutation::StillCast => "Still Cast",
        }
    }

    /// Get all available mutations
    pub fn all() -> &'static [SpellMutation] {
        &[
            SpellMutation::IncreasedDamage,
            SpellMutation::IncreasedRange,
            SpellMutation::SplitBeam,
            SpellMutation::HomingProjectile,
            SpellMutation::Ricochet,
            SpellMutation::ExpandedArea,
            SpellMutation::ExtendedDuration,
            SpellMutation::ManaEfficiency,
            SpellMutation::SilentCast,
            SpellMutation::StillCast,
        ]
    }
}

/// A researched and potentially mutated spell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchedSpell {
    /// The base spell type
    pub base_spell: SpellType,
    /// Mutations applied
    pub mutations: Vec<SpellMutation>,
    /// Power level (1-10)
    pub power: u8,
    /// Stability (0-100, lower = more unstable)
    pub stability: u8,
    /// Times this variant has been successfully cast
    pub times_cast: u32,
}

impl ResearchedSpell {
    /// Create a new researched spell
    pub fn new(base_spell: SpellType) -> Self {
        Self {
            base_spell,
            mutations: Vec::new(),
            power: 1,
            stability: 100,
            times_cast: 0,
        }
    }

    /// Add a mutation to this spell
    pub fn add_mutation(&mut self, mutation: SpellMutation) {
        if !self.mutations.contains(&mutation) {
            self.mutations.push(mutation);
            // Adding mutations reduces stability and increases power
            self.power = (self.power + 1).min(10);
            self.stability = (self.stability.saturating_sub(10)).max(10);
        }
    }

    /// Check if this spell has a mutation
    pub fn has_mutation(&self, mutation: SpellMutation) -> bool {
        self.mutations.contains(&mutation)
    }

    /// Get mana multiplier from mutations
    pub fn get_mana_multiplier(&self) -> f32 {
        let base = 1.0;
        let mutation_cost: f32 = self
            .mutations
            .iter()
            .map(|m| {
                match m {
                    SpellMutation::IncreasedDamage => 0.2,
                    SpellMutation::IncreasedRange => 0.2,
                    SpellMutation::SplitBeam => 0.5,
                    SpellMutation::HomingProjectile => 0.1,
                    SpellMutation::Ricochet => 0.1,
                    SpellMutation::ExpandedArea => 0.2,
                    SpellMutation::ExtendedDuration => 0.2,
                    SpellMutation::ManaEfficiency => -0.3, // Reduces cost!
                    SpellMutation::SilentCast => 0.1,
                    SpellMutation::StillCast => 0.1,
                }
            })
            .sum();

        (base + mutation_cost).max(0.5)
    }

    /// Get damage multiplier from mutations
    pub fn get_damage_multiplier(&self) -> f32 {
        let mut multiplier = 1.0;
        for mutation in &self.mutations {
            match mutation {
                SpellMutation::IncreasedDamage => multiplier *= 1.5,
                SpellMutation::SplitBeam => multiplier *= 0.7, // Two targets do less damage each
                _ => {}
            }
        }
        multiplier
    }
}

/// Active research project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProject {
    /// Target spell to research
    pub target_spell: SpellType,
    /// Target mutation (if any)
    pub target_mutation: Option<SpellMutation>,
    /// Progress toward discovery (0-100)
    pub progress: u32,
    /// Total progress required
    pub required_progress: u32,
}

impl ResearchProject {
    /// Create new research project
    pub fn new(target_spell: SpellType) -> Self {
        Self {
            target_spell,
            target_mutation: None,
            progress: 0,
            required_progress: 1000,
        }
    }

    /// Create project targeting a specific mutation
    pub fn with_mutation(target_spell: SpellType, mutation: SpellMutation) -> Self {
        Self {
            target_spell,
            target_mutation: Some(mutation),
            progress: 0,
            required_progress: 2000,
        }
    }

    /// Get percentage complete (0-100)
    pub fn percent_complete(&self) -> u32 {
        ((self.progress * 100) / self.required_progress).min(100)
    }

    /// Check if research is complete
    pub fn is_complete(&self) -> bool {
        self.progress >= self.required_progress
    }
}

/// Tracker for all researched spells
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpellResearchTracker {
    /// All researched spell variants
    pub researched_spells: HashMap<SpellType, Vec<ResearchedSpell>>,
    /// Currently active research project
    pub active_project: Option<ResearchProject>,
    /// Total research points earned
    pub total_research_points: u32,
    /// Spells successfully mutated
    pub mutations_discovered: u32,
}

impl SpellResearchTracker {
    /// Create new research tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Get researched variants of a spell
    pub fn get_variants(&self, spell_type: SpellType) -> Option<&Vec<ResearchedSpell>> {
        self.researched_spells.get(&spell_type)
    }

    /// Add a researched spell variant
    pub fn add_variant(&mut self, variant: ResearchedSpell) {
        self.researched_spells
            .entry(variant.base_spell)
            .or_insert_with(Vec::new)
            .push(variant);
    }

    /// Start a new research project
    pub fn start_project(&mut self, project: ResearchProject) {
        self.active_project = Some(project);
    }

    /// Advance active research
    pub fn advance_research(&mut self, mana_invested: u32) {
        if let Some(project) = &mut self.active_project {
            project.progress += mana_invested;
            self.total_research_points += mana_invested;
        }
    }

    /// Check if a project is complete
    pub fn check_project_completion(&mut self) -> Option<ResearchedSpell> {
        if let Some(project) = &self.active_project {
            if project.is_complete() {
                let project = self.active_project.take().unwrap();
                let mut spell = ResearchedSpell::new(project.target_spell);
                if let Some(mutation) = project.target_mutation {
                    spell.add_mutation(mutation);
                    self.mutations_discovered += 1;
                }
                return Some(spell);
            }
        }
        None
    }

    /// Get total mutations discovered
    pub fn total_mutations(&self) -> u32 {
        self.researched_spells
            .values()
            .flat_map(|v| v.iter().map(|s| s.mutations.len() as u32))
            .sum()
    }
}

/// Begin researching a spell
pub fn begin_research(spell_type: SpellType, tracker: &mut SpellResearchTracker) -> String {
    let project = ResearchProject::new(spell_type);
    tracker.start_project(project);
    format!("You begin researching {}...", spell_type.name())
}

/// Begin researching a specific mutation
pub fn begin_mutation_research(
    spell_type: SpellType,
    mutation: SpellMutation,
    tracker: &mut SpellResearchTracker,
) -> String {
    let project = ResearchProject::with_mutation(spell_type, mutation);
    tracker.start_project(project);
    format!(
        "You begin researching {} mutation for {}...",
        mutation.name(),
        spell_type.name()
    )
}

/// Experiment with a spell (risky discovery)
pub fn experiment_with_spell(
    spell_type: SpellType,
    tracker: &mut SpellResearchTracker,
    rng: &mut crate::rng::GameRng,
) -> ExperimentResult {
    // Pick a random mutation
    let mutations = SpellMutation::all();
    if let Some(mutation) = rng.choose(mutations) {
        let mut spell = ResearchedSpell::new(spell_type);
        spell.add_mutation(*mutation);

        // Risk of instability
        if rng.percent(30) {
            spell.stability = rng.rn2(50) as u8;
            return ExperimentResult::Unstable {
                mutation: *mutation,
                spell,
            };
        }

        tracker.add_variant(spell.clone());
        ExperimentResult::Success {
            mutation: *mutation,
            spell,
        }
    } else {
        ExperimentResult::Failed {
            message: "Experiment produced no results.".to_string(),
        }
    }
}

/// Results of spell experimentation
#[derive(Debug, Clone)]
pub enum ExperimentResult {
    /// Successful discovery
    Success {
        mutation: SpellMutation,
        spell: ResearchedSpell,
    },
    /// Discovery but unstable
    Unstable {
        mutation: SpellMutation,
        spell: ResearchedSpell,
    },
    /// Experiment failed
    Failed { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spell_mutation_names() {
        assert_eq!(SpellMutation::IncreasedDamage.name(), "Increased Damage");
        assert_eq!(SpellMutation::SplitBeam.name(), "Split Beam");
    }

    #[test]
    fn test_researched_spell_creation() {
        let spell = ResearchedSpell::new(SpellType::ForceBolt);
        assert_eq!(spell.base_spell, SpellType::ForceBolt);
        assert_eq!(spell.mutations.len(), 0);
        assert_eq!(spell.power, 1);
        assert_eq!(spell.stability, 100);
    }

    #[test]
    fn test_researched_spell_add_mutation() {
        let mut spell = ResearchedSpell::new(SpellType::Fireball);
        spell.add_mutation(SpellMutation::IncreasedDamage);
        assert!(spell.has_mutation(SpellMutation::IncreasedDamage));
        assert!(spell.power > 1);
    }

    #[test]
    fn test_researched_spell_mana_multiplier() {
        let mut spell = ResearchedSpell::new(SpellType::ForceBolt);
        let base_mult = spell.get_mana_multiplier();
        spell.add_mutation(SpellMutation::IncreasedDamage);
        let increased_mult = spell.get_mana_multiplier();
        assert!(increased_mult > base_mult);
    }

    #[test]
    fn test_research_project_creation() {
        let project = ResearchProject::new(SpellType::Fireball);
        assert_eq!(project.target_spell, SpellType::Fireball);
        assert_eq!(project.progress, 0);
        assert!(!project.is_complete());
    }

    #[test]
    fn test_research_project_with_mutation() {
        let project = ResearchProject::with_mutation(SpellType::Fireball, SpellMutation::SplitBeam);
        assert_eq!(project.target_mutation, Some(SpellMutation::SplitBeam));
        assert_eq!(project.required_progress, 2000);
    }

    #[test]
    fn test_research_tracker_creation() {
        let tracker = SpellResearchTracker::new();
        assert_eq!(tracker.total_research_points, 0);
        assert!(tracker.active_project.is_none());
    }

    #[test]
    fn test_research_tracker_add_variant() {
        let mut tracker = SpellResearchTracker::new();
        let spell = ResearchedSpell::new(SpellType::ForceBolt);
        tracker.add_variant(spell);
        assert!(tracker.get_variants(SpellType::ForceBolt).is_some());
    }
}
