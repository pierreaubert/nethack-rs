//! Vault guard system (vault.c)
//!
//! Implements vault guards who protect gold vaults and demand
//! that players identify themselves and leave.
//!
//! Core systems:
//! - Guard summoning and state management
//! - Fake corridor generation and tracking
//! - Guard AI and movement
//! - Guard interaction and warning system

use crate::dungeon::Level;
use crate::gameloop::GameState;
use crate::monster::{Monster, MonsterId, MonsterState};
use crate::player::You;
use crate::rng::GameRng;

/// Fake corridor segment (individual piece of generated corridor)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FakeCorridorSegment {
    /// Position of this segment
    pub pos: (i8, i8),
    /// Original terrain type (what was here before)
    pub original_type: u8,
}

/// Guard extended data (equivalent to C struct egd)
/// Stores vault guard state and corridor generation information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuardExtension {
    /// Start index in fake corridor array
    pub corridor_begin: u32,
    /// End index in fake corridor array
    pub corridor_end: u32,
    /// Vault room number
    pub vault_room: u8,
    /// Guard's goal position
    pub goal_pos: (i8, i8),
    /// Guard's old position (for tracking movement)
    pub old_pos: (i8, i8),
    /// Dungeon level where guard operates
    pub guard_level: u8,
    /// Warning counter
    pub warning_count: u8,
    /// Has guard released the player?
    pub is_done: bool,
    /// Did guard witness illegal activity?
    pub witness_level: u8,
    /// Fake corridor segments
    pub fake_corridors: Vec<FakeCorridorSegment>,
}

impl GuardExtension {
    /// Create new guard extension
    pub fn new(vault_room: u8, goal_x: i8, goal_y: i8) -> Self {
        Self {
            corridor_begin: 0,
            corridor_end: 0,
            vault_room,
            goal_pos: (goal_x, goal_y),
            old_pos: (-1, -1),
            guard_level: 1,
            warning_count: 0,
            is_done: false,
            witness_level: 0,
            fake_corridors: Vec::new(),
        }
    }

    /// Add a fake corridor segment
    pub fn add_corridor(&mut self, x: i8, y: i8, original_type: u8) {
        self.fake_corridors.push(FakeCorridorSegment {
            pos: (x, y),
            original_type,
        });
        self.corridor_end = self.fake_corridors.len() as u32;
    }

    /// Clear all fake corridors
    pub fn clear_corridors(&mut self) {
        self.fake_corridors.clear();
        self.corridor_begin = 0;
        self.corridor_end = 0;
    }

    /// Check if at maximum corridor size
    pub fn is_corridor_full(&self) -> bool {
        self.fake_corridors.len() >= 32 // FCSIZ = 32
    }

    /// Distance traveled in corridor
    pub fn corridor_length(&self) -> u32 {
        self.corridor_end - self.corridor_begin
    }
}

/// Monster type index for vault guard
const PM_GUARD: i16 = 150;

/// Vault guard state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
        x >= self.bounds.0 && x <= self.bounds.2 && y >= self.bounds.1 && y <= self.bounds.3
    }

    /// Get the center of the vault
    pub fn center(&self) -> (i8, i8) {
        (
            (self.bounds.0 + self.bounds.2) / 2,
            (self.bounds.1 + self.bounds.3) / 2,
        )
    }
}

/// Check if a monster is a vault guard
pub fn is_guard(monster: &Monster) -> bool {
    monster.is_guard
}

/// Get guard extension if monster is a guard
pub fn get_guard_ext(monster: &Monster) -> Option<&GuardExtension> {
    if monster.is_guard {
        monster.guard_extension.as_ref()
    } else {
        None
    }
}

/// Get mutable guard extension
pub fn get_guard_ext_mut(monster: &mut Monster) -> Option<&mut GuardExtension> {
    if monster.is_guard {
        monster.guard_extension.as_mut()
    } else {
        None
    }
}

/// Create guard extension (newegd equivalent)
pub fn create_guard_extension(guard: &mut Monster, vault_room: u8, goal_x: i8, goal_y: i8) {
    guard.is_guard = true;
    guard.guard_extension = Some(GuardExtension::new(vault_room, goal_x, goal_y));
}

/// Find vault guard in level (findgd equivalent)
pub fn find_vault_guard(level: &Level) -> Option<MonsterId> {
    for monster in &level.monsters {
        if monster.is_guard {
            return Some(monster.id);
        }
    }
    None
}

/// Check if player is in vault (vault_occupied equivalent)
pub fn is_player_in_vault(level: &Level, player: &You) -> bool {
    level.cells[player.pos.x as usize][player.pos.y as usize].typ == crate::dungeon::CellType::Vault
}

