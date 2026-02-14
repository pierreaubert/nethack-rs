//! Ritual casting system - Complex multi-turn spells with powerful effects
//!
//! Rituals take multiple turns to complete, consume mana over time, and can be interrupted.
//! When complete, they produce powerful magical effects.

use serde::{Deserialize, Serialize};

/// Types of ritual spells
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RitualSpellType {
    /// Summon powerful creatures
    GreaterSummoning,
    /// Heal everyone in a wide area
    MassHealing,
    /// Return a creature to life (ultra-powerful)
    Resurrection,
    /// Briefly stop time for all but caster
    TimeStop,
    /// Attempt to gain a wish (unstable)
    Wish,
    /// Create a meteor shower
    MeteorSwarm,
    /// Destroy all undead in area
    HolyNuking,
    /// Bind a creature to the caster's will
    DominateCreature,
}

impl RitualSpellType {
    /// Get human-readable name
    pub const fn name(&self) -> &'static str {
        match self {
            RitualSpellType::GreaterSummoning => "Greater Summoning",
            RitualSpellType::MassHealing => "Mass Healing",
            RitualSpellType::Resurrection => "Resurrection",
            RitualSpellType::TimeStop => "Time Stop",
            RitualSpellType::Wish => "Wish",
            RitualSpellType::MeteorSwarm => "Meteor Swarm",
            RitualSpellType::HolyNuking => "Holy Nuking",
            RitualSpellType::DominateCreature => "Dominate Creature",
        }
    }

    /// Get turns required to complete
    pub const fn turns_required(&self) -> u32 {
        match self {
            RitualSpellType::GreaterSummoning => 5,
            RitualSpellType::MassHealing => 3,
            RitualSpellType::Resurrection => 10,
            RitualSpellType::TimeStop => 8,
            RitualSpellType::Wish => 15, // Ultra-expensive
            RitualSpellType::MeteorSwarm => 7,
            RitualSpellType::HolyNuking => 6,
            RitualSpellType::DominateCreature => 9,
        }
    }

    /// Get mana cost per turn
    pub const fn mana_cost_per_turn(&self) -> i32 {
        match self {
            RitualSpellType::GreaterSummoning => 50,
            RitualSpellType::MassHealing => 30,
            RitualSpellType::Resurrection => 100,
            RitualSpellType::TimeStop => 80,
            RitualSpellType::Wish => 150,
            RitualSpellType::MeteorSwarm => 70,
            RitualSpellType::HolyNuking => 60,
            RitualSpellType::DominateCreature => 90,
        }
    }

    /// Get total mana required
    pub const fn total_mana_required(&self) -> i32 {
        self.mana_cost_per_turn() * (self.turns_required() as i32)
    }

    /// Get failure chance (0-100) if interrupted
    pub const fn interruption_risk(&self) -> i32 {
        match self {
            RitualSpellType::GreaterSummoning => 20,
            RitualSpellType::MassHealing => 10,
            RitualSpellType::Resurrection => 50, // Very risky
            RitualSpellType::TimeStop => 40,
            RitualSpellType::Wish => 90, // Ultra-risky
            RitualSpellType::MeteorSwarm => 30,
            RitualSpellType::HolyNuking => 25,
            RitualSpellType::DominateCreature => 35,
        }
    }
}

/// Progress tracking for a ritual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RitualProgress {
    /// Type of ritual
    pub ritual_type: RitualSpellType,
    /// Turns completed so far
    pub turns_completed: u32,
    /// Mana invested so far
    pub mana_invested: i32,
    /// Whether ritual was interrupted
    pub interrupted: bool,
    /// Location where ritual was started (for area-effect rituals)
    pub start_x: i8,
    pub start_y: i8,
}

impl RitualProgress {
    /// Create new ritual progress
    pub fn new(ritual_type: RitualSpellType, x: i8, y: i8) -> Self {
        Self {
            ritual_type,
            turns_completed: 0,
            mana_invested: 0,
            interrupted: false,
            start_x: x,
            start_y: y,
        }
    }

    /// Get percentage complete (0-100)
    pub fn percent_complete(&self) -> u32 {
        ((self.turns_completed * 100) / self.ritual_type.turns_required()).min(100)
    }

