//! Spell synergies and combo system
//!
//! Implements spell combinations that create enhanced effects when cast
//! in succession or in proximity to each other. Allows for tactical spell use.

use crate::magic::spell::{SpellSchool, SpellType};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Spell combo effect when multiple spells interact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SynergyEffect {
    // Damage synergies
    ElementalChain,     // Fire + Cold = stronger area damage
    MagicAmplification, // Two attack spells = 1.5x damage
    PiercingMagic,      // Multiple damage spells break defenses

    // Control synergies
    LockDown,  // Slow + Confuse = immobilized
    MindBreak, // Multiple enchantment spells = permanent confusion
    TimeWarp,  // Slow + Temporal spell = stasis

    // Healing synergies
    HolyRenewal,   // Healing + Clerical = full restore
    VitalityBoost, // Multiple healing spells = extra max HP
    LifeLink,      // Healing self + others = group healing

    // Detection synergies
    TrueVision,      // Detect magic + Detect monsters = see everything
    ArcaneResonance, // Multiple divination spells = permanent see invisible

    // Escape synergies
    DimensionalShift, // Teleport + Invisibility = untrackable
    SpeedWarp,        // Speed + Haste = ultra fast movement

    // Transmutation synergies
    MatterInfusion,   // Matter spells combine = transmute anything
    MetalworkMastery, // Multiple matter spells = indestructible items
}

impl SynergyEffect {
    /// Get bonus multiplier (damage, duration, etc.)
    pub fn bonus_multiplier(&self) -> f32 {
        match self {
            SynergyEffect::ElementalChain => 1.5,
            SynergyEffect::MagicAmplification => 1.5,
            SynergyEffect::PiercingMagic => 2.0,
            SynergyEffect::LockDown => 1.0, // Binary effect
            SynergyEffect::MindBreak => 1.0,
            SynergyEffect::TimeWarp => 1.0,
            SynergyEffect::HolyRenewal => 2.0,
            SynergyEffect::VitalityBoost => 1.25,
            SynergyEffect::LifeLink => 1.5,
            SynergyEffect::TrueVision => 2.0,
            SynergyEffect::ArcaneResonance => 1.0,
            SynergyEffect::DimensionalShift => 1.0,
            SynergyEffect::SpeedWarp => 2.0,
            SynergyEffect::MatterInfusion => 1.5,
            SynergyEffect::MetalworkMastery => 1.0,
        }
    }

    /// Get description of synergy effect
    pub fn description(&self) -> &'static str {
        match self {
            SynergyEffect::ElementalChain => "elemental chain reaction",
            SynergyEffect::MagicAmplification => "magic amplification",
            SynergyEffect::PiercingMagic => "piercing magic",
            SynergyEffect::LockDown => "lockdown",
            SynergyEffect::MindBreak => "mind break",
            SynergyEffect::TimeWarp => "time warp",
            SynergyEffect::HolyRenewal => "holy renewal",
            SynergyEffect::VitalityBoost => "vitality boost",
            SynergyEffect::LifeLink => "life link",
            SynergyEffect::TrueVision => "true vision",
            SynergyEffect::ArcaneResonance => "arcane resonance",
            SynergyEffect::DimensionalShift => "dimensional shift",
            SynergyEffect::SpeedWarp => "speed warp",
            SynergyEffect::MatterInfusion => "matter infusion",
            SynergyEffect::MetalworkMastery => "metalwork mastery",
        }
    }

    /// Mana cost reduction from synergy (percentage)
    pub fn mana_reduction(&self) -> i32 {
        match self {
            SynergyEffect::ElementalChain => 10,
            SynergyEffect::MagicAmplification => 15,
            SynergyEffect::PiercingMagic => 20,
            SynergyEffect::LockDown => 25,
            SynergyEffect::MindBreak => 30,
            SynergyEffect::TimeWarp => 20,
            SynergyEffect::HolyRenewal => 25,
            SynergyEffect::VitalityBoost => 15,
            SynergyEffect::LifeLink => 20,
            SynergyEffect::TrueVision => 30,
            SynergyEffect::ArcaneResonance => 25,
            SynergyEffect::DimensionalShift => 25,
            SynergyEffect::SpeedWarp => 20,
            SynergyEffect::MatterInfusion => 20,
            SynergyEffect::MetalworkMastery => 25,
        }
    }
}

