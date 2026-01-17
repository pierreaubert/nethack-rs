//! Main game loop (allmain.c)

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::action::{ActionResult, Command};
use crate::dungeon::{DLevel, Level};
use crate::object::Object;
use crate::player::You;
use crate::rng::GameRng;
use crate::world::{Context, Flags, TimeoutManager};
use crate::NORMAL_SPEED;

/// Result of a game loop tick
#[derive(Debug, Clone)]
pub enum GameLoopResult {
    /// Continue playing
    Continue,
    /// Player died with message
    PlayerDied(String),
    /// Player quit
    PlayerQuit,
    /// Player ascended
    PlayerWon,
    /// Save and quit
    SaveAndQuit,
}

/// Main game state
#[derive(Serialize, Deserialize)]
pub struct GameState {
    /// Player character
    pub player: You,

    /// Player inventory
    pub inventory: Vec<Object>,

    /// Current level
    pub current_level: Level,

    /// All visited levels
    #[serde(skip)]
    pub levels: HashMap<DLevel, Level>,

    /// Game flags
    pub flags: Flags,

    /// Current context
    pub context: Context,

    /// Random number generator
    pub rng: GameRng,

    /// Timeout manager for timed events
    pub timeouts: TimeoutManager,

    /// Turn counter
    pub turns: u64,

    /// Monster turn counter
    pub monster_turns: u64,

    /// Messages for the current turn
    #[serde(skip)]
    pub messages: Vec<String>,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new(GameRng::from_entropy())
    }
}

impl GameState {
    /// Create a new game with the given RNG
    pub fn new(mut rng: GameRng) -> Self {
        let dlevel = DLevel::main_dungeon_start();
        let current_level = Level::new_generated(dlevel, &mut rng);

        // Find upstairs to place player
        let (start_x, start_y) = current_level
            .find_upstairs()
            .unwrap_or((40, 10)); // Default fallback position

        let mut player = You::default();
        player.pos.x = start_x;
        player.pos.y = start_y;
        player.prev_pos = player.pos;

        Self {
            player,
            inventory: Vec::new(),
            current_level,
            levels: HashMap::new(),
            flags: Flags::default(),
            context: Context::default(),
            rng,
            timeouts: TimeoutManager::new(),
            turns: 0,
            monster_turns: 0,
            messages: Vec::new(),
        }
    }

    /// Add a message to display
    pub fn message(&mut self, msg: impl Into<String>) {
        self.messages.push(msg.into());
    }

    /// Clear messages
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    /// Add an object to the player's inventory
    pub fn add_to_inventory(&mut self, mut object: Object) -> char {
        // Find first available inventory letter
        let used_letters: std::collections::HashSet<char> =
            self.inventory.iter().map(|o| o.inv_letter).collect();

        let letter = ('a'..='z')
            .chain('A'..='Z')
            .find(|c| !used_letters.contains(c))
            .unwrap_or('$');

        object.inv_letter = letter;
        object.location = crate::object::ObjectLocation::PlayerInventory;
        self.inventory.push(object);
        letter
    }

    /// Remove an object from inventory by letter
    pub fn remove_from_inventory(&mut self, letter: char) -> Option<Object> {
        let idx = self.inventory.iter().position(|o| o.inv_letter == letter)?;
        Some(self.inventory.remove(idx))
    }

    /// Get object from inventory by letter
    pub fn get_inventory_item(&self, letter: char) -> Option<&Object> {
        self.inventory.iter().find(|o| o.inv_letter == letter)
    }

    /// Get mutable object from inventory by letter
    pub fn get_inventory_item_mut(&mut self, letter: char) -> Option<&mut Object> {
        self.inventory.iter_mut().find(|o| o.inv_letter == letter)
    }

    /// Calculate total inventory weight
    pub fn inventory_weight(&self) -> u32 {
        self.inventory.iter().map(|o| o.weight * o.quantity as u32).sum()
    }

