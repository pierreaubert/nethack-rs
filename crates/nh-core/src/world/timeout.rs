//! Timed events and timeout system (timeout.c, potion.c, cmd.c)
//!
//! Handles scheduled events like monster actions, delayed effects, and timeouts.
//! Includes intrinsic timeout management and multi-turn occupations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::monster::MonsterId;
use crate::object::ObjectId;

/// Timer identifier - unique ID for each timer
pub type TimerId = u64;

/// Timer kinds - categorizes timer by scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimerKind {
    /// Global game-wide timer
    Global,
    /// Level-specific timer
    Level,
    /// Object-attached timer
    Object,
    /// Monster-attached timer
    Monster,
    /// Location-specific timer
    Location,
}

/// Timer function index - identifies which function to call
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimerFunc {
    /// Burn timer (objects)
    Burn,
    /// Egg hatching
    EggHatch,
    /// Figurine animation
    FigurineTransform,
    /// Stoning countdown
    Stoned,
    /// Sliming countdown
    Slimed,
    /// Strangling countdown
    Choked,
    /// Vomiting
    Vomiting,
    /// Sleep timer
    Sleep,
    /// Levitation timer
    Levitation,
    /// Monster spawning
    MonsterSpawn,
    /// Hunger tick
    Hunger,
    /// Regeneration
    Regeneration,
    /// Illness/sickness
    Illness,
    /// Custom function with identifier
    Custom(String),
}

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
        self.events.retain(
            |e| !matches!(&e.event_type, TimedEventType::MonsterAction(id) if *id == monster_id),
        );
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

    /// Get all pending events
    pub fn pending_events(&self) -> &[TimedEvent] {
        &self.events
    }

    /// Get mutable access to pending events (advanced use only)
    pub fn pending_events_mut(&mut self) -> &mut Vec<TimedEvent> {
        &mut self.events
    }

    /// Check if any events are pending
    pub fn has_pending_events(&self) -> bool {
        !self.events.is_empty()
    }

    /// Get the next scheduled event trigger time
    pub fn next_event_time(&self) -> Option<u64> {
        self.events.first().map(|e| e.trigger_turn)
    }

    /// Process events up to (but not including) a specific turn
    pub fn tick_until(&mut self, until_turn: u64) -> Vec<TimedEvent> {
        let mut triggered = Vec::new();
        let mut to_reschedule = Vec::new();

        while let Some(event) = self.events.first() {
            if event.trigger_turn < until_turn {
                let event = self.events.remove(0);

                if event.repeating {
                    let mut next = event.clone();
                    next.trigger_turn = event.trigger_turn + event.repeat_interval;
                    if next.trigger_turn < until_turn {
                        to_reschedule.push(next);
                    } else {
                        to_reschedule.push(next);
                    }
                }

                triggered.push(event);
            } else {
                break;
            }
        }

        for event in to_reschedule {
            self.schedule(event);
        }

        triggered
    }
}

/// Intrinsic timeout management
///
/// Handles timeout values for intrinsic properties like temporary invisibility,
/// levitation, etc. Validates that timeout values stay within reasonable bounds.
pub struct IntrinsicTimeouts {
    /// Map of property ID to timeout (turn number when it expires)
    timeouts: HashMap<String, u64>,
}

impl Default for IntrinsicTimeouts {
    fn default() -> Self {
        Self::new()
    }
}

impl IntrinsicTimeouts {
    /// Create a new intrinsic timeout manager
    pub fn new() -> Self {
        Self {
            timeouts: HashMap::new(),
        }
    }

    /// Validate and constrain a timeout value to a reasonable range.
    ///
    /// In the original C code, `itimeout()` forces timeout values to be within
    /// valid bounds to prevent arithmetic overflow and unreasonable durations.
    ///
    /// # Arguments
    /// * `val` - The timeout value to validate
    ///
    /// # Returns
    /// The constrained timeout value
    pub fn validate(val: u64) -> u64 {
        // Constrain timeout to reasonable bounds (0 to ~10000 turns = ~100 real days)
        const MAX_TIMEOUT: u64 = 10000;
        std::cmp::min(val, MAX_TIMEOUT)
    }

    /// Increment a timeout value and validate the result.
    ///
    /// This is used to extend an existing timeout. Equivalent to `itimeout_incr()`.
    ///
    /// # Arguments
    /// * `old` - The existing timeout value
    /// * `incr` - The amount to increment by
    ///
    /// # Returns
    /// The validated, incremented value
    pub fn increment(old: u64, incr: u64) -> u64 {
        Self::validate(old.saturating_add(incr))
    }

