//! Common message templates for action modules

/// Get a "nothing happens" message variant
#[cfg(not(feature = "std"))]
use crate::compat::*;

pub fn nothing_happens() -> &'static str {
    "Nothing happens."
}

/// Get a "you can't do that" message for an action
pub fn cant_do_that(action: &str) -> String {
    format!("You can't {} right now.", action)
}

/// Get a message for trying to use something without having it
pub fn you_dont_have(item: &str) -> String {
    format!("You don't have {}.", item)
}

/// Get a message for trying to use something that's in the wrong state
pub fn wrong_state(item: &str, state: &str) -> String {
    format!("That {} is {}.", item, state)
}

/// Get a message for needing a free hand
pub fn need_free_hand() -> &'static str {
    "You need a free hand to do that."
}

/// Get a message for trying to wield incompatible items
pub fn incompatible_with(item1: &str, item2: &str) -> String {
    format!("You can't use {} with {}.", item1, item2)
}

/// Get a message for item being cursed
pub fn cursed_prevents(action: &str) -> String {
    format!("A curse prevents you from {}.", action)
}

/// Get a message for item being too heavy
pub fn too_heavy(item: &str) -> String {
    format!("That {} is too heavy.", item)
}

/// Get a message for trying to move in an invalid direction
pub fn invalid_direction() -> &'static str {
    "That's not a valid direction."
}

/// Get a message for trying to move off the map
pub fn blocked_by_wall() -> &'static str {
    "You can't move in that direction."
}

/// Get a message for running out of charges/ammunition
pub fn out_of_charges(item: &str) -> String {
    format!("The {} has no charges left.", item)
}

/// Get a message for an occupied slot when trying to wear armor
pub fn slot_occupied(item: &str) -> String {
    format!("You're already wearing {}.", item)
}

/// Get a message for success
pub fn you_succeed(action: &str) -> String {
    format!("You {}.", action)
}

/// Get a message for failure
pub fn you_fail(action: &str) -> String {
    format!("You fail to {}.", action)
}

/// Get a message for cancellation
pub fn you_cancel() -> &'static str {
    "Never mind."
}

/// Get a message for running into a monster
pub fn blocked_by_monster(monster_name: &str) -> String {
    format!("There is {} in the way.", monster_name)
}

/// Get a message for too encumbered
pub fn too_encumbered() -> &'static str {
    "You are carrying too much to do that."
}

/// Get a message for being stuck or restrained
pub fn stuck_or_restrained() -> &'static str {
    "You can't move."
}

/// Get a message for an action being interrupted
pub fn interrupted() -> &'static str {
    "Your action was interrupted."
}

/// Get a message for trying to do something on a tile that doesn't support it
pub fn invalid_location(action: &str) -> String {
    format!("You can't {} here.", action)
}

/// Get a message for a confused action result
pub fn while_confused() -> &'static str {
    "You stumble around confused."
}

/// Get a message for actions prevented by helplessness
pub fn helpless(reason: &str) -> String {
    format!("You can't {} while {}.", "act", reason)
}