    /// Calculate total armor class from worn equipment and dexterity
    /// NetHack AC: base 10, lower is better
    /// Armor bonus is subtracted, dexterity bonus is added (negative for good dex)
    pub fn calculate_armor_class(&self) -> i8 {
        const BASE_AC: i8 = 10;

        // Sum AC from all worn armor pieces
        let armor_ac: i32 = self
            .inventory
            .iter()
            .filter(|obj| obj.is_worn() && obj.is_armor())
            .map(|obj| obj.effective_ac() as i32)
            .sum();

        // Dexterity bonus (negative = better AC)
        let dex_bonus = self.player.attr_current.dexterity_ac_bonus();

        // Calculate final AC: base - armor protection + dex bonus
        // armor_ac is how much protection we have (positive = good)
        // dex_bonus is -4 to +3 (negative = better)
        let ac = BASE_AC as i32 - armor_ac + dex_bonus as i32;

        // Clamp to i8 range
        ac.clamp(-128, 127) as i8
    }

    /// Update player's armor class based on current equipment
    pub fn update_armor_class(&mut self) {
        self.player.armor_class = self.calculate_armor_class();
    }
}

/// Game loop controller
pub struct GameLoop {
    state: GameState,
}

impl GameLoop {
    /// Create a new game loop with the given state
    pub fn new(state: GameState) -> Self {
        Self { state }
    }

    /// Get reference to game state
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Get mutable reference to game state
    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    /// Execute a single game tick
    ///
    /// Based on moveloop() from allmain.c
    pub fn tick(&mut self, command: Command) -> GameLoopResult {
        // Execute player command
        let result = self.execute_command(command);

        match result {
            ActionResult::Success => {
                // Time passes - consume movement points
                self.state.player.movement_points -= NORMAL_SPEED;
                self.state.context.move_made = true;
            }
            ActionResult::NoTime => {
                // No time passes
            }
            ActionResult::Cancelled => {
                return GameLoopResult::Continue;
            }
            ActionResult::Failed(msg) => {
                self.state.message(msg);
                return GameLoopResult::Continue;
            }
            ActionResult::Died(msg) => {
                return GameLoopResult::PlayerDied(msg);
            }
            ActionResult::Save => {
                return GameLoopResult::SaveAndQuit;
            }
            ActionResult::Quit => {
                return GameLoopResult::PlayerQuit;
            }
        }

        // Process monsters while player is out of movement
        while self.state.player.movement_points < NORMAL_SPEED {
            self.state.context.monsters_moving = true;

            let monsters_moved = self.move_monsters();

            self.state.context.monsters_moving = false;

            if !monsters_moved {
                // New turn begins
                self.new_turn();
            }

            // Check if player died during monster actions
            if self.state.player.is_dead() {
                return GameLoopResult::PlayerDied("killed by a monster".to_string());
            }
        }

        GameLoopResult::Continue
    }

