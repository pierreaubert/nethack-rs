//! Player death and game end processing (end.c)
//!
//! Handles life saving, death processing, score calculation,
//! end-game disclosure, and vanquished creatures listing.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::monster::makemon::MonsterVitals;
use crate::monster::PerMonst;
use crate::player::{Attribute, HungerState, You};

// ============================================================================
// Death types (C: how in done())
// ============================================================================

/// How the player died or ended the game.
///
/// Matches C enum in end.c. Integer values match C for compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum DeathType {
    /// Killed by a monster or trap
    Killed = 0,
    /// Choked to death (on food)
    Choking = 1,
    /// Poisoned
    Poisoned = 2,
    /// Starved to death
    Starving = 3,
    /// Drowned
    Drowning = 4,
    /// Burned to death (in lava, etc.)
    Burning = 5,
    /// Dissolved (in acid, etc.)
    Dissolved = 6,
    /// Crushed (by a collapsing ceiling, etc.)
    Crushed = 7,
    /// Turned to stone
    Stoning = 8,
    /// Turned to slime
    TurnedSlime = 9,
    /// Genocided self
    Genocided = 10,
    /// Panicked (game error)
    Panicked = 11,
    /// Tricked (wizard mode shenanigans)
    Tricked = 12,
    /// Quit the game
    Quit = 13,
    /// Escaped the dungeon
    Escaped = 14,
    /// Ascended
    Ascended = 15,
}

impl DeathType {
    /// Whether this death type allows life saving
    pub fn allows_life_saving(self) -> bool {
        (self as i32) <= (DeathType::Genocided as i32)
    }

    /// Whether this is a "real" death (not quit/escape/ascend)
    pub fn is_death(self) -> bool {
        (self as i32) < (DeathType::Panicked as i32)
    }

    /// Whether bones file can be created
    pub fn allows_bones(self) -> bool {
        (self as i32) < (DeathType::Genocided as i32)
    }

    /// Default death message
    pub fn default_message(self) -> &'static str {
        match self {
            DeathType::Killed => "died",
            DeathType::Choking => "choked",
            DeathType::Poisoned => "poisoned",
            DeathType::Starving => "starved",
            DeathType::Drowning => "drowned",
            DeathType::Burning => "burned",
            DeathType::Dissolved => "dissolved",
            DeathType::Crushed => "crushed",
            DeathType::Stoning => "turned to stone",
            DeathType::TurnedSlime => "turned to slime",
            DeathType::Genocided => "genocided",
            DeathType::Panicked => "panicked",
            DeathType::Tricked => "tricked",
            DeathType::Quit => "quit",
            DeathType::Escaped => "escaped",
            DeathType::Ascended => "ascended",
        }
    }
}

// ============================================================================
// Killer tracking
// ============================================================================

/// Tracks the cause of death for score/message purposes.
///
/// Matches C `struct killer` in end.c.
#[derive(Debug, Clone, Default)]
pub struct Killer {
    /// Name of what killed the player
    pub name: String,
    /// How to format the name in death message
    pub format: KillerFormat,
}

/// How to prefix the killer's name in death messages.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum KillerFormat {
    /// "killed by a ___"
    #[default]
    KilledByAn,
    /// "killed by ___"
    KilledBy,
    /// No prefix, just the name
    NoPrefix,
}

// ============================================================================
// Life saving
// ============================================================================

/// Restore the player after a near-death experience (C: savelife from end.c).
///
/// Restores HP to max (with minimum based on level), cures hunger if
/// needed, and resets traps. Called when an amulet of life saving triggers
/// or in wizard mode.
pub fn savelife(player: &mut You, how: DeathType) {
    // Minimum HP is max(2 * level, 10)
    let uhpmin = (2 * player.exp_level).max(10);
    if player.hp_max < uhpmin {
        player.hp_max = uhpmin;
    }
    player.hp = player.hp_max;

    // Cure hunger if starving or choking
    if player.nutrition < 500 || how == DeathType::Choking {
        player.nutrition = 900;
        player.hunger_state = HungerState::NotHungry;
    }

    // Reset lava trap
    if player.utrap > 0 {
        player.utrap = 0;
    }
}

/// Check if the player has a life-saving amulet equipped.
///
/// Searches the player's worn items for an amulet of life saving.
/// Returns the inventory index if found.
pub fn player_has_life_saving(inventory: &[crate::object::Object]) -> Option<usize> {
    inventory.iter().position(|obj| {
        obj.worn_mask != 0
            && obj.name.as_ref().is_some_and(|n| n.contains("life saving"))
    })
}