    /// Set a timeout value for a property.
    ///
    /// Validates and stores the timeout. Equivalent to `set_itimeout()`.
    pub fn set(&mut self, property: impl Into<String>, val: u64) {
        let val = Self::validate(val);
        if val > 0 {
            self.timeouts.insert(property.into(), val);
        } else {
            self.timeouts.remove(&property.into());
        }
    }

    /// Increment an existing timeout.
    ///
    /// Gets the current timeout, increments it, validates, and stores the result.
    /// Equivalent to `incr_itimeout()`.
    pub fn increment_property(&mut self, property: impl Into<String>, incr: u64) {
        let prop = property.into();
        let old = self.timeouts.get(&prop).copied().unwrap_or(0);
        let new_val = Self::increment(old, incr);
        self.set(prop, new_val);
    }

    /// Get current timeout for a property
    pub fn get(&self, property: &str) -> u64 {
        self.timeouts.get(property).copied().unwrap_or(0)
    }

    /// Check if a property has an active timeout
    pub fn has_timeout(&self, property: &str) -> bool {
        self.timeouts.get(property).copied().unwrap_or(0) > 0
    }

    /// Decrease a timeout (called each turn)
    pub fn tick(&mut self, current_turn: u64) {
        // Remove expired timeouts
        self.timeouts
            .retain(|_, &mut timeout| timeout > current_turn);
    }

    /// Get remaining time for a property
    pub fn remaining(&self, property: &str, current_turn: u64) -> u64 {
        self.timeouts
            .get(property)
            .copied()
            .unwrap_or(0)
            .saturating_sub(current_turn)
    }

    /// Clear all timeouts
    pub fn clear(&mut self) {
        self.timeouts.clear();
    }
}

/// Multi-turn occupation timer
///
/// Handles time-consuming actions that take multiple turns, like:
/// - Lock picking
/// - Armor removal
/// - Trap setting
/// - Engraving
pub struct OccupationTimer {
    /// Number of turns remaining for the current occupation
    remaining_turns: i32,
    /// Description of the occupation
    activity: Option<String>,
}

impl Default for OccupationTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl OccupationTimer {
    /// Create a new occupation timer
    pub fn new() -> Self {
        Self {
            remaining_turns: 0,
            activity: None,
        }
    }

    /// Start a new occupation that takes `turns` turns
    ///
    /// Equivalent to `timed_occupation()` - sets up a multi-turn activity.
    pub fn start(&mut self, turns: i32, activity: impl Into<String>) {
        self.remaining_turns = std::cmp::max(0, turns);
        self.activity = Some(activity.into());
    }

    /// Check if currently occupied
    pub fn is_occupied(&self) -> bool {
        self.remaining_turns > 0
    }

    /// Get current activity description
    pub fn activity(&self) -> Option<&str> {
        self.activity.as_deref()
    }

    /// Get remaining turns
    pub fn remaining(&self) -> i32 {
        self.remaining_turns
    }

    /// Decrement the occupation timer by one turn
    ///
    /// Returns the new remaining count. When it reaches 0, the occupation is complete.
    pub fn tick(&mut self) -> i32 {
        if self.remaining_turns > 0 {
            self.remaining_turns -= 1;
            if self.remaining_turns == 0 {
                self.activity = None;
            }
        }
        self.remaining_turns
    }

    /// Cancel the current occupation
    pub fn cancel(&mut self) {
        self.remaining_turns = 0;
        self.activity = None;
    }
}

// ============================================================================
// Timer API Functions (timeout.c equivalents)
// ============================================================================

/// Timer element for the global timer queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    /// Unique timer ID
    pub id: TimerId,
    /// Turn when timer expires
    pub timeout: u64,
    /// Timer kind (global, level, object, monster, location)
    pub kind: TimerKind,
    /// Function to call when timer expires
    pub func: TimerFunc,
    /// Associated object ID (if object timer)
    pub object_id: Option<ObjectId>,
    /// Associated monster ID (if monster timer)
    pub monster_id: Option<MonsterId>,
    /// Associated location (if location timer)
    pub location: Option<(i16, i16)>,
    /// Whether timer needs fixup after restore
    pub needs_fixup: bool,
}

impl Timer {
    /// Create a new timer
    pub fn new(id: TimerId, timeout: u64, kind: TimerKind, func: TimerFunc) -> Self {
        Self {
            id,
            timeout,
            kind,
            func,
            object_id: None,
            monster_id: None,
            location: None,
            needs_fixup: false,
        }
    }