    /// Execute a player command
    fn execute_command(&mut self, command: Command) -> ActionResult {
        match command {
            Command::Move(dir) => self.do_move(dir),
            Command::Rest => {
                self.state.message("You wait.");
                ActionResult::Success
            }
            Command::GoUp => self.do_go_up(),
            Command::GoDown => self.do_go_down(),
            Command::Search => self.do_search(),
            Command::Quit => ActionResult::Quit,
            Command::Save => ActionResult::Save,

            // Object manipulation - these need item selection from UI
            Command::Pickup => crate::action::pickup::do_pickup(&mut self.state),
            Command::Drop => {
                // TODO: Get item letter from UI
                self.state.message("Drop which item?");
                ActionResult::NoTime
            }
            Command::Eat => {
                // TODO: Get item letter from UI
                self.state.message("Eat what?");
                ActionResult::NoTime
            }
            Command::Apply => {
                // TODO: Get item letter from UI
                self.state.message("Apply what?");
                ActionResult::NoTime
            }
            Command::Wear => {
                // TODO: Get item letter from UI
                self.state.message("Wear what?");
                ActionResult::NoTime
            }
            Command::TakeOff => {
                // TODO: Get item letter from UI
                self.state.message("Take off what?");
                ActionResult::NoTime
            }
            Command::Wield => {
                // TODO: Get item letter from UI
                self.state.message("Wield what?");
                ActionResult::NoTime
            }
            Command::PutOn => {
                // TODO: Get item letter from UI
                self.state.message("Put on what?");
                ActionResult::NoTime
            }
            Command::Remove => {
                // TODO: Get item letter from UI
                self.state.message("Remove what?");
                ActionResult::NoTime
            }

            // Directional actions
            Command::Open => {
                // TODO: Get direction from UI
                self.state.message("Open in which direction?");
                ActionResult::NoTime
            }
            Command::Close => {
                // TODO: Get direction from UI
                self.state.message("Close in which direction?");
                ActionResult::NoTime
            }
            Command::Kick => {
                // TODO: Get direction from UI
                self.state.message("Kick in which direction?");
                ActionResult::NoTime
            }

            // Special actions
            Command::Pray => crate::action::pray::do_pray(&mut self.state),
            Command::Engrave => {
                // TODO: Get text from UI
                crate::action::engrave::do_engrave(&mut self.state, "Elbereth")
            }

            // Information commands (no time cost)
            Command::Inventory => {
                // TODO: Display inventory via UI
                self.state.message("Inventory display not yet implemented.");
                ActionResult::NoTime
            }
            Command::Look => {
                self.state.message("You look around.");
                ActionResult::NoTime
            }
            Command::WhatsHere => {
                let objects = self.state.current_level.objects_at(
                    self.state.player.pos.x,
                    self.state.player.pos.y,
                );
                if objects.is_empty() {
                    self.state.message("There is nothing here.");
                } else {
                    self.state.message(format!("You see {} item(s) here.", objects.len()));
                }
                ActionResult::NoTime
            }

            _ => {
                self.state.message("That command is not yet implemented.");
                ActionResult::NoTime
            }
        }
    }

    /// Handle player movement
    fn do_move(&mut self, dir: crate::action::Direction) -> ActionResult {
        let (dx, dy) = dir.delta();
        let state = &mut self.state;

        let new_x = state.player.pos.x + dx;
        let new_y = state.player.pos.y + dy;

        // Check bounds
        if !state.current_level.is_valid_pos(new_x, new_y) {
            state.message("You cannot go that way.");
            return ActionResult::NoTime;
        }

        // Check for monster
        if let Some(monster_id) = state.current_level.monster_at(new_x, new_y).map(|m| m.id) {
            let monster = state.current_level.monster(monster_id).unwrap();
            if monster.is_hostile() {
                // Initiate combat
                let monster_name = monster.name.clone();

                // Simple unarmed attack for now
                let result = crate::combat::player_attack_monster(
                    &mut state.player,
                    state.current_level.monster_mut(monster_id).unwrap(),
                    None, // No weapon
                    &mut state.rng,
                );

                if result.hit {
                    state.message(format!("You hit the {} for {} damage!", monster_name, result.damage));

                    if result.defender_died {
                        state.message(format!("You kill the {}!", monster_name));
                        state.current_level.remove_monster(monster_id);
                    }
                } else {
                    state.message(format!("You miss the {}!", monster_name));
                }
                return ActionResult::Success;
            } else {
                state.message("You swap places with the monster.");
                // TODO: Swap positions
            }
        }

        // Check if walkable
        if !state.current_level.is_walkable(new_x, new_y) {
            state.message("You cannot move there.");
            return ActionResult::NoTime;
        }

        // Move player
        state.player.prev_pos = state.player.pos;
        state.player.pos.x = new_x;
        state.player.pos.y = new_y;
        state.player.moved = true;

        ActionResult::Success
    }

    /// Handle going up stairs
    fn do_go_up(&mut self) -> ActionResult {
        let state = &self.state;
        let cell = state
            .current_level
            .cell(state.player.pos.x as usize, state.player.pos.y as usize);

        if !matches!(cell.typ, crate::dungeon::CellType::Stairs) {
            self.state.message("You can't go up here.");
            return ActionResult::NoTime;
        }

        // TODO: Check if stairs go up, handle level change
        self.state.message("You climb up the stairs.");
        ActionResult::Success
    }

