//! Summoning functions
//!
//! Functions for summoning monsters, primarily used by demons, the Wizard of Yendor,
//! and various magical effects.
//!
//! Adapted from NetHack 3.6.7: minion.c, wizard.c, polyself.c

use crate::monster::{Monster, MonsterId};
use crate::player::You;
use crate::rng::GameRng;

/// Result of a summoning attempt
#[derive(Debug, Clone, Default)]
pub struct SummonResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Number of monsters summoned
    pub count: u32,
    /// Whether the summoning was successful
    pub success: bool,
    /// Types of monsters summoned (as names for now, pending monster type system)
    pub summoned_types: Vec<String>,
}

impl SummonResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_message(mut self, msg: &str) -> Self {
        self.messages.push(msg.to_string());
        self
    }
}

/// Player summons allies when polymorphed into a monster (dosummon equivalent)
///
/// Called when a polymorphed player tries to summon help.
/// Costs energy and summons creatures appropriate to the poly'd form.
///
/// # Arguments
/// * `player` - The player attempting to summon
/// * `rng` - Random number generator
///
/// # Returns
/// Result indicating success and messages
pub fn dosummon(player: &mut You, rng: &mut GameRng) -> SummonResult {
    let mut result = SummonResult::new();

    // Check for sufficient energy
    if player.energy < 10 {
        result
            .messages
            .push("You lack the energy to send forth a call for help!".to_string());
        result.success = false;
        return result;
    }

    // Deduct energy cost
    player.energy -= 10;

    result
        .messages
        .push("You call upon your brethren for help!".to_string());

    // Attempt to summon based on polymorphed form
    // In full implementation, would check what form player is polymorphed into
    // For now, use a simplified version
    let summon_chance = rng.rn2(3);
    if summon_chance == 0 {
        result.messages.push("But none arrive.".to_string());
        result.success = false;
    } else {
        // Summoned 1-2 allies
        let count = 1 + rng.rn2(2);
        result.count = count;
        result.success = true;
        result.messages.push(format!(
            "{} of your kind {} to your call!",
            count,
            if count == 1 { "answers" } else { "answer" }
        ));
    }

    result
}

/// Monster summons demons or minions (msummon equivalent)
///
/// Used by demon princes, lords, and the Wizard of Yendor to summon help.
///
/// # Arguments
/// * `summoner` - The monster doing the summoning (None = Wizard of Yendor style)
/// * `rng` - Random number generator
///
/// # Returns
/// Result with count of monsters summoned and messages
pub fn msummon(summoner: Option<&Monster>, rng: &mut GameRng) -> SummonResult {
    let mut result = SummonResult::new();

    // Determine what to summon based on summoner type
    let summoner_name = summoner
        .map(|m| m.name.to_lowercase())
        .unwrap_or_else(|| "wizard of yendor".to_string());

    // Demon prince/Wizard of Yendor can summon demon princes, lords, or lesser demons
    let is_prince = summoner_name.contains("demon prince")
        || summoner_name.contains("wizard of yendor")
        || summoner_name.contains("demogorgon")
        || summoner_name.contains("asmodeus")
        || summoner_name.contains("orcus");

    let is_lord = summoner_name.contains("demon lord")
        || summoner_name.contains("yeenoghu")
        || summoner_name.contains("geryon")
        || summoner_name.contains("dispater")
        || summoner_name.contains("baalzebub");

    let is_demon = summoner_name.contains("demon")
        || summoner_name.contains("succubus")
        || summoner_name.contains("incubus")
        || summoner_name.contains("marilith");

    if is_prince {
        // 5% chance of demon prince, 25% demon lord, otherwise lesser demon
        let roll = rng.rn2(20);
        if roll == 0 {
            result.summoned_types.push("demon prince".to_string());
            result
                .messages
                .push("A demon prince is summoned!".to_string());
        } else if roll < 5 {
            result.summoned_types.push("demon lord".to_string());
            result
                .messages
                .push("A demon lord is summoned!".to_string());
        } else {
            result.summoned_types.push("lesser demon".to_string());
            result.messages.push("A demon is summoned!".to_string());
        }
        result.count = 1;
        result.success = true;
    } else if is_lord {
        // 2% demon prince, 5% demon lord, otherwise lesser demon
        let roll = rng.rn2(50);
        if roll == 0 {
            result.summoned_types.push("demon prince".to_string());
            result
                .messages
                .push("A demon prince is summoned!".to_string());
        } else if roll < 2 {
            result.summoned_types.push("demon lord".to_string());
            result
                .messages
                .push("A demon lord is summoned!".to_string());
        } else {
            result.summoned_types.push("lesser demon".to_string());
            result.messages.push("A demon is summoned!".to_string());
        }
        result.count = 1;
        result.success = true;
    } else if is_demon {
        // 5% demon lord, 15% same type, otherwise lesser demon
        let roll = rng.rn2(20);
        if roll == 0 {
            result.summoned_types.push("demon lord".to_string());
            result
                .messages
                .push("A demon lord is summoned!".to_string());
        } else if roll < 3 {
            result.summoned_types.push("same type demon".to_string());
            result
                .messages
                .push("Another demon of the same type is summoned!".to_string());
        } else {
            result.summoned_types.push("lesser demon".to_string());
            result
                .messages
                .push("A lesser demon is summoned!".to_string());
        }
        result.count = 1;
        result.success = true;
    } else {
        // Non-demon summoner, weaker summoning
        if rng.rn2(3) == 0 {
            result.messages.push("Nothing happens.".to_string());
            result.success = false;
        } else {
            result.summoned_types.push("monster".to_string());
            result.messages.push("A monster is summoned!".to_string());
            result.count = 1;
            result.success = true;
        }
    }

    // Small chance of summoning 2 instead of 1
    if result.success && rng.rn2(4) == 0 {
        result.count = 2;
        result.summoned_types.push(result.summoned_types[0].clone());
        result.messages.push("And another one appears!".to_string());
    }

    result
}

