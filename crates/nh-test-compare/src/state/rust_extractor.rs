//! Extract unified game state from the Rust implementation.
//!
//! This module provides functionality to convert the Rust GameState
//! into a UnifiedGameState for comparison with the C implementation.

use nh_core::GameLoop;
use crate::state::common::*;

/// Wrapper for Rust game engine state extraction
pub struct RustGameEngine<'a> {
    game_loop: &'a mut GameLoop,
}

impl<'a> RustGameEngine<'a> {
    /// Create a new Rust game engine wrapper
    pub fn new(game_loop: &'a mut GameLoop) -> Self {
        Self { game_loop }
    }

    /// Extract unified state from Rust implementation
    pub fn extract_state(&self) -> UnifiedGameState {
        let state = self.game_loop.state();

        UnifiedGameState {
            player: UnifiedPlayer {
                name: state.player.name.clone(),
                role: state.player.role.to_string(),
                race: state.player.race.to_string(),
                gender: state.player.gender.to_string(),
                alignment: state.player.alignment.typ.to_string(),
            },
            position: (state.player.pos.x as i32, state.player.pos.y as i32),
            hp: state.player.hp,
            max_hp: state.player.hp_max,
            energy: state.player.energy,
            max_energy: state.player.energy_max,
            armor_class: state.player.armor_class as i32,
            gold: state.player.gold,
            experience_level: state.player.exp_level,
            strength: state.player.attr_current.get(nh_core::player::Attribute::Strength) as i32,
            dexterity: state.player.attr_current.get(nh_core::player::Attribute::Dexterity) as i32,
            constitution: state.player.attr_current.get(nh_core::player::Attribute::Constitution) as i32,
            intelligence: state.player.attr_current.get(nh_core::player::Attribute::Intelligence) as i32,
            wisdom: state.player.attr_current.get(nh_core::player::Attribute::Wisdom) as i32,
            charisma: state.player.attr_current.get(nh_core::player::Attribute::Charisma) as i32,
            current_level: state.current_level.dlevel.dungeon_num as i32 + 1,
            dungeon_depth: state.current_level.dlevel.depth() as i32,
            dungeon_visited: state.levels.keys()
                .map(|d| d.depth() as i32)
                .chain(std::iter::once(state.current_level.dlevel.depth() as i32))
                .collect(),
            has_amulet: state.flags.ascended, // Simplified - true when amulet is offered
            turn: state.turns,
            hunger: map_hunger_state(state.player.hunger_state),
            status_effects: extract_status_effects(state),
            inventory: state.inventory.iter().map(extract_object).collect(),
            nearby_monsters: extract_monsters(state),
            conduct: extract_conduct(state),
            is_dead: state.player.is_dead() || state.flags.ascended,
            death_message: if state.player.is_dead() || state.flags.ascended {
                Some("Killed in the Rust implementation".to_string())
            } else {
                None
            },
            is_won: state.flags.ascended,
        }
    }

    /// Execute an action on the Rust engine
    pub fn step(&mut self, action: &GameAction) -> (f64, String) {
        use nh_core::action::Command;

        let command = match action {
            GameAction::MoveNorth => Command::Move(nh_core::action::Direction::North),
            GameAction::MoveSouth => Command::Move(nh_core::action::Direction::South),
            GameAction::MoveEast => Command::Move(nh_core::action::Direction::East),
            GameAction::MoveWest => Command::Move(nh_core::action::Direction::West),
            GameAction::MoveNorthWest => Command::Move(nh_core::action::Direction::NorthWest),
            GameAction::MoveNorthEast => Command::Move(nh_core::action::Direction::NorthEast),
            GameAction::MoveSouthWest => Command::Move(nh_core::action::Direction::SouthWest),
            GameAction::MoveSouthEast => Command::Move(nh_core::action::Direction::SouthEast),
            GameAction::Wait => Command::Rest,
            GameAction::Pickup => Command::Pickup,
            GameAction::GoUp => Command::GoUp,
            GameAction::GoDown => Command::GoDown,
            GameAction::Inventory => Command::Inventory,
            GameAction::Look => Command::Look,
            GameAction::History => Command::History,
            GameAction::Help => Command::Help,
            GameAction::Save => Command::Save,
            GameAction::Quit => Command::Quit,
            _ => Command::Rest, // Simplified - many actions not fully implemented
        };

        let result = self.game_loop.tick(command);

        let message = match &result {
            nh_core::GameLoopResult::Continue => {
                self.game_loop.state().messages.last()
                    .cloned()
                    .unwrap_or_else(|| "Rust: action completed".to_string())
            }
            nh_core::GameLoopResult::PlayerDied(msg) => msg.clone(),
            nh_core::GameLoopResult::PlayerWon => "You escaped!".to_string(),
            nh_core::GameLoopResult::PlayerQuit => "Quit".to_string(),
            nh_core::GameLoopResult::SaveAndQuit => "Save and quit".to_string(),
        };

        // Calculate reward
        let reward = calculate_reward(self.game_loop.state(), action);

        (reward, message)
    }

    /// Get messages from last turn
    pub fn last_messages(&self) -> Vec<String> {
        self.game_loop.state().messages.clone()
    }