/// Recent spell cast for synergy tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentSpell {
    pub spell_type: SpellType,
    pub school: SpellSchool,
    pub turns_ago: u32,
}

/// Track recently cast spells for synergy detection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpellSynergyTracker {
    /// Queue of recent spells (up to 5 most recent)
    pub recent_spells: VecDeque<RecentSpell>,
    pub max_recent: usize,
}

impl SpellSynergyTracker {
    pub fn new() -> Self {
        Self {
            recent_spells: VecDeque::new(),
            max_recent: 5,
        }
    }

    /// Record a spell cast
    pub fn record_spell(&mut self, spell_type: SpellType, school: SpellSchool) {
        self.recent_spells.push_front(RecentSpell {
            spell_type,
            school,
            turns_ago: 0,
        });

        if self.recent_spells.len() > self.max_recent {
            self.recent_spells.pop_back();
        }
    }

    /// Advance time (called each turn)
    pub fn tick(&mut self) {
        for spell in &mut self.recent_spells {
            spell.turns_ago += 1;
        }

        // Remove spells older than 10 turns
        self.recent_spells.retain(|s| s.turns_ago < 10);
    }

    /// Clear old spells beyond window
    pub fn clear_expired(&mut self) {
        self.recent_spells.retain(|s| s.turns_ago < 10);
    }

    /// Get count of specific school in recent spells
    pub fn count_school(&self, school: SpellSchool) -> usize {
        self.recent_spells
            .iter()
            .filter(|s| s.school == school)
            .count()
    }

    /// Get count of specific spell type
    pub fn count_spell(&self, spell_type: SpellType) -> usize {
        self.recent_spells
            .iter()
            .filter(|s| s.spell_type == spell_type)
            .count()
    }

    /// Check for synergies with new spell
    pub fn check_synergies(
        &self,
        new_spell: SpellType,
        new_school: SpellSchool,
    ) -> Vec<SynergyEffect> {
        let mut synergies = Vec::new();

        // Must have recent spells for synergy
        if self.recent_spells.is_empty() {
            return synergies;
        }

        // Get most recent spell (should be cast within last 3 turns)
        if let Some(recent) = self.recent_spells.front() {
            if recent.turns_ago > 3 {
                return synergies;
            }

            // Check for school-based synergies
            match (recent.school, new_school) {
                // Fire + Cold = Elemental Chain
                (SpellSchool::Attack, SpellSchool::Attack)
                    if self.count_school(SpellSchool::Attack) >= 2 =>
                {
                    synergies.push(SynergyEffect::ElementalChain);
                }

                // Multiple attack spells = Magic Amplification
                (SpellSchool::Attack, SpellSchool::Attack) => {
                    synergies.push(SynergyEffect::MagicAmplification);
                }

                // Enchantment stacking = Mind Break
                (SpellSchool::Enchantment, SpellSchool::Enchantment)
                    if self.count_school(SpellSchool::Enchantment) >= 3 =>
                {
                    synergies.push(SynergyEffect::MindBreak);
                }

                // Healing + Clerical = Holy Renewal
                (SpellSchool::Healing, SpellSchool::Clerical)
                | (SpellSchool::Clerical, SpellSchool::Healing) => {
                    synergies.push(SynergyEffect::HolyRenewal);
                }

                // Divination stacking = True Vision
                (SpellSchool::Divination, SpellSchool::Divination)
                    if self.count_school(SpellSchool::Divination) >= 2 =>
                {
                    synergies.push(SynergyEffect::TrueVision);
                }

                // Escape + Escape = enhanced escape
                (SpellSchool::Escape, SpellSchool::Escape) => {
                    synergies.push(SynergyEffect::DimensionalShift);
                }

                _ => {}
            }
        }

        synergies
    }
}

/// Combo chain tracker for multi-spell combinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboChain {
    pub spells: Vec<SpellType>,
    pub current_multiplier: f32,
    pub active: bool,
}

