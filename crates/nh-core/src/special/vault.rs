//! Vault guard system (vault.c)
//!
//! Implements vault guards who protect gold vaults and demand
//! that players identify themselves and leave.

use crate::dungeon::Level;
use crate::gameloop::GameState;
use crate::monster::{Monster, MonsterId, MonsterState};
use crate::rng::GameRng;

/// Monster type index for vault guard
const PM_GUARD: i16 = 150;

/// Vault guard state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GuardState {
    /// Guard is not present
    #[default]
    Absent,
    /// Guard is approaching the vault
    Approaching,
    /// Guard is demanding identification
    Demanding,
    /// Guard is escorting player out
    Escorting,
    /// Guard is angry (player refused or attacked)
    Angry,
    /// Guard has been satisfied and left
    Satisfied,
}

/// Vault data for tracking guard interactions
#[derive(Debug, Clone)]
pub struct Vault {
    /// Vault room bounds (x1, y1, x2, y2)
    pub bounds: (i8, i8, i8, i8),
    /// Current guard state
    pub guard_state: GuardState,
    /// Guard monster ID (if present)
    pub guard_id: Option<MonsterId>,
    /// Number of times player has been warned
    pub warnings: u8,
    /// Whether player has given a name
    pub player_identified: bool,
    /// Name the player gave (may be fake)
    pub given_name: Option<String>,
    /// Turn when guard was summoned
    pub summon_turn: u64,
}

impl Vault {
    /// Create a new vault
    pub fn new(bounds: (i8, i8, i8, i8)) -> Self {
        Self {
            bounds,
            guard_state: GuardState::Absent,
            guard_id: None,
            warnings: 0,
            player_identified: false,
            given_name: None,
            summon_turn: 0,
        }
    }

    /// Check if a position is inside the vault
    pub fn contains(&self, x: i8, y: i8) -> bool {
        x >= self.bounds.0
            && x <= self.bounds.2
            && y >= self.bounds.1
            && y <= self.bounds.3
    }

    /// Get the center of the vault
    pub fn center(&self) -> (i8, i8) {
        (
            (self.bounds.0 + self.bounds.2) / 2,
            (self.bounds.1 + self.bounds.3) / 2,
        )
    }
}

/// Create a vault guard monster
pub fn create_guard(x: i8, y: i8, rng: &mut GameRng) -> Monster {
    let mut guard = Monster::new(MonsterId::NONE, PM_GUARD, x, y);

    guard.state = MonsterState::peaceful();
    guard.is_guard = true;
    guard.hp = 50 + rng.rnd(30) as i32;
    guard.hp_max = guard.hp;
    guard.level = 12;
    guard.name = generate_guard_name(rng);

    guard
}

/// Generate a guard name
fn generate_guard_name(rng: &mut GameRng) -> String {
    let names = [
        "Croesus",
        "Midas",
        "Plutus",
        "Mammon",
        "Dives",
        "Scrooge",
        "Goldfinger",
        "Richie",
    ];
    let idx = rng.rn2(names.len() as u32) as usize;
    names[idx].to_string()
}

/// Result of guard interaction
#[derive(Debug, Clone)]
pub enum GuardInteraction {
    /// Guard demands identification
    DemandId { message: String },
    /// Guard accepts the name
    AcceptName { message: String },
    /// Guard is suspicious of fake name
    Suspicious { message: String },
    /// Guard escorts player out
    Escort { message: String },
    /// Guard becomes angry
    Angry { message: String },
    /// Guard leaves satisfied
    Leave { message: String },
    /// No interaction needed
    None,
}

/// Handle player entering a vault
pub fn player_enters_vault(
    vault: &mut Vault,
    current_turn: u64,
) -> GuardInteraction {
    if vault.guard_state != GuardState::Absent {
        return GuardInteraction::None;
    }

    // Summon a guard
    vault.guard_state = GuardState::Approaching;
    vault.summon_turn = current_turn;

    GuardInteraction::None // Guard will arrive next turn
}

