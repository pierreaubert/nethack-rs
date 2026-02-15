//! Display/glyph system

use ratatui::style::Color;

/// Convert a NetHack color index to ratatui Color
pub fn nh_color(color: u8) -> Color {
    match color {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow, // Brown
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::White,
        8 => Color::DarkGray, // Bright black
        9 => Color::LightRed,
        10 => Color::LightGreen,
        11 => Color::LightYellow,
        12 => Color::LightBlue,
        13 => Color::LightMagenta,
        14 => Color::LightCyan,
        15 => Color::White, // Bright white
        _ => Color::White,
    }
}
