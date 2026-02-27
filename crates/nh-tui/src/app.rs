//! Application state and main UI controller

use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use nh_core::action::{Command, Direction as GameDirection};
use nh_core::object::ObjectClass;
use nh_core::player::{AlignmentType, Gender, Race, Role};
use nh_core::{GameLoop, GameLoopResult, GameState};
use strum::IntoEnumIterator;

use crate::input::key_to_command;
use crate::theme::Theme;
use crate::display::{self, GlyphSet, GraphicsMode};
use crate::widgets::{InventoryWidget, MapWidget, MessagesWidget, StatusWidget};

/// UI mode - what the app is currently displaying/waiting for
#[derive(Debug, Clone)]
pub enum UiMode {
    /// Normal gameplay
    Normal,
    /// Character creation wizard
    CharacterCreation(CharacterCreationState),
    /// Showing inventory (read-only)
    Inventory,
    /// Selecting an item for an action
    ItemSelect {
        prompt: String,
        action: PendingAction,
        filter: Option<ObjectClass>,
    },
    /// Selecting a direction
    DirectionSelect {
        prompt: String,
        action: PendingAction,
    },
    /// Typing an extended command (#command)
    ExtendedCommandInput { input: String },
    /// Showing help
    Help,
    /// Death screen showing final statistics
    DeathScreen { cause: String },
}

/// Character creation state machine
#[derive(Debug, Clone)]
pub enum CharacterCreationState {
    /// Entering player name
    EnterName { name: String },
    /// Asking if user wants random character
    AskRandom { name: String, cursor: usize },
    /// Selecting role
    SelectRole { name: String, cursor: usize },
    /// Selecting race
    SelectRace {
        name: String,
        role: Role,
        cursor: usize,
    },
    /// Selecting gender
    SelectGender {
        name: String,
        role: Role,
        race: Race,
        cursor: usize,
    },
    /// Selecting alignment
    SelectAlignment {
        name: String,
        role: Role,
        race: Race,
        gender: Gender,
        cursor: usize,
    },
    /// Done - ready to start game
    Done {
        name: String,
        role: Role,
        race: Race,
        gender: Gender,
        alignment: AlignmentType,
    },
}

/// Character creation result
#[derive(Debug, Clone)]
pub struct CharacterChoices {
    pub name: String,
    pub role: Role,
    pub race: Race,
    pub gender: Gender,
    pub alignment: AlignmentType,
}

/// Action waiting for additional input
#[derive(Debug, Clone, Copy)]
pub enum PendingAction {
    Drop,
    Eat,
    Apply,
    Wear,
    TakeOff,
    Wield,
    PutOn,
    Remove,
    Open,
    Close,
    Kick,
    Quaff,
    Read,
    Zap,
    Fight,
    Fire,
    Throw,
    /// Throw: item already selected, waiting for direction
    ThrowDir(char),
    Untrap,
    Force,
}

/// Application state
pub struct App {
    /// Game loop controller
    game_loop: GameLoop,

    /// Should quit
    should_quit: bool,

    /// Number pad mode
    num_pad: bool,

    /// Current UI mode
    mode: UiMode,

    /// Selection menu for item picking
    selection_cursor: usize,

    /// Color theme (adapts to light/dark terminal background)
    theme: Theme,

    /// Glyph set for rendering map features
    glyph_set: Box<dyn GlyphSet>,
}

impl App {
    /// Create a new application with a new game
    pub fn new(state: GameState, theme: Theme, graphics_mode: GraphicsMode) -> Self {
        Self {
            game_loop: GameLoop::new(state),
            should_quit: false,
            num_pad: false,
            mode: UiMode::Normal,
            selection_cursor: 0,
            theme,
            glyph_set: display::detect_glyph_set(graphics_mode),
        }
    }

    /// Get game state
    pub fn state(&self) -> &GameState {
        self.game_loop.state()
    }

    /// Get mutable game state
    pub fn state_mut(&mut self) -> &mut GameState {
        self.game_loop.state_mut()
    }

    /// Check if app should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Signal that the app should quit
    pub fn set_should_quit(&mut self) {
        self.should_quit = true;
    }

    /// Handle input event - returns a command if one should be executed
    pub fn handle_event(&mut self, event: Event) -> Option<Command> {
        if let Event::Key(key) = event {
            // Check for quit (always available)
            if key.code == KeyCode::Char('Q') && key.modifiers.contains(KeyModifiers::SHIFT) {
                self.should_quit = true;
                return None;
            }

            // Handle based on current UI mode
            match &self.mode {
                UiMode::Normal => self.handle_normal_input(key),
                UiMode::CharacterCreation(_) => {
                    self.handle_character_creation_input(key);
                    None
                }
                UiMode::Inventory => {
                    self.handle_inventory_input(key);
                    None
                }
                UiMode::ItemSelect { action, .. } => {
                    let action = *action;
                    self.handle_item_select_input(key, action)
                }
                UiMode::DirectionSelect { action, .. } => {
                    let action = *action;
                    self.handle_direction_select_input(key, action)
                }
                UiMode::ExtendedCommandInput { .. } => self.handle_extended_command_input(key),
                UiMode::Help => {
                    self.handle_help_input(key);
                    None
                }
                UiMode::DeathScreen { .. } => {
                    self.handle_death_screen_input(key);
                    None
                }
            }
        } else {
            None
        }
    }

