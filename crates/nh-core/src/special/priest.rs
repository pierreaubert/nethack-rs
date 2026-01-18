//! Priest system (priest.c)
//!
//! Implements temple priests, altars, and religious interactions.
//! Priests guard temples and can offer services like healing and blessing.

use crate::dungeon::Level;
use crate::gameloop::GameState;
use crate::monster::{Monster, MonsterId, MonsterState};
use crate::object::Object;
use crate::player::AlignmentType;
use crate::rng::GameRng;

/// Priest type index (simplified - in real NetHack uses PM_* constants)
const PM_PRIEST: i16 = 100;
const PM_PRIESTESS: i16 = 101;
const PM_HIGH_PRIEST: i16 = 102;

/// Altar alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AltarAlignment {
    #[default]
    Unaligned,
    Lawful,
    Neutral,
    Chaotic,
}

impl AltarAlignment {
    /// Convert from player alignment type
    pub fn from_alignment(align: AlignmentType) -> Self {
        match align {
            AlignmentType::Lawful => Self::Lawful,
            AlignmentType::Neutral => Self::Neutral,
            AlignmentType::Chaotic => Self::Chaotic,
        }
    }

    /// Check if this alignment matches the player's
    pub fn matches_player(&self, player_align: AlignmentType) -> bool {
        match self {
            Self::Lawful => player_align == AlignmentType::Lawful,
            Self::Neutral => player_align == AlignmentType::Neutral,
            Self::Chaotic => player_align == AlignmentType::Chaotic,
            Self::Unaligned => false,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Unaligned => "unaligned",
            Self::Lawful => "lawful",
            Self::Neutral => "neutral",
            Self::Chaotic => "chaotic",
        }
    }
}

/// Temple data for a level
#[derive(Debug, Clone)]
pub struct Temple {
    /// Position of the altar
    pub altar_x: i8,
    pub altar_y: i8,
    /// Alignment of the altar
    pub alignment: AltarAlignment,
    /// Priest monster ID (if present)
    pub priest_id: Option<MonsterId>,
    /// Whether the temple has been desecrated
    pub desecrated: bool,
    /// Whether the priest is angry
    pub priest_angry: bool,
}

impl Temple {
    /// Create a new temple
    pub fn new(altar_x: i8, altar_y: i8, alignment: AltarAlignment) -> Self {
        Self {
            altar_x,
            altar_y,
            alignment,
            priest_id: None,
            desecrated: false,
            priest_angry: false,
        }
    }
}

/// Create a priest monster for a temple
pub fn create_priest(
    x: i8,
    y: i8,
    alignment: AltarAlignment,
    is_high_priest: bool,
    rng: &mut GameRng,
) -> Monster {
    let is_female = rng.one_in(2);
    let monster_type = if is_high_priest {
        PM_HIGH_PRIEST
    } else if is_female {
        PM_PRIESTESS
    } else {
        PM_PRIEST
    };

    let mut priest = Monster::new(MonsterId::NONE, monster_type, x, y);

    // Set priest properties
    priest.state = MonsterState::peaceful();
    priest.is_priest = true;

    // HP based on whether high priest
    if is_high_priest {
        priest.hp = 80 + rng.rnd(40) as i32;
        priest.level = 25;
        priest.name = format!("high {} of {}", 
            if is_female { "priestess" } else { "priest" },
            alignment.name());
    } else {
        priest.hp = 30 + rng.rnd(20) as i32;
        priest.level = 10;
        priest.name = format!("{} of {}",
            if is_female { "priestess" } else { "priest" },
            alignment.name());
    }
    priest.hp_max = priest.hp;

    // Set alignment
    priest.alignment = match alignment {
        AltarAlignment::Lawful => 10,
        AltarAlignment::Neutral => 0,
        AltarAlignment::Chaotic => -10,
        AltarAlignment::Unaligned => 0,
    };

    priest
}

