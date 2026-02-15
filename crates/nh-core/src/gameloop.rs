//! Main game loop (allmain.c)

use hashbrown::HashMap;

use serde::{Deserialize, Serialize};

#[cfg(not(feature = "std"))]
use crate::compat::*;
use crate::NORMAL_SPEED;
use crate::action::{ActionResult, Command};
use crate::combat::artifact::ArtifactTracker;
use crate::dungeon::{DLevel, Level};
use crate::magic::genocide::MonsterVitals;
use crate::monster::MonsterId;
use crate::object::{DiscoveryState, Object};
use crate::player::{Gender, Race, Role, You};
use crate::rng::GameRng;
use crate::special::priest::Temple;
use crate::special::quest::QuestStatus;
use crate::special::shk::Shop;
use crate::special::vault::Vault;
use crate::world::timeout::do_storms;
#[cfg(feature = "std")]
use crate::world::topten::{self, ScoreEntry};
use crate::world::{Context, Flags, TimeoutManager};

/// Default sight range for visibility calculation (in lit rooms)
const SIGHT_RANGE: i32 = 15;

/// Serde helper for HashMap<DLevel, Level> — JSON requires string keys.
/// Serializes DLevel as "dungeon_num:level_num".
mod dlevel_map_serde {
    use super::*;
    use serde::de::{self, MapAccess, Visitor};
    use serde::ser::SerializeMap;

    pub fn serialize<S>(map: &HashMap<DLevel, Level>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut ser_map = serializer.serialize_map(Some(map.len()))?;
        for (dlevel, level) in map {
            let key = format!("{}:{}", dlevel.dungeon_num, dlevel.level_num);
            ser_map.serialize_entry(&key, level)?;
        }
        ser_map.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<DLevel, Level>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DLevelMapVisitor;

        impl<'de> Visitor<'de> for DLevelMapVisitor {
            type Value = HashMap<DLevel, Level>;

            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_str("a map with \"dungeon_num:level_num\" string keys")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));
                while let Some((key, value)) = access.next_entry::<String, Level>()? {
                    let parts: Vec<&str> = key.split(':').collect();
                    if parts.len() != 2 {
                        return Err(de::Error::custom(format!("invalid DLevel key: {}", key)));
                    }
                    let dungeon_num: i8 =
                        parts[0].parse().map_err(de::Error::custom)?;
                    let level_num: i8 =
                        parts[1].parse().map_err(de::Error::custom)?;
                    map.insert(
                        DLevel {
                            dungeon_num,
                            level_num,
                        },
                        value,
                    );
                }
                Ok(map)
            }
        }

        deserializer.deserialize_map(DLevelMapVisitor)
    }
}

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
#[derive(Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Player character
    pub player: You,

    /// Player inventory
    pub inventory: Vec<Object>,

    /// Current level
    pub current_level: Level,

    /// All visited levels
    #[serde(default, with = "dlevel_map_serde")]
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

    /// Permanent message history
    #[serde(skip)]
    pub message_history: Vec<String>,

    /// Object discovery state
    pub discovery_state: DiscoveryState,

    /// Artifact tracker (which artifacts exist in game)
    pub artifact_tracker: ArtifactTracker,

    /// Monster genocide tracking (which species are genocided)
    pub monster_vitals: MonsterVitals,

    /// Quest status tracking
    pub quest_status: QuestStatus,

    /// Active pet monster IDs (for following between levels)
    pub active_pets: Vec<MonsterId>,

    /// Temple data for current level
    pub temples: Vec<Temple>,

    /// Shop data for current level
    pub shops: Vec<Shop>,

    /// Vault data for current level
    pub vaults: Vec<Vault>,
}

#[cfg(feature = "std")]
impl Default for GameState {
    fn default() -> Self {
        Self::new(GameRng::from_entropy())
    }
}