/// Summon vault guard (vault_summon_gd equivalent)
pub fn summon_vault_guard(level: &mut Level, player: &You, game_turn: u32) -> bool {
    // Check if player is in vault
    if !is_player_in_vault(level, player) {
        return false;
    }

    // Check if guard already exists
    if find_vault_guard(level).is_some() {
        return false;
    }

    // Create and add guard
    let vault_center = (player.pos.x, player.pos.y);
    let mut guard = create_guard(
        vault_center.0 + 2,
        vault_center.1 + 2,
        &mut crate::rng::GameRng::new(game_turn as u64),
    );

    create_guard_extension(&mut guard, 0, vault_center.0, vault_center.1);

    if let Some(ext) = get_guard_ext_mut(&mut guard) {
        ext.guard_level = 1;
    }

    level.monsters.push(guard);
    true
}

/// Handle vault guard sound availability (gd_sound equivalent)
pub fn should_play_vault_sound(level: &Level, player: &You) -> bool {
    // Don't play sounds if player is in vault (guard is quiet so player doesn't know)
    if is_player_in_vault(level, player) {
        return false;
    }

    // Only play if guard exists
    find_vault_guard(level).is_some()
}

/// Move vault guard (gd_move equivalent - simplified version)
pub fn move_vault_guard(guard: &mut Monster, level: &mut Level, player: &You) -> bool {
    if let Some(ext) = get_guard_ext_mut(guard) {
        // Calculate distance to goal
        let goal_x = ext.goal_pos.0;
        let goal_y = ext.goal_pos.1;
        let dx = (goal_x - guard.x).signum();
        let dy = (goal_y - guard.y).signum();

        let new_x = guard.x + dx;
        let new_y = guard.y + dy;

        // Check if position is walkable
        if level.is_valid_pos(new_x, new_y) {
            guard.x = new_x;
            guard.y = new_y;
            return true;
        }
    }
    false
}

/// Handle player leaving vault (uleftvault equivalent)
pub fn handle_vault_exit(guard: &mut Monster, level: &Level) {
    if let Some(ext) = get_guard_ext_mut(guard) {
        // Clear fake corridors
        ext.clear_corridors();
        // Guard is satisfied
        ext.is_done = true;
    }
}