/// Process player life saving when dying.
///
/// If the player has an amulet of life saving:
/// - Consume the amulet
/// - Restore HP
/// - Reduce Constitution by 1
/// - Return true (player survived)
///
/// Returns true if player was saved.
pub fn try_life_saving(
    player: &mut You,
    inventory: &mut Vec<crate::object::Object>,
    how: DeathType,
) -> bool {
    if !how.allows_life_saving() {
        return false;
    }

    if let Some(idx) = player_has_life_saving(inventory) {
        // Consume the amulet
        inventory.remove(idx);

        // Lose 1 Constitution
        let con = player.attr_current.get(Attribute::Constitution);
        if con > 3 {
            player.attr_current.set(Attribute::Constitution, con - 1);
        }

        // Restore HP and cure conditions
        savelife(player, how);

        // Handle genocided — can't be saved
        if how == DeathType::Genocided {
            player.hp = 0;
            return false;
        }

        true
    } else {
        false
    }
}

// ============================================================================
// Score calculation
// ============================================================================

/// Calculate the player's final score.
///
/// Matches C scoring in really_done() from end.c:
/// - Gold collected (net gain from start)
/// - Depth bonus: 50 * (deepest - 1)
/// - Extra depth bonus for deep exploration (>20)
/// - Death penalty: -10% for dying (not quitting)
/// - Ascension bonus: score * 2 if retained original alignment
pub fn calculate_score(
    player: &You,
    starting_gold: i32,
    deepest_level: i32,
    how: DeathType,
) -> i64 {
    let mut score: i64 = 0;

    // Net gold gain
    let gold_gain = (player.gold as i64 - starting_gold as i64).max(0);
    score += gold_gain;

    // Death penalty
    if how.is_death() {
        score -= score / 10;
    }

    // Depth bonus
    score += 50 * (deepest_level as i64 - 1).max(0);

    // Extra depth bonus for deep exploration
    if deepest_level > 20 {
        let extra_depth = (deepest_level - 20).min(10);
        score += 1000 * extra_depth as i64;
    }

    // Ascension bonus
    if how == DeathType::Ascended {
        // Retained original alignment: score *= 2
        // For simplicity, always double (full alignment tracking is complex)
        score *= 2;
    }

    score.max(0)
}

/// Calculate bonus score from artifacts in inventory.
///
/// Matches C `artifact_score()` from end.c. Each artifact contributes
/// a flat bonus to the score.
pub fn artifact_score(inventory: &[crate::object::Object]) -> i64 {
    let mut count: i64 = 0;
    for obj in inventory {
        if obj.is_artifact() {
            // Each artifact is worth a bonus (C uses obj->oclass base cost;
            // we use a flat 2500 per artifact as approximation)
            count += 2500;
        }
    }
    count
}

// ============================================================================
// Disclosure
// ============================================================================

/// End-game disclosure information.
///
/// Collects all the information to display at game end:
/// vanquished creatures, genocided species, inventory, etc.
#[derive(Debug, Clone, Default)]
pub struct Disclosure {
    /// Vanquished creature summary: (name, count) sorted by count descending
    pub vanquished: Vec<(String, u16)>,
    /// Total creatures vanquished
    pub total_vanquished: u64,
    /// Genocided species
    pub genocided: Vec<String>,
    /// Number of extinct species
    pub extinct_count: usize,
}

/// Build vanquished creatures list from monster vitals.
///
/// Matches C `list_vanquished()` from end.c. Collects all monster types
/// with non-zero death counts, sorts by count.
pub fn list_vanquished(
    vitals: &[MonsterVitals],
    monsters_db: &[PerMonst],
) -> Vec<(String, u16)> {
    let mut vanquished: Vec<(String, u16)> = Vec::new();

    for (i, v) in vitals.iter().enumerate() {
        if v.died > 0 && i < monsters_db.len() {
            vanquished.push((monsters_db[i].name.to_string(), v.died));
        }
    }

    // Sort by count descending, then alphabetically
    vanquished.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    vanquished
}

/// Build genocided species list from monster vitals.
///
/// Matches C `list_genocided()` from end.c.
pub fn list_genocided(
    vitals: &[MonsterVitals],
    monsters_db: &[PerMonst],
) -> Vec<String> {
    let mut genocided: Vec<String> = Vec::new();

    for (i, v) in vitals.iter().enumerate() {
        if v.genocided && i < monsters_db.len() {
            genocided.push(monsters_db[i].name.to_string());
        }
    }

    genocided.sort();
    genocided
}