    /// Handle going down stairs
    fn do_go_down(&mut self) -> ActionResult {
        let state = &self.state;
        let cell = state
            .current_level
            .cell(state.player.pos.x as usize, state.player.pos.y as usize);

        if !matches!(cell.typ, crate::dungeon::CellType::Stairs) {
            self.state.message("You can't go down here.");
            return ActionResult::NoTime;
        }

        // TODO: Check if stairs go down, handle level change
        self.state.message("You descend the stairs.");
        ActionResult::Success
    }

    /// Handle searching
    fn do_search(&mut self) -> ActionResult {
        let state = &mut self.state;

        // Check adjacent squares for secret doors
        let px = state.player.pos.x;
        let py = state.player.pos.y;

        let mut found = false;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let x = px + dx;
                let y = py + dy;
                if state.current_level.is_valid_pos(x, y) {
                    let cell = state.current_level.cell_mut(x as usize, y as usize);
                    if cell.typ == crate::dungeon::CellType::SecretDoor {
                        // TODO: Check search skill, might not find it
                        if state.rng.one_in(3) {
                            cell.typ = crate::dungeon::CellType::Door;
                            found = true;
                            state.message("You find a hidden door!");
                        }
                    }
                }
            }
        }

        if !found {
            state.message("You search but find nothing.");
        }

        ActionResult::Success
    }

    /// Process all monster movement
    fn move_monsters(&mut self) -> bool {
        let state = &mut self.state;
        let mut any_moved = false;

        // Get list of monster IDs
        let monster_ids: Vec<_> = state
            .current_level
            .monsters
            .iter()
            .map(|m| m.id)
            .collect();

        for id in monster_ids {
            // Check if monster has enough movement and can act
            let can_act = state
                .current_level
                .monster(id)
                .map(|m| m.movement >= NORMAL_SPEED && m.can_act())
                .unwrap_or(false);

            if can_act {
                // Deduct movement points
                if let Some(monster) = state.current_level.monster_mut(id) {
                    monster.movement -= NORMAL_SPEED;
                }

                // Process monster AI
                let action = crate::monster::process_monster_ai(
                    id,
                    &mut state.current_level,
                    &state.player,
                    &mut state.rng,
                );

                match action {
                    crate::monster::AiAction::Moved(_, _) => {
                        any_moved = true;
                    }
                    crate::monster::AiAction::AttackedPlayer => {
                        any_moved = true;
                        // Execute monster attack on player
                        if let Some(monster) = state.current_level.monster(id) {
                            // Use first active attack
                            if let Some(attack) = monster.attacks.iter().find(|a| a.is_active()) {
                                let monster_name = monster.name.clone();
                                let result = crate::combat::monster_attack_player(
                                    monster,
                                    &mut state.player,
                                    attack,
                                    &mut state.rng,
                                );

                                if result.hit {
                                    state.message(format!("The {} hits you for {} damage!", monster_name, result.damage));

                                    if result.defender_died {
                                        state.message("You die...");
                                    }
                                } else {
                                    state.message("The monster misses you!");
                                }
                            }
                        }
                    }
                    crate::monster::AiAction::Waited => {
                        any_moved = true;
                    }
                    crate::monster::AiAction::None => {}
                }
            }

            // Check if player got movement back
            if state.player.movement_points >= NORMAL_SPEED {
                break;
            }
        }

        any_moved
    }

    /// Start a new turn
    fn new_turn(&mut self) {
        let state = &mut self.state;

        // Increment turn counter
        state.turns += 1;

        // Reallocate movement points to player
        let base_move = NORMAL_SPEED;
        // TODO: Add speed bonuses, encumbrance penalties
        state.player.movement_points += base_move;

        // Reallocate movement to monsters
        for monster in &mut state.current_level.monsters {
            // TODO: Get monster speed from permonst data
            monster.movement += NORMAL_SPEED;
        }

        // Process timed events
        let current_turn = state.turns;
        let triggered_events = state.timeouts.tick(current_turn);
        for event in triggered_events {
            Self::process_timed_event(state, event);
        }

        // Process player status timeouts
        state.player.properties.tick_timeouts();
        if state.player.confused_timeout > 0 {
            state.player.confused_timeout -= 1;
        }
        if state.player.stunned_timeout > 0 {
            state.player.stunned_timeout -= 1;
        }
        if state.player.blinded_timeout > 0 {
            state.player.blinded_timeout -= 1;
        }
        if state.player.hallucinating_timeout > 0 {
            state.player.hallucinating_timeout -= 1;
        }
        if state.player.sleeping_timeout > 0 {
            state.player.sleeping_timeout -= 1;
        }
        if state.player.paralyzed_timeout > 0 {
            state.player.paralyzed_timeout -= 1;
        }

        // Process hunger
        state.player.digest(1);
        if matches!(state.player.hunger_state, crate::player::HungerState::Starved) {
            // TODO: Player dies from starvation
        }

        // Regeneration
        Self::process_regeneration(state);
    }

    /// Process a triggered timed event
    fn process_timed_event(state: &mut GameState, event: crate::world::TimedEvent) {
        use crate::world::TimedEventType;

        match event.event_type {
            TimedEventType::MonsterSpawn => {
                // TODO: Spawn a random monster
            }
            TimedEventType::MonsterAction(monster_id) => {
                // Monster-specific timed action (e.g., breath weapon cooldown)
                if state.current_level.monster(monster_id).is_some() {
                    // TODO: Execute monster's special action
                }
            }
            TimedEventType::CorpseRot(object_id) => {
                // Remove rotted corpse
                state.current_level.remove_object(object_id);
                state.message("You smell something rotting.");
            }
            TimedEventType::EggHatch(object_id) => {
                // TODO: Hatch egg into monster
                state.current_level.remove_object(object_id);
            }
            TimedEventType::Stoning => {
                if !state.player.properties.has(crate::player::Property::StoneResistance) {
                    state.message("You have turned to stone.");
                    // TODO: Player death
                }
            }
            TimedEventType::Sliming => {
                state.message("You have turned into a green slime!");
                // TODO: Player death or polymorph
            }
            TimedEventType::Strangling => {
                state.player.take_damage(3);
                if state.player.is_dead() {
                    state.message("You suffocate.");
                }
            }
            TimedEventType::Vomiting => {
                state.message("You vomit.");
                state.player.nutrition = state.player.nutrition.saturating_sub(100);
            }
            TimedEventType::DelayedDeath(ref cause) => {
                state.message(format!("You die from {}.", cause));
                // TODO: Player death
            }
            _ => {
                // Other event types handled elsewhere or not yet implemented
            }
        }
    }

    /// Process HP and energy regeneration
    fn process_regeneration(state: &mut GameState) {
        use crate::player::Attribute;

        // HP regeneration every few turns based on Con
        let con = state.player.attr_current.get(Attribute::Constitution);
        let regen_rate = match con {
            0..=6 => 35,
            7..=10 => 30,
            11..=14 => 25,
            15..=17 => 20,
            _ => 15,
        };

        if state.turns % regen_rate as u64 == 0 && state.player.hp < state.player.hp_max {
            state.player.hp += 1;
        }

        // Energy regeneration
        let int = state.player.attr_current.get(Attribute::Intelligence);
        let energy_rate = match int {
            0..=6 => 40,
            7..=10 => 35,
            11..=14 => 30,
            15..=17 => 25,
            _ => 20,
        };

        if state.turns % energy_rate as u64 == 0 && state.player.energy < state.player.energy_max {
            state.player.energy += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{ObjectClass, ObjectId};
    use crate::player::Attribute;

    /// Create a test state with normal dexterity (10) for neutral AC bonus
    fn test_state() -> GameState {
        let mut state = GameState::default();
        // Set dexterity to 10 for neutral AC bonus (0)
        state.player.attr_current.set(Attribute::Dexterity, 10);
        state
    }

    #[test]
    fn test_base_armor_class() {
        let state = test_state();
        // Base AC should be 10 when no armor is worn
        // Dexterity 10 gives +0 bonus
        assert_eq!(state.calculate_armor_class(), 10);
    }

    #[test]
    fn test_armor_class_with_dexterity() {
        let mut state = test_state();

        // High dexterity (18+) gives -4 AC bonus
        state.player.attr_current.set(Attribute::Dexterity, 18);
        assert_eq!(state.calculate_armor_class(), 6); // 10 - 0 + (-4) = 6

        // Low dexterity (3) gives +3 AC penalty
        state.player.attr_current.set(Attribute::Dexterity, 3);
        assert_eq!(state.calculate_armor_class(), 13); // 10 - 0 + 3 = 13
    }

    #[test]
    fn test_armor_class_with_worn_armor() {
        let mut state = test_state();

        // Create a piece of armor with base AC 3 (like plate mail)
        let mut armor = Object::new(ObjectId(1), 0, ObjectClass::Armor);
        armor.base_ac = 3;
        armor.worn_mask = 1; // Mark as worn
        armor.inv_letter = 'a';

        state.inventory.push(armor);

        // AC should be 10 - 3 = 7 (with neutral dex)
        assert_eq!(state.calculate_armor_class(), 7);
    }

    #[test]
    fn test_armor_class_with_enchanted_armor() {
        let mut state = test_state();

        // Create +2 armor with base AC 3
        let mut armor = Object::new(ObjectId(1), 0, ObjectClass::Armor);
        armor.base_ac = 3;
        armor.enchantment = 2;
        armor.worn_mask = 1;
        armor.inv_letter = 'a';

        state.inventory.push(armor);

        // AC should be 10 - (3 + 2) = 5
        assert_eq!(state.calculate_armor_class(), 5);
    }

    #[test]
    fn test_armor_class_with_eroded_armor() {
        let mut state = test_state();

        // Create rusted armor (erosion1 = 2)
        let mut armor = Object::new(ObjectId(1), 0, ObjectClass::Armor);
        armor.base_ac = 3;
        armor.erosion1 = 2;
        armor.worn_mask = 1;
        armor.inv_letter = 'a';

        state.inventory.push(armor);

        // AC should be 10 - (3 - 2) = 9
        assert_eq!(state.calculate_armor_class(), 9);
    }

    #[test]
    fn test_armor_class_multiple_pieces() {
        let mut state = test_state();

        // Armor: base AC 3
        let mut suit = Object::new(ObjectId(1), 0, ObjectClass::Armor);
        suit.base_ac = 3;
        suit.worn_mask = 1;
        suit.inv_letter = 'a';
        state.inventory.push(suit);

        // Shield: base AC 1
        let mut shield = Object::new(ObjectId(2), 0, ObjectClass::Armor);
        shield.base_ac = 1;
        shield.worn_mask = 2;
        shield.inv_letter = 'b';
        state.inventory.push(shield);

        // Helm: base AC 1
        let mut helm = Object::new(ObjectId(3), 0, ObjectClass::Armor);
        helm.base_ac = 1;
        helm.worn_mask = 4;
        helm.inv_letter = 'c';
        state.inventory.push(helm);

        // AC should be 10 - (3 + 1 + 1) = 5
        assert_eq!(state.calculate_armor_class(), 5);
    }

    #[test]
    fn test_unworn_armor_not_counted() {
        let mut state = test_state();

        // Create armor that's in inventory but not worn
        let mut armor = Object::new(ObjectId(1), 0, ObjectClass::Armor);
        armor.base_ac = 5;
        armor.worn_mask = 0; // Not worn
        armor.inv_letter = 'a';

        state.inventory.push(armor);

        // AC should still be 10 (armor not worn)
        assert_eq!(state.calculate_armor_class(), 10);
    }

    #[test]
    fn test_update_armor_class() {
        let mut state = test_state();
        state.update_armor_class();

        // Initial AC should be 10 (no armor, neutral dex)
        assert_eq!(state.player.armor_class, 10);

        // Add some armor
        let mut armor = Object::new(ObjectId(1), 0, ObjectClass::Armor);
        armor.base_ac = 4;
        armor.worn_mask = 1;
        armor.inv_letter = 'a';
        state.inventory.push(armor);

        // Update AC
        state.update_armor_class();

        // AC should now be 6
        assert_eq!(state.player.armor_class, 6);
    }
}