/// Priest greeting when player enters temple
pub fn priest_greeting(state: &mut GameState, temple: &Temple) {
    if temple.priest_angry {
        state.message("\"You dare return?!\"");
        return;
    }

    let greeting = match temple.alignment {
        AltarAlignment::Lawful => "\"Welcome, seeker of justice.\"",
        AltarAlignment::Neutral => "\"Welcome, seeker of balance.\"",
        AltarAlignment::Chaotic => "\"Welcome, seeker of freedom.\"",
        AltarAlignment::Unaligned => "\"Welcome, traveler.\"",
    };
    state.message(greeting);
}

/// Result of praying at an altar
#[derive(Debug, Clone)]
pub enum PrayerResult {
    /// Prayer was successful
    Blessed { message: String },
    /// Prayer was ignored
    Ignored { message: String },
    /// Prayer angered the god
    Angered { message: String },
    /// Player was smote
    Smote { damage: i32, message: String },
}

/// Handle praying at an altar
pub fn pray_at_altar(
    state: &mut GameState,
    altar_alignment: AltarAlignment,
    rng: &mut GameRng,
) -> PrayerResult {
    let player_align = state.player.alignment.typ;
    let alignment_match = altar_alignment.matches_player(player_align);

    // Check prayer timeout
    if state.player.prayer_timeout > 0 {
        return PrayerResult::Ignored {
            message: "You feel that your prayers are not being heard.".to_string(),
        };
    }

    // Set prayer timeout (simplified - real NetHack has complex timing)
    state.player.prayer_timeout = 500;

    if alignment_match {
        // Praying at co-aligned altar
        let luck = state.player.luck;

        if luck >= 0 && rng.one_in(3) {
            // Blessed!
            state.player.hp = state.player.hp_max;
            PrayerResult::Blessed {
                message: "You feel a warm glow wash over you.".to_string(),
            }
        } else if luck < -5 {
            // Bad luck means god is displeased
            PrayerResult::Angered {
                message: "You feel that your god is displeased.".to_string(),
            }
        } else {
            PrayerResult::Ignored {
                message: "You feel that your prayer was heard.".to_string(),
            }
        }
    } else {
        // Praying at cross-aligned altar
        if rng.one_in(3) {
            // Smote for heresy!
            let damage = rng.dice(4, 6) as i32;
            PrayerResult::Smote {
                damage,
                message: format!(
                    "You are struck by a bolt of lightning for {} damage!",
                    damage
                ),
            }
        } else {
            PrayerResult::Angered {
                message: "You feel a surge of divine anger!".to_string(),
            }
        }
    }
}

/// Donation amounts and their effects
#[derive(Debug, Clone, Copy)]
pub enum DonationResult {
    /// Donation was too small
    TooSmall,
    /// Donation was accepted
    Accepted,
    /// Donation earned a blessing
    Blessed,
    /// Donation earned protection
    Protection,
    /// Donation earned clairvoyance
    Clairvoyance,
}

/// Handle donating gold at an altar
pub fn donate_at_altar(
    state: &mut GameState,
    amount: i32,
    altar_alignment: AltarAlignment,
    rng: &mut GameRng,
) -> DonationResult {
    if amount <= 0 {
        state.message("You need to donate something!");
        return DonationResult::TooSmall;
    }

    if state.player.gold < amount {
        state.message("You don't have that much gold.");
        return DonationResult::TooSmall;
    }

    let player_align = state.player.alignment.typ;
    let alignment_match = altar_alignment.matches_player(player_align);

    // Deduct gold
    state.player.gold -= amount;

    if !alignment_match {
        state.message("The altar glows briefly, then dims.");
        return DonationResult::Accepted;
    }

    // Effects based on donation amount
    let threshold = state.player.exp_level * 200;

    if amount < threshold / 4 {
        state.message("The altar briefly glows.");
        DonationResult::Accepted
    } else if amount < threshold / 2 {
        state.message("You feel a warm glow.");
        state.player.luck = (state.player.luck + 1).min(10);
        DonationResult::Accepted
    } else if amount < threshold {
        state.message("You feel blessed!");
        // Could bless an item here
        DonationResult::Blessed
    } else if rng.one_in(2) {
        state.message("You feel protected!");
        // Protection would be tracked in properties in a full implementation
        DonationResult::Protection
    } else {
        state.message("You have a vision of the dungeon!");
        DonationResult::Clairvoyance
    }
}

