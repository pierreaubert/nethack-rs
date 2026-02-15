//! Phase 26: Timeout, Occupation, and Timed Effect System
//!
//! Behavioral tests verifying player timeout fields (stoning, sliming, illness),
//! object age tracking (corpses, lamps, eggs), multi-turn occupation state,
//! and wounded-legs / fumbling property mechanics.

use nh_core::object::{Object, ObjectClass};
use nh_core::player::{Property, You};
use nh_core::world::timeout::{
    OccupationTimer, TimedEvent, TimedEventType, TimeoutManager, TimerFunc, TimerQueue,
};

// ============================================================================
// Test 1: Player has stoning timeout field, defaults to zero
// ============================================================================

#[test]
fn test_stoning_timeout_field() {
    let player = You::default();

    // Both the u16 status-effect timeout and the i32 countdown field exist
    assert_eq!(player.stoning_timeout, 0, "stoning_timeout should default to 0");
    assert_eq!(player.stoning, 0, "stoning countdown should default to 0");

    // The TimeoutManager supports scheduling a Stoning event
    let mut tm = TimeoutManager::new();
    tm.schedule(TimedEvent::new(5, TimedEventType::Stoning));
    assert!(
        tm.has_pending(&TimedEventType::Stoning),
        "TimeoutManager should track pending Stoning events"
    );
}

// ============================================================================
// Test 2: Stoning countdown via TimeoutManager tick
// ============================================================================

#[test]
fn test_stoning_countdown() {
    let mut tm = TimeoutManager::new();

    // Schedule stoning to trigger at turn 5 (simulates 5-turn countdown)
    tm.schedule(TimedEvent::new(5, TimedEventType::Stoning));

    // Turns 1-4: stoning has not yet triggered
    for turn in 1..5 {
        let triggered = tm.tick(turn);
        assert!(
            triggered.is_empty(),
            "Stoning should NOT trigger at turn {turn}"
        );
        assert!(tm.has_pending(&TimedEventType::Stoning));
    }

    // Turn 5: stoning triggers (petrification would happen)
    let triggered = tm.tick(5);
    assert_eq!(triggered.len(), 1, "Stoning should trigger at turn 5");
    assert_eq!(triggered[0].event_type, TimedEventType::Stoning);
    assert!(
        !tm.has_pending(&TimedEventType::Stoning),
        "After triggering, no pending Stoning events remain"
    );
}

// ============================================================================
// Test 3: Player has sliming timeout field
// ============================================================================

#[test]
fn test_sliming_timeout_field() {
    let player = You::default();
    assert_eq!(player.sliming_timeout, 0, "sliming_timeout should default to 0");

    // TimeoutManager supports Sliming events
    let mut tm = TimeoutManager::new();
    tm.schedule(TimedEvent::new(10, TimedEventType::Sliming));
    assert!(tm.has_pending(&TimedEventType::Sliming));

    // Cancelling the Sliming event removes it
    tm.cancel(&TimedEventType::Sliming);
    assert!(
        !tm.has_pending(&TimedEventType::Sliming),
        "Cancelling Sliming should remove the event"
    );
}

// ============================================================================
// Test 4: Player has illness/sickness timeout fields
// ============================================================================

#[test]
fn test_illness_timeout_field() {
    let player = You::default();

    // Multiple illness-related fields exist
    assert_eq!(player.sickness_timeout, 0, "sickness_timeout defaults to 0");
    assert_eq!(player.sick_food_timeout, 0, "sick_food_timeout defaults to 0");
    assert_eq!(player.sick_illness_timeout, 0, "sick_illness_timeout defaults to 0");
    assert_eq!(player.sick, 0, "sick countdown defaults to 0");
    assert!(player.sick_reason.is_none(), "sick_reason defaults to None");

    // TimerFunc::Illness exists for the timer queue
    let mut queue = TimerQueue::new();
    let id = queue.start_timer(100, nh_core::world::timeout::TimerKind::Global, TimerFunc::Illness);
    assert!(id.is_some(), "Should be able to create an Illness timer");
    assert_eq!(queue.peek_timer(&TimerFunc::Illness), 100);
}

// ============================================================================
// Test 5: Objects have an age field (creation time / timer)
// ============================================================================

#[test]
fn test_egg_object_age() {
    // Eggs track age for hatching
    let mut egg = Object::default();
    egg.class = ObjectClass::Food;
    egg.name = Some("egg".to_string());

    // Age defaults to 0
    assert_eq!(egg.age, 0, "Object age should default to 0");

    // Set age to simulate creation turn
    egg.age = 100;
    assert_eq!(egg.age, 100);

    // TimerFunc::EggHatch and TimedEventType::EggHatch exist for egg hatching
    let mut queue = TimerQueue::new();
    let obj_id = egg.id;
    let timer_id = queue.start_object_timer(150, TimerFunc::EggHatch, obj_id);
    assert!(timer_id.is_some(), "Should schedule egg hatch timer");
    assert!(
        queue.obj_has_timer(obj_id, &TimerFunc::EggHatch),
        "Object should have EggHatch timer"
    );
}

// ============================================================================
// Test 6: Lamp/candle objects track burn time via lit flag and age
// ============================================================================

