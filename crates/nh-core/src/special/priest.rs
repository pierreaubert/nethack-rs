//! Priest system (priest.c)
//!
//! Implements temple priests, altars, and religious interactions.
//! Priests guard temples and can offer services like healing and blessing.
//!
//! Core systems:
//! - Priest creation and shrine assignment (priestini, newepri)
//! - Priest AI and movement (pri_move)
//! - Priest interactions (priest_talk, priestname)
//! - Temple management (intemple, has_shrine, temple_occupied)

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::dungeon::Level;
use crate::gameloop::GameState;
use crate::monster::{Monster, MonsterId, MonsterState};
use crate::object::Object;
use crate::player::{AlignmentType, You};
use crate::rng::GameRng;
use hashbrown::HashMap;

/// Priest type index (simplified - in real NetHack uses PM_* constants)
const PM_PRIEST: i16 = 100;
const PM_PRIESTESS: i16 = 101;
const PM_HIGH_PRIEST: i16 = 102;

/// Altar alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum AltarAlignment {
    #[default]
    Unaligned,
    Lawful,
    Neutral,
    Chaotic,
}

/// Priest shrine level location (equivalent to C's d_level)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ShrineLevelId {
    pub dungeon: u8,
    pub level: u8,
}

impl Default for ShrineLevelId {
    fn default() -> Self {
        Self {
            dungeon: 0,
            level: 1,
        }
    }
}

