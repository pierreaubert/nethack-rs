//! Help screen widget

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

/// Help screen widget showing available commands
pub struct HelpWidget {
    page: usize,
}

impl HelpWidget {
    pub fn new() -> Self {
        Self { page: 0 }
    }

    pub fn page(mut self, page: usize) -> Self {
        self.page = page;
        self
    }

    pub fn next_page(&mut self) {
        self.page = (self.page + 1) % Self::page_count();
    }

    pub fn prev_page(&mut self) {
        if self.page == 0 {
            self.page = Self::page_count() - 1;
        } else {
            self.page -= 1;
        }
    }

    pub const fn page_count() -> usize {
        4
    }

    fn movement_help() -> &'static str {
        r#"Movement Commands
─────────────────
  y  k  u       7  8  9
   \ | /         \ | /
  h -.- l       4 -5- 6
   / | \         / | \
  b  j  n       1  2  3

  Vi keys (hjklyubn) or numpad

  <  Go up stairs
  >  Go down stairs
  .  Rest (wait one turn)
  s  Search adjacent squares

Running (capital letters):
  H J K L Y U B N - Run in direction
  
Travel:
  _  Travel to location (click)"#
    }

    fn action_help() -> &'static str {
        r#"Action Commands
───────────────
  ,  Pickup item(s)
  d  Drop item(s)
  e  Eat something
  q  Quaff (drink) a potion
  r  Read a scroll or spellbook
  z  Zap a wand
  a  Apply/use a tool
  t  Throw an item
  f  Fire from quiver

  o  Open a door
  c  Close a door
  k  Kick something

Equipment:
  w  Wield a weapon
  W  Wear armor
  T  Take off armor
  P  Put on accessory
  R  Remove accessory
  Q  Select ammunition"#
    }

    fn info_help() -> &'static str {
        r#"Information & Meta
──────────────────
  i  Show inventory
  I  Inventory of specific type
  :  Look at what's here
  /  What is this symbol?
  ;  What is at location?
  \  Show discovered items
  @  Toggle autopickup
  #  Extended command

  ?  This help screen
  S  Save and quit
  Q  Quit (no save)

  C  Call/name a monster
  N  Name an item

Extended commands (#):
  #pray    - Pray to your god
  #offer   - Sacrifice on altar
  #dip     - Dip item in liquid
  #sit     - Sit down
  #chat    - Talk to someone
  #turn    - Turn undead"#
    }

    fn symbols_help() -> &'static str {
        r#"Map Symbols
────────────
Dungeon Features:
  .  Floor / ground
  #  Corridor
  -  Horizontal wall
  |  Vertical wall
  +  Closed door
  '  Open door
  <  Stairs up
  >  Stairs down
  ^  Trap
  _  Altar
  {  Fountain
  }  Water / pool
  \  Throne

