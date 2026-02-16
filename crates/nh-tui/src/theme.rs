//! Terminal color theme system
//!
//! Provides adaptive color palettes for dark and light terminal backgrounds.
//! Auto-detects via COLORFGBG env var, or manual override with --light flag
//! or NH_LIGHT_BG=1 environment variable.

use ratatui::style::Color;

/// Color theme for terminal UI.
/// All UI code should use theme colors instead of hardcoded Color:: values.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    // General UI text
    /// Primary foreground text
    pub text: Color,
    /// Secondary/hint text (footers, instructions)
    pub text_dim: Color,
    /// Muted text (empty states, placeholder)
    pub text_muted: Color,

    // Borders
    /// Default border color
    pub border: Color,
    /// Informational border (help, character creation)
    pub border_accent: Color,
    /// Action border (item select, direction select, options)
    pub border_action: Color,
    /// Danger border (death screen)
    pub border_danger: Color,

    // Interactive elements
    /// Selected/cursor item foreground
    pub cursor_fg: Color,
    /// Selected/cursor item background
    pub cursor_bg: Color,
    /// Multi-select chosen items
    pub selected: Color,

    // Semantic colors
    /// Section headers, accent text
    pub accent: Color,
    /// Group headers (inventory class names)
    pub header: Color,
    /// Positive/good (blessed, maintained conducts)
    pub good: Color,
    /// Negative/bad (cursed, hostile, death)
    pub bad: Color,

    // Map terrain
    pub map_player: Color,
    pub map_pet: Color,
    pub map_hostile: Color,
    pub map_peaceful: Color,
    pub map_stone: Color,
    pub map_wall: Color,
    pub map_floor_lit: Color,
    pub map_floor_dark: Color,
    pub map_explored: Color,
    pub map_door: Color,
    pub map_stairs: Color,
    pub map_water: Color,
    pub map_lava: Color,
    pub map_fountain: Color,
    pub map_altar: Color,
    pub map_throne: Color,
    pub map_tree: Color,
    pub map_default: Color,

    // Map object fallback colors (for objects without asset registry icons)
    pub obj_coin: Color,
    pub obj_gem: Color,
    pub obj_potion: Color,
    pub obj_scroll: Color,
    pub obj_wand: Color,
    pub obj_weapon: Color,
    pub obj_food: Color,
    pub obj_default: Color,
}

impl Theme {
    /// Dark terminal background theme (default)
    pub fn dark() -> Self {
        Self {
            text: Color::White,
            text_dim: Color::DarkGray,
            text_muted: Color::Gray,
            border: Color::White,
            border_accent: Color::Cyan,
            border_action: Color::Yellow,
            border_danger: Color::Red,
            cursor_fg: Color::Yellow,
            cursor_bg: Color::DarkGray,
            selected: Color::Green,
            accent: Color::Cyan,
            header: Color::Yellow,
            good: Color::Green,
            bad: Color::Red,
            map_player: Color::White,
            map_pet: Color::White,
            map_hostile: Color::Red,
            map_peaceful: Color::Yellow,
            map_stone: Color::Black,
            map_wall: Color::Gray,
            map_floor_lit: Color::White,
            map_floor_dark: Color::DarkGray,
            map_explored: Color::DarkGray,
            map_door: Color::Yellow,
            map_stairs: Color::White,
            map_water: Color::Blue,
            map_lava: Color::Red,
            map_fountain: Color::Cyan,
            map_altar: Color::Gray,
            map_throne: Color::Yellow,
            map_tree: Color::Green,
            map_default: Color::White,
            obj_coin: Color::Yellow,
            obj_gem: Color::Cyan,
            obj_potion: Color::Magenta,
            obj_scroll: Color::White,
            obj_wand: Color::LightBlue,
            obj_weapon: Color::Gray,
            obj_food: Color::LightRed,
            obj_default: Color::Yellow,
        }
    }

    /// Light terminal background theme
    pub fn light() -> Self {
        Self {
            text: Color::Black,
            text_dim: Color::DarkGray,
            text_muted: Color::DarkGray,
            border: Color::DarkGray,
            border_accent: Color::Blue,
            border_action: Color::Yellow,
            border_danger: Color::Red,
            cursor_fg: Color::Yellow,
            cursor_bg: Color::DarkGray,
            selected: Color::Green,
            accent: Color::Blue,
            header: Color::Yellow,
            good: Color::Green,
            bad: Color::Red,
            map_player: Color::Black,
            map_pet: Color::Green,
            map_hostile: Color::Red,
            map_peaceful: Color::Yellow,
            map_stone: Color::White,
            map_wall: Color::DarkGray,
            map_floor_lit: Color::Black,
            map_floor_dark: Color::Gray,
            map_explored: Color::Gray,
            map_door: Color::Yellow,
            map_stairs: Color::Black,
            map_water: Color::Blue,
            map_lava: Color::Red,
            map_fountain: Color::Blue,
            map_altar: Color::DarkGray,
            map_throne: Color::Yellow,
            map_tree: Color::Green,
            map_default: Color::Black,
            obj_coin: Color::Yellow,
            obj_gem: Color::Blue,
            obj_potion: Color::Magenta,
            obj_scroll: Color::DarkGray,
            obj_wand: Color::Blue,
            obj_weapon: Color::DarkGray,
            obj_food: Color::Red,
            obj_default: Color::Yellow,
        }
    }