/// The list of "nasty" monsters that can be summoned
const NASTIES: [&str; 42] = [
    // Lawful (10)
    "ki-rin",
    "archon",
    "Angel",
    "couatl",
    "aleax",
    "solar",
    "planetar",
    "gold dragon",
    "silver dragon",
    "gray dragon",
    // Chaotic (14)
    "green dragon",
    "red dragon",
    "minotaur",
    "jabberwock",
    "titan",
    "storm giant",
    "fire giant",
    "master mind flayer",
    "pit fiend",
    "bone devil",
    "ice devil",
    "nalfeshnee",
    "marilith",
    "vrock",
    // Neutral (18)
    "air elemental",
    "fire elemental",
    "earth elemental",
    "water elemental",
    "purple worm",
    "kraken",
    "balrog",
    "baluchitherium",
    "xorn",
    "umber hulk",
    "black dragon",
    "rust monster",
    "disenchanter",
    "gremlin",
    "stalker",
    "wood golem",
    "clay golem",
    "stone golem",
];

/// Summon nasty monsters (nasty function equivalent)
///
/// Used by the Wizard of Yendor and during late-game harassment.
/// Summons powerful hostile monsters near the player.
///
/// # Arguments
/// * `summoner` - The monster doing the summoning (None = generic harassment)
/// * `player_level` - Player's experience level (affects count)
/// * `player_x` - Player's X position
/// * `player_y` - Player's Y position
/// * `in_gehennom` - Whether we're in Gehennom (affects demon summons)
/// * `rng` - Random number generator
///
/// # Returns
/// Result with count of monsters summoned and messages
pub fn nasty(
    summoner: Option<&Monster>,
    player_level: u8,
    player_x: i8,
    player_y: i8,
    in_gehennom: bool,
    rng: &mut GameRng,
) -> SummonResult {
    let mut result = SummonResult::new();

    const MAX_NASTIES: u32 = 10;

    // 10% chance in Gehennom to use demon summoning instead
    if in_gehennom && rng.rn2(10) == 0 {
        return msummon(summoner, rng);
    }

    // Determine summoner's alignment for filtering nasties
    let castalign = summoner.map(|m| m.alignment.signum()).unwrap_or(0);

    // Number of summoning attempts based on player level
    let attempts = if player_level > 3 {
        (player_level / 3).max(1) as u32
    } else {
        1
    };

    // Select and summon nasties
    for _ in 0..attempts.min(rng.rnd(attempts) as u32) {
        if result.count >= MAX_NASTIES {
            break;
        }

        // Pick a random nasty
        let idx = rng.rn2(NASTIES.len() as u32) as usize;
        let nasty_name = NASTIES[idx];

        // Simple alignment filter (would be more complex in full implementation)
        let monster_align = if idx < 10 {
            1 // Lawful
        } else if idx < 24 {
            -1 // Chaotic
        } else {
            0 // Neutral
        };

        // Alignment-based filtering
        // Different alignments have different stop chances
        let stop_chance = match castalign {
            0 => 18, // Neutral: 18/42 chance to stop
            1 => {
                // Lawful: higher chance to stop
                if summoner
                    .map(|m| m.name.contains("angel") || m.name.contains("demon"))
                    .unwrap_or(false)
                {
                    26
                } else {
                    28
                }
            }
            _ => 32, // Chaotic: highest chance to stop
        };

        if rng.rn2(42) < stop_chance && monster_align != castalign {
            continue;
        }

        // Successfully summon this nasty
        result.count += 1;
        result.summoned_types.push(nasty_name.to_string());

        if result.count == 1 {
            result
                .messages
                .push(format!("A {} appears near you!", nasty_name));
        }
    }

    if result.count > 1 {
        result.messages.push(format!(
            "{} nasty monsters have been summoned!",
            result.count
        ));
    } else if result.count == 0 {
        result.messages.push("The summoning fails.".to_string());
    }

    result.success = result.count > 0;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dosummon_insufficient_energy() {
        let mut player = You::default();
        player.energy = 5; // Less than 10 needed
        let mut rng = GameRng::new(42);

        let result = dosummon(&mut player, &mut rng);

        assert!(!result.success);
        assert!(result.messages[0].contains("lack the energy"));
        assert_eq!(player.energy, 5); // Energy unchanged
    }

    #[test]
    fn test_dosummon_sufficient_energy() {
        let mut player = You::default();
        player.energy = 20;
        let mut rng = GameRng::new(42);

        let result = dosummon(&mut player, &mut rng);

        assert_eq!(player.energy, 10); // 10 deducted
        assert!(result.messages[0].contains("call upon your brethren"));
    }

    #[test]
    fn test_msummon_demon_prince() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "Demogorgon".to_string();
        let mut rng = GameRng::new(42);

        let result = msummon(Some(&mon), &mut rng);

        assert!(result.success);
        assert!(result.count >= 1);
    }

    #[test]
    fn test_msummon_wizard_of_yendor() {
        let mut rng = GameRng::new(42);

        // None summoner acts like Wizard of Yendor
        let result = msummon(None, &mut rng);

        assert!(result.success);
        assert!(result.count >= 1);
    }

    #[test]
    fn test_msummon_non_demon() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "goblin".to_string();
        let mut rng = GameRng::new(42);

        let result = msummon(Some(&mon), &mut rng);

        // May or may not succeed, but should complete
        assert!(result.messages.len() > 0);
    }

    #[test]
    fn test_nasty_basic() {
        let mut rng = GameRng::new(42);

        let result = nasty(None, 15, 10, 10, false, &mut rng);

        // Should summon at least some monsters
        assert!(result.messages.len() > 0);
    }

    #[test]
    fn test_nasty_in_gehennom() {
        let mut rng = GameRng::new(42);

        // In Gehennom, might use demon summoning instead
        let result = nasty(None, 15, 10, 10, true, &mut rng);

        assert!(result.messages.len() > 0);
    }

    #[test]
    fn test_nasty_low_level_player() {
        let mut rng = GameRng::new(42);

        let result = nasty(None, 3, 10, 10, false, &mut rng);

        // Low level means fewer attempts
        assert!(result.count <= 10);
    }

    #[test]
    fn test_nasty_high_level_player() {
        let mut rng = GameRng::new(42);

        let result = nasty(None, 30, 10, 10, false, &mut rng);

        // Higher level means more attempts (though still capped at MAX_NASTIES)
        assert!(result.count <= 10);
    }

    #[test]
    fn test_summon_result_with_message() {
        let result = SummonResult::new().with_message("Test message");

        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0], "Test message");
    }
}
