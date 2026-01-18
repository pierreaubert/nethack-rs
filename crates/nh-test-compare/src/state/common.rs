//! Unified game state representation for comparing Rust and C implementations.
//!
//! This module defines a common state structure that both implementations
//! can be mapped to for comparison purposes.

use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

/// Unified player information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedPlayer {
    pub name: String,
    pub role: String,
    pub race: String,
    pub gender: String,
    pub alignment: String,
}

/// Unified object/item representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedObject {
    pub name: String,
    pub class: String,
    pub quantity: i32,
    pub enchantment: i32,
    pub cursed: bool,
    pub blessed: bool,
    pub armor_class: i32,
    pub damage: i32,
    pub weight: i32,
    pub value: i32,
}

/// Unified monster representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedMonster {
    pub name: String,
    pub symbol: char,
    pub level: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub armor_class: i32,
    pub position: (i32, i32),
    pub asleep: bool,
    pub peaceful: bool,
}

/// Hunger state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HungerState {
    Satisified,
    NotHungry,
    Hungry,
    Weak,
    Fainting,
    Starved,
}

/// Conduct tracking
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConductState {
    pub unvegetarian: i32,
    pub unvegan: i32,
    pub food: i32,
    pub gnostic: i32,
    pub weaphit: i32,
    pub killer: i32,
    pub literate: i32,
    pub polypiles: i32,
    pub polyselfs: i32,
    pub wishes: i32,
    pub wisharti: i32,
}

/// Status effects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatusEffect {
    Blind,
    Confused,
    Stunned,
    Hallucinating,
    Sick,
    FoodPoisoned,
    VenomPoisoned,
    Burned,
    Frozen,
    Strangled,
    Suffocating,
    FrozenSolid,
    Petrified,
    TurnedToStone,
    Unconscious,
    Sleeping,
    Paralyzed,
    Held,
    Infested,
}

/// Unified game state for comparison
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedGameState {
    /// Player information
    pub player: UnifiedPlayer,
    
    /// Player position (0-indexed)
    pub position: (i32, i32),
    
    /// Basic stats
    pub hp: i32,
    pub max_hp: i32,
    pub energy: i32,
    pub max_energy: i32,
    pub armor_class: i32,
    pub gold: i32,
    pub experience_level: i32,
    
    /// Attributes (1-25 range)
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
    
    /// Dungeon state
    pub current_level: i32,
    pub dungeon_depth: i32,
    pub dungeon_visited: Vec<i32>,
    pub has_amulet: bool,
    
    /// Game progress
    pub turn: u64,
    pub hunger: HungerState,
    pub status_effects: Vec<StatusEffect>,
    
    /// Inventory
    pub inventory: Vec<UnifiedObject>,
    
    /// Nearby monsters
    pub nearby_monsters: Vec<UnifiedMonster>,
    
    /// Conduct tracking
    pub conduct: ConductState,
    
    /// Game outcome
    pub is_dead: bool,
    pub death_message: Option<String>,
    pub is_won: bool,
}

impl UnifiedGameState {
    /// Create a default starting state
    pub fn default_start(role: &str, race: &str) -> Self {
        Self {
            player: UnifiedPlayer {
                name: "Player".to_string(),
                role: role.to_string(),
                race: race.to_string(),
                gender: "Male".to_string(),
                alignment: "Neutral".to_string(),
            },
            position: (40, 10),
            hp: 10,
            max_hp: 10,
            energy: 10,
            max_energy: 10,
            armor_class: 10,
            gold: 0,
            experience_level: 1,
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
            current_level: 1,
            dungeon_depth: 1,
            dungeon_visited: vec![1],
            has_amulet: false,
            turn: 0,
            hunger: HungerState::NotHungry,
            status_effects: Vec::new(),
            inventory: Vec::new(),
            nearby_monsters: Vec::new(),
            conduct: ConductState::default(),
            is_dead: false,
            death_message: None,
            is_won: false,
        }
    }
    
    /// Calculate distance between two states
    pub fn state_distance(&self, other: &Self) -> f64 {
        let mut distance = 0.0;
        
        // Position distance
        let pos_dist = ((self.position.0 - other.position.0).pow(2) as f64 
                      + (self.position.1 - other.position.1).pow(2) as f64).sqrt();
        distance += pos_dist / 100.0;
        
        // HP difference
        distance += ((self.hp - other.hp) as f64).abs() / 20.0;
        
        // Level difference
        distance += (self.dungeon_depth - other.dungeon_depth) as f64;
        
        // Gold difference
        distance += ((self.gold - other.gold) as f64).abs() / 100.0;
        
        distance
    }
    
    /// Check if the state represents a terminal condition
    pub fn is_terminal(&self) -> bool {
        self.is_dead || self.is_won
    }
    