    /// Auto-detect terminal background and return appropriate theme.
    /// Checks COLORFGBG env var and NH_LIGHT_BG override.
    pub fn detect() -> Self {
        if Self::is_light_background() {
            Self::light()
        } else {
            Self::dark()
        }
    }

    /// Map a NetHack color index (0-15) to an appropriate terminal color.
    /// Adjusts problematic colors (white-on-white, black-on-black) for the
    /// current theme.
    pub fn game_color(&self, index: u8) -> Color {
        // Check if we're on a light background by comparing map_player
        // (Black on light, White on dark)
        let is_light = self.map_player == Color::Black;

        if is_light {
            match index {
                0 => Color::DarkGray,      // Black → visible dark
                7 => Color::Black,         // White → Black
                15 => Color::Black,        // Bright White → Black
                1 => Color::Red,
                2 => Color::Green,
                3 => Color::Yellow,
                4 => Color::Blue,
                5 => Color::Magenta,
                6 => Color::Blue,          // Cyan → Blue (more contrast)
                8 => Color::DarkGray,
                9 => Color::Red,           // LightRed → Red
                10 => Color::Green,        // LightGreen → Green
                11 => Color::Yellow,
                12 => Color::Blue,         // LightBlue → Blue
                13 => Color::Magenta,
                14 => Color::Blue,         // LightCyan → Blue
                _ => Color::Black,
            }
        } else {
            match index {
                0 => Color::Black,
                1 => Color::Red,
                2 => Color::Green,
                3 => Color::Yellow,
                4 => Color::Blue,
                5 => Color::Magenta,
                6 => Color::Cyan,
                7 => Color::White,
                8 => Color::DarkGray,
                9 => Color::LightRed,
                10 => Color::LightGreen,
                11 => Color::LightYellow,
                12 => Color::LightBlue,
                13 => Color::LightMagenta,
                14 => Color::LightCyan,
                15 => Color::White,
                _ => Color::White,
            }
        }
    }

    fn is_light_background() -> bool {
        // Explicit override via environment variable
        if let Ok(val) = std::env::var("NH_LIGHT_BG") {
            return val == "1" || val.eq_ignore_ascii_case("true");
        }

        // COLORFGBG is set by many terminals (xterm, rxvt, iTerm2, etc.)
        // Format: "fg;bg" where values are color indices (0-15)
        // Light backgrounds typically have bg index >= 7 (excluding 8 which is bright black)
        if let Ok(colorfgbg) = std::env::var("COLORFGBG")
            && let Some(bg_str) = colorfgbg.rsplit(';').next()
            && let Ok(bg_idx) = bg_str.parse::<u8>()
        {
            return matches!(bg_idx, 7 | 9..=15);
        }

        false
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::detect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_theme_text_is_white() {
        let theme = Theme::dark();
        assert_eq!(theme.text, Color::White);
        assert_eq!(theme.map_player, Color::White);
    }

    #[test]
    fn test_light_theme_text_is_black() {
        let theme = Theme::light();
        assert_eq!(theme.text, Color::Black);
        assert_eq!(theme.map_player, Color::Black);
    }

    #[test]
    fn test_game_color_dark_theme() {
        let theme = Theme::dark();
        assert_eq!(theme.game_color(0), Color::Black);
        assert_eq!(theme.game_color(7), Color::White);
        assert_eq!(theme.game_color(15), Color::White);
    }

    #[test]
    fn test_game_color_light_theme() {
        let theme = Theme::light();
        // White should become Black on light bg
        assert_eq!(theme.game_color(7), Color::Black);
        assert_eq!(theme.game_color(15), Color::Black);
        // Black should become DarkGray (visible) on light bg
        assert_eq!(theme.game_color(0), Color::DarkGray);
    }

    #[test]
    fn test_saturated_colors_same_both_themes() {
        let dark = Theme::dark();
        let light = Theme::light();
        // Red, Green, Yellow, Blue, Magenta should be identical
        assert_eq!(dark.game_color(1), light.game_color(1)); // Red
        assert_eq!(dark.game_color(2), light.game_color(2)); // Green
        assert_eq!(dark.game_color(3), light.game_color(3)); // Yellow
        assert_eq!(dark.game_color(5), light.game_color(5)); // Magenta
    }
}