    /// Check if game is over
    pub fn is_game_over(&self) -> bool {
        let state = self.game_loop.state();
        state.player.is_dead() || state.flags.ascended
    }
}

/// Map Rust hunger state to unified hunger state
fn map_hunger_state(rust_hunger: nh_core::player::HungerState) -> HungerState {
    match rust_hunger {
        nh_core::player::HungerState::Satiated => HungerState::Satisified,
        nh_core::player::HungerState::NotHungry => HungerState::NotHungry,
        nh_core::player::HungerState::Hungry => HungerState::Hungry,
        nh_core::player::HungerState::Weak => HungerState::Weak,
        nh_core::player::HungerState::Fainting => HungerState::Fainting,
        nh_core::player::HungerState::Fainted => HungerState::Fainting, // Map fainted to weakest available
        nh_core::player::HungerState::Starved => HungerState::Starved,
    }
}

/// Extract status effects from Rust game state
fn extract_status_effects(state: &nh_core::GameState) -> Vec<StatusEffect> {
    let mut effects = Vec::new();

    if state.player.confused_timeout > 0 {
        effects.push(StatusEffect::Confused);
    }
    if state.player.stunned_timeout > 0 {
        effects.push(StatusEffect::Stunned);
    }
    if state.player.blinded_timeout > 0 {
        effects.push(StatusEffect::Blind);
    }
    if state.player.hallucinating_timeout > 0 {
        effects.push(StatusEffect::Hallucinating);
    }

    effects
}

/// Extract a unified object from Rust object
fn extract_object(obj: &nh_core::object::Object) -> UnifiedObject {
    let class_name = match obj.class {
        nh_core::object::ObjectClass::Weapon => "weapon",
        nh_core::object::ObjectClass::Armor => "armor",
        nh_core::object::ObjectClass::Ring => "ring",
        nh_core::object::ObjectClass::Amulet => "amulet",
        nh_core::object::ObjectClass::Tool => "tool",
        nh_core::object::ObjectClass::Food => "food",
        nh_core::object::ObjectClass::Potion => "potion",
        nh_core::object::ObjectClass::Scroll => "scroll",
        nh_core::object::ObjectClass::Spellbook => "spellbook",
        nh_core::object::ObjectClass::Wand => "wand",
        nh_core::object::ObjectClass::Coin => "coin",
        nh_core::object::ObjectClass::Gem => "gem",
        nh_core::object::ObjectClass::Rock => "rock",
        nh_core::object::ObjectClass::Ball => "ball",
        nh_core::object::ObjectClass::Chain => "chain",
        nh_core::object::ObjectClass::Venom => "venom",
        _ => "unknown",
    };

    UnifiedObject {
        name: format!("{:?}", obj.object_type),
        class: class_name.to_string(),
        quantity: obj.quantity,
        enchantment: obj.enchantment as i32,
        cursed: matches!(obj.buc, nh_core::object::BucStatus::Cursed),
        blessed: matches!(obj.buc, nh_core::object::BucStatus::Blessed),
        armor_class: 0,
        damage: 0,
        weight: obj.weight as i32,
        value: 0,
    }
}

/// Extract unified monsters from Rust game state
fn extract_monsters(state: &nh_core::GameState) -> Vec<UnifiedMonster> {
    state.current_level.monsters.iter()
        .filter(|m| {
            // Only include monsters on the same level and nearby
            let dx = (m.x as i32 - state.player.pos.x as i32).abs();
            let dy = (m.y as i32 - state.player.pos.y as i32).abs();
            dx <= 10 && dy <= 10
        })
        .map(|m| UnifiedMonster {
            name: m.name.clone(),
            symbol: '*', // Default symbol since we don't have permonst in Monster
            level: m.level as i32,
            hp: m.hp,
            max_hp: m.hp_max,
            armor_class: m.ac as i32,
            position: (m.x as i32, m.y as i32),
            asleep: m.state.sleeping,
            peaceful: m.state.peaceful,
        })
        .collect()
}

/// Extract conduct from Rust game state
fn extract_conduct(_state: &nh_core::GameState) -> ConductState {
    // Conduct tracking not yet implemented in Rust version
    ConductState::default()
}

/// Calculate reward for RL agent
fn calculate_reward(state: &nh_core::GameState, _action: &GameAction) -> f64 {
    let mut reward = 0.0;

    // Small reward for each turn
    reward += 0.01;

    // Check for death
    if state.player.is_dead() {
        reward -= 100.0;
    }

    // Check for ascension
    if state.flags.ascended {
        reward += 1000.0;
    }

    reward
}

#[cfg(test)]
mod tests {
    use super::*;
    use nh_core::GameLoop;
    use nh_core::GameState;
    use nh_core::GameRng;

    #[test]
    fn test_extract_state() {
        let rng = GameRng::new(42);
        let state = GameState::new(rng);
        let mut game_loop = GameLoop::new(state);

        // Execute one turn
        game_loop.tick(nh_core::action::Command::Rest);

        let extractor = RustGameEngine::new(&mut game_loop);
        let unified = extractor.extract_state();

        // Check basic fields
        assert!(!unified.is_dead);
    }
}