    /// Handle input in normal gameplay mode
    ///
    /// Key bindings follow the original NetHack cmd.c conventions.
    fn handle_normal_input(&mut self, key: crossterm::event::KeyEvent) -> Option<Command> {
        // Handle Ctrl key combos first
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                // Ctrl+D: kick (NetHack convention)
                KeyCode::Char('d') => {
                    self.enter_direction_select("Kick in which direction?", PendingAction::Kick);
                    None
                }
                _ => key_to_command(key, self.num_pad),
            };
        }

        match key.code {
            // ================================================================
            // Commands that need item selection
            // ================================================================
            KeyCode::Char('d') => {                                          // d: drop
                self.enter_item_select("Drop what?", PendingAction::Drop, None);
                None
            }
            KeyCode::Char('e') => {                                          // e: eat
                self.enter_item_select("Eat what?", PendingAction::Eat, Some(ObjectClass::Food));
                None
            }
            KeyCode::Char('a') => {                                          // a: apply
                self.enter_item_select(
                    "Apply what?",
                    PendingAction::Apply,
                    Some(ObjectClass::Tool),
                );
                None
            }
            KeyCode::Char('W') => {                                          // W: wear armor
                self.enter_item_select("Wear what?", PendingAction::Wear, Some(ObjectClass::Armor));
                None
            }
            KeyCode::Char('T') => {                                          // T: take off armor
                self.enter_item_select(
                    "Take off what?",
                    PendingAction::TakeOff,
                    Some(ObjectClass::Armor),
                );
                None
            }
            KeyCode::Char('w') => {                                          // w: wield weapon
                self.enter_item_select(
                    "Wield what?",
                    PendingAction::Wield,
                    Some(ObjectClass::Weapon),
                );
                None
            }
            KeyCode::Char('P') => {                                          // P: put on ring/amulet
                self.enter_item_select("Put on what?", PendingAction::PutOn, None);
                None
            }
            KeyCode::Char('R') => {                                          // R: remove ring/amulet
                self.enter_item_select("Remove what?", PendingAction::Remove, None);
                None
            }
            KeyCode::Char('q') => {                                          // q: quaff potion
                self.enter_item_select(
                    "Quaff what?",
                    PendingAction::Quaff,
                    Some(ObjectClass::Potion),
                );
                None
            }
            KeyCode::Char('r') => {                                          // r: read scroll/book
                self.enter_item_select(
                    "Read what?",
                    PendingAction::Read,
                    Some(ObjectClass::Scroll),
                );
                None
            }
            KeyCode::Char('z') => {                                          // z: zap wand
                self.enter_item_select("Zap what?", PendingAction::Zap, Some(ObjectClass::Wand));
                None
            }
            KeyCode::Char('t') => {                                          // t: throw
                self.enter_item_select("Throw what?", PendingAction::Throw, None);
                None
            }

            // ================================================================
            // Commands that need direction selection
            // ================================================================
            KeyCode::Char('o') => {                                          // o: open door
                self.enter_direction_select("Open in which direction?", PendingAction::Open);
                None
            }
            KeyCode::Char('c') => {                                          // c: close door
                self.enter_direction_select("Close in which direction?", PendingAction::Close);
                None
            }
            KeyCode::Char('F') => {                                          // F: fight (force attack)
                self.enter_direction_select("Fight in which direction?", PendingAction::Fight);
                None
            }
            KeyCode::Char('f') => {                                          // f: fire from quiver
                self.enter_direction_select("Fire in which direction?", PendingAction::Fire);
                None
            }

            // ================================================================
            // Extended command (#)
            // ================================================================
            KeyCode::Char('#') => {
                self.mode = UiMode::ExtendedCommandInput {
                    input: String::new(),
                };
                None
            }

            // ================================================================
            // Inventory display
            // ================================================================
            KeyCode::Char('i') => {
                self.mode = UiMode::Inventory;
                None
            }

            // Help
            KeyCode::Char('?') => {
                self.mode = UiMode::Help;
                None
            }

            // All other commands go through key_to_command (movement, simple actions, etc.)
            _ => key_to_command(key, self.num_pad),
        }
    }

    /// Handle input when viewing inventory
    fn handle_inventory_input(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char(' ') | KeyCode::Char('i') => {
                self.mode = UiMode::Normal;
            }
            _ => {}
        }
    }

    /// Handle input when selecting an item
    fn handle_item_select_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        action: PendingAction,
    ) -> Option<Command> {
        match key.code {
            KeyCode::Esc => {
                self.mode = UiMode::Normal;
                None
            }
            KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                // Check if this letter is in inventory
                if self.game_loop.state().get_inventory_item(c).is_some() {
                    self.mode = UiMode::Normal;
                    // action_with_item may transition to another mode (e.g., Throw → direction)
                    self.action_with_item(action, c)
                } else {
                    self.game_loop
                        .state_mut()
                        .message("You don't have that item.");
                    None
                }
            }
            _ => None,
        }
    }

    /// Handle input when selecting a direction
    fn handle_direction_select_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        action: PendingAction,
    ) -> Option<Command> {
        let direction = match key.code {
            KeyCode::Esc => {
                self.mode = UiMode::Normal;
                return None;
            }
            // Vi keys
            KeyCode::Char('h') if !self.num_pad => Some(GameDirection::West),
            KeyCode::Char('j') if !self.num_pad => Some(GameDirection::South),
            KeyCode::Char('k') if !self.num_pad => Some(GameDirection::North),
            KeyCode::Char('l') if !self.num_pad => Some(GameDirection::East),
            KeyCode::Char('y') if !self.num_pad => Some(GameDirection::NorthWest),
            KeyCode::Char('u') if !self.num_pad => Some(GameDirection::NorthEast),
            KeyCode::Char('b') if !self.num_pad => Some(GameDirection::SouthWest),
            KeyCode::Char('n') if !self.num_pad => Some(GameDirection::SouthEast),
            KeyCode::Char('.') => Some(GameDirection::Self_),
            // Arrow keys
            KeyCode::Up => Some(GameDirection::North),
            KeyCode::Down => Some(GameDirection::South),
            KeyCode::Left => Some(GameDirection::West),
            KeyCode::Right => Some(GameDirection::East),
            _ => None,
        };

        if let Some(dir) = direction {
            self.mode = UiMode::Normal;
            Some(self.action_with_direction(action, dir))
        } else {
            None
        }
    }

    /// Handle input in extended command mode (#command)
    fn handle_extended_command_input(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> Option<Command> {
        let input = match &self.mode {
            UiMode::ExtendedCommandInput { input } => input.clone(),
            _ => return None,
        };

        match key.code {
            KeyCode::Esc => {
                self.mode = UiMode::Normal;
                None
            }
            KeyCode::Enter => {
                self.mode = UiMode::Normal;
                if input.is_empty() {
                    None
                } else {
                    // Look up the extended command and dispatch
                    self.dispatch_extended_command(&input)
                }
            }
            KeyCode::Backspace => {
                let mut new_input = input;
                new_input.pop();
                self.mode = UiMode::ExtendedCommandInput { input: new_input };
                None
            }
            KeyCode::Char(c) if c.is_ascii_alphabetic() => {
                let mut new_input = input;
                new_input.push(c);
                self.mode = UiMode::ExtendedCommandInput { input: new_input };
                None
            }
            _ => None,
        }
    }

    /// Dispatch a named extended command to the appropriate Command
    fn dispatch_extended_command(&mut self, name: &str) -> Option<Command> {
        let lower = name.to_lowercase();
        match lower.as_str() {
            // Actions that need no extra input
            "pray" => Some(Command::Pray),
            "offer" => Some(Command::Offer),
            "sit" => Some(Command::Sit),
            "chat" => Some(Command::Chat),
            "pay" => Some(Command::Pay),
            "dip" => Some(Command::Dip),
            "jump" => Some(Command::Jump),
            "ride" => Some(Command::Ride),
            "wipe" => Some(Command::Wipe),
            "invoke" => Some(Command::Invoke),
            "turn" => Some(Command::TurnUndead),
            "monster" => Some(Command::MonsterAbility),
            "enhance" => Some(Command::EnhanceSkill),
            "loot" => Some(Command::Loot),
            "travel" => Some(Command::Travel),
            "twoweapon" => Some(Command::TwoWeapon),
            "swap" => Some(Command::SwapWeapon),
            "search" => Some(Command::Search),
            "save" => Some(Command::Save),
            "quit" => Some(Command::Quit),
            "discoveries" | "known" => Some(Command::Discoveries),
            "history" => Some(Command::History),
            "attributes" => Some(Command::ShowAttributes),
            "conduct" => Some(Command::ShowConduct),
            "overview" => Some(Command::DungeonOverview),
            "spells" => Some(Command::ShowSpells),
            "equipment" => Some(Command::ShowEquipment),
            "inventory" => Some(Command::Inventory),
            "vanquished" => Some(Command::Vanquished),
            "redraw" => Some(Command::Redraw),
            "gold" => Some(Command::CountGold),
            // Direction-needing commands go through direction select
            "untrap" => {
                self.enter_direction_select("Untrap in which direction?", PendingAction::Untrap);
                None
            }
            "force" => {
                self.enter_direction_select("Force in which direction?", PendingAction::Force);
                None
            }
            "fight" => {
                self.enter_direction_select("Fight in which direction?", PendingAction::Fight);
                None
            }
            "kick" => {
                self.enter_direction_select("Kick in which direction?", PendingAction::Kick);
                None
            }
            "open" => {
                self.enter_direction_select("Open in which direction?", PendingAction::Open);
                None
            }
            "close" => {
                self.enter_direction_select("Close in which direction?", PendingAction::Close);
                None
            }
            _ => {
                self.game_loop
                    .state_mut()
                    .message(format!("Unknown extended command: #{}", name));
                None
            }
        }
    }

    /// Handle input when viewing help
    fn handle_help_input(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char(' ') | KeyCode::Char('?') => {
                self.mode = UiMode::Normal;
            }
            _ => {}
        }
    }

    /// Enter item selection mode
    fn enter_item_select(
        &mut self,
        prompt: &str,
        action: PendingAction,
        filter: Option<ObjectClass>,
    ) {
        self.mode = UiMode::ItemSelect {
            prompt: prompt.to_string(),
            action,
            filter,
        };
        self.selection_cursor = 0;
    }

    /// Enter direction selection mode
    fn enter_direction_select(&mut self, prompt: &str, action: PendingAction) {
        self.mode = UiMode::DirectionSelect {
            prompt: prompt.to_string(),
            action,
        };
    }

    /// Create a command for an action with an item.
    /// Returns None if the action needs further input (e.g., Throw needs a direction next).
    fn action_with_item(&mut self, action: PendingAction, letter: char) -> Option<Command> {
        match action {
            PendingAction::Drop => Some(Command::ExtendedCommand(format!("drop {}", letter))),
            PendingAction::Eat => Some(Command::ExtendedCommand(format!("eat {}", letter))),
            PendingAction::Apply => Some(Command::ExtendedCommand(format!("apply {}", letter))),
            PendingAction::Wear => Some(Command::ExtendedCommand(format!("wear {}", letter))),
            PendingAction::TakeOff => Some(Command::ExtendedCommand(format!("takeoff {}", letter))),
            PendingAction::Wield => Some(Command::ExtendedCommand(format!("wield {}", letter))),
            PendingAction::PutOn => Some(Command::ExtendedCommand(format!("puton {}", letter))),
            PendingAction::Remove => Some(Command::ExtendedCommand(format!("remove {}", letter))),
            PendingAction::Quaff => Some(Command::ExtendedCommand(format!("quaff {}", letter))),
            PendingAction::Read => Some(Command::ExtendedCommand(format!("read {}", letter))),
            PendingAction::Zap => Some(Command::ExtendedCommand(format!("zap {}", letter))),
            PendingAction::Throw => {
                // Throw needs a direction after item selection
                self.enter_direction_select(
                    "Throw in which direction?",
                    PendingAction::ThrowDir(letter),
                );
                None
            }
            _ => None,
        }
    }

    /// Create a command for an action with a direction
    fn action_with_direction(&self, action: PendingAction, dir: GameDirection) -> Command {
        match action {
            PendingAction::Open => Command::Open(dir),
            PendingAction::Close => Command::Close(dir),
            PendingAction::Kick => Command::Kick(dir),
            PendingAction::Fight => Command::Fight(dir),
            PendingAction::Fire => Command::Fire(dir),
            PendingAction::ThrowDir(item) => Command::Throw(item, dir),
            PendingAction::Untrap => Command::Untrap(dir),
            PendingAction::Force => Command::Force(dir),
            _ => Command::Rest, // Should not happen
        }
    }

    /// Execute a command and update state
    pub fn execute(&mut self, command: Command) -> GameLoopResult {
        self.game_loop.state_mut().clear_messages();

        // Handle extended commands with parameters
        if let Command::ExtendedCommand(ref cmd) = command {
            return self.execute_extended_command(cmd);
        }

        let result = self.game_loop.tick(command);

        match &result {
            GameLoopResult::PlayerDied(cause) => {
                self.mode = UiMode::DeathScreen {
                    cause: cause.clone(),
                };
            }
            GameLoopResult::PlayerQuit => {
                self.should_quit = true;
            }
            GameLoopResult::SaveAndQuit => {
                self.should_quit = true;
            }
            _ => {}
        }

        result
    }

    /// Execute an extended command with parameters
    fn execute_extended_command(&mut self, cmd: &str) -> GameLoopResult {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return GameLoopResult::Continue;
        }

        let result = match parts[0] {
            "drop" if parts.len() > 1 => {
                let letter = parts[1].chars().next().unwrap_or(' ');
                nh_core::action::pickup::do_drop(self.game_loop.state_mut(), letter)
            }
            "eat" if parts.len() > 1 => {
                let letter = parts[1].chars().next().unwrap_or(' ');
                nh_core::action::eat::do_eat(self.game_loop.state_mut(), letter)
            }
            "apply" if parts.len() > 1 => {
                let letter = parts[1].chars().next().unwrap_or(' ');
                nh_core::action::apply::do_apply(self.game_loop.state_mut(), letter)
            }
            "wear" if parts.len() > 1 => {
                let letter = parts[1].chars().next().unwrap_or(' ');
                nh_core::action::wear::do_wear(self.game_loop.state_mut(), letter)
            }
            "takeoff" if parts.len() > 1 => {
                let letter = parts[1].chars().next().unwrap_or(' ');
                nh_core::action::wear::do_takeoff(self.game_loop.state_mut(), letter)
            }
            "wield" if parts.len() > 1 => {
                let letter = parts[1].chars().next().unwrap_or(' ');
                nh_core::action::wear::do_wield(self.game_loop.state_mut(), letter)
            }
            "puton" if parts.len() > 1 => {
                let letter = parts[1].chars().next().unwrap_or(' ');
                nh_core::action::wear::do_puton(self.game_loop.state_mut(), letter)
            }
            "remove" if parts.len() > 1 => {
                let letter = parts[1].chars().next().unwrap_or(' ');
                nh_core::action::wear::do_remove(self.game_loop.state_mut(), letter)
            }
            "open" if parts.len() > 1 => {
                if let Some(dir) = self.parse_direction(parts[1]) {
                    nh_core::action::open_close::do_open(self.game_loop.state_mut(), dir)
                } else {
                    nh_core::action::ActionResult::NoTime
                }
            }
            "close" if parts.len() > 1 => {
                if let Some(dir) = self.parse_direction(parts[1]) {
                    nh_core::action::open_close::do_close(self.game_loop.state_mut(), dir)
                } else {
                    nh_core::action::ActionResult::NoTime
                }
            }
            "kick" if parts.len() > 1 => {
                if let Some(dir) = self.parse_direction(parts[1]) {
                    nh_core::action::kick::do_kick(self.game_loop.state_mut(), dir)
                } else {
                    nh_core::action::ActionResult::NoTime
                }
            }
            "quaff" if parts.len() > 1 => {
                let letter = parts[1].chars().next().unwrap_or(' ');
                nh_core::action::quaff::do_quaff(self.game_loop.state_mut(), letter)
            }
            "read" if parts.len() > 1 => {
                let letter = parts[1].chars().next().unwrap_or(' ');
                nh_core::action::read::do_read(self.game_loop.state_mut(), letter)
            }
            "zap" if parts.len() > 1 => {
                let letter = parts[1].chars().next().unwrap_or(' ');
                // Zap needs a direction - for now, zap forward
                nh_core::action::zap::do_zap(self.game_loop.state_mut(), letter, None)
            }
            _ => {
                self.game_loop.state_mut().message("Unknown command.");
                nh_core::action::ActionResult::NoTime
            }
        };

        // Convert ActionResult to GameLoopResult
        match result {
            nh_core::action::ActionResult::Died(msg) => GameLoopResult::PlayerDied(msg),
            nh_core::action::ActionResult::Quit => GameLoopResult::PlayerQuit,
            nh_core::action::ActionResult::Save => GameLoopResult::SaveAndQuit,
            _ => GameLoopResult::Continue,
        }
    }

    /// Parse a direction string
    fn parse_direction(&self, s: &str) -> Option<GameDirection> {
        match s {
            "n" => Some(GameDirection::North),
            "s" => Some(GameDirection::South),
            "e" => Some(GameDirection::East),
            "w" => Some(GameDirection::West),
            "ne" => Some(GameDirection::NorthEast),
            "nw" => Some(GameDirection::NorthWest),
            "se" => Some(GameDirection::SouthEast),
            "sw" => Some(GameDirection::SouthWest),
            "." => Some(GameDirection::Self_),
            _ => None,
        }
    }

    /// Render the UI
    pub fn render(&mut self, frame: &mut Frame) {
        // Layout: map at top, status in middle, messages at bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(nh_core::ROWNO as u16 + 2), // Map + border
                Constraint::Length(2),                      // Status lines
                Constraint::Length(3),                      // Messages
            ])
            .split(frame.area());

        // Render map
        let state = self.game_loop.state();
        let map_widget =
            MapWidget::new(&state.current_level, &state.player, &self.theme, self.glyph_set.as_ref());
        frame.render_widget(map_widget, chunks[0]);

        // Render status and messages (re-borrow state after map rendering)
        {
            let state = self.game_loop.state();
            let status_widget = StatusWidget::new(&state.player, &self.theme);
            frame.render_widget(status_widget, chunks[1]);

            let messages_widget = MessagesWidget::new(&state.messages);
            frame.render_widget(messages_widget, chunks[2]);
        }

        // Render modal overlays based on mode (clone strings to avoid borrow conflicts)
        match self.mode.clone() {
            UiMode::Normal => {}
            UiMode::CharacterCreation(cc_state) => {
                self.render_character_creation(frame, cc_state);
            }
            UiMode::Inventory => self.render_inventory(frame),
            UiMode::ItemSelect { prompt, filter, .. } => {
                self.render_item_select(frame, &prompt, filter);
            }
            UiMode::DirectionSelect { prompt, .. } => {
                self.render_direction_select(frame, &prompt);
            }
            UiMode::ExtendedCommandInput { input } => {
                self.render_extended_command_input(frame, &input);
            }
            UiMode::Help => self.render_help(frame),
            UiMode::DeathScreen { cause } => {
                self.render_death_screen(frame, &cause);
            }
        }
    }

    /// Render inventory overlay
    fn render_inventory(&self, frame: &mut Frame) {
        let area = centered_rect(60, 80, frame.area());
        frame.render_widget(Clear, area);

        let inventory_widget =
            InventoryWidget::new(&self.game_loop.state().inventory, &self.theme);
        frame.render_widget(inventory_widget, area);
    }

    /// Render item selection overlay
    fn render_item_select(&self, frame: &mut Frame, prompt: &str, filter: Option<ObjectClass>) {
        let area = centered_rect(60, 80, frame.area());
        frame.render_widget(Clear, area);

        let state = self.game_loop.state();
        let items: Vec<_> = state
            .inventory
            .iter()
            .filter(|obj| filter.is_none_or(|f| obj.class == f))
            .collect();

        let block = Block::default()
            .title(prompt)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_action));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if items.is_empty() {
            let msg = Paragraph::new("You don't have anything suitable.")
                .style(Style::default().fg(self.theme.text_muted));
            frame.render_widget(msg, inner);
        } else {
            let list_items: Vec<ListItem> = items
                .iter()
                .map(|obj| {
                    let line = InventoryWidget::format_item(obj);
                    ListItem::new(line)
                })
                .collect();

            let list = List::new(list_items);
            frame.render_widget(list, inner);
        }
    }

    /// Render direction selection overlay
    fn render_direction_select(&self, frame: &mut Frame, prompt: &str) {
        let area = centered_rect(40, 30, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(prompt)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_action));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let help_text = if self.num_pad {
            "Use numpad or arrow keys\n. for self"
        } else {
            "y k u\n h . l\n b j n"
        };

        let paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(self.theme.text))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, inner);
    }

    /// Render extended command input overlay
    fn render_extended_command_input(&self, frame: &mut Frame, input: &str) {
        let area = centered_rect(50, 20, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title("# Extended Command")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_action));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let display = format!("#{}_", input);
        let paragraph = Paragraph::new(display)
            .style(Style::default().fg(self.theme.text));

        // Show matching commands below
        let matches = if input.is_empty() {
            "Type command name (e.g. pray, offer, sit, chat, jump, loot...)".to_string()
        } else {
            let lower = input.to_lowercase();
            let matching: Vec<&str> = [
                "pray", "offer", "sit", "chat", "pay", "dip", "jump", "ride", "wipe",
                "invoke", "turn", "monster", "enhance", "loot", "travel", "twoweapon",
                "untrap", "force", "kick", "open", "close", "fight", "discoveries",
                "history", "attributes", "conduct", "overview", "spells", "equipment",
                "vanquished", "redraw", "gold", "save", "quit", "search", "swap",
            ]
            .iter()
            .filter(|cmd| cmd.starts_with(&lower))
            .copied()
            .collect();
            if matching.is_empty() {
                format!("No matching command for '{}'", input)
            } else {
                matching.join(", ")
            }
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(inner);

        frame.render_widget(paragraph, chunks[0]);

        let matches_para = Paragraph::new(matches)
            .style(Style::default().fg(self.theme.text_dim))
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(matches_para, chunks[1]);
    }

    /// Render help overlay
    fn render_help(&self, frame: &mut Frame) {
        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        let help_text = r#"Movement: hjklyubn (vi keys) or arrow keys
         HJKLYUBN to run

Items:
  ,    Pickup         d  Drop          i  Inventory
  e    Eat            q  Quaff         r  Read
  a    Apply          z  Zap wand      t  Throw
  w    Wield          W  Wear armor    T  Take off
  P    Put on         R  Remove        $  Count gold

Actions:
  o    Open door      c  Close door    s  Search
  f    Fire           F  Fight         ^D Kick
  x    Swap weapon    X  Two-weapon    +  Enhance skill
  Z    Cast spell     _  Travel
  .    Rest/wait      <  Go up         >  Go down

Information:
  :    Look here      /  What is       ?  Help
  \    Discoveries    V  History       ^X Attributes
  ^P   Message log    ^R Redraw

Meta:
  #    Extended command (pray, offer, sit, chat, ...)
  S    Save game      Q  Quit

Press ESC or SPACE to close"#;

        let block = Block::default()
            .title("Help")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_accent));

        let paragraph = Paragraph::new(help_text)
            .block(block)
            .style(Style::default().fg(self.theme.text));

        frame.render_widget(paragraph, area);
    }

    /// Handle character creation input
    fn handle_character_creation_input(&mut self, key: crossterm::event::KeyEvent) {
        let current_state = match &self.mode {
            UiMode::CharacterCreation(s) => s.clone(),
            _ => return,
        };

        let new_state = match current_state {
            CharacterCreationState::EnterName { mut name } => match key.code {
                KeyCode::Enter => {
                    if name.is_empty() {
                        name = "Player".to_string();
                    }
                    CharacterCreationState::AskRandom { name, cursor: 0 }
                }
                KeyCode::Backspace => {
                    name.pop();
                    CharacterCreationState::EnterName { name }
                }
                KeyCode::Char(c) if name.len() < 32 => {
                    name.push(c);
                    CharacterCreationState::EnterName { name }
                }
                KeyCode::Esc => {
                    self.should_quit = true;
                    return;
                }
                _ => CharacterCreationState::EnterName { name },
            },
            CharacterCreationState::AskRandom { name, cursor } => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        // Random selection
                        let roles: Vec<Role> = Role::iter().collect();
                        let races: Vec<Race> = Race::iter().collect();
                        let genders: Vec<Gender> =
                            Gender::iter().filter(|g| *g != Gender::Neuter).collect();
                        let aligns: Vec<AlignmentType> = AlignmentType::iter().collect();

                        let role = roles[self.selection_cursor % roles.len()];
                        let race = races[(self.selection_cursor / 2) % races.len()];
                        let gender = genders[(self.selection_cursor / 3) % genders.len()];
                        let alignment = aligns[(self.selection_cursor / 5) % aligns.len()];

                        CharacterCreationState::Done {
                            name,
                            role,
                            race,
                            gender,
                            alignment,
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        CharacterCreationState::SelectRole { name, cursor: 0 }
                    }
                    KeyCode::Enter => {
                        if cursor == 0 {
                            // "Yes, pick for me" is selected
                            let roles: Vec<Role> = Role::iter().collect();
                            let races: Vec<Race> = Race::iter().collect();
                            let genders: Vec<Gender> =
                                Gender::iter().filter(|g| *g != Gender::Neuter).collect();
                            let aligns: Vec<AlignmentType> = AlignmentType::iter().collect();

                            let role = roles[self.selection_cursor % roles.len()];
                            let race = races[(self.selection_cursor / 2) % races.len()];
                            let gender =
                                genders[(self.selection_cursor / 3) % genders.len()];
                            let alignment =
                                aligns[(self.selection_cursor / 5) % aligns.len()];

                            CharacterCreationState::Done {
                                name,
                                role,
                                race,
                                gender,
                                alignment,
                            }
                        } else {
                            CharacterCreationState::SelectRole { name, cursor: 0 }
                        }
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.should_quit = true;
                        return;
                    }
                    KeyCode::Up | KeyCode::Char('k') => CharacterCreationState::AskRandom {
                        name,
                        cursor: if cursor == 0 { 1 } else { 0 },
                    },
                    KeyCode::Down | KeyCode::Char('j') => CharacterCreationState::AskRandom {
                        name,
                        cursor: if cursor == 0 { 1 } else { 0 },
                    },
                    _ => CharacterCreationState::AskRandom { name, cursor },
                }
            }
            CharacterCreationState::SelectRole { name, cursor } => {
                let roles: Vec<Role> = Role::iter().collect();
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        let new_cursor = if cursor == 0 {
                            roles.len() - 1
                        } else {
                            cursor - 1
                        };
                        CharacterCreationState::SelectRole {
                            name,
                            cursor: new_cursor,
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let new_cursor = (cursor + 1) % roles.len();
                        CharacterCreationState::SelectRole {
                            name,
                            cursor: new_cursor,
                        }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let role = roles[cursor];
                        CharacterCreationState::SelectRace {
                            name,
                            role,
                            cursor: 0,
                        }
                    }
                    KeyCode::Char('*') => {
                        let role = roles[self.selection_cursor % roles.len()];
                        self.selection_cursor = self.selection_cursor.wrapping_add(7);
                        CharacterCreationState::SelectRace {
                            name,
                            role,
                            cursor: 0,
                        }
                    }
                    KeyCode::Char(c) if c.is_ascii_lowercase() => {
                        let idx = (c as u8 - b'a') as usize;
                        if idx < roles.len() {
                            let role = roles[idx];
                            CharacterCreationState::SelectRace {
                                name,
                                role,
                                cursor: 0,
                            }
                        } else {
                            CharacterCreationState::SelectRole { name, cursor }
                        }
                    }
                    KeyCode::Esc => CharacterCreationState::AskRandom { name, cursor: 0 },
                    _ => CharacterCreationState::SelectRole { name, cursor },
                }
            }
            CharacterCreationState::SelectRace { name, role, cursor } => {
                // Filter races compatible with selected role
                let races: Vec<Race> = Race::iter()
                    .filter(|&r| nh_core::player::validrace(role, r))
                    .collect();
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        let new_cursor = if cursor == 0 {
                            races.len() - 1
                        } else {
                            cursor - 1
                        };
                        CharacterCreationState::SelectRace {
                            name,
                            role,
                            cursor: new_cursor,
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let new_cursor = (cursor + 1) % races.len();
                        CharacterCreationState::SelectRace {
                            name,
                            role,
                            cursor: new_cursor,
                        }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let race = races[cursor];
                        CharacterCreationState::SelectGender {
                            name,
                            role,
                            race,
                            cursor: 0,
                        }
                    }
                    KeyCode::Char('*') => {
                        let race = races[self.selection_cursor % races.len()];
                        self.selection_cursor = self.selection_cursor.wrapping_add(3);
                        CharacterCreationState::SelectGender {
                            name,
                            role,
                            race,
                            cursor: 0,
                        }
                    }
                    KeyCode::Char(c) if c.is_ascii_lowercase() => {
                        let idx = (c as u8 - b'a') as usize;
                        if idx < races.len() {
                            let race = races[idx];
                            CharacterCreationState::SelectGender {
                                name,
                                role,
                                race,
                                cursor: 0,
                            }
                        } else {
                            CharacterCreationState::SelectRace { name, role, cursor }
                        }
                    }
                    KeyCode::Esc => CharacterCreationState::SelectRole { name, cursor: 0 },
                    _ => CharacterCreationState::SelectRace { name, role, cursor },
                }
            }
            CharacterCreationState::SelectGender {
                name,
                role,
                race,
                cursor,
            } => {
                let genders = [Gender::Male, Gender::Female];
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        let new_cursor = if cursor == 0 { 1 } else { 0 };
                        CharacterCreationState::SelectGender {
                            name,
                            role,
                            race,
                            cursor: new_cursor,
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let new_cursor = if cursor == 0 { 1 } else { 0 };
                        CharacterCreationState::SelectGender {
                            name,
                            role,
                            race,
                            cursor: new_cursor,
                        }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let gender = genders[cursor];
                        CharacterCreationState::SelectAlignment {
                            name,
                            role,
                            race,
                            gender,
                            cursor: 0,
                        }
                    }
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        CharacterCreationState::SelectAlignment {
                            name,
                            role,
                            race,
                            gender: Gender::Male,
                            cursor: 0,
                        }
                    }
                    KeyCode::Char('f') | KeyCode::Char('F') => {
                        CharacterCreationState::SelectAlignment {
                            name,
                            role,
                            race,
                            gender: Gender::Female,
                            cursor: 0,
                        }
                    }
                    KeyCode::Char('*') => {
                        let gender = genders[self.selection_cursor % 2];
                        self.selection_cursor = self.selection_cursor.wrapping_add(1);
                        CharacterCreationState::SelectAlignment {
                            name,
                            role,
                            race,
                            gender,
                            cursor: 0,
                        }
                    }
                    KeyCode::Esc => CharacterCreationState::SelectRace {
                        name,
                        role,
                        cursor: 0,
                    },
                    _ => CharacterCreationState::SelectGender {
                        name,
                        role,
                        race,
                        cursor,
                    },
                }
            }
            CharacterCreationState::SelectAlignment {
                name,
                role,
                race,
                gender,
                cursor,
            } => {
                // Filter alignments compatible with role/race/gender
                let aligns: Vec<AlignmentType> = [
                    AlignmentType::Lawful,
                    AlignmentType::Neutral,
                    AlignmentType::Chaotic,
                ].into_iter()
                    .filter(|&a| nh_core::player::validalign(role, race, gender, a))
                    .collect();
                let aligns_len = aligns.len();
                // If only one alignment is valid, skip selection
                if aligns_len == 1 {
                    self.mode = UiMode::CharacterCreation(CharacterCreationState::Done {
                        name,
                        role,
                        race,
                        gender,
                        alignment: aligns[0],
                    });
                    return;
                }
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        let new_cursor = if cursor == 0 { aligns_len - 1 } else { cursor - 1 };
                        CharacterCreationState::SelectAlignment {
                            name,
                            role,
                            race,
                            gender,
                            cursor: new_cursor,
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let new_cursor = (cursor + 1) % aligns_len;
                        CharacterCreationState::SelectAlignment {
                            name,
                            role,
                            race,
                            gender,
                            cursor: new_cursor,
                        }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let alignment = aligns[cursor.min(aligns_len - 1)];
                        CharacterCreationState::Done {
                            name,
                            role,
                            race,
                            gender,
                            alignment,
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Char('L') if aligns.contains(&AlignmentType::Lawful) => {
                        CharacterCreationState::Done {
                            name, role, race, gender,
                            alignment: AlignmentType::Lawful,
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') if aligns.contains(&AlignmentType::Neutral) => {
                        CharacterCreationState::Done {
                            name, role, race, gender,
                            alignment: AlignmentType::Neutral,
                        }
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') if aligns.contains(&AlignmentType::Chaotic) => {
                        CharacterCreationState::Done {
                            name, role, race, gender,
                            alignment: AlignmentType::Chaotic,
                        }
                    }
                    KeyCode::Char('*') => {
                        let alignment = aligns[self.selection_cursor % aligns_len];
                        CharacterCreationState::Done {
                            name,
                            role,
                            race,
                            gender,
                            alignment,
                        }
                    }
                    KeyCode::Esc => CharacterCreationState::SelectGender {
                        name,
                        role,
                        race,
                        cursor: 0,
                    },
                    _ => CharacterCreationState::SelectAlignment {
                        name,
                        role,
                        race,
                        gender,
                        cursor,
                    },
                }
            }
            CharacterCreationState::Done { .. } => {
                // Already done, transition to normal mode
                self.mode = UiMode::Normal;
                return;
            }
        };

        self.mode = UiMode::CharacterCreation(new_state);
    }

    /// Render character creation modal
    fn render_character_creation(&self, frame: &mut Frame, state: CharacterCreationState) {
        let area = centered_rect(50, 60, frame.area());
        frame.render_widget(Clear, area);

        // Build items as owned Strings to avoid lifetime issues
        let (title, items, cursor, footer): (&str, Vec<(String, String)>, usize, &str) =
            match &state {
                CharacterCreationState::EnterName { name } => {
                    let display = if name.is_empty() {
                        "_".to_string()
                    } else {
                        format!("{}_", name)
                    };
                    let items = vec![("".to_string(), display)];
                    (
                        "Who are you?",
                        items,
                        0,
                        "Type your name, Enter to confirm, Esc to quit",
                    )
                }
                CharacterCreationState::AskRandom { cursor, .. } => {
                    let items = vec![
                        ("y".to_string(), "Yes, pick for me".to_string()),
                        ("n".to_string(), "No, let me choose".to_string()),
                    ];
                    (
                        "Shall I pick a character for you?",
                        items,
                        *cursor,
                        "Press y/n or q to quit",
                    )
                }
                CharacterCreationState::SelectRole { cursor, .. } => {
                    let roles: Vec<Role> = Role::iter().collect();
                    let items: Vec<(String, String)> = roles
                        .iter()
                        .enumerate()
                        .map(|(i, r)| {
                            let key = ((b'a' + i as u8) as char).to_string();
                            (key, r.to_string())
                        })
                        .collect();
                    (
                        "Pick a role:",
                        items,
                        *cursor,
                        "jk/arrows to move, Enter to select, * random, Esc back",
                    )
                }
                CharacterCreationState::SelectRace { role, cursor, .. } => {
                    // Show only races compatible with the selected role
                    let races: Vec<Race> = Race::iter()
                        .filter(|&r| nh_core::player::validrace(*role, r))
                        .collect();
                    let items: Vec<(String, String)> = races
                        .iter()
                        .enumerate()
                        .map(|(i, r)| {
                            let key = ((b'a' + i as u8) as char).to_string();
                            (key, r.to_string())
                        })
                        .collect();
                    (
                        "Pick a race:",
                        items,
                        *cursor,
                        "jk/arrows to move, Enter to select, * random, Esc back",
                    )
                }
                CharacterCreationState::SelectGender { cursor, .. } => {
                    let items = vec![
                        ("m".to_string(), "Male".to_string()),
                        ("f".to_string(), "Female".to_string()),
                    ];
                    (
                        "Pick a gender:",
                        items,
                        *cursor,
                        "jk/arrows to move, Enter to select, * random, Esc back",
                    )
                }
                CharacterCreationState::SelectAlignment { role, race, gender, cursor, .. } => {
                    // Show only alignments compatible with role/race/gender
                    let valid_aligns: Vec<AlignmentType> = [
                        AlignmentType::Lawful,
                        AlignmentType::Neutral,
                        AlignmentType::Chaotic,
                    ].into_iter()
                        .filter(|&a| nh_core::player::validalign(*role, *race, *gender, a))
                        .collect();
                    let items: Vec<(String, String)> = valid_aligns.iter().map(|a| {
                        let key = match a {
                            AlignmentType::Lawful => "l",
                            AlignmentType::Neutral => "n",
                            AlignmentType::Chaotic => "c",
                        };
                        (key.to_string(), format!("{:?}", a))
                    }).collect();
                    (
                        "Pick an alignment:",
                        items,
                        *cursor,
                        "jk/arrows to move, Enter to select, * random, Esc back",
                    )
                }
                CharacterCreationState::Done { .. } => {
                    let items: Vec<(String, String)> =
                        vec![("".to_string(), "Press any key to start".to_string())];
                    ("Character Created!", items, 0, "Your adventure begins!")
                }
            };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_accent));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Render items as a list with cursor highlight
        let list_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(i, (key, label))| {
                let style = if i == cursor {
                    Style::default()
                        .fg(self.theme.cursor_fg)
                        .bg(self.theme.cursor_bg)
                } else {
                    Style::default().fg(self.theme.text)
                };
                let prefix = if i == cursor { "> " } else { "  " };
                let text = if key.is_empty() {
                    format!("{}{}", prefix, label)
                } else {
                    format!("{}{} - {}", prefix, key, label)
                };
                ListItem::new(Line::from(Span::styled(text, style)))
            })
            .collect();

        let list = List::new(list_items);

        // Split inner area for list and footer
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(inner);

        frame.render_widget(list, inner_chunks[0]);

        let footer_para = Paragraph::new(footer)
            .style(Style::default().fg(self.theme.text_dim))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(footer_para, inner_chunks[1]);
    }

    /// Start character creation mode
    pub fn start_character_creation(&mut self) {
        self.mode = UiMode::CharacterCreation(CharacterCreationState::EnterName {
            name: String::new(),
        });
    }

    /// Start character creation with a pre-set name (from CLI)
    pub fn start_character_creation_with_name(&mut self, name: String) {
        self.mode =
            UiMode::CharacterCreation(CharacterCreationState::AskRandom { name, cursor: 0 });
    }

    /// Check if character creation is complete and get the choices
    pub fn get_character_choices(&self) -> Option<CharacterChoices> {
        if let UiMode::CharacterCreation(CharacterCreationState::Done {
            name,
            role,
            race,
            gender,
            alignment,
        }) = &self.mode
        {
            Some(CharacterChoices {
                name: name.clone(),
                role: *role,
                race: *race,
                gender: *gender,
                alignment: *alignment,
            })
        } else {
            None
        }
    }

    /// Check if in character creation mode
    pub fn is_creating_character(&self) -> bool {
        matches!(self.mode, UiMode::CharacterCreation(_))
    }

    /// Finish character creation and switch to normal mode
    pub fn finish_character_creation(&mut self) {
        self.mode = UiMode::Normal;
    }

    /// Handle death screen input
    fn handle_death_screen_input(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char(' ') | KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    /// Render the death screen modal with player statistics
    fn render_death_screen(&self, frame: &mut Frame, cause: &str) {
        use nh_core::player::Attribute;
        use ratatui::style::Stylize;

        let area = centered_rect(70, 85, frame.area());
        frame.render_widget(Clear, area);

        let state = self.game_loop.state();
        let player = &state.player;

        let mut lines: Vec<Line> = Vec::new();

        // Title
        lines.push(Line::from(vec![Span::styled(
            "  R.I.P.  ",
            Style::default().fg(self.theme.bad).bold(),
        )]));
        lines.push(Line::from(""));

        // Player identity
        lines.push(Line::from(vec![
            Span::styled(&player.name, Style::default().fg(self.theme.text).bold()),
            Span::raw(" the "),
            Span::styled(
                format!("{:?}", player.role),
                Style::default().fg(self.theme.header),
            ),
        ]));
        lines.push(Line::from(""));

        // Cause of death
        lines.push(Line::from(vec![
            Span::raw("Killed by: "),
            Span::styled(cause, Style::default().fg(self.theme.bad)),
        ]));
        lines.push(Line::from(""));

        // Basic stats
        lines.push(Line::from(Span::styled(
            "── Statistics ──",
            Style::default().fg(self.theme.accent),
        )));
        lines.push(Line::from(format!(
            "  Race: {:?}    Gender: {:?}    Alignment: {:?}",
            player.race, player.gender, player.alignment.typ
        )));
        lines.push(Line::from(format!(
            "  Level: {}    Experience: {}",
            player.exp_level, player.exp
        )));
        lines.push(Line::from(format!(
            "  HP: {}/{}    Energy: {}/{}",
            player.hp.max(0),
            player.hp_max,
            player.energy,
            player.energy_max
        )));
        lines.push(Line::from(format!(
            "  Gold: {}    Turns: {}",
            player.gold, player.turns_played
        )));
        lines.push(Line::from(format!(
            "  Dungeon Level: {}",
            player.level.depth()
        )));
        lines.push(Line::from(""));

        // Attributes
        lines.push(Line::from(Span::styled(
            "── Attributes ──",
            Style::default().fg(self.theme.accent),
        )));
        lines.push(Line::from(format!(
            "  Str: {:2}  Dex: {:2}  Con: {:2}  Int: {:2}  Wis: {:2}  Cha: {:2}",
            player.attr_current.get(Attribute::Strength),
            player.attr_current.get(Attribute::Dexterity),
            player.attr_current.get(Attribute::Constitution),
            player.attr_current.get(Attribute::Intelligence),
            player.attr_current.get(Attribute::Wisdom),
            player.attr_current.get(Attribute::Charisma)
        )));
        lines.push(Line::from(""));

        // Conducts
        lines.push(Line::from(Span::styled(
            "── Conducts ──",
            Style::default().fg(self.theme.accent),
        )));
        let mut conducts_maintained: Vec<&str> = Vec::new();
        let mut conducts_broken: Vec<String> = Vec::new();

        if player.conduct.is_foodless() {
            conducts_maintained.push("foodless");
        } else if player.conduct.food > 0 {
            conducts_broken.push(format!("ate {} times", player.conduct.food));
        }

        if player.conduct.is_vegan() {
            conducts_maintained.push("vegan");
        } else if player.conduct.is_vegetarian() {
            conducts_maintained.push("vegetarian");
        } else if player.conduct.unvegetarian > 0 {
            conducts_broken.push(format!("ate meat {} times", player.conduct.unvegetarian));
        }

        if player.conduct.is_atheist() {
            conducts_maintained.push("atheist");
        } else if player.conduct.gnostic > 0 {
            conducts_broken.push(format!("prayed {} times", player.conduct.gnostic));
        }

        if player.conduct.is_weaponless() {
            conducts_maintained.push("weaponless");
        } else if player.conduct.weaphit > 0 {
            conducts_broken.push(format!("hit with weapon {} times", player.conduct.weaphit));
        }

        if player.conduct.is_pacifist() {
            conducts_maintained.push("pacifist");
        } else if player.conduct.killer > 0 {
            conducts_broken.push(format!("killed {} creatures", player.conduct.killer));
        }

        if player.conduct.is_illiterate() {
            conducts_maintained.push("illiterate");
        } else if player.conduct.literate > 0 {
            conducts_broken.push(format!("read {} times", player.conduct.literate));
        }

        if player.conduct.is_wishless() {
            conducts_maintained.push("wishless");
        } else if player.conduct.wishes > 0 {
            conducts_broken.push(format!("made {} wishes", player.conduct.wishes));
        }

        if player.conduct.is_genocideless() {
            conducts_maintained.push("genocideless");
        } else if player.conduct.genocides > 0 {
            conducts_broken.push(format!("genocided {} times", player.conduct.genocides));
        }

        if !conducts_maintained.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("  Maintained: "),
                Span::styled(
                    conducts_maintained.join(", "),
                    Style::default().fg(self.theme.good),
                ),
            ]));
        }
        if !conducts_broken.is_empty() {
            for broken in &conducts_broken {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(broken, Style::default().fg(self.theme.text_dim)),
                ]));
            }
        }
        if conducts_maintained.is_empty() && conducts_broken.is_empty() {
            lines.push(Line::from("  No special conducts tracked"));
        }

        lines.push(Line::from(""));

        // Inventory summary
        lines.push(Line::from(Span::styled(
            "── Inventory ──",
            Style::default().fg(self.theme.accent),
        )));
        let inv_count = state.inventory.len();
        if inv_count == 0 {
            lines.push(Line::from("  No items"));
        } else {
            lines.push(Line::from(format!(
                "  {} item{}",
                inv_count,
                if inv_count == 1 { "" } else { "s" }
            )));
            // Show first few items
            for (i, item) in state.inventory.iter().take(5).enumerate() {
                let item_name = item
                    .name
                    .as_deref()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{:?}", item.class));
                lines.push(Line::from(format!(
                    "    {} - {}",
                    (b'a' + i as u8) as char,
                    item_name
                )));
            }
            if inv_count > 5 {
                lines.push(Line::from(format!("    ... and {} more", inv_count - 5)));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Press SPACE or ENTER to exit",
            Style::default().fg(self.theme.text_dim),
        )));

        let block = Block::default()
            .title(" Game Over ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_danger));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_death_screen_mode_transition() {
        // Test that UiMode::DeathScreen can be created
        let mode = UiMode::DeathScreen {
            cause: "killed by a goblin".to_string(),
        };
        assert!(matches!(mode, UiMode::DeathScreen { .. }));
    }

    #[test]
    fn test_death_screen_cause_stored() {
        let cause = "killed by a grid bug";
        let mode = UiMode::DeathScreen {
            cause: cause.to_string(),
        };
        if let UiMode::DeathScreen {
            cause: stored_cause,
        } = mode
        {
            assert_eq!(stored_cause, cause);
        } else {
            panic!("Expected DeathScreen mode");
        }
    }
}
