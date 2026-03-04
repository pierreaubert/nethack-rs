//! Player conduct tracking
//!
//! Tracks various challenge conducts (vegetarian, pacifist, etc.)

#[cfg(not(feature = "std"))]
use crate::compat::*;

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

/// Display conduct information (doconduct equivalent)
///
/// This is the command handler for the #conduct extended command.
/// Returns a list of strings describing the player's voluntary challenges.
///
/// # Arguments
/// * `conduct` - The player's conduct record
/// * `num_genocides` - Number of monster types genocided
/// * `final` - If true, use past tense (for game end summary)
pub fn doconduct(conduct: &Conduct, num_genocides: u32, final_display: bool) -> Vec<String> {
    show_conduct(conduct, num_genocides, final_display)
}

/// Show conduct information with detailed output (show_conduct equivalent)
///
/// # Arguments
/// * `conduct` - The player's conduct record
/// * `num_genocides` - Number of monster types genocided
/// * `final_display` - If true, use past tense (for game end summary)
pub fn show_conduct(conduct: &Conduct, num_genocides: u32, final_display: bool) -> Vec<String> {
    let mut lines = vec!["Voluntary challenges:".to_string()];

    let (have, has_been) = if final_display {
        ("went", "were")
    } else {
        ("have gone", "have been")
    };

    // Food conduct
    if conduct.food == 0 {
        lines.push(format!("  You {} without food.", have));
    } else if conduct.unvegan == 0 {
        lines.push(format!("  You {} followed a strict vegan diet.", have));
    } else if conduct.unvegetarian == 0 {
        lines.push(format!("  You {} vegetarian.", has_been));
    }

    // Atheist conduct
    if conduct.gnostic == 0 {
        lines.push(format!("  You {} an atheist.", has_been));
    }

    // Weaponless conduct
    if conduct.weaphit == 0 {
        lines.push("  You have never hit with a wielded weapon.".to_string());
    } else {
        lines.push(format!(
            "  You have used a wielded weapon {} time{}.",
            conduct.weaphit,
            if conduct.weaphit == 1 { "" } else { "s" }
        ));
    }

    // Pacifist conduct
    if conduct.killer == 0 {
        lines.push(format!("  You {} a pacifist.", has_been));
    }

    // Illiterate conduct
    if conduct.literate == 0 {
        lines.push(format!("  You {} illiterate.", has_been));
    } else {
        lines.push(format!(
            "  You have read items or engraved {} time{}.",
            conduct.literate,
            if conduct.literate == 1 { "" } else { "s" }
        ));
    }

    // Genocide conduct
    if num_genocides == 0 {
        lines.push("  You have never genocided any monsters.".to_string());
    } else {
        lines.push(format!(
            "  You have genocided {} type{} of monster{}.",
            num_genocides,
            if num_genocides == 1 { "" } else { "s" },
            if num_genocides == 1 { "" } else { "s" }
        ));
    }

    // Polypile conduct
    if conduct.polypiles == 0 {
        lines.push("  You have never polymorphed an object.".to_string());
    } else {
        lines.push(format!(
            "  You have polymorphed {} item{}.",
            conduct.polypiles,
            if conduct.polypiles == 1 { "" } else { "s" }
        ));
    }

    // Polyself conduct
    if conduct.polyselfs == 0 {
        lines.push("  You have never changed form.".to_string());
    } else {
        lines.push(format!(
            "  You have changed form {} time{}.",
            conduct.polyselfs,
            if conduct.polyselfs == 1 { "" } else { "s" }
        ));
    }

    // Wish conduct
    if conduct.wishes == 0 {
        lines.push("  You have used no wishes.".to_string());
    } else {
        let mut wish_str = format!(
            "  You have used {} wish{}",
            conduct.wishes,
            if conduct.wishes == 1 { "" } else { "es" }
        );

        if conduct.wisharti > 0 {
            if conduct.wisharti == conduct.wishes {
                let prefix = if conduct.wisharti > 2 {
                    "all "
                } else if conduct.wisharti == 2 {
                    "both "
                } else {
                    ""
                };
                wish_str.push_str(&format!(
                    " ({}for {})",
                    prefix,
                    if conduct.wisharti == 1 {
                        "an artifact"
                    } else {
                        "artifacts"
                    }
                ));
            } else {
                wish_str.push_str(&format!(
                    " ({} for {})",
                    conduct.wisharti,
                    if conduct.wisharti == 1 {
                        "an artifact"
                    } else {
                        "artifacts"
                    }
                ));
            }
        }
        wish_str.push('.');
        lines.push(wish_str);

        if conduct.wisharti == 0 {
            lines.push("  You have not wished for any artifacts.".to_string());
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conduct_default_is_clean() {
        let conduct = Conduct::default();
        assert!(conduct.is_vegetarian());
        assert!(conduct.is_vegan());
        assert!(conduct.is_foodless());
        assert!(conduct.is_atheist());
        assert!(conduct.is_weaponless());
        assert!(conduct.is_pacifist());
        assert!(conduct.is_illiterate());
        assert!(conduct.is_polypileless());
        assert!(conduct.is_polyselfless());
        assert!(conduct.is_wishless());
        assert!(conduct.is_artiwishless());
        assert!(conduct.is_genocideless());
    }

    #[test]
    fn test_conduct_violations() {
        let mut conduct = Conduct::default();

        conduct.ate_meat();
        assert!(!conduct.is_vegetarian());
        assert!(!conduct.is_vegan());
        assert!(!conduct.is_foodless());

        conduct.killed_monster();
        assert!(!conduct.is_pacifist());

        conduct.read_something();
        assert!(!conduct.is_illiterate());

        conduct.made_wish(true);
        assert!(!conduct.is_wishless());
        assert!(!conduct.is_artiwishless());
    }

    #[test]
    fn test_doconduct_output() {
        let conduct = Conduct::default();
        let lines = doconduct(&conduct, 0, false);
        assert!(!lines.is_empty());
        assert!(lines[0].contains("Voluntary challenges"));
    }

    #[test]
    fn test_show_conduct_with_violations() {
        let mut conduct = Conduct::default();
        conduct.weaphit = 5;
        conduct.wishes = 3;
        conduct.wisharti = 1;

        let lines = show_conduct(&conduct, 2, false);
        assert!(lines.iter().any(|l| l.contains("5 times")));
        assert!(lines.iter().any(|l| l.contains("3 wishes")));
        assert!(lines.iter().any(|l| l.contains("genocided 2 types")));
    }

    #[test]
    fn test_show_conduct_final_tense() {
        let conduct = Conduct::default();
        let lines = show_conduct(&conduct, 0, true);
        // Final display uses past tense
        assert!(
            lines
                .iter()
                .any(|l| l.contains("went") || l.contains("were"))
        );
    }
}
