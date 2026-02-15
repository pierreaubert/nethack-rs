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

/// Player hunger tracker
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct HungerTracker {
    /// Current hunger points (higher = less hungry)
    pub uhunger: i32,
    /// Current hunger state
    pub state: HungerState,
}

impl HungerTracker {
    /// Initialize hunger (init_uhunger equivalent)
    ///
    /// Sets initial hunger state to not hungry with 900 nutrition points.
    /// Called at the start of the game.
    pub fn init() -> Self {
        Self {
            uhunger: 900,
            state: HungerState::NotHungry,
        }
    }

    /// Perform regular hunger check (gethungry equivalent)
    ///
    /// Called regularly to decrease hunger and update state.
    /// Metabolism depends on form, equipment, and special effects.
    /// Simplified version - real implementation would check:
    /// - Current polymorphic form
    /// - Equipment effects (slow digestion, conflict, regeneration)
    /// - Whether sleeping/unconscious
    pub fn gethungry(&mut self) {
        // Basic hunger decrease per turn
        self.uhunger = self.uhunger.saturating_sub(1);
        self.update_state(true);
    }

    /// Increase hunger (morehungry equivalent)
    ///
    /// Decreases nutrition by the given amount, typically from
    /// vomiting or casting spells.
    pub fn morehungry(&mut self, num: i32) {
        self.uhunger = self.uhunger.saturating_sub(num);
        self.update_state(true);
    }

    /// Decrease hunger (lesshungry equivalent)
    ///
    /// Increases nutrition by the given amount, typically from eating.
    /// Warns the player if getting close to satiation.
    pub fn lesshungry(&mut self, num: i32) -> Option<String> {
        self.uhunger = (self.uhunger + num).min(2000);

        let mut warnings = Vec::new();

        if self.uhunger >= 2000 {
            warnings.push("You're too full to eat more!".to_string());
        } else if self.uhunger >= 1500 {
            warnings.push("You're having a hard time getting all of it down.".to_string());
        }

        self.update_state(false);

        if warnings.is_empty() {
            None
        } else {
            Some(warnings.join(" "))
        }
    }

    /// Update hunger state based on current nutrition (newuhs equivalent)
    ///
    /// This is a simplified version. The full C version has complex logic for:
    /// - Detecting state transitions during eating
    /// - Fainting from hunger
    /// - Starving to death
    /// - Strength penalties
    fn update_state(&mut self, incr: bool) {
        let new_state = HungerState::from_nutrition(self.uhunger);

        if new_state != self.state {
            match new_state {
                HungerState::Hungry => {
                    // Stopped current occupation if hungry while doing something
                }
                HungerState::Weak => {
                    // Apply temporary strength penalty
                }
                HungerState::Fainting => {
                    // Trigger fainting sequence
                }
                _ => {}
            }

            self.state = new_state;
        }
    }

    /// Get current hunger state index (stat_hunger_indx equivalent)
    ///
    /// Returns the numeric index of the current hunger state.
    /// Used by status line display.
    pub const fn stat_hunger_index(&self) -> u8 {
        self.state as u8
    }

    /// Check if player is fainting
    pub const fn is_fainting(&self) -> bool {
        matches!(self.state, HungerState::Fainting | HungerState::Fainted)
    }

    /// Check if player has fainted
    pub const fn is_fainted(&self) -> bool {
        matches!(self.state, HungerState::Fainted)
    }
}