/// Count extinct species (all born, all died, not genocided).
pub fn num_extinct(vitals: &[MonsterVitals]) -> usize {
    vitals
        .iter()
        .filter(|v| !v.genocided && v.born > 0 && v.died >= v.born)
        .count()
}

/// Build full disclosure for end of game.
pub fn build_disclosure(
    vitals: &[MonsterVitals],
    monsters_db: &[PerMonst],
) -> Disclosure {
    let vanquished = list_vanquished(vitals, monsters_db);
    let total_vanquished: u64 = vanquished.iter().map(|(_, c)| *c as u64).sum();
    let genocided = list_genocided(vitals, monsters_db);
    let extinct_count = num_extinct(vitals);

    Disclosure {
        vanquished,
        total_vanquished,
        genocided,
        extinct_count,
    }
}

/// Format a vanquished entry for display.
///
/// Matches C formatting: singular for count=1, plural with count for >1.
/// Unique monsters use "the" prefix and special count phrasing.
pub fn format_vanquished_entry(name: &str, count: u16, is_unique: bool) -> String {
    if is_unique {
        let prefix = if name.starts_with(|c: char| c.is_uppercase()) { "" } else { "the " };
        match count {
            1 => format!("{prefix}{name}"),
            2 => format!("{prefix}{name} (twice)"),
            3 => format!("{prefix}{name} (thrice)"),
            _ => format!("{prefix}{name} ({count} times)"),
        }
    } else {
        match count {
            1 => format!("a {name}"),
            _ => format!("{count:3} {name}s"),
        }
    }
}

/// Format the death message for the score entry.
///
/// Matches C formatting in done_in_by() / formatkiller().
pub fn format_death_message(killer: &Killer, how: DeathType) -> String {
    if killer.name.is_empty() {
        return how.default_message().to_string();
    }

    match killer.format {
        KillerFormat::NoPrefix => killer.name.clone(),
        KillerFormat::KilledBy => format!("killed by {}", killer.name),
        KillerFormat::KilledByAn => format!("killed by a {}", killer.name),
    }
}

// ============================================================================
// Full death processing
// ============================================================================

/// Process player death.
///
/// This is the main entry point matching C `done()` from end.c.
/// Checks life saving, processes the death, calculates score,
/// and builds disclosure.
///
/// Returns None if the player was life-saved (game continues),
/// or Some(GameEndInfo) if the game is truly over.
pub fn process_death(
    player: &mut You,
    inventory: &mut Vec<crate::object::Object>,
    how: DeathType,
    killer: &Killer,
    vitals: &[MonsterVitals],
    monsters_db: &[PerMonst],
    starting_gold: i32,
    deepest_level: i32,
) -> Option<GameEndInfo> {
    // Force HP to 0 for actual deaths
    if how.is_death() && player.hp > 0 {
        player.hp = 0;
    }

    // Check life saving
    if try_life_saving(player, inventory, how) {
        return None; // game continues
    }

    // Build end-game information
    let death_message = format_death_message(killer, how);
    let base_score = calculate_score(player, starting_gold, deepest_level, how);
    let bonus_score = artifact_score(inventory);
    let disclosure = build_disclosure(vitals, monsters_db);

    Some(GameEndInfo {
        how,
        death_message,
        score: base_score + bonus_score,
        disclosure,
        turns: 0, // caller should fill in
        deepest_level,
    })
}

/// Complete game end information.
#[derive(Debug, Clone)]
pub struct GameEndInfo {
    /// How the game ended
    pub how: DeathType,
    /// Formatted death message
    pub death_message: String,
    /// Final score
    pub score: i64,
    /// End-game disclosure
    pub disclosure: Disclosure,
    /// Total turns played
    pub turns: u32,
    /// Deepest level reached
    pub deepest_level: i32,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::empty_attacks;
    use crate::monster::makemon::MonsterVitals;
    use crate::monster::{MonsterFlags, MonsterResistances, MonsterSize, PerMonst};
    use crate::monster::MonsterSound;
    use crate::player::{Gender, Race, Role};

    fn test_player() -> You {
        You::new("Test".into(), Role::Valkyrie, Race::Human, Gender::Female)
    }

