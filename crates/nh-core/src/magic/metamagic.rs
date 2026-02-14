//! Metamagic system - Modify spells on-the-fly
//!
//! Allows players to modify spell behavior by spending additional mana or energy.
//! Metamagic effects include quickening, empowering, maximizing, and various other modifications.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of metamagic modifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetamagicType {
    /// Cast the spell as a bonus action instead of standard action
    Quicken,
    /// Increase spell effectiveness (damage/healing by 1.5x)
    Empower,
    /// Maximize random effects (no dice rolls, use maximum values)
    Maximize,
    /// Cast spell without verbal components (even if silenced)
    Silent,
    /// Cast spell without somatic components (even if restrained)
    Still,
    /// Double spell duration
    Extend,
    /// Increase spell area/radius by 50%
    Widen,
    /// Cast spell on two targets instead of one
    Twin,
    /// Add defense/resistance roll to your save DC
    Careful,
    /// Hide spell effects from detection (harder to identify)
    Subtle,
}

impl MetamagicType {
    /// Get human-readable name
    pub const fn name(&self) -> &'static str {
        match self {
            MetamagicType::Quicken => "Quicken",
            MetamagicType::Empower => "Empower",
            MetamagicType::Maximize => "Maximize",
            MetamagicType::Silent => "Silent",
            MetamagicType::Still => "Still",
            MetamagicType::Extend => "Extend",
            MetamagicType::Widen => "Widen",
            MetamagicType::Twin => "Twin",
            MetamagicType::Careful => "Careful",
            MetamagicType::Subtle => "Subtle",
        }
    }

    /// Get base mana cost for applying this metamagic
    pub const fn base_cost(&self) -> i32 {
        match self {
            MetamagicType::Quicken => 50, // Most expensive
            MetamagicType::Empower => 20,
            MetamagicType::Maximize => 30,
            MetamagicType::Silent => 15,
            MetamagicType::Still => 15,
            MetamagicType::Extend => 25,
            MetamagicType::Widen => 25,
            MetamagicType::Twin => 40, // Expensive (double effect)
            MetamagicType::Careful => 20,
            MetamagicType::Subtle => 10, // Cheapest
        }
    }

    /// Get all metamagic types
    pub fn all() -> &'static [MetamagicType] {
        &[
            MetamagicType::Quicken,
            MetamagicType::Empower,
            MetamagicType::Maximize,
            MetamagicType::Silent,
            MetamagicType::Still,
            MetamagicType::Extend,
            MetamagicType::Widen,
            MetamagicType::Twin,
            MetamagicType::Careful,
            MetamagicType::Subtle,
        ]
    }
}

/// Modifiers applied by a metamagic
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MetamagicModifier {
    /// Damage multiplier (1.0 = no change)
    pub damage_multiplier: f32,
    /// Duration multiplier (1.0 = no change)
    pub duration_multiplier: f32,
    /// Area radius multiplier (1.0 = no change)
    pub area_multiplier: f32,
    /// Additional mana cost (% of base, 0.0 = no additional cost)
    pub mana_cost_percent: f32,
    /// Difficulty increase (% added to failure chance)
    pub difficulty_increase: i32,
    /// Whether spell ignores resistances
    pub ignores_resistance: bool,
    /// Whether spell is rolled with maximum values instead of dice
    pub maximized: bool,
    /// Number of targets (1 = single, 2 = twin, etc.)
    pub target_count: u8,
    /// Whether spell requires verbal components
    pub requires_verbal: bool,
    /// Whether spell requires somatic components
    pub requires_somatic: bool,
}

impl Default for MetamagicModifier {
    fn default() -> Self {
        Self {
            damage_multiplier: 1.0,
            duration_multiplier: 1.0,
            area_multiplier: 1.0,
            mana_cost_percent: 0.0,
            difficulty_increase: 0,
            ignores_resistance: false,
            maximized: false,
            target_count: 1,
            requires_verbal: true,
            requires_somatic: true,
        }
    }
}