Objects:
  )  Weapon
  [  Armor
  =  Ring
  "  Amulet
  (  Tool
  %  Food
  !  Potion
  ?  Scroll
  +  Spellbook
  /  Wand
  $  Gold
  *  Gem / stone
  `  Boulder / statue

Creatures:
  @  You (the player)
  a-z A-Z  Monsters
  :  Lizard-like
  ;  Sea creature
  &  Demon"#
    }

    fn get_page_content(&self) -> (&'static str, &'static str) {
        match self.page {
            0 => ("Movement (1/4)", Self::movement_help()),
            1 => ("Actions (2/4)", Self::action_help()),
            2 => ("Information (3/4)", Self::info_help()),
            3 => ("Map Symbols (4/4)", Self::symbols_help()),
            _ => ("Help", ""),
        }
    }
}

impl Default for HelpWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for HelpWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let (title, content) = self.get_page_content();

        let block = Block::default()
            .title(format!(" {} - Press SPACE for next, ESC to close ", title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        block.render(area, buf);

        let paragraph = Paragraph::new(content)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });

        paragraph.render(inner, buf);
    }
}

/// Options menu widget
pub struct OptionsWidget<'a> {
    options: Vec<OptionItem<'a>>,
    cursor: usize,
}

/// A single option item
pub struct OptionItem<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub value: OptionValue,
}

/// Option value types
#[derive(Debug, Clone)]
pub enum OptionValue {
    Bool(bool),
    Int(i32),
    String(String),
    Choice(usize, Vec<&'static str>),
}

impl OptionValue {
    pub fn display(&self) -> String {
        match self {
            OptionValue::Bool(b) => if *b { "On" } else { "Off" }.to_string(),
            OptionValue::Int(i) => i.to_string(),
            OptionValue::String(s) => s.clone(),
            OptionValue::Choice(idx, choices) => choices.get(*idx).unwrap_or(&"?").to_string(),
        }
    }

    pub fn toggle(&mut self) {
        match self {
            OptionValue::Bool(b) => *b = !*b,
            OptionValue::Choice(idx, choices) => {
                *idx = (*idx + 1) % choices.len();
            }
            _ => {}
        }
    }

    pub fn increment(&mut self) {
        match self {
            OptionValue::Int(i) => *i += 1,
            OptionValue::Choice(idx, choices) => {
                *idx = (*idx + 1) % choices.len();
            }
            _ => {}
        }
    }

    pub fn decrement(&mut self) {
        match self {
            OptionValue::Int(i) => *i -= 1,
            OptionValue::Choice(idx, choices) => {
                if *idx == 0 {
                    *idx = choices.len() - 1;
                } else {
                    *idx -= 1;
                }
            }
            _ => {}
        }
    }
}

impl<'a> OptionsWidget<'a> {
    pub fn new() -> Self {
        Self {
            options: Vec::new(),
            cursor: 0,
        }
    }

    pub fn add_option(mut self, name: &'a str, description: &'a str, value: OptionValue) -> Self {
        self.options.push(OptionItem {
            name,
            description,
            value,
        });
        self
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn move_cursor(&mut self, delta: i32) {
        let new_pos = self.cursor as i32 + delta;
        self.cursor = new_pos.clamp(0, self.options.len() as i32 - 1) as usize;
    }

    pub fn toggle_current(&mut self) {
        if let Some(opt) = self.options.get_mut(self.cursor) {
            opt.value.toggle();
        }
    }

    pub fn get_option(&self, name: &str) -> Option<&OptionValue> {
        self.options
            .iter()
            .find(|o| o.name == name)
            .map(|o| &o.value)
    }

    pub fn get_option_mut(&mut self, name: &str) -> Option<&mut OptionValue> {
        self.options
            .iter_mut()
            .find(|o| o.name == name)
            .map(|o| &mut o.value)
    }
}

impl Default for OptionsWidget<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for &OptionsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let block = Block::default()
            .title(" Options - Enter to toggle, ESC to close ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.options.is_empty() {
            let empty = Paragraph::new("No options available.")
                .style(Style::default().fg(Color::Gray));
            empty.render(inner, buf);
            return;
        }

        // Calculate column widths
        let name_width = self
            .options
            .iter()
            .map(|o| o.name.len())
            .max()
            .unwrap_or(10);

        let mut y = inner.y;
        for (i, opt) in self.options.iter().enumerate() {
            if y >= inner.y + inner.height {
                break;
            }

            let is_selected = i == self.cursor;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let cursor = if is_selected { "> " } else { "  " };
            let name = format!("{:width$}", opt.name, width = name_width);
            let value = opt.value.display();

            let line = format!("{}{} : {}", cursor, name, value);
            buf.set_string(inner.x, y, &line, style);

            // Show description on next line if selected
            if is_selected && !opt.description.is_empty() {
                y += 1;
                if y < inner.y + inner.height {
                    let desc_style = Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC);
                    buf.set_string(inner.x + 2, y, opt.description, desc_style);
                }
            }

            y += 1;
        }
    }
}

/// Create default game options
pub fn default_options<'a>() -> OptionsWidget<'a> {
    OptionsWidget::new()
        .add_option("autopickup", "Automatically pick up items", OptionValue::Bool(true))
        .add_option("safe_pet", "Prevent attacking pets", OptionValue::Bool(true))
        .add_option("safe_peaceful", "Confirm attacking peacefuls", OptionValue::Bool(true))
        .add_option("verbose", "Show detailed messages", OptionValue::Bool(true))
        .add_option("confirm", "Confirm dangerous actions", OptionValue::Bool(true))
        .add_option("number_pad", "Use number pad for movement", OptionValue::Bool(false))
        .add_option(
            "graphics",
            "Display style",
            OptionValue::Choice(0, vec!["ASCII", "Unicode", "Tiles"]),
        )
        .add_option(
            "msg_window",
            "Message window style",
            OptionValue::Choice(0, vec!["Single", "Full", "Reversed"]),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_pages() {
        let mut help = HelpWidget::new();
        assert_eq!(help.page, 0);

        help.next_page();
        assert_eq!(help.page, 1);

        help.next_page();
        assert_eq!(help.page, 2);

        help.next_page();
        assert_eq!(help.page, 3);

        help.next_page();
        assert_eq!(help.page, 0);
    }

    #[test]
    fn test_option_toggle() {
        let mut opt = OptionValue::Bool(false);
        opt.toggle();
        assert!(matches!(opt, OptionValue::Bool(true)));

        let mut choice = OptionValue::Choice(0, vec!["A", "B", "C"]);
        choice.toggle();
        assert!(matches!(choice, OptionValue::Choice(1, _)));
    }

    #[test]
    fn test_options_widget() {
        let mut opts = default_options();
        assert!(opts.get_option("autopickup").is_some());

        opts.move_cursor(1);
        assert_eq!(opts.cursor(), 1);
    }
}