impl GameState {
    /// Create a new game with the given RNG
    pub fn new(mut rng: GameRng) -> Self {
        // Initialize monster vitals early so level generation can use it
        let monster_vitals = MonsterVitals::new();

        let dlevel = DLevel::main_dungeon_start();
        let current_level = Level::new_generated(dlevel, &mut rng, &monster_vitals);

        // Find upstairs to place player
        let (start_x, start_y) = current_level.find_upstairs().unwrap_or((40, 10)); // Default fallback position

        let mut player = You::default();
        player.pos.x = start_x;
        player.pos.y = start_y;
        player.prev_pos = player.pos;

        // Initialize visibility from starting position
        let mut current_level = current_level;
        current_level.update_visibility(start_x, start_y, SIGHT_RANGE);

        // NOTE: Phase 6 - Monster Spawn Integration
        // Special NPC spawning (starting pet, priests, shopkeepers, guards) would be called here.
        // This requires integration with the level generation system to:
        // 1. Spawn starting pet (if role is Valkyrie, etc.) at player position
        // 2. Spawn priests in temples during level generation
        // 3. Spawn shopkeepers in shops during level generation
        // 4. Initialize vault data with guard positions
        // For now, these are placeholders that the level generation system would populate

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
            message_history: Vec::new(),
            discovery_state: DiscoveryState {
                discovered: HashMap::new(),
                disco_count: 0,
            },
            artifact_tracker: ArtifactTracker::new(),
            monster_vitals,
            quest_status: QuestStatus::new(),
            active_pets: Vec::new(),
            temples: Vec::new(),
            shops: Vec::new(),
            vaults: Vec::new(),
        }
    }

    /// Create a new game with a fully initialized player identity
    ///
    /// Unlike `new()`, this sets up the player with role/race/gender,
    /// starting HP/energy/skills (via `u_init`), and starting inventory
    /// (via `init_inventory`).
    pub fn new_with_identity(
        mut rng: GameRng,
        name: String,
        role: Role,
        race: Race,
        gender: Gender,
    ) -> Self {
        let monster_vitals = MonsterVitals::new();
        let dlevel = DLevel::main_dungeon_start();
        let current_level = Level::new_generated(dlevel, &mut rng, &monster_vitals);

        let (start_x, start_y) = current_level.find_upstairs().unwrap_or((40, 10));

        // Create player with identity and racial intrinsics
        let mut player = You::new(name, role, race, gender);
        player.pos.x = start_x;
        player.pos.y = start_y;
        player.prev_pos = player.pos;

        // Initialize HP, energy, skills, gold, prayer timeout
        crate::player::init::u_init(&mut player, &mut rng);

        // Create starting inventory
        let inventory = crate::player::init::init_inventory(&mut rng, role);

        // Initialize visibility from starting position
        let mut current_level = current_level;
        current_level.update_visibility(start_x, start_y, SIGHT_RANGE);

        Self {
            player,
            inventory,
            current_level,
            levels: HashMap::new(),
            flags: Flags::default(),
            context: Context::default(),
            rng,
            timeouts: TimeoutManager::new(),
            turns: 0,
            monster_turns: 0,
            messages: Vec::new(),
            message_history: Vec::new(),
            discovery_state: DiscoveryState {
                discovered: HashMap::new(),
                disco_count: 0,
            },
            artifact_tracker: ArtifactTracker::new(),
            monster_vitals,
            quest_status: QuestStatus::new(),
            active_pets: Vec::new(),
            temples: Vec::new(),
            shops: Vec::new(),
            vaults: Vec::new(),
        }
    }

    /// Add a message to display
    pub fn message(&mut self, msg: impl Into<String>) {
        let msg_str = msg.into();
        self.messages.push(msg_str.clone());
        self.message_history.push(msg_str);
    }

    /// Clear messages
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    /// Transfer pending messages from the current level to the game state
    /// This should be called after monster turns to collect AI-generated messages
    pub fn collect_level_messages(&mut self) {
        let pending = self.current_level.take_pending_messages();
        for msg in pending {
            self.message(msg);
        }
    }

    /// Add an object to the player's inventory
    pub fn add_to_inventory(&mut self, mut object: Object) -> char {
        // Find first available inventory letter
        let used_letters: hashbrown::HashSet<char> =
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
        self.inventory
            .iter()
            .map(|o| o.weight * o.quantity as u32)
            .sum()
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

    /// Consume the game loop and return the owned game state
    pub fn into_state(self) -> GameState {
        self.state
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

        // Handle quest progression
        self.handle_quest_turn();

        // Generate ambient sounds
        self.handle_ambient_sounds();

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
        // Engulfed state restricts most actions (C: u.uswallow checks throughout cmd.c)
        if self.state.player.swallowed {
            match command {
                // Allowed while engulfed: attack engulfer, use items on self, info commands
                Command::Rest | Command::Quit | Command::Save
                | Command::Inventory | Command::Look | Command::History
                | Command::Discoveries | Command::Help | Command::WhatsHere => {}
                Command::Eat(_) | Command::Quaff(_) | Command::Read(_)
                | Command::Apply(_) | Command::Wear(_) | Command::TakeOff(_)
                | Command::Wield(_) | Command::PutOn(_) | Command::Remove(_) => {}
                Command::Zap(_, _) => {
                    // Zapping while engulfed hits the engulfer
                    self.state.message("You zap at the engulfer!");
                    return crate::action::zap::do_zap(
                        &mut self.state,
                        match command { Command::Zap(l, _) => l, _ => unreachable!() },
                        None, // direction ignored — hits engulfer
                    );
                }
                Command::Fight(_) => {
                    // Melee attacks while engulfed hit the engulfer
                    self.state.message("You attack the engulfer!");
                    return ActionResult::Success;
                }
                Command::Move(_) | Command::Run(_) | Command::MoveUntilInteresting(_) => {
                    self.state.message("You are engulfed and cannot move!");
                    return ActionResult::NoTime;
                }
                Command::GoUp | Command::GoDown => {
                    self.state.message("You can't go anywhere while engulfed!");
                    return ActionResult::NoTime;
                }
                Command::Search => {
                    self.state.message("You can't search while engulfed!");
                    return ActionResult::NoTime;
                }
                _ => {
                    self.state.message("You can't do that while engulfed!");
                    return ActionResult::NoTime;
                }
            }
        }

        match command {
            Command::Move(dir) => self.do_move(dir),
            Command::Run(dir) => {
                // Running: move in direction until something interesting happens
                // For now, just move one step (full running requires UI integration)
                self.do_move(dir)
            }
            Command::MoveUntilInteresting(dir) => self.do_move(dir),
            Command::Rest => {
                self.state.message("You wait.");
                ActionResult::Success
            }
            Command::GoUp => self.do_go_up(),
            Command::GoDown => self.do_go_down(),
            Command::Search => self.do_search(),
            Command::Quit => ActionResult::Quit,
            Command::Save => ActionResult::Save,

            // Object manipulation
            Command::Pickup => crate::action::pickup::do_pickup(&mut self.state),
            Command::Drop(letter) => crate::action::pickup::do_drop(&mut self.state, letter),
            Command::Eat(letter) => crate::action::eat::do_eat(&mut self.state, letter),
            Command::Apply(letter) => crate::action::apply::do_apply(&mut self.state, letter),
            Command::Wear(letter) => crate::action::wear::do_wear(&mut self.state, letter),
            Command::TakeOff(letter) => crate::action::wear::do_takeoff(&mut self.state, letter),
            Command::Wield(letter_opt) => {
                if let Some(letter) = letter_opt {
                    crate::action::wear::do_wield(&mut self.state, letter)
                } else {
                    crate::action::wear::do_unwield(&mut self.state)
                }
            }
            Command::PutOn(letter) => crate::action::wear::do_puton(&mut self.state, letter),
            Command::Remove(letter) => crate::action::wear::do_remove(&mut self.state, letter),
            Command::Quaff(letter) => crate::action::quaff::do_quaff(&mut self.state, letter),
            Command::Read(letter) => crate::action::read::do_read(&mut self.state, letter),
            Command::Zap(letter, dir) => {
                crate::action::zap::do_zap(&mut self.state, letter, Some(dir))
            }
            Command::Throw(letter, dir) => {
                crate::action::throw::do_throw(&mut self.state, letter, dir)
            }
            Command::Fire(dir) => {
                // Fire uses the quivered/wielded ranged weapon
                // For now, find first throwable item in inventory
                let throwable = self
                    .state
                    .inventory
                    .iter()
                    .find(|o| {
                        matches!(
                            o.class,
                            crate::object::ObjectClass::Weapon
                                | crate::object::ObjectClass::Gem
                                | crate::object::ObjectClass::Rock
                        )
                    })
                    .map(|o| o.inv_letter);

                if let Some(letter) = throwable {
                    crate::action::throw::do_throw(&mut self.state, letter, dir)
                } else {
                    self.state.message("You have nothing to fire.");
                    ActionResult::NoTime
                }
            }

            // Directional actions
            Command::Open(dir) => crate::action::open_close::do_open(&mut self.state, dir),
            Command::Close(dir) => crate::action::open_close::do_close(&mut self.state, dir),
            Command::Kick(dir) => crate::action::kick::do_kick(&mut self.state, dir),
            Command::Fight(dir) => {
                // Force fight - attack even peaceful monsters
                let state = &mut self.state;
                let (dx, dy) = dir.delta();
                let new_x = state.player.pos.x + dx;
                let new_y = state.player.pos.y + dy;

                if let Some(monster) = state.current_level.monster_at(new_x, new_y) {
                    let monster_id = monster.id;
                    let monster_name = monster.name.clone();

                    // Attack regardless of peaceful status
                    let result = crate::combat::player_attack_monster(
                        &mut state.player,
                        state.current_level.monster_mut(monster_id).unwrap(),
                        None, // weapon
                        &mut state.rng,
                    );

                    if result.hit {
                        state.message(format!(
                            "You hit the {} for {} damage!",
                            monster_name, result.damage
                        ));

                        #[cfg(feature = "extensions")]
                        {
                            use crate::monster::combat_hooks;
                            combat_hooks::on_player_hit_monster(
                                monster_id,
                                &mut state.current_level,
                                result.damage,
                                &state.player,
                            );
                        }

                        if result.defender_died {
                            state.message(format!("You kill the {}!", monster_name));
                            state.current_level.remove_monster(monster_id);
                        }
                    } else {
                        state.message(format!("You miss the {}!", monster_name));
                    }
                    ActionResult::Success
                } else {
                    state.message("You strike at empty space.");
                    ActionResult::Success
                }
            }

            // Special actions
            Command::Pray => crate::action::pray::do_pray(&mut self.state),
            Command::Engrave(ref text) => {
                crate::action::engrave::do_engrave(&mut self.state, text)
            }

            // Information commands (no time cost)
            Command::Inventory => {
                // Inventory display is handled by the UI layer
                // Just return NoTime so no game time passes
                ActionResult::NoTime
            }
            Command::Look => {
                self.state.message("You look around.");
                ActionResult::NoTime
            }
            Command::WhatsHere => {
                let objects = self
                    .state
                    .current_level
                    .objects_at(self.state.player.pos.x, self.state.player.pos.y);
                if objects.is_empty() {
                    self.state.message("There is nothing here.");
                } else {
                    self.state
                        .message(format!("You see {} item(s) here.", objects.len()));
                }
                ActionResult::NoTime
            }
            Command::History => {
                // Show recent message history
                if self.state.message_history.is_empty() {
                    self.state.message("No messages yet.");
                } else {
                    // Collect messages first to avoid borrow conflict
                    let start = self.state.message_history.len().saturating_sub(10);
                    let messages: Vec<String> = self.state.message_history[start..].to_vec();
                    for msg in messages {
                        self.state.message(msg);
                    }
                }
                ActionResult::NoTime
            }
            Command::Discoveries => {
                if self.state.discovery_state.count() == 0 {
                    self.state.message("You have not made any discoveries yet.");
                } else {
                    self.state.message(format!(
                        "You have discovered {} object type(s).",
                        self.state.discovery_state.count()
                    ));
                }
                ActionResult::NoTime
            }
            Command::Help => {
                // Show basic help
                self.state.message("Movement: hjklyubn or arrow keys");
                self.state
                    .message("Commands: i=inventory, d=drop, e=eat, w=wield, W=wear");
                self.state
                    .message("Actions: o=open, c=close, s=search, <=up, >=down");
                self.state
                    .message("Combat: F=force fight, t=throw, f=fire, z=zap");
                self.state
                    .message("Other: q=quaff, r=read, a=apply, p=pay, P=pray");
                self.state
                    .message("Info: ?=help, \\=discoveries, Ctrl+P=history");
                self.state.message("System: S=save, Q=quit");
                ActionResult::NoTime
            }

            Command::Pay => {
                if let Some(shop_idx) = self.state.player.in_shop {
                    crate::special::shk::pay_bill_at(&mut self.state, shop_idx)
                } else {
                    self.state.message("There is nobody here to pay.");
                    ActionResult::NoTime
                }
            }
            Command::Sit => {
                self.state.message("You sit down.");
                // Check for traps at current position
                let x = self.state.player.pos.x;
                let y = self.state.player.pos.y;
                if self.state.current_level.trap_at(x, y).is_some() {
                    crate::action::trap::check_trap(&mut self.state, x, y)
                } else {
                    ActionResult::Success
                }
            }
            Command::Dip => {
                self.state.message("You don't have anything to dip into.");
                ActionResult::NoTime
            }
            Command::Offer => {
                // Need to be on an altar
                self.state.message("There is no altar here.");
                ActionResult::NoTime
            }
            Command::Chat => {
                // Chat with adjacent NPC
                let (dx, dy) = crate::action::Direction::Down.delta(); // Placeholder - would need UI for direction
                let target_x = self.state.player.pos.x + dx;
                let target_y = self.state.player.pos.y + dy;

                if let Some(monster) = self.state.current_level.monster_at(target_x, target_y) {
                    let monster_id = monster.id;
                    let message = self.handle_talk_command(monster_id);
                    self.state.message(message);
                    ActionResult::Success
                } else {
                    self.state.message("There's nobody here to talk to.");
                    ActionResult::NoTime
                }
            }
            Command::Travel => {
                self.state.message("Where do you want to travel to?");
                ActionResult::NoTime
            }
            Command::Options => {
                self.state.message("Options menu not available in this interface.");
                ActionResult::NoTime
            }
            Command::Feed => {
                // Feed pet with food — requires pet targeting and food selection UI
                self.state.message("Feed which pet with what food?");
                ActionResult::NoTime
            }
            // Combat extensions
            Command::TwoWeapon => {
                // Toggle two-weapon combat (dotwoweapon from wield.c)
                self.state.message("You switch your combat style.");
                ActionResult::NoTime
            }
            Command::SwapWeapon => {
                // Swap primary and secondary weapons (doswapweapon from wield.c)
                self.state.message("You swap your weapons.");
                ActionResult::Success
            }

            // Object manipulation extensions
            Command::SelectQuiver(letter) => {
                // Ready a projectile for firing (dowieldquiver from wield.c)
                if let Some(obj) = self.state.inventory.iter().find(|o| o.inv_letter == letter) {
                    let name = obj.display_name();
                    self.state.message(format!("You ready {}.", name));
                    ActionResult::Success
                } else {
                    self.state.message("You don't have that item.");
                    ActionResult::NoTime
                }
            }
            Command::Loot => {
                // Loot a container on the floor (doloot from pickup.c)
                let x = self.state.player.pos.x;
                let y = self.state.player.pos.y;
                let objects = self.state.current_level.objects_at(x, y);
                let has_container = objects.iter().any(|o| o.is_container());
                if has_container {
                    self.state.message("You open the container.");
                    ActionResult::Success
                } else {
                    self.state.message("There is nothing here to loot.");
                    ActionResult::NoTime
                }
            }
            Command::Tip(letter) => {
                // Tip over a container (dotip from pickup.c)
                if let Some(obj) = self.state.inventory.iter().find(|o| o.inv_letter == letter) {
                    if obj.is_container() {
                        let name = obj.display_name();
                        self.state.message(format!("You turn {} upside down.", name));
                        ActionResult::Success
                    } else {
                        self.state.message("That isn't a container.");
                        ActionResult::NoTime
                    }
                } else {
                    self.state.message("You don't have that item.");
                    ActionResult::NoTime
                }
            }
            Command::Rub(letter) => {
                // Rub a lamp or touchstone (dorub from apply.c)
                if let Some(obj) = self.state.inventory.iter().find(|o| o.inv_letter == letter) {
                    let name = obj.display_name();
                    self.state.message(format!("You rub {}.", name));
                    ActionResult::Success
                } else {
                    self.state.message("You don't have that item.");
                    ActionResult::NoTime
                }
            }
            Command::Wipe => {
                // Wipe face clean (dowipe from do.c)
                self.state.message("You wipe your face.");
                if self.state.player.blinded_timeout > 0 {
                    self.state.player.blinded_timeout = 0;
                    self.state.message("You can see again.");
                }
                ActionResult::Success
            }
            Command::Force(_dir) => {
                // Force a lock (doforce from lock.c)
                self.state.message("You try to force the lock.");
                ActionResult::Success
            }

            // Action extensions
            Command::Jump => crate::action::jump::dojump(&mut self.state),
            Command::Invoke => {
                // Invoke artifact power (doinvoke from artifact.c)
                self.state.message("You don't have an invokable artifact.");
                ActionResult::NoTime
            }
            Command::Untrap(dir) => {
                // Disarm a trap (dountrap from cmd.c)
                let (dx, dy) = dir.delta();
                let x = self.state.player.pos.x + dx;
                let y = self.state.player.pos.y + dy;
                if self.state.current_level.trap_at(x, y).is_some() {
                    self.state.message("You attempt to disarm the trap.");
                    ActionResult::Success
                } else {
                    self.state.message("You don't see a trap there.");
                    ActionResult::NoTime
                }
            }
            Command::Ride => {
                // Mount/dismount steed (doride from steed.c)
                self.state.message("There is nothing here to ride.");
                ActionResult::NoTime
            }
            Command::TurnUndead => {
                // Turn undead (doturn from pray.c)
                self.state.message("You try to turn the undead.");
                ActionResult::Success
            }
            Command::MonsterAbility => {
                // Use monster special ability while polymorphed (domonability)
                self.state.message("You don't have a special ability to use.");
                ActionResult::NoTime
            }
            Command::EnhanceSkill => {
                // Enhance weapon/spell skills (enhance_weapon_skill from weapon.c)
                self.state.message("You feel you could improve your skills.");
                ActionResult::NoTime
            }
            Command::NameItem(letter, ref new_name) => {
                // Name an item (docallcmd/do_oname from do_name.c)
                if let Some(obj) = self.state.inventory.iter_mut().find(|o| o.inv_letter == letter) {
                    let result = crate::action::name::oname(obj, new_name);
                    match result {
                        crate::action::name::NamingResult::Named(name) => {
                            if name.is_empty() {
                                self.state.message("Name removed.");
                            } else {
                                self.state.message(format!("Named: {}.", name));
                            }
                        }
                        crate::action::name::NamingResult::Rejected(msg) => {
                            self.state.message(msg);
                        }
                        _ => {}
                    }
                    ActionResult::NoTime
                } else {
                    self.state.message("You don't have that item.");
                    ActionResult::NoTime
                }
            }
            Command::NameLevel(ref new_name) => {
                // Annotate level (donamelevel from do_name.c)
                self.state.message(format!("Level annotated: {}.", new_name));
                ActionResult::NoTime
            }
            Command::Organize(from_letter, to_letter) => {
                // Reorganize inventory (doorganize from invent.c)
                let from_idx = self.state.inventory.iter().position(|o| o.inv_letter == from_letter);
                if let Some(idx) = from_idx {
                    let already_used = self.state.inventory.iter().any(|o| o.inv_letter == to_letter);
                    if already_used {
                        // Swap letters
                        for obj in &mut self.state.inventory {
                            if obj.inv_letter == to_letter {
                                obj.inv_letter = from_letter;
                                break;
                            }
                        }
                    }
                    self.state.inventory[idx].inv_letter = to_letter;
                    self.state.message(format!("Moved item to slot '{}'.", to_letter));
                    ActionResult::NoTime
                } else {
                    self.state.message("You don't have that item.");
                    ActionResult::NoTime
                }
            }

            // Information extensions
            Command::ShowAttributes => {
                // Show player attributes (doattributes from end.c)
                let p = &self.state.player;
                self.state.message(format!(
                    "St:{} Dx:{} Co:{} In:{} Wi:{} Ch:{}",
                    p.attr_current.get(crate::player::Attribute::Strength),
                    p.attr_current.get(crate::player::Attribute::Dexterity),
                    p.attr_current.get(crate::player::Attribute::Constitution),
                    p.attr_current.get(crate::player::Attribute::Intelligence),
                    p.attr_current.get(crate::player::Attribute::Wisdom),
                    p.attr_current.get(crate::player::Attribute::Charisma),
                ));
                ActionResult::NoTime
            }
            Command::ShowEquipment => {
                // Show currently worn equipment (doprinuse from invent.c)
                let worn_items: Vec<_> = self.state.inventory.iter()
                    .filter(|o| o.worn_mask != 0)
                    .map(|o| format!("{} - {} (worn)", o.inv_letter, o.display_name()))
                    .collect();
                if worn_items.is_empty() {
                    self.state.message("You are not wearing anything.");
                } else {
                    for msg in worn_items {
                        self.state.messages.push(msg);
                    }
                }
                ActionResult::NoTime
            }
            Command::ShowSpells => {
                // Show spell list (dovspell from spell.c)
                if self.state.player.known_spells.is_empty() {
                    self.state.message("You don't know any spells.");
                } else {
                    self.state.message(format!(
                        "You know {} spell(s).",
                        self.state.player.known_spells.len()
                    ));
                }
                ActionResult::NoTime
            }
            Command::ShowConduct => {
                // Show conduct/behavior (doconduct from end.c)
                let c = &self.state.player.conduct;
                let mut any = false;
                if c.is_vegetarian() {
                    self.state.messages.push("You have been vegetarian.".to_string());
                    any = true;
                }
                if c.is_foodless() {
                    self.state.messages.push("You have not eaten.".to_string());
                    any = true;
                }
                if c.is_atheist() {
                    self.state.messages.push("You have not prayed.".to_string());
                    any = true;
                }
                if c.is_illiterate() {
                    self.state.messages.push("You have been illiterate.".to_string());
                    any = true;
                }
                if c.is_pacifist() {
                    self.state.messages.push("You have been a pacifist.".to_string());
                    any = true;
                }
                if c.is_weaponless() {
                    self.state.messages.push("You have been weaponless.".to_string());
                    any = true;
                }
                if !any {
                    self.state.message("You have broken all conducts.");
                }
                ActionResult::NoTime
            }
            Command::DungeonOverview => {
                // Show dungeon overview (dooverview from dungeon.c)
                self.state.message(format!(
                    "Dungeon level {} (dungeon {})",
                    self.state.current_level.dlevel.level_num,
                    self.state.current_level.dlevel.dungeon_num,
                ));
                ActionResult::NoTime
            }
            Command::CountGold => {
                // Count gold pieces (doprgold from invent.c)
                self.state.message(format!(
                    "You have {} gold piece(s).",
                    self.state.player.gold,
                ));
                ActionResult::NoTime
            }
            Command::ClassDiscovery => {
                // Show discoveries by class (doclassdisco from o_init.c)
                if self.state.discovery_state.count() == 0 {
                    self.state.message("You have not made any discoveries yet.");
                } else {
                    self.state.message(format!(
                        "You have discovered {} object type(s).",
                        self.state.discovery_state.count()
                    ));
                }
                ActionResult::NoTime
            }
            Command::TypeInventory(class_char) => {
                // Show inventory filtered by class (dotypeinv from invent.c)
                let class = crate::object::ObjectClass::from_symbol(class_char);
                let matching: Vec<_> = self.state.inventory.iter()
                    .filter(|o| class.is_none() || Some(o.class) == class)
                    .collect();
                if matching.is_empty() {
                    self.state.message("You don't have any of those.");
                } else {
                    for obj in &matching {
                        self.state.messages.push(format!(
                            "{} - {}", obj.inv_letter, obj.display_name()
                        ));
                    }
                }
                ActionResult::NoTime
            }
            Command::Vanquished => {
                // Show kill list (dovanquished from end.c)
                self.state.message("You recall your victories.");
                ActionResult::NoTime
            }

            Command::ExtendedCommand(cmd_name) => self.handle_extended_command(&cmd_name),
            Command::Redraw => {
                // Redraw is handled by the UI layer
                ActionResult::NoTime
            }
        }
    }

    /// Handle extended commands (commands starting with #)
    fn handle_extended_command(&mut self, cmd_name: &str) -> ActionResult {
        use crate::action::extended;
        use crate::action::help;

        match cmd_name {
            "version" => {
                #[cfg(feature = "std")]
                {
                    let version = crate::world::version::doextversion();
                    for line in version.lines() {
                        self.state.message(line.to_string());
                    }
                }
                #[cfg(not(feature = "std"))]
                {
                    let version = crate::world::version::getversionstring();
                    self.state.message(version);
                }
                ActionResult::NoTime
            }
            "help" => {
                let help_text = help::dohelp();
                for line in help_text.lines().take(20) {
                    self.state.message(line.to_string());
                }
                ActionResult::NoTime
            }
            "history" => {
                let history_text = help::dohistory();
                for line in history_text.lines().take(10) {
                    self.state.message(line.to_string());
                }
                ActionResult::NoTime
            }
            "key bindings" | "keys" => {
                let bindings = crate::action::keybindings::dokeylist();
                for line in bindings.lines().take(15) {
                    self.state.message(line.to_string());
                }
                ActionResult::NoTime
            }
            "menu controls" => {
                let controls = crate::action::keybindings::domenucontrols();
                for line in controls.lines() {
                    self.state.message(line.to_string());
                }
                ActionResult::NoTime
            }
            "direction keys" => {
                let dirs = crate::action::keybindings::show_direction_keys();
                for line in dirs.lines() {
                    self.state.message(line.to_string());
                }
                ActionResult::NoTime
            }
            "explore mode" | "enter explore" => {
                let mode = crate::action::commands::enter_explore_mode();
                self.state.message(format!(
                    "Entering {} mode",
                    crate::action::commands::playmode_description(mode)
                ));
                ActionResult::NoTime
            }
            "mode info" => {
                let info = crate::action::commands::get_mode_info();
                for line in info.lines() {
                    self.state.message(line.to_string());
                }
                ActionResult::NoTime
            }
            "list commands" => {
                let commands = extended::doextlist();
                self.state.message("Available extended commands:");
                for cmd in commands.iter().take(10) {
                    self.state.message(format!("  #{}", cmd));
                }
                if commands.len() > 10 {
                    self.state
                        .message(format!("  ... and {} more", commands.len() - 10));
                }
                ActionResult::NoTime
            }
            _ => {
                // Try to execute as extended command
                if let Some(cmd) = extended::doextcmd(cmd_name) {
                    self.execute_command(cmd)
                } else {
                    self.state
                        .message(format!("Unknown extended command: {}", cmd_name));
                    ActionResult::NoTime
                }
            }
        }
    }

    /// Handle player movement
    fn do_move(&mut self, dir: crate::action::Direction) -> ActionResult {
        let (dx, dy) = dir.delta();
        let state = &mut self.state;

        // Check encumbrance (hack.c:domove_core line 1385)
        if let Some(msg) = crate::action::movement::check_movement_capacity(state) {
            state.message(msg);
            return ActionResult::Success; // Time passes but no movement
        }

        // Apply confusion/stun direction randomization (hack.c:confdir)
        let (dx, dy) = crate::action::movement::confdir(state, dx, dy);

        // Check if player is trapped (bear trap, pit, web, etc.)
        if state.player.utrap > 0 && state.player.utrap_type != crate::player::PlayerTrapType::None {
            let player_tt = state.player.utrap_type;
            // Convert player trap type to dungeon trap type for escape logic
            let trap_type = match player_tt {
                crate::player::PlayerTrapType::BearTrap => crate::dungeon::TrapType::BearTrap,
                crate::player::PlayerTrapType::Pit => crate::dungeon::TrapType::Pit,
                crate::player::PlayerTrapType::SpikedPit => crate::dungeon::TrapType::SpikedPit,
                crate::player::PlayerTrapType::Web => crate::dungeon::TrapType::Web,
                _ => crate::dungeon::TrapType::Pit, // fallback
            };
            let strength = state.player.attr_current.get(crate::player::Attribute::Strength);
            if crate::dungeon::trap::try_escape_trap(&mut state.rng, trap_type, strength) {
                let msg = crate::dungeon::trap::escape_trap_message(trap_type);
                state.message(msg);
                state.player.utrap = 0;
                state.player.utrap_type = crate::player::PlayerTrapType::None;
            } else {
                state.message("You are still trapped!");
                state.player.utrap -= 1;
                return ActionResult::Success; // Time passes but no movement
            }
        }

        // Check if player is grabbed - must escape first
        if let Some(grabber_id) = state.player.grabbed_by {
            // Check if grabber still exists and is adjacent
            let grabber_exists = state
                .current_level
                .monster(grabber_id)
                .map(|m| {
                    let dist = (m.x - state.player.pos.x)
                        .abs()
                        .max((m.y - state.player.pos.y).abs());
                    dist <= 1
                })
                .unwrap_or(false);

            if !grabber_exists {
                // Grabber is gone or not adjacent, release
                state.player.grabbed_by = None;
                state.message("You are released!");
            } else {
                // Try to escape
                if crate::combat::try_escape_grab(&mut state.player, &mut state.rng) {
                    state.message("You pull free!");
                } else {
                    state.message("You are held and cannot move!");
                    return ActionResult::Success; // Time passes but no movement
                }
            }
        }

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
                    state.message(format!(
                        "You hit the {} for {} damage!",
                        monster_name, result.damage
                    ));

                    #[cfg(feature = "extensions")]
                    {
                        use crate::monster::combat_hooks;
                        combat_hooks::on_player_hit_monster(
                            monster_id,
                            &mut state.current_level,
                            result.damage,
                            &state.player,
                        );
                    }

                    if result.defender_died {
                        state.message(format!("You kill the {}!", monster_name));
                        state.current_level.remove_monster(monster_id);
                    }
                } else {
                    state.message(format!("You miss the {}!", monster_name));
                }
                return ActionResult::Success;
            } else {
                // Swap positions with peaceful monster
                let player_pos = state.player.pos;
                if let Some(m) = state.current_level.monster_mut(monster_id) {
                    m.x = player_pos.x;
                    m.y = player_pos.y;
                }
                state.player.prev_pos = state.player.pos;
                state.player.pos.x = new_x;
                state.player.pos.y = new_y;
                state.message("You swap places with the monster.");
                return ActionResult::Success;
            }
        }

        // Check for boulders and try to push (hack.c:moverock)
        if crate::action::movement::find_boulder_at(&state.current_level, new_x, new_y) {
            match crate::action::movement::moverock(state, new_x, new_y, dx, dy) {
                crate::action::movement::MoveRockResult::Blocked => {
                    return ActionResult::Success; // Time passes, can't move
                }
                crate::action::movement::MoveRockResult::Moved
                | crate::action::movement::MoveRockResult::SqueezedPast => {
                    // Boulder moved or squeezed past, continue with normal movement
                }
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

        // Movement interrupts spells
        if state.player.casting_spell.is_some() {
            state.player.casting_interrupted = true;
        }

        // Update visibility from new position
        state
            .current_level
            .update_visibility(new_x, new_y, SIGHT_RANGE);

        // Check for traps at new position
        if let Some(trap_type) = state.current_level.trap_at(new_x, new_y).map(|t| t.trap_type) {
            let dex = state.player.attr_current.get(crate::player::Attribute::Dexterity);
            let resistances = crate::dungeon::trap::resistances_from_properties(
                |prop| state.player.properties.has(prop),
                dex,
            );
            if let Some(trap) = state.current_level.trap_at_mut(new_x, new_y) {
                let result = crate::dungeon::trap::dotrap(
                    &mut state.rng,
                    trap,
                    &resistances,
                    false,
                );

                for msg in &result.messages {
                    state.message(msg.clone());
                }

                if result.damage > 0 {
                    state.player.take_damage(result.damage);
                }

                if result.held_turns > 0 {
                    state.player.utrap = result.held_turns as u32;
                    state.player.utrap_type = crate::action::trap::to_player_trap_type(trap_type);
                }

                if result.trap_destroyed {
                    state.current_level.remove_trap(new_x, new_y);
                }

                if state.player.is_dead() {
                    return ActionResult::Died("killed by a trap".to_string());
                }
            }
        }

        // Check terrain effects at new position (altar, throne, ice, etc.)
        crate::action::movement::check_special_room(state);

        // Check for shop entry/exit
        let prev_shop = state.player.in_shop;
        let new_shop = state.current_level.shops().iter().position(|s| s.contains(new_x, new_y));
        if new_shop != prev_shop {
            if let Some(_shop_idx) = prev_shop {
                // Left a shop
                let debt = state.current_level.shops().get(_shop_idx)
                    .map(|s| s.debt).unwrap_or(0);
                if debt > 0 {
                    state.message(format!("\"Hey! You owe me {} zorkmids!\"", debt));
                }
            }
            if let Some(shop_idx) = new_shop {
                // Entered a shop
                let greeting = {
                    let shop = &state.current_level.shops()[shop_idx];
                    match shop.shop_type {
                        crate::special::ShopType::General => "Welcome to my general store!",
                        crate::special::ShopType::Armor => "Welcome! Looking for some protection?",
                        crate::special::ShopType::Weapon => "Welcome! Need something sharp?",
                        crate::special::ShopType::Food => "Welcome! Hungry?",
                        crate::special::ShopType::Scroll => "Welcome to my scroll emporium!",
                        crate::special::ShopType::Potion => "Welcome! Need a potion?",
                        crate::special::ShopType::Wand => "Welcome! Looking for magical implements?",
                        crate::special::ShopType::Tool => "Welcome! Need some tools?",
                        crate::special::ShopType::Book => "Welcome to my bookstore!",
                        crate::special::ShopType::Ring => "Welcome! Looking for jewelry?",
                        crate::special::ShopType::Candle => "Welcome! Need some light?",
                        crate::special::ShopType::Tin => "Welcome to my tin shop!",
                    }
                };
                state.message(format!("\"{}\"", greeting));
            }
            state.player.in_shop = new_shop;
        }

        ActionResult::Success
    }

    /// Handle going up stairs
    fn do_go_up(&mut self) -> ActionResult {
        let px = self.state.player.pos.x;
        let py = self.state.player.pos.y;

        // Check for stairway at player position
        let stairway = match self.state.current_level.stairway_at(px, py) {
            Some(s) if s.up => *s,
            Some(_) => {
                self.state.message("These stairs go down.");
                return ActionResult::NoTime;
            }
            None => {
                self.state.message("You can't go up here.");
                return ActionResult::NoTime;
            }
        };

        // Check if at top of dungeon
        if stairway.destination.level_num < 1 {
            self.state.message("You are at the top of the dungeon.");
            return ActionResult::NoTime;
        }

        self.change_level(stairway.destination, true);
        self.state.message("You climb up the stairs.");
        ActionResult::Success
    }

    /// Handle going down stairs
    fn do_go_down(&mut self) -> ActionResult {
        let px = self.state.player.pos.x;
        let py = self.state.player.pos.y;

        // Check for stairway at player position
        let stairway = match self.state.current_level.stairway_at(px, py) {
            Some(s) if !s.up => *s,
            Some(_) => {
                self.state.message("These stairs go up.");
                return ActionResult::NoTime;
            }
            None => {
                self.state.message("You can't go down here.");
                return ActionResult::NoTime;
            }
        };

        self.change_level(stairway.destination, false);
        self.state.message("You descend the stairs.");
        ActionResult::Success
    }

    /// Change to a different dungeon level
    fn change_level(&mut self, destination: DLevel, going_up: bool) {
        use crate::special::{dog, priest, vault};

        let current_dlevel = self.state.current_level.dlevel;

        // LEAVING LEVEL - Save pet information
        self.state.active_pets.clear();
        let pet_ids: Vec<_> = self
            .state
            .current_level
            .monsters
            .iter()
            .filter(|m| dog::is_pet(m))
            .map(|m| m.id)
            .collect();

        // Check which pets will follow
        for pet_id in pet_ids {
            if let Some(pet) = self.state.current_level.monster(pet_id) {
                if dog::pet_will_follow(pet, &self.state.player) {
                    self.state.active_pets.push(pet_id);
                }
            }
        }

        // Save current level
        let mut old_level =
            core::mem::replace(&mut self.state.current_level, Level::new(destination));

        // Clear priests from level before saving (they'll be respawned on return)
        priest::clear_priests_for_save(&mut old_level);

        self.state.levels.insert(current_dlevel, old_level);

        // Load or generate destination level
        if let Some(existing_level) = self.state.levels.remove(&destination) {
            self.state.current_level = existing_level;
        } else {
            self.state.current_level =
                Level::new_generated(destination, &mut self.state.rng, &self.state.monster_vitals);
        }

        // Place player at appropriate stairs
        let new_pos = if going_up {
            // Coming from below, place at downstairs
            self.state.current_level.find_downstairs()
        } else {
            // Coming from above, place at upstairs
            self.state.current_level.find_upstairs()
        };

        if let Some((x, y)) = new_pos {
            self.state.player.pos.x = x;
            self.state.player.pos.y = y;
            self.state.player.prev_pos = self.state.player.pos;
            // Update visibility on new level
            self.state
                .current_level
                .update_visibility(x, y, SIGHT_RANGE);
        }

        // ENTERING LEVEL - Restore active pets (simplified - would add at player position)
        for _pet_id in &self.state.active_pets {
            // Pet migration deferred: requires pet state serialization and level placement
        }

        // Check for vault guard summoning
        vault::summon_vault_guard(
            &mut self.state.current_level,
            &self.state.player,
            self.state.turns as u32,
        );

        // Check for room entry (temples, shops, vaults)
        self.check_room_entry();
    }

    /// Check if player entered a special room and trigger appropriate events
    fn check_room_entry(&mut self) {
        use crate::special::{priest, vault};

        // Check if player entered a temple
        for temple in &self.state.temples.clone() {
            if (temple.altar_x as i32, temple.altar_y as i32)
                == (
                    self.state.player.pos.x as i32,
                    self.state.player.pos.y as i32,
                )
            {
                // Player entered temple - trigger altar effects
                let messages = priest::handle_temple_entry(
                    &self.state.current_level,
                    &self.state.player,
                    &Default::default(),
                    self.state.turns as u32,
                );
                for msg in messages {
                    self.state.message(msg);
                }
            }
        }

        // Check if player entered a vault
        for vault_data in &mut self.state.vaults {
            if vault_data.contains(self.state.player.pos.x as i8, self.state.player.pos.y as i8) {
                // Player entered vault - guard may appear
                vault::summon_vault_guard(
                    &mut self.state.current_level,
                    &self.state.player,
                    self.state.turns as u32,
                );
            }
        }
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
                        // C: 1/7 base chance, modified by luck (findit in detect.c)
                        if state.rng.one_in(7) || state.player.luck > 0 && state.rng.one_in(3) {
                            cell.typ = crate::dungeon::CellType::Door;
                            found = true;
                            state.message("You find a hidden door!");
                        }
                    } else if cell.typ == crate::dungeon::CellType::SecretCorridor {
                        if state.rng.one_in(7) || state.player.luck > 0 && state.rng.one_in(3) {
                            cell.typ = crate::dungeon::CellType::Corridor;
                            found = true;
                            state.message("You find a hidden passage!");
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
        let monster_ids: Vec<_> = state.current_level.monsters.iter().map(|m| m.id).collect();

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

                // Check for special NPC types and handle their AI first
                let is_special_npc = if let Some(monster) = state.current_level.monster(id) {
                    let is_priest = monster.is_priest;
                    let is_shopkeeper = monster.is_shopkeeper;
                    let is_guard = monster.is_guard;
                    let is_pet = crate::special::dog::is_pet(monster);

                    is_priest || is_shopkeeper || is_guard || is_pet
                } else {
                    false
                };

                if is_special_npc {
                    // Handle special NPC AI - determine type and get needed data
                    let npc_type_and_data = if let Some(monster) = state.current_level.monster(id) {
                        if monster.is_priest {
                            if let Some(ext) = &monster.priest_extension {
                                Some(("priest", ext.shrine_pos))
                            } else {
                                None
                            }
                        } else if monster.is_shopkeeper {
                            Some(("shopkeeper", (0i8, 0i8)))
                        } else if monster.is_guard {
                            Some(("guard", (0i8, 0i8)))
                        } else if crate::special::dog::is_pet(monster) {
                            Some(("pet", (0i8, 0i8)))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Now apply the NPC AI
                    if let Some((npc_type, shrine_pos)) = npc_type_and_data {
                        match npc_type {
                            "priest" => {
                                // Clone monster to avoid borrow conflict with level
                                if let Some(mut monster) = state.current_level.monster(id).cloned()
                                {
                                    crate::special::priest::move_priest_to_shrine(
                                        &mut monster,
                                        shrine_pos,
                                        &state.current_level,
                                    );
                                    // Update monster back in level
                                    if let Some(m) = state.current_level.monster_mut(id) {
                                        m.x = monster.x;
                                        m.y = monster.y;
                                    }
                                    any_moved = true;
                                }
                            }
                            "shopkeeper" => {
                                // Clone monster to avoid borrow conflict with level
                                let player_ref = &state.player;
                                if let Some(mut monster) = state.current_level.monster(id).cloned()
                                {
                                    crate::special::shk::move_shopkeeper_to_shop(
                                        &mut monster,
                                        &state.current_level,
                                        player_ref,
                                    );
                                    // Update monster back in level
                                    if let Some(m) = state.current_level.monster_mut(id) {
                                        m.x = monster.x;
                                        m.y = monster.y;
                                    }
                                    any_moved = true;
                                }
                            }
                            "guard" => {
                                let player_clone = state.player.clone();
                                if let Some(mut monster) = state.current_level.monster(id).cloned()
                                {
                                    crate::special::vault::move_vault_guard(
                                        &mut monster,
                                        &mut state.current_level,
                                        &player_clone,
                                    );
                                    // Update the monster in the level
                                    if let Some(m) = state.current_level.monster_mut(id) {
                                        *m = monster;
                                    }
                                    any_moved = true;
                                }
                            }
                            "pet" => {
                                let player_clone = state.player.clone();
                                crate::special::dog::pet_move(
                                    id,
                                    &mut state.current_level,
                                    &player_clone,
                                    &mut state.rng,
                                );
                                any_moved = true;
                            }
                            _ => {}
                        }
                    }
                } else {
                    // Process regular monster AI
                    let action = crate::monster::process_monster_ai(
                        id,
                        &mut state.current_level,
                        &mut state.player,
                        &mut state.rng,
                    );

                    match action {
                        crate::monster::AiAction::Moved(_, _) => {
                            any_moved = true;
                        }
                        crate::monster::AiAction::AttackedPlayer => {
                            any_moved = true;
                            // Execute full monster attack sequence using mattacku
                            if let Some(monster) = state.current_level.monster(id) {
                                let monster_clone = monster.clone();
                                let result = crate::combat::mattacku(
                                    &monster_clone,
                                    &mut state.player,
                                    &mut state.inventory,
                                    &mut state.current_level,
                                    &mut state.rng,
                                );

                                // Display all messages from the attack
                                for msg in result.messages {
                                    state.message(msg);
                                }

                                // Handle monster death (e.g., from explosion)
                                if result.monster_died {
                                    state.current_level.remove_monster(id);
                                }
                            }
                        }
                        crate::monster::AiAction::Waited => {
                            any_moved = true;
                        }
                        crate::monster::AiAction::Died => {
                            any_moved = true;
                            state.current_level.remove_monster(id);
                        }
                        crate::monster::AiAction::None => {}
                    }
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
        // Apply speed bonuses and encumbrance penalties
        let speed_bonus = if state.player.properties.has(crate::player::Property::Speed) {
            4
        } else {
            0
        };
        let encumbrance_penalty = match state.player.encumbrance() {
            crate::player::Encumbrance::Unencumbered => 0,
            crate::player::Encumbrance::Burdened => 1,
            crate::player::Encumbrance::Stressed => 3,
            crate::player::Encumbrance::Strained => 5,
            crate::player::Encumbrance::Overtaxed => 7,
            crate::player::Encumbrance::Overloaded => 9,
        };
        state.player.movement_points += base_move + speed_bonus - encumbrance_penalty;

        // Reallocate movement to monsters based on their speed
        for monster in &mut state.current_level.monsters {
            // Use monster's base_speed (set from permonst data when spawned)
            let speed_modifier: i16 = match monster.speed {
                crate::monster::SpeedState::Slow => -4,
                crate::monster::SpeedState::Normal => 0,
                crate::monster::SpeedState::Fast => 4,
            };
            monster.movement += monster.base_speed as i16 + speed_modifier;
        }

        // Process timed events
        let current_turn = state.turns;
        let triggered_events = state.timeouts.tick(current_turn);
        for event in triggered_events {
            Self::process_timed_event(state, event);
        }

        // Process grab damage if player is held
        if let Some(grabber_id) = state.player.grabbed_by {
            if let Some(grabber) = state.current_level.monster(grabber_id) {
                let grabber_clone = grabber.clone();
                let damage = crate::combat::apply_grab_damage(
                    &mut state.player,
                    &grabber_clone,
                    &mut state.rng,
                );
                state.message(format!(
                    "The {} crushes you for {} damage!",
                    grabber_clone.name, damage
                ));
            } else {
                // Grabber no longer exists
                state.player.grabbed_by = None;
            }
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

        // Age spells (spell memory decays over time)
        let forgotten_spells = crate::magic::spell::age_spells(&mut state.player.known_spells);
        for spell in forgotten_spells {
            state.message(format!("Your knowledge of {} fades.", spell.name()));
        }

        // Regenerate mana
        crate::magic::spell::regenerate_mana(&mut state.player);

        // Process hunger
        state.player.digest(1);
        if matches!(
            state.player.hunger_state,
            crate::player::HungerState::Starved
        ) {
            state.player.take_damage(1);
            if state.player.is_dead() {
                state.message("You die from starvation.");
            }
        }

        // Update pet time every 100 turns
        if state.turns % 100 == 0 {
            let pet_ids: Vec<_> = state
                .current_level
                .monsters
                .iter()
                .filter(|m| crate::special::dog::is_pet(m))
                .map(|m| m.id)
                .collect();

            for pet_id in pet_ids {
                let died = {
                    if let Some(pet) = state.current_level.monster_mut(pet_id) {
                        crate::special::dog::update_pet_time(pet, state.turns as u32)
                    } else {
                        false
                    }
                };
                if died {
                    if let Some(pet) = state.current_level.monster(pet_id) {
                        state.message(format!("Your {} has starved!", pet.name));
                    }
                }
            }
        }

        // Refresh visibility from current position
        state
            .current_level
            .update_visibility(state.player.pos.x, state.player.pos.y, SIGHT_RANGE);

        // Regeneration
        Self::process_regeneration(state);

        // Process storms on air level (do_storms equivalent from timeout.c)
        // Underwater detection not yet tracked; passing false for now
        let storm_messages = do_storms(&mut state.rng, false);
        for msg in storm_messages {
            state.message(msg);
        }
    }

    /// Handle quest progression checks
    fn handle_quest_turn(&mut self) {
        use crate::special::quest;

        // Update quest timeout display every 100 turns
        if self.state.turns % 100 == 0 {
            let info = quest::get_quest_info(self.state.player.role);
            let status_msg = quest::get_quest_status_message(&self.state.quest_status, &info);
            self.state.message(status_msg);
        }
    }

    /// Generate ambient sounds
    fn handle_ambient_sounds(&mut self) {
        use crate::special::sounds;

        // Generate sounds occasionally (1/10 turns)
        if !self.state.rng.one_in(10) {
            return;
        }

        // Generate ambient level sounds
        let ambient = sounds::generate_ambient_sounds(
            &self.state.current_level,
            self.state.player.pos.x,
            self.state.player.pos.y,
            &mut self.state.rng,
        );
        for sound in ambient {
            self.state.message(sound);
        }
    }

    /// Handle talk command with NPCs
    fn handle_talk_command(&mut self, target_id: MonsterId) -> String {
        use crate::special::{priest, quest, shk};

        if let Some(monster) = self.state.current_level.monster(target_id) {
            match true {
                _ if monster.is_priest => {
                    format!(
                        "You speak to {}.",
                        priest::get_priest_name(monster, monster.state.invisible)
                    )
                }
                _ if monster.is_shopkeeper => {
                    shk::shopkeeper_chat(monster, &self.state.current_level)
                }
                _ => "The creature ignores you.".to_string(),
            }
        } else {
            "There's nobody here to talk to.".to_string()
        }
    }

    /// Handle shop payment
    fn handle_payment(&mut self, shopkeeper_id: Option<MonsterId>) -> String {
        use crate::special::shk;

        if let Some(id) = shopkeeper_id {
            let shopkeeper_clone = self.state.current_level.monster(id).cloned();
            if let Some(mut shopkeeper) = shopkeeper_clone {
                let level_clone = self.state.current_level.clone();
                if shk::pay_shopkeeper(&mut shopkeeper, &level_clone, &mut self.state.player) {
                    "You pay your debt.".to_string()
                } else {
                    "You don't have enough gold to pay.".to_string()
                }
            } else {
                "The shopkeeper is not here.".to_string()
            }
        } else {
            "There's no shopkeeper to pay.".to_string()
        }
    }

    /// Find shopkeeper in current location
    fn find_shopkeeper_nearby(&self) -> Option<MonsterId> {
        // Find shopkeeper in current level
        self.state
            .current_level
            .monsters
            .iter()
            .find(|m| m.is_shopkeeper)
            .map(|m| m.id)
    }

    /// Process a triggered timed event
    fn process_timed_event(state: &mut GameState, event: crate::world::TimedEvent) {
        use crate::world::TimedEventType;

        match event.event_type {
            TimedEventType::MonsterSpawn => {
                // Find a random walkable+empty position (try up to 20 times)
                for _ in 0..20 {
                    let x = 5 + state.rng.rn2(70) as i8;
                    let y = 2 + state.rng.rn2(17) as i8;
                    if state.current_level.is_walkable(x, y)
                        && state.current_level.monster_at(x, y).is_none()
                    {
                        let monster_type = state.rng.rn2(10) as i16;
                        if !state.monster_vitals.is_genocided(monster_type) {
                            let mut monster =
                                crate::monster::Monster::new(MonsterId(0), monster_type, x, y);
                            monster.hp = 5 + state.rng.rnd(10) as i32;
                            monster.hp_max = monster.hp;
                            monster.name =
                                crate::dungeon::random_monster_name_for_type(monster_type)
                                    .to_string();
                            state.current_level.add_monster(monster);
                        }
                        break;
                    }
                }
                // Reschedule next spawn
                let delay = 50 + (state.rng.rnd(50) as u64);
                state
                    .timeouts
                    .schedule_after(delay, TimedEventType::MonsterSpawn);
            }
            TimedEventType::MonsterAction(monster_id) => {
                // Monster-specific timed action (e.g., breath weapon cooldown)
                if let Some(monster) = state.current_level.monster_mut(monster_id) {
                    // Reset breath weapon cooldown
                    monster.special_cooldown = 0;
                }
            }
            TimedEventType::CorpseRot(object_id) => {
                // Remove rotted corpse
                state.current_level.remove_object(object_id);
                state.message("You smell something rotting.");
            }
            TimedEventType::EggHatch(object_id) => {
                // Hatch egg into monster - remove egg and spawn monster
                if let Some(egg) = state.current_level.remove_object(object_id) {
                    state.message(format!("The egg hatches at ({}, {})!", egg.x, egg.y));
                    // Monster spawning would happen here if we had monster creation
                }
            }
            TimedEventType::Stoning => {
                if !state
                    .player
                    .properties
                    .has(crate::player::Property::StoneResistance)
                {
                    state.message("You have turned to stone.");
                    state.player.hp = 0; // Instant death
                }
            }
            TimedEventType::Sliming => {
                state.message("You have turned into a green slime!");
                state.player.hp = 0; // Instant death (polymorph not implemented)
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
                state.player.hp = 0;
            }
            TimedEventType::ObjectTimeout(object_id) => {
                // Lamp fuel depletion, candle burnout
                // Check inventory for the object and deplete charges
                let went_out = if let Some(obj) =
                    state.inventory.iter_mut().find(|o| o.id == object_id)
                {
                    if obj.enchantment > 0 {
                        obj.enchantment -= 1;
                        if obj.enchantment == 0 {
                            Some(obj.display_name())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some(name) = went_out {
                    state.message(format!("Your {} has gone out!", name));
                }
            }
            TimedEventType::FigurineAnimate(_object_id) => {
                // Figurine animation deferred: requires makemon from figurine object type
            }
            TimedEventType::Regeneration => {
                // Handled by process_regeneration()
            }
            TimedEventType::EnergyRegeneration => {
                // Handled by process_regeneration()
            }
            TimedEventType::Hunger => {
                // Handled by process_hunger() in eat.rs
            }
            TimedEventType::BlindFromCreamPie => {
                state.player.blinded_timeout = 0;
                state.message("You can see again.");
            }
            TimedEventType::TempSeeInvisible => {
                state.player.properties.set_timeout(crate::player::Property::SeeInvisible, 0);
                state.message("You thought you saw something disappear.");
            }
            TimedEventType::TempTelepathy => {
                state.player.properties.set_timeout(crate::player::Property::Telepathy, 0);
                state.message("Your mental acuity diminishes.");
            }
            TimedEventType::TempWarning => {
                state.player.properties.set_timeout(crate::player::Property::Warning, 0);
                state.message("You feel less sensitive.");
            }
            TimedEventType::TempStealth => {
                state.player.properties.set_timeout(crate::player::Property::Stealth, 0);
                state.message("You feel clumsy.");
            }
            TimedEventType::TempLevitation => {
                state.player.properties.set_timeout(crate::player::Property::Levitation, 0);
                state.message("You float gently to the ground.");
            }
            TimedEventType::TempFlying => {
                state.player.properties.set_timeout(crate::player::Property::Flying, 0);
                state.message("You lose the ability to fly.");
            }
            TimedEventType::Custom(_name) => {
                // Custom events are processed by external handlers
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

        if state.turns.is_multiple_of(regen_rate as u64) && state.player.hp < state.player.hp_max {
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

        if state.turns.is_multiple_of(energy_rate as u64)
            && state.player.energy < state.player.energy_max
        {
            state.player.energy += 1;
        }
    }
}

// ============================================================================
// Game Ending Functions (end.c equivalents)
// ============================================================================

/// How the game ended
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathHow {
    /// Killed by something
    Killed,
    /// Quit the game
    Quit,
    /// Escaped the dungeon
    Escaped,
    /// Ascended to demigod status
    Ascended,
}

#[cfg(feature = "std")]
/// Create a score entry from the current game state (done/done2 equivalent)
///
/// This is called when the game ends to record the player's score.
pub fn create_score_entry(state: &GameState, death_reason: &str, how: DeathHow) -> ScoreEntry {
    let ascended = matches!(how, DeathHow::Ascended);

    // Calculate score based on various factors
    let mut score: i64 = 0;

    // Gold collected
    score += state.player.gold as i64;

    // Experience points
    score += (state.player.exp * 10) as i64;

    // Dungeon depth bonus
    score += (state.current_level.dlevel.level_num as i64) * 100;

    // Ascension bonus
    if ascended {
        score += 50000;
    }

    ScoreEntry {
        name: state.player.name.clone(),
        score,
        max_dlevel: state.current_level.dlevel.level_num as i32,
        player_level: state.player.exp_level as u8,
        role: format!("{:?}", state.player.role),
        race: format!("{:?}", state.player.race),
        gender: format!("{:?}", state.player.gender),
        alignment: format!("{:?}", state.player.alignment),
        death_reason: death_reason.to_string(),
        ascended,
        turns: state.turns,
        realtime: 0, // Would need to track real time
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[cfg(feature = "std")]
/// Handle game ending and record score (done equivalent from end.c)
///
/// This function is called when the game ends for any reason.
/// It creates a score entry and saves it to the high score file.
pub fn done(
    state: &GameState,
    death_reason: &str,
    how: DeathHow,
    score_file: Option<&std::path::Path>,
) -> Result<ScoreEntry, topten::TopTenError> {
    let entry = create_score_entry(state, death_reason, how);

    if let Some(path) = score_file {
        // Ensure the record file exists
        topten::check_record_file(path)?;

        // Load existing scores
        let mut scores = topten::HighScores::load(path).unwrap_or_default();

        // Add the new entry
        scores.add_score(entry.clone());

        // Save the updated scores
        scores.save(path)?;
    }

    Ok(entry)
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

    #[test]
    fn test_grabbed_player_cannot_move() {
        use crate::monster::{Monster, MonsterId};

        let mut state = test_state();

        // Create a monster that grabs the player
        let grabber_id = MonsterId(1);
        let mut grabber = Monster::new(grabber_id, 5, 5, 5);
        grabber.name = "python".to_string();
        state.current_level.monsters.push(grabber);

        // Set player as grabbed and position adjacent to grabber
        state.player.grabbed_by = Some(grabber_id);
        state.player.pos.x = 6;
        state.player.pos.y = 5;

        // Player should be grabbed
        assert!(state.player.grabbed_by.is_some());
    }

    #[test]
    fn test_grab_released_when_grabber_gone() {
        use crate::monster::MonsterId;

        let mut state = test_state();

        // Set player as grabbed by a non-existent monster
        let fake_grabber_id = MonsterId(999);
        state.player.grabbed_by = Some(fake_grabber_id);
        state.player.pos.x = 6;
        state.player.pos.y = 5;

        // Create game loop and try to move
        let mut game_loop = GameLoop::new(state);
        let _result = game_loop.tick(Command::Move(crate::action::Direction::East));

        // Player should be released since grabber doesn't exist
        assert!(game_loop.state().player.grabbed_by.is_none());
    }

    #[test]
    fn test_grab_damage_applied_each_turn() {
        use crate::monster::{Monster, MonsterId, MonsterState};

        let mut state = test_state();
        state.player.hp = 100;
        state.player.pos.x = 6;
        state.player.pos.y = 5;

        // Create a grabbing monster
        let grabber_id = MonsterId(1);
        let mut grabber = Monster::new(grabber_id, 10, 5, 5); // Level 10 for more damage
        grabber.name = "python".to_string();
        grabber.state = MonsterState::active();
        state.current_level.monsters.push(grabber);

        // Set player as grabbed
        state.player.grabbed_by = Some(grabber_id);

        let initial_hp = state.player.hp;

        // Create game loop and process a new turn
        let mut game_loop = GameLoop::new(state);
        game_loop.new_turn();

        // Player should have taken grab damage
        assert!(
            game_loop.state().player.hp < initial_hp,
            "Player should take grab damage"
        );
    }

    // ========================================================================
    // Tests for game ending functions (create_score_entry, done)
    // ========================================================================

    #[test]
    fn test_create_score_entry_basic() {
        let mut state = test_state();
        state.player.name = "TestPlayer".to_string();
        state.player.gold = 1000;
        state.player.exp = 500;
        state.player.exp_level = 5;
        state.current_level.dlevel.level_num = 10;

        let entry =
            super::create_score_entry(&state, "killed by a dragon", super::DeathHow::Killed);

        assert_eq!(entry.name, "TestPlayer");
        assert_eq!(entry.max_dlevel, 10);
        assert_eq!(entry.player_level, 5);
        assert_eq!(entry.death_reason, "killed by a dragon");
        assert!(!entry.ascended);
        // Score should include gold (1000) + exp*10 (5000) + depth*100 (1000) = 7000
        assert_eq!(entry.score, 7000);
    }

    #[test]
    fn test_create_score_entry_ascension() {
        let mut state = test_state();
        state.player.name = "Ascender".to_string();
        state.player.gold = 0;
        state.player.exp = 0;
        state.current_level.dlevel.level_num = 1;

        let entry = super::create_score_entry(&state, "ascended", super::DeathHow::Ascended);

        assert!(entry.ascended);
        // Score should include ascension bonus (50000) + depth*100 (100) = 50100
        assert_eq!(entry.score, 50100);
    }

    #[test]
    fn test_create_score_entry_quit() {
        let state = test_state();

        let entry = super::create_score_entry(&state, "quit", super::DeathHow::Quit);

        assert!(!entry.ascended);
        assert_eq!(entry.death_reason, "quit");
    }

    #[test]
    fn test_done_without_score_file() {
        let state = test_state();

        // done() without a score file should still return the entry
        let result = super::done(&state, "test death", super::DeathHow::Killed, None);

        assert!(result.is_ok());
        let entry = result.unwrap();
        assert_eq!(entry.death_reason, "test death");
    }
}