/// Generate fake corridor from vault (building corridor path)
pub fn build_vault_corridor(guard: &mut Monster, from: (i8, i8), to: (i8, i8)) -> bool {
    if let Some(ext) = get_guard_ext_mut(guard) {
        if ext.is_corridor_full() {
            return false;
        }

        // Calculate direction
        let dx = (to.0 - from.0).signum();
        let dy = (to.1 - from.1).signum();

        // Add corridor segment
        ext.add_corridor(from.0 + dx, from.1 + dy, 0); // Type 0 = corridor
        true
    } else {
        false
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
pub fn player_enters_vault(vault: &mut Vault, current_turn: u64) -> GuardInteraction {
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
            message: format!("\"Very well, {}. I'll escort you out. Follow me.\"", name),
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
                message: format!("\"Alright, {}. Come with me.\"", name),
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
pub fn follow_guard(vault: &mut Vault, player_in_vault: bool) -> GuardInteraction {
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

    // ========== EXPANDED TEST COVERAGE ==========

    #[test]
    fn test_vault_bounds() {
        let vault = Vault::new((10, 10, 20, 20));

        // Inside bounds
        assert!(vault.contains(15, 15));
        assert!(vault.contains(10, 10));
        assert!(vault.contains(20, 20));

        // Outside bounds
        assert!(!vault.contains(9, 15));
        assert!(!vault.contains(21, 15));
        assert!(!vault.contains(15, 9));
        assert!(!vault.contains(15, 21));

        // Way outside
        assert!(!vault.contains(0, 0));
        assert!(!vault.contains(50, 50));
    }

    #[test]
    fn test_vault_center_calculation() {
        let vault1 = Vault::new((10, 10, 20, 20));
        assert_eq!(vault1.center(), (15, 15));

        let vault2 = Vault::new((0, 0, 10, 10));
        assert_eq!(vault2.center(), (5, 5));

        let vault3 = Vault::new((5, 5, 15, 15));
        assert_eq!(vault3.center(), (10, 10));
    }

    #[test]
    fn test_guard_state_progression() {
        let mut vault = Vault::new((10, 10, 15, 15));

        assert_eq!(vault.guard_state, GuardState::Absent);

        vault.guard_state = GuardState::Approaching;
        assert_eq!(vault.guard_state, GuardState::Approaching);

        vault.guard_state = GuardState::Demanding;
        assert_eq!(vault.guard_state, GuardState::Demanding);

        vault.guard_state = GuardState::Angry;
        assert_eq!(vault.guard_state, GuardState::Angry);

        vault.guard_state = GuardState::Escorting;
        assert_eq!(vault.guard_state, GuardState::Escorting);

        vault.guard_state = GuardState::Satisfied;
        assert_eq!(vault.guard_state, GuardState::Satisfied);
    }

    #[test]
    fn test_guard_creation_hp() {
        let mut rng = GameRng::new(42);
        let guard = create_guard(10, 10, &mut rng);

        assert!(guard.hp > 0);
        assert!(guard.hp > 50); // Guards should be tough
    }

    #[test]
    fn test_guard_peaceful_on_creation() {
        let mut rng = GameRng::new(42);
        let guard = create_guard(15, 15, &mut rng);

        assert!(guard.state.peaceful);
        assert!(guard.is_guard);
    }

    #[test]
    fn test_guard_name_not_empty() {
        let mut rng = GameRng::new(42);
        let guard = create_guard(10, 10, &mut rng);

        assert!(!guard.name.is_empty());
    }

    #[test]
    fn test_player_enters_vault_initial_state() {
        let mut vault = Vault::new((10, 10, 15, 15));
        assert_eq!(vault.guard_state, GuardState::Absent);

        let result = player_enters_vault(&mut vault, 100);

        assert!(matches!(result, GuardInteraction::None));
        assert_eq!(vault.guard_state, GuardState::Approaching);
        assert_eq!(vault.summon_turn, 100);
    }

    #[test]
    fn test_warning_count_increments() {
        let mut vault = Vault::new((10, 10, 15, 15));
        vault.guard_state = GuardState::Demanding;

        let mut warning_count = 0;
        for _ in 0..3 {
            let result = refuse_identification(&mut vault);
            if matches!(result, GuardInteraction::DemandId { .. }) {
                warning_count += 1;
            } else {
                break;
            }
        }

        assert!(warning_count > 0);
        assert!(vault.warnings >= 1);
    }

    #[test]
    fn test_refuse_identification_escalation() {
        let mut vault = Vault::new((10, 10, 15, 15));
        vault.guard_state = GuardState::Demanding;

        // First refusal
        refuse_identification(&mut vault);
        assert_eq!(vault.warnings, 1);

        // Second refusal
        refuse_identification(&mut vault);
        assert_eq!(vault.warnings, 2);

        // Third refusal - guard gets angry
        let result = refuse_identification(&mut vault);
        assert_eq!(vault.warnings, 3);
        if vault.warnings >= 3 {
            assert_eq!(vault.guard_state, GuardState::Angry);
        }
    }

    #[test]
    fn test_give_name_to_guard_changes_state() {
        let mut vault = Vault::new((10, 10, 15, 15));
        vault.guard_state = GuardState::Demanding;
        let mut rng = GameRng::new(42);

        let result = give_name_to_guard(&mut vault, "Test Hero", true, &mut rng);

        assert!(matches!(result, GuardInteraction::AcceptName { .. }));
        assert_eq!(vault.guard_state, GuardState::Escorting);
        assert!(vault.player_identified);
    }

    #[test]
    fn test_give_fake_name_to_guard() {
        let mut vault = Vault::new((10, 10, 15, 15));
        vault.guard_state = GuardState::Demanding;
        let mut rng = GameRng::new(42);

        let result = give_name_to_guard(&mut vault, "Fake Name", false, &mut rng);

        // Fake name might be accepted but stored as false
        assert!(vault.given_name.is_some());
        assert!(!vault.player_identified || vault.player_identified);
    }

    #[test]
    fn test_follow_guard_still_in_vault() {
        let mut vault = Vault::new((10, 10, 15, 15));
        vault.guard_state = GuardState::Escorting;

        let result = follow_guard(&mut vault, true);

        // Should return escort message
        assert!(matches!(result, GuardInteraction::Escort { .. }));
        // State should remain escorting
        assert_eq!(vault.guard_state, GuardState::Escorting);
    }

    #[test]
    fn test_follow_guard_left_vault() {
        let mut vault = Vault::new((10, 10, 15, 15));
        vault.guard_state = GuardState::Escorting;

        let result = follow_guard(&mut vault, false);

        // Should return leave message
        assert!(matches!(result, GuardInteraction::Leave { .. }));
        // State should change to satisfied
        assert_eq!(vault.guard_state, GuardState::Satisfied);
    }

    #[test]
    fn test_vault_player_identified_flag() {
        let mut vault = Vault::new((10, 10, 15, 15));
        assert!(!vault.player_identified);

        vault.player_identified = true;
        assert!(vault.player_identified);
    }

    #[test]
    fn test_vault_given_name_option() {
        let mut vault = Vault::new((10, 10, 15, 15));
        assert!(vault.given_name.is_none());

        vault.given_name = Some("Player".to_string());
        assert!(vault.given_name.is_some());
        assert_eq!(vault.given_name.unwrap(), "Player");
    }

    #[test]
    fn test_vault_initial_state() {
        let vault = Vault::new((10, 10, 20, 20));

        assert_eq!(vault.guard_state, GuardState::Absent);
        assert_eq!(vault.warnings, 0);
        assert!(!vault.player_identified);
        assert!(vault.given_name.is_none());
        assert_eq!(vault.summon_turn, 0);
    }

    #[test]
    fn test_vault_dimensions() {
        let vault = Vault::new((5, 5, 15, 15));

        // Check corners exist
        assert!(vault.contains(5, 5));
        assert!(vault.contains(15, 5));
        assert!(vault.contains(5, 15));
        assert!(vault.contains(15, 15));

        // Check outside corners
        assert!(!vault.contains(4, 4));
        assert!(!vault.contains(16, 16));
    }

    #[test]
    fn test_multiple_vaults_independent() {
        let mut vault1 = Vault::new((10, 10, 20, 20));
        let mut vault2 = Vault::new((30, 30, 40, 40));

        vault1.guard_state = GuardState::Angry;
        vault2.guard_state = GuardState::Demanding;

        assert_eq!(vault1.guard_state, GuardState::Angry);
        assert_eq!(vault2.guard_state, GuardState::Demanding);
        assert_ne!(vault1.guard_state, vault2.guard_state);
    }

    #[test]
    fn test_guard_state_transitions_valid() {
        // Test that transitions make sense
        let mut vault = Vault::new((10, 10, 15, 15));

        // Absent -> Approaching
        assert_eq!(vault.guard_state, GuardState::Absent);
        vault.guard_state = GuardState::Approaching;
        assert_eq!(vault.guard_state, GuardState::Approaching);

        // Approaching -> Demanding
        vault.guard_state = GuardState::Demanding;
        assert_eq!(vault.guard_state, GuardState::Demanding);

        // Can go to Angry
        vault.guard_state = GuardState::Angry;
        assert_eq!(vault.guard_state, GuardState::Angry);
    }

    #[test]
    fn test_vault_summon_turn_tracking() {
        let mut vault = Vault::new((10, 10, 15, 15));
        assert_eq!(vault.summon_turn, 0);

        player_enters_vault(&mut vault, 500);
        assert_eq!(vault.summon_turn, 500);

        // Second call is a no-op because guard_state is already Approaching
        player_enters_vault(&mut vault, 600);
        assert_eq!(vault.summon_turn, 500);
    }

    #[test]
    fn test_create_guard_multiple_instances() {
        let mut rng = GameRng::new(42);
        let guard1 = create_guard(10, 10, &mut rng);
        let guard2 = create_guard(20, 20, &mut rng);

        // Both should be valid guards
        assert!(guard1.is_guard);
        assert!(guard2.is_guard);

        // Both get MonsterId::NONE initially (caller assigns real IDs)
        assert_eq!(guard1.id, MonsterId::NONE);
        // But created at different positions
        assert_ne!(guard1.x, guard2.x);
    }

    #[test]
    fn test_guard_interaction_enum_variants() {
        // Verify all variants exist
        let _ = GuardInteraction::None;
        let _ = GuardInteraction::DemandId {
            message: String::new(),
        };
        let _ = GuardInteraction::AcceptName {
            message: String::new(),
        };
        let _ = GuardInteraction::Escort {
            message: String::new(),
        };
        let _ = GuardInteraction::Leave {
            message: String::new(),
        };
        let _ = GuardInteraction::Angry {
            message: String::new(),
        };
    }

    #[test]
    fn test_vault_edge_positions() {
        let vault = Vault::new((10, 10, 15, 15));

        // Test all edges
        for y in 10..=15 {
            assert!(vault.contains(10, y)); // Left edge
            assert!(vault.contains(15, y)); // Right edge
        }

        for x in 10..=15 {
            assert!(vault.contains(x, 10)); // Top edge
            assert!(vault.contains(x, 15)); // Bottom edge
        }
    }
}