    /// Get survival reward component
    pub fn survival_bonus(&self) -> f64 {
        if self.is_dead {
            return -100.0;
        }
        if self.is_won {
            return 1000.0;
        }
        // Small positive reward for each turn survived
        0.01
    }
}

/// Game action for RL agent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameAction {
    // Movement (8 directions + wait)
    MoveNorth,
    MoveSouth,
    MoveEast,
    MoveWest,
    MoveNorthEast,
    MoveNorthWest,
    MoveSouthEast,
    MoveSouthWest,
    Wait,
    
    // Combat
    AttackNorth,
    AttackSouth,
    AttackEast,
    AttackWest,
    
    // Interaction
    Pickup,
    DropFirst,
    DropSelected(char),
    EatFirst,
    EatSelected(char),
    WieldFirst,
    WieldSelected(char),
    WearFirst,
    WearSelected(char),
    TakeOffFirst,
    TakeOffSelected(char),
    
    // Magic
    QuaffFirst,
    QuaffSelected(char),
    ReadFirst,
    ReadSelected(char),
    ZapFirst,
    ZapDirection(char, i32, i32),
    
    // Navigation
    GoUp,
    GoDown,
    
    // Info
    Inventory,
    Look,
    History,
    Help,
    
    // Meta
    Save,
    Quit,
}

impl GameAction {
    /// Convert to single character command for C NetHack
    pub fn to_c_char(&self) -> char {
        match self {
            GameAction::MoveWest => 'h',
            GameAction::MoveSouth => 'j',
            GameAction::MoveNorth => 'k',
            GameAction::MoveEast => 'l',
            GameAction::MoveNorthWest => 'y',
            GameAction::MoveNorthEast => 'u',
            GameAction::MoveSouthWest => 'b',
            GameAction::MoveSouthEast => 'n',
            GameAction::Wait => '.',
            GameAction::Pickup => ',',
            GameAction::DropFirst => 'd',
            GameAction::EatFirst => 'e',
            GameAction::WieldFirst => 'w',
            GameAction::WearFirst => 'W',
            GameAction::TakeOffFirst => 'T',
            GameAction::QuaffFirst => 'q',
            GameAction::ReadFirst => 'r',
            GameAction::ZapFirst => 'z',
            GameAction::GoUp => '<',
            GameAction::GoDown => '>',
            GameAction::Inventory => 'i',
            GameAction::Look => '/',
            GameAction::History => '\\',
            GameAction::Help => '?',
            GameAction::Save => 'S',
            GameAction::Quit => 'Q',
            _ => '.',
        }
    }
    
    /// Get movement delta for directional actions
    pub fn movement_delta(&self) -> Option<(i32, i32)> {
        match self {
            GameAction::MoveNorth => Some((0, -1)),
            GameAction::MoveSouth => Some((0, 1)),
            GameAction::MoveEast => Some((1, 0)),
            GameAction::MoveWest => Some((-1, 0)),
            GameAction::MoveNorthWest => Some((-1, -1)),
            GameAction::MoveNorthEast => Some((1, -1)),
            GameAction::MoveSouthWest => Some((-1, 1)),
            GameAction::MoveSouthEast => Some((1, 1)),
            GameAction::AttackNorth => Some((0, -1)),
            GameAction::AttackSouth => Some((0, 1)),
            GameAction::AttackEast => Some((1, 0)),
            GameAction::AttackWest => Some((-1, 0)),
            GameAction::ZapDirection(_, dx, dy) => Some((*dx, *dy)),
            _ => None,
        }
    }
    
    /// Check if action consumes time
    pub fn consumes_time(&self) -> bool {
        match self {
            GameAction::Inventory | GameAction::Look | GameAction::History 
            | GameAction::Help | GameAction::Save | GameAction::Quit => false,
            _ => true,
        }
    }
}

/// Step result containing state and reward
#[derive(Debug, Clone)]
pub struct StepResult {
    pub state: UnifiedGameState,
    pub reward: f64,
    pub done: bool,
    pub message: String,
    pub is_terminal: bool,
}

/// Comparison result
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    pub rust_state: UnifiedGameState,
    pub c_state: UnifiedGameState,
    pub differences: Vec<StateDifference>,
    pub rust_messages: Vec<String>,
    pub c_messages: Vec<String>,
    pub turn: u64,
}

impl ComparisonResult {
    pub fn has_critical_differences(&self) -> bool {
        self.differences.iter().any(|d| d.severity == DifferenceSeverity::Critical)
    }
    
    pub fn has_differences(&self) -> bool {
        !self.differences.is_empty()
    }
}