/// Handle guard arriving at vault
pub fn guard_arrives(
    _state: &mut GameState,
    vault: &mut Vault,
    level: &mut Level,
    rng: &mut GameRng,
) -> GuardInteraction {
    if vault.guard_state != GuardState::Approaching {
        return GuardInteraction::None;
    }

    // Find a position for the guard near the vault entrance
    let (gx, gy) = find_guard_position(level, vault, rng);

    // Create and add the guard
    let guard = create_guard(gx, gy, rng);
    let guard_name = guard.name.clone();
    let guard_id = level.add_monster(guard);
    vault.guard_id = Some(guard_id);
    vault.guard_state = GuardState::Demanding;

    GuardInteraction::DemandId {
        message: format!(
            "{} the guard appears! \"Halt! Who goes there?\"",
            guard_name
        ),
    }
}

/// Find a position for the guard to appear
fn find_guard_position(level: &Level, vault: &Vault, rng: &mut GameRng) -> (i8, i8) {
    // Try to find a position just outside the vault
    let (x1, y1, x2, y2) = vault.bounds;

    let mut candidates = Vec::new();

    // Check positions around the vault
    for x in (x1 - 2)..=(x2 + 2) {
        for y in (y1 - 2)..=(y2 + 2) {
            // Skip inside the vault
            if vault.contains(x, y) {
                continue;
            }

            if level.is_valid_pos(x, y)
                && level.is_walkable(x, y)
                && level.monster_at(x, y).is_none()
            {
                candidates.push((x, y));
            }
        }
    }

    if candidates.is_empty() {
        // Fallback to vault center
        vault.center()
    } else {
        let idx = rng.rn2(candidates.len() as u32) as usize;
        candidates[idx]
    }
}

/// Handle player giving their name to the guard
pub fn give_name_to_guard(
    vault: &mut Vault,
    name: &str,
    is_real_name: bool,
    rng: &mut GameRng,
) -> GuardInteraction {
    if vault.guard_state != GuardState::Demanding {
        return GuardInteraction::None;
    }

    vault.player_identified = true;
    vault.given_name = Some(name.to_string());

    if is_real_name {
        vault.guard_state = GuardState::Escorting;
        GuardInteraction::AcceptName {
            message: format!(
                "\"Very well, {}. I'll escort you out. Follow me.\"",
                name
            ),
        }
    } else {
        // Guard might be suspicious of fake names
        if rng.one_in(3) {
            vault.guard_state = GuardState::Angry;
            GuardInteraction::Suspicious {
                message: "\"That name sounds fake! You're under arrest!\"".to_string(),
            }
        } else {
            vault.guard_state = GuardState::Escorting;
            GuardInteraction::AcceptName {
                message: format!(
                    "\"Alright, {}. Come with me.\"",
                    name
                ),
            }
        }
    }
}

/// Handle player refusing to identify
pub fn refuse_identification(vault: &mut Vault) -> GuardInteraction {
    vault.warnings += 1;

    if vault.warnings >= 3 {
        vault.guard_state = GuardState::Angry;
        GuardInteraction::Angry {
            message: "\"That's it! You're under arrest!\"".to_string(),
        }
    } else {
        GuardInteraction::DemandId {
            message: format!(
                "\"I said, who goes there?! ({} warning)\"",
                match vault.warnings {
                    1 => "first",
                    2 => "second",
                    _ => "final",
                }
            ),
        }
    }
}

/// Handle player following the guard out
pub fn follow_guard(
    vault: &mut Vault,
    player_in_vault: bool,
) -> GuardInteraction {
    if vault.guard_state != GuardState::Escorting {
        return GuardInteraction::None;
    }

    if !player_in_vault {
        // Player has left the vault
        vault.guard_state = GuardState::Satisfied;
        GuardInteraction::Leave {
            message: "\"Good. Don't let me catch you in there again.\"".to_string(),
        }
    } else {
        GuardInteraction::Escort {
            message: "\"This way. Keep moving.\"".to_string(),
        }
    }
}

/// Handle player attacking the guard
pub fn attack_guard(vault: &mut Vault, level: &mut Level) -> GuardInteraction {
    vault.guard_state = GuardState::Angry;

    // Make guard hostile
    if let Some(guard_id) = vault.guard_id {
        if let Some(guard) = level.monster_mut(guard_id) {
            guard.state.peaceful = false;
        }
    }

    GuardInteraction::Angry {
        message: "\"You'll pay for that!\"".to_string(),
    }
}