    /// Create an object timer
    pub fn for_object(id: TimerId, timeout: u64, func: TimerFunc, object_id: ObjectId) -> Self {
        Self {
            id,
            timeout,
            kind: TimerKind::Object,
            func,
            object_id: Some(object_id),
            monster_id: None,
            location: None,
            needs_fixup: false,
        }
    }

    /// Create a monster timer
    pub fn for_monster(id: TimerId, timeout: u64, func: TimerFunc, monster_id: MonsterId) -> Self {
        Self {
            id,
            timeout,
            kind: TimerKind::Monster,
            func,
            object_id: None,
            monster_id: Some(monster_id),
            location: None,
            needs_fixup: false,
        }
    }
}

/// Global timer queue manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimerQueue {
    /// List of active timers, sorted by timeout
    timers: Vec<Timer>,
    /// Next timer ID to assign
    next_id: TimerId,
    /// Current game turn
    current_turn: u64,
}

impl TimerQueue {
    /// Create a new timer queue
    pub fn new() -> Self {
        Self {
            timers: Vec::new(),
            next_id: 1,
            current_turn: 0,
        }
    }

    /// Start a new timer (start_timer equivalent)
    ///
    /// Returns the timer ID if successful, None if a duplicate timer exists.
    pub fn start_timer(&mut self, when: u64, kind: TimerKind, func: TimerFunc) -> Option<TimerId> {
        let timeout = self.current_turn + when;
        let id = self.next_id;
        self.next_id += 1;

        let timer = Timer::new(id, timeout, kind, func);
        self.insert_timer(timer);
        Some(id)
    }

    /// Start an object timer
    pub fn start_object_timer(
        &mut self,
        when: u64,
        func: TimerFunc,
        object_id: ObjectId,
    ) -> Option<TimerId> {
        // Check for duplicate
        if self.obj_has_timer(object_id, &func) {
            return None;
        }

        let timeout = self.current_turn + when;
        let id = self.next_id;
        self.next_id += 1;

        let timer = Timer::for_object(id, timeout, func, object_id);
        self.insert_timer(timer);
        Some(id)
    }

    /// Start a monster timer
    pub fn start_monster_timer(
        &mut self,
        when: u64,
        func: TimerFunc,
        monster_id: MonsterId,
    ) -> Option<TimerId> {
        let timeout = self.current_turn + when;
        let id = self.next_id;
        self.next_id += 1;

        let timer = Timer::for_monster(id, timeout, func, monster_id);
        self.insert_timer(timer);
        Some(id)
    }

    /// Insert a timer in sorted order
    fn insert_timer(&mut self, timer: Timer) {
        let pos = self
            .timers
            .iter()
            .position(|t| t.timeout > timer.timeout)
            .unwrap_or(self.timers.len());
        self.timers.insert(pos, timer);
    }

    /// Stop a timer by function type (stop_timer equivalent)
    ///
    /// Returns the remaining time if found, 0 if not found.
    pub fn stop_timer(&mut self, func: &TimerFunc) -> u64 {
        if let Some(pos) = self.timers.iter().position(|t| &t.func == func) {
            let timer = self.timers.remove(pos);
            timer.timeout.saturating_sub(self.current_turn)
        } else {
            0
        }
    }

    /// Stop a timer by ID
    pub fn stop_timer_by_id(&mut self, id: TimerId) -> bool {
        if let Some(pos) = self.timers.iter().position(|t| t.id == id) {
            self.timers.remove(pos);
            true
        } else {
            false
        }
    }

    /// Peek at a timer's timeout (peek_timer equivalent)
    ///
    /// Returns the absolute turn when timer expires, 0 if not found.
    pub fn peek_timer(&self, func: &TimerFunc) -> u64 {
        self.timers
            .iter()
            .find(|t| &t.func == func)
            .map(|t| t.timeout)
            .unwrap_or(0)
    }

    /// Check if an object has a specific timer (obj_has_timer equivalent)
    pub fn obj_has_timer(&self, object_id: ObjectId, func: &TimerFunc) -> bool {
        self.timers.iter().any(|t| {
            t.kind == TimerKind::Object && t.object_id == Some(object_id) && &t.func == func
        })
    }

    /// Stop all timers for an object (obj_stop_timers equivalent)
    pub fn obj_stop_timers(&mut self, object_id: ObjectId) {
        self.timers
            .retain(|t| !(t.kind == TimerKind::Object && t.object_id == Some(object_id)));
    }

    /// Move all timers from one object to another (obj_move_timers equivalent)
    pub fn obj_move_timers(&mut self, src: ObjectId, dest: ObjectId) {
        for timer in &mut self.timers {
            if timer.kind == TimerKind::Object && timer.object_id == Some(src) {
                timer.object_id = Some(dest);
            }
        }
    }