    /// Check if ritual is complete
    pub fn is_complete(&self) -> bool {
        self.turns_completed >= self.ritual_type.turns_required()
    }

    /// Advance ritual by one turn
    pub fn advance_turn(&mut self) {
        if !self.interrupted {
            self.turns_completed += 1;
            self.mana_invested += self.ritual_type.mana_cost_per_turn();
        }
    }

    /// Interrupt the ritual
    pub fn interrupt(&mut self) {
        self.interrupted = true;
    }

    /// Get descriptive status
    pub fn status_message(&self) -> String {
        if self.interrupted {
            format!("Ritual {} was interrupted!", self.ritual_type.name())
        } else {
            format!(
                "Ritual {} {}% complete ({}/{})",
                self.ritual_type.name(),
                self.percent_complete(),
                self.turns_completed,
                self.ritual_type.turns_required()
            )
        }
    }
}

/// Tracker for active rituals
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RitualTracker {
    /// Currently active ritual (if any)
    pub active_ritual: Option<RitualProgress>,
    /// Rituals completed successfully
    pub rituals_completed: u32,
    /// Rituals interrupted
    pub rituals_interrupted: u32,
}

impl RitualTracker {
    /// Create new ritual tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a ritual is in progress
    pub fn is_in_progress(&self) -> bool {
        self.active_ritual.is_some()
    }

    /// Get active ritual if any
    pub fn active_ritual(&self) -> Option<&RitualProgress> {
        self.active_ritual.as_ref()
    }

    /// Get mutable active ritual
    pub fn active_ritual_mut(&mut self) -> Option<&mut RitualProgress> {
        self.active_ritual.as_mut()
    }

    /// Reset tracker
    pub fn reset(&mut self) {
        self.active_ritual = None;
    }
}

/// Begin a ritual casting
pub fn begin_ritual(
    ritual_type: RitualSpellType,
    player_x: i8,
    player_y: i8,
    player_energy: i32,
) -> Result<RitualProgress, String> {
    // Check if player has enough mana
    if player_energy < ritual_type.total_mana_required() {
        return Err(format!(
            "Not enough energy to complete ritual. Need {} mana.",
            ritual_type.total_mana_required()
        ));
    }

    Ok(RitualProgress::new(ritual_type, player_x, player_y))
}

/// Advance a ritual by one turn
pub fn advance_ritual(
    ritual: &mut RitualProgress,
    disruption_chance: i32,
    rng: &mut crate::rng::GameRng,
) -> String {
    if ritual.is_complete() {
        return "Ritual is already complete!".to_string();
    }

    // Check for disruption
    if rng.percent(disruption_chance as u32) {
        ritual.interrupt();
        return format!(
            "The ritual is disrupted! {} % of mana is wasted.",
            ritual.ritual_type.interruption_risk()
        );
    }

    ritual.advance_turn();

    if ritual.is_complete() {
        format!("Ritual {} is complete!", ritual.ritual_type.name())
    } else {
        format!(
            "Continuing ritual {}... ({}/{})",
            ritual.ritual_type.name(),
            ritual.turns_completed,
            ritual.ritual_type.turns_required()
        )
    }
}

/// Complete a ritual and generate its effects
pub fn complete_ritual(ritual: &RitualProgress) -> RitualEffect {
    if !ritual.is_complete() || ritual.interrupted {
        return RitualEffect::Failed("Ritual was not completed or was interrupted.".to_string());
    }

    match ritual.ritual_type {
        RitualSpellType::GreaterSummoning => RitualEffect::Summoning {
            creature_count: 3,
            message: "Three powerful creatures materialize!".to_string(),
        },
        RitualSpellType::MassHealing => RitualEffect::Healing {
            heal_amount: 100,
            radius: 10,
            message: "Holy light bathes the area, healing all!".to_string(),
        },
        RitualSpellType::Resurrection => RitualEffect::Resurrection {
            message: "A creature is returned to life!".to_string(),
        },
        RitualSpellType::TimeStop => RitualEffect::TimeStop {
            duration: 3,
            message: "Time itself seems to freeze around you!".to_string(),
        },
        RitualSpellType::Wish => RitualEffect::Wish {
            message: "You feel the universe bending to your will...".to_string(),
        },
        RitualSpellType::MeteorSwarm => RitualEffect::Destruction {
            damage: 150,
            radius: 8,
            message: "Meteors rain down on the battlefield!".to_string(),
        },
        RitualSpellType::HolyNuking => RitualEffect::Destruction {
            damage: 200,
            radius: 10,
            message: "Divine fury strikes down all undead!".to_string(),
        },
        RitualSpellType::DominateCreature => RitualEffect::Domination {
            message: "A creature bends to your will!".to_string(),
        },
    }
}