/// Extended priest data (equivalent to C struct epri)
/// Stores shrine information and state for temple priests
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PriestExtension {
    /// Alignment of the shrine
    pub shrine_alignment: AltarAlignment,
    /// Room number of the shrine (index into rooms array)
    pub shrine_room: u8,
    /// Position of the altar
    pub shrine_pos: (i8, i8),
    /// Level where shrine is located
    pub shrine_level: ShrineLevelId,
    /// Move counter for limiting "intones" message verbosity
    pub intone_time: u32,
    /// Move counter for limiting entry messages
    pub enter_time: u32,
    /// Move counter for "forbidding feeling" message timing
    pub hostile_time: u32,
    /// Move counter for "sense of peace" message timing
    pub peaceful_time: u32,
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

impl PriestExtension {
    /// Create new priest extension for a shrine
    pub fn new(
        alignment: AltarAlignment,
        room: u8,
        shrine_x: i8,
        shrine_y: i8,
        level: ShrineLevelId,
    ) -> Self {
        Self {
            shrine_alignment: alignment,
            shrine_room: room,
            shrine_pos: (shrine_x, shrine_y),
            shrine_level: level,
            intone_time: 0,
            enter_time: 0,
            hostile_time: 0,
            peaceful_time: 0,
        }
    }

    /// Check if shrine location is valid
    pub fn has_valid_shrine(&self) -> bool {
        self.shrine_pos != (-1, -1)
    }
}

/// Temple data for a level
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

/// Check if a monster is a priest (newepri equivalent - checking for priest status)
pub fn is_priest(monster: &Monster) -> bool {
    monster.is_priest
}

/// Get priest extension if monster is a priest
pub fn get_priest_ext(monster: &Monster) -> Option<&PriestExtension> {
    if monster.is_priest {
        // In a full implementation, this would access the extension from the monster
        // For now, return None as we need to add this field to Monster struct
        None
    } else {
        None
    }
}

/// Get mutable priest extension
pub fn get_priest_ext_mut(monster: &mut Monster) -> Option<&mut PriestExtension> {
    if monster.is_priest {
        // In a full implementation, this would access the extension from the monster
        None
    } else {
        None
    }
}

/// Create or reinitialize priest extension (newepri equivalent)
/// Called when a monster becomes a priest
pub fn create_priest_extension(
    monster: &mut Monster,
    alignment: AltarAlignment,
    room: u8,
    shrine_x: i8,
    shrine_y: i8,
    level: ShrineLevelId,
) {
    // In full implementation, would allocate PriestExtension and attach to monster
    // For now, we note that this should be done
    monster.is_priest = true;
    // Store shrine data via Monster.priest_extension (to be added)
}

/// Free priest extension when priest becomes non-priest (free_epri equivalent)
pub fn free_priest_extension(monster: &mut Monster) {
    monster.is_priest = false;
    // In full implementation, would deallocate and detach the extension
}

/// Create a priest monster for a temple (priestini equivalent)
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
        priest.name = format!(
            "high {} of {}",
            if is_female { "priestess" } else { "priest" },
            alignment.name()
        );
    } else {
        priest.hp = 30 + rng.rnd(20) as i32;
        priest.level = 10;
        priest.name = format!(
            "{} of {}",
            if is_female { "priestess" } else { "priest" },
            alignment.name()
        );
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

/// Check if priest is in their own shrine (inhistemple equivalent)
pub fn is_in_own_shrine(priest: &Monster, shrine: &PriestExtension) -> bool {
    if !priest.is_priest {
        return false;
    }

    // Check if priest is at shrine location and on correct level
    priest.x == shrine.shrine_pos.0 && priest.y == shrine.shrine_pos.1
}

/// Check if shrine is still valid and properly aligned (has_shrine equivalent)
pub fn shrine_is_valid(
    level: &Level,
    shrine_pos: (i8, i8),
    expected_alignment: AltarAlignment,
) -> bool {
    // Check bounds
    if !level.is_valid_pos(shrine_pos.0, shrine_pos.1) {
        return false;
    }

    // Check if position has an altar
    if level.cells[shrine_pos.0 as usize][shrine_pos.1 as usize].typ
        != crate::dungeon::CellType::Altar
    {
        return false;
    }

    // Verify alignment matches (in full implementation, would check altar alignment from level data)
    // For now, just check it's an altar
    true
}

/// Find the priest assigned to a temple (findpriest equivalent)
pub fn find_shrine_priest(level: &Level, room_num: u8) -> Option<MonsterId> {
    // Search through monsters on this level for a priest in the specified room
    for monster in &level.monsters {
        if monster.is_priest {
            // In full implementation, would check shrine_room from extension
            // For now, return first priest found
            return Some(monster.id);
        }
    }
    None
}

/// Get formatted priest name with alignment and title (priestname equivalent)
pub fn get_priest_name(priest: &Monster, is_invisible: bool) -> String {
    let mut name = String::new();

    if is_invisible {
        name.push_str("invisible ");
    }

    // Gender-specific title
    let title = if priest.female { "priestess" } else { "priest" };
    name.push_str(title);

    // Append name with alignment
    name.push(' ');
    name.push_str(&priest.name);

    name
}

/// Handle priest interaction/conversation (priest_talk equivalent)
pub fn handle_priest_talk(priest: &mut Monster, player: &You, donation_amount: i32) -> String {
    let mut response = String::new();

    // Check if player and priest share alignment
    let co_aligned = match priest.alignment.signum() {
        0 => player.alignment.typ == crate::player::AlignmentType::Neutral,
        x if x > 0 => player.alignment.typ == crate::player::AlignmentType::Lawful,
        _ => player.alignment.typ == crate::player::AlignmentType::Chaotic,
    };

    if donation_amount <= 0 {
        // No donation
        if co_aligned {
            response = format!(
                "\"Greetings, fellow {}. May you find blessing in this place.\"",
                match priest.alignment.signum() {
                    x if x > 0 => "lawful",
                    0 => "neutral",
                    _ => "chaotic",
                }
            );
        } else {
            response = "\"I have no time for the unfaithful.\"".to_string();
        }
    } else if co_aligned {
        // Co-aligned donation
        if donation_amount < 100 {
            response = "\"Your meager donation is noted, but insufficient.\"".to_string();
        } else if donation_amount < 500 {
            response = "\"Your generous donation is appreciated.\"".to_string();
        } else {
            response = "\"Truly blessed are you. Go forth with our blessing.\"".to_string();
        }
    } else {
        // Cross-aligned donation
        response = "\"The altar accepts this offering, but you remain unfaithful.\"".to_string();
    }

    response
}

/// Move priest toward their shrine (pri_move equivalent)
pub fn move_priest_to_shrine(priest: &mut Monster, shrine_pos: (i8, i8), level: &Level) -> bool {
    // Simplified movement toward shrine position
    let dx = (shrine_pos.0 - priest.x).signum();
    let dy = (shrine_pos.1 - priest.y).signum();

    let new_x = priest.x + dx;
    let new_y = priest.y + dy;

    // Check if new position is walkable
    if level.is_valid_pos(new_x, new_y) {
        priest.x = new_x;
        priest.y = new_y;
        true
    } else {
        false
    }
}

/// Handle temple entry effects (intemple equivalent)
pub fn handle_temple_entry(
    level: &Level,
    player: &You,
    shrine: &PriestExtension,
    game_turn: u32,
) -> Vec<String> {
    let mut messages = Vec::new();

    // Check altar alignment
    let shrine_valid = shrine_is_valid(level, shrine.shrine_pos, shrine.shrine_alignment);

    if !shrine_valid {
        messages.push("You sense this shrine has been desecrated.".to_string());
    } else {
        // Check if player is co-aligned
        let co_aligned = shrine.shrine_alignment.matches_player(player.alignment.typ);

        if co_aligned {
            messages.push(format!(
                "You feel a sense of peace. This is a {} shrine.",
                shrine.shrine_alignment.name()
            ));
        } else {
            messages.push(format!(
                "You sense a forbidding presence. This is a {} shrine.",
                shrine.shrine_alignment.name()
            ));
        }
    }

    messages
}

/// Clear priests from a level when saving bones (clearpriests equivalent)
pub fn clear_priests_for_save(level: &mut Level) {
    // Remove all priests from the current level
    // This prevents priests from appearing in wrong locations when bones are restored
    level.monsters.retain(|m| !m.is_priest);
}

/// Restore priest after loading save file (restpriest equivalent)
pub fn restore_priest_after_load(
    priest: &mut Monster,
    current_level: ShrineLevelId,
    is_bones_file: bool,
) {
    if is_bones_file {
        // Adjust priest's shrine level to current player level if from bones file
        // This prevents shrine level mismatches
        // In full implementation, would update priest_extension.shrine_level
    }
}

/// Check if any temple room exists in an array of room IDs (temple_occupied equivalent)
pub fn find_temple_in_rooms(room_ids: &[u8]) -> Option<u8> {
    for &room_id in room_ids {
        // In full implementation, would check if room type is TEMPLE
        // For now, simplified check
        if room_id < 50 {
            // Arbitrary temple room number range
            return Some(room_id);
        }
    }
    None
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

    // ========== EXPANDED TEST COVERAGE ==========

    #[test]
    fn test_altar_alignment_default() {
        assert_eq!(AltarAlignment::default(), AltarAlignment::Unaligned);
    }

    #[test]
    fn test_altar_alignment_all_combinations() {
        // Test all alignment types
        let player_lawful = AlignmentType::Lawful;
        let player_neutral = AlignmentType::Neutral;
        let player_chaotic = AlignmentType::Chaotic;

        // Lawful altar
        assert!(AltarAlignment::Lawful.matches_player(player_lawful));
        assert!(!AltarAlignment::Lawful.matches_player(player_neutral));
        assert!(!AltarAlignment::Lawful.matches_player(player_chaotic));

        // Neutral altar
        assert!(!AltarAlignment::Neutral.matches_player(player_lawful));
        assert!(AltarAlignment::Neutral.matches_player(player_neutral));
        assert!(!AltarAlignment::Neutral.matches_player(player_chaotic));

        // Chaotic altar
        assert!(!AltarAlignment::Chaotic.matches_player(player_lawful));
        assert!(!AltarAlignment::Chaotic.matches_player(player_neutral));
        assert!(AltarAlignment::Chaotic.matches_player(player_chaotic));

        // Unaligned altar
        assert!(!AltarAlignment::Unaligned.matches_player(player_lawful));
        assert!(!AltarAlignment::Unaligned.matches_player(player_neutral));
        assert!(!AltarAlignment::Unaligned.matches_player(player_chaotic));
    }

    #[test]
    fn test_shrine_level_id_default() {
        let level = ShrineLevelId::default();
        assert_eq!(level.dungeon, 0);
        assert_eq!(level.level, 1);
    }

    #[test]
    fn test_priest_extension_new() {
        let ext = PriestExtension::new(AltarAlignment::Lawful, 1, 10, 10, ShrineLevelId::default());

        assert_eq!(ext.shrine_alignment, AltarAlignment::Lawful);
        assert_eq!(ext.shrine_room, 1);
        assert_eq!(ext.shrine_pos, (10, 10));
        assert_eq!(ext.intone_time, 0);
        assert_eq!(ext.enter_time, 0);
        assert_eq!(ext.hostile_time, 0);
        assert_eq!(ext.peaceful_time, 0);
        assert!(ext.has_valid_shrine());
    }

    #[test]
    fn test_priest_extension_invalid_shrine() {
        let ext =
            PriestExtension::new(AltarAlignment::Chaotic, 1, -1, -1, ShrineLevelId::default());

        assert!(!ext.has_valid_shrine());
    }

    #[test]
    fn test_is_priest() {
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        assert!(!is_priest(&monster));

        monster.is_priest = true;
        assert!(is_priest(&monster));
    }

    #[test]
    fn test_get_priest_ext_not_priest() {
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        assert!(get_priest_ext(&monster).is_none());
    }

    #[test]
    fn test_create_priest_female() {
        let mut rng = GameRng::new(1);
        let priest = create_priest(15, 15, AltarAlignment::Neutral, false, &mut rng);

        assert!(priest.is_priest);
        assert!(priest.state.peaceful);
        assert!(priest.name.contains("neutral"));
        assert_eq!(priest.level, 10);
        assert!(priest.hp > 0);
        assert!(priest.hp <= 50);
    }

    #[test]
    fn test_create_high_priest_female() {
        let mut rng = GameRng::new(2);
        let priest = create_priest(20, 20, AltarAlignment::Chaotic, true, &mut rng);

        assert!(priest.is_priest);
        assert!(priest.name.contains("high"));
        assert!(priest.name.contains("chaotic"));
        assert_eq!(priest.level, 25);
        assert!(priest.hp >= 80);
        assert!(priest.hp <= 120);
    }

    #[test]
    fn test_free_priest_extension() {
        let mut priest = Monster::new(MonsterId(1), 0, 5, 5);
        priest.is_priest = true;

        free_priest_extension(&mut priest);

        assert!(!priest.is_priest);
    }

    #[test]
    fn test_is_in_own_shrine() {
        let priest = Monster::new(MonsterId(1), 0, 10, 10);
        let shrine =
            PriestExtension::new(AltarAlignment::Lawful, 1, 10, 10, ShrineLevelId::default());

        // Would use modified monster.x, monster.y after read
        // For now just test that function exists
        let _result = is_in_own_shrine(&priest, &shrine);
    }

    #[test]
    fn test_temple_with_priest_id() {
        let mut temple = Temple::new(10, 10, AltarAlignment::Lawful);
        assert!(temple.priest_id.is_none());

        temple.priest_id = Some(MonsterId(1));
        assert!(temple.priest_id.is_some());
    }

    #[test]
    fn test_temple_desecration() {
        let mut temple = Temple::new(10, 10, AltarAlignment::Lawful);
        assert!(!temple.desecrated);

        temple.desecrated = true;
        assert!(temple.desecrated);
    }

    #[test]
    fn test_anger_priest_with_monster() {
        let mut level = crate::dungeon::Level::new(crate::dungeon::DLevel::main_dungeon_start());
        let mut priest_monster = Monster::new(MonsterId(1), 0, 10, 10);
        priest_monster.state.peaceful = true;
        level.monsters.push(priest_monster);

        let mut temple = Temple::new(10, 10, AltarAlignment::Lawful);
        temple.priest_id = Some(MonsterId(1));

        anger_priest(&mut temple, &mut level);

        assert!(temple.priest_angry);
        if let Some(priest) = level.monster(MonsterId(1)) {
            assert!(!priest.state.peaceful);
        }
    }

    #[test]
    fn test_create_priest_all_alignments() {
        let mut rng = GameRng::new(42);

        let lawful = create_priest(10, 10, AltarAlignment::Lawful, false, &mut rng);
        assert!(lawful.name.contains("lawful"));

        let neutral = create_priest(10, 10, AltarAlignment::Neutral, false, &mut rng);
        assert!(neutral.name.contains("neutral"));

        let chaotic = create_priest(10, 10, AltarAlignment::Chaotic, false, &mut rng);
        assert!(chaotic.name.contains("chaotic"));
    }

    #[test]
    fn test_get_priest_name_visible() {
        let priest = Monster::new(MonsterId(1), 0, 10, 10);
        let name = get_priest_name(&priest, false);
        assert!(!name.is_empty());
    }

    #[test]
    fn test_get_priest_name_invisible() {
        let priest = Monster::new(MonsterId(1), 0, 10, 10);
        let name = get_priest_name(&priest, true);
        assert!(!name.is_empty());
    }

    #[test]
    fn test_shrine_level_equality() {
        let level1 = ShrineLevelId {
            dungeon: 0,
            level: 5,
        };
        let level2 = ShrineLevelId {
            dungeon: 0,
            level: 5,
        };
        let level3 = ShrineLevelId {
            dungeon: 0,
            level: 6,
        };

        assert_eq!(level1, level2);
        assert_ne!(level1, level3);
    }

    #[test]
    fn test_priest_location() {
        let mut priest1 = Monster::new(MonsterId(1), 0, 10, 10);
        priest1.is_priest = true;

        let mut priest2 = Monster::new(MonsterId(2), 0, 15, 15);
        priest2.is_priest = true;

        assert_eq!(priest1.x, 10);
        assert_eq!(priest1.y, 10);
        assert_eq!(priest2.x, 15);
        assert_eq!(priest2.y, 15);
    }

    #[test]
    fn test_temple_list_operations() {
        let mut temples = Vec::new();
        temples.push(Temple::new(10, 10, AltarAlignment::Lawful));
        temples.push(Temple::new(20, 20, AltarAlignment::Neutral));
        temples.push(Temple::new(30, 30, AltarAlignment::Chaotic));

        assert_eq!(temples.len(), 3);
        assert_eq!(temples[0].alignment, AltarAlignment::Lawful);
        assert_eq!(temples[1].alignment, AltarAlignment::Neutral);
        assert_eq!(temples[2].alignment, AltarAlignment::Chaotic);
    }

    #[test]
    fn test_priest_name_format() {
        let priest = Monster::new(MonsterId(1), 0, 10, 10);
        let name = get_priest_name(&priest, false);
        assert!(!name.is_empty());
        assert!(!name.contains("invalid"));
    }

    #[test]
    fn test_create_priest_extension() {
        let mut priest = Monster::new(MonsterId(1), 0, 10, 10);
        assert!(!priest.is_priest);

        create_priest_extension(
            &mut priest,
            AltarAlignment::Lawful,
            1,
            10,
            10,
            ShrineLevelId::default(),
        );

        assert!(priest.is_priest);
    }
}