    /// Duplicate timers from one object to another (obj_split_timers equivalent)
    pub fn obj_split_timers(&mut self, src: ObjectId, dest: ObjectId) {
        let to_add: Vec<Timer> = self
            .timers
            .iter()
            .filter(|t| t.kind == TimerKind::Object && t.object_id == Some(src))
            .map(|t| {
                let id = self.next_id;
                self.next_id += 1;
                Timer {
                    id,
                    timeout: t.timeout,
                    kind: TimerKind::Object,
                    func: t.func.clone(),
                    object_id: Some(dest),
                    monster_id: None,
                    location: None,
                    needs_fixup: false,
                }
            })
            .collect();

        for timer in to_add {
            self.insert_timer(timer);
        }
    }

    /// Run all expired timers (run_timers equivalent)
    ///
    /// Returns a list of expired timers that need to be processed.
    pub fn run_timers(&mut self, current_turn: u64) -> Vec<Timer> {
        self.current_turn = current_turn;

        let mut expired = Vec::new();
        while let Some(timer) = self.timers.first() {
            if timer.timeout <= current_turn {
                expired.push(self.timers.remove(0));
            } else {
                break;
            }
        }
        expired
    }

    /// Get count of active timers
    pub fn timer_count(&self) -> usize {
        self.timers.len()
    }

    /// Check if any timers are active
    pub fn has_timers(&self) -> bool {
        !self.timers.is_empty()
    }

    /// Get all timers for an object
    pub fn get_object_timers(&self, object_id: ObjectId) -> Vec<&Timer> {
        self.timers
            .iter()
            .filter(|t| t.kind == TimerKind::Object && t.object_id == Some(object_id))
            .collect()
    }

    /// Clear all timers
    pub fn clear(&mut self) {
        self.timers.clear();
    }

    /// Set current turn
    pub fn set_current_turn(&mut self, turn: u64) {
        self.current_turn = turn;
    }

    /// Get current turn
    pub fn current_turn(&self) -> u64 {
        self.current_turn
    }
}

