//! Player conduct tracking
//!
//! Tracks various challenge conducts (vegetarian, pacifist, etc.)

use serde::{Deserialize, Serialize};

/// Player conduct record
///
/// Tracks violations of various conducts for challenge runs.
/// A value of 0 means the conduct has been maintained.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Conduct {
    /// Eaten meat (breaks vegetarian)
    pub unvegetarian: u32,

    /// Eaten animal products (breaks vegan)
    pub unvegan: u32,

    /// Eaten any food (breaks foodless)
    pub food: u32,

    /// Used divine powers (breaks atheist)
    pub gnostic: u32,

    /// Hit with weapon (breaks weaponless)
    pub weaphit: u32,

    /// Killed monsters (breaks pacifist)
    pub killer: u32,

    /// Read scrolls/spellbooks (breaks illiterate)
    pub literate: u32,

    /// Polymorphed objects (breaks polypileless)
    pub polypiles: u32,

    /// Polymorphed self (breaks polyselfless)
    pub polyselfs: u32,

    /// Made wishes (breaks wishless)
    pub wishes: u32,

    /// Wished for artifacts (breaks artiwishless)
    pub wisharti: u32,

    /// Genocided monsters (breaks genocideless)
    pub genocides: u32,
}

impl Conduct {
    /// Check if vegetarian conduct is maintained
    pub const fn is_vegetarian(&self) -> bool {
        self.unvegetarian == 0
    }

    /// Check if vegan conduct is maintained
    pub const fn is_vegan(&self) -> bool {
        self.unvegan == 0 && self.unvegetarian == 0
    }

    /// Check if foodless conduct is maintained
    pub const fn is_foodless(&self) -> bool {
        self.food == 0
    }

    /// Check if atheist conduct is maintained
    pub const fn is_atheist(&self) -> bool {
        self.gnostic == 0
    }

    /// Check if weaponless conduct is maintained
    pub const fn is_weaponless(&self) -> bool {
        self.weaphit == 0
    }

    /// Check if pacifist conduct is maintained
    pub const fn is_pacifist(&self) -> bool {
        self.killer == 0
    }

    /// Check if illiterate conduct is maintained
    pub const fn is_illiterate(&self) -> bool {
        self.literate == 0
    }

    /// Check if polypileless conduct is maintained
    pub const fn is_polypileless(&self) -> bool {
        self.polypiles == 0
    }

    /// Check if polyselfless conduct is maintained
    pub const fn is_polyselfless(&self) -> bool {
        self.polyselfs == 0
    }

    /// Check if wishless conduct is maintained
    pub const fn is_wishless(&self) -> bool {
        self.wishes == 0
    }

    /// Check if artifact wishless conduct is maintained
    pub const fn is_artiwishless(&self) -> bool {
        self.wisharti == 0
    }

    /// Check if genocideless conduct is maintained
    pub const fn is_genocideless(&self) -> bool {
        self.genocides == 0
    }

    /// Record eating non-vegan food
    pub fn ate_non_vegan(&mut self) {
        self.unvegan += 1;
        self.food += 1;
    }

    /// Record eating meat
    pub fn ate_meat(&mut self) {
        self.unvegetarian += 1;
        self.unvegan += 1;
        self.food += 1;
    }

    /// Record killing a monster
    pub fn killed_monster(&mut self) {
        self.killer += 1;
    }

    /// Record reading
    pub fn read_something(&mut self) {
        self.literate += 1;
    }

    /// Record making a wish
    pub fn made_wish(&mut self, for_artifact: bool) {
        self.wishes += 1;
        if for_artifact {
            self.wisharti += 1;
        }
    }
}