impl MetamagicModifier {
    /// Get modifier for a specific metamagic type
    pub fn for_metamagic(metamagic: MetamagicType) -> Self {
        match metamagic {
            MetamagicType::Quicken => Self {
                mana_cost_percent: 100.0, // Costs double mana
                difficulty_increase: 10,
                ..Default::default()
            },
            MetamagicType::Empower => Self {
                damage_multiplier: 1.5,
                mana_cost_percent: 50.0,
                difficulty_increase: 5,
                ..Default::default()
            },
            MetamagicType::Maximize => Self {
                maximized: true,
                mana_cost_percent: 100.0,
                difficulty_increase: 10,
                ..Default::default()
            },
            MetamagicType::Silent => Self {
                mana_cost_percent: 25.0,
                requires_verbal: false,
                difficulty_increase: 5,
                ..Default::default()
            },
            MetamagicType::Still => Self {
                mana_cost_percent: 25.0,
                requires_somatic: false,
                difficulty_increase: 5,
                ..Default::default()
            },
            MetamagicType::Extend => Self {
                duration_multiplier: 2.0,
                mana_cost_percent: 50.0,
                difficulty_increase: 5,
                ..Default::default()
            },
            MetamagicType::Widen => Self {
                area_multiplier: 1.5,
                mana_cost_percent: 50.0,
                difficulty_increase: 5,
                ..Default::default()
            },
            MetamagicType::Twin => Self {
                target_count: 2,
                mana_cost_percent: 100.0,
                difficulty_increase: 10,
                ..Default::default()
            },
            MetamagicType::Careful => Self {
                mana_cost_percent: 50.0,
                ignores_resistance: true,
                difficulty_increase: 0,
                ..Default::default()
            },
            MetamagicType::Subtle => Self {
                mana_cost_percent: 10.0,
                difficulty_increase: 0,
                ..Default::default()
            },
        }
    }

    /// Combine multiple modifiers (for stacking metamagics)
    pub fn combine(modifiers: &[MetamagicModifier]) -> Self {
        let mut result = Self::default();

        for modifier in modifiers {
            result.damage_multiplier *= modifier.damage_multiplier;
            result.duration_multiplier *= modifier.duration_multiplier;
            result.area_multiplier *= modifier.area_multiplier;
            result.mana_cost_percent += modifier.mana_cost_percent;
            result.difficulty_increase += modifier.difficulty_increase;
            result.ignores_resistance |= modifier.ignores_resistance;
            result.maximized |= modifier.maximized;
            result.target_count = result.target_count.max(modifier.target_count);
            result.requires_verbal &= modifier.requires_verbal;
            result.requires_somatic &= modifier.requires_somatic;
        }

        // Cap difficulty increase at 90%
        result.difficulty_increase = result.difficulty_increase.min(90);

        result
    }
}

/// Player's knowledge of metamagic techniques
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetamagicKnowledge {
    /// Known metamagics: metamagic -> mastery level (0-100)
    pub known: HashMap<MetamagicType, u8>,
    /// Total metamagic XP earned
    pub total_xp: u32,
    /// Metamagic points available to spend on new metamagics
    pub points_available: u32,
}

impl Default for MetamagicKnowledge {
    fn default() -> Self {
        Self {
            known: HashMap::new(),
            total_xp: 0,
            points_available: 0,
        }
    }
}

impl MetamagicKnowledge {
    /// Create new metamagic knowledge
    pub fn new() -> Self {
        Self::default()
    }

    /// Learn a new metamagic
    pub fn learn(&mut self, metamagic: MetamagicType) -> bool {
        if self.known.contains_key(&metamagic) {
            return false; // Already known
        }
        self.known.insert(metamagic, 1);
        true
    }

    /// Check if metamagic is known
    pub fn knows(&self, metamagic: MetamagicType) -> bool {
        self.known.contains_key(&metamagic)
    }

    /// Get mastery level for metamagic (0-100)
    pub fn mastery_level(&self, metamagic: MetamagicType) -> u8 {
        self.known.get(&metamagic).copied().unwrap_or(0)
    }

    /// Record successful metamagic use for XP
    pub fn record_use(&mut self, metamagic: MetamagicType) {
        if self.knows(metamagic) {
            self.total_xp = self.total_xp.saturating_add(1);
            // Increase mastery slowly (max 100)
            if let Some(level) = self.known.get_mut(&metamagic) {
                if *level < 100 {
                    *level = level.saturating_add(1);
                }
            }
        }
    }

    /// Get mana cost reduction from mastery (0% - 50%)
    pub fn get_mana_reduction(&self, metamagic: MetamagicType) -> f32 {
        let mastery = self.mastery_level(metamagic);
        (mastery as f32 / 200.0).min(0.5) // Max 50% reduction at mastery 100
    }

    /// Get difficulty reduction from mastery (0% - 30%)
    pub fn get_difficulty_reduction(&self, metamagic: MetamagicType) -> i32 {
        let mastery = self.mastery_level(metamagic);
        (mastery as i32 / 10).min(30) // Max 30 percentage point reduction
    }
}

/// Apply metamagics to modify a spell's parameters
pub fn apply_metamagic(
    base_damage: i32,
    base_duration: i32,
    base_area_radius: i32,
    metamagics: &[MetamagicType],
    knowledge: &MetamagicKnowledge,
) -> Option<AppliedMetamagic> {
    if metamagics.is_empty() {
        return None;
    }

    let mut modifiers = Vec::new();
    for &metamagic in metamagics {
        if !knowledge.knows(metamagic) {
            return None; // Unknown metamagic
        }
        modifiers.push(MetamagicModifier::for_metamagic(metamagic));
    }

    let combined = MetamagicModifier::combine(&modifiers);

    Some(AppliedMetamagic {
        metamagics: metamagics.to_vec(),
        damage: (base_damage as f32 * combined.damage_multiplier) as i32,
        duration: (base_duration as f32 * combined.duration_multiplier) as i32,
        area_radius: (base_area_radius as f32 * combined.area_multiplier) as i32,
        modifiers: combined,
    })
}