#[test]
fn test_lamp_burning_timeout() {
    use nh_core::object::{begin_burn, end_burn};

    let mut lamp = Object::default();
    lamp.class = ObjectClass::Tool;
    lamp.name = Some("oil lamp".to_string());
    lamp.age = 1500; // Initial fuel

    // Not lit initially
    assert!(!lamp.lit, "Lamp should not be lit initially");

    // Lighting the lamp resets age for burn tracking
    begin_burn(&mut lamp);
    assert!(lamp.lit, "Lamp should be lit after begin_burn");
    assert_eq!(lamp.age, 0, "Age resets to 0 for burn tracking");

    // Extinguishing the lamp
    end_burn(&mut lamp);
    assert!(!lamp.lit, "Lamp should be extinguished after end_burn");

    // TimerFunc::Burn exists for object burn timers
    let mut queue = TimerQueue::new();
    let timer_id = queue.start_object_timer(500, TimerFunc::Burn, lamp.id);
    assert!(timer_id.is_some(), "Should schedule burn timer for lamp");
    assert!(queue.obj_has_timer(lamp.id, &TimerFunc::Burn));
}

// ============================================================================
// Test 7: Player multi field for multi-turn occupation tracking
// ============================================================================

#[test]
fn test_multi_turn_occupation() {
    let mut player = You::default();

    // multi defaults to 0 (not occupied)
    assert_eq!(player.multi, 0, "multi should default to 0");
    assert!(player.multi_reason.is_none(), "multi_reason should default to None");

    // Negative multi = helpless (paralyzed, sleeping, etc.)
    player.multi = -5;
    player.multi_reason = Some("sleeping".to_string());
    assert!(player.multi < 0, "Negative multi means helpless");
    assert_eq!(player.multi_reason.as_deref(), Some("sleeping"));

    // OccupationTimer provides a structured API for the same concept
    let mut timer = OccupationTimer::new();
    assert!(!timer.is_occupied());

    timer.start(3, "eating");
    assert!(timer.is_occupied());
    assert_eq!(timer.remaining(), 3);
    assert_eq!(timer.activity(), Some("eating"));

    // Ticking decrements
    timer.tick();
    assert_eq!(timer.remaining(), 2);
    timer.tick();
    assert_eq!(timer.remaining(), 1);
    timer.tick();
    assert_eq!(timer.remaining(), 0);
    assert!(!timer.is_occupied(), "Occupation should end when remaining reaches 0");
    assert!(timer.activity().is_none(), "Activity clears when occupation ends");
}

// ============================================================================
// Test 8: Corpse objects track age for rotting
// ============================================================================

#[test]
fn test_corpse_age_tracking() {
    let mut corpse = Object::default();
    corpse.class = ObjectClass::Food;
    corpse.name = Some("corpse".to_string());
    corpse.corpse_type = 42; // Some monster type

    // Age tracks when the corpse was created (turn number)
    corpse.age = 50;
    assert_eq!(corpse.age, 50, "Corpse age should be set to creation turn");

    // CorpseRot event type exists in the timeout system
    let obj_id = corpse.id;
    let mut tm = TimeoutManager::new();
    tm.schedule(TimedEvent::new(100, TimedEventType::CorpseRot(obj_id)));
    assert!(
        tm.has_pending(&TimedEventType::CorpseRot(obj_id)),
        "Should have pending CorpseRot event"
    );

    // Simulate time passing: corpse rots at turn 100
    let triggered = tm.tick(100);
    assert_eq!(triggered.len(), 1);
    assert_eq!(triggered[0].event_type, TimedEventType::CorpseRot(obj_id));
}

// ============================================================================
// Test 9: Fumbling property exists on player
// ============================================================================

#[test]
fn test_fumbling_timeout() {
    let mut player = You::default();

    // Player does not have Fumbling by default
    assert!(
        !player.properties.has(Property::Fumbling),
        "Fumbling should not be active by default"
    );

    // Granting the Fumbling intrinsic makes it active
    player.properties.grant_intrinsic(Property::Fumbling);
    assert!(
        player.properties.has(Property::Fumbling),
        "Fumbling should be active after granting"
    );

    // Removing the Fumbling intrinsic clears it
    player.properties.remove_intrinsic(Property::Fumbling);
    assert!(
        !player.properties.has(Property::Fumbling),
        "Fumbling should be cleared after removing"
    );
}

// ============================================================================
// Test 10: Wounded legs timeout exists on player
// ============================================================================

#[test]
fn test_wounded_legs_timeout() {
    let mut player = You::default();

    // Wounded legs default to 0 duration
    assert_eq!(player.wounded_legs_left, 0, "Left leg should default to 0");
    assert_eq!(player.wounded_legs_right, 0, "Right leg should default to 0");
    assert!(
        !player.properties.has(Property::WoundedLegs),
        "WoundedLegs property should not be active by default"
    );

    // Setting wounded legs: left leg for 10 turns
    player.wounded_legs_left = 10;
    player.properties.grant_intrinsic(Property::WoundedLegs);
    assert_eq!(player.wounded_legs_left, 10);
    assert!(player.properties.has(Property::WoundedLegs));

    // Setting wounded legs: right leg for 15 turns
    player.wounded_legs_right = 15;
    assert_eq!(player.wounded_legs_right, 15);

    // Healing legs clears both durations and the property
    player.wounded_legs_left = 0;
    player.wounded_legs_right = 0;
    player.properties.remove_intrinsic(Property::WoundedLegs);
    assert_eq!(player.wounded_legs_left, 0);
    assert_eq!(player.wounded_legs_right, 0);
    assert!(!player.properties.has(Property::WoundedLegs));
}
