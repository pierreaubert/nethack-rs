//! Timed events and timeout system
//!
//! Handles scheduled events like monster actions, delayed effects, and timeouts.

use serde::{Deserialize, Serialize};

use crate::monster::MonsterId;
use crate::object::ObjectId;

/// Types of timed events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimedEventType {
    /// Monster spawning
    MonsterSpawn,
    /// Monster special action (e.g., dragon breath cooldown)
    MonsterAction(MonsterId),
    /// Object effect (e.g., lamp running out)
    ObjectTimeout(ObjectId),
    /// Corpse rotting
    CorpseRot(ObjectId),
    /// Egg hatching
    EggHatch(ObjectId),
    /// Figurine animation
    FigurineAnimate(ObjectId),
    /// Delayed instadeath (e.g., illness)
    DelayedDeath(String),
    /// Stoning countdown
    Stoning,
    /// Sliming countdown
    Sliming,
    /// Strangling countdown
    Strangling,
    /// Vomiting
    Vomiting,
    /// Regeneration tick
    Regeneration,
    /// Energy regeneration tick
    EnergyRegeneration,
    /// Hunger tick
    Hunger,
    /// Blindness from cream pie
    BlindFromCreamPie,
    /// Temporary see invisible
    TempSeeInvisible,
    /// Temporary telepathy
    TempTelepathy,
    /// Temporary warning
    TempWarning,
    /// Temporary stealth
    TempStealth,
    /// Temporary levitation
    TempLevitation,
    /// Temporary flying
    TempFlying,
    /// Custom event with identifier
    Custom(String),
}

/// A scheduled timed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedEvent {
    /// When the event triggers (turn number)
    pub trigger_turn: u64,
    /// Type of event
    pub event_type: TimedEventType,
    /// Optional data associated with the event
    pub data: Option<i32>,
    /// Whether this event repeats
    pub repeating: bool,
    /// Repeat interval (if repeating)
    pub repeat_interval: u64,
}

impl TimedEvent {
    /// Create a new one-shot timed event
    pub fn new(trigger_turn: u64, event_type: TimedEventType) -> Self {
        Self {
            trigger_turn,
            event_type,
            data: None,
            repeating: false,
            repeat_interval: 0,
        }
    }

    /// Create a new repeating timed event
    pub fn repeating(trigger_turn: u64, event_type: TimedEventType, interval: u64) -> Self {
        Self {
            trigger_turn,
            event_type,
            data: None,
            repeating: true,
            repeat_interval: interval,
        }
    }

    /// Add data to the event
    pub fn with_data(mut self, data: i32) -> Self {
        self.data = Some(data);
        self
    }

    /// Check if event should trigger at the given turn
    pub const fn should_trigger(&self, current_turn: u64) -> bool {
        current_turn >= self.trigger_turn
    }
}

/// Manager for timed events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimeoutManager {
    /// Scheduled events, sorted by trigger time
    events: Vec<TimedEvent>,
    /// Current turn for reference
    current_turn: u64,
}

