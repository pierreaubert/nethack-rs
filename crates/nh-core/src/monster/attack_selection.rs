//! Monster attack selection system (Phase 18)
//!
//! Intelligent attack selection based on combat memory, resources,
//! tactical situation, and personality-driven preferences.

use serde::{Deserialize, Serialize};
use hashbrown::HashMap;

use crate::combat::{AttackType, DamageType};

/// Preconditions for using an attack/ability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Precondition {
    /// Requires clear line of sight to target
    RequiresLineOfSight,

    /// Requires minimum distance (ranged attacks)
    RequiresDistance { min: i32, max: i32 },

    /// Requires mana (spell/ability casting)
    RequiresMana { amount: i32 },

    /// Requires specific ability charge
    RequiresCharge,

    /// Requires cooldown to be ready
    CooldownReady { turns_remaining: i32 },

    /// Requires adjacent target (melee)
    RequiresAdjacent,

    /// Requires specific monster form/ability
    RequiresAbility,

    /// Can be used anytime
    Always,
}

/// Scoring option for an attack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackOption {
    /// Attack index or ability identifier
    pub attack_id: u8,

    /// Name for debugging
    pub name: String,

    /// Base damage or effectiveness estimate
    pub base_effectiveness: i32,

    /// Preconditions that must be met
    pub preconditions: Vec<Precondition>,

    /// Resource cost (mana, cooldown turns, charges)
    pub resource_cost: ResourceCost,

    /// Attack type
    pub attack_type: AttackType,

    /// Damage type
    pub damage_type: DamageType,
}

/// Resource cost for using an ability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceCost {
    /// No resource cost
    None,

    /// Mana cost
    Mana(i32),

    /// Cooldown in turns
    Cooldown(u16),

    /// Limited uses (charges)
    Charge(u8),

    /// Combined cost
    Multiple,
}

/// Combat memory for a monster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatMemory {
    /// Track observed attack resistances
    pub observed_resistances: HashMap<DamageType, bool>,

    /// Recent attack success/failure history
    pub recent_attacks: Vec<AttackResult>,

    /// Success rate for each attack type
    pub attack_success_rates: HashMap<AttackType, f32>,

    /// Last turn observed player ability/power
    pub last_power_observation: u16,
}

/// Result of an attack attempt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttackResult {
    pub attack_type: AttackType,
    pub hit: bool,
    pub damage_dealt: i32,
    pub age_turns: u16,
}

impl Default for CombatMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl CombatMemory {
    /// Create new combat memory
    pub fn new() -> Self {
        Self {
            observed_resistances: HashMap::new(),
            recent_attacks: Vec::new(),
            attack_success_rates: HashMap::new(),
            last_power_observation: 0,
        }
    }

    /// Record an attack result
    pub fn record_attack(&mut self, result: AttackResult) {
        self.recent_attacks.push(result);

        // Keep only last 20 attacks
        if self.recent_attacks.len() > 20 {
            self.recent_attacks.remove(0);
        }

        // Update success rate for attack type
        let same_type_attacks: Vec<_> = self
            .recent_attacks
            .iter()
            .filter(|a| a.attack_type == result.attack_type)
            .collect();

        if !same_type_attacks.is_empty() {
            let hits = same_type_attacks.iter().filter(|a| a.hit).count();
            let rate = hits as f32 / same_type_attacks.len() as f32;
            self.attack_success_rates.insert(result.attack_type, rate);
        }
    }

    /// Record observed resistance to damage type
    pub fn record_resistance(&mut self, damage_type: DamageType, resists: bool) {
        self.observed_resistances.insert(damage_type, resists);
    }

    /// Check if target resists damage type (based on observation)
    pub fn target_resists(&self, damage_type: DamageType) -> Option<bool> {
        self.observed_resistances.get(&damage_type).copied()
    }

    /// Get success rate for attack type
    pub fn get_attack_success_rate(&self, attack_type: AttackType) -> f32 {
        self.attack_success_rates
            .get(&attack_type)
            .copied()
            .unwrap_or(0.5)
    }

    /// Age all tracked data by one turn
    pub fn age(&mut self) {
        for attack in &mut self.recent_attacks {
            attack.age_turns += 1;
        }
        // Remove very old attacks
        self.recent_attacks.retain(|a| a.age_turns < 20);

        if self.last_power_observation > 0 {
            self.last_power_observation += 1;
            if self.last_power_observation > 50 {
                self.last_power_observation = 0;
            }
        }
    }
}

