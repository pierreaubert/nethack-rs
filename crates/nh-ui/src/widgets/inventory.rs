//! Inventory display widget

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use nh_core::object::{Object, ObjectClass};

/// Inventory display widget
pub struct InventoryWidget<'a> {
    items: &'a [Object],
    title: &'a str,
    selected: Option<usize>,
}

impl<'a> InventoryWidget<'a> {
    pub fn new(items: &'a [Object]) -> Self {
        Self {
            items,
            title: "Inventory",
            selected: None,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    pub fn selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    /// Format an object for display
    fn format_item(obj: &Object) -> String {
        let quantity = if obj.quantity > 1 {
            format!("{} ", obj.quantity)
        } else {
            String::new()
        };

        let buc = if obj.buc_known {
            match obj.buc {
                nh_core::object::BucStatus::Blessed => "blessed ",
                nh_core::object::BucStatus::Cursed => "cursed ",
                nh_core::object::BucStatus::Uncursed => "",
            }
        } else {
            ""
        };

        let enchant = if obj.known && obj.enchantment != 0 {
            format!("{:+} ", obj.enchantment)
        } else {
            String::new()
        };

        let name = obj.name.as_deref().unwrap_or("item");

        format!(
            "{} - {}{}{}{}",
            obj.inv_letter, quantity, buc, enchant, name
        )
    }

    /// Get the class symbol for grouping
    fn class_header(class: ObjectClass) -> &'static str {
        match class {
            ObjectClass::Weapon => "Weapons",
            ObjectClass::Armor => "Armor",
            ObjectClass::Ring => "Rings",
            ObjectClass::Amulet => "Amulets",
            ObjectClass::Tool => "Tools",
            ObjectClass::Food => "Comestibles",
            ObjectClass::Potion => "Potions",
            ObjectClass::Scroll => "Scrolls",
            ObjectClass::Spellbook => "Spellbooks",
            ObjectClass::Wand => "Wands",
            ObjectClass::Coin => "Coins",
            ObjectClass::Gem => "Gems/Stones",
            ObjectClass::Rock => "Rocks",
            ObjectClass::Ball => "Iron balls",
            ObjectClass::Chain => "Chains",
            ObjectClass::Venom => "Venom",
            ObjectClass::Random | ObjectClass::IllObj => "Other",
        }
    }
}

impl Widget for InventoryWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area first
        Clear.render(area, buf);

        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.items.is_empty() {
            let empty = Paragraph::new("Not carrying anything.")
                .style(Style::default().fg(Color::Gray));
            empty.render(inner, buf);
            return;
        }

        // Group items by class (use Vec of tuples since ObjectClass doesn't impl Ord)
        let mut grouped: Vec<(ObjectClass, Vec<&Object>)> = Vec::new();
        for item in self.items {
            if let Some((_, items)) = grouped.iter_mut().find(|(c, _)| *c == item.class) {
                items.push(item);
            } else {
                grouped.push((item.class, vec![item]));
            }
        }

        // Build list items with headers
        let mut list_items: Vec<ListItem> = Vec::new();
        for (class, items) in grouped {
            // Add class header
            list_items.push(ListItem::new(Line::from(Span::styled(
                Self::class_header(class),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ))));

            // Add items in this class
            for item in items {
                let style = if item.is_cursed() && item.buc_known {
                    Style::default().fg(Color::Red)
                } else if item.is_blessed() && item.buc_known {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::White)
                };

                list_items.push(ListItem::new(Line::from(Span::styled(
                    format!("  {}", Self::format_item(item)),
                    style,
                ))));
            }
        }

        let list = List::new(list_items);
        Widget::render(list, inner, buf);
    }
}

/// Menu for selecting items (pickup, drop, etc.)
pub struct SelectionMenu<'a> {
    items: Vec<SelectionItem<'a>>,
    title: &'a str,
    multi_select: bool,
    cursor: usize,
}

/// An item in a selection menu
pub struct SelectionItem<'a> {
    pub letter: char,
    pub text: &'a str,
    pub selected: bool,
    pub data: usize,
}

impl<'a> SelectionMenu<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            items: Vec::new(),
            title,
            multi_select: false,
            cursor: 0,
        }
    }

    pub fn multi_select(mut self, multi: bool) -> Self {
        self.multi_select = multi;
        self
    }

    pub fn add_item(mut self, letter: char, text: &'a str, data: usize) -> Self {
        self.items.push(SelectionItem {
            letter,
            text,
            selected: false,
            data,
        });
        self
    }

    pub fn cursor(mut self, cursor: usize) -> Self {
        self.cursor = cursor.min(self.items.len().saturating_sub(1));
        self
    }

    pub fn toggle_current(&mut self) {
        if let Some(item) = self.items.get_mut(self.cursor) {
            item.selected = !item.selected;
        }
    }

    pub fn select_by_letter(&mut self, letter: char) -> bool {
        for item in &mut self.items {
            if item.letter == letter {
                item.selected = !item.selected;
                return true;
            }
        }
        false
    }

    pub fn get_selected(&self) -> Vec<usize> {
        self.items
            .iter()
            .filter(|i| i.selected)
            .map(|i| i.data)
            .collect()
    }

    pub fn move_cursor(&mut self, delta: i32) {
        let new_pos = self.cursor as i32 + delta;
        self.cursor = new_pos.clamp(0, self.items.len() as i32 - 1) as usize;
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl Widget for &SelectionMenu<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.items.is_empty() {
            let empty = Paragraph::new("Nothing here.")
                .style(Style::default().fg(Color::Gray));
            empty.render(inner, buf);
            return;
        }

        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let marker = if item.selected { "+" } else { "-" };
                let cursor_marker = if i == self.cursor { "> " } else { "  " };
                let text = format!("{}{} {} {}", cursor_marker, item.letter, marker, item.text);

                let style = if i == self.cursor {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if item.selected {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(Line::from(Span::styled(text, style)))
            })
            .collect();

        let list = List::new(list_items);
        Widget::render(list, inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_menu() {
        let mut menu = SelectionMenu::new("Test Menu")
            .add_item('a', "Item 1", 0)
            .add_item('b', "Item 2", 1)
            .add_item('c', "Item 3", 2);

        assert_eq!(menu.len(), 3);
        assert!(menu.get_selected().is_empty());

        menu.select_by_letter('b');
        assert_eq!(menu.get_selected(), vec![1]);

        menu.select_by_letter('a');
        assert_eq!(menu.get_selected(), vec![0, 1]);
    }

    #[test]
    fn test_cursor_movement() {
        let mut menu = SelectionMenu::new("Test")
            .add_item('a', "Item 1", 0)
            .add_item('b', "Item 2", 1)
            .add_item('c', "Item 3", 2);

        assert_eq!(menu.cursor, 0);

        menu.move_cursor(1);
        assert_eq!(menu.cursor, 1);

        menu.move_cursor(10);
        assert_eq!(menu.cursor, 2);

        menu.move_cursor(-10);
        assert_eq!(menu.cursor, 0);
    }
}