/// Effects produced by completing a ritual
#[derive(Debug, Clone)]
pub enum RitualEffect {
    /// Ritual failed
    Failed(String),
    /// Summon creatures
    Summoning {
        creature_count: i32,
        message: String,
    },
    /// Heal area
    Healing {
        heal_amount: i32,
        radius: i32,
        message: String,
    },
    /// Resurrect creature
    Resurrection { message: String },
    /// Stop time
    TimeStop { duration: u32, message: String },
    /// Grant a wish
    Wish { message: String },
    /// Massive damage area
    Destruction {
        damage: i32,
        radius: i32,
        message: String,
    },
    /// Dominate a creature
    Domination { message: String },
}

impl RitualEffect {
    /// Get the effect message
    pub fn message(&self) -> &str {
        match self {
            RitualEffect::Failed(msg) => msg,
            RitualEffect::Summoning { message, .. } => message,
            RitualEffect::Healing { message, .. } => message,
            RitualEffect::Resurrection { message } => message,
            RitualEffect::TimeStop { message, .. } => message,
            RitualEffect::Wish { message } => message,
            RitualEffect::Destruction { message, .. } => message,
            RitualEffect::Domination { message } => message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ritual_spell_type_names() {
        assert_eq!(
            RitualSpellType::GreaterSummoning.name(),
            "Greater Summoning"
        );
        assert_eq!(RitualSpellType::Wish.name(), "Wish");
    }

    #[test]
    fn test_ritual_spell_turns_required() {
        assert_eq!(RitualSpellType::MassHealing.turns_required(), 3);
        assert_eq!(RitualSpellType::Wish.turns_required(), 15);
    }

    #[test]
    fn test_ritual_progress_creation() {
        let progress = RitualProgress::new(RitualSpellType::MassHealing, 5, 10);
        assert_eq!(progress.ritual_type, RitualSpellType::MassHealing);
        assert_eq!(progress.turns_completed, 0);
        assert!(!progress.is_complete());
    }

    #[test]
    fn test_ritual_progress_completion() {
        let mut progress = RitualProgress::new(RitualSpellType::MassHealing, 5, 10);
        for _ in 0..3 {
            progress.advance_turn();
        }
        assert!(progress.is_complete());
    }

    #[test]
    fn test_ritual_progress_percent() {
        let mut progress = RitualProgress::new(RitualSpellType::MassHealing, 5, 10);
        assert_eq!(progress.percent_complete(), 0);

        progress.advance_turn();
        assert_eq!(progress.percent_complete(), 33); // 1/3

        progress.advance_turn();
        assert_eq!(progress.percent_complete(), 66); // 2/3

        progress.advance_turn();
        assert_eq!(progress.percent_complete(), 100); // 3/3
    }

    #[test]
    fn test_ritual_total_mana() {
        let total = RitualSpellType::MassHealing.total_mana_required();
        assert_eq!(total, 30 * 3);
    }

    #[test]
    fn test_ritual_tracker_creation() {
        let tracker = RitualTracker::new();
        assert!(!tracker.is_in_progress());
        assert!(tracker.active_ritual().is_none());
    }

    #[test]
    fn test_ritual_interrupt() {
        let mut progress = RitualProgress::new(RitualSpellType::MassHealing, 5, 10);
        progress.advance_turn();
        progress.interrupt();

        assert!(progress.interrupted);
        assert_eq!(progress.turns_completed, 1);
    }

    #[test]
    fn test_ritual_effect_messages() {
        let effect = RitualEffect::Summoning {
            creature_count: 3,
            message: "Test message".to_string(),
        };
        assert_eq!(effect.message(), "Test message");
    }
}