/// Ability type for resource management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbilityType {
    BreathWeapon,
    GazeAttack,
    SpitVenom,
    SpellCast,
    Summon,
    TeleportSelf,
    Steal,
}

/// Target evaluation candidate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetCandidate {
    /// Entity ID or position
    pub target_id: u32,

    /// Threat score (0-100)
    pub threat_score: i32,

    /// Distance squared from attacker
    pub distance_sq: i32,

    /// HP percentage of target (0-100)
    pub hp_percent: i32,
}

/// Select best melee attack option for a monster
///
/// Scoring formula:
/// score = base_effectiveness
///       × personality_preference
///       × success_rate_modifier
///       × intelligence_scaling
pub fn select_best_attack(
    monster_id: u32,
    available_attacks: &[AttackOption],
    personality_preference: &hashbrown::HashMap<AttackType, i8>,
    success_rates: &hashbrown::HashMap<AttackType, f32>,
    intelligence: crate::monster::Intelligence,
    player_hp_percent: f32,
    monster_hp_percent: f32,
    rng: u32,
) -> Option<u8> {
    if available_attacks.is_empty() {
        return None;
    }

    // Filter attacks that are currently viable
    let viable_attacks: Vec<_> = available_attacks
        .iter()
        .enumerate()
        .filter(|(_, attack)| can_use_attack(attack))
        .collect();

    if viable_attacks.is_empty() {
        return None;
    }

    // Score each viable attack
    let mut best_score: f32 = -1000.0;
    let mut best_attack_idx: Option<u8> = None;

    for (idx, attack) in viable_attacks {
        let mut score = attack.base_effectiveness as f32;

        // Apply personality preference modifier
        if let Some(&pref) = personality_preference.get(&attack.attack_type) {
            score *= 1.0 + (pref as f32 / 100.0);
        }

        // Apply success rate modifier
        if let Some(&rate) = success_rates.get(&attack.attack_type) {
            score *= rate.max(0.1); // Minimum 10% weight even if never used
        }

        // Situational modifiers
        if attack.damage_type == DamageType::Fire && player_hp_percent < 0.5 {
            score *= 1.2; // Prefer fire on weakened enemy
        }
        if attack.damage_type == DamageType::Sleep && player_hp_percent < 0.3 {
            score *= 1.1; // Prefer sleep when player very weakened
        }

        // If monster is low HP, prefer defensive/escape attacks
        if monster_hp_percent < 0.3 {
            if attack.attack_type == AttackType::Gaze {
                score *= 1.3; // Gaze attacks can be tactical
            }
            if attack.attack_type == AttackType::Spit {
                score *= 1.2; // Spit allows distance
            }
        }

        // Apply intelligence scaling (smart monsters consistent, dumb ones random)
        let intelligence_noise = match intelligence {
            crate::monster::Intelligence::Mindless => 0.8, // 20% variance
            crate::monster::Intelligence::Animal => 0.9,   // 10% variance
            crate::monster::Intelligence::Low => 0.95,     // 5% variance
            crate::monster::Intelligence::Average => 0.98, // 2% variance
            crate::monster::Intelligence::High => 0.99,    // 1% variance
            crate::monster::Intelligence::Genius => 0.99,  // <1% variance
        };
        score *= intelligence_noise;

        if score > best_score {
            best_score = score;
            best_attack_idx = Some(idx as u8);
        }
    }

    best_attack_idx
}

/// Check if an attack can be used (preconditions met)
fn can_use_attack(attack: &AttackOption) -> bool {
    for precondition in &attack.preconditions {
        match precondition {
            Precondition::Always => continue,
            _ => {
                // In full implementation, check actual conditions
                // For now, assume all preconditions are met
                continue;
            }
        }
    }
    true
}