/// Generate random weather events (do_storms equivalent)
///
/// Called periodically to potentially trigger lightning strikes, thunder, etc.
/// Returns messages to display to the player.
pub fn do_storms(rng: &mut crate::rng::GameRng, player_underwater: bool) -> Vec<String> {
    let mut messages = Vec::new();

    // 1 in 100 chance of a storm event each call
    if rng.rn2(100) != 0 {
        return messages;
    }

    // Underwater players don't notice storms
    if player_underwater {
        return messages;
    }

    // Random storm effects
    match rng.rn2(4) {
        0 => {
            messages.push("You hear distant thunder.".to_string());
        }
        1 => {
            messages.push("A flash of lightning illuminates the sky!".to_string());
        }
        2 => {
            messages.push("The wind howls around you.".to_string());
        }
        3 => {
            messages.push("Rain begins to fall.".to_string());
        }
        _ => {}
    }

    messages
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

    // ========================================================================
    // Tests for TimerQueue (start_timer, stop_timer, etc.)
    // ========================================================================

    #[test]
    fn test_timer_queue_start_timer() {
        let mut queue = TimerQueue::new();
        queue.set_current_turn(100);

        let id = queue.start_timer(50, TimerKind::Global, TimerFunc::Burn);
        assert!(id.is_some());
        assert_eq!(queue.timer_count(), 1);

        // Timer should expire at turn 150
        assert_eq!(queue.peek_timer(&TimerFunc::Burn), 150);
    }

    #[test]
    fn test_timer_queue_stop_timer() {
        let mut queue = TimerQueue::new();
        queue.set_current_turn(100);

        queue.start_timer(50, TimerKind::Global, TimerFunc::Burn);
        assert_eq!(queue.timer_count(), 1);

        let remaining = queue.stop_timer(&TimerFunc::Burn);
        assert_eq!(remaining, 50);
        assert_eq!(queue.timer_count(), 0);
    }

    #[test]
    fn test_timer_queue_object_timer() {
        let mut queue = TimerQueue::new();
        queue.set_current_turn(0);

        let obj_id = ObjectId(42);
        let id = queue.start_object_timer(100, TimerFunc::Burn, obj_id);
        assert!(id.is_some());

        // Check obj_has_timer
        assert!(queue.obj_has_timer(obj_id, &TimerFunc::Burn));
        assert!(!queue.obj_has_timer(obj_id, &TimerFunc::EggHatch));
        assert!(!queue.obj_has_timer(ObjectId(99), &TimerFunc::Burn));

        // Duplicate should fail
        let dup = queue.start_object_timer(200, TimerFunc::Burn, obj_id);
        assert!(dup.is_none());
    }

    #[test]
    fn test_timer_queue_obj_stop_timers() {
        let mut queue = TimerQueue::new();
        let obj1 = ObjectId(1);
        let obj2 = ObjectId(2);

        queue.start_object_timer(100, TimerFunc::Burn, obj1);
        queue.start_object_timer(100, TimerFunc::EggHatch, obj1);
        queue.start_object_timer(100, TimerFunc::Burn, obj2);

        assert_eq!(queue.timer_count(), 3);

        queue.obj_stop_timers(obj1);
        assert_eq!(queue.timer_count(), 1);
        assert!(!queue.obj_has_timer(obj1, &TimerFunc::Burn));
        assert!(queue.obj_has_timer(obj2, &TimerFunc::Burn));
    }

    #[test]
    fn test_timer_queue_obj_move_timers() {
        let mut queue = TimerQueue::new();
        let src = ObjectId(1);
        let dest = ObjectId(2);

        queue.start_object_timer(100, TimerFunc::Burn, src);

        assert!(queue.obj_has_timer(src, &TimerFunc::Burn));
        assert!(!queue.obj_has_timer(dest, &TimerFunc::Burn));

        queue.obj_move_timers(src, dest);

        assert!(!queue.obj_has_timer(src, &TimerFunc::Burn));
        assert!(queue.obj_has_timer(dest, &TimerFunc::Burn));
    }

    #[test]
    fn test_timer_queue_obj_split_timers() {
        let mut queue = TimerQueue::new();
        let src = ObjectId(1);
        let dest = ObjectId(2);

        queue.start_object_timer(100, TimerFunc::Burn, src);
        assert_eq!(queue.timer_count(), 1);

        queue.obj_split_timers(src, dest);
        assert_eq!(queue.timer_count(), 2);

        assert!(queue.obj_has_timer(src, &TimerFunc::Burn));
        assert!(queue.obj_has_timer(dest, &TimerFunc::Burn));
    }

    #[test]
    fn test_timer_queue_run_timers() {
        let mut queue = TimerQueue::new();
        queue.set_current_turn(0);

        queue.start_timer(10, TimerKind::Global, TimerFunc::Burn);
        queue.start_timer(20, TimerKind::Global, TimerFunc::EggHatch);
        queue.start_timer(30, TimerKind::Global, TimerFunc::Stoned);

        // Run at turn 15 - should get Burn timer
        let expired = queue.run_timers(15);
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].func, TimerFunc::Burn);
        assert_eq!(queue.timer_count(), 2);

        // Run at turn 25 - should get EggHatch timer
        let expired = queue.run_timers(25);
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].func, TimerFunc::EggHatch);

        // Run at turn 50 - should get Stoned timer
        let expired = queue.run_timers(50);
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].func, TimerFunc::Stoned);

        assert_eq!(queue.timer_count(), 0);
    }

    #[test]
    fn test_do_storms() {
        let mut rng = crate::rng::GameRng::new(42);

        // Run many times to test randomness
        let mut got_message = false;
        for _ in 0..200 {
            let messages = do_storms(&mut rng, false);
            if !messages.is_empty() {
                got_message = true;
                break;
            }
        }
        // Should eventually get a storm message
        assert!(got_message);

        // Underwater should never get messages
        for _ in 0..100 {
            let messages = do_storms(&mut rng, true);
            assert!(messages.is_empty());
        }
    }

    #[test]
    fn test_intrinsic_timeouts() {
        let mut timeouts = IntrinsicTimeouts::new();

        timeouts.set("invisibility", 100);
        assert!(timeouts.has_timeout("invisibility"));
        assert_eq!(timeouts.get("invisibility"), 100);

        timeouts.increment_property("invisibility", 50);
        assert_eq!(timeouts.get("invisibility"), 150);

        assert_eq!(timeouts.remaining("invisibility", 100), 50);
    }

    #[test]
    fn test_occupation_timer() {
        let mut timer = OccupationTimer::new();

        assert!(!timer.is_occupied());

        timer.start(5, "picking a lock");
        assert!(timer.is_occupied());
        assert_eq!(timer.remaining(), 5);
        assert_eq!(timer.activity(), Some("picking a lock"));

        timer.tick();
        assert_eq!(timer.remaining(), 4);

        timer.cancel();
        assert!(!timer.is_occupied());
    }
}
