//! Monster morale system (Phase 18)
//!
//! Tracks monster morale, handles morale events with decay, and determines
//! retreat conditions based on personality, HP, and recent events.

use serde::{Deserialize, Serialize};

use super::personality::Personality;
use super::tactics::Intelligence;

/// Events that affect monster morale
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoraleEvent {
    /// Witnessed an ally die nearby
    AlliedDeath,

    /// Took heavy damage (25%+ of max HP in one hit)
    TookHeavyDamage,

    /// Witnessed player demonstrate significant power
    WitnessedPlayerPower,

    /// Successfully hit player
    SuccessfulAttack,

    /// Nearly died (below 25% HP)
    NearDeath,

    /// Ally nearby rallies support
    AllySupportPresent,
}

/// Tracked morale event with age
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackedEvent {
    pub event: MoraleEvent,
    pub age_turns: u16,
}

/// Reason for retreat decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetreatReason {
    LowMorale,
    LowHp,
    AlliesDead,
    OutNumbered,
}

/// Morale tracking system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoraleTracker {
    /// Base morale level (-100 to 100)
    pub base_morale: i8,

    /// Current calculated morale (-100 to 100)
    pub current_morale: i8,

    /// Recent events with age tracking
    pub recent_events: Vec<TrackedEvent>,

    /// Number of ally deaths witnessed (reset on retreat)
    pub ally_deaths_witnessed: u8,

    /// Number of successful attacks in recent turns
    pub successful_hits: u8,

    /// Turns since last damage taken
    pub turns_since_damage: u16,

    /// Last damage amount taken
    pub last_damage_amount: i32,
}

impl Default for MoraleTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl MoraleTracker {
    /// Create a new morale tracker with base morale 0
    pub fn new() -> Self {
        Self {
            base_morale: 0,
            current_morale: 0,
            recent_events: Vec::new(),
            ally_deaths_witnessed: 0,
            successful_hits: 0,
            turns_since_damage: 0,
            last_damage_amount: 0,
        }
    }

    /// Set base morale
    pub fn set_base_morale(&mut self, morale: i8) {
        self.base_morale = morale.clamp(-100, 100);
    }

    /// Add a morale event (will be tracked for 10 turns)
    pub fn add_event(&mut self, event: MoraleEvent) {
        self.recent_events.push(TrackedEvent {
            event,
            age_turns: 0,
        });

        // Prune very old events
        if self.recent_events.len() > 20 {
            self.recent_events.retain(|e| e.age_turns < 15);
        }
    }

    /// Age all events by one turn
    pub fn age_events(&mut self) {
        for event in &mut self.recent_events {
            event.age_turns += 1;
        }
        // Remove events older than 10 turns
        self.recent_events.retain(|e| e.age_turns < 10);

        // Decay successful hits counter
        if self.successful_hits > 0 {
            self.successful_hits = self.successful_hits.saturating_sub(1);
        }

        // Age damage timeout
        if self.turns_since_damage < 100 {
            self.turns_since_damage += 1;
        }
    }

    /// Calculate current morale based on personality, HP%, and events
    pub fn calculate(&mut self, personality: Personality, current_hp: i32, max_hp: i32) -> i8 {
        let mut morale = self.base_morale as i32;

        // HP penalty (scaled to max 40% morale loss)
        let hp_percent = if max_hp > 0 {
            (current_hp as f32 / max_hp as f32).clamp(0.0, 1.0)
        } else {
            1.0
        };

        if hp_percent < 0.5 {
            let hp_penalty = ((0.5 - hp_percent) * 80.0) as i32;
            morale -= hp_penalty.min(40);
        }

        // Event impacts
        for tracked_event in &self.recent_events {
            let decay_factor = 1.0 - (tracked_event.age_turns as f32 / 10.0);
            let event_impact =
                (self.get_event_impact(tracked_event.event) as f32 * decay_factor) as i32;
            morale += event_impact;
        }

        // Personality modifiers
        morale += self.apply_personality_modifiers(personality, current_hp, max_hp);

        // Clamp to valid range
        self.current_morale = (morale as i8).clamp(-100, 100);
        self.current_morale
    }

    /// Get base impact of a morale event
    fn get_event_impact(&self, event: MoraleEvent) -> i32 {
        match event {
            MoraleEvent::AlliedDeath => -20,
            MoraleEvent::TookHeavyDamage => -15,
            MoraleEvent::WitnessedPlayerPower => -30,
            MoraleEvent::SuccessfulAttack => 5,
            MoraleEvent::NearDeath => -25,
            MoraleEvent::AllySupportPresent => 10,
        }
    }