impl TimeoutManager {
    /// Create a new timeout manager
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            current_turn: 0,
        }
    }

    /// Schedule a new event
    pub fn schedule(&mut self, event: TimedEvent) {
        // Insert in sorted order by trigger_turn
        let pos = self
            .events
            .iter()
            .position(|e| e.trigger_turn > event.trigger_turn)
            .unwrap_or(self.events.len());
        self.events.insert(pos, event);
    }

    /// Schedule an event to trigger after a delay
    pub fn schedule_after(&mut self, delay: u64, event_type: TimedEventType) {
        let event = TimedEvent::new(self.current_turn + delay, event_type);
        self.schedule(event);
    }

    /// Schedule a repeating event
    pub fn schedule_repeating(&mut self, delay: u64, interval: u64, event_type: TimedEventType) {
        let event = TimedEvent::repeating(self.current_turn + delay, event_type, interval);
        self.schedule(event);
    }

    /// Cancel all events of a specific type
    pub fn cancel(&mut self, event_type: &TimedEventType) {
        self.events.retain(|e| &e.event_type != event_type);
    }

    /// Cancel events associated with a specific monster
    pub fn cancel_monster_events(&mut self, monster_id: MonsterId) {
        self.events.retain(|e| {
            !matches!(&e.event_type, TimedEventType::MonsterAction(id) if *id == monster_id)
        });
    }

    /// Cancel events associated with a specific object
    pub fn cancel_object_events(&mut self, object_id: ObjectId) {
        self.events.retain(|e| {
            !matches!(
                &e.event_type,
                TimedEventType::ObjectTimeout(id)
                    | TimedEventType::CorpseRot(id)
                    | TimedEventType::EggHatch(id)
                    | TimedEventType::FigurineAnimate(id)
                    if *id == object_id
            )
        });
    }

    /// Process events for the current turn, returning triggered events
    pub fn tick(&mut self, current_turn: u64) -> Vec<TimedEvent> {
        self.current_turn = current_turn;

        let mut triggered = Vec::new();
        let mut to_reschedule = Vec::new();

        // Collect triggered events
        while let Some(event) = self.events.first() {
            if event.should_trigger(current_turn) {
                let event = self.events.remove(0);

                if event.repeating {
                    // Schedule next occurrence
                    let mut next = event.clone();
                    next.trigger_turn = current_turn + event.repeat_interval;
                    to_reschedule.push(next);
                }

                triggered.push(event);
            } else {
                break;
            }
        }

        // Reschedule repeating events
        for event in to_reschedule {
            self.schedule(event);
        }

        triggered
    }

    /// Check if there's a pending event of a specific type
    pub fn has_pending(&self, event_type: &TimedEventType) -> bool {
        self.events.iter().any(|e| &e.event_type == event_type)
    }

    /// Get the turn when a specific event type will trigger (if scheduled)
    pub fn next_trigger(&self, event_type: &TimedEventType) -> Option<u64> {
        self.events
            .iter()
            .find(|e| &e.event_type == event_type)
            .map(|e| e.trigger_turn)
    }

    /// Get remaining turns until a specific event triggers
    pub fn turns_until(&self, event_type: &TimedEventType) -> Option<u64> {
        self.next_trigger(event_type)
            .map(|t| t.saturating_sub(self.current_turn))
    }

    /// Get count of pending events
    pub fn pending_count(&self) -> usize {
        self.events.len()
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_and_tick() {
        let mut manager = TimeoutManager::new();

        manager.schedule_after(5, TimedEventType::MonsterSpawn);
        manager.schedule_after(3, TimedEventType::Hunger);
        manager.schedule_after(10, TimedEventType::Regeneration);

        assert_eq!(manager.pending_count(), 3);

        // Tick to turn 3
        let triggered = manager.tick(3);
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0].event_type, TimedEventType::Hunger);

        // Tick to turn 5
        let triggered = manager.tick(5);
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0].event_type, TimedEventType::MonsterSpawn);

        // Tick to turn 10
        let triggered = manager.tick(10);
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0].event_type, TimedEventType::Regeneration);

        assert_eq!(manager.pending_count(), 0);
    }

    #[test]
    fn test_repeating_events() {
        let mut manager = TimeoutManager::new();

        manager.schedule_repeating(5, 10, TimedEventType::Regeneration);

        // First trigger at turn 5
        let triggered = manager.tick(5);
        assert_eq!(triggered.len(), 1);
        assert!(manager.has_pending(&TimedEventType::Regeneration));

        // Next trigger at turn 15
        assert_eq!(manager.tick(10).len(), 0);
        assert_eq!(manager.tick(15).len(), 1);
    }

    #[test]
    fn test_cancel_events() {
        let mut manager = TimeoutManager::new();

        manager.schedule_after(5, TimedEventType::MonsterSpawn);
        manager.schedule_after(5, TimedEventType::Hunger);

        manager.cancel(&TimedEventType::MonsterSpawn);

        assert_eq!(manager.pending_count(), 1);
        assert!(!manager.has_pending(&TimedEventType::MonsterSpawn));
        assert!(manager.has_pending(&TimedEventType::Hunger));
    }

    #[test]
    fn test_turns_until() {
        let mut manager = TimeoutManager::new();
        manager.current_turn = 10;

        manager.schedule(TimedEvent::new(25, TimedEventType::Stoning));

        assert_eq!(manager.turns_until(&TimedEventType::Stoning), Some(15));
        assert_eq!(manager.turns_until(&TimedEventType::Sliming), None);
    }
}
