//! Message display widget

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

/// Widget for rendering messages
pub struct MessagesWidget<'a> {
    messages: &'a [String],
}

impl<'a> MessagesWidget<'a> {
    pub fn new(messages: &'a [String]) -> Self {
        Self { messages }
    }
}

impl Widget for MessagesWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text = if self.messages.is_empty() {
            String::new()
        } else {
            self.messages.join("  ")
        };

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::TOP))
            .wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }
}