    fn test_permonst(name: &'static str, level: i8) -> PerMonst {
        PerMonst {
            name,
            symbol: 'm',
            level,
            move_speed: 12,
            armor_class: 5,
            magic_resistance: 0,
            alignment: 0,
            gen_flags: 0x0020 | 3,
            attacks: empty_attacks(),
            corpse_weight: 100,
            corpse_nutrition: 50,
            sound: MonsterSound::Silent,
            size: MonsterSize::Medium,
            resistances: MonsterResistances::empty(),
            conveys: MonsterResistances::empty(),
            flags: MonsterFlags::empty(),
            difficulty: level.max(0) as u8,
            color: 0,
        }
    }

    // ---- DeathType tests ----

    #[test]
    fn test_death_type_allows_life_saving() {
        assert!(DeathType::Killed.allows_life_saving());
        assert!(DeathType::Choking.allows_life_saving());
        assert!(DeathType::Genocided.allows_life_saving());
        assert!(!DeathType::Panicked.allows_life_saving());
        assert!(!DeathType::Quit.allows_life_saving());
        assert!(!DeathType::Ascended.allows_life_saving());
    }

    #[test]
    fn test_death_type_is_death() {
        assert!(DeathType::Killed.is_death());
        assert!(DeathType::Starving.is_death());
        assert!(!DeathType::Quit.is_death());
        assert!(!DeathType::Escaped.is_death());
        assert!(!DeathType::Ascended.is_death());
    }

    #[test]
    fn test_death_type_allows_bones() {
        assert!(DeathType::Killed.allows_bones());
        assert!(DeathType::Stoning.allows_bones());
        assert!(!DeathType::Genocided.allows_bones());
    }

    // ---- savelife tests ----

    #[test]
    fn test_savelife_restores_hp() {
        let mut player = test_player();
        player.hp = 0;
        player.hp_max = 20;
        player.exp_level = 5;

        savelife(&mut player, DeathType::Killed);

        assert_eq!(player.hp, 20);
    }

    #[test]
    fn test_savelife_minimum_hp() {
        let mut player = test_player();
        player.hp = 0;
        player.hp_max = 5;
        player.exp_level = 8;

        savelife(&mut player, DeathType::Killed);

        // min HP = max(2*8, 10) = 16
        assert_eq!(player.hp_max, 16);
        assert_eq!(player.hp, 16);
    }

    #[test]
    fn test_savelife_cures_hunger() {
        let mut player = test_player();
        player.hp = 0;
        player.hp_max = 20;
        player.exp_level = 1;
        player.nutrition = 100;
        player.hunger_state = HungerState::Weak;

        savelife(&mut player, DeathType::Starving);

        assert_eq!(player.nutrition, 900);
        assert_eq!(player.hunger_state, HungerState::NotHungry);
    }

    #[test]
    fn test_savelife_cures_choking() {
        let mut player = test_player();
        player.hp = 0;
        player.hp_max = 20;
        player.exp_level = 1;
        player.nutrition = 2500;
        player.hunger_state = HungerState::Satiated;

        savelife(&mut player, DeathType::Choking);

        assert_eq!(player.nutrition, 900);
    }

    #[test]
    fn test_savelife_resets_trap() {
        let mut player = test_player();
        player.hp = 0;
        player.hp_max = 20;
        player.exp_level = 1;
        player.utrap = 5;

        savelife(&mut player, DeathType::Killed);

        assert_eq!(player.utrap, 0);
    }

    // ---- try_life_saving tests ----

    #[test]
    fn test_try_life_saving_with_amulet() {
        let mut player = test_player();
        player.hp = 0;
        player.hp_max = 20;
        player.exp_level = 5;
        player.attr_current.set(Attribute::Constitution, 18);

        let mut inventory = Vec::new();
        let mut amulet = crate::object::Object::new(
            crate::object::ObjectId(1), 0, crate::object::ObjectClass::Amulet,
        );
        amulet.name = Some("amulet of life saving".to_string());
        amulet.worn_mask = 1;
        inventory.push(amulet);

        let saved = try_life_saving(&mut player, &mut inventory, DeathType::Killed);

        assert!(saved);
        assert!(player.hp > 0);
        assert!(inventory.is_empty()); // amulet consumed
        // Constitution reduced by 1
        assert_eq!(player.attr_current.get(Attribute::Constitution), 17);
    }

    #[test]
    fn test_try_life_saving_no_amulet() {
        let mut player = test_player();
        player.hp = 0;
        let mut inventory = Vec::new();

        let saved = try_life_saving(&mut player, &mut inventory, DeathType::Killed);

        assert!(!saved);
    }

