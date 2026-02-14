//! Monster personality system (Phase 18)
//!
//! Defines personality types that drive monster behavior, decision-making,
//! and strategic preferences. Each personality modifies how a monster
//! approaches combat, retreat, and ally coordination.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use super::tactics::Intelligence;

/// Monster personality types
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum Personality {
    /// Charges in aggressively, prefers damage-dealing attacks
    /// Flees later, takes more risks
    #[default]
    Aggressive = 0,

    /// Plays defensively, uses defensive abilities, shields weak allies
    /// Retreats earlier, avoids unnecessary risks
    Defensive = 1,

    /// Calculates moves carefully, adapts to situation
    /// Balances offense and defense
    Tactical = 2,

    /// Avoids direct combat, uses hit-and-run tactics
    /// Retreats much earlier, prioritizes survival
    Coward = 3,

    /// Reckless combatant, will fight to near-death
    /// Rarely retreats, uses powerful attacks even when weakened
    Berserker = 4,

    /// Careful and methodical, evaluates threats
    /// Uses buffing/debuffing tactics before engaging
    Cautious = 5,
}

/// Attack type preferences per personality
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonalityProfile {
    pub personality: Personality,

    /// Base aggression level (-100 to 100)
    /// Higher = more aggressive attack choices
    pub aggression: i8,

    /// Self-preservation priority (-100 to 100)
    /// Higher = prioritizes own survival, retreats earlier
    pub self_preservation: i8,

    /// Ally loyalty factor (-100 to 100)
    /// Higher = considers ally welfare, may retreat to protect allies
    pub ally_loyalty: i8,

    /// Tactical depth (0-5)
    /// Higher = uses more complex tactical decisions
    pub tactical_depth: u8,

    /// Attack type preferences
    pub prefers_melee: i8, // -100 (never) to 100 (always)
    pub prefers_ranged: i8,  // Ranged attacks
    pub prefers_spells: i8,  // Spell casting
    pub prefers_breath: i8,  // Breath weapons
    pub prefers_special: i8, // Special abilities (gaze, spit, etc)
}

impl PersonalityProfile {
    /// Create personality profile for a given personality type
    pub fn for_personality(personality: Personality) -> Self {
        match personality {
            Personality::Aggressive => Self {
                personality,
                aggression: 80,
                self_preservation: -60,
                ally_loyalty: -40,
                tactical_depth: 2,
                prefers_melee: 90,
                prefers_ranged: 50,
                prefers_spells: 30,
                prefers_breath: 70,
                prefers_special: 40,
            },

            Personality::Defensive => Self {
                personality,
                aggression: -40,
                self_preservation: 70,
                ally_loyalty: 80,
                tactical_depth: 3,
                prefers_melee: 40,
                prefers_ranged: 70,
                prefers_spells: 80,
                prefers_breath: 30,
                prefers_special: 60,
            },

            Personality::Tactical => Self {
                personality,
                aggression: 20,
                self_preservation: 40,
                ally_loyalty: 50,
                tactical_depth: 5,
                prefers_melee: 50,
                prefers_ranged: 70,
                prefers_spells: 80,
                prefers_breath: 60,
                prefers_special: 70,
            },

            Personality::Coward => Self {
                personality,
                aggression: -80,
                self_preservation: 90,
                ally_loyalty: -70,
                tactical_depth: 2,
                prefers_melee: 20,
                prefers_ranged: 80,
                prefers_spells: 90,
                prefers_breath: 10,
                prefers_special: 40,
            },

            Personality::Berserker => Self {
                personality,
                aggression: 100,
                self_preservation: -90,
                ally_loyalty: -80,
                tactical_depth: 1,
                prefers_melee: 100,
                prefers_ranged: 20,
                prefers_spells: 10,
                prefers_breath: 80,
                prefers_special: 60,
            },

            Personality::Cautious => Self {
                personality,
                aggression: -20,
                self_preservation: 80,
                ally_loyalty: 60,
                tactical_depth: 4,
                prefers_melee: 30,
                prefers_ranged: 80,
                prefers_spells: 90,
                prefers_breath: 40,
                prefers_special: 80,
            },
        }
    }
}

