//! Display/glyph system

use ratatui::style::Color;

use crate::theme::Theme;

/// Convert a NetHack color index to ratatui Color (dark background assumed).
/// For theme-aware color mapping, use `Theme::game_color()` instead.
pub fn nh_color(color: u8) -> Color {
    Theme::dark().game_color(color)
}
