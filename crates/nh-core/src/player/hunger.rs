//! Player hunger state

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Hunger state levels
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize, Display, EnumIter,
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
    /// Get the nutrition threshold for this state
    pub const fn threshold(&self) -> i32 {
        match self {
            HungerState::Satiated => 2000,
            HungerState::NotHungry => 1000,
            HungerState::Hungry => 500,
            HungerState::Weak => 150,
            HungerState::Fainting => 50,
            HungerState::Fainted => 0,
            HungerState::Starved => -1,
        }
    }

    /// Calculate hunger state from nutrition value
    pub fn from_nutrition(nutrition: i32) -> Self {
        if nutrition >= 2000 {
            HungerState::Satiated
        } else if nutrition >= 1000 {
            HungerState::NotHungry
        } else if nutrition >= 500 {
            HungerState::Hungry
        } else if nutrition >= 150 {
            HungerState::Weak
        } else if nutrition >= 50 {
            HungerState::Fainting
        } else if nutrition >= 0 {
            HungerState::Fainted
        } else {
            HungerState::Starved
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
            HungerState::Satiated | HungerState::Weak | HungerState::Fainting
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
