//! Inventory display widget

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use nh_assets::registry::AssetRegistry;
use nh_core::data::objects::OBJECTS;
use nh_core::object::{Object, ObjectClass};

use crate::theme::Theme;

/// Inventory display widget
pub struct InventoryWidget<'a> {
    items: &'a [Object],
    title: &'a str,
    selected: Option<usize>,
    assets: &'a AssetRegistry,
    theme: &'a Theme,
}

impl<'a> InventoryWidget<'a> {
    pub fn new(items: &'a [Object], assets: &'a AssetRegistry, theme: &'a Theme) -> Self {
        Self {
            items,
            title: "Inventory",
            selected: None,
            assets,
            theme,
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

    /// Get the base name for an object from its type definition
    fn get_object_name(obj: &Object) -> &'static str {
        let idx = obj.object_type as usize;
        if idx < OBJECTS.len() {
            let def = &OBJECTS[idx];
            // If object is identified or has no description, use the real name
            // Otherwise use the unidentified description
            if obj.known || obj.desc_known || def.description.is_empty() {
                def.name
            } else {
                def.description
            }
        } else {
            "strange object"
        }
    }

    /// Format an object for display (like NetHack's doname)
    pub fn format_item(obj: &Object, assets: &AssetRegistry) -> Line<'static> {
        let mut parts: Vec<String> = Vec::new();

        // Quantity prefix (for stackable items or coins)
        let quantity_str = if obj.quantity > 1 || obj.class == ObjectClass::Coin {
            format!("{}", obj.quantity)
        } else {
            String::new()
        };

        // BUC status (only if known)
        let buc_str = if obj.buc_known {
            match obj.buc {
                nh_core::object::BucStatus::Blessed => "blessed",
                nh_core::object::BucStatus::Cursed => "cursed",
                nh_core::object::BucStatus::Uncursed => {
                    // Only show "uncursed" for items where it matters
                    if matches!(
                        obj.class,
                        ObjectClass::Weapon
                            | ObjectClass::Armor
                            | ObjectClass::Ring
                            | ObjectClass::Amulet
                            | ObjectClass::Wand
                            | ObjectClass::Tool
                            | ObjectClass::Potion
                            | ObjectClass::Scroll
                    ) {
                        "uncursed"
                    } else {
                        ""
                    }
                }
            }
        } else {
            ""
        };

        // Enchantment (only if known and non-zero, or for weapons/armor)
        let enchant_str = if obj.known {
            match obj.class {
                ObjectClass::Weapon | ObjectClass::Armor => {
                    format!("{:+}", obj.enchantment)
                }
                ObjectClass::Ring | ObjectClass::Wand => {
                    if obj.enchantment != 0 {
                        format!("{:+}", obj.enchantment)
                    } else {
                        String::new()
                    }
                }
                _ => String::new(),
            }
        } else {
            String::new()
        };

        // Erosion status
        let erosion_str = if obj.erosion1 > 0 || obj.erosion2 > 0 {
            let e1 = match obj.erosion1 {
                1 => "rusty ",
                2 => "very rusty ",
                3 => "thoroughly rusty ",
                _ => "",
            };
            let e2 = match obj.erosion2 {
                1 => "corroded ",
                2 => "very corroded ",
                3 => "thoroughly corroded ",
                _ => "",
            };
            format!("{}{}", e1, e2)
        } else {
            String::new()
        };

        // Greased
        let greased_str = if obj.greased { "greased " } else { "" };

        // Erosion-proof (if known)
        let proof_str = if obj.rust_known && obj.erosion_proof {
            match obj.class {
                ObjectClass::Weapon => "rustproof ",
                ObjectClass::Armor => "rustproof ",
                _ => "fireproof ",
            }
        } else {
            ""
        };

        // The actual object name
        let base_name = if let Some(ref custom_name) = obj.name {
            custom_name.as_str()
        } else {
            Self::get_object_name(obj)
        };

        // Build the display string
        if !quantity_str.is_empty() {
            parts.push(quantity_str);
        }
        if !buc_str.is_empty() {
            parts.push(buc_str.to_string());
        }
        if !enchant_str.is_empty() {
            parts.push(enchant_str);
        }
        parts.push(format!(
            "{}{}{}{}",
            greased_str, proof_str, erosion_str, base_name
        ));

        // Add worn/wielded status
        let worn_str = if obj.worn_mask != 0 {
            if obj.class == ObjectClass::Weapon {
                " (weapon in hand)"
            } else if obj.class == ObjectClass::Armor {
                " (being worn)"
            } else {
                " (in use)"
            }
        } else {
            ""
        };

        // Charges for wands (if known)
        let charges_str = if obj.known && obj.class == ObjectClass::Wand {
            format!(" ({}:{})", obj.recharged, obj.enchantment)
        } else {
            String::new()
        };

        let mut spans = vec![
            Span::raw(format!("{} - ", obj.inv_letter)),
        ];

        // Add the mapped icon if available
        if let Ok(icon) = assets.get_icon(obj) {
            let color = AssetRegistry::parse_color(&icon.tui_color).unwrap_or(Color::Yellow);
            spans.push(Span::styled(format!("{} ", icon.tui_char), Style::default().fg(color)));
        }

        spans.push(Span::raw(format!(
            "{}{}{}",
            parts.join(" "),
            charges_str,
            worn_str
        )));

        Line::from(spans)
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
            .border_style(Style::default().fg(self.theme.border));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.items.is_empty() {
            let empty = Paragraph::new("Not carrying anything.")
                .style(Style::default().fg(self.theme.text_muted));
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
                Style::default()
                    .fg(self.theme.header)
                    .add_modifier(Modifier::BOLD),
            ))));

            // Add items in this class
            for item in items {
                let mut line = Self::format_item(item, self.assets);

                // Override style for BUC if known
                if item.is_cursed() && item.buc_known {
                    line = line.style(Style::default().fg(self.theme.bad));
                } else if item.is_blessed() && item.buc_known {
                    line = line.style(Style::default().fg(self.theme.accent));
                }

                list_items.push(ListItem::new(line));
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
    theme: Theme,
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
            theme: Theme::detect(),
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
            .border_style(Style::default().fg(self.theme.border));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.items.is_empty() {
            let empty =
                Paragraph::new("Nothing here.").style(Style::default().fg(self.theme.text_muted));
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
                    Style::default()
                        .fg(self.theme.cursor_fg)
                        .add_modifier(Modifier::BOLD)
                } else if item.selected {
                    Style::default().fg(self.theme.selected)
                } else {
                    Style::default().fg(self.theme.text)
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
        // Use dark theme for testing
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