    #[test]
    fn test_try_life_saving_panicked_not_allowed() {
        let mut player = test_player();
        player.hp = 0;
        player.hp_max = 20;
        player.exp_level = 5;

        let mut inventory = Vec::new();
        let mut amulet = crate::object::Object::new(
            crate::object::ObjectId(1), 0, crate::object::ObjectClass::Amulet,
        );
        amulet.name = Some("amulet of life saving".to_string());
        amulet.worn_mask = 1;
        inventory.push(amulet);

        let saved = try_life_saving(&mut player, &mut inventory, DeathType::Panicked);

        assert!(!saved);
        assert_eq!(inventory.len(), 1); // amulet not consumed
    }

    // ---- Score calculation tests ----

    #[test]
    fn test_calculate_score_basic() {
        let mut player = test_player();
        player.gold = 500;

        let score = calculate_score(&player, 0, 5, DeathType::Killed);

        // 500 gold - 10% death penalty + 50 * 4 depth = 450 + 200 = 650
        assert_eq!(score, 650);
    }

    #[test]
    fn test_calculate_score_ascension() {
        let mut player = test_player();
        player.gold = 1000;

        let score = calculate_score(&player, 0, 25, DeathType::Ascended);

        // 1000 gold + 50*24 depth + 1000*5 extra depth = 1000 + 1200 + 5000 = 7200
        // Ascension doubles: 7200 * 2 = 14400
        assert_eq!(score, 14400);
    }

    #[test]
    fn test_calculate_score_quit() {
        let mut player = test_player();
        player.gold = 100;

        let score = calculate_score(&player, 0, 1, DeathType::Quit);

        // 100 gold, depth 1 = 0 depth bonus, quit = no penalty
        assert_eq!(score, 100);
    }

    #[test]
    fn test_calculate_score_net_gold() {
        let mut player = test_player();
        player.gold = 200;

        // Started with 500 gold, ended with 200
        let score = calculate_score(&player, 500, 1, DeathType::Quit);

        // Net gold gain = max(200-500, 0) = 0
        assert_eq!(score, 0);
    }

    // ---- Disclosure tests ----

    #[test]
    fn test_list_vanquished() {
        let monsters_db = vec![
            test_permonst("kobold", 1),
            test_permonst("goblin", 1),
            test_permonst("orc", 3),
        ];
        let mut vitals = vec![MonsterVitals::default(); 3];
        vitals[0].died = 5;
        vitals[1].died = 10;
        vitals[2].died = 3;

        let vanquished = list_vanquished(&vitals, &monsters_db);

        assert_eq!(vanquished.len(), 3);
        // Sorted by count descending
        assert_eq!(vanquished[0], ("goblin".to_string(), 10));
        assert_eq!(vanquished[1], ("kobold".to_string(), 5));
        assert_eq!(vanquished[2], ("orc".to_string(), 3));
    }

    #[test]
    fn test_list_vanquished_empty() {
        let monsters_db = vec![test_permonst("kobold", 1)];
        let vitals = vec![MonsterVitals::default()];

        let vanquished = list_vanquished(&vitals, &monsters_db);
        assert!(vanquished.is_empty());
    }

    #[test]
    fn test_list_genocided() {
        let monsters_db = vec![
            test_permonst("kobold", 1),
            test_permonst("goblin", 1),
            test_permonst("orc", 3),
        ];
        let mut vitals = vec![MonsterVitals::default(); 3];
        vitals[1].genocided = true;

        let genocided = list_genocided(&vitals, &monsters_db);

        assert_eq!(genocided, vec!["goblin".to_string()]);
    }

    #[test]
    fn test_num_extinct() {
        let mut vitals = vec![MonsterVitals::default(); 3];
        // Species 0: born 5, died 5 = extinct
        vitals[0].born = 5;
        vitals[0].died = 5;
        // Species 1: born 3, died 2 = not extinct
        vitals[1].born = 3;
        vitals[1].died = 2;
        // Species 2: genocided = not counted as extinct
        vitals[2].born = 1;
        vitals[2].died = 1;
        vitals[2].genocided = true;

        assert_eq!(num_extinct(&vitals), 1);
    }

    // ---- Format tests ----