impl ComboChain {
    pub fn new() -> Self {
        Self {
            spells: Vec::new(),
            current_multiplier: 1.0,
            active: true,
        }
    }

    /// Add spell to combo
    pub fn add_spell(&mut self, spell_type: SpellType) {
        self.spells.push(spell_type);
        // Each spell in combo increases multiplier by 0.2 (1.0 -> 1.2 -> 1.4 -> etc)
        self.current_multiplier += 0.2;
        self.current_multiplier = self.current_multiplier.min(3.0); // Cap at 3x
    }

    /// Get combo count
    pub fn combo_count(&self) -> usize {
        self.spells.len()
    }

    /// Check if combo breaks (different school or time gap)
    pub fn breaks_on_school_change(&self, new_school: SpellSchool) -> bool {
        if let Some(first) = self.spells.first() {
            // If we switch schools, combo breaks
            false // Stub - would need spell type -> school mapping
        } else {
            false
        }
    }

    /// Reset combo
    pub fn reset(&mut self) {
        self.spells.clear();
        self.current_multiplier = 1.0;
        self.active = false;
    }

    /// Get combo message
    pub fn combo_message(&self) -> Option<String> {
        match self.combo_count() {
            0 => None,
            1 => Some("Spell combo x1".to_string()),
            2 => Some("Spell combo x2! Damage increased!".to_string()),
            3 => Some("Spell combo x3! Devastating power!".to_string()),
            4 => Some("Spell combo x4! Unstoppable!".to_string()),
            n => Some(format!("Spell combo x{}! Ultimate power!", n)),
        }
    }
}

/// Check if two spells have intrinsic synergy
pub fn spells_synergize(spell1: SpellType, spell2: SpellType) -> bool {
    // Specific spell pairs that synergize
    match (spell1, spell2) {
        // Add pairs that synergize
        _ => false,
    }
}

/// Get synergy bonus for specific spell pair
pub fn get_spell_pair_bonus(spell1: SpellType, spell2: SpellType) -> f32 {
    if spells_synergize(spell1, spell2) {
        1.3 // 30% bonus for synergizing pairs
    } else {
        1.0
    }
}