/// Assign a personality to a monster based on intelligence
///
/// Distribution by intelligence level:
/// - Mindless/Animal: Mostly Aggressive/Defensive
/// - Low: Mix of Aggressive/Defensive/Coward
/// - Average: More variety, mostly Tactical/Defensive
/// - High: Tactical/Cautious/Defensive
/// - Genius: All types, weighted toward Tactical/Cautious
pub fn assign_personality(intelligence: Intelligence, seed: u32) -> Personality {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let hash = hasher.finish() as u32;
    let roll = hash % 100;

    match intelligence {
        Intelligence::Mindless => {
            // Mindless creatures are purely aggressive
            Personality::Aggressive
        }

        Intelligence::Animal => {
            // Animal intelligence: mostly aggressive, some defensive
            if roll < 70 {
                Personality::Aggressive
            } else {
                Personality::Defensive
            }
        }

        Intelligence::Low => {
            // Low intelligence: basic variation
            if roll < 50 {
                Personality::Aggressive
            } else if roll < 75 {
                Personality::Defensive
            } else {
                Personality::Coward
            }
        }

        Intelligence::Average => {
            // Average intelligence: good variety
            if roll < 25 {
                Personality::Aggressive
            } else if roll < 35 {
                Personality::Berserker
            } else if roll < 50 {
                Personality::Defensive
            } else if roll < 65 {
                Personality::Tactical
            } else if roll < 80 {
                Personality::Cautious
            } else {
                Personality::Coward
            }
        }

        Intelligence::High => {
            // High intelligence: prefers thoughtful personalities
            if roll < 15 {
                Personality::Aggressive
            } else if roll < 25 {
                Personality::Berserker
            } else if roll < 35 {
                Personality::Defensive
            } else if roll < 50 {
                Personality::Tactical
            } else if roll < 70 {
                Personality::Cautious
            } else {
                Personality::Coward
            }
        }

        Intelligence::Genius => {
            // Genius intelligence: mostly tactical and cautious
            if roll < 15 {
                Personality::Aggressive
            } else if roll < 20 {
                Personality::Berserker
            } else if roll < 25 {
                Personality::Defensive
            } else if roll < 45 {
                Personality::Tactical
            } else if roll < 70 {
                Personality::Cautious
            } else {
                Personality::Coward
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_profile_creation() {
        for personality in [
            Personality::Aggressive,
            Personality::Defensive,
            Personality::Tactical,
            Personality::Coward,
            Personality::Berserker,
            Personality::Cautious,
        ] {
            let profile = PersonalityProfile::for_personality(personality);
            assert_eq!(profile.personality, personality);
            assert!(profile.aggression >= -100 && profile.aggression <= 100);
            assert!(profile.self_preservation >= -100 && profile.self_preservation <= 100);
            assert!(profile.ally_loyalty >= -100 && profile.ally_loyalty <= 100);
            assert!(profile.tactical_depth <= 5);
        }
    }

    #[test]
    fn test_personality_traits() {
        let aggressive = PersonalityProfile::for_personality(Personality::Aggressive);
        let coward = PersonalityProfile::for_personality(Personality::Coward);

        // Aggressive should prefer melee and have high aggression
        assert!(aggressive.prefers_melee > coward.prefers_melee);
        assert!(aggressive.aggression > coward.aggression);

        // Coward should have high self-preservation
        assert!(coward.self_preservation > aggressive.self_preservation);
    }

    #[test]
    fn test_assign_personality_distribution() {
        // Test that assignment function doesn't panic for all intelligence levels
        for intel in [
            Intelligence::Mindless,
            Intelligence::Animal,
            Intelligence::Low,
            Intelligence::Average,
            Intelligence::High,
            Intelligence::Genius,
        ] {
            let personality = assign_personality(intel, 42);
            // Just verify we get a valid personality
            let _ = PersonalityProfile::for_personality(personality);
        }
    }

    #[test]
    fn test_mindless_personality() {
        // Mindless creatures should always be aggressive
        assert_eq!(
            assign_personality(Intelligence::Mindless, 0),
            Personality::Aggressive
        );
        assert_eq!(
            assign_personality(Intelligence::Mindless, 999),
            Personality::Aggressive
        );
    }
}