/// A detected difference between states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDifference {
    pub category: DifferenceCategory,
    pub field: String,
    pub rust_value: serde_json::Value,
    pub c_value: serde_json::Value,
    pub severity: DifferenceSeverity,
    pub description: String,
}

/// Categories of differences
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifferenceCategory {
    Position,
    Stats,
    Health,
    Energy,
    Attributes,
    Inventory,
    Monsters,
    Dungeon,
    Status,
    Timing,
    Message,
    Other,
}

/// Severity of a difference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DifferenceSeverity {
    Critical,  // Different game outcomes
    Major,     // Significant gameplay difference
    Minor,     // Cosmetic or minor difference
    Info,      // Just for tracking
}

impl DifferenceSeverity {
    pub fn is_critical(&self) -> bool {
        matches!(self, DifferenceSeverity::Critical)
    }
    
    pub fn is_major_or_worse(&self) -> bool {
        matches!(self, DifferenceSeverity::Critical | DifferenceSeverity::Major)
    }
}

/// Statistics about a comparison session
#[derive(Debug, Default, Clone)]
pub struct SessionStats {
    pub total_turns: u64,
    pub critical_differences: usize,
    pub major_differences: usize,
    pub minor_differences: usize,
    pub rust_deaths: usize,
    pub c_deaths: usize,
    pub diverging_turns: Vec<u64>,
}

impl SessionStats {
    pub fn add_difference(&mut self, diff: &StateDifference) {
        match diff.severity {
            DifferenceSeverity::Critical => self.critical_differences += 1,
            DifferenceSeverity::Major => self.major_differences += 1,
            DifferenceSeverity::Minor => self.minor_differences += 1,
            DifferenceSeverity::Info => {}
        }
    }
    
    pub fn summary(&self) -> String {
        format!(
            "Session Summary:\n\
             - Total turns: {}\n\
             - Critical differences: {}\n\
             - Major differences: {}\n\
             - Minor differences: {}\n\
             - Rust deaths: {}\n\
             - C deaths: {}\n\
             - Diverging turns: {:?}",
            self.total_turns,
            self.critical_differences,
            self.major_differences,
            self.minor_differences,
            self.rust_deaths,
            self.c_deaths,
            self.diverging_turns
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_start() {
        let state = UnifiedGameState::default_start("Wizard", "Human");
        
        assert_eq!(state.player.role, "Wizard");
        assert_eq!(state.player.race, "Human");
        assert_eq!(state.hp, 10);
        assert_eq!(state.position, (40, 10));
        assert!(!state.is_dead);
    }
    
    #[test]
    fn test_state_distance() {
        let state1 = UnifiedGameState::default_start("Rogue", "Elf");
        let mut state2 = UnifiedGameState::default_start("Rogue", "Elf");
        
        // Same state - distance should be 0
        assert_eq!(state1.state_distance(&state2), 0.0);
        
        // Move state2
        state2.position = (42, 12);
        let distance = state1.state_distance(&state2);
        assert!(distance > 0.0);
    }
    
    #[test]
    fn test_action_to_char() {
        assert_eq!(GameAction::MoveNorth.to_c_char(), 'k');
        assert_eq!(GameAction::MoveSouth.to_c_char(), 'j');
        assert_eq!(GameAction::MoveEast.to_c_char(), 'l');
        assert_eq!(GameAction::MoveWest.to_c_char(), 'h');
        assert_eq!(GameAction::Wait.to_c_char(), '.');
        assert_eq!(GameAction::Pickup.to_c_char(), ',');
        assert_eq!(GameAction::Inventory.to_c_char(), 'i');
    }
    
    #[test]
    fn test_action_delta() {
        assert_eq!(GameAction::MoveNorth.movement_delta(), Some((0, -1)));
        assert_eq!(GameAction::MoveSouth.movement_delta(), Some((0, 1)));
        assert_eq!(GameAction::MoveEast.movement_delta(), Some((1, 0)));
        assert_eq!(GameAction::MoveWest.movement_delta(), Some((-1, 0)));
        assert_eq!(GameAction::Wait.movement_delta(), None);
    }
    
    #[test]
    fn test_action_time_consumption() {
        assert!(!GameAction::Inventory.consumes_time());
        assert!(!GameAction::Look.consumes_time());
        assert!(GameAction::MoveNorth.consumes_time());
        assert!(GameAction::AttackNorth.consumes_time());
    }
    
    #[test]
    fn test_terminal_state() {
        let mut state = UnifiedGameState::default_start("Tourist", "Gnome");
        assert!(!state.is_terminal());
        
        state.is_dead = true;
        assert!(state.is_terminal());
        
        let mut state2 = UnifiedGameState::default_start("Samurai", "Human");
        state2.is_won = true;
        assert!(state2.is_terminal());
    }
}