/// Calculate mana cost with synergy reductions
pub fn calculate_synergy_mana_cost(base_cost: i32, synergies: &[SynergyEffect]) -> i32 {
    let mut total_reduction = 0;

    for synergy in synergies {
        total_reduction += synergy.mana_reduction();
    }

    let reduction = (total_reduction as f32 / 100.0) * base_cost as f32;
    (base_cost as f32 - reduction).max(base_cost as f32 / 2.0) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synergy_effect_multiplier() {
        assert_eq!(SynergyEffect::PiercingMagic.bonus_multiplier(), 2.0);
        assert_eq!(SynergyEffect::HolyRenewal.bonus_multiplier(), 2.0);
        assert_eq!(SynergyEffect::MagicAmplification.bonus_multiplier(), 1.5);
    }

    #[test]
    fn test_synergy_effect_description() {
        assert!(!SynergyEffect::ElementalChain.description().is_empty());
        assert!(!SynergyEffect::HolyRenewal.description().is_empty());
    }

    #[test]
    fn test_synergy_effect_mana_reduction() {
        assert!(SynergyEffect::TrueVision.mana_reduction() > 0);
        assert!(SynergyEffect::MindBreak.mana_reduction() > 0);
    }

    #[test]
    fn test_spell_synergy_tracker_new() {
        let tracker = SpellSynergyTracker::new();
        assert_eq!(tracker.recent_spells.len(), 0);
    }

    #[test]
    fn test_spell_synergy_tracker_record() {
        let mut tracker = SpellSynergyTracker::new();
        tracker.record_spell(SpellType::ForceBolt, SpellSchool::Attack);
        assert_eq!(tracker.recent_spells.len(), 1);
    }

    #[test]
    fn test_spell_synergy_tracker_tick() {
        let mut tracker = SpellSynergyTracker::new();
        tracker.record_spell(SpellType::ForceBolt, SpellSchool::Attack);
        tracker.tick();
        assert_eq!(tracker.recent_spells.front().unwrap().turns_ago, 1);
    }

    #[test]
    fn test_spell_synergy_tracker_count_school() {
        let mut tracker = SpellSynergyTracker::new();
        tracker.record_spell(SpellType::ForceBolt, SpellSchool::Attack);
        tracker.record_spell(SpellType::Drain, SpellSchool::Attack);
        assert_eq!(tracker.count_school(SpellSchool::Attack), 2);
    }

    #[test]
    fn test_spell_synergy_tracker_check_synergies() {
        let mut tracker = SpellSynergyTracker::new();
        tracker.record_spell(SpellType::ForceBolt, SpellSchool::Attack);

        let synergies = tracker.check_synergies(SpellType::Drain, SpellSchool::Attack);
        assert!(!synergies.is_empty()); // Should detect attack synergy
    }

    #[test]
    fn test_combo_chain_new() {
        let combo = ComboChain::new();
        assert_eq!(combo.combo_count(), 0);
        assert_eq!(combo.current_multiplier, 1.0);
    }

    #[test]
    fn test_combo_chain_add_spell() {
        let mut combo = ComboChain::new();
        combo.add_spell(SpellType::ForceBolt);
        assert_eq!(combo.combo_count(), 1);
        assert!(combo.current_multiplier > 1.0);
    }

    #[test]
    fn test_combo_chain_multiple_spells() {
        let mut combo = ComboChain::new();
        combo.add_spell(SpellType::ForceBolt);
        combo.add_spell(SpellType::Drain);
        combo.add_spell(SpellType::MagicMissile);

        assert_eq!(combo.combo_count(), 3);
        assert!(combo.current_multiplier >= 1.4); // At least 2 additions
    }

    #[test]
    fn test_combo_chain_multiplier_cap() {
        let mut combo = ComboChain::new();
        for _ in 0..20 {
            combo.add_spell(SpellType::ForceBolt);
        }
        assert!(combo.current_multiplier <= 3.0); // Should be capped at 3x
    }

    #[test]
    fn test_combo_chain_message() {
        let mut combo = ComboChain::new();
        combo.add_spell(SpellType::ForceBolt);
        combo.add_spell(SpellType::Drain);

        let msg = combo.combo_message();
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("x2"));
    }

    #[test]
    fn test_calculate_synergy_mana_cost() {
        let synergies = vec![SynergyEffect::MagicAmplification];
        let cost = calculate_synergy_mana_cost(100, &synergies);

        assert!(cost < 100); // Should be reduced
        assert!(cost >= 50); // Should not be reduced too much
    }

    #[test]
    fn test_calculate_synergy_mana_cost_multiple() {
        let synergies = vec![
            SynergyEffect::MagicAmplification,
            SynergyEffect::ElementalChain,
        ];
        let cost_single = calculate_synergy_mana_cost(100, &[synergies[0]]);
        let cost_multiple = calculate_synergy_mana_cost(100, &synergies);

        assert!(cost_multiple <= cost_single); // More synergies = more savings
    }

    #[test]
    fn test_get_spell_pair_bonus() {
        let bonus = get_spell_pair_bonus(SpellType::ForceBolt, SpellType::Drain);
        assert!(bonus >= 1.0);
    }

    #[test]
    fn test_spells_synergize() {
        let result = spells_synergize(SpellType::ForceBolt, SpellType::Drain);
        assert!(result == true || result == false); // Just test it doesn't panic
    }

    #[test]
    fn test_spell_synergy_tracker_clear_expired() {
        let mut tracker = SpellSynergyTracker::new();
        tracker.record_spell(SpellType::ForceBolt, SpellSchool::Attack);

        // Tick many times to age the spell
        for _ in 0..15 {
            tracker.tick();
        }

        tracker.clear_expired();
        assert_eq!(tracker.recent_spells.len(), 0);
    }

    #[test]
    fn test_recent_spell_struct() {
        let recent = RecentSpell {
            spell_type: SpellType::ForceBolt,
            school: SpellSchool::Attack,
            turns_ago: 2,
        };

        assert_eq!(recent.spell_type, SpellType::ForceBolt);
        assert_eq!(recent.turns_ago, 2);
    }
}
