//! Application state and main UI controller

use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

use nh_core::action::{Command, Direction as GameDirection};
use nh_core::object::ObjectClass;
use nh_core::{GameLoop, GameLoopResult, GameState};

use crate::input::key_to_command;
use crate::widgets::{InventoryWidget, MapWidget, MessagesWidget, StatusWidget};

/// UI mode - what the app is currently displaying/waiting for
#[derive(Debug, Clone)]
pub enum UiMode {
    /// Normal gameplay
    Normal,
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
