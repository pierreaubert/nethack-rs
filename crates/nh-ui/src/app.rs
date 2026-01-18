//! Application state and main UI controller

use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

use nh_core::action::{Command, Direction as GameDirection};
use nh_core::object::ObjectClass;
use nh_core::player::{AlignmentType, Gender, Race, Role};
use nh_core::{GameLoop, GameLoopResult, GameState};
use strum::IntoEnumIterator;

use crate::input::key_to_command;
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
    /// Showing help
    Help,
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
    SelectRace { name: String, role: Role, cursor: usize },
    /// Selecting gender
    SelectGender { name: String, role: Role, race: Race, cursor: usize },
    /// Selecting alignment
    SelectAlignment { name: String, role: Role, race: Race, gender: Gender, cursor: usize },
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
}

impl App {
    /// Create a new application with a new game
    pub fn new(state: GameState) -> Self {
        Self {
            game_loop: GameLoop::new(state),
            should_quit: false,
            num_pad: false,
            mode: UiMode::Normal,
            selection_cursor: 0,
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
                UiMode::Help => {
                    self.handle_help_input(key);
                    None
                }
            }
        } else {
            None
        }
    }

    /// Handle input in normal gameplay mode
    fn handle_normal_input(&mut self, key: crossterm::event::KeyEvent) -> Option<Command> {
        match key.code {
            // Commands that need item selection
            KeyCode::Char('d') => {
                self.enter_item_select("Drop what?", PendingAction::Drop, None);
                None
            }
            KeyCode::Char('e') => {
                self.enter_item_select("Eat what?", PendingAction::Eat, Some(ObjectClass::Food));
                None
            }
            KeyCode::Char('a') => {
                self.enter_item_select("Apply what?", PendingAction::Apply, Some(ObjectClass::Tool));
                None
            }
            KeyCode::Char('W') => {
                self.enter_item_select("Wear what?", PendingAction::Wear, Some(ObjectClass::Armor));
                None
            }
            KeyCode::Char('T') => {
                self.enter_item_select("Take off what?", PendingAction::TakeOff, Some(ObjectClass::Armor));
                None
            }
            KeyCode::Char('w') => {
                self.enter_item_select("Wield what?", PendingAction::Wield, Some(ObjectClass::Weapon));
                None
            }
            KeyCode::Char('P') => {
                self.enter_item_select("Put on what?", PendingAction::PutOn, None); // Rings or amulets
                None
            }
            KeyCode::Char('R') => {
                self.enter_item_select("Remove what?", PendingAction::Remove, None);
                None
            }
            KeyCode::Char('q') => {
                self.enter_item_select("Quaff what?", PendingAction::Quaff, Some(ObjectClass::Potion));
                None
            }
            KeyCode::Char('r') => {
                self.enter_item_select("Read what?", PendingAction::Read, Some(ObjectClass::Scroll));
                None
            }
            KeyCode::Char('z') => {
                self.enter_item_select("Zap what?", PendingAction::Zap, Some(ObjectClass::Wand));
                None
            }

            // Commands that need direction selection
            KeyCode::Char('o') => {
                self.enter_direction_select("Open in which direction?", PendingAction::Open);
                None
            }
            KeyCode::Char('c') => {
                self.enter_direction_select("Close in which direction?", PendingAction::Close);
                None
            }
            KeyCode::Char('k') if self.num_pad => {
                // 'k' is movement in vi mode, kick in numpad mode
                self.enter_direction_select("Kick in which direction?", PendingAction::Kick);
                None
            }
            KeyCode::Char('D') => {
                // Shift-D for kick in vi mode
                self.enter_direction_select("Kick in which direction?", PendingAction::Kick);
                None
            }

            // Inventory display
            KeyCode::Char('i') => {
                self.mode = UiMode::Inventory;
                None
            }

            // Help
            KeyCode::Char('?') => {
                self.mode = UiMode::Help;
                None
            }

            // All other commands go through normal input handling
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
                    Some(self.action_with_item(action, c))
                } else {
                    self.game_loop.state_mut().message("You don't have that item.");
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
    fn enter_item_select(&mut self, prompt: &str, action: PendingAction, filter: Option<ObjectClass>) {
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

    /// Create a command for an action with an item
    fn action_with_item(&self, action: PendingAction, letter: char) -> Command {
        match action {
            PendingAction::Drop => Command::ExtendedCommand(format!("drop {}", letter)),
            PendingAction::Eat => Command::ExtendedCommand(format!("eat {}", letter)),
            PendingAction::Apply => Command::ExtendedCommand(format!("apply {}", letter)),
            PendingAction::Wear => Command::ExtendedCommand(format!("wear {}", letter)),
            PendingAction::TakeOff => Command::ExtendedCommand(format!("takeoff {}", letter)),
            PendingAction::Wield => Command::ExtendedCommand(format!("wield {}", letter)),
            PendingAction::PutOn => Command::ExtendedCommand(format!("puton {}", letter)),
            PendingAction::Remove => Command::ExtendedCommand(format!("remove {}", letter)),
            PendingAction::Quaff => Command::ExtendedCommand(format!("quaff {}", letter)),
            PendingAction::Read => Command::ExtendedCommand(format!("read {}", letter)),
            PendingAction::Zap => Command::ExtendedCommand(format!("zap {}", letter)),
            _ => Command::ExtendedCommand("noop".to_string()),
        }
    }

    /// Create a command for an action with a direction
    fn action_with_direction(&self, action: PendingAction, dir: GameDirection) -> Command {
        let dir_str = match dir {
            GameDirection::North => "n",
            GameDirection::South => "s",
            GameDirection::East => "e",
            GameDirection::West => "w",
            GameDirection::NorthEast => "ne",
            GameDirection::NorthWest => "nw",
            GameDirection::SouthEast => "se",
            GameDirection::SouthWest => "sw",
            GameDirection::Self_ => ".",
            _ => ".",
        };
        match action {
            PendingAction::Open => Command::ExtendedCommand(format!("open {}", dir_str)),
            PendingAction::Close => Command::ExtendedCommand(format!("close {}", dir_str)),
            PendingAction::Kick => Command::ExtendedCommand(format!("kick {}", dir_str)),
            _ => Command::ExtendedCommand("noop".to_string()),
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
            GameLoopResult::PlayerDied(_) | GameLoopResult::PlayerQuit => {
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
    pub fn render(&self, frame: &mut Frame) {
        let state = self.state();

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
        let map_widget = MapWidget::new(&state.current_level, &state.player);
        frame.render_widget(map_widget, chunks[0]);

        // Render status
        let status_widget = StatusWidget::new(&state.player);
        frame.render_widget(status_widget, chunks[1]);

        // Render messages
        let messages_widget = MessagesWidget::new(&state.messages);
        frame.render_widget(messages_widget, chunks[2]);

        // Render modal overlays based on mode
        match &self.mode {
            UiMode::Normal => {}
            UiMode::CharacterCreation(state) => {
                self.render_character_creation(frame, state.clone());
            }
            UiMode::Inventory => self.render_inventory(frame),
            UiMode::ItemSelect { prompt, filter, .. } => {
                self.render_item_select(frame, prompt, *filter);
            }
            UiMode::DirectionSelect { prompt, .. } => {
                self.render_direction_select(frame, prompt);
            }
            UiMode::Help => self.render_help(frame),
        }
    }

    /// Render inventory overlay
    fn render_inventory(&self, frame: &mut Frame) {
        let area = centered_rect(60, 80, frame.area());
        frame.render_widget(Clear, area);

        let inventory_widget = InventoryWidget::new(&self.game_loop.state().inventory);
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
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if items.is_empty() {
            let msg = Paragraph::new("You don't have anything suitable.")
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(msg, inner);
        } else {
            let list_items: Vec<ListItem> = items
                .iter()
                .map(|obj| {
                    let name = obj.name.as_deref().unwrap_or("item");
                    let text = format!("{} - {}", obj.inv_letter, name);
                    ListItem::new(Line::from(Span::styled(text, Style::default().fg(Color::White))))
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
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let help_text = if self.num_pad {
            "Use numpad or arrow keys\n. for self"
        } else {
            "y k u\n h . l\n b j n"
        };

        let paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::White))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, inner);
    }

    /// Render help overlay
    fn render_help(&self, frame: &mut Frame) {
        let area = centered_rect(70, 80, frame.area());
        frame.render_widget(Clear, area);

        let help_text = r#"Movement: hjklyubn (vi keys) or arrow keys
Commands:
  , g  Pickup items
  d    Drop item
  e    Eat food
  q    Quaff potion
  r    Read scroll
  z    Zap wand
  a    Apply tool
  w    Wield weapon
  W    Wear armor
  T    Take off armor
  P    Put on ring/amulet
  R    Remove ring/amulet
  o    Open door
  c    Close door
  D    Kick
  s    Search
  .    Rest
  <    Go up stairs
  >    Go down stairs
  i    Inventory
  S    Save game
  Q    Quit

Press ESC or SPACE to close"#;

        let block = Block::default()
            .title("Help")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let paragraph = Paragraph::new(help_text)
            .block(block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }

    /// Handle character creation input
    fn handle_character_creation_input(&mut self, key: crossterm::event::KeyEvent) {
        let current_state = match &self.mode {
            UiMode::CharacterCreation(s) => s.clone(),
            _ => return,
        };

        let new_state = match current_state {
            CharacterCreationState::EnterName { mut name } => {
                match key.code {
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
                    _ => CharacterCreationState::EnterName { name }
                }
            }
            CharacterCreationState::AskRandom { name, cursor } => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        // Random selection
                        let roles: Vec<Role> = Role::iter().collect();
                        let races: Vec<Race> = Race::iter().collect();
                        let genders: Vec<Gender> = Gender::iter().filter(|g| *g != Gender::Neuter).collect();
                        let aligns: Vec<AlignmentType> = AlignmentType::iter().collect();
                        
                        let role = roles[self.selection_cursor % roles.len()];
                        let race = races[(self.selection_cursor / 2) % races.len()];
                        let gender = genders[(self.selection_cursor / 3) % genders.len()];
                        let alignment = aligns[(self.selection_cursor / 5) % aligns.len()];
                        
                        CharacterCreationState::Done { name, role, race, gender, alignment }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Enter => {
                        CharacterCreationState::SelectRole { name, cursor: 0 }
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.should_quit = true;
                        return;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        CharacterCreationState::AskRandom { name, cursor: if cursor == 0 { 1 } else { 0 } }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        CharacterCreationState::AskRandom { name, cursor: if cursor == 0 { 1 } else { 0 } }
                    }
                    _ => CharacterCreationState::AskRandom { name, cursor }
                }
            }
            CharacterCreationState::SelectRole { name, cursor } => {
                let roles: Vec<Role> = Role::iter().collect();
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        let new_cursor = if cursor == 0 { roles.len() - 1 } else { cursor - 1 };
                        CharacterCreationState::SelectRole { name, cursor: new_cursor }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let new_cursor = (cursor + 1) % roles.len();
                        CharacterCreationState::SelectRole { name, cursor: new_cursor }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let role = roles[cursor];
                        CharacterCreationState::SelectRace { name, role, cursor: 0 }
                    }
                    KeyCode::Char('*') => {
                        let role = roles[self.selection_cursor % roles.len()];
                        self.selection_cursor = self.selection_cursor.wrapping_add(7);
                        CharacterCreationState::SelectRace { name, role, cursor: 0 }
                    }
                    KeyCode::Char(c) if c.is_ascii_lowercase() => {
                        let idx = (c as u8 - b'a') as usize;
                        if idx < roles.len() {
                            let role = roles[idx];
                            CharacterCreationState::SelectRace { name, role, cursor: 0 }
                        } else {
                            CharacterCreationState::SelectRole { name, cursor }
                        }
                    }
                    KeyCode::Esc => CharacterCreationState::AskRandom { name, cursor: 0 },
                    _ => CharacterCreationState::SelectRole { name, cursor }
                }
            }
            CharacterCreationState::SelectRace { name, role, cursor } => {
                let races: Vec<Race> = Race::iter().collect();
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        let new_cursor = if cursor == 0 { races.len() - 1 } else { cursor - 1 };
                        CharacterCreationState::SelectRace { name, role, cursor: new_cursor }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let new_cursor = (cursor + 1) % races.len();
                        CharacterCreationState::SelectRace { name, role, cursor: new_cursor }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let race = races[cursor];
                        CharacterCreationState::SelectGender { name, role, race, cursor: 0 }
                    }
                    KeyCode::Char('*') => {
                        let race = races[self.selection_cursor % races.len()];
                        self.selection_cursor = self.selection_cursor.wrapping_add(3);
                        CharacterCreationState::SelectGender { name, role, race, cursor: 0 }
                    }
                    KeyCode::Char(c) if c.is_ascii_lowercase() => {
                        let idx = (c as u8 - b'a') as usize;
                        if idx < races.len() {
                            let race = races[idx];
                            CharacterCreationState::SelectGender { name, role, race, cursor: 0 }
                        } else {
                            CharacterCreationState::SelectRace { name, role, cursor }
                        }
                    }
                    KeyCode::Esc => CharacterCreationState::SelectRole { name, cursor: 0 },
                    _ => CharacterCreationState::SelectRace { name, role, cursor }
                }
            }
            CharacterCreationState::SelectGender { name, role, race, cursor } => {
                let genders = [Gender::Male, Gender::Female];
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        let new_cursor = if cursor == 0 { 1 } else { 0 };
                        CharacterCreationState::SelectGender { name, role, race, cursor: new_cursor }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let new_cursor = if cursor == 0 { 1 } else { 0 };
                        CharacterCreationState::SelectGender { name, role, race, cursor: new_cursor }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let gender = genders[cursor];
                        CharacterCreationState::SelectAlignment { name, role, race, gender, cursor: 0 }
                    }
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        CharacterCreationState::SelectAlignment { name, role, race, gender: Gender::Male, cursor: 0 }
                    }
                    KeyCode::Char('f') | KeyCode::Char('F') => {
                        CharacterCreationState::SelectAlignment { name, role, race, gender: Gender::Female, cursor: 0 }
                    }
                    KeyCode::Char('*') => {
                        let gender = genders[self.selection_cursor % 2];
                        self.selection_cursor = self.selection_cursor.wrapping_add(1);
                        CharacterCreationState::SelectAlignment { name, role, race, gender, cursor: 0 }
                    }
                    KeyCode::Esc => CharacterCreationState::SelectRace { name, role, cursor: 0 },
                    _ => CharacterCreationState::SelectGender { name, role, race, cursor }
                }
            }
            CharacterCreationState::SelectAlignment { name, role, race, gender, cursor } => {
                let aligns = [AlignmentType::Lawful, AlignmentType::Neutral, AlignmentType::Chaotic];
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        let new_cursor = if cursor == 0 { 2 } else { cursor - 1 };
                        CharacterCreationState::SelectAlignment { name, role, race, gender, cursor: new_cursor }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let new_cursor = (cursor + 1) % 3;
                        CharacterCreationState::SelectAlignment { name, role, race, gender, cursor: new_cursor }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let alignment = aligns[cursor];
                        CharacterCreationState::Done { name, role, race, gender, alignment }
                    }
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        CharacterCreationState::Done { name, role, race, gender, alignment: AlignmentType::Lawful }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        CharacterCreationState::Done { name, role, race, gender, alignment: AlignmentType::Neutral }
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        CharacterCreationState::Done { name, role, race, gender, alignment: AlignmentType::Chaotic }
                    }
                    KeyCode::Char('*') => {
                        let alignment = aligns[self.selection_cursor % 3];
                        CharacterCreationState::Done { name, role, race, gender, alignment }
                    }
                    KeyCode::Esc => CharacterCreationState::SelectGender { name, role, race, cursor: 0 },
                    _ => CharacterCreationState::SelectAlignment { name, role, race, gender, cursor }
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
        let (title, items, cursor, footer): (&str, Vec<(String, String)>, usize, &str) = match &state {
            CharacterCreationState::EnterName { name } => {
                let display = if name.is_empty() {
                    "_".to_string()
                } else {
                    format!("{}_", name)
                };
                let items = vec![("".to_string(), display)];
                ("Who are you?", items, 0, "Type your name, Enter to confirm, Esc to quit")
            }
            CharacterCreationState::AskRandom { cursor, .. } => {
                let items = vec![
                    ("y".to_string(), "Yes, pick for me".to_string()),
                    ("n".to_string(), "No, let me choose".to_string()),
                ];
                ("Shall I pick a character for you?", items, *cursor, "Press y/n or q to quit")
            }
            CharacterCreationState::SelectRole { cursor, .. } => {
                let roles: Vec<Role> = Role::iter().collect();
                let items: Vec<(String, String)> = roles.iter().enumerate()
                    .map(|(i, r)| {
                        let key = ((b'a' + i as u8) as char).to_string();
                        (key, r.to_string())
                    })
                    .collect();
                ("Pick a role:", items, *cursor, "jk/arrows to move, Enter to select, * random, Esc back")
            }
            CharacterCreationState::SelectRace { cursor, .. } => {
                let races: Vec<Race> = Race::iter().collect();
                let items: Vec<(String, String)> = races.iter().enumerate()
                    .map(|(i, r)| {
                        let key = ((b'a' + i as u8) as char).to_string();
                        (key, r.to_string())
                    })
                    .collect();
                ("Pick a race:", items, *cursor, "jk/arrows to move, Enter to select, * random, Esc back")
            }
            CharacterCreationState::SelectGender { cursor, .. } => {
                let items = vec![
                    ("m".to_string(), "Male".to_string()),
                    ("f".to_string(), "Female".to_string()),
                ];
                ("Pick a gender:", items, *cursor, "jk/arrows to move, Enter to select, * random, Esc back")
            }
            CharacterCreationState::SelectAlignment { cursor, .. } => {
                let items = vec![
                    ("l".to_string(), "Lawful".to_string()),
                    ("n".to_string(), "Neutral".to_string()),
                    ("c".to_string(), "Chaotic".to_string()),
                ];
                ("Pick an alignment:", items, *cursor, "jk/arrows to move, Enter to select, * random, Esc back")
            }
            CharacterCreationState::Done { .. } => {
                let items: Vec<(String, String)> = vec![("".to_string(), "Press any key to start".to_string())];
                ("Character Created!", items, 0, "Your adventure begins!")
            }
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Render items as a list with cursor highlight
        let list_items: Vec<ListItem> = items.iter().enumerate()
            .map(|(i, (key, label))| {
                let style = if i == cursor {
                    Style::default().fg(Color::Yellow).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
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
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(footer_para, inner_chunks[1]);
    }

    /// Start character creation mode
    pub fn start_character_creation(&mut self) {
        self.mode = UiMode::CharacterCreation(CharacterCreationState::EnterName { name: String::new() });
    }

    /// Start character creation with a pre-set name (from CLI)
    pub fn start_character_creation_with_name(&mut self, name: String) {
        self.mode = UiMode::CharacterCreation(CharacterCreationState::AskRandom { name, cursor: 0 });
    }

    /// Check if character creation is complete and get the choices
    pub fn get_character_choices(&self) -> Option<CharacterChoices> {
        if let UiMode::CharacterCreation(CharacterCreationState::Done { name, role, race, gender, alignment }) = &self.mode {
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
