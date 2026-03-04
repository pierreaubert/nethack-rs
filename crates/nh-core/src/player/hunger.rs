//! Player hunger state

#[cfg(not(feature = "std"))]
use crate::compat::*;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Hunger state levels
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
    Display,
    EnumIter,
)]
#[repr(u8)]
pub enum HungerState {
    /// Over-fed (negative effects)
    Satiated = 0,

    /// Normal state
    #[default]
    NotHungry = 1,

    /// Getting hungry
    Hungry = 2,

    /// Weak from hunger
    Weak = 3,

    /// About to faint
    Fainting = 4,

    /// Currently fainted
    Fainted = 5,

    /// Dead from starvation
    Starved = 6,
}

impl HungerState {
    /// Get the nutrition threshold for this state (C: newuhs eat.c:2936-2939)
    pub const fn threshold(&self) -> i32 {
        match self {
            HungerState::Satiated => 1000, // C: h > 1000 → SATIATED
            HungerState::NotHungry => 150, // C: h > 150 → NOT_HUNGRY
            HungerState::Hungry => 50,     // C: h > 50 → HUNGRY
            HungerState::Weak => 0,        // C: h > 0 → WEAK
            HungerState::Fainting => -1,   // C: else → FAINTING
            HungerState::Fainted => -1,    // Set by newuhs logic, not threshold
            HungerState::Starved => -1,    // Set by newuhs logic, not threshold
        }
    }

    /// Calculate hunger state from nutrition value (C: newuhs in eat.c:2936-2939)
    ///
    /// C thresholds: h > 1000 → SATIATED, h > 150 → NOT_HUNGRY,
    ///               h > 50 → HUNGRY, h > 0 → WEAK, else → FAINTING
    /// Note: FAINTED and STARVED are set by special logic in newuhs(), not thresholds
    pub fn from_nutrition(nutrition: i32) -> Self {
        if nutrition > 1000 {
            HungerState::Satiated
        } else if nutrition > 150 {
            HungerState::NotHungry
        } else if nutrition > 50 {
            HungerState::Hungry
        } else if nutrition > 0 {
            HungerState::Weak
        } else {
            HungerState::Fainting
        }
    }

    /// Check if player can act normally
    pub const fn can_act(&self) -> bool {
        !matches!(self, HungerState::Fainted | HungerState::Starved)
    }

    /// Check if player suffers penalties
    pub const fn has_penalty(&self) -> bool {
        matches!(
            self,
            HungerState::Satiated | HungerState::Hungry | HungerState::Weak | HungerState::Fainting
        )
    }

    /// Get status line display string
    pub const fn status_string(&self) -> Option<&'static str> {
        match self {
            HungerState::Satiated => Some("Satiated"),
            HungerState::NotHungry => None,
            HungerState::Hungry => Some("Hungry"),
            HungerState::Weak => Some("Weak"),
            HungerState::Fainting => Some("Fainting"),
            HungerState::Fainted => Some("Fainted"),
            HungerState::Starved => Some("Starved"),
        }
    }
}