/// Result of applying metamagic to a spell
#[derive(Debug, Clone)]
pub struct AppliedMetamagic {
    pub metamagics: Vec<MetamagicType>,
    pub damage: i32,
    pub duration: i32,
    pub area_radius: i32,
    pub modifiers: MetamagicModifier,
}

/// Calculate total mana cost with metamagic
pub fn calculate_metamagic_cost(
    base_mana_cost: i32,
    metamagics: &[MetamagicType],
    knowledge: &MetamagicKnowledge,
) -> Option<i32> {
    if metamagics.is_empty() {
        return Some(base_mana_cost);
    }

    let mut total_cost = base_mana_cost as f32;

    for &metamagic in metamagics {
        if !knowledge.knows(metamagic) {
            return None;
        }

        let base_cost = metamagic.base_cost() as f32;
        let reduction = knowledge.get_mana_reduction(metamagic);
        let cost_after_reduction = base_cost * (1.0 - reduction);
        total_cost += cost_after_reduction;
    }

    Some(total_cost as i32)
}

/// Check if a metamagic can be applied to a spell
pub fn can_apply_metamagic(
    spell_type: crate::magic::spell::SpellType,
    metamagic: MetamagicType,
) -> bool {
    // Most metamagics can be applied to most spells
    // Some restrictions based on spell type:
    match metamagic {
        MetamagicType::Empower | MetamagicType::Maximize => {
            // Can't empower or maximize spells that don't do damage/healing
            spell_type.deals_damage() || spell_type.heals()
        }
        MetamagicType::Extend => {
            // Can't extend spells with no duration
            spell_type.has_duration()
        }
        MetamagicType::Widen => {
            // Can't widen single-target spells
            spell_type.is_area_effect()
        }
        MetamagicType::Twin => {
            // Can't twin spells that don't target
            spell_type.can_target_multiple()
        }
        _ => true, // All others can be applied to any spell
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metamagic_type_names() {
        assert_eq!(MetamagicType::Quicken.name(), "Quicken");
        assert_eq!(MetamagicType::Empower.name(), "Empower");
    }

    #[test]
    fn test_metamagic_base_costs() {
        assert_eq!(MetamagicType::Subtle.base_cost(), 10);
        assert_eq!(MetamagicType::Quicken.base_cost(), 50);
    }

    #[test]
    fn test_metamagic_modifier_combination() {
        let mod1 = MetamagicModifier::for_metamagic(MetamagicType::Empower);
        let mod2 = MetamagicModifier::for_metamagic(MetamagicType::Extend);

        let combined = MetamagicModifier::combine(&[mod1, mod2]);
        assert_eq!(combined.damage_multiplier, 1.5);
        assert_eq!(combined.duration_multiplier, 2.0);
    }

    #[test]
    fn test_metamagic_knowledge_learning() {
        let mut knowledge = MetamagicKnowledge::new();
        assert!(!knowledge.knows(MetamagicType::Quicken));

        knowledge.learn(MetamagicType::Quicken);
        assert!(knowledge.knows(MetamagicType::Quicken));

        // Can't learn twice
        assert!(!knowledge.learn(MetamagicType::Quicken));
    }

    #[test]
    fn test_metamagic_knowledge_mastery() {
        let mut knowledge = MetamagicKnowledge::new();
        knowledge.learn(MetamagicType::Empower);

        assert_eq!(knowledge.mastery_level(MetamagicType::Empower), 1);

        for _ in 0..50 {
            knowledge.record_use(MetamagicType::Empower);
        }

        assert!(knowledge.mastery_level(MetamagicType::Empower) > 1);
    }

    #[test]
    fn test_calculate_metamagic_cost() {
        let knowledge = {
            let mut k = MetamagicKnowledge::new();
            k.learn(MetamagicType::Silent);
            k
        };

        let cost = calculate_metamagic_cost(100, &[MetamagicType::Silent], &knowledge);
        assert!(cost.is_some());

        let cost_val = cost.unwrap();
        assert!(cost_val > 100); // Should be more than base
        assert!(cost_val < 150); // But not too much more
    }

    #[test]
    fn test_metamagic_cost_unknown() {
        let knowledge = MetamagicKnowledge::new();
        let cost = calculate_metamagic_cost(100, &[MetamagicType::Quicken], &knowledge);
        assert!(cost.is_none()); // Unknown metamagic
    }
}