    #[test]
    fn test_format_vanquished_unique() {
        assert_eq!(
            format_vanquished_entry("Medusa", 1, true),
            "Medusa"
        );
        assert_eq!(
            format_vanquished_entry("Medusa", 2, true),
            "Medusa (twice)"
        );
        assert_eq!(
            format_vanquished_entry("lich king", 1, true),
            "the lich king"
        );
    }

    #[test]
    fn test_format_vanquished_normal() {
        assert_eq!(
            format_vanquished_entry("kobold", 1, false),
            "a kobold"
        );
        assert_eq!(
            format_vanquished_entry("kobold", 5, false),
            "  5 kobolds"
        );
    }

    #[test]
    fn test_format_death_message() {
        let killer = Killer {
            name: "dragon".to_string(),
            format: KillerFormat::KilledByAn,
        };
        assert_eq!(
            format_death_message(&killer, DeathType::Killed),
            "killed by a dragon"
        );

        let killer = Killer {
            name: "".to_string(),
            format: KillerFormat::KilledByAn,
        };
        assert_eq!(
            format_death_message(&killer, DeathType::Starving),
            "starved"
        );
    }

    // ---- artifact_score tests ----

    #[test]
    fn test_artifact_score_none() {
        let inventory: Vec<crate::object::Object> = Vec::new();
        assert_eq!(artifact_score(&inventory), 0);
    }

    #[test]
    fn test_artifact_score_with_artifact() {
        let mut obj = crate::object::Object::new(
            crate::object::ObjectId(1), 0, crate::object::ObjectClass::Weapon,
        );
        obj.artifact = 1; // mark as artifact
        let inventory = vec![obj];

        assert_eq!(artifact_score(&inventory), 2500);
    }

    // ---- build_disclosure tests ----

    #[test]
    fn test_build_disclosure() {
        let monsters_db = vec![
            test_permonst("kobold", 1),
            test_permonst("goblin", 1),
        ];
        let mut vitals = vec![MonsterVitals::default(); 2];
        vitals[0].died = 3;
        vitals[0].born = 3;
        vitals[1].genocided = true;

        let disclosure = build_disclosure(&vitals, &monsters_db);

        assert_eq!(disclosure.vanquished.len(), 1);
        assert_eq!(disclosure.total_vanquished, 3);
        assert_eq!(disclosure.genocided.len(), 1);
        assert_eq!(disclosure.extinct_count, 1);
    }

    // ---- process_death tests ----

    #[test]
    fn test_process_death_life_saved() {
        let mut player = test_player();
        player.hp = 0;
        player.hp_max = 20;
        player.exp_level = 5;
        player.attr_current.set(Attribute::Constitution, 15);

        let mut inventory = Vec::new();
        let mut amulet = crate::object::Object::new(
            crate::object::ObjectId(1), 0, crate::object::ObjectClass::Amulet,
        );
        amulet.name = Some("amulet of life saving".to_string());
        amulet.worn_mask = 1;
        inventory.push(amulet);

        let killer = Killer::default();
        let result = process_death(
            &mut player, &mut inventory, DeathType::Killed, &killer,
            &[], &[], 0, 5,
        );

        assert!(result.is_none()); // life saved
        assert!(player.hp > 0);
    }

    #[test]
    fn test_process_death_actual_death() {
        let mut player = test_player();
        player.hp = 0;
        player.hp_max = 20;
        player.gold = 100;

        let mut inventory = Vec::new();
        let killer = Killer {
            name: "dragon".to_string(),
            format: KillerFormat::KilledByAn,
        };
        let monsters_db = vec![test_permonst("kobold", 1)];
        let mut vitals = vec![MonsterVitals::default()];
        vitals[0].died = 5;

        let result = process_death(
            &mut player, &mut inventory, DeathType::Killed, &killer,
            &vitals, &monsters_db, 0, 5,
        );

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.how, DeathType::Killed);
        assert_eq!(info.death_message, "killed by a dragon");
        assert!(info.score > 0);
        assert_eq!(info.disclosure.vanquished.len(), 1);
    }

    #[test]
    fn test_process_death_ascension() {
        let mut player = test_player();
        player.hp = 50;
        player.hp_max = 50;
        player.gold = 10000;

        let mut inventory = Vec::new();
        let killer = Killer {
            name: "".to_string(),
            format: KillerFormat::NoPrefix,
        };

        let result = process_death(
            &mut player, &mut inventory, DeathType::Ascended, &killer,
            &[], &[], 0, 30,
        );

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.how, DeathType::Ascended);
        assert!(info.score > 0);
    }
}