/// Handle sacrificing a corpse at an altar
pub fn sacrifice_at_altar(
    state: &mut GameState,
    corpse: &Object,
    altar_alignment: AltarAlignment,
    rng: &mut GameRng,
) -> bool {
    let player_align = state.player.alignment.typ;
    let alignment_match = altar_alignment.matches_player(player_align);

    // Check if it's a corpse
    if corpse.class != crate::object::ObjectClass::Food || corpse.corpse_type < 0 {
        state.message("That's not a corpse!");
        return false;
    }

    state.message("You sacrifice the corpse.");

    if alignment_match {
        // Sacrifice at co-aligned altar
        if rng.one_in(3) {
            state.message("Your god is pleased!");
            state.player.luck = (state.player.luck + 1).min(10);
        } else {
            state.message("Your sacrifice is consumed in a flash of light.");
        }
    } else {
        // Sacrifice at cross-aligned altar
        if rng.one_in(4) {
            state.message("You feel the wrath of a foreign god!");
            state.player.luck = (state.player.luck - 1).max(-10);
        } else {
            state.message("The altar glows briefly.");
        }
    }

    true
}

/// Check if player is standing on an altar
pub fn on_altar(level: &Level, x: i8, y: i8) -> bool {
    if !level.is_valid_pos(x, y) {
        return false;
    }
    level.cells[x as usize][y as usize].typ == crate::dungeon::CellType::Altar
}

/// Anger the temple priest
pub fn anger_priest(temple: &mut Temple, level: &mut Level) {
    temple.priest_angry = true;

    if let Some(priest_id) = temple.priest_id {
        if let Some(priest) = level.monster_mut(priest_id) {
            priest.state.peaceful = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_altar_alignment() {
        assert!(AltarAlignment::Lawful.matches_player(AlignmentType::Lawful));
        assert!(!AltarAlignment::Lawful.matches_player(AlignmentType::Chaotic));
        assert!(!AltarAlignment::Unaligned.matches_player(AlignmentType::Neutral));
    }

    #[test]
    fn test_altar_alignment_from_player() {
        assert_eq!(
            AltarAlignment::from_alignment(AlignmentType::Lawful),
            AltarAlignment::Lawful
        );
        assert_eq!(
            AltarAlignment::from_alignment(AlignmentType::Chaotic),
            AltarAlignment::Chaotic
        );
    }

    #[test]
    fn test_create_priest() {
        let mut rng = GameRng::new(42);
        let priest = create_priest(10, 10, AltarAlignment::Lawful, false, &mut rng);

        assert!(priest.state.peaceful);
        assert!(priest.is_priest);
        assert!(priest.hp > 0);
        assert!(priest.name.contains("lawful"));
    }

    #[test]
    fn test_create_high_priest() {
        let mut rng = GameRng::new(42);
        let priest = create_priest(10, 10, AltarAlignment::Neutral, true, &mut rng);

        assert!(priest.name.contains("high"));
        assert!(priest.level == 25);
        assert!(priest.hp >= 80);
    }

    #[test]
    fn test_temple_creation() {
        let temple = Temple::new(20, 10, AltarAlignment::Chaotic);

        assert_eq!(temple.altar_x, 20);
        assert_eq!(temple.altar_y, 10);
        assert_eq!(temple.alignment, AltarAlignment::Chaotic);
        assert!(!temple.desecrated);
        assert!(!temple.priest_angry);
    }
}