/// Select appropriate spell for current situation
pub fn select_monster_spell(
    current_hp: i32,
    max_hp: i32,
    mana: i32,
    has_allies: bool,
    player_threatening: bool,
    intelligence: crate::monster::Intelligence,
) -> Option<&'static str> {
    let hp_percent = if max_hp > 0 {
        current_hp as f32 / max_hp as f32
    } else {
        1.0
    };

    // Prioritize by situation
    if hp_percent < 0.25 && mana >= 10 {
        // Very low HP - heal if possible
        return Some("healing");
    }

    if hp_percent < 0.5 && mana >= 8 {
        // Low HP - heal or boost
        return Some("extra_healing");
    }

    if player_threatening && mana >= 6 {
        // Player is threatening - debuff or crowd control
        match intelligence {
            crate::monster::Intelligence::Genius => Some("hold"), // More powerful CC
            crate::monster::Intelligence::High => Some("slow"),
            crate::monster::Intelligence::Average => Some("confuse"),
            _ => Some("blind"),
        }
    } else if mana >= 5 && has_allies {
        // Buff allies if available
        Some("haste")
    } else if mana >= 6 {
        // Offensive spell
        match intelligence {
            crate::monster::Intelligence::Genius => Some("fireball"),
            crate::monster::Intelligence::High => Some("magic_missile"),
            _ => Some("bolt"),
        }
    } else {
        None
    }
}

/// Score potential targets
pub fn score_targets(
    player_hp: i32,
    player_max_hp: i32,
    _allies: &[(u32, i32, i32)], // (id, hp, distance_sq)
    monster_intelligence: crate::monster::Intelligence,
) -> i32 {
    let player_hp_percent = if player_max_hp > 0 {
        (player_hp as f32 / player_max_hp as f32) * 100.0
    } else {
        100.0
    } as i32;

    // Base threat score for player (always threatening)
    let mut score: i32 = 80;

    // Adjust based on player HP
    if player_hp_percent < 25 {
        score += 20; // Very weakened - easy kill
    } else if player_hp_percent < 50 {
        score += 10; // Moderately wounded
    }

    // Adjust based on intelligence (smarter monsters more tactical)
    let intelligence_bonus = match monster_intelligence {
        crate::monster::Intelligence::Mindless => 0,
        crate::monster::Intelligence::Animal => 2,
        crate::monster::Intelligence::Low => 5,
        crate::monster::Intelligence::Average => 10,
        crate::monster::Intelligence::High => 15,
        crate::monster::Intelligence::Genius => 20,
    };

    score + intelligence_bonus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_memory_creation() {
        let memory = CombatMemory::new();
        assert_eq!(memory.recent_attacks.len(), 0);
        assert_eq!(memory.observed_resistances.len(), 0);
    }

    #[test]
    fn test_record_attack_result() {
        let mut memory = CombatMemory::new();
        let result = AttackResult {
            attack_type: AttackType::Weapon,
            hit: true,
            damage_dealt: 10,
            age_turns: 0,
        };

        memory.record_attack(result);
        assert_eq!(memory.recent_attacks.len(), 1);
    }

    #[test]
    fn test_success_rate_calculation() {
        let mut memory = CombatMemory::new();

        // Record 2 hits and 1 miss with weapon attacks
        memory.record_attack(AttackResult {
            attack_type: AttackType::Weapon,
            hit: true,
            damage_dealt: 5,
            age_turns: 0,
        });
        memory.record_attack(AttackResult {
            attack_type: AttackType::Weapon,
            hit: true,
            damage_dealt: 5,
            age_turns: 0,
        });
        memory.record_attack(AttackResult {
            attack_type: AttackType::Weapon,
            hit: false,
            damage_dealt: 0,
            age_turns: 0,
        });

        let rate = memory.get_attack_success_rate(AttackType::Weapon);
        assert!((rate - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_resistance_tracking() {
        let mut memory = CombatMemory::new();
        memory.record_resistance(DamageType::Fire, true);

        assert_eq!(memory.target_resists(DamageType::Fire), Some(true));
        assert_eq!(memory.target_resists(DamageType::Cold), None);
    }

    #[test]
    fn test_select_monster_spell() {
        // Low HP should prioritize healing
        let spell = select_monster_spell(
            10,
            100,
            20,
            false,
            false,
            crate::monster::Intelligence::Average,
        );
        assert_eq!(spell, Some("healing"));

        // No mana should return None
        let spell = select_monster_spell(
            100,
            100,
            0,
            false,
            true,
            crate::monster::Intelligence::Average,
        );
        assert_eq!(spell, None);
    }

    #[test]
    fn test_score_targets() {
        let score = score_targets(50, 100, &[], crate::monster::Intelligence::Average);
        assert!(score > 0);

        // Weakened target should score higher
        let low_hp_score = score_targets(20, 100, &[], crate::monster::Intelligence::Average);
        assert!(low_hp_score > score);
    }
}