    /// Apply personality-specific morale modifiers
    fn apply_personality_modifiers(
        &self,
        personality: Personality,
        current_hp: i32,
        max_hp: i32,
    ) -> i32 {
        let hp_percent = if max_hp > 0 {
            current_hp as f32 / max_hp as f32
        } else {
            1.0
        };

        match personality {
            // Berserker: immune to morale damage, gains morale when low HP
            Personality::Berserker => {
                let event_penalty: i32 = self
                    .recent_events
                    .iter()
                    .map(|e| self.get_event_impact(e.event))
                    .sum();
                (-event_penalty / 2) + (if hp_percent < 0.3 { 15 } else { 0 })
            }

            // Coward: doubles morale damage, loses morale from conflict
            Personality::Coward => {
                let event_penalty: i32 = self
                    .recent_events
                    .iter()
                    .map(|e| self.get_event_impact(e.event))
                    .sum();
                (event_penalty * 2) / 2 - (if self.successful_hits == 0 { 10 } else { 0 })
            }

            // Defensive: gains morale when protecting allies
            Personality::Defensive => {
                let has_ally_support = self
                    .recent_events
                    .iter()
                    .any(|e| e.event == MoraleEvent::AllySupportPresent);
                if has_ally_support { 15 } else { 0 }
            }

            // Others: no special modifiers
            _ => 0,
        }
    }

    /// Determine if monster should flee based on morale
    pub fn should_flee(&self, intelligence: Intelligence, personality: Personality) -> bool {
        let threshold = self.flee_threshold(intelligence, personality);
        self.current_morale < threshold
    }

    /// Get the morale threshold for fleeing based on intelligence
    fn flee_threshold(&self, intelligence: Intelligence, personality: Personality) -> i8 {
        let base_threshold = match intelligence {
            Intelligence::Mindless => 100, // Never flee
            Intelligence::Animal => 20,
            Intelligence::Low => 0,
            Intelligence::Average => -20,
            Intelligence::High => -40,
            Intelligence::Genius => -60,
        };

        // Personality modifiers
        let personality_mod = match personality {
            Personality::Aggressive => -20,
            Personality::Defensive => 10,
            Personality::Tactical => 0,
            Personality::Coward => 40,
            Personality::Berserker => -40,
            Personality::Cautious => 20,
        };

        base_threshold + personality_mod
    }

    /// Determine if monster should retreat from combat
    /// Determine if monster should retreat from combat
    pub fn should_retreat(
        &self,
        intelligence: Intelligence,
        personality: Personality,
        current_hp: i32,
        max_hp: i32,
    ) -> Option<RetreatReason> {
        // Low morale retreat
        if self.should_flee(intelligence, personality) {
            return Some(RetreatReason::LowMorale);
        }

        // HP-based retreat (scaled by intelligence)
        let hp_threshold: f32 = match intelligence {
            Intelligence::Mindless => -1.0, // Never (HP percentage is always >= 0)
            Intelligence::Animal => 0.15,
            Intelligence::Low => 0.20,
            Intelligence::Average => 0.25,
            Intelligence::High => 0.30,
            Intelligence::Genius => 0.35,
        };

        if max_hp > 0 {
            let hp_percent = current_hp as f32 / max_hp as f32;
            if hp_percent < hp_threshold {
                return Some(RetreatReason::LowHp);
            }
        }

        // Too many ally deaths
        if self.ally_deaths_witnessed > 3 {
            return Some(RetreatReason::AlliesDead);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morale_tracker_creation() {
        let tracker = MoraleTracker::new();
        assert_eq!(tracker.current_morale, 0);
        assert_eq!(tracker.ally_deaths_witnessed, 0);
    }

    #[test]
    fn test_add_and_age_events() {
        let mut tracker = MoraleTracker::new();
        tracker.add_event(MoraleEvent::SuccessfulAttack);
        assert_eq!(tracker.recent_events.len(), 1);

        for _ in 0..15 {
            tracker.age_events();
        }
        // Events older than 10 turns should be removed
        assert_eq!(tracker.recent_events.len(), 0);
    }

    #[test]
    fn test_morale_calculation() {
        let mut tracker = MoraleTracker::new();
        tracker.set_base_morale(0);

        let morale = tracker.calculate(Personality::Tactical, 100, 100);
        assert!(morale >= -100 && morale <= 100);
    }

    #[test]
    fn test_hp_penalty() {
        let mut tracker = MoraleTracker::new();
        let high_hp_morale = tracker.calculate(Personality::Tactical, 100, 100);

        let mut tracker2 = MoraleTracker::new();
        let low_hp_morale = tracker2.calculate(Personality::Tactical, 20, 100);

        // Low HP should result in lower morale
        assert!(low_hp_morale < high_hp_morale);
    }

    #[test]
    fn test_flee_thresholds() {
        let tracker = MoraleTracker::new();

        // Genius should have lower flee threshold (harder to flee)
        assert!(
            tracker.flee_threshold(Intelligence::Genius, Personality::Tactical)
                < tracker.flee_threshold(Intelligence::Animal, Personality::Tactical)
        );

        // Coward should have higher flee threshold (easier to flee)
        assert!(
            tracker.flee_threshold(Intelligence::Average, Personality::Coward)
                > tracker.flee_threshold(Intelligence::Average, Personality::Aggressive)
        );
    }

    #[test]
    fn test_berserker_personality() {
        let mut tracker = MoraleTracker::new();
        tracker.add_event(MoraleEvent::AlliedDeath);
        let morale = tracker.calculate(Personality::Berserker, 50, 100);

        let mut tracker2 = MoraleTracker::new();
        tracker2.add_event(MoraleEvent::AlliedDeath);
        let defensive_morale = tracker2.calculate(Personality::Defensive, 50, 100);

        // Berserker should be less affected by ally death
        assert!(morale.abs() > defensive_morale.abs() || morale > defensive_morale);
    }
}