/// Handle guard leaving after escort
pub fn guard_leaves(vault: &mut Vault, level: &mut Level) {
    if vault.guard_state != GuardState::Satisfied {
        return;
    }

    // Remove the guard
    if let Some(guard_id) = vault.guard_id {
        level.remove_monster(guard_id);
    }

    vault.guard_id = None;
    vault.guard_state = GuardState::Absent;
    vault.warnings = 0;
    vault.player_identified = false;
    vault.given_name = None;
}

/// Check if player has gold from the vault
pub fn player_has_vault_gold(state: &GameState) -> bool {
    // In a full implementation, we'd track which gold came from the vault
    // For now, just check if player has significant gold
    state.player.gold > 100
}

/// Guard demands player drop vault gold
pub fn demand_gold(state: &mut GameState, vault: &mut Vault) -> GuardInteraction {
    if !player_has_vault_gold(state) {
        return GuardInteraction::None;
    }

    vault.warnings += 1;

    GuardInteraction::DemandId {
        message: "\"Drop that gold! It belongs to the vault!\"".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_creation() {
        let vault = Vault::new((10, 10, 15, 15));

        assert!(vault.contains(12, 12));
        assert!(!vault.contains(5, 5));
        assert_eq!(vault.guard_state, GuardState::Absent);
    }

    #[test]
    fn test_vault_center() {
        let vault = Vault::new((10, 10, 20, 20));
        assert_eq!(vault.center(), (15, 15));
    }

    #[test]
    fn test_create_guard() {
        let mut rng = GameRng::new(42);
        let guard = create_guard(10, 10, &mut rng);

        assert!(guard.state.peaceful);
        assert!(guard.is_guard);
        assert!(guard.hp > 0);
        assert!(!guard.name.is_empty());
    }

    #[test]
    fn test_player_enters_vault() {
        let mut vault = Vault::new((10, 10, 15, 15));

        let result = player_enters_vault(&mut vault, 100);
        assert!(matches!(result, GuardInteraction::None));
        assert_eq!(vault.guard_state, GuardState::Approaching);
        assert_eq!(vault.summon_turn, 100);
    }

    #[test]
    fn test_refuse_identification() {
        let mut vault = Vault::new((10, 10, 15, 15));
        vault.guard_state = GuardState::Demanding;

        // First warning
        let result = refuse_identification(&mut vault);
        assert!(matches!(result, GuardInteraction::DemandId { .. }));
        assert_eq!(vault.warnings, 1);

        // Second warning
        let result = refuse_identification(&mut vault);
        assert!(matches!(result, GuardInteraction::DemandId { .. }));
        assert_eq!(vault.warnings, 2);

        // Third warning - guard gets angry
        let result = refuse_identification(&mut vault);
        assert!(matches!(result, GuardInteraction::Angry { .. }));
        assert_eq!(vault.guard_state, GuardState::Angry);
    }

    #[test]
    fn test_give_real_name() {
        let mut vault = Vault::new((10, 10, 15, 15));
        vault.guard_state = GuardState::Demanding;
        let mut rng = GameRng::new(42);

        let result = give_name_to_guard(&mut vault, "Hero", true, &mut rng);

        assert!(matches!(result, GuardInteraction::AcceptName { .. }));
        assert_eq!(vault.guard_state, GuardState::Escorting);
        assert!(vault.player_identified);
        assert_eq!(vault.given_name, Some("Hero".to_string()));
    }

    #[test]
    fn test_follow_guard_out() {
        let mut vault = Vault::new((10, 10, 15, 15));
        vault.guard_state = GuardState::Escorting;

        // Still in vault
        let result = follow_guard(&mut vault, true);
        assert!(matches!(result, GuardInteraction::Escort { .. }));

        // Left vault
        let result = follow_guard(&mut vault, false);
        assert!(matches!(result, GuardInteraction::Leave { .. }));
        assert_eq!(vault.guard_state, GuardState::Satisfied);
    }
}
